//! JavaScript / TypeScript language extractor.
//!
//! Extracts variable declarations, function/type definitions, arrow functions,
//! and test/describe/it blocks.

use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct JsTsExtractor;

impl Extractor for JsTsExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        for name in var_names(line) {
            out.variables.push(name.to_string());
        }
        for name in fn_names(line) {
            out.functions.push(name.to_string());
        }
        for name in test_names(line) {
            out.tests.push(name);
        }
        for name in import_names(line) {
            out.imports.push(name);
        }

        out
    }
}

/// Extract variable names: const, let, var (with optional export/declare).
fn var_names(line: &str) -> Vec<&str> {
    let prefixes = [
        "export const ",
        "export let ",
        "export var ",
        "export let ", // Vue/Svelte props
        "declare const ",
        "declare let ",
        "const ",
        "let ",
        "var ",
    ];

    if let Some(stripped) = line.strip_prefix("<script") {
        if !stripped.contains("lang=") && !stripped.starts_with(' ') && !stripped.starts_with('>') {
            return vec![];
        }
        if let Some(eq_pos) = stripped.find("lang=") {
            let lang_part = &stripped[eq_pos..];
            if lang_part.starts_with("lang=") {
                let after = &lang_part[5..];
                if let Some(end) = after.find(|c: char| !c.is_alphanumeric()) {
                    let lang = &after[..end];
                    if lang != "ts" && lang != "typescript" {
                        return vec![];
                    }
                }
            }
        }
    }

    let mut matches: Vec<(usize, &str)> = Vec::new();
    for prefix in prefixes {
        let mut start = 0;
        while let Some(pos) = line[start..].find(prefix) {
            let abs = start + pos;
            let after = line[abs + prefix.len()..].trim_start();
            // Skip destructuring patterns.
            if after.starts_with('{') || after.starts_with('[') {
                start = abs + prefix.len();
                continue;
            }
            if let Some(name) = ident::prefix(after) {
                if !matches
                    .iter()
                    .any(|(p, _)| *p == abs || (abs < *p + 10 && abs > p.saturating_sub(20)))
                {
                    matches.push((abs, name));
                }
            }
            start = abs + prefix.len();
        }
    }
    matches.sort_by_key(|(pos, _)| *pos);
    matches.dedup_by_key(|(pos, _)| *pos);
    matches.into_iter().map(|(_, name)| name).collect()
}

/// Extract function names: declarations, arrow functions, and type definitions.
fn fn_names(line: &str) -> Vec<&str> {
    let mut out: Vec<(usize, &str)> = Vec::new();

    // Function declarations.
    let fn_patterns = [
        "export default async function ",
        "export default function ",
        "export async function ",
        "export function ",
        "async function ",
        "function ",
    ];
    collect_patterns(line, &fn_patterns, &mut out);

    // Arrow functions: `const NAME = (...) =>` or `const NAME = async (`.
    let arrow_kw = ["export const ", "export let ", "const ", "let ", "var "];
    for kw in arrow_kw {
        let mut start = 0;
        while let Some(pos) = line[start..].find(kw) {
            let abs = start + pos;
            let after = &line[abs + kw.len()..];
            if let Some(name) = ident::prefix(after) {
                let rest = line[abs + kw.len() + name.len()..].trim_start();
                if let Some(after_eq) = rest.strip_prefix('=') {
                    let rhs = after_eq.trim_start();
                    if (rhs.starts_with('(') || rhs.starts_with("async"))
                        && !out.iter().any(|(p, _)| *p == abs)
                    {
                        out.push((abs, name));
                    }
                }
            }
            start = abs + kw.len();
        }
    }

    // Type declarations: type, interface, enum.
    let type_patterns = [
        "export type ",
        "export interface ",
        "export enum ",
        "type ",
        "interface ",
        "enum ",
    ];
    collect_patterns(line, &type_patterns, &mut out);

    // Vue/Svelte: defineProps, defineEmits, defineModel
    let vue_svelte_fns = ["defineProps", "defineEmits", "defineModel", "defineOptions"];
    for pat in vue_svelte_fns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let after = &line[start + pos + pat.len()..];
            if after.starts_with('(') {
                out.push((start + pos, pat));
            }
            start = start + pos + pat.len();
        }
    }

    // Class declarations
    let class_patterns = ["export default class ", "export class ", "class "];
    collect_patterns(line, &class_patterns, &mut out);

    // React/NestJS: component, hook, service patterns
    let component_patterns = ["@Component(", "@Injectable(", "@Controller(", "@Entity("];
    for pat in component_patterns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let after = &line[start + pos + pat.len()..];
            if let Some(name) = ident::extract_string_arg(after) {
                out.push((start + pos, name));
            }
            start = start + pos + pat.len();
        }
    }

    out.sort_by_key(|(pos, _)| *pos);
    out.into_iter().map(|(_, name)| name).collect()
}

