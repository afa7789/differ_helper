//! Go language extractor.
//!
//! Extracts `var`/`const` declarations, `func`/`type` definitions,
//! and `Test*`/`Benchmark*`/`Example*` test functions.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct GoExtractor;

impl Extractor for GoExtractor {
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

        out
    }
}

/// Extract Go variable and constant declarations.
///
/// Matches: `var name`, `const name`, and short declarations `name :=`.
fn var_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `var name` or `const name` at start of line.
    for kw in ["var ", "const "] {
        if let Some(after) = trimmed.strip_prefix(kw) {
            if let Some(name) = ident::prefix(after) {
                // Skip `var (` block openers.
                if name != "(" {
                    out.push(name);
                }
            }
        }
    }

    // Short variable declaration: `name :=`  (only at start of statement).
    if let Some(name) = ident::prefix(trimmed) {
        let rest = trimmed[name.len()..].trim_start();
        if rest.starts_with(":=") {
            out.push(name);
        }
    }

    out
}

/// Extract function and type definitions.
///
/// Matches: `func Name(`, `func (receiver) Name(`, `type Name struct/interface/...`.
fn fn_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `func Name(` or `func (receiver) Name(`.
    if let Some(after) = trimmed.strip_prefix("func ") {
        if after.starts_with('(') {
            // Method with receiver: `func (r *Receiver) Name(`.
            if let Some(close) = after.find(')') {
                let after_recv = after[close + 1..].trim_start();
                if let Some(name) = ident::prefix(after_recv) {
                    out.push(name);
                }
            }
        } else if let Some(name) = ident::prefix(after) {
            out.push(name);
        }
    }

    // `type Name struct`, `type Name interface`, etc.
    if let Some(after) = trimmed.strip_prefix("type ") {
        if let Some(name) = ident::prefix(after) {
            out.push(name);
        }
    }

    out
}

/// Extract Go test function names.
///
/// Matches: `func TestXxx(`, `func BenchmarkXxx(`, `func ExampleXxx(`.
fn test_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();

    let Some(after) = trimmed.strip_prefix("func ") else {
        return Vec::new();
    };

    if let Some(name) = ident::prefix(after) {
        if name.starts_with("Test") || name.starts_with("Benchmark") || name.starts_with("Example")
        {
            return vec![name];
        }
    }

    Vec::new()
}

#[cfg(test)]
#[path = "../tests/langs/go_test.rs"]
mod tests;
