# Git diff analysis and duplicate removal — workflow

Use this workflow to analyze a git diff, extract variables/functions/tests, analyze duplicates, remove them, and run lint/tests until stable.

---

## Step 0 — Install differ_helper and discover the git diff

1. Install Rust if not already installed:
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

2. Clone and install differ_helper globally (one-time):
   git clone https://github.com/afa7789/differ_helper /tmp/differ_helper
   cd /tmp/differ_helper && make install && cd -
   After this, differ_helper is available directly in the terminal via ~/.cargo/bin.

3. Fetch and determine base branch of the current repo:
   git fetch origin
   git remote show origin | grep HEAD
   Use the branch name shown (e.g. next, main, or master).

4. Generate the diff and save to a file:
   git diff origin/<BASE>...HEAD > /tmp/diff.txt
   Replace <BASE> with the branch from step 3 (e.g. next or main).

---

## Step 1 — Extract names from the diff

Run differ_helper on the diff file:
   differ_helper /tmp/diff.txt

The binary will output:
   VARIABLES:
   - <var_name> -> <file_path>

   FUNCTIONS:
   - <func_name> -> <file_path>

   TESTS:
   - <test_name> -> <file_path>

Entries are already deduplicated by (name, file) and sorted by file path then name.
Use this output for Steps 2, 3, and 4.

---

## Steps 2, 3 and 4 — Run in parallel after Step 1

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

---

## Step 5 — Remove duplicates

Using only the duplicates flagged in Steps 2, 3 and 4:

1. For each duplicate, decide which version to keep (prefer more descriptive name or more complete implementation).
2. Provide the exact code changes to remove the duplicates.
3. List every file that must be updated after removal.

Do not remove: same method name on different types, or constants that intentionally mirror the same concept across modules.

---

## Step 6 — Run lint and CI/CD

Run the project's lint/CI pipeline. Fix every style and format issue. List every file changed and what was fixed.

---

## Step 7 — Run tests

Run the full unit test suite. Report:

- Which tests passed
- Which tests failed, with error messages
- Root cause of each failure

If any test fails, fix and re-run before continuing.

---

## Loop — Repeat until stable

Repeat Steps 6 and 7 until:

1. Lint and CI pass with no warnings or errors.
2. All unit tests pass.

After each iteration, briefly summarize what was fixed and the current status of both checks.
