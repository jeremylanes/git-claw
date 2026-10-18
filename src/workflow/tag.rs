//! SemVer release tag manager.

use crate::cli::output::print_success;
use crate::core::config::Config;
use crate::core::conventional::{determine_bump, parse_conventional_commit, ConventionalCommit};
use crate::core::semver::{parse_semver, validate_semver, SemVer};
use crate::infra::git::{
    git_commit_files, git_create_annotated_tag, git_get_commits_since, git_get_latest_tag,
    git_is_clean, resolve_primary_repo_root, resolve_repo_name,
};
use crate::workflow::changelog::{
    current_date_ymd, generate_changelog_entry, update_changelog_file,
};
use crate::workflow::error::WorkflowError;
use std::path::Path;

/// Options for release tagging workflow.
pub struct TagOptions<'a> {
    /// Target release version string (e.g., "v0.3.0" or "0.3.0").
    /// If omitted (`None`), inferred automatically from conventional commits.
    pub version: Option<&'a str>,
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
                    sections.contains(&sec)
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

/// Helper to extract base SemVer from manifest files (Cargo.toml, pyproject.toml, project.toml).
fn read_base_version_from_manifests(repo_root: &Path) -> Option<SemVer> {
    let cargo_toml = repo_root.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            if let Ok(val) = content.parse::<toml::Value>() {
                if let Some(ver) = val
                    .get("package")
                    .and_then(|p| p.get("version"))
                    .and_then(|v| v.as_str())
                {
                    if let Some(mut sem) = parse_semver(ver) {
                        sem.has_v_prefix = true;
                        return Some(sem);
                    }
                }
            }
        }
    }
    for manifest_name in ["pyproject.toml", "project.toml"] {
        let manifest_path = repo_root.join(manifest_name);
        if manifest_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(val) = content.parse::<toml::Value>() {
                    let ver = val
                        .get("project")
                        .and_then(|p| p.get("version"))
                        .and_then(|v| v.as_str())
                        .or_else(|| {
                            val.get("tool")
                                .and_then(|t| t.get("poetry"))
                                .and_then(|p| p.get("version"))
                                .and_then(|v| v.as_str())
                        });
                    if let Some(ver_str) = ver {
                        if let Some(mut sem) = parse_semver(ver_str) {
                            sem.has_v_prefix = true;
                            return Some(sem);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Validates SemVer, checks clean trunk, synchronizes manifest versions if present
/// (Rust `Cargo.toml` and Python `pyproject.toml` / `project.toml`), and creates
/// an annotated Git release tag.
///
/// If `options.version` is omitted (`None`), the next SemVer version and release notes
/// are automatically inferred from Conventional Commits since the latest Git tag.
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
/// // Explicit version:
/// tag_release(TagOptions {
///     version: Some("v0.3.0"),
///     repo_root: Some(Path::new(".")),
/// })?;
///
/// // Inferred from conventional commits:
/// tag_release(TagOptions {
///     version: None,
///     repo_root: Some(Path::new(".")),
/// })?;
/// # Ok(())
/// # }
/// ```
pub fn tag_release(options: TagOptions<'_>) -> Result<(), WorkflowError> {
    let toplevel = resolve_primary_repo_root(options.repo_root)?;
    let repo_name = resolve_repo_name(&toplevel);

    let config_path = toplevel.join(".git-claw.toml");
    let config = Config::load_or_default(&config_path, &repo_name)?;

    if !git_is_clean(&toplevel)? {
        return Err(WorkflowError::DirtyWorkingTree);
    }

    let mut files_to_commit: Vec<&str> = Vec::new();

    let target_version = match options.version {
        Some(v) => {
            if !validate_semver(v) {
                return Err(WorkflowError::InvalidSemVer(v.to_string()));
            }
            v.to_string()
        }
        None => {
            let latest_tag = git_get_latest_tag(&toplevel)?;
            let raw_commits = git_get_commits_since(&toplevel, latest_tag.as_deref())?;
            if raw_commits.is_empty() {
                return Err(WorkflowError::ExecutionFailed(
                    "No commits found since last release tag".to_string(),
                ));
            }

            let conventional_commits: Vec<ConventionalCommit> = raw_commits
                .iter()
                .filter_map(|msg| parse_conventional_commit(msg))
                .collect();
            let bump_type = determine_bump(&conventional_commits);

            let base_semver = if let Some(tag) = &latest_tag {
                parse_semver(tag).unwrap_or(SemVer {
                    major: 0,
                    minor: 1,
                    patch: 0,
                    has_v_prefix: tag.starts_with('v'),
                })
            } else {
                read_base_version_from_manifests(&toplevel).unwrap_or(SemVer {
                    major: 0,
                    minor: 1,
                    patch: 0,
                    has_v_prefix: true,
                })
            };

            let target = base_semver.bump(bump_type).to_string();
            print_success(format!(
                "Inferred bump: {:?} -> version '{}'",
                bump_type, target
            ));

            let date = current_date_ymd();
            let entry = generate_changelog_entry(&target, &date, &conventional_commits);
            update_changelog_file(&toplevel, &entry)?;
            files_to_commit.push("CHANGELOG.md");
            print_success("Updated CHANGELOG.md".to_string());

            target
        }
    };

    let clean_version = target_version.trim_start_matches('v');

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
                print_success(format!(
                    "Updated {} to version '{}'",
                    manifest_name, clean_version
                ));
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
        let commit_msg = format!("chore: release {}", target_version);
        git_commit_files(&toplevel, &files_to_commit, &commit_msg)?;
    }

    let message = format!("Release {}", target_version);
    git_create_annotated_tag(&toplevel, &target_version, config.main_branch(), &message)?;

    print_success(format!(
        "Created annotated release tag '{}' on branch '{}'",
        target_version,
        config.main_branch()
    ));

    Ok(())
}
