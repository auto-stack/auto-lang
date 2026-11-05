//! Retained module source context and the existing single-body transaction.
use super::*;
use ace_core::source::ast::Item;
use ace_core::source::modules::edit_module_function;
use ace_core::source::resolve::{reanchor_env_declarations, CheckedEnv};
use ace_core::source::{
    env_free_fn_index, prepare_module_candidate, resolve_env_with_supply, FnCandidate,
};
use std::collections::{BTreeMap, BTreeSet};

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
pub struct PreparedModuleBody {
    pub owner: String,
    pub candidate: FnCandidate,
}
pub struct PreparedModuleBatch {
    pub base_source: String,
    pub bound: BoundModules,
    pub bodies: Vec<PreparedModuleBody>,
    pub contexts: BTreeMap<String, String>,
    targets: BTreeMap<String, ModuleReceipt>,
}
impl PreparedModuleEdit {
    pub fn base_source(&self) -> &str {
        &self.base_source
    }
}
impl ModuleSession {
    /// One candidate source/contract revision, still unpublished. The physical
    /// owner set comes from a consumer's real provider closure. Contract readers
    /// come from the query's existing positive dependency pages, not a second
    /// call-graph/effect solver.
    pub fn prepare_batch(
        &mut self,
        edit: Option<(&str, &str, &str)>,
        contexts: BTreeMap<String, String>,
        physical_owners: &BTreeSet<String>,
        budget: FrontBudget,
    ) -> Result<PreparedModuleBatch, String> {
        let bound = match edit {
            Some((module, name, fragment)) => {
                edit_module_function(&self.bound, module, name, fragment, budget)
                    .map_err(|e| format!("{e:?}"))?
            }
            None => self.bound.clone(),
        };
        let old_contexts = self.query.declaration_contexts().clone();
        self.query
            .set_declaration_contexts(&bound.program, contexts.clone())
            .map_err(|e| e.to_string())?;
        let result = (|| {
            let changed: BTreeSet<_> = old_contexts
                .keys()
                .chain(contexts.keys())
                .filter(|name| old_contexts.get(*name) != contexts.get(*name))
                .cloned()
                .collect();
            let mut owners = self.query.declaration_readers(&changed);
            owners.extend(physical_owners.iter().cloned());
            if let Some((module, name, _)) = edit {
                let owner = bound
                    .origins
                    .iter()
                    .find(|(_, o)| o.0 == module && o.1 == name)
                    .ok_or("module batch edit owner absent")?
                    .0
                    .clone();
                owners.insert(owner);
            }
            let mut bodies = Vec::new();
            for owner in owners {
                let item = bound
                    .program
                    .items
                    .iter()
                    .find_map(|i| match i {
                        Item::Fn(f) if f.name.name == owner => Some(f),
                        _ => None,
                    })
                    .ok_or("module batch owner absent")?;
                let index = env_free_fn_index(&self.env, &owner)
                    .ok_or("module batch environment owner absent")?;
                let page = self
                    .query
                    .infer_body(&bound.program, &owner)
                    .map_err(|e| e.to_string())?;
                let ds = fact_diagnostics(&bound, &item.body, &page.facts);
                if !ds.is_empty() {
                    return Err(format!("{ds:?}"));
                }
                let mut supply = BodyTypeSupply::new();
                supply
                    .add_body(&item.body, &page.facts)
                    .map_err(|e| e.to_string())?;
                let candidate =
                    prepare_module_candidate(&mut self.env, item, index, &bound, &supply)
                        .map_err(|e| format!("{e:?}"))?;
                bodies.push(PreparedModuleBody { owner, candidate });
            }
            Ok(PreparedModuleBatch {
                base_source: self.bound.input_hash.clone(),
                bound,
                bodies,
                contexts,
                targets: self.targets.clone(),
            })
        })();
        // Candidate preparation never changes the live contract. Cached pages
        // may contain candidate results; ordinary keys/read checks keep those
        // from being used against the old inputs after rejection.
        self.query
            .set_declaration_contexts(&self.bound.program, old_contexts)
            .map_err(|e| e.to_string())?;
        result
    }

    pub fn validate_batch(&self, batch: &PreparedModuleBatch) -> Result<(), String> {
        if batch.base_source != self.bound.input_hash {
            return Err("stale module batch source".into());
        }
        let mut slots = BTreeSet::new();
        for body in &batch.bodies {
            body.candidate
                .validate(&self.env)
                .map_err(|e| format!("{e:?}"))?;
            if body.owner != body.candidate.slot_name() || !slots.insert(body.candidate.fn_index())
            {
                return Err("module batch body identity/duplicate".into());
            }
        }
        Ok(())
    }

    pub fn prepare_batch_receipt(
        &mut self,
        store: &mut ModuleStore,
        batch: &mut PreparedModuleBatch,
        targets: BTreeMap<String, ModuleReceipt>,
    ) -> Result<ModuleReceipt, String> {
        self.validate_batch(batch)?;
        let current = self.query.declaration_contexts().clone();
        self.query
            .set_declaration_contexts(&batch.bound.program, batch.contexts.clone())
            .map_err(|e| e.to_string())?;
        let result =
            store.prepare_context_targets(&batch.bound, &mut self.query, &self.env, &targets);
        self.query
            .set_declaration_contexts(&self.bound.program, current)
            .map_err(|e| e.to_string())?;
        let receipt = result?;
        batch.targets = targets;
        Ok(receipt)
    }

