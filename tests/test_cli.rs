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
        .stdout(predicate::str::contains(format!(
            "git-claw {}",
            env!("CARGO_PKG_VERSION")
        )));
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
        .stdout(predicate::str::contains("<NAME>"))
        .stdout(predicate::str::contains("--cd"));
}

#[test]
fn test_spike_start_help() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["spike", "start", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Create and start a new spike worktree",
        ))
        .stdout(predicate::str::contains("<NAME>"))
        .stdout(predicate::str::contains("--cd"));
}

#[test]
fn test_list_command_stub() {
    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.arg("list");
    cmd.assert().success();
}

#[test]
fn test_help_does_not_leak_doctests() {
    use clap::CommandFactory;
    use git_claw::cli::Cli;

    fn assert_clean_help(cmd: &mut clap::Command, path: &str) {
        let help = cmd.render_long_help().to_string();
        for forbidden in ["# Examples", "try_parse_from_args", "```"] {
            assert!(
                !help.contains(forbidden),
                "Help output for `{path}` leaked doctest token `{forbidden}`:\n{help}"
            );
        }
        let sub_names: Vec<String> = cmd
            .get_subcommands()
            .map(|s| s.get_name().to_string())
            .collect();
        for name in sub_names {
            if let Some(sub) = cmd.find_subcommand_mut(&name) {
                let child_path = format!("{path} {name}");
                assert_clean_help(sub, &child_path);
            }
        }
    }

    let mut root = Cli::command();
    assert_clean_help(&mut root, "git-claw");

    let mut bin_cmd = Command::cargo_bin("git-claw").expect("binary exists");
    bin_cmd.arg("--help");
    bin_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("# Examples").not())
        .stdout(predicate::str::contains("try_parse_from_args").not())
        .stdout(predicate::str::contains("```").not());
}
