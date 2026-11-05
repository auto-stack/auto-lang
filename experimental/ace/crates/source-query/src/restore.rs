//! PLAN-934 T-07: the fact-side restore contract (929 owns source/CST/decl
//! restore; this layer owns the facts page and its admission).
//!
//! Restore is a READ-ONLY discipline: pages are decoded and deep-verified
//! against the restored source, then installed as the bind's type supply.
//! No solver runs, no semantic body is re-derived to "verify" a page, and a
//! missing or stale page is a REFUSAL (the cold path is a separate,
//! explicit lane — never a silent fallback).
//!
//! Verification order (all before any type is consumed):
//!   current source body walk → page body version → node ids/kinds/
//!   duplicates/coverage/diagnostics → envelope → bind.
//!
//! The first-edit legs use the same pages as the warm baseline: a restored
//! session seeded with every page re-solves ONLY the body that changed.

use crate::apply::{apply_fn_sig, AppliedFn, ApplyError, ApplyReport, BodyTypeSupply};
use crate::facts::{FactsError, InferBodyFacts};
use crate::ident::owner_logical;
use crate::keys::QueryKey;
use crate::produce::produce_fn_sig;
use crate::walk::{walk_body, WalkError};
use ace_core::atom_text::Diagnostic;
use ace_core::source::ast::{Block, Item, Program};
use ace_core::source::resolve::{resolve_with_supply, ResolvedProgram};
use ace_core::source::SourceProfile;
use std::collections::BTreeMap;

/// Why a restore was refused. Every variant keeps the old program usable.
#[derive(Clone, Debug)]
pub enum RestoreError {
    /// A required page is absent for this owner: the restore lane is
    /// refused (an explicit cold build is a different request).
    MissingRequiredPage {
        owner: String,
    },
    /// The page does not describe the current source body.
    Facts(FactsError),
    Walk(WalkError),
    /// Core's bind rejected the restored supply (bad/missing fact).
    Bind(Vec<Diagnostic>),
    /// The restored module carries declarations outside the finite surface.
    Profile {
        what: &'static str,
    },
    /// A signature fact could not be derived from the declaration.
    Signature(String),
}

impl std::fmt::Display for RestoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestoreError::MissingRequiredPage { owner } => {
                write!(f, "required fact page for `{owner}` is missing")
            }
            RestoreError::Facts(e) => write!(f, "{e}"),
            RestoreError::Walk(e) => write!(f, "{e}"),
            RestoreError::Bind(ds) => {
                let first = ds
                    .first()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .unwrap_or_else(|| "bind failed".into());
                write!(f, "{first} ({} diagnostic(s))", ds.len())
            }
            RestoreError::Profile { what } => {
                write!(f, "`{what}` is outside the finite restore surface")
            }
            RestoreError::Signature(m) => write!(f, "signature fact unavailable: {m}"),
        }
    }
}

/// Deep-verify ONE restored page against the restored source, without
/// solving. Returns the page's complete key on success.
pub fn restore_verify(
    program: &Program,
    package: &str,
    module: &str,
    name: &str,
    page: Option<&InferBodyFacts>,
) -> Result<QueryKey, RestoreError> {
    let page = page.ok_or_else(|| RestoreError::MissingRequiredPage {
        owner: name.to_string(),
    })?;
    if page.name != name {
        return Err(RestoreError::Facts(FactsError::BodyVersionMismatch {
            facts: format!("page of `{}`", page.name),
            body: name.to_string(),
        }));
    }
    let body = body_of(program, name).ok_or_else(|| RestoreError::MissingRequiredPage {
        owner: name.to_string(),
    })?;
    let walk = walk_body(body).map_err(RestoreError::Walk)?;
    // Shape/coverage/envelope first: a stale or partial page never reaches
    // a type consumer.
    page.validate_against(&walk).map_err(RestoreError::Facts)?;
    page.admit(&walk).map_err(RestoreError::Facts)?;
    let sig = produce_fn_sig(program, package, module, name)
        .map_err(|e| RestoreError::Signature(e.to_string()))?;
    Ok(QueryKey::infer_body(
        owner_logical(package, module, name),
        sig.version,
        walk.version,
        &[],
    ))
}

/// Bind a whole restored module from its pages ONLY: every free function
/// needs a page, nothing is solved, and the types come from the pages.
pub fn restore_bind(
    program: &Program,
    capabilities: &[String],
    package: &str,
    module: &str,
    pages: &BTreeMap<String, InferBodyFacts>,
    profile: SourceProfile,
) -> Result<(ResolvedProgram, Vec<AppliedFn>, BodyTypeSupply), RestoreError> {
    let mut names: Vec<String> = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(f) => names.push(f.name.name.clone()),
            Item::Type(_) => {
                return Err(RestoreError::Profile {
                    what: "record declaration",
                })
            }
            Item::Enum(_) => {
                return Err(RestoreError::Profile {
                    what: "enum declaration",
                })
            }
        }
    }
    if names.is_empty() {
        return Ok((
            ResolvedProgram {
                fns: Vec::new(),
                records: Vec::new(),
                lists: Vec::new(),
                variants: Vec::new(),
                function_types: Vec::new(),
            },
            Vec::new(),
            BodyTypeSupply::new(),
        ));
    }

    let mut supply = BodyTypeSupply::new();
    let mut sigs = Vec::new();
    let mut applied = Vec::new();
    for name in &names {
        let page = pages
            .get(name)
            .ok_or_else(|| RestoreError::MissingRequiredPage {
                owner: name.clone(),
            })?;
        restore_verify(program, package, module, name, Some(page))?;
        let body = body_of(program, name).expect("name came from the same program");
        supply.add_body(body, page).map_err(|e| match e {
            ApplyError::Walk(w) => RestoreError::Walk(w),
            ApplyError::Facts(f) => RestoreError::Facts(f),
            _ => RestoreError::Facts(FactsError::BodyVersionMismatch {
                facts: "page".into(),
                body: name.clone(),
            }),
        })?;
        let sig = produce_fn_sig(program, package, module, name)
            .map_err(|e| RestoreError::Signature(e.to_string()))?;
        sigs.push((name.clone(), sig));
        applied.push(AppliedFn {
            name: name.clone(),
            sig_hit: true,
            body_hit: true,
            body_version: page.body_version.as_str().to_string(),
            report: ApplyReport {
                exprs: page.nodes.len(),
                bindings: page.bindings.len(),
            },
        });
    }
    let mut resolved =
        resolve_with_supply(program, capabilities, profile, &supply).map_err(RestoreError::Bind)?;
    for (name, sig) in &sigs {
        if let Some(rfn) = resolved
            .fns
            .iter_mut()
            .find(|f| &f.name == name && f.owner_rec.is_none())
        {
            apply_fn_sig(rfn, sig).map_err(|e| RestoreError::Signature(e.to_string()))?;
        }
    }
    Ok((resolved, applied, supply))
}

fn body_of<'a>(program: &'a Program, name: &str) -> Option<&'a Block> {
    program.items.iter().find_map(|item| match item {
        Item::Fn(f) if f.name.name == name => Some(&f.body),
        _ => None,
    })
}
