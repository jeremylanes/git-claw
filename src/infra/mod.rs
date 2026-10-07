//! Infrastructure Layer (side-effects, external process execution, filesystem I/O).

pub mod docker;
pub mod env_file;
pub mod error;
pub mod fs;
pub mod git;
pub mod hook;
pub mod lock;
pub mod registry;
pub mod worktree;

pub use docker::generate_docker_compose_override;
pub use env_file::{sanitize_compose_project_name, write_env_worktree};
pub use error::{GitError, HookError, LockError, RegistryError, WorktreeError};
pub use fs::link_shared_cache_directories;
pub use git::{
    git_checkout, git_is_clean, git_merge, git_worktree_prune, resolve_git_common_dir,
    resolve_repo_name, resolve_toplevel,
};
pub use hook::{
    execute_hook_command, run_post_finish_hook, run_post_start_hook, run_pre_finish_hook,
};
pub use lock::SlotLock;
pub use registry::{SlotRecord, SlotRegistry};
pub use worktree::{create_worktree, delete_branch, remove_worktree};
