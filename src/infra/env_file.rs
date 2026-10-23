//! `.env.worktree` generation and environment variable injection.

use crate::core::config::EnvironmentConfig;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Sanitizes a string to lowercase alphanumerics and underscores for Docker Compose project names.
///
/// # Examples
///
/// ```
/// use git_claw::infra::env_file::sanitize_compose_project_name;
///
/// let name = sanitize_compose_project_name("my-repo", "feature-x", 1);
/// assert_eq!(name, "my_repo_feature_x_1");
/// ```
pub fn sanitize_compose_project_name(repo: &str, worktree_name: &str, slot_id: u32) -> String {
    let raw = format!("{}_{}_{}", repo, worktree_name, slot_id);
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Extracts the base database name from an existing value, stripping quotes,
/// leading `test_` or `test` prefixes, and trailing slot or worker numbers.
///
/// If the extracted name is empty or numeric and a fallback is provided,
/// the fallback is used instead.
///
/// # Examples
///
/// ```
/// use git_claw::infra::env_file::extract_base_db;
///
/// assert_eq!(extract_base_db("test_tune_slot_1", Some("tune")), "tune");
/// assert_eq!(extract_base_db("test_tune_1", Some("tune")), "tune");
/// assert_eq!(extract_base_db("test_db", None), "db");
/// ```
pub fn extract_base_db(val: &str, fallback: Option<&str>) -> String {
    let clean = val.trim().trim_matches(|c| c == '\'' || c == '"');
    let mut stripped = clean
        .strip_prefix("test_")
        .unwrap_or_else(|| clean.strip_prefix("test").unwrap_or(clean));

    if let Some(pos) = stripped.rfind("_slot_") {
        let suffix = &stripped[pos + 6..];
        let parts: Vec<&str> = suffix.split('_').collect();
        if !parts.is_empty()
            && parts
                .iter()
                .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        {
            stripped = &stripped[..pos];
        }
    } else if let Some(pos) = stripped.rfind('_') {
        let suffix = &stripped[pos + 1..];
        if pos > 0 && !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            stripped = &stripped[..pos];
        }
    }

    if !stripped.is_empty() && !stripped.chars().all(|c| c.is_ascii_digit()) {
        stripped.to_string()
    } else if let Some(fb) = fallback {
        fb.to_string()
    } else if !stripped.is_empty() {
        stripped.to_string()
    } else {
        "db".to_string()
    }
}

/// Generates `.env.worktree` inside the worktree directory.
///
/// Injects slot ID, worktree name and path, Docker Compose project name,
/// isolated test database names (`TEST_DATABASE_NAME`, `CLAW_TEST_DB`),
/// calculated dynamic ports, optional Python runtime environment (`PYENV_VERSION`),
/// and virtualenv variables (`VIRTUAL_ENV`, `PATH`) when `.venv` is present.
///
/// # Examples
///
/// ```
/// use git_claw::infra::env_file::write_env_worktree;
/// use git_claw::core::config::EnvironmentConfig;
/// use std::collections::BTreeMap;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let dir = tempdir().unwrap();
/// let wt = dir.path().join("wt-1");
/// fs::create_dir_all(&wt).unwrap();
///
/// let env = EnvironmentConfig {
///     name: Some("3.12.0".to_string()),
///     ..Default::default()
/// };
/// write_env_worktree(&wt, "repo", "wt-1", 1, &BTreeMap::new(), Some(&env)).unwrap();
/// let content = fs::read_to_string(wt.join(".env.worktree")).unwrap();
/// assert!(content.contains("PYENV_VERSION=3.12.0"));
/// assert!(content.contains("TEST_DATABASE_NAME=test_repo_slot_1"));
/// assert!(content.contains("CLAW_TEST_DB=test_repo_slot_1"));
/// ```
pub fn write_env_worktree(
    worktree_path: &Path,
    repo_name: &str,
    worktree_name: &str,
    slot_id: u32,
    effective_ports: &BTreeMap<String, u16>,
    environment: Option<&EnvironmentConfig>,
) -> std::io::Result<()> {
    let compose_project = sanitize_compose_project_name(repo_name, worktree_name, slot_id);
    let env_path = worktree_path.join(".env.worktree");

    let mut lines = Vec::new();
    lines.push(format!("CLAW_SLOT_ID={}", slot_id));
    lines.push(format!("CLAW_WORKTREE_NAME={}", worktree_name));
    lines.push(format!("CLAW_WORKTREE_PATH={}", worktree_path.display()));
    lines.push(format!("COMPOSE_PROJECT_NAME={}", compose_project));

    let default_db_name = repo_name.replace('-', "_");
    let test_db_name = format!("test_{}_slot_{}", default_db_name, slot_id);
    lines.push(format!("TEST_DATABASE_NAME={}", test_db_name));
    lines.push(format!("CLAW_TEST_DB={}", test_db_name));

    for (key, port) in effective_ports {
        lines.push(format!("{}={}", key, port));
    }

    if let Some(env) = environment {
        if let Some(ref name) = env.name {
            lines.push(format!("PYENV_VERSION={}", name));
        }
    }

    let venv_path = worktree_path.join(".venv");
    if venv_path.exists() {
        lines.push(format!("VIRTUAL_ENV={}", venv_path.display()));
        lines.push(format!("PATH={}/bin:$PATH", venv_path.display()));
    }

    let content = lines.join("\n") + "\n";
    fs::write(&env_path, content)?;

    Ok(())
}

