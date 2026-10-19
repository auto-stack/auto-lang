//! PLAN-796 T-02 contract test surface (`sema_identity_` filter): the
//! complete meta/identity/codec coverage — nominal refs, restart
//! order-independence, recursive groups, typed views, decode bounds and
//! the category-matrix invariants (V-01 / AC-01).

use ac_sema_contract::codec::{
    decode_page, encode_page, CachedResult, CodecError, DepFingerprint, DepManifest, FieldDesc,
    FnMeta, InferFacts, Mode, ModuleIndex, NegDep, PageBody, PageHeader, PageKind, PageView,
    RecGroup, RecordMeta, RecordShape, ResultFacts, SemaPage, TypeDesc, MAX_NESTING_DEPTH,
    SEMA_CODEC_VERSION, SEMA_PAGE_SCHEMA_V2,
};
use ac_sema_contract::{
    logical_id, ContentVersion, DeclKind, DeclRef, PackageId, SEMA_PRODUCER_ID,
};

fn pkg() -> PackageId {
    PackageId::new("test.pkg")
}

fn decl(name: &str) -> DeclRef {
    DeclRef::new(
        &pkg(),
        "m",
        DeclKind::Record,
        name,
        ContentVersion::of_canonical(name.as_bytes()),
    )
}

fn fndecl(name: &str) -> DeclRef {
    DeclRef::new(
        &pkg(),
        "m",
        DeclKind::Fn,
        name,
        ContentVersion::of_canonical(name.as_bytes()),
    )
}

fn index_page() -> SemaPage {
    SemaPage {
        header: PageHeader {
            schema: SEMA_PAGE_SCHEMA_V2.into(),
            producer: SEMA_PRODUCER_ID.into(),
            codec_version: SEMA_CODEC_VERSION,
            kind: PageKind::ModuleIndex,
            content: ContentVersion::of_canonical(b"c"),
        },
        body: PageBody::ModuleIndex(ModuleIndex {
            package: pkg(),
            module: "m".into(),
            fn_names: vec![],
            record_names: vec![],
            decl_versions: vec![],
        }),
    }
}

/// The full description set (§5.1, for 797's consumption) round-trips.
#[test]
fn sema_identity_typedesc_full_set_roundtrip() {
    let t = TypeDesc::Fn {
        params: vec![
            TypeDesc::Unit,
            TypeDesc::I32,
            TypeDesc::U32,
            TypeDesc::I64,
            TypeDesc::U64,
            TypeDesc::F64,
            TypeDesc::Bool,
            TypeDesc::Char,
            TypeDesc::ByteBuffer,
            TypeDesc::Opaque {
                name: "extern".into(),
            },
        ],
        ret: Box::new(TypeDesc::Record { decl: decl("Pair") }.with_mode(Mode::Owned)),
    };
    let bytes = ac_sema_contract::codec::encode_typedesc_bytes(&t).unwrap();
    let back = ac_sema_contract::codec::decode_typedesc_bytes(&bytes).unwrap();
    assert_eq!(t, back);
}

/// Recursive records compose through INDIRECT refs into a verifiable
/// group (A.next: B, B.prev: A); the group content version is
/// member-order free.
#[test]
fn sema_identity_recgroup_indirect_cycle_verifiable() {
    let a = RecordShape {
        decl: decl("A"),
        fields: vec![FieldDesc {
            name: "next".into(),
            ty: TypeDesc::Record { decl: decl("B") },
        }],
    };
    let b = RecordShape {
        decl: decl("B"),
        fields: vec![FieldDesc {
            name: "prev".into(),
            ty: TypeDesc::Record { decl: decl("A") },
        }],
    };
    let g1 = RecGroup {
        members: vec![a.clone(), b.clone()],
    };
    let g2 = RecGroup {
        members: vec![b.clone(), a.clone()],
    }; // rotated: same group
    assert_eq!(
        g1.group_content(),
        g2.group_content(),
        "group version is member-order free"
    );
    assert!(g1.verify().is_ok());
    // duplicate member → controlled rejection, not a panic (V-01)
    let bad = RecGroup {
        members: vec![a.clone(), a],
    };
    assert!(bad.verify().is_err());
    // content change in ONE member changes the group version
    let mut c = g1.clone();
    c.members[0].fields.push(FieldDesc {
        name: "extra".into(),
        ty: TypeDesc::I32,
    });
    assert_ne!(g1.group_content(), c.group_content());
}

