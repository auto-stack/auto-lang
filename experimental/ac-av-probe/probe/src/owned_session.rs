//! PLAN-821 R1-F07/R2-F07': the REAL owned incremental session — a
//! production install channel for `av.owned/1` over the D828 interfaces
//! (`lower_function_owned_unit` → `FunctionCode::admit_unit` →
//! `ProgramSnapshot::link` → `execute_program`).
//!
//! R2 hardening (review-r2 F821-R1-07 close):
//! - TRANSACTIONAL installs: `edit_batch` builds the ENTIRE candidate
//!   aside (frontend compile → invalidation detection → per-unit
//!   lower/verify/encode/admit → checked LINK of the full next resident
//!   set → entry RUN check) and only then PUBLISHES atomically. ANY
//!   failure — frontend, lower, verify, admit, link, or the run oracle —
//!   restores the source, revision, identity, residents, production
//!   counters AND the executable to the PRIOR revision exactly (a
//!   rejected backend-unsupported candidate keeps the old program
//!   runnable, never a panic on a half-updated resident set).
//! - The formal `incremental_gate` now covers the WHOLE production
//!   surface: per-revision FRONTEND compiles (the real whole-module
//!   front-end the hot path still rides — the counter makes it visible,
//!   not hidden), per-unit lower/encode/admit production, object
//!   identity (Arc::ptr_eq) and mint GENERATION moves on untouched
//!   residents, and the resident SET membership. A forced or silent
//!   whole recompute, an object swap without a mint bump, or a missed
//!   invalidation reddens the SAME gate.
//!
//! What makes this the real entry (vs. a test wrapper): ONE frontend
//! compile per revision; per-function identity from the checked HIR
//! (span-free — position shifts do not invalidate untouched bodies);
//! content-precise invalidation (own body changed, or the module's TYPE
//! CONTEXT changed — a signature/layout change stales every unit's
//! module-wide import table, the exact V-13(3) semantics); production
//! counters incremented by THIS code at the real production points.

use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::{fnv1a64, ExecOutcome};
use auto_av_core::function_code::FunctionCode;
use auto_av_core::program_snapshot::ProgramSnapshot;
use auto_av_core::{abc_verify, mir_verify};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Real production counters for one function unit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Production {
    pub lower: u32,
    pub encode: u32,
    pub admit: u32,
}

impl Production {
    fn bump(&mut self) {
        self.lower += 1;
        self.encode += 1;
        self.admit += 1;
    }
}

/// The checked projection one revision is keyed on: the module's type
/// context fingerprint plus each function def's span-free body
/// fingerprint.
#[derive(Clone, Debug)]
struct RevisionIdentity {
    type_ctx: u64,
    bodies: BTreeMap<String, u64>,
}

