use ratatui::text::Line;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(crate) const TAB_STOP: usize = 4;

/// Extracts the plain text from a rendered ratatui `Line` by concatenating all span contents.
pub fn line_plain_text(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

fn is_code_gutter_span(content: &str) -> bool {
    let inner = content.strip_prefix('│').and_then(|s| s.strip_suffix('│'));
    match inner {
        Some(s) if !s.is_empty() => {
            let mut has_digit = false;
            for b in s.bytes() {
                if b.is_ascii_digit() {
                    has_digit = true;
                } else if b != b' ' {
                    return false;
                }
            }
            has_digit
        }
        _ => false,
    }
}

/// Extracts the searchable text from a rendered line, stripping code-block
/// gutter decorations so that searches match only the content.
pub fn line_searchable_text(line: &Line<'_>) -> String {
    let spans = &line.spans;
    let has_pipe = spans.iter().take(4).any(|s| s.content.contains('│'));
    if !has_pipe {
        return line_plain_text(line);
    }
    let mut gutter_end = None;
    for (i, span) in spans.iter().enumerate() {
        if is_code_gutter_span(span.content.as_ref()) {
            gutter_end = Some(i);
            break;
        }
    }
    let Some(ge) = gutter_end else {
        return line_plain_text(line);
    };
    let last = spans.len().saturating_sub(1);
    let start = ge + 1;
    if start > last {
        return String::new();
    }
    let end = if last > start && spans[last].content.as_ref() == "│" {
        last
    } else {
        spans.len()
    };
    spans[start..end]
        .iter()
        .map(|s| s.content.as_ref())
        .collect()
}

/// Builds a list of searchable text for each rendered line.
pub fn build_searchable_lines(lines: &[Line<'_>]) -> Vec<String> {
    lines.iter().map(line_searchable_text).collect()
}

/// Truncates a string to fit within `max_width` display columns, appending
/// an ellipsis (`…`) if truncation occurs.
pub fn truncate_display_width(text: &str, max_width: usize) -> String {
    if display_width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }

    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + ch_w > max_width.saturating_sub(1) {
            break;
        }
        out.push(ch);
        used += ch_w;
    }
    out.push('\u{2026}');
    out
}

/// Computes the display width of a string in terminal columns, treating tabs
/// as stops every 4 columns.
pub fn display_width(text: &str) -> usize {
    let mut width = 0;
    let mut parts = text.split('\t').peekable();
    while let Some(segment) = parts.next() {
        width += UnicodeWidthStr::width(segment);
        if parts.peek().is_some() {
            width += TAB_STOP - (width % TAB_STOP);
        }
    }
    width
}

/// Expands tab characters to spaces, aligning to 4-column tab stops
/// relative to the given `start_width`.
pub(crate) fn expand_tabs(text: &str, start_width: usize) -> String {
    if !text.contains('\t') {
        return text.to_string();
    }

    let mut out = String::new();
    let mut width = start_width;
    let mut parts = text.split('\t').peekable();
    while let Some(segment) = parts.next() {
        out.push_str(segment);
        width += UnicodeWidthStr::width(segment);
        if parts.peek().is_some() {
            let spaces = TAB_STOP - (width % TAB_STOP);
            out.push_str(&" ".repeat(spaces));
            width += spaces;
        }
    }
    out
}
