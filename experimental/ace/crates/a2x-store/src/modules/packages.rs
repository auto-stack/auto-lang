//! PLAN-960 packages A2X target: the same session/provider/cache machinery
//! as the 950 module lane, in the record-text family (the packages surface
//! compiles in the TextRecord domain). Cold emits fragments and assembles a
//! fileset; restore re-reads the persisted fragments (no re-emission) and
//! re-assembles. No warm body-edit path in this first cut — declaration or
//! body changes recompile through the packages lane.
//!
//! Boundary: fragment names are free functions; type-bound methods are
//! owner-qualified in the semantic view and close with the A2X follow-up
//! (named rejection, never silently dropped).
use super::*;
use ace_core::source::packages::BoundPackages;
use ace_source_query::modules::packages::PackageProduct;

pub const PKG_A2X_SCHEMA: &str = "a2x-source-packages/1";

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PkgRoot {
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

pub struct PackageA2xTarget {
    pub inner: A2xSession,
    root: PkgRoot,
    pub current: Assembly,
}

fn pkg_mapped_origins(
    bound: &BoundPackages,
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
                .ok_or(format!(
                    "A2X packages fragment `{}` is not a free function (type-bound methods close with the A2X follow-up)",
                    r.name
                ))?;
            Ok((r.index, (f.span.start as u32, f.span.end as u32)))
        })
        .collect()
}

