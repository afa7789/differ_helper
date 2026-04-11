use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    NimExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn const_declaration() {
    let e = extract("const MaxRetries = 3");
    assert_eq!(e.variables, vec!["MaxRetries"]);
}

#[test]
fn let_declaration() {
    let e = extract("let data = readFile(path)");
    assert_eq!(e.variables, vec!["data"]);
}

#[test]
fn var_declaration() {
    let e = extract("var counter: int = 0");
    assert_eq!(e.variables, vec!["counter"]);
}

#[test]
fn proc_def() {
    let e = extract("proc process*(data: string) {.inline.} =");
    assert_eq!(e.functions, vec!["process"]);
}

#[test]
fn func_def() {
    let e = extract("func add(a, b: int): int = a + b");
    assert_eq!(e.functions, vec!["add"]);
}

#[test]
fn method_def() {
    let e = extract("method handle(event: Event) =");
    assert_eq!(e.functions, vec!["handle"]);
}

#[test]
fn iterator_def() {
    let e = extract("iterator items*[T](seq: seq[T]): T =");
    assert_eq!(e.functions, vec!["items"]);
}

#[test]
fn template_def() {
    let e = extract("template assertEq(a, b: untyped) =");
    assert_eq!(e.functions, vec!["assertEq"]);
}

#[test]
fn macro_def() {
    let e = extract("macro myMacro*(sym: NimNode): NimNode =");
    assert_eq!(e.functions, vec!["myMacro"]);
}

#[test]
fn type_def() {
    let e = extract("type MyObject = object");
    assert_eq!(e.functions, vec!["MyObject"]);
}

#[test]
fn enum_def() {
    let e = extract("enum Color");
    assert_eq!(e.functions, vec!["Color"]);
}

#[test]
fn concept_def() {
    let e = extract("concept Serializable[T]");
    assert_eq!(e.functions, vec!["Serializable"]);
}

#[test]
fn import_statement() {
    let e = extract("import std/[strutils, os]");
    assert_eq!(e.imports, vec!["std"]);
}

#[test]
fn from_import() {
    let e = extract("from std/algorithm import sort");
    assert_eq!(e.imports, vec!["std"]);
}

#[test]
fn include_statement() {
    let e = extract("include std/assertions");
    assert_eq!(e.imports, vec!["std"]);
}

#[test]
fn test_declaration() {
    let e = extract("test \"addition works\":");
    assert_eq!(e.tests, vec!["addition works"]);
}

#[test]
fn suite_declaration() {
    let e = extract("suite \"main\":");
    assert_eq!(e.tests, vec!["main"]);
}
