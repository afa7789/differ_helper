//! # differ_helper
//!
//! Parse a git unified diff and extract VARIABLES, FUNCTIONS, TESTS, and
//! IMPORTS from added lines (`+`), associating each with the current file path.
//! Also detects security-sensitive patterns (WARNINGS).
//!
//! ## Usage
//!
//! ```sh
//! # Auto-detect: diffs current branch against its origin
//! differ_helper
//!
//! # Diff against a specific branch or ref
//! differ_helper main
//! differ_helper origin/develop
//! differ_helper v1.2.0
//! differ_helper abc123f
//!
//! # Read a diff file directly
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
        Some(arg) => resolve_arg(&arg)?,
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

/// Resolve the CLI argument: could be a file path, a branch/tag, a commit hash,
/// or a range like `main..HEAD` or `main...HEAD`.
///
/// Detection order:
/// 1. If it contains `..` → treat as git diff range directly.
/// 2. If it's a valid git ref → diff merge-base of that ref against HEAD.
/// 3. If it's a file on disk → read as diff file.
/// 4. Error.
fn resolve_arg(arg: &str) -> io::Result<String> {
    // Range syntax: `base..head` or `base...head`.
    if arg.contains("..") {
        return git_diff_range(arg);
    }

    // Try as git ref (branch, tag, commit hash).
    if is_git_ref(arg) {
        return git_diff_against(arg);
    }

    // Try as file.
    if std::path::Path::new(arg).exists() {
        return fs::read_to_string(arg).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e));
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("'{arg}' is not a file, branch, tag, or commit hash"),
    ))
}

/// Diff a range like `main..HEAD` or `main...feature`.
fn git_diff_range(range: &str) -> io::Result<String> {
    run_git_diff(&["diff", range])
}

/// Diff HEAD against the merge-base with a given ref.
fn git_diff_against(target: &str) -> io::Result<String> {
    // Find where HEAD and target diverged, diff from there to HEAD.
    if let Some(base) = git_output(&["merge-base", target, "HEAD"]) {
        let label = format!("{target}...HEAD");
        eprintln!("(diff: git diff {})", label);
        return run_git_diff(&["diff", &base, "HEAD"]);
    }

    // If merge-base fails (unrelated histories), diff directly.
    eprintln!("(diff: git diff {}..HEAD)", target);
    run_git_diff(&["diff", target, "HEAD"])
}

/// Check if a string resolves to a valid git ref.
fn is_git_ref(s: &str) -> bool {
    git_output(&["rev-parse", "--verify", &format!("{s}^{{commit}}")]).is_some()
}

/// Auto-detect the best diff to analyze (no argument given).
///
/// 1. Find where current branch diverged from its upstream and diff everything.
/// 2. Fall back to unstaged changes (`git diff`).
/// 3. Fall back to staged changes (`git diff --cached`).
fn git_diff_auto() -> io::Result<String> {
    if let Some((base, label)) = find_merge_base() {
        eprintln!("(auto: git diff {}...HEAD)", label);
        if let Ok(content) = run_git_diff(&["diff", &base, "HEAD"]) {
            return Ok(content);
        }
    }

    // Fallback: unstaged, then staged.
    for args in [vec!["diff"], vec!["diff", "--cached"]] {
        if let Ok(content) = run_git_diff(&args) {
            eprintln!("(auto: git {})", args.join(" "));
            return Ok(content);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no git diff found — pass a branch name, diff file, or run inside a git repo with changes",
    ))
}

/// Find the merge-base between HEAD and the best upstream ref.
/// Returns (commit_hash, human_readable_label).
///
/// Priority:
/// 1. Tracked upstream of current branch (`@{upstream}`).
/// 2. Remote HEAD (whatever the default branch is on origin).
/// 3. Common default branch names: main, master, develop, next.
fn find_merge_base() -> Option<(String, String)> {
    // 1. Tracked upstream.
    if let Some(upstream) = git_output(&["rev-parse", "--abbrev-ref", "@{upstream}"]) {
        if let Some(base) = git_output(&["merge-base", &upstream, "HEAD"]) {
            return Some((base, upstream));
        }
    }

    // 2. Remote HEAD → the default branch configured on origin.
    if let Some(remote_head) = git_output(&["symbolic-ref", "refs/remotes/origin/HEAD"]) {
        let label = remote_head
            .strip_prefix("refs/remotes/")
            .unwrap_or(&remote_head)
            .to_string();
        if let Some(base) = git_output(&["merge-base", &remote_head, "HEAD"]) {
            return Some((base, label));
        }
    }

    // 3. Common branch names.
    for branch in [
        "origin/main",
        "origin/master",
        "origin/develop",
        "origin/next",
    ] {
        if let Some(base) = git_output(&["merge-base", branch, "HEAD"]) {
            return Some((base, branch.to_string()));
        }
    }

    None
}

/// Run a git diff command and return non-empty output or an error.
fn run_git_diff(args: &[&str]) -> io::Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(io::Error::other)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(stderr.to_string()));
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();
    if content.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "diff is empty — no changes found",
        ));
    }

    Ok(content)
}

/// Run a git command and return its trimmed stdout, or None on failure.
fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
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
