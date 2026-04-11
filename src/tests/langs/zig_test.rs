use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    ZigExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn const_declaration() {
    let e = extract("const max_retries: u32 = 3;");
    assert_eq!(e.variables, vec!["max_retries"]);
}

#[test]
fn var_declaration() {
    let e = extract("var buffer: [100]u8 = undefined;");
    assert_eq!(e.variables, vec!["buffer"]);
}

#[test]
fn function_def() {
    let e = extract("pub fn process(data: []const u8) !void {");
    assert_eq!(e.functions, vec!["process"]);
}

#[test]
fn function_def_no_pub() {
    let e = extract("fn calculate(a: i32, b: i32) i32 {");
    assert_eq!(e.functions, vec!["calculate"]);
}

#[test]
fn struct_def() {
    let e = extract("const Config = struct {");
    assert_eq!(e.functions, vec!["Config"]);
}

#[test]
fn enum_def() {
    let e = extract("const Status = enum {");
    assert_eq!(e.functions, vec!["Status"]);
}

#[test]
fn union_def() {
    let e = extract("const Result = union(enum) {");
    assert_eq!(e.functions, vec!["Result"]);
}

#[test]
fn type_def() {
    let e = extract("const IntList = type;");
    assert_eq!(e.functions, vec!["IntList"]);
}

#[test]
fn test_declaration() {
    let e = extract("test \"addition works\" {");
    assert_eq!(e.tests, vec!["addition works"]);
}

#[test]
fn import_statement() {
    let e = extract("@import(\"std\").foo;");
    assert_eq!(e.imports, vec!["std"]);
}
