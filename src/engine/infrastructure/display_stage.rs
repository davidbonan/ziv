use std::ops::Range;
use std::sync::Mutex;

use crate::color::domain::matrix3::RgbMatrix;
use crate::color::domain::oklab::oklab;
use crate::color::domain::srgb_transfer;
use crate::color::domain::working_space::{
    DISPLAY_LUMA_WEIGHTS, DisplayTransform, luminance_weights,
};
use crate::develop::domain::color_grading::ColorGrading;
use crate::develop::domain::color_mixer::{CHROMA_FADE, ColorMixer, RANGE_COUNT};
use crate::develop::domain::development::{AdjustmentFactors, Development};
use crate::develop::domain::edit::{Edit, MOST_MASKS};
use crate::develop::domain::feathered_edge::FEATHER_REACH;
use crate::develop::domain::linear_gradient::{LinearGradient, SHORTEST_GRADIENT_SQUARED};
use crate::develop::domain::mask::{CoverageSource, Mask, MaskShape, photo_extent};
use crate::develop::domain::overlay::{OVERLAY_COLOUR, OVERLAY_OPACITY};
use crate::develop::domain::polygon::{MOST_CORNERS, Polygon};
use crate::develop::domain::radial_gradient::{FEATHER_RANGE, SMALLEST_EXTENT};
use crate::develop::domain::tone::{
    BLACKS_REACH, DARKEST_LUMINANCE, HIGHLIGHTS_WIDTH_STOPS, SHADOWS_WIDTH_STOPS,
};

use super::coverage_layers::no_coverage_layer;
use super::curve_lookup::CurveLookup;
use super::source_texture::SourceTexture;

const OKLAB_ROWS: usize = 12;
const COLOR_MIXER_ROWS: usize = 1 + RANGE_COUNT + OKLAB_ROWS;
const COLOR_GRADING_ROWS: usize = 6;
const STAGE_ROWS: usize = 13 + COLOR_MIXER_ROWS + COLOR_GRADING_ROWS;
const ADJUSTMENT_ROWS: usize = 5;
const CORNER_ROWS: usize = MOST_CORNERS / 2;
const MASK_ROWS: usize = ADJUSTMENT_ROWS + 3 + CORNER_ROWS;
// The kinds of shape, as `display_stage.wgsl` reads them.
const LINEAR_GRADIENT_KIND: f32 = 0.0;
const RADIAL_GRADIENT_KIND: f32 = 1.0;
const RECTANGLE_KIND: f32 = 2.0;
const POLYGON_KIND: f32 = 3.0;
const COVERAGE_LAYER_KIND: f32 = 4.0;
const UNIFORM_ROWS: usize = STAGE_ROWS + ADJUSTMENT_ROWS + (MOST_MASKS + 1) * MASK_ROWS;
pub const DISPLAY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sampling {
    /// Filtered, for a photo shown at or below 100 %.
    Smooth,
    /// Nearest texel, so magnified photo pixels stay sharp squares.
    Pixelated,
}

/// One render: the source region developed, then encoded for display.
/// The region is in texture coordinates: `[0, 0]` is the top-left of the source,
/// `[1, 1]` its bottom-right.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayRequest {
    pub region_min: [f32; 2],
    pub region_size: [f32; 2],
    pub size: [u32; 2],
    pub sampling: Sampling,
    pub development: Development,
    /// The mask whose coverage is shown as a veil over the photo.
    pub overlaid_mask: Option<Mask>,
}

impl DisplayRequest {
    /// The rows `rows` of a render of the whole source at `size`.
    pub fn strip_of_whole_source(size: [u32; 2], rows: Range<u32>) -> Self {
        let [width, height] = size;
        Self {
            region_min: [0.0, rows.start as f32 / height as f32],
            region_size: [1.0, rows.len() as f32 / height as f32],
            size: [width, rows.len() as u32],
            ..Self::whole_source(size)
        }
    }

    pub fn whole_source(size: [u32; 2]) -> Self {
        Self {
            region_min: [0.0; 2],
            region_size: [1.0; 2],
            size,
            sampling: Sampling::Smooth,
            development: Development::default(),
            overlaid_mask: None,
        }
    }
}

