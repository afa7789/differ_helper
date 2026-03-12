//! Parse git unified diff and extract VARIABLES, FUNCTIONS and TESTS from added lines (+),
//! associating each with the current file path.
//!
//! Supported languages (by extension):
//!   Rust       .rs
//!   MASM       .masm
//!   TypeScript .ts .tsx
//!   JavaScript .js .jsx
//!   CSS        .css
//!   SQL        .sql
//!
//! Output format:
//!   VARIABLES:
//!   - <name> -> <file_path>
//!   FUNCTIONS:
//!   - <name> -> <file_path>
//!   TESTS:
//!   - <name> -> <file_path>

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;

// ── language tag ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Lang {
    Rust,
    Masm,
    JsTs,
    Css,
    Sql,
    Unknown,
}

fn detect_lang(file: &str) -> Lang {
    if file.ends_with(".rs") {
        Lang::Rust
    } else if file.ends_with(".masm") {
        Lang::Masm
    } else if file.ends_with(".ts")
        || file.ends_with(".tsx")
        || file.ends_with(".js")
        || file.ends_with(".jsx")
    {
        Lang::JsTs
    } else if file.ends_with(".css") {
        Lang::Css
    } else if file.ends_with(".sql") {
        Lang::Sql
    } else {
        Lang::Unknown
    }
}

// ── main ────────────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let diff_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/diff_origin_next.txt".to_string());
    let content = fs::read_to_string(&diff_path)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let mut current_file: Option<String> = None;
    let mut variables: Vec<(String, String)> = Vec::new();
    let mut functions: Vec<(String, String)> = Vec::new();
    let mut tests: Vec<(String, String)> = Vec::new();
    let mut in_test_block = false; // Rust #[test] state machine

    for line in content.lines() {
        let line = format!("{line}\n");

        // ── track current file ──────────────────────────────────────────
        if line.starts_with("diff --git ") {
            let rest = line.trim_start_matches("diff --git ");
            if let Some(path) = rest.split_whitespace().next() {
                current_file = Some(path.trim_start_matches("a/").to_string());
                in_test_block = false;
            }
        } else if line.starts_with("+++ ") {
            let rest = line.trim_start_matches("+++ ").trim();
            current_file = Some(rest.trim_start_matches("b/").to_string());
        }

        let file = match &current_file {
            Some(f) if f != "/dev/null" => f.as_str(),
            _ => continue,
        };

        if !line.starts_with('+') {
            in_test_block = false;
            continue;
        }

        let added = line.trim_start_matches('+');
        let lang = detect_lang(file);

        // ── dispatch per language ───────────────────────────────────────
        match lang {
            Lang::Rust => {
                for name in rust_var_names(added) {
                    if name != "self" && name != "mut" {
                        variables.push((name.to_string(), file.to_string()));
                    }
                }
                for name in rust_fn_names(added) {
                    if name != "self" && name != "mut" {
                        functions.push((name.to_string(), file.to_string()));
                    }
                }
                // Rust tests: #[test] state machine
                if added.contains("#[test]") && added.contains("fn ") {
                    if let Some(name) = test_fn_on_line(added) {
                        tests.push((name.to_string(), file.to_string()));
                    }
                }
                if added.contains("#[test]") {
                    in_test_block = true;
                }
                if in_test_block && added.contains("fn ") && !added.contains("#[test]") {
                    if let Some(name) = test_fn_on_line(added) {
                        if name != "self" && name != "mut" {
                            tests.push((name.to_string(), file.to_string()));
                        }
                    }
                    in_test_block = false;
                }
            }

            Lang::Masm => {
                for name in masm_const_names(added) {
                    variables.push((name.to_string(), file.to_string()));
                }
                for name in masm_proc_names(added) {
                    functions.push((name.to_string(), file.to_string()));
                }
                for name in masm_use_names(added) {
                    functions.push((name.to_string(), file.to_string()));
                }
            }

            Lang::JsTs => {
                for name in jsts_var_names(added) {
                    variables.push((name.to_string(), file.to_string()));
                }
                for name in jsts_fn_names(added) {
                    functions.push((name.to_string(), file.to_string()));
                }
                for name in jsts_test_names(added) {
                    tests.push((name.to_string(), file.to_string()));
                }
            }

            Lang::Css => {
                for name in css_var_names(added) {
                    variables.push((name.to_string(), file.to_string()));
                }
                for name in css_selector_names(added) {
                    functions.push((name.to_string(), file.to_string()));
                }
            }

            Lang::Sql => {
                for name in sql_object_names(added) {
                    functions.push((name.to_string(), file.to_string()));
                }
            }

            Lang::Unknown => {}
        }
    }

    dedup_and_print("VARIABLES", &mut variables);
    dedup_and_print("FUNCTIONS", &mut functions);
    dedup_and_print("TESTS", &mut tests);

    Ok(())
}

