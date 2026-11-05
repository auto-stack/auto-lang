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
    pub context_symbol: Option<String>,
}

/// Untrusted per-declaration facts carried by HIR/MIR. Possession is not a
/// credential: a consumer must compare them with the current admitted host
/// environment. No provider implementation digest enters this signature key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDecl {
    pub name: String,
    pub provider: String,
    pub logical: String,
    pub content_version: String,
    pub review_id: String,
    pub header_digest: String,
    pub physical_table_digest: String,
    pub context_symbol: Option<String>,
    pub function: crate::FnFact,
    pub effects: Vec<String>,
    pub capabilities: Vec<String>,
}

impl SourceDecl {
    pub fn validate(&self) -> Result<(), String> {
        manual_facts(
            std::slice::from_ref(&self.function),
            &self.effects,
            &self.capabilities,
        )?;
        let needs_context = matches!(self.function.form, FnForm::Fallible { ctx: Some(_), .. });
        if needs_context != self.context_symbol.is_some()
            || self.context_symbol.as_ref().is_some_and(|s| !ident(s))
        {
            return Err("foreign.context: invalid target context binding".into());
        }
        if !ident(&self.name)
            || !ident(&self.function.symbol)
            || self.logical.is_empty()
            || self.content_version.is_empty()
            || self.provider.is_empty()
            || self.review_id.is_empty()
            || !digest(&self.header_digest)
            || !digest(&self.physical_table_digest)
        {
            return Err("foreign.declaration: invalid identity/provenance".into());
        }
        let scalar = |k: &LaneKind| matches!(k, LaneKind::I32 | LaneKind::Bool);
        let valid = match &self.function.form {
            FnForm::Pure { params, ret } => params.iter().all(scalar) && scalar(ret),
            FnForm::Fallible { params, outs, .. } => {
                params.iter().all(scalar) && outs.len() == 1 && outs.iter().all(scalar)
            }
        };
        if !valid {
            return Err("foreign.signature: expected finite scalar signature".into());
        }
        InterfaceFacts {
            format: crate::FACTS_FORMAT.into(),
            logical: self.logical.clone(),
            content_version: self.content_version.clone(),
            source: crate::FactsSource::Typed {
                id: self.review_id.clone(),
            },
            functions: vec![self.function.clone()],
            effects: self.effects.clone(),
            capabilities: self.capabilities.clone(),
        }
        .validate()
    }

    pub fn digest(&self) -> String {
        crate::blake3_hex(&serde_json::to_vec(self).expect("serializable declaration"))
    }

    pub fn params(&self) -> &[LaneKind] {
        match &self.function.form {
            FnForm::Pure { params, .. } | FnForm::Fallible { params, .. } => params,
        }
    }

