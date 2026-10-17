//! SemVer release tag manager.

use crate::cli::output::print_success;
use crate::core::config::Config;
use crate::core::semver::validate_semver;
use crate::infra::git::{
    git_commit_files, git_create_annotated_tag, git_is_clean, resolve_repo_name, resolve_toplevel,
};
use crate::workflow::error::WorkflowError;
use std::path::Path;

/// Options for release tagging workflow.
pub struct TagOptions<'a> {
    /// Target release version string (e.g., "v0.3.0" or "0.3.0").
    pub version: &'a str,
    /// Optional explicit path to the repository root.
    pub repo_root: Option<&'a Path>,
}

/// Updates the `version` field under specified section headers in a TOML string.
///
/// Preserves comments, indentation, and formatting.
/// Returns `None` if no matching section contains a `version` key,
/// or if the version is already equal to `new_version`.
///
/// # Arguments
///
/// * `content` - Raw string content of the TOML file.
/// * `sections` - Slice of target section names (e.g., `&["project", "tool.poetry"]`).
/// * `new_version` - Target SemVer version string (with or without 'v' prefix).
///
/// # Example
///
/// ```
/// use git_claw::workflow::update_toml_version;
///
/// let toml = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";
/// let updated = update_toml_version(toml, &["package"], "0.2.0");
/// assert_eq!(
///     updated,
///     Some("[package]\nname = \"demo\"\nversion = \"0.2.0\"\n".to_string())
/// );
/// ```
pub fn update_toml_version(content: &str, sections: &[&str], new_version: &str) -> Option<String> {
    let clean_version = new_version.trim_start_matches('v');
    let mut in_target_section = false;
    let mut replaced = false;
    let mut result = String::with_capacity(content.len() + 16);

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            let without_comment = if let Some(idx) = trimmed.find('#') {
                trimmed[..idx].trim()
            } else {
                trimmed
            };
            in_target_section = without_comment
                .strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .map(|s| {
                    let sec = s.trim();
                    sections.iter().any(|&target| target == sec)
                })
                .unwrap_or(false);
        } else if trimmed.starts_with("[[") {
            in_target_section = false;
        } else if in_target_section && !replaced {
            if let Some(rest) = trimmed.strip_prefix("version") {
                let rest = rest.trim_start();
                if let Some(after_eq) = rest.strip_prefix('=') {
                    let after_eq = after_eq.trim_start();
                    if let Some(quote) = after_eq.chars().next() {
                        if quote == '"' || quote == '\'' {
                            let after_quote = &after_eq[quote.len_utf8()..];
                            if let Some(end_idx) = after_quote.find(quote) {
                                let current_version = &after_quote[..end_idx];
                                if current_version == clean_version {
                                    return None;
                                }
                                let old_pattern = format!("{}{}{}", quote, current_version, quote);
                                let new_pattern = format!("{}{}{}", quote, clean_version, quote);
                                let updated_line = line.replacen(&old_pattern, &new_pattern, 1);
                                result.push_str(&updated_line);
                                replaced = true;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        result.push_str(line);
    }

    if replaced {
        Some(result)
    } else {
        None
    }
}

/// Updates the `version` field under `[package]` in a `Cargo.toml` string.
///
/// Preserves comments, indentation, and formatting.
/// Returns `None` if `[package]` is not found, the `version` key is missing,
/// or the version is already equal to `new_version`.
///
/// # Arguments
///
/// * `content` - Raw string content of `Cargo.toml`.
/// * `new_version` - Target SemVer version string (without 'v' prefix).
///
/// # Example
///
/// ```
/// use git_claw::workflow::update_cargo_toml_version;
///
/// let toml = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";
/// let updated = update_cargo_toml_version(toml, "0.2.0");
/// assert_eq!(
///     updated,
///     Some("[package]\nname = \"demo\"\nversion = \"0.2.0\"\n".to_string())
/// );
///
/// let unchanged = update_cargo_toml_version(toml, "0.1.0");
/// assert_eq!(unchanged, None);
/// ```
pub fn update_cargo_toml_version(content: &str, new_version: &str) -> Option<String> {
    update_toml_version(content, &["package"], new_version)
}

/// Updates the `version` field under `[project]` or `[tool.poetry]` in a `pyproject.toml` or `project.toml` string.
///
/// Preserves comments, indentation, and formatting.
/// Returns `None` if neither section contains a `version` key,
/// or if the version is already equal to `new_version`.
///
/// # Arguments
///
/// * `content` - Raw string content of `pyproject.toml` or `project.toml`.
/// * `new_version` - Target SemVer version string (with or without 'v' prefix).
///
/// # Example
///
/// ```
/// use git_claw::workflow::update_pyproject_toml_version;
///
/// let toml = "[project]\nname = \"demo\"\nversion = \"0.1.0\"\n";
/// let updated = update_pyproject_toml_version(toml, "0.2.0");
/// assert_eq!(
///     updated,
///     Some("[project]\nname = \"demo\"\nversion = \"0.2.0\"\n".to_string())
/// );
///
/// let poetry_toml = "[tool.poetry]\nname = \"demo\"\nversion = \"0.1.0\"\n";
/// let updated_poetry = update_pyproject_toml_version(poetry_toml, "0.2.0");
/// assert_eq!(
///     updated_poetry,
///     Some("[tool.poetry]\nname = \"demo\"\nversion = \"0.2.0\"\n".to_string())
/// );
///
/// let unchanged = update_pyproject_toml_version(toml, "0.1.0");
/// assert_eq!(unchanged, None);
/// ```
pub fn update_pyproject_toml_version(content: &str, new_version: &str) -> Option<String> {
    update_toml_version(content, &["project", "tool.poetry"], new_version)
}

/// Validates SemVer, checks clean trunk, synchronizes manifest versions if present
/// (Rust `Cargo.toml` and Python `pyproject.toml` / `project.toml`), and creates
/// an annotated Git release tag.
///
/// # Arguments
///
/// * `options` - Tagging options specifying version and optional repository root.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::workflow::{tag_release, TagOptions};
///
/// # fn run() -> Result<(), git_claw::workflow::WorkflowError> {
/// tag_release(TagOptions {
///     version: "v0.3.0",
///     repo_root: Some(Path::new(".")),
/// })?;
/// # Ok(())
/// # }
/// ```
pub fn tag_release(options: TagOptions<'_>) -> Result<(), WorkflowError> {
    if !validate_semver(options.version) {
        return Err(WorkflowError::InvalidSemVer(options.version.to_string()));
    }

    let toplevel = resolve_toplevel(options.repo_root)?;
    let repo_name = resolve_repo_name(&toplevel);

    let config_path = toplevel.join(".git-claw.toml");
    let config = Config::load_or_default(&config_path, &repo_name)?;

    if !git_is_clean(&toplevel)? {
        return Err(WorkflowError::DirtyWorkingTree);
    }

    let clean_version = options.version.trim_start_matches('v');
    let mut files_to_commit: Vec<&str> = Vec::new();

    let cargo_toml_path = toplevel.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let content = std::fs::read_to_string(&cargo_toml_path)?;
        if let Some(new_content) = update_cargo_toml_version(&content, clean_version) {
            std::fs::write(&cargo_toml_path, new_content)?;
            files_to_commit.push("Cargo.toml");
            print_success(format!("Updated Cargo.toml to version '{}'", clean_version));

            let cargo_lock_path = toplevel.join("Cargo.lock");
            if cargo_lock_path.exists() {
                let _ = std::process::Command::new("cargo")
                    .current_dir(&toplevel)
                    .arg("check")
                    .output();
                files_to_commit.push("Cargo.lock");
            }
        }
    }

    let mut python_updated = false;
    for manifest_name in ["pyproject.toml", "project.toml"] {
        let manifest_path = toplevel.join(manifest_name);
        if manifest_path.exists() {
            let content = std::fs::read_to_string(&manifest_path)?;
            if let Some(new_content) = update_pyproject_toml_version(&content, clean_version) {
                std::fs::write(&manifest_path, new_content)?;
                files_to_commit.push(manifest_name);
                print_success(format!("Updated {} to version '{}'", manifest_name, clean_version));
                python_updated = true;
            }
        }
    }

    if python_updated {
        let uv_lock_path = toplevel.join("uv.lock");
        if uv_lock_path.exists() {
            let _ = std::process::Command::new("uv")
                .arg("lock")
                .current_dir(&toplevel)
                .output();
            files_to_commit.push("uv.lock");
        }
    }

    if !files_to_commit.is_empty() {
        let commit_msg = format!("chore: release {}", options.version);
        git_commit_files(&toplevel, &files_to_commit, &commit_msg)?;
    }

    let message = format!("Release {}", options.version);
    git_create_annotated_tag(&toplevel, options.version, config.main_branch(), &message)?;

    print_success(format!(
        "Created annotated release tag '{}' on branch '{}'",
        options.version,
        config.main_branch()
    ));

    Ok(())
}
