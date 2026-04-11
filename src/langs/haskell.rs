//! Haskell language extractor.
//!
//! Extracts `let`, `data`, `type`, `newtype` declarations,
//! function definitions, and test functions.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct HaskellExtractor;

impl Extractor for HaskellExtractor {
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

    // `let name =` at start of line.
    if trimmed.starts_with("let ") {
        if let Some(after) = trimmed.strip_prefix("let ") {
            if let Some(name) = ident::prefix(after) {
                if name != "(" {
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

    let Some(name) = ident::prefix(trimmed) else {
        return out;
    };

    // Skip Haskell keywords that would incorrectly match as function names.
    let keywords = [
        "data", "newtype", "type", "class", "instance", "module", "import", "let", "where",
    ];
    if keywords.contains(&name) {
        return out;
    }

    let rest = trimmed[name.len()..].trim_start();
    if rest.starts_with('=') || rest.starts_with("::") {
        out.push(name);
    } else if rest.starts_with('|') {
        out.push(name);
    } else if rest.starts_with('{') || rest.starts_with('(') {
    } else if rest.is_empty() {
    } else if rest.contains('=') {
        out.push(name);
    }

    out
}

fn type_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    if trimmed.starts_with("newtype ") {
        if let Some(name) = ident::prefix(&trimmed[8..]) {
            out.push(name);
        }
    } else if trimmed.starts_with("class ") {
        if let Some(name) = ident::prefix(&trimmed[6..]) {
            out.push(name);
        }
    } else if trimmed.starts_with("data ") {
        if let Some(name) = ident::prefix(&trimmed[5..]) {
            out.push(name);
        }
    } else if trimmed.starts_with("type ") {
        if let Some(name) = ident::prefix(&trimmed[5..]) {
            out.push(name);
        }
    } else if trimmed.starts_with("type family ") {
        if let Some(name) = ident::prefix(&trimmed[12..]) {
            out.push(name);
        }
    }

    out
}

fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `import qualified Module` or `import Module`.
    if let Some(after) = trimmed.strip_prefix("import ") {
        let is_qualified = after.starts_with("qualified ");
        let name = if is_qualified {
            after.strip_prefix("qualified ").unwrap_or("")
        } else {
            after
        };

        // Get the module name (could have dots: Data.List).
        // Find first whitespace or ( to get module name.
        let module_end = name
            .find(|c: char| c.is_whitespace() || c == '(' || c == '(')
            .unwrap_or(name.len());
        let module_name = name[..module_end].trim();
        if !module_name.is_empty() && module_name != "module" {
            out.push(module_name.to_string());
        }
    }

    out
}

fn test_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();

    // HSpec: `it "description" $` or `it "description" $ do`.
    if trimmed.starts_with("it ") {
        if let Some(after) = trimmed.strip_prefix("it ") {
            if let Some(name) = ident::extract_string_arg(after) {
                return vec![name];
            }
        }
    }

    // QuickCheck: `prop_name`.
    if trimmed.starts_with("prop_") {
        if let Some(name) = ident::prefix(trimmed) {
            return vec![name];
        }
    }

    // HUnit: `testCase "description"`.
    if trimmed.starts_with("testCase ") {
        if let Some(after) = trimmed.strip_prefix("testCase ") {
            if let Some(name) = ident::extract_string_arg(after) {
                return vec![name];
            }
        }
    }

    Vec::new()
}

#[cfg(test)]
#[path = "../tests/langs/haskell_test.rs"]
mod tests;
