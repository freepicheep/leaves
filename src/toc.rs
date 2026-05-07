/// A single entry in the table of contents extracted from parsed markdown.
#[derive(Clone, Debug)]
pub struct TocEntry {
    /// The heading level (1–6).
    pub level: u8,
    /// The plain-text title of the heading.
    pub title: String,
    /// The zero-based line index in the rendered output where this heading appears.
    pub line: usize,
}

/// Returns `true` when the TOC contains exactly one H1 and at least one H2,
/// meaning the lone H1 should be hidden from sidebar displays.
pub fn should_hide_single_h1(toc: &[TocEntry]) -> bool {
    let h1_count = toc.iter().filter(|entry| entry.level == 1).count();
    let has_h2 = toc.iter().any(|entry| entry.level == 2);
    h1_count == 1 && has_h2
}

/// Returns `true` when there are no H1 headings but at least one H2,
/// meaning H2s should be promoted to root level in displays.
pub fn should_promote_h2_when_no_h1(toc: &[TocEntry]) -> bool {
    !toc.iter().any(|entry| entry.level == 1) && toc.iter().any(|entry| entry.level == 2)
}

/// Maps a raw heading level to a display level, applying promotion rules.
pub fn toc_display_level(level: u8, hide_single_h1: bool, promote_h2_root: bool) -> u8 {
    if hide_single_h1 || promote_h2_root {
        match level {
            2 => 1,
            3 => 2,
            _ => level,
        }
    } else {
        level
    }
}

/// Normalizes a TOC by filtering to relevant heading levels and applying
/// H1/H2 promotion rules.
pub fn normalize_toc(mut toc: Vec<TocEntry>) -> Vec<TocEntry> {
    if should_hide_single_h1(&toc) || should_promote_h2_when_no_h1(&toc) {
        toc.retain(|entry| matches!(entry.level, 1..=3));
    } else {
        toc.retain(|entry| matches!(entry.level, 1..=2));
    }
    toc
}
