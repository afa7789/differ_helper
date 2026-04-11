//! Kotlin language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct KotlinExtractor;

impl Extractor for KotlinExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in import_names(line) {
            out.imports.push(name);
        }
        for name in fn_names(line) {
            out.functions.push(name.to_string());
        }
        for name in type_names(line) {
            out.functions.push(name.to_string());
        }

        out
    }
}

fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("import ") {
        if let Some(as_pos) = rest.find(" as ") {
            let before = &rest[..as_pos];
            if let Some(name) = ident::extract_string_arg(before) {
                return vec![name.to_string()];
            }
        }
        if let Some(name) = ident::extract_string_arg(rest) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}

fn fn_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    if trimmed.starts_with('@') {
        return Vec::new();
    }
    if let Some(after) = trimmed.strip_prefix("fun ") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name];
        }
    }
    if trimmed.contains('(') && !trimmed.contains("{}") {
        let paren_pos = trimmed.find('(').unwrap_or(0);
        let before_paren = &trimmed[..paren_pos];
        let name_start = before_paren
            .rfind(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let candidate = &before_paren[name_start..];
        if let Some(name) = ident::prefix(candidate) {
            if name_start > 0 {
                return vec![name];
            }
        }
    }
    Vec::new()
}

fn type_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let prefixes = [
        "class ",
        "data class ",
        "object ",
        "interface ",
        "sealed class ",
        "enum class ",
        "annotation class ",
    ];
    for prefix in prefixes {
        if let Some(after) = trimmed.strip_prefix(prefix) {
            let after = after.trim_start();
            if let Some(name) = ident::prefix(after) {
                if name != "object" {
                    return vec![name];
                }
            }
        }
    }
    Vec::new()
}
