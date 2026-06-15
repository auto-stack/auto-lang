//! W0007 warning construction — Plan 310 Phase 1.
//!
//! Builds [`crate::error::Warning::EscapeFallback`] entries from escape-analysis
//! decisions. Phase 1 only *collects* these into the analyzer; nothing is
//! surfaced to the transpiler output yet (Phase 2 wires `RustTrans.warnings`).

use crate::ast::Name;
use crate::error::{span_from, Warning};
use crate::trans::escape::escape_map::OwnershipTier;
use miette::SourceSpan;

/// Build an EscapeFallback warning for a binding that fell back from a borrow
/// to a more expensive tier.
///
/// `name`           — the binding name (e.g. `x` in `let x = ...`).
/// `tier`           — the fallback tier assigned by the analyzer.
/// `reason`         — why the analyzer could not prove a borrow was safe.
/// `span`           — source location for the diagnostic label.
pub fn build_warning(name: &Name, tier: OwnershipTier, reason: &str, span: SourceSpan) -> Warning {
    Warning::EscapeFallback {
        name: name.to_string(),
        reason: reason.to_string(),
        tier_desc: tier.description().to_string(),
        span,
    }
}

/// Helper: synthesize a SourceSpan at a byte offset (used when real source
/// spans aren't readily available during analysis).
pub fn span_at(offset: usize, len: usize) -> SourceSpan {
    span_from(offset, len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Name;

    #[test]
    fn test_build_warning_rc() {
        let w = build_warning(
            &Name::from("buf"),
            OwnershipTier::RcRefCell,
            "captured by closure at line 5",
            span_at(10, 3),
        );
        match w {
            Warning::EscapeFallback { name, reason, tier_desc, .. } => {
                assert_eq!(name, "buf");
                assert_eq!(reason, "captured by closure at line 5");
                assert_eq!(tier_desc, "Rc<RefCell<T>>");
            }
            _ => panic!("expected EscapeFallback variant"),
        }
    }

    #[test]
    fn test_build_warning_clone() {
        let w = build_warning(
            &Name::from("n"),
            OwnershipTier::Clone,
            "returned from function",
            span_at(0, 1),
        );
        match w {
            Warning::EscapeFallback { tier_desc, .. } => {
                assert_eq!(tier_desc, "explicit clone");
            }
            _ => panic!("expected EscapeFallback variant"),
        }
    }
}
