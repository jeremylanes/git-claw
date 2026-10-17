//! Semantic versioning validation, parsing, and bumping.

use std::fmt;

/// Semantic version bump type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BumpType {
    /// Increments the patch version (e.g., 1.0.0 -> 1.0.1).
    Patch,
    /// Increments the minor version and resets patch (e.g., 1.0.1 -> 1.1.0).
    Minor,
    /// Increments the major version and resets minor and patch (e.g., 1.1.0 -> 2.0.0).
    Major,
}

/// Represents a parsed Semantic Version (SemVer 2.0.0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemVer {
    /// Major version number.
    pub major: u64,
    /// Minor version number.
    pub minor: u64,
    /// Patch version number.
    pub patch: u64,
    /// Whether the original version string was prefixed with 'v'.
    pub has_v_prefix: bool,
}

impl SemVer {
    /// Bumps this semantic version according to the specified [`BumpType`].
    ///
    /// Preserves the `has_v_prefix` flag.
    ///
    /// # Arguments
    ///
    /// * `bump` - The type of bump to perform (Patch, Minor, Major).
    ///
    /// # Example
    ///
    /// ```
    /// use git_claw::core::semver::{BumpType, SemVer};
    ///
    /// let v = SemVer { major: 1, minor: 2, patch: 3, has_v_prefix: true };
    /// assert_eq!(v.bump(BumpType::Patch).to_string(), "v1.2.4");
    /// assert_eq!(v.bump(BumpType::Minor).to_string(), "v1.3.0");
    /// assert_eq!(v.bump(BumpType::Major).to_string(), "v2.0.0");
    /// ```
    pub fn bump(&self, bump: BumpType) -> SemVer {
        match bump {
            BumpType::Major => SemVer {
                major: self.major + 1,
                minor: 0,
                patch: 0,
                has_v_prefix: self.has_v_prefix,
            },
            BumpType::Minor => SemVer {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
                has_v_prefix: self.has_v_prefix,
            },
            BumpType::Patch => SemVer {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
                has_v_prefix: self.has_v_prefix,
            },
        }
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_v_prefix {
            write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Parses a version string into a [`SemVer`] struct.
///
/// Validates SemVer 2.0.0 format (with optional leading 'v') and extracts
/// major, minor, and patch numbers while discarding any prerelease/build metadata.
///
/// # Arguments
///
/// * `version` - Version string to parse (e.g., "1.2.3", "v0.4.0", "v1.0.0-beta.1").
///
/// # Example
///
/// ```
/// use git_claw::core::semver::{parse_semver, SemVer};
///
/// let v = parse_semver("v1.2.3").unwrap();
/// assert_eq!(v, SemVer { major: 1, minor: 2, patch: 3, has_v_prefix: true });
///
/// let v2 = parse_semver("0.1.0").unwrap();
/// assert_eq!(v2, SemVer { major: 0, minor: 1, patch: 0, has_v_prefix: false });
///
/// assert!(parse_semver("invalid").is_none());
/// ```
pub fn parse_semver(version: &str) -> Option<SemVer> {
    if !validate_semver(version) {
        return None;
    }

    let has_v_prefix = version.starts_with('v');
    let s = version.strip_prefix('v').unwrap_or(version);
    let core = s.split(['+', '-']).next().unwrap_or(s);
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major = parts[0].parse::<u64>().ok()?;
    let minor = parts[1].parse::<u64>().ok()?;
    let patch = parts[2].parse::<u64>().ok()?;

    Some(SemVer {
        major,
        minor,
        patch,
        has_v_prefix,
    })
}

/// Validates whether a version string matches SemVer 2.0.0 (with optional leading 'v').
///
/// # Arguments
///
/// * `version` - Version string to validate.
///
/// # Example
///
/// ```
/// use git_claw::core::semver::validate_semver;
///
/// assert!(validate_semver("v1.2.3"));
/// assert!(validate_semver("0.1.0"));
/// assert!(!validate_semver("1.0"));
/// ```
pub fn validate_semver(version: &str) -> bool {
    let s = version.strip_prefix('v').unwrap_or(version);
    if s.is_empty() {
        return false;
    }

    // Split off build metadata (+)
    let (s, _build) = match s.split_once('+') {
        Some((core, b)) => {
            if b.is_empty()
                || !b
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
            {
                return false;
            }
            (core, Some(b))
        }
        None => (s, None),
    };

    // Split off pre-release (-)
    let (core, _pre) = match s.split_once('-') {
        Some((c, p)) => {
            if p.is_empty()
                || !p
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
            {
                return false;
            }
            (c, Some(p))
        }
        None => (s, None),
    };

    // Core must be MAJOR.MINOR.PATCH
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    for part in parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        // No leading zero if length > 1
        if part.len() > 1 && part.starts_with('0') {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_semver() {
        assert!(validate_semver("1.0.0"));
        assert!(validate_semver("v1.0.0"));
        assert!(validate_semver("v0.1.0"));
        assert!(validate_semver("v1.2.3-alpha.1"));
        assert!(validate_semver("v1.2.3+build.42"));
        assert!(validate_semver("v1.2.3-rc.1+build.1"));
    }

    #[test]
    fn test_invalid_semver() {
        assert!(!validate_semver(""));
        assert!(!validate_semver("v"));
        assert!(!validate_semver("1.0"));
        assert!(!validate_semver("v1.0"));
        assert!(!validate_semver("1.0.0.0"));
        assert!(!validate_semver("v01.0.0"));
        assert!(!validate_semver("latest"));
        assert!(!validate_semver("v1.0.0-"));
    }

    #[test]
    fn test_parse_semver() {
        let v1 = parse_semver("v1.2.3").expect("valid");
        assert_eq!(
            v1,
            SemVer {
                major: 1,
                minor: 2,
                patch: 3,
                has_v_prefix: true
            }
        );
        assert_eq!(v1.to_string(), "v1.2.3");

        let v2 = parse_semver("0.4.12").expect("valid");
        assert_eq!(
            v2,
            SemVer {
                major: 0,
                minor: 4,
                patch: 12,
                has_v_prefix: false
            }
        );
        assert_eq!(v2.to_string(), "0.4.12");

        let v3 = parse_semver("v2.0.0-rc.1+build.5").expect("valid");
        assert_eq!(
            v3,
            SemVer {
                major: 2,
                minor: 0,
                patch: 0,
                has_v_prefix: true
            }
        );

        assert!(parse_semver("not-semver").is_none());
        assert!(parse_semver("v1.0").is_none());
    }

    #[test]
    fn test_semver_bump() {
        let v = SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            has_v_prefix: true,
        };
        assert_eq!(v.bump(BumpType::Patch).to_string(), "v1.2.4");
        assert_eq!(v.bump(BumpType::Minor).to_string(), "v1.3.0");
        assert_eq!(v.bump(BumpType::Major).to_string(), "v2.0.0");

        let v_plain = SemVer {
            major: 0,
            minor: 1,
            patch: 0,
            has_v_prefix: false,
        };
        assert_eq!(v_plain.bump(BumpType::Patch).to_string(), "0.1.1");
        assert_eq!(v_plain.bump(BumpType::Minor).to_string(), "0.2.0");
        assert_eq!(v_plain.bump(BumpType::Major).to_string(), "1.0.0");
    }
}
