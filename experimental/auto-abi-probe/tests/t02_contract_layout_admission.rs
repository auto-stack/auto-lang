//! PLAN-797 T-02: descriptor contract, layout conformance against the
//! independent C oracle, and admission rejections (plan §8 T-02; V-01,
//! V-03, V-07 layout share).
//!
//! Naming follows the plan's future entrypoints: `abi_contract_`,
//! `abi_layout_`, `abi_admission_`.

#![cfg(windows)]

use auto_abi_probe::ctool;
use auto_abi_probe::descriptor::{ModuleDescriptor, ParamKind, RetKind, SymbolForm};
use auto_abi_probe::layout::{
    self, FieldDefOwned, FieldTyOwned, LayoutError, RecordDefOwned, SemScalar, MAX_NESTING_DEPTH,
};
use auto_abi_probe::link::link_pe;
use auto_abi_probe::run;
use auto_abi_probe::validate::{admit, admit_descriptor, probe_expectation, Rejection};

use std::collections::BTreeMap;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// abi_contract_ — descriptor identity + admission (V-01)
// ---------------------------------------------------------------------------

/// V-01 core: every mismatch dimension is rejected BEFORE any native
/// call (admission returns Err; nothing links or runs — the scratch dir
/// for this case never gains an exe), while the legal sibling passes.
#[test]
fn abi_contract_admission_rejects_each_dimension() {
    let expect = probe_expectation();

    let variants: Vec<(&str, Box<dyn Fn(&mut ModuleDescriptor)>)> = vec![
        (
            "unknown-format",
            Box::new(|d: &mut ModuleDescriptor| d.format = "autoabi.descriptor/999".into()),
        ),
        (
            "wrong-domain",
            Box::new(|d: &mut ModuleDescriptor| d.abi_domain = "something-else".into()),
        ),
        (
            "wrong-abi-version",
            Box::new(|d: &mut ModuleDescriptor| d.abi_version = "0.2".into()),
        ),
        (
            "wrong-target",
            Box::new(|d: &mut ModuleDescriptor| d.target = "x86_64-unknown-linux-gnu".into()),
        ),
        (
            "wrong-profile",
            Box::new(|d: &mut ModuleDescriptor| {
                d.profile = "autoabi.win-x64.v0.2-candidate".into()
            }),
        ),
        (
            "unsupported-packing",
            Box::new(|d: &mut ModuleDescriptor| d.packing = "packed-1".into()),
        ),
        (
            "semref-mismatch",
            Box::new(|d: &mut ModuleDescriptor| d.semref_hash = "0".repeat(64)),
        ),
        (
            "missing-symbol",
            Box::new(|d: &mut ModuleDescriptor| d.symbols.clear()),
        ),
        (
            "signature-mismatch",
            Box::new(|d: &mut ModuleDescriptor| {
                if let SymbolForm::Pure { params, .. } = &mut d.symbols[0].form {
                    params[0] = ParamKind::U32;
                }
            }),
        ),
        (
            "service-missing",
            Box::new(|d: &mut ModuleDescriptor| {
                d.runtime.services.push(("hot-reload".into(), 1));
            }),
        ),
        (
            "service-version-too-old",
            Box::new(|d: &mut ModuleDescriptor| {
                d.runtime.services.insert(0, ("alloc".into(), 2));
            }),
        ),
        (
            "layout-mismatch",
            Box::new(|d: &mut ModuleDescriptor| {
                // Reorder RecMix fields in the PRODUCER descriptor: the
                // consumer lays out its own definition and must refuse.
                if let Some(rec) = d.records.iter_mut().find(|r| r.name == "RecMix") {
                    rec.fields.swap(0, 1);
                }
            }),
        ),
    ];

    for (name, mutate) in variants {
        let mut d = ModuleDescriptor::probe_module();
        mutate(&mut d);
        let bytes = d.encode().unwrap();
        let rej = admit(&bytes, &expect)
            .err()
            .unwrap_or_else(|| panic!("{name}: expected rejection"));
        // Structured, dimension-specific rejection — never a raw decode
        // failure (each variant is a well-formed descriptor).
        assert!(
            !matches!(rej, Rejection::Decode(_)),
            "{name}: got raw decode rejection {rej:?}"
        );
        // The rejection is observable and specific.
        let msg = format!("{rej}");
        assert!(msg.contains("abi.admission"), "{name}: {msg}");
        // Native call count is ZERO by construction: admission produced
        // no executable path, and no probe exe exists for this case.
        let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/autoabi-t01")
            .join(format!("v01_reject_{name}"));
        assert!(
            !scratch.join("probe.exe").is_file(),
            "{name}: rejected module must never leave a runnable PE"
        );
    }

    // Legal sibling: the untouched probe module admits.
    let good = ModuleDescriptor::probe_module().encode().unwrap();
    let ok = admit(&good, &expect).expect("legal sibling must admit");
    assert_eq!(ok.admitted_symbols, vec!["p797_mix".to_string()]);
}

#[test]
fn abi_contract_descriptor_roundtrip_and_fingerprint() {
    let d = ModuleDescriptor::probe_module();
    let bytes = d.encode().unwrap();
    assert_eq!(ModuleDescriptor::decode(&bytes).unwrap(), d);
    // Truncated bytes: controlled decode rejection, no panic.
    let err = ModuleDescriptor::decode(&bytes[..bytes.len() / 2]).unwrap_err();
    assert!(err.0.contains("decode"));
    // Fingerprint is a content function: same content, same hash; a
    // changed symbol set, a different hash.
    assert_eq!(
        d.fingerprint().unwrap(),
        ModuleDescriptor::probe_module().fingerprint().unwrap()
    );
    let mut d2 = d.clone();
    d2.symbols.push(auto_abi_probe::descriptor::SymbolEntry {
        name: "p797_extra".into(),
        form: SymbolForm::Fallible {
            ctx: None,
            params: vec![],
            outs: vec![ParamKind::I64],
        },
    });
    assert_ne!(d.fingerprint().unwrap(), d2.fingerprint().unwrap());
    // Signature spellings are exact and form-aware.
    assert_eq!(d2.symbols[1].form.signature(), "fallible()->status,out i64");
    assert_eq!(RetKind::Unit.spell(), "unit");
}

