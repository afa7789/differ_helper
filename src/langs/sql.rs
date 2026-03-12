//! SQL language extractor.
//!
//! Extracts object names from DDL statements (CREATE, ALTER, DROP).

use crate::extract::{Extracted, Extractor, ExtractorState};

pub struct SqlExtractor;

impl Extractor for SqlExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();
        for name in object_names(line) {
            out.functions.push(name);
        }
        out
    }
}

/// DDL patterns to match (case-insensitive via uppercased comparison).
const DDL_PATTERNS: &[&str] = &[
    "CREATE TABLE ",
    "CREATE OR REPLACE TABLE ",
    "ALTER TABLE ",
    "DROP TABLE ",
    "CREATE INDEX ",
    "CREATE UNIQUE INDEX ",
    "DROP INDEX ",
    "CREATE FUNCTION ",
    "CREATE OR REPLACE FUNCTION ",
    "DROP FUNCTION ",
    "CREATE VIEW ",
    "CREATE OR REPLACE VIEW ",
    "DROP VIEW ",
    "CREATE TRIGGER ",
    "CREATE OR REPLACE TRIGGER ",
    "DROP TRIGGER ",
    "CREATE TYPE ",
    "CREATE POLICY ",
    "DROP POLICY ",
    "CREATE EXTENSION ",
    "DROP EXTENSION ",
    "CREATE SCHEMA ",
    "DROP SCHEMA ",
];

/// Extract SQL object names from DDL statements.
fn object_names(line: &str) -> Vec<String> {
    let upper = line.to_ascii_uppercase();
    let mut out = Vec::new();

    for pat in DDL_PATTERNS {
        let mut start = 0;
        while let Some(pos) = upper[start..].find(pat) {
            let abs = start + pos;
            let after = &line[abs + pat.len()..];
            if let Some(name) = sql_ident(after) {
                out.push(name);
            }
            start = abs + pat.len();
        }
    }
    out
}

/// Extract a SQL identifier (possibly schema-qualified and/or quoted).
fn sql_ident(s: &str) -> Option<String> {
    let s = s.trim_start();

    // Handle `IF NOT EXISTS` / `IF EXISTS` clauses.
    let s = if s.to_ascii_uppercase().starts_with("IF NOT EXISTS ") {
        s[14..].trim_start()
    } else if s.to_ascii_uppercase().starts_with("IF EXISTS ") {
        s[10..].trim_start()
    } else {
        s
    };

    let mut result = String::new();
    let mut rest = s;

    loop {
        if rest.starts_with('"') {
            // Quoted identifier.
            let inner = &rest[1..];
            let end = inner.find('"')?;
            result.push_str(&rest[..end + 2]);
            rest = &inner[end + 1..];
        } else {
            // Unquoted identifier.
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if end == 0 {
                break;
            }
            result.push_str(&rest[..end]);
            rest = &rest[end..];
        }

        // Check for schema.name continuation.
        if rest.starts_with('.') {
            result.push('.');
            rest = &rest[1..];
        } else {
            break;
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod tests {
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
}
