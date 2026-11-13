//! Integration tests for structured `--json` CLI output across lifecycle commands.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn setup_git_repo(
    with_remote: bool,
) -> (
    Option<tempfile::TempDir>,
    tempfile::TempDir,
    tempfile::TempDir,
) {
    let remote_dir = if with_remote {
        let dir = tempdir().expect("remote dir");
        let init_remote = std::process::Command::new("git")
            .args(["init", "--bare", "-b", "main"])
            .current_dir(dir.path())
            .output()
            .expect("git init bare");
        assert!(init_remote.status.success());
        Some(dir)
    } else {
        None
    };

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

    if let Some(ref remote) = remote_dir {
        let add_remote = std::process::Command::new("git")
            .args(["remote", "add", "origin", &remote.path().to_string_lossy()])
            .current_dir(repo_path)
            .output()
            .expect("git remote add");
        assert!(add_remote.status.success());
    }

    let canonical_wt_parent = canonical_str(wt_parent.path());
    fs::write(repo_path.join("README.md"), "# JSON Test").expect("write readme");
    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[ports]
web = 8000
"#,
        canonical_wt_parent
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");

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

    if remote_dir.is_some() {
        let push = std::process::Command::new("git")
            .args(["push", "-u", "origin", "main"])
            .current_dir(repo_path)
            .output()
            .expect("git push main");
        assert!(push.status.success());
    }

    (remote_dir, repo_dir, wt_parent)
}

