use std::collections::BTreeSet;

/// The photos a mark or a removal acts on, by their place in the open series.
/// It always holds the selected photo, which the series itself remembers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PhotoSelection {
    besides_selected: BTreeSet<usize>,
}

impl PhotoSelection {
    pub fn contains(&self, photo: usize, selected: usize) -> bool {
        photo == selected || self.besides_selected.contains(&photo)
    }

    /// Every photo of the selection, in series order.
    pub fn photos(&self, selected: usize) -> Vec<usize> {
        let mut photos = self.besides_selected.clone();
        photos.insert(selected);
        photos.into_iter().collect()
    }

    /// Adds `photo` to the selection or takes it out; the last photo of a
    /// selection stays in. Returns the photo to select.
    pub fn toggle(&mut self, photo: usize, selected: usize) -> usize {
        if !self.contains(photo, selected) {
            self.besides_selected.insert(selected);
            return photo;
        }
        self.besides_selected.remove(&photo);
        if photo != selected {
            return selected;
        }
        self.besides_selected.pop_first().unwrap_or(selected)
    }

    pub fn of(photos: impl IntoIterator<Item = usize>) -> Self {
        Self {
            besides_selected: photos.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_starts_as_the_selected_photo_alone() {
        let selection = PhotoSelection::default();

        assert_eq!(selection.photos(2), [2]);
    }

    #[test]
    fn toggling_adds_a_photo_and_selects_it_then_takes_it_out() {
        let mut selection = PhotoSelection::default();

        let selected = selection.toggle(5, 2);
        assert_eq!((selected, selection.photos(selected)), (5, vec![2, 5]));

        let selected = selection.toggle(2, 5);
        assert_eq!((selected, selection.photos(selected)), (5, vec![5]));
    }

    #[test]
    fn toggling_the_selected_photo_out_selects_another_of_the_selection() {
        let mut selection = PhotoSelection::default();
        let selected = selection.toggle(5, 2);

        let selected = selection.toggle(5, selected);

        assert_eq!((selected, selection.photos(selected)), (2, vec![2]));
    }

    #[test]
    fn the_last_photo_of_a_selection_stays_in() {
        let mut selection = PhotoSelection::default();

        let selected = selection.toggle(2, 2);

        assert_eq!((selected, selection.photos(selected)), (2, vec![2]));
    }

    #[test]
    fn selection_of_photos_holds_them_and_the_selected_one() {
        assert_eq!(PhotoSelection::of([0, 3]).photos(1), [0, 1, 3]);
    }
}
