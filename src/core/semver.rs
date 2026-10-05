//! Semantic versioning validation.

/// Validates whether a version string matches SemVer 2.0.0 (with optional leading 'v').
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
}
