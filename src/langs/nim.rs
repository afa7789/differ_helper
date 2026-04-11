//! Nim language extractor.
//!
//! Extracts `const`, `let`, `var` declarations, function/procedure
//! definitions, types, and tests.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct NimExtractor;

impl Extractor for NimExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in var_names(line) {
            out.variables.push(name.to_string());
        }
        for name in fn_names(line) {
            out.functions.push(name.to_string());
        }
        for name in type_names(line) {
            out.functions.push(name.to_string());
        }
        for name in import_names(line) {
            out.imports.push(name);
        }
        for name in test_names(line) {
            out.tests.push(name.to_string());
        }

        out
    }
}

fn var_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `const name`, `let name`, `var name`.
    for kw in ["const ", "let ", "var "] {
        if let Some(after) = trimmed.strip_prefix(kw) {
            if let Some(name) = ident::prefix(after) {
                if name != "=" && name != "*" {
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

    // `proc name(`, `func name(`, `method name(`, `iterator name(`.
    for kw in ["proc ", "func ", "method ", "iterator "] {
        if let Some(after) = trimmed.strip_prefix(kw) {
            if let Some(name) = ident::prefix(after) {
                // Handle backticks: `proc `name`() =`.
                if let Some(inner) = name.strip_prefix('`') {
                    if let Some(end) = inner.find('`') {
                        out.push(&inner[..end]);
                        return out;
                    }
                }
                out.push(name);
            }
        }
    }

    // `template name(`.
    if trimmed.starts_with("template ") {
        if let Some(after) = trimmed.strip_prefix("template ") {
            if let Some(name) = ident::prefix(after) {
                out.push(name);
            }
        }
    }

    // `macro name(`.
    if trimmed.starts_with("macro ") {
        if let Some(after) = trimmed.strip_prefix("macro ") {
            if let Some(name) = ident::prefix(after) {
                out.push(name);
            }
        }
    }

    out
}

fn type_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `type Name =`.
    if trimmed.starts_with("type ") {
        if let Some(after) = trimmed.strip_prefix("type ") {
            if let Some(name) = ident::prefix(after) {
                out.push(name);
            }
        }
    }

    // `object`, `enum`, `concept`.
    for kw in ["object ", "enum ", "concept "] {
        if let Some(after) = trimmed.strip_prefix(kw) {
            if let Some(name) = ident::prefix(after) {
                out.push(name);
            }
        }
    }

    out
}

fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `import module`, `from module import`.
    if trimmed.starts_with("import ") {
        let after = trimmed.strip_prefix("import ").unwrap_or("");
        // Get first identifier.
        if let Some(name) = ident::prefix(after) {
            // Skip `except`, `as`, `import`.
            if name != "except" && name != "as" && name != "import" {
                out.push(name.to_string());
            }
        }
    }

    // `from module import ...`.
    if trimmed.starts_with("from ") {
        let after = trimmed.strip_prefix("from ").unwrap_or("");
        if let Some(name) = ident::prefix(after) {
            out.push(name.to_string());
        }
    }

    // `include module`.
    if trimmed.starts_with("include ") {
        let after = trimmed.strip_prefix("include ").unwrap_or("");
        if let Some(name) = ident::prefix(after) {
            out.push(name.to_string());
        }
    }

    out
}

fn test_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();

    // `test "name":` or `test "name"`.
    if trimmed.starts_with("test ") {
        if let Some(after) = trimmed.strip_prefix("test ") {
            if let Some(name) = ident::extract_string_arg(after) {
                return vec![name];
            }
        }
    }

    // `suite "name":`.
    if trimmed.starts_with("suite ") {
        if let Some(after) = trimmed.strip_prefix("suite ") {
            if let Some(name) = ident::extract_string_arg(after) {
                return vec![name];
            }
        }
    }

    Vec::new()
}

#[cfg(test)]
#[path = "../tests/langs/nim_test.rs"]
mod tests;
