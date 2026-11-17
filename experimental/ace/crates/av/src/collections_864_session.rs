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
    ///
    /// PLAN-1030: the outer fingerprint map advances on the SAME success
    /// boundary as the inner session's accepted revision. A rejected
    /// revision keeps the previously admitted map, so re-submitting the
    /// same rejecting source is a genuine retry, never a "hit" that
    /// silently carries the old revision's code.
    pub fn warmup(&mut self, src: &str) -> Result<ApplyOutcome, String> {
        let units = Self::compile_units(src, &self.entry)?;
        let mut candidate: BTreeMap<String, u64> = BTreeMap::new();
        // CollectionsUpdateSession::apply_revision takes owned modules via
        // the submitted slice — rebuild the owned vector for the call.
        let mut owned: Vec<(&str, MirModule)> = Vec::new();
        for (name, u) in &units {
            candidate.insert(name.clone(), Self::fnv(format!("{u:?}").as_bytes()));
            owned.push((name.as_str(), u.clone()));
        }
        let outcome = self.inner.apply_revision(&owned)?;
        // COMMIT: the inner session installed this candidate, so the outer
        // map may now describe it.
        self.fps = candidate;
        self.edits_total += 1;
        self.edits_rebuilt += outcome.gate.rebuilt_units as u32;
        Ok(outcome)
    }

    /// Body edit: recompile, submit ONLY the functions whose content fp
    /// changed; unchanged functions ride the session's carry path (their
    /// production counters stay zero for this revision).
    ///
    /// PLAN-1030: the candidate map is built aside and installed only when
    /// `apply_revision` accepts. A rejected revision (compile/admission,
    /// link, guest trap, resource or persist failure) leaves the last
    /// admitted fingerprint map in place; a retry of the same rejecting
    /// source is therefore rejected again instead of being read as a
    /// pure hit over the previous revision's code.
    pub fn edit(&mut self, new_src: &str) -> Result<(ApplyOutcome, u32, u32), String> {
        let units = Self::compile_units(new_src, &self.entry)?;
        let mut candidate = self.fps.clone();
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
                    candidate.insert(name.clone(), fp);
                    owned.push((name.as_str(), u.clone()));
                }
            }
        }
        let outcome = self.inner.apply_revision(&owned)?;
        // COMMIT: only an accepted revision moves the outer map.
        self.fps = candidate;
        self.edits_total += 1;
        self.edits_rebuilt += changed;
        self.edits_carried += carried;
        Ok((outcome, changed, carried))
    }
}

#[cfg(test)]
mod audit1030 {
    //! PLAN-1030 AVTX-02: the outer source-fingerprint map is a transaction
    //! over the INNER session's accepted revision. A rejected revision must
    //! leave `fps`, the inner revision, the inner unit objects and the
    //! accepted counters exactly where they were; a retry of the same
    //! rejecting source must be rejected again (it is not a "hit" over the
    //! previous revision's code).
    use super::*;
    use std::sync::Arc;

    const BASE: &str = "type Item { name Str; code int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).code + helper()\n}\n";
    /// B: only the index changes — compiles and admits, traps at execution
    /// (`av.trap.list-index`, SEM-02's single index trap code).
    const B_TRAPPING: &str = "type Item { name Str; code int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(1).code + helper()\n}\n";
    const HELPER6: &str = "type Item { name Str; code int }\nfn helper() int { return 6 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).code + helper()\n}\n";
    const HELPER6_TRAPPING: &str = "type Item { name Str; code int }\nfn helper() int { return 6 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(1).code + helper()\n}\n";
    const BODY_EDIT: &str = "type Item { name Str; code int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).code + helper() + 0\n}\n";

    /// The accepted revision the inner session currently executes.
    fn accepted_rev(s: &CollectionsSourceSession) -> u64 {
        s.inner
            .snapshot()
            .expect("the accepted revision links")
            .revision()
    }

    /// The inner unit object identity (Arc) for one def id.
    fn unit_arc(s: &CollectionsSourceSession, name: &str) -> usize {
        s.inner
            .unit(name)
            .map(|fc| Arc::as_ptr(fc) as *const () as usize)
            .unwrap_or(0)
    }

