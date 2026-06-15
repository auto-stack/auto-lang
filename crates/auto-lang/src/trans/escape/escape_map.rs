//! Escape analysis decision map — Plan 310 Phase 1.
//!
//! Maps each local binding (identified by name + scope id) to an
//! [`OwnershipTier`] decided by the escape analyzer. The RustTrans queries
//! this map at `Expr::View`/`Expr::Mut` sites to decide whether to emit a
//! borrow (`&x`) or fall back to clone / `Rc<RefCell<T>>`.
//!
//! See `docs/plans/310-auto-ownership-escape-analysis.md` §3 for the tier model.

use crate::ast::Name;
use std::collections::HashMap;
use std::fmt;

/// The five ownership strategies, ordered from cheapest (Tier 0) to most
/// expensive (Tier 4). The analyzer assigns every tracked binding one tier;
/// the transpiler then generates code accordingly.
///
/// **Conservative invariant**: a binding marked [`OwnershipTier::BorrowView`]
/// or [`OwnershipTier::BorrowMut`] MUST be sound as a Rust `&`/`&mut`. If the
/// analyzer cannot prove that, it picks a higher tier. False positives
/// (borrowable values treated as owned/Rc) are acceptable; false negatives
/// (emit `&` that rustc rejects) are bugs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwnershipTier {
    /// Tier 0 — value is `'static` or fully owned; no borrowing involved.
    /// Default for literals and move expressions.
    Owned,
    /// Tier 1 — immutable borrow `&x`. Analyzer proved the value does not
    /// escape its defining scope and no mutable borrow conflicts occur.
    BorrowView,
    /// Tier 1 — mutable borrow `&mut x`. Same soundness conditions as
    /// BorrowView, plus exclusive access.
    BorrowMut,
    /// Tier 2 — escape detected, but the type is `Copy` or small enough that
    /// a clone is cheaper than a smart pointer. Generates `x.clone()`.
    Clone,
    /// Tier 3 — escape detected, fall back to `Rc<RefCell<T>>` (single-thread).
    /// Generates `Rc::clone(&x)` at use sites.
    RcRefCell,
    /// Tier 4 — reserved for cross-thread escape (Phase 3): `Arc<Mutex<T>>`.
    /// Not assigned by the synchronous analyzer in Phase 1/2.
    ArcMutex,
}

impl OwnershipTier {
    /// Human-readable description for the W0007 warning help text.
    pub fn description(self) -> &'static str {
        match self {
            OwnershipTier::Owned => "owned value",
            OwnershipTier::BorrowView => "immutable borrow (&T)",
            OwnershipTier::BorrowMut => "mutable borrow (&mut T)",
            OwnershipTier::Clone => "explicit clone",
            OwnershipTier::RcRefCell => "Rc<RefCell<T>>",
            OwnershipTier::ArcMutex => "Arc<Mutex<T>>",
        }
    }

    /// True if this tier corresponds to a Rust reference (Tier 1).
    pub fn is_borrow(self) -> bool {
        matches!(self, OwnershipTier::BorrowView | OwnershipTier::BorrowMut)
    }

    /// True if this tier is a smart-pointer fallback (Tier 3 or 4).
    pub fn is_smart_pointer(self) -> bool {
        matches!(self, OwnershipTier::RcRefCell | OwnershipTier::ArcMutex)
    }
}

impl fmt::Display for OwnershipTier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

/// Key for the escape decision map.
///
/// A binding is identified by its (scope-depth, name). Two `x` in sibling
/// scopes are distinct bindings. The depth is the lexical nesting level
/// (0 = function body, 1 = first nested block, ...).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindingId {
    pub scope_depth: usize,
    pub name: Name,
}

/// The decision table produced by [`crate::trans::escape::analyzer::EscapeAnalyzer`].
///
/// Phase 1 contract: the map is populated and queryable, but the transpiler
/// does NOT yet consult it — output bytes must stay identical. Phase 2 wires
/// the map into `Expr::View`/`Expr::Mut` generation.
#[derive(Debug, Default)]
pub struct EscapeMap {
    decisions: HashMap<BindingId, OwnershipTier>,
    /// Reasons for non-borrow tiers, used in W0007 messages.
    reasons: HashMap<BindingId, String>,
}

