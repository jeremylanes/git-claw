//! Branch type definitions and kebab-case name validation.

use crate::core::error::BranchError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Allowed worktree branch types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BranchType {
    Feature,
    Bugfix,
    Hotfix,
    Spike,
}

impl BranchType {
    /// Returns the branch type prefix string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Feature => "feature",
            Self::Bugfix => "bugfix",
            Self::Hotfix => "hotfix",
            Self::Spike => "spike",
        }
    }

    /// Formats the full Git branch name: `<type>/<name>`.
    pub fn format_branch_name(&self, name: &str) -> Result<String, BranchError> {
        validate_leaf_name(name)?;
        Ok(format!("{}/{}", self.as_str(), name))
    }
}

impl fmt::Display for BranchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for BranchType {
    type Err = BranchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "feature" => Ok(Self::Feature),
            "bugfix" => Ok(Self::Bugfix),
            "hotfix" => Ok(Self::Hotfix),
            "spike" => Ok(Self::Spike),
            _ => Err(BranchError::InvalidBranchType(s.to_string())),
        }
    }
}

/// Validates that a leaf worktree name conforms to kebab-case convention.
pub fn validate_leaf_name(name: &str) -> Result<(), BranchError> {
    if name.is_empty() {
        return Err(BranchError::EmptyName);
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(BranchError::InvalidKebabCase(name.to_string()));
    }

    let is_valid = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    if !is_valid {
        return Err(BranchError::InvalidKebabCase(name.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_leaf_names() {
        assert!(validate_leaf_name("auth").is_ok());
        assert!(validate_leaf_name("user-profile-v2").is_ok());
        assert!(validate_leaf_name("fix123").is_ok());
    }

    #[test]
    fn test_invalid_leaf_names() {
        assert!(matches!(
            validate_leaf_name(""),
            Err(BranchError::EmptyName)
        ));
        assert!(matches!(
            validate_leaf_name("-leading"),
            Err(BranchError::InvalidKebabCase(_))
        ));
        assert!(matches!(
            validate_leaf_name("trailing-"),
            Err(BranchError::InvalidKebabCase(_))
        ));
        assert!(matches!(
            validate_leaf_name("Uppercase"),
            Err(BranchError::InvalidKebabCase(_))
        ));
        assert!(matches!(
            validate_leaf_name("with_underscore"),
            Err(BranchError::InvalidKebabCase(_))
        ));
    }

    #[test]
    fn test_branch_formatting() {
        assert_eq!(
            BranchType::Feature.format_branch_name("auth").unwrap(),
            "feature/auth"
        );
        assert_eq!(
            BranchType::Bugfix.format_branch_name("null-ptr").unwrap(),
            "bugfix/null-ptr"
        );
    }
}
