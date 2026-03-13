//! CSS language extractor.
//!
//! Extracts custom properties (`--var-name`) and class/id selectors.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

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
        for name in import_names(line) {
            out.imports.push(name);
        }

        out
    }
}

/// Extract CSS `@import url("...")` or `@import "..."`.
fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if let Some(after) = trimmed.strip_prefix("@import ") {
        let after = after.trim();
        // `@import url("...")`.
        if let Some(rest) = after.strip_prefix("url(") {
            if let Some(name) = ident::extract_string_arg(rest) {
                return vec![name.to_string()];
            }
        }
        // `@import "..."` or `@import '...'`.
        if let Some(name) = ident::extract_string_arg(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
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
#[path = "../tests/langs/css_test.rs"]
mod tests;
