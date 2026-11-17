//! PLAN-978 T-08/T-09 (AC-ID-RUN/AC-ID-PACK, V-ID-SELECT): the provider
//! artifact selection + admission record.
//!
//! Frozen rules (plan §5 ID-01/03/04):
//! * the selection carries the provider id, its generation, the artifacts
//!   with REAL content hashes and the EXACT pin the consumer locked;
//! * admission re-checks: pin == on-disk hash, artifacts present,
//!   required subset intact, CURRENT authorization valid — every check
//!   BEFORE dispatch;
//! * a NEW artifact hash (e.g. optional symbols added, required subset
//!   intact) does NOT auto-upgrade a precise pin — an explicit selection
//!   update re-admits it;
//! * compatibility (crate::compat) and selection are SEPARATE ledgers:
//!   "ABI compatible" never silently releases a pin, and a pin match
//!   never waives the compatibility check.

use serde::{Deserialize, Serialize};

pub const SELECTION_SCHEMA: &str = "ace-m2-build/provider-selection/1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderArtifact {
    pub provider_id: String,
    /// Path relative to the candidate SDK root.
    pub path: String,
    pub sha256: String,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSelection {
    pub schema: String,
    pub provider_id: String,
    pub generation: u64,
    pub artifacts: Vec<ProviderArtifact>,
    /// The EXACT content hash the consumer pinned (a changed hash with an
    /// intact required subset still requires an explicit re-admission).
    pub pinned_sha256: String,
    /// The current-run authorization token (opaque; re-validated at every
    /// admission — never cached as standing trust).
    pub authorization: String,
}

impl ProviderSelection {
    pub fn new(
        provider_id: &str,
        generation: u64,
        artifacts: Vec<ProviderArtifact>,
        pinned_sha256: &str,
        authorization: &str,
    ) -> Self {
        Self {
            schema: SELECTION_SCHEMA.into(),
            provider_id: provider_id.into(),
            generation,
            artifacts,
            pinned_sha256: pinned_sha256.into(),
            authorization: authorization.into(),
        }
    }
}

/// A named selection rejection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionRejection {
    pub code: &'static str,
    pub detail: String,
}

impl std::fmt::Display for SelectionRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionReceipt {
    pub provider_id: String,
    pub generation: u64,
    pub admitted_sha256: String,
}

/// Admit one selection for dispatch.
///
/// * `on_disk_sha256` — the REAL hash of the artifact bytes as read now;
/// * `required_subset_intact` — the consumer's required symbol subset is
///   still offered (the caller holds the compat ledger);
/// * `authorization_valid` — the CURRENT authorization (per-run).
pub fn admit_selection(
    selection: &ProviderSelection,
    on_disk_sha256: &str,
    required_subset_intact: bool,
    authorization_valid: bool,
) -> Result<AdmissionReceipt, SelectionRejection> {
    if !authorization_valid {
        return Err(SelectionRejection {
            code: "m2build.selection.authorization",
            detail: "the selection's authorization is not currently valid \
                     (restored grants never admit; re-authorize per run)"
                .into(),
        });
    }
    if selection.pinned_sha256 != on_disk_sha256 {
        return Err(SelectionRejection {
            code: "m2build.selection.pin",
            detail: format!(
                "precise pin {} != on-disk {} — an intact required subset does NOT \
                 auto-upgrade a pin; update the selection explicitly and re-admit",
                selection.pinned_sha256, on_disk_sha256
            ),
        });
    }
    if selection.artifacts.is_empty() {
        return Err(SelectionRejection {
            code: "m2build.selection.artifact-missing",
            detail: "the selection declares no artifacts".into(),
        });
    }
    let primary = selection
        .artifacts
        .iter()
        .find(|a| a.sha256 == on_disk_sha256);
    let Some(primary) = primary else {
        return Err(SelectionRejection {
            code: "m2build.selection.artifact-missing",
            detail: format!("no declared artifact carries the on-disk hash {on_disk_sha256}"),
        });
    };
    if !required_subset_intact {
        return Err(SelectionRejection {
            code: "m2build.selection.required-subset",
            detail: "the provider no longer offers the consumer's required subset".into(),
        });
    }
    Ok(AdmissionReceipt {
        provider_id: selection.provider_id.clone(),
        generation: selection.generation,
        admitted_sha256: primary.sha256.clone(),
    })
}
