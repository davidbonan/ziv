//! IEC 61966-2-1 piecewise transfer function.

pub const LINEAR_SLOPE: f32 = 12.92;
pub const LINEAR_CUTOFF: f32 = 0.003_130_8;
pub const ENCODED_CUTOFF: f32 = 0.040_45;
pub const OFFSET: f32 = 0.055;
pub const GAMMA: f32 = 2.4;

pub fn encode(linear: f32) -> f32 {
    if linear <= LINEAR_CUTOFF {
        return linear * LINEAR_SLOPE;
    }
    (1.0 + OFFSET) * linear.powf(1.0 / GAMMA) - OFFSET
}

pub fn decode(encoded: f32) -> f32 {
    if encoded <= ENCODED_CUTOFF {
        return encoded / LINEAR_SLOPE;
    }
    ((encoded + OFFSET) / (1.0 + OFFSET)).powf(GAMMA)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_and_white_are_fixed_points() {
        assert_eq!(encode(0.0), 0.0);
        assert!((encode(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn middle_grey_encodes_to_its_known_value() {
        assert!((encode(0.18) - 0.461_356).abs() < 1e-5);
    }

    #[test]
    fn both_branches_meet_at_the_cutoff() {
        let below = encode(LINEAR_CUTOFF);
        let above = encode(LINEAR_CUTOFF + f32::EPSILON);

        assert!((below - above).abs() < 1e-5);
        assert!((below - ENCODED_CUTOFF).abs() < 1e-5);
    }

    #[test]
    fn every_8_bit_code_survives_decode_then_encode() {
        for code in 0..=255u8 {
            let encoded = f32::from(code) / 255.0;
            let round_trip = (encode(decode(encoded)) * 255.0).round();

            assert_eq!(round_trip, f32::from(code));
        }
    }
}
