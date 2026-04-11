//! Shell language extractor (bash, zsh, sh).

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct ShellExtractor;

impl Extractor for ShellExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in fn_names(line) {
            out.functions.push(name);
        }
        for name in import_names(line) {
            out.imports.push(name);
        }

        out
    }
}

fn fn_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();

    if let Some(after) = trimmed.strip_prefix("function ") {
        let after = after.trim_start();
        if after.ends_with('(') {
            let name = after.trim_end_matches('(');
            return vec![name.to_string()];
        }
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }

    if let Some(after) = trimmed.strip_prefix("alias ") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }

    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return Vec::new();
    }

    if trimmed.contains('(') && trimmed.contains(')') && !trimmed.contains("()") {
        let paren_pos = trimmed.find('(').unwrap_or(0);
        let before = &trimmed[..paren_pos];
        if before.contains(' ') {
            let name = before.rsplit(' ').next().unwrap_or("");
            if let Some(n) = ident::prefix(name) {
                return vec![n.to_string()];
            }
        }
    }

    Vec::new()
}

fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();

    if let Some(after) = trimmed.strip_prefix("source ") {
        let after = after.trim_start();
        if let Some(name) = ident::extract_string_arg(after) {
            return vec![name.to_string()];
        }
        if let Some(stripped) = after.strip_prefix('$') {
            return vec![stripped.to_string()];
        }
    }

    if trimmed.starts_with(".") && !trimmed.starts_with("..") {
        let after = trimmed[1..].trim_start();
        if after.starts_with('/') || after.ends_with(".sh") {
            if let Some(name) = ident::extract_string_arg(after) {
                return vec![name.to_string()];
            }
            return vec![after.to_string()];
        }
    }

    Vec::new()
}
