//! # differ_helper
//!
//! Parse a git unified diff and extract VARIABLES, FUNCTIONS and TESTS from
//! added lines (`+`), associating each with the current file path.
//!
//! ## Supported languages
//!
//! | Language       | Extensions                           |
//! |----------------|--------------------------------------|
//! | Rust           | `.rs`                                |
//! | MASM           | `.masm`                              |
//! | TypeScript/JS  | `.ts` `.tsx` `.js` `.jsx`            |
//! | CSS            | `.css`                               |
//! | SQL            | `.sql`                               |
//! | Python         | `.py` `.pyi`                         |
//! | Go             | `.go`                                |
//! | C              | `.c` `.h`                            |
//! | C++            | `.cpp` `.cxx` `.cc` `.hpp` `.hxx`    |

mod extract;
mod ident;
mod lang;
mod langs;
mod output;

use std::env;
use std::fs;
use std::io;

use rayon::prelude::*;

use extract::ExtractorState;

/// A list of `(name, file_path)` pairs for extracted symbols.
type SymbolList = Vec<(String, String)>;

fn main() -> io::Result<()> {
    let diff_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/diff_origin_next.txt".to_string());

    let content = fs::read_to_string(&diff_path)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let (mut variables, mut functions, mut tests) = parse_diff(&content);

    output::dedup_and_print("VARIABLES", &mut variables);
    output::dedup_and_print("FUNCTIONS", &mut functions);
    output::dedup_and_print("TESTS", &mut tests);

    Ok(())
}

/// A single file section from a unified diff.
struct FileSection {
    path: String,
    added_lines: Vec<String>,
}

/// Split a unified diff into per-file sections.
///
/// This is a lightweight sequential pass that groups added lines by file.
fn split_into_sections(content: &str) -> Vec<FileSection> {
    let mut sections: Vec<FileSection> = Vec::new();
    let mut current_path: Option<String> = None;

    for line in content.lines() {
        let line_with_nl = format!("{line}\n");

        if line_with_nl.starts_with("diff --git ") {
            let rest = line_with_nl.trim_start_matches("diff --git ");
            if let Some(path) = rest.split_whitespace().next() {
                let path = path.trim_start_matches("a/").to_string();
                current_path = Some(path.clone());
                sections.push(FileSection {
                    path,
                    added_lines: Vec::new(),
                });
            }
        } else if line_with_nl.starts_with("+++ ") {
            let rest = line_with_nl.trim_start_matches("+++ ").trim();
            let path = rest.trim_start_matches("b/").to_string();
            current_path = Some(path.clone());
            if let Some(section) = sections.last_mut() {
                section.path = path;
            } else {
                sections.push(FileSection {
                    path,
                    added_lines: Vec::new(),
                });
            }
        } else if line_with_nl.starts_with('+') {
            if let Some(ref p) = current_path {
                if p != "/dev/null" {
                    if let Some(section) = sections.last_mut() {
                        section.added_lines.push(line_with_nl);
                    }
                }
            }
        }
    }

    sections
}

/// Extract symbols from a single file section.
fn extract_section(section: &FileSection) -> (SymbolList, SymbolList, SymbolList) {
    let detected_lang = lang::detect(&section.path);
    let Some(extractor) = langs::extractor_for(detected_lang) else {
        return (Vec::new(), Vec::new(), Vec::new());
    };

    let mut variables: SymbolList = Vec::new();
    let mut functions: SymbolList = Vec::new();
    let mut tests: SymbolList = Vec::new();
    let mut state = ExtractorState::default();

    for line in &section.added_lines {
        let added = line.trim_start_matches('+');
        let extracted = extractor.extract_line(added, &mut state);

        for name in extracted.variables {
            variables.push((name, section.path.clone()));
        }
        for name in extracted.functions {
            functions.push((name, section.path.clone()));
        }
        for name in extracted.tests {
            tests.push((name, section.path.clone()));
        }
    }

    (variables, functions, tests)
}

/// Parse a unified diff and extract symbols from all added lines.
///
/// Files are processed in parallel using rayon for maximum throughput.
fn parse_diff(content: &str) -> (SymbolList, SymbolList, SymbolList) {
    let sections = split_into_sections(content);

    // Process each file section in parallel.
    let results: Vec<(SymbolList, SymbolList, SymbolList)> =
        sections.par_iter().map(extract_section).collect();

    // Merge results from all parallel tasks.
    let mut variables: SymbolList = Vec::new();
    let mut functions: SymbolList = Vec::new();
    let mut tests: SymbolList = Vec::new();

    for (v, f, t) in results {
        variables.extend(v);
        functions.extend(f);
        tests.extend(t);
    }

    (variables, functions, tests)
}

#[cfg(test)]
#[path = "tests/main_test.rs"]
mod tests;
