# differ_helper

Parse a git unified diff and extract **variables**, **functions**, and **tests** from added lines (`+`), with file paths.

Repository: https://github.com/afa7789/differ_helper

## Build

```bash
cargo build --release
```

Binary: `target/release/differ_helper`

## Usage

1. Generate a diff and save to a file:

   ```bash
   git fetch origin
   git diff origin/main...HEAD > diff.txt
   ```

   (Use your base branch: `main`, `master`, `next`, etc. — check with `git remote show origin | grep HEAD`.)

2. Run the binary on the diff file:

   ```bash
   ./target/release/differ_helper diff.txt
   ```

   Or with default path `/tmp/diff_origin_next.txt`:

   ```bash
   ./target/release/differ_helper
   ```

## Output

- **VARIABLES:** `var_name -> file_path` (Rust `let`/`const`/`static`, MASM `const NAME =`)
- **FUNCTIONS:** `func_name -> file_path` (Rust `fn`/`pub fn`/`async fn`/`pub(crate) fn`)
- **TESTS:** `test_name -> file_path` (lines with `#[test]` and `fn`)

Output is deduplicated by `(name, file)` and sorted by file then name.
