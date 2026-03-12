use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    let ext = RustExtractor;
    let mut state = ExtractorState::default();
    ext.extract_line(line, &mut state)
}

#[test]
fn variables() {
    let e = extract("    let x = 5;");
    assert_eq!(e.variables, vec!["x"]);
}

#[test]
fn let_mut() {
    let e = extract("    let mut count = 0;");
    assert_eq!(e.variables, vec!["count"]);
}

#[test]
fn functions() {
    let e = extract("pub fn process(data: &str) {");
    assert_eq!(e.functions, vec!["process"]);
}

#[test]
fn async_fn() {
    let e = extract("async fn fetch_data() {");
    assert_eq!(e.functions, vec!["fetch_data"]);
}

#[test]
fn test_single_line() {
    let e = extract("#[test] fn it_works() {");
    assert_eq!(e.tests, vec!["it_works"]);
}

#[test]
fn test_multi_line() {
    let ext = RustExtractor;
    let mut state = ExtractorState::default();
    ext.extract_line("#[test]", &mut state);
    assert!(state.in_test_block);
    let e = ext.extract_line("fn it_works() {", &mut state);
    assert_eq!(e.tests, vec!["it_works"]);
    assert!(!state.in_test_block);
}
