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
