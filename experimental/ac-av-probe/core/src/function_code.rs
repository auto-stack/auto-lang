//! Per-function installed code (PLAN-828 T-01 first case).
//!
//! A `FunctionCode` is the session's resident handle for ONE function's
//! admitted execution code. It can only be constructed from an ABC-admitted
//! single-function unit credential ([`crate::abc_verify::LoadedModule`] with
//! `is_unit` set): the encode→decode→`verify_abc` admission of the candidate
//! unit is the trust step, and this type adds no way to skip it. Unchanged
//! functions keep the SAME `Arc<FunctionCode>` across program-snapshot
//! revisions — reuse is by object identity, never by re-decode.

use crate::abc_verify::LoadedModule;
use std::sync::Arc;

/// Admitted single-function execution code.
///
/// Construction goes through [`FunctionCode::admit_unit`] only; the unit
/// credential inside is never executable on its own (`is_unit` kind), it is
/// linked into a [`crate::program_snapshot::ProgramSnapshot`] before any
/// dispatch.
#[derive(Clone, Debug)]
pub struct FunctionCode {
    name: String,
    unit: LoadedModule,
}

impl FunctionCode {
    /// Admit an ABC-verified unit as resident function code. Rejects
    /// non-unit credentials, multi-function payloads and units whose module
    /// disagrees with the unit kind — the caller cannot hand in a forged
    /// executable closure through this seam.
    pub fn admit_unit(loaded: LoadedModule) -> Result<Self, String> {
        let m = loaded.module();
        if loaded.executable() {
            return Err(format!(
                "function code admission requires a re-verifiable unit credential, `{}` is an executable closure",
                m.funcs.first().map(|f| f.name.as_str()).unwrap_or("?")
            ));
        }
        if !m.is_unit {
            return Err("function code admission requires is_unit kind".to_string());
        }
        if m.funcs.len() != 1 {
            return Err(format!(
                "function code must hold exactly one function, found {}",
                m.funcs.len()
            ));
        }
        Ok(FunctionCode {
            name: m.funcs[0].name.clone(),
            unit: loaded,
        })
    }

    /// The function's stable logical name (the def id text, e.g. `d_main`).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The admitted unit credential (never executable standalone).
    pub fn unit(&self) -> &LoadedModule {
        &self.unit
    }
}

/// Resident shared handle: identity reuse across snapshots is `Arc::ptr_eq`.
pub type SharedFunctionCode = Arc<FunctionCode>;
