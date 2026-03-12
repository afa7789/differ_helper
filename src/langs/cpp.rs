//! C++ language extractor.
//!
//! Extends C extraction with support for `class`, `namespace`, `template`,
//! `constexpr`, and C++ test frameworks (Google Test, Catch2, Boost.Test).

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct CppExtractor;

/// Reuse C extractor for the base cases.
use super::c::CExtractor;

impl Extractor for CppExtractor {
    fn extract_line(&self, line: &str, state: &mut ExtractorState) -> Extracted {
        // Start with C-level extraction (macros, structs, functions, etc.).
        let mut out = CExtractor.extract_line(line, state);

        let trimmed = line.trim();

        // C++-specific: class, namespace, constexpr, template declarations.
        extract_cpp_keywords(trimmed, &mut out);

        // C++ test frameworks.
        extract_cpp_tests(trimmed, &mut out);

        out
    }
}

/// Extract C++-specific keywords: `class`, `namespace`, `constexpr`.
fn extract_cpp_keywords(trimmed: &str, out: &mut Extracted) {
    for kw in ["class ", "namespace "] {
        if let Some(after) = trimmed.strip_prefix(kw) {
            if let Some(name) = ident::prefix(after) {
                if !out.functions.contains(&name.to_string()) {
                    out.functions.push(name.to_string());
                }
            }
        }
    }

    // `constexpr auto NAME = ...` or `constexpr int NAME = ...`.
    if let Some(after) = trimmed.strip_prefix("constexpr ") {
        if let Some(eq_pos) = after.find('=') {
            let before_eq = after[..eq_pos].trim();
            let last_space = before_eq.rfind(' ').unwrap_or(0);
            let candidate = if last_space > 0 {
                &before_eq[last_space + 1..]
            } else {
                before_eq
            };
            if let Some(name) = ident::prefix(candidate) {
                out.variables.push(name.to_string());
            }
        }
    }
}

/// Extract test names from C++ test frameworks.
///
/// Supports: `TEST(suite, name)`, `TEST_F(suite, name)`, `TEST_P(suite, name)`,
/// `TEST_CASE("name")` (Catch2), `BOOST_AUTO_TEST_CASE(name)`.
fn extract_cpp_tests(trimmed: &str, out: &mut Extracted) {
    // Google Test: TEST(Suite, Name), TEST_F(Suite, Name), TEST_P(Suite, Name).
    for prefix in ["TEST(", "TEST_F(", "TEST_P("] {
        if let Some(inner) = trimmed.strip_prefix(prefix) {
            if let Some(close) = inner.find(')') {
                let args = &inner[..close];
                let parts: Vec<&str> = args.split(',').collect();
                if parts.len() == 2 {
                    let suite = parts[0].trim();
                    let name = parts[1].trim();
                    out.tests.push(format!("{suite}.{name}"));
                }
            }
        }
    }

    // Catch2: TEST_CASE("description").
    if let Some(after) = trimmed.strip_prefix("TEST_CASE(") {
        if let Some(name) = ident::extract_string_arg(after) {
            out.tests.push(name.to_string());
        }
    }

    // Boost.Test: BOOST_AUTO_TEST_CASE(name).
    if let Some(after) = trimmed.strip_prefix("BOOST_AUTO_TEST_CASE(") {
        if let Some(name) = ident::prefix(after) {
            out.tests.push(name.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::ExtractorState;

    fn extract(line: &str) -> Extracted {
        CppExtractor.extract_line(line, &mut ExtractorState::default())
    }

    #[test]
    fn class_def() {
        let e = extract("class Widget {");
        assert_eq!(e.functions, vec!["Widget"]);
    }

    #[test]
    fn namespace_def() {
        let e = extract("namespace detail {");
        assert_eq!(e.functions, vec!["detail"]);
    }

    #[test]
    fn constexpr_var() {
        let e = extract("constexpr int MAX_SIZE = 1024;");
        assert_eq!(e.variables, vec!["MAX_SIZE"]);
    }

    #[test]
    fn google_test() {
        let e = extract("TEST(ParserSuite, HandlesEmpty) {");
        assert_eq!(e.tests, vec!["ParserSuite.HandlesEmpty"]);
    }

    #[test]
    fn google_test_f() {
        let e = extract("TEST_F(ServerFixture, StartsCleanly) {");
        assert_eq!(e.tests, vec!["ServerFixture.StartsCleanly"]);
    }

    #[test]
    fn catch2_test_case() {
        let e = extract("TEST_CASE(\"vectors can be sized and resized\") {");
        assert_eq!(e.tests, vec!["vectors can be sized and resized"]);
    }

    #[test]
    fn boost_test() {
        let e = extract("BOOST_AUTO_TEST_CASE(test_addition) {");
        assert_eq!(e.tests, vec!["test_addition"]);
    }

    #[test]
    fn inherits_c_define() {
        let e = extract("#define BUFFER_SIZE 4096");
        assert_eq!(e.variables, vec!["BUFFER_SIZE"]);
    }

    #[test]
    fn inherits_c_function() {
        let e = extract("void render(Scene* scene) {");
        assert_eq!(e.functions, vec!["render"]);
    }
}
