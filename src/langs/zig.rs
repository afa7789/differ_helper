//! Zig language extractor.
//!
//! Extracts `const`/`var` declarations, function/type definitions,
//! and test functions.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct ZigExtractor;

impl Extractor for ZigExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in var_names(line) {
            out.variables.push(name.to_string());
        }
        for name in fn_names(line) {
            out.functions.push(name.to_string());
        }
        for name in test_names(line) {
            out.tests.push(name.to_string());
        }
        for name in import_names(line) {
            out.imports.push(name);
        }

        out
    }
}

fn var_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    for kw in ["const ", "var "] {
        if let Some(after) = trimmed.strip_prefix(kw) {
            if let Some(name) = ident::prefix(after) {
                if name != "=" {
                    out.push(name);
                }
            }
        }
    }

    out
}

fn fn_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `fn name(` or `pub fn name(`.
    for kw in ["pub fn ", "fn "] {
        if let Some(after) = trimmed.strip_prefix(kw) {
            if let Some(name) = ident::prefix(after) {
                out.push(name);
            }
        }
    }

    // `const Name = struct`, `const Name = enum`, `const Name = union`, `const Name = type`.
    if let Some(after) = trimmed.strip_prefix("const ") {
        if let Some(name) = ident::prefix(after) {
            let rest = after[name.len()..].trim_start();
            if rest.starts_with("= struct")
                || rest.starts_with("= enum")
                || rest.starts_with("= union")
                || rest.starts_with("= type")
            {
                out.push(name);
            }
        }
    }

    // `struct Name`, `enum Name`, `union Name`, `type Name`.
    for kw in ["struct ", "enum ", "union ", "type "] {
        if let Some(after) = trimmed.strip_prefix(kw) {
            if let Some(name) = ident::prefix(after) {
                out.push(name);
            }
        }
    }

    // `pub const`, `pub var` (already covered above, but for completeness).
    if let Some(after) = trimmed.strip_prefix("pub ") {
        for kw in ["const ", "var "] {
            if let Some(name) = after.strip_prefix(kw).and_then(ident::prefix) {
                out.push(name);
            }
        }
    }

    out
}

fn test_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();

    // `test "name"` or `test "name" {`.
    if let Some(after) = trimmed.strip_prefix("test ") {
        if let Some(name) = ident::extract_string_arg(after) {
            return vec![name];
        }
    }

    // `test "name"`.
    Vec::new()
}

fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();

    // `@import("path")` or `@import("path")`.
    if let Some(after) = trimmed.strip_prefix("@import(") {
        if let Some(name) = ident::extract_string_arg(after) {
            return vec![name.to_string()];
        }
    }

    // `@import("path")`.
    if let Some(start) = trimmed.find("@import(") {
        let after = &trimmed[start + 8..];
        if let Some(name) = ident::extract_string_arg(after) {
            return vec![name.to_string()];
        }
    }

    Vec::new()
}

#[cfg(test)]
#[path = "../tests/langs/zig_test.rs"]
mod tests;