fn adjustment_rows(factors: &AdjustmentFactors) -> [[f32; 4]; ADJUSTMENT_ROWS] {
    let [balance_red, balance_green, balance_blue] =
        factors.white_balance.map(|[r, g, b]| [r, g, b, 0.0]);
    let tone = &factors.tone;
    [
        balance_red,
        balance_green,
        balance_blue,
        [
            factors.exposure_gain,
            tone.contrast_slope,
            tone.highlights_shift,
            tone.shadows_shift,
        ],
        [
            tone.whites_stretch,
            tone.blacks_offset,
            factors.presence.vibrance,
            factors.presence.saturation,
        ],
    ]
}

struct ShapeRows {
    kind: f32,
    corner_count: usize,
    coverage_layer: usize,
    geometry: [f32; 4],
    orientation: [f32; 4],
    corners: [[f32; 4]; CORNER_ROWS],
}

impl ShapeRows {
    fn of_kind(kind: f32, geometry: [f32; 4]) -> Self {
        Self {
            kind,
            corner_count: 0,
            coverage_layer: 0,
            geometry,
            orientation: [0.0; 4],
            corners: [[0.0; 4]; CORNER_ROWS],
        }
    }
}

fn polygon_rows(polygon: &Polygon) -> ShapeRows {
    let mut corners = [[0.0; 4]; CORNER_ROWS];
    for (row, pair) in corners.iter_mut().zip(polygon.corners.chunks(2)) {
        let [x, y] = pair[0];
        let [next_x, next_y] = pair.get(1).copied().unwrap_or_default();
        *row = [x, y, next_x, next_y];
    }
    ShapeRows {
        corner_count: polygon.corners.len().min(MOST_CORNERS),
        corners,
        ..ShapeRows::of_kind(POLYGON_KIND, [0.0; 4])
    }
}

fn shape_rows(edit: &Edit, shape: &MaskShape) -> ShapeRows {
    match shape {
        MaskShape::LinearGradient(LinearGradient { full, none }) => {
            ShapeRows::of_kind(LINEAR_GRADIENT_KIND, [full[0], full[1], none[0], none[1]])
        }
        MaskShape::RadialGradient(gradient) => {
            let (sine, cosine) = gradient.rotation.sin_cos();
            let [centre_x, centre_y] = gradient.centre;
            let [across, down] = gradient.radii;
            ShapeRows {
                orientation: [cosine, sine, 0.0, 0.0],
                ..ShapeRows::of_kind(RADIAL_GRADIENT_KIND, [centre_x, centre_y, across, down])
            }
        }
        MaskShape::Rectangle(rectangle) => {
            let [centre_x, centre_y] = rectangle.centre;
            let [half_width, half_height] = rectangle.half_size.map(f32::abs);
            ShapeRows::of_kind(
                RECTANGLE_KIND,
                [centre_x, centre_y, half_width, half_height],
            )
        }
        MaskShape::Polygon(polygon) => polygon_rows(polygon),
        MaskShape::Brush(_) | MaskShape::Zone(_) => ShapeRows {
            coverage_layer: edit.coverage_layer_of(shape).unwrap_or(0),
            ..ShapeRows::of_kind(COVERAGE_LAYER_KIND, [0.0; 4])
        },
    }
}

fn mask_rows(development: &Development, mask: &Mask) -> Vec<[f32; 4]> {
    let inverted = if mask.is_inverted { 1.0 } else { 0.0 };
    let feather = mask.shape.feather().unwrap_or(0.0) / FEATHER_RANGE.end();
    let shape = shape_rows(&development.edit, &mask.shape);
    let corners_or_layer = shape.corner_count.max(shape.coverage_layer) as f32;
    let coverage = [inverted, shape.kind, feather, corners_or_layer];
    let mut rows = Vec::with_capacity(MASK_ROWS);
    rows.extend(adjustment_rows(&development.mask_factors(mask)));
    rows.extend([coverage, shape.geometry, shape.orientation]);
    rows.extend(shape.corners);
    rows
}