    pub fn result(&self) -> LaneKind {
        match &self.function.form {
            FnForm::Pure { ret, .. } => *ret,
            FnForm::Fallible { outs, .. } => outs[0],
        }
    }
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
    declarations: Vec<SourceDecl>,
    declaration_digests: BTreeMap<String, String>,
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
fn manual_facts(
    functions: &[crate::FnFact],
    effects: &[String],
    capabilities: &[String],
) -> Result<(), String> {
    if !capabilities.iter().any(|c| c == SOURCE_CAP) {
        return Err(
            "foreign.capability: explicit source capability is required in the declaration".into(),
        );
    }
    if functions.iter().any(|f| f.semantic_ref.is_some()) {
        return Err("foreign.semantic-ref: this manual lane has no semantic-service authority; unresolved or persisted refs are not callable".into());
    }
    if effects
        .iter()
        .any(|e| e != "checked" && !functions.iter().any(|f| e == &format!("pure:{}", f.symbol)))
    {
        return Err(
            "foreign.effect: unknown/ownership effects are outside the manual scalar lane".into(),
        );
    }
    for f in functions {
        let required = match f.form {
            FnForm::Pure { .. } => format!("pure:{}", f.symbol),
            FnForm::Fallible { .. } => "checked".into(),
        };
        // A reviewed mixed interface can carry the module's `checked`
        // effects while each total C form remains explicitly Pure. This is
        // the existing bool/order/fallible fixture's admitted contract.
        if !effects.contains(&required)
            && !(matches!(f.form, FnForm::Pure { .. }) && effects.iter().any(|e| e == "checked"))
        {
            return Err("foreign.effect: missing explicit totality/error evidence".into());
        }
    }
    Ok(())
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
        if !matches!(&self.facts.source,crate::FactsSource::Typed{id} if !id.is_empty()) {
            return Err("foreign.source: manual imports require an explicitly reviewed typed source, never a self-claimed checked-source credential".into());
        }
        manual_facts(
            &self.facts.functions,
            &self.facts.effects,
            &self.facts.capabilities,
        )?;
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
            let needs_context = matches!(f.form, FnForm::Fallible { ctx: Some(_), .. });
            if needs_context != d.context_symbol.is_some()
                || d.context_symbol.as_ref().is_some_and(|s| !ident(s))
            {
                return Err("foreign.context: explicit target context symbol required exactly for ctx forms".into());
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
        qualify_host(&self, &contract_digest, host)?;
        let plan = PhysicalPlan::project(&self.facts)?;
        let declarations = project_source_declarations(&self);
        let declaration_digests = declarations
            .iter()
            .map(|d| (d.name.clone(), d.digest()))
            .collect();
        Ok(AdmittedForeign {
            declarations,
            declaration_digests,
            manifest: self,
            plan,
            contract_digest,
            provider_digest: host.provider_digest.clone(),
            generation: host.binding_generation,
        })
    }
}

impl AdmittedForeign {
    pub fn source_declarations(&self) -> Vec<SourceDecl> {
        self.declarations.clone()
    }
    pub fn declaration_digest(&self, name: &str) -> Option<&str> {
        self.declaration_digests.get(name).map(String::as_str)
    }
    pub fn admits_declaration(&self, declaration: &SourceDecl) -> bool {
        self.declarations
            .binary_search_by(|d| d.name.cmp(&declaration.name))
            .ok()
            .is_some_and(|index| self.declarations[index] == *declaration)
    }
    /// Revalidate current permission/registry/binding metadata. The already
    /// admitted immutable physical plan is neither reconstructed nor encoded.
    pub fn requalify(&self, host: &HostAuthority) -> Result<(), String> {
        qualify_host(&self.manifest, &self.contract_digest, host)?;
        if host.provider_digest != self.provider_digest
            || host.binding_generation != self.generation
        {
            return Err("foreign.authority.binding-changed".into());
        }
        Ok(())
    }
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

fn project_source_declarations(manifest: &ManualManifest) -> Vec<SourceDecl> {
    let mut declarations: Vec<_> = manifest
        .declarations
        .iter()
        .map(|d| {
            let mut effects: Vec<_> = manifest
                .facts
                .effects
                .iter()
                .filter(|e| !e.starts_with("pure:") || *e == &format!("pure:{}", d.symbol))
                .cloned()
                .collect();
            effects.sort();
            let mut capabilities = manifest.facts.capabilities.clone();
            capabilities.sort();
            SourceDecl {
                name: d.source_name.clone(),
                provider: manifest.provider.clone(),
                logical: format!("{}/{}", manifest.facts.logical, d.symbol),
                content_version: manifest.facts.content_version.clone(),
                review_id: manifest.review_id.clone(),
                header_digest: manifest.header_digest.clone(),
                physical_table_digest: manifest.physical_table_digest.clone(),
                context_symbol: d.context_symbol.clone(),
                function: manifest
                    .facts
                    .function(&d.symbol)
                    .expect("admitted symbol")
                    .clone(),
                effects,
                capabilities,
            }
        })
        .collect();
    declarations.sort_by(|a, b| a.name.cmp(&b.name));
    declarations
}
fn qualify_host(
    manifest: &ManualManifest,
    contract_digest: &str,
    host: &HostAuthority,
) -> Result<(), String> {
    if !host.capabilities.contains(SOURCE_CAP)
        || manifest
            .facts
            .capabilities
            .iter()
            .any(|c| !host.capabilities.contains(c))
    {
        return Err("foreign.capability: current host denies source FFI".into());
    }
    if host.review_id != manifest.review_id
        || host.contract_digest != contract_digest
        || host.provider != manifest.provider
        || host.header_digest != manifest.header_digest
        || host.physical_table_digest != manifest.physical_table_digest
        || host.target != manifest.target
        || !digest(&host.provider_digest)
    {
        return Err("foreign.authority: current review/provider binding does not match".into());
    }
    if manifest.declarations.iter().any(|d| {
        !host.available_symbols.contains(&d.symbol)
            || d.context_symbol
                .as_ref()
                .is_some_and(|s| !host.available_symbols.contains(s))
    }) {
        return Err("foreign.symbol: provider lacks a reviewed symbol".into());
    }
    Ok(())
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
                context_symbol: None,
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
    fn requalification_uses_admitted_plan_and_current_registry_metadata() {
        let (m, host) = fixture();
        let environment = m.admit(&host).unwrap();
        for declaration in environment.source_declarations() {
            assert_eq!(
                environment.declaration_digest(&declaration.name),
                Some(declaration.digest().as_str())
            );
        }
        environment.requalify(&host).unwrap();
        let mut denied = host.clone();
        denied.capabilities.clear();
        assert!(environment
            .requalify(&denied)
            .unwrap_err()
            .starts_with("foreign.capability"));
        let mut changed = host.clone();
        changed.binding_generation += 1;
        assert_eq!(
            environment.requalify(&changed).unwrap_err(),
            "foreign.authority.binding-changed"
        );
        changed = host.clone();
        changed.provider_digest = "d".repeat(64);
        assert_eq!(
            environment.requalify(&changed).unwrap_err(),
            "foreign.authority.binding-changed"
        );
        changed = host.clone();
        changed.contract_digest = "e".repeat(64);
        assert!(environment.requalify(&changed).is_err());
        changed = host.clone();
        changed.available_symbols.clear();
        assert!(environment
            .requalify(&changed)
            .unwrap_err()
            .starts_with("foreign.symbol"));
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
    #[test]
    fn manual_source_never_admits_unknown_effects_or_unqualified_semantic_services() {
        let (m, h) = fixture();
        let mut candidates = Vec::new();
        let mut x = m.clone();
        x.facts.effects.push("owns:raw-pointer".into());
        candidates.push(x);
        let mut x = m.clone();
        x.facts.effects.clear();
        candidates.push(x);
        let mut x = m.clone();
        x.facts.capabilities.clear();
        candidates.push(x);
        let mut x = m.clone();
        x.facts.functions[0].semantic_ref = Some(("wrong-service".into(), "stale-version".into()));
        candidates.push(x);
        let mut x = m.clone();
        x.facts.source = FactsSource::CheckedSource {
            module: "forged-safe".into(),
            source_sha256: "0".repeat(64),
        };
        candidates.push(x);
        for candidate in candidates {
            assert!(candidate.clone().admit(&h).is_err());
            assert!(ManualManifest::decode(&serde_json::to_vec(&candidate).unwrap()).is_err());
        }
        assert!(m.admit(&h).is_ok());
    }
}
