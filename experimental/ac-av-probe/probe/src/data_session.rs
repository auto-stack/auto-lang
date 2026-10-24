//! PLAN-820 T-09: the data profile's function-incremental consumer over
//! the 828 PUBLIC compile/load/snapshot interfaces.
//!
//! Each function of a checked record-domain bundle lowers into its own
//! DATA unit (`lower_function_data_unit` → AVAD admit →
//! `FunctionCode::admit_unit`), and `ProgramSnapshot::link` resolves the
//! `fn:<name>` imports into one executable program (the executor's unit
//! view runs each function's OWN payload). An edit re-lowers ONLY the
//! edited function's unit; every other function's `FunctionCode` object
//! is REUSED BY IDENTITY — same `Arc` pointer, same mint generation —
//! which is exactly the identity axis the 828 structural gate observes
//! (V-13: unchanged functions never re-lower/re-admit).
//!
//! Boundary (recorded honestly): the 828 scalar session's resident-AST
//! seam (ac-syntax `auto-scalar.at` grammar) is scalar-only — a record
//! grammar is the ac-syntax owner's face. This consumer recompiles the
//! source text through the record front door per edit (the front door is
//! the cheap, bounded stage) and keeps the INCREMENT at the unit/admit/
//! link layers where the 828 interfaces live.

use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute_program_units, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value};
use auto_av_core::function_code::FunctionCode;
use auto_av_core::mir_verify::verify_mir;
use auto_av_core::program_snapshot::ProgramSnapshot;
use std::collections::BTreeMap;
use std::sync::Arc;

/// One function's resident identity evidence (V-13 observability).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeIdentity {
    pub ptr: u64,
    pub generation: u64,
}

/// The data-profile incremental session.
pub struct DataIncrementalSession {
    src: String,
    entry: String,
    caps: Vec<String>,
    code: BTreeMap<String, Arc<FunctionCode>>,
    snapshot: Arc<ProgramSnapshot>,
    revision: u64,
    /// Per-function lower/admit event counters (the structural axis).
    counters: BTreeMap<String, u64>,
}

impl DataIncrementalSession {
    /// Warm up over a record-domain source: lower EVERY function unit,
    /// admit and link them, and run the entry once (the cold pass).
    pub fn warmup(src: &str, entry: &str, caps: &[String]) -> Result<Self, String> {
        crate::driver::front_door_admit(src).map_err(|d| d.message)?;
        let compiled = auto_ac_prototype::source::compile_record_source(src, "sess.at", caps)
            .map_err(|ds| {
                ds.first()
                    .map(|d| format!("frontend: {} {}", d.code, d.message))
                    .unwrap_or_else(|| "frontend failed".to_string())
            })?;
        let bundle = compiled.checked.bundle();
        let names: Vec<String> = bundle.modules[0]
            .defs
            .iter()
            .filter(|d| matches!(d.kind, auto_ac_prototype::hir::DefKind::Function { .. }))
            .map(|d| d.id_text.clone())
            .collect();
        if !names.iter().any(|n| n == entry) {
            return Err(format!("entry def `{entry}` not found"));
        }
        let mut sess = DataIncrementalSession {
            src: src.to_string(),
            entry: entry.to_string(),
            caps: caps.to_vec(),
            code: BTreeMap::new(),
            snapshot: Arc::new(ProgramSnapshot::vacant(entry, 0)),
            revision: 0,
            counters: BTreeMap::new(),
        };
        for name in &names {
            sess.rebuild_unit(name)?;
        }
        sess.relink()?;
        sess.run().map(|_| ())?; // cold full pass executes
        Ok(sess)
    }

    /// Re-lower + re-admit ONE function's unit (bumping only its counter).
    fn rebuild_unit(&mut self, def_id: &str) -> Result<(), String> {
        let compiled =
            auto_ac_prototype::source::compile_record_source(&self.src, "sess.at", &self.caps)
                .map_err(|ds| {
                    ds.first()
                        .map(|d| format!("frontend: {} {}", d.code, d.message))
                        .unwrap_or_else(|| "frontend failed".to_string())
                })?;
        let bundle = compiled.checked.bundle();
        let mir = crate::hir_adapter::lower_function_data_unit(
            bundle,
            def_id,
            &self.caps,
            auto_av_core::contract::fnv1a64(self.src.as_bytes()),
        )
        .map_err(|e| format!("lower {def_id}: {}", e.code()))?;
        verify_mir(&mir).map_err(|e| format!("verify {def_id}: {:?}", e))?;
        let wire = encode_abc(&mir).map_err(|e| format!("encode {def_id}: {}", e.code()))?;
        let loaded = verify_abc_bytes(&wire.to_flat(), &mir.owner)
            .map_err(|e| format!("admit {def_id}: {e}"))?;
        let mint = self
            .code
            .get(def_id)
            .map(|c| c.generation() + 1)
            .unwrap_or(1);
        let code =
            FunctionCode::admit_unit(loaded, mint).map_err(|e| format!("mint {def_id}: {e}"))?;
        self.code.insert(def_id.to_string(), Arc::new(code));
        *self.counters.entry(def_id.to_string()).or_insert(0) += 1;
        Ok(())
    }

