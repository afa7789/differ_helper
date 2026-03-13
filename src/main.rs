//! # differ_helper
//!
//! Parse a git unified diff and extract VARIABLES, FUNCTIONS, TESTS, and
//! IMPORTS from added lines (`+`), associating each with the current file path.
//! Also detects security-sensitive patterns (WARNINGS).
//!
//! ## Usage
//!
//! ```sh
//! # Auto-detect: runs `git diff HEAD~1` in the current directory
//! differ_helper
//!
//! # Or pass a diff file explicitly
//! differ_helper /path/to/diff.txt
//! ```
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
mod security;

use std::env;
use std::fs;
use std::io;
use std::process::Command;

use rayon::prelude::*;

use extract::ExtractorState;

/// A list of `(name, file_path)` pairs for extracted symbols.
type SymbolList = Vec<(String, String)>;

fn main() -> io::Result<()> {
    let content = match env::args().nth(1) {
        Some(path) => {
            fs::read_to_string(&path).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        }
        None => git_diff_auto()?,
    };

    let (mut variables, mut functions, mut tests, mut imports, warnings) = parse_diff(&content);

    output::dedup_and_print("VARIABLES", &mut variables);
    output::dedup_and_print("FUNCTIONS", &mut functions);
    output::dedup_and_print("TESTS", &mut tests);
    output::dedup_and_print("IMPORTS", &mut imports);

    if !warnings.is_empty() {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        output::print_warnings(&mut handle, &warnings).expect("failed to write to stdout");
    }

    Ok(())
}

/// Run `git diff HEAD~1` automatically. Falls back to `git diff` (staged + unstaged),
/// then to `git diff --cached` (staged only).
fn git_diff_auto() -> io::Result<String> {
    for args in [
        vec!["diff", "HEAD~1"],
        vec!["diff"],
        vec!["diff", "--cached"],
    ] {
        if let Ok(output) = Command::new("git").args(&args).output() {
            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout).to_string();
                if !content.trim().is_empty() {
                    eprintln!("(auto: git {})", args.join(" "));
                    return Ok(content);
                }
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no git diff found — pass a diff file as argument or run inside a git repo with changes",
    ))
}

/// A single file section from a unified diff.
struct FileSection {
    path: String,
    added_lines: Vec<String>,
}

/// Split a unified diff into per-file sections.
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

/// Extraction result for a single file section.
struct SectionResult {
    variables: SymbolList,
    functions: SymbolList,
    tests: SymbolList,
    imports: SymbolList,
    warnings: Vec<security::Warning>,
}

/// Extract symbols from a single file section.
fn extract_section(section: &FileSection) -> SectionResult {
    let detected_lang = lang::detect(&section.path);

    let mut variables: SymbolList = Vec::new();
    let mut functions: SymbolList = Vec::new();
    let mut tests: SymbolList = Vec::new();
    let mut imports: SymbolList = Vec::new();

    if let Some(extractor) = langs::extractor_for(detected_lang) {
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
            for name in extracted.imports {
                imports.push((name, section.path.clone()));
            }
        }
    }

    let warnings = security::scan_lines(&section.added_lines, &section.path);

    SectionResult {
        variables,
        functions,
        tests,
        imports,
        warnings,
    }
}

/// Parse a unified diff and extract symbols from all added lines.
///
/// Files are processed in parallel using rayon for maximum throughput.
fn parse_diff(
    content: &str,
) -> (
    SymbolList,
    SymbolList,
    SymbolList,
    SymbolList,
    Vec<security::Warning>,
) {
    let sections = split_into_sections(content);

    let results: Vec<SectionResult> = sections.par_iter().map(extract_section).collect();

    let mut variables: SymbolList = Vec::new();
    let mut functions: SymbolList = Vec::new();
    let mut tests: SymbolList = Vec::new();
    let mut imports: SymbolList = Vec::new();
    let mut warnings: Vec<security::Warning> = Vec::new();

    for r in results {
        variables.extend(r.variables);
        functions.extend(r.functions);
        tests.extend(r.tests);
        imports.extend(r.imports);
        warnings.extend(r.warnings);
    }

    (variables, functions, tests, imports, warnings)
}

#[cfg(test)]
#[path = "tests/main_test.rs"]
mod tests;