// ── identifier helpers ──────────────────────────────────────────────────────

fn ident_prefix(s: &str) -> Option<&str> {
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if i == 0 {
            if c == '_' || c.is_ascii_alphabetic() || c == '$' {
                end = i + c.len_utf8();
            } else {
                return None;
            }
        } else if c == '_' || c.is_ascii_alphanumeric() || c == '$' {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    if end > 0 {
        Some(&s[..end])
    } else {
        None
    }
}

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

/// Extract the content inside the first pair of quotes (single or double) or backticks.
fn extract_string_arg(s: &str) -> Option<&str> {
    let s = s.trim_start();
    let quote = s.chars().next()?;
    if quote != '\'' && quote != '"' && quote != '`' {
        return None;
    }
    let inner = &s[1..];
    if let Some(end) = inner.find(quote) {
        let val = &inner[..end];
        if !val.is_empty() {
            return Some(val);
        }
    }
    None
}

// ── Rust extractors ─────────────────────────────────────────────────────────

fn rust_var_names(line: &str) -> Vec<&str> {
    let mut matches: Vec<(usize, &str)> = Vec::new();
    for prefix in ["let mut ", "const ", "static ", "let "] {
        let mut start = 0;
        while let Some(pos) = line[start..].find(prefix) {
            let after = &line[start + pos + prefix.len()..];
            if let Some(name) = ident_prefix(after) {
                if prefix == "let " && name == "mut" {
                    start += pos + prefix.len() + name.len();
                    continue;
                }
                matches.push((start + pos, name));
            }
            start += pos + prefix.len();
        }
    }
    matches.sort_by_key(|(pos, _)| *pos);
    matches.into_iter().map(|(_, name)| name).collect()
}

fn rust_fn_names(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let patterns = ["pub(crate) fn ", "pub fn ", "async fn ", "fn "];
    let mut search_start = 0;
    while search_start < line.len() {
        let mut best: Option<(usize, &str)> = None;
        for p in patterns {
            if let Some(pos) = line[search_start..].find(p) {
                let abs_pos = search_start + pos;
                let after = &line[abs_pos + p.len()..];
                if let Some(name) = ident_prefix(after) {
                    if best.map_or(true, |(b, _)| abs_pos < b) {
                        best = Some((abs_pos, name));
                    }
                }
            }
        }
        if let Some((pos, name)) = best {
            out.push(name);
            search_start = pos + name.len();
        } else {
            break;
        }
    }
    out
}

fn test_fn_on_line(line: &str) -> Option<&str> {
    if let Some(pos) = line.find("fn ") {
        let after = &line[pos + 3..];
        ident_prefix(after)
    } else {
        None
    }
}

// ── MASM extractors ─────────────────────────────────────────────────────────

fn masm_const_names(line: &str) -> Vec<&str> {
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
/// Handles typed params like `proc name(param: type)`.
fn masm_proc_names(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let patterns = ["pub proc ", "proc "];
    for pat in patterns {
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

/// Extract MASM use/pub use names: `use miden::path::module`, `pub use module::name`.
fn masm_use_names(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let patterns = ["pub use ", "use "];
    for pat in patterns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            // Only match at line start (after trimming whitespace)
            let before = line[..abs].trim();
            if !before.is_empty() {
                start = abs + pat.len();
                continue;
            }
            let after = &line[abs + pat.len()..];
            // Grab the full module path (letters, digits, underscores, ::)
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

/// Extract a MASM identifier (stops at whitespace, parens, or special chars).
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

// ── JS / TS extractors ──────────────────────────────────────────────────────

/// Extract variable names from JS/TS: const, let, var (with optional export/declare).
fn jsts_var_names(line: &str) -> Vec<&str> {
    let mut matches: Vec<(usize, &str)> = Vec::new();
    let prefixes = [
        "export const ",
        "export let ",
        "export var ",
        "declare const ",
        "declare let ",
        "const ",
        "let ",
        "var ",
    ];
    for prefix in prefixes {
        let mut start = 0;
        while let Some(pos) = line[start..].find(prefix) {
            let abs = start + pos;
            let after = &line[abs + prefix.len()..];
            // handle destructuring: skip { or [
            let after = after.trim_start();
            if after.starts_with('{') || after.starts_with('[') {
                start = abs + prefix.len();
                continue;
            }
            if let Some(name) = ident_prefix(after) {
                // avoid duplicates from overlapping prefixes (e.g. "export const" vs "const")
                if !matches.iter().any(|(p, _)| *p == abs || abs < *p + 10 && abs > p.saturating_sub(20)) {
                    matches.push((abs, name));
                }
            }
            start = abs + prefix.len();
        }
    }
    matches.sort_by_key(|(pos, _)| *pos);
    matches.dedup_by_key(|(pos, _)| *pos);
    matches.into_iter().map(|(_, name)| name).collect()
}

/// Extract function names from JS/TS.
/// Detects: function declarations, export function, async function,
/// method definitions (name(...)), and arrow functions assigned to identifiers.
fn jsts_fn_names(line: &str) -> Vec<&str> {
    let mut out: Vec<(usize, &str)> = Vec::new();

    // function declarations
    let fn_patterns = [
        "export default async function ",
        "export default function ",
        "export async function ",
        "export function ",
        "async function ",
        "function ",
    ];
    for pat in fn_patterns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            let after = &line[abs + pat.len()..];
            if let Some(name) = ident_prefix(after) {
                if !out.iter().any(|(p, _)| *p == abs) {
                    out.push((abs, name));
                }
            }
            start = abs + pat.len();
        }
    }

    // Arrow functions: const/let/var NAME = (...) => or const NAME = async (
    // Also catches: export const NAME = (...) =>
    let arrow_kw = ["export const ", "export let ", "const ", "let ", "var "];
    for kw in arrow_kw {
        let mut start = 0;
        while let Some(pos) = line[start..].find(kw) {
            let abs = start + pos;
            let after = &line[abs + kw.len()..];
            if let Some(name) = ident_prefix(after) {
                let rest = line[abs + kw.len() + name.len()..].trim_start();
                // Check for = followed by arrow-ish pattern or async
                if rest.starts_with('=') {
                    let rhs = rest[1..].trim_start();
                    if rhs.starts_with('(')
                        || rhs.starts_with("async")
                        || rhs.starts_with("async(")
                    {
                        if !out.iter().any(|(p, _)| *p == abs) {
                            out.push((abs, name));
                        }
                    }
                }
            }
            start = abs + kw.len();
        }
    }

    // Type declarations: type NAME = / interface NAME / enum NAME
    let type_patterns = [
        "export type ",
        "export interface ",
        "export enum ",
        "type ",
        "interface ",
        "enum ",
    ];
    for pat in type_patterns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            let after = &line[abs + pat.len()..];
            if let Some(name) = ident_prefix(after) {
                if !out.iter().any(|(p, _)| *p == abs) {
                    out.push((abs, name));
                }
            }
            start = abs + pat.len();
        }
    }

    out.sort_by_key(|(pos, _)| *pos);
    out.into_iter().map(|(_, name)| name).collect()
}

/// Extract test names from describe/it/test blocks.
fn jsts_test_names(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let patterns = [
        "describe(",
        "it(",
        "test(",
        "it.each(",
        "test.each(",
        "describe.each(",
        "it.only(",
        "test.only(",
        "describe.only(",
        "it.skip(",
        "test.skip(",
        "describe.skip(",
    ];
    for pat in patterns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            let after = &line[abs + pat.len()..];
            if let Some(name) = extract_string_arg(after) {
                out.push(name.to_string());
            }
            start = abs + pat.len();
        }
    }
    out
}

