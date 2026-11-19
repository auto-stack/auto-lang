//! PLAN-1107 T-02 (A2PY admission face): the A2PY-side view of the
//! **neutral** Python op declaration contract.
//!
//! The contract itself lives in `ace_foreign_contract::py_ops` — the neutral
//! home shared by every target. This module only exposes it to the A2PY
//! pipeline in the shape the compile path wants, and documents the boundary:
//!
//! * A2PY does not compile, load or link the AV provider (no PyO3, no
//!   `f0_text_provider.dll`, no AV session). It admits the declaration set so
//!   the ONE shared front door resolves `use.py` programs identically.
//! * The operations it will emit are native Python; the AV `p967.frame/1`
//!   encoding stays an AV implementation detail.
//! * `ace-abi` binds the neutral table to the live provider source by a
//!   consistency test, so the two can never drift silently.

pub use ace_foreign_contract::py_ops::{PyOpDecl, PY_OPS};

/// The declaration-only environment the A2PY compile lane passes to the shared
/// front door.
pub fn py_declaration_contract(
) -> Result<ace_foreign_contract::manual::AdmittedForeign, String> {
    ace_foreign_contract::py_ops::language_contract()
}

/// The frozen declaration contract as a manual manifest (inspection/tests).
pub fn py_declaration_manifest() -> ace_foreign_contract::manual::ManualManifest {
    ace_foreign_contract::py_ops::language_manifest()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a2py_admits_the_neutral_contract_without_a_provider() {
        let admitted = py_declaration_contract().expect("declaration-only admission");
        assert_eq!(admitted.source_declarations_ref().len(), PY_OPS.len());
        let mut names: Vec<String> = admitted
            .source_declarations_ref()
            .iter()
            .map(|d| d.name.clone())
            .collect();
        names.sort();
        let mut frozen: Vec<String> = ace_core::py_ops::PY_OP_NAMES
            .iter()
            .map(|s| s.to_string())
            .collect();
        frozen.sort();
        assert_eq!(names, frozen, "the A2PY admission is the frozen op set");
    }
}