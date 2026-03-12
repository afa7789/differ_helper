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

/// Helper: build a diff with multiple files.
fn make_multi_diff(files: &[(&str, &[&str])]) -> String {
    let mut diff = String::new();
    for (filename, lines) in files {
        diff.push_str(&make_diff(filename, lines));
    }
    diff
}

// ── splitting ──────────────────────────────────────────────

#[test]
fn split_single_file() {
    let diff = make_diff("src/lib.rs", &["let x = 1;"]);
    let sections = split_into_sections(&diff);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].path, "src/lib.rs");
    assert_eq!(sections[0].added_lines.len(), 1);
}

#[test]
fn split_multiple_files() {
    let diff = make_multi_diff(&[
        ("a.rs", &["let x = 1;"]),
        ("b.py", &["y = 2"]),
        ("c.go", &["var z int"]),
    ]);
    let sections = split_into_sections(&diff);
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].path, "a.rs");
    assert_eq!(sections[1].path, "b.py");
    assert_eq!(sections[2].path, "c.go");
}

#[test]
fn split_dev_null_skipped() {
    let diff = "diff --git a/old.rs b/old.rs\n+++ /dev/null\n+let x = 1;\n";
    let sections = split_into_sections(diff);
    // Section exists but no added lines (dev/null filtered).
    assert!(sections.iter().all(|s| s.added_lines.is_empty()));
}

// ── per-language end-to-end ────────────────────────────────

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
fn css_end_to_end() {
    let diff = make_diff("styles.css", &[".container {", "  --primary-color: #333;"]);
    let (vars, fns, _) = parse_diff(&diff);
    assert!(vars.iter().any(|(n, _)| n == "--primary-color"));
    assert!(fns.iter().any(|(n, _)| n == ".container"));
}

#[test]
fn masm_end_to_end() {
    let diff = make_diff(
        "kernel.masm",
        &[
            "const MAX = 100",
            "pub proc compute(x: felt)",
            "use miden::crypto::hash",
        ],
    );
    let (vars, fns, _) = parse_diff(&diff);
    assert!(vars.iter().any(|(n, _)| n == "MAX"));
    assert!(fns.iter().any(|(n, _)| n == "compute"));
    assert!(fns.iter().any(|(n, _)| n == "miden::crypto::hash"));
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

// ── parallel processing ────────────────────────────────────

#[test]
fn multi_file_parallel_extraction() {
    let diff = make_multi_diff(&[
        ("src/lib.rs", &["pub fn alpha() {", "let a = 1;"]),
        ("app.py", &["def beta():", "X = 10"]),
        ("cmd/main.go", &["func Gamma() {", "var g int"]),
        ("lib/util.c", &["#define D 1", "void delta(void) {"]),
        ("src/w.cpp", &["class Epsilon {", "TEST(S, T) {"]),
    ]);
    let (vars, fns, tests) = parse_diff(&diff);

    // Check all files contributed.
    assert!(fns.iter().any(|(n, _)| n == "alpha"));
    assert!(fns.iter().any(|(n, _)| n == "beta"));
    assert!(fns.iter().any(|(n, _)| n == "Gamma"));
    assert!(fns.iter().any(|(n, _)| n == "delta"));
    assert!(fns.iter().any(|(n, _)| n == "Epsilon"));

    assert!(vars.iter().any(|(n, _)| n == "a"));
    assert!(vars.iter().any(|(n, _)| n == "X"));
    assert!(vars.iter().any(|(n, _)| n == "g"));
    assert!(vars.iter().any(|(n, _)| n == "D"));

    assert!(tests.iter().any(|(n, _)| n == "S.T"));
}

#[test]
fn extract_section_unknown_lang() {
    let section = FileSection {
        path: "notes.txt".to_string(),
        added_lines: vec!["+some random text\n".to_string()],
    };
    let (v, f, t) = extract_section(&section);
    assert!(v.is_empty());
    assert!(f.is_empty());
    assert!(t.is_empty());
}

// ── jsx/tsx support ────────────────────────────────────────

#[test]
fn jsx_tsx_end_to_end() {
    let diff = make_multi_diff(&[
        (
            "components/Button.tsx",
            &["export function Button() {", "const styles = {};"],
        ),
        (
            "pages/Home.jsx",
            &["export default function Home() {", "let count = 0;"],
        ),
    ]);
    let (vars, fns, _) = parse_diff(&diff);
    assert!(fns
        .iter()
        .any(|(n, f)| n == "Button" && f.ends_with(".tsx")));
    assert!(fns.iter().any(|(n, f)| n == "Home" && f.ends_with(".jsx")));
    assert!(vars.iter().any(|(n, _)| n == "count"));
}