    pub fn commit_batch(&mut self, batch: PreparedModuleBatch) -> Result<(), String> {
        self.validate_batch(&batch)?;
        // Validate the new catalog before the infallible body swaps.
        self.query
            .set_declaration_contexts(&batch.bound.program, batch.contexts)
            .map_err(|e| e.to_string())?;
        FnCandidate::commit_batch(
            batch.bodies.into_iter().map(|b| b.candidate).collect(),
            &mut self.env,
        )
        .map_err(|e| format!("{e:?}"))?;
        reanchor_env_declarations(&mut self.env, &batch.bound.program)
            .map_err(|e| format!("{e:?}"))?;
        self.bound = batch.bound;
        self.targets = batch.targets;
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use ace_core::source::modules::MODULES_CAPABILITY;
    fn session() -> ModuleSession {
        let input = [
            ModuleInput {
                id: "std".into(),
                source: "pub fn answer() int { return 42 }".into(),
            },
            ModuleInput {
                id: "app".into(),
                source:
                    "use std.answer\nfn local() int { return 7 } fn main() int { return answer() }"
                        .into(),
            },
        ];
        let mut query = SourceQuerySession::new("pkg", "app");
        let product = compile_module_product(
            &input,
            "app",
            &[MODULES_CAPABILITY.into()],
            &mut query,
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        ModuleSession::from_product(&product, query).unwrap()
    }
    #[test]
    fn rejected_contract_candidate_does_not_hide_affected_readers_on_retry() {
        let mut session = session();
        let owner = session
            .bound
            .origins
            .iter()
            .find(|(_, o)| o.0 == "std")
            .unwrap()
            .0
            .clone();
        let contexts = BTreeMap::from([(owner.clone(), "a".repeat(64))]);
        let before = session.query.counters().solver_runs;
        let (cancelled, syntax) = ace_core::source::parser::capture_syntax_work(|| {
            session
                .prepare_batch(
                    None,
                    contexts.clone(),
                    &BTreeSet::new(),
                    FrontBudget::WORK_OWNED_64MIB,
                )
                .unwrap()
        });
        assert!(syntax.parsed_functions.is_empty());
        assert!(syntax.tokenized_ranges.is_empty());
        assert_eq!(session.query.counters().solver_runs - before, 2);
        assert_eq!(
            cancelled
                .bodies
                .iter()
                .map(|b| b.owner.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([owner.as_str(), "main"])
        );
        assert!(cancelled
            .bodies
            .iter()
            .all(|b| b.candidate.receipt().real_body_plans == 1));
        assert!(session.query.declaration_contexts().is_empty());
        drop(cancelled);
        let retry = session
            .prepare_batch(
                None,
                contexts.clone(),
                &BTreeSet::new(),
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        assert_eq!(
            retry.bodies.len(),
            2,
            "cached candidate facts never hide target invalidation after cancellation"
        );
        let stale = session
            .prepare_batch(
                None,
                contexts.clone(),
                &BTreeSet::new(),
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        session.commit_batch(retry).unwrap();
        assert!(session.validate_batch(&stale).is_err());
        assert_eq!(session.query.declaration_contexts(), &contexts);
        let no_change = session
            .prepare_batch(
                None,
                contexts,
                &BTreeSet::new(),
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        assert!(no_change.bodies.is_empty());
    }
    #[test]
    fn batch_duplicate_and_foreign_candidates_fail_before_any_swap() {
        let mut session = session();
        let first = session
            .prepare(
                "std",
                "answer",
                "pub fn answer() int { return 41 }",
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        let duplicate = session
            .prepare(
                "std",
                "answer",
                "pub fn answer() int { return 40 }",
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        assert!(FnCandidate::commit_batch(
            vec![first.candidate, duplicate.candidate],
            &mut session.env
        )
        .is_err());
        let current = session
            .prepare(
                "std",
                "answer",
                "pub fn answer() int { return 39 }",
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        assert_eq!(
            current.candidate.base_generation(),
            0,
            "duplicate rejection must not install the first body"
        );
        let mut foreign = self::session();
        assert!(current.candidate.validate(&foreign.env).is_err());
        let physical = BTreeSet::from([current.owner.clone()]);
        let batch = session
            .prepare_batch(
                None,
                BTreeMap::new(),
                &physical,
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        assert_eq!(batch.bodies.len(), 1);
        assert_eq!(batch.bodies[0].owner, current.owner);
        session.commit_batch(batch).unwrap();
        assert!(current.candidate.validate(&session.env).is_err());
        // A failed foreign validation did not publish into that environment.
        let control = foreign
            .prepare(
                "std",
                "answer",
                "pub fn answer() int { return 38 }",
                FrontBudget::WORK_OWNED_64MIB,
            )
            .unwrap();
        assert_eq!(control.candidate.base_generation(), 0);
    }
}
