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
    assert!(script.contains("should_cd"));
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
    let config = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), config).expect("write config");

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
    let canonical_root = repo_path.canonicalize().expect("canonicalize");

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
echo "AFTER_NO_CD: $(pwd)"
claw feature start auto-nav-cd --cd
echo "AFTER_CD: $(pwd)"
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
    let expected_dest = wt_parent.path().join("auto-nav-cd");

    // Verify without --cd shell remains in repo_path, and with --cd navigates to expected_dest
    assert!(
        stdout.contains(&format!("AFTER_NO_CD: {}", canonical_root.display())),
        "expected stdout without --cd to remain in repo dir {}, got:\n{}",
        canonical_root.display(),
        stdout
    );
    assert!(
        stdout.contains(&format!("AFTER_CD: {}", expected_dest.display())),
        "expected stdout with --cd to contain navigated dir {}, got:\n{}",
        expected_dest.display(),
        stdout
    );
}

#[test]
fn test_generate_shell_hook_fish_content() {
    let script = generate_shell_hook("fish");
    assert!(script.contains("function claw"));
    assert!(script.contains("function git"));
    assert!(script.contains("command git-claw cd"));
    assert!(script.contains("test \"$argv[2]\" = \"start\""));
    assert!(script.contains("test \"$argv[1]\" = \"finish\""));
    assert!(script.contains("test \"$argv[2]\" = \"drop\""));
    assert!(script.contains("main_dest"));
    assert!(script.contains("not test -d \"$PWD\""));
    assert!(script.contains("should_cd"));
}

#[test]
fn test_generate_shell_hook_bash_auto_cd_finish() {
    let script = generate_shell_hook("bash");
    assert!(script.contains("claw()"));
    assert!(script.contains("git()"));
    assert!(script.contains("[ \"$1\" = \"finish\" ]"));
    assert!(script.contains("[ \"$2\" = \"drop\" ]"));
    assert!(script.contains(
        "main_dest=\"$(command git-claw cd main 2>/dev/null || command git-claw cd 2>/dev/null)\""
    ));
    assert!(script.contains("[ ! -d \"$PWD\" ]"));
}

#[test]
fn test_cli_shell_hook_fish_command() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["shell-hook", "fish"])
        .assert()
        .success()
        .stdout(predicates::str::contains("function claw"))
        .stdout(predicates::str::contains("function git"));
}

#[test]
fn test_shell_hook_finish_auto_cd_in_bash() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();
    let canonical_root = repo_path.canonicalize().expect("canonicalize");

    let bin_path = assert_cmd::cargo::cargo_bin("git-claw");
    let bin_dir = bin_path.parent().expect("bin dir");

    let hook_script = generate_shell_hook("bash");
    let test_script = format!(
        r#"
export PATH="{}:$PATH"
{}

cd "{}"
claw feature start finish-nav --cd
worktree_dir="$(pwd)"
echo "WORKTREE_DIR: $worktree_dir"
claw finish
echo "FINAL_DIR: $(pwd)"
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
    let expected_wt = wt_parent.path().join("finish-nav");

    assert!(
        stdout.contains(&format!("WORKTREE_DIR: {}", expected_wt.display())),
        "expected stdout to contain worktree dir {}, got:\n{}",
        expected_wt.display(),
        stdout
    );

    assert!(
        stdout.contains(&format!("FINAL_DIR: {}", canonical_root.display())),
        "expected stdout to contain final dir {}, got:\n{}",
        canonical_root.display(),
        stdout
    );
}