fn strip_spans(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("span: Span { start: ") {
        out.push_str(&rest[..i]);
        let after = &rest[i..];
        if let Some(close) = after.find('}') {
            rest = &after[close + 1..];
        } else {
            out.push_str(after);
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn identity_of(checked: &auto_ac_prototype::verify::CheckedModule) -> RevisionIdentity {
    let m = checked
        .bundle()
        .modules
        .first()
        .cloned()
        .unwrap_or_else(|| panic!("checked module without a module payload"));
    let type_ctx = fnv1a64(strip_spans(&format!("{:?}", m.types)).as_bytes());
    let mut bodies = BTreeMap::new();
    for d in &m.defs {
        if let auto_ac_prototype::hir::DefKind::Function { ty, body, .. } = &d.kind {
            // `body` is a BodyId INDEX — the CONTENT lives in the bodies
            // table; the fingerprint hashes the span-free content.
            let content = m
                .bodies
                .get(body.0 as usize)
                .map(|b| strip_spans(&format!("{b:?}")))
                .unwrap_or_default();
            bodies.insert(
                d.id_text.clone(),
                fnv1a64(format!("{ty:?}|{content}").as_bytes()),
            );
        }
    }
    RevisionIdentity { type_ctx, bodies }
}

/// The owned incremental session.
pub struct OwnedFunctionSession {
    src: String,
    entry: String,
    caps: Vec<String>,
    /// Frontend compiles issued (whole-module, the real production point).
    pub frontends: u32,
    code: BTreeMap<String, Arc<FunctionCode>>,
    production: BTreeMap<String, Production>,
    identity: RevisionIdentity,
    revision: u64,
    last_invalidated: Vec<String>,
}

impl OwnedFunctionSession {
    /// Cold warm-up: one frontend compile, every unit admitted at
    /// revision 1.
    pub fn warmup(src: &str, entry: &str, caps: &[String]) -> Result<Self, String> {
        let mut s = OwnedFunctionSession {
            src: src.to_string(),
            entry: entry.to_string(),
            caps: caps.to_vec(),
            frontends: 0,
            code: BTreeMap::new(),
            production: BTreeMap::new(),
            identity: RevisionIdentity {
                type_ctx: 0,
                bodies: BTreeMap::new(),
            },
            revision: 1,
            last_invalidated: Vec::new(),
        };
        let (checked, ident) = s.compile()?;
        // Build-aside then publish (the same transaction as edits).
        let mut code = BTreeMap::new();
        let mut production = BTreeMap::new();
        s.produce_into(
            &checked,
            &ident.bodies.keys().cloned().collect::<Vec<_>>(),
            &mut code,
            &mut production,
        )?;
        let linked = ProgramSnapshot::link(code.values().cloned().collect(), &s.entry, s.revision)
            .map_err(|e| format!("warmup link: {e}"))?;
        let (out, _) = execute_program(&linked, &ExecLimits::default(), &CancelFlag::new(false));
        if !matches!(out, ExecOutcome::Success { .. }) {
            return Err(format!("warmup run: {out:?}"));
        }
        s.code = code;
        s.production = production;
        s.identity = ident;
        s.last_invalidated = s.code.keys().cloned().collect();
        Ok(s)
    }

    /// ONE real frontend compile of the CURRENT source.
    fn compile(
        &mut self,
    ) -> Result<(auto_ac_prototype::verify::CheckedModule, RevisionIdentity), String> {
        let src = self.src.clone();
        self.compile_of(&src)
    }

    fn compile_of(
        &mut self,
        src: &str,
    ) -> Result<(auto_ac_prototype::verify::CheckedModule, RevisionIdentity), String> {
        self.frontends += 1;
        let compiled =
            auto_ac_prototype::source::compile_text_record_source(src, "session.at", &self.caps)
                .map_err(|ds| {
                    ds.first()
                        .map(|d| format!("{} {}", d.code, d.message))
                        .unwrap_or_else(|| "frontend".to_string())
                })?;
        let ident = identity_of(&compiled.checked);
        Ok((compiled.checked, ident))
    }

    /// Lower/verify/encode/admit ONE unit into the ASIDE tables (the
    /// production points all run; nothing resident is touched).
    fn produce_unit_aside(
        &self,
        checked: &auto_ac_prototype::verify::CheckedModule,
        def_id: &str,
        revision: u64,
    ) -> Result<(String, Arc<FunctionCode>, Production), String> {
        let bundle = checked.bundle();
        let snap = fnv1a64(self.src.as_bytes());
        let unit = crate::hir_adapter::lower_function_owned_unit(&bundle, def_id, &self.caps, snap)
            .map_err(|e| format!("lower {def_id}: {}", e.code()))?;
        mir_verify::verify_mir(&unit).map_err(|e| format!("verify {def_id}: {e:?}"))?;
        let wire = encode_abc(&unit).map_err(|e| format!("encode {def_id}: {}", e.code()))?;
        let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &unit.owner)
            .map_err(|e| format!("abc admit {def_id}: {e:?}"))?;
        let _ = decode_abc(&wire.to_flat());
        let fc = FunctionCode::admit_unit(loaded, revision)
            .map_err(|e| format!("admit {def_id}: {e}"))?;
        Ok((
            def_id.to_string(),
            Arc::new(fc),
            Production {
                lower: 1,
                encode: 1,
                admit: 1,
            },
        ))
    }

    /// Produce a SET of units aside (shared helper for warmup/edits).
    fn produce_into(
        &self,
        checked: &auto_ac_prototype::verify::CheckedModule,
        names: &[String],
        code: &mut BTreeMap<String, Arc<FunctionCode>>,
        production: &mut BTreeMap<String, Production>,
    ) -> Result<(), String> {
        for n in names {
            let (name, fc, prod) = self.produce_unit_aside(checked, n, self.revision)?;
            code.insert(name, fc);
            production.insert(n.clone(), prod);
        }
        Ok(())
    }

    /// Apply ONE source edit (see [`Self::edit_batch`]).
    pub fn edit(&mut self, start: u64, end: u64, insert: &[u8]) -> Result<Vec<String>, String> {
        self.edit_batch(&[(start, end, insert.to_vec())])
    }

    /// Apply a BATCH of edits ATOMICALLY (all ranges against the CURRENT
    /// source, applied back-to-front; ONE frontend compile after the
    /// batch). The ENTIRE candidate installs aside — invalidation
    /// detection, per-unit lower/verify/encode/admit, the checked LINK of
    /// the full next resident set and an entry RUN check — and publishes
    /// only when EVERY stage succeeds. ANY failure (including a backend
    /// `lower.*` refusal of a frontend-legal candidate) restores the
    /// source, revision, identity, residents, production counters and
    /// keeps the PRIOR program runnable.
    pub fn edit_batch(&mut self, edits: &[(u64, u64, Vec<u8>)]) -> Result<Vec<String>, String> {
        let mut ranges: Vec<(u64, u64, &[u8])> = edits
            .iter()
            .map(|(a, b, i)| (*a, *b, i.as_slice()))
            .collect();
        ranges.sort_by_key(|(a, _, _)| std::cmp::Reverse(*a));
        let mut next = self.src.clone();
        for (start, end, insert) in ranges {
            let bytes = next.clone().into_bytes();
            if (start as usize) > bytes.len() || (end as usize) > bytes.len() || start > end {
                return Err(format!("edit range {start}..{end} out of bounds"));
            }
            let mut patched = String::from_utf8_lossy(&bytes[..start as usize]).into_owned();
            patched.push_str(&String::from_utf8_lossy(insert));
            patched.push_str(&String::from_utf8_lossy(&bytes[end as usize..]));
            next = patched;
        }

        // ---- CANDIDATE TRANSACTION (build everything aside) -------------
        // The frontend compile counts even on later failure (it really
        // ran); every OTHER counter only moves on publish.
        self.frontends += 1;
        let frontends_before = self.frontends - 1;
        let candidate = self.build_candidate(&next);
        let (checked, ident, invalidated, code, production, linked) = match candidate {
            Ok(x) => x,
            Err(e) => {
                // R2: the frontend compile above already bumped `frontends`
                // (it ran); everything else is untouched BY CONSTRUCTION
                // (aside tables). Nothing to roll back — the resident state
                // was never modified.
                let _ = frontends_before;
                return Err(e);
            }
        };
        // Entry RUN check on the linked candidate BEFORE publishing.
        let (out, _) = execute_program(&linked, &ExecLimits::default(), &CancelFlag::new(false));
        if !matches!(out, ExecOutcome::Success { .. }) {
            return Err(format!("candidate run: {out:?}"));
        }

        // ---- ATOMIC PUBLISH ------------------------------------------------
        self.revision += 1;
        self.src = next;
        self.code = code;
        // Merge production: untouched residents keep their counters;
        // re-produced ones advance by the aside production.
        let mut merged = self.production.clone();
        for (name, prod) in production {
            let e = merged.entry(name).or_default();
            e.lower += prod.lower;
            e.encode += prod.encode;
            e.admit += prod.admit;
        }
        self.production = merged;
        self.identity = ident;
        self.last_invalidated = invalidated.clone();
        let _ = checked;
        Ok(invalidated)
    }

    /// Build the FULL candidate aside. Returns the checked module, the
    /// candidate identity, the invalidated set, the next resident set,
    /// the aside production deltas, and the LINKED candidate snapshot.
    #[allow(clippy::type_complexity)]
    fn build_candidate(
        &self,
        next_src: &str,
    ) -> Result<
        (
            auto_ac_prototype::verify::CheckedModule,
            RevisionIdentity,
            Vec<String>,
            BTreeMap<String, Arc<FunctionCode>>,
            BTreeMap<String, Production>,
            ProgramSnapshot,
        ),
        String,
    > {
        let saved_src = self.src.clone();
        let saved_frontends = self.frontends;
        // compile_of borrows &mut self for the counter; emulate without
        // touching self: compile directly and count via a local.
        let (checked, ident) = {
            let compiled = auto_ac_prototype::source::compile_text_record_source(
                next_src,
                "session.at",
                &self.caps,
            )
            .map_err(|ds| {
                ds.first()
                    .map(|d| format!("{} {}", d.code, d.message))
                    .unwrap_or_else(|| "frontend".to_string())
            })?;
            let ident = identity_of(&compiled.checked);
            (compiled.checked, ident)
        };
        let _ = saved_src;
        let _ = saved_frontends;
        // Invalidation: own body changed, or the module's TYPE CONTEXT
        // changed (signatures/layouts stale every unit's import table).
        let type_ctx_changed = ident.type_ctx != self.identity.type_ctx;
        let mut invalidated = Vec::new();
        for (name, fp) in &ident.bodies {
            let changed = type_ctx_changed || self.identity.bodies.get(name) != Some(fp);
            if changed {
                invalidated.push(name.clone());
            }
        }
        // The NEXT resident set: unchanged residents keep their objects;
        // removed functions drop out.
        let mut code: BTreeMap<String, Arc<FunctionCode>> = self
            .code
            .iter()
            .filter(|(n, _)| ident.bodies.contains_key(*n))
            .map(|(n, c)| (n.clone(), c.clone()))
            .collect();
        // Aside production for the invalidated set only.
        let mut production: BTreeMap<String, Production> = BTreeMap::new();
        let next_revision = self.revision + 1;
        for name in &invalidated {
            let (n, fc, prod) = self.produce_unit_aside(&checked, name, next_revision)?;
            code.insert(n, fc);
            production.insert(name.clone(), prod);
        }
        let linked =
            ProgramSnapshot::link(code.values().cloned().collect(), &self.entry, next_revision)
                .map_err(|e| format!("candidate link: {e}"))?;
        Ok((checked, ident, invalidated, code, production, linked))
    }

    /// Link the resident set and run the entry.
    pub fn run(&self) -> (ExecOutcome, auto_av_core::av::ExecTrace) {
        let code: Vec<Arc<FunctionCode>> = self.code.values().cloned().collect();
        let prog = ProgramSnapshot::link(code, &self.entry, self.revision)
            .expect("the resident set links");
        execute_program(&prog, &ExecLimits::default(), &CancelFlag::new(false))
    }

    pub fn run_i32(&self) -> i32 {
        let (out, _) = self.run();
        let ExecOutcome::Success {
            value: auto_av_core::contract::Value::I32(v),
            ..
        } = out
        else {
            panic!("expected i32 success, got {out:?}")
        };
        v
    }

    /// The FORMAL degradation detector (R2: covers the WHOLE production
    /// surface). After an edit whose invalidated set is `expected`:
    /// - exactly ONE frontend compile per revision (the real whole-module
    ///   front-end the hot path still rides — visible, never hidden);
    /// - per-unit production advanced ONLY on the invalidated set;
    /// - untouched residents keep their Arc identity AND mint generation
    ///   (an object swap without a mint bump reddens);
    /// - the resident set membership matches (missed invalidation and
    ///   silent removal both redden).
    pub fn incremental_gate(
        &self,
        before: &BTreeMap<String, Production>,
        before_snapshot: &GateSnapshot,
        frontends_before: u32,
        expected: &[&str],
    ) -> Result<(), String> {
        // (0) exactly one frontend compile per edit.
        if self.frontends != frontends_before + 1 {
            return Err(format!(
                "incremental gate: frontend compiles {} -> {} (expected exactly +1 per revision)",
                frontends_before, self.frontends
            ));
        }
        let expect_set: std::collections::BTreeSet<&str> = expected.iter().copied().collect();
        // (1) production only on the invalidated set.
        for (name, now) in &self.production {
            let was = before.get(name).copied().unwrap_or_default();
            if expect_set.contains(name.as_str()) {
                if *now == was {
                    return Err(format!(
                        "incremental gate: `{name}` was invalidated but produced NOTHING"
                    ));
                }
            } else if *now != was {
                return Err(format!(
                    "incremental gate: `{name}` produced outside the invalidated set (whole-recompute leakage)"
                ));
            }
        }
        // (2) untouched residents keep object identity + generation.
        for (name, was_code) in &before_snapshot.residents {
            if expect_set.contains(name.as_str()) {
                continue;
            }
            match self.code.get(name) {
                None => {
                    return Err(format!(
                        "incremental gate: untouched resident `{name}` vanished from the set"
                    ))
                }
                Some(now_code) => {
                    if !Arc::ptr_eq(was_code, now_code) {
                        return Err(format!(
                            "incremental gate: untouched resident `{name}` was object-swapped"
                        ));
                    }
                    if now_code.generation() != was_code.generation() {
                        return Err(format!(
                            "incremental gate: untouched resident `{name}` mint bumped without content"
                        ));
                    }
                }
            }
        }
        // (3) the set grows only by invalidated names.
        for name in self.code.keys() {
            if !before_snapshot.residents.contains_key(name) && !expect_set.contains(name.as_str())
            {
                return Err(format!(
                    "incremental gate: `{name}` appeared outside the invalidated set"
                ));
            }
        }
        Ok(())
    }

    /// Production snapshot (the caller takes `before` copies between
    /// edits and hands them to [`Self::incremental_gate`]).
    pub fn production(&self) -> BTreeMap<String, Production> {
        self.production.clone()
    }

    /// The identity snapshot the gate compares residents against.
    pub fn gate_snapshot(&self) -> GateSnapshot {
        GateSnapshot {
            residents: self
                .code
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    pub fn frontends(&self) -> u32 {
        self.frontends
    }

    pub fn code_of(&self, def: &str) -> Arc<FunctionCode> {
        self.code.get(def).expect("resident").clone()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn last_invalidated(&self) -> &[String] {
        &self.last_invalidated
    }

    /// The forced-whole channel: re-produce EVERY unit at a new revision
    /// (the degradation the gate must catch when the caller expects an
    /// incremental set).
    pub fn force_whole_recompute(&mut self) -> Result<(), String> {
        let (checked, _) = self.compile()?;
        self.revision += 1;
        let mut code = BTreeMap::new();
        let mut production = BTreeMap::new();
        let names: Vec<String> = checked
            .bundle()
            .modules
            .first()
            .map(|m| {
                m.defs
                    .iter()
                    .filter(|d| matches!(d.kind, auto_ac_prototype::hir::DefKind::Function { .. }))
                    .map(|d| d.id_text.clone())
                    .collect()
            })
            .unwrap_or_default();
        for n in &names {
            let (name, fc, prod) = self.produce_unit_aside(&checked, n, self.revision)?;
            code.insert(name, fc);
            production.insert(n.clone(), prod);
        }
        let linked =
            ProgramSnapshot::link(code.values().cloned().collect(), &self.entry, self.revision)
                .map_err(|e| format!("forced link: {e}"))?;
        let (out, _) = execute_program(&linked, &ExecLimits::default(), &CancelFlag::new(false));
        if !matches!(out, ExecOutcome::Success { .. }) {
            return Err(format!("forced run: {out:?}"));
        }
        self.code = code;
        let mut merged = self.production.clone();
        for (name, prod) in production {
            let e = merged.entry(name).or_default();
            e.lower += prod.lower;
            e.encode += prod.encode;
            e.admit += prod.admit;
        }
        self.production = merged;
        self.last_invalidated = self.code.keys().cloned().collect();
        Ok(())
    }
}

/// The resident-identity snapshot for the formal gate.
pub struct GateSnapshot {
    residents: BTreeMap<String, Arc<FunctionCode>>,
}
