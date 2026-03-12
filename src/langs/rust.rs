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

/// Extract the function name from a line that contains `fn `.
fn extract_test_fn(line: &str) -> Option<&str> {
    let pos = line.find("fn ")?;
    ident::prefix(&line[pos + 3..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract(line: &str) -> Extracted {
        let ext = RustExtractor;
        let mut state = ExtractorState::default();
        ext.extract_line(line, &mut state)
    }

    #[test]
    fn variables() {
        let e = extract("    let x = 5;");
        assert_eq!(e.variables, vec!["x"]);
    }

    #[test]
    fn let_mut() {
        let e = extract("    let mut count = 0;");
        assert_eq!(e.variables, vec!["count"]);
    }

    #[test]
    fn functions() {
        let e = extract("pub fn process(data: &str) {");
        assert_eq!(e.functions, vec!["process"]);
    }

    #[test]
    fn async_fn() {
        let e = extract("async fn fetch_data() {");
        assert_eq!(e.functions, vec!["fetch_data"]);
    }

    #[test]
    fn test_single_line() {
        let e = extract("#[test] fn it_works() {");
        assert_eq!(e.tests, vec!["it_works"]);
    }

    #[test]
    fn test_multi_line() {
        let ext = RustExtractor;
        let mut state = ExtractorState::default();
        ext.extract_line("#[test]", &mut state);
        assert!(state.in_test_block);
        let e = ext.extract_line("fn it_works() {", &mut state);
        assert_eq!(e.tests, vec!["it_works"]);
        assert!(!state.in_test_block);
    }
}