fn color_mixer_rows(mixer: &ColorMixer) -> [[f32; 4]; COLOR_MIXER_ROWS] {
    let is_applied = match mixer.is_default() {
        true => 0.0,
        false => 1.0,
    };
    let ranges = mixer.range_factors().map(|range| {
        [
            range.centre,
            range.hue_turn,
            range.chroma_scale,
            range.lightness_scale,
        ]
    });
    let oklab = oklab();
    let matrices: [&RgbMatrix; 4] = [
        &oklab.working_to_lms,
        &oklab.lms_to_lab,
        &oklab.lab_to_lms,
        &oklab.lms_to_working,
    ];
    let mut rows = [[0.0; 4]; COLOR_MIXER_ROWS];
    rows[0] = [CHROMA_FADE, is_applied, 0.0, 0.0];
    rows[1..=RANGE_COUNT].copy_from_slice(&ranges);
    let matrix_rows = matrices
        .into_iter()
        .flatten()
        .map(|[first, second, third]| [*first, *second, *third, 0.0]);
    for (row, matrix_row) in rows[1 + RANGE_COUNT..].iter_mut().zip(matrix_rows) {
        *row = matrix_row;
    }
    rows
}

fn color_grading_rows(grading: &ColorGrading) -> [[f32; 4]; COLOR_GRADING_ROWS] {
    let is_applied = match grading.changes_nothing() {
        true => 0.0,
        false => 1.0,
    };
    let factors = grading.factors();
    let [for_red, for_green, for_blue] = DISPLAY_LUMA_WEIGHTS;
    let [shadows, midtones, highlights, global] = factors
        .offsets
        .map(|[red, green, blue]| [red, green, blue, 0.0]);
    [
        [for_red, for_green, for_blue, is_applied],
        [
            factors.balance_exponent,
            factors.passage,
            factors.shadows_edge,
            factors.highlights_edge,
        ],
        shadows,
        midtones,
        highlights,
        global,
    ]
}

fn stage_rows(
    transform: &DisplayTransform,
    request: &DisplayRequest,
    source: &SourceTexture,
) -> [[f32; 4]; STAGE_ROWS] {
    let [red, green, blue] = transform
        .working_to_rec709()
        .map(|[r, g, b]| [r, g, b, 0.0]);
    let transfer = [
        srgb_transfer::LINEAR_SLOPE,
        srgb_transfer::LINEAR_CUTOFF,
        srgb_transfer::OFFSET,
        srgb_transfer::GAMMA,
    ];
    let [region_x, region_y] = request.region_min;
    let [region_width, region_height] = request.region_size;
    let region = [region_x, region_y, region_width, region_height];
    let development = &request.development;
    let tone = development.photo_factors().tone;
    let tone_shape = [
        tone.middle_grey,
        tone.white_stops(),
        HIGHLIGHTS_WIDTH_STOPS,
        SHADOWS_WIDTH_STOPS,
    ];
    let tone_limits = [BLACKS_REACH, DARKEST_LUMINANCE, 0.0, 0.0];
    let mask_limits = [
        SHORTEST_GRADIENT_SQUARED,
        SMALLEST_EXTENT,
        FEATHER_REACH,
        0.0,
    ];
    let [for_red, for_green, for_blue] = luminance_weights();
    let luminance = [for_red, for_green, for_blue, 0.0];
    let base_rendering = match development.base_rendering().tone_curve() {
        Some(curve) => [curve.contrast, curve.shoulder, 1.0, 0.0],
        None => [1.0, 1.0, 0.0, 0.0],
    };
    let [extent_x, extent_y] = photo_extent(source.size());
    let mask_count = development.edit.visible_masks().take(MOST_MASKS).count();
    let enhancement_share = match source.is_enhanced() {
        true => development.edit.enhancement_share(),
        false => 0.0,
    };
    let photo = [extent_x, extent_y, mask_count as f32, enhancement_share];
    let [veil_red, veil_green, veil_blue] = OVERLAY_COLOUR;
    let veil_opacity = match request.overlaid_mask {
        Some(_) => OVERLAY_OPACITY,
        None => 0.0,
    };
    let overlay = [veil_red, veil_green, veil_blue, veil_opacity];
    let tone_curves = match development.edit.tone_curves.is_identity() {
        true => [0.0; 4],
        false => [1.0, 0.0, 0.0, 0.0],
    };
    let fixed = [
        red,
        green,
        blue,
        transfer,
        region,
        tone_shape,
        tone_limits,
        luminance,
        base_rendering,
        photo,
        overlay,
        mask_limits,
        tone_curves,
    ];
    let mut rows = [[0.0; 4]; STAGE_ROWS];
    rows[..fixed.len()].copy_from_slice(&fixed);
    let graded = fixed.len() + COLOR_MIXER_ROWS;
    rows[fixed.len()..graded].copy_from_slice(&color_mixer_rows(&development.edit.color_mixer));
    rows[graded..].copy_from_slice(&color_grading_rows(&development.edit.color_grading));
    rows
}

