//! Filesystem utilities and shared cache directory symlinking.

use crate::core::config::{CacheConfig, CacheStrategy};
use std::fs;
use std::io;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(windows)]
use std::os::windows::fs::symlink_dir as symlink;

/// Links configured cache directories from the primary repository into the worktree
/// using symlinks, unless `isolated` is specified or strategy is isolated.
/// Returns the number of symlinks successfully created.
pub fn link_shared_cache_directories(
    repo_root: &Path,
    worktree_path: &Path,
    cache_config: &CacheConfig,
    isolated: bool,
) -> io::Result<usize> {
    if isolated || cache_config.strategy == CacheStrategy::Isolated {
        return Ok(0);
    }

    let mut linked_count = 0;
    for dir_rel in &cache_config.directories {
        let dir_rel = dir_rel.trim();
        if dir_rel.is_empty() {
            continue;
        }

        let primary_dir = repo_root.join(dir_rel);
        if !primary_dir.exists() {
            fs::create_dir_all(&primary_dir)?;
        }

        let worktree_target = worktree_path.join(dir_rel);
        if worktree_target.exists() || worktree_target.is_symlink() {
            let _ = fs::remove_file(&worktree_target).or_else(|_| fs::remove_dir(&worktree_target));
        }

        if let Some(parent) = worktree_target.parent() {
            fs::create_dir_all(parent)?;
        }

        symlink(&primary_dir, &worktree_target)?;
        linked_count += 1;
    }

    Ok(linked_count)
}
