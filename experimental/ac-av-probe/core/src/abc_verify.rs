//! ABC admission — the only constructor of the executable
//! [`LoadedModule`] (PLAN-798 §5.2).
//!
//! The wire decode ([`crate::abc::decode_abc`]) performs pure structure
//! checks; this stage re-stamps identity (owner/profile/versions come from
//! the ENVELOPE, never from the payload's own claims) and runs the full
//! deep admission over the decoded module. AV executes nothing that did not
//! pass through here — including bytes this same compiler produced.

use crate::abc::DecodedAbc;
use crate::contract::{AvSemanticsVersion, AV_SEMANTICS};
use crate::mir::MirModule;
use crate::mir_verify::{verify_mir_with_deadline, CheckedMir, MirVerifyError};

/// Rejection at ABC admission: codec/format, verifier, or identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbcAdmissionError {
    Decode(crate::abc::AbcDecodeError),
    Verify(MirVerifyError),
    Identity(String),
}

impl std::fmt::Display for AbcAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbcAdmissionError::Decode(e) => write!(f, "abc decode: {}", e.code()),
            AbcAdmissionError::Verify(e) => write!(f, "abc verify: {}", e.code()),
            AbcAdmissionError::Identity(m) => write!(f, "abc identity: {}", m),
        }
    }
}

/// Verified, executable module closure. AV accepts ONLY this handle. The
/// AV semantics version is carried on the credential: a store restore that
/// re-verifies under a different runtime version is a mismatch, not a hit.
#[derive(Clone, Debug)]
pub struct LoadedModule {
    checked: CheckedMir,
    av_semantics: AvSemanticsVersion,
}

impl LoadedModule {
    pub fn module(&self) -> &MirModule {
        self.checked.module()
    }

    pub fn av_semantics(&self) -> AvSemanticsVersion {
        self.av_semantics
    }

    pub fn stats(&self) -> &crate::mir_verify::VerifyStats {
        self.checked.stats()
    }
}

/// Admit decoded ABC bytes as an executable module. `expected_owner` is the
/// owner identity the ENVELOPE demands (the payload cannot self-assert it).
pub fn verify_abc(
    decoded: DecodedAbc,
    expected_owner: &crate::mir::OwnerScope,
) -> Result<LoadedModule, AbcAdmissionError> {
    let mut module = decoded.module;
    // Identity re-stamp: the decoded content is admitted under the CALLER's
    // owner scope; wire bytes never carry their own authority.
    module.owner = expected_owner.clone();
    for f in module.funcs.iter_mut() {
        f.owner = expected_owner.clone();
    }
    let checked = verify_mir_with_deadline(&module, deadline())
        .map_err(AbcAdmissionError::Verify)?;
    Ok(LoadedModule { checked, av_semantics: AV_SEMANTICS })
}

fn deadline() -> std::time::Instant {
    std::time::Instant::now() + crate::contract::MAX_VERIFY_WALL
}

/// Admit from raw (flat) bytes.
pub fn verify_abc_bytes(
    bytes: &[u8],
    expected_owner: &crate::mir::OwnerScope,
) -> Result<LoadedModule, AbcAdmissionError> {
    let decoded = crate::abc::decode_abc(bytes).map_err(AbcAdmissionError::Decode)?;
    verify_abc(decoded, expected_owner)
}
