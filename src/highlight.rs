use crate::theme::MarkdownTheme;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::{ParseSyntaxError, SyntaxDefinition, SyntaxSet},
    LoadingError,
};

/// Builds a `SyntaxSet` from syntect's defaults plus syntax definitions bundled
/// with leaves.
///
/// This includes Nushell support for fenced code blocks tagged as `nu` or
/// `nushell`.
pub fn syntax_set_with_bundled_syntaxes() -> Result<SyntaxSet, ParseSyntaxError> {
    let nushell = SyntaxDefinition::load_from_str(
        include_str!("../assets/syntaxes/nushell.sublime-syntax"),
        true,
        Some("nushell"),
    )?;
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    builder.add(nushell);
    Ok(builder.build())
}

/// Builds a `ThemeSet` from syntect's defaults plus themes bundled with leaves.
///
/// This currently includes bat's `ansi` theme for terminal-palette syntax
/// highlighting.
pub fn theme_set_with_bundled_themes() -> Result<ThemeSet, LoadingError> {
    let mut themes = ThemeSet::load_defaults();
    let mut ansi = std::io::Cursor::new(include_bytes!("../assets/themes/ansi.tmTheme"));
    themes
        .themes
        .insert("ansi".to_string(), ThemeSet::load_from_reader(&mut ansi)?);
    Ok(themes)
}

/// Applies search-highlight styling (background color) to every span in a line.
///
/// This is useful for highlighting the currently-active search match in a
/// markdown viewer.
pub fn highlight_line<'a>(line: &Line<'a>, theme: &MarkdownTheme) -> Line<'a> {
    Line::from(
        line.spans
            .iter()
            .map(|span| {
                Span::styled(
                    span.content.clone(),
                    span.style.bg(theme.search_highlight_bg),
                )
            })
            .collect::<Vec<_>>(),
    )
}

/// Resolves a code-block language tag to a syntect `SyntaxReference`.
///
/// Handles common aliases (e.g. `ts` → TypeScript, `py` → Python, `sh` → Bash)
/// and falls back to plain text when no match is found.
pub fn resolve_syntax<'a>(lang: &str, ss: &'a SyntaxSet) -> &'a syntect::parsing::SyntaxReference {
    let raw = lang.trim();
    let normalized = raw
        .split(|c: char| c.is_whitespace() || c == ',' || c == '{')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    let aliases: &[&str] = match normalized.as_str() {
        "ts" | "typescript" => &[
            "JavaScript",
            "js",
            "javascript",
            "TypeScript",
            "ts",
            "typescript",
        ],
        "tsx" => &["JSX", "jsx", "JavaScript", "js", "typescriptreact", "tsx"],
        "js" | "javascript" => &["JavaScript", "js", "javascript"],
        "jsx" => &["JSX", "jsx", "JavaScript React"],
        "shell" | "bash" | "sh" | "zsh" => &["Bourne Again Shell (bash)", "bash", "sh"],
        "py" | "python" => &["Python", "py", "python"],
        "c" => &["C", "c"],
        "cpp" | "cxx" | "cc" | "c++" => &["C++", "cpp", "cxx", "cc"],
        "json" => &["JSON", "json"],
        "toml" => &["TOML", "toml"],
        "java" => &["Java", "java"],
        "kt" | "kotlin" => &["Kotlin", "kt", "kotlin"],
        "ps1" | "powershell" | "pwsh" => &["PowerShell", "ps1", "powershell"],
        "nu" | "nushell" => &["nushell", "Nushell", "Nu", "nu"],
        "docker" | "dockerfile" => &["Dockerfile", "dockerfile"],
        "yml" | "yaml" => &["YAML", "yml", "yaml"],
        "rs" | "rust" => &["Rust", "rs", "rust"],
        _ if normalized.is_empty() => &[],
        _ => &[],
    };

    ss.find_syntax_by_token(raw)
        .or_else(|| ss.find_syntax_by_extension(raw))
        .or_else(|| ss.find_syntax_by_token(&normalized))
        .or_else(|| ss.find_syntax_by_extension(&normalized))
        .or_else(|| {
            aliases.iter().find_map(|alias| {
                ss.find_syntax_by_token(alias)
                    .or_else(|| ss.find_syntax_by_extension(alias))
                    .or_else(|| ss.find_syntax_by_name(alias))
            })
        })
        .unwrap_or_else(|| ss.find_syntax_plain_text())
}

