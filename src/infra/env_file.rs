//! `.env.worktree` generation and environment variable injection.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Sanitizes a string to lowercase alphanumerics and underscores for Docker Compose project names.
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

/// Generates `.env.worktree` inside the worktree directory.
pub fn write_env_worktree(
    worktree_path: &Path,
    repo_name: &str,
    worktree_name: &str,
    slot_id: u32,
    effective_ports: &BTreeMap<String, u16>,
) -> std::io::Result<()> {
    let compose_project = sanitize_compose_project_name(repo_name, worktree_name, slot_id);
    let env_path = worktree_path.join(".env.worktree");

    let mut lines = Vec::new();
    lines.push(format!("CLAW_SLOT_ID={}", slot_id));
    lines.push(format!("CLAW_WORKTREE_NAME={}", worktree_name));
    lines.push(format!("CLAW_WORKTREE_PATH={}", worktree_path.display()));
    lines.push(format!("COMPOSE_PROJECT_NAME={}", compose_project));

    for (key, port) in effective_ports {
        lines.push(format!("{}={}", key, port));
    }

    let content = lines.join("\n") + "\n";
    fs::write(&env_path, content)?;

    Ok(())
}

/// Updates port variables in-place within an environment file content string,
/// and appends any missing port variables at the end.
pub fn update_env_content_with_ports(
    content: &str,
    raw_ports: &BTreeMap<String, u16>,
    slot_id: u32,
    compose_project_name: Option<&str>,
) -> String {
    let mut updated_lines = Vec::new();
    let mut matched_ports = BTreeSet::new();
    let mut matched_compose_project = false;

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

        if let Some((key, _val)) = line.split_once('=') {
            let key_trimmed = key.trim();
            let key_upper = key_trimmed.to_ascii_uppercase();

            if key_upper == "COMPOSE_PROJECT_NAME" {
                if let Some(proj) = compose_project_name {
                    updated_lines.push(format!("COMPOSE_PROJECT_NAME={}", proj));
                    matched_compose_project = true;
                    continue;
                }
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

    let mut result = updated_lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Recursively copies a directory and its contents from `src` to `dst`.
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
/// creating parent directories, and updating matched port variables and COMPOSE_PROJECT_NAME
/// in `.env` files in-place.
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
            let updated = update_env_content_with_ports(&content, raw_ports, slot_id, compose_project_name);
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
}
