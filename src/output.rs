//! Output formatting: deduplication and sorted printing of extracted symbols.

use std::collections::HashSet;

/// Deduplicate `(name, file)` pairs and print them sorted by file, then name.
pub fn dedup_and_print(section: &str, items: &mut Vec<(String, String)>) {
    let mut seen = HashSet::new();
    items.retain(|(n, f)| seen.insert((n.clone(), f.clone())));
    items.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    println!("{section}:");
    for (name, path) in items.iter() {
        println!("- {name} -> {path}");
    }
    println!();
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
        let mut seen = HashSet::new();
        items.retain(|(n, f)| seen.insert((n.clone(), f.clone())));
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn sort_by_file_then_name() {
        let mut items = vec![
            ("z".to_string(), "b.rs".to_string()),
            ("a".to_string(), "b.rs".to_string()),
            ("m".to_string(), "a.rs".to_string()),
        ];
        items.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        assert_eq!(items[0], ("m".to_string(), "a.rs".to_string()));
        assert_eq!(items[1], ("a".to_string(), "b.rs".to_string()));
        assert_eq!(items[2], ("z".to_string(), "b.rs".to_string()));
    }
}
