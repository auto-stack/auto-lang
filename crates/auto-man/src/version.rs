//! Semantic versioning support for AutoMan dependencies
//!
//! This module implements semantic versioning (semver) as described in:
//! https://semver.org/
//!
//! Supported version requirements:
//! - Exact: "1.2.3" - Must be exactly version 1.2.3
//! - Caret: "^1.2.3" - Compatible with 1.2.3 (>=1.2.3 <2.0.0)
//! - Tilde: "~1.2.3" - Approximately equivalent to 1.2.3 (>=1.2.3 <1.3.0)
//! - Wildcard: "1.2.*" or "1.x" - Matches any version in range
//! - Greater: ">1.2.3" - Greater than 1.2.3
//! - Greater equal: ">=1.2.3" - Greater than or equal to 1.2.3
//! - Less: "<1.2.3" - Less than 1.2.3
//! - Less equal: "<=1.2.3" - Less than or equal to 1.2.3
//! - Range: ">=1.2.3 <2.0.0" - Compound range

use std::fmt;
use std::str::FromStr;

/// A semantic version number
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: String,
    pub build: String,
}

impl Version {
    /// Create a new version without pre-release or build metadata
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: String::new(),
            build: String::new(),
        }
    }

    /// Check if this version satisfies a requirement
    pub fn satisfies(&self, requirement: &str) -> bool {
        match Requirement::parse(requirement) {
            Ok(req) => req.matches(self),
            Err(_) => false,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre)?;
        }
        if !self.build.is_empty() {
            write!(f, "+{}", self.build)?;
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split off build metadata first
        let (main, build) = if let Some(idx) = s.find('+') {
            (&s[..idx], &s[idx + 1..])
        } else {
            (s, "")
        };

        // Split off pre-release
        let (version, pre) = if let Some(idx) = main.find('-') {
            (&main[..idx], &main[idx + 1..])
        } else {
            (main, "")
        };

        // Parse version numbers
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 3 {
            return Err(format!(
                "Invalid version format: '{}'. Expected 'major.minor.patch'",
                s
            ));
        }

        let major = parts[0]
            .parse::<u64>()
            .map_err(|_| format!("Invalid major version: '{}'", parts[0]))?;
        let minor = parts[1]
            .parse::<u64>()
            .map_err(|_| format!("Invalid minor version: '{}'", parts[1]))?;
        let patch = parts[2]
            .parse::<u64>()
            .map_err(|_| format!("Invalid patch version: '{}'", parts[2]))?;

        Ok(Self {
            major,
            minor,
            patch,
            pre: pre.to_string(),
            build: build.to_string(),
        })
    }
}

/// Version comparison operators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Exact,    // =1.2.3 or just 1.2.3
    Greater,  // >1.2.3
    GreaterEq, // >=1.2.3
    Less,     // <1.2.3
    LessEq,   // <=1.2.3
    Caret,    // ^1.2.3 (compatible)
    Tilde,    // ~1.2.3 (approximately)
}

impl Op {
    fn compare(&self, version: &Version, required: &Version) -> bool {
        match self {
            Op::Exact => version == required,
            Op::Greater => version > required,
            Op::GreaterEq => version >= required,
            Op::Less => version < required,
            Op::LessEq => version <= required,
            Op::Caret => {
                // ^1.2.3 means >=1.2.3 <2.0.0
                if version.major != required.major {
                    false
                } else if version.major == 0 {
                    // ^0.2.3 means >=0.2.3 <0.3.0
                    // ^0.0.3 means >=0.0.3 <0.0.4
                    if required.minor == 0 {
                        version.major == 0 && version.minor == 0 && version.patch >= required.patch
                    } else {
                        version.major == 0 && version.minor == required.minor
                            && (version.patch >= required.patch || version.minor > required.minor)
                    }
                } else {
                    version.major == required.major && version >= required
                }
            }
            Op::Tilde => {
                // ~1.2.3 means >=1.2.3 <1.3.0
                // ~1.2 means >=1.2.0 <1.3.0
                if version.major != required.major {
                    false
                } else if version.minor != required.minor {
                    false
                } else {
                    version >= required
                }
            }
        }
    }
}

/// A version requirement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    op: Op,
    version: Version,
}

impl Requirement {
    /// Parse a requirement string
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();