pub(crate) fn syntect_to_color(c: syntect::highlighting::Color) -> ratatui::style::Color {
    use ratatui::style::Color;

    match (c.r, c.g, c.b, c.a) {
        // bat's ANSI/base16 tmThemes encode terminal palette indexes as
        // #RRGGBB00, where RR is the ANSI palette number.
        (0, 0, 0, 0) => Color::Black,
        (1, 0, 0, 0) => Color::Red,
        (2, 0, 0, 0) => Color::Green,
        (3, 0, 0, 0) => Color::Yellow,
        (4, 0, 0, 0) => Color::Blue,
        (5, 0, 0, 0) => Color::Magenta,
        (6, 0, 0, 0) => Color::Cyan,
        (7, 0, 0, 0) => Color::White,
        (8, 0, 0, 0) => Color::DarkGray,
        (9, 0, 0, 0) => Color::LightRed,
        (10, 0, 0, 0) => Color::LightGreen,
        (11, 0, 0, 0) => Color::LightYellow,
        (12, 0, 0, 0) => Color::LightBlue,
        (13, 0, 0, 0) => Color::LightMagenta,
        (14, 0, 0, 0) => Color::LightCyan,
        (15, 0, 0, 0) => Color::Gray,
        // bat uses #00000001 for the terminal default foreground/background.
        (_, _, _, 1) => Color::Reset,
        _ => Color::Rgb(c.r, c.g, c.b),
    }
}

pub(crate) struct CodeLine {
    pub(crate) content_spans: Vec<Span<'static>>,
}

