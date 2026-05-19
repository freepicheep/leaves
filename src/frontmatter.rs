const FRONTMATTER_VERTICAL_THRESHOLD: usize = 5;

pub(crate) fn extract_frontmatter(src: &str) -> (&str, Option<Vec<(String, String)>>) {
    let Some(rest) = src.strip_prefix("---\n") else {
        return (src, None);
    };

    let mut offset = 4usize;
    for line in rest.split_inclusive('\n') {
        if line == "---\n" || line == "...\n" || line == "---" || line == "..." {
            let fm_block = &src[4..offset];
            let content = &src[offset + line.len()..];
            let pairs = parse_pairs(fm_block);
            if pairs.is_empty() {
                return (content, None);
            }
            return (content, Some(pairs));
        }
        offset += line.len();
    }

    (src, None)
}

pub(crate) fn is_vertical(pairs: &[(String, String)]) -> bool {
    pairs.len() >= FRONTMATTER_VERTICAL_THRESHOLD
}

fn parse_pairs(block: &str) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    let lines: Vec<&str> = block.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            i += 1;
            continue;
        }

        let Some(colon_pos) = trimmed.find(':') else {
            i += 1;
            continue;
        };

        let key = trimmed[..colon_pos].trim().to_string();
        let raw_value = trimmed[colon_pos + 1..].trim();

        if key.is_empty() {
            i += 1;
            continue;
        }

        if raw_value == ">-" || raw_value == ">" || raw_value == "|" || raw_value == "|-" {
            let mut parts: Vec<&str> = Vec::new();
            i += 1;
            while i < lines.len() && is_indented(lines[i]) {
                let part = lines[i].trim();
                if !part.is_empty() {
                    parts.push(part);
                }
                i += 1;
            }
            pairs.push((key, parts.join(" ")));
        } else if raw_value.is_empty() {
            let mut items: Vec<&str> = Vec::new();
            i += 1;
            while i < lines.len() && is_indented(lines[i]) {
                let part = lines[i].trim();
                if let Some(item) = part.strip_prefix("- ") {
                    items.push(item.trim());
                } else if !part.is_empty() {
                    items.push(part);
                }
                i += 1;
            }
            if items.is_empty() {
                pairs.push((key, String::new()));
            } else {
                pairs.push((key, items.join(", ")));
            }
        } else {
            pairs.push((key, strip_quotes(raw_value).to_string()));
            i += 1;
        }
    }

    pairs
}

fn is_indented(line: &str) -> bool {
    line.starts_with(' ') || line.starts_with('\t')
}

fn strip_quotes(s: &str) -> &str {
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        return &s[1..s.len() - 1];
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_returns_source_unchanged() {
        let src = "# Hello\n\nNot a frontmatter doc.";
        let (rest, pairs) = extract_frontmatter(src);
        assert_eq!(rest, src);
        assert!(pairs.is_none());
    }

    #[test]
    fn missing_closing_delimiter_returns_source_unchanged() {
        let src = "---\ntitle: Hello\n\nNo close marker";
        let (rest, pairs) = extract_frontmatter(src);
        assert_eq!(rest, src);
        assert!(pairs.is_none());
    }

    #[test]
    fn empty_frontmatter_block_returns_no_pairs() {
        let src = "---\n---\nbody";
        let (rest, pairs) = extract_frontmatter(src);
        assert_eq!(rest, "body");
        assert!(pairs.is_none());
    }

    #[test]
    fn parses_simple_key_value_pairs() {
        let src = "---\ntitle: Hello\nauthor: Ada\n---\nbody\n";
        let (rest, pairs) = extract_frontmatter(src);
        assert_eq!(rest, "body\n");
        let pairs = pairs.expect("pairs present");
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("title".to_string(), "Hello".to_string()));
        assert_eq!(pairs[1], ("author".to_string(), "Ada".to_string()));
    }

    #[test]
    fn dot_dot_dot_terminates_frontmatter() {
        let src = "---\ntitle: Hello\n...\nbody";
        let (rest, pairs) = extract_frontmatter(src);
        assert_eq!(rest, "body");
        assert_eq!(pairs.unwrap()[0].1, "Hello");
    }

    #[test]
    fn strips_quotes_around_values() {
        let src = "---\ntitle: \"Quoted\"\nname: 'Single'\n---\n";
        let (_rest, pairs) = extract_frontmatter(src);
        let pairs = pairs.unwrap();
        assert_eq!(pairs[0].1, "Quoted");
        assert_eq!(pairs[1].1, "Single");
    }

    #[test]
    fn folded_block_scalar_joins_lines() {
        let src = "---\nsummary: >-\n  line one\n  line two\n---\n";
        let (_rest, pairs) = extract_frontmatter(src);
        let pairs = pairs.unwrap();
        assert_eq!(pairs[0].0, "summary");
        assert_eq!(pairs[0].1, "line one line two");
    }

    #[test]
    fn list_values_join_with_comma() {
        let src = "---\ntags:\n  - rust\n  - cli\n  - tui\n---\n";
        let (_rest, pairs) = extract_frontmatter(src);
        let pairs = pairs.unwrap();
        assert_eq!(pairs[0].0, "tags");
        assert_eq!(pairs[0].1, "rust, cli, tui");
    }

    #[test]
    fn empty_value_with_no_indented_items_produces_empty_string() {
        let src = "---\nempty:\nnext: value\n---\n";
        let (_rest, pairs) = extract_frontmatter(src);
        let pairs = pairs.unwrap();
        assert_eq!(pairs[0], ("empty".to_string(), String::new()));
        assert_eq!(pairs[1], ("next".to_string(), "value".to_string()));
    }

    #[test]
    fn comments_and_blank_lines_are_skipped() {
        let src = "---\n# a comment\n\ntitle: Hi\n# trailing\n---\n";
        let (_rest, pairs) = extract_frontmatter(src);
        let pairs = pairs.unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].1, "Hi");
    }

    #[test]
    fn lines_without_colon_or_empty_key_are_skipped() {
        let src = "---\nnocolon line\n: empty-key\ngood: value\n---\n";
        let (_rest, pairs) = extract_frontmatter(src);
        let pairs = pairs.unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("good".to_string(), "value".to_string()));
    }

    #[test]
    fn is_vertical_threshold_is_five() {
        let four: Vec<_> = (0..4).map(|i| (format!("k{i}"), "v".to_string())).collect();
        let five: Vec<_> = (0..5).map(|i| (format!("k{i}"), "v".to_string())).collect();
        assert!(!is_vertical(&four));
        assert!(is_vertical(&five));
    }
}
