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
        if let Some(space_pos) = trimmed.find(' ') {
            if let Some(name) = ident::extract_string_arg(&trimmed[space_pos..]) {
                return vec![name.to_string()];
            }
        }
    }
    if let Some(rest) = trimmed.strip_prefix("gem ") {
        if let Some(name) = ident::extract_string_arg(rest) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}

fn fn_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if let Some(after) = trimmed.strip_prefix("def ") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    if let Some(after) = trimmed.strip_prefix("def self.") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}

fn class_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if let Some(after) = trimmed.strip_prefix("class ") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            if name != "<<" {
                return vec![name.to_string()];
            }
        }
    }
    if let Some(after) = trimmed.strip_prefix("module ") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}
