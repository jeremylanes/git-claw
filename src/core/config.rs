//! Configuration model and zero-config defaults loader.

use crate::core::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

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

/// Strongly-typed configuration parsed from `.git-claw.toml` with zero-config defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub project: Option<ProjectConfig>,
    #[serde(default)]
    pub ports: BTreeMap<String, u16>,
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub cache: CacheConfig,
}

impl Config {
    /// Constructs default configuration for a given repository name.
    pub fn default_for_repo(repo_name: &str) -> Self {
        Self {
            project: Some(ProjectConfig {
                main_branch: "main".to_string(),
                worktree_root: format!("../{}-worktrees", repo_name),
            }),
            ports: BTreeMap::new(),
            hooks: HooksConfig::default(),
            cache: CacheConfig::default(),
        }
    }

    /// Returns the configured main branch name.
    pub fn main_branch(&self) -> &str {
        self.project
            .as_ref()
            .map(|p| p.main_branch.as_str())
            .unwrap_or("main")
    }

    /// Returns the configured worktree root path, falling back to default for repo if empty.
    pub fn worktree_root(&self, repo_name: &str) -> String {
        self.project
            .as_ref()
            .and_then(|p| {
                if p.worktree_root.trim().is_empty() {
                    None
                } else {
                    Some(p.worktree_root.clone())
                }
            })
            .unwrap_or_else(|| format!("../{}-worktrees", repo_name))
    }

    /// Parses a TOML string into `Config`, filling missing fields with defaults.
    pub fn parse_toml_str(toml_str: &str, repo_name: &str) -> Result<Self, ConfigError> {
        let mut parsed: Config = toml::from_str(toml_str).map_err(|e| ConfigError::ParseError {
            message: e.to_string(),
        })?;

        // Ensure project config has deterministic defaults
        if let Some(ref mut project) = parsed.project {
            if project.worktree_root.trim().is_empty() {
                project.worktree_root = format!("../{}-worktrees", repo_name);
            }
        } else {
            parsed.project = Some(ProjectConfig {
                main_branch: "main".to_string(),
                worktree_root: format!("../{}-worktrees", repo_name),
            });
        }

        Ok(parsed)
    }

    /// Loads configuration from `.git-claw.toml` at the given path, or returns defaults if absent.
    pub fn load_or_default(path: &Path, repo_name: &str) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default_for_repo(repo_name));
        }

        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::IoError {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;

        Self::parse_toml_str(&content, repo_name)
    }
}
