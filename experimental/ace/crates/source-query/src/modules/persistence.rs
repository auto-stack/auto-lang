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

pub const MODULE_ROOT_SCHEMA: &str = "source-modules-root/1";
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
    cst: String,
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
    units: Vec<UnitRef>,
    facts: Vec<FactRef>,
}

pub struct ModuleRecovery {
    pub bound: BoundModules,
    pub session: SourceQuerySession,
}
pub struct ModuleStore {
    syntax: SyntaxArtifactStore,
    sink: ManagedStoreSink,
    project: String,
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
        let grammar = module_grammar()?;
        let mut units = Vec::new();
        let mut edges = Vec::new();
        for segment in &product.bound.segments {
            let syntax = &product.bound.units[&segment.module];
            if syntax.syntax.len() != 1 {
                return Err(
                    "candidate needs current parsed CST, not a reconstructed declaration list"
                        .into(),
                );
            }
            let tokens = syntax.tokens.as_ref().ok_or("candidate tokens absent")?;
            let pages = Rc::new(PageStore::new());
            let snap = Snapshot::from_text(&pages, &segment.source);
            let a02 = self
                .syntax
                .put_tokens(grammar.program(), &snap, tokens)
                .map_err(|e| format!("{e:?}"))?;
            let cst = self
                .syntax
                .put_cst_unpublished(grammar.program(), &snap, &syntax.syntax[0], &a02)
                .map_err(|e| format!("{e:?}"))?;
            units.push(UnitRef {
                id: segment.module.clone(),
                source_sha: sha256_hex(segment.source.as_bytes()),
                cst: cst.clone(),
            });
            edges.push((cst, RefKind::RequiredData));
        }
        let mut facts = Vec::new();
        // These are cache hits after the product compile, not a second solver.
        for name in product.bound.origins.keys() {
            let key = session
                .body_key(&product.bound.program, name)
                .map_err(|e| e.to_string())?;
            let page = session
                .infer_body(&product.bound.program, name)
                .map_err(|e| e.to_string())?;
            if !page.hit {
                return Err(format!("module preparation unexpectedly produced `{name}`"));
            }
            // Request revisions are lifecycle observations, not immutable
            // semantic content. The module recipe also binds every signature
            // read, unlike the original bare S934 lookup address.
            let catalog = session
                .catalog(&product.bound.program)
                .map_err(|e| e.to_string())?;
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
        let root = Root {
            schema: MODULE_ROOT_SCHEMA.into(),
            binding: MODULE_BINDING_VERSION.into(),
            grammar: product.bound.grammar_id.clone(),
            entry: entry.into(),
            package: session.package().into(),
            module: session.module().into(),
            capabilities: canonical_caps(caps),
            input_hash: product.bound.input_hash.clone(),
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
            .map(|u| u.cst.clone())
            .chain(root.facts.iter().map(|f| f.artifact.clone()))
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
            let pages = Rc::new(PageStore::new());
            let snap = Snapshot::from_text(&pages, src);
            let view = self
                .syntax
                .open_cst(grammar.program(), &snap)
                .map_err(|e| format!("{e:?}"))?
                .ok_or("missing module CST")?;
            // The caller-held root receipt protects this expected A03 identity.
            // open_cst rechecks the source+grammar key and all pages on demand.
            if view.artifact_id() != u.cst {
                return Err("module CST receipt mismatch".into());
            }
            let expected = self
                .sink
                .store()
                .get_artifact_by_id(&u.cst)
                .map_err(|e| e.to_string())?
                .ok_or("required module CST absent")?;
            if expected.kind != ChunkKind::CstManifest {
                return Err("module CST carrier mismatch".into());
            }
            if view.source_bytes().map_err(|e| format!("{e:?}"))? != src.as_bytes() {
                return Err("module CST source mismatch".into());
            }
            let tokens = self
                .syntax
                .get_tokens(grammar.program(), &snap)
                .map_err(|e| format!("{e:?}"))?
                .ok_or("required module token stream absent")?;
            let mut trees = Vec::new();
            for i in 0..view.decl_count() {
                trees.push(view.decl_subtree(i).map_err(|e| format!("{e:?}"))?);
            }
            let mut syntax = project_stored_module(src, trees, &grammar_id, budget)
                .map_err(|e| format!("{e:?}"))?;
            syntax.tokens = Some(tokens);
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
        Ok(ModuleRecovery { bound, session })
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
                .map(|u| (u.cst.clone(), RefKind::RequiredData))
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
}
