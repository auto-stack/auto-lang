//! Finite module adaptation of the existing A2X session/provider/cache.
//! Preparation writes immutable fragments but never replaces current output.
use super::*;
use ace_core::source::modules::BoundModules;
use ace_source_query::modules::{ModuleProduct, ModuleReceipt, PreparedModuleEdit};
use serde::{Deserialize, Serialize};
const SCHEMA: &str = "a2x-source-modules/1";
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    name: String,
    index: usize,
    sem_fp: String,
    artifact: String,
    digest: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Root {
    schema: String,
    source: String,
    owner: String,
    backend: String,
    caps: Vec<String>,
    runtime: String,
    target_inputs: Vec<(String, String)>,
    fragments: Vec<Fragment>,
    files: Vec<(String, String)>,
    manifest: String,
}
pub struct ModuleA2xTarget {
    inner: A2xSession,
    root: Root,
    current: Assembly,
}
pub struct ModuleA2xEdit {
    base: String,
    root: Root,
    current: Assembly,
}
fn mapped_origins(
    bound: &BoundModules,
    fragments: &[Fragment],
) -> Result<Vec<(usize, (u32, u32))>, String> {
    fragments
        .iter()
        .map(|r| {
            let f = bound
                .program
                .items
                .iter()
                .find_map(|i| match i {
                    ace_core::source::ast::Item::Fn(f) if f.name.name == r.name => Some(f),
                    _ => None,
                })
                .ok_or("A2X module owner absent")?;
            Ok((r.index, (f.span.start as u32, f.span.end as u32)))
        })
        .collect()
}
fn fragments_for<'a>(root: &'a Root, text: &'a [String]) -> Vec<FragmentText<'a>> {
    root.fragments
        .iter()
        .zip(text)
        .map(|(r, t)| FragmentText {
            def_index: r.index,
            source_name: &r.name,
            sem_fp: &r.sem_fp,
            text: t,
        })
        .collect()
}
fn assemble(
    inner: &A2xSession,
    root: &Root,
    bound: &BoundModules,
    text: &[String],
) -> Result<Assembly, String> {
    let origins = mapped_origins(bound, &root.fragments)?;
    let fs = inner
        .provider
        .assemble_fileset(FilesetRequest {
            entry: "main",
            input_hash: &root.source,
            family: Family::Scalar,
            fragments: &fragments_for(root, text),
            view: None,
            runtime: &root.runtime,
            prelude_seed: None,
            origins: Some(&origins),
            source_sha256: Some(&root.source),
        })
        .map_err(|e| format!("{e:?}"))?;
    Ok(Assembly {
        files: fs.files,
        manifest: fs.manifest,
    })
}
fn seal_files(root: &mut Root, assembly: &Assembly) {
    root.files = assembly
        .files
        .iter()
        .map(|(p, c)| (p.clone(), sha256_hex(c.as_bytes())))
        .collect();
    root.manifest = sha256_hex(assembly.manifest.as_bytes());
}
fn read_fragments(inner: &A2xSession, root: &Root) -> Result<Vec<String>, String> {
    let mut text = Vec::new();
    for r in &root.fragments {
        let mut inputs = vec![
            ("owner".into(), root.owner.clone()),
            ("decl".into(), r.name.clone()),
            ("sem_fp".into(), r.sem_fp.clone()),
        ];
        inputs.extend(
            inner
                .provider
                .fragment_key_inputs(&sha256_hex(root.runtime.as_bytes())),
        );
        let key = QueryKey::new(inner.layer_fragment(), inputs);
        let a = inner
            .store
            .get_artifact(inner.layer_fragment().id, &key)
            .map_err(|e| format!("{e:?}"))?
            .ok_or("required A2X module fragment absent")?;
        if a.artifact_id != r.artifact
            || a.kind != ChunkKind::Object
            || sha256_hex(&a.payload) != r.digest
        {
            return Err("A2X module fragment identity mismatch".into());
        }
        text.push(String::from_utf8(a.payload).map_err(|e| format!("{e:?}"))?);
    }
    Ok(text)
}
impl ModuleA2xTarget {
    pub fn cold<P: TargetProvider + 'static>(
        path: &Path,
        project: &str,
        owner: &str,
        quota: u64,
        provider: P,
        product: &ModuleProduct,
    ) -> Result<Self, String> {
        let mut inner = A2xSession::open(path, project, owner, quota, provider)?;
        inner.profile = SourceProfile::Scalar;
        inner.caps = product.bound.capabilities.clone();
        inner.source_sha256 = Some(product.bound.input_hash.clone());
        inner
            .ensure_target_supports()
            .map_err(|e| format!("{e:?}"))?;
        let view =
            SemanticView::from_checked(&product.compiled.checked).map_err(|e| format!("{e:?}"))?;
        // Reuse the existing batched producer/resident/store loop, while the
        // multi-source owner publishes its own source-set anchor below.
        inner
            .assemble_view_internal(&view, &product.bound.input_hash, "main", "", false, false)
            .map_err(|e| format!("{e:?}"))?;
        let runtime = inner
            .provider
            .runtime_source(&view, Family::Scalar)
            .map_err(|e| format!("{e:?}"))?;
        let mut root = Root {
            schema: SCHEMA.into(),
            source: product.bound.input_hash.clone(),
            owner: owner.into(),
            backend: inner.current_backend().into(),
            caps: product.bound.capabilities.clone(),
            target_inputs: inner
                .provider
                .fragment_key_inputs(&sha256_hex(runtime.as_bytes())),
            runtime,
            fragments: Vec::new(),
            files: Vec::new(),
            manifest: String::new(),
        };
        for f in view.functions() {
            let r = &inner.resident[&(root.backend.clone(), f.source_name.into())];
            let key = inner.fragment_key(f, &r.sem_fp, &sha256_hex(root.runtime.as_bytes()));
            let a = inner
                .store
                .get_artifact(inner.layer_fragment().id, &key)
                .map_err(|e| format!("{e:?}"))?
                .ok_or("new A2X module fragment absent")?;
            root.fragments.push(Fragment {
                name: r.source_name.clone(),
                index: r.def_index,
                sem_fp: r.sem_fp.clone(),
                artifact: a.artifact_id,
                digest: sha256_hex(&a.payload),
            });
        }
        root.fragments.sort_by_key(|r| r.index);
        let text = read_fragments(&inner, &root)?;
        let current = assemble(&inner, &root, &product.bound, &text)?;
        seal_files(&mut root, &current);
        Ok(Self {
            inner,
            root,
            current,
        })
    }
    pub fn current(&self) -> &Assembly {
        &self.current
    }
    pub fn stats(&self) -> &ProducerStats {
        self.inner.stats()
    }
    pub fn fragment_receipts(&self) -> BTreeMap<String, String> {
        self.root
            .fragments
            .iter()
            .map(|r| (r.name.clone(), r.artifact.clone()))
            .collect()
    }
    pub fn prepare(&mut self, edit: &PreparedModuleEdit) -> Result<ModuleA2xEdit, String> {
        if self.root.caps != edit.bound.capabilities {
            return Err("A2X module capabilities changed".into());
        }
        let parts = edit.candidate.parts();
        let def = format!("d_{}", edit.owner);
        let index = parts.bundle.modules[0]
            .defs
            .iter()
            .position(|d| d.id_text == def)
            .ok_or("A2X module candidate def absent")?;
        let cred = verify_function_ref(&parts.bundle, index).map_err(|e| format!("{e:?}"))?;
        let view = SemanticView::from_parts(parts, &cred).map_err(|e| format!("{e:?}"))?;
        let f = view
            .functions()
            .first()
            .ok_or("empty A2X module candidate")?;
        if f.source_name != edit.owner {
            return Err("A2X module candidate owner mismatch".into());
        }
        let sem_fp = semantic_fingerprint(&view, f).map_err(|e| format!("{e:?}"))?;
        let runtime = self
            .inner
            .provider
            .runtime_source(&view, Family::Scalar)
            .map_err(|e| format!("{e:?}"))?;
        if runtime != self.root.runtime {
            return Err("A2X module runtime changed outside the body-edit contract".into());
        }
        self.inner
            .produce_fragment(&view, f, &sem_fp, &sha256_hex(runtime.as_bytes()), false)
            .map_err(|e| format!("{e:?}"))?;
        let key = self
            .inner
            .fragment_key(f, &sem_fp, &sha256_hex(runtime.as_bytes()));
        let a = self
            .inner
            .store
            .get_artifact(self.inner.layer_fragment().id, &key)
            .map_err(|e| format!("{e:?}"))?
            .ok_or("new A2X module fragment absent")?;
        let mut root = self.root.clone();
        root.source = edit.bound.input_hash.clone();
        let r = root
            .fragments
            .iter_mut()
            .find(|r| r.name == f.source_name)
            .ok_or("A2X module owner absent")?;
        r.sem_fp = sem_fp;
        r.artifact = a.artifact_id;
        r.digest = sha256_hex(&a.payload);
        let text = read_fragments(&self.inner, &root)?;
        let current = assemble(&self.inner, &root, &edit.bound, &text)?;
        seal_files(&mut root, &current);
        Ok(ModuleA2xEdit {
            base: self.root.source.clone(),
            root,
            current,
        })
    }
    pub fn validate(&self, edit: &ModuleA2xEdit) -> Result<(), String> {
        if edit.base != self.root.source {
            return Err("stale A2X module edit".into());
        }
        Ok(())
    }
    pub fn commit(&mut self, edit: ModuleA2xEdit) -> Result<(), String> {
        self.validate(&edit)?;
        self.root = edit.root;
        self.current = edit.current;
        self.inner.source_sha256 = Some(self.root.source.clone());
        Ok(())
    }
    pub fn receipt(&mut self) -> Result<ModuleReceipt, String> {
        write_root(&mut self.inner, &self.root)
    }
    pub fn prepared_receipt(&mut self, edit: &ModuleA2xEdit) -> Result<ModuleReceipt, String> {
        if edit.base != self.root.source {
            return Err("stale A2X module receipt".into());
        }
        write_root(&mut self.inner, &edit.root)
    }
    pub fn restore<P: TargetProvider + 'static>(
        path: &Path,
        project: &str,
        owner: &str,
        quota: u64,
        provider: P,
        receipt: &ModuleReceipt,
        bound: &BoundModules,
    ) -> Result<Self, String> {
        let mut inner = A2xSession::open(path, project, owner, quota, provider)?;
        inner.profile = SourceProfile::Scalar;
        inner.caps = bound.capabilities.clone();
        inner.source_sha256 = Some(bound.input_hash.clone());
        let a = inner
            .store
            .get_artifact_by_id_bounded(&receipt.artifact, 512 * 1024)
            .map_err(|e| format!("{e:?}"))?
            .ok_or("A2X module root absent")?;
        if a.kind != ChunkKind::SnapshotRecord || sha256_hex(&a.payload) != receipt.digest {
            return Err("A2X module root receipt mismatch".into());
        }
        let root: Root = serde_json::from_slice(&a.payload).map_err(|e| format!("{e:?}"))?;
        if root.schema != SCHEMA
            || root.source != bound.input_hash
            || root.owner != owner
            || root.backend != inner.current_backend()
            || root.caps != bound.capabilities
            || root.target_inputs
                != inner
                    .provider
                    .fragment_key_inputs(&sha256_hex(root.runtime.as_bytes()))
        {
            return Err("A2X module producer/source/runtime identity mismatch".into());
        }
        let names: std::collections::BTreeSet<_> =
            root.fragments.iter().map(|f| f.name.clone()).collect();
        if names.len() != root.fragments.len() || names != bound.origins.keys().cloned().collect() {
            return Err("A2X module declaration set mismatch".into());
        }
        let required: std::collections::BTreeSet<_> =
            root.fragments.iter().map(|r| r.artifact.clone()).collect();
        if inner
            .store
            .required_targets(&receipt.artifact)
            .map_err(|e| format!("{e:?}"))?
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            != required
        {
            return Err("A2X module required edges mismatch".into());
        }
        let text = read_fragments(&inner, &root)?;
        let current = assemble(&inner, &root, bound, &text)?;
        let mut verified = root.clone();
        seal_files(&mut verified, &current);
        if verified.files != root.files || verified.manifest != root.manifest {
            return Err("A2X module file-set receipt mismatch".into());
        }
        Ok(Self {
            inner,
            root,
            current,
        })
    }
}
fn write_root(inner: &mut A2xSession, root: &Root) -> Result<ModuleReceipt, String> {
    let payload = serde_json::to_vec(root).map_err(|e| format!("{e:?}"))?;
    if payload.len() > 512 * 1024 {
        return Err("A2X module root resource budget".into());
    }
    inner
        .quota_guard(payload.len() as u64)
        .map_err(|e| format!("{e:?}"))?;
    let digest = sha256_hex(&payload);
    let key = QueryKey {
        layer: "AXM950",
        components: vec![
            ("schema".into(), SCHEMA.into()),
            ("digest".into(), digest.clone()),
        ],
    };
    if let Some(a) = inner
        .store
        .get_artifact("AXM950", &key)
        .map_err(|e| format!("{e:?}"))?
    {
        return Ok(ModuleReceipt {
            artifact: a.artifact_id,
            digest,
        });
    }
    inner.stats.persist += 1;
    inner.stats.bytes_persisted += payload.len() as u64;
    let artifact = inner
        .store
        .put_artifact(PutRequest {
            layer: "AXM950",
            key,
            kind: ChunkKind::SnapshotRecord,
            payload,
            edges: root
                .fragments
                .iter()
                .map(|r| (r.artifact.clone(), RefKind::RequiredData))
                .collect(),
            project: None,
        })
        .map_err(|e| format!("{e:?}"))?;
    Ok(ModuleReceipt { artifact, digest })
}
