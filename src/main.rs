//! # differ_helper
//!
//! Parse a git unified diff and extract VARIABLES, FUNCTIONS and TESTS from
//! added lines (`+`), associating each with the current file path.
//!
//! ## Supported languages
//!
//! | Language       | Extensions                        |
//! |----------------|-----------------------------------|
//! | Rust           | `.rs`                             |
//! | MASM           | `.masm`                           |
//! | TypeScript/JS  | `.ts` `.tsx` `.js` `.jsx`         |
//! | CSS            | `.css`                            |
//! | SQL            | `.sql`                            |
//! | Python         | `.py` `.pyi`                      |
//! | Go             | `.go`                             |
//! | C              | `.c` `.h`                         |
//! | C++            | `.cpp` `.cxx` `.cc` `.hpp` `.hxx` |

mod extract;
mod ident;
mod lang;
mod langs;
mod output;

use std::env;
use std::fs;
use std::io;

use extract::ExtractorState;

/// A list of `(name, file_path)` pairs for extracted symbols.
type SymbolList = Vec<(String, String)>;

fn main() -> io::Result<()> {
    let diff_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/diff_origin_next.txt".to_string());

    let content = fs::read_to_string(&diff_path)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let (variables, functions, tests) = parse_diff(&content);

    let mut variables = variables;
    let mut functions = functions;
    let mut tests = tests;

    output::dedup_and_print("VARIABLES", &mut variables);
    output::dedup_and_print("FUNCTIONS", &mut functions);
    output::dedup_and_print("TESTS", &mut tests);

    Ok(())
}

