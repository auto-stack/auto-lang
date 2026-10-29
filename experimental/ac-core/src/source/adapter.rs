//! Atom text adapter: checked source frontend → Schema binder → verifier
//! (PLAN-745 T-03).
//!
//! Lowers a resolved + type-checked source program into core-i32 Atom text
//! and feeds it through the *existing* pipeline (`descriptor::bind` → span
//! re-anchoring → `verify::verify`). The binder stays the category/reference
//! binding entry point and the verifier the only way to obtain a
//! `CheckedModule`; this adapter never bypasses either.
//!
//! Deterministic encoding choices (plan §5.2/§5.3):
//! - every syntactic occurrence gets a fresh expr/block/place id — no
//!   sharing, no folding, no reordering;
//! - `true`/`false` encode as fresh `0 < 1` / `0 < 0` subgraphs whose
//!   synthesized spans point back at the literal token;
//! - `while cond { body }` lowers to `loop { if cond { body } else { break } }`
//!   so the condition is re-evaluated every iteration and after `continue`;
//!   the synthesized blocks/loop id project onto the `while` condition;
//! - statements after a terminator lower into separate unreferenced (dead)
//!   blocks — faithful to their unreachability; the verifier rejects them
//!   with `verify.block-structure` at the dead statement;
//! - a `break`/`continue` outside any loop targets a phantom loop emitted
//!   just before it in the same block, so the existing verifier owns the
//!   rejection (`verify.loop-target-scope`).
//!
//! Span re-anchoring: binder output spans live in generated-text
//! coordinates. Each table entry (type/def/body/local/place/expr/block) is
//! re-anchored to its intended source span by emission order, and every
//! emitted token records a fragment mapping used as the fallback for any
//! other generated position. Missing coverage is a controlled internal
//! diagnostic — never a panic, never a fabricated source error.

use super::ast::{BinOp, UnaryOp};
use super::resolve::{
    param_position, RBlock, RExprKind, RStmt, ResolvedArg, ResolvedExpr, ResolvedFn,
    ResolvedProgram, VT,
};
use super::typecheck;
use super::{SourceProfile, SOURCE_PROFILE, TRACE_CAPABILITY};
use crate::atom_text::{Diagnostic, Span, Stage};
use crate::verify::CheckedModule;
use std::collections::BTreeMap;

/// Successful source compilation: the checked module plus the receipt data
/// traveling with the adapter result (profile identity, input content hash,
/// the generated internal Atom encoding).
#[derive(Clone, Debug)]
pub struct CompiledSource {
    pub profile: &'static str,
    /// FNV-1a 64-bit content hash of the input bytes (change detection;
    /// not cryptographic).
    pub input_hash: String,
    /// The generated Atom text (deterministic internal encoding, not a
    /// public interchange format).
    pub atom_text: String,
    pub checked: CheckedModule,
}

pub fn compile_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::Core)?;
    let checked = crate::verify::verify(parts.bundle)?;
    Ok(CompiledSource {
        profile: SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Compile under the scalar-i32 source profile (PLAN-767): parse/resolve/
/// typecheck on the scalar surface, bind against the scalar descriptor and
/// verify through the scalar entry. Receipt data mirrors [`compile_source`]
/// with the scalar profile identity.
pub fn compile_scalar_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::Scalar)?;
    let checked = crate::verify::verify_scalar(parts.bundle)?;
    Ok(CompiledSource {
        profile: super::SCALAR_SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Compile under the entry-unit source profile (PLAN-862, T-03a): the
/// frontend half (parse → resolve → typecheck, including the frozen entry
/// matrix) is implemented; the Atom/HIR emission for this profile is NOT
/// (T-03b), so this entry returns the controlled `source.unsupported`
/// gate rather than emitting another domain's document. Frontend-level
/// consumers drive [`super::parse_program_with`] +
/// [`super::resolve_with`] + [`super::typecheck_with`] directly.
pub fn compile_entry_unit_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::EntryUnit)?;
    // Unreachable while the emit gate is in place; kept so the shape of
    // this entry matches the other profiles when T-03b lands.
    let checked = crate::verify::verify_entry_unit(parts.bundle)?;
    Ok(CompiledSource {
        profile: super::ENTRY_UNIT_SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Compile under the text source profile (PLAN-770): parse/resolve/
/// typecheck on the text surface, bind against the text descriptor and
/// verify through the text entry (which also runs the resource walk and
/// attaches the `TextResourcePlan`).
pub fn compile_text_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::Text)?;
    let checked = crate::verify::verify_text(parts.bundle)?;
    Ok(CompiledSource {
        profile: super::TEXT_SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Compile under the collections source profile (PLAN-864): the
/// source-domain twin of `av.collections/1` — parse/resolve/typecheck on
/// the collections surface, bind against the collections descriptor,
/// verify through the collections entry (no text resource plan; profile 4
/// owns cleanup at runtime).
pub fn compile_collections_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::Collections)?;
    let checked = crate::verify::verify_collections(parts.bundle)?;
    Ok(CompiledSource {
        profile: super::COLLECTIONS_SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Compile under PLAN-865's source-closures profile. The frontend inherits
/// collections and entry-unit syntax but emits the separate closures HIR
/// identity and verifier entry.
pub fn compile_closures_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::Closures)?;
    let checked = crate::verify::verify_closures(parts.bundle)?;
    Ok(CompiledSource {
        profile: super::CLOSURES_SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Compile under the record source profile (PLAN-771): the scalar surface
/// plus records — parse/resolve/typecheck, bind against the record
/// descriptor, verify through the record entry.
pub fn compile_record_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::Record)?;
    let checked = crate::verify::verify_record(parts.bundle)?;
    Ok(CompiledSource {
        profile: super::RECORD_SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// Compile under the text-record source profile (PLAN-782): the union of
/// the text and record surfaces — parse/resolve/typecheck, bind against
/// the text-record descriptor and verify through the combination entry
/// (T-02 structural gates; the extended resource walk lands in T-04).
pub fn compile_text_record_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::TextRecord)?;
    let checked = crate::verify::verify_text_record(parts.bundle)?;
    Ok(CompiledSource {
        profile: super::TEXT_RECORD_SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

/// One frontend stage boundary (PLAN-788 T-03 observation seam). The
/// source frontend reports parse/resolve/typecheck failures all as
/// `Stage::Text` + `source.*` codes; the boundary that produced them is
/// only observable here — never from the rendered string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceStage {
    Parse,
    Resolve,
    Typecheck,
    Emit,
    Bind,
}

impl SourceStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceStage::Parse => "parse",
            SourceStage::Resolve => "resolve",
            SourceStage::Typecheck => "typecheck",
            SourceStage::Emit => "emit",
            SourceStage::Bind => "bind",
        }
    }
}

/// Narrow structured observation for the source frontend (PLAN-788 T-03):
/// one callback per COMPLETED stage boundary with the diagnostics that
/// boundary produced (empty slice = success). The pipeline is not
/// duplicated — [`compile_parts_with_observer`] runs the exact same
/// sequence as [`compile_parts_with`] and existing callers are unchanged.
pub trait SourceObserver {
    fn stage(&mut self, _stage: SourceStage, _diagnostics: &[Diagnostic]) {}
}

/// [`compile_parts_with`] with stage-boundary observation (PLAN-788 T-03).
pub fn compile_parts_with_observer(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    profile: SourceProfile,
    observer: &mut dyn SourceObserver,
) -> Result<SourceParts, Vec<Diagnostic>> {
    compile_parts_observed(src, module_path, capabilities, profile, Some(observer))
}

/// Compile up to (but not including) verification under the explicit
/// profile: the span-projected bundle plus the adapter receipt data. This
/// is the seam the §6 mutation matrix exercises — a tampered bundle must
/// go back through the real verifier of ITS domain and can never reuse
/// Checked credentials.
pub fn compile_parts_with(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    profile: SourceProfile,
) -> Result<SourceParts, Vec<Diagnostic>> {
    compile_parts_observed(src, module_path, capabilities, profile, None)
}

fn compile_parts_observed(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    profile: SourceProfile,
    mut observer: Option<&mut dyn SourceObserver>,
) -> Result<SourceParts, Vec<Diagnostic>> {
    let program = match super::parse_program_with(src, profile) {
        Ok(p) => {
            if let Some(o) = observer.as_deref_mut() {
                o.stage(SourceStage::Parse, &[]);
            }
            p
        }
        Err(d) => {
            if let Some(o) = observer.as_deref_mut() {
                o.stage(SourceStage::Parse, std::slice::from_ref(&d));
            }
            return Err(vec![d]);
        }
    };
    compile_program_observed(&program, src, module_path, capabilities, profile, observer)
}

/// Compile an already-parsed program up to (but not including) verification
/// (PLAN-828 T-02): the function-incremental session supplies the program
/// assembled from the CST projection of the changed declaration plus
/// signature-stub items, so no whole-module parse runs on the edit path.
/// `src` is only used for span re-anchoring; unchanged-entry completions are
/// identical to [`compile_parts_with`].
pub fn compile_program_parts(
    program: &super::ast::Program,
    src: &str,
    module_path: &str,
    capabilities: &[String],
    profile: SourceProfile,
) -> Result<SourceParts, Vec<Diagnostic>> {
    compile_program_observed(program, src, module_path, capabilities, profile, None)
}

/// PLAN-828 D-01 observation: what a single-body emission actually
/// walked, reported by the PRODUCER (not reconstructed by a caller).
/// A PASS under the §5.8 F-01 gate REQUIRES this attestation — the old
/// whole-program entry produces none ("缺必需观测也应拒绝出PASS证据").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyEmitReceipt {
    /// REAL bodies planned and rendered (the changed function — 1).
    pub real_body_plans: usize,
    /// BODY-LESS declaration scaffolds emitted for the other functions
    /// (the interface-directory traversal §5.6 allows; both the old
    /// no body is planned, rendered or bound for them).
    pub scaffold_fns: usize,
}

/// PLAN-828 D-01: emit + bind ONE function against the RETAINED checked
/// environment ([`super::resolve::CheckedEnv`] with the re-checked body
/// already grafted at `fn_index`). The single-def document renders with
/// the SAME Emitter rules as the whole-module pipeline — callee/type
/// lookups consult the full environment — and binds to exactly the HIR
/// def the whole-module compile produces for that function.
pub fn emit_function_body(
    env: &super::resolve::CheckedEnv,
    fn_index: usize,
    src: &str,
    module_path: &str,
) -> Result<(SourceParts, BodyEmitReceipt), Vec<Diagnostic>> {
    if env.closed() {
        return Err(vec![Diagnostic::new(
            Stage::Text,
            "source.closed",
            "the checked environment is closed; emission is rejected",
            Span { start: 0, end: 0 },
        )]);
    }
    let profile = env.profile;
    let mut em = Emitter::new(&env.program, src.len(), profile);
    em.scope = Some(fn_index);
    em.plan_indexed(fn_index);
    let atom_text = em.render(module_path);
    let receipt = BodyEmitReceipt {
        real_body_plans: 1,
        scaffold_fns: env.program.fns.len().saturating_sub(1),
    };
    let descriptor = match profile.hir_domain() {
        crate::hir::HirDomain::Core => crate::descriptor(),
        crate::hir::HirDomain::Scalar => crate::scalar_descriptor(),
        crate::hir::HirDomain::Text => crate::text_descriptor(),
        crate::hir::HirDomain::Record => crate::record_descriptor(),
        crate::hir::HirDomain::TextRecord => crate::text_record_descriptor(),
        crate::hir::HirDomain::EntryUnit => crate::entry_unit_descriptor(),
        crate::hir::HirDomain::HostConsole => crate::host_console_descriptor(),
        crate::hir::HirDomain::Collections => crate::collections_descriptor(),
        crate::hir::HirDomain::Closures => crate::closures_descriptor(),
    };
    let mut bundle = match crate::descriptor::bind(descriptor, &atom_text) {
        Ok(b) => b,
        Err(ds) => {
            let detail: Vec<String> = ds.iter().map(|d| d.render("atom", &atom_text)).collect();
            return Err(vec![Diagnostic::new(
                Stage::Text,
                "source.internal",
                format!(
                    "internal adapter error: generated Atom text failed to bind\n{}",
                    detail.join("\n")
                ),
                Span { start: 0, end: 0 },
            )]);
        }
    };
    match em.reanchor_spans(&mut bundle, src) {
        Ok(()) => {}
        Err(d) => return Err(vec![d]),
    }
    Ok((
        SourceParts {
            profile: profile.as_str(),
            input_hash: fnv1a_hex(src.as_bytes()),
            atom_text,
            bundle,
            planned_fns: em.plans.len(),
        },
        receipt,
    ))
}

/// PLAN-828 D-03 (T01-C05): one prepared single-function CANDIDATE. The
/// candidate — not the environment — owns the new body entry (moved out
/// during prepare, never a clone) together with its ownership facts: the
/// environment identity and publication generation it was prepared
/// against and the slot it belongs to. While a candidate lives, the
/// environment still serves the CURRENT published semantics; only
/// [`FnCandidate::commit`] ever installs, exactly once, after validating
/// closed/owner/base. Disposition never writes the wrong state:
/// [`FnCandidate::cancel`] (and a plain drop) take no environment
/// reference, so an uncommitted candidate cannot modify any current
/// semantics by construction.
pub struct FnCandidate {
    env_id: u64,
    base_generation: u64,
    fn_index: usize,
    slot_name: String,
    entry: ResolvedFn,
    parts: SourceParts,
    receipt: BodyEmitReceipt,
}

impl FnCandidate {
    /// The environment slot this candidate installs into on commit.
    pub fn fn_index(&self) -> usize {
        self.fn_index
    }

    /// The function name of the candidate's slot.
    pub fn slot_name(&self) -> &str {
        &self.slot_name
    }

    /// The environment identity this candidate was prepared against.
    pub fn owner_env_id(&self) -> u64 {
        self.env_id
    }

    /// The publication generation this candidate was prepared against; a
    /// commit onto a newer generation is a controlled stale rejection.
    pub fn base_generation(&self) -> u64 {
        self.base_generation
    }

    /// The candidate's own frozen artifacts (single-body document bound
    /// and re-anchored against the candidate source).
    pub fn parts(&self) -> &SourceParts {
        &self.parts
    }

    /// The producer attestation of the candidate's emission.
    pub fn receipt(&self) -> BodyEmitReceipt {
        self.receipt
    }

    /// Commit — the SINGLE publication point: validate closed → owner →
    /// base → slot, then install THIS candidate's entry and bump the
    /// environment generation. A closed environment, a foreign
    /// environment (identity, not name+slot), an expired base (a newer
    /// commit was published since prepare) or a slot that no longer holds
    /// this function is a controlled rejection that modifies nothing.
    /// The session calls this at its one commit point, together with the
    /// root/source/revision publication.
    pub fn commit(self, env: &mut super::resolve::CheckedEnv) -> Result<(), Vec<Diagnostic>> {
        if env.closed {
            return Err(vec![Diagnostic::new(
                Stage::Bind,
                "source.closed",
                "the checked environment is closed; committing a candidate is a rejection",
                Span { start: 0, end: 0 },
            )]);
        }
        if self.env_id != env.id {
            return Err(vec![Diagnostic::new(
                Stage::Bind,
                "source.foreign",
                format!(
                    "candidate for `{}` was prepared against environment #{}; this is environment #{} — a matching function name and slot is not environment identity",
                    self.slot_name, self.env_id, env.id
                ),
                Span { start: 0, end: 0 },
            )]);
        }
        if self.base_generation != env.generation {
            return Err(vec![Diagnostic::new(
                Stage::Bind,
                "source.stale",
                format!(
                    "candidate for `{}` was prepared against generation {} but the environment is at generation {} (a newer commit was published); an old candidate never overwrites a newer result",
                    self.slot_name, self.base_generation, env.generation
                ),
                Span { start: 0, end: 0 },
            )]);
        }
        if env.program.fns.get(self.fn_index).map(|f| f.name.as_str())
            != Some(self.slot_name.as_str())
        {
            return Err(vec![Diagnostic::new(
                Stage::Bind,
                "source.incremental",
                format!(
                    "slot {} no longer holds `{}`; the candidate no longer applies",
                    self.fn_index, self.slot_name
                ),
                Span { start: 0, end: 0 },
            )]);
        }
        env.program.fns[self.fn_index] = self.entry;
        env.generation += 1;
        Ok(())
    }

    /// Cancel: discard the uncommitted candidate. Takes NO environment
    /// reference — cancellation cannot modify any current semantics, of
    /// any environment, by construction; dropping the candidate is
    /// equivalent.
    pub fn cancel(self) {}
}

/// PLAN-828 T-02: the PRODUCTION candidate document transaction — recheck
/// + single-body emission, WITHOUT verification: the consuming session
/// decides between content-addressed reuse of a stored, already-verified
/// pair (no re-verify) and the full verify/lower/admit chain before
/// publishing. Identical D-03 invariants to
/// [`prepare_function_candidate`]: the bounded temporary graft is restored
/// BEFORE returning (current is never published at prepare time), the
/// candidate owns the new entry (moved) plus owner/base facts and its own
/// artifacts; commit installs once, cancel/drop touch nothing.
pub fn prepare_candidate_document(
    env: &mut super::resolve::CheckedEnv,
    item: &super::ast::FnItem,
    fn_index: usize,
    src: &str,
    module_path: &str,
) -> Result<FnCandidate, Vec<Diagnostic>> {
    // Bounded temporary graft (exclusive access): the re-check grafts the
    // candidate body and hands back the displaced CURRENT entry.
    let current = super::resolve::recheck_swap(env, item, fn_index)?;
    match emit_function_body(env, fn_index, src, module_path) {
        Ok((parts, receipt)) => {
            // Restore current and take the candidate BEFORE returning —
            // publication happens only at FnCandidate::commit.
            let entry = std::mem::replace(&mut env.program.fns[fn_index], current);
            Ok(FnCandidate {
                env_id: env.id,
                base_generation: env.generation,
                fn_index,
                slot_name: item.name.name.clone(),
                entry,
                parts,
                receipt,
            })
        }
        Err(ds) => {
            env.program.fns[fn_index] = current;
            Err(ds)
        }
    }
}

/// PLAN-828 D-03 (T01-C05): the minimal transactional single-function
/// prepare. Within one exclusive-access bounded transaction the candidate
/// body is temporarily grafted so the REAL single-body resolve/typecheck,
/// emission and function-level verification all run against it — then,
/// BEFORE returning, the current entry is restored and the CANDIDATE
/// carries the new entry (moved) plus its owner/base facts. The
/// environment therefore never publishes at prepare time: a successful
/// prepare leaves current semantics byte-identical; a failure at ANY
/// stage restores the current entry first. No environment clone — the
/// transaction's whole bounded state is the one moved entry plus the
/// candidate's artifacts.
pub fn prepare_function_candidate(
    env: &mut super::resolve::CheckedEnv,
    item: &super::ast::FnItem,
    fn_index: usize,
    src: &str,
    module_path: &str,
) -> Result<FnCandidate, Vec<Diagnostic>> {
    let candidate = prepare_candidate_document(env, item, fn_index, src, module_path)?;
    let def = format!("d_{}", item.name.name);
    let def_idx = candidate.parts().bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == def)
        .unwrap_or(fn_index);
    // The document prepare already restored current; a verify failure just
    // discards the candidate (drop is inert).
    match crate::verify::verify_function(candidate.parts().bundle.clone(), def_idx) {
        Ok(_) => Ok(candidate),
        Err(ds) => Err(ds),
    }
}

fn compile_program_observed(
    program: &super::ast::Program,
    src: &str,
    module_path: &str,
    capabilities: &[String],
    profile: SourceProfile,
    mut observer: Option<&mut dyn SourceObserver>,
) -> Result<SourceParts, Vec<Diagnostic>> {
    let resolved = match super::resolve_with(program, capabilities, profile) {
        Ok(r) => {
            if let Some(o) = observer.as_deref_mut() {
                o.stage(SourceStage::Resolve, &[]);
            }
            r
        }
        Err(ds) => {
            if let Some(o) = observer.as_deref_mut() {
                o.stage(SourceStage::Resolve, &ds);
            }
            return Err(ds);
        }
    };
    let _ = match typecheck::typecheck_with(&resolved, profile) {
        Ok(v) => {
            if let Some(o) = observer.as_deref_mut() {
                o.stage(SourceStage::Typecheck, &[]);
            }
            v
        }
        Err(ds) => {
            if let Some(o) = observer.as_deref_mut() {
                o.stage(SourceStage::Typecheck, &ds);
            }
            return Err(ds);
        }
    };

    let mut em = Emitter::new(&resolved, src.len(), profile);
    em.plan_module();
    let atom_text = em.render(module_path);
    if let Some(o) = observer.as_deref_mut() {
        o.stage(SourceStage::Emit, &[]);
    }

    let descriptor = match profile.hir_domain() {
        crate::hir::HirDomain::Core => crate::descriptor(),
        crate::hir::HirDomain::Scalar => crate::scalar_descriptor(),
        crate::hir::HirDomain::Text => crate::text_descriptor(),
        crate::hir::HirDomain::Record => crate::record_descriptor(),
        crate::hir::HirDomain::TextRecord => crate::text_record_descriptor(),
        crate::hir::HirDomain::EntryUnit => crate::entry_unit_descriptor(),
        crate::hir::HirDomain::HostConsole => crate::host_console_descriptor(),
        crate::hir::HirDomain::Collections => crate::collections_descriptor(),
        crate::hir::HirDomain::Closures => crate::closures_descriptor(),
    };
    let mut bundle = match crate::descriptor::bind(descriptor, &atom_text) {
        Ok(b) => b,
        Err(ds) => {
            let detail: Vec<String> = ds.iter().map(|d| d.render("atom", &atom_text)).collect();
            let diag = Diagnostic::new(
                Stage::Text,
                "source.internal",
                format!(
                    "internal adapter error: generated Atom text failed to bind\n{}",
                    detail.join("\n")
                ),
                Span { start: 0, end: 0 },
            );
            if let Some(o) = observer.as_deref_mut() {
                o.stage(SourceStage::Bind, std::slice::from_ref(&diag));
            }
            return Err(vec![diag]);
        }
    };
    match em.reanchor_spans(&mut bundle, src) {
        Ok(()) => {
            if let Some(o) = observer.as_deref_mut() {
                o.stage(SourceStage::Bind, &[]);
            }
        }
        Err(d) => {
            if let Some(o) = observer.as_deref_mut() {
                o.stage(SourceStage::Bind, std::slice::from_ref(&d));
            }
            return Err(vec![d]);
        }
    }
    Ok(SourceParts {
        profile: profile.as_str(),
        input_hash: fnv1a_hex(src.as_bytes()),
        atom_text,
        bundle,
        planned_fns: em.plans.len(),
    })
}

/// Unverified core-profile compilation output (see [`compile_parts_with`]);
/// the 745-era mutation-matrix test seam.
pub fn compile_parts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<SourceParts, Vec<Diagnostic>> {
    compile_parts_with(src, module_path, capabilities, SourceProfile::Core)
}

/// Unverified compilation output (see [`compile_parts`]).
pub struct SourceParts {
    pub profile: &'static str,
    /// FNV-1a 64-bit content hash of the input bytes.
    pub input_hash: String,
    /// The generated Atom text (deterministic internal encoding).
    pub atom_text: String,
    /// The bound, span-projected bundle; verify it before any use.
    pub bundle: crate::hir::Bundle,
    /// PLAN-828 F-01 observation: how many functions this emission
    /// planned and rendered — from the producer itself.
    pub planned_fns: usize,
}

fn fnv1a_hex(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:016x}", h)
}

/// Atom type id of a value type (PLAN-771: record types are
/// `t_rec_<name>`, nominal by declaration; PLAN-862: Unit has no Atom type
/// id because it is a RESULT-only type — reaching here with `VT::Unit`
/// would mean a value of unit type, which the entry-unit profile rejects).
fn ty_name(
    vt: VT,
    records: &[crate::source::resolve::ResolvedRecord],
    lists: &[VT],
    variants: &[crate::source::resolve::ResolvedVariant],
) -> String {
    match vt {
        VT::I32 => "t_i32".to_string(),
        VT::Bool => "t_bool".to_string(),
        VT::Str => "t_str".to_string(),
        VT::StrOwned => "t_Str".to_string(),
        VT::Char => "t_char".to_string(),
        VT::Rec(i) => format!("t_rec_{}", records[i].name),
        // PLAN-864: list identity is its element type; variants are
        // nominal by name (the type-table entries render in plan_module).
        VT::List(i) => format!(
            "t_list_{}",
            ty_name(lists[i], records, lists, variants).trim_start_matches("t_")
        ),
        VT::Variant(i) => format!("t_variant_{}", variants[i].name),
        VT::Function(i) => format!("t_closure_sig_{}", i),
        // not reachable through a compiling entry-unit program (Unit is a
        // result type); the Atom id keeps the name meaningful for any
        // hand-built diagnostic path (T-03b wires the result-side emit).
        // PLAN-862: `unit` is a BUILTIN TypeUse word in the entry-unit
        // descriptor, so no named `t_unit` type entry exists — the bareword
        // resolves to `BuiltinType::Unit` at bind time.
        VT::Unit => "unit".to_string(),
    }
}

fn internal_diag(msg: String) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.internal",
        msg,
        Span { start: 0, end: 0 },
    )
}

