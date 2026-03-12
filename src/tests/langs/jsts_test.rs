use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    JsTsExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn variables() {
    let e = extract("export const API_KEY = 'abc';");
    assert_eq!(e.variables, vec!["API_KEY"]);
}

#[test]
fn function_declaration() {
    let e = extract("export function processData(input) {");
    assert_eq!(e.functions, vec!["processData"]);
}

#[test]
fn arrow_function() {
    let e = extract("const handler = async (req, res) => {");
    assert_eq!(e.functions, vec!["handler"]);
}

#[test]
fn type_declaration() {
    let e = extract("export interface UserProfile {");
    assert_eq!(e.functions, vec!["UserProfile"]);
}

#[test]
fn test_block() {
    let e = extract("  describe('UserService', () => {");
    assert_eq!(e.tests, vec!["UserService"]);
}

#[test]
fn skip_destructuring() {
    let e = extract("const { x, y } = point;");
    assert!(e.variables.is_empty());
}