impl PackageA2xTarget {
    pub fn cold<P: TargetProvider + 'static>(
        path: &Path,
        project: &str,
        owner: &str,
        quota: u64,
        provider: P,
        product: &PackageProduct,
    ) -> Result<Self, String> {
        let mut inner = A2xSession::open(path, project, owner, quota, provider)?;
        inner.profile = SourceProfile::TextRecord;
        inner.caps = product.bound.capabilities.clone();
        inner.source_sha256 = Some(product.bound.input_hash.clone());
        inner
            .ensure_target_supports()
            .map_err(|e| format!("{e:?}"))?;
        let view = SemanticView::from_checked(&product.compiled.checked).map_err(|e| format!("{e:?}"))?;
        let runtime = inner
            .provider
            .runtime_source(&view, Family::RecordText)
            .map_err(|e| format!("{e:?}"))?;
        inner
            .assemble_view_with_contexts(
                &view,
                &product.bound.input_hash,
                "main",
                "",
                false,
                false,
                &BTreeMap::new(),
            )
            .map_err(|e| format!("{e:?}"))?;
        let mut root = PkgRoot {
            schema: PKG_A2X_SCHEMA.into(),
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
            let key = inner.fragment_key_with_context(
                f,
                &r.sem_fp,
                &sha256_hex(root.runtime.as_bytes()),
                None,
            );
            let a = inner
                .store
                .get_artifact(inner.layer_fragment().id, &key)
                .map_err(|e| format!("{e:?}"))?
                .ok_or("new A2X packages fragment absent")?;
            root.fragments.push(Fragment {
                name: r.source_name.clone(),
                index: r.def_index,
                sem_fp: r.sem_fp.clone(),
                artifact: a.artifact_id,
                digest: sha256_hex(&a.payload),
                context: String::new(),
            });
        }
        root.fragments.sort_by_key(|r| r.index);
        let current = Self::assemble(&inner, &mut root, &product.bound, &view)?;
        Ok(Self {
            inner,
            root,
            current,
        })
    }

    fn assemble(
        inner: &A2xSession,
        root: &mut PkgRoot,
        bound: &BoundPackages,
        view: &SemanticView,
    ) -> Result<Assembly, String> {
        let text = read_pkg_fragments(inner, root)?;
        let origins = pkg_mapped_origins(bound, &root.fragments)?;
        let fragments = root
            .fragments
            .iter()
            .zip(&text)
            .map(|(r, t)| FragmentText {
                def_index: r.index,
                source_name: &r.name,
                sem_fp: &r.sem_fp,
                text: t,
            })
            .collect::<Vec<_>>();
        // record-text filesets carry the record table seed (captured from
        // the live view; the warm path cannot re-derive it later)
        let seed = inner
            .provider
            .capture_prelude_seed(view, Family::RecordText)
            .map_err(|e| format!("{e:?}"))?;
        let fs = inner
            .provider
            .assemble_fileset(FilesetRequest {
                entry: "main",
                input_hash: &root.source,
                family: Family::RecordText,
                fragments: &fragments,
                view: Some(view),
                runtime: &root.runtime,
                prelude_seed: seed.as_deref(),
                origins: Some(&origins),
                source_sha256: Some(&root.source),
            })
            .map_err(|e| format!("{e:?}"))?;
        let current = Assembly {
            files: fs.files,
            manifest: fs.manifest,
        };
        root.files = current
            .files
            .iter()
            .map(|(p, c)| (p.clone(), sha256_hex(c.as_bytes())))
            .collect();
        root.manifest = sha256_hex(current.manifest.as_bytes());
        Ok(current)
    }

    pub fn receipt(&mut self) -> Result<ModuleReceipt, String> {
        let payload = serde_json::to_vec(&self.root).map_err(|e| format!("{e:?}"))?;
        if payload.len() > 512 * 1024 {
            return Err("A2X packages root resource budget".into());
        }
        self.inner
            .quota_guard(payload.len() as u64)
            .map_err(|e| format!("{e:?}"))?;
        let digest = sha256_hex(&payload);
        let key = QueryKey {
            layer: "AXM960",
            components: vec![
                ("schema".into(), PKG_A2X_SCHEMA.into()),
                ("digest".into(), digest.clone()),
            ],
        };
        if let Some(a) = self
            .inner
            .store
            .get_artifact("AXM960", &key)
            .map_err(|e| format!("{e:?}"))?
        {
            return Ok(ModuleReceipt {
                artifact: a.artifact_id,
                digest,
            });
        }
        self.inner.stats.persist += 1;
        self.inner.stats.bytes_persisted += payload.len() as u64;
        let artifact = self
            .inner
            .store
            .put_artifact(PutRequest {
                layer: "AXM960",
                key,
                kind: ChunkKind::SnapshotRecord,
                payload,
                edges: self
                    .root
                    .fragments
                    .iter()
                    .map(|r| (r.artifact.clone(), RefKind::RequiredData))
                    .collect(),
                project: None,
            })
            .map_err(|e| format!("{e:?}"))?;
        Ok(ModuleReceipt { artifact, digest })
    }

    pub fn restore<P: TargetProvider + 'static>(
        path: &Path,
        project: &str,
        owner: &str,
        quota: u64,
        provider: P,
        receipt: &ModuleReceipt,
        product: &PackageProduct,
    ) -> Result<Self, String> {
        let mut inner = A2xSession::open(path, project, owner, quota, provider)?;
        inner.profile = SourceProfile::TextRecord;
        inner.caps = product.bound.capabilities.clone();
        inner.source_sha256 = Some(product.bound.input_hash.clone());
        inner
            .ensure_target_supports()
            .map_err(|e| format!("{e:?}"))?;
        let a = inner
            .store
            .get_artifact_by_id_bounded(&receipt.artifact, 512 * 1024)
            .map_err(|e| format!("{e:?}"))?
            .ok_or("A2X packages root absent")?;
        if a.kind != ChunkKind::SnapshotRecord || sha256_hex(&a.payload) != receipt.digest {
            return Err("A2X packages root receipt mismatch".into());
        }
        let root: PkgRoot = serde_json::from_slice(&a.payload).map_err(|e| format!("{e:?}"))?;
        if root.schema != PKG_A2X_SCHEMA
            || root.source != product.bound.input_hash
            || root.owner != owner
            || root.backend != inner.current_backend()
            || root.caps != product.bound.capabilities
            || root.target_inputs
                != inner
                    .provider
                    .fragment_key_inputs(&sha256_hex(root.runtime.as_bytes()))
        {
            return Err("A2X packages producer/source/runtime identity mismatch".into());
        }
        let names: std::collections::BTreeSet<_> =
            root.fragments.iter().map(|f| f.name.clone()).collect();
        if names.len() != root.fragments.len()
            || names
                != product
                    .bound
                    .origins
                    .keys()
                    .filter(|k| !k.contains('.'))
                    .cloned()
                    .collect()
        {
            return Err("A2X packages declaration set mismatch".into());
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
            return Err("A2X packages required fragment closure mismatch".into());
        }
        let mut root = root;
        let view = SemanticView::from_checked(&product.compiled.checked).map_err(|e| format!("{e:?}"))?;
        let current = Self::assemble(&inner, &mut root, &product.bound, &view)?;
        Ok(Self {
            inner,
            root,
            current,
        })
    }

    pub fn stats(&self) -> &ProducerStats {
        self.inner.stats()
    }
}

fn read_pkg_fragments(inner: &A2xSession, root: &PkgRoot) -> Result<Vec<String>, String> {
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
            .ok_or("required A2X packages fragment absent")?;
        if a.artifact_id != r.artifact
            || a.kind != ChunkKind::Object
            || sha256_hex(&a.payload) != r.digest
        {
            return Err("A2X packages fragment identity mismatch".into());
        }
        text.push(String::from_utf8(a.payload).map_err(|e| format!("{e:?}"))?);
    }
    Ok(text)
}
