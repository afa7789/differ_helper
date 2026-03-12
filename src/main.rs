//! Parse git unified diff and extract VARIABLES and FUNCTIONS from added lines (+),
//! associating each with the current file path.
//!
//! Output format:
//!   VARIABLES:
//!   - <var_name> -> <file_path>
//!   FUNCTIONS:
//!   - <func_name> -> <file_path>
//!   TESTS:
//!   - <test_name> -> <file_path>

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;

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
    let mut in_test_block = false;

    for line in content.lines() {
        let line = format!("{line}\n");
        if line.starts_with("diff --git ") {
            let rest = line.trim_start_matches("diff --git ");
            if let Some(path) = rest.split_whitespace().next() {
                current_file = Some(path.trim_start_matches("a/").to_string());
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

        let content = line.trim_start_matches('+');

        // Variables: Rust let/let mut/const/static (all occurrences)
        for name in rust_var_names(content) {
            if name != "self" && name != "mut" {
                variables.push((name.to_string(), file.to_string()));
            }
        }

        // MASM const NAME = (only in .masm files, all occurrences)
        if file.contains(".masm") {
            for name in masm_const_names(content) {
                variables.push((name.to_string(), file.to_string()));
            }
        }

        // Functions: all (pub )?(async )?fn NAME or pub(crate) fn NAME
        for name in rust_fn_names(content) {
            if name != "self" && name != "mut" {
                functions.push((name.to_string(), file.to_string()));
            }
        }

        // Tests: #[test] ... fn NAME on same line
        if content.contains("#[test]") && content.contains("fn ") {
            if let Some(name) = test_fn_on_line(content) {
                tests.push((name.to_string(), file.to_string()));
            }
        }
        if content.contains("#[test]") {
            in_test_block = true;
        }
        if in_test_block && content.contains("fn ") && !content.contains("#[test]") {
            if let Some(name) = test_fn_on_line(content) {
                if name != "self" && name != "mut" {
                    tests.push((name.to_string(), file.to_string()));
                }
            }
            in_test_block = false;
        }
    }

    dedup_and_print("VARIABLES", &mut variables);
    dedup_and_print("FUNCTIONS", &mut functions);
    dedup_and_print("TESTS", &mut tests);

    Ok(())
}

fn ident_prefix(s: &str) -> Option<&str> {
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if i == 0 {
            if c == '_' || c.is_ascii_alphabetic() {
                end = i + c.len_utf8();
            } else {
                return None;
            }
        } else if c == '_' || c.is_ascii_alphanumeric() {
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

/// Find all Rust variable declarations (let, let mut, const, static) and return identifier names.
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

/// Find all MASM const NAME = on this line.
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

/// Find all Rust fn names on this line (pub fn, async fn, pub(crate) fn, fn).
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
