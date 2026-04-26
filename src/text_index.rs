use std::ops::Range;

pub fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

pub fn byte_to_char_index(text: &str, byte_index: usize) -> usize {
    text[..byte_index.min(text.len())].chars().count()
}

pub fn slice_char_range(text: &str, range: Range<usize>) -> &str {
    let start = char_to_byte_index(text, range.start);
    let end = char_to_byte_index(text, range.end);
    &text[start..end]
}

pub fn replace_char_range(text: &mut String, range: Range<usize>, replacement: &str) {
    let start = char_to_byte_index(text, range.start);
    let end = char_to_byte_index(text, range.end);
    text.replace_range(start..end, replacement);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_unicode_indices() {
        let text = "aé日b";

        assert_eq!(char_to_byte_index(text, 0), 0);
        assert_eq!(char_to_byte_index(text, 2), "aé".len());
        assert_eq!(char_to_byte_index(text, 99), text.len());
        assert_eq!(byte_to_char_index(text, "aé".len()), 2);
    }

    #[test]
    fn slices_and_replaces_by_char_range() {
        let mut text = String::from("one café line");

        assert_eq!(slice_char_range(&text, 4..8), "café");
        replace_char_range(&mut text, 4..8, "plain");
        assert_eq!(text, "one plain line");
    }
}