// ---------------------------------------------------------------------------
// abi_layout_ — conformance against the independent C oracle (V-03)
// ---------------------------------------------------------------------------

struct CLayout {
    sizes: BTreeMap<String, (u64, u64)>, // record -> (size, align)
    fields: BTreeMap<(String, String), (u64, u64)>, // (record, field) -> (offset, width)
}

fn owned_defs() -> Vec<RecordDefOwned> {
    use FieldTyOwned::Scalar;
    let rec_mix = RecordDefOwned {
        name: "RecMix".into(),
        fields: vec![
            FieldDefOwned {
                name: "flag".into(),
                ty: Scalar(SemScalar::Bool),
            },
            FieldDefOwned {
                name: "count".into(),
                ty: Scalar(SemScalar::I32),
            },
            FieldDefOwned {
                name: "tag".into(),
                ty: Scalar(SemScalar::Char),
            },
            FieldDefOwned {
                name: "big".into(),
                ty: Scalar(SemScalar::I64),
            },
        ],
    };
    vec![
        rec_mix,
        RecordDefOwned {
            name: "RecPad".into(),
            fields: vec![
                FieldDefOwned {
                    name: "a".into(),
                    ty: Scalar(SemScalar::U8),
                },
                FieldDefOwned {
                    name: "b".into(),
                    ty: Scalar(SemScalar::I64),
                },
                FieldDefOwned {
                    name: "c".into(),
                    ty: Scalar(SemScalar::U8),
                },
            ],
        },
        RecordDefOwned {
            name: "RecNested".into(),
            fields: vec![
                FieldDefOwned {
                    name: "x".into(),
                    ty: Scalar(SemScalar::I32),
                },
                FieldDefOwned {
                    name: "m".into(),
                    ty: FieldTyOwned::Record("RecMix".into()),
                },
                FieldDefOwned {
                    name: "d".into(),
                    ty: Scalar(SemScalar::F64),
                },
            ],
        },
        RecordDefOwned {
            name: "RecOrderB".into(),
            fields: vec![
                FieldDefOwned {
                    name: "big".into(),
                    ty: Scalar(SemScalar::I64),
                },
                FieldDefOwned {
                    name: "flag".into(),
                    ty: Scalar(SemScalar::Bool),
                },
                FieldDefOwned {
                    name: "count".into(),
                    ty: Scalar(SemScalar::I32),
                },
                FieldDefOwned {
                    name: "tag".into(),
                    ty: Scalar(SemScalar::Char),
                },
            ],
        },
    ]
}

/// Compile + run the C layout oracle and parse its wire format. Each
/// caller passes a unique scratch case name (tests run in parallel and
/// every scratch dir is recreated from empty).
fn run_c_layout_oracle(case: &str) -> CLayout {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = run::scratch(case);
    let src = manifest.join("cside/layout_probe.c");
    let obj = dir.join("layout_probe.obj");
    ctool::compile_c(&src, &manifest.join("cside"), &obj).expect("cl compiles the layout oracle");
    let exe = link_pe(&dir, &[obj], "p797_layout_entry").expect("link layout oracle PE");
    let pe = run::run_pe(&exe).expect("run layout oracle PE");
    assert_eq!(pe.code, 0, "layout oracle exit; stdout:\n{}", pe.stdout);

    let mut c = CLayout {
        sizes: BTreeMap::new(),
        fields: BTreeMap::new(),
    };
    for line in pe.stdout.lines() {
        let t: Vec<&str> = line.split_whitespace().collect();
        match t.as_slice() {
            ["R", name, "S", size, "A", align] => {
                c.sizes.insert(
                    name.to_string(),
                    (size.parse().unwrap(), align.parse().unwrap()),
                );
            }
            ["F", rec, field, "O", off, "W", w] => {
                c.fields.insert(
                    (rec.to_string(), field.to_string()),
                    (off.parse().unwrap(), w.parse().unwrap()),
                );
            }
            ["N", rec, field, sub, "O", off, "W", w] => {
                // The C oracle emits the dotted path (e.g. "m.flag") as
                // the field plus the bare subfield name; cross-check the
                // two agree, then key by the dotted path — the same
                // shape the Rust layout engine flattens to.
                assert!(field.ends_with(sub), "bad nested oracle line: {line:?}");
                c.fields.insert(
                    (rec.to_string(), field.to_string()),
                    (off.parse().unwrap(), w.parse().unwrap()),
                );
            }
            ["END"] => break,
            _ => panic!("bad oracle line: {line:?}"),
        }
    }
    assert!(!c.sizes.is_empty(), "oracle produced no records");
    c
}

