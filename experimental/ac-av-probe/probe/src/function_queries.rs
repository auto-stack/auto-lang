//! Function query layer for the resident session (PLAN-828 T-03).
//!
//! [`FnEnv`] is the session's interface environment: every function's
//! signature facts plus a content revision of the whole declaration
//! context. Body edits consume it through queries; a body credential
//! ([`CheckedFunction`]) is only reusable while the context revision it was
//! verified under still matches — a credential minted against a different
//! context is rejected by [`FnEnv::credential_current`], never silently
//! re-stamped. Scope is one owner (module): same-name functions of another
//! owner are simply not present, mirroring the 796 semantic-ref owner axis
//! at the session layer.

use auto_ac_prototype::hir::{self, HirDomain};
use auto_ac_prototype::verify::{declaration_context_fingerprint, CheckedFunction};
use std::collections::BTreeMap;

/// One function's interface facts (what a caller may depend on).
#[derive(Clone, Debug)]
pub struct FnInterface {
    /// Def id text (bundle identity, e.g. `d_helper`).
    pub def_id: String,
    /// Source-level name (e.g. `helper`).
    pub source_name: String,
    /// Canonical signature content (`TypeKind::Function` debug form —
    /// changes whenever the signature's types change).
    pub sig: String,
}

impl FnInterface {
    /// Whether a call site may keep its binding: interface equality.
    pub fn same_interface(&self, other: &FnInterface) -> bool {
        self.def_id == other.def_id && self.sig == other.sig
    }
}

/// The owner-scoped interface environment of one resident module.
#[derive(Clone, Debug)]
pub struct FnEnv {
    /// Module identity (HIR module id text) — the owner axis of every
    /// query below.
    pub owner: String,
    /// Content revision over all def ids + signatures; two envs with equal
    /// revisions provide identical cross-function facts.
    revision: String,
    /// The ac-core declaration-context fingerprint derived from THIS
    /// environment's own bundle (never a caller-supplied string). Body
    /// credentials are current only while this value equals the
    /// credential's bound revision.
    context_revision: String,
    /// The HIR domain of THIS environment's bundle — a credential minted
    /// in one domain is never current in another (F-828-R1-06/R2: a
    /// Scalar credential must not flow into a Core environment even when
    /// the module declarations read identically).
    domain: HirDomain,
    fns: Vec<FnInterface>,
    by_name: BTreeMap<String, usize>,
    /// Intrinsic declaration table of THIS environment's bundle:
    /// symbol → content-canonical signature shape (the binding facts a
    /// body-read credential must still find intact).
    intrinsics: BTreeMap<String, String>,
}

impl FnEnv {
    /// Build from a single-module bound bundle (the warm-up or a candidate
    /// synth bundle). Function defs only; scalar/core domains only.
    pub fn from_bundle(bundle: &hir::Bundle) -> Result<FnEnv, String> {
        if !matches!(bundle.domain, HirDomain::Scalar | HirDomain::Core) {
            return Err(format!(
                "function queries serve the scalar/core domains, not `{}`",
                bundle.domain.as_str()
            ));
        }
        let m = bundle
            .modules
            .first()
            .ok_or_else(|| "bundle has no module".to_string())?;
        let owner = m.id_text.clone();
        let mut fns = Vec::new();
        for d in &m.defs {
            let hir::DefKind::Function { name, .. } = &d.kind else {
                continue;
            };
            // `DefKind::Function::name` is the emitter's internal symbol
            // (`sf_<name>` for free functions); the query layer speaks
            // SOURCE names. Methods (`d_f<i>_<name>`) are outside the
            // scalar session's scope and stay unstripped (they never match
            // a scalar source name).
            let source_name = name
                .strip_prefix("sf_")
                .map(|n| n.to_string())
                .unwrap_or_else(|| name.clone());
            fns.push(FnInterface {
                def_id: d.id_text.clone(),
                source_name,
                sig: sig_debug_of(m, d),
            });
        }
        let mut intrinsics = BTreeMap::new();
        for d in &m.defs {
            if let hir::DefKind::Intrinsic { symbol, ty, .. } = &d.kind {
                let shape = m
                    .types
                    .get(ty.0 as usize)
                    .map(|t| match &t.kind {
                        hir::TypeKind::Function(sig) => {
                            let params: Vec<String> = sig
                                .params
                                .iter()
                                .map(|p| hir::TypeUse::shape(p, m))
                                .collect();
                            format!(
                                "fn({})->{}",
                                params.join(","),
                                hir::TypeUse::shape(&sig.result, m)
                            )
                        }
                        other => format!("{other:?}"),
                    })
                    .unwrap_or_else(|| "?".to_string());
                intrinsics.insert(symbol.clone(), shape);
            }
        }
        let mut by_name = BTreeMap::new();
        for (i, f) in fns.iter().enumerate() {
            if by_name.insert(f.source_name.clone(), i).is_some() {
                return Err(format!("duplicate function name `{}`", f.source_name));
            }
        }
        let mut revision = String::new();
        mix_revision(&mut revision, &owner);
        for f in &fns {
            mix_revision(&mut revision, &f.def_id);
            mix_revision(&mut revision, &f.sig);
        }
        Ok(FnEnv {
            context_revision: declaration_context_fingerprint(m),
            domain: bundle.domain,
            owner,
            revision,
            fns,
            by_name,
            intrinsics,
        })
    }

