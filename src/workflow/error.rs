//! Workflow error types.

use crate::core::error::{BranchError, ConfigError, PortError};
use crate::infra::error::{GitError, HookError, LockError, RegistryError, WorktreeError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Slot name '{0}' is already active in this repository")]
    DuplicateSlotName(String),

    #[error("Worktree '{0}' not found in active slots registry")]
    WorktreeNotFound(String),

    #[error("Cannot infer active worktree from current directory '{0}'")]
    CannotInferWorktree(String),

    #[error("Primary repository working tree has uncommitted modifications; cannot merge safely")]
    DirtyWorkingTree,

    #[error("Branch '{0}' already exists in this repository")]
    BranchExists(String),

    #[error("Command execution failed: {0}")]
    CommandExecutionFailed(String),

    #[error("No command specified to execute")]
    NoCommandSpecified,

    #[error("Invalid SemVer version '{0}': expected format like 'v1.2.0' or '1.2.0'")]
    InvalidSemVer(String),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Port(#[from] PortError),

    #[error(transparent)]
    Branch(#[from] BranchError),

    #[error(transparent)]
    Git(#[from] GitError),

    #[error(transparent)]
    Lock(#[from] LockError),

    #[error(transparent)]
    Registry(#[from] RegistryError),

    #[error(transparent)]
    Worktree(#[from] WorktreeError),

    #[error(transparent)]
    Hook(#[from] HookError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