    #[test]
    fn audit1030_avtx_02() {
        let mut s = CollectionsSourceSession::new(None, "d_main").expect("session");
        let cold = s.warmup(BASE).expect("cold admission");
        assert_eq!(cold.value, Some(6), "A = get(0) + helper(5)");
        assert_eq!(cold.gate.rebuilt_units, 2, "cold rebuilds both functions");
        let fps_a = s.fps.clone();
        let rev_a = accepted_rev(&s);
        assert_eq!((rev_a, s.edits_total), (1, 1));
        let helper_a = unit_arc(&s, "d_helper");
        let main_a = unit_arc(&s, "d_main");
        assert!(helper_a != 0 && main_a != 0, "both units are installed");

        // (1) B traps at EXECUTION (not at parse/verify/admission): the
        // inner build-aside transaction rejects it and keeps A installed.
        let rejected = s.edit(B_TRAPPING);
        let err = rejected.expect_err("B traps, so the revision is rejected");
        assert!(
            err.contains("ListIndex"),
            "the rejection is the controlled execution trap, not a front-end error: {err}"
        );
        assert_eq!(s.fps, fps_a, "a rejected revision must not move fps");
        assert_eq!(accepted_rev(&s), rev_a, "the inner revision stays A");
        assert_eq!(unit_arc(&s, "d_helper"), helper_a, "A's helper object");
        assert_eq!(unit_arc(&s, "d_main"), main_a, "A's main object");
        assert_eq!(
            (s.edits_total, s.edits_rebuilt, s.edits_carried),
            (1, 2, 0),
            "a rejected revision is not an accepted edit"
        );

        // (2) THE regression: retrying the same rejecting source must be
        // rejected again. The pre-fix code had already stored B's fp, read
        // the retry as a pure hit, submitted nothing and returned A's 6.
        let retried = s.edit(B_TRAPPING);
        let err = retried.expect_err("the retry of B is still B");
        assert!(err.contains("ListIndex"), "same controlled trap: {err}");
        assert_eq!(s.fps, fps_a, "the retry attempt did not move fps either");
        assert_eq!(accepted_rev(&s), rev_a);
        assert_eq!(unit_arc(&s, "d_main"), main_a);
        assert_eq!(s.edits_total, 1);

        // (3) a legitimate edit after the failures still admits and moves
        // both maps together: main rebuilds, helper rides the carry path.
        let (body, changed, carried) = s.edit(BODY_EDIT).expect("body edit admits");
        assert_eq!(body.value, Some(6));
        assert_eq!((changed, carried), (1, 1));
        assert_eq!(body.revision, rev_a + 1);
        assert_eq!(unit_arc(&s, "d_helper"), helper_a, "helper still carried");
        assert!(unit_arc(&s, "d_main") != main_a, "main was rebuilt");
        let fps_body = s.fps.clone();
        assert_ne!(fps_body, fps_a, "the accepted edit did move fps");

        // (4) ... and the following identical submission IS a hit.
        let (hit, changed_hit, carried_hit) = s.edit(BODY_EDIT).expect("hit");
        assert_eq!((changed_hit, carried_hit), (0, 2));
        assert_eq!(hit.value, Some(6));
        assert_eq!(hit.revision, rev_a + 2);

        // (5) from the new accepted base, B is still rejected and the base
        // (body-edit map) survives unchanged.
        assert!(
            s.edit(B_TRAPPING).is_err(),
            "B rejects from the new base too"
        );
        assert_eq!(s.fps, fps_body, "the new base's map survives the rejection");
        assert_eq!(accepted_rev(&s), rev_a + 2);
        assert_eq!(s.edits_total, 3);

        // (6) cold failure: a first warmup that traps leaves NO fingerprint
        // behind (the pre-fix code cleared then repopulated fps).
        let mut fresh = CollectionsSourceSession::new(None, "d_main").expect("session");
        assert!(fresh.warmup(B_TRAPPING).is_err(), "cold B traps");
        assert!(fresh.fps.is_empty(), "a rejected cold admit accepts no fp");
        let cold = fresh.warmup(BASE).expect("cold A admits");
        assert_eq!((cold.value, cold.gate.rebuilt_units), (Some(6), 2));
        assert_eq!(accepted_rev(&fresh), 1);
    }

    #[test]
    fn audit1030_avtx_02_warmup() {
        let mut s = CollectionsSourceSession::new(None, "d_main").expect("session");
        s.warmup(BASE).expect("cold A");
        let fps_a = s.fps.clone();
        let helper_a = unit_arc(&s, "d_helper");

        // The whole-source warmup rejects (helper body changed AND main
        // traps); the accepted A revision must survive intact.
        assert!(
            s.warmup(HELPER6_TRAPPING).is_err(),
            "the trapping warmup is rejected"
        );
        assert_eq!(s.fps, fps_a, "a rejected warmup keeps A's fingerprint map");
        assert_eq!(accepted_rev(&s), 1);
        assert_eq!(unit_arc(&s, "d_helper"), helper_a, "A's helper survived");
        assert_eq!(s.edits_total, 1, "the rejected warmup is not an edit");

        // The decision input: edit to the VALID helper-6 source. Because
        // the rejected warmup's fps was never accepted, the helper differs
        // from A and must rebuild — the result is 7. Pre-fix the stale
        // helper-6 fingerprint rode the carry path and returned 6.
        let (out, changed, carried) = s.edit(HELPER6).expect("helper edit admits");
        assert_eq!(changed, 1, "helper content changed against ACCEPTED A");
        assert_eq!(carried, 1, "main is unchanged against ACCEPTED A");
        assert_eq!(out.value, Some(7), "code(1) + helper(6)");
        assert_eq!(out.revision, 2);
        assert!(unit_arc(&s, "d_helper") != helper_a, "helper was rebuilt");
    }

