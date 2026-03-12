/// Extraction results from a single line of added diff content.
///
/// Each extractor returns owned `String` names so that callers don't need to
/// worry about borrow lifetimes across the diff-parsing loop.
#[derive(Default)]
pub struct Extracted {
    pub variables: Vec<String>,
    pub functions: Vec<String>,
    pub tests: Vec<String>,
}

/// Per-language extraction logic.
///
/// Implementors parse a single added line and return any discovered symbols.
/// The `state` parameter allows stateful extraction across consecutive lines
/// (e.g. Rust's `#[test]` annotation appearing on the line before `fn`).
pub trait Extractor {
    /// Parse `line` and return extracted symbols.
    ///
    /// `state` is language-specific mutable state carried between lines.
    /// Use `()` when no inter-line state is needed.
    fn extract_line(&self, line: &str, state: &mut ExtractorState) -> Extracted;
}

/// Shared mutable state that persists across lines within the same file.
///
/// This avoids forcing each language to define its own state type while
/// keeping the trait object-safe.
#[derive(Default)]
pub struct ExtractorState {
    /// Rust: whether the previous added line contained `#[test]`.
    pub in_test_block: bool,
}
