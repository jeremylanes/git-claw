//! git-claw CLI entrypoint.

use clap::CommandFactory;
use git_claw::cli::{self, Cli, Commands, SpikeAction, WorktreeAction};
use git_claw::core::branch::BranchType;
use git_claw::workflow::{self, StartOptions, WorkflowError};
use serde::Serialize;
use std::process;

fn exit_on_error(err: WorkflowError) -> ! {
    cli::output::print_error(err);
    process::exit(1);
}

fn emit_summary<T: Serialize>(result: Result<T, WorkflowError>, json: bool) {
    match result {
        Ok(summary) => {
            if json {
                if let Err(err) = cli::output::print_json(&summary) {
                    cli::output::print_error(err);
                    process::exit(1);
                }
            }
        }
        Err(err) => exit_on_error(err),
    }
}

fn handle_start(branch_type: BranchType, name: String, isolated: bool, json: bool) {
    let result = workflow::start_worktree_summary(
        StartOptions {
            branch_type,
            name: &name,
            isolated,
            repo_root: None,
        },
        json,
    );
    emit_summary(result, json);
}

fn handle_publish(
    branch_type: Option<BranchType>,
    name: Option<String>,
    remote: Option<String>,
    json: bool,
) {
    let result = workflow::publish_worktree_summary(
        workflow::PublishOptions {
            branch_type,
            name: name.as_deref(),
            remote: remote.as_deref(),
            repo_root: None,
        },
        json,
    );
    emit_summary(result, json);
}

fn handle_worktree_action(branch_type: BranchType, action: WorktreeAction) {
    match action {
        WorktreeAction::Start {
            name,
            isolated,
            cd: _,
            json,
        } => handle_start(branch_type, name, isolated, json),
        WorktreeAction::Publish { name, remote, json } => {
            handle_publish(Some(branch_type), name, remote, json)
        }
    }
}

fn main() {
    let cli = Cli::parse_from_env();

    match cli.command {
        Some(Commands::Feature { action }) => handle_worktree_action(BranchType::Feature, action),
        Some(Commands::Bugfix { action }) => handle_worktree_action(BranchType::Bugfix, action),
        Some(Commands::Hotfix { action }) => handle_worktree_action(BranchType::Hotfix, action),
        Some(Commands::Spike {
            action:
                SpikeAction::Start {
                    name,
                    isolated,
                    cd: _,
                    json,
                },
        }) => handle_start(BranchType::Spike, name, isolated, json),
        Some(Commands::Spike {
            action: SpikeAction::Drop { name, json },
        }) => {
            let result = workflow::drop_spike_summary(
                workflow::DropSpikeOptions {
                    name: &name,
                    repo_root: None,
                },
                json,
            );
            emit_summary(result, json);
        }
        Some(Commands::Publish { name, remote, json }) => handle_publish(None, name, remote, json),
        Some(Commands::Finish {
            name,
            force,
            keep_remote,
            json,
        }) => {
            let result = workflow::finish_worktree_summary(
                workflow::FinishOptions {
                    name: name.as_deref(),
                    force,
                    keep_remote,
                    repo_root: None,
                },
                json,
            );
            emit_summary(result, json);
        }
        Some(Commands::List { json }) => {
            if let Err(err) = workflow::list_worktrees_with_output(None, json) {
                exit_on_error(err);
            }
        }
        Some(Commands::Run { name, command }) => {
            match workflow::run_command(workflow::RunOptions {
                name,
                command,
                repo_root: None,
            }) {
                Ok(code) => process::exit(code),
                Err(err) => exit_on_error(err),
            }
        }
        Some(Commands::Open { name }) => {
            if let Err(err) = workflow::open_worktree(workflow::OpenOptions {
                name: name.as_deref(),
                editor: None,
                repo_root: None,
            }) {
                exit_on_error(err);
            }
        }
        Some(Commands::Cd { name }) => {
            if let Err(err) = workflow::cd_worktree(workflow::CdOptions {
                name: name.as_deref(),
                repo_root: None,
            }) {
                exit_on_error(err);
            }
        }
        Some(Commands::Tag { version }) => {
            if let Err(err) = workflow::tag_release(workflow::TagOptions {
                version: version.as_deref(),
                repo_root: None,
            }) {
                exit_on_error(err);
            }
        }
        Some(Commands::Init { yes, json }) => {
            let result = workflow::run_init_workflow_summary(
                workflow::InitOptions {
                    yes,
                    repo_root: None,
                },
                json,
            );
            emit_summary(result, json);
        }
        Some(Commands::ShellHook { shell }) => {
            if let Err(err) = workflow::run_shell_hook(workflow::ShellHookOptions { shell: &shell })
            {
                exit_on_error(err);
            }
        }
        Some(Commands::Completion { shell }) => {
            if let Err(err) = workflow::run_completion_workflow(shell) {
                exit_on_error(err);
            }
        }
        None => {
            if let Err(err) = Cli::command().print_help() {
                cli::output::print_error(err);
                process::exit(1);
            }
            println!();
        }
    }
}