    /// THIS environment's declaration-context fingerprint (self-derived
    /// from the bundle the env was built on).
    pub fn context_revision(&self) -> &str {
        &self.context_revision
    }

    /// THIS environment's HIR domain.
    pub fn domain(&self) -> HirDomain {
        self.domain
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn interfaces(&self) -> &[FnInterface] {
        &self.fns
    }

    /// Interface facts of one function by source name, scoped to this
    /// owner's environment. `None` covers both unknown names and
    /// same-name-other-owner situations — the session never conflates
    /// owners.
    pub fn interface_of(&self, source_name: &str) -> Option<&FnInterface> {
        self.by_name.get(source_name).map(|&i| &self.fns[i])
    }

    pub fn def_id_of(&self, source_name: &str) -> Option<&str> {
        self.interface_of(source_name).map(|f| f.def_id.as_str())
    }

    /// Whether a body credential may still be consumed under THIS
    /// environment (F-828-R1-06): the def must exist here under the same
    /// def id, the credential's OWNER must be this environment's owner,
    /// and the credential's context revision must equal the fingerprint
    /// THIS environment derived from its own bundle. No caller-supplied
    /// string participates — a changed declaration context invalidates
    /// every outstanding credential (re-verify, never re-stamp).
    pub fn credential_current(&self, credential: &CheckedFunction) -> bool {
        self.fns
            .iter()
            .any(|f| f.def_id == credential.def_id_text())
            && credential.owner() == self.owner
            && credential.domain() == self.domain
            && credential.context_revision() == self.context_revision
            // Body-READ intrinsic facts: every intrinsic this credential's
            // body actually references must still be declared HERE with
            // the identical binding shape (F-828-R1-06/R2a).
            && credential
                .referenced_intrinsics()
                .iter()
                .all(|(sym, shape)| self.intrinsics.get(sym) == Some(shape))
    }

    /// Fast-path gate for one edit: the changed function's interface in the
    /// candidate bundle must equal the resident environment's. Any drift
    /// (signature change, rename, def-set change) is a non-fast-path edit.
    pub fn interface_unchanged(&self, candidate: &FnEnv, source_name: &str) -> bool {
        match (
            self.interface_of(source_name),
            candidate.interface_of(source_name),
        ) {
            (Some(a), Some(b)) => a.same_interface(b) && self.revision == candidate.revision,
            _ => false,
        }
    }
}

/// Content-canonical signature rendering: type references resolve to their
/// scalar shape, never their table index (a stub program and a real-body
/// program of the same declarations must render identically).
fn sig_debug_of(m: &hir::Module, d: &hir::Def) -> String {
    fn shape(u: &hir::TypeUse, m: &hir::Module, depth: u8) -> String {
        if depth == 0 {
            return "!".to_string();
        }
        match u {
            hir::TypeUse::Builtin(b) => format!("{b:?}"),
            hir::TypeUse::Ref(tid) => match m.types.get(tid.0 as usize).map(|t| &t.kind) {
                Some(hir::TypeKind::Function(sig)) => {
                    let params: Vec<String> =
                        sig.params.iter().map(|p| shape(p, m, depth - 1)).collect();
                    format!(
                        "fn({})->{}",
                        params.join(","),
                        shape(&sig.result, m, depth - 1)
                    )
                }
                Some(hir::TypeKind::I32) => "i32".to_string(),
                Some(hir::TypeKind::Bool) => "bool".to_string(),
                Some(hir::TypeKind::Str) => "str".to_string(),
                Some(hir::TypeKind::StrOwned) => "Str".to_string(),
                Some(hir::TypeKind::Char) => "char".to_string(),
                Some(hir::TypeKind::Record(_)) => "rec".to_string(),
                None => "oob".to_string(),
            },
        }
    }
    let ty = d.kind.fn_type().map(|t| t.0).unwrap_or(u32::MAX);
    match m.types.get(ty as usize).map(|t| &t.kind) {
        Some(hir::TypeKind::Function(sig)) => {
            let params: Vec<String> = sig.params.iter().map(|p| shape(p, m, 8)).collect();
            format!("fn({})->{}", params.join(","), shape(&sig.result, m, 8))
        }
        other => format!("{other:?}"),
    }
}

fn mix_revision(out: &mut String, part: &str) {
    out.push_str(part);
    out.push('\u{1}');
}
