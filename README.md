# Leaves

`leaves` is a Markdown-to-Ratatui rendering library extracted from the [leaf](https://github.com/RivoLink/leaf) TUI project.
It provides a robust rendering pipeline that converts Markdown text directly into Ratatui `Line` objects.

## Features

- **Rich Markdown Elements**: Renders standard Markdown features like bold, italics, strikethrough, lists, and blockquotes.
- **Syntax Highlighting**: Uses `syntect` to parse code blocks and inline code with proper highlighting. Includes bat's `ansi` theme for terminal-palette code highlighting.
- **Tables**: Parses and visually structures Markdown tables.
- **Special Integrations**:
  - **LaTeX**: Converts simple LaTeX mathematical expressions into Unicode (via `unicodeit`).
  - **Mermaid**: Renders basic mermaid diagrams directly inside the terminal.
- **Table of Contents**: Extracts an organized list of headers, which you can easily use to build a sidebar navigation component.
- **Themable**: Includes built-in themes (`OCEAN_DARK`, `ARCTIC`, `FOREST`, `SOLARIZED_DARK`, `TERMINAL`) and exposes an accessible `MarkdownTheme` struct for full customization.

## Usage

Add `leaves` to your Ratatui project:

```toml
[dependencies]
leaves = { path = "path/to/leaves" }
```

In your application code:

```rust
use leaves::{
    parse_markdown, syntax_set_with_bundled_syntaxes, theme_set_with_bundled_themes, MarkdownTheme,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize syntect resources
    let ss = syntax_set_with_bundled_syntaxes()?;
    let ts = theme_set_with_bundled_themes()?;
    let syntect_theme = &ts.themes["base16-ocean.dark"];

    // 2. Choose a markdown theme (e.g., Ocean Dark)
    let md_theme = MarkdownTheme::default(); 

    // 3. Parse the markdown text
    let markdown_text = "# Hello\n\nSome **bold** text and `inline code`!";
    let (lines, toc) = parse_markdown(
        markdown_text,
        &ss,
        syntect_theme,
        &md_theme,
    );

    // `lines` is a `Vec<Line<'static>>` that you can directly render in Ratatui:
    // e.g., ratatui::widgets::Paragraph::new(lines).render(area, buf);

    // `toc` is a `Vec<TocEntry>` that contains the headings and their line numbers
    for entry in toc {
        println!("Heading '{}' is at level {} and starts on line {}", entry.title, entry.level, entry.line);
    }

    Ok(())
}
```

For terminal-palette code highlighting, use the bundled bat ANSI theme together
with Leaves' terminal markdown theme:

```rust
use leaves::{
    parse_markdown, syntax_set_with_bundled_syntaxes, theme::TERMINAL,
    theme_set_with_bundled_themes,
};

let ss = syntax_set_with_bundled_syntaxes()?;
let ts = theme_set_with_bundled_themes()?;
let syntect_theme = &ts.themes["ansi"];

let (lines, toc) = parse_markdown(markdown_text, &ss, syntect_theme, &TERMINAL);
```

## Structure

- `src/theme.rs`: Contains the `MarkdownTheme` and preset themes.
- `src/parse.rs`: Core engine powering `parse_markdown()`.
- `src/toc.rs`: Data structure for handling headings.
- `src/highlight.rs`: Syntax highlighting helpers.
- `themes/ansi.tmTheme`: Bundled bat ANSI theme for terminal-palette code highlighting.
- `src/width.rs`: Display width utilities.

## Credit

To the [leaf](https://github.com/RivoLink/leaf) project for making this possible. `leaves` is really just an abstraction fo the work they did there so other tools can enjoy nice Markdown rendering in the terminal.

Each of these wonderful crates:
- `ratatui`
- `pulldown-cmark`
- `syntect`
- `unicode-width`
- `unicodeit`
- `mmdflux`

And...
- `bat` for providing the custom `syntect` themes.
- [nushell_sublime_syntax ](https://github.com/kurokirasama/nushell_sublime_syntax) for Nu syntax highlighting
