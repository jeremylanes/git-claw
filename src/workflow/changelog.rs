//! Changelog generation and update utilities.

use crate::core::conventional::ConventionalCommit;
use std::path::Path;

/// Generates a markdown changelog release entry grouped by conventional commit categories.
///
/// Formats under `## [<version>] - <date>` followed by categorized commit lists:
/// - `### Breaking Changes`
/// - `### Features`
/// - `### Bug Fixes`
/// - `### Performance Improvements`
/// - `### Other Changes`
///
/// If a commit has an optional scope, it is formatted as `- **<scope>**: <description>`.
/// Otherwise, it is formatted as `- <description>`.
///
/// # Arguments
///
/// * `version` - Target release version string (e.g., "v1.2.0").
/// * `date` - Formatted release date (e.g., "2026-10-17").
/// * `commits` - Slice of conventional commits included in the release.
///
/// # Example
///
/// ```
/// use git_claw::core::conventional::ConventionalCommit;
/// use git_claw::workflow::changelog::generate_changelog_entry;
///
/// let commits = vec![
///     ConventionalCommit {
///         commit_type: "feat".to_string(),
///         scope: Some("auth".to_string()),
///         is_breaking: false,
///         description: "add OAuth2 login".to_string(),
///     },
/// ];
/// let entry = generate_changelog_entry("v1.1.0", "2026-10-17", &commits);
/// assert!(entry.contains("## [v1.1.0] - 2026-10-17"));
/// assert!(entry.contains("### Features"));
/// assert!(entry.contains("- **auth**: add OAuth2 login"));
/// ```
pub fn generate_changelog_entry(
    version: &str,
    date: &str,
    commits: &[ConventionalCommit],
) -> String {
    let mut breaking = Vec::new();
    let mut features = Vec::new();
    let mut fixes = Vec::new();
    let mut perf = Vec::new();
    let mut other = Vec::new();

    for commit in commits {
        if commit.is_breaking {
            breaking.push(commit);
        } else {
            match commit.commit_type.as_str() {
                "feat" => features.push(commit),
                "fix" => fixes.push(commit),
                "perf" => perf.push(commit),
                _ => other.push(commit),
            }
        }
    }

    let mut entry = format!("## [{}] - {}\n", version, date);
    let sections = [
        ("Breaking Changes", breaking),
        ("Features", features),
        ("Bug Fixes", fixes),
        ("Performance Improvements", perf),
        ("Other Changes", other),
    ];

    for (title, list) in sections {
        if !list.is_empty() {
            entry.push_str(&format!("\n### {}\n\n", title));
            for c in list {
                if let Some(scope) = &c.scope {
                    entry.push_str(&format!("- **{}**: {}\n", scope, c.description));
                } else {
                    entry.push_str(&format!("- {}\n", c.description));
                }
            }
        }
    }

    entry
}

/// Updates or creates `CHANGELOG.md` in the specified repository root.
///
/// If `CHANGELOG.md` exists:
/// - If it contains `# Changelog`, inserts the release entry immediately after the title line.
/// - Otherwise, prepends the release entry before existing contents.
/// If `CHANGELOG.md` does not exist, creates it with a `# Changelog` heading and the entry.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root.
/// * `entry` - Release entry markdown string to insert.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::workflow::changelog::update_changelog_file;
///
/// # fn run() -> std::io::Result<()> {
/// update_changelog_file(Path::new("."), "## [v1.0.0] - 2026-10-17\n\n### Features\n- init\n")?;
/// # Ok(())
/// # }
/// ```
pub fn update_changelog_file(repo_root: &Path, entry: &str) -> std::io::Result<()> {
    let changelog_path = repo_root.join("CHANGELOG.md");
    let trimmed_entry = entry.trim();

    if changelog_path.exists() {
        let content = std::fs::read_to_string(&changelog_path)?;
        let new_content = if let Some(idx) = content.find("# Changelog") {
            let line_end = content[idx..]
                .find('\n')
                .map(|i| idx + i + 1)
                .unwrap_or(content.len());
            let prefix = &content[..line_end];
            let suffix = &content[line_end..];
            let suffix_clean = suffix.trim_start_matches(['\r', '\n']);
            if suffix_clean.is_empty() {
                format!("{}\n\n{}\n", prefix.trim_end(), trimmed_entry)
            } else {
                format!(
                    "{}\n\n{}\n\n{}",
                    prefix.trim_end(),
                    trimmed_entry,
                    suffix_clean
                )
            }
        } else {
            format!("{}\n\n{}", trimmed_entry, content)
        };
        std::fs::write(&changelog_path, new_content)?;
    } else {
        std::fs::write(
            &changelog_path,
            format!("# Changelog\n\n{}\n", trimmed_entry),
        )?;
    }

    Ok(())
}

