//! Slot registry persistence in `<git-common-dir>/claw/slots.json` and auto-GC.

use crate::infra::error::RegistryError;
use crate::infra::git::git_worktree_prune;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Single allocated slot record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotRecord {
    pub id: u32,
    pub name: String,
    pub branch_type: String,
    pub branch: String,
    pub path: String,
    pub allocated_at: String,
}

/// Root JSON object of `slots.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotRegistry {
    pub version: u32,
    pub slots: Vec<SlotRecord>,
}

impl SlotRegistry {
    pub const CURRENT_VERSION: u32 = 1;
    pub const CREATION_GRACE_PERIOD: Duration = Duration::from_secs(30);

    /// Creates an empty registry with version 1.
    pub fn empty() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            slots: Vec::new(),
        }
    }

    /// Loads registry from path, or returns an empty registry if the file does not exist.
    pub fn load_or_empty(path: &Path) -> Result<Self, RegistryError> {
        if !path.exists() {
            return Ok(Self::empty());
        }

        let content = fs::read_to_string(path).map_err(|e| RegistryError::IoError {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;

        let registry: SlotRegistry =
            serde_json::from_str(&content).map_err(|e| RegistryError::JsonError {
                path: path.to_string_lossy().to_string(),
                source: e,
            })?;

        Ok(registry)
    }

    /// Saves the registry atomically to disk using a temporary file and rename.
    pub fn save_atomic(&self, path: &Path) -> Result<(), RegistryError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| RegistryError::IoError {
                path: parent.to_string_lossy().to_string(),
                source: e,
            })?;
        }

        let tmp_path = format!("{}.tmp.{}", path.display(), std::process::id());
        let tmp_path = PathBuf::from(tmp_path);

        let json = serde_json::to_string_pretty(self).map_err(|e| RegistryError::JsonError {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;

        let mut file = File::create(&tmp_path).map_err(|e| RegistryError::IoError {
            path: tmp_path.to_string_lossy().to_string(),
            source: e,
        })?;

        file.write_all(json.as_bytes())
            .map_err(|e| RegistryError::IoError {
                path: tmp_path.to_string_lossy().to_string(),
                source: e,
            })?;

        file.sync_all().map_err(|e| RegistryError::IoError {
            path: tmp_path.to_string_lossy().to_string(),
            source: e,
        })?;

        fs::rename(&tmp_path, path).map_err(|e| RegistryError::IoError {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;

        Ok(())
    }

    /// Finds a slot by ID.
    pub fn find_by_id(&self, id: u32) -> Option<&SlotRecord> {
        self.slots.iter().find(|s| s.id == id)
    }

    /// Finds a slot by leaf name.
    pub fn find_by_name(&self, name: &str) -> Option<&SlotRecord> {
        self.slots.iter().find(|s| s.name == name)
    }

    /// Returns all currently occupied Slot IDs.
    pub fn occupied_ids(&self) -> Vec<u32> {
        self.slots.iter().map(|s| s.id).collect()
    }

    /// Removes a slot by leaf name, returning true if removed.
    pub fn remove_by_name(&mut self, name: &str) -> bool {
        let initial_len = self.slots.len();
        self.slots.retain(|s| s.name != name);
        self.slots.len() < initial_len
    }

    /// Removes a slot by ID, returning true if removed.
    pub fn remove_by_id(&mut self, id: u32) -> bool {
        let initial_len = self.slots.len();
        self.slots.retain(|s| s.id != id);
        self.slots.len() < initial_len
    }

    /// Auto-GC: inspects recorded worktrees, removes entries whose directory does not exist on disk
    /// (ignoring records within the grace period), saves the registry atomically, and invokes git worktree prune.
    pub fn auto_gc(
        &mut self,
        registry_path: &Path,
        repo_root: Option<&Path>,
        respect_grace_period: bool,
    ) -> Result<usize, RegistryError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let initial_count = self.slots.len();
        self.slots.retain(|slot| {
            let path = Path::new(&slot.path);
            if path.exists() {
                return true;
            }

            if respect_grace_period {
                if let Ok(ts) = slot.allocated_at.parse::<u64>() {
                    if now.saturating_sub(ts) < Self::CREATION_GRACE_PERIOD.as_secs() {
                        return true;
                    }
                }
            }

            false
        });

        let purged = initial_count - self.slots.len();
        if purged > 0 {
            self.save_atomic(registry_path)?;
            let _ = git_worktree_prune(repo_root);
        }

        Ok(purged)
    }
}
