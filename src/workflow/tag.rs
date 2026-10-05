//! SemVer release tag manager.

use crate::cli::output::print_success;
use crate::core::config::Config;
use crate::core::semver::validate_semver;
use crate::infra::git::{
    git_create_annotated_tag, git_is_clean, resolve_repo_name, resolve_toplevel,
};
use crate::workflow::error::WorkflowError;
use std::path::Path;

pub struct TagOptions<'a> {
    pub version: &'a str,
    pub repo_root: Option<&'a Path>,
}

/// Validates SemVer, checks clean trunk, and creates an annotated Git release tag.
pub fn tag_release(options: TagOptions<'_>) -> Result<(), WorkflowError> {
    if !validate_semver(options.version) {
        return Err(WorkflowError::InvalidSemVer(options.version.to_string()));
    }

    let toplevel = resolve_toplevel(options.repo_root)?;
    let repo_name = resolve_repo_name(&toplevel);

    let config_path = toplevel.join(".git-claw.toml");
    let config = Config::load_or_default(&config_path, &repo_name)?;

    if !git_is_clean(&toplevel)? {
        return Err(WorkflowError::DirtyWorkingTree);
    }

    let message = format!("Release {}", options.version);
    git_create_annotated_tag(&toplevel, options.version, config.main_branch(), &message)?;

    print_success(format!(
        "Created annotated release tag '{}' on branch '{}'",
        options.version,
        config.main_branch()
    ));

    Ok(())
}