/// Extract JS/TS imports: `import ... from "mod"`, `require("mod")`.
fn import_names(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let mut out = Vec::new();

    // `import ... from "module"` or `import "module"`.
    if trimmed.starts_with("import ") || trimmed.starts_with("export ") {
        if let Some(from_pos) = trimmed.find("from ") {
            let after = &trimmed[from_pos + 5..];
            if let Some(name) = ident::extract_string_arg(after) {
                out.push(name.to_string());
                return out;
            }
        }
        // `import "module"` (side-effect import).
        if let Some(after) = trimmed.strip_prefix("import ") {
            if let Some(name) = ident::extract_string_arg(after) {
                out.push(name.to_string());
                return out;
            }
        }
    }

    // `require("module")`.
    let mut start = 0;
    while let Some(pos) = trimmed[start..].find("require(") {
        let abs = start + pos;
        let after = &trimmed[abs + 8..];
        if let Some(name) = ident::extract_string_arg(after) {
            out.push(name.to_string());
        }
        start = abs + 8;
    }

    out
}

/// Extract test names from describe/it/test blocks.
fn test_names(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let patterns = [
        "describe(",
        "it(",
        "test(",
        "it.each(",
        "test.each(",
        "describe.each(",
        "it.only(",
        "test.only(",
        "describe.only(",
        "it.skip(",
        "test.skip(",
        "describe.skip(",
        "given(",
        "when(",
        "then(",
        "and(",
        "but(",
    ];
    for pat in patterns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            let after = &line[abs + pat.len()..];
            if let Some(name) = ident::extract_string_arg(after) {
                out.push(name.to_string());
            }
            start = abs + pat.len();
        }
    }
    // Jest/Vitest test decorators
    let decorators = ["@test(", "@it(", "@spec("];
    for pat in decorators {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let after = &line[start + pat.len()..];
            if let Some(name) = ident::extract_string_arg(after) {
                out.push(name.to_string());
            }
            start = start + pos + pat.len();
        }
    }
    out
}

/// Shared helper: find all patterns and collect identifiers that follow them.
///
/// Deduplicates by position to avoid overlapping matches (e.g. "export function "
/// and "function " matching the same declaration).
fn collect_patterns<'a>(line: &'a str, patterns: &[&str], out: &mut Vec<(usize, &'a str)>) {
    for pat in patterns {
        let mut start = 0;
        while let Some(pos) = line[start..].find(pat) {
            let abs = start + pos;
            let after = &line[abs + pat.len()..];
            if let Some(name) = ident::prefix(after) {
                // Skip if we already captured this name from an overlapping longer pattern.
                if !out.iter().any(|(_, n)| *n == name) {
                    out.push((abs, name));
                }
            }
            start = abs + pat.len();
        }
    }
}

#[cfg(test)]
#[path = "../tests/langs/jsts_test.rs"]
mod tests;
