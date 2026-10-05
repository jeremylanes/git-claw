//! Domain error types.

use thiserror::Error;

/// Configuration errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file at '{path}': {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse .git-claw.toml: {message}")]
    ParseError { message: String },
}

/// Port calculation errors.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum PortError {
    #[error("Port calculation overflow for key '{key}': base port {base_port} + slot {slot_id} exceeds 65535")]
    PortOverflow {
        key: String,
        base_port: u16,
        slot_id: u32,
    },
}

/// Branch type and naming errors.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum BranchError {
    #[error("Invalid branch type '{0}'; expected feature, bugfix, hotfix, or spike")]
    InvalidBranchType(String),

    #[error("Worktree name cannot be empty")]
    EmptyName,

    #[error("Invalid worktree name '{0}'; must be lowercase kebab-case (e.g. 'my-feature-1')")]
    InvalidKebabCase(String),
}
