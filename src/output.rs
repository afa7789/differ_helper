//! Output formatting: deduplication and sorted printing of extracted symbols.

use std::collections::HashSet;
use std::io::{self, Write};

use crate::security::Warning;

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

/// Print security warnings, deduplicated by (pattern, file).
pub fn print_warnings(w: &mut dyn Write, warnings: &[Warning]) -> io::Result<()> {
    let mut seen = HashSet::new();
    let mut deduped: Vec<(&str, &str)> = Vec::new();
    for warning in warnings {
        if seen.insert((warning.pattern, warning.file.as_str())) {
            deduped.push((warning.pattern, warning.file.as_str()));
        }
    }
    deduped.sort_by(|a, b| a.1.cmp(b.1).then(a.0.cmp(b.0)));

    writeln!(w, "WARNINGS:")?;
    for (pattern, file) in &deduped {
        writeln!(w, "- {pattern} -> {file}")?;
    }
    writeln!(w)?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/output_test.rs"]
mod tests;
