//! Presentation Layer (CLI parsing and terminal output).

pub mod args;
pub mod output;

#[allow(unused_imports)]
pub use args::{sanitize_args, Cli, Commands, SpikeAction, WorktreeAction};
