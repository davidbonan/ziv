use super::photo_view::PhotoRegion;
use crate::models::domain::model_runner::ModelOutput;

// The class the detection model gives to a human.
const PERSON_CLASS: usize = 1;
const BOX_VALUES: usize = 4;

fn is_likely(logit: f32) -> bool {
    logit > 0.0
}

/// The persons a detection model found, from left to right. `boxes` holds a
/// centre and a size a candidate, in shares of the photo; `scores` a logit a
/// class for each candidate.
pub fn persons_found(boxes: &ModelOutput, scores: &ModelOutput) -> Vec<PhotoRegion> {
    let classes = scores.shape.last().copied().unwrap_or(1).max(1);
    let (boxes, _) = boxes.values.as_chunks::<BOX_VALUES>();
    let candidates = boxes.iter().zip(scores.values.chunks_exact(classes));
    let mut persons: Vec<PhotoRegion> = candidates
        .filter(|(_, scores)| {
            let likeliest =
                (0..classes).max_by(|one, other| scores[*one].total_cmp(&scores[*other]));
            likeliest == Some(PERSON_CLASS) && is_likely(scores[PERSON_CLASS])
        })
        .map(|(found, _)| PhotoRegion::around([found[0], found[1]], [found[2], found[3]]))
        .collect();
    persons.sort_by(|one, other| one.centre()[0].total_cmp(&other.centre()[0]));
    persons
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOG_CLASS: usize = 2;

    fn scores_for(classes_and_logits: &[(usize, f32)]) -> ModelOutput {
        let scores = classes_and_logits.iter().flat_map(|(class, logit)| {
            let mut scores = [-5.0; 3];
            scores[*class] = *logit;
            scores
        });
        ModelOutput {
            shape: vec![1, classes_and_logits.len(), 3],
            values: scores.collect(),
        }
    }

    #[test]
    fn likely_persons_are_kept_from_left_to_right() {
        let boxes = ModelOutput {
            shape: vec![1, 4, 4],
            values: vec![
                0.8, 0.5, 0.2, 0.6, // a person on the right
                0.5, 0.5, 0.2, 0.2, // a dog
                0.2, 0.5, 0.2, 0.6, // a person on the left
                0.5, 0.2, 0.1, 0.1, // maybe a person
            ],
        };
        let scores = scores_for(&[
            (PERSON_CLASS, 3.0),
            (DOG_CLASS, 3.0),
            (PERSON_CLASS, 2.0),
            (PERSON_CLASS, -1.0),
        ]);

        let persons = persons_found(&boxes, &scores);

        let centres: Vec<f32> = persons.iter().map(|person| person.centre()[0]).collect();
        assert_eq!(persons.len(), 2);
        assert!((centres[0] - 0.2).abs() < 1e-6 && (centres[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn person_at_the_edge_is_cut_at_the_photo() {
        let boxes = ModelOutput {
            shape: vec![1, 1, 4],
            values: vec![0.05, 0.5, 0.2, 0.6],
        };

        let persons = persons_found(&boxes, &scores_for(&[(PERSON_CLASS, 3.0)]));

        assert_eq!(persons[0].min[0], 0.0);
        assert!((persons[0].size[0] - 0.15).abs() < 1e-6);
    }
}
