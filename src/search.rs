use std::ops::Range;

use crate::text_index::{byte_to_char_index, char_to_byte_index};

pub fn find_next(text: &str, needle: &str, start_char: usize) -> Option<Range<usize>> {
    if needle.is_empty() {
        return None;
    }

    let start_byte = char_to_byte_index(text, start_char);
    find_from_byte(text, needle, start_byte).or_else(|| find_from_byte(text, needle, 0))
}

pub fn find_previous(text: &str, needle: &str, start_char: usize) -> Option<Range<usize>> {
    if needle.is_empty() {
        return None;
    }

    let start_byte = char_to_byte_index(text, start_char);
    let before = &text[..start_byte.min(text.len())];

    before
        .match_indices(needle)
        .last()
        .or_else(|| text.match_indices(needle).last())
        .map(|(byte_start, matched)| {
            let start = byte_to_char_index(text, byte_start);
            let end = start + matched.chars().count();
            start..end
        })
}

pub fn count_matches(text: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }

    text.matches(needle).count()
}

fn find_from_byte(text: &str, needle: &str, start_byte: usize) -> Option<Range<usize>> {
    let byte_start = start_byte.min(text.len());
    text[byte_start..].find(needle).map(|relative| {
        let match_start = byte_start + relative;
        let start = byte_to_char_index(text, match_start);
        let end = start + needle.chars().count();
        start..end
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_next_match_from_cursor() {
        let text = "alpha beta alpha";

        assert_eq!(find_next(text, "alpha", 1), Some(11..16));
        assert_eq!(find_next(text, "alpha", 11), Some(11..16));
        assert_eq!(find_next(text, "alpha", 16), Some(0..5));
    }

    #[test]
    fn finds_previous_match_from_cursor() {
        let text = "alpha beta alpha";

        assert_eq!(find_previous(text, "alpha", 16), Some(11..16));
        assert_eq!(find_previous(text, "alpha", 11), Some(0..5));
        assert_eq!(find_previous(text, "alpha", 0), Some(11..16));
    }

    #[test]
    fn reports_no_match_for_missing_or_empty_query() {
        assert_eq!(find_next("abc", "z", 0), None);
        assert_eq!(find_previous("abc", "", 0), None);
        assert_eq!(count_matches("abc", ""), 0);
    }

    #[test]
    fn handles_unicode_literals() {
        let text = "café cafe café";

        assert_eq!(find_next(text, "café", 1), Some(10..14));
        assert_eq!(count_matches(text, "café"), 2);
    }
}
