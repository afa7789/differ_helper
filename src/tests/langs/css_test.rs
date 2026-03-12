use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    CssExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn custom_properties() {
    let e = extract("  --primary-color: #333;");
    assert_eq!(e.variables, vec!["--primary-color"]);
}

#[test]
fn class_selector() {
    let e = extract(".container {");
    assert_eq!(e.functions, vec![".container"]);
}

#[test]
fn id_selector() {
    let e = extract("#main-header {");
    assert_eq!(e.functions, vec!["#main-header"]);
}