/// Returns the current UTC date formatted as `YYYY-MM-DD`.
///
/// Computes date from system time using Gregorian calendar arithmetic without external dependencies.
///
/// # Example
///
/// ```
/// use git_claw::workflow::changelog::current_date_ymd;
///
/// let today = current_date_ymd();
/// assert_eq!(today.len(), 10);
/// assert_eq!(&today[4..5], "-");
/// assert_eq!(&today[7..8], "-");
/// ```
pub fn current_date_ymd() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (secs / 86400) as i64;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_changelog_entry_grouping() {
        let commits = vec![
            ConventionalCommit {
                commit_type: "fix".to_string(),
                scope: None,
                is_breaking: true,
                description: "break everything".to_string(),
            },
            ConventionalCommit {
                commit_type: "feat".to_string(),
                scope: Some("auth".to_string()),
                is_breaking: false,
                description: "support jwt".to_string(),
            },
            ConventionalCommit {
                commit_type: "fix".to_string(),
                scope: None,
                is_breaking: false,
                description: "null pointer exception".to_string(),
            },
            ConventionalCommit {
                commit_type: "perf".to_string(),
                scope: Some("db".to_string()),
                is_breaking: false,
                description: "index lookup".to_string(),
            },
            ConventionalCommit {
                commit_type: "chore".to_string(),
                scope: None,
                is_breaking: false,
                description: "update license".to_string(),
            },
        ];

        let entry = generate_changelog_entry("v1.0.0", "2026-10-17", &commits);
        assert!(entry.starts_with("## [v1.0.0] - 2026-10-17\n"));
        assert!(entry.contains("### Breaking Changes\n\n- break everything\n"));
        assert!(entry.contains("### Features\n\n- **auth**: support jwt\n"));
        assert!(entry.contains("### Bug Fixes\n\n- null pointer exception\n"));
        assert!(entry.contains("### Performance Improvements\n\n- **db**: index lookup\n"));
        assert!(entry.contains("### Other Changes\n\n- update license\n"));
    }

    #[test]
    fn test_update_changelog_file_fresh() {
        let dir = tempdir().unwrap();
        let entry = "## [v0.1.0] - 2026-10-17\n\n### Features\n\n- init";
        update_changelog_file(dir.path(), entry).unwrap();

        let content = std::fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
        assert_eq!(
            content,
            "# Changelog\n\n## [v0.1.0] - 2026-10-17\n\n### Features\n\n- init\n"
        );
    }

    #[test]
    fn test_update_changelog_file_existing_with_header() {
        let dir = tempdir().unwrap();
        let initial =
            "# Changelog\n\n## [v0.1.0] - 2026-10-16\n\n### Features\n\n- initial release\n";
        std::fs::write(dir.path().join("CHANGELOG.md"), initial).unwrap();

        let entry = "## [v0.2.0] - 2026-10-17\n\n### Features\n\n- new feature";
        update_changelog_file(dir.path(), entry).unwrap();

        let content = std::fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
        let expected = "# Changelog\n\n## [v0.2.0] - 2026-10-17\n\n### Features\n\n- new feature\n\n## [v0.1.0] - 2026-10-16\n\n### Features\n\n- initial release\n";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_current_date_ymd_format() {
        let date = current_date_ymd();
        assert_eq!(date.len(), 10);
        let parts: Vec<&str> = date.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].len(), 4);
        assert_eq!(parts[1].len(), 2);
        assert_eq!(parts[2].len(), 2);
    }
}