/// V-03: the Rust layout engine matches the C compiler's own
/// sizeof/__alignof/offsetof for every fixture record — padding, field
/// order, and nested records included.
#[test]
fn abi_layout_matches_c_oracle() {
    let oracle = run_c_layout_oracle("layout_oracle_main");
    let defs = owned_defs();
    let table: BTreeMap<String, RecordDefOwned> =
        defs.iter().map(|r| (r.name.clone(), r.clone())).collect();

    for def in &defs {
        let l = layout::layout_record_owned(&table, &def.name).expect("layout computes");
        let (c_size, c_align) = oracle
            .sizes
            .get(&def.name)
            .unwrap_or_else(|| panic!("C oracle lacks {}", def.name));
        assert_eq!(
            (l.size as u64, l.align as u64),
            (*c_size, *c_align),
            "{} size/align vs C",
            def.name
        );
        // Field widths/offsets: the nested engine flattens `m.count`
        // style paths, the C oracle reports them the same way.
        for f in &l.fields {
            let (c_off, c_w) = oracle
                .fields
                .get(&(def.name.clone(), f.name.clone()))
                .unwrap_or_else(|| panic!("C oracle lacks {}.{}", def.name, f.name));
            assert_eq!(
                (f.offset as u64, f.size as u64),
                (*c_off, *c_w),
                "{}.{} offset/width vs C",
                def.name,
                f.name
            );
        }
    }
}

/// V-03 negative control: a mutated (wrong) expected offset must FAIL
/// the comparison — proving the conformance check discriminates and a
/// green `abi_layout_matches_c_oracle` is not vacuous.
#[test]
fn abi_layout_wrong_offset_control_fails() {
    let mut oracle = run_c_layout_oracle("layout_oracle_ctrl");
    // Corrupt one C-measured offset by +4.
    let key = ("RecMix".to_string(), "count".to_string());
    let (off, w) = oracle.fields[&key];
    oracle.fields.insert(key, (off + 4, w));
    let defs = owned_defs();
    let table: BTreeMap<String, RecordDefOwned> =
        defs.iter().map(|r| (r.name.clone(), r.clone())).collect();
    let l = layout::layout_record_owned(&table, "RecMix").unwrap();
    let f = l.fields.iter().find(|f| f.name == "count").unwrap();
    let (c_off, _) = oracle.fields[&("RecMix".to_string(), "count".to_string())];
    assert_ne!(f.offset as u64, c_off, "mutated control must not match");
}

/// Declaration order is load-bearing: RecMix vs RecOrderB produce
/// different offsets for same-named fields (both against the C oracle).
#[test]
fn abi_layout_field_order_is_load_bearing() {
    let oracle = run_c_layout_oracle("layout_oracle_order");
    let (mix_flag, _) = oracle.fields[&("RecMix".to_string(), "flag".to_string())];
    let (orderb_flag, _) = oracle.fields[&("RecOrderB".to_string(), "flag".to_string())];
    assert_ne!(
        mix_flag, orderb_flag,
        "same field name, different order => different offset"
    );
}

// ---------------------------------------------------------------------------
// abi_admission_ — controlled layout/bad-frame failures (V-07 share)
// ---------------------------------------------------------------------------

#[test]
fn abi_admission_recursive_record_rejected() {
    let mut table = BTreeMap::new();
    table.insert(
        "RecA".to_string(),
        RecordDefOwned {
            name: "RecA".into(),
            fields: vec![FieldDefOwned {
                name: "b".into(),
                ty: FieldTyOwned::Record("RecB".into()),
            }],
        },
    );
    table.insert(
        "RecB".to_string(),
        RecordDefOwned {
            name: "RecB".into(),
            fields: vec![FieldDefOwned {
                name: "a".into(),
                ty: FieldTyOwned::Record("RecA".into()),
            }],
        },
    );
    let err = layout::layout_record_owned(&table, "RecA").unwrap_err();
    assert!(
        matches!(err, LayoutError::RecursiveRecord(_)),
        "got {err:?}"
    );
    // Self-recursion is the same controlled failure.
    let mut selfy = BTreeMap::new();
    selfy.insert(
        "RecSelf".to_string(),
        RecordDefOwned {
            name: "RecSelf".into(),
            fields: vec![FieldDefOwned {
                name: "next".into(),
                ty: FieldTyOwned::Record("RecSelf".into()),
            }],
        },
    );
    assert!(matches!(
        layout::layout_record_owned(&selfy, "RecSelf"),
        Err(LayoutError::RecursiveRecord(_))
    ));
}

#[test]
fn abi_admission_unknown_record_rejected() {
    let table = BTreeMap::new();
    let err = layout::layout_record_owned(&table, "Nope").unwrap_err();
    assert_eq!(err, LayoutError::UnknownRecord("Nope".into()));
}

#[test]
fn abi_admission_oversize_record_is_resource_rejection() {
    // F-797-R4-10: BOUNDED construction — a fan hierarchy (17 defs)
    // whose flattened size exceeds the 8MiB STAGING budget. The
    // two-phase engine refuses in the SIZE phase (memoized walk over
    // the def table), never materializing the ~million-entry flattened
    // vector, so this test runs in table-space memory, not leaf-space.
    let mut table = BTreeMap::new();
    let scalar = FieldDefOwned {
        name: "b".into(),
        ty: FieldTyOwned::Scalar(SemScalar::I64),
    };
    table.insert(
        "Fan0".to_string(),
        RecordDefOwned {
            name: "Fan0".into(),
            fields: vec![scalar],
        },
    );
    for level in 1..7usize {
        let fields: Vec<FieldDefOwned> = (0..16)
            .map(|_| FieldDefOwned {
                name: format!("f{level}"),
                ty: FieldTyOwned::Record(format!("Fan{}", level - 1)),
            })
            .collect();
        table.insert(
            format!("Fan{level}"),
            RecordDefOwned {
                name: format!("Fan{level}"),
                fields,
            },
        );
    }
    // Fan6 = 16^6 leaves x 8B = 128MiB > 64MiB absolute ceiling — the
    // size phase stops at the first record crossing it.
    match layout::layout_record_owned(&table, "Fan6") {
        Err(LayoutError::RecordTooLarge { .. } | LayoutError::StagingTooLarge { .. }) => {}
        other => panic!("expected a budget rejection, got {other:?}"),
    }
    // Fan5 = 8MiB of payload — exactly at the staging boundary this is
    // 8MiB flattened == the budget cap (allowed); one more fan level is
    // refused before materialization. Build Fan5b with 17 children to
    // cross the cap while staying under the absolute ceiling:
    let mut fields: Vec<FieldDefOwned> = (0..17)
        .map(|_| FieldDefOwned {
            name: "g".into(),
            ty: FieldTyOwned::Record("Fan4".into()),
        })
        .collect();
    fields.push(FieldDefOwned {
        name: "pad".into(),
        ty: FieldTyOwned::Scalar(SemScalar::Bool),
    });
    table.insert(
        "Fan5b".to_string(),
        RecordDefOwned {
            name: "Fan5b".into(),
            fields,
        },
    );
    match layout::layout_record_owned(&table, "Fan5b") {
        Err(LayoutError::StagingTooLarge { record, size }) => {
            assert!(record.contains("Fan"));
            assert!(size > 8 * 1024 * 1024);
        }
        other => panic!("expected StagingTooLarge, got {other:?}"),
    }
}

