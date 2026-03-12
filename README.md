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

## Prerequisites

1. **Install Rust** — via [rustup](https://rustup.rs/) (recommended):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   This installs `rustc`, `cargo`, and `rustup`. Follow the on-screen instructions and restart your shell.

   Alternatively, see the [official install page](https://www.rust-lang.org/tools/install) for other methods (Homebrew, distro packages, etc.).

2. **Verify installation:**

   ```bash
   cargo --version   # e.g. cargo 1.82.0
   rustc --version   # e.g. rustc 1.82.0
   ```

3. **Make** (optional) — for using the Makefile targets. Pre-installed on macOS and most Linux distributions. On Windows, use `cargo` commands directly.

## Install

```bash
# Option 1: Install to ~/.cargo/bin (available system-wide)
make install
# or: cargo install --path .

# Option 2: Build release binary locally
make release
# Binary at: target/release/differ_helper
```

## Usage

```bash
# Generate a diff
git diff origin/main...HEAD > /tmp/diff.txt

# Run
differ_helper /tmp/diff.txt
# or: ./target/release/differ_helper /tmp/diff.txt
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

## Makefile Targets

| Target         | Description                                  |
|----------------|----------------------------------------------|
| `make build`   | Debug build                                  |
| `make release` | Optimized release build                      |
| `make test`    | Run all tests                                |
| `make lint`    | Run clippy with `-D warnings`                |
| `make fmt`     | Check formatting                             |
| `make fmt-fix` | Auto-fix formatting                          |
| `make check`   | Full CI check (fmt + lint + test)            |
| `make install` | Install binary to `~/.cargo/bin`             |
| `make clean`   | Remove build artifacts                       |

### Cross-compilation

To build binaries for other platforms, install [cross](https://github.com/cross-rs/cross):

```bash
cargo install cross

make cross-linux       # x86_64 Linux
make cross-linux-arm   # aarch64 Linux
make cross-windows     # x86_64 Windows
make cross-all         # All platforms + macOS
```

## Architecture

```
src/
├── main.rs          # CLI + parallel diff orchestration (rayon)
├── lang.rs          # Language detection by extension
├── extract.rs       # Extractor trait + result types
├── ident.rs         # Shared identifier parsing helpers
├── output.rs        # Dedup + sorted output
├── langs/
│   ├── mod.rs       # Extractor factory
│   ├── rust.rs      # Rust extractor
│   ├── python.rs    # Python extractor
│   ├── go.rs        # Go extractor
│   ├── c.rs         # C extractor
│   ├── cpp.rs       # C++ extractor (inherits C)
│   ├── jsts.rs      # JS/TS extractor
│   ├── css.rs       # CSS extractor
│   ├── sql.rs       # SQL extractor
│   └── masm.rs      # MASM extractor
└── tests/
    ├── main_test.rs
    ├── ident_test.rs
    ├── lang_test.rs
    ├── output_test.rs
    └── langs/
        ├── rust_test.rs
        ├── python_test.rs
        ├── go_test.rs
        ├── c_test.rs
        ├── cpp_test.rs
        ├── jsts_test.rs
        ├── css_test.rs
        ├── sql_test.rs
        └── masm_test.rs
```

Adding a new language: implement `Extractor` in a new file under `langs/`, add two lines in `langs/mod.rs` and `lang.rs`.

## Tests

```bash
make test               # 104 unit + integration tests
cargo tarpaulin         # ~96% line coverage
```

## CI

GitHub Actions runs on every push/PR to `main`:
- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo test`

## License

MIT
