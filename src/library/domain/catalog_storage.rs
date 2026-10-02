use super::catalog::Catalog;

/// Where the catalog is kept between two runs of the app.
pub trait CatalogStorage {
    /// `Ok(None)` when no catalog was ever stored; `Err` with the reason when
    /// what is stored cannot be used.
    fn stored_catalog(&self) -> Result<Option<Catalog>, String>;

    fn store_catalog(&self, catalog: &Catalog) -> Result<(), String>;
}
