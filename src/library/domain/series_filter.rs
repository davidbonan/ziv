use serde::{Deserialize, Serialize};

use super::mark::{Mark, Rating};

/// Which photos of a series are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SeriesFilter {
    /// Every photo, the rejected ones included.
    #[default]
    All,
    /// The photos rated that much or more and not rejected.
    AtLeast(Rating),
}

impl SeriesFilter {
    /// No star asked is every photo.
    pub fn at_least(stars: u8) -> Self {
        match stars {
            0 => Self::All,
            stars => Self::AtLeast(Rating::of(stars)),
        }
    }

    /// The stars a photo needs; none for every photo.
    pub fn stars(self) -> u8 {
        match self {
            Self::All => 0,
            Self::AtLeast(rating) => rating.stars(),
        }
    }

    pub fn shows(self, mark: &Mark) -> bool {
        match self {
            Self::All => true,
            Self::AtLeast(rating) => mark.rating >= rating && !mark.is_rejected,
        }
    }
}

/// "2 stars or more".
pub fn stars_or_more(stars: u8) -> String {
    match stars {
        1 => "1 star or more".to_owned(),
        stars => format!("{stars} stars or more"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mark(stars: u8, is_rejected: bool) -> Mark {
        Mark {
            rating: Rating::of(stars),
            is_rejected,
        }
    }

    #[test]
    fn every_photo_passes_the_filter_of_all_rejected_ones_included() {
        assert!(SeriesFilter::All.shows(&mark(0, true)));
    }

    #[test]
    fn a_star_filter_shows_the_photos_rated_enough_and_not_rejected() {
        let two_or_more = SeriesFilter::at_least(2);

        assert!(two_or_more.shows(&mark(2, false)));
        assert!(two_or_more.shows(&mark(5, false)));
        assert!(!two_or_more.shows(&mark(1, false)));
        assert!(!two_or_more.shows(&mark(4, true)));
    }

    #[test]
    fn asking_for_no_star_is_asking_for_every_photo() {
        assert_eq!(SeriesFilter::at_least(0), SeriesFilter::All);
    }
}
