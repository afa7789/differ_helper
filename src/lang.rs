/// Supported programming languages, identified by file extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Masm,
    JsTs,
    Css,
    Sql,
    Python,
    Go,
    C,
    Cpp,
    Java,
    Kotlin,
    Swift,
    ObjC,
    Cs,
    Unknown,
}

/// Map a file path to its language based on extension.
pub fn detect(file: &str) -> Lang {
    // Extract extension (last `.xxx` segment, lowercased for safety).
    let ext = file.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => Lang::Rust,
        "masm" => Lang::Masm,
        "ts" | "tsx" | "js" | "jsx" | "vue" | "svelte" | "astro" => Lang::JsTs,
        "css" => Lang::Css,
        "sql" => Lang::Sql,
        "py" | "pyi" => Lang::Python,
        "go" => Lang::Go,
        "c" | "h" => Lang::C,
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => Lang::Cpp,
        "java" => Lang::Java,
        "kt" | "kts" => Lang::Kotlin,
        "swift" => Lang::Swift,
        "m" | "mm" => Lang::ObjC,
        "cs" => Lang::Cs,
        _ => Lang::Unknown,
    }
}

#[cfg(test)]
#[path = "tests/lang_test.rs"]
mod tests;
