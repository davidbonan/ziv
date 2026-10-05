use std::fmt;

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const BYTES_PER_KILOBYTE: f64 = 1e3;
const BYTES_PER_MEGABYTE: f64 = 1e6;
const BYTES_PER_GIGABYTE: f64 = 1e9;

/// When the camera says the photo was shot, by its own clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShotAt {
    pub year: u16,
    /// 1 is January.
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

impl ShotAt {
    /// From the way EXIF writes it, "2026:09:14 18:42:07". `None` for anything else.
    pub fn from_exif(text: &str) -> Option<Self> {
        let mut numbers = text.trim().split([':', ' ']).map(str::parse::<u16>);
        let mut next = || numbers.next()?.ok();
        let (year, month, day, hour, minute) = (next()?, next()?, next()?, next()?, next()?);
        let is_on_the_calendar = (1..=12).contains(&month) && (1..=31).contains(&day);
        (is_on_the_calendar && hour < 24 && minute < 60).then_some(Self {
            year,
            month: month as u8,
            day: day as u8,
            hour: hour as u8,
            minute: minute as u8,
        })
    }
}

/// "14 Sep 2026, 18:42".
impl fmt::Display for ShotAt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            year,
            month,
            day,
            hour,
            minute,
        } = self;
        let month = MONTH_NAMES[usize::from(*month) - 1];
        write!(formatter, "{day} {month} {year}, {hour:02}:{minute:02}")
    }
}

/// What a file says about its photo besides how it was shot; `None` for what it does not say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhotoDetails {
    /// Width and height of the upright photo.
    pub pixel_size: Option<[u32; 2]>,
    pub shot_at: Option<ShotAt>,
    pub file_bytes: Option<u64>,
}

fn file_size_label(bytes: u64) -> String {
    let bytes = bytes as f64;
    if bytes < BYTES_PER_MEGABYTE {
        return format!("{:.0} KB", bytes / BYTES_PER_KILOBYTE);
    }
    if bytes < BYTES_PER_GIGABYTE {
        return format!("{:.1} MB", bytes / BYTES_PER_MEGABYTE);
    }
    format!("{:.1} GB", bytes / BYTES_PER_GIGABYTE)
}

impl PhotoDetails {
    /// "7008 × 4672", "14 Sep 2026, 18:42", "34.1 MB": the ones the file gives, in that order.
    pub fn labels(&self) -> Vec<String> {
        [
            self.pixel_size
                .map(|[width, height]| format!("{width} × {height}")),
            self.shot_at.map(|shot_at| shot_at.to_string()),
            self.file_bytes.map(file_size_label),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_detail_is_worded_in_order() {
        let details = PhotoDetails {
            pixel_size: Some([7008, 4672]),
            shot_at: ShotAt::from_exif("2026:09:14 18:42:07"),
            file_bytes: Some(34_120_000),
        };

        assert_eq!(
            details.labels(),
            ["7008 × 4672", "14 Sep 2026, 18:42", "34.1 MB"]
        );
    }

    #[test]
    fn a_detail_the_file_does_not_give_is_left_out() {
        let details = PhotoDetails {
            file_bytes: Some(512_300),
            ..PhotoDetails::default()
        };

        assert_eq!(details.labels(), ["512 KB"]);
    }

    #[test]
    fn a_date_that_is_not_one_is_no_date() {
        for text in [
            "",
            "0000:00:00 00:00:00",
            "2026:13:01 10:00:00",
            "yesterday",
        ] {
            assert_eq!(ShotAt::from_exif(text), None, "{text}");
        }
    }

    #[test]
    fn early_hours_keep_two_digits() {
        let shot_at = ShotAt::from_exif("2026:01:02 07:05:00").unwrap();

        assert_eq!(shot_at.to_string(), "2 Jan 2026, 07:05");
    }
}