/// F-797-R4-01: a same-width re-spelling is rejected on the SEMANTIC
/// axis even when producer and consumer agree physically. The attack:
/// both sides carry a u32 `count` field (layouts still match), and the
/// descriptor even re-mints its semantic ref through the REAL contract
/// — but that mint produces a DIFFERENT content version than the
/// consumer's frozen binding, so admission dies before layout talk.
#[test]
fn abi_admission_same_width_respell_is_semantic_reject() {
    use auto_abi_probe::ac_sema_contract::codec::{
        encode_typedesc_bytes, FieldDesc, RecGroup, RecordShape, TypeDesc,
    };
    use auto_abi_probe::ac_sema_contract::identity::{ContentVersion, DeclKind, DeclRef};
    use auto_abi_probe::descriptor::{ModuleDescriptor, SemanticRef};

    let mut d = ModuleDescriptor::probe_module();
    // attacker re-spells count as u32 in the physical record…
    if let Some(rec) = d.records.iter_mut().find(|r| r.name == "RecMix") {
        if let Some(f) = rec.fields.iter_mut().find(|f| f.name == "count") {
            f.ty = FieldTyOwned::Scalar(SemScalar::U32);
        }
    }
    // …and consistently re-mints the semantic ref for the mutated shape
    let canonical = encode_typedesc_bytes(&TypeDesc::I64).unwrap();
    let decl = DeclRef::new(
        &auto_abi_probe::semref::package(),
        auto_abi_probe::semref::BINDING_MODULE,
        DeclKind::Record,
        "RecMix",
        ContentVersion::of_canonical(&canonical),
    );
    let mutated_shape = RecordShape {
        decl,
        fields: vec![
            FieldDesc {
                name: "flag".into(),
                ty: TypeDesc::Bool,
            },
            FieldDesc {
                name: "count".into(),
                ty: TypeDesc::U32,
            },
            FieldDesc {
                name: "tag".into(),
                ty: TypeDesc::Char,
            },
            FieldDesc {
                name: "big".into(),
                ty: TypeDesc::I64,
            },
        ],
    };
    let mutated_version = RecGroup {
        members: vec![mutated_shape],
    }
    .group_content();
    d.semantic_refs = vec![SemanticRef {
        logical: auto_abi_probe::semref::rec_mix_ref()
            .unwrap()
            .logical
            .as_str()
            .into(),
        content_version: mutated_version.as_str().into(),
    }];
    // the consumer expectation also uses the u32 physical record so the
    // LAYOUT comparison alone would pass…
    let mut expect = probe_expectation();
    if let Some(rec) = expect
        .required_records
        .iter_mut()
        .find(|r| r.name == "RecMix")
    {
        if let Some(f) = rec.fields.iter_mut().find(|f| f.name == "count") {
            f.ty = FieldTyOwned::Scalar(SemScalar::U32);
        }
    }
    // …but the SEMANTIC ref axis rejects the mutated content version.
    let want_logical = auto_abi_probe::semref::rec_mix_ref()
        .unwrap()
        .logical
        .as_str()
        .to_string();
    match admit_descriptor(&d, &expect) {
        Err(Rejection::SemanticRefMismatch {
            logical,
            expected_version,
            found_version,
        }) => {
            assert_eq!(logical, want_logical);
            assert_ne!(expected_version, found_version);
        }
        other => panic!("expected SemanticRefMismatch, got {other:?}"),
    }
}

/// F-797-R4-01 (R5): the ORIGINAL R4 attack — payload tampered, label
/// kept ORIGINAL (the lie rides the payload, not the label). Admission
/// now RECOMPUTES the content version from the actual payload and
/// rejects even though the carried label matches the binding.
#[test]
fn abi_admission_tampered_payload_original_label_rejected() {
    let mut d = ModuleDescriptor::probe_module();
    // tamper the payload only: count i32 -> u32
    if let Some(rec) = d.records.iter_mut().find(|r| r.name == "RecMix") {
        if let Some(f) = rec.fields.iter_mut().find(|f| f.name == "count") {
            f.ty = FieldTyOwned::Scalar(SemScalar::U32);
        }
    }
    // semantic_refs stay EXACTLY the binding's originals (the label lies)
    // AND the consumer expectation also physically re-spells count —
    // layout comparison alone would pass; the payload-recompute axis
    // is the only thing that can catch this.
    let mut expect = probe_expectation();
    if let Some(rec) = expect
        .required_records
        .iter_mut()
        .find(|r| r.name == "RecMix")
    {
        if let Some(f) = rec.fields.iter_mut().find(|f| f.name == "count") {
            f.ty = FieldTyOwned::Scalar(SemScalar::U32);
        }
    }
    match admit_descriptor(&d, &expect) {
        Err(Rejection::SemanticRefMismatch { found_version, .. }) => {
            assert!(
                found_version.starts_with("payload-recomputed"),
                "rejection must come from the PAYLOAD axis, got {found_version}"
            );
        }
        other => panic!("expected payload-bound SemanticRefMismatch, got {other:?}"),
    }
}

