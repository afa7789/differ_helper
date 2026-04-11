//! Objective-C language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct ObjCExtractor;

impl Extractor for ObjCExtractor {
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
    if let Some(after) = trimmed.strip_prefix("#import ") {
        let after = after.trim_start();
        if let Some(stripped) = after.strip_prefix('<') {
            if let Some(end) = stripped.find('>') {
                return vec![stripped[..end].to_string()];
            }
        }
        if let Some(stripped) = after.strip_prefix('"') {
            if let Some(end) = stripped.find('"') {
                return vec![stripped[..end + 1].to_string()];
            }
        }
    }
    if let Some(after) = trimmed.strip_prefix("#include ") {
        let after = after.trim_start();
        if let Some(stripped) = after.strip_prefix('<') {
            if let Some(end) = stripped.find('>') {
                return vec![stripped[..end].to_string()];
            }
        }
        if let Some(stripped) = after.strip_prefix('"') {
            if let Some(end) = stripped.find('"') {
                return vec![stripped[..end + 1].to_string()];
            }
        }
    }
    Vec::new()
}

fn fn_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    if trimmed.starts_with('-') || trimmed.starts_with('+') {
        let after = trimmed[1..].trim_start();
        if after.starts_with('(') {
            if let Some(close) = after.find(')') {
                let rest = &after[close + 1..].trim_start();
                if let Some(name) = ident::prefix(rest) {
                    return vec![name];
                }
            }
        }
    }
    if trimmed.starts_with("@implementation ") {
        return vec![];
    }
    if !trimmed.contains('(') || trimmed.contains("{}") {
        return Vec::new();
    }
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
    Vec::new()
}

fn type_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    if let Some(after) = trimmed.strip_prefix("@interface ") {
        let after = after.trim_start();
        #[allow(clippy::manual_pattern_char_comparison)]
        let name_end = after
            .find(|c: char| c == '(' || c == ':' || c == '{')
            .unwrap_or(after.len());
        if let Some(name) = ident::prefix(&after[..name_end]) {
            return vec![name];
        }
    }
    if let Some(after) = trimmed.strip_prefix("@protocol ") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name];
        }
    }
    if let Some(after) = trimmed.strip_prefix("@class ") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name];
        }
    }
    Vec::new()
}
