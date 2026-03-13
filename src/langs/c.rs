//! C language extractor.
//!
//! Extracts `#define` macros, `typedef`, `struct`, `enum`, `union` declarations,
//! and function definitions (heuristic-based, since C lacks a `fn` keyword).

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct CExtractor;

impl Extractor for CExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in macro_names(line) {
            out.variables.push(name.to_string());
        }
        for name in type_and_fn_names(line) {
            out.functions.push(name.to_string());
        }
        for name in include_names(line) {
            out.imports.push(name);
        }

        out
    }
}

/// Extract `#include` directives.
fn include_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if let Some(after) = trimmed.strip_prefix("#include ") {
        let after = after.trim();
        // `#include <header>` or `#include "header"`.
        if (after.starts_with('<') && after.contains('>'))
            || (after.starts_with('"') && after.len() > 1)
        {
            let delim_end = if after.starts_with('<') { '>' } else { '"' };
            let inner = &after[1..];
            if let Some(end) = inner.find(delim_end) {
                let name = &inner[..end];
                if !name.is_empty() {
                    return vec![name.to_string()];
                }
            }
        }
    }
    Vec::new()
}

/// Extract `#define` macro names.
fn macro_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    if !trimmed.starts_with("#define ") {
        return Vec::new();
    }
    let after = &trimmed[8..];
    if let Some(name) = ident::prefix(after) {
        vec![name]
    } else {
        Vec::new()
    }
}

/// Extract struct/enum/union/typedef names and function definitions.
fn type_and_fn_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `typedef ... NAME;` — grab the last identifier before `;`.
    if trimmed.starts_with("typedef ") {
        if let Some(name) = last_ident_before_semicolon(trimmed) {
            out.push(name);
            return out;
        }
    }

    // `struct Name {`, `enum Name {`, `union Name {`.
    for kw in ["struct ", "enum ", "union "] {
        if let Some(after) = trimmed.strip_prefix(kw) {
            if let Some(name) = ident::prefix(after) {
                out.push(name);
                return out;
            }
        }
    }

    // Function definitions: heuristic — `type name(` at line start
    // (not indented, not a macro, not a control keyword).
    if !trimmed.starts_with('#')
        && !trimmed.starts_with("//")
        && !trimmed.starts_with("/*")
        && !trimmed.starts_with("return ")
        && !trimmed.starts_with("if ")
        && !trimmed.starts_with("for ")
        && !trimmed.starts_with("while ")
        && !trimmed.starts_with("switch ")
        && trimmed.contains('(')
        && !trimmed.contains("};")
    {
        if let Some(name) = extract_c_function_name(trimmed) {
            out.push(name);
        }
    }

    out
}

/// Extract the last identifier before a semicolon (for typedef).
fn last_ident_before_semicolon(line: &str) -> Option<&str> {
    let line = line.trim_end_matches(';').trim();
    // Walk backwards to find the last word.
    let last_space = line.rfind(|c: char| c.is_whitespace() || c == '*' || c == ')')?;
    let candidate = &line[last_space + 1..];
    ident::prefix(candidate)
}

/// Heuristic: extract function name from a C function definition line.
///
/// Looks for `identifier(` preceded by a return type.
fn extract_c_function_name(line: &str) -> Option<&str> {
    let paren_pos = line.find('(')?;
    let before_paren = line[..paren_pos].trim();

    // The function name is the last identifier before `(`.
    let name_start = before_paren
        .rfind(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);

    let candidate = &before_paren[name_start..];
    if let Some(name) = ident::prefix(candidate) {
        // Must have something before it (the return type).
        if name_start > 0 || before_paren.contains(' ') || before_paren.contains('*') {
            return Some(name);
        }
    }
    None
}

#[cfg(test)]
#[path = "../tests/langs/c_test.rs"]
mod tests;
