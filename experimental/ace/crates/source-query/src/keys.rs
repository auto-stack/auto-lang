//! Query identity of the SQ-A layer (PLAN-934 §5.2 "完整键").
//!
//! A query key is the COMPLETE identity of one answer: the logical owner,
//! each semantic axis version, the query capability set (profile /
//! algorithm / schema / producer / target-relevant semantics) and the
//! caller's capability list. It never contains a module source hash, an
//! absolute span, an intern slot or an unrelated body/candidate domain —
//! those are provenance inputs, not identity.
//!
//! Two different spellings of the same components must never collide: the
//! canonical form is a domain-separated, length-prefixed hash over tagged
//! components, and the capability list is sorted so caller order cannot
//! change the key.

use crate::ident::{sq_hash, SQ_ALGO, SQ_PRODUCER, SQ_PROFILE, SQ_SCHEMA};
use ace_semantics_contract::{ContentVersion, LogicalId};

/// Target-relevant semantics marker. The infer facts are
/// target-INDEPENDENT (physical layout belongs to the backend); a future
/// target-coupled semantic must split this namespace explicitly instead of
/// silently reusing a key.
pub const TARGET_SEMANTICS: &str = "logical-only/v1";

/// Hash domain of a query key.
pub const KEY_DOMAIN: &str = "ac934/query-key/v1";

/// The two queries this layer answers.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum QueryKind {
    FnSig,
    InferBody,
}

impl QueryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            QueryKind::FnSig => "fn-sig",
            QueryKind::InferBody => "infer-body",
        }
    }
}

/// Complete identity of one query answer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryKey {
    pub query: QueryKind,
    pub logical: LogicalId,
    /// Signature axis of the owner (always present).
    pub sig_version: ContentVersion,
    /// Body axis (infer-body only).
    pub body_version: Option<ContentVersion>,
    pub profile: &'static str,
    pub algo: &'static str,
    pub schema: &'static str,
    pub producer: &'static str,
    pub target: &'static str,
    /// Caller capabilities, sorted and deduplicated (order-free).
    pub capabilities: Vec<String>,
}

impl QueryKey {
    pub fn fn_sig(
        logical: LogicalId,
        sig_version: ContentVersion,
        capabilities: &[String],
    ) -> Self {
        QueryKey {
            query: QueryKind::FnSig,
            logical,
            sig_version,
            body_version: None,
            profile: SQ_PROFILE,
            algo: SQ_ALGO,
            schema: SQ_SCHEMA,
            producer: SQ_PRODUCER,
            target: TARGET_SEMANTICS,
            capabilities: normalise_caps(capabilities),
        }
    }

    pub fn infer_body(
        logical: LogicalId,
        sig_version: ContentVersion,
        body_version: ContentVersion,
        capabilities: &[String],
    ) -> Self {
        QueryKey {
            query: QueryKind::InferBody,
            logical,
            sig_version,
            body_version: Some(body_version),
            profile: SQ_PROFILE,
            algo: SQ_ALGO,
            schema: SQ_SCHEMA,
            producer: SQ_PRODUCER,
            target: TARGET_SEMANTICS,
            capabilities: normalise_caps(capabilities),
        }
    }

    /// Canonical, deterministic encoding of the whole key.
    pub fn canonical(&self) -> String {
        let query = self.query.as_str().as_bytes().to_vec();
        let logical = self.logical.as_str().as_bytes().to_vec();
        let sig = self.sig_version.as_str().as_bytes().to_vec();
        let body = self
            .body_version
            .as_ref()
            .map(|v| v.as_str().as_bytes().to_vec())
            .unwrap_or_else(|| b"-".to_vec());
        let mut parts: Vec<Vec<u8>> = vec![
            query,
            logical,
            sig,
            body,
            self.profile.as_bytes().to_vec(),
            self.algo.as_bytes().to_vec(),
            self.schema.as_bytes().to_vec(),
            self.producer.as_bytes().to_vec(),
            self.target.as_bytes().to_vec(),
        ];
        for c in &self.capabilities {
            parts.push(c.as_bytes().to_vec());
        }
        let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
        sq_hash(KEY_DOMAIN, &refs)
    }
}

fn normalise_caps(caps: &[String]) -> Vec<String> {
    let mut v: Vec<String> = caps.to_vec();
    v.sort();
    v.dedup();
    v
}
