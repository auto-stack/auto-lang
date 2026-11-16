//! PLAN-978 T-03/T-08 (AC-ID-RUN, V-ID-RUN/REJECT): the runtime
//! compatibility record — the CONSUMER's current requirements checked
//! against a PROVIDER's offer, independently of how either was built.
//!
//! Frozen rules (plan §5 ID-01..04):
//! * target OS/arch/env/width/endianness must match;
//! * the ABI domain and interface schema must match (versioned — never
//!   "same major so fine");
//! * every consumer-required symbol must be offered with the SAME
//!   signature hash (a required subset — extra offered symbols are fine);
//! * the semantic owner must match (routing a call across owners is a
//!   rejection, not a fallback);
//! * threading model must match;
//! * every declared runtime dependency of the consumer must be satisfied
//!   by the offer (CRT, interpreter, system libraries);
//! * **the building rustc identity is NOT a compared axis** — a provider
//!   built by a different rustc is NOT rejected when the above hold, and
//!   a matching rustc label alone admits nothing.

use serde::{Deserialize, Serialize};

use crate::M2Reject;

pub const COMPAT_SCHEMA: &str = "ace-m2-build/runtime-compat/1";

/// The physical target axis (facts, not toolchain labels).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetAxis {
    pub os: String,
    pub arch: String,
    pub env: String,
    pub bits: u32,
    pub endianness: String,
}

impl TargetAxis {
    /// The axis of THIS machine (the only host this package validates on;
    /// Linux legs are deferred to PLAN-1063 per the 2026-11-15 ruling).
    pub fn host() -> Self {
        if cfg!(target_os = "windows") {
            Self {
                os: "windows".into(),
                arch: "x86_64".into(),
                env: "msvc".into(),
                bits: 64,
                endianness: "little".into(),
            }
        } else {
            Self {
                os: std::env::consts::OS.into(),
                arch: std::env::consts::ARCH.into(),
                env: std::env::consts::FAMILY.into(),
                bits: (std::mem::size_of::<usize>() * 8) as u32,
                endianness: if cfg!(target_endian = "little") {
                    "little".into()
                } else {
                    "big".into()
                },
            }
        }
    }
}

/// One required/offered symbol with a signature hash (the interface
/// contract, not a name-only match).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolContract {
    pub name: String,
    pub signature_hash: String,
}

/// What the CURRENT consumer requires (derived from the consumer's own
/// declarations at admission time — never copied from a candidate
/// payload; plan §5 ID-04).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumerRequirements {
    pub schema: String,
    pub target: TargetAxis,
    /// e.g. "C" (the frozen F0 C-ABI surface) — a domain fact.
    pub abi_domain: String,
    /// e.g. "ace-stdlib-assembly/1" — the interface schema both sides
    /// must speak.
    pub interface_schema: String,
    pub required_symbols: Vec<SymbolContract>,
    /// The semantic owner of the required surface.
    pub semantic_owner: String,
    pub threading: String,
    /// Declared runtime dependencies ("msvc-crt:VCRUNTIME140+ucrt",
    /// "cpython>=3.10", ...). Build tools are NOT runtime dependencies.
    pub runtime_dependencies: Vec<String>,
}

impl ConsumerRequirements {
    /// The requirements of the precompiled native/A2R program lane on the
    /// host platform (the B01-class scalar program: entry symbol only,
    /// MSVC CRT family recorded from the toolchain family the lane
    /// linked, never a guess).
    pub fn native_program(entry_symbol: &str, signature_hash: &str) -> Self {
        Self {
            schema: COMPAT_SCHEMA.into(),
            target: TargetAxis::host(),
            abi_domain: "C".into(),
            interface_schema: "ace-stdlib-assembly/1".into(),
            required_symbols: vec![SymbolContract {
                name: entry_symbol.into(),
                signature_hash: signature_hash.into(),
            }],
            semantic_owner: "auto.m2-build/program".into(),
            threading: "single".into(),
            runtime_dependencies: vec!["msvc-crt:VCRUNTIME140+ucrt".into()],
        }
    }
}

/// What one provider artifact OFFERS (recorded from the artifact itself).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderOffer {
    pub target: TargetAxis,
    pub abi_domain: String,
    pub interface_schema: String,
    pub offered_symbols: Vec<SymbolContract>,
    pub semantic_owner: String,
    pub threading: String,
    pub runtime_dependencies: Vec<String>,
}

