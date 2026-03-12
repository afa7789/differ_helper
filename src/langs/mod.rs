//! Per-language extraction modules.
//!
//! Each module implements [`Extractor`] for its target language.

pub mod c;
pub mod cpp;
pub mod css;
pub mod go;
pub mod jsts;
pub mod masm;
pub mod python;
pub mod rust;
pub mod sql;

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
        Lang::Unknown => None,
    }
}
