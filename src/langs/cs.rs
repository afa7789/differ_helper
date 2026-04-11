//! C# language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct CsExtractor;

impl Extractor for CsExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in import_names(line) {
            out.imports.push(name);
        }
        for name in fn_names(line) {
            out.functions.push(name);
        }
        for name in type_names(line) {
            out.functions.push(name);
        }

        out
    }
}

fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("using ") {
        if let Some(semi) = trimmed.find(';') {
            let inner = &trimmed[5..semi];
            if let Some(name) = ident::prefix(inner.trim()) {
                return vec![name.to_string()];
            }
        }
    }
    if trimmed.starts_with("using static ") {
        if let Some(semi) = trimmed.find(';') {
            let inner = &trimmed[12..semi];
            if let Some(name) = ident::prefix(inner.trim()) {
                return vec![name.to_string()];
            }
        }
    }
    Vec::new()
}

fn fn_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with('[') {
        return Vec::new();
    }
    let prefixes = ["public ", "private ", "protected ", "internal "];
    for prefix in prefixes {
        if trimmed.starts_with(prefix) {
            let rest = &trimmed[prefix.len()..];
            if let Some(name) = extract_method_name(rest) {
                return vec![name];
            }
        }
    }
    if trimmed.starts_with("async ") {
        if let Some(name) = extract_method_name(&trimmed[6..]) {
            return vec![name];
        }
    }
    Vec::new()
}

fn extract_method_name(rest: &str) -> Option<String> {
    for kw in [
        "void ", "int ", "string ", "bool ", "var ", "Task<", "Task ", "async ",
    ] {
        if rest.starts_with(kw) {
            let after = &rest[kw.len()..];
            if let Some(name) = ident::prefix(after) {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn type_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let prefixes = [
        "public class ",
        "class ",
        "public struct ",
        "struct ",
        "public interface ",
        "interface ",
        "public enum ",
        "enum ",
        "public record ",
        "record ",
    ];
    for prefix in prefixes {
        if trimmed.starts_with(prefix) {
            let after = &trimmed[prefix.len()..].trim_start();
            if let Some(name) = ident::prefix(after) {
                return vec![name.to_string()];
            }
        }
    }
    Vec::new()
}
