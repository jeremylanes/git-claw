//! Application Layer (workflow orchestration and command handlers).

pub mod completion;
pub mod error;
pub mod finish;
pub mod init;
pub mod list;
pub mod nav;
pub mod publish;
pub mod run;
pub mod shell;
pub mod spike;
pub mod start;
pub mod tag;

pub use completion::{generate_completion, run_completion_workflow};
pub use error::WorkflowError;
pub use finish::{finish_worktree, FinishOptions};
pub use init::{run_init_workflow, InitOptions};
pub use list::list_worktrees;
pub use nav::{cd_worktree, open_worktree, CdOptions, OpenOptions};
pub use publish::{publish_worktree, resolve_active_worktree_slot, PublishOptions};
pub use run::{run_command, RunOptions};
pub use shell::{generate_shell_hook, run_shell_hook, ShellHookOptions};
pub use spike::{drop_spike, DropSpikeOptions};
pub use start::{start_worktree, StartOptions};
pub use tag::{tag_release, update_cargo_toml_version, TagOptions};
