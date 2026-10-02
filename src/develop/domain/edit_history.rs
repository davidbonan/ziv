use std::mem::replace;

use super::edit::Edit;

/// The successive states of one photo's edit. A gesture that changes the edit
/// many times in a row, like a slider drag, is undone as a whole.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditHistory {
    undoable: Vec<Edit>,
    current: Edit,
    redoable: Vec<Edit>,
    is_gesture_ongoing: bool,
    has_gesture_opened_a_step: bool,
}

impl EditHistory {
    pub fn starting_from(edit: Edit) -> Self {
        Self {
            current: edit,
            ..Self::default()
        }
    }

    pub fn current(&self) -> &Edit {
        &self.current
    }

    /// A change made while a gesture is ongoing belongs to the step that
    /// gesture opened. Whether the edit is another one than before.
    pub fn change(&mut self, edit: Edit) -> bool {
        if edit == self.current {
            return false;
        }
        let previous = replace(&mut self.current, edit);
        if !self.has_gesture_opened_a_step {
            self.undoable.push(previous);
            self.redoable.clear();
        }
        self.has_gesture_opened_a_step = self.is_gesture_ongoing;
        true
    }

    pub fn set_gesture_ongoing(&mut self, is_ongoing: bool) {
        if is_ongoing != self.is_gesture_ongoing {
            self.has_gesture_opened_a_step = false;
        }
        self.is_gesture_ongoing = is_ongoing;
    }

    /// Whether there was something to undo.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undoable.pop() else {
            return false;
        };
        self.redoable.push(replace(&mut self.current, previous));
        true
    }

    /// Whether there was something to redo.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redoable.pop() else {
            return false;
        };
        self.undoable.push(replace(&mut self.current, next));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::develop::domain::adjustments::Adjustments;

    fn exposed(exposure: f32) -> Edit {
        Edit::from(Adjustments {
            exposure,
            ..Adjustments::default()
        })
    }

    fn changed(edits: &[Edit]) -> EditHistory {
        let mut history = EditHistory::default();
        for edit in edits {
            history.change(edit.clone());
        }
        history
    }

    #[test]
    fn undo_goes_back_one_change_and_redo_brings_it_back() {
        let mut history = changed(&[exposed(1.0), exposed(2.0)]);

        assert!(history.undo());
        assert_eq!(*history.current(), exposed(1.0));
        assert!(history.redo());
        assert_eq!(*history.current(), exposed(2.0));
    }

    #[test]
    fn nothing_to_undo_or_redo_leaves_the_edit_as_it_is() {
        let mut history = EditHistory::starting_from(exposed(1.0));

        assert!(!history.undo());
        assert!(!history.redo());
        assert_eq!(*history.current(), exposed(1.0));
    }

    #[test]
    fn a_change_after_an_undo_discards_what_could_be_redone() {
        let mut history = changed(&[exposed(1.0), exposed(2.0)]);
        history.undo();

        history.change(exposed(3.0));

        assert!(!history.redo());
        assert_eq!(*history.current(), exposed(3.0));
    }

    #[test]
    fn a_whole_gesture_is_one_step() {
        let mut history = changed(&[exposed(1.0)]);

        history.set_gesture_ongoing(true);
        history.change(exposed(1.2));
        history.change(exposed(1.3));
        history.set_gesture_ongoing(false);
        history.undo();

        assert_eq!(*history.current(), exposed(1.0));
    }

    #[test]
    fn two_gestures_in_a_row_are_two_steps() {
        let mut history = EditHistory::default();
        for exposure in [1.0, 2.0] {
            history.set_gesture_ongoing(true);
            history.change(exposed(exposure - 0.5));
            history.change(exposed(exposure));
            history.set_gesture_ongoing(false);
        }

        history.undo();

        assert_eq!(*history.current(), exposed(1.0));
    }

    #[test]
    fn setting_the_same_edit_again_is_not_a_step() {
        let mut history = changed(&[exposed(1.0), exposed(1.0)]);

        history.undo();

        assert_eq!(*history.current(), Edit::default());
        assert!(!history.undo());
    }
}
