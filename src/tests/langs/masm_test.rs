use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    MasmExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn constants() {
    let e = extract("const MAX_SIZE = 100");
    assert_eq!(e.variables, vec!["MAX_SIZE"]);
}

#[test]
fn procedures() {
    let e = extract("pub proc compute(x: felt)");
    assert_eq!(e.functions, vec!["compute"]);
}

#[test]
fn use_imports() {
    let e = extract("use miden::crypto::hash");
    assert_eq!(e.functions, vec!["miden::crypto::hash"]);
}

#[test]
fn const_invalid_name() {
    let e = extract("const @invalid = 1");
    assert!(e.variables.is_empty());
}

#[test]
fn const_empty_name() {
    let e = extract("const  = 1");
    assert!(e.variables.is_empty());
}

#[test]
fn use_not_at_line_start() {
    let e = extract("  foo use bar::baz");
    assert!(e.functions.is_empty());
}

#[test]
fn empty_ident() {
    let e = extract("proc ");
    assert!(e.functions.is_empty());
}

#[test]
fn pub_use_import() {
    let e = extract("pub use miden::stdlib::crypto");
    assert_eq!(e.functions, vec!["miden::stdlib::crypto"]);
}
