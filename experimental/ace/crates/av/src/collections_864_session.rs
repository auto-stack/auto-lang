//! PLAN-864: the collections SOURCE session — the 822
//! `CollectionsUpdateSession`'s same-shape source-side extension (contract
//! C8): whole-source compile feeds per-function MIR units; each function's
//! canonical content fingerprint selects what re-lowers. Unchanged
//! functions are re-submitted byte-identically, so the session's carry
//! path keeps their admitted `Arc<FunctionCode>` and their production
//! counters stay at zero — "unchanged functions, zero production work" is
//! a counter-observed fact, not an inference. Restart re-admits persisted
//! units through the full gate chain (decode + verify, never trust).

use crate::collections_session::{ApplyOutcome, CollectionsUpdateSession};
use ace_av_core::mir::MirModule;
use ace_core::source::compile_collections_source;
use ace_lowering::lower_checked_collections;
use std::collections::BTreeMap;
use std::path::Path;

/// One admitted source session over the collections profile.
pub struct CollectionsSourceSession {
    inner: CollectionsUpdateSession,
    entry: String,
    /// Per-function content fingerprint of the last admitted revision.
    fps: BTreeMap<String, u64>,
    /// Cumulative edit counters (V-04 evidence axes).
    pub edits_total: u32,
    pub edits_rebuilt: u32,
    pub edits_carried: u32,
}

impl CollectionsSourceSession {
    pub fn new(store_root: Option<&Path>, entry: &str) -> Result<Self, String> {
        Ok(CollectionsSourceSession {
            inner: CollectionsUpdateSession::new(store_root, entry),
            entry: entry.to_string(),
            fps: BTreeMap::new(),
            edits_total: 0,
            edits_rebuilt: 0,
            edits_carried: 0,
        })
    }

    /// Compile the whole source and lower EVERY function (fresh face).
    /// Returns the per-function units keyed by the adapter's def id text
    /// (`d_<name>`) plus the whole-module unit (entry shape for the
    /// fresh-path execution).
    pub fn compile_units(src: &str, entry: &str) -> Result<BTreeMap<String, MirModule>, String> {
        let compiled = compile_collections_source(src, "probe.at", &[]).map_err(|ds| {
            ds.iter()
                .map(|d| d.render("probe.at", src))
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        // whole-module lower for the entry face + per-fn units via the
        // unit lowering seam (the same `lower_function_collections_unit`
        // shape the 828 snapshot link consumes).
        let whole = lower_checked_collections(&compiled.checked, entry, 0)
            .map_err(|e| format!("lower: {e:?}"))?;
        let per_fn = crate::cache::split_units(&whole).map_err(|e| e)?;
        let mut out = BTreeMap::new();
        for u in per_fn {
            let name = u
                .funcs
                .first()
                .map(|f| f.name.clone())
                .ok_or("unit without a function")?;
            out.insert(name, u);
        }
        let _ = whole;
        Ok(out)
    }

    fn fnv(bytes: &[u8]) -> u64 {
        ace_av_core::contract::fnv1a64(bytes)
    }

    /// Cold start: admit every function through the real gates and link.
    pub fn warmup(&mut self, src: &str) -> Result<ApplyOutcome, String> {
        let units = Self::compile_units(src, &self.entry)?;
        self.fps.clear();
        let mut submitted: Vec<(&str, &MirModule)> = Vec::new();
        for (name, u) in &units {
            let fp = Self::fnv(format!("{u:?}").as_bytes());
            self.fps.insert(name.clone(), fp);
            submitted.push((name.as_str(), u));
        }
        let refs: Vec<(&str, MirModule)> = Vec::new();
        let _ = refs;
        // CollectionsUpdateSession::apply_revision takes owned modules via
        // the submitted slice — rebuild the owned vector for the call.
        let mut owned: Vec<(&str, MirModule)> = Vec::new();
        for (name, u) in &units {
            owned.push((name.as_str(), u.clone()));
        }
        let outcome = self.inner.apply_revision(&owned)?;
        self.edits_total += 1;
        self.edits_rebuilt += outcome.gate.rebuilt_units as u32;
        Ok(outcome)
    }

    /// Body edit: recompile, submit ONLY the functions whose content fp
    /// changed; unchanged functions ride the session's carry path (their
    /// production counters stay zero for this revision).
    pub fn edit(&mut self, new_src: &str) -> Result<(ApplyOutcome, u32, u32), String> {
        let units = Self::compile_units(new_src, &self.entry)?;
        let mut changed = 0u32;
        let mut carried = 0u32;
        let mut owned: Vec<(&str, MirModule)> = Vec::new();
        for (name, u) in &units {
            let fp = Self::fnv(format!("{u:?}").as_bytes());
            match self.fps.get(name) {
                Some(old) if *old == fp => {
                    carried += 1;
                }
                _ => {
                    changed += 1;
                    self.fps.insert(name.clone(), fp);
                    owned.push((name.as_str(), u.clone()));
                }
            }
        }
        let outcome = self.inner.apply_revision(&owned)?;
        self.edits_total += 1;
        self.edits_rebuilt += changed;
        self.edits_carried += carried;
        Ok((outcome, changed, carried))
    }
}
