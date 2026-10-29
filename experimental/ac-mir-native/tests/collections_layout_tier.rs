mod common;
use ac_mn::collections_layout::{closure_fingerprint, descriptors};
use auto_av_core::mir::*;

fn types() -> MirModule {
    let mut m = MirModule::new_collections(OwnerScope {
        schema: "collections889".into(),
        module: "layout".into(),
    });
    let r = RecordTypeRef {
        owner_module: "owner".into(),
        name: "Item".into(),
        fields: vec![
            MirFieldDef {
                name: "text".into(),
                ty: 2,
            },
            MirFieldDef {
                name: "code".into(),
                ty: 0,
            },
        ],
    };
    m.types = vec![
        MirType::I32,
        MirType::Bool,
        MirType::Str,
        MirType::Record(r.clone()),
        MirType::Variant(VariantTypeRef {
            owner_module: "owner".into(),
            name: "Token".into(),
            arms: vec![
                VariantArm {
                    name: "Data".into(),
                    payload: Some(3),
                },
                VariantArm {
                    name: "End".into(),
                    payload: None,
                },
            ],
        }),
        MirType::List(ListTypeRef { elem: 4 }),
        MirType::RecordView(r),
        MirType::StrView,
        MirType::Fn(FnSig {
            params: vec![6],
            result: 4,
        }),
    ];
    m
}
#[test]
fn layouts_preserve_nominal_graph_and_ignore_index_remapping() {
    let m = types();
    let original = descriptors(&m).unwrap();
    assert_eq!(original[2].fingerprint, original[7].fingerprint);
    assert_eq!(original[3].fingerprint, original[6].fingerprint);
    assert_ne!(original[2].borrowed, original[7].borrowed);
    let mut mode = m.clone();
    if let MirType::Fn(s) = &mut mode.types[8] {
        s.params[0] = 3;
    }
    let mode = descriptors(&mode).unwrap();
    assert_ne!(
        mode[8].fingerprint, original[8].fingerprint,
        "borrow/own signature modes must differ"
    );
    let mut remapped = m.clone();
    let n = m.types.len();
    remapped.types.reverse();
    let remap = |i: &mut TypeIdx| *i = (n - 1 - *i as usize) as TypeIdx;
    for t in &mut remapped.types {
        match t {
            MirType::Record(r) | MirType::RecordView(r) => {
                for f in &mut r.fields {
                    remap(&mut f.ty)
                }
            }
            MirType::List(l) => remap(&mut l.elem),
            MirType::Variant(v) => {
                for a in &mut v.arms {
                    if let Some(t) = &mut a.payload {
                        remap(t)
                    }
                }
            }
            MirType::Fn(s) => {
                for p in &mut s.params {
                    remap(p)
                }
                remap(&mut s.result)
            }
            _ => {}
        }
    }
    let reverse = descriptors(&remapped).unwrap();
    for i in 0..n {
        assert_eq!(original[i], reverse[n - 1 - i]);
    }
    assert_eq!(
        closure_fingerprint(&original),
        closure_fingerprint(&reverse)
    );
    for mutation in 0..5 {
        let mut changed = m.clone();
        match mutation {
            0 => {
                if let MirType::Record(r) = &mut changed.types[3] {
                    r.name.push('X')
                }
            }
            1 => {
                if let MirType::Record(r) = &mut changed.types[3] {
                    r.owner_module.push('X')
                }
            }
            2 => {
                if let MirType::Record(r) = &mut changed.types[3] {
                    r.fields[0].name.push('X')
                }
            }
            3 => {
                if let MirType::Variant(v) = &mut changed.types[4] {
                    v.arms.swap(0, 1)
                }
            }
            _ => {
                if let MirType::Variant(v) = &mut changed.types[4] {
                    v.arms[0].payload = Some(0)
                }
            }
        }
        let new = descriptors(&changed).unwrap();
        assert_ne!(original[4].fingerprint, new[4].fingerprint);
        assert_ne!(original[5].fingerprint, new[5].fingerprint);
    }
}
#[test]
fn layout_deep_acyclic_walk_and_invalid_graph_controls() {
    let mut m = types();
    m.types.truncate(3);
    for i in 0..4096 {
        m.types.push(MirType::Variant(VariantTypeRef {
            owner_module: "deep".into(),
            name: format!("V{i}"),
            arms: vec![VariantArm {
                name: "Next".into(),
                payload: Some((m.types.len() - 1) as u32),
            }],
        }));
    }
    std::thread::Builder::new()
        .stack_size(65536)
        .spawn(move || assert_eq!(descriptors(&m).unwrap().len(), 4099))
        .unwrap()
        .join()
        .unwrap();
    let mut bad = types();
    bad.types[5] = MirType::List(ListTypeRef { elem: 5 });
    assert!(descriptors(&bad).unwrap_err().message().contains("cyclic"));
    bad.types[5] = MirType::List(ListTypeRef { elem: 999 });
    assert!(descriptors(&bad)
        .unwrap_err()
        .message()
        .contains("reference"));
}
#[test]
fn independently_valid_unit_with_wrong_nominal_return_traps_before_consumption() {
    let source = "type Item { name Str; code int }\nfn make() Item { return Item(\"中\", 41) }\nfn main() int { let p = make()\n return p.name.len() + p.code }";
    let compiled =
        auto_ac_prototype::source::compile_collections_source(source, "layout.at", &[]).unwrap();
    let m = auto_mir_lower::lower_checked_collections(&compiled.checked, "d_main", 0).unwrap();
    let positive = common::collections::run_module(&m, "layout-return-positive", Some(source));
    assert_eq!(
        positive.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&positive.stderr)
    );
    common::collections::assert_balanced(&positive);
    let mut changed = 0;
    let negative =
        common::collections::run_module_with_units(&m, "layout-return-wrong-unit", None, |unit| {
            if unit.funcs[0].name == "d_make" {
                for ty in &mut unit.types {
                    if let MirType::Record(r) | MirType::RecordView(r) = ty {
                        r.name.push_str("Wrong");
                        changed += 1;
                    }
                }
            }
        });
    assert!(changed > 0, "mutation did not reach the unit");
    assert_eq!(
        negative.exit_code,
        Some(70),
        "{}",
        String::from_utf8_lossy(&negative.stderr)
    );
    let text = String::from_utf8_lossy(&negative.stderr);
    assert!(
        text.contains("MN-COLL-TYPE refusal=4\n") && text.contains("MN-COLL-TRAP code=23\n"),
        "{text}"
    );
    common::collections::assert_balanced(&negative);
}
