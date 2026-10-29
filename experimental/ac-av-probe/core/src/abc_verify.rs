//! ABC admission — the only constructor of the executable
//! [`LoadedModule`] (PLAN-798 §5.2).
//!
//! The wire decode ([`crate::abc::decode_abc`]) performs pure structure
//! checks; this stage re-stamps identity (owner/profile/versions come from
//! the ENVELOPE, never from the payload's own claims) and runs the full
//! deep admission over the decoded module. AV executes nothing that did not
//! pass through here — including bytes this same compiler produced.

use crate::abc::DecodedAbc;
use crate::compact_abc::{DecodedCompactAbc, COMPACT_ABC_PROFILE, COMPACT_ABC_VERSION};
use crate::contract::AvSemanticsVersion;
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
/// All wire generations admit here: AVAB (core-i32), AVAD v1 (data,
/// PLAN-820) and AVAD v2 (owned, PLAN-821);
/// the decoded profile id selects the admitted version tuple and the AV
/// semantics version stamped on the credential.
pub fn verify_abc(
    decoded: DecodedAbc,
    expected_owner: &crate::mir::OwnerScope,
) -> Result<LoadedModule, AbcAdmissionError> {
    // Version/profile re-check on the PUBLIC struct (F798-R3-01): decode
    // rejects unknown values on the wire, but a hand-constructed
    // DecodedAbc must not smuggle them past this constructor.
    let (want_version, want_profile, av_semantics) =
        if decoded.profile == crate::abc::ABC_PROFILE_CORE_I32 {
            (
                crate::abc::ABC_VERSION,
                crate::abc::ABC_PROFILE_CORE_I32,
                crate::contract::AV_SEMANTICS,
            )
        } else if decoded.profile == crate::contract::ABC_PROFILE_DATA {
            (
                crate::contract::ABC_DATA_VERSION,
                crate::contract::ABC_PROFILE_DATA,
                crate::contract::AV_DATA_SEMANTICS,
            )
        } else if decoded.profile == crate::contract::ABC_PROFILE_OWNED {
            (
                crate::contract::ABC_OWNED_VERSION,
                crate::contract::ABC_PROFILE_OWNED,
                crate::contract::AV_OWNED_SEMANTICS,
            )
        } else if decoded.profile == crate::contract::ABC_PROFILE_COLLECTIONS {
            (
                crate::contract::ABC_COLLECTIONS_VERSION,
                crate::contract::ABC_PROFILE_COLLECTIONS,
                crate::contract::AV_COLLECTIONS_SEMANTICS,
            )
        } else if decoded.profile == crate::contract::ABC_PROFILE_CLOSURES {
            (
                crate::contract::ABC_CLOSURES_VERSION,
                crate::contract::ABC_PROFILE_CLOSURES,
                crate::contract::AV_CLOSURES_SEMANTICS,
            )
        } else if decoded.profile == crate::contract::ABC_PROFILE_DYNAMIC {
            (
                crate::contract::ABC_DYNAMIC_VERSION,
                crate::contract::ABC_PROFILE_DYNAMIC,
                crate::contract::AV_DYNAMIC_SEMANTICS,
            )
        } else if decoded.profile == crate::contract::ABC_PROFILE_HOST_CONSOLE {
            (
                crate::contract::ABC_HOST_CONSOLE_VERSION,
                crate::contract::ABC_PROFILE_HOST_CONSOLE,
                crate::contract::AV_HOST_CONSOLE_SEMANTICS,
            )
        } else if decoded.profile == crate::contract::ABC_PROFILE_ENTRY_UNIT {
            // PLAN-862: the entry-unit generation (`AVAU` v1, id 7).
            (
                crate::contract::ABC_ENTRY_UNIT_VERSION,
                crate::contract::ABC_PROFILE_ENTRY_UNIT,
                crate::contract::AV_ENTRY_UNIT_SEMANTICS,
            )
        } else {
            return Err(AbcAdmissionError::Identity(format!(
                "decoded profile id {} is not an admitted profile",
                decoded.profile
            )));
        };
    if decoded.version != want_version {
        return Err(AbcAdmissionError::Identity(format!(
            "decoded version {} is not the admitted wire version {} for profile id {}",
            decoded.version, want_version, decoded.profile
        )));
    }
    if decoded.profile != want_profile {
        return Err(AbcAdmissionError::Identity(format!(
            "decoded profile id {} is not the admitted profile id {}",
            decoded.profile, want_profile
        )));
    }
    let profile_name = match want_profile {
        crate::abc::ABC_PROFILE_CORE_I32 => crate::contract::AV_PROFILE,
        crate::contract::ABC_PROFILE_DATA => crate::contract::AV_DATA_PROFILE,
        crate::contract::ABC_PROFILE_OWNED => crate::contract::AV_OWNED_PROFILE,
        crate::contract::ABC_PROFILE_COLLECTIONS => crate::contract::AV_COLLECTIONS_PROFILE,
        crate::contract::ABC_PROFILE_CLOSURES => crate::contract::AV_CLOSURES_PROFILE,
        crate::contract::ABC_PROFILE_DYNAMIC => crate::contract::AV_DYNAMIC_PROFILE,
        crate::contract::ABC_PROFILE_ENTRY_UNIT => crate::contract::AV_ENTRY_UNIT_PROFILE,
        crate::contract::ABC_PROFILE_HOST_CONSOLE => crate::contract::AV_HOST_CONSOLE_PROFILE,
        _ => unreachable!("identity checked above"),
    };
    if decoded.module.profile != profile_name {
        return Err(AbcAdmissionError::Identity(
            "wire profile and module semantic profile differ".into(),
        ));
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
    let executable = executable_closure(&module);
    Ok(LoadedModule {
        checked,
        av_semantics,
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

/// Executable-closure credential shared by every admission entry: a
/// re-verifiable UNIT is never an executable entry, and any `Call`
/// targeting an import means the closure is unresolved (pending unit
/// linkage or a host-side symbol).
fn executable_closure(module: &MirModule) -> bool {
    !module.is_unit
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
        })
}

/// PLAN-826: the compact wire (AVAC/1) admission — the SAME deep admission
/// contract as [`verify_abc`] (envelope owner re-stamp, MIR verification
/// with deadline, executable-closure credential) for the compact encoding
/// of the scalar profile. The compact profile id 1 denotes the SAME
/// `av.core-i32/1` profile and AV semantics as the legacy AVAB core wire:
/// the wire GENERATION differs, the admitted semantics do not. AVAC bytes
/// never admit through `verify_abc` and AVAB bytes never admit here — the
/// magic isolates the generations at decode.
pub fn verify_compact_abc(
    decoded: DecodedCompactAbc,
    expected_owner: &crate::mir::OwnerScope,
) -> Result<LoadedModule, AbcAdmissionError> {
    // Version/profile re-check on the PUBLIC struct (same rule as
    // verify_abc): decode rejects unknown wire values, but a
    // hand-constructed DecodedCompactAbc must not smuggle them past this
    // constructor.
    if decoded.profile != COMPACT_ABC_PROFILE {
        return Err(AbcAdmissionError::Identity(format!(
            "decoded compact profile id {} is not the admitted compact profile id {}",
            decoded.profile, COMPACT_ABC_PROFILE
        )));
    }
    if decoded.version != COMPACT_ABC_VERSION {
        return Err(AbcAdmissionError::Identity(format!(
            "decoded compact version {} is not the admitted compact wire version {}",
            decoded.version, COMPACT_ABC_VERSION
        )));
    }
    let mut module = decoded.module;
    // Identity re-stamp: same rule as verify_abc — wire bytes never carry
    // their own authority.
    module.owner = expected_owner.clone();
    for f in module.funcs.iter_mut() {
        f.owner = expected_owner.clone();
    }
    let checked =
        verify_mir_with_deadline(&module, deadline()).map_err(AbcAdmissionError::Verify)?;
    let executable = executable_closure(&module);
    Ok(LoadedModule {
        checked,
        av_semantics: crate::contract::AV_SEMANTICS,
        executable,
    })
}

/// Admit compact wire bytes (AVAC magic) from raw bytes.
pub fn verify_compact_abc_bytes(
    bytes: &[u8],
    expected_owner: &crate::mir::OwnerScope,
) -> Result<LoadedModule, AbcAdmissionError> {
    let decoded =
        crate::compact_abc::decode_compact_abc(bytes).map_err(AbcAdmissionError::Decode)?;
    verify_compact_abc(decoded, expected_owner)
}
