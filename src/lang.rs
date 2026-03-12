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
    Unknown,
}

/// Map a file path to its language based on extension.
pub fn detect(file: &str) -> Lang {
    // Extract extension (last `.xxx` segment, lowercased for safety).
    let ext = file.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => Lang::Rust,
        "masm" => Lang::Masm,
        "ts" | "tsx" | "js" | "jsx" => Lang::JsTs,
        "css" => Lang::Css,
        "sql" => Lang::Sql,
        "py" | "pyi" => Lang::Python,
        "go" => Lang::Go,
        "c" | "h" => Lang::C,
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => Lang::Cpp,
        _ => Lang::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_rust() {
        assert_eq!(detect("src/main.rs"), Lang::Rust);
    }

    #[test]
    fn detect_python() {
        assert_eq!(detect("app/models.py"), Lang::Python);
        assert_eq!(detect("stubs/types.pyi"), Lang::Python);
    }

    #[test]
    fn detect_go() {
        assert_eq!(detect("cmd/server.go"), Lang::Go);
    }

    #[test]
    fn detect_c_cpp() {
        assert_eq!(detect("lib/parser.c"), Lang::C);
        assert_eq!(detect("lib/parser.h"), Lang::C);
        assert_eq!(detect("lib/parser.cpp"), Lang::Cpp);
        assert_eq!(detect("lib/parser.hpp"), Lang::Cpp);
        assert_eq!(detect("lib/parser.cc"), Lang::Cpp);
    }

    #[test]
    fn detect_jsts() {
        assert_eq!(detect("app.ts"), Lang::JsTs);
        assert_eq!(detect("app.tsx"), Lang::JsTs);
        assert_eq!(detect("app.js"), Lang::JsTs);
        assert_eq!(detect("app.jsx"), Lang::JsTs);
    }

    #[test]
    fn detect_unknown() {
        assert_eq!(detect("README.md"), Lang::Unknown);
        assert_eq!(detect("Makefile"), Lang::Unknown);
    }
}