    fn temp_store_root(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("plan1030-{tag}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn audit1030_avtx_02_persist() {
        // (A) COLD persist fault: the very first admission cannot reach its
        // store write, so the session must stay empty; clearing the fault
        // lets the SAME source admit through the real store. This is the
        // "clear the fault, then the same input succeeds" leg of the
        // contract — it is taken on the cold admission because the store's
        // A22 key identity refuses every later revision that re-submits an
        // already-persisted unit (see (B) and the Plan evidence note).
        let cold_root = temp_store_root("cold");
        std::fs::write(&cold_root, b"plan1030: not a managed store directory")
            .expect("occupy the store root with a file");
        let mut cold = CollectionsSourceSession::new(Some(&cold_root), "d_main").expect("session");
        let err = cold
            .warmup(BASE)
            .expect_err("the cold admission cannot persist");
        assert!(
            err.contains("open store"),
            "the persist stage failed: {err}"
        );
        assert!(cold.fps.is_empty(), "a rejected cold admit accepts no fp");
        assert_eq!(cold.edits_total, 0, "not an accepted edit");
        assert!(cold.inner.unit("d_main").is_none(), "no unit was installed");
        assert!(
            cold.warmup(BASE).is_err(),
            "the retry is still the same rejecting admission"
        );
        assert!(cold.fps.is_empty());

        std::fs::remove_file(&cold_root).expect("clear the persist fault");
        let admitted = cold
            .warmup(BASE)
            .expect("the cleared store admits the same source");
        assert_eq!(admitted.value, Some(6));
        assert_eq!(admitted.revision, 1);
        assert_eq!(admitted.production.store_writes, 2, "both units persisted");
        assert_eq!(cold.fps.len(), 2, "the accepted cold admit moved fps");
        assert_eq!(cold.edits_total, 1);
        let _ = std::fs::remove_dir_all(&cold_root);

        // (B) an EXISTING accepted revision + a late persist fault inside a
        // body edit: the whole attempt must be rejected without moving the
        // outer map, the inner revision or the installed objects.
        let root = temp_store_root("edit");
        let marker = root.join(ace_artifacts::store::MARKER_NAME);
        let mut s = CollectionsSourceSession::new(Some(&root), "d_main").expect("session");
        let coldrun = s
            .warmup(BASE)
            .expect("cold admission with a real managed store");
        assert_eq!(coldrun.value, Some(6));
        assert_eq!(coldrun.production.store_writes, 2, "both units persist");
        assert!(
            marker.is_file(),
            "managed store marker: {}",
            marker.display()
        );
        let fps_a = s.fps.clone();
        let helper_a = unit_arc(&s, "d_helper");
        let main_a = unit_arc(&s, "d_main");
        let rev_a = accepted_rev(&s);

        // PERSIST fault: an unknown marker version makes the managed store
        // refuse to open, so produce_unit fails at its store write stage
        // (after verify/encode/admit — a LATE failure inside the revision).
        std::fs::write(
            &marker,
            format!(
                "{{\"store_id\":\"plan1030-fault\",\"version\":{},\"created_epoch\":0}}",
                ace_artifacts::store::STORE_VERSION + 1
            ),
        )
        .expect("inject the persist fault");

        let err = s
            .edit(BODY_EDIT)
            .expect_err("the persist fault rejects the revision");
        assert!(
            err.contains("open store"),
            "the persist stage is the failure: {err}"
        );
        assert_eq!(s.fps, fps_a, "a failed persist must not move fps");
        assert_eq!(accepted_rev(&s), rev_a, "the inner revision stays");
        assert_eq!(unit_arc(&s, "d_helper"), helper_a);
        assert_eq!(unit_arc(&s, "d_main"), main_a);
        assert_eq!(s.edits_total, 1, "not an accepted edit");
        let attempt = s.inner.production_counters();
        assert_eq!(
            attempt.admitted, 1,
            "the attempt really ran the gates up to the persist write"
        );
        assert_eq!(attempt.store_writes, 0, "the persist write did not succeed");

        // The retry is the same rejecting revision — never a hit over A.
        let err = s.edit(BODY_EDIT).expect_err("the retry is still rejected");
        assert!(err.contains("open store"), "same persist failure: {err}");
        assert_eq!(s.fps, fps_a);
        assert_eq!(accepted_rev(&s), rev_a);

        let _ = std::fs::remove_dir_all(&root);
    }
}
