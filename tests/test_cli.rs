use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Deterministic git worktree orchestration engine",
        ))
        .stdout(predicate::str::contains("feature"))
        .stdout(predicate::str::contains("finish"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("git-claw 0.1.0"));
}

#[test]
fn test_git_subcommand_transparent_claw_prefix() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["claw", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Deterministic git worktree orchestration engine",
        ))
        .stdout(predicate::str::contains("feature"));
}

#[test]
fn test_invalid_subcommand() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.arg("invalid-subcommand");
    cmd.assert().failure().code(2);
}

#[test]
fn test_feature_start_help() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["feature", "start", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create and start a new worktree"))
        .stdout(predicate::str::contains("<NAME>"));
}

#[test]
fn test_list_command_stub() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.arg("list");
    cmd.assert().success();
}
