use std::fmt;

use serde::{Deserialize, Serialize};

use super::catalog::Catalog;

pub const CURRENT_VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
struct CatalogDocument {
    version: u32,
    #[serde(flatten)]
    catalog: Catalog,
}

#[derive(Deserialize)]
struct VersionOnly {
    version: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentError {
    NotACatalog(String),
    FromNewerZiv { version: u32 },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotACatalog(reason) => write!(formatter, "it cannot be read ({reason})"),
            Self::FromNewerZiv { version } => write!(
                formatter,
                "it was written by a newer ziv (format {version}, this ziv reads up to {CURRENT_VERSION})"
            ),
        }
    }
}

/// `Err` for a photo path that is not Unicode: JSON cannot hold it.
pub fn catalog_document(catalog: &Catalog) -> Result<String, String> {
    let document = CatalogDocument {
        version: CURRENT_VERSION,
        catalog: catalog.clone(),
    };
    serde_json::to_string_pretty(&document).map_err(|error| error.to_string())
}

pub fn catalog_of_document(text: &str) -> Result<Catalog, DocumentError> {
    let not_a_catalog = |error: serde_json::Error| DocumentError::NotACatalog(error.to_string());
    let VersionOnly { version } = serde_json::from_str(text).map_err(not_a_catalog)?;
    if version > CURRENT_VERSION {
        return Err(DocumentError::FromNewerZiv { version });
    }
    let document: CatalogDocument = serde_json::from_str(text).map_err(not_a_catalog)?;
    if !document.catalog.is_consistent() {
        let reason = "it selects a series or a photo it does not hold";
        return Err(DocumentError::NotACatalog(reason.to_owned()));
    }
    Ok(document.catalog)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::library::domain::import_day::ImportDay;
    use crate::library::domain::mark::Rating;
    use crate::library::domain::series::{Import, Series};

    fn catalog_of_two_series() -> Catalog {
        let day = ImportDay {
            year: 2026,
            month: 9,
            day: 14,
        };
        let folder_series = Import {
            folder: Some(PathBuf::from("/shoots/Lofoten")),
            photos: vec![
                PathBuf::from("/shoots/Lofoten/1.ARW"),
                PathBuf::from("/shoots/Lofoten/2.ARW"),
            ],
        };
        let photo_set = Import {
            folder: None,
            photos: vec![PathBuf::from("/a/x.jpg"), PathBuf::from("/b/y.jpg")],
        };
        let mut catalog = Catalog::default();
        catalog.import(Series::imported(folder_series, day).unwrap());
        catalog.import(Series::imported(photo_set, day).unwrap());
        catalog.select_next();
        catalog
    }

    #[test]
    fn catalog_survives_its_document() {
        let catalog = catalog_of_two_series();

        let document = catalog_document(&catalog).unwrap();

        assert_eq!(catalog_of_document(&document), Ok(catalog));
    }

    #[test]
    fn marks_survive_the_document() {
        let mut catalog = catalog_of_two_series();
        catalog.rate(&[0, 1], Rating::of(4));
        catalog.toggle_rejected(&[1]);

        let document = catalog_document(&catalog).unwrap();

        assert_eq!(catalog_of_document(&document), Ok(catalog));
    }

    #[test]
    fn document_written_before_marks_existed_is_read_without_any() {
        let document = r#"{ "version": 1, "series": [], "open": null }"#;

        assert_eq!(catalog_of_document(document), Ok(Catalog::default()));
    }

    #[test]
    fn document_of_a_newer_ziv_is_refused() {
        let document = r#"{ "version": 99, "series": [], "open": null }"#;

        assert_eq!(
            catalog_of_document(document),
            Err(DocumentError::FromNewerZiv { version: 99 })
        );
    }

    #[test]
    fn text_that_is_not_a_catalog_is_refused() {
        assert!(matches!(
            catalog_of_document("not json"),
            Err(DocumentError::NotACatalog(_))
        ));
    }

    #[test]
    fn document_opening_a_series_it_does_not_hold_is_refused() {
        let document = r#"{ "version": 1, "series": [], "open": 3 }"#;

        assert!(matches!(
            catalog_of_document(document),
            Err(DocumentError::NotACatalog(_))
        ));
    }
}