/// The truthful-label variant (label re-minted for the tampered shape)
/// must ALSO reject — both attack classes exist as permanent cases.
#[test]
fn abi_admission_truthful_label_tampered_payload_also_rejected() {
    let mut d = ModuleDescriptor::probe_module();
    if let Some(rec) = d.records.iter_mut().find(|r| r.name == "RecMix") {
        if let Some(f) = rec.fields.iter_mut().find(|f| f.name == "count") {
            f.ty = FieldTyOwned::Scalar(SemScalar::U32);
        }
    }
    // recompute the truthful label from the tampered payload
    if let Some(rec) = d.records.iter().find(|r| r.name == "RecMix") {
        let fields: Vec<(String, SemScalar)> = rec
            .fields
            .iter()
            .filter_map(|f| match f.ty {
                FieldTyOwned::Scalar(s) => Some((f.name.clone(), s)),
                _ => None,
            })
            .collect();
        let v = auto_abi_probe::semref::content_version_of_payload("RecMix", &fields);
        d.semantic_refs[0].content_version = v;
    }
    match admit_descriptor(&d, &probe_expectation()) {
        Err(Rejection::SemanticRefMismatch { .. }) => {}
        other => panic!("expected SemanticRefMismatch, got {other:?}"),
    }
}

/// F-797-R4-10 (R8): the staging budget is the EXACT representation
/// bytes the materialized layout will own — struct bytes PLUS the real
/// path strings. Short-name fan control: 16^4 = 65,536 leaves
/// materialize well inside 8MiB and the ACTUAL owned bytes (vector
/// capacity + name capacities) equal the predicted bound; one more
/// level (~49MB representation) is refused BEFORE the vector exists.
#[test]
fn abi_admission_staging_gate_counts_representation_bytes() {
    use auto_abi_probe::layout::{FieldDefOwned, FieldTyOwned, RecordDefOwned, SemScalar};
    let mut table = BTreeMap::new();
    let scalar = || FieldDefOwned {
        name: "b".into(),
        ty: FieldTyOwned::Scalar(SemScalar::Bool),
    };
    table.insert(
        "Fan0".to_string(),
        RecordDefOwned {
            name: "Fan0".into(),
            fields: vec![scalar()],
        },
    );
    for level in 1..6usize {
        let fields: Vec<FieldDefOwned> = (0..16)
            .map(|_| FieldDefOwned {
                name: format!("f{level}"),
                ty: FieldTyOwned::Record(format!("Fan{}", level - 1)),
            })
            .collect();
        table.insert(
            format!("Fan{level}"),
            RecordDefOwned {
                name: format!("Fan{level}"),
                fields,
            },
        );
    }
    // Fan4: 65,536 leaves; path "f4.f3.f2.f1.b" = 13 bytes per leaf.
    let ok = layout::layout_record_owned(&table, "Fan4").expect("Fan4 materializes");
    assert_eq!(ok.fields.len(), 65_536);
    let actual = ok.fields.capacity() * std::mem::size_of::<layout::FieldLayout>()
        + ok.fields.iter().map(|f| f.name.capacity()).sum::<usize>();
    let predicted = 65_536usize * (std::mem::size_of::<layout::FieldLayout>() + 13);
    assert!(
        actual <= predicted,
        "actual owned {actual} must not exceed the predicted representation {predicted}"
    );
    assert!(
        (actual as u64) < layout::MAX_STAGING_BYTES,
        "actual owned {actual} within the staging budget"
    );
    assert_eq!(ok.fields[0].name, "f4.f3.f2.f1.b");
    match layout::layout_record_owned(&table, "Fan5") {
        Err(LayoutError::StagingTooLarge { size, .. }) => {
            assert!(
                size > layout::MAX_STAGING_BYTES,
                "size={size} must be representation bytes"
            );
        }
        other => panic!("expected StagingTooLarge, got {other:?}"),
    }
}

/// F-797-R4-10 (R8) — the ORIGINAL R5 counterexample, verbatim: 5
/// record nodes, 4 levels of 16-way branching, 64-BYTE field names,
/// 65,536B physical payload. The R5 gate (leaves × (struct+24)) let it
/// through and the materialized vector+String capacities owned
/// 19,398,656B > 8MiB. The exact gate must REFUSE it pre-allocation —
/// path strings are first-class budget items, and the refusal is a
/// controlled RESOURCE state, not an ABI rejection.
#[test]
fn abi_admission_staging_gate_original_long_name_input_refused() {
    let mut table = BTreeMap::new();
    table.insert(
        "F0".to_string(),
        RecordDefOwned {
            name: "F0".into(),
            fields: vec![FieldDefOwned {
                name: "leaf".into(),
                ty: FieldTyOwned::Scalar(SemScalar::U8),
            }],
        },
    );
    for level in 1..=4usize {
        table.insert(
            format!("F{level}"),
            RecordDefOwned {
                name: format!("F{level}"),
                fields: (0..16)
                    .map(|i| FieldDefOwned {
                        name: format!("f{i:02}_{}", "x".repeat(60)),
                        ty: FieldTyOwned::Record(format!("F{}", level - 1)),
                    })
                    .collect(),
            },
        );
    }
    match layout::layout_record_owned(&table, "F4") {
        Ok(l) => panic!(
            "the original long-name input must be refused pre-allocation; got {} fields",
            l.fields.len()
        ),
        Err(LayoutError::StagingTooLarge { size, .. }) => {
            // per leaf: 32B struct + 264B path (4 × (64+1) + len("leaf"));
            // the R8-measured owned lower bound was 19,398,656B
            assert!(
                size >= 19_398_656,
                "predicted representation {size} must cover the measured 19,398,656B lower bound"
            );
            assert!(size > layout::MAX_STAGING_BYTES);
        }
        other => panic!("expected StagingTooLarge (resource), got {other:?}"),
    }
}