// ── CSS extractors ──────────────────────────────────────────────────────────

/// Extract CSS custom properties (--var-name).
fn css_var_names(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(pos) = line[start..].find("--") {
        let abs = start + pos;
        // Grab the property name: letters, digits, hyphens, underscores
        let rest = &line[abs + 2..];
        let end = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
            .unwrap_or(rest.len());
        if end > 0 {
            out.push(&line[abs..abs + 2 + end]); // includes the "--" prefix
        }
        start = abs + 2 + end;
    }
    out
}

/// Extract CSS class selectors (.class-name) and id selectors (#id-name).
fn css_selector_names(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let mut out = Vec::new();
    // Only look at selector lines (not property declarations)
    if trimmed.contains(':') && !trimmed.ends_with('{') && !trimmed.starts_with('.') && !trimmed.starts_with('#') {
        return out;
    }
    for prefix in ['.', '#'] {
        let mut start = 0;
        while start < line.len() {
            if let Some(pos) = line[start..].find(prefix) {
                let abs = start + pos;
                // Skip if preceded by alphanumeric (it's part of a value, not a selector)
                if abs > 0 {
                    let prev = line.as_bytes()[abs - 1];
                    if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'-' {
                        start = abs + 1;
                        continue;
                    }
                }
                let rest = &line[abs + 1..];
                let end = rest
                    .find(|c: char| {
                        !c.is_ascii_alphanumeric() && c != '-' && c != '_'
                    })
                    .unwrap_or(rest.len());
                if end > 0 {
                    out.push(&line[abs..abs + 1 + end]);
                }
                start = abs + 1 + end;
            } else {
                break;
            }
        }
    }
    out
}

