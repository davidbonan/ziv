use std::fmt;

use serde::{Deserialize, Serialize};

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// The calendar day a series was imported, where the user was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportDay {
    pub year: i32,
    /// 1 is January.
    pub month: u8,
    pub day: u8,
}

/// "14 Sep": the year is left out, a series is worked on within weeks.
impl fmt::Display for ImportDay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let month = usize::from(self.month)
            .checked_sub(1)
            .and_then(|index| MONTH_NAMES.get(index))
            .unwrap_or(&"?");
        write!(formatter, "{} {month}", self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_reads_as_day_then_short_month() {
        let day = ImportDay {
            year: 2026,
            month: 9,
            day: 14,
        };

        assert_eq!(day.to_string(), "14 Sep");
    }
}
