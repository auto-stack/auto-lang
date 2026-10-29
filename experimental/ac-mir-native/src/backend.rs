//! PLAN-871 native backend selection (driver layer, freeze F1/F5).
//!
//! `ac-core` owns the frontend/Checked HIR; `ac-mn` owns the shared-MIR
//! backend; this module is the ONLY place the two are compared and chosen,
//! so neither crate gains a reverse dependency (the `ac-core ⇄ ac-mn` cycle
//! stays impossible). It is consumed by the `ac-native` CLI and by the
//! `ac-case-worker` driver arms — never by `ac-core` itself.
//!
//! Selection grammar (frozen in PLAN-871 T-02 §F1, read on EVERY call, no
//! process memoization):
//!
//! - `AC_NATIVE_BACKEND` unset / empty          → the default table below;
//! - `mn` | `legacy`                           → that backend for all five;
//! - `core=legacy,scalar=mn,…`                 → per-profile overrides on
//!   top of the default table (migration-window fallback);
//! - any other value (unknown profile, unknown backend, malformed pair) is
//!   an EXPLICIT error — never a silent default.
//!
//! The five profiles are separate identities: a switch (T-04) is a
//! per-profile decision and a fallback is per profile too.

use crate::config;

/// The five frozen source profiles (same names as `SourceProfile`/HIR
/// domains, spelled as they appear in reports and case files).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Profile {
    Core,
    Scalar,
    Text,
    Record,
    TextRecord,
}

impl Profile {
    pub const ALL: [Profile; 5] = [
        Profile::Core,
        Profile::Scalar,
        Profile::Text,
        Profile::Record,
        Profile::TextRecord,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Profile::Core => "core",
            Profile::Scalar => "scalar",
            Profile::Text => "text",
            Profile::Record => "record",
            Profile::TextRecord => "text-record",
        }
    }

    pub fn parse(s: &str) -> Option<Profile> {
        Profile::ALL.into_iter().find(|p| p.as_str() == s)
    }
}

/// The two native backends of the migration window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// The retiring `ac-core` HIR→CLIF lowering (`native.rs`).
    Legacy,
    /// The shared MIR chain (`auto-mir-lower` + `ac-mn`).
    Mn,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Backend::Legacy => "legacy",
            Backend::Mn => "mn",
        }
    }

    pub fn parse(s: &str) -> Option<Backend> {
        match s {
            "legacy" => Some(Backend::Legacy),
            "mn" => Some(Backend::Mn),
            _ => None,
        }
    }

    /// The report/toolchain axis value (F1/F5): the legacy backend reports
    /// its lowering identity, mn reports its own abi + lowering version so
    /// the two can never be confused in a report or a cache key.
    pub fn tag(self) -> String {
        match self {
            Backend::Legacy => "cranelift-0.126/lower_object".to_string(),
            Backend::Mn => format!(
                "mn:{}+mir-v{}",
                config::MN_ABI,
                auto_mir_lower::MIR_LOWERING_VERSION
            ),
        }
    }
}

/// Malformed `AC_NATIVE_BACKEND`: surfaced as a typed refusal, never a
/// silent fallback (a typo cannot quietly keep running the other backend).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecError(pub String);

impl std::fmt::Display for SpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SpecError {}

/// T-04 switch point: after the five-profile G-COV + shadow gates passed
/// (T-03 receipt `attachments/871/work/shadow-t03.md`) the default table
/// is `mn` for every profile. Reverting a profile is an env override
/// (`AC_NATIVE_BACKEND=<profile>=legacy`), never an edit here.
pub fn default_backend(_profile: Profile) -> Backend {
    Backend::Mn
}

/// The frozen default table, in report order.
pub fn default_table() -> Vec<(Profile, Backend)> {
    Profile::ALL
        .into_iter()
        .map(|p| (p, default_backend(p)))
        .collect()
}

