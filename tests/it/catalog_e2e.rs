use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::edit_storage::EditStorage;
use ziv::develop::infrastructure::sidecar_files::{SidecarFiles, has_sidecar};
use ziv::library::application::stored_catalog::StoredCatalog;
use ziv::library::domain::catalog::Catalog;
use ziv::library::domain::catalog_storage::CatalogStorage;
use ziv::library::domain::import_day::ImportDay;
use ziv::library::domain::mark::Rating;
use ziv::library::domain::series::Series;
use ziv::library::domain::series_filter::SeriesFilter;
use ziv::library::infrastructure::catalog_file::CatalogFile;
use ziv::library::infrastructure::photo_files::import_of;
use ziv::photo::infrastructure::file_decoder::FileDecoder;

const DAY: ImportDay = ImportDay {
    year: 2026,
    month: 9,
    day: 14,
};

/// A folder named `name` holding empty files named `files`.
fn shoot_folder(parent: &Path, name: &str, files: &[&str]) -> PathBuf {
    let folder = parent.join(name);
    fs::create_dir(&folder).unwrap();
    for file in files {
        fs::write(folder.join(file), b"").unwrap();
    }
    folder
}

fn series_imported_from(opened: &[PathBuf]) -> Series {
    let import = import_of(opened, |path| FileDecoder.supports(path));
    Series::imported(import, DAY).unwrap()
}

fn names(photos: &[PathBuf]) -> Vec<String> {
    photos
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn imported_folder_is_a_series_named_after_it_with_its_photos_in_natural_order() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(
        disk.path(),
        "Lofoten",
        &["DSC10.ARW", "DSC2.ARW", "notes.txt"],
    );

    let series = series_imported_from(std::slice::from_ref(&folder));

    assert_eq!(series.name, "Lofoten");
    assert_eq!(series.folder, Some(folder));
    assert_eq!(names(series.session.photos()), ["DSC2.ARW", "DSC10.ARW"]);
}

#[test]
fn imported_files_are_a_photo_set_named_after_its_size_and_day() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(disk.path(), "Lofoten", &["a.jpg", "b.jpg", "c.jpg"]);

    let series = series_imported_from(&[folder.join("a.jpg"), folder.join("c.jpg")]);

    assert_eq!(series.name, "2 photos, 14 Sep");
    assert_eq!(series.folder, None);
}

#[test]
fn catalog_is_found_again_as_it_was_left() {
    let disk = tempfile::tempdir().unwrap();
    let data = disk.path().join("data");
    let first = shoot_folder(disk.path(), "first", &["1.jpg", "2.jpg", "3.jpg"]);
    let second = shoot_folder(disk.path(), "second", &["4.jpg"]);
    let mut stored = StoredCatalog::read_from(CatalogFile::in_folder(data.clone()));
    stored.change(|catalog| catalog.import(series_imported_from(&[second])));
    stored.change(|catalog| catalog.import(series_imported_from(std::slice::from_ref(&first))));
    stored.change(|catalog| catalog.select(2));

    let relaunched = StoredCatalog::read_from(CatalogFile::in_folder(data.clone()));

    assert_eq!(relaunched.current(), stored.current());
    assert_eq!(
        relaunched.current().selected_photo(),
        Some(first.join("3.jpg").as_path())
    );
    let files: Vec<_> = fs::read_dir(data).unwrap().flatten().collect();
    assert_eq!(files.len(), 1, "only the catalog is left: {files:?}");
}

#[test]
fn data_folder_without_catalog_holds_no_series() {
    let disk = tempfile::tempdir().unwrap();

    let file = CatalogFile::in_folder(disk.path().join("never written"));

    assert_eq!(file.stored_catalog(), Ok(None));
    assert_eq!(
        StoredCatalog::read_from(file).current(),
        &Catalog::default()
    );
}

#[test]
fn folder_imported_again_gives_its_series_the_photos_added_since() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(disk.path(), "Lofoten", &["1.jpg", "2.jpg"]);
    let mut catalog = Catalog::default();
    catalog.import(series_imported_from(std::slice::from_ref(&folder)));
    fs::write(folder.join("3.jpg"), b"").unwrap();

    catalog.import(series_imported_from(std::slice::from_ref(&folder)));

    assert_eq!(catalog.series().len(), 1);
    assert_eq!(names(catalog.photos()), ["1.jpg", "2.jpg", "3.jpg"]);
}

