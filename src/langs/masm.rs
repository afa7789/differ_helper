//! MASM (Miden Assembly) language extractor.
//!
//! Extracts `const` declarations, `proc` definitions, and `use`/`pub use` imports.

use crate::extract::{Extracted, Extractor, ExtractorState};

pub struct MasmExtractor;

impl Extractor for MasmExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in const_names(line) {
            out.variables.push(name.to_string());
        }
        for name in proc_names(line) {
            out.functions.push(name.to_string());
        }
        for name in use_names(line) {
            out.functions.push(name.to_string());
        }

        out
    }
}

/// Extract MASM constant names: `const NAME = value`.
fn const_names(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(pos) = line[start..].find("const ") {
        let after = &line[start + pos + 6..];
        if let Some(name) = masm_const_name(after) {
            out.push(name);
            start += pos + 6 + name.len();
        } else {
            start += pos + 6;
        }
    }
    out
}

/// Extract MASM procedure names: `pub proc name`, `proc name`.
fn proc_names(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    for pat in ["pub proc ", "proc "] {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            let after = &line[abs + pat.len()..];
            if let Some(name) = masm_ident(after) {
                if !out.contains(&name) {
                    out.push(name);
                }
            }
            start = abs + pat.len();
        }
    }
    out
}

/// Extract MASM use/pub use imports: `use miden::path::module`.
fn use_names(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    for pat in ["pub use ", "use "] {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            // Only match at line start (after trimming whitespace).
            let before = line[..abs].trim();
            if !before.is_empty() {
                start = abs + pat.len();
                continue;
            }
            let after = &line[abs + pat.len()..];
            let end = after
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != ':')
                .unwrap_or(after.len());
            if end > 0 {
                out.push(&after[..end]);
            }
            start = abs + pat.len() + end;
        }
    }
    out
}

/// Parse a MASM constant name (alphanumeric + underscores, stops at whitespace or `=`).
fn masm_const_name(s: &str) -> Option<&str> {
    let s = s.trim_start();
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if c == '_' || c.is_ascii_alphanumeric() {
            end = i + c.len_utf8();
        } else if c.is_whitespace() || c == '=' {
            break;
        } else {
            return None;
        }
    }
    if end > 0 {
        Some(&s[..end])
    } else {
        None
    }
}

/// Parse a MASM identifier (stops at whitespace, parens, or special chars).
fn masm_ident(s: &str) -> Option<&str> {
    let s = s.trim_start();
    let end = s
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(s.len());
    if end > 0 {
        Some(&s[..end])
    } else {
        None
    }
}

#[cfg(test)]
#[path = "../tests/langs/masm_test.rs"]
mod tests;
