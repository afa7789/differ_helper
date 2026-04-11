//! Swift language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct SwiftExtractor;

impl Extractor for SwiftExtractor {
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
    if trimmed.starts_with("import ") {
        let after = trimmed[7..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
        if after.starts_with("struct ") || after.starts_with("class ") || after.starts_with("enum ")
        {
            return vec![];
        }
    }
    Vec::new()
}

fn fn_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    if trimmed.starts_with("func ") {
        let after = &trimmed[5..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name];
        }
    }
    if trimmed.starts_with("@") {
        return Vec::new();
    }
    if trimmed.contains('(')
        && !trimmed.contains("{}")
        && !trimmed.starts_with("var ")
        && !trimmed.starts_with("let ")
    {
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
        "struct ",
        "class ",
        "enum ",
        "protocol ",
        "extension ",
        "actor ",
    ];
    for prefix in prefixes {
        if trimmed.starts_with(prefix) {
            let after = &trimmed[prefix.len()..].trim_start();
            if let Some(name) = ident::prefix(after) {
                return vec![name];
            }
        }
    }
    Vec::new()
}
