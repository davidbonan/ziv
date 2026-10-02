use std::fmt;
use std::ops::ControlFlow;
use std::path::PathBuf;

use crate::enhance::domain::enhancement::{Enhancement, difference_level};
use crate::enhance::domain::enhancement_model::{
    ENHANCEMENT_MODEL, INPUT_CHANNELS, MODEL_INPUT, MOST_NOISE,
};
use crate::enhance::domain::model_encoding::ModelEncoding;
use crate::enhance::domain::noise_level::noise_level;
use crate::enhance::domain::overlapping_tiles::{
    TILE_SIDE, TilePlace, mirrored, tile_count_along, tile_origin,
};
use crate::enhance::domain::unsharp_mask::sharpened;
use crate::models::application::model_store::{ModelError, ModelStore};
use crate::models::domain::model::Model;
use crate::models::domain::model_runner::{ModelInput, ModelRunner};
use crate::models::domain::model_source::ModelSource;
use crate::photo::domain::working_image::WorkingImage;

const CHANNELS: usize = 3;
const GREEN: usize = 1;
const PLANE_LENGTH: usize = TILE_SIDE * TILE_SIDE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnhancementError {
    Cancelled,
    Photo(String),
    Model(ModelError),
    Inference(String),
    Storage(String),
}

impl fmt::Display for EnhancementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(formatter, "it was cancelled"),
            Self::Model(error) => {
                write!(
                    formatter,
                    "the {} model is missing: {error}",
                    ENHANCEMENT_MODEL.name
                )
            }
            Self::Photo(reason) => write!(formatter, "the photo could not be read ({reason})"),
            Self::Inference(reason) => write!(formatter, "the model failed ({reason})"),
            Self::Storage(reason) => {
                write!(formatter, "its enhancement could not be written ({reason})")
            }
        }
    }
}

/// What an enhancement is busy with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnhancementStep {
    Downloading {
        model: &'static Model,
        received: u64,
    },
    /// `share` of the photo is done, from 0 to 1.
    Enhancing { share: f32 },
}

/// The photo as the model reads it, tile by tile.
struct SeenPhoto<'a> {
    image: &'a WorkingImage,
    encoding: ModelEncoding,
    /// The deviation of the photo's noise, as the model is told.
    noise: f32,
}

impl<'a> SeenPhoto<'a> {
    fn size(&self) -> [usize; 2] {
        [self.image.width(), self.image.height()].map(|side| side as usize)
    }

    fn seen(&self, [x, y]: [usize; 2], channel: usize) -> f32 {
        let working = self.image.pixel(x as u32, y as u32)[channel];
        self.encoding.encoded(working).clamp(0.0, 1.0)
    }

    fn of(image: &'a WorkingImage) -> Self {
        let mut photo = Self {
            image,
            encoding: ModelEncoding::of(image),
            noise: 0.0,
        };
        let [width, height] = photo.size();
        let places = (0..height).flat_map(|y| (0..width).map(move |x| [x, y]));
        let greens: Vec<f32> = places.map(|place| photo.seen(place, GREEN)).collect();
        photo.noise = noise_level(&greens, width).min(MOST_NOISE);
        photo
    }

    /// What the model takes for the tile at `origin`: its three planes, the
    /// photo mirrored beyond its border, then the noise to remove.
    fn tile(&self, [left, top]: [usize; 2]) -> Vec<f32> {
        let [width, height] = self.size();
        let columns: Vec<usize> = (left..left + TILE_SIDE)
            .map(|x| mirrored(x, width))
            .collect();
        let mut planes = Vec::with_capacity(INPUT_CHANNELS * PLANE_LENGTH);
        for channel in 0..CHANNELS {
            for y in (top..top + TILE_SIDE).map(|y| mirrored(y, height)) {
                planes.extend(columns.iter().map(|x| self.seen([*x, y], channel)));
            }
        }
        planes.resize(INPUT_CHANNELS * PLANE_LENGTH, self.noise);
        planes
    }
}

/// What the model and the sharpening changed in each tile of one row of tiles.
#[derive(Default)]
struct TileRow {
    differences: Vec<Vec<f32>>,
}

impl TileRow {
    fn difference_at(&self, column: &TilePlace, [channel, row]: [usize; 2]) -> f32 {
        column.blended(|tile, offset| {
            self.differences[tile][channel * PLANE_LENGTH + row * TILE_SIDE + offset]
        })
    }
}