/// Field order is declaration order and SURVIVES round-trip — order is
/// language-observable (774 D-05), never canonicalized for hashing.
#[test]
fn sema_identity_recordmeta_preserves_field_order() {
    let meta = RecordMeta {
        shape: RecordShape {
            decl: decl("T"),
            fields: vec![
                FieldDesc {
                    name: "zeta".into(),
                    ty: TypeDesc::I32,
                },
                FieldDesc {
                    name: "alpha".into(),
                    ty: TypeDesc::Bool,
                },
            ],
        },
        methods: vec![FnMeta {
            decl: fndecl("get"),
            params: vec![("self".into(), TypeDesc::I32, Mode::View)],
            ret: TypeDesc::I32,
            receiver: "view",
        }],
        group_content: ContentVersion::of_canonical(b"g"),
    };
    let mut page = index_page();
    page.header.kind = PageKind::RecordMeta;
    page.body = PageBody::RecordMeta(meta);
    let bytes = encode_page(&page).unwrap();
    let back = decode_page(&bytes).unwrap();
    match &back.body {
        PageBody::RecordMeta(m) => {
            let names: Vec<&str> = m.shape.fields.iter().map(|f| f.name.as_str()).collect();
            assert_eq!(names, vec!["zeta", "alpha"], "declaration order preserved");
            assert_eq!(m.shape.decl.name, "T");
        }
        _ => panic!("wrong body kind"),
    }
    // deterministic: re-encode of the decoded page is byte-identical
    assert_eq!(encode_page(&back).unwrap(), bytes);
}

/// Deep nesting: 4096 levels decode fine (iteratively); beyond the cap
/// is a controlled TooDeep rejection — never a stack overflow on
/// hostile bytes.
#[test]
fn sema_identity_decode_deep_nesting_bounds() {
    fn mode_bytes(depth: usize) -> Vec<u8> {
        // tag 13 (Mode) + mode byte 1 (View), repeated, then I32
        let mut v = Vec::with_capacity(depth * 2 + 1);
        for _ in 0..depth {
            v.extend_from_slice(&[13, 1]);
        }
        v.push(2);
        v
    }
    let ok = mode_bytes(4096);
    assert!(
        ac_sema_contract::codec::decode_typedesc_bytes(&ok).is_ok(),
        "4096 levels must decode"
    );
    let too = mode_bytes((MAX_NESTING_DEPTH + 1) as usize);
    match ac_sema_contract::codec::decode_typedesc_bytes(&too) {
        Err(CodecError::TooDeep { .. }) => {}
        other => panic!("expected TooDeep, got {other:?}"),
    }
}

#[test]
fn sema_identity_decode_truncated_and_trailing_rejected() {
    let bytes =
        ac_sema_contract::codec::encode_typedesc_bytes(&TypeDesc::Opaque { name: "x".into() })
            .unwrap();
    assert!(matches!(
        ac_sema_contract::codec::decode_typedesc_bytes(&bytes[..bytes.len() - 1]),
        Err(CodecError::Truncated { .. })
    ));
    let mut bytes = encode_page(&index_page()).unwrap();
    bytes.push(0);
    assert!(matches!(
        decode_page(&bytes),
        Err(CodecError::TrailingBytes(_))
    ));
}

/// Wrong envelope owner/schema/codec-version are controlled admission
/// failures (never reinterpretation).
#[test]
fn sema_identity_page_envelope_admission() {
    let mut page = index_page();
    page.header.producer = "some-other-producer/9".into();
    assert!(matches!(
        encode_page(&page),
        Err(CodecError::BadProducer { .. })
    ));
    page.header.producer = SEMA_PRODUCER_ID.into();
    page.header.schema = "ac796/wrong/v9".into();
    assert!(matches!(
        encode_page(&page),
        Err(CodecError::BadSchema { .. })
    ));
    // decode-side codec-version drift: bump the version byte after the
    // two length-prefixed strings
    page.header.schema = SEMA_PAGE_SCHEMA_V2.into();
    let mut bytes = encode_page(&page).unwrap();
    let schema_len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let off = 4 + schema_len;
    let prod_len = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()) as usize;
    let cv = off + 4 + prod_len;
    bytes[cv] = 9;
    assert!(matches!(
        decode_page(&bytes),
        Err(CodecError::UnsupportedCodec(_))
    ));
}

