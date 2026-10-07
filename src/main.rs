//! git-claw CLI entrypoint.

use clap::CommandFactory;
use git_claw::cli::{self, Cli, Commands, SpikeAction, WorktreeAction};
use git_claw::core::branch::BranchType;
use git_claw::workflow::{self, StartOptions};
use std::process;

fn main() {
    let cli = Cli::parse_from_env();

    match cli.command {
        Some(Commands::Feature {
            action: WorktreeAction::Start { name, isolated },
        }) => {
            if let Err(e) = workflow::start_worktree(StartOptions {
                branch_type: BranchType::Feature,
                name: &name,
                isolated,
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::Bugfix {
            action: WorktreeAction::Start { name, isolated },
        }) => {
            if let Err(e) = workflow::start_worktree(StartOptions {
                branch_type: BranchType::Bugfix,
                name: &name,
                isolated,
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::Hotfix {
            action: WorktreeAction::Start { name, isolated },
        }) => {
            if let Err(e) = workflow::start_worktree(StartOptions {
                branch_type: BranchType::Hotfix,
                name: &name,
                isolated,
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::Spike {
            action: SpikeAction::Start { name, isolated },
        }) => {
            if let Err(e) = workflow::start_worktree(StartOptions {
                branch_type: BranchType::Spike,
                name: &name,
                isolated,
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::Spike {
            action: SpikeAction::Drop { name },
        }) => {
            if let Err(e) = workflow::drop_spike(workflow::DropSpikeOptions {
                name: &name,
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::Finish { name, force }) => {
            if let Err(e) = workflow::finish_worktree(workflow::FinishOptions {
                name: name.as_deref(),
                force,
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::List) => {
            if let Err(e) = workflow::list_worktrees(None) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::Run { name, command }) => {
            match workflow::run_command(workflow::RunOptions {
                name,
                command,
                repo_root: None,
            }) {
                Ok(code) => process::exit(code),
                Err(e) => {
                    cli::output::print_error(e);
                    process::exit(1);
                }
            }
        }
        Some(Commands::Open { name }) => {
            if let Err(e) = workflow::open_worktree(workflow::OpenOptions {
                name: name.as_deref(),
                editor: None,
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::Cd { name }) => {
            if let Err(e) = workflow::cd_worktree(workflow::CdOptions {
                name: name.as_deref(),
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::Tag { version }) => {
            if let Err(e) = workflow::tag_release(workflow::TagOptions {
                version: &version,
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::Init { yes }) => {
            if let Err(e) = workflow::run_init_workflow(workflow::InitOptions {
                yes,
                repo_root: None,
            }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        Some(Commands::ShellHook { shell }) => {
            if let Err(e) = workflow::run_shell_hook(workflow::ShellHookOptions { shell: &shell }) {
                cli::output::print_error(e);
                process::exit(1);
            }
        }
        None => {
            if let Err(e) = Cli::command().print_help() {
                cli::output::print_error(e);
                process::exit(1);
            }
            println!();
        }
    }
}
