/// Extract the longest valid identifier prefix from `s`.
///
/// Valid characters: ASCII letters, digits, underscores, and `$` (for JS/TS).
/// The first character must not be a digit.
pub fn prefix(s: &str) -> Option<&str> {
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if i == 0 {
            if c == '_' || c.is_ascii_alphabetic() || c == '$' {
                end = i + c.len_utf8();
            } else {
                return None;
            }
        } else if c == '_' || c.is_ascii_alphanumeric() || c == '$' {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    if end > 0 {
        Some(&s[..end])
    } else {
        None
    }
}

/// Extract the content inside the first pair of matching quotes or backticks.
///
/// Supports single quotes, double quotes, and backtick delimiters.
pub fn extract_string_arg(s: &str) -> Option<&str> {
    let s = s.trim_start();
    let quote = s.chars().next()?;
    if quote != '\'' && quote != '"' && quote != '`' {
        return None;
    }
    let inner = &s[1..];
    let end = inner.find(quote)?;
    let val = &inner[..end];
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

/// Scan `line` for all occurrences of `prefix_pattern` and collect the
/// identifier that immediately follows each match.
///
/// This is the core DRY helper used by most language extractors.
pub fn find_all_after_patterns<'a>(line: &'a str, patterns: &[&str]) -> Vec<(usize, &'a str)> {
    let mut matches: Vec<(usize, &'a str)> = Vec::new();
    for pat in patterns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            let after = &line[abs + pat.len()..];
            if let Some(name) = prefix(after) {
                if !matches.iter().any(|(p, _)| *p == abs) {
                    matches.push((abs, name));
                }
            }
            start = abs + pat.len();
        }
    }
    matches.sort_by_key(|(pos, _)| *pos);
    matches
}

#[cfg(test)]
#[path = "tests/ident_test.rs"]
mod tests;
