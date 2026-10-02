use crate::develop::domain::zone::PersonPart;

use super::photo_view::PhotoRegion;

/// The persons found in the photo, and what the user chose to mask of whom.
#[derive(Debug, Clone, PartialEq)]
pub struct PeoplePick {
    persons: Vec<PhotoRegion>,
    is_person_chosen: Vec<bool>,
    parts: Vec<PersonPart>,
}

impl PeoplePick {
    /// Every person chosen, no part yet.
    pub fn among(persons: Vec<PhotoRegion>) -> Self {
        Self {
            is_person_chosen: vec![true; persons.len()],
            persons,
            parts: Vec::new(),
        }
    }

    /// Each person, from left to right, and whether they are chosen.
    pub fn persons(&self) -> impl Iterator<Item = (&PhotoRegion, bool)> {
        self.persons
            .iter()
            .zip(self.is_person_chosen.iter().copied())
    }

    pub fn is_part_chosen(&self, part: PersonPart) -> bool {
        self.parts.contains(&part)
    }

    pub fn with_person_toggled(mut self, person: usize) -> Self {
        self.is_person_chosen[person] = !self.is_person_chosen[person];
        self
    }

    pub fn with_part_toggled(mut self, part: PersonPart) -> Self {
        match self.is_part_chosen(part) {
            true => self.parts.retain(|chosen| *chosen != part),
            false => self.parts.push(part),
        }
        self
    }

    pub fn chosen_persons(&self) -> Vec<PhotoRegion> {
        let chosen = self.persons().filter(|(_, is_chosen)| *is_chosen);
        chosen.map(|(person, _)| *person).collect()
    }

    /// In the order parts are listed, whatever the order they were chosen in.
    pub fn chosen_parts(&self) -> Vec<PersonPart> {
        let chosen = PersonPart::ALL
            .into_iter()
            .filter(|part| self.is_part_chosen(*part));
        chosen.collect()
    }

    pub fn can_create_masks(&self) -> bool {
        !self.parts.is_empty() && self.is_person_chosen.contains(&true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_persons() -> PeoplePick {
        let person_at = |left: f32| PhotoRegion {
            min: [left, 0.2],
            size: [0.2, 0.6],
        };
        PeoplePick::among(vec![person_at(0.1), person_at(0.6)])
    }

    #[test]
    fn everyone_is_chosen_at_first_and_no_part_is() {
        let pick = two_persons();

        assert_eq!(pick.chosen_persons().len(), 2);
        assert!(pick.chosen_parts().is_empty());
        assert!(!pick.can_create_masks());
    }

    #[test]
    fn masks_can_be_created_once_a_person_and_a_part_are_chosen() {
        let with_a_part = two_persons().with_part_toggled(PersonPart::Hair);
        let with_nobody = with_a_part
            .clone()
            .with_person_toggled(0)
            .with_person_toggled(1);

        assert!(with_a_part.can_create_masks());
        assert!(!with_nobody.can_create_masks());
    }

    #[test]
    fn chosen_parts_are_in_their_listed_order_and_can_be_unchosen() {
        let pick = two_persons()
            .with_part_toggled(PersonPart::Clothes)
            .with_part_toggled(PersonPart::Lips)
            .with_part_toggled(PersonPart::Skin)
            .with_part_toggled(PersonPart::Lips);

        assert_eq!(pick.chosen_parts(), [PersonPart::Skin, PersonPart::Clothes]);
    }

    #[test]
    fn unchosen_person_is_left_out() {
        let pick = two_persons().with_person_toggled(0);

        let chosen = pick.chosen_persons();

        assert_eq!(chosen.len(), 1);
        assert_eq!(chosen[0].min[0], 0.6);
    }
}
