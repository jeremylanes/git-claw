use assert_cmd::Command;
use git_claw::workflow::generate_shell_hook;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn test_generate_shell_hook_content() {
    let script = generate_shell_hook("bash");
    assert!(script.contains("claw()"));
    assert!(script.contains("git()"));
    assert!(script.contains("command git-claw cd"));
    assert!(script.contains("cd \"$dest\""));
}

#[test]
fn test_cli_shell_hook_command() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.arg("shell-hook")
        .assert()
        .success()
        .stdout(predicates::str::contains("claw()"))
        .stdout(predicates::str::contains("git()"));
}

fn setup_repo() -> (tempfile::TempDir, tempfile::TempDir) {
    let repo_dir = tempdir().expect("repo dir");
    let wt_parent = tempdir().expect("wt parent dir");

    let repo_path = repo_dir.path();
    let init = StdCommand::new("git")
        .args(["init", "-b", "main"])
        .current_dir(repo_path)
        .output()
        .expect("git init");
    assert!(init.status.success());

    let _ = StdCommand::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(repo_path)
        .output();
    let _ = StdCommand::new("git")
        .args(["config", "user.email", "tester@test.local"])
        .current_dir(repo_path)
        .output();

    fs::write(repo_path.join("README.md"), "# Test Shell Hook").expect("write readme");

    let add = StdCommand::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    assert!(add.status.success());

    let commit = StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");
    assert!(commit.status.success());

    (repo_dir, wt_parent)
}

#[test]
fn test_shell_hook_execution_in_bash() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    // Configure .git-claw.toml to point to custom wt_parent
    let config = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), config).expect("write config");

    // Locate the built git-claw debug binary
    let bin_path = assert_cmd::cargo::cargo_bin("git-claw");
    let bin_dir = bin_path.parent().expect("bin dir");

    // Create a bash script that sources the hook and runs claw feature start
    let hook_script = generate_shell_hook("bash");
    let test_script = format!(
        r#"
export PATH="{}:$PATH"
{}

cd "{}"
claw feature start auto-nav
pwd
"#,
        bin_dir.display(),
        hook_script,
        repo_path.display()
    );

    let output = StdCommand::new("bash")
        .args(["-c", &test_script])
        .output()
        .expect("run bash");

    assert!(
        output.status.success(),
        "bash execution failed: stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_dest = wt_parent.path().join("auto-nav");

    // Verify the bash shell actually navigated (pwd output matches expected_dest)
    assert!(
        stdout.contains(&expected_dest.display().to_string()),
        "expected stdout to contain navigated dir {}, got:\n{}",
        expected_dest.display(),
        stdout
    );
}
