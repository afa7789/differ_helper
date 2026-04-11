//! PHP language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct PhpExtractor;

impl Extractor for PhpExtractor {
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
    if trimmed.starts_with("require ") || trimmed.starts_with("include ") {
        if trimmed.contains('(') {
            if let Some(name) =
                ident::extract_string_arg(trimmed.split_whitespace().nth(1).unwrap_or(""))
            {
                return vec![name.to_string()];
            }
        }
    }
    if trimmed.starts_with("use ") && !trimmed.contains(" use ") {
        let rest = &trimmed[4..].trim_end();
        if rest.ends_with(';') {
            let rest = rest.trim_end_matches(';');
        }
        let parts: Vec<&str> = rest.split(" as ").collect();
        let name = parts[0];
        if let Some(n) = ident::prefix(name) {
            return vec![n.to_string()];
        }
    }
    Vec::new()
}

fn fn_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("function ") {
        let after = &trimmed[9..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    if trimmed.starts_with("public function ")
        || trimmed.starts_with("private function ")
        || trimmed.starts_with("protected function ")
    {
        let prefix = if trimmed.starts_with("public function ") {
            14
        } else if trimmed.starts_with("private function ") {
            15
        } else {
            17
        };
        let after = &trimmed[prefix..].trim_start();
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
            return vec![name.to_string()];
        }
    }
    if trimmed.starts_with("interface ") {
        let after = &trimmed[9..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    if trimmed.starts_with("trait ") {
        let after = &trimmed[6..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}
