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
    if let Some(after) = trimmed.strip_prefix("resource ") {
        let after = after.trim_start();
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
    if let Some(after) = trimmed.strip_prefix("data ") {
        let after = after.trim_start();
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
    if let Some(after) = trimmed.strip_prefix("variable ") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}

fn module_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if let Some(after) = trimmed.strip_prefix("module ") {
        let after = after.trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }
    Vec::new()
}