impl EscapeMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the analyzer's decision for a binding. The first recorded
    /// decision for a given id wins only if "more restrictive" — but in
    /// practice the analyzer visits each binding once per scope, so a plain
    /// insert suffices. We keep the most restrictive (highest-tier) decision
    /// to stay conservative across control-flow branches.
    pub fn record(&mut self, id: BindingId, tier: OwnershipTier, reason: impl Into<String>) {
        match self.decisions.get(&id).copied() {
            Some(prev) => {
                // Keep the more restrictive (higher-ordinal) tier.
                if tier.ordinal() > prev.ordinal() {
                    self.decisions.insert(id.clone(), tier);
                    self.reasons.insert(id, reason.into());
                }
            }
            None => {
                self.decisions.insert(id.clone(), tier);
                self.reasons.insert(id, reason.into());
            }
        }
    }

    /// Query the decision for a binding visible at the given scope depth.
    /// Searches from the exact depth outward to shallower scopes (lexical
    /// scoping: a binding shadows same-named outer bindings).
    pub fn lookup(&self, scope_depth: usize, name: &Name) -> Option<OwnershipTier> {
        for depth in (0..=scope_depth).rev() {
            if let Some(tier) = self
                .decisions
                .get(&BindingId { scope_depth: depth, name: name.clone() })
            {
                return Some(*tier);
            }
        }
        None
    }

    /// Reason text for a binding (for W0007). Same shadowing as [`lookup`].
    pub fn reason_for(&self, scope_depth: usize, name: &Name) -> Option<&str> {
        for depth in (0..=scope_depth).rev() {
            if let Some(r) = self
                .reasons
                .get(&BindingId { scope_depth: depth, name: name.clone() })
            {
                return Some(r.as_str());
            }
        }
        None
    }

    /// Number of decisions recorded (for diagnostics / tests).
    pub fn len(&self) -> usize {
        self.decisions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.decisions.is_empty()
    }

    /// Iterator over all (BindingId, tier) pairs (for debugging / tests).
    pub fn iter(&self) -> impl Iterator<Item = (&BindingId, &OwnershipTier)> {
        self.decisions.iter()
    }
}

impl OwnershipTier {
    /// Ordering by restrictiveness: Owned < Borrow* < Clone < smart pointers.
    /// Higher = more runtime cost. Used by [`EscapeMap::record`] to keep the
    /// conservative (worst-case) decision across branches.
    fn ordinal(self) -> u8 {
        match self {
            OwnershipTier::Owned => 0,
            OwnershipTier::BorrowView => 1,
            OwnershipTier::BorrowMut => 2,
            OwnershipTier::Clone => 3,
            OwnershipTier::RcRefCell => 4,
            OwnershipTier::ArcMutex => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_lookup() {
        let mut m = EscapeMap::new();
        let id = BindingId { scope_depth: 0, name: "x".into() };
        m.record(id.clone(), OwnershipTier::BorrowView, "local use only");
        assert_eq!(m.lookup(0, &"x".into()), Some(OwnershipTier::BorrowView));
        assert_eq!(m.reason_for(0, &"x".into()), Some("local use only"));
    }

    #[test]
    fn test_shadowing_lookup() {
        let mut m = EscapeMap::new();
        m.record(
            BindingId { scope_depth: 0, name: "x".into() },
            OwnershipTier::Owned,
            "outer",
        );
        m.record(
            BindingId { scope_depth: 2, name: "x".into() },
            OwnershipTier::BorrowView,
            "inner",
        );
        // At depth 2, the inner binding shadows the outer.
        assert_eq!(m.lookup(2, &"x".into()), Some(OwnershipTier::BorrowView));
        // At depth 0, only the outer is visible.
        assert_eq!(m.lookup(0, &"x".into()), Some(OwnershipTier::Owned));
        // At depth 1 (between), the outer is the nearest enclosing.
        assert_eq!(m.lookup(1, &"x".into()), Some(OwnershipTier::Owned));
    }

    #[test]
    fn test_conservative_merge_keeps_higher_tier() {
        let mut m = EscapeMap::new();
        let id = BindingId { scope_depth: 0, name: "x".into() };
        m.record(id.clone(), OwnershipTier::BorrowView, "branch A: local");
        m.record(id.clone(), OwnershipTier::RcRefCell, "branch B: escapes");
        // The more restrictive tier wins (conservative across branches).
        assert_eq!(m.lookup(0, &"x".into()), Some(OwnershipTier::RcRefCell));
    }
}
