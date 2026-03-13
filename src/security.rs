//! Built-in security pattern detection for added diff lines.
//!
//! Scans raw added lines for common security-sensitive patterns:
//! hardcoded secrets, dangerous function calls, injection risks, etc.

/// A security warning found in a diff line.
pub struct Warning {
    pub pattern: &'static str,
    pub file: String,
}

/// Case-sensitive patterns.
const PATTERNS: &[(&str, &str)] = &[
    ("password", "hardcoded password"),
    ("passwd", "hardcoded password"),
    ("secret", "hardcoded secret"),
    ("api_key", "hardcoded API key"),
    ("apikey", "hardcoded API key"),
    ("api-key", "hardcoded API key"),
    ("access_token", "hardcoded access token"),
    ("private_key", "hardcoded private key"),
    ("credential", "hardcoded credential"),
    ("eval(", "dangerous eval()"),
    ("exec(", "dangerous exec()"),
    ("innerHTML", "potential XSS via innerHTML"),
    (
        "dangerouslySetInnerHTML",
        "potential XSS via dangerouslySetInnerHTML",
    ),
    ("document.write", "potential XSS via document.write"),
    ("subprocess", "shell command execution"),
    ("os.system", "shell command execution"),
    ("Runtime.exec", "shell command execution"),
    ("TODO", "TODO marker"),
    ("FIXME", "FIXME marker"),
    ("HACK", "HACK marker"),
    ("unsafe ", "unsafe block"),
    ("unwrap()", "unwrap() may panic"),
];

/// Case-insensitive patterns (checked against lowercased line).
const CI_PATTERNS: &[(&str, &str)] = &[
    ("select ", "possible raw SQL query"),
    ("insert into", "possible raw SQL query"),
    ("delete from", "possible raw SQL query"),
];

/// Scan a set of added lines for security-sensitive patterns.
pub fn scan_lines(lines: &[String], file: &str) -> Vec<Warning> {
    let mut warnings = Vec::new();

    for line in lines {
        let raw = line.trim_start_matches('+');
        let trimmed = raw.trim();

        // Skip comments.
        if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("--")
        {
            continue;
        }

        let mut found = false;

        // Case-sensitive patterns.
        for (pat, label) in PATTERNS {
            if trimmed.contains(pat) {
                warnings.push(Warning {
                    pattern: label,
                    file: file.to_string(),
                });
                found = true;
                break;
            }
        }

        // Case-insensitive patterns (only if no match above).
        if !found {
            let lower = trimmed.to_ascii_lowercase();
            for (pat, label) in CI_PATTERNS {
                if lower.contains(pat) {
                    warnings.push(Warning {
                        pattern: label,
                        file: file.to_string(),
                    });
                    break;
                }
            }
        }
    }

    warnings
}

#[cfg(test)]
#[path = "tests/security_test.rs"]
mod tests;
