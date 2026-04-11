//! Lua language extractor.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct LuaExtractor;

impl Extractor for LuaExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in fn_names(line) {
            out.functions.push(name);
        }
        for name in import_names(line) {
            out.imports.push(name);
        }

        out
    }
}

fn fn_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();

    if trimmed.starts_with("function ") {
        let after = &trimmed[9..].trim_start();
        if after.contains('.') {
            let parts: Vec<&str> = splitn(after, '.', 2);
            if let Some(name) = ident::prefix(parts[0]) {
                return vec![name.to_string()];
            }
        }
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }

    if trimmed.starts_with("local function ") {
        let after = &trimmed[13..].trim_start();
        if let Some(name) = ident::prefix(after) {
            return vec![name.to_string()];
        }
    }

    Vec::new()
}

fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();

    if trimmed.starts_with("require ") || trimmed.starts_with("dofile ") {
        let after = trimmed.split_whitespace().nth(1).unwrap_or("");
        if let Some(name) = ident::extract_string_arg(after) {
            return vec![name.to_string()];
        }
        return vec![after.to_string()];
    }

    if trimmed.starts_with("local ") && trimmed.contains(" = require") {
        let parts: Vec<&str> = trimmed.split('=').collect();
        if parts.len() > 1 {
            let left = parts[0].trim();
            let name = left.split_whitespace().last().unwrap_or("");
            if let Some(n) = ident::prefix(name) {
                return vec![n.to_string()];
            }
        }
    }

    Vec::new()
}

fn splitn(s: &str, delim: char, n: usize) -> Vec<&str> {
    let mut result = Vec::new();
    let mut count = 0;
    let mut last = 0;
    for (i, c) in s.char_indices() {
        if c == delim {
            result.push(&s[last..i]);
            last = i + 1;
            count += 1;
            if count >= n - 1 {
                break;
            }
        }
    }
    result.push(&s[last..]);
    result
}