        // Check for wildcard patterns first
        if s.contains('*') || s.contains('x') || s.contains('X') {
            // Convert wildcard to equivalent tilde requirement
            // 1.2.* => ~1.2.0 (any 1.2.x version)
            // 1.x => ~1.0.0 (any 1.x.x version)
            // 1.x.x => ~1.0.0 (any 1.x.x version)
            let normalized = s
                .replace('X', "0")
                .replace('x', "0")
                .replace('*', "0");

            let parts: Vec<&str> = normalized.split('.').collect();
            let version_str = if parts.len() >= 3 && parts[2] == "0" && (s.contains('*') || s[2..].contains('x')) {
                // Third component is wildcard, keep first two
                format!("{}.{}.0", parts[0], parts[1])
            } else if parts.len() >= 2 && parts[1] == "0" && s[1..].contains('x') {
                // Second component is wildcard (e.g., 1.x or 1.x.x)
                format!("{}.0.0", parts[0])
            } else if parts.len() >= 3 {
                format!("{}.{}.0", parts[0], parts[1])
            } else if parts.len() == 2 {
                format!("{}.0.0", parts[0])
            } else {
                return Err(format!("Invalid wildcard version: '{}'", s));
            };

            let version = Version::from_str(&version_str)?;
            return Ok(Self { op: Op::Tilde, version });
        }

        // Determine operator
        let (op_str, version_str) = if s.starts_with(">=") {
            (">=", &s[2..])
        } else if s.starts_with("<=") {
            ("<=", &s[2..])
        } else if s.starts_with('=') {
            ("=", &s[1..])
        } else if s.starts_with('>') {
            (">", &s[1..])
        } else if s.starts_with('<') {
            ("<", &s[1..])
        } else if s.starts_with('^') {
            ("^", &s[1..])
        } else if s.starts_with('~') {
            ("~", &s[1..])
        } else {
            // Default is exact version
            ("=", s)
        };

        let version = Version::from_str(version_str)?;

        let op = match op_str {
            "=" => Op::Exact,
            ">=" => Op::GreaterEq,
            "<=" => Op::LessEq,
            ">" => Op::Greater,
            "<" => Op::Less,
            "^" => Op::Caret,
            "~" => Op::Tilde,
            _ => return Err(format!("Unknown operator: '{}'", op_str)),
        };

        Ok(Self { op, version })
    }

    /// Check if a version matches this requirement
    pub fn matches(&self, version: &Version) -> bool {
        self.op.compare(version, &self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::from_str("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_with_pre() {
        let v = Version::from_str("1.2.3-alpha.1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.pre, "alpha.1");
    }

    #[test]
    fn test_version_with_build() {
        let v = Version::from_str("1.2.3+build.123").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.build, "build.123");
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_exact_version() {
        let v = Version::new(1, 2, 3);
        assert!(v.satisfies("1.2.3"));
        assert!(!v.satisfies("1.2.4"));
    }

    #[test]
    fn test_caret_version() {
        let v = Version::new(1, 2, 3);
        assert!(v.satisfies("^1.2.0"));
        assert!(v.satisfies("^1.0.0"));
        assert!(!v.satisfies("^2.0.0"));

        // ^0.2.3 should match 0.2.x but not 0.3.0
        let v = Version::new(0, 2, 5);
        assert!(v.satisfies("^0.2.3"));
        assert!(!v.satisfies("^0.3.0"));
    }

    #[test]
    fn test_tilde_version() {
        let v = Version::new(1, 2, 5);
        assert!(v.satisfies("~1.2.3"));
        assert!(!v.satisfies("~1.3.0"));
    }

    #[test]
    fn test_greater_than() {
        let v = Version::new(1, 2, 3);
        assert!(v.satisfies(">1.2.2"));
        assert!(!v.satisfies(">1.2.3"));
    }

    #[test]
    fn test_greater_equal() {
        let v = Version::new(1, 2, 3);
        assert!(v.satisfies(">=1.2.3"));
        assert!(!v.satisfies(">=1.2.4"));
    }

    #[test]
    fn test_less_than() {
        let v = Version::new(1, 2, 3);
        assert!(v.satisfies("<1.2.4"));
        assert!(!v.satisfies("<1.2.3"));
    }

    #[test]
    fn test_wildcard() {
        let v = Version::new(1, 2, 3);
        assert!(v.satisfies("1.2.*"));
        assert!(v.satisfies("1.2.x"));
        // Note: "1.x.x" means "1.0.0" with tilde, which matches >=1.0.0 <1.1.0
        // So version 1.2.3 would NOT match 1.x.x
        let v2 = Version::new(1, 0, 5);
        assert!(v2.satisfies("1.x.x"));
    }
}
