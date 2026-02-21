//! Lists related humanization.

use std::fmt::Display;

/// Convert a list of items into a human-readable string with commas and "and".
///
/// # Examples
/// ```
/// use speakhuman::lists::natural_list;
/// assert_eq!(natural_list(&["one", "two", "three"]), "one, two and three");
/// assert_eq!(natural_list(&["one", "two"]), "one and two");
/// assert_eq!(natural_list(&["one"]), "one");
/// ```
pub fn natural_list<T: Display>(items: &[T]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].to_string(),
        2 => format!("{} and {}", items[0], items[1]),
        _ => {
            let head: Vec<String> = items[..items.len() - 1]
                .iter()
                .map(|i| i.to_string())
                .collect();
            format!("{} and {}", head.join(", "), items[items.len() - 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_list_three() {
        assert_eq!(
            natural_list(&["one", "two", "three"]),
            "one, two and three"
        );
    }

    #[test]
    fn test_natural_list_two() {
        assert_eq!(natural_list(&["one", "two"]), "one and two");
    }

    #[test]
    fn test_natural_list_one() {
        assert_eq!(natural_list(&["one"]), "one");
    }

    #[test]
    fn test_natural_list_numbers() {
        assert_eq!(natural_list(&[1, 2, 3]), "1, 2 and 3");
    }

    #[test]
    fn test_natural_list_empty_string() {
        assert_eq!(natural_list(&[""]), "");
    }
}