pub(crate) fn highlight_code(
    code: &str,
    lang: &str,
    ss: &SyntaxSet,
    theme: &Theme,
    render_width: usize,
) -> (Vec<CodeLine>, usize, usize) {
    use crate::width::{display_width, expand_tabs};
    use syntect::util::LinesWithEndings;
    use unicode_width::UnicodeWidthStr;

    let syntax = resolve_syntax(lang, ss);
    let mut hl = HighlightLines::new(syntax, theme);

    let mut raw: Vec<(Vec<Span<'static>>, usize)> = Vec::new();
    for line_str in LinesWithEndings::from(code) {
        let regions = hl.highlight_line(line_str, ss).unwrap_or_default();
        let mut spans = Vec::new();
        let mut text_width: usize = 0;
        for (st, text) in &regions {
            let t = expand_tabs(text.trim_end_matches('\n'), text_width);
            if t.is_empty() {
                continue;
            }
            text_width += display_width(&t);
            let mut rs = Style::default().fg(syntect_to_color(st.foreground));
            if st
                .font_style
                .contains(syntect::highlighting::FontStyle::BOLD)
            {
                rs = rs.add_modifier(Modifier::BOLD);
            }
            if st
                .font_style
                .contains(syntect::highlighting::FontStyle::ITALIC)
            {
                rs = rs.add_modifier(Modifier::ITALIC);
            }
            if st
                .font_style
                .contains(syntect::highlighting::FontStyle::UNDERLINE)
            {
                rs = rs.add_modifier(Modifier::UNDERLINED);
            }
            spans.push(Span::styled(t, rs));
        }
        raw.push((spans, text_width));
    }

    let label = if lang.is_empty() { "text" } else { lang };
    let total_lines = raw.len();
    let digit_width = total_lines.max(1).to_string().len();
    let gutter_width = digit_width + 2;
    let max_text = raw.iter().map(|(_, w)| *w).max().unwrap_or(0);
    let max_inner_width = render_width
        .saturating_sub(4)
        .max(UnicodeWidthStr::width(label) + 3);
    let min_inner = (UnicodeWidthStr::width(label) + 3)
        .max(44)
        .min(max_inner_width);
    let inner_width = (max_text + 2 + gutter_width)
        .max(min_inner)
        .min(max_inner_width);

    let mut out = Vec::new();
    for (spans, _text_width) in raw {
        out.push(CodeLine {
            content_spans: spans,
        });
    }
    (out, inner_width, digit_width)
}

#[cfg(test)]
mod tests {
    use super::{
        highlight_code, resolve_syntax, syntax_set_with_bundled_syntaxes, syntect_to_color,
        theme_set_with_bundled_themes,
    };
    use ratatui::style::Color as RatatuiColor;
    use syntect::highlighting::Color as SyntectColor;

    #[test]
    fn bundled_syntax_set_resolves_nushell_aliases() {
        let ss = syntax_set_with_bundled_syntaxes().expect("bundled syntax should load");

        assert_eq!(resolve_syntax("nu", &ss).name, "nushell");
        assert_eq!(resolve_syntax("nushell", &ss).name, "nushell");
    }

    #[test]
    fn bundled_theme_set_includes_bat_ansi_theme() {
        let themes = theme_set_with_bundled_themes().expect("bundled themes should load");

        assert!(themes.themes.contains_key("ansi"));
    }

    #[test]
    fn nushell_highlighting_distinguishes_custom_commands_from_arguments() {
        let ss = syntax_set_with_bundled_syntaxes().expect("bundled syntax should load");
        let themes = theme_set_with_bundled_themes().expect("bundled themes should load");
        let theme = &themes.themes["ansi"];

        let (lines, _, _) = highlight_code(
            "use /path/to/nu-salesforce *\nsf query 'SELECT Id FROM Account LIMIT 10'\nlet x = $env.SALESFORCE_USERNAME\n",
            "nu",
            &ss,
            theme,
            80,
        );
        let spans = lines
            .iter()
            .flat_map(|line| line.content_spans.iter())
            .collect::<Vec<_>>();

        let sf = spans
            .iter()
            .find(|span| span.content.as_ref() == "sf")
            .expect("custom command should be highlighted");
        let query = spans
            .iter()
            .find(|span| span.content.as_ref() == "query")
            .expect("custom subcommand should be highlighted");
        let sql = spans
            .iter()
            .find(|span| span.content.contains("SELECT Id"))
            .expect("quoted SQL should be highlighted as a string");
        let let_keyword = spans
            .iter()
            .find(|span| span.content.as_ref() == "let")
            .expect("let should remain highlighted as a Nushell declaration");
        let wildcard = spans
            .iter()
            .find(|span| span.content.as_ref() == "*")
            .expect("use wildcard import should be highlighted");
        let env_var = spans
            .iter()
            .find(|span| span.content.as_ref() == "SALESFORCE_USERNAME")
            .expect("environment variable member should be highlighted");

        assert_eq!(sf.style.fg, Some(RatatuiColor::Blue));
        assert_eq!(query.style.fg, Some(RatatuiColor::Cyan));
        assert_eq!(sql.style.fg, Some(RatatuiColor::Green));
        assert_eq!(let_keyword.style.fg, Some(RatatuiColor::Magenta));
        assert_eq!(wildcard.style.fg, Some(RatatuiColor::Magenta));
        assert_eq!(env_var.style.fg, Some(RatatuiColor::Cyan));
    }

    #[test]
    fn bat_ansi_palette_colors_are_mapped_to_terminal_colors() {
        assert_eq!(
            syntect_to_color(SyntectColor {
                r: 1,
                g: 0,
                b: 0,
                a: 0
            }),
            RatatuiColor::Red
        );
        assert_eq!(
            syntect_to_color(SyntectColor {
                r: 0,
                g: 0,
                b: 0,
                a: 1
            }),
            RatatuiColor::Reset
        );
    }
}
