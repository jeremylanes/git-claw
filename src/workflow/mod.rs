//! Application Layer (workflow orchestration and command handlers).

pub mod error;
pub mod finish;
pub mod list;
pub mod nav;
pub mod run;
pub mod spike;
pub mod start;
pub mod tag;

pub use error::WorkflowError;
pub use finish::{finish_worktree, FinishOptions};
pub use list::list_worktrees;
pub use nav::{cd_worktree, open_worktree, CdOptions, OpenOptions};
pub use run::{run_command, RunOptions};
pub use spike::{drop_spike, DropSpikeOptions};
pub use start::{start_worktree, StartOptions};
pub use tag::{tag_release, TagOptions};
