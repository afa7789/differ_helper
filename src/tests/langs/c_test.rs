use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    CExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn define_macro() {
    let e = extract("#define MAX_SIZE 1024");
    assert_eq!(e.variables, vec!["MAX_SIZE"]);
}

#[test]
fn struct_def() {
    let e = extract("struct Node {");
    assert_eq!(e.functions, vec!["Node"]);
}

#[test]
fn enum_def() {
    let e = extract("enum Color {");
    assert_eq!(e.functions, vec!["Color"]);
}

#[test]
fn typedef_def() {
    let e = extract("typedef unsigned long size_t;");
    assert_eq!(e.functions, vec!["size_t"]);
}

#[test]
fn function_def() {
    let e = extract("int parse_input(const char *buf) {");
    assert_eq!(e.functions, vec!["parse_input"]);
}

#[test]
fn pointer_return_function() {
    let e = extract("char *strdup(const char *s) {");
    assert_eq!(e.functions, vec!["strdup"]);
}

#[test]
fn void_function() {
    let e = extract("void cleanup(void) {");
    assert_eq!(e.functions, vec!["cleanup"]);
}

#[test]
fn define_no_name() {
    let e = extract("#define ");
    assert!(e.variables.is_empty());
}

#[test]
fn union_def() {
    let e = extract("union Data {");
    assert_eq!(e.functions, vec!["Data"]);
}

#[test]
fn skip_control_flow() {
    let e = extract("if (x > 0) {");
    assert!(e.functions.is_empty());
}

#[test]
fn skip_return() {
    let e = extract("return foo(bar);");
    assert!(e.functions.is_empty());
}

#[test]
fn static_function() {
    let e = extract("static int helper(int x) {");
    assert_eq!(e.functions, vec!["helper"]);
}
