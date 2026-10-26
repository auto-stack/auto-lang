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
    use crate::contract::{fnv1a64, RecordValue, StrHandle, StrView, StrViewSrc, Value};
    use crate::dynamic::{
        DynPackError, DynTakeError, DynType, DynValue, TraversalOptions, TraversalStop,
        MAX_DYN_NODES,
    };
    use crate::owned_text::StrArena;
    use crate::portable::{
        freeze, unfreeze, PortableError, RecordBinding, RestoreContext, PORTABLE_MAGIC,
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
                fields: &["id", "name", "point"],
                ty_index: 3,
            },
            RecordBinding {
                owner: "demo",
                name: "Point",
                fields: &["x", "y"],
                ty_index: 7,
            },
        ];
        (root_val, root_ty, bindings)
    }

    fn pack_demo() -> DynValue {
        let (v, t, _) = demo_tree();
        DynValue::pack(v, t).expect("demo tree is whitelisted")
    }

    fn default_ctx<'a>(bindings: &'a [RecordBinding<'a>]) -> RestoreContext<'a> {
        RestoreContext::new(bindings)
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
            let back = unfreeze(&bytes, &default_ctx(&[]), &TraversalOptions::default()).unwrap();
            assert!(back.is(&t), "type survived for {:?}", v);
            assert_eq!(back.as_value(), &v, "bit-exact roundtrip for {:?}", v);
        }
    }

    #[test]
    fn av_dynamic_824_portable_roundtrip_str_char() {
        for s in ["", "auto", "héllo→中文🦀", "e\u{0301}"] {
            let dv = DynValue::pack(strv(s), DynType::Str).unwrap();
            let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
            let back = unfreeze(&bytes, &default_ctx(&[]), &TraversalOptions::default()).unwrap();
            assert_eq!(back.as_value(), &strv(s), "Str bytes for {:?}", s);
        }
        for c in ['a', '中', '\u{10FFFF}', '\u{0301}'] {
            let dv = DynValue::pack(Value::Char(c), DynType::Char).unwrap();
            let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
            let back = unfreeze(&bytes, &default_ctx(&[]), &TraversalOptions::default()).unwrap();
            assert_eq!(back.as_value(), &Value::Char(c));
        }
    }

    #[test]
    fn av_dynamic_824_portable_roundtrip_nested_records() {
        let dv = pack_demo();
        let (_, _, bindings) = demo_tree();
        let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
        let back = unfreeze(
            &bytes,
            &default_ctx(&bindings),
            &TraversalOptions::default(),
        )
        .unwrap();
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
        // Same-width DIFFERENT-name record never matches (820 nominal rule).
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
        let back = unfreeze(
            &bytes,
            &default_ctx(&bindings),
            &TraversalOptions::default(),
        )
        .unwrap();
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
        // A Str-typed take of an i32 pack is a checked failure too.
        let s = DynValue::pack(strv("x"), DynType::Str).unwrap();
        assert!(s.take_as(&DynType::I32).is_err());
    }

    #[test]
    fn av_dynamic_824_deep_clone_independent_expands_sharing() {
        // Shared DAG (legal in-memory 820 pure-value sharing): one Point Rc
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
        assert_ne!(Rc::as_ptr(a), Rc::as_ptr(b), "sharing was expanded");
        // The ORIGINAL still shares (untouched).
        let (oa, ob) = match dv.as_value() {
            Value::Record(r) => match (&r.fields[0], &r.fields[1]) {
                (Value::Record(a), Value::Record(b)) => (a, b),
                _ => panic!(),
            },
            _ => panic!(),
        };
        assert_eq!(Rc::as_ptr(oa), Rc::as_ptr(ob));
        // Str deep clone mints an independent detached buffer.
        let s = DynValue::pack(strv("payload"), DynType::Str).unwrap();
        let sc = s.deep_clone(&TraversalOptions::default()).unwrap();
        assert_eq!(sc.as_value(), s.as_value());
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
        let err = unfreeze(&bytes, &default_ctx(&bindings), &opts).unwrap_err();
        assert_eq!(
            err,
            PortableError::Budget {
                code: "av.resource.dyn-nodes"
            }
        );
        // The bytes are NOT consumed: a clean retry fully succeeds.
        let back = unfreeze(
            &bytes,
            &default_ctx(&bindings),
            &TraversalOptions::default(),
        )
        .unwrap();
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
        // buffer IS the deterministic rejection (the fault seam never fires).
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
    fn av_dynamic_824_reject_nested_borrows() {
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
        // Direct pack of a view / a MutRef: same admission.
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
    }

    #[test]
    fn av_dynamic_824_type_surface_closed_no_fn_or_list() {
        // Exhaustive match WITHOUT a wildcard arm: `DynType` provably has
        // no Fn/closure/borrow/List/Variant category — the whitelist is
        // closed by construction (V-12: no fake future categories).
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
            }
        }
        assert_eq!(kind(&DynType::Str), "scalar");
        assert_eq!(kind(&DynType::record("o", "R", vec![])), "record");
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
        // Arity mismatch between value and declared type.
        let ty = DynType::record("demo", "R2", vec![("a", DynType::I32)]);
        let err = DynValue::pack(record(1, vec![]), ty).unwrap_err();
        assert!(matches!(err, DynPackError::TypeMismatch { .. }));
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

    fn scalar_body(v: i32) -> Vec<u8> {
        let mut b = vec![0u8];
        b.extend_from_slice(&v.to_le_bytes());
        b
    }

    fn record_node(owner: &str, name: &str, fields: &[&str], children: &[&[u8]]) -> Vec<u8> {
        let mut b = vec![8u8];
        b.extend_from_slice(&(owner.len() as u16).to_le_bytes());
        b.extend_from_slice(owner.as_bytes());
        b.extend_from_slice(&(name.len() as u16).to_le_bytes());
        b.extend_from_slice(name.as_bytes());
        b.extend_from_slice(&(fields.len() as u16).to_le_bytes());
        for f in fields {
            b.extend_from_slice(&(f.len() as u16).to_le_bytes());
            b.extend_from_slice(f.as_bytes());
        }
        for c in children {
            b.extend_from_slice(c);
        }
        b
    }

    #[test]
    fn av_dynamic_824_wire_header_negatives() {
        let good_body = scalar_body(7);
        let good = wire(1, &good_body);
        let ctx = default_ctx(&[]);
        // Bad magic.
        let mut m = good.clone();
        m[0] = b'X';
        m.truncate(m.len() - 8);
        let sum = fnv1a64(&m);
        m.extend_from_slice(&sum.to_le_bytes());
        assert_eq!(
            unfreeze(&m, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "magic" })
        );
        // Bad version (2) — a different format generation.
        let mut v = good.clone();
        v[4] = 2;
        v.truncate(v.len() - 8);
        let sum = fnv1a64(&v);
        v.extend_from_slice(&sum.to_le_bytes());
        assert_eq!(
            unfreeze(&v, &ctx, &TraversalOptions::default()),
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
                unfreeze(&g, &ctx, &TraversalOptions::default()),
                Err(PortableError::Invalid { reason }),
                "header byte {}",
                pos
            );
        }
        // Declared node count above the ceiling: resource, before parsing.
        let big = wire(MAX_DYN_NODES + 1, &good_body);
        assert_eq!(
            unfreeze(&big, &ctx, &TraversalOptions::default()),
            Err(PortableError::Budget {
                code: "av.resource.dyn-nodes"
            })
        );
    }

    #[test]
    fn av_dynamic_824_wire_payload_negatives() {
        let ctx = default_ctx(&[]);
        // Unknown tag.
        let t = wire(1, &[9u8, 1, 0, 0, 0]);
        assert_eq!(
            unfreeze(&t, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "tag" })
        );
        // Bool domain: 2 is neither false nor true.
        let b = wire(1, &[1u8, 2]);
        assert_eq!(
            unfreeze(&b, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "bool-domain"
            })
        );
        // Char domain: a UTF-16 surrogate is not a Unicode scalar.
        let mut cbody = vec![6u8];
        cbody.extend_from_slice(&0xD800u32.to_le_bytes());
        let c = wire(1, &cbody);
        assert_eq!(
            unfreeze(&c, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "char-domain"
            })
        );
        // Bad UTF-8 in a Str payload.
        let mut sbody = vec![7u8];
        sbody.extend_from_slice(&2u32.to_le_bytes());
        sbody.extend_from_slice(&[0xFF, 0xFE]);
        let s = wire(1, &sbody);
        assert_eq!(
            unfreeze(&s, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "utf8" })
        );
        // Truncated mid-node and missing fields.
        let half = wire(1, &[0u8, 7]);
        assert_eq!(
            unfreeze(&half, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "truncated"
            })
        );
        let binding = [RecordBinding {
            owner: "d",
            name: "R",
            fields: &["a"],
            ty_index: 1,
        }];
        let rctx = default_ctx(&binding);
        let missing = wire(
            2,
            &record_node("d", "R", &["a"], &[]), // child never arrives
        );
        assert_eq!(
            unfreeze(&missing, &rctx, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "truncated"
            })
        );
        // Duplicate field names in one record header.
        let dupf = wire(
            2,
            &record_node("d", "R", &["a", "a"], &[&scalar_body(1), &scalar_body(2)]),
        );
        assert_eq!(
            unfreeze(&dupf, &rctx, &TraversalOptions::default()),
            Err(PortableError::Invalid {
                reason: "duplicate-field"
            })
        );
        // Trailing bytes after a complete root.
        let mut trail = wire(1, &scalar_body(1));
        let body_len = trail.len() - 8;
        trail.insert(body_len, 0);
        trail.truncate(trail.len() - 8);
        let sum = fnv1a64(&trail);
        trail.extend_from_slice(&sum.to_le_bytes());
        assert_eq!(
            unfreeze(&trail, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "trailing" })
        );
    }

    #[test]
    fn av_dynamic_824_wire_checksum_and_node_count_integrity() {
        let dv = pack_demo();
        let (_, _, bindings) = demo_tree();
        let ctx = default_ctx(&bindings);
        let good = freeze(&dv, &TraversalOptions::default()).unwrap();
        assert!(unfreeze(&good, &ctx, &TraversalOptions::default()).is_ok());
        // Good structure + bad checksum: flip only the trailer.
        let mut badsum = good.clone();
        let last = badsum.len() - 1;
        badsum[last] ^= 0xFF;
        assert_eq!(
            unfreeze(&badsum, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "checksum" })
        );
        // Bad structure with a RE-COMPUTED checksum is still rejected by
        // the structural gate (a good hash never launders bad bytes).
        let mut badtag = good.clone();
        badtag[HEADER] = 9; // unknown tag at the root
        const HEADER: usize = 16;
        badtag.truncate(badtag.len() - 8);
        let sum = fnv1a64(&badtag);
        badtag.extend_from_slice(&sum.to_le_bytes());
        assert_eq!(
            unfreeze(&badtag, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "tag" })
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
                unfreeze(&lie, &ctx, &TraversalOptions::default()),
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
            unfreeze(&flip, &ctx, &TraversalOptions::default()),
            Err(PortableError::Invalid { reason: "checksum" })
        );
    }

    #[test]
    fn av_dynamic_824_admission_unknown_and_layout_mismatch() {
        let dv = pack_demo();
        let (_, _, bindings) = demo_tree();
        let bytes = freeze(&dv, &TraversalOptions::default()).unwrap();
        // No bindings at all: unknown nominal type at the decode gate.
        let empty = default_ctx(&[]);
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
            fields: &["id", "title", "point"], // was "name"
            ty_index: 3,
        }];
        assert_eq!(
            unfreeze(&bytes, &default_ctx(&renamed), &TraversalOptions::default()),
            Err(PortableError::Admission {
                code: "portable.layout-mismatch"
            })
        );
        // Reordered fields likewise.
        let reordered = [RecordBinding {
            owner: "demo",
            name: "Root",
            fields: &["name", "id", "point"],
            ty_index: 3,
        }];
        assert_eq!(
            unfreeze(
                &bytes,
                &default_ctx(&reordered),
                &TraversalOptions::default()
            ),
            Err(PortableError::Admission {
                code: "portable.layout-mismatch"
            })
        );
        // Nothing was consumed: the correct context still restores.
        let back = unfreeze(
            &bytes,
            &default_ctx(&bindings),
            &TraversalOptions::default(),
        )
        .unwrap();
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
            unfreeze(&padded, &default_ctx(&[]), &TraversalOptions::default()),
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
        let mut body = scalar_body(1);
        for _ in 0..299 {
            body = record_node("deep", "N", &["x"], &[&body]);
        }
        let wire300 = wire(300, &body);
        let binding = [RecordBinding {
            owner: "deep",
            name: "N",
            fields: &["x"],
            ty_index: 1,
        }];
        assert_eq!(
            unfreeze(
                &wire300,
                &default_ctx(&binding),
                &TraversalOptions::default()
            ),
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
                fields: &["a", "b"],
                ty_index: 2,
            },
            RecordBinding {
                owner: "big",
                name: "Mid",
                fields: &leak_names_1024(),
                ty_index: 1,
            },
        ];
        on.set(true);
        assert_eq!(
            unfreeze(&bytes, &default_ctx(&bindings), &opts),
            Err(PortableError::Cancelled)
        );
        on.set(false);
        assert!(unfreeze(
            &bytes,
            &default_ctx(&bindings),
            &TraversalOptions::default()
        )
        .is_ok());
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
