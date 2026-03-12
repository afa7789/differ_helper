use super::*;

#[test]
fn ident_prefix_basic() {
    assert_eq!(prefix("foo_bar"), Some("foo_bar"));
    assert_eq!(prefix("_private"), Some("_private"));
    assert_eq!(prefix("$jquery"), Some("$jquery"));
    assert_eq!(prefix("123bad"), None);
    assert_eq!(prefix(""), None);
}

#[test]
fn ident_prefix_stops_at_special() {
    assert_eq!(prefix("name(arg)"), Some("name"));
    assert_eq!(prefix("x = 1"), Some("x"));
}

#[test]
fn string_arg_extraction() {
    assert_eq!(extract_string_arg("'hello'"), Some("hello"));
    assert_eq!(extract_string_arg("\"world\""), Some("world"));
    assert_eq!(extract_string_arg("`tmpl`"), Some("tmpl"));
    assert_eq!(extract_string_arg("''"), None);
    assert_eq!(extract_string_arg("no quotes"), None);
}

#[test]
fn find_patterns() {
    let line = "let x = 1; const y = 2;";
    let matches = find_all_after_patterns(line, &["let ", "const "]);
    let names: Vec<&str> = matches.into_iter().map(|(_, n)| n).collect();
    assert_eq!(names, vec!["x", "y"]);
}