/// Parse a raw `AC_NATIVE_BACKEND` value into per-profile overrides.
/// `None`/empty = no overrides. Fully validates the whole value before
/// returning: a partially valid string yields NOTHING (no half-applied
/// switch).
pub fn overrides(raw: Option<&str>) -> Result<Vec<(Profile, Backend)>, SpecError> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    match raw {
        "mn" => Ok(Profile::ALL.into_iter().map(|p| (p, Backend::Mn)).collect()),
        "legacy" => Ok(Profile::ALL
            .into_iter()
            .map(|p| (p, Backend::Legacy))
            .collect()),
        _ => {
            let mut out: Vec<(Profile, Backend)> = Vec::new();
            for token in raw.split(',') {
                let token = token.trim();
                if token.is_empty() {
                    continue;
                }
                let Some((name, value)) = token.split_once('=') else {
                    return Err(SpecError(format!(
                        "AC_NATIVE_BACKEND token {token:?} is neither `mn`/`legacy` nor a `profile=backend` pair"
                    )));
                };
                let Some(profile) = Profile::parse(name.trim()) else {
                    return Err(SpecError(format!(
                        "AC_NATIVE_BACKEND names unknown profile {name:?} (known: core, scalar, text, record, text-record)"
                    )));
                };
                let Some(backend) = Backend::parse(value.trim()) else {
                    return Err(SpecError(format!(
                        "AC_NATIVE_BACKEND profile {} names unknown backend {value:?} (expected legacy|mn)",
                        profile.as_str()
                    )));
                };
                if out.iter().any(|(p, _)| *p == profile) {
                    return Err(SpecError(format!(
                        "AC_NATIVE_BACKEND sets profile {} twice",
                        profile.as_str()
                    )));
                }
                out.push((profile, backend));
            }
            if out.is_empty() {
                return Err(SpecError(
                    "AC_NATIVE_BACKEND is set but names no profile".to_string(),
                ));
            }
            Ok(out)
        }
    }
}

/// Resolve one profile from an explicit raw value (pure; used by tests and
/// by callers that read the env once per request).
pub fn resolve_from(raw: Option<&str>, profile: Profile) -> Result<Backend, SpecError> {
    let overrides = overrides(raw)?;
    Ok(overrides
        .into_iter()
        .find(|(p, _)| *p == profile)
        .map(|(_, b)| b)
        .unwrap_or_else(|| default_backend(profile)))
}

/// Resolve one profile from the process environment (read on every call —
/// a running worker never memoizes a switch; F1).
pub fn resolve(profile: Profile) -> Result<Backend, SpecError> {
    let raw = std::env::var("AC_NATIVE_BACKEND").ok();
    resolve_from(raw.as_deref(), profile)
}

/// The observable switch state (per profile), for reports/diagnostics.
pub fn table(raw: Option<&str>) -> Result<Vec<(Profile, Backend)>, SpecError> {
    let overrides = overrides(raw)?;
    Ok(Profile::ALL
        .into_iter()
        .map(|p| {
            let b = overrides
                .iter()
                .find(|(q, _)| *q == p)
                .map(|(_, b)| *b)
                .unwrap_or_else(|| default_backend(p));
            (p, b)
        })
        .collect())
}

#[cfg(test)]
mod tests_871 {
    use super::*;

    #[test]
    fn default_table_is_mn_after_the_gates() {
        for (p, b) in default_table() {
            assert_eq!(b, Backend::Mn, "{} defaults to mn", p.as_str());
        }
    }

    #[test]
    fn global_values_flip_every_profile() {
        for raw in ["mn", "legacy"] {
            let t = table(Some(raw)).unwrap();
            assert_eq!(t.len(), 5);
            let want = Backend::parse(raw).unwrap();
            for (_, b) in t {
                assert_eq!(b, want);
            }
        }
    }

    #[test]
    fn per_profile_fallback_is_independent() {
        let raw = Some("core=legacy,text=legacy");
        assert_eq!(resolve_from(raw, Profile::Core).unwrap(), Backend::Legacy);
        assert_eq!(resolve_from(raw, Profile::Text).unwrap(), Backend::Legacy);
        assert_eq!(resolve_from(raw, Profile::Scalar).unwrap(), Backend::Mn);
        assert_eq!(resolve_from(raw, Profile::Record).unwrap(), Backend::Mn);
        assert_eq!(resolve_from(raw, Profile::TextRecord).unwrap(), Backend::Mn);
    }

    #[test]
    fn unset_and_empty_mean_the_default_table() {
        assert_eq!(resolve_from(None, Profile::Core).unwrap(), Backend::Mn);
        assert_eq!(
            resolve_from(Some("  "), Profile::Text).unwrap(),
            Backend::Mn
        );
    }

    #[test]
    fn malformed_specs_are_explicit_errors() {
        for bad in [
            "core",
            "core=clif",
            "widget=mn",
            "core=mn,core=legacy",
            " , ",
        ] {
            let e = overrides(Some(bad)).expect_err(bad);
            assert!(!e.0.is_empty());
            // The whole value is refused: no half-applied switch.
            assert!(
                resolve_from(Some(bad), Profile::Scalar).is_err(),
                "{bad} must not resolve any profile"
            );
        }
    }

    #[test]
    fn tags_are_distinct_and_versioned() {
        let legacy = Backend::Legacy.tag();
        let mn = Backend::Mn.tag();
        assert_eq!(legacy, "cranelift-0.126/lower_object");
        assert_eq!(
            mn,
            format!(
                "mn:{}+mir-v{}",
                config::MN_ABI,
                auto_mir_lower::MIR_LOWERING_VERSION
            )
        );
        assert_ne!(legacy, mn);
    }
}
