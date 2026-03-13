# Git diff analysis and duplicate removal — workflow

Use this workflow to analyze a git diff, extract variables/functions/tests/imports, analyze duplicates, check for deprecated dependencies, remove issues, and run lint/tests until stable.

---

## Step 0 — Install differ_helper and discover the git diff

1. Install Rust if not already installed:
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

2. First time — clone and install:
   git clone https://github.com/afa7789/differ_helper /tmp/differ_helper
   cd /tmp/differ_helper && make install && cd -

   Already installed — update to latest version:
   cd /tmp/differ_helper && git pull && make reinstall && cd -

   If the /tmp clone was lost (reboot etc.), just re-clone from scratch.

   `make install` compiles and copies the binary to ~/.cargo/bin/differ_helper.
   `make reinstall` removes the old binary first, then installs fresh.
   ~/.cargo/bin is already in your $PATH (rustup sets this up).

3. Ensure the remote is fetched:
   git fetch origin

---

## Step 1 — Extract names from the diff

Run differ_helper in the current repo:

   differ_helper

This auto-detects where the current branch diverged from its upstream (e.g. origin/main) and diffs everything since that point. No need to generate a diff file manually.

You can also target a specific base:

   differ_helper main
   differ_helper origin/develop
   differ_helper v1.2.0

Or pass a diff file directly:

   differ_helper /path/to/diff.txt

The output will contain:

   VARIABLES:
   - <var_name> -> <file_path>

   FUNCTIONS:
   - <func_name> -> <file_path>

   TESTS:
   - <test_name> -> <file_path>

   IMPORTS:
   - <import_path> -> <file_path>

   WARNINGS:
   - <security_pattern> -> <file_path>

Entries are already deduplicated by (name, file) and sorted by file path then name.
Use this output for Steps 2, 3, 4 and 5.

---

## Steps 2, 3, 4 and 5 — Run in parallel after Step 1

### Step 2 — Analyze variables

For each variable from the Step 1 list:

1. Explain what it likely represents (name + context).
2. After all, identify duplicates: same name in different files, or same concept with different names.
3. At the end, add a WARNING section listing duplicate/overlapping variables and where they appear.

### Step 3 — Analyze functions

For each function from the Step 1 list:

1. Explain what it likely does (name + context).
2. After all, identify duplicates: same name in different files, or same logic with different names.
3. At the end, add a WARNING section listing only true duplicates (redundant implementations or helpers).

### Step 4 — Analyze unit tests

For each test from the Step 1 list:

1. Explain what it is likely testing (name + context).
2. Identify duplicate tests (same scenario tested multiple times).
3. At the end, add a WARNING section listing duplicate/overlapping tests.

### Step 5 — Analyze imports (parallel agent)

For each import from the Step 1 IMPORTS list:

1. Identify the package/module being imported and its purpose.
2. Check if the package is deprecated, archived, or unmaintained. Look for:
   - Official deprecation notices (e.g. `moment.js` → `dayjs`/`date-fns`, `request` → `got`/`node-fetch`).
   - Archived or unmaintained GitHub repositories.
   - Known security vulnerabilities (CVEs, npm advisories, RustSec).
   - Packages superseded by standard library alternatives (e.g. `atoi` in Go is unnecessary since `strconv` exists).
3. Check if a more modern or idiomatic alternative exists that is widely adopted.
4. At the end, add a WARNING section listing:
   - Deprecated imports and their recommended replacement.
   - Imports with known security issues.
   - Imports that could be replaced by standard library features.
   - Duplicate imports (same package imported in multiple files where a shared module would be cleaner).

Only flag real issues — do not warn about stable, well-maintained packages just because alternatives exist.

**Action rules:**
- If the fix is a simple drop-in replacement (e.g. swapping one import path for another, updating a function name), apply the refactoring directly.
- If the migration is complex (different API surface, requires rewriting logic, or touches many files), do NOT refactor — just report it clearly in the final output so the user knows and can plan the migration.

---

## Step 6 — Remove duplicates

Using only the duplicates flagged in Steps 2, 3, 4 and 5:

1. For each duplicate, decide which version to keep (prefer more descriptive name or more complete implementation).
2. For each deprecated/problematic import flagged in Step 5, provide the migration path and exact code changes.
3. Provide the exact code changes to remove the duplicates.
4. List every file that must be updated after removal.

Do not remove: same method name on different types, or constants that intentionally mirror the same concept across modules.

---

## Step 7 — Run lint and CI/CD

Run the project's lint/CI pipeline. Fix every style and format issue. List every file changed and what was fixed.

---

## Step 8 — Run tests

Run the full unit test suite. Report:

- Which tests passed
- Which tests failed, with error messages
- Root cause of each failure

If any test fails, fix and re-run before continuing.

---

## Loop — Repeat until stable

Repeat Steps 7 and 8 until:

1. Lint and CI pass with no warnings or errors.
2. All unit tests pass.

After each iteration, briefly summarize what was fixed and the current status of both checks.
