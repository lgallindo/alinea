use crate::text_index::{byte_to_char_index, char_to_byte_index};

const CURSOR_MARKER: char = '\u{E000}';

pub fn reflow_text(text: &str, width: usize) -> String {
    if width == 0 || text.is_empty() {
        return text.to_owned();
    }

    let mut lines = Vec::new();
    let mut paragraph = Vec::new();
    let ends_with_newline = text.ends_with('\n');

    for line in text.lines() {
        if line.trim().is_empty() {
            flush_paragraph(&mut paragraph, width, &mut lines);
            lines.push(String::new());
        } else {
            paragraph.push(line.trim().to_owned());
        }
    }

    flush_paragraph(&mut paragraph, width, &mut lines);

    let mut reflowed = lines.join("\n");
    if ends_with_newline {
        reflowed.push('\n');
    }

    reflowed
}

pub fn reflow_text_with_cursor(text: &str, cursor_char: usize, width: usize) -> (String, usize) {
    let marker_byte = char_to_byte_index(text, cursor_char);
    let mut marked = String::with_capacity(text.len() + CURSOR_MARKER.len_utf8());
    marked.push_str(&text[..marker_byte]);
    marked.push(CURSOR_MARKER);
    marked.push_str(&text[marker_byte..]);

    let reflowed = reflow_text(&marked, width);
    let Some(marker_byte) = reflowed.find(CURSOR_MARKER) else {
        return (reflow_text(text, width), cursor_char);
    };

    let cursor = byte_to_char_index(&reflowed, marker_byte);
    let mut cleaned = reflowed;
    cleaned.remove(marker_byte);
    (cleaned, cursor)
}

fn flush_paragraph(paragraph: &mut Vec<String>, width: usize, lines: &mut Vec<String>) {
    if paragraph.is_empty() {
        return;
    }

    let joined = paragraph.join(" ");
    lines.extend(wrap_paragraph(&joined, width));
    paragraph.clear();
}

fn wrap_paragraph(paragraph: &str, width: usize) -> Vec<String> {
    let mut wrapped = Vec::new();
    let mut line = String::new();

    for word in paragraph.split_whitespace() {
        if line.is_empty() {
            line.push_str(word);
            continue;
        }

        let proposed_len = line.chars().count() + 1 + word.chars().count();
        if proposed_len <= width {
            line.push(' ');
            line.push_str(word);
        } else {
            wrapped.push(line);
            line = word.to_owned();
        }
    }

    if !line.is_empty() {
        wrapped.push(line);
    }

    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_short_lines_alone() {
        assert_eq!(reflow_text("short line", 80), "short line");
    }

    #[test]
    fn wraps_long_plain_prose() {
        let text = "alpha beta gamma delta";

        assert_eq!(reflow_text(text, 12), "alpha beta\ngamma delta");
    }

    #[test]
    fn treats_blank_lines_as_paragraph_boundaries() {
        let text = "alpha beta gamma\n\none two three";

        assert_eq!(reflow_text(text, 11), "alpha beta\ngamma\n\none two\nthree");
    }

    #[test]
    fn keeps_long_words_intact() {
        assert_eq!(
            reflow_text("supercalifragilistic prose", 8),
            "supercalifragilistic\nprose"
        );
    }

    #[test]
    fn preserves_trailing_newline() {
        assert_eq!(reflow_text("alpha beta\n", 5), "alpha\nbeta\n");
    }

    #[test]
    fn uses_custom_columns() {
        assert_eq!(reflow_text("one two three four", 7), "one two\nthree\nfour");
        assert_eq!(reflow_text("one two three four", 20), "one two three four");
    }

    #[test]
    fn tracks_cursor_across_reflow() {
        let text = "alpha beta gamma";
        let (wrapped, cursor) = reflow_text_with_cursor(text, 12, 10);

        assert_eq!(wrapped, "alpha beta\ngamma");
        assert_eq!(
            &wrapped[..char_to_byte_index(&wrapped, cursor)],
            "alpha beta\ng"
        );
    }
}