/// Updates port variables, Compose project name, and test database configuration in-place
/// within an environment file content string, and appends any missing variables at the end.
///
/// If `TEST_DATABASE_NAME` or `CLAW_TEST_DB` is present, it is rewritten to `test_{base}_slot_{slot_id}`.
/// If absent but a base database is detected (`DATABASE_NAME`, `POSTGRES_DB`, or `MYSQL_DATABASE`),
/// both `TEST_DATABASE_NAME=test_{base}_slot_{slot_id}` and `CLAW_TEST_DB=test_{base}_slot_{slot_id}` are appended.
///
/// # Examples
///
/// ```
/// use git_claw::infra::env_file::update_env_content_with_ports;
/// use std::collections::BTreeMap;
///
/// let content = "DATABASE_NAME=mydb\nPORT=3000\n";
/// let updated = update_env_content_with_ports(content, &BTreeMap::new(), 2, None);
/// assert!(updated.contains("TEST_DATABASE_NAME=test_mydb_slot_2"));
/// assert!(updated.contains("CLAW_TEST_DB=test_mydb_slot_2"));
/// ```
pub fn update_env_content_with_ports(
    content: &str,
    raw_ports: &BTreeMap<String, u16>,
    slot_id: u32,
    compose_project_name: Option<&str>,
) -> String {
    let mut updated_lines = Vec::new();
    let mut matched_ports = BTreeSet::new();
    let mut matched_compose_project = false;
    let mut matched_test_db = false;
    let mut matched_claw_test_db = false;

    // Scan for base database name
    let mut base_db_name: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key_upper = key.trim().to_ascii_uppercase();
            if key_upper == "DATABASE_NAME"
                || key_upper == "POSTGRES_DB"
                || key_upper == "MYSQL_DATABASE"
            {
                let clean = val.trim().trim_matches(|c| c == '\'' || c == '"');
                if !clean.is_empty() && (base_db_name.is_none() || key_upper == "DATABASE_NAME") {
                    base_db_name = Some(clean.to_string());
                }
            } else if (key_upper == "TEST_DATABASE_NAME" || key_upper == "CLAW_TEST_DB")
                && base_db_name.is_none()
            {
                let extracted = extract_base_db(val, None);
                if !extracted.is_empty() && extracted != "db" {
                    base_db_name = Some(extracted);
                }
            }
        }
    }

    let mut effective_map = BTreeMap::new();
    for (k, v) in raw_ports {
        let eff = v.saturating_add(slot_id as u16);
        let formatted = crate::core::port::format_port_env_key(k);
        effective_map.insert(k.to_ascii_uppercase(), (k.clone(), eff));
        effective_map.insert(formatted, (k.clone(), eff));
    }

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            updated_lines.push(line.to_string());
            continue;
        }

        if let Some((key, val)) = line.split_once('=') {
            let key_trimmed = key.trim();
            let key_upper = key_trimmed.to_ascii_uppercase();

            if key_upper == "COMPOSE_PROJECT_NAME" {
                if let Some(proj) = compose_project_name {
                    updated_lines.push(format!("COMPOSE_PROJECT_NAME={}", proj));
                    matched_compose_project = true;
                    continue;
                }
            }

            if key_upper == "TEST_DATABASE_NAME" {
                matched_test_db = true;
                let base = extract_base_db(val, base_db_name.as_deref());
                updated_lines.push(format!("TEST_DATABASE_NAME=test_{}_slot_{}", base, slot_id));
                continue;
            }

            if key_upper == "CLAW_TEST_DB" {
                matched_claw_test_db = true;
                let base = extract_base_db(val, base_db_name.as_deref());
                updated_lines.push(format!("CLAW_TEST_DB=test_{}_slot_{}", base, slot_id));
                continue;
            }

            if let Some((orig_key, eff_port)) = effective_map.get(&key_upper) {
                updated_lines.push(format!("{}={}", key.trim_end(), eff_port));
                matched_ports.insert(orig_key.clone());
            } else {
                updated_lines.push(line.to_string());
            }
        } else {
            updated_lines.push(line.to_string());
        }
    }

    // Append any unmatched declared ports
    for (raw_key, base_port) in raw_ports {
        if !matched_ports.contains(raw_key) {
            let eff = base_port.saturating_add(slot_id as u16);
            let formatted_key = crate::core::port::format_port_env_key(raw_key);
            updated_lines.push(format!("{}={}", formatted_key, eff));
        }
    }

    // Append COMPOSE_PROJECT_NAME if not already matched
    if let Some(proj) = compose_project_name {
        if !matched_compose_project {
            updated_lines.push(format!("COMPOSE_PROJECT_NAME={}", proj));
        }
    }

    // Append TEST_DATABASE_NAME and CLAW_TEST_DB if not already matched
    if let Some(ref base) = base_db_name {
        if !matched_test_db {
            updated_lines.push(format!("TEST_DATABASE_NAME=test_{}_slot_{}", base, slot_id));
        }
        if !matched_claw_test_db {
            updated_lines.push(format!("CLAW_TEST_DB=test_{}_slot_{}", base, slot_id));
        }
    }

    let mut result = updated_lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Recursively copies a directory and its contents from `src` to `dst`.
