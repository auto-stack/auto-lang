//! Backend-free, explicit assembly inputs. These are facts, never grants or
//! executable addresses. Audited legacy references are evidence of a signature,
//! not a claim that the operation is executable in an ACE source profile.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SCHEMA: &str = "ace-stdlib-assembly/1";
pub const SCALAR_MODULES: &str = "ac.source.modules.scalar/v1";
pub const AV_PRODUCER: &str = "ace-av/modules/1";
pub const RUST_PRODUCER: &str = "ace-a2x-rust/modules/1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Signature {
    pub parameters: Vec<String>,
    pub modes: Vec<String>,
    pub returns: String,
}

/// Explicit finite declaration contract. A requirement change participates in
/// signature-read freshness, independently of the implementation body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub module: String,
    pub symbol: String,
    pub semantic_version: u32,
    pub signature: Signature,
    pub requirements: Vec<String>,
}
impl Declaration {
    pub fn version(&self) -> String {
        digest("declaration", self)
    }
}

/// Created by a producer that has compared the real public declaration with
/// its final independently registered adapter. Admission re-compares current
/// evidence; decoding this record cannot establish that comparison by itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceLevel {
    SignatureChecked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationReference {
    pub verification: ReferenceLevel,
    pub operation: String,
    pub semantic_version: u32,
    pub module: String,
    pub symbol: String,
    pub signature: Signature,
    pub requirements: Vec<String>,
    pub source_id: String,
    pub declaration_hash: String,
    pub adapter: String,
    pub producer: String,
    pub adapter_contract_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInput {
    pub source_id: String,
    pub content_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticAssembly {
    pub profile: String,
    pub host: String,
    pub platform: String,
    pub declarations: Vec<Declaration>,
    pub references: Vec<OperationReference>,
    pub sources: Vec<SourceInput>,
}

/// Physical implementation inputs belong to the consumer, not to signature
/// readers. `owners` identifies the real functions whose physical keys read
/// this input. Empty means assembly/link-only, never "invalidate everyone".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderInput {
    pub source_id: String,
    pub content_hash: String,
    pub owners: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consumer {
    pub producer: String,
    pub abi: String,
    pub runtime: String,
    pub toolchain: String,
    pub providers: Vec<ProviderInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    schema: String,
    semantic: SemanticAssembly,
    consumer: Consumer,
    fingerprint: String,
    consumer_fingerprint: String,
}

fn digest<T: Serialize>(domain: &str, data: &T) -> String {
    let mut h = blake3::Hasher::new();
    for bytes in [
        SCHEMA.as_bytes(),
        domain.as_bytes(),
        &serde_json::to_vec(data).unwrap(),
    ] {
        h.update(&(bytes.len() as u64).to_le_bytes());
        h.update(bytes);
    }
    h.finalize().to_hex().to_string()
}
fn sorted_unique<'a>(items: impl IntoIterator<Item = &'a str>) -> bool {
    let mut previous = None;
    for value in items {
        if value.is_empty() || previous.is_some_and(|p| p >= value) {
            return false;
        }
        previous = Some(value);
    }
    true
}
fn portable(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('/')
        && !id.contains([':', '\\'])
        && id.split('/').all(|s| !matches!(s, "" | "." | ".."))
}
fn hex(value: &str, len: usize) -> bool {
    value.len() == len && value.bytes().all(|b| matches!(b,b'0'..=b'9'|b'a'..=b'f'))
}
fn signature(s: &Signature) -> bool {
    s.parameters.len() == s.modes.len()
        && !s.returns.is_empty()
        && s.parameters.iter().all(|s| !s.is_empty())
        && s.modes
            .iter()
            .all(|s| matches!(s.as_str(), "View" | "Take" | "Mut"))
}
impl Manifest {
    pub fn freeze(mut semantic: SemanticAssembly, mut consumer: Consumer) -> Result<Self, String> {
        semantic
            .declarations
            .sort_by(|a, b| (&a.module, &a.symbol).cmp(&(&b.module, &b.symbol)));
        semantic
            .references
            .sort_by(|a, b| a.operation.cmp(&b.operation));
        semantic
            .sources
            .sort_by(|a, b| a.source_id.cmp(&b.source_id));
        consumer
            .providers
            .sort_by(|a, b| a.source_id.cmp(&b.source_id));
        for d in &mut semantic.declarations {
            d.requirements.sort();
        }
        for r in &mut semantic.references {
            r.requirements.sort();
        }
        for p in &mut consumer.providers {
            p.owners.sort();
        }
        let result = Self {
            schema: SCHEMA.into(),
            fingerprint: digest("common", &semantic),
            consumer_fingerprint: digest("consumer", &(&semantic, &consumer)),
            semantic,
            consumer,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn semantic(&self) -> &SemanticAssembly {
        &self.semantic
    }
    pub fn consumer(&self) -> &Consumer {
        &self.consumer
    }
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    pub fn consumer_fingerprint(&self) -> &str {
        &self.consumer_fingerprint
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err("assembly.size".into());
        }
        Ok(bytes)
    }
    /// Structural decoding is deliberately separate from freshness admission.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 4 * 1024 * 1024 {
            return Err("assembly.size".into());
        }
        let result: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("assembly.codec: {e}"))?;
        result.validate()?;
        Ok(result)
    }
    /// Expected inputs must be produced from the current source, contracts and
    /// provider closure, not copied out of the candidate receipt.
    pub fn admit(bytes: &[u8], current: &Self) -> Result<Self, String> {
        current.validate()?;
        let found = Self::decode(bytes)?;
        if &found != current {
            return Err("assembly.stale-identity".into());
        }
        Ok(found)
    }
    pub fn validate(&self) -> Result<(), String> {
        let s = &self.semantic;
        let c = &self.consumer;
        if self.schema != SCHEMA {
            return Err("assembly.schema".into());
        }
        if s.profile != SCALAR_MODULES {
            return Err("assembly.profile".into());
        }
        if !matches!(c.producer.as_str(), AV_PRODUCER | RUST_PRODUCER) {
            return Err("assembly.producer".into());
        }
        if [&s.host, &s.platform, &c.abi, &c.runtime, &c.toolchain]
            .iter()
            .any(|x| x.is_empty())
        {
            return Err("assembly.identity".into());
        }
        let mut decls = BTreeSet::new();
        for d in &s.declarations {
            if !portable(&d.module)
                || !portable(&d.symbol)
                || d.semantic_version == 0
                || !signature(&d.signature)
                || !sorted_unique(d.requirements.iter().map(String::as_str))
                || !decls.insert((&d.module, &d.symbol))
            {
                return Err("assembly.declaration".into());
            }
        }
        if !s
            .declarations
            .windows(2)
            .all(|w| (&w[0].module, &w[0].symbol) < (&w[1].module, &w[1].symbol))
        {
            return Err("assembly.order".into());
        }
        if !sorted_unique(s.references.iter().map(|x| x.operation.as_str()))
            || !sorted_unique(s.sources.iter().map(|x| x.source_id.as_str()))
            || !sorted_unique(c.providers.iter().map(|x| x.source_id.as_str()))
        {
            return Err("assembly.duplicate-or-order".into());
        }
        for r in &s.references {
            if r.semantic_version == 0
                || !portable(&r.operation)
                || !portable(&r.module)
                || !portable(&r.symbol)
                || !signature(&r.signature)
                || !portable(&r.source_id)
                || !portable(&r.producer)
                || !hex(&r.declaration_hash, 16)
                || r.adapter.is_empty()
                || !hex(&r.adapter_contract_hash, 64)
                || !sorted_unique(r.requirements.iter().map(String::as_str))
            {
                return Err("assembly.reference".into());
            }
        }
        for (id, hash) in s
            .sources
            .iter()
            .map(|p| (&p.source_id, &p.content_hash))
            .chain(c.providers.iter().map(|p| (&p.source_id, &p.content_hash)))
        {
            if !portable(id) || !hex(hash, 64) {
                return Err("assembly.input".into());
            }
        }
        for p in &c.providers {
            if !sorted_unique(p.owners.iter().map(String::as_str)) {
                return Err("assembly.provider-owner".into());
            }
        }
        if self.fingerprint != digest("common", s)
            || self.consumer_fingerprint != digest("consumer", &(s, c))
        {
            return Err("assembly.fingerprint".into());
        }
        Ok(())
    }
    pub fn provider_version(&self, owner: &str) -> String {
        let providers: Vec<_> = self
            .consumer
            .providers
            .iter()
            .filter(|p| p.owners.iter().any(|x| x == owner))
            .collect();
        digest("physical-owner", &providers)
    }
}