// ---------------------------------------------------------------------------
// Planning: assign ids and build the lowered body shapes
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
enum EKind {
    Const(i32),
    /// PLAN-862: the unit value — a Unit body's return operand.
    Unit,
    Read(usize),
    Bin(BinOp, usize, usize),
    /// Lazy boolean short-circuit (PLAN-767): `&&`/`||` surface Binary
    /// nodes lower to the HIR `Logical` kind — never to an eager binary.
    Logical(crate::hir::LogicOp, usize, usize),
    Unary(crate::hir::UnaryOp, usize),
    Call {
        callee_fn: Option<usize>,
        callee_intrinsic: Option<&'static str>,
        eval: Vec<usize>,
        bindings: Vec<(u32, u32)>,
        /// moved flags parallel to `bindings` (PLAN-770 `.move` markers)
        moved: Vec<bool>,
    },
    CallClosure {
        callee: usize,
        eval: Vec<usize>,
    },
    CreateClosure {
        callee_fn: usize,
        captures: Vec<(usize, crate::hir::CaptureMode)>,
    },
    /// String literal (PLAN-770); the VT records the contextual borrowing
    /// (Str) or promoting (StrOwned) choice.
    StrLit(String),
    /// Char literal (PLAN-770): one Unicode scalar.
    CharLit(u32),
    /// Builtin text/char method (PLAN-770); args are planned expr indices.
    Text {
        method: crate::source::resolve::BuiltinMethod,
        recv: usize,
        args: Vec<usize>,
        /// `Some(target)` only for `To` conversions.
        to: Option<super::resolve::VT>,
    },
    // -- record profile (PLAN-771) --
    /// Positional/named construction: `eval` is the SOURCE-order
    /// evaluation list; `bindings` map (field decl idx, arg idx).
    RecordCtor {
        rec: usize,
        eval: Vec<usize>,
        bindings: Vec<(u32, u32)>,
    },
    /// Field read through a flattened declaration-index path.
    FieldGet {
        base: usize,
        path: Vec<u32>,
    },
    /// Read-only borrow (view param / instance receiver argument).
    BorrowView {
        base: usize,
    },
    /// Exclusive borrow (`p.mut`).
    BorrowMut {
        local: usize,
    },
    /// Builtin deep clone `r.clone()` (PLAN-782): one allocation per
    /// non-empty Str field, copyable fields by value.
    RecordClone {
        rec: usize,
        base: usize,
    },
    // -- PLAN-864 collections surface --
    ListNew {
        list: usize,
    },
    ListLen {
        list: usize,
    },
    ListPush {
        list: usize,
        value: usize,
    },
    ListInsert {
        list: usize,
        index: usize,
        value: usize,
    },
    ListGet {
        list: usize,
        index: usize,
    },
    ListSet {
        list: usize,
        index: usize,
        value: usize,
    },
    ListRemove {
        list: usize,
        index: usize,
    },
    ListClear {
        list: usize,
    },
    VariantCtor {
        variant: usize,
        disc: u32,
        payload: Option<usize>,
    },
    VariantGuardTest {
        root: usize,
        variant: usize,
        disc: u32,
    },
    VariantProject {
        root: usize,
        variant: usize,
        disc: u32,
    },
    /// First-class `.move` wrapper (PLAN-782 combination profile): the
    /// whole-root ownership transfer in let/assign/return/argument value
    /// positions.
    Move {
        inner: usize,
    },
}

struct ENode {
    kind: EKind,
    vt: VT,
    span: Span,
}

