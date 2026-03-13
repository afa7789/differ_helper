use crate::security;

#[test]
fn detects_hardcoded_password() {
    let lines = vec!["+    let password = \"hunter2\";\n".to_string()];
    let warnings = security::scan_lines(&lines, "src/auth.rs");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].pattern, "hardcoded password");
}

#[test]
fn detects_eval() {
    let lines = vec!["+eval(user_input)\n".to_string()];
    let warnings = security::scan_lines(&lines, "app.js");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].pattern, "dangerous eval()");
}

#[test]
fn detects_todo() {
    let lines = vec!["+    // TODO: fix this later\n".to_string()];
    // Comments are skipped.
    let warnings = security::scan_lines(&lines, "lib.rs");
    assert_eq!(warnings.len(), 0);
}

#[test]
fn detects_raw_sql() {
    let lines = vec!["+    db.execute(\"SELECT * FROM users WHERE id = \" + id)\n".to_string()];
    let warnings = security::scan_lines(&lines, "db.py");
    assert!(!warnings.is_empty());
}

#[test]
fn skips_clean_lines() {
    let lines = vec![
        "+let count = 0;\n".to_string(),
        "+fn process() {}\n".to_string(),
    ];
    let warnings = security::scan_lines(&lines, "lib.rs");
    assert_eq!(warnings.len(), 0);
}

#[test]
fn detects_api_key() {
    let lines = vec!["+const api_key = \"abc123\";\n".to_string()];
    let warnings = security::scan_lines(&lines, "config.ts");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].pattern, "hardcoded API key");
}

#[test]
fn detects_innerhtml() {
    let lines = vec!["+element.innerHTML = userContent;\n".to_string()];
    let warnings = security::scan_lines(&lines, "ui.js");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].pattern, "potential XSS via innerHTML");
}
