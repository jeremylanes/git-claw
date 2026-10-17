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
    let mut in_package = false;
    let mut replaced = false;
    let mut result = String::with_capacity(content.len() + 16);

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            let without_comment = if let Some(idx) = trimmed.find('#') {
                trimmed[..idx].trim()
            } else {
                trimmed
            };
            in_package = without_comment == "[package]"
                || without_comment
                    .strip_prefix('[')
                    .and_then(|s| s.strip_suffix(']'))
                    .map(|s| s.trim())
                    == Some("package");
        } else if in_package && !replaced {
            if let Some(rest) = trimmed.strip_prefix("version") {
                let rest = rest.trim_start();
                if let Some(after_eq) = rest.strip_prefix('=') {
                    let after_eq = after_eq.trim_start();
                    if let Some(quote) = after_eq.chars().next() {
                        if quote == '"' || quote == '\'' {
                            let after_quote = &after_eq[quote.len_utf8()..];
                            if let Some(end_idx) = after_quote.find(quote) {
                                let current_version = &after_quote[..end_idx];
                                if current_version == new_version {
                                    return None;
                                }
                                let old_pattern = format!("{}{}{}", quote, current_version, quote);
                                let new_pattern = format!("{}{}{}", quote, new_version, quote);
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

/// Validates SemVer, checks clean trunk, synchronizes Cargo.toml version if present,
/// and creates an annotated Git release tag.
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

    let cargo_toml_path = toplevel.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let clean_version = options.version.trim_start_matches('v');
        let content = std::fs::read_to_string(&cargo_toml_path)?;
        if let Some(new_content) = update_cargo_toml_version(&content, clean_version) {
            std::fs::write(&cargo_toml_path, new_content)?;

            let cargo_lock_path = toplevel.join("Cargo.lock");
            let commit_files: &[&str] = if cargo_lock_path.exists() {
                let _ = std::process::Command::new("cargo")
                    .current_dir(&toplevel)
                    .arg("check")
                    .output();
                &["Cargo.toml", "Cargo.lock"]
            } else {
                &["Cargo.toml"]
            };

            let commit_msg = format!("chore: release {}", options.version);
            git_commit_files(&toplevel, commit_files, &commit_msg)?;
            print_success(format!("Updated Cargo.toml to version '{}'", clean_version));
        }
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
