//! Configuration model and zero-config defaults loader.

use crate::core::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Expands leading '~/' to the user's home directory.
pub fn expand_home(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}/{}", home.trim_end_matches('/'), stripped);
        }
    }
    path.to_string()
}

/// Returns the default centralized worktree root: `~/.git-claw/worktrees/<repo_name>`.
pub fn default_worktree_root(repo_name: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        format!(
            "{}/.git-claw/worktrees/{}",
            home.trim_end_matches('/'),
            repo_name
        )
    } else {
        format!("~/.git-claw/worktrees/{}", repo_name)
    }
}

/// Resolves the global user configuration path.
/// Can be overridden with `GIT_CLAW_GLOBAL_CONFIG` for testing.
pub fn global_config_path() -> Option<PathBuf> {
    std::env::var("GIT_CLAW_GLOBAL_CONFIG")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".git-claw/config.toml"))
        })
}

/// Cache sharing strategy for worktree dependencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CacheStrategy {
    #[default]
    Shared,
    Isolated,
}

/// Project-level configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_main_branch")]
    pub main_branch: String,
    #[serde(default)]
    pub worktree_root: String,
}

fn default_main_branch() -> String {
    "main".to_string()
}

/// Lifecycle hooks.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HooksConfig {
    pub post_start: Option<String>,
    pub pre_finish: Option<String>,
    pub post_finish: Option<String>,
}

/// Cache sharing configuration.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub strategy: CacheStrategy,
    #[serde(default)]
    pub directories: Vec<String>,
}

/// Untracked files duplication configuration.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesConfig {
    #[serde(default)]
    pub copy: Vec<String>,
}

/// Docker Compose override configuration.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerConfig {
    #[serde(default)]
    pub compose_file: Option<String>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub shared_services: Vec<String>,
}

impl DockerConfig {
    /// Returns true if any Docker configuration option is set.
    pub fn is_enabled(&self) -> bool {
        self.compose_file.is_some() || self.network.is_some() || !self.shared_services.is_empty()
    }
}

/// Strongly-typed configuration parsed from `.git-claw.toml` or `~/.git-claw/config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub project: Option<ProjectConfig>,
    #[serde(default)]
    pub ports: BTreeMap<String, u16>,
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub files: FilesConfig,
    #[serde(default)]
    pub docker: DockerConfig,
}

impl Config {
    /// Constructs default configuration for a given repository name.
    pub fn default_for_repo(repo_name: &str) -> Self {
        Self {
            project: Some(ProjectConfig {
                main_branch: "main".to_string(),
                worktree_root: default_worktree_root(repo_name),
            }),
            ports: BTreeMap::new(),
            hooks: HooksConfig::default(),
            cache: CacheConfig::default(),
            files: FilesConfig::default(),
            docker: DockerConfig::default(),
        }
    }

    /// Returns the configured main branch name.
    pub fn main_branch(&self) -> &str {
        self.project
            .as_ref()
            .map(|p| p.main_branch.as_str())
            .unwrap_or("main")
    }

    /// Returns the configured worktree root path, falling back to centralized default if empty.
    pub fn worktree_root(&self, repo_name: &str) -> String {
        let raw = self
            .project
            .as_ref()
            .and_then(|p| {
                if p.worktree_root.trim().is_empty() {
                    None
                } else {
                    Some(p.worktree_root.clone())
                }
            })
            .unwrap_or_else(|| default_worktree_root(repo_name));
        expand_home(&raw)
    }

    /// Merges other config on top of self, where non-default fields in other take precedence.
    pub fn merge_with(&mut self, other: Config) {
        if let Some(other_proj) = other.project {
            if let Some(ref mut self_proj) = self.project {
                if !other_proj.main_branch.trim().is_empty() {
                    self_proj.main_branch = other_proj.main_branch;
                }
                if !other_proj.worktree_root.trim().is_empty() {
                    self_proj.worktree_root = other_proj.worktree_root;
                }
            } else {
                self.project = Some(other_proj);
            }
        }

        for (k, v) in other.ports {
            self.ports.insert(k, v);
        }

        if other.hooks.post_start.is_some() {
            self.hooks.post_start = other.hooks.post_start;
        }
        if other.hooks.pre_finish.is_some() {
            self.hooks.pre_finish = other.hooks.pre_finish;
        }
        if other.hooks.post_finish.is_some() {
            self.hooks.post_finish = other.hooks.post_finish;
        }

        if other.cache.strategy != CacheStrategy::default() {
            self.cache.strategy = other.cache.strategy;
        }
        if !other.cache.directories.is_empty() {
            self.cache.directories = other.cache.directories;
        }

        if !other.files.copy.is_empty() {
            self.files.copy = other.files.copy;
        }

        if other.docker.compose_file.is_some() {
            self.docker.compose_file = other.docker.compose_file;
        }
        if other.docker.network.is_some() {
            self.docker.network = other.docker.network;
        }
        if !other.docker.shared_services.is_empty() {
            self.docker.shared_services = other.docker.shared_services;
        }
    }

    /// Parses a TOML string into `Config`, filling missing fields with defaults.
    pub fn parse_toml_str(toml_str: &str, repo_name: &str) -> Result<Self, ConfigError> {
        let mut parsed: Config = toml::from_str(toml_str).map_err(|e| ConfigError::ParseError {
            message: e.to_string(),
        })?;

        // Ensure project config has deterministic defaults
        if let Some(ref mut project) = parsed.project {
            if project.worktree_root.trim().is_empty() {
                project.worktree_root = default_worktree_root(repo_name);
            }
        } else {
            parsed.project = Some(ProjectConfig {
                main_branch: "main".to_string(),
                worktree_root: default_worktree_root(repo_name),
            });
        }

        Ok(parsed)
    }

    /// Loads configuration respecting the two-tier hierarchy:
    /// 1. Hardcoded system defaults.
    /// 2. Merged with global user config (`~/.git-claw/config.toml` or `GIT_CLAW_GLOBAL_CONFIG`).
    /// 3. Merged with local repository config (`path`).
    pub fn load_or_default(path: &Path, repo_name: &str) -> Result<Self, ConfigError> {
        let mut base = Self::default_for_repo(repo_name);

        // 1. Try loading global configuration
        if let Some(global_path) = global_config_path() {
            if global_path.exists() {
                let global_content =
                    std::fs::read_to_string(&global_path).map_err(|e| ConfigError::IoError {
                        path: global_path.to_string_lossy().to_string(),
                        source: e,
                    })?;
                let global_config = Self::parse_toml_str(&global_content, repo_name)?;
                base.merge_with(global_config);
            }
        }

        // 2. Try loading local repository configuration
        if path.exists() {
            let local_content =
                std::fs::read_to_string(path).map_err(|e| ConfigError::IoError {
                    path: path.to_string_lossy().to_string(),
                    source: e,
                })?;
            let local_config = Self::parse_toml_str(&local_content, repo_name)?;
            base.merge_with(local_config);
        }

        Ok(base)
    }
}
