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
        for name in import_names(line) {
            out.imports.push(name);
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

/// Extract Python imports: `import X`, `from X import Y`.
fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();

    // `from module import ...`
    if let Some(after) = trimmed.strip_prefix("from ") {
        let end = after.find(" import").unwrap_or(after.len());
        if end > 0 {
            return vec![after[..end].trim().to_string()];
        }
    }

    // `import module` or `import module as alias`
    if let Some(after) = trimmed.strip_prefix("import ") {
        return after
            .split(',')
            .filter_map(|part| {
                let name = part.split(" as ").next()?.trim();
                if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                }
            })
            .collect();
    }

    Vec::new()
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
#[path = "../tests/langs/python_test.rs"]
mod tests;
