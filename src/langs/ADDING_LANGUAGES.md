# Adding a New Language

This guide walks you through adding support for a new programming language to differ_helper.

## What we extract from each language

Every language extractor looks at **added lines** (`+` lines) in a git diff and tries to identify 4 categories of symbols:

| Category      | What to look for                                                         | Examples                                                    |
|---------------|--------------------------------------------------------------------------|-------------------------------------------------------------|
| **variables** | Named value declarations, constants, assignments at module/global scope  | `let x`, `const Y`, `var z`, `#define MAX`, `--css-var`     |
| **functions** | Function, method, class, struct, type, or interface definitions          | `fn foo`, `def bar`, `func Baz`, `class Widget`, `type Cfg` |
| **tests**     | Test function/block declarations specific to the language's test frameworks | `#[test] fn`, `def test_`, `TEST(Suite, Name)`, `describe(` |
| **imports**   | Import/include/require statements that introduce external dependencies   | `use std::io`, `import os`, `#include <stdio.h>`, `require(`|

Not every language has all 4. For example, CSS has no tests. SQL has no imports. That's fine — just return empty vectors for categories that don't apply.

## Step by step

### 1. Create the extractor file

Create `src/langs/<language>.rs`. Use the simplest existing extractor as a template — `go.rs` is a good starting point.

The file must:

- Define a public struct: `pub struct MyLangExtractor;`
- Implement the `Extractor` trait from `crate::extract`
- Wire up test module at the bottom

```rust
use crate::extract::{Extracted, Extractor, ExtractorState};
use crate::ident;

pub struct MyLangExtractor;

impl Extractor for MyLangExtractor {
    fn extract_line(&self, line: &str, _state: &mut ExtractorState) -> Extracted {
        let mut out = Extracted::default();

        // variables: push to out.variables
        // functions: push to out.functions
        // tests:     push to out.tests
        // imports:   push to out.imports

        out
    }
}

#[cfg(test)]
#[path = "../tests/langs/mylang_test.rs"]
mod tests;
```

### 2. Register the language

Two files need changes:

**`src/lang.rs`** — Add a variant to the `Lang` enum and a match arm in `detect()`:

```rust
pub enum Lang {
    // ...existing...
    MyLang,
    Unknown,
}

pub fn detect(file: &str) -> Lang {
    match ext {
        // ...existing...
        "ext1" | "ext2" => Lang::MyLang,
        _ => Lang::Unknown,
    }
}
```

**`src/langs/mod.rs`** — Add the module and wire it into `extractor_for()`:

```rust
pub mod mylang;

pub fn extractor_for(lang: Lang) -> Option<Box<dyn Extractor>> {
    match lang {
        // ...existing...
        Lang::MyLang => Some(Box::new(mylang::MyLangExtractor)),
        Lang::Unknown => None,
    }
}
```

### 3. Write tests

Create `src/tests/langs/mylang_test.rs`. Follow the pattern from existing test files. At minimum, test:

- Each category you extract (variables, functions, tests, imports)
- Edge cases (empty lines, comments, nested scopes)
- Lines that should NOT match (false positive prevention)

```rust
use crate::extract::{Extractor, ExtractorState};
use crate::langs::mylang::MyLangExtractor;

#[test]
fn variable_declaration() {
    let mut state = ExtractorState::default();
    let result = MyLangExtractor.extract_line("let x = 1;", &mut state);
    assert_eq!(result.variables, vec!["x"]);
}
```

Also add an end-to-end test in `src/tests/main_test.rs` that parses a full diff for your language.

### 4. Update the doc comment

Add your language to the table in the doc comment at the top of `src/main.rs`.

## Key helpers available

The `crate::ident` module provides reusable helpers so you don't have to write low-level parsing:

| Helper                        | What it does                                                              |
|-------------------------------|---------------------------------------------------------------------------|
| `ident::prefix(s)`            | Returns the longest valid identifier starting at `s` (letters, digits, `_`, `$`) |
| `ident::extract_string_arg(s)` | Extracts content inside first pair of quotes/backticks                    |
| `ident::find_all_after_patterns(line, &["pat1 ", "pat2 "])` | Finds all identifiers following any of the given patterns in a line |

Most extractors are built almost entirely on top of `find_all_after_patterns`. Check `rust.rs` for a clean example.

## ExtractorState

The `ExtractorState` struct carries mutable state across consecutive lines within the same file. Currently it has one field (`in_test_block: bool`) used by Rust for multi-line `#[test]` annotations.

If your language needs cross-line state (e.g., tracking whether you're inside a test block), add a field to `ExtractorState` in `src/extract.rs`. Keep it simple — most languages don't need this.

## Tips

- **Order patterns from longest to shortest** when they overlap. For example, `"pub fn "` before `"fn "`, otherwise `"fn "` matches first and you get `"pub"` as the name.
- **Use `strip_prefix`** instead of manual slicing — clippy will complain otherwise.
- **One line at a time.** The extractor only sees individual added lines, not the full file. Design patterns around what's visible on a single line.
- **Run `make test` and `make lint`** before submitting. CI runs `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test`.

## Checklist

- [ ] `src/langs/<language>.rs` — extractor struct + `Extractor` impl
- [ ] `src/lang.rs` — `Lang` variant + extension mapping in `detect()`
- [ ] `src/langs/mod.rs` — module declaration + `extractor_for()` arm
- [ ] `src/tests/langs/<language>_test.rs` — unit tests
- [ ] `src/tests/main_test.rs` — end-to-end test
- [ ] `src/main.rs` — doc comment table updated
