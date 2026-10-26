//! PLAN-824 T-10 harness crate root — the same-source seam.
//!
//! `dynamic` and `portable` are compiled from the PRODUCTION files
//! `experimental/ac-av-probe/core/src/{dynamic,portable}.rs` (relative
//! #[path] below; the files are not declared in core's lib.rs, so the core
//! crate itself is untouched). Inside those files, `crate::contract`
//! resolves through the re-export of the delivered auto-av-core public
//! module below — after the T-02/T-03 core window adds `mod dynamic;` and
//! `mod portable;` to core's lib.rs, the same files compile unchanged
//! against core's own modules (single source, no copy, no cycle).
//!
//! Test families (contract-decision.md §9, V-14 module axis):
//! `av_dynamic_824_portable_*` (freeze/restore equivalence),
//! `av_dynamic_824_take_*` (checked access), `av_dynamic_824_rollback_*`
//! (failure keeps source / clean retry), `av_dynamic_824_reject_*`
//! (recursive whitelist), `av_dynamic_824_wire_*` (wire negatives),
//! `av_dynamic_824_budget_*` (budgets/cancel).

pub use auto_av_core::{contract, owned_text};

#[path = "../../../core/src/dynamic.rs"]
pub mod dynamic;
#[path = "../../../core/src/portable.rs"]
pub mod portable;

#[cfg(test)]
mod tests {
    use crate::contract::{
        fnv1a64, ListHandle, RecordValue, StrHandle, StrView, StrViewSrc, Value, VariantValue,
    };
    use crate::dynamic::{
        DynPackError, DynTakeError, DynType, DynValue, TraversalOptions, TraversalStop,
        MAX_DYN_NODES,
    };
    use crate::owned_text::StrArena;
    use crate::portable::{
        freeze, unfreeze, ArmSig, PortableError, RecordBinding, RestoreContext, VariantBinding,
        PORTABLE_MAGIC,
    };
    use std::cell::Cell;
    use std::rc::Rc;

    // ------------------------------------------------------------------
    // helpers
    // ------------------------------------------------------------------

    fn record(ty: u32, fields: Vec<Value>) -> Value {
        Value::Record(Rc::new(RecordValue { ty, fields }))
    }

    fn strv(s: &str) -> Value {
        Value::Str(StrHandle::detached(
            s.as_bytes().to_vec().into_boxed_slice(),
        ))
    }

    fn listv(elems: Vec<Value>) -> Value {
        Value::List(ListHandle::detached(elems))
    }

    fn variant(ty: u32, disc: u32, payload: Option<Value>) -> Value {
        Value::Variant(VariantValue {
            ty,
            disc,
            payload: payload.map(Rc::new),
        })
    }

    /// The nested demo tree used by most cases:
    /// Root{ id: i32, name: Str, point: Point{x: i64, y: u64} }.
    fn demo_tree() -> (Value, DynType, Vec<RecordBinding<'static>>) {
        let point_ty = DynType::record(
            "demo",
            "Point",
            vec![("x", DynType::I64), ("y", DynType::U64)],
        );
        let root_ty = DynType::record(
            "demo",
            "Root",
            vec![
                ("id", DynType::I32),
                ("name", DynType::Str),
                ("point", point_ty.clone()),
            ],
        );
        let point_val = record(7, vec![Value::I64(-5), Value::U64(11)]);
        let root_val = record(3, vec![Value::I32(42), strv("héllo→中文🦀"), point_val]);
        let bindings = vec![
            RecordBinding {
                owner: "demo",
                name: "Root",
                fields: vec!["id", "name", "point"],
                ty_index: 3,
            },
            RecordBinding {
                owner: "demo",
                name: "Point",
                fields: vec!["x", "y"],
                ty_index: 7,
            },
        ];
        (root_val, root_ty, bindings)
    }

    fn pack_demo() -> DynValue {
        let (v, t, _) = demo_tree();
        DynValue::pack(v, t).expect("demo tree is whitelisted")
    }

