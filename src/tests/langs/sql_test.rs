use super::*;
use crate::extract::ExtractorState;

fn extract(line: &str) -> Extracted {
    SqlExtractor.extract_line(line, &mut ExtractorState::default())
}

#[test]
fn create_table() {
    let e = extract("CREATE TABLE users (");
    assert_eq!(e.functions, vec!["users"]);
}

#[test]
fn create_if_not_exists() {
    let e = extract("CREATE TABLE IF NOT EXISTS orders (");
    assert_eq!(e.functions, vec!["orders"]);
}

#[test]
fn schema_qualified() {
    let e = extract("CREATE VIEW public.active_users AS");
    assert_eq!(e.functions, vec!["public.active_users"]);
}

#[test]
fn drop_statement() {
    let e = extract("DROP TABLE IF EXISTS legacy_data;");
    assert_eq!(e.functions, vec!["legacy_data"]);
}

#[test]
fn quoted_identifier() {
    let e = extract("CREATE TABLE \"My Table\" (");
    assert_eq!(e.functions, vec!["\"My Table\""]);
}

#[test]
fn quoted_schema_qualified() {
    let e = extract("CREATE TABLE \"public\".\"user-data\" (");
    assert_eq!(e.functions, vec!["\"public\".\"user-data\""]);
}

#[test]
fn empty_after_create() {
    let e = extract("CREATE TABLE ");
    assert!(e.functions.is_empty());
}

#[test]
fn case_insensitive() {
    let e = extract("create table accounts (");
    assert_eq!(e.functions, vec!["accounts"]);
}
