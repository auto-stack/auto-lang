//! Explicit finite Rust assembly mode over the original target-neutral session.
//! Semantic-plan keys stay target-neutral; the new context is a versioned input
//! of physical fragment keys only. No AV dependency is introduced here.
use super::*;
use ace_semantics_contract::assembly::{Consumer, ProviderInput, RUST_PRODUCER};
use ace_source_query::{
    assembly::{
        entry_requirements, function_input_version, manifest_contexts, ASSEMBLY_HOST,
        ASSEMBLY_PLATFORM,
    },
    modules::PreparedModuleBatch,
};

pub fn assembly_consumer(
    provider: &dyn TargetProvider,
    runtime: &str,
    providers: Vec<ProviderInput>,
) -> Result<Consumer, String> {
    let d = provider.descriptor();
    if d.lang != "rust" || d.backend != "auto-a2x-rust" {
        return Err("A2X assembly requires the finite Rust producer".into());
    }
    let runtime_hash = sha256_hex(runtime.as_bytes());
    // The emitted-source toolchain identity is the current provider's exact
    // language/edition/backend/printer recipe. External executable certification
    // remains owned by the existing Rust consumer; no compiler executable is
    // persisted in this source/fragment snapshot.
    let toolchain = sha256_hex(
        &serde_json::to_vec(&provider.fragment_key_inputs(&runtime_hash))
            .map_err(|e| e.to_string())?,
    );
    Ok(Consumer {
        producer: RUST_PRODUCER.into(),
        abi: format!("{}/{}/{}", d.lang, d.standard, d.backend_version),
        runtime: runtime_hash,
        toolchain,
        providers,
    })
}
pub(super) fn admit_manifest(
    bound: &BoundModules,
    manifest: &Manifest,
    provider: &dyn TargetProvider,
    runtime: &str,
) -> Result<(), String> {
    manifest_contexts(bound, manifest)?;
    if !cfg!(all(windows, target_arch = "x86_64", target_env = "msvc"))
        || manifest.semantic().platform != ASSEMBLY_PLATFORM
    {
        return Err("A2X assembly platform mismatch".into());
    }
    if manifest.semantic().host != ASSEMBLY_HOST {
        return Err("A2X assembly host mismatch".into());
    }
    if manifest.consumer()
        != &assembly_consumer(provider, runtime, manifest.consumer().providers.clone())?
    {
        return Err("A2X assembly producer/ABI/runtime/toolchain mismatch".into());
    }
    Ok(())
}
impl ModuleA2xTarget {
    pub fn consumer_for_product(
        product: &ModuleProduct,
        provider: &dyn TargetProvider,
        providers: Vec<ProviderInput>,
    ) -> Result<Consumer, String> {
        let view =
            SemanticView::from_checked(&product.compiled.checked).map_err(|e| format!("{e:?}"))?;
        let runtime = provider
            .runtime_source(&view, Family::Scalar)
            .map_err(|e| format!("{e:?}"))?;
        assembly_consumer(provider, &runtime, providers)
    }
    pub fn cold_assembly<P: TargetProvider + 'static>(
        path: &Path,
        project: &str,
        owner: &str,
        quota: u64,
        provider: P,
        product: &ModuleProduct,
        manifest: &Manifest,
    ) -> Result<Self, String> {
        Self::cold_inner(
            path,
            project,
            owner,
            quota,
            provider,
            product,
            Some(manifest),
        )
    }
    pub fn restore_assembly<P: TargetProvider + 'static>(
        path: &Path,
        project: &str,
        owner: &str,
        quota: u64,
        provider: P,
        receipt: &ModuleReceipt,
        bound: &BoundModules,
        manifest: &Manifest,
    ) -> Result<Self, String> {
        Self::restore_inner(
            path,
            project,
            owner,
            quota,
            provider,
            receipt,
            bound,
            Some(manifest),
        )
    }
    pub fn run_authorized<T>(
        &self,
        authorize: &mut impl FnMut(&str, &[String]) -> Result<(), String>,
        run: impl FnOnce(&Assembly) -> Result<T, String>,
    ) -> Result<T, String> {
        let manifest = self
            .root
            .assembly
            .as_ref()
            .ok_or("A2X target has no assembly contract")?;
        authorize(&manifest.semantic().host, &self.root.requirements)?;
        run(&self.current)
    }
    pub fn prepare_batch(
        &mut self,
        batch: &PreparedModuleBatch,
        manifest: &Manifest,
    ) -> Result<ModuleA2xEdit, String> {
        if self.root.assembly.is_none() {
            return Err("A2X target has no assembly contract".into());
        }
        if self.root.source != batch.base_source || self.root.caps != batch.bound.capabilities {
            return Err("A2X assembly source/capability base mismatch".into());
        }
        admit_manifest(
            &batch.bound,
            manifest,
            &*self.inner.provider,
            &self.root.runtime,
        )?;
        if batch.contexts != manifest_contexts(&batch.bound, manifest)? {
            return Err("A2X assembly declaration context mismatch".into());
        }
        let mut root = self.root.clone();
        root.source = batch.bound.input_hash.clone();
        root.assembly = Some(manifest.clone());
        root.requirements = entry_requirements(&batch.bound, manifest)?;
        let mut seen = std::collections::BTreeSet::new();
        for body in &batch.bodies {
            if body.candidate.slot_name() != body.owner || !seen.insert(body.owner.clone()) {
                return Err("A2X assembly duplicate/foreign owner".into());
            }
            let parts = body.candidate.parts();
            let def = format!("d_{}", body.owner);
            let index = parts.bundle.modules[0]
                .defs
                .iter()
                .position(|d| d.id_text == def)
                .ok_or("A2X assembly candidate def absent")?;
            let cred = verify_function_ref(&parts.bundle, index).map_err(|e| format!("{e:?}"))?;
            self.inner.stats.verify += 1;
            let view = SemanticView::from_parts(parts, &cred).map_err(|e| format!("{e:?}"))?;
            let f = view
                .functions()
                .first()
                .ok_or("empty A2X assembly candidate")?;
            if f.source_name != body.owner {
                return Err("A2X assembly candidate owner mismatch".into());
            }
            let runtime = self
                .inner
                .provider
                .runtime_source(&view, Family::Scalar)
                .map_err(|e| format!("{e:?}"))?;
            if runtime != root.runtime {
                return Err("A2X assembly runtime changed outside body contract".into());
            }
            let sem_fp = semantic_fingerprint(&view, f).map_err(|e| format!("{e:?}"))?;
            let context = function_input_version(&batch.bound, manifest, &body.owner)?;
            let runtime_hash = sha256_hex(runtime.as_bytes());
            self.inner
                .produce_fragment_with_context(
                    &view,
                    f,
                    &sem_fp,
                    &runtime_hash,
                    false,
                    Some(&context),
                )
                .map_err(|e| format!("{e:?}"))?;
            let key =
                self.inner
                    .fragment_key_with_context(f, &sem_fp, &runtime_hash, Some(&context));
            let a = self
                .inner
                .store
                .get_artifact(self.inner.layer_fragment().id, &key)
                .map_err(|e| format!("{e:?}"))?
                .ok_or("A2X assembly produced fragment absent")?;
            let r = root
                .fragments
                .iter_mut()
                .find(|r| r.name == body.owner)
                .ok_or("A2X assembly owner absent")?;
            r.sem_fp = sem_fp;
            r.artifact = a.artifact_id;
            r.digest = sha256_hex(&a.payload);
            r.context = context;
        }
        for r in &root.fragments {
            if r.context != function_input_version(&batch.bound, manifest, &r.name)? {
                return Err("A2X assembly omitted affected owner".into());
            }
        }
        let text = read_fragments(&self.inner, &root)?;
        let current = assemble(&self.inner, &root, &batch.bound, &text)?;
        seal_files(&mut root, &current);
        Ok(ModuleA2xEdit {
            base: root_revision(&self.root),
            root,
            current,
        })
    }
}