/// Typed view accessors are KIND-CHECKED: reading a FnMeta out of a
/// ModuleIndex page is a controlled error, not a reinterpretation.
#[test]
fn sema_identity_view_kind_checked_access() {
    let page = index_page();
    let v = PageView::new(&page);
    assert!(v.try_as_fn_meta().is_err());
    assert!(v.try_as_record_meta().is_err());
    assert!(
        v.field("x").is_none(),
        "field lookup on a non-record page is None"
    );
}

/// CachedResult pages round-trip with manifest + facts intact
/// (positive and negative dependencies).
#[test]
fn sema_identity_cached_result_roundtrip_with_manifest() {
    let target = logical_id(&pkg(), "m", DeclKind::Fn, "main");
    let mut page = index_page();
    page.header.kind = PageKind::CachedResult;
    page.body = PageBody::CachedResult(CachedResult {
        query: "infer-body",
        target: target.clone(),
        facts: ResultFacts::Infer(InferFacts {
            fn_logical: target,
            bindings: vec![("x".into(), TypeDesc::I32)],
            expr_types: vec![(3, 9, TypeDesc::Bool)],
            diagnostics: vec![],
        }),
        manifest: DepManifest {
            positives: vec![DepFingerprint {
                kind: "fn-sig",
                origin: "helper".into(),
                fp: "abc".into(),
            }],
            negatives: vec![NegDep {
                domain: "fn",
                name: "g".into(),
                set_fp: "def".into(),
            }],
        },
    });
    let bytes = encode_page(&page).unwrap();
    let back = decode_page(&bytes).unwrap();
    assert_eq!(back, page);
}

/// The registered matrix is complete and self-consistent: unique ids,
/// schemas for implemented/schema-only rows, owners for future rows
/// (AC-01 auditable coverage).
#[test]
fn sema_identity_matrix_invariants_hold() {
    use ac_sema_contract::metadata::{matrix_invariants, MetaSupport, CATEGORY_MATRIX};
    assert_eq!(matrix_invariants(), Ok(()));
    // the v0.6-committed categories the plan names explicitly are all
    // registered (declarations, types, fields, methods, constraints,
    // attributes, source, ownership, effect, CT, generated, runtime
    // projection, FFI)
    for id in [
        "decl.fn",
        "decl.record",
        "decl.scope.import",
        "type.logical",
        "type.fields",
        "fn.body.infer",
        "generic.params.constraints",
        "generic.instances",
        "visibility.attributes",
        "source.provenance",
        "ownership.effect",
        "ct.request.result",
        "generated.items",
        "runtime.projection",
        "ffi.interface",
    ] {
        assert!(
            ac_sema_contract::metadata::category(id).is_some(),
            "missing category row {id}"
        );
    }
    // support levels are honest: implemented rows registered; future
    // rows never claim a schema (nothing reads as full-Auto reflection)
    let implemented = CATEGORY_MATRIX
        .iter()
        .filter(|c| c.support == MetaSupport::Implemented)
        .count();
    assert!(
        implemented >= 6,
        "the probe's implemented surface is registered"
    );
    for c in CATEGORY_MATRIX {
        if c.support == MetaSupport::Future {
            assert_eq!(c.schema, "-", "future row {} must not claim a schema", c.id);
        }
        if c.support == MetaSupport::SchemaOnly {
            assert_ne!(c.schema, "-");
        }
    }
}

