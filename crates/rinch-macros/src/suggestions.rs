//! Suggestion utilities for helpful error messages.
//!
//! Provides Levenshtein distance calculation for suggesting
//! corrections when users make typos in property names.

/// Calculate the Levenshtein distance between two strings.
///
/// This is the minimum number of single-character edits (insertions,
/// deletions, or substitutions) required to change one string into the other.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // Handle empty strings
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Create distance matrix
    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    // Initialize first column
    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }

    // Initialize first row
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    // Fill in the rest of the matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = (matrix[i - 1][j] + 1) // deletion
                .min(matrix[i][j - 1] + 1) // insertion
                .min(matrix[i - 1][j - 1] + cost); // substitution
        }
    }

    matrix[a_len][b_len]
}

/// Find the closest matching property name from a list of valid names.
///
/// Returns `Some(suggestion)` if a close match is found (distance <= 3),
/// `None` otherwise.
pub fn find_closest_prop(unknown: &str, valid: &[&str]) -> Option<String> {
    valid
        .iter()
        .map(|&p| (p, levenshtein_distance(unknown, p)))
        .filter(|(_, d)| *d <= 3) // Only suggest if reasonably close
        .min_by_key(|(_, d)| *d)
        .map(|(name, _)| name.to_string())
}

/// Format a helpful error message for an unknown property.
pub fn format_unknown_prop_error(component: &str, unknown_prop: &str, valid_props: &[&str]) -> String {
    let mut msg = format!(
        "unknown property `{}` for `{}` component",
        unknown_prop, component
    );

    if let Some(suggestion) = find_closest_prop(unknown_prop, valid_props) {
        msg.push_str(&format!("\n\nDid you mean `{}`?", suggestion));
    }

    if !valid_props.is_empty() {
        msg.push_str("\n\nValid properties are: ");
        msg.push_str(&valid_props.join(", "));
    }

    msg
}

/// Format a helpful error message for a missing required property.
pub fn format_missing_prop_error(component: &str, missing_prop: &str) -> String {
    format!(
        "missing required property `{}` for `{}` component",
        missing_prop, component
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
    }

    #[test]
    fn test_levenshtein_same() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_one_char() {
        assert_eq!(levenshtein_distance("hello", "hallo"), 1); // substitution
        assert_eq!(levenshtein_distance("hello", "hell"), 1); // deletion
        assert_eq!(levenshtein_distance("hello", "helloo"), 1); // insertion
    }

    #[test]
    fn test_find_closest() {
        let valid = vec!["title", "width", "height", "visible"];
        assert_eq!(find_closest_prop("titl", &valid), Some("title".into()));
        assert_eq!(find_closest_prop("widht", &valid), Some("width".into()));
        assert_eq!(find_closest_prop("xyz", &valid), None);
    }
}
