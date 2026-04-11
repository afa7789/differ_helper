/// Supported programming languages, identified by file extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Masm,
    JsTs,
    Css,
    Sql,
    Python,
    Go,
    C,
    Cpp,
    Java,
    Kotlin,
    Swift,
    ObjC,
    Cs,
    Ruby,
    Php,
    Shell,
    Lua,
    Terraform,
    Yaml,
    Zig,
    Haskell,
    Nim,
    Unknown,
}

impl Lang {
    #[allow(dead_code)]
    pub fn is_known(self) -> bool {
        self != Lang::Unknown
    }
}

/// Map a file path to its language based on extension.
pub fn detect(file: &str) -> Lang {
    detect_from_path(file)
}

/// Detect language from file content (for shebang).
#[allow(dead_code)]
pub fn detect_from_content(content: &str) -> Lang {
    let first_line = content.lines().next().unwrap_or("");
    let trimmed = first_line.trim();

    if trimmed.starts_with("#!") {
        if trimmed.contains("ruby") || trimmed.contains("rb") {
            return Lang::Ruby;
        }
        if trimmed.contains("python") {
            return Lang::Python;
        }
        if trimmed.contains("node") {
            return Lang::JsTs;
        }
        if trimmed.contains("bash") || trimmed.contains("sh") || trimmed.contains("zsh") {
            return Lang::Shell;
        }
        if trimmed.contains("php") {
            return Lang::Php;
        }
        if trimmed.contains("lua") {
            return Lang::Lua;
        }
    }

    if trimmed.starts_with("#!/") {
        return Lang::Shell;
    }

    Lang::Unknown
}

/// Detect language from filename patterns.
pub fn detect_from_path(file: &str) -> Lang {
    let filename = file.split('/').next_back().unwrap_or(file);

    match filename {
        "Makefile" | "makefile" | "CMakeLists.txt" | "Dockerfile" | "Dockerfile.prod" => {
            Lang::Shell
        }
        "Vagrantfile" => Lang::Ruby,
        "Gemfile" | "Rakefile" => Lang::Ruby,
        "Podfile" | "Cartfile" => Lang::Ruby,
        "package.json" | "tsconfig.json" | "eslintrc" | ".eslintrc" => Lang::JsTs,
        ".env" | ".env.sample" => Lang::JsTs,
        "terraform.tf" | "terraform.tfvars" => Lang::Terraform,
        _ => detect_from_ext(file),
    }
}

fn detect_from_ext(file: &str) -> Lang {
    let ext = file.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => Lang::Rust,
        "masm" => Lang::Masm,
        "ts" | "tsx" | "js" | "jsx" | "vue" | "svelte" | "astro" => Lang::JsTs,
        "css" | "scss" | "sass" | "less" => Lang::Css,
        "sql" => Lang::Sql,
        "py" | "pyi" => Lang::Python,
        "go" => Lang::Go,
        "c" | "h" => Lang::C,
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => Lang::Cpp,
        "java" => Lang::Java,
        "kt" | "kts" => Lang::Kotlin,
        "swift" => Lang::Swift,
        "m" | "mm" => Lang::ObjC,
        "cs" => Lang::Cs,
        "rb" => Lang::Ruby,
        "php" => Lang::Php,
        "sh" | "bash" | "zsh" | "fish" => Lang::Shell,
        "lua" => Lang::Lua,
        "tf" | "tfvars" => Lang::Terraform,
        "yml" | "yaml" => Lang::Yaml,
        "toml" => Lang::Yaml,
        "zig" => Lang::Zig,
        "hs" => Lang::Haskell,
        "nim" => Lang::Nim,
        _ => Lang::Unknown,
    }
}

#[cfg(test)]
#[path = "tests/lang_test.rs"]
mod tests;
