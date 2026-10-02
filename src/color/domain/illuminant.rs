use super::matrix3::{self, Matrix3};

/// Tint units per unit of distance to the Planckian locus in CIE 1960 uv.
const TINT_PER_UV: f64 = 3000.0;
const MIRED: f64 = 1.0e6;
const HOTTEST_KELVIN: f64 = 100_000.0;
const COOLEST_KELVIN: f64 = 1_000.0;
const BISECTIONS: usize = 60;
const BRADFORD: Matrix3 = [
    [0.8951, 0.2664, -0.1614],
    [-0.7502, 1.7135, 0.0367],
    [0.0389, -0.0685, 1.0296],
];

/// The light a photo was taken under: a black-body temperature in kelvin and
/// how far the light is from it, toward green when positive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Illuminant {
    pub temperature: f32,
    pub tint: f32,
}

// Krystek's approximation of the Planckian locus.
fn planckian_uv(kelvin: f64) -> [f64; 2] {
    let squared = kelvin * kelvin;
    [
        (0.860_117_757 + 1.541_182_54e-4 * kelvin + 1.286_412_12e-7 * squared)
            / (1.0 + 8.424_202_35e-4 * kelvin + 7.081_451_63e-7 * squared),
        (0.317_398_726 + 4.228_062_45e-5 * kelvin + 4.204_816_91e-8 * squared)
            / (1.0 - 2.897_418_16e-5 * kelvin + 1.614_560_53e-7 * squared),
    ]
}

fn toward_hotter(kelvin: f64) -> [f64; 2] {
    let step = kelvin * 1.0e-3;
    let [hotter_u, hotter_v] = planckian_uv(kelvin + step);
    let [cooler_u, cooler_v] = planckian_uv(kelvin - step);
    let [u, v] = [hotter_u - cooler_u, hotter_v - cooler_v];
    let length = u.hypot(v);
    [u / length, v / length]
}

fn toward_green(kelvin: f64) -> [f64; 2] {
    let [u, v] = toward_hotter(kelvin);
    [v, -u]
}

fn dot([left_u, left_v]: [f64; 2], [right_u, right_v]: [f64; 2]) -> f64 {
    left_u * right_u + left_v * right_v
}

fn offset_from_locus([u, v]: [f64; 2], kelvin: f64) -> [f64; 2] {
    let [locus_u, locus_v] = planckian_uv(kelvin);
    [u - locus_u, v - locus_v]
}

fn nearest_kelvin(uv: [f64; 2]) -> f64 {
    let mut hottest = MIRED / HOTTEST_KELVIN;
    let mut coolest = MIRED / COOLEST_KELVIN;
    for _ in 0..BISECTIONS {
        let middle = (hottest + coolest) / 2.0;
        let kelvin = MIRED / middle;
        if dot(offset_from_locus(uv, kelvin), toward_hotter(kelvin)) > 0.0 {
            coolest = middle;
        } else {
            hottest = middle;
        }
    }
    MIRED / hottest
}

impl Illuminant {
    fn uv(&self) -> [f64; 2] {
        let kelvin = f64::from(self.temperature);
        let distance = f64::from(self.tint) / TINT_PER_UV;
        let [u, v] = planckian_uv(kelvin);
        let [green_u, green_v] = toward_green(kelvin);
        [u + distance * green_u, v + distance * green_v]
    }

    /// CIE XYZ of this light's white, at unit luminance.
    pub fn white_xyz(&self) -> [f64; 3] {
        let [u, v] = self.uv();
        [1.5 * u / v, 1.0, (4.0 - u - 10.0 * v) / (2.0 * v)]
    }

    pub fn of_white_xyz([x, y, z]: [f64; 3]) -> Self {
        let sum = x + 15.0 * y + 3.0 * z;
        let uv = [4.0 * x / sum, 6.0 * y / sum];
        let kelvin = nearest_kelvin(uv);
        let tint = dot(offset_from_locus(uv, kelvin), toward_green(kelvin)) * TINT_PER_UV;
        Self {
            temperature: kelvin as f32,
            tint: tint as f32,
        }
    }

    /// The CIE XYZ transform under which this light's white becomes the white
    /// of `reference` (Bradford chromatic adaptation).
    pub fn adaptation_to(&self, reference: &Illuminant) -> Matrix3 {
        let cone_response =
            |illuminant: &Illuminant| matrix3::transform(&BRADFORD, illuminant.white_xyz());
        let [from, to] = [cone_response(self), cone_response(reference)];
        let scaled = [0, 1, 2].map(|cone| BRADFORD[cone].map(|cell| cell * to[cone] / from[cone]));
        let from_cones = matrix3::inverse(&BRADFORD).expect("the Bradford matrix is invertible");
        matrix3::multiply(&from_cones, &scaled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const D65_XYZ: [f64; 3] = [0.950_47, 1.0, 1.088_83];
    const IDENTITY: Matrix3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    fn daylight() -> Illuminant {
        Illuminant {
            temperature: 5200.0,
            tint: 12.0,
        }
    }

    fn assert_matrix_close(actual: &Matrix3, expected: &Matrix3) {
        for (actual, expected) in actual.iter().flatten().zip(expected.iter().flatten()) {
            assert!(
                (actual - expected).abs() < 1e-9,
                "{actual} is not {expected}"
            );
        }
    }

    #[test]
    fn d65_white_is_near_6500_kelvin_with_little_tint() {
        let d65 = Illuminant::of_white_xyz(D65_XYZ);

        assert!((6400.0..6600.0).contains(&d65.temperature), "{d65:?}");
        assert!(d65.tint.abs() < 15.0, "{d65:?}");
    }

    #[test]
    fn an_illuminant_is_found_back_from_its_white() {
        for illuminant in [
            daylight(),
            Illuminant {
                temperature: 2800.0,
                tint: -40.0,
            },
            Illuminant {
                temperature: 20000.0,
                tint: 90.0,
            },
        ] {
            let found = Illuminant::of_white_xyz(illuminant.white_xyz());

            assert!(
                (found.temperature / illuminant.temperature - 1.0).abs() < 1e-3,
                "{found:?} is not {illuminant:?}"
            );
            assert!((found.tint - illuminant.tint).abs() < 0.1, "{found:?}");
        }
    }

    #[test]
    fn positive_tint_is_greener_and_hotter_is_bluer() {
        let [_, _, daylight_blue] = daylight().white_xyz();
        let hotter = Illuminant {
            temperature: 9000.0,
            ..daylight()
        };
        let greener = Illuminant {
            tint: 60.0,
            ..daylight()
        };

        assert!(hotter.white_xyz()[2] > daylight_blue);
        let [x, _, z] = greener.white_xyz();
        let [daylight_x, _, daylight_z] = daylight().white_xyz();
        assert!(x + z < daylight_x + daylight_z);
    }

    #[test]
    fn adapting_to_the_same_light_changes_nothing() {
        assert_matrix_close(&daylight().adaptation_to(&daylight()), &IDENTITY);
    }

    #[test]
    fn adaptation_sends_the_white_of_one_light_to_the_other() {
        let tungsten = Illuminant {
            temperature: 2850.0,
            tint: 0.0,
        };

        let adapted =
            matrix3::transform(&tungsten.adaptation_to(&daylight()), tungsten.white_xyz());

        for (adapted, expected) in adapted.iter().zip(daylight().white_xyz()) {
            assert!((adapted - expected).abs() < 1e-9);
        }
    }
}