    fn ctx<'a>(
        records: &'a [RecordBinding<'a>],
        variants: &'a [VariantBinding<'a>],
    ) -> RestoreContext<'a> {
        RestoreContext::new(records, variants)
    }

    // ------------------------------------------------------------------
    // V-14a — freeze/restore equivalence (bit- and byte-exact)
    // ------------------------------------------------------------------

    #[test]
    fn av_dynamic_824_portable_roundtrip_scalars() {
        let scalars: Vec<(Value, DynType)> = vec![
            (Value::I32(i32::MIN), DynType::I32),
            (Value::I32(i32::MAX), DynType::I32),
            (Value::I64(i64::MIN), DynType::I64),
            (Value::U64(u64::MAX), DynType::U64),
            // NaN payload / signed zeros / infinities survive BIT-EXACT.
            (Value::F32(f32::from_bits(0x7fc0_0001)), DynType::F32),
            (
                Value::F64(f64::from_bits(0x7ff8_0000_0000_0001)),
                DynType::F64,
            ),
            (Value::F32(0.0), DynType::F32),
            (Value::F32(-0.0), DynType::F32),
            (Value::F64(f64::INFINITY), DynType::F64),
            (Value::F64(f64::NEG_INFINITY), DynType::F64),
            (Value::Bool(true), DynType::Bool),
            (Value::Bool(false), DynType::Bool),
        ];
        for (v, t) in scalars {
            let dv = DynValue::pack(v.clone(), t.clone()).unwrap();
            let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
            let back = unfreeze(&bytes, &ctx(&[], &[]), &TraversalOptions::default()).unwrap();
            assert!(back.is(&t), "type survived for {:?}", v);
            assert_eq!(back.as_value(), &v, "bit-exact roundtrip for {:?}", v);
        }
    }

    #[test]
    fn av_dynamic_824_portable_roundtrip_str_char() {
        for s in ["", "auto", "héllo→中文🦀", "e\u{0301}"] {
            let dv = DynValue::pack(strv(s), DynType::Str).unwrap();
            let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
            let back = unfreeze(&bytes, &ctx(&[], &[]), &TraversalOptions::default()).unwrap();
            assert_eq!(back.as_value(), &strv(s), "Str bytes for {:?}", s);
        }
        for c in ['a', '中', '\u{10FFFF}', '\u{0301}'] {
            let dv = DynValue::pack(Value::Char(c), DynType::Char).unwrap();
            let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
            let back = unfreeze(&bytes, &ctx(&[], &[]), &TraversalOptions::default()).unwrap();
            assert_eq!(back.as_value(), &Value::Char(c));
        }
    }

    #[test]
    fn av_dynamic_824_portable_roundtrip_nested_records() {
        let dv = pack_demo();
        let (_, _, bindings) = demo_tree();
        let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
        let back = unfreeze(&bytes, &ctx(&bindings, &[]), &TraversalOptions::default()).unwrap();
        assert!(back.is(dv.ty()), "nominal type identity survives");
        assert_eq!(back.as_value(), dv.as_value(), "value tree equals");
        // The positional index came from the CURRENT context (Point→7).
        match back.as_value() {
            Value::Record(r) => match &r.fields[2] {
                Value::Record(p) => assert_eq!(p.ty, 7, "ty_index resolved via context"),
                _ => panic!("point field"),
            },
            _ => panic!("root"),
        }
    }

    #[test]
    fn av_dynamic_824_portable_roundtrip_list_variant() {
        // List<Record>: two notes; PLUS the empty-list identity probe —
        // an empty List<Str> must restore as List<Str>, not List<i32>.
        let note_ty = DynType::record(
            "demo",
            "Note",
            vec![("text", DynType::Str), ("n", DynType::I32)],
        );
        let list_ty = DynType::list(note_ty.clone());
        let list_val = listv(vec![
            record(4, vec![strv("a"), Value::I32(1)]),
            record(4, vec![strv("b"), Value::I32(2)]),
        ]);
        let dv = DynValue::pack(list_val, list_ty.clone()).unwrap();
        let records = [RecordBinding {
            owner: "demo",
            name: "Note",
            fields: vec!["text", "n"],
            ty_index: 4,
        }];
        let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
        let back = unfreeze(&bytes, &ctx(&records, &[]), &TraversalOptions::default()).unwrap();
        assert!(back.is(&list_ty), "structural list identity survives");
        assert_eq!(back.as_value(), dv.as_value());

        let empty = DynValue::pack(listv(vec![]), DynType::list(DynType::Str)).unwrap();
        let ebytes = freeze(&empty, &TraversalOptions::default()).unwrap();
        let eback = unfreeze(&ebytes, &ctx(&[], &[]), &TraversalOptions::default()).unwrap();
        assert!(
            eback.is(&DynType::list(DynType::Str)),
            "empty list keeps its element type"
        );
        assert!(matches!(eback.as_value(), Value::List(h) if h.rc.borrow().elems.is_empty()));

        // Variant: Option-like (None / Some payload) + Result-like Ok with
        // an unused Err arm.
        let opt_ty = DynType::variant(
            "demo",
            "Opt",
            vec![("none", None), ("some", Some(DynType::I64))],
        );
        let variants = [VariantBinding {
            owner: "demo",
            name: "Opt",
            arms: vec![
                ArmSig {
                    name: "none",
                    has_payload: false,
                },
                ArmSig {
                    name: "some",
                    has_payload: true,
                },
            ],
            ty_index: 9,
        }];
        for (val, disc) in [
            (variant(9, 0, None), 0),
            (variant(9, 1, Some(Value::I64(-1))), 1),
        ] {
            let dv = DynValue::pack(val, opt_ty.clone()).unwrap();
            let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
            let back =
                unfreeze(&bytes, &ctx(&[], &variants), &TraversalOptions::default()).unwrap();
            assert!(back.is(&opt_ty), "nominal variant identity survives");
            assert_eq!(back.as_value(), dv.as_value(), "disc {} roundtrip", disc);
        }
        let res_ty = DynType::variant(
            "demo",
            "Res",
            vec![("ok", Some(DynType::U64)), ("err", Some(DynType::Str))],
        );
        let res_variants = [VariantBinding {
            owner: "demo",
            name: "Res",
            arms: vec![
                ArmSig {
                    name: "ok",
                    has_payload: true,
                },
                ArmSig {
                    name: "err",
                    has_payload: true,
                },
            ],
            ty_index: 10,
        }];
        let ok_val = DynValue::pack(variant(10, 0, Some(Value::U64(5))), res_ty.clone()).unwrap();
        let bytes = freeze(&ok_val, &TraversalOptions::default()).unwrap();
        let back = unfreeze(
            &bytes,
            &ctx(&[], &res_variants),
            &TraversalOptions::default(),
        )
        .unwrap();
        assert!(back.is(&res_ty), "unused arms' payload types survive");
    }

    #[test]
    fn av_dynamic_824_portable_freeze_deterministic() {
        let dv = pack_demo();
        let a = freeze(&dv, &TraversalOptions::default()).unwrap();
        let b = freeze(&dv, &TraversalOptions::default()).unwrap();
        assert_eq!(a, b, "same source freezes twice to identical bytes");
        // An INDEPENDENTLY built equal value freezes to the same bytes
        // (V-14 same-source hash oracle).
        let other = pack_demo();
        let c = freeze(&other, &TraversalOptions::default()).unwrap();
        assert_eq!(a, c);
        // Header sanity: magic, version 1, kind 1, node count 6
        // (root + id + name + point + point.x + point.y).
        assert_eq!(&a[0..4], &PORTABLE_MAGIC[..]);
        assert_eq!(u16::from_le_bytes([a[4], a[5]]), 1);
        assert_eq!(a[6], 1);
        assert_eq!(u32::from_le_bytes([a[8], a[9], a[10], a[11]]), 6);
    }

    // ------------------------------------------------------------------
    // V-14b — checked access (wrong-type take keeps the source)
    // ------------------------------------------------------------------

    #[test]
    fn av_dynamic_824_take_wrong_type_preserves_source() {
        let dv = pack_demo();
        let (_, root_ty, _) = demo_tree();
        // Same-width DIFFERENT-name record never matches (nominal rule).
        let stranger = DynType::record(
            "demo",
            "Stranger",
            vec![
                ("id", DynType::I32),
                ("name", DynType::Str),
                (
                    "point",
                    DynType::record(
                        "demo",
                        "Point",
                        vec![("x", DynType::I64), ("y", DynType::U64)],
                    ),
                ),
            ],
        );
        assert!(!dv.is(&stranger));
        let err = match dv.take_as(&stranger) {
            Ok(_) => panic!("wrong nominal type must fail"),
            Err(e) => e,
        };
        assert_eq!(err.code(), "av.dyn.take-type");
        // THE source is preserved: still readable, still take-able.
        let source = err.into_source();
        assert!(source.borrow_as(&root_ty).is_some());
        let (_, _, bindings) = demo_tree();
        let bytes = freeze(&source, &TraversalOptions::default()).unwrap();
        let back = unfreeze(&bytes, &ctx(&bindings, &[]), &TraversalOptions::default()).unwrap();
        assert_eq!(back.as_value(), source.as_value());
        let v = source.take_as(&root_ty).expect("right type now succeeds");
        assert!(matches!(v, Value::Record(_)));
    }

    #[test]
    fn av_dynamic_824_take_no_cross_kind_coercion() {
        let i32v = DynValue::pack(Value::I32(1), DynType::I32).unwrap();
        // bool/int/char/Str are never interchangeable guesses.
        assert!(!i32v.is(&DynType::Bool));
        assert!(!i32v.is(&DynType::I64));
        assert!(!i32v.is(&DynType::Char));
        assert!(!i32v.is(&DynType::Str));
        assert!(i32v.borrow_as(&DynType::Bool).is_none());
        let err = i32v.take_as(&DynType::Bool).unwrap_err();
        assert!(matches!(
            err.into_source().take_as(&DynType::I32),
            Ok(Value::I32(1))
        ));
        let s = DynValue::pack(strv("x"), DynType::Str).unwrap();
        assert!(s.take_as(&DynType::I32).is_err());
        // List vs Variant vs Record are distinct kinds; a List<i32> is not
        // a List<i64> (structural element identity).
        let li = DynValue::pack(listv(vec![Value::I32(1)]), DynType::list(DynType::I32)).unwrap();
        assert!(!li.is(&DynType::list(DynType::I64)));
        assert!(!li.is(&DynType::record("o", "R", vec![])));
        assert!(!li.is(&DynType::variant("o", "V", vec![])));
    }

    #[test]
    fn av_dynamic_824_deep_clone_independent_expands_sharing() {
        // Shared DAG (legal in-memory pure-value sharing): one Point Rc
        // referenced by two fields.
        let point_ty = DynType::record(
            "demo",
            "Point",
            vec![("x", DynType::I64), ("y", DynType::U64)],
        );
        let twin_ty = DynType::record(
            "demo",
            "Twin",
            vec![("a", point_ty.clone()), ("b", point_ty.clone())],
        );
        let shared = Rc::new(RecordValue {
            ty: 9,
            fields: vec![Value::I64(1), Value::U64(2)],
        });
        let twin_val = record(
            8,
            vec![
                Value::Record(Rc::clone(&shared)),
                Value::Record(Rc::clone(&shared)),
            ],
        );
        let dv = DynValue::pack(twin_val, twin_ty.clone()).unwrap();
        // Deep clone: independent fresh Rc per EXPANDED node.
        let clone = dv.deep_clone(&TraversalOptions::default()).unwrap();
        assert!(clone.is(&twin_ty));
        assert_eq!(clone.as_value(), dv.as_value());
        let (a, b) = match clone.as_value() {
            Value::Record(r) => match (&r.fields[0], &r.fields[1]) {
                (Value::Record(a), Value::Record(b)) => (a, b),
                _ => panic!(),
            },
            _ => panic!(),
        };
        assert_ne!(Rc::as_ptr(a), Rc::as_ptr(b), "record sharing was expanded");
        // The ORIGINAL still shares (untouched).
        let (oa, ob) = match dv.as_value() {
            Value::Record(r) => match (&r.fields[0], &r.fields[1]) {
                (Value::Record(a), Value::Record(b)) => (a, b),
                _ => panic!(),
            },
            _ => panic!(),
        };
        assert_eq!(Rc::as_ptr(oa), Rc::as_ptr(ob));
        // List deep clone mints an independent buffer; Str an independent
        // detached buffer.
        let lv = DynValue::pack(
            listv(vec![Value::I32(3), Value::I32(4)]),
            DynType::list(DynType::I32),
        )
        .unwrap();
        let lc = lv.deep_clone(&TraversalOptions::default()).unwrap();
        match (lv.as_value(), lc.as_value()) {
            (Value::List(a), Value::List(b)) => {
                assert_ne!(Rc::as_ptr(&a.rc) as usize, Rc::as_ptr(&b.rc) as usize);
            }
            _ => panic!(),
        }
        let s = DynValue::pack(strv("payload"), DynType::Str).unwrap();
        assert_eq!(
            s.deep_clone(&TraversalOptions::default())
                .unwrap()
                .as_value(),
            s.as_value()
        );
    }

    // ------------------------------------------------------------------
    // V-14c — failure keeps the source / clean retry (rollback)
    // ------------------------------------------------------------------

    #[test]
    fn av_dynamic_824_rollback_unfreeze_fail_then_retry() {
        let dv = pack_demo();
        let (_, _, bindings) = demo_tree();
        let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
        // Injected fault at the 3rd node — classified as the node budget
        // (resource-boundary equivalence), nothing half-installed.
        let mut opts = TraversalOptions::default();
        opts.fail_after_nodes = Some(3);
        let err = unfreeze(&bytes, &ctx(&bindings, &[]), &opts).unwrap_err();
        assert_eq!(
            err,
            PortableError::Budget {
                code: "av.resource.dyn-nodes"
            }
        );
        // The bytes are NOT consumed: a clean retry fully succeeds.
        let back = unfreeze(&bytes, &ctx(&bindings, &[]), &TraversalOptions::default()).unwrap();
        assert_eq!(back.as_value(), dv.as_value());
    }

    #[test]
    fn av_dynamic_824_rollback_freeze_failure_preserves_source_ledger() {
        // Arena-ledger oracle: an ATTACHED Str whose buffer is referenced
        // by two fields — pack is legal (pure-value sharing), freeze is the
        // boundary that rejects the shared graph. The source must stay
        // alive (no free events), readable and take-able.
        let arena = StrArena::new();
        let h = arena.alloc(b"ledgered");
        assert_eq!(arena.live(), 1);
        let ty = DynType::record(
            "demo",
            "Two",
            vec![("a", DynType::Str), ("b", DynType::Str)],
        );
        let val = record(5, vec![Value::Str(h.clone()), Value::Str(h)]);
        let dv = DynValue::pack(val, ty.clone()).unwrap();
        // No fault injection needed: the second reference to the shared
        // buffer IS the deterministic rejection (the seam never fires).
        let err = freeze(&dv, &TraversalOptions::default()).unwrap_err();
        assert_eq!(
            err,
            PortableError::Unsupported {
                category: "shared-graph"
            },
            "the second reference is the rejection"
        );
        assert_eq!(arena.live(), 1, "source buffer still owned by the source");
        assert!(
            arena.free_events().is_empty(),
            "no leak: nothing dropped early"
        );
        // The source survives the failed freeze untouched.
        let v = dv.take_as(&ty).unwrap();
        let (a, b) = match &v {
            Value::Record(r) => (&r.fields[0], &r.fields[1]),
            _ => panic!(),
        };
        assert_eq!(a, b);
        assert_eq!(arena.live(), 1);
        drop(v);
        assert_eq!(arena.live(), 0, "final release reports exactly once");
    }

    // ------------------------------------------------------------------
    // V-14d — recursive whitelist rejection
    // ------------------------------------------------------------------

    #[test]
    fn av_dynamic_824_reject_nested_borrows_and_place() {
        // A StrView smuggled at depth 2 (a record field) is rejected with
        // the depth where it was found — the outer record does not mask it.
        let view = Value::StrView(StrView {
            src: StrViewSrc::Share(StrHandle::detached(b"src".to_vec().into_boxed_slice())),
            start: 0,
            end: 1,
        });
        let ty = DynType::record(
            "demo",
            "Bad",
            vec![(
                "inner",
                DynType::record("demo", "In", vec![("v", DynType::Str)]),
            )],
        );
        let val = record(1, vec![record(2, vec![view])]);
        let err = DynValue::pack(val, ty).unwrap_err();
        assert_eq!(
            err,
            DynPackError::Unsupported {
                category: "str-view",
                depth: 3
            },
            "borrow categories are unsupported at any depth"
        );
        // Direct pack of a view / a MutRef / a Place: same admission.
        let direct = DynValue::pack(
            Value::StrView(StrView {
                src: StrViewSrc::Static {
                    buf: Rc::new(b"const".to_vec().into_boxed_slice()),
                },
                start: 0,
                end: 1,
            }),
            DynType::Str,
        )
        .unwrap_err();
        assert_eq!(direct.code(), "av.dyn.unsupported");
        let mr = DynValue::pack(
            Value::MutRef(crate::contract::MutRefV {
                frame: 0,
                slot: 1,
                gen: 1,
            }),
            DynType::I32,
        )
        .unwrap_err();
        assert!(matches!(
            mr,
            DynPackError::Unsupported {
                category: "mut-ref",
                ..
            }
        ));
        // PLAN-822 index Place token nested in a list element: rejected
        // like any borrow category (r2 closed whitelist).
        let pl = DynValue::pack(
            listv(vec![Value::Place(crate::contract::ListPlaceV {
                frame: 0,
                slot: 0,
                buf_id: 1,
                epoch: 0,
                index: 0,
                elem_gen: 0,
            })]),
            DynType::list(DynType::I32),
        )
        .unwrap_err();
        assert!(matches!(
            pl,
            DynPackError::Unsupported {
                category: "place",
                depth: 2
            }
        ));
        // A declared-type/value-kind mismatch is a DIFFERENT class (invalid
        // input, not unsupported category).
        let mm = DynValue::pack(Value::I32(3), DynType::Str).unwrap_err();
        assert_eq!(mm.code(), "av.dyn.pack-type");
    }

    #[test]
    fn av_dynamic_824_shared_dag_pack_ok_freeze_rejects() {
        let point_ty = DynType::record(
            "demo",
            "Point",
            vec![("x", DynType::I64), ("y", DynType::U64)],
        );
        let twin_ty = DynType::record(
            "demo",
            "Twin",
            vec![("a", point_ty.clone()), ("b", point_ty.clone())],
        );
        let shared = Rc::new(RecordValue {
            ty: 9,
            fields: vec![Value::I64(1), Value::U64(2)],
        });
        let val = record(
            8,
            vec![
                Value::Record(Rc::clone(&shared)),
                Value::Record(Rc::clone(&shared)),
            ],
        );
        let dv = DynValue::pack(val, twin_ty.clone()).unwrap();
        let err = freeze(&dv, &TraversalOptions::default()).unwrap_err();
        assert_eq!(
            err,
            PortableError::Unsupported {
                category: "shared-graph"
            }
        );
        // Failure kept the source: take succeeds, and a TREE-shaped copy of
        // the same content freezes fine.
        let v = dv.take_as(&twin_ty).unwrap();
        let tree_val = record(
            8,
            vec![
                record(9, vec![Value::I64(1), Value::U64(2)]),
                record(9, vec![Value::I64(1), Value::U64(2)]),
            ],
        );
        let tree = DynValue::pack(tree_val, twin_ty).unwrap();
        let bytes = freeze(&tree, &TraversalOptions::default()).unwrap();
        assert!(!bytes.is_empty());
        drop(v);
        // A shared LIST buffer in two fields is the same rejection.
        let shared_list = ListHandle::detached(vec![Value::I32(7)]);
        let lt = DynType::record(
            "demo",
            "LT",
            vec![
                ("a", DynType::list(DynType::I32)),
                ("b", DynType::list(DynType::I32)),
            ],
        );
        let lv = DynValue::pack(
            record(
                11,
                vec![Value::List(shared_list.clone()), Value::List(shared_list)],
            ),
            lt,
        )
        .unwrap();
        assert_eq!(
            freeze(&lv, &TraversalOptions::default()),
            Err(PortableError::Unsupported {
                category: "shared-graph"
            })
        );
    }

    #[test]
    fn av_dynamic_824_type_surface_closed_no_fn_or_borrow_category() {
        // Exhaustive match WITHOUT a wildcard arm: `DynType` provably has
        // no Fn/closure/borrow/Place category — the whitelist is closed by
        // construction (V-12: no fake future categories).
        fn kind(t: &DynType) -> &'static str {
            match t {
                DynType::Bool
                | DynType::I32
                | DynType::I64
                | DynType::U64
                | DynType::F32
                | DynType::F64
                | DynType::Char
                | DynType::Str => "scalar",
                DynType::Record { .. } => "record",
                DynType::List(_) => "list",
                DynType::Variant { .. } => "variant",
            }
        }
        assert_eq!(kind(&DynType::Str), "scalar");
        assert_eq!(kind(&DynType::record("o", "R", vec![])), "record");
        assert_eq!(kind(&DynType::list(DynType::I32)), "list");
        assert_eq!(kind(&DynType::variant("o", "V", vec![])), "variant");
    }

    #[test]
    fn av_dynamic_824_malformed_types_rejected() {
        let dup = DynType::record(
            "demo",
            "Dup",
            vec![("x", DynType::I32), ("x", DynType::I32)],
        );
        let err = DynValue::pack(Value::I32(1), dup).unwrap_err();
        assert_eq!(
            err,
            DynPackError::InvalidType {
                reason: "duplicate-field"
            }
        );
        // Wider than the core MAX_RECORD_FIELDS guard.
        let wide = DynType::record(
            "demo",
            "Wide",
            (0..1025).map(|i| (leak_name(i), DynType::I32)).collect(),
        );
        let err = DynValue::pack(Value::I32(1), wide).unwrap_err();
        assert_eq!(
            err,
            DynPackError::InvalidType {
                reason: "field-count"
            }
        );
        // Duplicate variant arm names / disc / payload-presence mismatch.
        let dup_arm = DynType::variant("demo", "DV", vec![("a", None), ("a", Some(DynType::I32))]);
        let err = DynValue::pack(variant(1, 0, None), dup_arm).unwrap_err();
        assert_eq!(
            err,
            DynPackError::InvalidType {
                reason: "duplicate-arm"
            }
        );
        // Arity mismatch between value and declared type.
        let ty = DynType::record("demo", "R2", vec![("a", DynType::I32)]);
        let err = DynValue::pack(record(1, vec![]), ty).unwrap_err();
        assert!(matches!(err, DynPackError::TypeMismatch { .. }));
        let err = DynValue::pack(
            variant(1, 5, None),
            DynType::variant("demo", "O", vec![("n", None), ("s", Some(DynType::I32))]),
        )
        .unwrap_err();
        assert!(
            matches!(err, DynPackError::TypeMismatch { .. }),
            "disc out of range"
        );
        let err = DynValue::pack(
            variant(1, 1, None),
            DynType::variant("demo", "O2", vec![("n", None), ("s", Some(DynType::I32))]),
        )
        .unwrap_err();
        assert!(
            matches!(err, DynPackError::TypeMismatch { .. }),
            "payload presence"
        );
        // List longer than MAX_LIST_ELEMS (65_536).
        let long = DynValue::pack(
            listv((0..=MAX_DYN_NODES).map(|_| Value::I32(1)).collect()),
            DynType::list(DynType::I32),
        )
        .unwrap_err();
        assert_eq!(long.code(), "av.dyn.invalid-type");
    }

    /// Unique `&'static str` field names without leaking per iteration:
    /// one leaked pool (1100 names: 1024 Mid leaves + 64+ root fields)
    /// reused by all tests.
    fn leak_name(i: usize) -> &'static str {
        use std::sync::OnceLock;
        static NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
        let names = NAMES.get_or_init(|| {
            (0..1100)
                .map(|i| Box::leak(format!("f{}", i).into_boxed_str()) as &'static str)
                .collect()
        });
        names[i]
    }

    // ------------------------------------------------------------------
    // V-14e — wire negatives (structure is never bypassed)
    // ------------------------------------------------------------------

    /// Hand-built wire bytes: header + body + checksum (test-side builder
    /// independent of the production encoder for negative payloads).
    fn wire(node_count: u32, body: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&PORTABLE_MAGIC);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.push(1); // kind
        out.push(0); // flags
        out.extend_from_slice(&node_count.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(body);
        let sum = fnv1a64(&out);
        out.extend_from_slice(&sum.to_le_bytes());
        out
    }

    /// Test-side type-descriptor writer (mirrors the frozen wire grammar).
    fn tydesc(t: &DynType) -> Vec<u8> {
        fn rec(t: &DynType, out: &mut Vec<u8>) {
            match t {
                DynType::Bool => out.push(0),
                DynType::I32 => out.push(1),
                DynType::I64 => out.push(2),
                DynType::U64 => out.push(3),
                DynType::F32 => out.push(4),
                DynType::F64 => out.push(5),
                DynType::Char => out.push(6),
                DynType::Str => out.push(7),
                DynType::List(e) => {
                    out.push(8);
                    rec(e, out);
                }
                DynType::Record {
                    owner,
                    name,
                    fields,
                } => {
                    out.push(9);
                    out.extend_from_slice(&(owner.len() as u16).to_le_bytes());
                    out.extend_from_slice(owner.as_bytes());
                    out.extend_from_slice(&(name.len() as u16).to_le_bytes());
                    out.extend_from_slice(name.as_bytes());
                    out.extend_from_slice(&(fields.len() as u16).to_le_bytes());
                    for f in fields {
                        out.extend_from_slice(&(f.name.len() as u16).to_le_bytes());
                        out.extend_from_slice(f.name.as_bytes());
                        rec(&f.ty, out);
                    }
                }
                DynType::Variant { owner, name, arms } => {
                    out.push(10);
                    out.extend_from_slice(&(owner.len() as u16).to_le_bytes());
                    out.extend_from_slice(owner.as_bytes());
                    out.extend_from_slice(&(name.len() as u16).to_le_bytes());
                    out.extend_from_slice(name.as_bytes());
                    out.extend_from_slice(&(arms.len() as u16).to_le_bytes());
                    for a in arms {
                        out.extend_from_slice(&(a.name.len() as u16).to_le_bytes());
                        out.extend_from_slice(a.name.as_bytes());
                        match &a.payload {
                            Some(p) => {
                                out.push(1);
                                rec(p, out);
                            }
                            None => out.push(0),
                        }
                    }
                }
            }
        }
        let mut out = Vec::new();
        rec(t, &mut out);
        out
    }

    fn i32_node(v: i32) -> Vec<u8> {
        let mut b = tydesc(&DynType::I32);
        b.push(0);
        b.extend_from_slice(&v.to_le_bytes());
        b
    }

    fn record_node(ty: &DynType, children: &[&[u8]]) -> Vec<u8> {
        let mut b = tydesc(ty);
        b.push(8);
        for c in children {
            b.extend_from_slice(c);
        }
        b
    }

    #[test]
    fn av_dynamic_824_wire_header_negatives() {
        let good_body = i32_node(7);
        let good = wire(1, &good_body);
        let empty = ctx(&[], &[]);
        // Bad magic.
        let mut m = good.clone();
        m[0] = b'X';
        m.truncate(m.len() - 8);
        let sum = fnv1a64(&m);
        m.extend_from_slice(&sum.to_le_bytes());
        assert_eq!(
            unfreeze(&m, &empty, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "magic" })
        );
        // Bad version (2) — a different format generation.
        let mut v = good.clone();
        v[4] = 2;
        v.truncate(v.len() - 8);
        let sum = fnv1a64(&v);
        v.extend_from_slice(&sum.to_le_bytes());
        assert_eq!(
            unfreeze(&v, &empty, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "version" })
        );
        // Unknown kind / nonzero flags / nonzero reserved.
        for (pos, reason) in [(6, "kind"), (7, "flags"), (12, "reserved")] {
            let mut g = good.clone();
            g[pos] = if pos == 6 { 9 } else { 1 };
            g.truncate(g.len() - 8);
            let sum = fnv1a64(&g);
            g.extend_from_slice(&sum.to_le_bytes());
            assert_eq!(
                unfreeze(&g, &empty, &TraversalOptions::default()),
                Err(PortableError::Invalid { reason }),
                "header byte {}",
                pos
            );
        }
        // Declared node count above the ceiling: resource, before parsing.
        let big = wire(MAX_DYN_NODES + 1, &good_body);
        assert_eq!(
            unfreeze(&big, &empty, &TraversalOptions::default()),
            Err(PortableError::Budget {
                code: "av.resource.dyn-nodes"
            })
        );
    }

    #[test]
    fn av_dynamic_824_wire_payload_negatives() {
        let empty = ctx(&[], &[]);
        // Unknown payload tag (after a valid tydesc).
        let t = wire(1, &{
            let mut b = tydesc(&DynType::I32);
            b.push(11);
            b
        });
        assert_eq!(
            unfreeze(&t, &empty, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "tag" })
        );
        // Unknown tydesc tag.
        let t2 = wire(1, &[99u8, 0, 1, 0, 0, 0]);
        assert_eq!(
            unfreeze(&t2, &empty, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "tydesc" })
        );
        // Bool domain: 2 is neither false nor true.
        let b = wire(1, &{
            let mut b = tydesc(&DynType::Bool);
            b.push(1);
            b.push(2);
            b
        });
        assert_eq!(
            unfreeze(&b, &empty, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "bool-domain"
            })
        );
        // Char domain: a UTF-16 surrogate is not a Unicode scalar.
        let c = wire(1, &{
            let mut b = tydesc(&DynType::Char);
            b.push(6);
            b.extend_from_slice(&0xD800u32.to_le_bytes());
            b
        });
        assert_eq!(
            unfreeze(&c, &empty, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "char-domain"
            })
        );
        // Bad UTF-8 in a Str payload.
        let s = wire(1, &{
            let mut b = tydesc(&DynType::Str);
            b.push(7);
            b.extend_from_slice(&2u32.to_le_bytes());
            b.extend_from_slice(&[0xFF, 0xFE]);
            b
        });
        assert_eq!(
            unfreeze(&s, &empty, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "utf8" })
        );
        // Truncated mid-node and missing fields.
        let half = wire(1, &[1u8, 7]);
        assert_eq!(
            unfreeze(&half, &empty, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "truncated"
            })
        );
        let binding = [RecordBinding {
            owner: "d",
            name: "R",
            fields: vec!["a"],
            ty_index: 1,
        }];
        let rctx = ctx(&binding, &[]);
        let rt = DynType::record("d", "R", vec![("a", DynType::I32)]);
        let missing = wire(2, &record_node(&rt, &[])); // child never arrives
        assert_eq!(
            unfreeze(&missing, &rctx, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "truncated"
            })
        );
        // Duplicate field names in one record tydesc.
        let dupf = {
            let mut b = vec![9u8];
            b.extend_from_slice(&1u16.to_le_bytes());
            b.push(b'd');
            b.extend_from_slice(&1u16.to_le_bytes());
            b.push(b'R');
            b.extend_from_slice(&2u16.to_le_bytes());
            for _ in 0..2 {
                b.extend_from_slice(&1u16.to_le_bytes());
                b.push(b'a');
                b.push(1); // field tydesc: I32
            }
            b.push(8); // TAG_RECORD
            b.extend_from_slice(&i32_node(1));
            b.extend_from_slice(&i32_node(2));
            b
        };
        let dup_wire = wire(3, &dupf);
        assert_eq!(
            unfreeze(&dup_wire, &rctx, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "duplicate-field"
            })
        );
        // Trailing bytes after a complete root.
        let mut trail = wire(1, &i32_node(1));
        let body_len = trail.len() - 8;
        trail.insert(body_len, 0);
        trail.truncate(trail.len() - 8);
        let sum = fnv1a64(&trail);
        trail.extend_from_slice(&sum.to_le_bytes());
        assert_eq!(
            unfreeze(&trail, &empty, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "trailing" })
        );
    }

    #[test]
    fn av_dynamic_824_wire_type_consistency_negatives() {
        // A list node whose tydesc says List<i32> but whose child carries
        // an f32 tydesc: declared-vs-observed mismatch is Invalid.
        let bad_list = {
            let mut child = tydesc(&DynType::F32);
            child.push(4); // TAG_F32
            child.extend_from_slice(&1.0f32.to_bits().to_le_bytes());
            let mut b = tydesc(&DynType::list(DynType::I32));
            b.push(9); // TAG_LIST
            b.extend_from_slice(&1u32.to_le_bytes());
            b.extend_from_slice(&child);
            b
        };
        let w = wire(2, &bad_list);
        assert_eq!(
            unfreeze(&w, &ctx(&[], &[]), &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "type" })
        );
        // A record whose field type disagrees with the observed child.
        let rt = DynType::record("d", "R", vec![("a", DynType::I32)]);
        let binding = [RecordBinding {
            owner: "d",
            name: "R",
            fields: vec!["a"],
            ty_index: 1,
        }];
        let bad_rec = {
            let mut child = tydesc(&DynType::I64);
            child.push(2); // TAG_I64
            child.extend_from_slice(&1i64.to_le_bytes());
            let mut b = tydesc(&rt);
            b.push(8);
            b.extend_from_slice(&child);
            b
        };
        let w2 = wire(2, &bad_rec);
        assert_eq!(
            unfreeze(&w2, &ctx(&binding, &[]), &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "type" })
        );
        // Variant disc out of range of the declared arms.
        let vt = DynType::variant("d", "V", vec![("n", None), ("s", Some(DynType::I32))]);
        let vsig = [VariantBinding {
            owner: "d",
            name: "V",
            arms: vec![
                ArmSig {
                    name: "n",
                    has_payload: false,
                },
                ArmSig {
                    name: "s",
                    has_payload: true,
                },
            ],
            ty_index: 2,
        }];
        let mut vbody = tydesc(&vt);
        vbody.push(10);
        vbody.extend_from_slice(&7u32.to_le_bytes());
        let w3 = wire(1, &vbody);
        assert_eq!(
            unfreeze(&w3, &ctx(&[], &vsig), &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "type" })
        );
    }

    #[test]
    fn av_dynamic_824_wire_checksum_and_node_count_integrity() {
        let dv = pack_demo();
        let (_, _, bindings) = demo_tree();
        let c = ctx(&bindings, &[]);
        let good = freeze(&dv, &TraversalOptions::default()).unwrap();
        assert!(unfreeze(&good, &c, &TraversalOptions::default()).is_ok());
        // Good structure + bad checksum: flip only the trailer.
        let mut badsum = good.clone();
        let last = badsum.len() - 1;
        badsum[last] ^= 0xFF;
        assert_eq!(
            unfreeze(&badsum, &c, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "checksum" })
        );
        // Bad structure with a RE-COMPUTED checksum is still rejected by
        // the structural gate (a good hash never launders bad bytes).
        let mut badtag = good.clone();
        const HEADER: usize = 16;
        badtag[HEADER] = 99; // unknown tydesc tag at the root
        badtag.truncate(badtag.len() - 8);
        let sum = fnv1a64(&badtag);
        badtag.extend_from_slice(&sum.to_le_bytes());
        assert_eq!(
            unfreeze(&badtag, &c, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "tydesc" })
        );
        // A lying node_count (declared != parsed) with a valid checksum.
        for delta in [1i64, -1] {
            let mut lie = good.clone();
            let n = u32::from_le_bytes([lie[8], lie[9], lie[10], lie[11]]) as i64 + delta;
            lie[8..12].copy_from_slice(&(n as u32).to_le_bytes());
            lie.truncate(lie.len() - 8);
            let sum = fnv1a64(&lie);
            lie.extend_from_slice(&sum.to_le_bytes());
            assert_eq!(
                unfreeze(&lie, &c, &TraversalOptions::default()),
                Err(PortableError::Invalid {
                    reason: "node-count"
                }),
                "declared count off by {}",
                delta
            );
        }
        // Bit-flipped payload byte keeps structure but breaks the checksum:
        // the last body byte is inside the deepest u64 payload.
        let mut flip = good.clone();
        let body_end = flip.len() - 8;
        flip[body_end - 1] ^= 0x01;
        assert_eq!(
            unfreeze(&flip, &c, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "checksum" })
        );
    }

    #[test]
    fn av_dynamic_824_admission_unknown_and_layout_mismatch() {
        let dv = pack_demo();
        let (_, _, bindings) = demo_tree();
        let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
        // No bindings at all: unknown nominal type at the decode gate.
        let empty = ctx(&[], &[]);
        assert_eq!(
            unfreeze(&bytes, &empty, &TraversalOptions::default()),
            Err(PortableError::Admission {
                code: "portable.unknown-type"
            })
        );
        // Renamed field: a layout/revision change never reuses stale data.
        let renamed = [RecordBinding {
            owner: "demo",
            name: "Root",
            fields: vec!["id", "title", "point"], // was "name"
            ty_index: 3,
        }];
        assert_eq!(
            unfreeze(&bytes, &ctx(&renamed, &[]), &TraversalOptions::default()),
            Err(PortableError::Admission {
                code: "portable.layout-mismatch"
            })
        );
        // Reordered fields likewise.
        let reordered = [RecordBinding {
            owner: "demo",
            name: "Root",
            fields: vec!["name", "id", "point"],
            ty_index: 3,
        }];
        assert_eq!(
            unfreeze(&bytes, &ctx(&reordered, &[]), &TraversalOptions::default()),
            Err(PortableError::Admission {
                code: "portable.layout-mismatch"
            })
        );
        // Variant admission: payload-ness flip is a layout mismatch.
        let vt = DynType::variant("demo", "Opt", vec![("n", None)]);
        let vsig = [VariantBinding {
            owner: "demo",
            name: "Opt",
            arms: vec![ArmSig {
                name: "n",
                has_payload: true,
            }], // wire says none
            ty_index: 5,
        }];
        let vw = freeze(
            &DynValue::pack(variant(5, 0, None), vt).unwrap(),
            &TraversalOptions::default(),
        )
        .unwrap();
        assert_eq!(
            unfreeze(&vw, &ctx(&[], &vsig), &TraversalOptions::default()),
            Err(PortableError::Admission {
                code: "portable.layout-mismatch"
            })
        );
        // Nothing was consumed: the correct context still restores.
        let back = unfreeze(&bytes, &ctx(&bindings, &[]), &TraversalOptions::default()).unwrap();
        assert_eq!(back.as_value(), dv.as_value());
    }

    // ------------------------------------------------------------------
    // V-14f — budgets / cancellation
    // ------------------------------------------------------------------

    /// Root with 64 `Mid` fields, each Mid holding 1024 i32 leaves:
    /// 1 + 64 + 65536 = 65601 nodes — one above the ceiling.
    fn oversized_tree() -> (Value, DynType) {
        let mid_ty = DynType::record(
            "big",
            "Mid",
            (0..1024).map(|i| (leak_name(i), DynType::I32)).collect(),
        );
        let root_ty = DynType::record(
            "big",
            "Root",
            (0..64)
                .map(|i| (leak_name(i + 1024), mid_ty.clone()))
                .collect(),
        );
        let mids: Vec<Value> = (0..64)
            .map(|_| record(1, (0..1024).map(|_| Value::I32(1)).collect()))
            .collect();
        (record(2, mids), root_ty)
    }

    #[test]
    fn av_dynamic_824_budget_nodes_freeze() {
        let (v, t) = oversized_tree();
        let dv = DynValue::pack(v, t).unwrap();
        assert_eq!(
            freeze(&dv, &TraversalOptions::default()),
            Err(PortableError::Budget {
                code: "av.resource.dyn-nodes"
            })
        );
    }

    #[test]
    fn av_dynamic_824_budget_bytes() {
        // A 9 MiB Str payload exceeds the 8 MiB data line on freeze — the
        // charge lands BEFORE the payload append.
        let big = "x".repeat(9 * 1024 * 1024);
        let dv = DynValue::pack(
            Value::Str(StrHandle::detached(big.into_bytes().into_boxed_slice())),
            DynType::Str,
        )
        .unwrap();
        assert_eq!(
            freeze(&dv, &TraversalOptions::default()),
            Err(PortableError::Budget {
                code: "av.resource.dyn-bytes"
            })
        );
        // An input larger than the line is rejected before any parse.
        let small = freeze(
            &DynValue::pack(Value::I32(1), DynType::I32).unwrap(),
            &TraversalOptions::default(),
        )
        .unwrap();
        let mut padded = small.clone();
        padded.resize(small.len() + 9 * 1024 * 1024, 0);
        assert_eq!(
            unfreeze(&padded, &ctx(&[], &[]), &TraversalOptions::default()),
            Err(PortableError::Budget {
                code: "av.resource.dyn-bytes"
            })
        );
    }

    #[test]
    fn av_dynamic_824_budget_depth() {
        // A 257-deep chain (256 records + scalar) is rejected at pack with
        // the depth budget — built ITERATIVELY (the test never recurses).
        let mut val = Value::I32(1);
        let mut ty = DynType::I32;
        for _ in 0..256 {
            ty = DynType::record("deep", "N", vec![("x", ty)]);
            val = record(1, vec![val]);
        }
        let err = DynValue::pack(val, ty).unwrap_err();
        assert_eq!(err.code(), "av.resource.dyn-depth");
        // A 300-deep hand-built wire is rejected at the same budget by
        // unfreeze (admission consistent across pack and decode).
        let mut body = i32_node(1);
        for _ in 0..299 {
            let mut b = tydesc(&DynType::record("deep", "N", vec![("x", DynType::I32)]));
            b.push(8);
            b.extend_from_slice(&body);
            body = b;
        }
        let wire300 = wire(300, &body);
        let binding = [RecordBinding {
            owner: "deep",
            name: "N",
            fields: vec!["x"],
            ty_index: 1,
        }];
        assert_eq!(
            unfreeze(&wire300, &ctx(&binding, &[]), &TraversalOptions::default()),
            Err(PortableError::Budget {
                code: "av.resource.dyn-depth"
            })
        );
    }

    #[test]
    fn av_dynamic_824_cancel_probe() {
        // 1 + 2×1024 = 2049 nodes: the cancellation flag is polled at the
        // first 1024-node boundary and stops the traversal.
        let mid_ty = DynType::record(
            "big",
            "Mid",
            (0..1024).map(|i| (leak_name(i), DynType::I32)).collect(),
        );
        let root_ty = DynType::record(
            "big",
            "Root",
            vec![("a", mid_ty.clone()), ("b", mid_ty.clone())],
        );
        let val = record(
            2,
            vec![
                record(1, (0..1024).map(|_| Value::I32(1)).collect()),
                record(1, (0..1024).map(|_| Value::I32(1)).collect()),
            ],
        );
        let dv = DynValue::pack(val, root_ty).unwrap();
        // Flag ON from the start: the probe at the first 1024-node
        // boundary stops the traversal deterministically.
        let on = Cell::new(true);
        let cancel = || on.get();
        let mut opts = TraversalOptions::default();
        opts.cancel = Some(&cancel);
        assert_eq!(freeze(&dv, &opts), Err(PortableError::Cancelled));
        // Not a flake of the budget: the same value freezes fine unflagged.
        on.set(false);
        assert!(freeze(&dv, &TraversalOptions::default()).is_ok());
        // Unfreeze-side cancellation on the same bytes.
        let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
        let bindings = [
            RecordBinding {
                owner: "big",
                name: "Root",
                fields: vec!["a", "b"],
                ty_index: 2,
            },
            RecordBinding {
                owner: "big",
                name: "Mid",
                fields: leak_names_1024().to_vec(),
                ty_index: 1,
            },
        ];
        on.set(true);
        assert_eq!(
            unfreeze(&bytes, &ctx(&bindings, &[]), &opts),
            Err(PortableError::Cancelled)
        );
        on.set(false);
        assert!(unfreeze(&bytes, &ctx(&bindings, &[]), &TraversalOptions::default()).is_ok());
    }

    /// The 1024 field names of `Mid` as one `&'static [&'static str]`.
    fn leak_names_1024() -> &'static [&'static str] {
        use std::sync::OnceLock;
        static FIELDS: OnceLock<&'static [&'static str]> = OnceLock::new();
        FIELDS.get_or_init(|| {
            let v: Vec<&'static str> = (0..1024)
                .map(|i| Box::leak(format!("f{}", i).into_boxed_str()) as &'static str)
                .collect();
            Box::leak(v.into_boxed_slice())
        })
    }

    #[test]
    fn av_dynamic_824_traversal_stop_codes_distinct() {
        // The 分态 contract: each stop carries its own stable code.
        assert_eq!(TraversalStop::Nodes.code(), "av.resource.dyn-nodes");
        assert_eq!(TraversalStop::Bytes.code(), "av.resource.dyn-bytes");
        assert_eq!(TraversalStop::Depth.code(), "av.resource.dyn-depth");
        assert_eq!(TraversalStop::Steps.code(), "av.resource.dyn-steps");
        assert_eq!(TraversalStop::Cancelled.code(), "av.dyn.cancelled");
        let take_err = DynTakeError {
            expected: "bool".into(),
            actual: "i32".into(),
            source: DynValue::pack(Value::I32(1), DynType::I32).unwrap(),
        };
        assert_eq!(take_err.code(), "av.dyn.take-type");
    }
}
