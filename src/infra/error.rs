//! Infrastructure error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Not a git repository (or any of the parent directories)")]
    NotAGitRepository,

    #[error("Git command failed ({command}): {message}")]
    CommandFailed { command: String, message: String },

    #[error("Failed to execute git process: {0}")]
    ProcessFailed(#[from] std::io::Error),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LockError {
    #[error("Timed out waiting to acquire advisory lock at '{path}' after {timeout_secs}s")]
    Timeout { path: String, timeout_secs: u64 },

    #[error("Failed to open or create lock file at '{path}': {message}")]
    IoError { path: String, message: String },
}

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Failed to read registry at '{path}': {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse registry JSON at '{path}': {source}")]
    JsonError {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}

#[derive(Error, Debug)]
pub enum WorktreeError {
    #[error("Failed to create worktree at '{path}': {message}")]
    CreationFailed { path: String, message: String },

    #[error("Failed to remove worktree at '{path}': {message}")]
    RemovalFailed { path: String, message: String },

    #[error("Branch already exists: '{0}'")]
    BranchAlreadyExists(String),

    #[error("Worktree directory already exists: '{0}'")]
    DirectoryAlreadyExists(String),

    #[error("I/O error during worktree operation: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum HookError {
    #[error("Hook execution failed: {0}")]
    ExecutionFailed(String),

    #[error(
        "Pre-finish hook '{hook}' failed with exit code {code}; aborting (use --force to override)"
    )]
    PreFinishFailed { hook: String, code: i32 },
}
