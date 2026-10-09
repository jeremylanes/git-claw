use git_claw::core::branch::BranchType;
use git_claw::workflow::{start_worktree, StartOptions};
use std::fs;
use tempfile::tempdir;

fn setup_repo() -> (tempfile::TempDir, tempfile::TempDir) {
    let repo_dir = tempdir().expect("repo dir");
    let wt_parent = tempdir().expect("wt parent dir");

    let repo_path = repo_dir.path();
    let init = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(repo_path)
        .output()
        .expect("git init");
    assert!(init.status.success());

    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "tester@test.local"])
        .current_dir(repo_path)
        .output();

    fs::write(repo_path.join("README.md"), "# Test Files Copy").expect("write readme");

    let add = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    assert!(add.status.success());

    let commit = std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");
    assert!(commit.status.success());

    (repo_dir, wt_parent)
}

#[test]
fn test_untracked_files_duplication_and_port_merging() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    // 1. Create untracked .env and config files in primary repo
    let primary_env = r#"
# Database config
DATABASE_URL=postgres://localhost:5432/db
APP_PORT=80
SECRET_KEY=secret123
"#;
    fs::write(repo_path.join(".env"), primary_env).expect("write primary .env");

    let sub_dir = repo_path.join("config");
    fs::create_dir_all(&sub_dir).expect("create config dir");
    fs::write(sub_dir.join("settings.env"), "CUSTOM_PORT=5000\n").expect("write settings.env");
    fs::write(repo_path.join("untracked.txt"), "verbatim content").expect("write untracked");

    let media_dir = repo_path.join("src/media");
    fs::create_dir_all(&media_dir).expect("create media dir");
    fs::write(media_dir.join("photo.jpg"), "photo-bytes").expect("write photo");

    // 2. Configure .git-claw.toml with [files] copy and [ports]
    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[files]
copy = [".env", "config/settings.env", "untracked.txt", "src/media", "nonexistent.env"]

[ports]
APP_PORT = 80
CUSTOM_PORT = 5000
NEW_EXTRA_PORT = 9999
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");

    // 3. Start worktree
    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "copy-test",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");

    let wt_path = wt_parent.path().join("copy-test");
    assert!(wt_path.exists());

    // 4. Verify copied .env in worktree
    let wt_env_path = wt_path.join(".env");
    assert!(wt_env_path.exists());
    let wt_env = fs::read_to_string(&wt_env_path).expect("read wt .env");

    assert!(wt_env.contains("APP_PORT=81")); // in-place offset: 80 + slot 1 = 81
    assert!(wt_env.contains("NEW_EXTRA_PORT=10000")); // appended port
    assert!(wt_env.contains("DATABASE_URL=postgres://localhost:5432/db")); // preserved
    assert!(wt_env.contains("SECRET_KEY=secret123")); // preserved
    assert!(wt_env.contains("# Database config")); // comments preserved

    // 5. Verify copied settings.env in worktree
    let wt_settings_path = wt_path.join("config/settings.env");
    assert!(wt_settings_path.exists());
    let wt_settings = fs::read_to_string(&wt_settings_path).expect("read wt settings");
    assert!(wt_settings.contains("CUSTOM_PORT=5001")); // in-place offset: 5000 + 1 = 5001

    // 6. Verify non-env file copied verbatim
    let wt_untracked = fs::read_to_string(wt_path.join("untracked.txt")).expect("read untracked");
    assert_eq!(wt_untracked, "verbatim content");

    // 7. Verify directory copied recursively
    let wt_photo = fs::read_to_string(wt_path.join("src/media/photo.jpg")).expect("read photo");
    assert_eq!(wt_photo, "photo-bytes");

    // 7. Verify primary repo .env is UNTOUCHED
    let primary_check = fs::read_to_string(repo_path.join(".env")).expect("read primary");
    assert!(primary_check.contains("APP_PORT=80"));
    assert!(!primary_check.contains("APP_PORT=81"));
}
