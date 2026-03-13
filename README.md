# differ_helper

A fast, zero-config CLI that parses **git unified diffs** and extracts every **variable**, **function**, **test**, and **import** introduced in the changeset, grouped by file. Also flags **security-sensitive patterns** (hardcoded secrets, dangerous calls, injection risks).

Built in Rust with **parallel file processing** via [rayon](https://github.com/rayon-rs/rayon) — each file section is extracted concurrently for maximum throughput on large diffs.

## Supported Languages

| Language      | Extensions                             | Variables                       | Functions                         | Tests                              | Imports                          |
|---------------|----------------------------------------|---------------------------------|-----------------------------------|------------------------------------|---------------------------------|
| Rust          | `.rs`                                  | `let`, `const`, `static`        | `fn`, `pub fn`, `async fn`        | `#[test] fn`                       | `use`, `extern crate`           |
| Python        | `.py`, `.pyi`                          | module-level assignments        | `def`, `async def`, `class`       | `test_*`, `TestCase`               | `import`, `from X import`       |
| Go            | `.go`                                  | `var`, `const`, `:=`            | `func`, `type`                    | `Test*`, `Benchmark*`, `Example*`  | `import "pkg"`                  |
| C             | `.c`, `.h`                             | `#define`                       | `struct`, `enum`, `typedef`, fns  | —                                  | `#include`                      |
| C++           | `.cpp`, `.cxx`, `.cc`, `.hpp`, `.hxx`  | `#define`, `constexpr`          | `class`, `namespace`, fns         | GTest, Catch2, Boost.Test          | `#include`                      |
| TypeScript/JS | `.ts`, `.tsx`, `.js`, `.jsx`           | `const`, `let`, `var`           | `function`, arrows, types         | `describe`, `it`, `test`           | `import from`, `require()`      |
| CSS           | `.css`                                 | `--custom-props`                | `.class`, `#id` selectors         | —                                  | `@import`                       |
| SQL           | `.sql`                                 | —                               | DDL objects (CREATE/ALTER/DROP)    | —                                  | —                               |
| MASM          | `.masm`                                | `const`                         | `proc`, `use`                     | —                                  | —                               |

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

### Updating

```bash
git pull && make reinstall
```

`make reinstall` removes the old binary from `~/.cargo/bin` and installs the new version from source.

### Uninstalling

```bash
make uninstall
```

## Usage

```bash
# Auto-detect: diffs current branch against its origin
differ_helper

# Diff against a specific branch, tag, or commit
differ_helper main
differ_helper origin/develop
differ_helper v1.2.0
differ_helper abc123f

# Range syntax
differ_helper main..HEAD
differ_helper main...feature

# Read a diff file directly
differ_helper /path/to/diff.txt
```

### How auto-detection works (no arguments)

When run without arguments, differ_helper automatically finds the best diff:

1. Finds the **merge-base** between HEAD and its upstream (tracked branch, `origin/main`, `origin/master`, `origin/develop`, or `origin/next`).
2. Diffs **everything** the current branch has that's different from that base.
3. Falls back to unstaged changes (`git diff`), then staged changes (`git diff --cached`).

This means you can just `cd` into your repo and run `differ_helper` — it figures out the right diff automatically.

### How argument detection works

When you pass an argument, differ_helper detects what it is:

1. If it contains `..` → treated as a **git range** (`git diff main..HEAD`).
2. If it resolves to a **git ref** (branch, tag, commit hash) → diffs against the merge-base with HEAD.
3. If it's a **file on disk** → reads it as a unified diff file.

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

IMPORTS:
- std::collections::HashMap -> src/lib.rs
- os -> app/models.py

WARNINGS:
- hardcoded password -> config/secrets.py
- dangerous eval() -> utils/parser.js
```

Entries are deduplicated by `(name, file)` and sorted by file path, then name. Warnings are deduplicated by `(pattern, file)`.

### Security patterns detected

The WARNINGS section flags 22 built-in patterns including:

- **Secrets**: `password`, `secret`, `api_key`, `access_token`, `private_key`, `credential`
- **Dangerous calls**: `eval()`, `exec()`, `subprocess`, `os.system`, `Runtime.exec`
- **XSS risks**: `innerHTML`, `dangerouslySetInnerHTML`, `document.write`
- **Raw SQL**: `SELECT`, `INSERT INTO`, `DELETE FROM` (case-insensitive)
- **Code markers**: `TODO`, `FIXME`, `HACK`
- **Rust-specific**: `unsafe`, `unwrap()`

Comments are automatically skipped.

## Makefile Targets

| Target           | Description                                  |
|------------------|----------------------------------------------|
| `make build`     | Debug build                                  |
| `make release`   | Optimized release build                      |
| `make test`      | Run all tests                                |
| `make lint`      | Run clippy with `-D warnings`                |
| `make fmt`       | Check formatting                             |
| `make fmt-fix`   | Auto-fix formatting                          |
| `make check`     | Full CI check (fmt + lint + test)            |
| `make install`   | Install binary to `~/.cargo/bin`             |
| `make uninstall` | Remove binary from `~/.cargo/bin`            |
| `make reinstall` | Remove old binary and install from source    |
| `make clean`     | Remove build artifacts                       |

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
├── main.rs          # CLI + smart arg detection + parallel diff orchestration (rayon)
├── lang.rs          # Language detection by extension
├── extract.rs       # Extractor trait + result types
├── ident.rs         # Shared identifier parsing helpers
├── output.rs        # Dedup + sorted output
├── security.rs      # Built-in security pattern detection
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
│   ├── masm.rs      # MASM extractor
│   └── ADDING_LANGUAGES.md  # Guide for adding new languages
└── tests/
    ├── main_test.rs
    ├── ident_test.rs
    ├── lang_test.rs
    ├── output_test.rs
    ├── security_test.rs
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

Adding a new language: see [`src/langs/ADDING_LANGUAGES.md`](src/langs/ADDING_LANGUAGES.md) for a step-by-step guide.

## Tests

```bash
make test               # 119 unit + integration tests
cargo tarpaulin         # line coverage report
```

## CI

GitHub Actions runs on every push/PR to `main`:
- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo test`

## License

MIT