/// A named compatibility rejection (checked BEFORE any dispatch — plan
/// §5 ID-03: never call first and see).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatRejection {
    pub code: &'static str,
    pub axis: String,
    pub detail: String,
}

impl std::fmt::Display for CompatRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]: {}", self.code, self.axis, self.detail)
    }
}

/// Check consumer requirements against a provider offer. Every axis is
/// checked; the FIRST mismatch is returned as a named rejection.
pub fn check_compatibility(
    consumer: &ConsumerRequirements,
    offer: &ProviderOffer,
) -> Result<(), CompatRejection> {
    let c = &consumer.target;
    let o = &offer.target;
    if c.os != o.os {
        return Err(CompatRejection {
            code: "m2build.compat.target-os",
            axis: "target.os".into(),
            detail: format!("consumer requires `{}`, offer is `{}`", c.os, o.os),
        });
    }
    if c.arch != o.arch {
        return Err(CompatRejection {
            code: "m2build.compat.target-arch",
            axis: "target.arch".into(),
            detail: format!("consumer requires `{}`, offer is `{}`", c.arch, o.arch),
        });
    }
    if c.env != o.env {
        return Err(CompatRejection {
            code: "m2build.compat.target-env",
            axis: "target.env".into(),
            detail: format!("consumer requires `{}`, offer is `{}`", c.env, o.env),
        });
    }
    if c.bits != o.bits {
        return Err(CompatRejection {
            code: "m2build.compat.target-bits",
            axis: "target.bits".into(),
            detail: format!("consumer requires `{}`, offer is `{}`", c.bits, o.bits),
        });
    }
    if c.endianness != o.endianness {
        return Err(CompatRejection {
            code: "m2build.compat.target-endianness",
            axis: "target.endianness".into(),
            detail: format!(
                "consumer requires `{}`, offer is `{}`",
                c.endianness, o.endianness
            ),
        });
    }
    if consumer.abi_domain != offer.abi_domain {
        return Err(CompatRejection {
            code: "m2build.compat.abi-domain",
            axis: "abi_domain".into(),
            detail: format!(
                "consumer requires `{}`, offer is `{}` (same CPU or repr(C) alone proves nothing)",
                consumer.abi_domain, offer.abi_domain
            ),
        });
    }
    if consumer.interface_schema != offer.interface_schema {
        return Err(CompatRejection {
            code: "m2build.compat.interface-schema",
            axis: "interface_schema".into(),
            detail: format!(
                "consumer requires `{}`, offer is `{}` (versioned — not same-major-lenient)",
                consumer.interface_schema, offer.interface_schema
            ),
        });
    }
    for req in &consumer.required_symbols {
        let Some(offered) = offer.offered_symbols.iter().find(|s| s.name == req.name) else {
            return Err(CompatRejection {
                code: "m2build.compat.required-symbol",
                axis: format!("symbol:{}", req.name),
                detail: format!("required symbol `{}` not offered", req.name),
            });
        };
        if offered.signature_hash != req.signature_hash {
            return Err(CompatRejection {
                code: "m2build.compat.symbol-signature",
                axis: format!("symbol:{}", req.name),
                detail: format!(
                    "required signature `{}`, offered `{}`",
                    req.signature_hash, offered.signature_hash
                ),
            });
        }
    }
    if consumer.semantic_owner != offer.semantic_owner {
        return Err(CompatRejection {
            code: "m2build.compat.semantic-owner",
            axis: "semantic_owner".into(),
            detail: format!(
                "consumer requires `{}`, offer is `{}` (no cross-owner rerouting)",
                consumer.semantic_owner, offer.semantic_owner
            ),
        });
    }
    if consumer.threading != offer.threading {
        return Err(CompatRejection {
            code: "m2build.compat.threading",
            axis: "threading".into(),
            detail: format!(
                "consumer requires `{}`, offer is `{}`",
                consumer.threading, offer.threading
            ),
        });
    }
    for dep in &consumer.runtime_dependencies {
        if !offer.runtime_dependencies.contains(dep) {
            return Err(CompatRejection {
                code: "m2build.compat.runtime-dep",
                axis: format!("runtime-dep:{dep}"),
                detail: format!("declared runtime dependency `{dep}` not satisfied by the offer"),
            });
        }
    }
    Ok(())
}

/// Convenience: the requirements record as a reject on its own (used by
/// SDK admission when no offer is even present).
pub fn compat_reject_as_m2(r: CompatRejection) -> M2Reject {
    M2Reject::new(r.code, r.detail)
}
