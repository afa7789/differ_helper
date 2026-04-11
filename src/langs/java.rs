//! Java language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct JavaExtractor;

impl Extractor for JavaExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in import_names(line) {
            out.imports.push(name);
        }
        for name in fn_names(line) {
            out.functions.push(name.to_string());
        }
        for name in var_names(line) {
            out.variables.push(name.to_string());
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
        if let Some(from_pos) = rest.find(" from ") {
            let after = &rest[from_pos + 6..];
            if let Some(name) = ident::extract_string_arg(after) {
                return vec![name.to_string()];
            }
        }
        if let Some(static_pos) = rest.find(" static ") {
            let after = &rest[static_pos + 8..];
            if let Some(name) = ident::extract_string_arg(after) {
                return vec![name.to_string()];
            }
        }
        if let Some(name) = ident::extract_string_arg(rest) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}

fn var_names(line: &str) -> Vec<String> {
    let prefixes = [
        "private ",
        "protected ",
        "public ",
        "static ",
        "final ",
        "final private ",
        "final public ",
        "final static ",
    ];
    let mut out = Vec::new();
    for prefix in prefixes {
        if let Some(after) = line.strip_prefix(prefix) {
            if after.starts_with("int ")
                || after.starts_with("String ")
                || after.starts_with("var ")
                || after.starts_with("List<")
                || after.starts_with("Map<")
            {
                if let Some(name) = ident::prefix(after.split_whitespace().nth(1).unwrap_or("")) {
                    out.push(name.to_string());
                }
            } else if let Some(after_const) = after.strip_prefix("const ") {
                if let Some(name) = ident::prefix(after_const) {
                    out.push(name.to_string());
                }
            }
        }
    }
    out
}

fn fn_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    if trimmed.starts_with('@') {
        return Vec::new();
    }
    if !trimmed.contains('(') || trimmed.contains("{}") {
        return Vec::new();
    }
    let paren_pos = trimmed.find('(').unwrap_or(0);
    let before_paren = &trimmed[..paren_pos];
    if before_paren.is_empty() {
        return Vec::new();
    }
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
    Vec::new()
}

fn type_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let prefixes = [
        "public class ",
        "class ",
        "public interface ",
        "interface ",
        "public enum ",
        "enum ",
        "public record ",
        "record ",
    ];
    for prefix in prefixes {
        if let Some(after) = trimmed.strip_prefix(prefix) {
            if let Some(name) = ident::prefix(after) {
                return vec![name];
            }
        }
    }
    Vec::new()
}