#[test]
fn test_shell_hook_finish_auto_cd_in_fish() {
    if StdCommand::new("fish").arg("--version").output().is_err() {
        return;
    }

    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();
    let canonical_root = repo_path.canonicalize().expect("canonicalize");

    let bin_path = assert_cmd::cargo::cargo_bin("git-claw");
    let bin_dir = bin_path.parent().expect("bin dir");

    let hook_script = generate_shell_hook("fish");
    let test_script = format!(
        r#"
set -gx PATH "{}" $PATH
{}

cd "{}"
claw feature start fish-nav --cd
echo "WORKTREE_DIR: " (pwd)
claw finish
echo "FINAL_DIR: " (pwd)
"#,
        bin_dir.display(),
        hook_script,
        repo_path.display()
    );

    let output = StdCommand::new("fish")
        .args(["-c", &test_script])
        .output()
        .expect("run fish");

    assert!(
        output.status.success(),
        "fish execution failed: stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_wt = wt_parent.path().join("fish-nav");

    assert!(
        stdout.contains(&format!("WORKTREE_DIR:  {}", expected_wt.display())),
        "expected stdout to contain worktree dir {}, got:\n{}",
        expected_wt.display(),
        stdout
    );

    assert!(
        stdout.contains(&format!("FINAL_DIR:  {}", canonical_root.display())),
        "expected stdout to contain final dir {}, got:\n{}",
        canonical_root.display(),
        stdout
    );
}

#[test]
fn test_shell_hook_git_claw_finish_with_name_in_fish() {
    if StdCommand::new("fish").arg("--version").output().is_err() {
        return;
    }

    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();
    let canonical_root = repo_path.canonicalize().expect("canonicalize");

    let bin_path = assert_cmd::cargo::cargo_bin("git-claw");
    let bin_dir = bin_path.parent().expect("bin dir");

    let hook_script = generate_shell_hook("fish");
    let test_script = format!(
        r#"
set -gx PATH "{}" $PATH
{}

cd "{}"
git claw feature start fish-name-nav --cd
echo "WORKTREE_DIR: " (pwd)
git claw finish fish-name-nav
echo "FINAL_DIR: " (pwd)
"#,
        bin_dir.display(),
        hook_script,
        repo_path.display()
    );

    let output = StdCommand::new("fish")
        .args(["-c", &test_script])
        .output()
        .expect("run fish");

    assert!(
        output.status.success(),
        "fish execution failed: stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_wt = wt_parent.path().join("fish-name-nav");

    assert!(
        stdout.contains(&format!("WORKTREE_DIR:  {}", expected_wt.display())),
        "expected stdout to contain worktree dir {}, got:\n{}",
        expected_wt.display(),
        stdout
    );

    assert!(
        stdout.contains(&format!("FINAL_DIR:  {}", canonical_root.display())),
        "expected stdout to contain final dir {}, got:\n{}",
        canonical_root.display(),
        stdout
    );
}

#[test]
fn test_shell_hook_execution_in_fish() {
    if StdCommand::new("fish").arg("--version").output().is_err() {
        return;
    }

    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();
    let canonical_root = repo_path.canonicalize().expect("canonicalize");

    let bin_path = assert_cmd::cargo::cargo_bin("git-claw");
    let bin_dir = bin_path.parent().expect("bin dir");

    let hook_script = generate_shell_hook("fish");
    let test_script = format!(
        r#"
set -gx PATH "{}" $PATH
{}

cd "{}"
claw feature start fish-no-cd
echo "AFTER_NO_CD: " (pwd)
claw feature start fish-with-cd --cd
echo "AFTER_CD: " (pwd)
"#,
        bin_dir.display(),
        hook_script,
        repo_path.display()
    );

    let output = StdCommand::new("fish")
        .args(["-c", &test_script])
        .output()
        .expect("run fish");

    assert!(
        output.status.success(),
        "fish execution failed: stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_dest = wt_parent.path().join("fish-with-cd");

    assert!(
        stdout.contains(&format!("AFTER_NO_CD:  {}", canonical_root.display())),
        "expected fish without --cd to remain in repo dir {}, got:\n{}",
        canonical_root.display(),
        stdout
    );
    assert!(
        stdout.contains(&format!("AFTER_CD:  {}", expected_dest.display())),
        "expected fish with --cd to contain navigated dir {}, got:\n{}",
        expected_dest.display(),
        stdout
    );
}
