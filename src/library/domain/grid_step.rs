/// A move of the selection in a grid of photos laid out in rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridStep {
    Previous,
    Next,
    Above,
    Below,
}

impl GridStep {
    /// The photo reached from `index` among `count` photos in rows of
    /// `columns`; `index` itself where the grid ends.
    pub fn from(self, index: usize, count: usize, columns: usize) -> usize {
        let reached = match self {
            Self::Previous => index.checked_sub(1),
            Self::Next => Some(index + 1),
            Self::Above => index.checked_sub(columns),
            Self::Below => Some(index + columns),
        };
        reached.filter(|reached| *reached < count).unwrap_or(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COUNT: usize = 7;
    const COLUMNS: usize = 3;

    fn reached(step: GridStep, index: usize) -> usize {
        step.from(index, COUNT, COLUMNS)
    }

    #[test]
    fn steps_reach_the_neighbours_in_the_row_and_in_the_column() {
        assert_eq!(reached(GridStep::Previous, 4), 3);
        assert_eq!(reached(GridStep::Next, 4), 5);
        assert_eq!(reached(GridStep::Above, 4), 1);
        assert_eq!(reached(GridStep::Below, 3), 6);
    }

    #[test]
    fn steps_stop_where_the_grid_ends() {
        assert_eq!(reached(GridStep::Previous, 0), 0);
        assert_eq!(reached(GridStep::Next, 6), 6);
        assert_eq!(reached(GridStep::Above, 2), 2);
        assert_eq!(reached(GridStep::Below, 4), 4);
    }
}
