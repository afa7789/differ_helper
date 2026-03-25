# Release Process

## How to release a new version

1. **Bump version and create tag:**

```bash
make tag VERSION=0.2.0
```

This will:
- Update the version in `Cargo.toml`
- Commit the change
- Create a git tag `v0.2.0`

2. **Push to trigger the release workflow:**

```bash
git push origin main --tags
```

3. The CD pipeline (`.github/workflows/release.yml`) will automatically:
   - Run checks (fmt, clippy, tests)
   - Build binaries for all platforms
   - Create a GitHub Release with the artifacts

## Supported platforms

| Target | OS |
|---|---|
| `x86_64-unknown-linux-gnu` | Linux x86_64 |
| `aarch64-unknown-linux-gnu` | Linux ARM64 |
| `x86_64-apple-darwin` | macOS Intel |
| `aarch64-apple-darwin` | macOS Apple Silicon |
| `x86_64-pc-windows-gnu` | Windows x86_64 |

## Installation

### From GitHub Release

Download the binary for your platform from the [Releases](https://github.com/afa7789/differ_helper/releases) page.

### From source

```bash
cargo install --git https://github.com/afa7789/differ_helper
```