/// A row of tiles and the one above it, which it overlaps.
struct StackedTileRows<'a> {
    above: &'a TileRow,
    current: &'a TileRow,
    current_row: usize,
}

impl StackedTileRows<'_> {
    fn difference_at(&self, line: &TilePlace, column: &TilePlace, channel: usize) -> f32 {
        line.blended(|row, offset| {
            let tiles = if row == self.current_row {
                self.current
            } else {
                self.above
            };
            tiles.difference_at(column, [channel, offset])
        })
    }

    fn line_levels(&self, line: TilePlace, columns: &[TilePlace]) -> impl Iterator<Item = u8> {
        let channels = columns
            .iter()
            .flat_map(|column| (0..CHANNELS).map(move |channel| (column, channel)));
        channels.map(move |(column, channel)| {
            difference_level(self.difference_at(&line, column, channel))
        })
    }
}

/// Follows an enhancement, and stops it by answering `Break`.
pub type OnStep<'a> = &'a mut dyn FnMut(EnhancementStep) -> ControlFlow<()>;

fn continuing(answer: ControlFlow<()>) -> Result<(), EnhancementError> {
    match answer {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(()) => Err(EnhancementError::Cancelled),
    }
}

/// Removes the noise of a photo and strengthens its detail, with a model
/// fetched when first needed.
pub struct Enhancer<Source, Runner> {
    store: ModelStore<Source>,
    runner: Runner,
}

/// One photo being enhanced.
struct PhotoEnhancing<'a, Runner> {
    runner: &'a Runner,
    model_file: PathBuf,
    photo: SeenPhoto<'a>,
    tile_counts: [usize; 2],
}

impl<Runner: ModelRunner> PhotoEnhancing<'_, Runner> {
    fn tile_differences(&self, seen: Vec<f32>) -> Result<Vec<f32>, EnhancementError> {
        let input = ModelInput {
            name: MODEL_INPUT,
            shape: [1, INPUT_CHANNELS, TILE_SIDE, TILE_SIDE],
            values: seen.clone(),
        };
        let outputs = self
            .runner
            .outputs(&self.model_file, input)
            .map_err(EnhancementError::Inference)?;
        let cleaned = outputs
            .first()
            .filter(|output| output.values.len() == CHANNELS * PLANE_LENGTH)
            .ok_or_else(|| EnhancementError::Inference("unexpected output".to_owned()))?;
        let sharpened = cleaned
            .values
            .as_chunks::<PLANE_LENGTH>()
            .0
            .iter()
            .flat_map(|plane| sharpened(plane, TILE_SIDE));
        let differences = sharpened.zip(seen).map(|(enhanced, seen)| enhanced - seen);
        Ok(differences.collect())
    }

    fn tile_row(&self, row: usize, on_step: OnStep<'_>) -> Result<TileRow, EnhancementError> {
        let [columns, rows] = self.tile_counts;
        let mut differences = Vec::with_capacity(columns);
        for column in 0..columns {
            let seen = self.photo.tile([column, row].map(tile_origin));
            differences.push(self.tile_differences(seen)?);
            let share = (row * columns + column + 1) as f32 / (rows * columns) as f32;
            continuing(on_step(EnhancementStep::Enhancing { share }))?;
        }
        Ok(TileRow { differences })
    }

    /// The lines of the photo that are whole once `row` is computed.
    fn lines_settled_by(&self, row: usize) -> std::ops::Range<usize> {
        let is_last = row + 1 == self.tile_counts[1];
        let end = if is_last {
            self.photo.image.height() as usize
        } else {
            tile_origin(row + 1)
        };
        tile_origin(row)..end
    }

    fn difference_levels(&self, on_step: OnStep<'_>) -> Result<Vec<u8>, EnhancementError> {
        let [width, height] = self.photo.size();
        let [columns, rows] = self.tile_counts;
        let column_places: Vec<TilePlace> = (0..width).map(|x| TilePlace::at(x, columns)).collect();
        let mut levels = Vec::with_capacity(width * height * CHANNELS);
        let mut above = TileRow::default();
        for row in 0..rows {
            let current = self.tile_row(row, on_step)?;
            let stacked = StackedTileRows {
                above: &above,
                current: &current,
                current_row: row,
            };
            for y in self.lines_settled_by(row) {
                levels.extend(stacked.line_levels(TilePlace::at(y, rows), &column_places));
            }
            above = current;
        }
        Ok(levels)
    }
}

