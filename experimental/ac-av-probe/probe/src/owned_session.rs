//! PLAN-821 R1-F07: the REAL owned incremental session — a production
//! install channel for `av.owned/1` over the D828 interfaces
//! (`lower_function_owned_unit` → `FunctionCode::admit_unit` →
//! `ProgramSnapshot::link` → `execute_program`).
//!
//! What makes this the real entry (vs. a test wrapper):
//! - ONE frontend compile per revision (`compile_text_record_source`);
//!   per-function identity is derived from the checked HIR — edits are
//!   detected AUTOMATICALLY (no hand-picked def ids);
//! - the invalidation rule is content-precise: a unit re-produces when
//!   ITS body fingerprint changed, or when the module's TYPE CONTEXT
//!   changed (a signature/layout change invalidates every unit's
//!   module-wide import table — the exact V-13(3) semantics);
//! - production counters (frontend compiles; per-unit lower/encode/
//!   admit) are incremented by THIS code at the real production points;
//! - `incremental_gate` is the formal degradation detector: after an
//!   edit, production may advance ONLY on the invalidated set — a
//!   forced or silent whole recompute REDDENS the same gate.

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
/// context fingerprint plus each function def's body fingerprint.
#[derive(Clone, Debug)]
struct RevisionIdentity {
    type_ctx: u64,
    bodies: BTreeMap<String, u64>,
}

/// Strip `span: Span { start: N, end: M }` projections from a Debug
/// rendering — the identity is CONTENT-based; unrelated position shifts
/// (an edit earlier in the file) must not invalidate untouched bodies.
fn strip_spans(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("span: Span { start: ") {
        out.push_str(&rest[..i]);
        // consume through the closing brace of the Span struct
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
            // table; the fingerprint must hash the content (a body edit
            // that leaves indices intact changes nothing otherwise).
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
    /// The units re-produced by the LAST edit (the detector's raw record).
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
        s.identity = ident;
        s.produce_all(&checked)?;
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

    fn unit_of(
        &mut self,
        checked: &auto_ac_prototype::verify::CheckedModule,
        def_id: &str,
    ) -> Result<Arc<FunctionCode>, String> {
        let bundle = checked.bundle();
        let snap = fnv1a64(self.src.as_bytes());
        let unit = crate::hir_adapter::lower_function_owned_unit(&bundle, def_id, &self.caps, snap)
            .map_err(|e| format!("lower {def_id}: {}", e.code()))?;
        mir_verify::verify_mir(&unit).map_err(|e| format!("verify {def_id}: {e:?}"))?;
        let wire = encode_abc(&unit).map_err(|e| format!("encode {def_id}: {}", e.code()))?;
        let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &unit.owner)
            .map_err(|e| format!("abc admit {def_id}: {e:?}"))?;
        let _ = decode_abc(&wire.to_flat());
        let fc = FunctionCode::admit_unit(loaded, self.revision)
            .map_err(|e| format!("admit {def_id}: {e}"))?;
        self.production
            .entry(def_id.to_string())
            .or_default()
            .bump();
        Ok(Arc::new(fc))
    }

    fn produce_all(
        &mut self,
        checked: &auto_ac_prototype::verify::CheckedModule,
    ) -> Result<(), String> {
        let m = checked.bundle().modules.first().cloned().expect("module");
        for d in &m.defs {
            if matches!(d.kind, auto_ac_prototype::hir::DefKind::Function { .. }) {
                let fc = self.unit_of(checked, &d.id_text)?;
                self.code.insert(d.id_text.clone(), fc);
            }
        }
        Ok(())
    }

    /// Apply ONE source edit (see [`Self::edit_batch`]).
    pub fn edit(&mut self, start: u64, end: u64, insert: &[u8]) -> Result<Vec<String>, String> {
        self.edit_batch(&[(start, end, insert.to_vec())])
    }

    /// Apply a BATCH of edits ATOMICALLY (all ranges against the CURRENT
    /// source, applied back-to-front; ONE frontend compile after the
    /// batch). Interface changes that no single-site intermediate would
    /// typecheck (a param retyping: signature + body + call sites) ride
    /// this channel — the batch is the atomic rewrite unit.
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
        // Compile the CANDIDATE first: a rejected batch leaves the
        // resident state (source, revision, identity) untouched — the
        // frontend compile itself still counts (it really ran).
        let (checked, ident) = self.compile_of(&next)?;
        self.revision += 1;
        self.src = next;
        // The invalidation rule (V-13(2)/(3)): a unit re-produces when its
        // OWN body changed, or when the module's type context changed
        // (signatures/layouts part every unit's module-wide import table).
        let type_ctx_changed = ident.type_ctx != self.identity.type_ctx;
        let mut invalidated = Vec::new();
        for (name, fp) in &ident.bodies {
            let changed = type_ctx_changed || self.identity.bodies.get(name) != Some(fp);
            if changed {
                invalidated.push(name.clone());
            }
        }
        // Removed functions: drop their residents.
        let removed: Vec<String> = self
            .code
            .keys()
            .filter(|n| !ident.bodies.contains_key(*n))
            .cloned()
            .collect();
        for n in &removed {
            self.code.remove(n);
            self.production.remove(n);
        }
        for name in &invalidated {
            let fc = self.unit_of(&checked, name)?;
            self.code.insert(name.clone(), fc);
        }
        self.identity = ident;
        self.last_invalidated = invalidated.clone();
        Ok(invalidated)
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

    /// The FORMAL degradation detector: after an edit whose invalidated
    /// set is `expected`, production may have advanced ONLY there. A
    /// whole-module recompute (forced or silent), an object swap without
    /// a mint bump, or a missed invalidation REDDENS this gate.
    pub fn incremental_gate(
        &self,
        before: &BTreeMap<String, Production>,
        expected: &[&str],
    ) -> Result<(), String> {
        let expect_set: std::collections::BTreeSet<&str> = expected.iter().copied().collect();
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
        Ok(())
    }

    /// Production snapshot (the caller takes `before` copies between
    /// edits and hands them to [`Self::incremental_gate`]).
    pub fn production(&self) -> BTreeMap<String, Production> {
        self.production.clone()
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
        self.revision += 1;
        let (checked, _) = self.compile()?;
        self.produce_all(&checked)?;
        self.last_invalidated = self.code.keys().cloned().collect();
        Ok(())
    }
}