/// F-797-R4-01: a descriptor MISSING the bound semantic ref (the record
/// vanished) is rejected even when no layout is demanded.
#[test]
fn abi_admission_missing_semantic_ref_rejected() {
    let mut d = ModuleDescriptor::probe_module();
    d.semantic_refs.clear();
    let want_logical = auto_abi_probe::semref::rec_mix_ref()
        .unwrap()
        .logical
        .as_str()
        .to_string();
    match admit_descriptor(&d, &probe_expectation()) {
        Err(Rejection::MissingSemanticRef { logical }) => {
            assert_eq!(logical, want_logical);
        }
        other => panic!("expected MissingSemanticRef, got {other:?}"),
    }
}

#[test]
fn abi_admission_deep_chain_bounded_and_working() {
    // Chain depth 4096 must WORK (plan §5.5: at least 4096); one more is
    // a controlled rejection.
    fn chain(depth: usize) -> BTreeMap<String, RecordDefOwned> {
        let mut t = BTreeMap::new();
        for i in 0..depth {
            let ty = if i + 1 < depth {
                FieldTyOwned::Record(format!("R{}", i + 1))
            } else {
                FieldTyOwned::Scalar(SemScalar::I64)
            };
            t.insert(
                format!("R{i}"),
                RecordDefOwned {
                    name: format!("R{i}"),
                    fields: vec![FieldDefOwned {
                        name: "next".into(),
                        ty,
                    }],
                },
            );
        }
        t
    }
    // chain(n) lays out records R0..R(n-1); the deepest record sits at
    // recursion depth n-1 and the guard rejects depth > MAX. So a chain
    // of MAX+1 records still computes (depth MAX), and MAX+2 records
    // (depth MAX+1) is the first controlled rejection.
    let ok_table = chain(MAX_NESTING_DEPTH + 1);
    let l = layout::layout_record_owned(&ok_table, "R0").expect("4096-deep chain computes");
    assert_eq!(l.align, 8);
    assert!(!l.fields.is_empty());

    let deep_table = chain(MAX_NESTING_DEPTH + 2);
    match layout::layout_record_owned(&deep_table, "R0") {
        Err(LayoutError::NestingTooDeep(d)) => assert!(d > MAX_NESTING_DEPTH),
        other => panic!("expected NestingTooDeep, got {other:?}"),
    }
}

