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
    if trimmed.starts_with("#import ") {
        let after = trimmed[8..].trim_start();
        if after.starts_with('<') {
            if let Some(end) = after.find('>') {
                return vec![after[1..end].to_string()];
            }
        }
        if after.starts_with('"') {
            if let Some(end) = after[1..].find('"') {
                return vec![after[1..end + 1].to_string()];
            }
        }
    }
    if trimmed.starts_with("#include ") {
        let after = trimmed[9..].trim_start();
        if after.starts_with('<') {
            if let Some(end) = after.find('>') {
                return vec![after[1..end].to_string()];
            }
        }
        if after.starts_with('"') {
            if let Some(end) = after[1..].find('"') {
                return vec![after[1..end + 1].to_string()];
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
    if trimmed.starts_with("@interface ") {
        let after = trimmed[11..].trim_start();
        let name_end = after
            .find(|c: char| c == '(' || c == ':' || c == '{')
            .unwrap_or(after.len());
        if let Some(name) = ident::prefix(&after[..name_end]) {
            return vec![name];
        }
    }
    if trimmed.starts_with("@protocol ") {
        let after = trimmed[10..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name];
        }
    }
    if trimmed.starts_with("@class ") {
        let after = trimmed[7..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name];
        }
    }
    Vec::new()
}
