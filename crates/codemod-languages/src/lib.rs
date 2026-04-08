//! Language support plugins for codemod-pilot.
//!
//! This crate provides tree-sitter based language adapters for
//! each supported programming language.

pub mod javascript;
pub mod typescript;
#[allow(dead_code)]
mod utils;

use codemod_core::language::LanguageAdapter;

/// Get a language adapter by name.
pub fn get_language(name: &str) -> Option<Box<dyn LanguageAdapter>> {
    match name.to_lowercase().as_str() {
        "typescript" | "ts" => Some(Box::new(typescript::TypeScriptAdapter)),
        "javascript" | "js" => Some(Box::new(javascript::JavaScriptAdapter)),
        _ => None,
    }
}

/// Get all available language names.
pub fn available_languages() -> Vec<&'static str> {
    vec!["typescript", "javascript"]
}

/// Auto-detect language from file extension.
pub fn detect_language(path: &std::path::Path) -> Option<Box<dyn LanguageAdapter>> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "ts" | "tsx" => Some(Box::new(typescript::TypeScriptAdapter)),
        "js" | "jsx" | "mjs" | "cjs" => Some(Box::new(javascript::JavaScriptAdapter)),
        _ => None,
    }
}