#[derive(Clone, Copy)]
enum SKind {
    Let {
        local: usize,
        value: usize,
    },
    Assign {
        place: usize,
        value: usize,
    },
    Return {
        value: usize,
    },
    If {
        cond: usize,
        then: usize,
        els: Option<usize>,
    },
    Loop {
        target: usize,
        body: usize,
    },
    Break {
        target: usize,
    },
    Continue {
        target: usize,
    },
    /// PLAN-864: a unit-effect statement (push/insert/set/clear) — the
    /// value is evaluated for effect only; no result slot exists.
    Eval {
        value: usize,
    },
}

struct SNode {
    kind: SKind,
    span: Span,
}

struct BNode {
    stmts: Vec<SNode>,
    span_override: Option<Span>,
    /// Statement index (inclusive) the block span covers; statements after
    /// the first terminator stay in the block for the dataflow walk but are
    /// dead, so they do not widen the block's diagnostic span.
    span_cut: Option<usize>,
    /// Fallback for blocks whose lowered statement list is empty.
    fallback: Span,
    span: Span,
}

struct PNode {
    local: usize,
    /// PLAN-771: field-path place — declaration indices per segment;
    /// empty = whole-local place.
    path: Vec<u32>,
    vt: VT,
    span: Span,
}

struct FnPlan {
    exprs: Vec<ENode>,
    blocks: Vec<BNode>,
    places: Vec<PNode>,
    /// Span of each registered loop id (while → condition, loop/phantom →
    /// the statement).
    loops: Vec<Span>,

    /// whether any text operation/literal was planned (drives the
    /// ac.runtime.text.v1 capability requirement)
    pub has_text_ops: bool,
}

/// Planning context: shares the program (immutable) and the intrinsic
/// first-use map (mutable) across functions without borrowing conflicts.
struct PlanCtx<'p> {
    program: &'p ResolvedProgram,
    intrinsic_first_use: &'p mut BTreeMap<&'static str, Span>,
}

