#![allow(dead_code)]

use leaves::theme::MarkdownTheme;
use leaves::width::line_plain_text;
use ratatui::backend::TestBackend;
use ratatui::{text::Line, widgets::Paragraph, Terminal};
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
};

pub fn test_assets() -> (SyntaxSet, Theme) {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = ts.themes["base16-ocean.dark"].clone();
    (ss, theme)
}

pub fn test_md_theme() -> MarkdownTheme {
    MarkdownTheme::default()
}

pub fn render_buffer(lines: &[Line<'static>]) -> ratatui::buffer::Buffer {
    let width = lines
        .iter()
        .map(|line| line.width())
        .max()
        .unwrap_or(1)
        .max(1) as u16;
    let height = lines.len().max(1) as u16;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            f.render_widget(Paragraph::new(lines.to_vec()), f.area());
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

pub fn find_symbol(buffer: &ratatui::buffer::Buffer, symbol: &str) -> Option<(u16, u16)> {
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            if buffer
                .cell((x, y))
                .is_some_and(|cell| cell.symbol() == symbol)
            {
                return Some((x, y));
            }
        }
    }
    None
}

pub fn rendered_non_empty_lines(lines: &[Line<'static>]) -> Vec<String> {
    lines
        .iter()
        .map(line_plain_text)
        .filter(|line| !line.is_empty())
        .collect()
}
