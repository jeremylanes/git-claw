use assert_cmd::Command;
use clap_complete::Shell;
use git_claw::workflow::generate_completion;
use predicates::prelude::*;

#[test]
fn test_generate_completion_fish_unit() {
    let mut buffer = Vec::new();
    generate_completion(Shell::Fish, &mut buffer);
    let output = String::from_utf8(buffer).expect("valid utf-8");
    assert!(output.contains("complete -c git-claw"));
    assert!(output.contains("feature"));
    assert!(output.contains("finish"));
}

#[test]
fn test_generate_completion_bash_unit() {
    let mut buffer = Vec::new();
    generate_completion(Shell::Bash, &mut buffer);
    let output = String::from_utf8(buffer).expect("valid utf-8");
    assert!(output.contains("_git__claw"));
    assert!(output.contains("git-claw"));
}

#[test]
fn test_generate_completion_zsh_unit() {
    let mut buffer = Vec::new();
    generate_completion(Shell::Zsh, &mut buffer);
    let output = String::from_utf8(buffer).expect("valid utf-8");
    assert!(output.contains("compdef"));
}

#[test]
fn test_cli_completion_fish() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["completion", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c git-claw"));
}

#[test]
fn test_cli_completion_bash() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_git__claw"));
}

#[test]
fn test_cli_completion_zsh() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["completion", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("compdef"));
}

#[test]
fn test_cli_git_subcommand_completion() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["claw", "completion", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c git-claw"));
}

#[test]
fn test_cli_completion_alias() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c git-claw"));
}

#[test]
fn test_cli_completion_invalid_shell() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["completion", "unknown_shell"])
        .assert()
        .failure();
}