fn parse_stdout_json(output: &std::process::Output) -> Value {
    let stdout = String::from_utf8(output.stdout.clone()).expect("utf8 stdout");
    serde_json::from_str(&stdout).unwrap_or_else(|err| {
        panic!(
            "stdout is not valid JSON: {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            err,
            stdout,
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn canonical_str(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

#[test]
fn test_cli_list_json_empty_and_active() {
    let (_remote, repo_dir, wt_parent) = setup_git_repo(false);
    let repo_path = repo_dir.path();

    let empty_out = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["list", "--json"])
        .output()
        .expect("run list --json");
    assert!(empty_out.status.success());
    let empty_json = parse_stdout_json(&empty_out);
    assert_eq!(empty_json["worktrees"].as_array().unwrap().len(), 0);

    let start_out = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["feature", "start", "alpha", "--json"])
        .output()
        .expect("run start --json");
    assert!(start_out.status.success());

    let list_out = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["claw", "list", "--json"])
        .output()
        .expect("run claw list --json");
    assert!(list_out.status.success());
    let list_json = parse_stdout_json(&list_out);
    let items = list_json["worktrees"].as_array().expect("worktrees array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["slot_id"], 1);
    assert_eq!(items[0]["name"], "alpha");
    assert_eq!(items[0]["branch"], "feature/alpha");
    assert_eq!(
        items[0]["path"],
        canonical_str(&wt_parent.path().join("alpha"))
    );
    assert_eq!(items[0]["ports"]["PORT_WEB"], 8001);
    assert_eq!(items[0]["status"], "active");
}

#[test]
fn test_cli_start_json_all_branch_types_and_noisy_hook() {
    let (_remote, repo_dir, wt_parent) = setup_git_repo(false);
    let repo_path = repo_dir.path();

    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[ports]
web = 8000

[hooks]
post_start = "echo NOISY_HOOK_STDOUT"
"#,
        canonical_str(wt_parent.path())
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");
    let _ = std::process::Command::new("git")
        .args(["commit", "-am", "Add noisy post_start hook"])
        .current_dir(repo_path)
        .output();

    let feat_out = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["feature", "start", "auth-flow", "--json"])
        .output()
        .expect("feature start --json");
    assert!(feat_out.status.success());
    assert!(String::from_utf8_lossy(&feat_out.stderr).contains("NOISY_HOOK_STDOUT"));
    let feat_json = parse_stdout_json(&feat_out);
    assert_eq!(feat_json["slot_id"], 1);
    assert_eq!(feat_json["name"], "auth-flow");
    assert_eq!(feat_json["branch"], "feature/auth-flow");
    assert_eq!(
        feat_json["path"],
        canonical_str(&wt_parent.path().join("auth-flow"))
    );
    assert_eq!(feat_json["ports"]["PORT_WEB"], 8001);
    assert!(feat_json["test_database"]
        .as_str()
        .unwrap()
        .ends_with("_slot_1"));

    for (branch_type, name, expected_branch, expected_slot) in [
        ("bugfix", "fix-one", "bugfix/fix-one", 2),
        ("hotfix", "hot-one", "hotfix/hot-one", 3),
        ("spike", "exp-one", "spike/exp-one", 4),
    ] {
        let out = Command::cargo_bin("git-claw")
            .expect("binary")
            .current_dir(repo_path)
            .args([branch_type, "start", name, "--json"])
            .output()
            .expect("start --json");
        assert!(out.status.success());
        let json = parse_stdout_json(&out);
        assert_eq!(json["slot_id"], expected_slot);
        assert_eq!(json["name"], name);
        assert_eq!(json["branch"], expected_branch);
        assert_eq!(json["ports"]["PORT_WEB"], 8000 + expected_slot);
    }
}

#[test]
fn test_cli_publish_finish_and_spike_drop_json() {
    let (_remote, repo_dir, wt_parent) = setup_git_repo(true);
    let repo_path = repo_dir.path();

    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[ports]
web = 8000

[hooks]
pre_finish = "echo NOISY_PRE_FINISH"
post_finish = "echo NOISY_POST_FINISH"
"#,
        canonical_str(wt_parent.path())
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");
    let _ = std::process::Command::new("git")
        .args(["commit", "-am", "Add finish hooks"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(repo_path)
        .output();

    // 1. Start and publish feature
    let start_feat = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["feature", "start", "billing", "--json"])
        .output()
        .expect("start billing");
    assert!(start_feat.status.success());

    let wt_billing = wt_parent.path().join("billing");
    fs::write(wt_billing.join("bill.txt"), "billing").expect("write bill.txt");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_billing)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Add billing"])
        .current_dir(&wt_billing)
        .output();

    let pub_out = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["feature", "publish", "billing", "--json"])
        .output()
        .expect("feature publish --json");
    assert!(pub_out.status.success());
    let pub_json = parse_stdout_json(&pub_out);
    assert_eq!(pub_json["name"], "billing");
    assert_eq!(pub_json["branch"], "feature/billing");
    assert_eq!(pub_json["remote"], "origin");

    // 2. Finish feature with remote deletion
    let finish_out = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["finish", "billing", "--json"])
        .output()
        .expect("finish --json");
    assert!(finish_out.status.success());
    let finish_json = parse_stdout_json(&finish_out);
    assert_eq!(finish_json["name"], "billing");
    assert_eq!(finish_json["branch"], "feature/billing");
    assert_eq!(finish_json["target_branch"], "main");
    assert_eq!(finish_json["slot_id"], 1);
    assert_eq!(finish_json["primary_root"], canonical_str(repo_path));
    assert_eq!(finish_json["remote_deleted"], true);

    // 3. Top-level publish --json + finish --keep-remote --json
    let start_bug = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["bugfix", "start", "fix-auth", "--json"])
        .output()
        .expect("start fix-auth");
    assert!(start_bug.status.success());

    let wt_bug = wt_parent.path().join("fix-auth");
    fs::write(wt_bug.join("fix.txt"), "fix").expect("write fix.txt");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_bug)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Fix auth"])
        .current_dir(&wt_bug)
        .output();

    let pub_short = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["publish", "fix-auth", "--json"])
        .output()
        .expect("publish --json");
    assert!(pub_short.status.success());
    let pub_short_json = parse_stdout_json(&pub_short);
    assert_eq!(pub_short_json["name"], "fix-auth");
    assert_eq!(pub_short_json["branch"], "bugfix/fix-auth");
    assert_eq!(pub_short_json["remote"], "origin");

    let finish_keep = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["finish", "fix-auth", "--keep-remote", "--json"])
        .output()
        .expect("finish --keep-remote --json");
    assert!(finish_keep.status.success());
    let finish_keep_json = parse_stdout_json(&finish_keep);
    assert_eq!(finish_keep_json["remote_deleted"], false);

    // 4. Spike drop --json
    let start_spike = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["spike", "start", "poc-one", "--json"])
        .output()
        .expect("spike start");
    assert!(start_spike.status.success());

    let drop_out = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["spike", "drop", "poc-one", "--json"])
        .output()
        .expect("spike drop --json");
    assert!(drop_out.status.success());
    let drop_json = parse_stdout_json(&drop_out);
    assert_eq!(drop_json["name"], "poc-one");
    assert_eq!(drop_json["branch"], "spike/poc-one");
    assert_eq!(drop_json["slot_id"], 1);
    assert_eq!(drop_json["primary_root"], canonical_str(repo_path));
}

