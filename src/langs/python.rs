//! Python language extractor.
//!
//! Extracts module-level variable assignments, function/class definitions,
//! and test functions (pytest `test_*` and unittest patterns).

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct PythonExtractor;

impl Extractor for PythonExtractor {
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

/// Extract Python variable assignments at module/class level.
///
/// Matches patterns like `NAME = ...`, `NAME: type = ...`, and `NAME: type`.
/// Skips indented lines (local variables inside functions) — only captures
/// top-level (0 indent) and class-level (4-space indent) assignments.
fn var_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();

    // Only top-level or single-indent (class body) assignments.
    let indent = line.len() - line.trim_start().len();
    if indent > 4 {
        return Vec::new();
    }

    // Skip decorators, comments, control flow, imports.
    if trimmed.starts_with('@')
        || trimmed.starts_with('#')
        || trimmed.starts_with("def ")
        || trimmed.starts_with("async def ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("import ")
        || trimmed.starts_with("from ")
        || trimmed.starts_with("if ")
        || trimmed.starts_with("elif ")
        || trimmed.starts_with("else:")
        || trimmed.starts_with("for ")
        || trimmed.starts_with("while ")
        || trimmed.starts_with("return ")
        || trimmed.starts_with("raise ")
        || trimmed.starts_with("yield ")
        || trimmed.starts_with("try:")
        || trimmed.starts_with("except")
        || trimmed.starts_with("finally:")
        || trimmed.starts_with("with ")
    {
        return Vec::new();
    }

    // Look for `NAME =` or `NAME:` (type annotation).
    if let Some(name) = ident::prefix(trimmed) {
        let rest = trimmed[name.len()..].trim_start();
        if rest.starts_with('=') && !rest.starts_with("==") {
            return vec![name];
        }
        if rest.starts_with(':') {
            return vec![name];
        }
    }

    Vec::new()
}

/// Extract function and class definitions.
fn fn_names(line: &str) -> Vec<&str> {
    let patterns = ["async def ", "def ", "class "];
    let mut out = Vec::new();

    for pat in patterns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            // Only match at start of line (after whitespace).
            let before = line[..abs].trim();
            if !before.is_empty() && !before.ends_with('@') {
                start = abs + pat.len();
                continue;
            }
            let after = &line[abs + pat.len()..];
            if let Some(name) = ident::prefix(after) {
                out.push(name);
            }
            start = abs + pat.len();
        }
    }
    out
}

/// Extract test function names (pytest convention: `test_*` functions).
fn test_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();

    // pytest: `def test_something(...):`
    for pat in ["async def ", "def "] {
        if let Some(pos) = trimmed.find(pat) {
            let after = &trimmed[pos + pat.len()..];
            if let Some(name) = ident::prefix(after) {
                if name.starts_with("test_") {
                    return vec![name];
                }
            }
        }
    }

    // unittest: `class TestSomething(unittest.TestCase):`
    if let Some(pos) = trimmed.find("class ") {
        let after = &trimmed[pos + 6..];
        if let Some(name) = ident::prefix(after) {
            if name.starts_with("Test") {
                return vec![name];
            }
        }
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::ExtractorState;

    fn extract(line: &str) -> Extracted {
        PythonExtractor.extract_line(line, &mut ExtractorState::default())
    }

    #[test]
    fn module_variable() {
        let e = extract("MAX_RETRIES = 3");
        assert_eq!(e.variables, vec!["MAX_RETRIES"]);
    }

    #[test]
    fn type_annotated_variable() {
        let e = extract("name: str = 'hello'");
        assert_eq!(e.variables, vec!["name"]);
    }

    #[test]
    fn function_def() {
        let e = extract("def process_data(items):");
        assert_eq!(e.functions, vec!["process_data"]);
    }

    #[test]
    fn async_function_def() {
        let e = extract("async def fetch_data(url):");
        assert_eq!(e.functions, vec!["fetch_data"]);
    }

    #[test]
    fn class_def() {
        let e = extract("class UserService:");
        assert_eq!(e.functions, vec!["UserService"]);
    }

    #[test]
    fn pytest_test() {
        let e = extract("def test_user_creation():");
        assert_eq!(e.tests, vec!["test_user_creation"]);
        // Also appears as function.
        assert_eq!(e.functions, vec!["test_user_creation"]);
    }

    #[test]
    fn unittest_class() {
        let e = extract("class TestUserService(unittest.TestCase):");
        assert_eq!(e.tests, vec!["TestUserService"]);
    }

    #[test]
    fn skip_local_variable() {
        let e = extract("        result = compute()");
        assert!(e.variables.is_empty());
    }

    #[test]
    fn skip_import() {
        let e = extract("from os import path");
        assert!(e.variables.is_empty());
    }

    #[test]
    fn skip_comparison() {
        let e = extract("if x == 5:");
        assert!(e.variables.is_empty());
    }
}
