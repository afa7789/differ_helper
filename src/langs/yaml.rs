//! YAML/TOML language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct YamlExtractor;

impl Extractor for YamlExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in key_names(line) {
            out.variables.push(name);
        }
        for name in anchor_names(line) {
            out.functions.push(name);
        }

        out
    }
}

fn key_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();

    if trimmed.ends_with(':') && !trimmed.starts_with('-') {
        let key = trimmed.trim_end_matches(':');
        if !key.is_empty() && !key.starts_with('#') {
            if let Some(name) = ident::prefix(key) {
                return vec![name.to_string()];
            }
        }
    }

    if trimmed.starts_with('-') && !trimmed.starts_with("---") {
        let after = trimmed[1..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }

    Vec::new()
}

fn anchor_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();

    if trimmed.starts_with("&") {
        let after = &trimmed[1..].trim_start();
        if after.starts_with("anchor ") {
            let name = after[7..].trim_start();
            if let Some(n) = ident::prefix(name) {
                return vec![n.to_string()];
            }
        }
    }

    Vec::new()
}
