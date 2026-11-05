//! Backend-neutral ordinary-source transaction. The compiler ports, query
//! solver, catalog and single-body emitter are shared with the product entry.
//! A restored session retains declaration metadata and required fact pages;
//! unchanged ASTs remain unmaterialized, including on its very first edit.
use crate::{
    BodyTypeSupply, BodyWalk, FnSigFacts, InferBodyFacts, LetDecl, NodeId, NodeInfo, PortableTy,
    QueryKey, SignatureCatalog, SourceQuerySession,
};
use ace_core::{
    atom_text::{Diagnostic, Span},
    source::{
        adapter::{prepare_candidate_document_with_supply, FnCandidate},
        ast::{Item, Program},
        cst::{
            units::{self, StagedCandidate, UnitCatalog},
            FrontBudget, FRONT_CAPS_VERSION,
        },
        decl_env,
        resolve::{self, CheckedEnv},
        SourceParts, SourceProfile,
    },
    verify::{CheckedFunction, CheckedModule},
};
use ace_foreign_contract::manual::AdmittedForeign;
use ace_semantics_contract::ContentVersion;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub const SCHEMA: &str = "ac-source-facts-session/1";
pub const MAX_ROOT_BYTES: usize = 8 << 20;
pub const MAX_PAGE_BYTES: usize = 1 << 20;
pub const MAX_SOURCE_BYTES: usize = 8 << 20;
const MAX_FUNCTIONS: usize = 8192;

#[derive(Debug)]
pub enum Error {
    Diagnostics(Vec<Diagnostic>),
    Reject(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diagnostics(ds) => write!(f, "source: {ds:?}"),
            Self::Reject(s) => f.write_str(s),
        }
    }
}
impl std::error::Error for Error {}
fn reject(e: impl std::fmt::Debug) -> Error {
    Error::Reject(format!("source.frontend: {e:?}"))
}
fn syntax_error(e: units::EditError) -> Error {
    match e {
        units::EditError::Diag(ds) => Error::Diagnostics(ds),
        other => reject(other),
    }
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn caps_key(caps: &[String]) -> String {
    let mut caps = caps.to_vec();
    caps.sort();
    caps.dedup();
    crate::sq_hash(
        "source-facts/capabilities/1",
        &caps.iter().map(|c| c.as_bytes()).collect::<Vec<_>>(),
    )
}

/// Real entries to production ports, distinct from restore validation. A
/// failure still contributes its attempted work. Target counters belong to
/// the selected backend, not to this backend-independent frontend.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Meter {
    pub whole_parse_attempts: u64,
    pub body_parse_attempts: u64,
    pub body_parse_produced: u64,
    pub hir_attempts: u64,
    pub hir_produced: u64,
    pub page_encode_attempts: u64,
    pub page_produced: u64,
    pub restore_pages: u64,
    pub restore_nodes: u64,
}

/// Bytes already encoded at production time. The host can persist new pages
/// through the original ManagedStore and keep unchanged page handles without
/// re-reading or re-encoding their payloads.
#[derive(Clone)]
pub struct Page {
    pub digest: String,
    pub bytes: Arc<Vec<u8>>,
}
impl Page {
    fn new(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() > MAX_PAGE_BYTES {
            return Err(reject("fact page budget"));
        }
        Ok(Self {
            digest: sha(&bytes),
            bytes: Arc::new(bytes),
        })
    }
}