///
/// # Examples
///
/// ```
/// use git_claw::infra::env_file::copy_dir_all;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let src = tempdir().unwrap();
/// let dst = tempdir().unwrap();
/// fs::write(src.path().join("file.txt"), "content").unwrap();
/// copy_dir_all(src.path(), &dst.path().join("sub")).unwrap();
/// assert!(dst.path().join("sub/file.txt").exists());
/// ```
pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Copies declared untracked files and directories from `repo_root` to `worktree_path`,
/// creating parent directories, and updating matched port variables, COMPOSE_PROJECT_NAME,
/// and test database configuration in `.env` files in-place.
///
/// # Examples
///
/// ```
/// use git_claw::infra::env_file::copy_and_merge_untracked_files;
/// use std::collections::BTreeMap;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let root = tempdir().unwrap();
/// let wt = tempdir().unwrap();
/// fs::write(root.path().join("app.env"), "DATABASE_NAME=tune\n").unwrap();
/// copy_and_merge_untracked_files(
///     root.path(),
///     wt.path(),
///     &["app.env".to_string()],
///     &BTreeMap::new(),
///     1,
///     Some("proj_1"),
/// ).unwrap();
/// let content = fs::read_to_string(wt.path().join("app.env")).unwrap();
/// assert!(content.contains("TEST_DATABASE_NAME=test_tune_slot_1"));
/// ```
pub fn copy_and_merge_untracked_files(
    repo_root: &Path,
    worktree_path: &Path,
    files_to_copy: &[String],
    raw_ports: &BTreeMap<String, u16>,
    slot_id: u32,
    compose_project_name: Option<&str>,
) -> std::io::Result<()> {
    for rel_path in files_to_copy {
        let src = repo_root.join(rel_path);
        let dest = worktree_path.join(rel_path);

        if !src.exists() {
            continue;
        }

        if src.is_dir() {
            copy_dir_all(&src, &dest)?;
            continue;
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        let is_env_file = rel_path.ends_with(".env") || rel_path.contains(".env");
        if is_env_file {
            let content = fs::read_to_string(&src)?;
            let updated =
                update_env_content_with_ports(&content, raw_ports, slot_id, compose_project_name);
            fs::write(&dest, updated)?;
        } else {
            fs::copy(&src, &dest)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_compose_project_name() {
        assert_eq!(
            sanitize_compose_project_name("git-claw", "user-auth", 2),
            "git_claw_user_auth_2"
        );
        assert_eq!(
            sanitize_compose_project_name("Repo.V1", "feat#1", 1),
            "repo_v1_feat_1_1"
        );
    }

    #[test]
    fn test_extract_base_db() {
        assert_eq!(extract_base_db("test_tune_slot_1", Some("tune")), "tune");
        assert_eq!(extract_base_db("test_tune_slot_2", None), "tune");
        assert_eq!(extract_base_db("test_tune_1", Some("tune")), "tune");
        assert_eq!(extract_base_db("test_tune", None), "tune");
        assert_eq!(extract_base_db("test_tune_2", None), "tune");
        assert_eq!(extract_base_db("tune", None), "tune");
        assert_eq!(extract_base_db("test_1", Some("tune")), "tune");
        assert_eq!(extract_base_db("", Some("tune")), "tune");
        assert_eq!(extract_base_db("test_postgres_db_3", None), "postgres_db");
        assert_eq!(
            extract_base_db("test_postgres_db_slot_3", None),
            "postgres_db"
        );
    }

    #[test]
    fn test_update_env_content_with_ports() {
        let original = r#"
# Comments
SECRET=xyz
APP_PORT=80
OTHER=abc
"#;
        let mut ports = BTreeMap::new();
        ports.insert("APP_PORT".to_string(), 80);
        ports.insert("EXTRA_PORT".to_string(), 9000);

        let updated = update_env_content_with_ports(original, &ports, 2, Some("my_proj_1"));
        assert!(updated.contains("APP_PORT=82"));
        assert!(updated.contains("SECRET=xyz"));
        assert!(updated.contains("EXTRA_PORT=9002"));
        assert!(updated.contains("COMPOSE_PROJECT_NAME=my_proj_1"));
        assert!(updated.contains("# Comments"));
    }

    #[test]
    fn test_update_env_content_with_test_db() {
        // Appends TEST_DATABASE_NAME and CLAW_TEST_DB when DATABASE_NAME is present
        let original = "DATABASE_NAME=tune\nPORT=80\n";
        let updated = update_env_content_with_ports(original, &BTreeMap::new(), 1, None);
        assert!(updated.contains("DATABASE_NAME=tune"));
        assert!(updated.contains("TEST_DATABASE_NAME=test_tune_slot_1"));
        assert!(updated.contains("CLAW_TEST_DB=test_tune_slot_1"));

        // Rewrites existing TEST_DATABASE_NAME and CLAW_TEST_DB in-place
        let existing =
            "DATABASE_NAME=tune\nTEST_DATABASE_NAME=test_tune_slot_1\nCLAW_TEST_DB=test_tune_slot_1\n";
        let updated_slot2 = update_env_content_with_ports(existing, &BTreeMap::new(), 2, None);
        assert!(updated_slot2.contains("TEST_DATABASE_NAME=test_tune_slot_2"));
        assert!(updated_slot2.contains("CLAW_TEST_DB=test_tune_slot_2"));
        assert!(!updated_slot2.contains("test_tune_slot_1"));

        // Supports POSTGRES_DB
        let pg = "POSTGRES_DB=pgtune\n";
        let updated_pg = update_env_content_with_ports(pg, &BTreeMap::new(), 3, None);
        assert!(updated_pg.contains("TEST_DATABASE_NAME=test_pgtune_slot_3"));
        assert!(updated_pg.contains("CLAW_TEST_DB=test_pgtune_slot_3"));

        // Supports MYSQL_DATABASE
        let my = "MYSQL_DATABASE=mytune\n";
        let updated_my = update_env_content_with_ports(my, &BTreeMap::new(), 4, None);
        assert!(updated_my.contains("TEST_DATABASE_NAME=test_mytune_slot_4"));
        assert!(updated_my.contains("CLAW_TEST_DB=test_mytune_slot_4"));
    }
}
