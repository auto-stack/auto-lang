//! Pre-call admission (plan §5.1, V-01/V-07): every compatibility
//! dimension is checked BEFORE the consumer links or calls anything —
//! a rejection means the native callee is invoked ZERO times, by
//! construction (this module never returns executable paths on error).
//!
//! The consumer states its expectation ([`ConsumerExpectation`]); the
//! producer side is the encoded module descriptor. Rejections are
//! structured, never stringly "failed" booleans, so a wrong-target
//! module cannot be mistaken for a missing symbol.

use crate::descriptor::{ModuleDescriptor, DESCRIPTOR_FORMAT};
use crate::layout::{layout_record_owned, RecordDefOwned};
use std::collections::BTreeMap;

/// What a consumer requires before it may call into a module.
#[derive(Clone, Debug)]
pub struct ConsumerExpectation {
    pub abi_domain: String,
    pub abi_version: String,
    pub target: String,
    pub profile: String,
    pub packing: String,
    /// (symbol, canonical signature) the consumer intends to call.
    pub required_symbols: Vec<(String, String)>,
    /// (record name, consumer-side definition) the consumer lays out
    /// itself; the descriptor's types must produce the SAME layout.
    pub required_records: Vec<RecordDefOwned>,
    /// (service, min version) the consumer's host offers.
    pub offered_services: Vec<(String, u32)>,
    /// Semantic binding hash the consumer built against.
    pub semref_hash: String,
    /// Per-record semantic refs (logical + content version) the
    /// consumer binds (F-797-R4-01) — minted through the real contract.
    pub semantic_refs: Vec<crate::descriptor::SemanticRef>,
}

/// Structured admission rejections — each names the exact dimension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    Decode(String),
    UnknownFormat {
        found: String,
        known: String,
    },
    WrongDomain {
        expected: String,
        found: String,
    },
    WrongAbiVersion {
        expected: String,
        found: String,
    },
    WrongTarget {
        expected: String,
        found: String,
    },
    WrongProfile {
        expected: String,
        found: String,
    },
    UnsupportedPacking {
        found: String,
    },
    SemRefMismatch {
        expected: String,
        found: String,
    },
    /// F-797-R4-01: a required record's SEMANTIC ref (nominal logical +
    /// content version minted by the real contract) does not match the
    /// consumer's binding — a same-width re-spelling mints a different
    /// content version and is rejected even when layouts coincide.
    SemanticRefMismatch {
        logical: String,
        expected_version: String,
        found_version: String,
    },
    MissingSemanticRef {
        logical: String,
    },
    MissingSymbol {
        symbol: String,
    },
    SignatureMismatch {
        symbol: String,
        expected: String,
        found: String,
    },
    UnknownRecord {
        record: String,
    },
    LayoutMismatch {
        record: String,
        expected: String,
        found: String,
    },
    ServiceMissing {
        service: String,
        need: u32,
        offered: u32,
    },
}

impl std::fmt::Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "abi.admission: {self:?}")
    }
}

/// Successful admission: what the consumer may now call.
#[derive(Clone, Debug)]
pub struct Admission {
    pub interface_fingerprint: String,
    pub admitted_symbols: Vec<String>,
}

/// Admit a producer descriptor bytes against the consumer expectation.
/// `Ok` is the only path that ever leads to a link/run — every `Err`
/// leaves the native callee un-invoked (count 0).
pub fn admit(
    descriptor_bytes: &[u8],
    expect: &ConsumerExpectation,
) -> Result<Admission, Rejection> {
    let desc: ModuleDescriptor = decode_checked(descriptor_bytes)?;
    admit_descriptor(&desc, expect)
}

fn decode_checked(bytes: &[u8]) -> Result<ModuleDescriptor, Rejection> {
    let d: ModuleDescriptor =
        serde_json::from_slice(bytes).map_err(|e| Rejection::Decode(format!("decode: {e}")))?;
    if d.format != DESCRIPTOR_FORMAT {
        return Err(Rejection::UnknownFormat {
            found: d.format,
            known: DESCRIPTOR_FORMAT.into(),
        });
    }
    Ok(d)
}

