//! The signature catalog of one module (PLAN-934 T-03).
//!
//! A signature page is a pure DECLARATION fact: it never reads, walks or
//! solves a body. The catalog is therefore the cheap, stable index the
//! query session and the consumers use for
//!
//! * the function candidate set (the negative-dependency input),
//! * per-name signature versions (the positive dependency of every body
//!   that calls it),
//! * logical-id ↔ name mapping (the provenance binding of a page, never a
//!   source hash or span).
//!
//! Building a catalog is O(items) over declarations only — a body edit
//! cannot change any version in it.

use crate::facts::FnSigFacts;
use crate::ident::{owner_logical, signature_version, sq_hash};
use crate::produce::{produce_fn_sig, ProduceError};
use ace_core::source::ast::{Item, Program};
use ace_semantics_contract::{ContentVersion, LogicalId};
use std::collections::BTreeMap;

/// Immutable signature index of one module.
#[derive(Clone, Debug)]
pub struct SignatureCatalog {
    package: String,
    module: String,
    /// Name → signature page, in name order (deterministic).
    by_name: BTreeMap<String, FnSigFacts>,
    /// Logical id → name (a page's provenance resolves back to a name).
    by_logical: BTreeMap<LogicalId, String>,
    candidates: Vec<String>,
    candidates_fp: String,
}

impl SignatureCatalog {
    /// Replace only declaration metadata; no AST/body is projected or walked.
    pub fn replace_foreign(
        &self,
        previous: &ace_foreign_contract::manual::AdmittedForeign,
        next: &ace_foreign_contract::manual::AdmittedForeign,
    ) -> Result<Self, ProduceError> {
        let names: std::collections::BTreeSet<_> = previous
            .source_declarations()
            .into_iter()
            .map(|d| d.name)
            .collect();
        let foreign = Self::build_with_foreign(
            &Program { items: Vec::new() },
            &self.package,
            &self.module,
            Some(next),
        )?;
        let entries = self
            .entries()
            .filter(|f| !names.contains(&f.name))
            .cloned()
            .chain(foreign.entries().cloned())
            .collect();
        Self::from_entries(&self.package, &self.module, entries)
    }
    pub fn entries(&self) -> impl Iterator<Item = &FnSigFacts> {
        self.by_name.values()
    }

