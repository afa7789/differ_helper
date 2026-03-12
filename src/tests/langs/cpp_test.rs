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

#[test]
fn constexpr_no_space_in_type() {
    let e = extract("constexpr MAX = 42;");
    assert_eq!(e.variables, vec!["MAX"]);
}

#[test]
fn test_p() {
    let e = extract("TEST_P(ParamSuite, Works) {");
    assert_eq!(e.tests, vec!["ParamSuite.Works"]);
}

#[test]
fn class_no_duplicate_with_c_fn() {
    let e = extract("class Foo {");
    assert_eq!(e.functions, vec!["Foo"]);
}
