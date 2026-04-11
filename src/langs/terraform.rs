//! Terraform language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct TerraformExtractor;

impl Extractor for TerraformExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in resource_names(line) {
            out.functions.push(name);
        }
        for name in data_names(line) {
            out.functions.push(name);
        }
        for name in variable_names(line) {
            out.variables.push(name);
        }
        for name in module_names(line) {
            out.functions.push(name);
        }

        out
    }
}

fn resource_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("resource ") {
        let after = &trimmed[9..].trim_start();
        let parts: Vec<&str> = after.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = format!("{}_{}", parts[0], parts[1]);
            return vec![name];
        }
    }
    Vec::new()
}

fn data_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("data ") {
        let after = &trimmed[4..].trim_start();
        let parts: Vec<&str> = after.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = format!("data_{}_{}", parts[0], parts[1]);
            return vec![name];
        }
    }
    Vec::new()
}

fn variable_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("variable ") {
        let after = &trimmed[9..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}

fn module_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("module ") {
        let after = &trimmed[7..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}