#[test]
fn test_cli_init_json_requires_yes_and_emits_summary() {
    let repo_dir = tempdir().expect("repo dir");
    let repo_path = repo_dir.path();

    let init = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(repo_path)
        .output()
        .expect("git init");
    assert!(init.status.success());

    fs::write(repo_path.join(".env"), "APP_PORT=4000\n").expect("write .env");
    fs::write(repo_path.join(".python-version"), "3.12.1\n").expect("write .python-version");
    fs::write(
        repo_path.join("docker-compose.yml"),
        "services:\n  web:\n    image: app\n",
    )
    .expect("write compose");

    // 1. init --json without --yes must fail explicitly without creating config
    let err_out = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["init", "--json"])
        .output()
        .expect("init --json");
    assert!(!err_out.status.success());
    assert!(String::from_utf8_lossy(&err_out.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&err_out.stderr).contains("--yes"));
    assert!(!repo_path.join(".git-claw.toml").exists());

    // 2. init --yes --json must succeed with pure JSON
    let ok_out = Command::cargo_bin("git-claw")
        .expect("binary")
        .current_dir(repo_path)
        .args(["init", "--yes", "--json"])
        .output()
        .expect("init --yes --json");
    assert!(ok_out.status.success());
    let init_json = parse_stdout_json(&ok_out);
    assert_eq!(
        init_json["config_path"],
        canonical_str(&repo_path.join(".git-claw.toml"))
    );
    assert_eq!(init_json["main_branch"], "main");
    assert_eq!(init_json["ports"]["APP_PORT"], 4000);
    assert!(init_json["copy_files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == ".env"));
    assert_eq!(init_json["environment_type"], "pyenv");
    assert_eq!(init_json["compose_file"], "docker-compose.yml");
}

#[test]
fn test_shell_hook_cd_with_json_keeps_stdout_valid_json() {
    let (_remote, repo_dir, wt_parent) = setup_git_repo(false);
    let repo_path = repo_dir.path();
    let bin_path = assert_cmd::cargo::cargo_bin("git-claw");
    let bin_dir = bin_path.parent().expect("bin dir");
    let hook_script = git_claw::workflow::generate_shell_hook("bash");

    let script = format!(
        r#"
export PATH="{}:$PATH"
{}
cd "{}"
claw feature start hook-json --cd --json
"#,
        bin_dir.display(),
        hook_script,
        repo_path.display()
    );

    let out = std::process::Command::new("bash")
        .args(["-c", &script])
        .output()
        .expect("run bash shell hook with --cd --json");
    assert!(
        out.status.success(),
        "bash failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json = parse_stdout_json(&out);
    assert_eq!(json["name"], "hook-json");
    assert_eq!(json["slot_id"], 1);
    assert_eq!(
        json["path"],
        canonical_str(&wt_parent.path().join("hook-json"))
    );
}
