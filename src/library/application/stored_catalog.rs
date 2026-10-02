use crate::library::domain::catalog::Catalog;
use crate::library::domain::catalog_storage::CatalogStorage;

/// The catalog, kept stored as it changes.
pub struct StoredCatalog<Storage> {
    storage: Storage,
    catalog: Catalog,
    /// Why what is stored could not be used; it is then never written over.
    unusable_storage: Option<String>,
}

impl<Storage: CatalogStorage> StoredCatalog<Storage> {
    /// Starts from what `storage` holds: no series when it holds nothing, or nothing usable.
    pub fn read_from(storage: Storage) -> Self {
        let (catalog, unusable_storage) = match storage.stored_catalog() {
            Ok(stored) => (stored.unwrap_or_default(), None),
            Err(reason) => (Catalog::default(), Some(reason)),
        };
        Self {
            storage,
            catalog,
            unusable_storage,
        }
    }

    pub fn current(&self) -> &Catalog {
        &self.catalog
    }

    pub fn unusable_storage(&self) -> Option<&str> {
        self.unusable_storage.as_deref()
    }

    /// Applies `change` and stores the catalog when it changed it.
    /// Returns why it could not be stored.
    pub fn change(&mut self, change: impl FnOnce(&mut Catalog)) -> Option<String> {
        let before = self.catalog.clone();
        change(&mut self.catalog);
        if self.catalog == before || self.unusable_storage.is_some() {
            return None;
        }
        self.storage.store_catalog(&self.catalog).err()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;

    use super::*;
    use crate::library::domain::import_day::ImportDay;
    use crate::library::domain::series::{Import, Series};

    #[derive(Clone, Default)]
    struct MemoryStorage {
        stored: Rc<RefCell<Option<Catalog>>>,
        writes: Rc<RefCell<usize>>,
        unreadable: Option<&'static str>,
        refuses_writes: Rc<RefCell<bool>>,
    }

    impl CatalogStorage for MemoryStorage {
        fn stored_catalog(&self) -> Result<Option<Catalog>, String> {
            match self.unreadable {
                Some(reason) => Err(reason.to_owned()),
                None => Ok(self.stored.borrow().clone()),
            }
        }

        fn store_catalog(&self, catalog: &Catalog) -> Result<(), String> {
            *self.writes.borrow_mut() += 1;
            if *self.refuses_writes.borrow() {
                return Err("read-only folder".to_owned());
            }
            *self.stored.borrow_mut() = Some(catalog.clone());
            Ok(())
        }
    }

    fn two_photos() -> Series {
        let import = Import {
            folder: Some(PathBuf::from("/shoot")),
            photos: vec![PathBuf::from("/shoot/1.jpg"), PathBuf::from("/shoot/2.jpg")],
        };
        let day = ImportDay {
            year: 2026,
            month: 9,
            day: 14,
        };
        Series::imported(import, day).unwrap()
    }

    #[test]
    fn every_change_is_stored_and_found_again() {
        let storage = MemoryStorage::default();
        let mut stored = StoredCatalog::read_from(storage.clone());

        stored.change(|catalog| catalog.import(two_photos()));
        stored.change(Catalog::select_next);

        let reopened = StoredCatalog::read_from(storage);
        assert_eq!(reopened.current(), stored.current());
        assert_eq!(reopened.current().selected_index(), Some(1));
    }

    #[test]
    fn change_that_changes_nothing_is_not_stored() {
        let storage = MemoryStorage::default();
        let mut stored = StoredCatalog::read_from(storage.clone());
        stored.change(|catalog| catalog.import(two_photos()));

        stored.change(Catalog::select_previous);

        assert_eq!(*storage.writes.borrow(), 1);
    }

    #[test]
    fn catalog_that_cannot_be_read_starts_empty_and_is_never_written_over() {
        let storage = MemoryStorage {
            unreadable: Some("written by a newer ziv"),
            ..MemoryStorage::default()
        };
        let mut stored = StoredCatalog::read_from(storage.clone());

        let failure = stored.change(|catalog| catalog.import(two_photos()));

        assert_eq!(stored.unusable_storage(), Some("written by a newer ziv"));
        assert_eq!(stored.current().series().len(), 1);
        assert_eq!(failure, None);
        assert_eq!(*storage.writes.borrow(), 0);
    }

    #[test]
    fn failed_write_is_reported_and_the_next_change_tries_again() {
        let storage = MemoryStorage::default();
        *storage.refuses_writes.borrow_mut() = true;
        let mut stored = StoredCatalog::read_from(storage.clone());

        let failure = stored.change(|catalog| catalog.import(two_photos()));
        assert_eq!(failure.as_deref(), Some("read-only folder"));

        *storage.refuses_writes.borrow_mut() = false;
        let failure = stored.change(Catalog::select_next);

        assert_eq!(failure, None);
        assert_eq!(storage.stored.borrow().as_ref(), Some(stored.current()));
    }
}
