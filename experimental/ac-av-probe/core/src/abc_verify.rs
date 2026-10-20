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

/// Verified module credential. `executable` distinguishes a RE-VERIFIABLE
/// UNIT from an EXECUTABLE CLOSURE (F798-R6-02 + F798-R12-02): a unit kind
/// (or any `Call` targeting an import — pending unit linkage, or an
/// intrinsically host-side symbol) means the module still needs closure
/// assembly/resolution — the executor refuses it instead of running into
/// "unresolved import" infra mid-dispatch. AV executes ONLY executable
/// closures; the unit→closure conversion is the driver's closure assembly
/// (whole-module encode → this verifier), never a flag flip.
#[derive(Clone, Debug)]
pub struct LoadedModule {
    checked: CheckedMir,
    av_semantics: AvSemanticsVersion,
    executable: bool,
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

    /// Whether this credential admits EXECUTION (a fully resolved closure).
    pub fn executable(&self) -> bool {
        self.executable
    }
}

/// Admit decoded ABC bytes as an executable module. `expected_owner` is the
/// owner identity the ENVELOPE demands (the payload cannot self-assert it).
pub fn verify_abc(
    decoded: DecodedAbc,
    expected_owner: &crate::mir::OwnerScope,
) -> Result<LoadedModule, AbcAdmissionError> {
    // Version/profile re-check on the PUBLIC struct (F798-R3-01): decode
    // rejects unknown values on the wire, but a hand-constructed
    // DecodedAbc must not smuggle them past this constructor.
    if decoded.version != crate::abc::ABC_VERSION {
        return Err(AbcAdmissionError::Identity(format!(
            "decoded version {} is not the admitted wire version {}",
            decoded.version,
            crate::abc::ABC_VERSION
        )));
    }
    if decoded.profile != crate::abc::ABC_PROFILE_CORE_I32 {
        return Err(AbcAdmissionError::Identity(format!(
            "decoded profile id {} is not the admitted profile id {}",
            decoded.profile,
            crate::abc::ABC_PROFILE_CORE_I32
        )));
    }
    let mut module = decoded.module;
    // Identity re-stamp: the decoded content is admitted under the CALLER's
    // owner scope; wire bytes never carry their own authority.
    module.owner = expected_owner.clone();
    for f in module.funcs.iter_mut() {
        f.owner = expected_owner.clone();
    }
    let checked =
        verify_mir_with_deadline(&module, deadline()).map_err(AbcAdmissionError::Verify)?;
    // Executable-closure credential (F798-R6-02 + F798-R12-02): the
    // credential admits EXECUTION only for a COMPLETE closure —
    //   * the module must declare complete-module KIND (a re-verifiable
    //     UNIT is verifiable and cacheable but is never an executable
    //     entry until closure assembly links it into a whole module), and
    //   * no `Call` may target an import (pending unit linkage or a
    //     host-side symbol — the closure must be resolved).
    // Every public entry (raw bytes, hand-built DecodedAbc, driver closure,
    // restore) funnels through this one constructor, so the unit/complete
    // split is uniform across all of them.
    let executable = !module.is_unit
        && !module.funcs.iter().any(|f| {
            f.blocks.iter().any(|b| {
                b.instrs.iter().any(|i| {
                    matches!(
                        i,
                        crate::mir::MirInstr::Call {
                            callee: crate::mir::CalleeRef::Import(_),
                            ..
                        }
                    )
                })
            })
        });
    Ok(LoadedModule {
        checked,
        av_semantics: AV_SEMANTICS,
        executable,
    })
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
