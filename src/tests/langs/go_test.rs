use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    GoExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn var_declaration() {
    let e = extract("var maxRetries int = 3");
    assert_eq!(e.variables, vec!["maxRetries"]);
}

#[test]
fn const_declaration() {
    let e = extract("const DefaultTimeout = 30");
    assert_eq!(e.variables, vec!["DefaultTimeout"]);
}

#[test]
fn short_declaration() {
    let e = extract("result := compute()");
    assert_eq!(e.variables, vec!["result"]);
}

#[test]
fn function_def() {
    let e = extract("func ProcessData(items []Item) error {");
    assert_eq!(e.functions, vec!["ProcessData"]);
}

#[test]
fn method_def() {
    let e = extract("func (s *Server) HandleRequest(w http.ResponseWriter) {");
    assert_eq!(e.functions, vec!["HandleRequest"]);
}

#[test]
fn type_def() {
    let e = extract("type Config struct {");
    assert_eq!(e.functions, vec!["Config"]);
}

#[test]
fn test_function() {
    let e = extract("func TestParseConfig(t *testing.T) {");
    assert_eq!(e.tests, vec!["TestParseConfig"]);
    assert_eq!(e.functions, vec!["TestParseConfig"]);
}

#[test]
fn benchmark_function() {
    let e = extract("func BenchmarkSort(b *testing.B) {");
    assert_eq!(e.tests, vec!["BenchmarkSort"]);
}

#[test]
fn var_block_opener() {
    let e = extract("var (");
    assert!(e.variables.is_empty());
}
