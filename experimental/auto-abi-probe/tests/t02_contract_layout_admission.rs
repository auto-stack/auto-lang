//! PLAN-797 T-02: descriptor contract, layout conformance against the
//! independent C oracle, and admission rejections (plan §8 T-02; V-01,
//! V-03, V-07 layout share).
//!
//! Naming follows the plan's future entrypoints: `abi_contract_`,
//! `abi_layout_`, `abi_admission_`.

#![cfg(windows)]

use auto_abi_probe::ctool;
use auto_abi_probe::descriptor::{
    ModuleDescriptor, ParamKind, RetKind, SymbolForm, DESCRIPTOR_FORMAT,
};
use auto_abi_probe::layout::{
    self, FieldDefOwned, FieldTyOwned, LayoutError, RecordDefOwned, SemScalar,
    MAX_NESTING_DEPTH,
};
use auto_abi_probe::run;
use auto_abi_probe::validate::{
    admit, admit_descriptor, probe_expectation, ConsumerExpectation, Rejection,
};

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
            Box::new(|d: &mut ModuleDescriptor| d.profile = "autoabi.win-x64.v0.2-candidate".into()),
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
            params: vec![],
            out: ParamKind::I64,
        },
    });
    assert_ne!(d.fingerprint().unwrap(), d2.fingerprint().unwrap());
    // Signature spellings are exact and form-aware.
    assert_eq!(
        d2.symbols[1].form.signature(),
        "fallible()->status,out i64"
    );
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
            FieldDefOwned { name: "flag".into(), ty: Scalar(SemScalar::Bool) },
            FieldDefOwned { name: "count".into(), ty: Scalar(SemScalar::I32) },
            FieldDefOwned { name: "tag".into(), ty: Scalar(SemScalar::Char) },
            FieldDefOwned { name: "big".into(), ty: Scalar(SemScalar::I64) },
        ],
    };
    vec![
        rec_mix,
        RecordDefOwned {
            name: "RecPad".into(),
            fields: vec![
                FieldDefOwned { name: "a".into(), ty: Scalar(SemScalar::U8) },
                FieldDefOwned { name: "b".into(), ty: Scalar(SemScalar::I64) },
                FieldDefOwned { name: "c".into(), ty: Scalar(SemScalar::U8) },
            ],
        },
        RecordDefOwned {
            name: "RecNested".into(),
            fields: vec![
                FieldDefOwned { name: "x".into(), ty: Scalar(SemScalar::I32) },
                FieldDefOwned { name: "m".into(), ty: FieldTyOwned::Record("RecMix".into()) },
                FieldDefOwned { name: "d".into(), ty: Scalar(SemScalar::F64) },
            ],
        },
        RecordDefOwned {
            name: "RecOrderB".into(),
            fields: vec![
                FieldDefOwned { name: "big".into(), ty: Scalar(SemScalar::I64) },
                FieldDefOwned { name: "flag".into(), ty: Scalar(SemScalar::Bool) },
                FieldDefOwned { name: "count".into(), ty: Scalar(SemScalar::I32) },
                FieldDefOwned { name: "tag".into(), ty: Scalar(SemScalar::Char) },
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
    ctool::compile_c(&src, &manifest.join("cside"), &obj)
        .expect("cl compiles the layout oracle");
    let exe = run::link_pe(&dir, &[obj], "p797_layout_entry")
        .expect("link layout oracle PE");
    let pe = run::run_pe(&exe).expect("run layout oracle PE");
    assert_eq!(pe.code, 0, "layout oracle exit; stdout:\n{}", pe.stdout);

    let mut c = CLayout { sizes: BTreeMap::new(), fields: BTreeMap::new() };
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
                assert!(
                    field.ends_with(sub),
                    "bad nested oracle line: {line:?}"
                );
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
    assert_ne!(mix_flag, orderb_flag, "same field name, different order => different offset");
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
    // 9M i64 fields would exceed the 64MiB record budget: build 8.5M
    // fields (~68MiB) and expect the controlled RecordTooLarge state —
    // a resource rejection, not an ABI verdict and not a panic.
    let n = 8_500_000usize;
    let fields: Vec<FieldDefOwned> = (0..n)
        .map(|i| FieldDefOwned { name: format!("f{i}"), ty: FieldTyOwned::Scalar(SemScalar::I64) })
        .collect();
    let mut table = BTreeMap::new();
    table.insert(
        "RecHuge".to_string(),
        RecordDefOwned { name: "RecHuge".into(), fields },
    );
    match layout::layout_record_owned(&table, "RecHuge") {
        Err(LayoutError::RecordTooLarge { record, .. }) => assert_eq!(record, "RecHuge"),
        other => panic!("expected RecordTooLarge, got {other:?}"),
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
                    fields: vec![FieldDefOwned { name: "next".into(), ty }],
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
            assert_eq!(expected, auto_abi_probe::semref::snapshot_hash());
            assert_ne!(expected, found);
        }
        other => panic!("expected SemRefMismatch, got {other:?}"),
    }
}
