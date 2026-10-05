//! Clap CLI argument definitions and Git subcommand normalization.

use clap::{Parser, Subcommand};
use std::ffi::OsString;

/// Deterministic git worktree orchestration engine for developers and AI agents.
#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(
    name = "git-claw",
    bin_name = "git-claw",
    version,
    about = "Deterministic git worktree orchestration engine for developers and AI agents"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Commands {
    /// Manage feature worktrees
    Feature {
        #[command(subcommand)]
        action: WorktreeAction,
    },
    /// Manage bugfix worktrees
    Bugfix {
        #[command(subcommand)]
        action: WorktreeAction,
    },
    /// Manage hotfix worktrees
    Hotfix {
        #[command(subcommand)]
        action: WorktreeAction,
    },
    /// Manage spike worktrees
    Spike {
        #[command(subcommand)]
        action: SpikeAction,
    },
    /// Complete a worktree, run tests, merge into main, and clean up
    Finish {
        /// Worktree name (inferred from cwd if omitted)
        name: Option<String>,
        /// Force completion even if pre-finish hooks fail
        #[arg(long)]
        force: bool,
    },
    /// List all active worktree slots
    List,
    /// Run an arbitrary command inside a worktree with environment injected
    Run {
        /// Worktree name (inferred from cwd if omitted)
        name: Option<String>,
        /// Shell command to execute
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
    /// Open the worktree in configured editor ($EDITOR or code/cursor)
    Open {
        /// Worktree name (inferred from cwd if omitted)
        name: Option<String>,
    },
    /// Print the worktree path (usable with cd $(git claw cd <name>))
    Cd {
        /// Worktree name (inferred from cwd if omitted)
        name: Option<String>,
    },
    /// Create an annotated SemVer release tag on main
    Tag {
        /// SemVer version tag (e.g., v1.2.0)
        version: String,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum WorktreeAction {
    /// Create and start a new worktree
    Start {
        /// Name of the branch/worktree (kebab-case)
        name: String,
        /// Skip shared cache directory symlinking
        #[arg(long)]
        isolated: bool,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum SpikeAction {
    /// Create and start a new spike worktree
    Start {
        /// Name of the spike branch/worktree (kebab-case)
        name: String,
        /// Skip shared cache directory symlinking
        #[arg(long)]
        isolated: bool,
    },
    /// Drop a spike worktree without merging
    Drop {
        /// Name of the spike branch/worktree
        name: String,
    },
}

/// Normalizes CLI arguments to transparently support both direct `git-claw`
/// and git subcommand `git claw` (or `git-claw claw`) execution.
pub fn sanitize_args<I, T>(args: I) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    T: AsRef<std::ffi::OsStr> + Clone,
{
    let mut vec: Vec<T> = args.into_iter().collect();
    if vec.len() > 1 && vec[1].as_ref() == "claw" {
        vec.remove(1);
    }
    vec
}

impl Cli {
    /// Parse arguments from process environment after normalizing leading 'claw' subcommand.
    pub fn parse_from_env() -> Self {
        let args = sanitize_args(std::env::args_os());
        Self::parse_from(args)
    }

    /// Try parsing arguments from an iterator of OsString.
    #[allow(dead_code)]
    pub fn try_parse_from_args<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let raw: Vec<OsString> = args.into_iter().map(Into::into).collect();
        let sanitized = sanitize_args(raw);
        Self::try_parse_from(sanitized)
    }
}