    /// Reconstitute declaration metadata only. The source-bound required page
    /// owns the stored version; no body, AST projection or solver runs here.
    pub fn from_entries(
        package: &str,
        module: &str,
        entries: Vec<FnSigFacts>,
    ) -> Result<Self, ProduceError> {
        let mut by_name = BTreeMap::new();
        let mut by_logical = BTreeMap::new();
        for facts in entries {
            if facts.check_envelope().is_err()
                || facts.logical != owner_logical(package, module, &facts.name)
                || by_name.contains_key(&facts.name)
                || facts.version.as_str().len() != 64
            {
                return Err(ProduceError::Profile {
                    what: "invalid restored signature catalog",
                });
            }
            by_logical.insert(facts.logical.clone(), facts.name.clone());
            by_name.insert(facts.name.clone(), facts);
        }
        let candidates: Vec<String> = by_name.keys().cloned().collect();
        let parts: Vec<&[u8]> = candidates.iter().map(|n| n.as_bytes()).collect();
        let candidates_fp = sq_hash("ac934/candidates/fn/v1", &parts);
        Ok(Self {
            package: package.into(),
            module: module.into(),
            by_name,
            by_logical,
            candidates,
            candidates_fp,
        })
    }
    /// Apply only the explicit declaration/effect inputs read by this owner.
    /// The empty map leaves every original SQ-A version byte-for-byte intact.
    pub fn with_declaration_contexts(
        mut self,
        contexts: &BTreeMap<String, String>,
    ) -> Result<Self, ProduceError> {
        for (name, context) in contexts {
            let facts = self
                .by_name
                .get_mut(name)
                .ok_or_else(|| ProduceError::UnknownFn { name: name.clone() })?;
            if context.len() != 64 || !context.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(ProduceError::Profile {
                    what: "invalid declaration context digest",
                });
            }
            facts.version = ContentVersion(sq_hash(
                "ac944/declaration-context/v1",
                &[facts.version.as_str().as_bytes(), context.as_bytes()],
            ));
        }
        Ok(self)
    }
    /// Build the catalog from a parsed module. Declarations outside the
    /// finite signature surface are a controlled rejection.
    pub fn build(program: &Program, package: &str, module: &str) -> Result<Self, ProduceError> {
        Self::build_with_foreign(program, package, module, None)
    }

    pub fn build_with_foreign(
        program: &Program,
        package: &str,
        module: &str,
        foreign: Option<&ace_foreign_contract::manual::AdmittedForeign>,
    ) -> Result<Self, ProduceError> {
        let mut by_name: BTreeMap<String, FnSigFacts> = BTreeMap::new();
        let mut by_logical: BTreeMap<LogicalId, String> = BTreeMap::new();
        for item in &program.items {
            match item {
                Item::Fn(f) => {
                    let facts = produce_fn_sig(program, package, module, &f.name.name)?;
                    by_logical.insert(facts.logical.clone(), facts.name.clone());
                    by_name.insert(facts.name.clone(), facts);
                }
                Item::Type(_) => {
                    return Err(ProduceError::Profile {
                        what: "record declaration",
                    })
                }
                Item::Enum(_) => {
                    return Err(ProduceError::Profile {
                        what: "enum declaration",
                    })
                }
            }
        }
        if let Some(environment) = foreign {
            for declaration in environment.source_declarations() {
                if by_name.contains_key(&declaration.name) {
                    return Err(ProduceError::Profile {
                        what: "foreign/source declaration conflict",
                    });
                }
                let lane = |k| {
                    if k == ace_foreign_contract::LaneKind::Bool {
                        crate::facts::PortableTy::Bool
                    } else {
                        crate::facts::PortableTy::I32
                    }
                };
                let facts = FnSigFacts {
                    name: declaration.name.clone(),
                    logical: owner_logical(package, module, &declaration.name),
                    version: ContentVersion::of_canonical(
                        environment
                            .declaration_digest(&declaration.name)
                            .expect("admitted declaration digest")
                            .as_bytes(),
                    ),
                    params: declaration
                        .params()
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(i, k)| (format!("arg{i}"), lane(k)))
                        .collect(),
                    ret: lane(declaration.result()),
                    profile: crate::ident::SQ_PROFILE,
                    schema: crate::ident::SQ_SCHEMA,
                    producer: crate::ident::SQ_PRODUCER,
                };
                by_logical.insert(facts.logical.clone(), facts.name.clone());
                by_name.insert(facts.name.clone(), facts);
            }
        }
        let candidates: Vec<String> = by_name.keys().cloned().collect();
        let candidates_fp = {
            let parts: Vec<&[u8]> = candidates.iter().map(|n| n.as_bytes()).collect();
            sq_hash("ac934/candidates/fn/v1", &parts)
        };
        Ok(SignatureCatalog {
            package: package.to_string(),
            module: module.to_string(),
            by_name,
            by_logical,
            candidates,
            candidates_fp,
        })
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn module(&self) -> &str {
        &self.module
    }

    pub fn get(&self, name: &str) -> Option<&FnSigFacts> {
        self.by_name.get(name)
    }

    pub fn version(&self, name: &str) -> Option<&ContentVersion> {
        self.by_name.get(name).map(|f| &f.version)
    }

    /// Recompute the version a name would have RIGHT NOW (a declaration
    /// read, no body work) — the dependency check of a cached page.
    pub fn current_version(&self, name: &str) -> Option<ContentVersion> {
        self.by_name.get(name).map(|f| f.version.clone())
    }

    pub fn by_logical(&self, id: &LogicalId) -> Option<&FnSigFacts> {
        self.by_logical.get(id).and_then(|n| self.by_name.get(n))
    }

    /// Sorted function names (the candidate set of negative dependencies).
    pub fn candidates(&self) -> &[String] {
        &self.candidates
    }

    /// Fingerprint of the candidate set (negative-dependency input).
    pub fn candidates_fp(&self) -> &str {
        &self.candidates_fp
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// Logical id of a name in this module (independent re-derivation; a
    /// page whose id does not match the catalog is foreign).
    pub fn logical_of(&self, name: &str) -> Option<LogicalId> {
        if self.by_name.contains_key(name) {
            Some(owner_logical(&self.package, &self.module, name))
        } else {
            None
        }
    }

    /// Declaration version of a name, from the AST (used by the session to
    /// build dependency fingerprints without a body walk).
    pub fn declared_version(name: &str, f: &ace_core::source::ast::FnItem) -> ContentVersion {
        debug_assert_eq!(name, f.name.name);
        signature_version(f)
    }
}