#[test]
fn edited_count_is_the_number_of_photos_with_a_sidecar_and_follows_a_reset() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(disk.path(), "Lofoten", &["1.jpg", "2.jpg", "3.jpg"]);
    let series = series_imported_from(std::slice::from_ref(&folder));
    let brighter = Edit::from(Adjustments {
        exposure: 1.0,
        ..Adjustments::default()
    });
    for photo in ["1.jpg", "3.jpg"] {
        SidecarFiles
            .store_edit(&folder.join(photo), &brighter)
            .unwrap();
    }
    assert_eq!(series.edited_count(has_sidecar), 2);

    SidecarFiles
        .store_edit(&folder.join("3.jpg"), &Edit::default())
        .unwrap();

    assert_eq!(series.edited_count(has_sidecar), 1);
}

#[test]
fn located_series_is_back_with_its_edits() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(disk.path(), "Lofoten", &["1.jpg", "2.jpg"]);
    let mut catalog = Catalog::default();
    catalog.import(series_imported_from(std::slice::from_ref(&folder)));
    let brighter = Edit::from(Adjustments {
        exposure: 1.0,
        ..Adjustments::default()
    });
    SidecarFiles
        .store_edit(&folder.join("2.jpg"), &brighter)
        .unwrap();
    let moved = disk.path().join("Archive");
    fs::rename(&folder, &moved).unwrap();
    assert!(catalog.series()[0].is_missing(Path::exists));

    let found = catalog.relocate(0, &moved, Path::exists);

    let series = &catalog.series()[0];
    assert_eq!(found, 2);
    assert!(!series.is_missing(Path::exists));
    assert_eq!(series.folder, Some(moved.clone()));
    assert_eq!(series.edited_count(has_sidecar), 1);
    assert_eq!(
        SidecarFiles.stored_edit(&moved.join("2.jpg")),
        Ok(Some(brighter))
    );
}

#[test]
fn folder_without_the_photos_of_the_series_locates_nothing() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(disk.path(), "Lofoten", &["1.jpg"]);
    let elsewhere = shoot_folder(disk.path(), "Elsewhere", &["9.jpg"]);
    let mut catalog = Catalog::default();
    catalog.import(series_imported_from(std::slice::from_ref(&folder)));
    let before = catalog.clone();

    assert_eq!(catalog.relocate(0, &elsewhere, Path::exists), 0);
    assert_eq!(catalog, before);
}

#[test]
fn removed_series_leaves_its_photos_and_sidecars_on_disk() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(disk.path(), "Lofoten", &["1.jpg"]);
    let brighter = Edit::from(Adjustments {
        exposure: 1.0,
        ..Adjustments::default()
    });
    SidecarFiles
        .store_edit(&folder.join("1.jpg"), &brighter)
        .unwrap();
    let mut stored = StoredCatalog::read_from(CatalogFile::in_folder(disk.path().join("data")));
    stored.change(|catalog| catalog.import(series_imported_from(std::slice::from_ref(&folder))));

    stored.change(|catalog| catalog.remove(0));

    assert!(stored.current().series().is_empty());
    assert!(folder.join("1.jpg").exists());
    assert!(has_sidecar(&folder.join("1.jpg")));
}

fn data_folder_holding(disk: &Path, catalog: &str) -> PathBuf {
    let data = disk.join("data");
    fs::create_dir(&data).unwrap();
    fs::write(data.join("catalog.json"), catalog).unwrap();
    data
}