/// F-796-R11-01: a method's nominal identity includes its OWNING record —
/// `A.get`, `B.get` and a top-level `get` in the same module are three
/// DISTINCT declarations; the id is stable across mint order/restarts,
/// tampered owners are rejected, and an ownerless method cannot verify.
#[test]
fn sema_identity_r11_method_owner_is_part_of_the_nominal_identity() {
    use ac_sema_contract::method_logical_id;

    let a_get = method_logical_id(&pkg(), "m", "A", "get");
    let b_get = method_logical_id(&pkg(), "m", "B", "get");
    let top_get = logical_id(&pkg(), "m", DeclKind::Fn, "get");
    assert_ne!(a_get, b_get, "A.get and B.get must not share a logical id");
    assert_ne!(
        a_get, top_get,
        "A.get and top-level get must not share a logical id"
    );
    assert_ne!(
        b_get, top_get,
        "B.get and top-level get must not share a logical id"
    );
    // the three-way distinctness is not an accident of one name: same
    // owner, different method names stay distinct too
    assert_ne!(
        method_logical_id(&pkg(), "m", "A", "get"),
        method_logical_id(&pkg(), "m", "A", "set"),
    );

    // stability: minting is a pure function of the nominal path
    assert_eq!(a_get, method_logical_id(&pkg(), "m", "A", "get"));
    // a different module is a different declaration
    assert_ne!(
        a_get,
        method_logical_id(&pkg(), "other", "A", "get"),
        "module is part of the method identity"
    );

    let decl = DeclRef::new_method(
        &pkg(),
        "m",
        "A",
        "get",
        ContentVersion::of_canonical(b"fn get() i32"),
    );
    assert_eq!(decl.kind, DeclKind::Method);
    assert_eq!(decl.owner.as_deref(), Some("A"));
    assert_eq!(decl.logical, a_get);
    assert!(decl.verify().is_ok(), "a minted method ref verifies");

    // tampered owner: the stored logical no longer matches the components
    let mut tampered = decl.clone();
    tampered.owner = Some("B".into());
    assert!(
        tampered.verify().is_err(),
        "swapping the owner must break verify"
    );

    // ownerless method: cannot re-derive at all
    let mut ownerless = decl.clone();
    ownerless.owner = None;
    assert!(
        ownerless.verify().is_err(),
        "a method without an owner cannot verify"
    );

    // a Fn-kind ref with a stray owner is likewise rejected (the owner
    // only ever belongs to Method)
    let mut stray = fndecl("get");
    stray.owner = Some("A".into());
    assert!(
        stray.verify().is_err(),
        "a Fn ref with an owner cannot verify"
    );
}

/// F-796-R11-01: the method owner survives the page codec — encode a
/// RecordMeta carrying methods, decode it back, and the kind/owner/
/// logical identity is preserved (old v1 payloads with ownerless method
/// decls are unreachable under the v2 schema/codec axes).
#[test]
fn sema_identity_r11_method_declref_codec_roundtrip() {
    let method = |owner: &str, name: &str| {
        DeclRef::new_method(
            &pkg(),
            "m",
            owner,
            name,
            ContentVersion::of_canonical(format!("fn {owner}.{name}").as_bytes()),
        )
    };
    let page = SemaPage {
        header: PageHeader {
            schema: SEMA_PAGE_SCHEMA_V2.into(),
            producer: SEMA_PRODUCER_ID.into(),
            codec_version: SEMA_CODEC_VERSION,
            kind: PageKind::RecordMeta,
            content: ContentVersion::of_canonical(b"rec"),
        },
        body: PageBody::RecordMeta(RecordMeta {
            shape: RecordShape {
                decl: decl("A"),
                fields: vec![FieldDesc {
                    name: "value".into(),
                    ty: TypeDesc::I32,
                }],
            },
            methods: vec![
                FnMeta {
                    decl: method("A", "get"),
                    params: vec![],
                    ret: TypeDesc::I32,
                    receiver: "view",
                },
                FnMeta {
                    decl: method("A", "set"),
                    params: vec![("value".into(), TypeDesc::I32, Mode::View)],
                    ret: TypeDesc::Unit,
                    receiver: "view",
                },
            ],
            group_content: ContentVersion::of_canonical(b"group"),
        }),
    };
    let bytes = encode_page(&page).expect("encode");
    let back = decode_page(&bytes).expect("decode");
    let PageBody::RecordMeta(m) = back.body else {
        panic!("record meta page decoded into another kind");
    };
    assert_eq!(m.methods.len(), 2);
    for (got, owner, name) in [(&m.methods[0], "A", "get"), (&m.methods[1], "A", "set")] {
        assert_eq!(got.decl.kind, DeclKind::Method, "{name} kind survives");
        assert_eq!(
            got.decl.owner.as_deref(),
            Some(owner),
            "{name} owner survives"
        );
        assert_eq!(got.decl.name, name);
        assert!(got.decl.verify().is_ok(), "{name} verifies after roundtrip");
    }
    assert_ne!(
        m.methods[0].decl.logical, m.methods[1].decl.logical,
        "same-owner methods stay distinct"
    );
    assert_ne!(
        m.methods[0].decl.logical,
        logical_id(&pkg(), "m", DeclKind::Fn, "get"),
        "the method is not the top-level fn"
    );
}
