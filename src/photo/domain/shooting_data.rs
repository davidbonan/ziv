/// A speed at least this long is told in seconds, a shorter one as a fraction.
const SLOWEST_FRACTION_SECONDS: f32 = 0.3;

/// The settings the camera wrote in the file; `None` for one it did not write.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShootingData {
    pub iso: Option<u32>,
    /// In millimeters.
    pub focal_length: Option<f32>,
    /// The f-number.
    pub aperture: Option<f32>,
    /// In seconds.
    pub shutter_speed: Option<f32>,
}

fn with_one_decimal_at_most(value: f32) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    match rounded.fract() == 0.0 {
        true => format!("{rounded:.0}"),
        false => format!("{rounded:.1}"),
    }
}

fn shutter_speed_label(seconds: f32) -> String {
    if seconds < SLOWEST_FRACTION_SECONDS {
        return format!("1/{:.0} s", 1.0 / seconds);
    }
    format!("{} s", with_one_decimal_at_most(seconds))
}

impl ShootingData {
    /// "ISO 400", "35 mm", "f/2.8", "1/250 s": the values the file carries, in
    /// that order. One it carries as zero is left out.
    pub fn labels(&self) -> Vec<String> {
        let measured = |value: Option<f32>| value.filter(|value| *value > 0.0);
        let iso = self.iso.filter(|iso| *iso > 0);
        [
            iso.map(|iso| format!("ISO {iso}")),
            measured(self.focal_length).map(|millimeters| format!("{millimeters:.0} mm")),
            measured(self.aperture).map(|number| format!("f/{}", with_one_decimal_at_most(number))),
            measured(self.shutter_speed).map(shutter_speed_label),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shot_at(shutter_speed: f32) -> ShootingData {
        ShootingData {
            shutter_speed: Some(shutter_speed),
            ..ShootingData::default()
        }
    }

    #[test]
    fn every_value_is_worded_in_order() {
        let shooting_data = ShootingData {
            iso: Some(400),
            focal_length: Some(35.0),
            aperture: Some(2.8),
            shutter_speed: Some(1.0 / 250.0),
        };

        assert_eq!(
            shooting_data.labels(),
            ["ISO 400", "35 mm", "f/2.8", "1/250 s"]
        );
    }

    #[test]
    fn a_value_missing_or_zero_is_left_out() {
        let shooting_data = ShootingData {
            iso: Some(100),
            focal_length: Some(0.0),
            aperture: None,
            shutter_speed: Some(2.0),
        };

        assert_eq!(shooting_data.labels(), ["ISO 100", "2 s"]);
        assert!(ShootingData::default().labels().is_empty());
    }

    #[test]
    fn a_whole_aperture_has_no_decimal_and_a_focal_length_is_rounded() {
        let shooting_data = ShootingData {
            focal_length: Some(23.6),
            aperture: Some(8.0),
            ..ShootingData::default()
        };

        assert_eq!(shooting_data.labels(), ["24 mm", "f/8"]);
    }

    #[test]
    fn a_short_speed_is_a_fraction_and_a_long_one_is_in_seconds() {
        assert_eq!(shot_at(0.25).labels(), ["1/4 s"]);
        assert_eq!(shot_at(0.5).labels(), ["0.5 s"]);
        assert_eq!(shot_at(30.0).labels(), ["30 s"]);
    }
}
