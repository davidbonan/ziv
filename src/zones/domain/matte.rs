use std::ops::RangeInclusive;

/// The coverage, 0 … 255, a model gives as logits: 0 is half covered.
pub fn coverage_of_logits(logits: &[f32]) -> Vec<u8> {
    logits
        .iter()
        .map(|logit| (255.0 / (1.0 + (-logit).exp())).round() as u8)
        .collect()
}

/// How sure a model is, 0 … 1, from the shares it gives: not at all up to
/// the start of `unsure`, fully from its end, progressively between.
pub fn sureness_of_shares(shares: &[f32], unsure: &RangeInclusive<f32>) -> Vec<f32> {
    let width = unsure.end() - unsure.start();
    shares
        .iter()
        .map(|share| ((share - unsure.start()) / width).clamp(0.0, 1.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logits_become_coverage_through_the_sigmoid() {
        assert_eq!(coverage_of_logits(&[-20.0, 0.0, 20.0]), [0, 128, 255]);
    }

    #[test]
    fn shares_are_unsure_then_progressively_sure() {
        let sureness = sureness_of_shares(&[-0.2, 0.25, 0.5, 0.75, 1.3], &(0.25..=0.75));

        assert_eq!(sureness, [0.0, 0.0, 0.5, 1.0, 1.0]);
    }
}