fn uniform_values(
    transform: &DisplayTransform,
    request: &DisplayRequest,
    source: &SourceTexture,
) -> Vec<[f32; 4]> {
    let development = &request.development;
    let masks = development.edit.visible_masks().take(MOST_MASKS);
    let mut rows = Vec::with_capacity(UNIFORM_ROWS);
    rows.extend(stage_rows(transform, request, source));
    rows.extend(adjustment_rows(&development.photo_factors()));
    rows.extend(masks.flat_map(|mask| mask_rows(development, mask)));
    rows.resize(UNIFORM_ROWS - MASK_ROWS, [0.0; 4]);
    match &request.overlaid_mask {
        Some(mask) => rows.extend(mask_rows(development, mask)),
        None => rows.resize(UNIFORM_ROWS, [0.0; 4]),
    }
    rows
}

/// Develops the source with an edit, then encodes it for display: the display
/// transform stays the last step.
pub struct DisplayStage {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    smooth_sampler: wgpu::Sampler,
    pixelated_sampler: wgpu::Sampler,
    no_coverage_layer: wgpu::TextureView,
    curve_lookup: CurveLookup,
    transform: DisplayTransform,
    // The uniforms and the curve lookup are shared: a render writes them, then draws with them.
    one_render_at_a_time: Mutex<()>,
}

impl DisplayStage {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("display_stage.wgsl"));
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ziv display stage"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                targets: &[Some(DISPLAY_FORMAT.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ziv display stage uniforms"),
            size: size_of::<[[f32; 4]; UNIFORM_ROWS]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let smooth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ziv display stage smooth sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let pixelated_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ziv display stage pixelated sampler"),
            ..Default::default()
        });

        Self {
            device: device.clone(),
            queue: queue.clone(),
            pipeline,
            uniforms,
            smooth_sampler,
            pixelated_sampler,
            no_coverage_layer: no_coverage_layer(device),
            curve_lookup: CurveLookup::new(device),
            transform: DisplayTransform::default(),
            one_render_at_a_time: Mutex::new(()),
        }
    }

    pub fn render(&self, source: &SourceTexture, request: &DisplayRequest) -> wgpu::Texture {
        let device = &self.device;
        let [width, height] = request.size;
        let values = uniform_values(&self.transform, request, source);
        let _render = self
            .one_render_at_a_time
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.queue
            .write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&values));
        let tone_curves = &request.development.edit.tone_curves;
        if !tone_curves.is_identity() {
            self.curve_lookup.write(&self.queue, tone_curves);
        }
        let sampler = match request.sampling {
            Sampling::Smooth => &self.smooth_sampler,
            Sampling::Pixelated => &self.pixelated_sampler,
        };

        let sources: Vec<CoverageSource<'_>> =
            request.development.edit.coverage_sources().collect();
        let mut coverage_layers = source.coverage_layers();
        let coverage_layers = coverage_layers
            .showing((device, &self.queue), source.size(), &sources)
            .unwrap_or(&self.no_coverage_layer);

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ziv display output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DISPLAY_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ziv display stage"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(source.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(coverage_layers),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.smooth_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(source.enhancement_view()),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(self.curve_lookup.view()),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ziv display stage"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ziv display stage"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        target
    }
}
