//! Source-bound module roots over the existing syntax and semantic page store.
//! The receipt is held by the host; a self-consistent rewritten root is not an
//! authority. Recovery reads and admits existing data, never scans or parses.
use super::*;
use crate::store::ManagedStoreSink;
use crate::store::{encode_page, SQ_LAYER};
use ace_artifacts::{
    codec::ChunkKind,
    identity::{QueryKey, RefKind},
    store::PutRequest,
};
use ace_core::runtime_contract::sha256_hex;
use ace_core::source::modules::{
    bind_module_syntax, module_grammar, project_stored_module, MODULE_BINDING_VERSION,
};
use ace_syntax_artifacts::SyntaxArtifactStore;
use ace_syntax_runtime::snapshot::{PageStore, Snapshot};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    rc::Rc,
};

pub const MODULE_ROOT_SCHEMA: &str = "source-modules-root/2";
const ROOT_KIND: &str = "source-modules-current";
const MAX_ROOT_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModuleReceipt {
    pub artifact: String,
    pub digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitRef {
    id: String,
    source_sha: String,
    pieces: Vec<PieceRef>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PieceRef {
    cst: String,
    placements: Vec<PlacementRef>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlacementRef {
    index: usize,
    mint: (usize, usize),
    current: (usize, usize),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FactRef {
    name: String,
    artifact: String,
    key: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Root {
    schema: String,
    binding: String,
    grammar: String,
    entry: String,
    package: String,
    module: String,
    capabilities: Vec<String>,
    input_hash: String,
    decl_env: String,
    targets: BTreeMap<String, ModuleReceipt>,
    units: Vec<UnitRef>,
    facts: Vec<FactRef>,
}

pub struct ModuleRecovery {
    pub bound: BoundModules,
    pub session: SourceQuerySession,
    pub env: ace_core::source::resolve::CheckedEnv,
    pub targets: BTreeMap<String, ModuleReceipt>,
}
pub struct ModuleStore {
    syntax: SyntaxArtifactStore,
    sink: ManagedStoreSink,
    project: String,
}
fn validate_declarations(
    bound: &BoundModules,
    dump: &ace_core::source::decl_env::DeclEnvDump,
) -> Result<(), String> {
    use ace_core::source::ast::{Item, ParamMode, TypeRef, TypeWord};
    let vt = |t: &ace_core::source::ast::TypeAnn| match t.kind {
        TypeRef::Builtin(TypeWord::Int | TypeWord::I32) => Some("i32"),
        TypeRef::Builtin(TypeWord::Bool) => Some("bool"),
        _ => None,
    };
    let functions: Vec<_> = bound
        .program
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some(f),
            _ => None,
        })
        .collect();
    if dump.fns.len() != functions.len()
        || !dump.records.is_empty()
        || !dump.lists.is_empty()
        || !dump.variants.is_empty()
        || !dump.function_types.is_empty()
    {
        return Err("module declaration environment shape mismatch".into());
    }
    for (d, f) in dump.fns.iter().zip(functions) {
        let params: Option<Vec<_>> = f
            .params
            .iter()
            .map(|p| vt(&p.ty).map(str::to_string))
            .collect();
        if d.name != f.name.name
            || d.param_names
                != f.params
                    .iter()
                    .map(|p| p.name.name.clone())
                    .collect::<Vec<_>>()
            || Some(d.params.clone()) != params
            || Some(d.result.as_str()) != vt(&f.ret)
            || d.param_modes
                != f.params
                    .iter()
                    .map(|p| p.mode == ParamMode::Mut)
                    .collect::<Vec<_>>()
            || d.receiver != "free"
            || d.owner_rec.is_some()
            || d.name_span != (f.name.span.start, f.name.span.end)
            || d.item_span != (f.span.start, f.span.end)
        {
            return Err(format!(
                "module declaration environment signature/span mismatch for `{}`",
                f.name.name
            ));
        }
    }
    Ok(())
}
fn canonical_caps(caps: &[String]) -> Vec<String> {
    caps.iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

impl ModuleStore {
    pub fn open(path: &Path, project: &str) -> Result<Self, String> {
        Ok(Self {
            syntax: SyntaxArtifactStore::open(path, project).map_err(|e| format!("{e:?}"))?,
            sink: ManagedStoreSink::open(path, None).map_err(|e| e.to_string())?,
            project: project.into(),
        })
    }
    /// Write a candidate closure; it is not CURRENT until publish succeeds.
    /// A caller retains the old receipt and executable throughout preparation.
    pub fn prepare(
        &mut self,
        product: &ModuleProduct,
        session: &mut SourceQuerySession,
        entry: &str,
        caps: &[String],
    ) -> Result<ModuleReceipt, String> {
        if entry != product.bound.entry
            || canonical_caps(caps) != product.bound.capabilities
            || canonical_caps(session.capabilities()) != canonical_caps(caps)
            || product.compiled.input_hash != product.bound.input_hash
        {
            return Err("module candidate identity mismatch".into());
        }
        let env = ace_core::source::resolve_env_with_supply(
            &product.bound.program,
            caps,
            SourceProfile::Scalar,
            Some(&product.supply),
        )
        .map_err(|e| format!("{e:?}"))?;
        self.prepare_context(&product.bound, session, &env)
    }
    /// Persist source/query/declaration context after a checked single-body
    /// prepare; targets remain a host-owned prepare/publish transaction.
    pub fn prepare_context(
        &mut self,
        bound: &BoundModules,
        session: &mut SourceQuerySession,
        env: &ace_core::source::resolve::CheckedEnv,
    ) -> Result<ModuleReceipt, String> {
        self.prepare_context_targets(bound, session, env, &BTreeMap::new())
    }
    pub fn prepare_context_targets(
        &mut self,
        bound: &BoundModules,
        session: &mut SourceQuerySession,
        env: &ace_core::source::resolve::CheckedEnv,
        targets: &BTreeMap<String, ModuleReceipt>,
    ) -> Result<ModuleReceipt, String> {
        let entry = &bound.entry;
        let caps = &bound.capabilities;
        if canonical_caps(session.capabilities()) != *caps {
            return Err("module candidate capabilities mismatch".into());
        }
        let grammar = module_grammar()?;
        let mut units = Vec::new();
        let mut edges = Vec::new();
        for segment in &bound.segments {
            let unit = &bound.units[&segment.module];
            let mut pieces = Vec::new();
            for piece in &unit.pieces {
                let cst = if let Some(id) = &piece.stored_cst {
                    let v = self
                        .syntax
                        .open_cst_receipt(grammar.program(), id)
                        .map_err(|e| format!("{e:?}"))?
                        .ok_or("missing retained CST origin")?;
                    if v.source_bytes().map_err(|e| format!("{e:?}"))? != piece.source.as_bytes() {
                        return Err("retained CST source mismatch".into());
                    }
                    id.clone()
                } else {
                    let tree = piece.tree.as_ref().ok_or("new syntax origin tree absent")?;
                    let tokens = piece
                        .tokens
                        .as_ref()
                        .ok_or("new syntax origin tokens absent")?;
                    let pages = Rc::new(PageStore::new());
                    let snap = Snapshot::from_text(&pages, &piece.source);
                    let a02 = self
                        .syntax
                        .put_tokens(grammar.program(), &snap, tokens)
                        .map_err(|e| format!("{e:?}"))?;
                    self.syntax
                        .put_cst_unpublished(grammar.program(), &snap, tree, &a02)
                        .map_err(|e| format!("{e:?}"))?
                };
                let mut placements = Vec::new();
                for d in &piece.placements {
                    if piece.source.get(d.mint.start..d.mint.end)
                        != segment.source.get(d.current.start..d.current.end)
                    {
                        return Err("candidate declaration origin mismatch".into());
                    }
                    placements.push(PlacementRef {
                        index: d.index,
                        mint: (d.mint.start, d.mint.end),
                        current: (d.current.start, d.current.end),
                    });
                }
                pieces.push(PieceRef {
                    cst: cst.clone(),
                    placements,
                });
                edges.push((cst, RefKind::RequiredData));
            }
            units.push(UnitRef {
                id: segment.module.clone(),
                source_sha: sha256_hex(segment.source.as_bytes()),
                pieces,
            });
        }
        for target in targets.values() {
            self.verify_target(target, &bound.input_hash)?;
            edges.push((target.artifact.clone(), RefKind::RequiredData));
        }
        let mut facts = Vec::new();
        // These are cache hits after the product compile, not a second solver.
        for name in bound.origins.keys() {
            let key = session
                .body_key(&bound.program, name)
                .map_err(|e| e.to_string())?;
            let page = session
                .infer_body(&bound.program, name)
                .map_err(|e| e.to_string())?;
            if !page.hit {
                return Err(format!("module preparation unexpectedly produced `{name}`"));
            }
            // Request revisions are lifecycle observations, not immutable
            // semantic content. The module recipe also binds every signature
            // read, unlike the original bare S934 lookup address.
            let catalog = session.catalog(&bound.program).map_err(|e| e.to_string())?;
            let mut reads: BTreeMap<String, String> = BTreeMap::new();
            for name in std::iter::once(name).chain(page.facts.sig_reads.iter()) {
                reads.insert(
                    name.clone(),
                    catalog
                        .version(name)
                        .map(|v| v.as_str().to_string())
                        .unwrap_or_else(|| "absent".into()),
                );
            }
            for negative in &page.facts.negatives {
                reads.insert(
                    format!("negative:{}:{}", negative.domain, negative.name),
                    catalog
                        .version(&negative.name)
                        .map(|v| v.as_str().to_string())
                        .unwrap_or_else(|| "absent".into()),
                );
            }
            let read_hash = sha256_hex(&serde_json::to_vec(&reads).map_err(|e| e.to_string())?);
            let mut canonical = page.facts.clone();
            canonical.revision = 0;
            let payload = encode_page(&key, &canonical).map_err(|e| e.to_string())?;
            let storage_key = QueryKey {
                layer: SQ_LAYER,
                components: vec![
                    ("module-page-recipe".into(), "sq-module-page/1".into()),
                    ("read-versions".into(), read_hash),
                    ("sq-key".into(), key.canonical()),
                ],
            };
            let artifact = self
                .sink
                .store()
                .put_artifact(PutRequest {
                    layer: SQ_LAYER,
                    key: storage_key,
                    kind: ChunkKind::SnapshotRecord,
                    payload,
                    edges: Vec::new(),
                    project: None,
                })
                .map_err(|e| e.to_string())?;
            facts.push(FactRef {
                name: name.clone(),
                artifact: artifact.clone(),
                key: key.canonical(),
            });
            edges.push((artifact, RefKind::RequiredData));
        }
        let mut dump = ace_core::source::decl_env::decl_env_dump(
            env,
            MODULE_BINDING_VERSION,
            &bound.input_hash,
        );
        for f in &mut dump.fns {
            let ast = bound
                .program
                .items
                .iter()
                .find_map(|i| match i {
                    ace_core::source::ast::Item::Fn(a) if a.name.name == f.name => Some(a),
                    _ => None,
                })
                .ok_or("module declaration context missing owner")?;
            f.name_span = (ast.name.span.start, ast.name.span.end);
            f.item_span = (ast.span.start, ast.span.end);
        }
        validate_declarations(bound, &dump)?;
        let root = Root {
            schema: MODULE_ROOT_SCHEMA.into(),
            binding: MODULE_BINDING_VERSION.into(),
            grammar: bound.grammar_id.clone(),
            entry: entry.into(),
            package: session.package().into(),
            module: session.module().into(),
            capabilities: canonical_caps(caps),
            input_hash: bound.input_hash.clone(),
            decl_env: ace_core::source::decl_env::encode_decl_env(&dump),
            targets: targets.clone(),
            units,
            facts,
        };
        let payload = serde_json::to_vec(&root).map_err(|e| e.to_string())?;
        if payload.len() > MAX_ROOT_BYTES {
            return Err("module root resource budget".into());
        }
        let digest = sha256_hex(&payload);
        let key = QueryKey {
            layer: "S950",
            components: vec![
                ("root-digest".into(), digest.clone()),
                ("schema".into(), MODULE_ROOT_SCHEMA.into()),
            ],
        };
        let artifact = self
            .sink
            .store()
            .put_artifact(PutRequest {
                layer: "S950",
                key,
                kind: ChunkKind::SnapshotRecord,
                payload,
                edges,
                project: None,
            })
            .map_err(|e| e.to_string())?;
        Ok(ModuleReceipt { artifact, digest })
    }
    fn verify_target(&self, target: &ModuleReceipt, source: &str) -> Result<(), String> {
        let a = self
            .sink
            .store()
            .get_artifact_by_id_bounded(&target.artifact, MAX_ROOT_BYTES as u64)
            .map_err(|e| e.to_string())?
            .ok_or("required module target absent")?;
        if a.kind != ChunkKind::SnapshotRecord || sha256_hex(&a.payload) != target.digest {
            return Err("module target receipt mismatch".into());
        }
        let value: serde_json::Value =
            serde_json::from_slice(&a.payload).map_err(|e| e.to_string())?;
        if value.get("source").and_then(|v| v.as_str()) != Some(source) {
            return Err("module target source binding mismatch".into());
        }
        Ok(())
    }
    fn read_root(&self, receipt: &ModuleReceipt) -> Result<Root, String> {
        let artifact = self
            .sink
            .store()
            .get_artifact_by_id_bounded(&receipt.artifact, MAX_ROOT_BYTES as u64)
            .map_err(|e| e.to_string())?
            .ok_or("missing module root")?;
        if artifact.kind != ChunkKind::SnapshotRecord
            || sha256_hex(&artifact.payload) != receipt.digest
        {
            return Err("module root receipt mismatch".into());
        }
        let root: Root = serde_json::from_slice(&artifact.payload)
            .map_err(|e| format!("module root codec: {e}"))?;
        if root.schema != MODULE_ROOT_SCHEMA || root.binding != MODULE_BINDING_VERSION {
            return Err("module root schema/binding mismatch".into());
        }
        let grammar = module_grammar()?;
        if root.grammar != format!("{}", grammar.program().identity.grammar_id.0)
            || root.capabilities != canonical_caps(&root.capabilities)
            || !root
                .capabilities
                .iter()
                .any(|c| c == ace_core::source::modules::MODULES_CAPABILITY)
            || root
                .capabilities
                .iter()
                .any(|c| c == ace_core::source::cst::INSTANCES_CAPABILITY)
        {
            return Err("module root grammar/capability mismatch".into());
        }
        let units: BTreeSet<_> = root.units.iter().map(|u| &u.id).collect();
        let facts: BTreeSet<_> = root.facts.iter().map(|f| &f.name).collect();
        if units.len() != root.units.len()
            || !units.contains(&root.entry)
            || facts.len() != root.facts.len()
        {
            return Err("module root duplicate/missing declaration".into());
        }
        let required: BTreeSet<_> = root
            .units
            .iter()
            .flat_map(|u| u.pieces.iter().map(|p| p.cst.clone()))
            .chain(root.facts.iter().map(|f| f.artifact.clone()))
            .chain(root.targets.values().map(|r| r.artifact.clone()))
            .collect();
        let actual: BTreeSet<_> = self
            .sink
            .store()
            .required_targets(&receipt.artifact)
            .map_err(|e| e.to_string())?
            .into_iter()
            .collect();
        if actual != required {
            return Err("module root required edges mismatch".into());
        }
        for target in root.targets.values() {
            self.verify_target(target, &root.input_hash)?;
        }
        Ok(root)
    }
    /// The existing store root protocol: register the complete new closure
    /// before removing old roots. Any failure before that point preserves old.
    pub fn publish(
        &self,
        candidate: &ModuleReceipt,
        base: Option<&ModuleReceipt>,
        cancelled: bool,
    ) -> Result<(), String> {
        if cancelled {
            return Err("module publication cancelled".into());
        }
        self.read_root(candidate)?;
        if let Some(base) = base {
            self.read_root(base)?;
        }
        let roots = self
            .sink
            .store()
            .root_artifacts(&self.project, ROOT_KIND)
            .map_err(|e| e.to_string())?;
        let expected = base.map(|b| vec![b.artifact.clone()]).unwrap_or_default();
        if roots.iter().cloned().collect::<BTreeSet<_>>() != expected.into_iter().collect() {
            return Err("module publication base changed".into());
        }
        self.sink
            .store()
            .register_root(&self.project, &candidate.artifact, ROOT_KIND)
            .map_err(|e| e.to_string())?;
        for old in roots {
            if old != candidate.artifact {
                self.sink
                    .store()
                    .remove_root(&self.project, ROOT_KIND, &old)
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
    pub fn restore(
        &self,
        receipt: &ModuleReceipt,
        inputs: &[ModuleInput],
        entry: &str,
        caps: &[String],
        package: &str,
        module: &str,
        budget: FrontBudget,
    ) -> Result<ModuleRecovery, String> {
        let root = self.read_root(receipt)?;
        let grammar = module_grammar()?;
        let grammar_id = format!("{}", grammar.program().identity.grammar_id.0);
        if root.entry != entry
            || root.package != package
            || root.module != module
            || root.grammar != grammar_id
            || root.capabilities != canonical_caps(caps)
        {
            return Err("module recovery identity mismatch".into());
        }
        if self
            .syntax
            .get_grammar(grammar.program())
            .map_err(|e| format!("{e:?}"))?
            .is_none()
        {
            return Err("required module grammar absent".into());
        }
        let mut sources = BTreeMap::new();
        let mut bytes = 0usize;
        for i in inputs {
            bytes = bytes
                .checked_add(i.source.len())
                .ok_or("module recovery byte budget")?;
            if bytes > budget.work_bytes
                || sources.insert(i.id.clone(), i.source.as_str()).is_some()
            {
                return Err("module source set/budget mismatch".into());
            }
        }
        if root.units.len() != sources.len() {
            return Err("module source set mismatch".into());
        }
        let mut units = BTreeMap::new();
        for u in root.units {
            let src = *sources.get(&u.id).ok_or("module identity absent")?;
            if sha256_hex(src.as_bytes()) != u.source_sha || units.contains_key(&u.id) {
                return Err("module source binding/duplicate mismatch".into());
            }
            use ace_core::source::modules::{
                assemble_relocated_module, DeclarationPlacement, SyntaxPiece,
            };
            let mut pieces = Vec::new();
            let mut projected = Vec::new();
            for p in u.pieces {
                let view = self
                    .syntax
                    .open_cst_receipt(grammar.program(), &p.cst)
                    .map_err(|e| format!("{e:?}"))?
                    .ok_or("missing module CST origin")?;
                let minted = String::from_utf8(view.source_bytes().map_err(|e| format!("{e:?}"))?)
                    .map_err(|e| e.to_string())?;
                if minted.len() > budget.work_bytes {
                    return Err("module syntax origin byte budget".into());
                }
                let mut trees = Vec::new();
                let mut placements = Vec::new();
                let mut selected = BTreeSet::new();
                for d in p.placements {
                    if !selected.insert(d.index) {
                        return Err("duplicate CST origin declaration".into());
                    }
                    let (a, b) = view.decl_range(d.index).map_err(|e| format!("{e:?}"))?;
                    if (a as usize, b as usize) != d.mint {
                        return Err("CST origin extent mismatch".into());
                    }
                    let mint = Span {
                        start: d.mint.0,
                        end: d.mint.1,
                    };
                    let current = Span {
                        start: d.current.0,
                        end: d.current.1,
                    };
                    if minted.get(mint.start..mint.end) != src.get(current.start..current.end) {
                        return Err("module declaration origin slice mismatch".into());
                    }
                    trees.push(view.decl_subtree(d.index).map_err(|e| format!("{e:?}"))?);
                    placements.push(DeclarationPlacement {
                        index: d.index,
                        mint,
                        current,
                    });
                }
                if placements.is_empty() {
                    return Err("empty CST origin selection".into());
                }
                let syntax = project_stored_module(&minted, trees, &grammar_id, budget)
                    .map_err(|e| format!("{e:?}"))?;
                projected.push((syntax, placements.clone()));
                let pages = Rc::new(PageStore::new());
                let snap = Snapshot::from_text(&pages, &minted);
                if self
                    .syntax
                    .get_tokens(grammar.program(), &snap)
                    .map_err(|e| format!("{e:?}"))?
                    .is_none()
                {
                    return Err("required module token stream absent".into());
                }
                pieces.push(SyntaxPiece {
                    source: Rc::from(minted.as_str()),
                    tree: None,
                    tokens: None,
                    placements,
                    stored_cst: Some(p.cst),
                });
            }
            let syntax = assemble_relocated_module(src, projected, pieces, grammar_id.clone())?;
            units.insert(u.id, syntax);
        }
        let bound =
            bind_module_syntax(inputs, entry, caps, budget, units).map_err(|e| format!("{e:?}"))?;
        if bound.input_hash != root.input_hash {
            return Err("module source graph digest mismatch".into());
        }
        let mut session =
            SourceQuerySession::new(package, module).with_capabilities(caps.iter().cloned());
        let mut seen = BTreeSet::new();
        for f in root.facts {
            if !seen.insert(f.name.clone()) || !bound.origins.contains_key(&f.name) {
                return Err("module fact declaration mismatch".into());
            }
            let key = session
                .body_key(&bound.program, &f.name)
                .map_err(|e| e.to_string())?;
            if key.canonical() != f.key {
                return Err("module fact key mismatch".into());
            }
            let page = self
                .sink
                .get_checked(&f.artifact, &key)
                .map_err(|e| e.to_string())?;
            session
                .seed(&bound.program, &page)
                .map_err(|e| e.to_string())?;
        }
        if seen.len() != bound.origins.len() {
            return Err("missing required module fact".into());
        }
        let dump = ace_core::source::decl_env::decode_decl_env(&root.decl_env)
            .map_err(|e| format!("{e:?}"))?;
        let names: Vec<_> = bound
            .program
            .items
            .iter()
            .filter_map(|i| match i {
                ace_core::source::ast::Item::Fn(f) => Some(f.name.name.as_str()),
                _ => None,
            })
            .collect();
        if !dump.fns.iter().map(|f| f.name.as_str()).eq(names) {
            return Err("module declaration environment owner mismatch".into());
        }
        validate_declarations(&bound, &dump)?;
        let env = ace_core::source::decl_env::decl_env_restore(
            &dump,
            SourceProfile::Scalar,
            MODULE_BINDING_VERSION,
            &bound.input_hash,
        )
        .map_err(|e| format!("{e:?}"))?;
        Ok(ModuleRecovery {
            bound,
            session,
            env,
            targets: root.targets,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ace_core::source::modules::MODULES_CAPABILITY;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temp(std::path::PathBuf);
    impl Temp {
        fn new() -> Self {
            let root = std::env::var_os("AUTO_PLAN_ARTIFACTS")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir);
            Self(root.join(format!(
                "module-store-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            )))
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn inputs(value: i32) -> Vec<ModuleInput> {
        vec![
            ModuleInput {
                id: "std".into(),
                source: format!("pub fn answer() int {{ return {value} }}"),
            },
            ModuleInput {
                id: "app".into(),
                source:
                    "use std.answer\nfn local() int { return 7 } fn main() int { return answer() }"
                        .into(),
            },
        ]
    }
    fn caps() -> Vec<String> {
        vec![MODULES_CAPABILITY.into()]
    }
    fn build(source: &[ModuleInput], session: &mut SourceQuerySession) -> ModuleProduct {
        compile_module_product(
            source,
            "app",
            &caps(),
            session,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap()
    }
    #[test]
    fn stored_module_cst_and_facts_restore_without_scanning_parsing_or_solving() {
        let temp = Temp::new();
        let source = inputs(42);
        let receipt = {
            let mut store = ModuleStore::open(&temp.0, "test").unwrap();
            let mut session = SourceQuerySession::new("pkg", "app");
            let (product, work) =
                ace_core::source::parser::capture_syntax_work(|| build(&source, &mut session));
            assert_eq!(work.tokenized_ranges.len(), 2);
            assert_eq!(work.parsed_functions.values().sum::<u64>(), 3);
            let before = session.counters().solver_runs;
            let receipt = store
                .prepare(&product, &mut session, "app", &caps())
                .unwrap();
            assert_eq!(session.counters().solver_runs, before);
            assert!(
                store
                    .sink
                    .store()
                    .root_artifacts("test", "cst-manifest")
                    .unwrap()
                    .is_empty(),
                "candidate CSTs cannot replace a current syntax root"
            );
            store.publish(&receipt, None, false).unwrap();
            receipt
        };
        let store = ModuleStore::open(&temp.0, "test").unwrap();
        let (recovery, work) = ace_core::source::parser::capture_syntax_work(|| {
            store.restore(
                &receipt,
                &source,
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB,
            )
        });
        let mut recovery = recovery.unwrap();
        assert!(
            work.tokenized_ranges.is_empty() && work.parsed_functions.is_empty(),
            "{work:?}"
        );
        assert_eq!(recovery.session.counters().solver_runs, 0);
        for name in recovery.bound.origins.keys() {
            assert!(
                recovery
                    .session
                    .infer_body(&recovery.bound.program, name)
                    .unwrap()
                    .hit
            );
        }
        assert_eq!(recovery.session.counters().solver_runs, 0);
        assert!(store
            .restore(
                &receipt,
                &inputs(43),
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB
            )
            .err()
            .unwrap()
            .contains("source binding"));
        let mut bad = receipt.clone();
        bad.digest = "0".repeat(64);
        assert!(store
            .restore(
                &bad,
                &source,
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB
            )
            .is_err());
        assert!(store
            .restore(
                &receipt,
                &source,
                "app",
                &[],
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB
            )
            .is_err());
    }
    #[test]
    fn candidate_cancellation_and_actual_root_publication_failure_preserve_old_recovery() {
        let temp = Temp::new();
        let mut store = ModuleStore::open(&temp.0, "test").unwrap();
        let mut session = SourceQuerySession::new("pkg", "app");
        let first = build(&inputs(42), &mut session);
        let old = store.prepare(&first, &mut session, "app", &caps()).unwrap();
        store.publish(&old, None, false).unwrap();
        let next = build(&inputs(43), &mut session);
        let candidate = store.prepare(&next, &mut session, "app", &caps()).unwrap();
        assert!(store.publish(&candidate, Some(&old), true).is_err());
        store.sink.store().db().execute_batch("CREATE TRIGGER reject_module_root BEFORE INSERT ON roots WHEN NEW.kind='source-modules-current' BEGIN SELECT RAISE(ABORT,'injected module root failure'); END;").unwrap();
        assert!(store
            .publish(&candidate, Some(&old), false)
            .unwrap_err()
            .contains("injected module root failure"));
        store
            .sink
            .store()
            .db()
            .execute_batch("DROP TRIGGER reject_module_root")
            .unwrap();
        assert_eq!(
            store
                .sink
                .store()
                .root_artifacts("test", ROOT_KIND)
                .unwrap(),
            vec![old.artifact.clone()]
        );
        drop(store);
        let store = ModuleStore::open(&temp.0, "test").unwrap();
        assert!(store
            .restore(
                &old,
                &inputs(42),
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB
            )
            .is_ok());
        store.publish(&candidate, Some(&old), false).unwrap();
        drop(store);
        let store = ModuleStore::open(&temp.0, "test").unwrap();
        assert!(store
            .restore(
                &candidate,
                &inputs(43),
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB
            )
            .is_ok());
    }
    #[test]
    fn recovery_refuses_missing_required_parent_edges_before_consumption() {
        let temp = Temp::new();
        let mut store = ModuleStore::open(&temp.0, "test").unwrap();
        let mut session = SourceQuerySession::new("pkg", "app");
        let p = build(&inputs(42), &mut session);
        let receipt = store.prepare(&p, &mut session, "app", &caps()).unwrap();
        let deleted = store
            .sink
            .store()
            .db()
            .execute("DELETE FROM edges WHERE src=?1", [&receipt.artifact])
            .unwrap();
        assert!(deleted > 0);
        assert!(store
            .restore(
                &receipt,
                &inputs(42),
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB
            )
            .err()
            .unwrap()
            .contains("required edges"));
        assert!(store.publish(&receipt, None, false).is_err());
    }
    #[test]
    fn foreign_root_identities_and_duplicate_declarations_reject_before_publication() {
        let temp = Temp::new();
        let mut store = ModuleStore::open(&temp.0, "test").unwrap();
        let mut session = SourceQuerySession::new("pkg", "app");
        let p = build(&inputs(42), &mut session);
        assert!(store.prepare(&p, &mut session, "std", &caps()).is_err());
        let original = store.prepare(&p, &mut session, "app", &caps()).unwrap();
        store.publish(&original, None, false).unwrap();
        let artifact = store
            .sink
            .store()
            .get_artifact_by_id(&original.artifact)
            .unwrap()
            .unwrap();
        for kind in [
            "schema",
            "binding",
            "grammar",
            "capabilities",
            "duplicate-unit",
            "duplicate-fact",
            "unknown-field",
        ] {
            let mut value: serde_json::Value = serde_json::from_slice(&artifact.payload).unwrap();
            match kind {
                "schema" | "binding" | "grammar" => {
                    value[kind] = serde_json::json!("foreign-version")
                }
                "capabilities" => value[kind] = serde_json::json!([]),
                "duplicate-unit" => {
                    let duplicate = value["units"][0].clone();
                    value["units"].as_array_mut().unwrap().push(duplicate);
                }
                "duplicate-fact" => {
                    let duplicate = value["facts"][0].clone();
                    value["facts"].as_array_mut().unwrap().push(duplicate);
                }
                _ => value["unexpected-authority"] = serde_json::json!(true),
            }
            let payload = serde_json::to_vec(&value).unwrap();
            let root: Root = serde_json::from_slice(&artifact.payload).unwrap();
            let edges = root
                .units
                .iter()
                .flat_map(|u| {
                    u.pieces
                        .iter()
                        .map(|p| (p.cst.clone(), RefKind::RequiredData))
                })
                .chain(
                    root.facts
                        .iter()
                        .map(|f| (f.artifact.clone(), RefKind::RequiredData)),
                )
                .collect();
            let digest = sha256_hex(&payload);
            let id = store
                .sink
                .store()
                .put_artifact(PutRequest {
                    layer: "S950",
                    key: QueryKey {
                        layer: "S950",
                        components: vec![("negative".into(), digest.clone())],
                    },
                    kind: ChunkKind::SnapshotRecord,
                    payload,
                    edges,
                    project: None,
                })
                .unwrap();
            let bad = ModuleReceipt {
                artifact: id,
                digest,
            };
            assert!(
                store
                    .restore(
                        &bad,
                        &inputs(42),
                        "app",
                        &caps(),
                        "pkg",
                        "app",
                        FrontBudget::WORK_OWNED_64MIB
                    )
                    .is_err(),
                "{kind}"
            );
            assert!(
                store.publish(&bad, Some(&original), false).is_err(),
                "{kind}"
            );
            assert_eq!(
                store
                    .sink
                    .store()
                    .root_artifacts("test", ROOT_KIND)
                    .unwrap(),
                vec![original.artifact.clone()]
            );
        }
        drop(store);
        assert!(ModuleStore::open(&temp.0, "test")
            .unwrap()
            .restore(
                &original,
                &inputs(42),
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB
            )
            .is_ok());
    }
    #[test]
    fn restored_first_body_edit_reuses_unchanged_declarations_and_survives_next_recovery() {
        let temp = Temp::new();
        let mut source = inputs(42);
        source[0].source.push_str(" fn helper() int { return 9 }");
        let old = {
            let mut store = ModuleStore::open(&temp.0, "edit").unwrap();
            let mut query = SourceQuerySession::new("pkg", "app");
            let p = build(&source, &mut query);
            let root = store.prepare(&p, &mut query, "app", &caps()).unwrap();
            store.publish(&root, None, false).unwrap();
            root
        };
        let mut store = ModuleStore::open(&temp.0, "edit").unwrap();
        let recovery = store
            .restore(
                &old,
                &source,
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        let mut session = ModuleSession::from_recovery(recovery);
        let ((edit, body), syntax) = ace_core::source::parser::capture_syntax_work(|| {
            ace_core::body_work::capture_body_work(|| {
                session.prepare(
                    "std",
                    "answer",
                    "pub fn answer() int { return 123 }",
                    FrontBudget::WORK_OWNED_64MIB,
                )
            })
        });
        let edit = edit.unwrap();
        assert_eq!(syntax.parsed_functions.len(), 1);
        assert_eq!(syntax.parsed_functions["answer"], 1);
        assert_eq!(syntax.tokenized_ranges.len(), 1);
        assert_eq!(body.len(), 1, "{body:?}");
        let work = body["m3_std_answer"];
        assert_eq!(
            (work.resolve, work.typecheck, work.plan, work.bind),
            (1, 1, 1, 1)
        );
        assert_eq!(
            edit.bound.binding_walks,
            BTreeSet::from(["m3_std_answer".into()])
        );
        assert_eq!(session.query.counters().solver_runs, 1);
        let candidate = session.prepare_receipt(&mut store, &edit).unwrap();
        assert_eq!(
            session.query.counters().solver_runs,
            1,
            "persist cannot solve unchanged functions"
        );
        session.validate(&edit).unwrap();
        store.publish(&candidate, Some(&old), false).unwrap();
        session.commit(edit).unwrap();
        source = session
            .bound
            .segments
            .iter()
            .map(|s| ModuleInput {
                id: s.module.clone(),
                source: s.source.clone(),
            })
            .collect();
        drop(session);
        drop(store);
        let store = ModuleStore::open(&temp.0, "edit").unwrap();
        let recovery = store
            .restore(
                &candidate,
                &source,
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        let mut session = ModuleSession::from_recovery(recovery);
        let ((next, body), syntax) = ace_core::source::parser::capture_syntax_work(|| {
            ace_core::body_work::capture_body_work(|| {
                session.prepare(
                    "std",
                    "answer",
                    "pub fn answer() int { return 43 }",
                    FrontBudget::WORK_OWNED_64MIB,
                )
            })
        });
        let next = next.unwrap();
        assert_eq!(body.len(), 1);
        assert_eq!(syntax.parsed_functions.len(), 1);
        assert_eq!(
            next.bound.binding_walks,
            BTreeSet::from(["m3_std_answer".into()])
        );
        assert_eq!(session.query.counters().solver_runs, 1);
        // Dropping the prepared edit does not publish new source or context.
        drop(next);
        assert!(session
            .bound
            .segments
            .iter()
            .any(|s| s.module == "std" && s.source.contains("return 123")));
        assert!(store
            .restore(
                &candidate,
                &source,
                "app",
                &caps(),
                "pkg",
                "app",
                FrontBudget::WORK_OWNED_64MIB
            )
            .is_ok());
    }
}