#[test]
fn catalog_of_a_newer_ziv_or_a_damaged_one_is_reported_and_left_untouched() {
    for unusable in [r#"{ "version": 99, "future": true }"#, "{ not json"] {
        let disk = tempfile::tempdir().unwrap();
        let folder = shoot_folder(disk.path(), "Lofoten", &["1.jpg"]);
        let data = data_folder_holding(disk.path(), unusable);
        let mut stored = StoredCatalog::read_from(CatalogFile::in_folder(data.clone()));

        stored.change(|catalog| catalog.import(series_imported_from(&[folder])));

        assert!(stored.unusable_storage().is_some(), "{unusable}");
        assert_eq!(
            fs::read_to_string(data.join("catalog.json")).unwrap(),
            unusable
        );
    }
}

#[test]
fn data_folder_that_cannot_be_written_reports_the_failure() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(disk.path(), "Lofoten", &["1.jpg"]);
    let locked = disk.path().join("locked");
    fs::create_dir(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).unwrap();
    let mut stored = StoredCatalog::read_from(CatalogFile::in_folder(locked.join("data")));

    let failure = stored.change(|catalog| catalog.import(series_imported_from(&[folder])));
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();

    assert_eq!(stored.unusable_storage(), None);
    assert!(failure.is_some());
    assert_eq!(stored.current().series().len(), 1);
}

#[test]
fn marks_are_found_again_after_a_relaunch_and_make_no_photo_an_edited_one() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(disk.path(), "Lofoten", &["a.jpg", "b.jpg"]);
    let data = disk.path().join("data");
    let mut catalog = StoredCatalog::read_from(CatalogFile::in_folder(data.clone()));
    let series = series_imported_from(std::slice::from_ref(&folder));
    catalog.change(|catalog| catalog.import(series));

    catalog.change(|catalog| catalog.rate(&[0, 1], Rating::of(4)));
    catalog.change(|catalog| catalog.toggle_rejected(&[1]));

    let relaunched = StoredCatalog::read_from(CatalogFile::in_folder(data));
    let marks = ["a.jpg", "b.jpg"].map(|name| relaunched.current().mark_of(&folder.join(name)));
    assert_eq!(marks.map(|mark| mark.rating.stars()), [4, 4]);
    assert_eq!(marks.map(|mark| mark.is_rejected), [false, true]);
    assert_eq!(
        relaunched.current().series()[0].edited_count(has_sidecar),
        0
    );
}

#[test]
fn the_filter_of_each_series_is_found_again_after_a_relaunch() {
    let disk = tempfile::tempdir().unwrap();
    let kept = shoot_folder(disk.path(), "Kept", &["a.jpg"]);
    let new = shoot_folder(disk.path(), "New", &["b.jpg"]);
    let data = disk.path().join("data");
    let mut catalog = StoredCatalog::read_from(CatalogFile::in_folder(data.clone()));
    catalog.change(|catalog| catalog.import(series_imported_from(std::slice::from_ref(&kept))));
    catalog.change(|catalog| catalog.set_filter(SeriesFilter::at_least(2)));
    catalog.change(|catalog| catalog.import(series_imported_from(std::slice::from_ref(&new))));

    let relaunched = StoredCatalog::read_from(CatalogFile::in_folder(data));

    let filters: Vec<SeriesFilter> = relaunched
        .current()
        .series()
        .iter()
        .map(|series| series.filter)
        .collect();
    assert_eq!(filters, [SeriesFilter::All, SeriesFilter::at_least(2)]);
}

#[test]
fn photo_removed_from_its_series_stays_on_disk_and_out_of_a_new_import_of_the_folder() {
    let disk = tempfile::tempdir().unwrap();
    let folder = shoot_folder(disk.path(), "Lofoten", &["a.jpg", "b.jpg"]);
    let data = disk.path().join("data");
    let mut catalog = StoredCatalog::read_from(CatalogFile::in_folder(data.clone()));
    catalog.change(|catalog| catalog.import(series_imported_from(std::slice::from_ref(&folder))));

    catalog.change(|catalog| catalog.remove_photos(&[0]));

    assert!(folder.join("a.jpg").exists());
    let mut relaunched = StoredCatalog::read_from(CatalogFile::in_folder(data));
    fs::write(folder.join("c.jpg"), b"").unwrap();
    relaunched.change(|catalog| catalog.import(series_imported_from(&[folder])));
    assert_eq!(names(relaunched.current().photos()), ["b.jpg", "c.jpg"]);
}
