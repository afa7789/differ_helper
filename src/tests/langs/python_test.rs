use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    PythonExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn module_variable() {
    let e = extract("MAX_RETRIES = 3");
    assert_eq!(e.variables, vec!["MAX_RETRIES"]);
}

#[test]
fn type_annotated_variable() {
    let e = extract("name: str = 'hello'");
    assert_eq!(e.variables, vec!["name"]);
}

#[test]
fn function_def() {
    let e = extract("def process_data(items):");
    assert_eq!(e.functions, vec!["process_data"]);
}

#[test]
fn async_function_def() {
    let e = extract("async def fetch_data(url):");
    assert_eq!(e.functions, vec!["fetch_data"]);
}

#[test]
fn class_def() {
    let e = extract("class UserService:");
    assert_eq!(e.functions, vec!["UserService"]);
}

#[test]
fn pytest_test() {
    let e = extract("def test_user_creation():");
    assert_eq!(e.tests, vec!["test_user_creation"]);
    // Also appears as function.
    assert_eq!(e.functions, vec!["test_user_creation"]);
}

#[test]
fn unittest_class() {
    let e = extract("class TestUserService(unittest.TestCase):");
    assert_eq!(e.tests, vec!["TestUserService"]);
}

#[test]
fn skip_local_variable() {
    let e = extract("        result = compute()");
    assert!(e.variables.is_empty());
}

#[test]
fn skip_import() {
    let e = extract("from os import path");
    assert!(e.variables.is_empty());
}

#[test]
fn skip_comparison() {
    let e = extract("if x == 5:");
    assert!(e.variables.is_empty());
}
