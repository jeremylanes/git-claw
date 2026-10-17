//! Conventional Commits parser and SemVer bump calculator.

use crate::core::semver::BumpType;

/// Standard Conventional Commit types recognized by the parser.
pub const STANDARD_TYPES: &[&str] = &[
    "feat", "fix", "perf", "refactor", "chore", "docs", "style", "test", "build", "ci",
];

/// Parsed Conventional Commit record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionalCommit {
    /// Commit type (e.g., "feat", "fix", "chore").
    pub commit_type: String,
    /// Optional scope qualifier (e.g., Some("auth") for `feat(auth): ...`).
    pub scope: Option<String>,
    /// Whether the commit introduces a breaking change.
    pub is_breaking: bool,
    /// Summary description from the subject line.
    pub description: String,
}

/// Parses a commit message string into a [`ConventionalCommit`].
///
/// Recognizes standard types (`feat`, `fix`, `perf`, `refactor`, `chore`, `docs`,
/// `style`, `test`, `build`, `ci`) and optional scopes. A breaking change is detected
/// if the type/scope header ends with `!` or if `BREAKING CHANGE:` / `BREAKING-CHANGE:`
/// appears in the commit body or footers.
///
/// Returns `None` if the commit does not adhere to the Conventional Commits specification.
///
/// # Arguments
///
/// * `message` - Full git commit message (subject and optional body/footers).
///
/// # Example
///
/// ```
/// use git_claw::core::conventional::parse_conventional_commit;
///
/// let commit = parse_conventional_commit("feat(auth)!: add OAuth2 provider").unwrap();
/// assert_eq!(commit.commit_type, "feat");
/// assert_eq!(commit.scope, Some("auth".to_string()));
/// assert!(commit.is_breaking);
/// assert_eq!(commit.description, "add OAuth2 provider");
/// ```
pub fn parse_conventional_commit(message: &str) -> Option<ConventionalCommit> {
    let first_line = message.lines().next()?.trim();
    let (header_prefix, description) = first_line.split_once(": ")?;
    let description = description.trim();
    if description.is_empty() {
        return None;
    }

    let (prefix_without_bang, bang) = if let Some(stripped) = header_prefix.strip_suffix('!') {
        (stripped, true)
    } else {
        (header_prefix, false)
    };

    let (commit_type, scope) = if let Some((t, rest)) = prefix_without_bang.split_once('(') {
        let scope_val = rest.strip_suffix(')')?;
        (t.trim(), Some(scope_val.trim().to_string()))
    } else {
        (prefix_without_bang.trim(), None)
    };

    if !STANDARD_TYPES.contains(&commit_type) {
        return None;
    }

    let is_breaking = bang
        || message.contains("BREAKING CHANGE:")
        || message.contains("BREAKING-CHANGE:")
        || message.contains("BREAKING CHANGE ");

    Some(ConventionalCommit {
        commit_type: commit_type.to_string(),
        scope,
        is_breaking,
        description: description.to_string(),
    })
}

/// Determines the SemVer [`BumpType`] from a list of conventional commits.
///
/// Hierarchy:
/// 1. If any commit is breaking -> [`BumpType::Major`]
/// 2. Else if any commit has type "feat" -> [`BumpType::Minor`]
/// 3. Otherwise -> [`BumpType::Patch`]
///
/// # Arguments
///
/// * `commits` - Slice of parsed conventional commits.
///
/// # Example
///
/// ```
/// use git_claw::core::conventional::{determine_bump, ConventionalCommit};
/// use git_claw::core::semver::BumpType;
///
/// let commits = vec![
///     ConventionalCommit {
///         commit_type: "fix".to_string(),
///         scope: None,
///         is_breaking: false,
///         description: "fix bug".to_string(),
///     },
/// ];
/// assert_eq!(determine_bump(&commits), BumpType::Patch);
/// ```
pub fn determine_bump(commits: &[ConventionalCommit]) -> BumpType {
    if commits.iter().any(|c| c.is_breaking) {
        BumpType::Major
    } else if commits.iter().any(|c| c.commit_type == "feat") {
        BumpType::Minor
    } else {
        BumpType::Patch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_feat() {
        let commit = parse_conventional_commit("feat: add search endpoint").unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, None);
        assert!(!commit.is_breaking);
        assert_eq!(commit.description, "add search endpoint");
    }

    #[test]
    fn test_parse_scoped_fix() {
        let commit = parse_conventional_commit("fix(router): resolve redirect loop").unwrap();
        assert_eq!(commit.commit_type, "fix");
        assert_eq!(commit.scope, Some("router".to_string()));
        assert!(!commit.is_breaking);
        assert_eq!(commit.description, "resolve redirect loop");
    }

    #[test]
    fn test_parse_breaking_with_bang() {
        let commit = parse_conventional_commit("feat!: drop v1 API support").unwrap();
        assert_eq!(commit.commit_type, "feat");
        assert_eq!(commit.scope, None);
        assert!(commit.is_breaking);
        assert_eq!(commit.description, "drop v1 API support");

        let commit_scoped = parse_conventional_commit("refactor(db)!: change schema").unwrap();
        assert_eq!(commit_scoped.commit_type, "refactor");
        assert_eq!(commit_scoped.scope, Some("db".to_string()));
        assert!(commit_scoped.is_breaking);
    }

    #[test]
    fn test_parse_breaking_in_body() {
        let msg = "fix: update auth flow\n\nBREAKING CHANGE: credentials format changed";
        let commit = parse_conventional_commit(msg).unwrap();
        assert_eq!(commit.commit_type, "fix");
        assert!(commit.is_breaking);

        let msg_hyphen = "chore: update deps\n\nBREAKING-CHANGE: requires node 20";
        let commit_hyphen = parse_conventional_commit(msg_hyphen).unwrap();
        assert!(commit_hyphen.is_breaking);
    }

    #[test]
    fn test_parse_non_conventional() {
        assert!(parse_conventional_commit("Initial commit").is_none());
        assert!(parse_conventional_commit("Merge branch 'main'").is_none());
        assert!(parse_conventional_commit("customtype: something").is_none());
        assert!(parse_conventional_commit("feat: ").is_none());
    }

    #[test]
    fn test_determine_bump() {
        let fix = ConventionalCommit {
            commit_type: "fix".to_string(),
            scope: None,
            is_breaking: false,
            description: "bug fix".to_string(),
        };
        let feat = ConventionalCommit {
            commit_type: "feat".to_string(),
            scope: None,
            is_breaking: false,
            description: "new feature".to_string(),
        };
        let breaking = ConventionalCommit {
            commit_type: "fix".to_string(),
            scope: None,
            is_breaking: true,
            description: "breaking fix".to_string(),
        };

        assert_eq!(determine_bump(&[]), BumpType::Patch);
        assert_eq!(determine_bump(&[fix.clone()]), BumpType::Patch);
        assert_eq!(determine_bump(&[fix.clone(), feat.clone()]), BumpType::Minor);
        assert_eq!(determine_bump(&[fix, feat, breaking]), BumpType::Major);
    }
}