// ── SQL extractors ──────────────────────────────────────────────────────────

/// Extract SQL object names from CREATE/ALTER/DROP statements.
fn sql_object_names(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let upper = line.to_ascii_uppercase();

    let ddl_patterns = [
        "CREATE TABLE ",
        "CREATE OR REPLACE TABLE ",
        "ALTER TABLE ",
        "DROP TABLE ",
        "CREATE INDEX ",
        "CREATE UNIQUE INDEX ",
        "DROP INDEX ",
        "CREATE FUNCTION ",
        "CREATE OR REPLACE FUNCTION ",
        "DROP FUNCTION ",
        "CREATE VIEW ",
        "CREATE OR REPLACE VIEW ",
        "DROP VIEW ",
        "CREATE TRIGGER ",
        "CREATE OR REPLACE TRIGGER ",
        "DROP TRIGGER ",
        "CREATE TYPE ",
        "CREATE POLICY ",
        "DROP POLICY ",
        "CREATE EXTENSION ",
        "DROP EXTENSION ",
        "CREATE SCHEMA ",
        "DROP SCHEMA ",
    ];

    for pat in ddl_patterns {
        let mut start = 0;
        while let Some(pos) = upper[start..].find(pat) {
            let abs = start + pos;
            let after = &line[abs + pat.len()..];
            if let Some(name) = sql_ident(after) {
                out.push(name);
            }
            start = abs + pat.len();
        }
    }
    out
}

/// Extract a SQL identifier (possibly schema-qualified and/or quoted).
fn sql_ident(s: &str) -> Option<String> {
    let s = s.trim_start();

    // Handle IF NOT EXISTS / IF EXISTS
    let s = if s.to_ascii_uppercase().starts_with("IF NOT EXISTS ") {
        s[14..].trim_start()
    } else if s.to_ascii_uppercase().starts_with("IF EXISTS ") {
        s[10..].trim_start()
    } else {
        s
    };

    let mut result = String::new();
    let mut rest = s;

    loop {
        if rest.starts_with('"') {
            // Quoted identifier
            let inner = &rest[1..];
            if let Some(end) = inner.find('"') {
                result.push_str(&rest[..end + 2]);
                rest = &inner[end + 1..];
            } else {
                break;
            }
        } else {
            // Unquoted identifier
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if end == 0 {
                break;
            }
            result.push_str(&rest[..end]);
            rest = &rest[end..];
        }

        // Check for schema.name continuation
        if rest.starts_with('.') {
            result.push('.');
            rest = &rest[1..];
        } else {
            break;
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

// ── output ──────────────────────────────────────────────────────────────────

fn dedup_and_print(section: &str, items: &mut Vec<(String, String)>) {
    let mut seen = HashSet::new();
    items.retain(|(n, f)| seen.insert((n.clone(), f.clone())));
    items.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    println!("{section}:");
    for (name, path) in items {
        println!("- {name} -> {path}");
    }
    println!();
}
