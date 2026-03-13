//! Rust language extractor.
//!
//! Extracts `let`, `const`, `static` variables, `fn` definitions,
//! and `#[test]`-annotated test functions.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct RustExtractor;

const VAR_PATTERNS: &[&str] = &["let mut ", "const ", "static ", "let "];
const FN_PATTERNS: &[&str] = &["pub(crate) fn ", "pub fn ", "async fn ", "fn "];
const SKIP_NAMES: &[&str] = &["self", "mut"];

impl Extractor for RustExtractor {
    fn extract_line(&self, line: &str, state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        // Variables
        for (_, name) in ident::find_all_after_patterns(line, VAR_PATTERNS) {
            if !SKIP_NAMES.contains(&name) {
                out.variables.push(name.to_string());
            }
        }

        // Functions (deduplicate overlapping pattern matches at the same name).
        let fn_matches = ident::find_all_after_patterns(line, FN_PATTERNS);
        let mut seen_names: Vec<&str> = Vec::new();
        for (_, name) in &fn_matches {
            if !SKIP_NAMES.contains(name) && !seen_names.contains(name) {
                seen_names.push(name);
                out.functions.push(name.to_string());
            }
        }

        // Imports: `use`, `extern crate`.
        for name in import_names(line) {
            out.imports.push(name);
        }

        // Tests: handle `#[test]` on the same line as `fn`, or on the preceding line.
        if line.contains("#[test]") && line.contains("fn ") {
            if let Some(name) = extract_test_fn(line) {
                out.tests.push(name.to_string());
            }
        }
        if line.contains("#[test]") {
            state.in_test_block = true;
        }
        if state.in_test_block && line.contains("fn ") && !line.contains("#[test]") {
            if let Some(name) = extract_test_fn(line) {
                if !SKIP_NAMES.contains(&name) {
                    out.tests.push(name.to_string());
                }
            }
            state.in_test_block = false;
        }

        out
    }
}

/// Extract Rust import paths: `use path::to::module` and `extern crate name`.
fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `pub use ...`, `use ...`
    for prefix in ["pub use ", "use "] {
        if let Some(after) = trimmed.strip_prefix(prefix) {
            let end = after
                .find(|c: char| c == ';' || c == '{' || c.is_whitespace())
                .unwrap_or(after.len());
            if end > 0 {
                out.push(after[..end].to_string());
            }
            return out;
        }
    }

    // `extern crate name`
    if let Some(after) = trimmed.strip_prefix("extern crate ") {
        let end = after
            .find(|c: char| c == ';' || c.is_whitespace())
            .unwrap_or(after.len());
        if end > 0 {
            out.push(after[..end].to_string());
        }
    }

    out
}

/// Extract the function name from a line that contains `fn `.
fn extract_test_fn(line: &str) -> Option<&str> {
    let pos = line.find("fn ")?;
    ident::prefix(&line[pos + 3..])
}

#[cfg(test)]
#[path = "../tests/langs/rust_test.rs"]
mod tests;
