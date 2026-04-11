use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    HaskellExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn function_def() {
    let e = extract("add :: Int -> Int -> Int");
    assert_eq!(e.functions, vec!["add"]);
}

#[test]
fn function_binding() {
    let e = extract("add x y = x + y");
    assert_eq!(e.functions, vec!["add"]);
}

#[test]
fn pattern_matching() {
    let e = extract("fib 0 = 0");
    assert_eq!(e.functions, vec!["fib"]);
}

#[test]
fn data_declaration() {
    let e = extract("data Maybe a = Nothing | Just a");
    assert_eq!(e.functions, vec!["Maybe"]);
}

#[test]
fn newtype_declaration() {
    let e = extract("newtype Identity a = Identity a");
    assert_eq!(e.functions, vec!["Identity"]);
}

#[test]
fn type_declaration() {
    let e = extract("type Alias = Int");
    assert_eq!(e.functions, vec!["Alias"]);
}

#[test]
fn class_declaration() {
    let e = extract("class Functor f where");
    assert_eq!(e.functions, vec!["Functor"]);
}

#[test]
fn let_binding() {
    let e = extract("let x = 5 in x");
    assert_eq!(e.variables, vec!["x"]);
}

#[test]
fn import_qualified() {
    let e = extract("import qualified Data.List as L");
    assert_eq!(e.imports, vec!["Data.List"]);
}

#[test]
fn import_simple() {
    let e = extract("import Data.Text");
    assert_eq!(e.imports, vec!["Data.Text"]);
}

#[test]
fn hspec_test() {
    let e = extract("it \"should add correctly\" $ do");
    assert_eq!(e.tests, vec!["should add correctly"]);
}

#[test]
fn quickcheck_prop() {
    let e = extract("prop_reverse :: [Int] -> Property");
    assert_eq!(e.tests, vec!["prop_reverse"]);
}