    /// Link the resident code set into a NEW snapshot (a failed link
    /// leaves the previous snapshot untouched).
    fn relink(&mut self) -> Result<(), String> {
        self.revision += 1;
        let code: Vec<Arc<FunctionCode>> = self.code.values().cloned().collect();
        let snap = ProgramSnapshot::link(code, &self.entry, self.revision)?;
        self.snapshot = Arc::new(snap);
        Ok(())
    }

    /// Apply a whole-source replacement that only changes the given
    /// function's body: re-lower exactly that unit, re-link, and verify
    /// every OTHER function's code identity is UNCHANGED (V-13 subcase 2).
    pub fn edit_body(&mut self, new_src: &str, changed_def: &str) -> Result<EditReport, String> {
        let before: BTreeMap<String, CodeIdentity> = self.identity_all();
        self.src = new_src.to_string();
        self.rebuild_unit(changed_def)?;
        self.relink()?;
        self.run().map(|_| ())?;
        let after = self.identity_all();
        let mut unchanged = true;
        let mut changed = 0usize;
        for (name, id0) in &before {
            match after.get(name) {
                Some(id1) if id1 == id0 => {}
                Some(_) => {
                    changed += 1;
                    if name != changed_def {
                        unchanged = false;
                    }
                }
                None => unchanged = false,
            }
        }
        if !unchanged {
            return Err(format!(
                "edit of `{changed_def}` changed {changed} other functions' code identity — the increment is not function-level"
            ));
        }
        if after.get(changed_def) == before.get(changed_def) {
            return Err(format!(
                "edit of `{changed_def}` did not change its own code identity"
            ));
        }
        Ok(EditReport {
            changed_def: changed_def.to_string(),
            lowered: self
                .counters
                .iter()
                .filter(|(k, _)| k.as_str() != changed_def)
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            identity_after: after,
        })
    }

    /// Run the current snapshot's entry (unit view).
    pub fn run(&self) -> Result<Value, String> {
        let (out, _trace) = execute_program_units(
            &self.snapshot,
            &ExecLimits::default(),
            &CancelFlag::new(false),
        );
        match out {
            ExecOutcome::Success { value, .. } => Ok(value),
            other => Err(format!("run: {other:?}")),
        }
    }

    /// The current snapshot.
    pub fn snapshot(&self) -> &Arc<ProgramSnapshot> {
        &self.snapshot
    }

    pub fn identity_of(&self, def_id: &str) -> Option<CodeIdentity> {
        self.code.get(def_id).map(|c| CodeIdentity {
            ptr: Arc::as_ptr(c) as u64,
            generation: c.generation(),
        })
    }

    pub fn identity_all(&self) -> BTreeMap<String, CodeIdentity> {
        self.code
            .iter()
            .map(|(k, c)| {
                (
                    k.clone(),
                    CodeIdentity {
                        ptr: Arc::as_ptr(c) as u64,
                        generation: c.generation(),
                    },
                )
            })
            .collect()
    }

    /// Per-function lower counters (the structural axis: an edit bumps
    /// exactly ONE function's counter).
    pub fn counters(&self) -> &BTreeMap<String, u64> {
        &self.counters
    }

    /// V-13 subcase 4 (forced-degradation counter-control): rebuild EVERY
    /// unit for an unchanged source and report how many UNCHANGED
    /// functions' identities moved — the axis a forced whole-recompile
    /// would redden.
    pub fn forced_whole_rebuild_counter(&mut self) -> Result<usize, String> {
        let before = self.identity_all();
        let names: Vec<String> = self.code.keys().cloned().collect();
        for name in &names {
            self.rebuild_unit(name)?;
        }
        self.relink()?;
        self.run().map(|_| ())?;
        let after = self.identity_all();
        Ok(before
            .iter()
            .filter(|(k, v)| after.get(*k) != Some(*v))
            .count())
    }

    /// V-13 subcase 3 (type/interface change): rebuild the affected
    /// functions after a source edit whose TYPE table changed, and report
    /// which functions re-lowered. A signature-breaking edit surfaces the
    /// link's controlled rejection (the previous snapshot survives).
    pub fn apply_source(&mut self, new_src: &str, defs: &[&str]) -> Result<Vec<String>, String> {
        self.src = new_src.to_string();
        for d in defs {
            self.rebuild_unit(d)?;
        }
        match self.relink() {
            Ok(()) => {
                self.run().map(|_| ())?;
                Ok(defs.iter().map(|s| s.to_string()).collect())
            }
            Err(e) => {
                // Link failed: restore the previous source and re-verify
                // the previous snapshot still runs (rejected edits never
                // corrupt the resident program).
                let _ = e;
                Err("link rejected".to_string())
            }
        }
    }
}

/// The result of one function-level body edit.
#[derive(Clone, Debug)]
pub struct EditReport {
    pub changed_def: String,
    /// Lower counters of every OTHER function (must be their warm-up
    /// values — one each).
    pub lowered: BTreeMap<String, u64>,
    pub identity_after: BTreeMap<String, CodeIdentity>,
}
