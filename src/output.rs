//! Output formatting: deduplication and sorted printing of extracted symbols.

use std::collections::HashSet;
use std::io::{self, Write};

/// Deduplicate `(name, file)` pairs, sort by file then name, and write to `w`.
pub fn dedup_and_write(
    w: &mut dyn Write,
    section: &str,
    items: &mut Vec<(String, String)>,
) -> io::Result<()> {
    let mut seen = HashSet::new();
    items.retain(|(n, f)| seen.insert((n.clone(), f.clone())));
    items.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    writeln!(w, "{section}:")?;
    for (name, path) in items.iter() {
        writeln!(w, "- {name} -> {path}")?;
    }
    writeln!(w)?;
    Ok(())
}

/// Convenience wrapper that prints to stdout.
pub fn dedup_and_print(section: &str, items: &mut Vec<(String, String)>) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    dedup_and_write(&mut handle, section, items).expect("failed to write to stdout");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_removes_duplicates() {
        let mut items = vec![
            ("foo".to_string(), "a.rs".to_string()),
            ("foo".to_string(), "a.rs".to_string()),
            ("bar".to_string(), "b.rs".to_string()),
        ];
        let mut buf = Vec::new();
        dedup_and_write(&mut buf, "TEST", &mut items).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("TEST:"));
        assert!(output.contains("- foo -> a.rs"));
        assert!(output.contains("- bar -> b.rs"));
        // Only 2 entries (deduped).
        assert_eq!(output.matches("- ").count(), 2);
    }

    #[test]
    fn sort_by_file_then_name() {
        let mut items = vec![
            ("z".to_string(), "b.rs".to_string()),
            ("a".to_string(), "b.rs".to_string()),
            ("m".to_string(), "a.rs".to_string()),
        ];
        let mut buf = Vec::new();
        dedup_and_write(&mut buf, "FUNCS", &mut items).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().filter(|l| l.starts_with("- ")).collect();
        assert_eq!(lines[0], "- m -> a.rs");
        assert_eq!(lines[1], "- a -> b.rs");
        assert_eq!(lines[2], "- z -> b.rs");
    }

    #[test]
    fn empty_items() {
        let mut items: Vec<(String, String)> = Vec::new();
        let mut buf = Vec::new();
        dedup_and_write(&mut buf, "EMPTY", &mut items).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, "EMPTY:\n\n");
    }
}
