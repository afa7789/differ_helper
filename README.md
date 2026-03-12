# differ_helper

A fast, zero-config CLI that parses **git unified diffs** and extracts every **variable**, **function**, and **test** introduced in the changeset, grouped by file.

Built in Rust with **parallel file processing** via [rayon](https://github.com/rayon-rs/rayon) — each file section is extracted concurrently for maximum throughput on large diffs.

## Supported Languages

| Language      | Extensions                             | Variables                       | Functions                         | Tests                              |
|---------------|----------------------------------------|---------------------------------|-----------------------------------|------------------------------------|
| Rust          | `.rs`                                  | `let`, `const`, `static`        | `fn`, `pub fn`, `async fn`        | `#[test] fn`                       |
| Python        | `.py`, `.pyi`                          | module-level assignments        | `def`, `async def`, `class`       | `test_*`, `TestCase`               |
| Go            | `.go`                                  | `var`, `const`, `:=`            | `func`, `type`                    | `Test*`, `Benchmark*`, `Example*`  |
| C             | `.c`, `.h`                             | `#define`                       | `struct`, `enum`, `typedef`, fns  | —                                  |
| C++           | `.cpp`, `.cxx`, `.cc`, `.hpp`, `.hxx`  | `#define`, `constexpr`          | `class`, `namespace`, fns         | GTest, Catch2, Boost.Test          |
| TypeScript/JS | `.ts`, `.tsx`, `.js`, `.jsx`           | `const`, `let`, `var`           | `function`, arrows, types         | `describe`, `it`, `test`           |
| CSS           | `.css`                                 | `--custom-props`                | `.class`, `#id` selectors         | —                                  |
| SQL           | `.sql`                                 | —                               | DDL objects (CREATE/ALTER/DROP)    | —                                  |
| MASM          | `.masm`                                | `const`                         | `proc`, `use`                     | —                                  |

## Install

```bash
cargo build --release
# Binary: target/release/differ_helper
```

## Usage

```bash
# Generate a diff
git diff origin/main...HEAD > /tmp/diff.txt

# Run
./target/release/differ_helper /tmp/diff.txt
```

Without arguments it reads from `/tmp/diff_origin_next.txt`.

## Output

```
VARIABLES:
- count -> src/lib.rs
- MAX_RETRIES -> app/models.py

FUNCTIONS:
- process -> src/lib.rs
- UserService -> app/models.py

TESTS:
- it_works -> src/lib.rs
- test_user_creation -> app/models.py
```

Entries are deduplicated by `(name, file)` and sorted by file path, then name.

## Architecture

```
src/
├── main.rs          # CLI + parallel diff orchestration (rayon)
├── lang.rs          # Language detection by extension
├── extract.rs       # Extractor trait + result types
├── ident.rs         # Shared identifier parsing helpers
├── output.rs        # Dedup + sorted output
└── langs/
    ├── mod.rs       # Extractor factory
    ├── rust.rs      # Rust extractor
    ├── python.rs    # Python extractor
    ├── go.rs        # Go extractor
    ├── c.rs         # C extractor
    ├── cpp.rs       # C++ extractor (inherits C)
    ├── jsts.rs      # JS/TS extractor
    ├── css.rs       # CSS extractor
    ├── sql.rs       # SQL extractor
    └── masm.rs      # MASM extractor
```

Adding a new language: implement `Extractor` in a new file under `langs/`, add two lines in `langs/mod.rs` and `lang.rs`.

## Tests

```bash
cargo test          # 104 unit + integration tests
cargo tarpaulin     # ~96% line coverage
```

## CI

GitHub Actions runs on every push/PR to `main`:
- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo test`

## License

MIT