/// V-01 semref dimension end-to-end: a descriptor built against a
/// different semantic-ref snapshot cannot admit (its semantic inputs
/// are not what the consumer validated against).
#[test]
fn abi_admission_semref_gate_end_to_end() {
    let mut d = ModuleDescriptor::probe_module();
    let expect = probe_expectation();
    assert!(admit_descriptor(&d, &expect).is_ok());
    d.semref_hash = auto_abi_probe::semref::blake3_hex(b"other-snapshot");
    match admit_descriptor(&d, &expect) {
        Err(Rejection::SemRefMismatch { expected, found }) => {
            assert_eq!(expected, auto_abi_probe::semref::binding_hash());
            assert_ne!(expected, found);
        }
        other => panic!("expected SemRefMismatch, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// F-797-R4-02 promised-surface closures
// ---------------------------------------------------------------------------

/// F-797-R4-02 (R5): the PRODUCTION builder describes record views as
/// record views — `descriptor_for_defs(&[rec_sum])` must not collapse
/// the pointer lane to i64.
#[test]
fn abi_contract_production_builder_spells_recview() {
    use auto_abi_probe::decl::{AbiScalar, BodyKind, FnDecl};
    let defs = [FnDecl {
        symbol: "p797_rec_sum",
        params: &[AbiScalar::Ptr],
        ret: AbiScalar::I64,
        body: BodyKind::RecSumV1,
    }];
    let d = auto_abi_probe::build::descriptor_for_defs(&defs);
    let sig = d
        .symbols
        .iter()
        .find(|s| s.name == "p797_rec_sum")
        .unwrap()
        .form
        .signature();
    assert_eq!(sig, "pure(rec:RecMix:const)->i64");
}

/// The fallible form SPELLS its explicit context parameter — a consumer
/// compiling against the proposal signature can derive the real machine
/// shape from the descriptor alone.
#[test]
fn abi_contract_fallible_form_spells_ctx() {
    let defs = [mix_def_div()];
    let d = auto_abi_probe::build::descriptor_for_defs(&defs);
    let sig = d
        .symbols
        .iter()
        .find(|s| s.name == "p797_div")
        .unwrap()
        .form
        .signature();
    assert!(
        sig.starts_with("fallible(ctx:nullable,i64,i64)->status,out"),
        "got {sig:?}"
    );
}

fn mix_def_div() -> auto_abi_probe::decl::FnDecl {
    use auto_abi_probe::decl::{AbiScalar, BodyKind, FnDecl};
    FnDecl {
        symbol: "p797_div",
        params: &[
            AbiScalar::Ptr,
            AbiScalar::I64,
            AbiScalar::I64,
            AbiScalar::Ptr,
        ],
        ret: AbiScalar::I32,
        body: BodyKind::FallibleDivV1,
    }
}

/// F-797-R4-02 (R8): the C provider descriptor must mirror the
/// hand-audited HEADER, item by item. The oracle here is the header
/// TEXT itself — the declared-prototype NAME SET is parsed
/// mechanically from `cside/interop_resource.h` (so the descriptor can
/// neither drop nor invent public entries), and the expected PHYSICAL
/// signature of each prototype is hand-derived from that header (the
/// table below cites each line; it is written from the header, never
/// from `interop_c_provider_descriptor`). A descriptor whose own
/// signature were fed back as the expectation could not catch a
/// mis-description; this can and did (ledger 3-out, two_step
/// handle-out, fold callback ctx / element count).
#[test]
fn abi_contract_c_provider_descriptor_matches_independent_header() {
    use auto_abi_probe::descriptor::{interop_c_provider_descriptor, ModuleDescriptor};
    let header = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cside/interop_resource.h"),
    )
    .expect("hand-audited header readable");

    // Mechanical half: the declared public-prototype name set.
    let mut declared: Vec<String> = Vec::new();
    for line in header.lines() {
        let t = line.trim();
        if t.starts_with('*') || t.starts_with("/*") || t.starts_with("//") {
            continue;
        }
        let Some(at) = t.find("p797_") else { continue };
        let Some(paren) = t[at..].find('(') else {
            continue;
        };
        let name = t[at..at + paren].trim().to_string();
        if !name.is_empty() && !declared.contains(&name) {
            declared.push(name);
        }
    }
    declared.sort();

    // Hand-derived half (from the header text, one row per prototype):
    //   long long p797_h_create(int size)                       — owned handle BY VALUE
    //   int p797_h_set(long long h, int idx, long long v)       — status-only
    //   int p797_h_get(long long h, int idx, long long *out)
    //   int p797_h_destroy(long long h)                         — status-only
    //   int p797_ledger(long long *live, long long *allocs, long long *frees) — THREE outs
    //   int p797_buf_sum(const unsigned char *p, uint64_t len, long long *out)
    //   int p797_buf_copy(unsigned char *dst, uint64_t cap, const unsigned char *src, uint64_t len)
    //   void p797_fail_nth_alloc(int n)
    //   int p797_two_step(int x, long long *out)                — out carries an OWNED handle
    //   long long p797_fold(long long (*cb)(long long ctx, long long item),
    //                       long long ctx, const long long *items, uint64_t n) — cb ctx + ELEMENT count
    //   long long p797_stored_cb(void)
    //   int p797_cbuf_sum_utf8(const unsigned char *p, uint64_t len, long long *chars)
    //   int p797_crec_valid(const RecMix *r, long long *sum)
    //   int p797_crec_mutate(RecMix *r, long long seed, long long *sum) — MUT view
    //   int p797_crec_make(int seed, RecMix *out)               — caller-owned OUT record
    let expected: &[(&str, &str)] = &[
        ("p797_h_create", "pure(i32)->handle:interop-resource"),
        (
            "p797_h_set",
            "fallible(handle:interop-resource,i32,i64)->status",
        ),
        (
            "p797_h_get",
            "fallible(handle:interop-resource,i32)->status,out i64",
        ),
        (
            "p797_h_destroy",
            "fallible(handle:interop-resource)->status",
        ),
        ("p797_ledger", "fallible()->status,out i64,i64,i64"),
        ("p797_buf_sum", "fallible(buf:bytes,u64)->status,out i64"),
        (
            "p797_buf_copy",
            "fallible(buf:out,u64,buf:bytes,u64)->status",
        ),
        ("p797_fail_nth_alloc", "pure(i32)->unit"),
        (
            "p797_two_step",
            "fallible(i32)->status,out handle:interop-resource",
        ),
        (
            "p797_fold",
            "pure(callback(ctx:i64,i64)->i64,i64,buf:i64,u64)->i64",
        ),
        ("p797_stored_cb", "pure()->i64"),
        (
            "p797_cbuf_sum_utf8",
            "fallible(buf:utf8,u64)->status,out i64",
        ),
        (
            "p797_crec_valid",
            "fallible(rec:RecMix:const)->status,out i64",
        ),
        (
            "p797_crec_mutate",
            "fallible(rec:RecMix:mut,i64)->status,out i64",
        ),
        ("p797_crec_make", "fallible(i32)->status,out rec:RecMix:out"),
    ];
    let mut table_names: Vec<&str> = expected.iter().map(|(n, _)| *n).collect();
    table_names.sort_unstable();
    assert_eq!(
        declared, table_names,
        "hand-derived table and header prototype set must match exactly"
    );

    let d = interop_c_provider_descriptor();
    let bytes = d.encode().unwrap();
    let back = ModuleDescriptor::decode(&bytes).unwrap();
    assert_eq!(d, back, "descriptor/3 roundtrip");
    let mut described: Vec<(String, String)> = d
        .symbols
        .iter()
        .map(|s| (s.name.clone(), s.form.signature()))
        .collect();
    described.sort();
    let mut expected_sorted: Vec<(String, String)> = expected
        .iter()
        .map(|(n, s)| (n.to_string(), s.to_string()))
        .collect();
    expected_sorted.sort();
    assert_eq!(
        described, expected_sorted,
        "every descriptor signature must equal the header-derived physical form"
    );

    // Ownership semantics the signature alone cannot carry (hand audit):
    // the out-handle of p797_two_step names the SAME provider whose
    // p797_h_destroy accepts it — the destroy lane a consumer needs.
    let two_step_out = d
        .symbols
        .iter()
        .find(|s| s.name == "p797_two_step")
        .and_then(|s| match &s.form {
            SymbolForm::Fallible { outs, .. } => outs.first().cloned(),
            _ => None,
        })
        .expect("two_step spells an out slot");
    assert_eq!(
        two_step_out,
        ParamKind::Handle {
            provider: "interop-resource".into()
        }
    );
    let destroys_handle = d.symbols.iter().any(|s| {
        s.name == "p797_h_destroy"
            && matches!(
                &s.form,
                SymbolForm::Fallible { params, .. }
                    if params.first() == Some(&ParamKind::Handle {
                        provider: "interop-resource".into()
                    })
            )
    });
    assert!(
        destroys_handle,
        "the provider of the two_step out-handle must also expose its destroy entry"
    );

    // Admissible: a consumer requiring EXACTLY these header-derived
    // signatures passes admission (the consumability half).
    let mut expect = probe_expectation();
    expect.required_symbols = described.clone();
    admit_descriptor(&d, &expect).expect("C provider descriptor admits");
}

/// A capability the profile DOES NOT offer (storing callbacks) is not
/// just undocumented — REQUIRING it is an admission rejection with
/// native call count 0 (F-797-R4-02: rejecting the capability is an
/// entrance property, not a doc footnote).
#[test]
fn abi_contract_absent_capability_symbol_rejected() {
    let d = ModuleDescriptor::probe_module();
    let mut expect = probe_expectation();
    expect
        .required_symbols
        .push(("p797_fold_save".into(), "pure(handle)->i64".into()));
    match admit_descriptor(&d, &expect) {
        Err(Rejection::MissingSymbol { symbol }) => assert_eq!(symbol, "p797_fold_save"),
        other => panic!("expected MissingSymbol, got {other:?}"),
    }
}

/// F-797-R4-10 (R9): the memo/index and recursion working state are
/// budget items too — the R9 counterexample is a LEGAL acyclic
/// 1024-node chain whose RECORD names are 1950 bytes each (~4MiB of
/// table input, a single-field output): the materialized representation
/// is tiny but the four memo maps + chain/seen name copies exceeded
/// 8MiB of live staging. The gate must refuse it BEFORE the state grows
/// (a controlled resource rejection); a short-name chain of the same
/// shape computes fine.
#[test]
fn abi_admission_index_budget_counts_memo_state() {
    fn chain_table(name_len: usize) -> BTreeMap<String, RecordDefOwned> {
        let mk_name = |i: usize| -> String {
            if name_len <= 8 {
                format!("S{i}")
            } else {
                let base = format!("S{i}-");
                format!("{base}{}", "x".repeat(name_len - base.len()))
            }
        };
        let mut t = BTreeMap::new();
        for i in 0..1024usize {
            let ty = if i + 1 < 1024 {
                FieldTyOwned::Record(mk_name(i + 1))
            } else {
                FieldTyOwned::Scalar(SemScalar::I64)
            };
            t.insert(
                mk_name(i),
                RecordDefOwned {
                    name: mk_name(i),
                    fields: vec![FieldDefOwned {
                        name: "next".into(),
                        ty,
                    }],
                },
            );
        }
        t
    }
    let short = chain_table(6);
    let short_first = short.keys().next().cloned().unwrap();
    // short names: index state is tiny — computes (the legal control)
    let ok = layout::layout_record_owned(&short, &short_first).expect("short-name chain computes");
    // a single-field chain: ONE leaf (the R9 shape — "只有 one field"),
    // reached through 1024 records
    assert_eq!(ok.fields.len(), 1);
    assert_eq!(ok.fields[0].name.matches('.').count(), 1023);
    // 1950-byte record names: the memo/index estimate crosses the 8MiB
    // staging budget — refused pre-allocation as a RESOURCE state
    let long = chain_table(1950);
    let long_first = long.keys().next().cloned().unwrap();
    match layout::layout_record_owned(&long, &long_first) {
        Ok(l) => panic!(
            "the R9 long-record-name chain must be refused (index/memo budget); got {} fields",
            l.fields.len()
        ),
        Err(LayoutError::StagingTooLarge { size, .. }) => {
            assert!(size > layout::MAX_STAGING_BYTES);
        }
        other => panic!("expected StagingTooLarge (resource), got {other:?}"),
    }
}

/// F-797-R4-10 (R10): a TINY table with one long record name must NOT be
/// refused — the R9 post-hoc estimate charged every map entry at the
/// maximum name length times the maximum depth, rejecting a one-record
/// 2000-byte-name graph whose actual peak is ~11KB. The incremental
/// accounting charges only what the walk actually uses.
#[test]
fn abi_admission_tiny_graph_with_long_name_computes() {
    let name = format!("S-{}", "x".repeat(1990));
    let mut table = BTreeMap::new();
    table.insert(
        name.clone(),
        RecordDefOwned {
            name: name.clone(),
            fields: vec![FieldDefOwned {
                name: "leaf".into(),
                ty: FieldTyOwned::Scalar(SemScalar::U8),
            }],
        },
    );
    let l = layout::layout_record_owned(&table, &name)
        .expect("a one-record graph with a long name computes (actual cost ~KBs)");
    assert_eq!(l.fields.len(), 1);
    assert_eq!(l.fields[0].name, "leaf");
    // a mixed table — many short names plus one long name — also pays
    // only its actual usage
    let mut mixed = BTreeMap::new();
    mixed.insert(
        name.clone(),
        RecordDefOwned {
            name: name.clone(),
            fields: vec![FieldDefOwned {
                name: "next".into(),
                ty: FieldTyOwned::Record("S0".into()),
            }],
        },
    );
    for i in 0..1000usize {
        let ty = if i + 1 < 1000 {
            FieldTyOwned::Record(format!("S{}", i + 1))
        } else {
            FieldTyOwned::Scalar(SemScalar::I64)
        };
        mixed.insert(
            format!("S{i}"),
            RecordDefOwned {
                name: format!("S{i}"),
                fields: vec![FieldDefOwned {
                    name: "next".into(),
                    ty,
                }],
            },
        );
    }
    let ok = layout::layout_record_owned(&mixed, &name).expect("mixed table computes");
    assert_eq!(ok.fields.len(), 1, "single-field chain: one leaf");
}
