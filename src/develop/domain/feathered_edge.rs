use super::radial_gradient::SMALLEST_EXTENT;

/// How far inside a rectangle or a polygon a feather of 100 reaches, in photo units.
pub const FEATHER_REACH: f32 = 0.2;

/// Coverage at `depth` inside a shape's outline (negative outside): none on
/// the outline, full once the feather is crossed.
pub fn feathered_coverage(depth: f32, feather: f32) -> f32 {
    let width = (feather / 100.0 * FEATHER_REACH).max(SMALLEST_EXTENT);
    let progress = (depth / width).clamp(0.0, 1.0);
    progress * progress * (3.0 - 2.0 * progress)
}