/// A checkpoint root contains metadata and REQUIRED content edges only.
/// Expected root digest comes from the accepted host root, never this payload.
pub struct Checkpoint {
    pub root: Vec<u8>,
    pub digest: String,
    pub pages: BTreeMap<String, Page>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ColdObservation {
    pub whole_parse_attempts: u64,
    pub queries: crate::Counters,
}

struct Resident {
    page: Page,
    mint_origin: usize,
}

pub struct Frontend {
    catalog: UnitCatalog,
    env: CheckedEnv,
    signatures: SignatureCatalog,
    queries: SourceQuerySession,
    foreign: AdmittedForeign,
    capabilities: Vec<String>,
    package: String,
    module: String,
    residents: BTreeMap<String, Resident>,
    meter: Meter,
    per_function: BTreeMap<String, Meter>,
}

/// Prepared HIR + an opaque core transaction. No source/environment/current
/// artifact changes until the host has prepared and validated its target.
pub struct Candidate {
    syntax: StagedCandidate,
    semantic: FnCandidate,
    checked: CheckedFunction,
    resident: Resident,
}
impl Candidate {
    pub fn parts(&self) -> &SourceParts {
        self.semantic.parts()
    }
    pub fn checked(&self) -> &CheckedFunction {
        &self.checked
    }
    pub fn owner(&self) -> &str {
        self.semantic.slot_name()
    }
    pub fn source(&self) -> &str {
        self.syntax.new_src()
    }
    pub fn page(&self) -> &Page {
        &self.resident.page
    }
}

impl Frontend {
    pub fn cold(
        source: &str,
        module: &str,
        package: &str,
        capabilities: &[String],
        foreign: AdmittedForeign,
    ) -> Result<(Self, CheckedModule), Error> {
        Self::cold_observed(
            source,
            module,
            package,
            capabilities,
            foreign,
            &mut ColdObservation::default(),
        )
    }
    pub fn cold_observed(
        source: &str,
        module: &str,
        package: &str,
        capabilities: &[String],
        foreign: AdmittedForeign,
        observation: &mut ColdObservation,
    ) -> Result<(Self, CheckedModule), Error> {
        if source.len() > MAX_SOURCE_BYTES {
            return Err(reject("source budget"));
        }
        observation.whole_parse_attempts += 1;
        let catalog =
            UnitCatalog::build(source, SourceProfile::Scalar, FrontBudget::WORK_OWNED_64MIB)
                .map_err(syntax_error)?;
        if catalog.fns.len() > MAX_FUNCTIONS {
            return Err(reject("function budget"));
        }
        let mut queries = SourceQuerySession::new(package, module)
            .with_capabilities(capabilities.iter().cloned());
        queries.set_foreign(Some(foreign.clone()));
        observation.whole_parse_attempts += 1;
        let product = crate::compile_scalar_product(
            source,
            module,
            capabilities,
            package,
            module,
            &mut queries,
        )
        .map_err(Error::Diagnostics);
        observation.queries = queries.counters();
        let product = product?;
        // Cold catalog already owns these syntax nodes. Preserve declaration
        // order rather than parsing the module a third time for retained env.
        let mut items: Vec<_> = catalog.fns.values().collect();
        items.sort_by_key(|s| s.span.start);
        let program = Program {
            items: items
                .iter()
                .map(|s| Item::Fn(s.item.as_ref().expect("cold catalog").as_ref().clone()))
                .collect(),
        };
        let env = resolve::resolve_env_with_supply(
            &program,
            capabilities,
            SourceProfile::Scalar,
            Some(&product.supply),
        )
        .map_err(Error::Diagnostics)?;
        let signatures = queries.catalog(&program).map_err(reject)?;
        let mut residents = BTreeMap::new();
        let mut meter = Meter {
            whole_parse_attempts: observation.whole_parse_attempts,
            ..Meter::default()
        };
        let mut per_function = BTreeMap::new();
        for s in items {
            let item = s.item.as_ref().expect("cold catalog");
            let (facts, deps) = queries
                .resident_page(&item.name.name)
                .ok_or_else(|| reject("missing cold fact"))?;
            let walk = crate::walk_body(&item.body).map_err(reject)?;
            let resident = encode_resident(
                facts,
                deps,
                &signatures,
                &walk,
                s.span.start,
                capabilities,
                &source[s.span.start..s.span.end],
            )?;
            residents.insert(item.name.name.clone(), resident);
            per_function.insert(
                item.name.name.clone(),
                Meter {
                    hir_attempts: 1,
                    hir_produced: 1,
                    page_encode_attempts: 1,
                    page_produced: 1,
                    ..Meter::default()
                },
            );
            meter.hir_attempts += 1;
            meter.hir_produced += 1;
            meter.page_encode_attempts += 1;
            meter.page_produced += 1;
        }
        Ok((
            Self {
                catalog,
                env,
                signatures,
                queries,
                foreign,
                capabilities: capabilities.to_vec(),
                package: package.into(),
                module: module.into(),
                residents,
                meter,
                per_function,
            },
            product.compiled.checked,
        ))
    }

