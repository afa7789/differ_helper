//! Per-language extraction modules.
//!
//! Each module implements [`Extractor`] for its target language.

pub mod c;
pub mod cpp;
pub mod cs;
pub mod css;
pub mod go;
pub mod haskell;
pub mod java;
pub mod jsts;
pub mod kotlin;
pub mod lua;
pub mod masm;
pub mod nim;
pub mod objc;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod shell;
pub mod sql;
pub mod swift;
pub mod terraform;
pub mod yaml;
pub mod zig;

use crate::extract::Extractor;
use crate::lang::Lang;

/// Return the appropriate extractor for a given language.
///
/// Returns `None` for `Lang::Unknown` since no extraction is possible.
pub fn extractor_for(lang: Lang) -> Option<Box<dyn Extractor>> {
    match lang {
        Lang::Rust => Some(Box::new(rust::RustExtractor)),
        Lang::Masm => Some(Box::new(masm::MasmExtractor)),
        Lang::JsTs => Some(Box::new(jsts::JsTsExtractor)),
        Lang::Css => Some(Box::new(css::CssExtractor)),
        Lang::Sql => Some(Box::new(sql::SqlExtractor)),
        Lang::Python => Some(Box::new(python::PythonExtractor)),
        Lang::Go => Some(Box::new(go::GoExtractor)),
        Lang::C => Some(Box::new(c::CExtractor)),
        Lang::Cpp => Some(Box::new(cpp::CppExtractor)),
        Lang::Java => Some(Box::new(java::JavaExtractor)),
        Lang::Kotlin => Some(Box::new(kotlin::KotlinExtractor)),
        Lang::Swift => Some(Box::new(swift::SwiftExtractor)),
        Lang::ObjC => Some(Box::new(objc::ObjCExtractor)),
        Lang::Cs => Some(Box::new(cs::CsExtractor)),
        Lang::Ruby => Some(Box::new(ruby::RubyExtractor)),
        Lang::Php => Some(Box::new(php::PhpExtractor)),
        Lang::Shell => Some(Box::new(shell::ShellExtractor)),
        Lang::Lua => Some(Box::new(lua::LuaExtractor)),
        Lang::Terraform => Some(Box::new(terraform::TerraformExtractor)),
        Lang::Yaml => Some(Box::new(yaml::YamlExtractor)),
        Lang::Zig => Some(Box::new(zig::ZigExtractor)),
        Lang::Haskell => Some(Box::new(haskell::HaskellExtractor)),
        Lang::Nim => Some(Box::new(nim::NimExtractor)),
        Lang::Unknown => None,
    }
}
