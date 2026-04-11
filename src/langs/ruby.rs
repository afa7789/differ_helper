//! Ruby language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct RubyExtractor;

impl Extractor for RubyExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in import_names(line) {
            out.imports.push(name);
        }
        for name in fn_names(line) {
            out.functions.push(name);
        }
        for name in class_names(line) {
            out.functions.push(name);
        }

        out
    }
}

fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("require ") || trimmed.starts_with("require_relative ") {
        if let Some(name) = ident::extract_string_arg(&trimmed[trimmed.find(' ').unwrap_or(0)..]) {
            return vec![name.to_string()];
        }
    }
    if trimmed.starts_with("gem ") {
        if let Some(name) = ident::extract_string_arg(&trimmed[4..]) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}

fn fn_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("def ") {
        let after = &trimmed[4..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    if trimmed.starts_with("def self.") {
        let after = &trimmed[8..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    if trimmed.starts_with("def ") {
        let after = &trimmed[3..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}

fn class_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("class ") {
        let after = &trimmed[6..].trim_start();
        if let Some(name) = ident::prefix(after) {
            if name != "<<" {
                return vec![name.to_string()];
            }
        }
    }
    if trimmed.starts_with("module ") {
        let after = &trimmed[7..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}
