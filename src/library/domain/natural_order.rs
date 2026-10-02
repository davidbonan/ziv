use std::cmp::Ordering;
use std::iter::Peekable;
use std::str::Chars;

fn take_number(characters: &mut Peekable<Chars<'_>>) -> u128 {
    let mut number = 0u128;
    while let Some(digit) = characters
        .peek()
        .and_then(|character| character.to_digit(10))
    {
        number = number.saturating_mul(10).saturating_add(u128::from(digit));
        characters.next();
    }
    number
}

/// Orders names the way people read them: case-insensitive, digit runs compared
/// as numbers (`DSC2` before `DSC10`).
pub fn natural_order(left: &str, right: &str) -> Ordering {
    let mut left_characters = left.chars().peekable();
    let mut right_characters = right.chars().peekable();
    loop {
        let (Some(&left_next), Some(&right_next)) =
            (left_characters.peek(), right_characters.peek())
        else {
            return left_characters
                .peek()
                .is_some()
                .cmp(&right_characters.peek().is_some());
        };
        let ordering = if left_next.is_ascii_digit() && right_next.is_ascii_digit() {
            take_number(&mut left_characters).cmp(&take_number(&mut right_characters))
        } else {
            left_characters.next();
            right_characters.next();
            left_next.to_lowercase().cmp(right_next.to_lowercase())
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_runs_are_compared_as_numbers() {
        assert_eq!(natural_order("DSC2.ARW", "DSC10.ARW"), Ordering::Less);
        assert_eq!(natural_order("a10b2", "a10b10"), Ordering::Less);
    }

    #[test]
    fn letter_case_is_ignored() {
        assert_eq!(natural_order("b.jpg", "A.jpg"), Ordering::Greater);
        assert_eq!(natural_order("ABC", "abc"), Ordering::Equal);
    }

    #[test]
    fn shorter_name_comes_first_when_it_is_a_prefix() {
        assert_eq!(natural_order("DSC1", "DSC1-edit"), Ordering::Less);
    }
}