    pub fn source(&self) -> &str {
        self.catalog.src()
    }
    pub fn module(&self) -> &str {
        &self.module
    }
    /// Locate one edited owner using the already admitted unit directory and
    /// byte replacement extent. This is source-map metadata work, never a
    /// second parser or a whole-module projection on the warm path.
    pub fn owner_replacement(&self, source: &str) -> Result<(String, String), Error> {
        if source.len() > MAX_SOURCE_BYTES || source == self.source() {
            return Err(reject("unchanged or over-budget source"));
        }
        let old = self.source().as_bytes();
        let new = source.as_bytes();
        let prefix = old.iter().zip(new).take_while(|(a, b)| a == b).count();
        let suffix = old[prefix..]
            .iter()
            .rev()
            .zip(new[prefix..].iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        let old_end = old.len() - suffix;
        let mut matches = self
            .catalog
            .fns
            .iter()
            .filter(|(_, f)| f.span.start <= prefix && f.span.end >= old_end);
        let (name, unit) = matches
            .next()
            .ok_or_else(|| reject("edit is outside one current source owner"))?;
        if matches.next().is_some() {
            return Err(reject("ambiguous owner edit"));
        }
        let end = unit
            .span
            .end
            .checked_add_signed(source.len() as isize - old.len() as isize)
            .ok_or_else(|| reject("edit span overflow"))?;
        let declaration = source
            .get(unit.span.start..end)
            .ok_or_else(|| reject("edit UTF-8 boundary"))?;
        if self.source().as_bytes()[..unit.span.start] != new[..unit.span.start]
            || self.source().as_bytes()[unit.span.end..] != new[end..]
        {
            return Err(reject("edit crosses source owners"));
        }
        Ok((name.clone(), declaration.into()))
    }
    pub fn foreign(&self) -> &AdmittedForeign {
        &self.foreign
    }
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
    /// A separately re-admitted provider implementation can change the link
    /// binding without changing any declaration/query/body artifact.
    pub fn rebind_provider(&mut self, foreign: AdmittedForeign) -> Result<(), Error> {
        if foreign.source_declarations() != self.foreign.source_declarations() {
            return Err(reject("provider rebind changed declarations"));
        }
        self.queries.set_foreign(Some(foreign.clone()));
        self.foreign = foreign;
        Ok(())
    }
    pub fn meter(&self) -> Meter {
        self.meter
    }
    pub fn function_meter(&self, name: &str) -> Meter {
        self.per_function.get(name).copied().unwrap_or_default()
    }
    pub fn query_counters(&self) -> crate::Counters {
        self.queries.counters()
    }
    pub fn page(&self, name: &str) -> Option<&Page> {
        self.residents.get(name).map(|r| &r.page)
    }
    pub fn function_count(&self) -> usize {
        self.residents.len()
    }
    pub fn body_identity(&self, name: &str) -> Option<usize> {
        resolve::env_free_fn_index(&self.env, name)
            .and_then(|i| resolve::env_fn_body_identity(&self.env, i))
    }

    pub fn prepare(&mut self, owner: &str, declaration: &str) -> Result<Candidate, Error> {
        let old = self
            .catalog
            .fns
            .get(owner)
            .ok_or_else(|| reject("unknown function"))?;
        if declaration == &self.source()[old.span.start..old.span.end] {
            return Err(reject("unchanged edit"));
        }
        if self
            .source()
            .len()
            .saturating_sub(old.span.end - old.span.start)
            .saturating_add(declaration.len())
            > MAX_SOURCE_BYTES
        {
            return Err(reject("source budget"));
        }
        self.meter.body_parse_attempts += 1;
        self.per_function
            .entry(owner.into())
            .or_default()
            .body_parse_attempts += 1;
        let syntax = self
            .catalog
            .stage_owner_edit(owner, declaration, FrontBudget::WORK_OWNED_64MIB)
            .map_err(syntax_error)?;
        self.meter.body_parse_produced += syntax.produced() as u64;
        self.per_function
            .entry(owner.into())
            .or_default()
            .body_parse_produced += syntax.produced() as u64;
        let item = syntax
            .owner_item()
            .ok_or_else(|| reject("changed syntax not materialized"))?;
        if item.name.name != owner
            || self.signatures.version(owner) != Some(&crate::signature_version(item))
        {
            return Err(reject("body edit changed declaration signature"));
        }
        let answer = self
            .queries
            .infer_item_with_catalog(item, &self.signatures)
            .map_err(reject)?;
        let mut supply = BodyTypeSupply::new().with_foreign(self.foreign.clone());
        supply.add_body(&item.body, &answer.facts).map_err(reject)?;
        let slot = resolve::env_free_fn_index(&self.env, owner)
            .ok_or_else(|| reject("unknown semantic slot"))?;
        self.meter.hir_attempts += 1;
        self.per_function
            .entry(owner.into())
            .or_default()
            .hir_attempts += 1;
        let semantic = prepare_candidate_document_with_supply(
            &mut self.env,
            item,
            slot,
            syntax.new_src(),
            &self.module,
            Some(&supply),
        )
        .map_err(Error::Diagnostics)?;
        let checked = ace_core::verify::verify_foreign_function(
            &semantic.parts().bundle,
            slot,
            &self.foreign,
        )
        .map_err(Error::Diagnostics)?;
        self.meter.hir_produced += semantic.receipt().real_body_plans as u64;
        self.per_function
            .entry(owner.into())
            .or_default()
            .hir_produced += semantic.receipt().real_body_plans as u64;
        let walk = crate::walk_body(&item.body).map_err(reject)?;
        let (_, deps) = self
            .queries
            .resident_page(owner)
            .ok_or_else(|| reject("missing candidate facts"))?;
        self.meter.page_encode_attempts += 1;
        self.per_function
            .entry(owner.into())
            .or_default()
            .page_encode_attempts += 1;
        let resident = encode_resident(
            &answer.facts,
            deps,
            &self.signatures,
            &walk,
            item.span.start,
            &self.capabilities,
            declaration,
        )?;
        self.meter.page_produced += 1;
        self.per_function
            .entry(owner.into())
            .or_default()
            .page_produced += 1;
        Ok(Candidate {
            syntax,
            semantic,
            checked,
            resident,
        })
    }

    pub fn validate(&self, candidate: &Candidate) -> Result<(), Error> {
        self.catalog
            .validate_staged(&candidate.syntax)
            .map_err(reject)?;
        candidate
            .semantic
            .validate(&self.env)
            .map_err(Error::Diagnostics)
    }

    /// Exclusive validation covers BOTH subtransactions before either writes.
    /// Their commits have no fallible producer or I/O after these checks.
    pub fn commit(&mut self, candidate: Candidate) -> Result<(), Error> {
        self.validate(&candidate)?;
        let name = candidate.owner().to_string();
        candidate
            .semantic
            .commit(&mut self.env)
            .map_err(Error::Diagnostics)?;
        self.catalog
            .commit_staged(candidate.syntax)
            .map_err(reject)?;
        self.residents.insert(name, candidate.resident);
        Ok(())
    }

    /// Only root/declaration/source-map METADATA is encoded here. Resident
    /// body pages retain their exact Arc and digest across unrelated edits.
    pub fn checkpoint(&self) -> Result<Checkpoint, Error> {
        let source_sha = sha(self.source().as_bytes());
        let mut env =
            decl_env::decl_env_dump(&self.env, &caps_key(&self.capabilities), &source_sha);
        for f in &mut env.fns {
            let unit = self
                .catalog
                .fns
                .get(&f.name)
                .ok_or_else(|| reject("env/catalog disagreement"))?;
            let delta = unit.span.start as i64 - f.item_span.0 as i64;
            f.name_span = (shift(f.name_span.0, delta)?, shift(f.name_span.1, delta)?);
            f.item_span = (unit.span.start, unit.span.end);
        }
        let signatures = self.signatures.entries().map(SigRow::from).collect();
        let edges = self
            .residents
            .iter()
            .map(|(name, r)| {
                (
                    name.clone(),
                    Edge {
                        digest: r.page.digest.clone(),
                        mint_origin: r.mint_origin,
                    },
                )
            })
            .collect();
        let wire = Root {
            schema: SCHEMA.into(),
            package: self.package.clone(),
            module: self.module.clone(),
            source: self.source().into(),
            source_sha,
            capabilities: self.capabilities.clone(),
            catalog: units::encode_manifest(&self.catalog.to_manifest(FRONT_CAPS_VERSION)),
            declarations: decl_env::encode_decl_env(&env),
            signatures,
            edges,
        };
        let root = serde_json::to_vec(&wire).map_err(reject)?;
        if root.len() > MAX_ROOT_BYTES {
            return Err(reject("root budget"));
        }
        let digest = sha(&root);
        let pages = self
            .residents
            .iter()
            .map(|(n, r)| (n.clone(), r.page.clone()))
            .collect();
        Ok(Checkpoint {
            root,
            digest,
            pages,
        })
    }

    /// R-only admission. No parse/solve/bind/emit/encode or target/tool calls.
    /// Source authorization is the independently expected accepted root hash.
    pub fn restore(
        root: &[u8],
        expected_digest: &str,
        capabilities: &[String],
        foreign: AdmittedForeign,
        mut read: impl FnMut(&str) -> Option<Arc<Vec<u8>>>,
    ) -> Result<Self, Error> {
        if root.len() > MAX_ROOT_BYTES || sha(root) != expected_digest {
            return Err(reject("root identity or budget"));
        }
        let root: Root = serde_json::from_slice(root).map_err(reject)?;
        if root.schema != SCHEMA
            || root.source.len() > MAX_SOURCE_BYTES
            || sha(root.source.as_bytes()) != root.source_sha
            || root.capabilities != capabilities
            || root.edges.len() > MAX_FUNCTIONS
        {
            return Err(reject("root envelope"));
        }
        let manifest = units::decode_manifest(&root.catalog).map_err(reject)?;
        let catalog = UnitCatalog::from_restored(
            &root.source,
            &manifest,
            SourceProfile::Scalar,
            FRONT_CAPS_VERSION,
        )
        .map_err(reject)?;
        let declarations = decl_env::decode_decl_env(&root.declarations).map_err(reject)?;
        if !declarations.records.is_empty()
            || !declarations.lists.is_empty()
            || !declarations.variants.is_empty()
            || !declarations.function_types.is_empty()
        {
            return Err(reject("foreign scalar declaration profile"));
        }
        let env = decl_env::decl_env_restore_with_foreign(
            &declarations,
            SourceProfile::Scalar,
            &caps_key(capabilities),
            &root.source_sha,
            Some(&foreign),
        )
        .map_err(reject)?;
        let signatures = SignatureCatalog::from_entries(
            &root.package,
            &root.module,
            root.signatures
                .iter()
                .map(|s| s.decode(&root.package, &root.module))
                .collect::<Result<_, _>>()?,
        )
        .map_err(reject)?;
        validate_signatures(&signatures, &declarations, &foreign)?;
        if root.edges.len() != catalog.fns.len() || declarations.fns.len() != catalog.fns.len() {
            return Err(reject("required coverage"));
        }
        let mut declared = BTreeSet::new();
        for f in &declarations.fns {
            let unit = catalog
                .fns
                .get(&f.name)
                .ok_or_else(|| reject("declaration owner missing"))?;
            if !declared.insert(&f.name)
                || f.item_span != (unit.span.start, unit.span.end)
                || f.name_span.0 < unit.span.start
                || f.name_span.1 > unit.span.end
                || root.source.get(f.name_span.0..f.name_span.1) != Some(f.name.as_str())
            {
                return Err(reject("declaration source-map binding"));
            }
        }
        let mut queries = SourceQuerySession::new(&root.package, &root.module)
            .with_capabilities(capabilities.iter().cloned());
        queries.set_foreign(Some(foreign.clone()));
        let mut residents = BTreeMap::new();
        let mut meter = Meter::default();
        for (name, edge) in root.edges {
            let unit = catalog
                .fns
                .get(&name)
                .ok_or_else(|| reject("foreign required owner"))?;
            let bytes = read(&edge.digest).ok_or_else(|| reject("missing required fact page"))?;
            if bytes.len() > MAX_PAGE_BYTES || sha(&bytes) != edge.digest {
                return Err(reject("required fact content identity"));
            }
            let wire: FactWire = serde_json::from_slice(&bytes).map_err(reject)?;
            if wire.schema != SCHEMA
                || wire.name != name
                || wire.origin != edge.mint_origin
                || wire.capabilities != caps_key(capabilities)
                || wire.unit_sha256 != sha(&root.source.as_bytes()[unit.span.start..unit.span.end])
            {
                return Err(reject("required fact envelope"));
            }
            let (key, facts) = crate::decode_page(&wire.facts).map_err(reject)?;
            if facts.negatives.iter().any(|n| n.domain == "fn")
                && wire.candidates != signatures.candidates_fp()
            {
                return Err(reject("required negative dependency changed"));
            }
            let sig = signatures
                .current_version(&name)
                .ok_or_else(|| reject("missing owner signature"))?;
            let expected = QueryKey::infer_body(
                crate::owner_logical(&root.package, &root.module, &name),
                sig,
                facts.body_version.clone(),
                capabilities,
            );
            if key != expected.canonical() || facts.name != name {
                return Err(reject("required fact query key"));
            }
            let walk = wire.walk.decode(
                &facts,
                unit.span,
                unit.span.start as i64 - edge.mint_origin as i64,
                &root.source,
            )?;
            let deps: Vec<_> = wire
                .dependencies
                .into_iter()
                .map(|(n, v)| (n, ContentVersion(v)))
                .collect();
            queries
                .seed_with_walk(&signatures, &facts, &walk, &deps)
                .map_err(reject)?;
            meter.restore_pages += 1;
            meter.restore_nodes += walk.nodes.len() as u64;
            residents.insert(
                name,
                Resident {
                    page: Page {
                        digest: edge.digest,
                        bytes,
                    },
                    mint_origin: edge.mint_origin,
                },
            );
        }
        Ok(Self {
            catalog,
            env,
            signatures,
            queries,
            foreign,
            capabilities: capabilities.to_vec(),
            package: root.package,
            module: root.module,
            residents,
            meter,
            per_function: BTreeMap::new(),
        })
    }
}

fn shift(n: usize, delta: i64) -> Result<usize, Error> {
    usize::try_from(n as i128 + delta as i128).map_err(reject)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Root {
    schema: String,
    package: String,
    module: String,
    source: String,
    source_sha: String,
    capabilities: Vec<String>,
    catalog: String,
    declarations: String,
    signatures: Vec<SigRow>,
    edges: BTreeMap<String, Edge>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Edge {
    digest: String,
    mint_origin: usize,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SigRow {
    name: String,
    version: String,
    params: Vec<(String, String)>,
    ret: String,
}
impl From<&FnSigFacts> for SigRow {
    fn from(s: &FnSigFacts) -> Self {
        Self {
            name: s.name.clone(),
            version: s.version.as_str().into(),
            params: s
                .params
                .iter()
                .map(|(n, t)| (n.clone(), t.as_str().into()))
                .collect(),
            ret: s.ret.as_str().into(),
        }
    }
}
fn ty(s: &str) -> Result<PortableTy, Error> {
    match s {
        "i32" => Ok(PortableTy::I32),
        "bool" => Ok(PortableTy::Bool),
        "void" => Ok(PortableTy::Unit),
        _ => Err(reject("unknown portable type")),
    }
}
impl SigRow {
    fn decode(&self, package: &str, module: &str) -> Result<FnSigFacts, Error> {
        if self.version.len() != 64 || !self.version.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(reject("signature version"));
        }
        Ok(FnSigFacts {
            name: self.name.clone(),
            logical: crate::owner_logical(package, module, &self.name),
            version: ContentVersion(self.version.clone()),
            params: self
                .params
                .iter()
                .map(|(n, t)| Ok((n.clone(), ty(t)?)))
                .collect::<Result<_, Error>>()?,
            ret: ty(&self.ret)?,
            profile: crate::SQ_PROFILE,
            schema: crate::SQ_SCHEMA,
            producer: crate::SQ_PRODUCER,
        })
    }
}
fn validate_signatures(
    catalog: &SignatureCatalog,
    decls: &decl_env::DeclEnvDump,
    foreign: &AdmittedForeign,
) -> Result<(), Error> {
    let mut expected = BTreeSet::new();
    for f in &decls.fns {
        let sig = catalog
            .get(&f.name)
            .ok_or_else(|| reject("missing local signature"))?;
        let params: Vec<_> = f
            .param_names
            .iter()
            .cloned()
            .zip(f.params.iter().cloned())
            .collect();
        let row = SigRow::from(sig);
        let result = if f.result == "unit" {
            "void"
        } else {
            &f.result
        };
        if row.params != params
            || row.ret != result
            || f.param_modes.iter().any(|m| *m)
            || f.owner_rec.is_some()
            || !expected.insert(f.name.clone())
        {
            return Err(reject("local signature mismatch"));
        }
    }
    // Declaration-only extraction uses the same catalog builder and no bodies.
    let foreign_catalog = SignatureCatalog::build_with_foreign(
        &Program { items: vec![] },
        catalog.package(),
        catalog.module(),
        Some(foreign),
    )
    .map_err(reject)?;
    for sig in foreign_catalog.entries() {
        if catalog.get(&sig.name) != Some(sig) || !expected.insert(sig.name.clone()) {
            return Err(reject("foreign signature mismatch"));
        }
    }
    if expected.len() != catalog.len() {
        return Err(reject("extra restored signature"));
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FactWire {
    schema: String,
    name: String,
    origin: usize,
    unit_sha256: String,
    capabilities: String,
    candidates: String,
    dependencies: Vec<(String, String)>,
    facts: Vec<u8>,
    walk: WalkWire,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WalkWire {
    nodes: Vec<(u32, String, usize, usize)>,
    lets: Vec<(u32, String)>,
}
impl WalkWire {
    fn from_walk(w: &BodyWalk) -> Self {
        Self {
            nodes: w
                .nodes
                .iter()
                .map(|n| (n.id.0, n.kind.into(), n.span.start, n.span.end))
                .collect(),
            lets: w.lets.iter().map(|l| (l.node.0, l.name.clone())).collect(),
        }
    }
    fn decode(
        &self,
        facts: &InferBodyFacts,
        owner: Span,
        delta: i64,
        source: &str,
    ) -> Result<BodyWalk, Error> {
        if self.nodes.is_empty() || self.nodes.len() > crate::walk::MAX_BODY_NODES {
            return Err(reject("walk count"));
        }
        let mut nodes = Vec::with_capacity(self.nodes.len());
        for (i, (id, kind, start, end)) in self.nodes.iter().enumerate() {
            let start = shift(*start, delta)?;
            let end = shift(*end, delta)?;
            if *id as usize != i
                || start > end
                || start < owner.start
                || end > owner.end
                || !source.is_char_boundary(start)
                || !source.is_char_boundary(end)
            {
                return Err(reject("walk ids/spans"));
            }
            let kind = match kind.as_str() {
                "block" => "block",
                "expr.binary" => "expr.binary",
                "expr.bool" => "expr.bool",
                "expr.call" => "expr.call",
                "expr.int" => "expr.int",
                "expr.unary" => "expr.unary",
                "expr.var" => "expr.var",
                "stmt.assign" => "stmt.assign",
                "stmt.break" => "stmt.break",
                "stmt.continue" => "stmt.continue",
                "stmt.expr" => "stmt.expr",
                "stmt.if" => "stmt.if",
                "stmt.let" => "stmt.let",
                "stmt.loop" => "stmt.loop",
                "stmt.return" => "stmt.return",
                "stmt.while" => "stmt.while",
                _ => return Err(reject("unknown walk kind")),
            };
            nodes.push(NodeInfo {
                id: NodeId(*id),
                kind,
                span: Span { start, end },
            });
        }
        let mut seen = BTreeSet::new();
        let lets = self
            .lets
            .iter()
            .map(|(id, name)| {
                if !seen.insert(*id)
                    || nodes.get(*id as usize).map(|n| n.kind) != Some("stmt.let")
                    || name.is_empty()
                {
                    return Err(reject("walk let binding"));
                }
                Ok(LetDecl {
                    node: NodeId(*id),
                    name: name.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if nodes.iter().filter(|n| n.kind == "stmt.let").count() != lets.len() {
            return Err(reject("walk let coverage"));
        }
        let expr_nodes = nodes.iter().filter(|n| n.is_expr()).count();
        Ok(BodyWalk {
            nodes,
            lets,
            version: facts.body_version.clone(),
            expr_nodes,
        })
    }
}
fn encode_resident(
    facts: &InferBodyFacts,
    deps: &[(String, ContentVersion)],
    catalog: &SignatureCatalog,
    walk: &BodyWalk,
    origin: usize,
    capabilities: &[String],
    unit_source: &str,
) -> Result<Resident, Error> {
    facts.admit(walk).map_err(reject)?;
    let key = QueryKey::infer_body(
        facts.logical.clone(),
        catalog
            .current_version(&facts.name)
            .ok_or_else(|| reject("missing owner signature"))?,
        facts.body_version.clone(),
        capabilities,
    );
    let payload = crate::encode_page(&key, facts).map_err(reject)?;
    let wire = FactWire {
        schema: SCHEMA.into(),
        name: facts.name.clone(),
        origin,
        unit_sha256: sha(unit_source.as_bytes()),
        capabilities: caps_key(capabilities),
        candidates: catalog.candidates_fp().into(),
        dependencies: deps
            .iter()
            .map(|(n, v)| (n.clone(), v.as_str().into()))
            .collect(),
        facts: payload,
        walk: WalkWire::from_walk(walk),
    };
    Ok(Resident {
        page: Page::new(serde_json::to_vec(&wire).map_err(reject)?)?,
        mint_origin: origin,
    })
}
