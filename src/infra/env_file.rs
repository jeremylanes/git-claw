//! `.env.worktree` generation and environment variable injection.

use std::collections::BTreeMap;
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
}