impl<Source: ModelSource, Runner: ModelRunner> Enhancer<Source, Runner> {
    pub fn new(store: ModelStore<Source>, runner: Runner) -> Self {
        Self { store, runner }
    }

    fn model_file(&self, on_step: OnStep<'_>) -> Result<PathBuf, EnhancementError> {
        let mut answer = ControlFlow::Continue(());
        let mut on_received = |received| {
            answer = on_step(EnhancementStep::Downloading {
                model: &ENHANCEMENT_MODEL,
                received,
            });
        };
        let file = self
            .store
            .model_file(&ENHANCEMENT_MODEL, &mut on_received)
            .map_err(EnhancementError::Model)?;
        continuing(answer).map(|()| file)
    }

    /// What enhancing changes in `image`. `on_step` follows the work and
    /// stops it by answering `Break`.
    pub fn enhancement(
        &self,
        image: &WorkingImage,
        on_step: OnStep<'_>,
    ) -> Result<Enhancement, EnhancementError> {
        let model_file = self.model_file(on_step)?;
        continuing(on_step(EnhancementStep::Enhancing { share: 0.0 }))?;
        let photo = SeenPhoto::of(image);
        let encoding = photo.encoding;
        let enhancing = PhotoEnhancing {
            runner: &self.runner,
            model_file,
            tile_counts: photo.size().map(tile_count_along),
            photo,
        };
        let levels = enhancing.difference_levels(on_step)?;
        let enhancement = Enhancement::new([image.width(), image.height()], encoding, levels);
        Ok(enhancement.expect("one level per channel of each pixel"))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    use super::*;
    use crate::models::domain::model::RemoteFile;
    use crate::models::domain::model_runner::ModelOutput;

    struct NoNetwork;

    impl ModelSource for NoNetwork {
        fn download(&self, _: &RemoteFile) -> Result<Box<dyn Read>, String> {
            Err("no network".to_owned())
        }
    }

    fn answering(mut values: Vec<f32>) -> Result<Vec<ModelOutput>, String> {
        values.truncate(CHANNELS * PLANE_LENGTH);
        Ok(vec![ModelOutput {
            shape: vec![1, CHANNELS, TILE_SIDE, TILE_SIDE],
            values,
        }])
    }

    /// Finds nothing to clean.
    struct ChangingNothing;

    impl ModelRunner for ChangingNothing {
        fn outputs(&self, _: &Path, input: ModelInput) -> Result<Vec<ModelOutput>, String> {
            answering(input.values)
        }
    }

    /// Answers each tile a flat grey, lighter from one tile to the next.
    #[derive(Default)]
    struct LighterEachTile {
        tiles_answered: Cell<usize>,
    }

    impl ModelRunner for LighterEachTile {
        fn outputs(&self, _: &Path, input: ModelInput) -> Result<Vec<ModelOutput>, String> {
            let grey = 0.2 + 0.1 * self.tiles_answered.get() as f32;
            self.tiles_answered.set(self.tiles_answered.get() + 1);
            answering(vec![grey; input.values.len()])
        }
    }

    struct Failing;

    impl ModelRunner for Failing {
        fn outputs(&self, _: &Path, _: ModelInput) -> Result<Vec<ModelOutput>, String> {
            Err("out of memory".to_owned())
        }
    }

    fn model_folder() -> tempfile::TempDir {
        let folder = tempfile::tempdir().unwrap();
        fs::write(
            folder.path().join(ENHANCEMENT_MODEL.file_name()),
            b"weights",
        )
        .unwrap();
        folder
    }

    fn enhancer_running<Runner: ModelRunner>(
        models: &tempfile::TempDir,
        runner: Runner,
    ) -> Enhancer<NoNetwork, Runner> {
        Enhancer::new(ModelStore::new(models.path().to_owned(), NoNetwork), runner)
    }

    const WIDTH: u32 = 700;
    const HEIGHT: u32 = 530;

    /// Wider and taller than one tile; smooth, and above white at its far corner.
    fn smooth_photo() -> WorkingImage {
        let pixels = (0..WIDTH * HEIGHT).map(|index| {
            let [x, y] = [index % WIDTH, index / WIDTH];
            let along = (x + y) as f32 / (WIDTH + HEIGHT) as f32;
            [along * 3.0, along, along * 0.5]
        });
        WorkingImage::new(WIDTH, HEIGHT, pixels.collect())
    }

    fn black_photo() -> WorkingImage {
        WorkingImage::new(WIDTH, HEIGHT, vec![[0.0; 3]; (WIDTH * HEIGHT) as usize])
    }

    #[test]
    fn photo_the_model_leaves_alone_comes_back_whole_up_to_its_border() {
        let models = model_folder();
        let enhancer = enhancer_running(&models, ChangingNothing);
        let photo = smooth_photo();

        let enhancement = enhancer
            .enhancement(&photo, &mut |_| ControlFlow::Continue(()))
            .unwrap();

        let enhanced = enhancement.enhanced(&photo).unwrap();
        let channels = enhanced.pixels().iter().flatten();
        let furthest = channels
            .zip(photo.pixels().iter().flatten())
            .map(|(enhanced, original)| (enhanced - original).abs())
            .fold(0.0, f32::max);
        assert!(furthest < 0.02, "{furthest}");
        assert!(enhanced.pixel(WIDTH - 1, HEIGHT - 1)[0] > 2.9);
    }

    #[test]
    fn tiles_meet_without_a_seam() {
        let models = model_folder();
        let enhancer = enhancer_running(&models, LighterEachTile::default());

        let enhancement = enhancer
            .enhancement(&black_photo(), &mut |_| ControlFlow::Continue(()))
            .unwrap();

        let level_at = |x: u32, y: u32| {
            i32::from(enhancement.difference_levels()[((y * WIDTH + x) * 3) as usize])
        };
        let corners = [
            [0, 0],
            [WIDTH - 1, 0],
            [0, HEIGHT - 1],
            [WIDTH - 1, HEIGHT - 1],
        ];
        let greys = corners.map(|[x, y]| level_at(x, y));
        assert_eq!(
            greys,
            [0.2, 0.3, 0.4, 0.5].map(|grey| i32::from(difference_level(grey)))
        );
        let positions = (1..WIDTH).flat_map(|x| (1..HEIGHT).map(move |y| [x, y]));
        let largest_step = positions
            .map(|[x, y]| {
                let here = level_at(x, y);
                (here - level_at(x - 1, y))
                    .abs()
                    .max((here - level_at(x, y - 1)).abs())
            })
            .max();
        assert!(largest_step <= Some(4), "{largest_step:?}");
    }

    #[test]
    fn progress_goes_tile_by_tile_up_to_the_whole_photo() {
        let models = model_folder();
        let enhancer = enhancer_running(&models, ChangingNothing);
        let mut steps = Vec::new();

        let enhanced = enhancer.enhancement(&black_photo(), &mut |step| {
            steps.push(step);
            ControlFlow::Continue(())
        });

        assert!(enhanced.is_ok());
        let shares = [0.0, 0.25, 0.5, 0.75, 1.0];
        assert_eq!(
            steps,
            shares.map(|share| EnhancementStep::Enhancing { share })
        );
    }

    #[test]
    fn enhancement_stops_at_the_tile_it_is_cancelled_on() {
        let models = model_folder();
        let enhancer = enhancer_running(&models, ChangingNothing);
        let mut shares = Vec::new();

        let enhanced = enhancer.enhancement(&black_photo(), &mut |step| {
            let EnhancementStep::Enhancing { share } = step else {
                return ControlFlow::Continue(());
            };
            shares.push(share);
            if share > 0.0 {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });

        assert_eq!(enhanced, Err(EnhancementError::Cancelled));
        assert_eq!(shares, [0.0, 0.25]);
    }

    #[test]
    fn failures_of_the_model_and_of_its_download_are_reported() {
        let models = model_folder();
        let nothing_downloaded = tempfile::tempdir().unwrap();
        let mut on_step = |_| ControlFlow::Continue(());

        let failed = enhancer_running(&models, Failing).enhancement(&black_photo(), &mut on_step);
        let missing = enhancer_running(&nothing_downloaded, ChangingNothing)
            .enhancement(&black_photo(), &mut on_step);

        assert_eq!(
            failed,
            Err(EnhancementError::Inference("out of memory".to_owned()))
        );
        let no_network = ModelError::Download("no network".to_owned());
        assert_eq!(missing, Err(EnhancementError::Model(no_network)));
    }
}
