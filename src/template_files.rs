//! Recursive file listing with .gitignore support.
//!
//! ## .gitignore semantics
//!
//! - **Negation:** `!pattern` un-ignores paths that match `pattern`. The last matching
//!   pattern wins (e.g. `*.log` then `!important.log` keeps `important.log`).
//! - **Supported:** Basic globs (`*`), exact names, `*.ext`, directory patterns (`dir/`).
//!
//! ## Limitations
//!
//! - **No negation of directory-only patterns:** `!dir/` is not specially handled.
//! - **No `**`:** Double-glob (e.g. `**/foo`) is not supported; use single `*` or path segments.
//! - **No escaped `!`:** Leading `!` always means negation.
//! - **Pattern scope:** Only considers .gitignore files in ancestor directories of the
//!   walk root; nested .gitignore semantics match git (patterns relative to that file’s dir).

use std::{
    fs,
    path::{Path, PathBuf},
};

/// Parsed .gitignore pattern: (raw line, negated).
fn parse_gitignore_line(line: &str) -> Option<(String, bool)> {
    let s = line.trim();
    if s.is_empty() || s.starts_with('#') {
        return None;
    }
    let (pattern, negated) = if s.starts_with('!') && s.len() > 1 {
        (s[1..].trim().to_string(), true)
    } else {
        (s.to_string(), false)
    };
    if pattern.is_empty() || pattern == "!" {
        None
    } else {
        Some((pattern, negated))
    }
}

pub fn get_all_files_in_dir_recursive(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    fn visit_dir(
        dir: &Path,
        root_dir: &Path,
        output: &mut Vec<PathBuf>,
        gitignore_patterns: &mut Vec<(PathBuf, Vec<(String, bool)>)>,
    ) -> anyhow::Result<()> {
        if dir.is_dir() {
            if dir.file_name().and_then(|n| n.to_str()) == Some(".git") {
                return Ok(());
            }

            // Check for .gitignore file in this directory
            let gitignore_path = dir.join(".gitignore");
            if gitignore_path.exists() {
                if let Ok(content) = fs::read_to_string(&gitignore_path) {
                    let patterns: Vec<(String, bool)> =
                        content.lines().filter_map(parse_gitignore_line).collect();
                    if !patterns.is_empty() {
                        gitignore_patterns.push((dir.to_path_buf(), patterns));
                    }
                }
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                // Check if path matches any gitignore pattern
                if is_ignored(&path, gitignore_patterns) {
                    continue;
                }

                if path.is_dir() {
                    visit_dir(&path, root_dir, output, gitignore_patterns)?;
                } else {
                    output.push(path);
                }
            }
        }
        Ok(())
    }

    let mut output = Vec::new();
    let mut gitignore_patterns = Vec::new();
    visit_dir(dir, dir, &mut output, &mut gitignore_patterns)?;
    Ok(output)
}

fn is_ignored(path: &Path, gitignore_patterns: &[(PathBuf, Vec<(String, bool)>)]) -> bool {
    let mut ignored = false;
    for (gitignore_dir, patterns) in gitignore_patterns {
        if !path.starts_with(gitignore_dir) {
            continue;
        }
        if let Ok(relative_path) = path.strip_prefix(gitignore_dir) {
            let path_str = relative_path.to_string_lossy();
            // Last matching pattern wins; negation un-ignores.
            for (pattern, negated) in patterns {
                if matches_gitignore_pattern(&path_str, pattern, path.is_dir()) {
                    ignored = !*negated;
                }
            }
        }
    }
    ignored
}

fn matches_gitignore_pattern(path: &str, pattern: &str, is_dir: bool) -> bool {
    // Handle directory-only patterns (ending with /)
    if pattern.ends_with('/') {
        if !is_dir {
            return false;
        }
        let pattern = &pattern[..pattern.len() - 1];
        return matches_gitignore_pattern(path, pattern, true);
    }

    // Convert gitignore pattern to regex-like matching
    // Handle simple cases: exact match, wildcards, and directory patterns

    // Exact match
    if pattern == path || pattern == path.trim_start_matches('/') {
        return true;
    }

    // Handle patterns starting with / (root-relative)
    let pattern = if pattern.starts_with('/') {
        &pattern[1..]
    } else {
        pattern
    };

    // Simple wildcard matching
    if pattern.contains('*') {
        // Convert * to .* for basic regex-like matching
        let regex_pattern = pattern.replace(".", "\\.").replace("*", ".*");

        // Use simple string matching for basic cases
        if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            if re.is_match(path) {
                return true;
            }
        }
    }

    // Check if any component matches
    for component in path.split('/') {
        if component == pattern || (pattern.starts_with("*") && component.ends_with(&pattern[1..]))
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_gitignore_negation() {
        assert_eq!(
            super::parse_gitignore_line("!important.log"),
            Some(("important.log".into(), true))
        );
        assert_eq!(
            super::parse_gitignore_line("*.log"),
            Some(("*.log".into(), false))
        );
        assert_eq!(
            super::parse_gitignore_line("  !foo  "),
            Some(("foo".into(), true))
        );
        assert_eq!(super::parse_gitignore_line("# comment"), None);
        assert_eq!(super::parse_gitignore_line("!"), None);
    }

    #[test]
    fn negation_unignores_matching_path() -> anyhow::Result<()> {
        let tmp = std::env::temp_dir().join("openapi2mcp_gitignore_negation");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp)?;
        fs::write(tmp.join(".gitignore"), "*.log\n!important.log\n")?;
        fs::write(tmp.join("a.log"), "")?;
        fs::write(tmp.join("b.log"), "")?;
        fs::write(tmp.join("important.log"), "")?;
        fs::write(tmp.join("other.txt"), "")?;

        let files = get_all_files_in_dir_recursive(&tmp)?;
        let names: Vec<_> = files
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
            .collect();

        assert!(
            names.contains(&"important.log"),
            "!important.log should un-ignore; got {:?}",
            names
        );
        assert!(
            names.contains(&"other.txt"),
            "non-matching file should be included; got {:?}",
            names
        );
        assert!(
            !names.contains(&"a.log"),
            "*.log should ignore a.log; got {:?}",
            names
        );
        assert!(
            !names.contains(&"b.log"),
            "*.log should ignore b.log; got {:?}",
            names
        );

        let _ = fs::remove_dir_all(&tmp);
        Ok(())
    }
}
