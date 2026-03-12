//! CSS language extractor.
//!
//! Extracts custom properties (`--var-name`) and class/id selectors.

use crate::extract::{Extracted, Extractor, ExtractorState};

pub struct CssExtractor;

impl Extractor for CssExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in var_names(line) {
            out.variables.push(name.to_string());
        }
        for name in selector_names(line) {
            out.functions.push(name.to_string());
        }

        out
    }
}

/// Extract CSS custom properties (`--var-name`).
fn var_names(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(pos) = line[start..].find("--") {
        let abs = start + pos;
        let rest = &line[abs + 2..];
        let end = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
            .unwrap_or(rest.len());
        if end > 0 {
            out.push(&line[abs..abs + 2 + end]);
        }
        start = abs + 2 + end;
    }
    out
}

/// Extract CSS class (`.class`) and id (`#id`) selectors.
fn selector_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    // Skip property declaration lines (not selector lines).
    if trimmed.contains(':')
        && !trimmed.ends_with('{')
        && !trimmed.starts_with('.')
        && !trimmed.starts_with('#')
    {
        return Vec::new();
    }

    let mut out = Vec::new();
    for prefix in ['.', '#'] {
        let mut start = 0;
        while start < line.len() {
            let Some(pos) = line[start..].find(prefix) else {
                break;
            };
            let abs = start + pos;

            // Skip if preceded by an alphanumeric char (part of a value, not a selector).
            if abs > 0 {
                let prev = line.as_bytes()[abs - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'-' {
                    start = abs + 1;
                    continue;
                }
            }

            let rest = &line[abs + 1..];
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
                .unwrap_or(rest.len());
            if end > 0 {
                out.push(&line[abs..abs + 1 + end]);
            }
            start = abs + 1 + end;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::ExtractorState;

    fn extract(line: &str) -> Extracted {
        CssExtractor.extract_line(line, &mut ExtractorState::default())
    }

    #[test]
    fn custom_properties() {
        let e = extract("  --primary-color: #333;");
        assert_eq!(e.variables, vec!["--primary-color"]);
    }

    #[test]
    fn class_selector() {
        let e = extract(".container {");
        assert_eq!(e.functions, vec![".container"]);
    }

    #[test]
    fn id_selector() {
        let e = extract("#main-header {");
        assert_eq!(e.functions, vec!["#main-header"]);
    }
}