impl PlanCtx<'_> {
    fn plan_fn(&mut self, f: &ResolvedFn) -> FnPlan {
        crate::body_work::record_plan(&f.work_key(&self.program.records));
        // r3 R3-04: a `move Str` parameter is a CLEANUP obligation (its
        // exit frees lower to ac_rt_own_free) even with zero text
        // operations in the body — such a function needs the runtime too.
        let has_move_str_param = f.locals.iter().any(|l| {
            l.is_param && l.mode == super::ast::ParamMode::Move && matches!(l.vt, VT::StrOwned)
        });
        let mut plan = FnPlan {
            has_text_ops: has_move_str_param,
            exprs: Vec::new(),
            blocks: Vec::new(),
            places: Vec::new(),
            loops: Vec::new(),
        };
        // Termination classification for every block, computed bottom-up
        // with an explicit stack (R6: block nesting is unlimited; the
        // previous recursive stmt_term/block_flow walked per level and
        // revisited shared spines, which was both a depth and an O(n²)
        // hazard).
        let terms = block_terms(&f.body);
        let entry = self.plan_block_machine(&mut plan, f, &f.body, &terms);
        debug_assert_eq!(entry, 0);
        // Finalize block spans: override, else the join of the source spans
        // of the statements lowered into the block (cut at the first
        // terminator — trailing statements are dead), else the source
        // block's own span.
        for b in plan.blocks.iter_mut() {
            b.span = match b.span_override {
                Some(s) => s,
                None => {
                    let up_to = b
                        .span_cut
                        .map_or(b.stmts.len(), |c| (c + 1).min(b.stmts.len()));
                    b.stmts[..up_to]
                        .iter()
                        .map(|s| s.span)
                        .reduce(|a, c| a.join(c))
                        .unwrap_or(b.fallback)
                }
            };
        }
        plan
    }

    /// Plan one function body with an explicit block-frame stack (R6: no
    /// recursion, unlimited nesting). Each frame owns the plan-side block
    /// it is filling (`cur`, switching to a dead block after a terminator);
    /// closing a frame assembles the if/while/loop statement into its
    /// parent's home block. Block-id allocation order matches the previous
    /// recursive walk exactly (then before else, body before the while
    /// test/stop blocks).
    fn plan_block_machine(
        &mut self,
        plan: &mut FnPlan,
        f: &ResolvedFn,
        root: &RBlock,
        terms: &BTreeMap<usize, Option<Term>>,
    ) -> usize {
        struct PFrame<'a> {
            src: &'a RBlock,
            idx: usize,
            /// Plan-side block allocated for this source block — the arm a
            /// closing continuation assembles.
            bid: usize,
            /// Current fill target; switches to a dead block after a
            /// terminator.
            cur: usize,
            split_after: bool,
            /// Termination of the block-opening statement currently being
            /// planned (if/while/loop): its SNode lands only when the child
            /// frame closes, so the span_cut/split_after bookkeeping waits
            /// for that close (the recursive form got this for free — the
            /// recursion completed before the caller's term check ran).
            pending_term: Option<Term>,
            cont: PCont<'a>,
        }
        enum PCont<'a> {
            FnBody,
            /// The then-arm closed; `els` is the pending else source block.
            ThenArm {
                cond: usize,
                els: Option<&'a RBlock>,
                home: usize,
                span: Span,
            },
            ElseArm {
                cond: usize,
                then: usize,
                home: usize,
                span: Span,
            },
            WhileArm {
                cond: usize,
                home: usize,
                /// The while condition's span (k_stop/k_test anchors).
                cond_span: Span,
                /// The whole while statement's span (the loop SNode anchor).
                stmt_span: Span,
                target: usize,
            },
            LoopArm {
                home: usize,
                span: Span,
                target: usize,
            },
        }
        let entry = plan.blocks.len();
        plan.blocks.push(BNode {
            stmts: Vec::new(),
            span_override: None,
            span_cut: None,
            fallback: root.span,
            span: root.span,
        });
        let mut stack: Vec<PFrame> = vec![PFrame {
            src: root,
            idx: 0,
            bid: entry,
            cur: entry,
            split_after: false,
            pending_term: None,
            cont: PCont::FnBody,
        }];
        loop {
            let action = {
                let frame = stack.last_mut().expect("plan frame");
                let src: &RBlock = frame.src;
                let next = src.stmts.get(frame.idx);
                if next.is_some() {
                    frame.idx += 1;
                }
                next
            };
            let Some(s) = action else {
                let frame = stack.pop().expect("plan frame");
                let bid = frame.bid;
                // Whether this close lands the pending if/while/loop SNode
                // in its parent (the else-arm push defers it one more
                // frame).
                let mut landed = true;
                match frame.cont {
                    PCont::FnBody => {}
                    PCont::ThenArm {
                        cond,
                        els,
                        home,
                        span,
                    } => match els {
                        Some(eb) => {
                            let k_else = plan.blocks.len();
                            plan.blocks.push(BNode {
                                stmts: Vec::new(),
                                span_override: None,
                                span_cut: None,
                                fallback: eb.span,
                                span: eb.span,
                            });
                            stack.push(PFrame {
                                src: eb,
                                idx: 0,
                                bid: k_else,
                                cur: k_else,
                                split_after: false,
                                pending_term: None,
                                cont: PCont::ElseArm {
                                    cond,
                                    then: bid,
                                    home,
                                    span,
                                },
                            });
                            landed = false;
                        }
                        None => {
                            plan.blocks[home].stmts.push(SNode {
                                kind: SKind::If {
                                    cond,
                                    then: bid,
                                    els: None,
                                },
                                span,
                            });
                        }
                    },
                    PCont::ElseArm {
                        cond,
                        then,
                        home,
                        span,
                    } => {
                        plan.blocks[home].stmts.push(SNode {
                            kind: SKind::If {
                                cond,
                                then,
                                els: Some(bid),
                            },
                            span,
                        });
                    }
                    PCont::WhileArm {
                        cond,
                        home,
                        cond_span,
                        stmt_span,
                        target,
                    } => {
                        let k_body = bid;
                        let k_stop = plan.blocks.len();
                        plan.blocks.push(BNode {
                            stmts: vec![SNode {
                                kind: SKind::Break { target },
                                span: cond_span,
                            }],
                            span_override: Some(cond_span),
                            span_cut: None,
                            fallback: cond_span,
                            span: cond_span,
                        });
                        let k_test = plan.blocks.len();
                        plan.blocks.push(BNode {
                            stmts: vec![SNode {
                                kind: SKind::If {
                                    cond,
                                    then: k_body,
                                    els: Some(k_stop),
                                },
                                span: cond_span,
                            }],
                            span_override: Some(cond_span),
                            span_cut: None,
                            fallback: cond_span,
                            span: cond_span,
                        });
                        plan.blocks[home].stmts.push(SNode {
                            kind: SKind::Loop {
                                target,
                                body: k_test,
                            },
                            span: stmt_span,
                        });
                    }
                    PCont::LoopArm { home, span, target } => {
                        plan.blocks[home].stmts.push(SNode {
                            kind: SKind::Loop { target, body: bid },
                            span,
                        });
                    }
                }
                // Only the FnBody frame closes with no parent: the whole
                // function body is planned.
                if stack.is_empty() {
                    return entry;
                }
                // The pending if/while/loop statement's SNode just landed
                // in the parent's home block: apply its termination
                // bookkeeping now (span_cut counts the SNode, exactly like
                // the recursive form did after plan_stmt returned).
                if landed {
                    let parent = stack.last_mut().expect("plan frame");
                    let home = parent.cur;
                    if let Some(term) = parent.pending_term.take() {
                        if term != Term::No {
                            if plan.blocks[home].span_cut.is_none() {
                                let n = plan.blocks[home].stmts.len();
                                plan.blocks[home].span_cut = Some(n - 1);
                            }
                            parent.split_after = matches!(term, Term::Ret | Term::Div);
                        }
                    }
                }
                continue;
            };
            // Termination bookkeeping. Simple statements land their SNode
            // synchronously; block-opening statements (if/while/loop) land
            // it when the child frame closes, so their bookkeeping is
            // deferred (pending_term) to that close.
            let term = stmt_term_memo(s, terms);
            let opens_block = matches!(
                s,
                RStmt::If { .. } | RStmt::While { .. } | RStmt::Loop { .. }
            );
            // captured BEFORE the match: block statements push child frames
            // on top of this one
            let parent_idx = stack.len() - 1;
            let frame = stack.last_mut().expect("plan frame");
            if frame.split_after {
                // Dead code after a terminator (return, a valid
                // break/continue, or an if/loop whose every path left):
                // its own unreferenced block — faithful to its
                // unreachability, and the verifier rejects it with
                // `verify.block-structure` anchored AT the dead statement.
                // After a STRAY break/continue trailing statements stay in
                // the same block so the dataflow walk still runs and owns
                // the loop-target-scope rejection.
                frame.cur = plan.blocks.len();
                // The dead block's span stays pinned to its FIRST dead
                // statement: the sharpest anchor for the verifier's
                // unreachable-block rejection.
                plan.blocks.push(BNode {
                    stmts: Vec::new(),
                    span_override: Some(stmt_span(s)),
                    span_cut: None,
                    fallback: stmt_span(s),
                    span: stmt_span(s),
                });
                frame.split_after = false;
            }
            let home = frame.cur;
            match s {
                RStmt::Let { local, value, span } => {
                    let v = self.plan_expr(plan, f, value);
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Let {
                            local: local.0,
                            value: v,
                        },
                        span: *span,
                    });
                }
                RStmt::Assign {
                    local,
                    path,
                    value,
                    span,
                } => {
                    let v = self.plan_expr(plan, f, value);
                    // walk the field path to the final type; a path place
                    // writes through the root local's storage (PLAN-771)
                    let mut vt = f.locals[local.0].vt;
                    for (ri, fi) in path {
                        vt = self.program.records[*ri].fields[*fi].vt;
                    }
                    let place = plan.places.len();
                    plan.places.push(PNode {
                        local: local.0,
                        path: path.iter().map(|(_, fi)| *fi as u32).collect(),
                        vt,
                        span: *span,
                    });
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Assign { place, value: v },
                        span: *span,
                    });
                }
                RStmt::Return { value, span } => {
                    let v = self.plan_expr(plan, f, value);
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Return { value: v },
                        span: *span,
                    });
                }
                RStmt::If {
                    cond,
                    then,
                    els,
                    span,
                } => {
                    let c = self.plan_expr(plan, f, cond);
                    let k_then = plan.blocks.len();
                    plan.blocks.push(BNode {
                        stmts: Vec::new(),
                        span_override: None,
                        span_cut: None,
                        fallback: then.span,
                        span: then.span,
                    });
                    stack.push(PFrame {
                        src: then,
                        idx: 0,
                        bid: k_then,
                        cur: k_then,
                        split_after: false,
                        pending_term: None,
                        cont: PCont::ThenArm {
                            cond: c,
                            els: els.as_ref(),
                            home,
                            span: *span,
                        },
                    });
                }
                RStmt::While {
                    cond,
                    body,
                    span,
                    target,
                } => {
                    // loop { if cond { body } else { break } }: the condition is
                    // owned by the if inside the loop, so it re-runs every
                    // iteration and after every continue.
                    ensure_loop(plan, target.0, cond.span);
                    let c = self.plan_expr(plan, f, cond);
                    let k_body = plan.blocks.len();
                    plan.blocks.push(BNode {
                        stmts: Vec::new(),
                        span_override: None,
                        span_cut: None,
                        fallback: body.span,
                        span: body.span,
                    });
                    stack.push(PFrame {
                        src: body,
                        idx: 0,
                        bid: k_body,
                        cur: k_body,
                        split_after: false,
                        pending_term: None,
                        cont: PCont::WhileArm {
                            cond: c,
                            home,
                            cond_span: cond.span,
                            stmt_span: *span,
                            target: target.0,
                        },
                    });
                }
                RStmt::Loop { body, span, target } => {
                    ensure_loop(plan, target.0, *span);
                    let k_body = plan.blocks.len();
                    plan.blocks.push(BNode {
                        stmts: Vec::new(),
                        span_override: None,
                        span_cut: None,
                        fallback: body.span,
                        span: body.span,
                    });
                    stack.push(PFrame {
                        src: body,
                        idx: 0,
                        bid: k_body,
                        cur: k_body,
                        split_after: false,
                        pending_term: None,
                        cont: PCont::LoopArm {
                            home,
                            span: *span,
                            target: target.0,
                        },
                    });
                }
                // PLAN-862 T-03a: on the entry-unit surface an expression
                // statement exists only as the SCRIPT tail (gated off before
                // planning). PLAN-864: on the collections surface it is the
                // unit-effect statement — the value is planned and emitted
                // as an `eval` statement (unit result, no slot).
                RStmt::ExprStmt { value, span }
                    if value.vt == VT::Unit
                        || matches!(value.kind, RExprKind::ListRemove { .. }) =>
                {
                    let v = self.plan_expr(plan, f, value);
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Eval { value: v },
                        span: *span,
                    });
                }
                RStmt::ExprStmt { .. } => {}
                RStmt::Break { target, span } => {
                    let t = self.phantom_target(plan, home, *target, *span);
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Break { target: t },
                        span: *span,
                    });
                }
                RStmt::Continue { target, span } => {
                    let t = self.phantom_target(plan, home, *target, *span);
                    plan.blocks[home].stmts.push(SNode {
                        kind: SKind::Continue { target: t },
                        span: *span,
                    });
                }
            }
            if term != Term::No {
                let frame = &mut stack[parent_idx];
                if opens_block {
                    frame.pending_term = Some(term);
                } else {
                    if plan.blocks[home].span_cut.is_none() {
                        let n = plan.blocks[home].stmts.len();
                        plan.blocks[home].span_cut = Some(n - 1);
                    }
                    frame.split_after = matches!(term, Term::Ret | Term::Div);
                }
            }
        }
    }

    /// Target id for a break/continue; stray ones get a phantom loop
    /// registered just before them in the same block so the existing
    /// verifier owns the rejection.
    fn phantom_target(
        &mut self,
        plan: &mut FnPlan,
        cur: usize,
        target: Option<super::resolve::LoopRef>,
        span: Span,
    ) -> usize {
        match target {
            Some(t) => t.0,
            None => {
                let ph = plan.loops.len();
                plan.loops.push(span);
                let k_empty = plan.blocks.len();
                plan.blocks.push(BNode {
                    stmts: Vec::new(),
                    span_override: Some(span),
                    span_cut: None,
                    fallback: span,
                    span: span,
                });
                plan.blocks[cur].stmts.push(SNode {
                    kind: SKind::Loop {
                        target: ph,
                        body: k_empty,
                    },
                    span,
                });
                ph
            }
        }
    }

    fn plan_expr(&mut self, plan: &mut FnPlan, f: &ResolvedFn, e: &ResolvedExpr) -> usize {
        // Iterative planning (R3-QA-02): legal operator chains are
        // unbounded in depth, so no recursion. Leaves plan through
        // `plan_leaf`; binary nodes combine pre-allocated expr indices
        // after both operands are planned (lhs first — same ordering as
        // the previous recursive form).
        enum Task<'a> {
            Plan {
                node: &'a ResolvedExpr,
                slot: usize,
            },
            /// nested-call assembly: all argument slots planned, build the
            /// call node (R5: calls must not recurse through plan_leaf)
            CallOut {
                node: &'a ResolvedExpr,
                dest: usize,
                arg_slots: Vec<usize>,
            },
            CallClosureOut {
                node: &'a ResolvedExpr,
                dest: usize,
                callee_slot: usize,
                arg_slots: Vec<usize>,
            },
            /// receiver + argument slots planned; assemble the builtin
            /// text method node (PLAN-770; iterative so method chains of
            /// any depth cannot recurse).
            TextOut {
                method: crate::source::resolve::BuiltinMethod,
                to: Option<VT>,
                vt: VT,
                span: Span,
                recv_slot: usize,
                arg_slots: Vec<usize>,
                dest: usize,
            },
            Combine {
                op: BinOp,
                span: Span,
                vt: VT,
                lslot: usize,
                rslot: usize,
                dest: usize,
            },
            /// operand slot planned; build the unary node
            UnaryOut {
                op: UnaryOp,
                span: Span,
                oslot: usize,
                dest: usize,
            },
        }
        // slot -> planned expr table index
        let mut slot_idx: Vec<usize> = vec![usize::MAX];
        let mut stack: Vec<Task> = vec![Task::Plan { node: e, slot: 0 }];
        while let Some(task) = stack.pop() {
            match task {
                Task::Plan { node, slot } => {
                    if let RExprKind::CallClosure { callee, args } = &node.kind {
                        let callee_slot = slot_idx.len();
                        slot_idx.push(usize::MAX);
                        let mut arg_slots = Vec::with_capacity(args.len());
                        let mut subtasks = Vec::with_capacity(args.len() + 2);
                        subtasks.push(Task::Plan {
                            node: callee,
                            slot: callee_slot,
                        });
                        for arg in args {
                            let arg_slot = slot_idx.len();
                            slot_idx.push(usize::MAX);
                            arg_slots.push(arg_slot);
                            subtasks.push(Task::Plan {
                                node: &arg.value,
                                slot: arg_slot,
                            });
                        }
                        subtasks.push(Task::CallClosureOut {
                            node,
                            dest: slot,
                            callee_slot,
                            arg_slots,
                        });
                        for task in subtasks.into_iter().rev() {
                            stack.push(task);
                        }
                        continue;
                    }
                    if let RExprKind::CallFn { args, .. } = &node.kind {
                        if !args.is_empty() {
                            let mut arg_slots = Vec::with_capacity(args.len());
                            let mut subtasks: Vec<Task> = Vec::with_capacity(args.len() + 1);
                            for a in args {
                                let s = slot_idx.len();
                                slot_idx.push(usize::MAX);
                                arg_slots.push(s);
                                subtasks.push(Task::Plan {
                                    node: &a.value,
                                    slot: s,
                                });
                            }
                            subtasks.push(Task::CallOut {
                                node,
                                dest: slot,
                                arg_slots,
                            });
                            for t in subtasks.into_iter().rev() {
                                stack.push(t);
                            }
                            continue;
                        }
                    }
                    if let RExprKind::Binary { op, lhs, rhs } = &node.kind {
                        let lslot = slot_idx.len();
                        slot_idx.push(usize::MAX);
                        let rslot = slot_idx.len();
                        slot_idx.push(usize::MAX);
                        stack.push(Task::Combine {
                            op: *op,
                            span: node.span,
                            vt: node.vt,
                            lslot,
                            rslot,
                            dest: slot,
                        });
                        stack.push(Task::Plan {
                            node: rhs,
                            slot: rslot,
                        });
                        stack.push(Task::Plan {
                            node: lhs,
                            slot: lslot,
                        });
                    } else if let RExprKind::Method {
                        recv,
                        method,
                        to,
                        args,
                    } = &node.kind
                    {
                        let recv_slot = slot_idx.len();
                        slot_idx.push(usize::MAX);
                        let mut arg_slots = Vec::with_capacity(args.len());
                        let mut subtasks = Vec::with_capacity(args.len() + 1);
                        // `.to` carries its target in `to`: none of its
                        // arguments are values; other methods plan each
                        // argument (TypeWord markers never plan)
                        if *method != crate::source::resolve::BuiltinMethod::To {
                            for a in args {
                                if matches!(a.value.kind, RExprKind::TypeWord(_)) {
                                    continue;
                                }
                                let sl = slot_idx.len();
                                slot_idx.push(usize::MAX);
                                arg_slots.push(sl);
                                subtasks.push(Task::Plan {
                                    node: &a.value,
                                    slot: sl,
                                });
                            }
                        }
                        subtasks.push(Task::TextOut {
                            method: *method,
                            to: *to,
                            vt: node.vt,
                            span: node.span,
                            recv_slot,
                            arg_slots,
                            dest: slot,
                        });
                        let mut ordered = Vec::new();
                        // the type-level char.from_codepoint receiver is a
                        // marker, not a value (slot stays usize::MAX)
                        if !matches!(recv.kind, RExprKind::TypeWord(_)) {
                            ordered.push(Task::Plan {
                                node: recv,
                                slot: recv_slot,
                            });
                        }
                        ordered.extend(subtasks);
                        for t in ordered.into_iter().rev() {
                            stack.push(t);
                        }
                    } else if let RExprKind::Unary { op, operand } = &node.kind {
                        let oslot = slot_idx.len();
                        slot_idx.push(usize::MAX);
                        stack.push(Task::UnaryOut {
                            op: *op,
                            span: node.span,
                            oslot,
                            dest: slot,
                        });
                        stack.push(Task::Plan {
                            node: operand,
                            slot: oslot,
                        });
                    } else {
                        slot_idx[slot] = self.plan_leaf(plan, f, node);
                    }
                }
                Task::CallOut {
                    node,
                    dest,
                    arg_slots,
                } => {
                    let RExprKind::CallFn { fn_index, args } = &node.kind else {
                        unreachable!("CallOut on non-call node")
                    };
                    let callee = self.program.fns.get(*fn_index);
                    let mut eval = Vec::with_capacity(arg_slots.len());
                    for (idx, s) in arg_slots.iter().enumerate() {
                        let planned = slot_idx[*s];
                        // PLAN-771 borrow discipline for record args (the
                        // iterative path mirrors plan_arg); a `.move`
                        // wrapper argument is a whole-root transfer, never
                        // a view
                        let arg_vt = plan.exprs[planned].vt;
                        let is_borrow = matches!(plan.exprs[planned].kind, EKind::BorrowMut { .. });
                        let arg_moved = matches!(plan.exprs[planned].kind, EKind::Move { .. });
                        if matches!(arg_vt, VT::Rec(_)) && !is_borrow && !arg_moved {
                            let pos = callee.and_then(|c| match &args[idx].name {
                                None => {
                                    if idx < c.params.len() {
                                        Some(idx)
                                    } else {
                                        None
                                    }
                                }
                                Some(n) => c.param_names.iter().position(|p| *p == n.name),
                            });
                            let param_mut = pos
                                .and_then(|i| callee.and_then(|c| c.param_modes.get(i).copied()))
                                .unwrap_or(false);
                            if !param_mut {
                                let span = plan.exprs[planned].span;
                                plan.exprs.push(ENode {
                                    kind: EKind::BorrowView { base: planned },
                                    vt: arg_vt,
                                    span,
                                });
                                eval.push(plan.exprs.len() - 1);
                                continue;
                            }
                        }
                        eval.push(planned);
                    }
                    let mut bindings = Vec::new();
                    let mut moved = Vec::new();
                    if let Some(callee) = self.program.fns.get(*fn_index) {
                        for (idx, a) in args.iter().enumerate() {
                            if let Some(pos) = param_position(idx, a, callee) {
                                bindings.push((pos as u32, idx as u32));
                                moved.push(a.moved);
                            }
                        }
                    }
                    plan.exprs.push(ENode {
                        kind: EKind::Call {
                            callee_fn: Some(*fn_index),
                            callee_intrinsic: None,
                            eval,
                            bindings,
                            moved,
                        },
                        vt: node.vt,
                        span: node.span,
                    });
                    slot_idx[dest] = plan.exprs.len() - 1;
                }
                Task::CallClosureOut {
                    node,
                    dest,
                    callee_slot,
                    arg_slots,
                } => {
                    let callee = slot_idx[callee_slot];
                    let eval = arg_slots.iter().map(|slot| slot_idx[*slot]).collect();
                    plan.exprs.push(ENode {
                        kind: EKind::CallClosure { callee, eval },
                        vt: node.vt,
                        span: node.span,
                    });
                    slot_idx[dest] = plan.exprs.len() - 1;
                }
                Task::TextOut {
                    method,
                    to,
                    vt,
                    span,
                    recv_slot,
                    arg_slots,
                    dest,
                } => {
                    let r = slot_idx[recv_slot];
                    let mut eval = Vec::with_capacity(arg_slots.len());
                    for sl in &arg_slots {
                        eval.push(slot_idx[*sl]);
                    }
                    plan.has_text_ops = true;
                    plan.exprs.push(ENode {
                        kind: EKind::Text {
                            method,
                            recv: r,
                            args: eval,
                            to,
                        },
                        vt,
                        span,
                    });
                    slot_idx[dest] = plan.exprs.len() - 1;
                }
                Task::Combine {
                    op,
                    span,
                    vt,
                    lslot,
                    rslot,
                    dest,
                } => {
                    let l = slot_idx[lslot];
                    let r = slot_idx[rslot];
                    match op {
                        BinOp::And => {
                            plan.exprs.push(ENode {
                                kind: EKind::Logical(crate::hir::LogicOp::And, l, r),
                                vt: VT::Bool,
                                span,
                            });
                            slot_idx[dest] = plan.exprs.len() - 1;
                        }
                        BinOp::Or => {
                            plan.exprs.push(ENode {
                                kind: EKind::Logical(crate::hir::LogicOp::Or, l, r),
                                vt: VT::Bool,
                                span,
                            });
                            slot_idx[dest] = plan.exprs.len() - 1;
                        }
                        _ => {
                            // `vt` is the RESOLVED result type: it carries
                            // the text-concat fact (PLAN-770) while
                            // arithmetic keeps the operator's i32 result
                            // type. r2 QA-02: a binary over TEXT operands
                            // (concat or eq/ne) lowers to ac_rt_* calls —
                            // the module must declare the runtime capability
                            // even when no method-call node exists.
                            if matches!(plan.exprs[l].vt, VT::Str | VT::StrOwned)
                                || matches!(plan.exprs[r].vt, VT::Str | VT::StrOwned)
                            {
                                plan.has_text_ops = true;
                            }
                            plan.exprs.push(ENode {
                                kind: EKind::Bin(op, l, r),
                                vt,
                                span,
                            });
                            slot_idx[dest] = plan.exprs.len() - 1;
                        }
                    }
                }
                Task::UnaryOut {
                    op,
                    span,
                    oslot,
                    dest,
                } => {
                    let o = slot_idx[oslot];
                    let (hop, vt) = match op {
                        UnaryOp::Neg => (crate::hir::UnaryOp::NegI32, VT::I32),
                        UnaryOp::Not => (crate::hir::UnaryOp::NotBool, VT::Bool),
                    };
                    plan.exprs.push(ENode {
                        kind: EKind::Unary(hop, o),
                        vt,
                        span,
                    });
                    slot_idx[dest] = plan.exprs.len() - 1;
                }
            }
        }
        slot_idx[0]
    }

    /// Plan a non-binary leaf (called only from the iterative walk).
    /// Plan one call argument: a record-typed argument to a view param is
    /// wrapped in a view borrow (ABI-D); `mut` record params receive the
    /// explicit `p.mut` borrow unchanged (a missing `.mut` stays a plain
    /// value and is rejected downstream); scalar args pass by value
    /// (PLAN-771 / 774 D-04).
    fn plan_arg(
        &mut self,
        plan: &mut FnPlan,
        f: &ResolvedFn,
        a: &ResolvedArg,
        idx: usize,
        callee: Option<&ResolvedFn>,
    ) -> usize {
        let is_record = matches!(a.value.vt, VT::Rec(_));
        let is_borrow_mut = matches!(a.value.kind, RExprKind::BorrowMut { .. });
        // a `.move` wrapper argument is a whole-root transfer, never a view
        // borrow (on the combination surface the wrapper IS the move
        // marker — CallBinding.moved is domain-forbidden there)
        if !is_record || is_borrow_mut || matches!(a.value.kind, RExprKind::Move { .. }) {
            return self.plan_expr(plan, f, &a.value);
        }
        let pos = callee.and_then(|c| match &a.name {
            None => {
                if idx < c.params.len() {
                    Some(idx)
                } else {
                    None
                }
            }
            Some(n) => c.param_names.iter().position(|p| *p == n.name),
        });
        let param_mut = pos
            .and_then(|i| callee.and_then(|c| c.param_modes.get(i).copied()))
            .unwrap_or(false);
        let slot = self.plan_expr(plan, f, &a.value);
        if param_mut {
            return slot; // missing `.mut` — plain value, rejected downstream
        }
        plan.exprs.push(ENode {
            kind: EKind::BorrowView { base: slot },
            vt: a.value.vt,
            span: a.value.span,
        });
        plan.exprs.len() - 1
    }

    fn plan_leaf(&mut self, plan: &mut FnPlan, f: &ResolvedFn, e: &ResolvedExpr) -> usize {
        match &e.kind {
            RExprKind::Binary { .. } | RExprKind::Unary { .. } => {
                unreachable!("binary/unary handled by plan_expr walk")
            }
            RExprKind::IntLit(v) => {
                plan.exprs.push(ENode {
                    kind: EKind::Const(*v),
                    vt: VT::I32,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CreateClosure { fn_index, captures } => {
                plan.exprs.push(ENode {
                    kind: EKind::CreateClosure {
                        callee_fn: *fn_index,
                        captures: captures
                            .iter()
                            .map(|(local, mode)| (local.0, *mode))
                            .collect(),
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::BoolLit(v) => {
                // true = 0 < 1, false = 0 < 0 — a fresh subgraph per
                // occurrence, spans pointing at the literal token.
                plan.exprs.push(ENode {
                    kind: EKind::Const(0),
                    vt: VT::I32,
                    span: e.span,
                });
                let z = plan.exprs.len() - 1;
                plan.exprs.push(ENode {
                    kind: EKind::Const(i32::from(*v)),
                    vt: VT::I32,
                    span: e.span,
                });
                let o = plan.exprs.len() - 1;
                plan.exprs.push(ENode {
                    kind: EKind::Bin(BinOp::Lt, z, o),
                    vt: VT::Bool,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::ReadLocal(l) => {
                plan.exprs.push(ENode {
                    kind: EKind::Read(l.0),
                    vt: f.locals[l.0].vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            // PLAN-862: the unit value (a Unit body's synthesized return
            // operand). Payload-free: one node, type `unit`.
            RExprKind::Unit => {
                plan.exprs.push(ENode {
                    kind: EKind::Unit,
                    vt: VT::Unit,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::StrLit(v) => {
                plan.has_text_ops = true;
                plan.exprs.push(ENode {
                    kind: EKind::StrLit(v.clone()),
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CharLit(v) => {
                plan.has_text_ops = true;
                plan.exprs.push(ENode {
                    kind: EKind::CharLit(*v),
                    vt: VT::Char,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::Method { .. } => {
                unreachable!("method nodes are planned by the iterative TextOut walk")
            }
            // RExprKind::Move planning lives with the other combination
            // kinds further down in this match (PLAN-782: on the text
            // surface the wrapper is still unwrapped by the call arms and
            // never reaches the plan; on the combination surface it is a
            // first-class value).
            RExprKind::TypeWord(_) => {
                // consumed by the Text planning arm; a stray marker was
                // rejected by the typecheck walk
                unreachable!("type-word markers are never planned as values")
            }
            RExprKind::CallFn { fn_index, args } => {
                let callee = self.program.fns.get(*fn_index);
                let mut eval = Vec::new();
                for (idx, a) in args.iter().enumerate() {
                    eval.push(self.plan_arg(plan, f, a, idx, callee));
                }
                let mut bindings = Vec::new();
                let mut moved = Vec::new();
                if let Some(callee) = self.program.fns.get(*fn_index) {
                    for (idx, a) in args.iter().enumerate() {
                        if let Some(pos) = param_position(idx, a, callee) {
                            bindings.push((pos as u32, idx as u32));
                            moved.push(a.moved);
                        }
                    }
                }
                plan.exprs.push(ENode {
                    kind: EKind::Call {
                        callee_fn: Some(*fn_index),
                        callee_intrinsic: None,
                        eval,
                        bindings,
                        moved,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CallClosure { .. } => {
                unreachable!("closure calls are planned by the iterative closure-call walk")
            }
            RExprKind::CtorPos { rec, args } => {
                // positional: eval order = source order; binding i -> i
                let mut eval = Vec::new();
                for a in args {
                    eval.push(self.plan_expr(plan, f, &a.value));
                }
                let bindings = (0..eval.len() as u32)
                    .map(|i| (i, i))
                    .collect::<Vec<(u32, u32)>>();
                plan.exprs.push(ENode {
                    kind: EKind::RecordCtor {
                        rec: *rec,
                        eval,
                        bindings,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CtorNamed { rec, fields } => {
                // named: eval order = WRITTEN order; bindings map each
                // declaration field to its argument (never re-sorted by
                // field order; PLAN-771 §5.2)
                let mut eval = Vec::new();
                for (_, v) in fields {
                    eval.push(self.plan_expr(plan, f, v));
                }
                let mut bindings = Vec::new();
                for (ai, (name, _)) in fields.iter().enumerate() {
                    if let Some(fi) = self.program.records[*rec]
                        .fields
                        .iter()
                        .position(|fl| fl.name == *name)
                    {
                        bindings.push((fi as u32, ai as u32));
                    }
                }
                bindings.sort_by_key(|(fi, _)| *fi);
                plan.exprs.push(ENode {
                    kind: EKind::RecordCtor {
                        rec: *rec,
                        eval,
                        bindings,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::FieldGet { base, path } => {
                // flatten chained single-segment FieldGets into one path
                let mut segs: Vec<u32> = path.iter().map(|(_, fi)| *fi as u32).collect();
                let mut cur = base;
                loop {
                    let next = match &cur.kind {
                        RExprKind::FieldGet {
                            base: inner,
                            path: ip,
                        } => {
                            let mut more: Vec<u32> = ip.iter().map(|(_, fi)| *fi as u32).collect();
                            more.extend(segs);
                            segs = more;
                            inner
                        }
                        _ => break,
                    };
                    cur = next;
                }
                let base_slot = self.plan_expr(plan, f, cur);
                plan.exprs.push(ENode {
                    kind: EKind::FieldGet {
                        base: base_slot,
                        path: segs,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::RecordClone { base } => {
                let VT::Rec(ri) = e.vt else {
                    // unreachable: resolve dispatch types clones as records
                    panic!("record clone planned with non-record type");
                };
                let bs = self.plan_expr(plan, f, base.as_ref());
                plan.has_text_ops = true; // clone lowers runtime calls for Str fields
                plan.exprs.push(ENode {
                    kind: EKind::RecordClone { rec: ri, base: bs },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::Move { inner } => {
                // combination profile: the wrapper survives as the value
                // (let/assign/return positions and — via resolve keeping
                // the wrapper — call argument values).
                let is = self.plan_expr(plan, f, inner.as_ref());
                plan.exprs.push(ENode {
                    kind: EKind::Move { inner: is },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            // -- PLAN-864 collections surface: leaf planning with child
            // recursion through the driver, mirroring the RecordClone arm.
            RExprKind::ListNew { list } => {
                plan.exprs.push(ENode {
                    kind: EKind::ListNew { list: *list },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::ListLen { list } | RExprKind::ListClear { list } => {
                let l = self.plan_expr(plan, f, list);
                let kind = match &e.kind {
                    RExprKind::ListLen { .. } => EKind::ListLen { list: l },
                    _ => EKind::ListClear { list: l },
                };
                plan.exprs.push(ENode {
                    kind,
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::ListPush { list, value } => {
                let l = self.plan_expr(plan, f, list);
                let v = self.plan_expr(plan, f, value);
                plan.exprs.push(ENode {
                    kind: EKind::ListPush { list: l, value: v },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::ListGet { list, index } | RExprKind::ListRemove { list, index } => {
                let l = self.plan_expr(plan, f, list);
                let i = self.plan_expr(plan, f, index);
                let kind = match &e.kind {
                    RExprKind::ListGet { .. } => EKind::ListGet { list: l, index: i },
                    _ => EKind::ListRemove { list: l, index: i },
                };
                plan.exprs.push(ENode {
                    kind,
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::ListInsert { list, index, value }
            | RExprKind::ListSet { list, index, value } => {
                let l = self.plan_expr(plan, f, list);
                let i = self.plan_expr(plan, f, index);
                let v = self.plan_expr(plan, f, value);
                let kind = match &e.kind {
                    RExprKind::ListInsert { .. } => EKind::ListInsert {
                        list: l,
                        index: i,
                        value: v,
                    },
                    _ => EKind::ListSet {
                        list: l,
                        index: i,
                        value: v,
                    },
                };
                plan.exprs.push(ENode {
                    kind,
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::ListFieldGet { list, index, path }
            | RExprKind::ElemView { list, index, path } => {
                // PLAN-864: the HIR shape is `field_get` over a `list_get`
                // (the lowering fuses the pair into LIST_FIELD_GET or the
                // remove/view/read/reinsert sequence); a whole-element
                // view renders as the bare list_get whose terminal type
                // is Str. HIR verify owns the view-escape rules.
                let l = self.plan_expr(plan, f, list);
                let i = self.plan_expr(plan, f, index);
                let elem_vt = match list.vt {
                    VT::List(li) => self.program.lists.get(li).copied().unwrap_or(e.vt),
                    _ => e.vt,
                };
                plan.exprs.push(ENode {
                    kind: EKind::ListGet { list: l, index: i },
                    vt: elem_vt,
                    span: e.span,
                });
                let get_id = plan.exprs.len() - 1;
                if path.is_empty() {
                    return get_id;
                }
                let segs: Vec<u32> = path.iter().map(|(_, fi)| *fi as u32).collect();
                plan.exprs.push(ENode {
                    kind: EKind::FieldGet {
                        base: get_id,
                        path: segs,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::VariantCtor {
                variant,
                disc,
                payload,
            } => {
                let pl = payload.as_ref().map(|p| self.plan_expr(plan, f, p));
                plan.exprs.push(ENode {
                    kind: EKind::VariantCtor {
                        variant: *variant,
                        disc: *disc,
                        payload: pl,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::VariantGuardTest {
                local,
                variant,
                disc,
            }
            | RExprKind::VariantProject {
                local,
                variant,
                disc,
            } => {
                let r = plan.exprs.len();
                plan.exprs.push(ENode {
                    kind: EKind::Read(local.0),
                    vt: f.locals[local.0].vt,
                    span: e.span,
                });
                let kind = match &e.kind {
                    RExprKind::VariantGuardTest { .. } => EKind::VariantGuardTest {
                        root: r,
                        variant: *variant,
                        disc: *disc,
                    },
                    _ => EKind::VariantProject {
                        root: r,
                        variant: *variant,
                        disc: *disc,
                    },
                };
                plan.exprs.push(ENode {
                    kind,
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CtorUnknown { name, .. } => {
                let _ = name;
                unreachable!("resolver: CtorUnknown never escapes resolution")
            }
            RExprKind::MethodCall {
                fn_index,
                recv,
                args,
            } => {
                let callee = self.program.fns.get(*fn_index);
                let mut eval = Vec::new();
                let mut bindings = Vec::new();
                // moved flags stay PARALLEL TO `bindings` (one flag per
                // emitted binding; the receiver binding is never a move)
                let mut moved: Vec<bool> = Vec::new();
                if let Some(r) = recv {
                    // instance: the receiver is evaluated EXACTLY ONCE,
                    // first, as a view borrow (param 0)
                    let rs = self.plan_expr(plan, f, r);
                    plan.exprs.push(ENode {
                        kind: EKind::BorrowView { base: rs },
                        vt: r.vt,
                        span: r.span,
                    });
                    eval.push(plan.exprs.len() - 1);
                    bindings.push((0u32, 0u32));
                    moved.push(false);
                }
                let recv_shift = if recv.is_some() { 1usize } else { 0 };
                for (idx, a) in args.iter().enumerate() {
                    eval.push(self.plan_arg(plan, f, a, idx, callee));
                }
                if let Some(c) = callee {
                    for (idx, a) in args.iter().enumerate() {
                        if let Some(pos) = param_position(idx, a, c) {
                            // PLAN-881: POSITIONAL arguments index the
                            // WRITTEN params (`param_position` returns the
                            // source index) and need the receiver shift;
                            // NAMED arguments look the receiver-INCLUSIVE
                            // param-name table up (a view method's names
                            // lead with `self`), so the shift is already
                            // in the position — applying it twice bound
                            // one-past-the-end params.
                            let param = match &a.name {
                                None => pos + recv_shift,
                                Some(_) => pos,
                            };
                            bindings.push((param as u32, (idx + recv_shift) as u32));
                            moved.push(a.moved);
                        }
                    }
                }
                plan.exprs.push(ENode {
                    kind: EKind::Call {
                        callee_fn: Some(*fn_index),
                        callee_intrinsic: None,
                        eval,
                        bindings,
                        moved,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::BorrowMut { local } => {
                plan.exprs.push(ENode {
                    kind: EKind::BorrowMut { local: local.0 },
                    vt: f.locals[local.0].vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
            RExprKind::CallIntrinsic { symbol, args } => {
                let mut eval: Vec<usize> = Vec::new();
                for arg in args {
                    let mut i = self.plan_expr(plan, f, arg);
                    if arg.vt == VT::StrOwned {
                        plan.exprs.push(ENode {
                            kind: EKind::Text {
                                method: super::resolve::BuiltinMethod::To,
                                recv: i,
                                args: Vec::new(),
                                to: Some(VT::Str),
                            },
                            vt: VT::Str,
                            span: arg.span,
                        });
                        i = plan.exprs.len() - 1;
                    }
                    eval.push(i);
                }
                let bindings = (0..eval.len()).map(|i| (i as u32, i as u32)).collect();
                let moved = vec![false; eval.len()];
                self.intrinsic_first_use.entry(symbol).or_insert(e.span);
                plan.exprs.push(ENode {
                    kind: EKind::Call {
                        callee_fn: None,
                        callee_intrinsic: Some(symbol),
                        eval,
                        bindings,
                        moved,
                    },
                    vt: e.vt,
                    span: e.span,
                });
                plan.exprs.len() - 1
            }
        }
    }
}
/// How a statement's execution leaves its block, mirroring the verifier's
/// conservative Flow semantics (used only to decide where dead code starts
/// so the verifier can anchor its rejection at the illegal trailing
/// statement; the verifier itself stays the authority).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Term {
    /// Control falls through to the next statement.
    No,
    /// All paths leave via `return`.
    Ret,
    /// All paths leave via a valid `break`/`continue` (or a stray one —
    /// still Diverged for the verifier's flow lattice).
    Div,
    /// A stray `break`/`continue` with no enclosing loop: a terminator for
    /// the walk, but trailing statements must stay in the SAME block so the
    /// dataflow walk still runs and owns the loop-target-scope rejection.
    Stray,
}

fn stmt_term_memo(s: &RStmt, terms: &BTreeMap<usize, Option<Term>>) -> Term {
    let flow_of = |b: &RBlock| -> Option<Term> {
        terms
            .get(&(b as *const RBlock as usize))
            .copied()
            .unwrap_or(None)
    };
    match s {
        RStmt::Return { .. } => Term::Ret,
        // PLAN-862: reachable only in the gated-off entry-unit emitter.
        RStmt::ExprStmt { .. } => Term::No,
        RStmt::Break { target, .. } | RStmt::Continue { target, .. } => match target {
            Some(_) => Term::Div,
            None => Term::Stray,
        },
        RStmt::If { then, els, .. } => match els {
            // (Returned, Returned) -> Returned; (Diverged, Diverged) ->
            // Diverged; anything mixed (or no else) continues conservatively,
            // exactly like verify's flow lattice. A Stray arm NEVER counts
            // as a valid terminator for the combination (R2-QA-03): folding
            // (Stray, Stray) into Diverged split trailing statements into
            // disconnected dead blocks, verify's structure gate failed
            // first and skipped the dataflow walk — hiding the real
            // loop-target-scope errors (D10). Stray arms keep trailing
            // statements in the same block so the walk runs and owns the
            // rejection; the pure (Div, Div) case (D08) still splits.
            Some(e) => {
                let (a, b) = (flow_of(then), flow_of(e));
                if a == Some(Term::Stray) || b == Some(Term::Stray) {
                    Term::Stray
                } else {
                    match (a, b) {
                        (Some(Term::Ret), Some(Term::Ret)) => Term::Ret,
                        (Some(Term::Div), Some(Term::Div)) => Term::Div,
                        _ => Term::No,
                    }
                }
            }
            None => Term::No,
        },
        RStmt::Loop { body, .. } => {
            if flow_of(body) == Some(Term::Ret) {
                Term::Ret
            } else {
                Term::No
            }
        }
        RStmt::While { .. } | RStmt::Let { .. } | RStmt::Assign { .. } => Term::No,
    }
}

/// First-terminator classification of every block in the tree, keyed by
/// block address, computed bottom-up with an explicit stack (R6: unlimited
/// block nesting — the recursive per-level restatement was both a depth
/// hazard and quadratic on deep chains).
fn block_terms(root: &RBlock) -> BTreeMap<usize, Option<Term>> {
    enum Task<'a> {
        Enter(&'a RBlock),
        Exit(&'a RBlock),
    }
    let key = |b: &RBlock| b as *const RBlock as usize;
    let mut memo: BTreeMap<usize, Option<Term>> = BTreeMap::new();
    let mut stack: Vec<Task> = vec![Task::Enter(root)];
    while let Some(task) = stack.pop() {
        match task {
            Task::Enter(b) => {
                if memo.contains_key(&key(b)) {
                    continue;
                }
                stack.push(Task::Exit(b));
                for s in &b.stmts {
                    match s {
                        RStmt::If { then, els, .. } => {
                            stack.push(Task::Enter(then));
                            if let Some(e) = els {
                                stack.push(Task::Enter(e));
                            }
                        }
                        RStmt::While { body, .. } | RStmt::Loop { body, .. } => {
                            stack.push(Task::Enter(body));
                        }
                        _ => {}
                    }
                }
            }
            Task::Exit(b) => {
                let flow = b
                    .stmts
                    .iter()
                    .map(|s| stmt_term_memo(s, &memo))
                    .find(|t| *t != Term::No);
                memo.insert(key(b), flow);
            }
        }
    }
    memo
}

fn ensure_loop(plan: &mut FnPlan, id: usize, span: Span) {
    while plan.loops.len() <= id {
        plan.loops.push(span);
    }
    plan.loops[id] = span;
}

fn stmt_span(s: &RStmt) -> Span {
    match s {
        RStmt::Let { span, .. }
        | RStmt::Assign { span, .. }
        | RStmt::Return { span, .. }
        | RStmt::If { span, .. }
        | RStmt::While { span, .. }
        | RStmt::Loop { span, .. }
        | RStmt::Break { span, .. }
        | RStmt::Continue { span, .. } => *span,
        // PLAN-862: the script tail's own span.
        RStmt::ExprStmt { span, .. } => *span,
    }
}

// ---------------------------------------------------------------------------
// Rendering: deterministic Atom text + fragment/entry span records
// ---------------------------------------------------------------------------

/// Entry spans recorded during rendering, aligned with the binder's table
/// order (the re-anchoring projection).
struct EmitSpans {
    types: Vec<Span>,
    defs: Vec<Span>,
    bodies: Vec<BodySpans>,
}

struct BodySpans {
    span: Span,
    locals: Vec<Span>,
    places: Vec<Span>,
    exprs: Vec<Span>,
    blocks: Vec<Span>,
}

struct Emitter<'p> {
    program: &'p ResolvedProgram,
    src_len: usize,
    profile: SourceProfile,
    plans: Vec<FnPlan>,
    intrinsic_first_use: BTreeMap<&'static str, Span>,
    /// Filled by [`Emitter::render`]: token-level fallback map.
    frags: Vec<(Span, Span)>,
    spans: Option<EmitSpans>,
    /// PLAN-828 D-01 single-function scope: when set, exactly this
    /// function's BODY is planned/rendered; every other function emits a
    /// BODY-LESS declaration scaffold. The types section stays the full
    /// interface directory.
    scope: Option<usize>,
}

impl<'p> Emitter<'p> {
    fn new(program: &'p ResolvedProgram, src_len: usize, profile: SourceProfile) -> Self {
        Emitter {
            program,
            src_len,
            profile,
            plans: Vec::new(),
            intrinsic_first_use: BTreeMap::new(),
            frags: Vec::new(),
            spans: None,
            scope: None,
        }
    }

    fn plan_module(&mut self) {
        for f in &self.program.fns {
            let mut ctx = PlanCtx {
                program: self.program,
                intrinsic_first_use: &mut self.intrinsic_first_use,
            };
            let plan = ctx.plan_fn(f);
            self.plans.push(plan);
        }
    }

    /// PLAN-828 D-01: plan the ONE real changed body plus SIGNATURE
    /// SCAFFOLDS (empty-body stubs) for every other function —
    /// cross-function references in the document bind against the
    /// scaffolds (the interface-directory traversal §5.6 allows), and no
    /// unchanged body is walked.
    /// PLAN-828 D-01: plan exactly ONE function's body (the changed
    /// one); every other function never enters planning at all.
    fn plan_indexed(&mut self, idx: usize) {
        let mut ctx = PlanCtx {
            program: self.program,
            intrinsic_first_use: &mut self.intrinsic_first_use,
        };
        let plan = ctx.plan_fn(&self.program.fns[idx]);
        self.plans.push(plan);
    }

    fn ty_name(&self, vt: VT) -> String {
        ty_name(
            vt,
            &self.program.records,
            &self.program.lists,
            &self.program.variants,
        )
    }

    /// PLAN-862: a TypeUse reference as emitted text. Named types are the
    /// quoted table ids (`"t_i32"`), the text builtins are barewords
    /// (`str`/`Str`/`char`) and the unit result is the bareword `unit`
    /// (there is no `t_unit` table entry: Unit is a builtin word).
    fn ty_use_ref(&self, vt: VT) -> String {
        match vt {
            VT::Unit => "unit".to_string(),
            other => format!("\"{}\"", self.ty_name(other)),
        }
    }

    fn esc(s: &str) -> String {
        let mut out = String::new();
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                _ => out.push(c),
            }
        }
        out
    }

    fn render(&mut self, module_path: &str) -> String {
        let program_span = Span {
            start: 0,
            end: self.src_len,
        };
        let mut t = String::new();
        let mut frags = Vec::new();
        let mut spans = EmitSpans {
            types: Vec::new(),
            defs: Vec::new(),
            bodies: Vec::new(),
        };

        macro_rules! frag {
            ($text:expr, $src:expr) => {{
                let start = t.len();
                t.push_str($text);
                frags.push((
                    Span {
                        start,
                        end: t.len(),
                    },
                    $src,
                ));
            }};
        }
        macro_rules! nl {
            ($indent:expr) => {{
                t.push('\n');
                for _ in 0..$indent {
                    t.push(' ');
                }
            }};
        }

        let use_intrinsics: Vec<&'static str> = self.intrinsic_first_use.keys().copied().collect();

        // The generated document carries the profile's HIR identity triple;
        // the binder stamps the matching domain and the domain-gated verify
        // entry owns the rest (PLAN-767 §5.1).
        let (schema, profile_word) = match self.profile.hir_domain() {
            crate::hir::HirDomain::Core => ("auto.hir.core.draft", "core-i32-draft"),
            crate::hir::HirDomain::Scalar => ("auto.hir.scalar.draft", "scalar-i32-draft"),
            crate::hir::HirDomain::Text => ("auto.hir.text.draft", "text-i32-draft"),
            crate::hir::HirDomain::Record => ("auto.hir.record.draft", "record-i32-draft"),
            crate::hir::HirDomain::TextRecord => {
                ("auto.hir.text-record.draft", "text-record-i32-draft")
            }
            // PLAN-862: the entry-unit emitter is gated off before render.
            crate::hir::HirDomain::EntryUnit => ("auto.hir.entry-unit.draft", "entry-unit-draft"),
            crate::hir::HirDomain::HostConsole => {
                ("auto.hir.host-console.draft", "host-console-draft")
            }
            // PLAN-864: the collections emitter renders the collections
            // identity triple (List/Variant types + collection expressions).
            crate::hir::HirDomain::Collections => {
                ("auto.hir.collections.draft", "collections-draft")
            }
            crate::hir::HirDomain::Closures => ("auto.hir.closures.draft", "source-closures-draft"),
        };
        frag!(
            &format!(
                "hir(schema: \"{}\", revision: 1u32, profile: \"{}\")",
                schema, profile_word
            ),
            program_span
        );
        t.push(' ');
        frag!("{", program_span);
        nl!(4);

        let mut requires: Vec<&'static str> = Vec::new();
        for sym in &use_intrinsics {
            let cap = super::console::signature(sym)
                .map(|(_, cap)| cap)
                .unwrap_or(TRACE_CAPABILITY);
            if !requires.contains(&cap) {
                requires.push(cap);
            }
        }
        if self.profile.is_text() && self.plans.iter().any(|p| p.has_text_ops) {
            requires.push(RUNTIME_TEXT_CAPABILITY);
        }
        if !requires.is_empty() {
            let req_span = if use_intrinsics.is_empty() {
                program_span
            } else {
                self.intrinsic_first_use[use_intrinsics[0]]
            };
            let items: Vec<String> = requires.iter().map(|r| format!("\"{}\"", r)).collect();
            frag!(&format!("requires: [{}]", items.join(", ")), req_span);
            nl!(4);
        }

        frag!("module m_src", program_span);
        t.push(' ');
        frag!("{", program_span);
        nl!(8);
        frag!(
            &format!("path: \"{}\"", Self::esc(module_path)),
            program_span
        );
        nl!(8);

        // -- types ----------------------------------------------------------
        frag!("types", program_span);
        t.push(' ');
        frag!("{", program_span);
        nl!(12);
        frag!("type t_i32 i32 {}", program_span);
        spans.types.push(program_span);
        nl!(12);
        frag!("type t_bool bool {}", program_span);
        spans.types.push(program_span);
        nl!(12);
        if self.profile.is_text() {
            frag!("type t_str str {}", program_span);
            spans.types.push(program_span);
            nl!(12);
            frag!("type t_Str Str {}", program_span);
            spans.types.push(program_span);
            nl!(12);
            frag!("type t_char char {}", program_span);
            spans.types.push(program_span);
            nl!(12);
        }
        // PLAN-864: the collections surface owns `Str` values (literals,
        // fields, elements) — the named entry the ty_name ref resolves to.
        if self.profile.is_collections() {
            if self.profile == super::SourceProfile::Closures {
                frag!("type t_str str {}", program_span);
                spans.types.push(program_span);
                nl!(12);
            }
            frag!("type t_Str Str {}", program_span);
            spans.types.push(program_span);
            nl!(12);
        }

        // PLAN-771: record types (declaration order; fields carry their
        // value types — builtin or record refs).
        let is_record_profile = self.profile.is_record();
        if is_record_profile {
            for rec in &self.program.records {
                let fields: Vec<String> = rec
                    .fields
                    .iter()
                    .map(|fl| {
                        format!(
                            "{{ name: \"{}\", type: {} }}",
                            Self::esc(&fl.name),
                            ty_name(
                                fl.vt,
                                &self.program.records,
                                &self.program.lists,
                                &self.program.variants,
                            )
                        )
                    })
                    .collect();
                frag!(
                    &format!(
                        "type t_rec_{} record {{ fields: [{}] }}",
                        rec.name,
                        fields.join(", ")
                    ),
                    rec.name_span
                );
                spans.types.push(rec.name_span);
                nl!(12);
            }
        }
        // PLAN-864: List/Variant type-table entries (collections surface).
        if self.profile.is_collections() {
            for (_li, elem) in self.program.lists.iter().enumerate() {
                let ename = ty_name(
                    *elem,
                    &self.program.records,
                    &self.program.lists,
                    &self.program.variants,
                );
                frag!(
                    &format!(
                        "type t_list_{} list {{ elem: {} }}",
                        ename.trim_start_matches("t_"),
                        ename
                    ),
                    program_span
                );
                spans.types.push(program_span);
                nl!(12);
            }
            for v in &self.program.variants {
                let arms: Vec<String> = v
                    .arms
                    .iter()
                    .map(|(n, p)| match p {
                        Some(pvt) => format!(
                            "{{ name: \"{}\", payload: {} }}",
                            Self::esc(n),
                            ty_name(
                                *pvt,
                                &self.program.records,
                                &self.program.lists,
                                &self.program.variants,
                            )
                        ),
                        None => format!("{{ name: \"{}\" }}", Self::esc(n)),
                    })
                    .collect();
                frag!(
                    &format!(
                        "type t_variant_{} variant {{ arms: [{}] }}",
                        v.name,
                        arms.join(", ")
                    ),
                    v.name_span
                );
                spans.types.push(v.name_span);
                nl!(12);
            }
        }
        // PLAN-828 T-02: in single-function scope, every span NOT belonging
        // to the scoped function is inert interface metadata — its anchor is
        // the RETAINED environment's own source, not the src passed to this
        // emission, so it may legitimately fall outside the (re-written)
        // current source. Such spans emit as zero (never consumed; same
        // rationale as the old zeroed stub items) so re-anchoring only ever
        // validates spans that really anchor the passed source.
        let meta_span = |fi: usize| -> Span {
            if self.scope.is_some_and(|s| s != fi) {
                Span { start: 0, end: 0 }
            } else {
                self.program.fns[fi].item_span
            }
        };
        // PLAN-865: source function types have structural identity within
        // this module and lower to the existing HIR function type shape.
        if self.profile.is_closures() {
            for (index, signature) in self.program.function_types.iter().enumerate() {
                let params: Vec<String> = signature
                    .params
                    .iter()
                    .map(|param| self.ty_use_ref(*param))
                    .collect();
                frag!(
                    &format!(
                        "type t_closure_sig_{} closure {{ params: [{}]; result: {} }}",
                        index,
                        params.join(", "),
                        self.ty_use_ref(signature.result)
                    ),
                    program_span
                );
                spans.types.push(program_span);
                nl!(12);
            }
        }
        for (fi, f) in self.program.fns.iter().enumerate() {
            let params: Vec<String> = f.params.iter().map(|p| self.ty_use_ref(*p)).collect();
            let tid = if f.receiver == crate::source::ast::Receiver::Free {
                format!("t_f_{}", f.name)
            } else {
                format!("t_f{}_{}", fi, f.name)
            };
            let span = meta_span(fi);
            frag!(
                &format!(
                    "type {} function {{ params: [{}]; result: {} }}",
                    tid,
                    params.join(", "),
                    self.ty_use_ref(f.result)
                ),
                span
            );
            spans.types.push(span);
            nl!(12);
        }
        for sym in &use_intrinsics {
            let span = self.intrinsic_first_use[*sym];
            frag!(
                &format!(
                    "type {} function {{ params: [{}]; result: {} }}",
                    intrinsic_type_id(sym),
                    super::console::signature(sym)
                        .map(|(ty, _)| self.ty_use_ref(ty))
                        .unwrap_or_default(),
                    if super::console::signature(sym).is_some() {
                        "unit"
                    } else {
                        "\"t_i32\""
                    }
                ),
                span
            );
            spans.types.push(span);
            nl!(12);
        }
        frag!("}", program_span);
        nl!(8);

        // -- declarations -----------------------------------------------------
        frag!("declarations", program_span);
        t.push(' ');
        frag!("{", program_span);
        nl!(12);
        for (fi, f) in self.program.fns.iter().enumerate() {
            // The declared `name` is the object-file symbol. Source
            // functions carry an `sf_` prefix so the native namespace stays
            // isolated from the backend's fixed imports/exports
            // (`ExitProcess`, `ac_start`): a source function may legally
            // carry either name (QA-02) without a symbol-table collision.
            // Identity stays in the def id, so resolution, recursion and
            // the CLI's entry-by-name lookup are unaffected. Methods
            // (record profile) carry index-qualified ids: a free function
            // and a method may share a source name (PLAN-771).
            // PLAN-828 D-01: in single-function scope every OTHER
            // function emits a BODY-LESS declaration scaffold — the def
            // exists for cross-function references without any body
            // document.
            let scaffold = self.scope.is_some() && self.scope != Some(fi);
            if f.receiver == crate::source::ast::Receiver::Free {
                let body_field = if scaffold {
                    String::new()
                } else {
                    format!("; body: \"b_{}\"", f.name)
                };
                // T-02: a scaffold declaration's span is inert metadata
                // (see the types section note) — zero, never the retained
                // entry's stale source span.
                let span = if scaffold {
                    Span { start: 0, end: 0 }
                } else {
                    f.item_span
                };
                frag!(
                    &format!(
                        "function d_{} t_f_{} {{ name: \"sf_{}\"{} }}",
                        f.name,
                        f.name,
                        Self::esc(&f.name),
                        body_field
                    ),
                    span
                );
                spans.defs.push(span);
                nl!(12);
                continue;
            }
            let rec_name = f
                .owner_rec
                .map(|ri| self.program.records[ri].name.clone())
                .unwrap_or_default();
            let recv_word = match f.receiver {
                crate::source::ast::Receiver::Static => "static",
                _ => "view",
            };
            // PLAN-881 (R3-F07 parity): a SCAFFOLD method declaration is
            // body-less — the single-method document keeps the owner/
            // receiver/mode facts without a body document (the binder
            // rejects a dangling body reference otherwise).
            let body_field = if scaffold {
                String::new()
            } else {
                format!("; body: \"b_f{}_{}\"", fi, f.name)
            };
            frag!(
                &format!(
                    "method d_f{}_{} t_f{}_{} {{ name: \"sm_{}_{}\"{}; record: t_rec_{}; receiver: {} }}",
                    fi,
                    f.name,
                    fi,
                    f.name,
                    rec_name,
                    f.name,
                    body_field,
                    rec_name,
                    recv_word
                ),
                f.item_span
            );
            spans.defs.push(f.item_span);
            nl!(12);
        }
        for sym in &use_intrinsics {
            let span = self.intrinsic_first_use[*sym];
            frag!(
                &format!(
                    "intrinsic {} {} {{ symbol: \"{}\"; effect: observable }}",
                    intrinsic_def_id(sym),
                    intrinsic_type_id(sym),
                    sym
                ),
                span
            );
            spans.defs.push(span);
            nl!(12);
        }
        frag!("}", program_span);
        nl!(8);

        // -- bodies ------------------------------------------------------------
        frag!("bodies", program_span);
        t.push(' ');
        frag!("{", program_span);
        nl!(12);
        for (fi, f) in self
            .program
            .fns
            .iter()
            .enumerate()
            .filter(|(i, _)| self.scope.is_none() || self.scope == Some(*i))
        {
            let plan = &self.plans[if self.scope.is_some() { 0 } else { fi }];
            let mut bs = BodySpans {
                span: f.name_span,
                locals: Vec::new(),
                places: Vec::new(),
                exprs: Vec::new(),
                blocks: Vec::new(),
            };
            let (body_id, def_id) = if f.receiver == crate::source::ast::Receiver::Free {
                (format!("b_{}", f.name), format!("d_{}", f.name))
            } else {
                (
                    format!("b_f{}_{}", fi, f.name),
                    format!("d_f{}_{}", fi, f.name),
                )
            };
            frag!(&format!("body {} {{", body_id), f.name_span);
            nl!(16);
            frag!(&format!("owner: \"{}\"", def_id), f.name_span);
            nl!(16);
            frag!("entry: \"k0\"", f.name_span);
            nl!(16);

            // locals
            frag!("locals", f.name_span);
            t.push(' ');
            frag!("{", f.name_span);
            nl!(20);
            for (li, l) in f.locals.iter().enumerate() {
                // text (PLAN-770): emit the mode word when it deviates
                // from the default view; record (PLAN-771): the record
                // schema REQUIRES the mode on params. The combination
                // surface (782) additionally carries `move` (its schema
                // has the three-case enum; the plain record schema is
                // view/mut only and its front door never yields move).
                let mode_suffix = if l.is_param {
                    if self.profile.is_record() {
                        format!(
                            "; mode: {}",
                            match l.mode {
                                super::ast::ParamMode::Mut => "mut",
                                super::ast::ParamMode::Move
                                    if self.profile == super::SourceProfile::TextRecord =>
                                {
                                    "move"
                                }
                                _ => "view",
                            }
                        )
                    } else {
                        match l.mode {
                            super::ast::ParamMode::Mut => "; mode: mut".to_string(),
                            super::ast::ParamMode::Move => "; mode: move".to_string(),
                            super::ast::ParamMode::Default => String::new(),
                        }
                    }
                } else {
                    String::new()
                };
                let line = if l.is_param {
                    format!(
                        "local l{} {} {{ name: \"{}\"; role: param; index: {}u32{} }}",
                        li,
                        self.ty_name(l.vt),
                        Self::esc(&l.name),
                        l.param_index.unwrap_or(0),
                        mode_suffix
                    )
                } else if l.is_capture {
                    format!(
                        "local l{} {} {{ name: \"{}\"; role: capture; index: {}u32 }}",
                        li,
                        self.ty_name(l.vt),
                        Self::esc(&l.name),
                        l.capture_index.unwrap_or(0)
                    )
                } else {
                    format!(
                        "local l{} {} {{ name: \"{}\"; role: binding; mutable: {} }}",
                        li,
                        self.ty_name(l.vt),
                        Self::esc(&l.name),
                        l.mutable
                    )
                };
                frag!(&line, l.span);
                bs.locals.push(l.span);
                nl!(20);
            }
            frag!("}", f.name_span);
            nl!(16);

            // places
            if !plan.places.is_empty() {
                frag!("places", f.name_span);
                t.push(' ');
                frag!("{", f.name_span);
                nl!(20);
                for (pi, p) in plan.places.iter().enumerate() {
                    let (kind_word, path_field) = if p.path.is_empty() {
                        ("local", String::new())
                    } else {
                        let segs: Vec<String> =
                            p.path.iter().map(|f| format!("{}u32", f)).collect();
                        ("field_path", format!("; path: [{}]", segs.join(", ")))
                    };
                    frag!(
                        &format!(
                            "place p{} {} {{ local: l{}; type: {}; access: mutable{} }}",
                            pi,
                            kind_word,
                            p.local,
                            self.ty_name(p.vt),
                            path_field
                        ),
                        p.span
                    );
                    bs.places.push(p.span);
                    nl!(20);
                }
                frag!("}", f.name_span);
                nl!(16);
            }

            // expressions
            if !plan.exprs.is_empty() {
                frag!("expressions", f.name_span);
                t.push(' ');
                frag!("{", f.name_span);
                nl!(20);
                for (ei, e) in plan.exprs.iter().enumerate() {
                    let ty = self.ty_name(e.vt);
                    let line = match &e.kind {
                        EKind::Const(v) => {
                            format!("expr e{} constant {{ type: {}; value: {}i32 }}", ei, ty, v)
                        }
                        // PLAN-862: the unit value carries only its type.
                        EKind::Unit => format!("expr e{} unit {{ type: {} }}", ei, ty),
                        EKind::Read(l) => {
                            format!("expr e{} read_local {{ type: {}; local: l{} }}", ei, ty, l)
                        }
                        EKind::Bin(op, l, r) => {
                            // PLAN-770: `+` planned with a text result type
                            // is concatenation, not add_i32
                            if e.vt.is_text() {
                                format!(
                                    "expr e{} text_concat {{ type: {}; lhs: e{}; rhs: e{} }}",
                                    ei, ty, l, r
                                )
                            } else {
                                let (ops, overflow) = bin_op_atom(*op);
                                format!(
                                    "expr e{} binary {{ type: {}; op: {}{}; lhs: e{}; rhs: e{} }}",
                                    ei, ty, ops, overflow, l, r
                                )
                            }
                        }
                        EKind::Logical(op, l, r) => {
                            format!(
                                "expr e{} logical {{ type: {}; op: {}; lhs: e{}; rhs: e{} }}",
                                ei,
                                ty,
                                op.as_str(),
                                l,
                                r
                            )
                        }
                        EKind::Unary(op, o) => {
                            format!(
                                "expr e{} unary {{ type: {}; op: {}; operand: e{} }}",
                                ei,
                                ty,
                                op.as_str(),
                                o
                            )
                        }
                        EKind::StrLit(v) => {
                            format!(
                                "expr e{} str_lit {{ type: {}; text: \"{}\" }}",
                                ei,
                                ty,
                                Self::esc(v)
                            )
                        }
                        EKind::CharLit(v) => {
                            format!("expr e{} char_lit {{ type: char; scalar: {}u32 }}", ei, v)
                        }
                        EKind::Text {
                            method,
                            recv,
                            args,
                            to,
                        } => {
                            use crate::source::resolve::BuiltinMethod as BM;
                            let recv_s = format!("e{}", recv);
                            match method {
                                BM::Len => format!(
                                    "expr e{} text_len {{ type: {}; operand: {} }}",
                                    ei, ty, recv_s
                                ),
                                BM::LenBytes => format!(
                                    "expr e{} text_len_bytes {{ type: {}; operand: {} }}",
                                    ei, ty, recv_s
                                ),
                                BM::CharAt => format!(
                                    "expr e{} text_char_at {{ type: {}; operand: {}; index: e{} }}",
                                    ei, ty, recv_s, args[0]
                                ),
                                BM::Slice => format!(
                                    "expr e{} text_slice {{ type: {}; operand: {}; start: e{}; end: e{} }}",
                                    ei, ty, recv_s, args[0], args[1]
                                ),
                                BM::Clone => format!(
                                    "expr e{} text_clone {{ type: {}; operand: {} }}",
                                    ei, ty, recv_s
                                ),
                                BM::ClosureClone => format!(
                                    "expr e{} closure_clone {{ type: {}; operand: {} }}",
                                    ei, ty, recv_s
                                ),
                                BM::To => {
                                    if matches!(to, Some(super::resolve::VT::StrOwned)) {
                                        format!(
                                            "expr e{} text_to_owned {{ type: {}; operand: {} }}",
                                            ei, ty, recv_s
                                        )
                                    } else {
                                        format!(
                                            "expr e{} text_to_borrowed {{ type: {}; operand: {} }}",
                                            ei, ty, recv_s
                                        )
                                    }
                                }
                                BM::ToCodepoint => format!(
                                    "expr e{} char_to_codepoint {{ type: {}; operand: {} }}",
                                    ei, ty, recv_s
                                ),
                                BM::FromCodepoint => format!(
                                    "expr e{} char_from_codepoint {{ type: char; operand: e{} }}",
                                    ei, args[0]
                                ),
                            }
                        }
                        EKind::Call {
                            callee_fn,
                            callee_intrinsic,
                            eval,
                            bindings,
                            moved,
                        } => {
                            let callee = match (callee_fn, callee_intrinsic) {
                                (Some(i), _) => {
                                    let cf = &self.program.fns[*i];
                                    if cf.receiver == crate::source::ast::Receiver::Free {
                                        format!("d_{}", cf.name)
                                    } else {
                                        format!("d_f{}_{}", i, cf.name)
                                    }
                                }
                                (None, Some(s)) => intrinsic_def_id(s).to_string(),
                                _ => "d_unknown".to_string(),
                            };
                            let eval_s: Vec<String> =
                                eval.iter().map(|x| format!("e{}", x)).collect();
                            let bind_s: Vec<String> = bindings
                                .iter()
                                .zip(moved.iter())
                                .map(|((p, a), m)| {
                                    if *m {
                                        format!("{{ param: {}u32, arg: {}u32, moved: true }}", p, a)
                                    } else {
                                        format!("{{ param: {}u32, arg: {}u32 }}", p, a)
                                    }
                                })
                                .collect();
                            format!(
                                "expr e{} call {{ type: {}; callee: {}; eval_args: [{}]; bindings: [{}] }}",
                                ei,
                                ty,
                                callee,
                                eval_s.join(", "),
                                bind_s.join(", ")
                            )
                        }
                        EKind::CallClosure { callee, eval } => {
                            let eval_s: Vec<String> =
                                eval.iter().map(|x| format!("e{}", x)).collect();
                            format!(
                                "expr e{} call_closure {{ type: {}; base: e{}; eval_args: [{}] }}",
                                ei,
                                ty,
                                callee,
                                eval_s.join(", ")
                            )
                        }
                        EKind::CreateClosure {
                            callee_fn,
                            captures,
                        } => {
                            let callee = self
                                .program
                                .fns
                                .get(*callee_fn)
                                .map(|function| format!("d_{}", function.name))
                                .unwrap_or_else(|| "d_unknown".to_string());
                            let capture_text = captures
                                .iter()
                                .map(|(local, mode)| {
                                    let mode = match mode {
                                        crate::hir::CaptureMode::Move => "move",
                                        crate::hir::CaptureMode::Borrow => "borrow",
                                    };
                                    format!("{{ local: l{}, mode: {} }}", local, mode)
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!(
                                "expr e{} create_closure {{ type: {}; callee: {}; captures: [{}] }}",
                                ei, ty, callee, capture_text
                            )
                        }
                        EKind::RecordCtor {
                            rec,
                            eval,
                            bindings,
                        } => {
                            let rec_name = &self.program.records[*rec].name;
                            let eval_s: Vec<String> =
                                eval.iter().map(|x| format!("e{}", x)).collect();
                            let bind_s: Vec<String> = bindings
                                .iter()
                                .map(|(f, a)| format!("{{ field: {}u32, arg: {}u32 }}", f, a))
                                .collect();
                            format!(
                                "expr e{} record_ctor {{ type: {}; record: t_rec_{}; field_args: [{}]; ctor_bindings: [{}] }}",
                                ei,
                                ty,
                                rec_name,
                                eval_s.join(", "),
                                bind_s.join(", ")
                            )
                        }
                        EKind::FieldGet { base, path } => {
                            let segs: Vec<String> =
                                path.iter().map(|f| format!("{}u32", f)).collect();
                            format!(
                                "expr e{} field_get {{ type: {}; base: e{}; path: [{}] }}",
                                ei,
                                ty,
                                base,
                                segs.join(", ")
                            )
                        }
                        EKind::BorrowView { base } => {
                            format!(
                                "expr e{} borrow_view {{ type: {}; base: e{} }}",
                                ei, ty, base
                            )
                        }
                        EKind::BorrowMut { local } => {
                            format!(
                                "expr e{} borrow_mut {{ type: {}; local: l{} }}",
                                ei, ty, local
                            )
                        }
                        EKind::RecordClone { rec, base } => {
                            // type ids are nominal (`t_rec_<name>`), not the
                            // record index
                            let rec_name = &self.program.records[*rec].name;
                            format!(
                                "expr e{} record_clone {{ type: {}; record: t_rec_{}; base: e{} }}",
                                ei, ty, rec_name, base
                            )
                        }
                        EKind::Move { inner } => {
                            format!("expr e{} move {{ type: {}; operand: e{} }}", ei, ty, inner)
                        }
                        // -- PLAN-864 collections surface --
                        EKind::ListNew { list } => {
                            let lname = self.ty_name(VT::List(*list));
                            format!(
                                "expr e{} list_new {{ type: {}; record: {} }}",
                                ei, ty, lname
                            )
                        }
                        EKind::ListLen { list } => {
                            format!("expr e{} list_len {{ type: {}; list: e{} }}", ei, ty, list)
                        }
                        EKind::ListClear { list } => {
                            format!(
                                "expr e{} list_clear {{ type: {}; list: e{} }}",
                                ei, ty, list
                            )
                        }
                        EKind::ListPush { list, value } => {
                            format!(
                                "expr e{} list_push {{ type: {}; list: e{}; value: e{} }}",
                                ei, ty, list, value
                            )
                        }
                        EKind::ListInsert { list, index, value }
                        | EKind::ListSet { list, index, value } => {
                            let word = match &e.kind {
                                EKind::ListInsert { .. } => "list_insert",
                                _ => "list_set",
                            };
                            format!(
                                "expr e{} {} {{ type: {}; list: e{}; index: e{}; value: e{} }}",
                                ei, word, ty, list, index, value
                            )
                        }
                        EKind::ListGet { list, index } => {
                            format!(
                                "expr e{} list_get {{ type: {}; list: e{}; index: e{} }}",
                                ei, ty, list, index
                            )
                        }
                        EKind::ListRemove { list, index } => {
                            format!(
                                "expr e{} list_remove {{ type: {}; list: e{}; index: e{} }}",
                                ei, ty, list, index
                            )
                        }
                        EKind::VariantCtor {
                            variant,
                            disc,
                            payload,
                        } => {
                            let vname = self.ty_name(VT::Variant(*variant));
                            match payload {
                                Some(pl) => format!(
                                    "expr e{} variant_ctor {{ type: {}; record: {}; disc: {}u32; payload: e{} }}",
                                    ei, ty, vname, disc, pl
                                ),
                                None => format!(
                                    "expr e{} variant_ctor {{ type: {}; record: {}; disc: {}u32 }}",
                                    ei, ty, vname, disc
                                ),
                            }
                        }
                        EKind::VariantGuardTest { root, disc, .. } => {
                            format!(
                                "expr e{} variant_guard_test {{ type: {}; operand: e{}; disc: {}u32 }}",
                                ei, ty, root, disc
                            )
                        }
                        EKind::VariantProject { root, disc, .. } => {
                            format!(
                                "expr e{} variant_project {{ type: {}; operand: e{}; disc: {}u32 }}",
                                ei, ty, root, disc
                            )
                        }
                    };
                    frag!(&line, e.span);
                    bs.exprs.push(e.span);
                    nl!(20);
                }
                frag!("}", f.name_span);
                nl!(16);
            }

            // blocks
            frag!("blocks", f.name_span);
            t.push(' ');
            frag!("{", f.name_span);
            nl!(20);
            for (bi, b) in plan.blocks.iter().enumerate() {
                frag!(&format!("block k{} {{", bi), b.span);
                nl!(24);
                for s in &b.stmts {
                    let line = match &s.kind {
                        SKind::Let { local, value } => {
                            format!("let(local: l{}, value: e{}) {{}}", local, value)
                        }
                        SKind::Assign { place, value } => {
                            format!("assign(place: p{}, value: e{}) {{}}", place, value)
                        }
                        SKind::Return { value } => format!("return(value: e{}) {{}}", value),
                        SKind::If { cond, then, els } => match els {
                            Some(e) => {
                                format!("if(cond: e{}, then: k{}, else: k{}) {{}}", cond, then, e)
                            }
                            None => format!("if(cond: e{}, then: k{}) {{}}", cond, then),
                        },
                        SKind::Loop { target, body } => {
                            format!("loop(id: q{}, body: k{}) {{}}", target, body)
                        }
                        SKind::Break { target } => format!("break(target: q{}) {{}}", target),
                        SKind::Continue { target } => {
                            format!("continue(target: q{}) {{}}", target)
                        }
                        SKind::Eval { value } => {
                            format!("eval(value: e{}) {{}}", value)
                        }
                    };
                    frag!(&line, s.span);
                    nl!(24);
                }
                frag!("}", b.span);
                bs.blocks.push(b.span);
                nl!(20);
            }
            frag!("}", f.name_span);
            nl!(16);

            frag!("}", f.name_span);
            nl!(12);
            spans.bodies.push(bs);
        }
        frag!("}", program_span);
        nl!(8);
        frag!("}", program_span);
        nl!(4);
        frag!("}", program_span);
        t.push('\n');

        self.frags = frags;
        self.spans = Some(spans);
        t
    }

    /// Re-anchor every binder-produced span to its intended source span
    /// (entry order), with the token fragment map as fallback, then assert
    /// the §5.3 post-conditions (in-bounds, UTF-8 boundaries, no generated
    /// positions left).
    fn reanchor_spans(&self, bundle: &mut crate::hir::Bundle, src: &str) -> Result<(), Diagnostic> {
        let Some(spans) = &self.spans else {
            return Err(internal_diag("render output missing".to_string()));
        };
        let frag_project = |s: Span| -> Option<Span> {
            let mut acc: Option<Span> = None;
            for (g, o) in &self.frags {
                if g.end <= s.start {
                    continue;
                }
                if g.start >= s.end {
                    break;
                }
                acc = Some(acc.map_or(*o, |p: Span| p.join(*o)));
            }
            acc
        };
        let check = |s: Span| -> Result<Span, Diagnostic> {
            if s.start > src.len() || s.end > src.len() {
                return Err(internal_diag(format!(
                    "span {:?} escapes the source after projection",
                    s
                )));
            }
            if !src.is_char_boundary(s.start) || !src.is_char_boundary(s.end) {
                return Err(internal_diag(format!(
                    "span {:?} is not on UTF-8 boundaries after projection",
                    s
                )));
            }
            Ok(s)
        };

        let mismatch = |what: &str, expect: usize, got: usize| -> Option<Diagnostic> {
            if expect != got {
                Some(internal_diag(format!(
                    "generated table size mismatch for {}: {} emitted, {} bound",
                    what, expect, got
                )))
            } else {
                None
            }
        };

        // Single module per this profile's emission.
        let m = match bundle.modules.get_mut(0) {
            Some(m) => m,
            None => {
                return Err(internal_diag("binder produced no module".to_string()));
            }
        };
        if let Some(d) = mismatch("types", spans.types.len(), m.types.len())
            .or_else(|| mismatch("defs", spans.defs.len(), m.defs.len()))
            .or_else(|| mismatch("bodies", spans.bodies.len(), m.bodies.len()))
        {
            return Err(d);
        }

        m.span = check(frag_project(m.span).unwrap_or(m.span))?;
        for (i, ty) in m.types.iter_mut().enumerate() {
            ty.span = check(spans.types[i])?;
        }
        for (i, d) in m.defs.iter_mut().enumerate() {
            d.span = check(spans.defs[i])?;
        }
        for (j, (b, eb)) in m.bodies.iter_mut().zip(spans.bodies.iter()).enumerate() {
            let _ = j;
            if let Some(d) = mismatch("locals", eb.locals.len(), b.locals.len())
                .or_else(|| mismatch("places", eb.places.len(), b.places.len()))
                .or_else(|| mismatch("exprs", eb.exprs.len(), b.exprs.len()))
                .or_else(|| mismatch("blocks", eb.blocks.len(), b.blocks.len()))
            {
                return Err(d);
            }
            b.span = check(eb.span)?;
            for (i, l) in b.locals.iter_mut().enumerate() {
                l.span = check(eb.locals[i])?;
            }
            for (i, p) in b.places.iter_mut().enumerate() {
                p.span = check(eb.places[i])?;
            }
            for (i, e) in b.exprs.iter_mut().enumerate() {
                e.span = check(eb.exprs[i])?;
            }
            for (i, bl) in b.blocks.iter_mut().enumerate() {
                bl.span = check(eb.blocks[i])?;
            }
        }
        Ok(())
    }
}

fn intrinsic_type_id(sym: &str) -> String {
    format!("t_i_{}", intrinsic_id_suffix(sym))
}

/// Atom-text encoding of a surface binary operator: the descriptor word
/// plus the `overflow: trap` attribute for the arithmetic whose overflow
/// is policy-governed (div/rem traps are inherent; comparisons cannot
/// overflow).
fn bin_op_atom(op: BinOp) -> (&'static str, &'static str) {
    match op {
        BinOp::Add => ("add_i32", "; overflow: trap"),
        BinOp::Sub => ("sub_i32", "; overflow: trap"),
        BinOp::Mul => ("mul_i32", "; overflow: trap"),
        BinOp::Div => ("div_i32", ""),
        BinOp::Rem => ("rem_i32", ""),
        BinOp::Lt => ("lt_i32", ""),
        BinOp::Le => ("le_i32", ""),
        BinOp::Gt => ("gt_i32", ""),
        BinOp::Ge => ("ge_i32", ""),
        BinOp::Eq => ("eq", ""),
        BinOp::Ne => ("ne", ""),
        BinOp::And | BinOp::Or => unreachable!("logical ops lower to EKind::Logical"),
    }
}

/// Capability every text-profile module that lowers runtime calls must
/// declare and the caller must explicitly provide (PLAN-770 §2). The
/// frozen string lives with the verifier front door (text_resources.rs)
/// so verify and the source adapter cannot drift apart (r2 QA-02).
pub use crate::text_resources::RUNTIME_TEXT_CAPABILITY;

fn intrinsic_def_id(sym: &str) -> String {
    format!("i_{}", intrinsic_id_suffix(sym))
}

/// Compile the explicit PLAN-863 console source domain. Grants are checked at AV admission.
pub fn compile_host_console_source(
    src: &str,
    module_path: &str,
    capabilities: &[String],
) -> Result<CompiledSource, Vec<Diagnostic>> {
    let parts = compile_parts_with(src, module_path, capabilities, SourceProfile::HostConsole)?;
    let checked = crate::verify::verify_host_console(parts.bundle)?;
    Ok(CompiledSource {
        profile: super::HOST_CONSOLE_SOURCE_PROFILE,
        input_hash: parts.input_hash,
        atom_text: parts.atom_text,
        checked,
    })
}

fn intrinsic_id_suffix(sym: &str) -> String {
    if sym.starts_with("auto.console.") {
        sym.replace('.', "_")
    } else {
        sym.rsplit('.').next().unwrap_or("x").to_string()
    }
}
