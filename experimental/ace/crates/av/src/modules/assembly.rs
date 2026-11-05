//! Versioned assembly mode of the existing module target. All executable
//! production/admission continues through the original lowering/ABC path.
use super::*;
use ace_semantics_contract::assembly::{Consumer, ProviderInput, AV_PRODUCER};
use ace_source_query::{
    assembly::{
        entry_requirements, function_input_versions, manifest_contexts, ASSEMBLY_HOST,
        ASSEMBLY_PLATFORM,
    },
    modules::PreparedModuleBatch,
};

pub fn assembly_consumer(providers: Vec<ProviderInput>) -> Consumer {
    Consumer {
        producer: AV_PRODUCER.into(),
        abi: ace_av_core::ABC_SCHEMA.into(),
        runtime: format!(
            "{};{:?}",
            ace_av_core::AV_PROFILE,
            ace_av_core::contract::AV_SEMANTICS
        ),
        toolchain: format!(
            "{};{:?};{}",
            ace_av_core::MIR_SCHEMA,
            ace_av_core::contract::MIR_SEMANTICS,
            ace_av_core::PRODUCER_ID
        ),
        providers,
    }
}
pub(super) fn admit_manifest(bound: &BoundModules, manifest: &Manifest) -> Result<(), String> {
    manifest_contexts(bound, manifest)?;
    if !cfg!(all(windows, target_arch = "x86_64", target_env = "msvc"))
        || manifest.semantic().platform != ASSEMBLY_PLATFORM
    {
        return Err("AV assembly platform mismatch".into());
    }
    if manifest.semantic().host != ASSEMBLY_HOST {
        return Err("AV assembly host mismatch".into());
    }
    if manifest.consumer() != &assembly_consumer(manifest.consumer().providers.clone()) {
        return Err("AV assembly producer/ABI/runtime/toolchain mismatch".into());
    }
    Ok(())
}
impl ModuleAvTarget {
    pub fn cold_assembly(
        path: &Path,
        product: &ModuleProduct,
        manifest: &Manifest,
    ) -> Result<Self, String> {
        Self::cold_inner(path, product, Some(manifest))
    }
    pub fn restore_assembly(
        path: &Path,
        receipt: &ModuleReceipt,
        bound: &BoundModules,
        manifest: &Manifest,
    ) -> Result<Self, String> {
        Self::restore_inner(path, receipt, bound, Some(manifest))
    }
    /// Called on every run, including restored targets. No grant or callable
    /// host pointer is retained or serialized by this target.
    pub fn run_authorized(
        &self,
        authorize: &mut impl FnMut(&str, &[String]) -> Result<(), String>,
    ) -> Result<i32, String> {
        let manifest = self
            .root
            .assembly
            .as_ref()
            .ok_or("AV target has no assembly contract")?;
        authorize(&manifest.semantic().host, &self.root.requirements)?;
        self.execute_current()
    }
    pub fn prepare_batch(
        &mut self,
        batch: &PreparedModuleBatch,
        manifest: &Manifest,
    ) -> Result<ModuleAvEdit, String> {
        if self.root.assembly.is_none() {
            return Err("AV target has no assembly contract".into());
        }
        if self.root.source != batch.base_source || self.root.caps != batch.bound.capabilities {
            return Err("AV assembly source/capability base mismatch".into());
        }
        admit_manifest(&batch.bound, manifest)?;
        if batch.contexts != manifest_contexts(&batch.bound, manifest)? {
            return Err("AV assembly declaration context mismatch".into());
        }
        let mut root = self.root.clone();
        root.source = batch.bound.input_hash.clone();
        root.assembly = Some(manifest.clone());
        root.requirements = entry_requirements(&batch.bound, manifest)?;
        let revision = self.current.revision() + 1;
        let versions = function_input_versions(&batch.bound, manifest)?;
        let mut codes = self.current.code().to_vec();
        let mut slots = std::collections::BTreeSet::new();
        for body in &batch.bodies {
            if body.candidate.slot_name() != body.owner {
                return Err("AV assembly candidate owner mismatch".into());
            }
            let def = format!("d_{}", body.owner);
            let slot = root
                .units
                .iter()
                .position(|u| u.name == def)
                .ok_or("AV assembly owner absent")?;
            if !slots.insert(slot) {
                return Err("duplicate AV assembly body".into());
            }
            let context = versions
                .get(&body.owner)
                .ok_or("AV assembly context owner absent")?
                .clone();
            if !batch.body_edited(&body.owner) && root.units[slot].context == context {
                continue;
            }
            let parts = body.candidate.parts();
            let index = parts.bundle.modules[0]
                .defs
                .iter()
                .position(|d| d.id_text == def)
                .ok_or("AV assembly candidate def absent")?;
            ace_core::verify::verify_function_ref(&parts.bundle, index)
                .map_err(|e| format!("{e:?}"))?;
            self.lowered += 1;
            let unit = ace_lowering::lower_function_unit(
                &parts.bundle,
                &def,
                &parts.bundle.requires,
                revision,
            )
            .map_err(|e| format!("{e:?}"))?;
            let (record, code) = persist_unit_with_context(
                &self.store,
                &unit,
                span(&batch.bound, &def)?,
                &root.caps,
                revision,
                &context,
            )?;
            root.units[slot] = record;
            codes[slot] = code;
        }
        for unit in &root.units {
            let owner = unit
                .name
                .strip_prefix("d_")
                .ok_or("AV assembly def identity")?;
            if versions.get(owner) != Some(&unit.context) {
                return Err("AV assembly omitted affected owner".into());
            }
        }
        let snapshot = link(&root, &batch.bound, codes, revision)?;
        Ok(ModuleAvEdit {
            base: root_revision(&self.root),
            root,
            snapshot,
        })
    }
}

impl ace_source_query::assembly::AssemblyTarget for ModuleAvTarget {
    type Candidate = ModuleAvEdit;
    fn label(&self) -> &'static str {
        "av"
    }
    fn current_manifest(&self) -> Result<&Manifest, String> {
        self.root
            .assembly
            .as_ref()
            .ok_or_else(|| "AV target has no assembly contract".into())
    }
    fn prepare_assembly(
        &mut self,
        batch: &PreparedModuleBatch,
        manifest: &Manifest,
    ) -> Result<Self::Candidate, String> {
        self.prepare_batch(batch, manifest)
    }
    fn validate_assembly(&self, candidate: &Self::Candidate) -> Result<(), String> {
        self.validate(candidate)
    }
    fn commit_assembly(&mut self, candidate: Self::Candidate) -> Result<(), String> {
        self.commit(candidate)
    }
    fn current_receipt(&mut self) -> Result<ModuleReceipt, String> {
        self.receipt()
    }
    fn current_digest(&self) -> Result<String, String> {
        Ok(root_revision(&self.root))
    }
    fn candidate_receipt(&mut self, candidate: &Self::Candidate) -> Result<ModuleReceipt, String> {
        self.prepared_receipt(candidate)
    }
}