/// Parse a unified diff and extract symbols from all added lines.
fn parse_diff(content: &str) -> (SymbolList, SymbolList, SymbolList) {
    let mut current_file: Option<String> = None;
    let mut variables: SymbolList = Vec::new();
    let mut functions: SymbolList = Vec::new();
    let mut tests: SymbolList = Vec::new();
    let mut state = ExtractorState::default();

    for line in content.lines() {
        let line = format!("{line}\n");

        // Track the current file path from diff headers.
        if line.starts_with("diff --git ") {
            let rest = line.trim_start_matches("diff --git ");
            if let Some(path) = rest.split_whitespace().next() {
                current_file = Some(path.trim_start_matches("a/").to_string());
                state = ExtractorState::default();
            }
        } else if line.starts_with("+++ ") {
            let rest = line.trim_start_matches("+++ ").trim();
            current_file = Some(rest.trim_start_matches("b/").to_string());
        }

        let file = match &current_file {
            Some(f) if f != "/dev/null" => f.as_str(),
            _ => continue,
        };

        // Only process added lines.
        if !line.starts_with('+') {
            state.in_test_block = false;
            continue;
        }

        let added = line.trim_start_matches('+');
        let detected_lang = lang::detect(file);

        let Some(extractor) = langs::extractor_for(detected_lang) else {
            continue;
        };

        let extracted = extractor.extract_line(added, &mut state);

        for name in extracted.variables {
            variables.push((name, file.to_string()));
        }
        for name in extracted.functions {
            functions.push((name, file.to_string()));
        }
        for name in extracted.tests {
            tests.push((name, file.to_string()));
        }
    }

    (variables, functions, tests)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal diff for a single file.
    fn make_diff(filename: &str, added_lines: &[&str]) -> String {
        let mut diff = format!("diff --git a/{filename} b/{filename}\n");
        diff.push_str(&format!("+++ b/{filename}\n"));
        for line in added_lines {
            diff.push_str(&format!("+{line}\n"));
        }
        diff
    }

    #[test]
    fn rust_end_to_end() {
        let diff = make_diff(
            "src/lib.rs",
            &[
                "let count = 0;",
                "pub fn process() {",
                "#[test]",
                "fn it_works() {",
            ],
        );
        let (vars, fns, tests) = parse_diff(&diff);
        assert!(vars.iter().any(|(n, _)| n == "count"));
        assert!(fns.iter().any(|(n, _)| n == "process"));
        assert!(tests.iter().any(|(n, _)| n == "it_works"));
    }

    #[test]
    fn python_end_to_end() {
        let diff = make_diff(
            "app/models.py",
            &[
                "MAX_RETRIES = 3",
                "def process_data(items):",
                "class UserService:",
                "def test_user_creation():",
            ],
        );
        let (vars, fns, tests) = parse_diff(&diff);
        assert!(vars.iter().any(|(n, _)| n == "MAX_RETRIES"));
        assert!(fns.iter().any(|(n, _)| n == "process_data"));
        assert!(fns.iter().any(|(n, _)| n == "UserService"));
        assert!(tests.iter().any(|(n, _)| n == "test_user_creation"));
    }

    #[test]
    fn go_end_to_end() {
        let diff = make_diff(
            "cmd/server.go",
            &[
                "var maxRetries int = 3",
                "func ProcessData(items []Item) error {",
                "type Config struct {",
                "func TestParseConfig(t *testing.T) {",
            ],
        );
        let (vars, fns, tests) = parse_diff(&diff);
        assert!(vars.iter().any(|(n, _)| n == "maxRetries"));
        assert!(fns.iter().any(|(n, _)| n == "ProcessData"));
        assert!(fns.iter().any(|(n, _)| n == "Config"));
        assert!(tests.iter().any(|(n, _)| n == "TestParseConfig"));
    }

    #[test]
    fn c_end_to_end() {
        let diff = make_diff(
            "lib/parser.c",
            &[
                "#define MAX_SIZE 1024",
                "struct Node {",
                "int parse_input(const char *buf) {",
            ],
        );
        let (vars, fns, _) = parse_diff(&diff);
        assert!(vars.iter().any(|(n, _)| n == "MAX_SIZE"));
        assert!(fns.iter().any(|(n, _)| n == "Node"));
        assert!(fns.iter().any(|(n, _)| n == "parse_input"));
    }

    #[test]
    fn cpp_end_to_end() {
        let diff = make_diff(
            "src/widget.cpp",
            &[
                "class Widget {",
                "namespace detail {",
                "TEST(ParserSuite, HandlesEmpty) {",
            ],
        );
        let (_, fns, tests) = parse_diff(&diff);
        assert!(fns.iter().any(|(n, _)| n == "Widget"));
        assert!(fns.iter().any(|(n, _)| n == "detail"));
        assert!(tests.iter().any(|(n, _)| n == "ParserSuite.HandlesEmpty"));
    }

    #[test]
    fn sql_end_to_end() {
        let diff = make_diff(
            "migrations/001.sql",
            &["CREATE TABLE IF NOT EXISTS users ("],
        );
        let (_, fns, _) = parse_diff(&diff);
        assert!(fns.iter().any(|(n, _)| n == "users"));
    }

    #[test]
    fn jsts_end_to_end() {
        let diff = make_diff(
            "app.ts",
            &[
                "export const API_KEY = 'abc';",
                "export function processData(input) {",
                "  describe('UserService', () => {",
            ],
        );
        let (vars, fns, tests) = parse_diff(&diff);
        assert!(vars.iter().any(|(n, _)| n == "API_KEY"));
        assert!(fns.iter().any(|(n, _)| n == "processData"));
        assert!(tests.iter().any(|(n, _)| n == "UserService"));
    }

    #[test]
    fn unknown_lang_produces_nothing() {
        let diff = make_diff("README.md", &["# Hello World"]);
        let (vars, fns, tests) = parse_diff(&diff);
        assert!(vars.is_empty());
        assert!(fns.is_empty());
        assert!(tests.is_empty());
    }

    #[test]
    fn dev_null_skipped() {
        let diff = "diff --git a/old.rs b/old.rs\n+++ /dev/null\n+let x = 1;\n";
        let (vars, _, _) = parse_diff(diff);
        assert!(vars.is_empty());
    }

    #[test]
    fn css_end_to_end() {
        let diff = make_diff("styles.css", &[".container {", "  --primary-color: #333;"]);
        let (vars, fns, _) = parse_diff(&diff);
        assert!(vars.iter().any(|(n, _)| n == "--primary-color"));
        assert!(fns.iter().any(|(n, _)| n == ".container"));
    }
}
