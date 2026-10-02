use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use ziv::photo::domain::decode_error::DecodeError;
use ziv::photo::domain::thumbnail::Thumbnail;
use ziv::photo::infrastructure::file_decoder::FileDecoder;
use ziv::photo::infrastructure::thumbnail_cache::ThumbnailCache;

struct Shoot {
    disk: tempfile::TempDir,
    photo: PathBuf,
    decodings: Cell<usize>,
}

fn shoot() -> Shoot {
    let disk = tempfile::tempdir().unwrap();
    let photo = disk.path().join("patches.png");
    fs::copy("tests/fixtures/patches.png", &photo).unwrap();
    Shoot {
        disk,
        photo,
        decodings: Cell::new(0),
    }
}

impl Shoot {
    fn cache(&self) -> ThumbnailCache {
        ThumbnailCache::in_folder(self.disk.path().join("cache"))
    }

    fn thumbnail_through(&self, cache: &ThumbnailCache) -> Result<Thumbnail, DecodeError> {
        cache.thumbnail_of(&self.photo, |photo: &Path| {
            self.decodings.set(self.decodings.get() + 1);
            FileDecoder.decode_thumbnail(photo)
        })
    }
}

fn close_to(left: &Thumbnail, right: &Thumbnail) -> bool {
    let is_same_size = (left.width, left.height) == (right.width, right.height);
    let channels = left.rgba.iter().zip(&right.rgba);
    is_same_size && channels.into_iter().all(|(a, b)| a.abs_diff(*b) < 40)
}

#[test]
fn second_run_shows_the_thumbnail_without_decoding_the_photo() {
    let shoot = shoot();
    let decoded = shoot.thumbnail_through(&shoot.cache()).unwrap();

    let kept = shoot.thumbnail_through(&shoot.cache()).unwrap();

    assert_eq!(shoot.decodings.get(), 1);
    assert!(close_to(&kept, &decoded), "the kept thumbnail is the photo");
}

#[test]
fn photo_changed_on_disk_is_decoded_again() {
    let shoot = shoot();
    shoot.thumbnail_through(&shoot.cache()).unwrap();
    let later = SystemTime::now() + Duration::from_secs(60);
    fs::File::options()
        .write(true)
        .open(&shoot.photo)
        .unwrap()
        .set_modified(later)
        .unwrap();

    shoot.thumbnail_through(&shoot.cache()).unwrap();

    assert_eq!(shoot.decodings.get(), 2);
}

#[test]
fn cache_that_cannot_be_written_still_gives_the_thumbnail() {
    let shoot = shoot();
    let blocked = shoot.disk.path().join("a file where the cache should be");
    fs::write(&blocked, b"").unwrap();
    let cache = ThumbnailCache::in_folder(blocked.join("cache"));

    assert!(shoot.thumbnail_through(&cache).is_ok());
    assert!(shoot.thumbnail_through(&cache).is_ok());
    assert_eq!(shoot.decodings.get(), 2);
}

#[test]
fn photo_that_cannot_be_decoded_fails_as_without_a_cache() {
    let shoot = shoot();
    fs::write(&shoot.photo, b"not a picture").unwrap();

    assert!(shoot.thumbnail_through(&shoot.cache()).is_err());
}
