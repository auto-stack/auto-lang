//! PLAN-938: untrusted manual source declarations and current host admission.
//!
//! A manifest is never authority. The compilation host supplies an independently
//! reviewed contract digest and CURRENT provider/header identities. No addresses
//! or persisted safety credentials occur in this model.
use crate::{FnForm, InterfaceFacts, LaneKind, PhysicalPlan};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MANUAL_SCHEMA: &str = "ace.foreign.manual/1";
pub const SOURCE_CAP: &str = "foreign.scalar";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManualDecl {
    pub source_name: String,
    pub symbol: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManualManifest {
    pub schema: String,
    pub review_id: String,
    pub provider: String,
    pub header_digest: String,
    pub physical_table_digest: String,
    pub target: String,
    pub profile: String,
    pub callconv: String,
    pub direction: String,
    pub facts: InterfaceFacts,
    pub declarations: Vec<ManualDecl>,
}

/// Process-local host input. This must come from the compilation host's current
/// review registry and loaded provider, not from a field beside the manifest.
#[derive(Clone, Debug)]
pub struct HostAuthority {
    pub review_id: String,
    pub contract_digest: String,
    pub provider: String,
    pub provider_digest: String,
    pub header_digest: String,
    pub physical_table_digest: String,
    pub target: String,
    pub capabilities: BTreeSet<String>,
    pub available_symbols: BTreeSet<String>,
    pub binding_generation: u64,
}

/// Intentionally not serializable. Construction requires current host admission.
#[derive(Clone, Debug)]
pub struct AdmittedForeign {
    manifest: ManualManifest,
    plan: PhysicalPlan,
    contract_digest: String,
    provider_digest: String,
    generation: u64,
}

fn ident(s: &str) -> bool {
    let mut bytes = s.bytes();
    matches!(bytes.next(), Some(b) if b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

impl ManualManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != MANUAL_SCHEMA {
            return Err("foreign.schema: unknown manual declaration version".into());
        }
        if self.target != crate::plan::TARGET
            || self.profile != crate::plan::PROFILE
            || self.callconv != "windows-x64"
            || self.direction != "source-import"
        {
            return Err("foreign.abi: unsupported target/profile/callconv/direction".into());
        }
        if self.review_id.is_empty()
            || self.provider.is_empty()
            || !digest(&self.header_digest)
            || !digest(&self.physical_table_digest)
        {
            return Err("foreign.source: missing reviewed provenance".into());
        }
        self.facts.validate()?;
        if self.declarations.is_empty() || self.declarations.len() != self.facts.functions.len() {
            return Err(
                "foreign.selection: declarations must cover every function exactly once".into(),
            );
        }
        let mut names = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        for d in &self.declarations {
            if !ident(&d.source_name)
                || !ident(&d.symbol)
                || !names.insert(&d.source_name)
                || !symbols.insert(&d.symbol)
            {
                return Err("foreign.name: invalid or duplicate name/symbol".into());
            }
            let f = self
                .facts
                .function(&d.symbol)
                .ok_or("foreign.symbol: declaration has no logical facts")?;
            let scalar = |k: &LaneKind| matches!(k, LaneKind::I32 | LaneKind::Bool);
            let supported = match &f.form {
                FnForm::Pure { params, ret } => params.iter().all(scalar) && scalar(ret),
                FnForm::Fallible { params, outs, .. } => {
                    params.iter().all(scalar) && outs.len() == 1 && outs.iter().all(scalar)
                }
            };
            if !supported {
                return Err("foreign.signature: source requires i32/bool and one result".into());
            }
        }
        Ok(())
    }

    /// Contract identity excludes provider implementation bytes. A body-only
    /// provider edit changes the link/binding root, never the caller signature.
    pub fn contract_digest(&self) -> Result<String, String> {
        self.validate()?;
        let mut canonical = self.clone();
        canonical
            .declarations
            .sort_by(|a, b| a.source_name.cmp(&b.source_name));
        canonical.facts =
            serde_json::from_str(&self.facts.canonical()?).map_err(|e| e.to_string())?;
        Ok(crate::blake3_hex(
            &serde_json::to_vec(&canonical).map_err(|e| e.to_string())?,
        ))
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > crate::codec::MAX_CODEC_BYTES {
            return Err("foreign.budget: manual manifest exceeds decode budget".into());
        }
        let manifest: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("foreign.decode: {e}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn admit(self, host: &HostAuthority) -> Result<AdmittedForeign, String> {
        let contract_digest = self.contract_digest()?;
        if !host.capabilities.contains(SOURCE_CAP)
            || self
                .facts
                .capabilities
                .iter()
                .any(|c| !host.capabilities.contains(c))
        {
            return Err("foreign.capability: current host denies source FFI".into());
        }
        if host.review_id != self.review_id
            || host.contract_digest != contract_digest
            || host.provider != self.provider
            || host.header_digest != self.header_digest
            || host.physical_table_digest != self.physical_table_digest
            || host.target != self.target
            || !digest(&host.provider_digest)
        {
            return Err("foreign.authority: current review/provider binding does not match".into());
        }
        if self
            .declarations
            .iter()
            .any(|d| !host.available_symbols.contains(&d.symbol))
        {
            return Err("foreign.symbol: provider lacks a reviewed symbol".into());
        }
        let plan = PhysicalPlan::project(&self.facts)?;
        Ok(AdmittedForeign {
            manifest: self,
            plan,
            contract_digest,
            provider_digest: host.provider_digest.clone(),
            generation: host.binding_generation,
        })
    }
}

impl AdmittedForeign {
    pub fn manifest(&self) -> &ManualManifest {
        &self.manifest
    }
    pub fn plan(&self) -> &PhysicalPlan {
        &self.plan
    }
    pub fn contract_digest(&self) -> &str {
        &self.contract_digest
    }
    pub fn provider_digest(&self) -> &str {
        &self.provider_digest
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn declarations(&self) -> BTreeMap<&str, &crate::FnFact> {
        self.manifest
            .declarations
            .iter()
            .map(|d| {
                (
                    d.source_name.as_str(),
                    self.manifest
                        .facts
                        .function(&d.symbol)
                        .expect("admitted symbol"),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FactsSource, FnFact, FACTS_FORMAT};

    fn fixture() -> (ManualManifest, HostAuthority) {
        let m = ManualManifest {
            schema: MANUAL_SCHEMA.into(),
            review_id: "review/c-provider-v1".into(),
            provider: "independent-c".into(),
            header_digest: "a".repeat(64),
            physical_table_digest: "b".repeat(64),
            target: crate::plan::TARGET.into(),
            profile: crate::plan::PROFILE.into(),
            callconv: "windows-x64".into(),
            direction: "source-import".into(),
            facts: InterfaceFacts {
                format: FACTS_FORMAT.into(),
                logical: "c/provider".into(),
                content_version: "v1".into(),
                source: FactsSource::Typed {
                    id: "manual/review/c-provider-v1".into(),
                },
                functions: vec![FnFact {
                    symbol: "c_add".into(),
                    form: FnForm::Pure {
                        params: vec![LaneKind::I32, LaneKind::I32],
                        ret: LaneKind::I32,
                    },
                    error_model: None,
                    semantic_ref: None,
                }],
                effects: vec!["pure:c_add".into()],
                capabilities: vec![SOURCE_CAP.into()],
            },
            declarations: vec![ManualDecl {
                source_name: "foreign_add".into(),
                symbol: "c_add".into(),
            }],
        };
        let host = HostAuthority {
            review_id: m.review_id.clone(),
            contract_digest: m.contract_digest().unwrap(),
            provider: m.provider.clone(),
            provider_digest: "c".repeat(64),
            header_digest: m.header_digest.clone(),
            physical_table_digest: m.physical_table_digest.clone(),
            target: m.target.clone(),
            capabilities: [SOURCE_CAP.into()].into_iter().collect(),
            available_symbols: ["c_add".into()].into_iter().collect(),
            binding_generation: 7,
        };
        (m, host)
    }

    #[test]
    fn current_authority_is_required_and_cannot_be_restored_from_json() {
        let (m, host) = fixture();
        let bytes = serde_json::to_vec(&m).unwrap();
        let decoded = ManualManifest::decode(&bytes).unwrap();
        assert_eq!(decoded.clone().admit(&host).unwrap().generation(), 7);
        let mut revoked = host.clone();
        revoked.capabilities.clear();
        assert!(decoded
            .admit(&revoked)
            .unwrap_err()
            .contains("foreign.capability"));
        let mut forged: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        forged["safe"] = true.into();
        assert!(ManualManifest::decode(&serde_json::to_vec(&forged).unwrap()).is_err());
    }

    #[test]
    fn provider_body_changes_binding_not_contract() {
        let (m, host) = fixture();
        let old = m.clone().admit(&host).unwrap();
        let mut edited = host.clone();
        edited.provider_digest = "d".repeat(64);
        edited.binding_generation += 1;
        let new = m.admit(&edited).unwrap();
        assert_eq!(old.contract_digest(), new.contract_digest());
        assert_eq!(old.plan(), new.plan());
        assert_ne!(old.provider_digest(), new.provider_digest());
        assert_ne!(old.generation(), new.generation());
    }

    #[test]
    fn rejects_wrong_review_provenance_target_and_missing_symbol() {
        let (m, host) = fixture();
        for axis in 0..7 {
            let mut h = host.clone();
            match axis {
                0 => h.review_id.push('x'),
                1 => h.contract_digest = "0".repeat(64),
                2 => h.header_digest = "0".repeat(64),
                3 => h.physical_table_digest = "0".repeat(64),
                4 => h.target = "other".into(),
                5 => h.provider.push('x'),
                _ => h.available_symbols.clear(),
            }
            assert!(m.clone().admit(&h).is_err(), "axis {axis}");
        }
        assert!(m.admit(&host).is_ok());
    }

    #[test]
    fn selection_and_source_type_boundary_are_not_typed_bridge_support() {
        let (m, _) = fixture();
        let mut empty = m.clone();
        empty.declarations.clear();
        assert!(empty.validate().unwrap_err().contains("foreign.selection"));
        let mut wide = m.clone();
        wide.facts.functions[0].form = FnForm::Pure {
            params: vec![LaneKind::I64],
            ret: LaneKind::Bool,
        };
        assert!(wide.validate().unwrap_err().contains("foreign.signature"));
        let mut bad_ident = m.clone();
        bad_ident.declarations[0].source_name = "3bad".into();
        assert!(bad_ident.validate().unwrap_err().contains("foreign.name"));
        assert!(m.validate().is_ok());
    }

    #[test]
    fn strict_decode_budget_version_and_contract_tampering() {
        let (m, h) = fixture();
        let mut other = m.clone();
        other.schema = "ace.foreign.manual/0".into();
        assert!(ManualManifest::decode(&serde_json::to_vec(&other).unwrap()).is_err());
        assert!(ManualManifest::decode(&vec![b' '; crate::codec::MAX_CODEC_BYTES + 1]).is_err());
        other = m;
        other.facts.functions[0].form = FnForm::Pure {
            params: vec![LaneKind::Bool, LaneKind::I32],
            ret: LaneKind::I32,
        };
        assert!(other.admit(&h).unwrap_err().contains("foreign.authority"));
    }
}
