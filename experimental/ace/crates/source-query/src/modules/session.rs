//! Retained module source context and the existing single-body transaction.
use super::*;
use ace_core::source::ast::Item;
use ace_core::source::modules::edit_module_function;
use ace_core::source::resolve::{reanchor_env_declarations, CheckedEnv};
use ace_core::source::{
    env_free_fn_index, prepare_module_candidate, resolve_env_with_supply, FnCandidate,
};

pub struct ModuleSession {
    pub bound: BoundModules,
    pub query: SourceQuerySession,
    env: CheckedEnv,
    pub targets: std::collections::BTreeMap<String, ModuleReceipt>,
}
pub struct PreparedModuleEdit {
    base_source: String,
    pub bound: BoundModules,
    pub owner: String,
    pub candidate: FnCandidate,
    targets: std::collections::BTreeMap<String, ModuleReceipt>,
}
impl PreparedModuleEdit {
    pub fn base_source(&self) -> &str {
        &self.base_source
    }
}
impl ModuleSession {
    pub fn from_product(
        product: &ModuleProduct,
        query: SourceQuerySession,
    ) -> Result<Self, String> {
        let env = resolve_env_with_supply(
            &product.bound.program,
            &product.bound.capabilities,
            SourceProfile::Scalar,
            Some(&product.supply),
        )
        .map_err(|e| format!("{e:?}"))?;
        Ok(Self {
            bound: product.bound.clone(),
            query,
            env,
            targets: std::collections::BTreeMap::new(),
        })
    }
    pub fn from_recovery(recovery: ModuleRecovery) -> Self {
        Self {
            bound: recovery.bound,
            query: recovery.session,
            env: recovery.env,
            targets: recovery.targets,
        }
    }
    pub fn prepare(
        &mut self,
        module: &str,
        name: &str,
        fragment: &str,
        budget: FrontBudget,
    ) -> Result<PreparedModuleEdit, String> {
        let bound = edit_module_function(&self.bound, module, name, fragment, budget)
            .map_err(|e| format!("{e:?}"))?;
        let owner = bound
            .origins
            .iter()
            .find(|(_, origin)| origin.0 == module && origin.1 == name)
            .map(|(n, _)| n.clone())
            .ok_or("module edit owner absent")?;
        let item = bound
            .program
            .items
            .iter()
            .find_map(|i| match i {
                Item::Fn(f) if f.name.name == owner => Some(f),
                _ => None,
            })
            .ok_or("module edit item absent")?;
        let index =
            env_free_fn_index(&self.env, &owner).ok_or("module edit environment owner absent")?;
        let page = self
            .query
            .infer_body(&bound.program, &owner)
            .map_err(|e| e.to_string())?;
        let mut supply = BodyTypeSupply::new();
        let ds = fact_diagnostics(&bound, &item.body, &page.facts);
        if !ds.is_empty() {
            return Err(format!("{ds:?}"));
        }
        supply
            .add_body(&item.body, &page.facts)
            .map_err(|e| e.to_string())?;
        let candidate = prepare_module_candidate(&mut self.env, item, index, &bound, &supply)
            .map_err(|ds| {
                format!(
                    "{:?}",
                    ds.into_iter().map(|d| bound.locate(d)).collect::<Vec<_>>()
                )
            })?;
        Ok(PreparedModuleEdit {
            base_source: self.bound.input_hash.clone(),
            bound,
            owner,
            candidate,
            targets: self.targets.clone(),
        })
    }
    pub fn prepare_receipt(
        &mut self,
        store: &mut ModuleStore,
        edit: &PreparedModuleEdit,
    ) -> Result<ModuleReceipt, String> {
        edit.candidate
            .validate(&self.env)
            .map_err(|e| format!("{e:?}"))?;
        store.prepare_context_targets(&edit.bound, &mut self.query, &self.env, &edit.targets)
    }
    pub fn prepare_current_receipt(
        &mut self,
        store: &mut ModuleStore,
        targets: &std::collections::BTreeMap<String, ModuleReceipt>,
    ) -> Result<ModuleReceipt, String> {
        store.prepare_context_targets(&self.bound, &mut self.query, &self.env, targets)
    }
    pub fn prepare_receipt_targets(
        &mut self,
        store: &mut ModuleStore,
        edit: &mut PreparedModuleEdit,
        targets: std::collections::BTreeMap<String, ModuleReceipt>,
    ) -> Result<ModuleReceipt, String> {
        self.validate(edit)?;
        let root =
            store.prepare_context_targets(&edit.bound, &mut self.query, &self.env, &targets)?;
        edit.targets = targets;
        Ok(root)
    }
    /// Called under the same exclusive host transaction after targets and the
    /// source root are ready. Candidate validation occurs before any publish.
    pub fn validate(&self, edit: &PreparedModuleEdit) -> Result<(), String> {
        edit.candidate
            .validate(&self.env)
            .map_err(|e| format!("{e:?}"))
    }
    pub fn commit(&mut self, edit: PreparedModuleEdit) -> Result<(), String> {
        self.validate(&edit)?;
        edit.candidate
            .commit(&mut self.env)
            .map_err(|e| format!("{e:?}"))?;
        reanchor_env_declarations(&mut self.env, &edit.bound.program)
            .map_err(|e| format!("{e:?}"))?;
        self.bound = edit.bound;
        self.targets = edit.targets;
        Ok(())
    }
}