pub fn admit_descriptor(
    d: &ModuleDescriptor,
    expect: &ConsumerExpectation,
) -> Result<Admission, Rejection> {
    if d.abi_domain != expect.abi_domain {
        return Err(Rejection::WrongDomain {
            expected: expect.abi_domain.clone(),
            found: d.abi_domain.clone(),
        });
    }
    if d.abi_version != expect.abi_version {
        return Err(Rejection::WrongAbiVersion {
            expected: expect.abi_version.clone(),
            found: d.abi_version.clone(),
        });
    }
    if d.target != expect.target {
        return Err(Rejection::WrongTarget {
            expected: expect.target.clone(),
            found: d.target.clone(),
        });
    }
    if d.profile != expect.profile {
        return Err(Rejection::WrongProfile {
            expected: expect.profile.clone(),
            found: d.profile.clone(),
        });
    }
    if d.packing != expect.packing {
        return Err(Rejection::UnsupportedPacking {
            found: d.packing.clone(),
        });
    }
    if d.semref_hash != expect.semref_hash {
        return Err(Rejection::SemRefMismatch {
            expected: expect.semref_hash.clone(),
            found: d.semref_hash.clone(),
        });
    }

    // Symbols: every required symbol exists with the exact signature.
    let mut admitted = Vec::new();
    for (name, want_sig) in &expect.required_symbols {
        let entry =
            d.symbols
                .iter()
                .find(|s| &s.name == name)
                .ok_or_else(|| Rejection::MissingSymbol {
                    symbol: name.clone(),
                })?;
        let found_sig = entry.form.signature();
        if &found_sig != want_sig {
            return Err(Rejection::SignatureMismatch {
                symbol: name.clone(),
                expected: want_sig.clone(),
                found: found_sig,
            });
        }
        admitted.push(name.clone());
    }

    // F-797-R4-01: every semantic ref the consumer binds must be
    // present in the descriptor with the SAME content version. The
    // consumer's refs come from the real semantic contract, so a
    // same-width re-spelling (u32 for i32) mints a different version
    // and dies HERE, before any layout comparison.
    for want_ref in &expect.semantic_refs {
        let found = d
            .semantic_refs
            .iter()
            .find(|r| r.logical == want_ref.logical);
        match found {
            None => {
                return Err(Rejection::MissingSemanticRef {
                    logical: want_ref.logical.clone(),
                })
            }
            Some(found) if found.content_version == want_ref.content_version => {}
            Some(found) => {
                return Err(Rejection::SemanticRefMismatch {
                    logical: want_ref.logical.clone(),
                    expected_version: want_ref.content_version.clone(),
                    found_version: found.content_version.clone(),
                })
            }
        }
    }

    // Records: the descriptor's OWN types, laid out under the frozen
    // rules, must match the consumer's independently computed layout.
    let table: BTreeMap<String, RecordDefOwned> = d
        .records
        .iter()
        .map(|r| (r.name.clone(), r.clone()))
        .collect();
    for req in &expect.required_records {
        let found = layout_record_owned(&table, &req.name).map_err(|e| {
            match e {
                // A required record the descriptor cannot lay out at all
                // (recursive/unknown/oversize) surfaces as its specific
                // controlled layout error, not a panic.
                crate::layout::LayoutError::UnknownRecord(n) => {
                    Rejection::UnknownRecord { record: n }
                }
                other => Rejection::LayoutMismatch {
                    record: req.name.clone(),
                    expected: "computable layout".into(),
                    found: other.to_string(),
                },
            }
        })?;
        let consumer_table: BTreeMap<String, RecordDefOwned> = expect
            .required_records
            .iter()
            .map(|r| (r.name.clone(), r.clone()))
            .collect();
        let want = layout_record_owned(&consumer_table, &req.name).map_err(|e| {
            Rejection::LayoutMismatch {
                record: req.name.clone(),
                expected: "computable layout".into(),
                found: e.to_string(),
            }
        })?;
        if found != want {
            return Err(Rejection::LayoutMismatch {
                record: req.name.clone(),
                expected: format!("{want:?}"),
                found: format!("{found:?}"),
            });
        }
    }

    // Runtime services: producer requirements must be satisfiable by
    // what the consumer's host offers (>= required version).
    for (service, need) in &d.runtime.services {
        match expect.offered_services.iter().find(|(n, _)| n == service) {
            Some((_, offered)) if offered >= need => {}
            Some((_, offered)) => {
                return Err(Rejection::ServiceMissing {
                    service: service.clone(),
                    need: *need,
                    offered: *offered,
                })
            }
            None => {
                return Err(Rejection::ServiceMissing {
                    service: service.clone(),
                    need: *need,
                    offered: 0,
                })
            }
        }
    }

    Ok(Admission {
        interface_fingerprint: d.fingerprint().map_err(|e| Rejection::Decode(e.0))?,
        admitted_symbols: admitted,
    })
}

/// Convenience: the consumer expectation matching the probe module
/// (used by tests and, from T-03 on, by the leg drivers).
pub fn probe_expectation() -> ConsumerExpectation {
    let d = crate::descriptor::ModuleDescriptor::probe_module();
    ConsumerExpectation {
        abi_domain: crate::descriptor::ABI_DOMAIN.into(),
        abi_version: crate::descriptor::ABI_VERSION.into(),
        target: crate::layout::TARGET.into(),
        profile: crate::layout::ABI_PROFILE.into(),
        packing: crate::descriptor::PACKING.into(),
        required_symbols: d
            .symbols
            .iter()
            .map(|s| (s.name.clone(), s.form.signature()))
            .collect(),
        required_records: d.records.clone(),
        offered_services: vec![
            ("alloc".into(), 1),
            ("free".into(), 1),
            ("handle-registry".into(), 1),
        ],
        semref_hash: crate::semref::binding_hash().into(),
        semantic_refs: vec![crate::descriptor::SemanticRef {
            logical: crate::semref::rec_mix_ref()
                .expect("record ref mints")
                .logical
                .as_str()
                .to_string(),
            content_version: crate::semref::rec_mix_ref()
                .expect("record ref mints")
                .content
                .as_str()
                .to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::{ModuleDescriptor, ParamKind, SymbolForm};

    fn admit_probe() -> Admission {
        let d = ModuleDescriptor::probe_module();
        admit_descriptor(&d, &probe_expectation()).expect("legal sibling must pass")
    }

    #[test]
    fn legal_probe_module_admits() {
        let a = admit_probe();
        assert_eq!(a.admitted_symbols, vec!["p797_mix".to_string()]);
        assert_eq!(a.interface_fingerprint.len(), 64);
    }

    #[test]
    fn signature_mismatch_named_exactly() {
        let mut d = ModuleDescriptor::probe_module();
        if let SymbolForm::Pure { params, .. } = &mut d.symbols[0].form {
            params[0] = ParamKind::U32;
        }
        let r = admit_descriptor(&d, &probe_expectation()).unwrap_err();
        match r {
            Rejection::SignatureMismatch {
                symbol, expected, ..
            } => {
                assert_eq!(symbol, "p797_mix");
                assert!(expected.starts_with("pure(i32,"));
            }
            other => panic!("expected SignatureMismatch, got {other:?}"),
        }
    }
}
