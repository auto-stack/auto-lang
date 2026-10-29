//! V-03 container clone starts at untrusted typed MIR, not invented source syntax.
mod common;
use auto_av_core::mir::*;

fn fixture(clone_variant: bool) -> MirModule {
    let owner = OwnerScope {
        schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.into(),
        module: "collections889".into(),
    };
    let mut m = MirModule::new_collections(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Bool,
        MirType::Str,
        MirType::Record(RecordTypeRef {
            owner_module: owner.module.clone(),
            name: "Item".into(),
            fields: vec![
                MirFieldDef {
                    name: "name".into(),
                    ty: 2,
                },
                MirFieldDef {
                    name: "code".into(),
                    ty: 0,
                },
            ],
        }),
        MirType::Variant(VariantTypeRef {
            owner_module: owner.module.clone(),
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
        MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        }),
        MirType::StrView,
    ];
    m.consts = vec![
        MirConst::Str("中".as_bytes().into()),
        MirConst::I32(41),
        MirConst::I32(0),
    ];
    let slot_types = [
        2, 0, 3, 4, 5, 5, 0, 4, 1, 3, 2, 7, 0, 0, 0, 0, 0, 4, 1, 3, 2, 7, 0, 0, 0, 0, 4,
    ];
    let slots = slot_types
        .iter()
        .map(|ty| MirSlot {
            ty: *ty,
            kind: SlotKind::Temp,
            name: None,
        })
        .collect();
    use MirInstr::*;
    let mut first = vec![
        Const { dst: 0, value: 0 },
        Const { dst: 1, value: 1 },
        Aggregate {
            dst: 2,
            record_ty: 3,
            operands: vec![0, 1],
        },
        VariantPack {
            dst: 3,
            variant_ty: 4,
            disc: 0,
            payload: Some(2),
        },
    ];
    if clone_variant {
        first.push(Clone { dst: 26, src: 3 });
    }
    first.extend([
        ListNew {
            dst: 4,
            list_ty: 5,
            cap: 1,
        },
        ListPush {
            root: 4,
            value: if clone_variant { 26 } else { 3 },
        },
    ]);
    if clone_variant {
        first.push(Drop { slot: 3 });
    }
    first.extend([
        Clone { dst: 5, src: 4 },
        Const { dst: 6, value: 2 },
        ListRemove {
            dst: 7,
            root: 5,
            index: 6,
        },
        VariantGuard {
            dst: 8,
            src: 7,
            disc: 0,
        },
    ]);
    let second = vec![
        VariantProject {
            dst: 9,
            root: 7,
            disc: 0,
        },
        FieldGet {
            dst: 10,
            base: 9,
            path: vec![0],
        },
        View { dst: 11, src: 10 },
        StrLen { dst: 12, view: 11 },
        BorrowEnd { token: 11 },
        FieldGet {
            dst: 13,
            base: 9,
            path: vec![1],
        },
        Bin {
            op: BinOp::AddI32,
            dst: 14,
            lhs: 12,
            rhs: 13,
        },
        Drop { slot: 9 },
        Drop { slot: 10 },
        ListRemove {
            dst: 17,
            root: 4,
            index: 6,
        },
        VariantGuard {
            dst: 18,
            src: 17,
            disc: 0,
        },
    ];
    let last = vec![
        VariantProject {
            dst: 19,
            root: 17,
            disc: 0,
        },
        FieldGet {
            dst: 20,
            base: 19,
            path: vec![0],
        },
        View { dst: 21, src: 20 },
        StrLen { dst: 22, view: 21 },
        BorrowEnd { token: 21 },
        FieldGet {
            dst: 23,
            base: 19,
            path: vec![1],
        },
        Bin {
            op: BinOp::AddI32,
            dst: 24,
            lhs: 22,
            rhs: 23,
        },
        Bin {
            op: BinOp::AddI32,
            dst: 25,
            lhs: 14,
            rhs: 24,
        },
        Drop { slot: 19 },
        Drop { slot: 20 },
    ];
    m.funcs.push(MirFunction {
        owner,
        name: "d_main".into(),
        sig: 6,
        slots,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![
            MirBlock {
                instrs: first,
                term: Terminator::Branch {
                    cond: 8,
                    then: 1,
                    else_: 2,
                },
            },
            MirBlock {
                instrs: second,
                term: Terminator::Branch {
                    cond: 18,
                    then: 3,
                    else_: 2,
                },
            },
            MirBlock {
                instrs: vec![Const { dst: 16, value: 2 }],
                term: Terminator::Return { value: 16 },
            },
            MirBlock {
                instrs: last,
                term: Terminator::Return { value: 25 },
            },
        ],
    });
    m
}

#[test]
fn typed_list_and_variant_clone_preserve_independent_owned_payloads() {
    for (root_variant, text_allocs) in [(false, 2), (true, 3)] {
        let m = fixture(root_variant);
        // The builder itself is not trusted; this is the same verifier used by codegen.
        auto_av_core::verify_mir(&m).unwrap();
        let run = common::collections::run_module(
            &m,
            if root_variant {
                "typed-variant-list-clone"
            } else {
                "typed-list-clone"
            },
            None,
        );
        assert_eq!(
            run.exit_code,
            Some(84),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        common::collections::assert_balanced(&run.stderr);
        let text = String::from_utf8_lossy(&run.stderr);
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("CLONE id="))
                .count(),
            text_allocs - 1
        );
        assert!(
            text.contains(&format!("allocs={text_allocs} frees={text_allocs} live=0")),
            "{text}"
        );
    }
}

#[test]
fn typed_wrong_list_element_rejected_before_object_production() {
    let mut m = fixture(false);
    m.is_unit = true;
    ac_mn::collections_lower::admit_unit(&m).expect("unchanged unit is a valid control");
    m.types[5] = MirType::List(ListTypeRef { elem: 0 });
    let error = ac_mn::collections_lower::emit_unit_coff(&m).unwrap_err();
    assert!(matches!(error, ac_mn::MnError::Admit("mir", _)), "{error}");
    assert!(error.message().contains("mir.list-type"), "{error}");
}
