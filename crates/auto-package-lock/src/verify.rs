//! Single-sourced lock verification rule.
//!
//! Extracted verbatim from `auto-man`'s `LockFile::verify` (PLAN-795):
//! error strings, aggregation order (resolved-dependency iteration order,
//! joined with `\n`) and the PLAN-635 D6 local-path branch are frozen
//! historical behavior.
//!
//! Honest limits: this checks lock presence, version match and
//! materialized-path existence for commit-less local path deps. A pass is
//! NOT proof of Git content integrity or supply-chain authenticity, and
//! path existence is not a commit verification.

use crate::model::{LockedEntryView, ResolvedDependency};
use std::path::Path;

/// Verify that all resolved dependencies match the locked entries.
///
/// `locked` is the lock's entry list (any type exposing the entry view);
/// `resolved` is the caller-projected dependency set (Dep/Device filtered,
/// in the caller's iteration order — error order follows this order).
pub fn verify_dependencies<L: LockedEntryView>(
    locked: &[L],
    resolved: &[ResolvedDependency],
) -> Result<(), String> {
    let mut errors = Vec::new();

    for dep in resolved {
        match locked.iter().find(|d| d.lock_name() == dep.name) {
            Some(locked_entry) => {
                if locked_entry.lock_version() != dep.version {
                    errors.push(format!(
                        "Dependency '{}' version mismatch: lockfile has {}, but resolved {}",
                        dep.name,
                        locked_entry.lock_version(),
                        dep.version
                    ));
                }
                // PLAN-635 D6: local path deps (no git commit pin) verify
                // by materialized-path existence.
                if locked_entry.lock_commit().is_empty() && !locked_entry.lock_path().is_empty() {
                    let p = Path::new(locked_entry.lock_path());
                    if !p.is_dir() {
                        errors.push(format!(
                            "Dependency '{}' path '{}' is not materialized",
                            dep.name,
                            locked_entry.lock_path()
                        ));
                    }
                }
            }
            None => {
                errors.push(format!("Dependency '{}' not found in lock file", dep.name));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LockEntry;

    fn entry(name: &str, version: &str, commit: &str, path: &str) -> LockEntry {
        LockEntry {
            name: name.to_string(),
            version: version.to_string(),
            url: String::new(),
            commit: commit.to_string(),
            path: path.to_string(),
        }
    }

    fn resolved(name: &str, version: &str) -> ResolvedDependency {
        ResolvedDependency::new(name, version)
    }

    #[test]
    fn empty_resolved_set_passes() {
        let locked = [entry("a", "1.0.0", "c1", "")];
        assert_eq!(verify_dependencies(&locked, &[]), Ok(()));
    }

    #[test]
    fn version_mismatch_error_is_verbatim() {
        let locked = [entry("lib-a", "1.0.0", "c1", "")];
        let err = verify_dependencies(&locked, &[resolved("lib-a", "2.0.0")]).unwrap_err();
        assert_eq!(
            err,
            "Dependency 'lib-a' version mismatch: lockfile has 1.0.0, but resolved 2.0.0"
        );
    }

    #[test]
    fn missing_entry_error_is_verbatim() {
        let err = verify_dependencies::<LockEntry>(&[], &[resolved("ghost", "1.0.0")]).unwrap_err();
        assert_eq!(err, "Dependency 'ghost' not found in lock file");
    }

    #[test]
    fn errors_aggregate_in_resolved_order_joined_by_newline() {
        let locked = [entry("a", "1.0.0", "c1", "")];
        let err = verify_dependencies(
            &locked,
            &[resolved("a", "9.9.9"), resolved("ghost", "1.0.0")],
        )
        .unwrap_err();
        assert_eq!(
            err,
            "Dependency 'a' version mismatch: lockfile has 1.0.0, but resolved 9.9.9\n\
             Dependency 'ghost' not found in lock file"
        );
    }

    #[test]
    fn first_duplicate_entry_wins_for_verification() {
        let locked = [
            entry("dup", "1.0.0", "c1", ""),
            entry("dup", "2.0.0", "c1", ""),
        ];
        // Resolved matches the FIRST entry only.
        assert_eq!(
            verify_dependencies(&locked, &[resolved("dup", "1.0.0")]),
            Ok(())
        );
        assert!(verify_dependencies(&locked, &[resolved("dup", "2.0.0")]).is_err());
    }

    #[test]
    fn empty_commit_and_empty_path_skips_existence_check() {
        // Empty path with empty commit: nothing to check — passes. This is
        // historical behavior; do not "harden" it here (PLAN-795 keeps the
        // old protocol bit-for-bit).
        let locked = [entry("a", "1.0.0", "", "")];
        assert_eq!(
            verify_dependencies(&locked, &[resolved("a", "1.0.0")]),
            Ok(())
        );
    }
}
