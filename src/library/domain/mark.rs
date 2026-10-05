use serde::{Deserialize, Serialize};

pub const MOST_STARS: u8 = 5;

/// From 0 to 5 stars; 0 is unrated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
pub struct Rating(u8);

impl Rating {
    /// More than five stars is five.
    pub const fn of(stars: u8) -> Self {
        match stars > MOST_STARS {
            true => Self(MOST_STARS),
            false => Self(stars),
        }
    }

    pub fn stars(self) -> u8 {
        self.0
    }
}

impl From<u8> for Rating {
    fn from(stars: u8) -> Self {
        Self::of(stars)
    }
}

impl From<Rating> for u8 {
    fn from(rating: Rating) -> Self {
        rating.0
    }
}

/// What culling says of a photo. It is not an edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Mark {
    #[serde(default)]
    pub rating: Rating,
    #[serde(default)]
    pub is_rejected: bool,
}

impl Mark {
    /// Whether it says nothing: unrated and not rejected.
    pub fn is_blank(&self) -> bool {
        *self == Self::default()
    }

    /// ", 3 stars", ", rejected": what assistive technology reads after the photo's name.
    pub fn spoken(&self) -> String {
        let stars = match self.rating.stars() {
            0 => String::new(),
            1 => ", 1 star".to_owned(),
            stars => format!(", {stars} stars"),
        };
        let rejected = match self.is_rejected {
            true => ", rejected",
            false => "",
        };
        format!("{stars}{rejected}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rating_never_has_more_than_five_stars() {
        assert_eq!(Rating::of(9).stars(), 5);
        assert_eq!(Rating::of(3).stars(), 3);
    }

    #[test]
    fn a_mark_is_spoken_stars_first() {
        let mark = Mark {
            rating: Rating::of(1),
            is_rejected: true,
        };

        assert_eq!(mark.spoken(), ", 1 star, rejected");
        assert_eq!(Mark::default().spoken(), "");
    }
}
