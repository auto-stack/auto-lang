//! PLAN-923 V-04/V-05: the checked List-place machine chain over real native
//! PEs — mint -> legal read, stale refusal BEFORE any buffer access after a
//! structural change or element replacement, and legal neighbours staying
//! usable. Mir semantics are the delivered PLAN-822 surface (ListPlace /
//! PlaceGet / PlaceSet); native admission + runtime token registry are the
//! new surface under test.
#[path = "../common/mod.rs"]
mod common;
use ace_av_core::mir::*;
use ace_native::collections_lower::WrapperKnobs;

fn place_fixture(kind: &str) -> MirModule {
    let owner = OwnerScope {
        schema: ace_av_core::contract::MIR_COLLECTIONS_SCHEMA.into(),
        module: "collections923-place".into(),
    };
    let mut m = MirModule::new_collections(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Bool,
        MirType::List(ListTypeRef { elem: 0 }),
        MirType::Place(ListTypeRef { elem: 0 }),
        MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        }),
    ];
    m.consts = vec![
        MirConst::I32(0),
        MirConst::I32(1),
        MirConst::I32(2),
    ];
    // 0 List, 1 c1, 2 c2, 3 i0, 4 i1, 5 placeA, 6 outA, 7 placeB, 8 outB,
    // 9 placeC, 10 outC
    let slot_types = [2, 0, 0, 0, 0, 3, 0, 3, 0, 3, 0];
    let slots = slot_types
        .iter()
        .map(|ty| MirSlot {
            ty: *ty,
            kind: SlotKind::Temp,
            name: None,
        })
        .collect();
    use MirInstr::*;
    let base = vec![
        Const { dst: 1, value: 1 },
        Const { dst: 2, value: 2 },
        Const { dst: 3, value: 0 },
        Const { dst: 4, value: 1 },
        ListNew { dst: 0, list_ty: 2, cap: 1 },
        ListPush { root: 0, value: 1 },
        ListPush { root: 0, value: 2 },
    ];
    let instrs = match kind {
        // mint -> legal read of elem[1] (=2) -> return 2
        "legal-mint-get" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 4 },
                PlaceGet { dst: 6, src: 5 },
            ]);
            v
        }
        // mint elem[0] -> structural push -> old token read traps STALE(27)
        "stale-after-push" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 3 },
                ListPush { root: 0, value: 2 },
                PlaceGet { dst: 6, src: 5 },
            ]);
            v
        }
        // mint elem[1] -> in-place set of that element -> old token traps
        "stale-after-set" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 4 },
                ListSet { root: 0, index: 4, value: 2 },
                PlaceGet { dst: 6, src: 5 },
            ]);
            v
        }
        // sibling legality: replace elem[1]; elem[0]'s token stays legal (1)
        "sibling-legal" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 3 },
                ListSet { root: 0, index: 4, value: 2 },
                PlaceGet { dst: 6, src: 5 },
            ]);
            v
        }
        // remove: mint elem[0] -> structural remove -> old token traps
        "stale-after-remove" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 3 },
                ListRemove { dst: 8, root: 0, index: 3 },
                PlaceGet { dst: 6, src: 5 },
            ]);
            v
        }
        // fresh token after the change is legal: mint B post-set -> get 2
        "fresh-after-change" => {
            let mut v = base;
            v.extend([
                ListSet { root: 0, index: 4, value: 2 },
                ListPlace { dst: 7, base: 0, index: 4 },
                PlaceGet { dst: 8, src: 7 },
            ]);
            v
        }
        // V-04 write path: PlaceSet commits; a fresh token reads the new
        // value (the set's own token is retired by the runtime).
        "legal-place-set" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 4 },
                ListPlace { dst: 7, base: 0, index: 3 },
                PlaceSet { place: 7, value: 2 },
                ListPlace { dst: 9, base: 0, index: 3 },
                PlaceGet { dst: 10, src: 9 },
            ]);
            v
        }
        // Two places on the SAME element: the set retires both the set's
        // own token and the pre-set sibling on that element.
        "set-retires-same-elem-sibling" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 3 },
                ListPlace { dst: 7, base: 0, index: 3 },
                PlaceSet { place: 7, value: 2 },
                PlaceGet { dst: 6, src: 5 },
            ]);
            v
        }
        // V-04/V-05 write-first: a stale PlaceSet traps BEFORE any buffer
        // write (same pre-access validation as PlaceGet). The trailing
        // PlaceGet is unreachable at runtime — it only defines slot 6 so
        // verify accepts the return.
        "stale-set-after-push" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 4 },
                ListPush { root: 0, value: 2 },
                PlaceSet { place: 5, value: 2 },
                PlaceGet { dst: 6, src: 5 },
            ]);
            v
        }
        // V-05 foreign-token defense: after a PlaceSet retires its own
        // token the slot holds EMPTY bits; a later PlaceGet of those bits
        // must refuse INVALID before any dereference (never a dangling
        // pointer read).
        "foreign-token-get" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 4 },
                PlaceSet { place: 5, value: 2 },
                PlaceGet { dst: 6, src: 5 },
            ]);
            v
        }
        // V-05: an injected growth failure (seam 3, second grow after the
        // base's own grow) traps INJECTED with the list and minted place
        // uncommitted; the sweep cleans every token (no leak). The
        // same-process retry that keeps the token legal is the runtime
        // probe's seam-retry (V-02), because a language trap ends the PE.
        "failed-grow-keeps-token" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 4 },
                ListInsert { root: 0, index: 3, value: 1 },
                PlaceGet { dst: 6, src: 5 },
            ]);
            v
        }
        // Two places on two slots stay independent: replacing elem[1]
        // retires only the elem[1] place; the elem[0] place reads on.
        "cross-slot-sibling-legal" => {
            let mut v = base;
            v.extend([
                ListPlace { dst: 5, base: 0, index: 4 },
                ListPlace { dst: 7, base: 0, index: 3 },
                ListSet { root: 0, index: 4, value: 2 },
                PlaceGet { dst: 6, src: 7 },
            ]);
            v
        }
        other => panic!("unknown fixture {other}"),
    };
    let term = match kind {
        "legal-mint-get" => Terminator::Return { value: 6 },
        "sibling-legal" => Terminator::Return { value: 6 },
        "fresh-after-change" => Terminator::Return { value: 8 },
        "legal-place-set" => Terminator::Return { value: 10 },
        "failed-grow-keeps-token" => Terminator::Return { value: 6 },
        "cross-slot-sibling-legal" => Terminator::Return { value: 6 },
        _ => Terminator::Return { value: 6 },
    };
    m.funcs.push(MirFunction {
        owner,
        name: "d_main".into(),
        sig: 4,
        slots,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock { instrs, term }],
    });
    m
}

fn trap_code(run: &ace_native::link_driver::PeRun) -> Option<i64> {
    if run.exit_code != Some(70) {
        return None;
    }
    let text = String::from_utf8_lossy(&run.stderr);
    text.lines().find_map(|line| {
        let rest = line.strip_prefix("MN-COLL-TRAP code=")?;
        rest.parse::<i64>().ok()
    })
}

fn run_place_with_knobs(
    kind: &str,
    knobs: &ace_native::collections_lower::WrapperKnobs,
) -> ace_native::link_driver::PeRun {
    let m = place_fixture(kind);
    ace_av_core::verify_mir(&m).unwrap();
    let units = ace_av::cache::split_units(&m).unwrap();
    common::collections::run_units_with_knobs(
        &units,
        "d_main",
        &format!("place-{kind}"),
        None,
        knobs,
    )
}

fn run_place(kind: &str) -> ace_native::link_driver::PeRun {
    run_place_with_knobs(kind, &WrapperKnobs::default())
}

#[test]
fn place_mint_get_legal_read_and_fresh_token_after_change() {
    let legal = run_place("legal-mint-get");
    assert_eq!(
        legal.exit_code,
        Some(2),
        "{}",
        String::from_utf8_lossy(&legal.stderr)
    );
    common::collections::assert_balanced(&legal);
    // A fresh token minted after the change reads the current value.
    let fresh = run_place("fresh-after-change");
    assert_eq!(
        fresh.exit_code,
        Some(2),
        "{}",
        String::from_utf8_lossy(&fresh.stderr)
    );
    common::collections::assert_balanced(&fresh);
}

#[test]
fn place_stale_traps_before_any_buffer_access_after_struct_change() {
    for kind in ["stale-after-push", "stale-after-remove"] {
        let run = run_place(kind);
        assert_eq!(
            trap_code(&run),
            Some(27),
            "{kind}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        // The trap aborts before the stale buffer was ever read; the runtime
        // cleans every owner/token (live=0, place tokens retired by sweep).
        let text = String::from_utf8_lossy(&run.stderr);
        for field in [
            "MN-COLL live=0",
            "MN-COLL bytes=0",
            "MN-COLL invalid=0",
            "MN-COLL arena_live=0",
        ] {
            assert!(
                text.contains(&format!("{field}\n")),
                "{kind} missing {field}: {text}"
            );
        }
    }
}

#[test]
fn place_element_replace_retires_own_token_but_keeps_siblings_legal() {
    let stale = run_place("stale-after-set");
    assert_eq!(
        trap_code(&stale),
        Some(27),
        "{}",
        String::from_utf8_lossy(&stale.stderr)
    );
    // A sibling element's token stays legal after a different element's
    // in-place replacement (generation is per-element, not per-list).
    let sibling = run_place("sibling-legal");
    assert_eq!(
        sibling.exit_code,
        Some(1),
        "{}",
        String::from_utf8_lossy(&sibling.stderr)
    );
    common::collections::assert_balanced(&sibling);
}

#[test]
fn place_non_copy_element_is_named_rejection_not_silent_widening() {
    // List<Str> element place: native admission must name-reject.
    let owner = OwnerScope {
        schema: ace_av_core::contract::MIR_COLLECTIONS_SCHEMA.into(),
        module: "collections923-place".into(),
    };
    let mut m = MirModule::new_collections(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Bool,
        MirType::Str,
        MirType::List(ListTypeRef { elem: 2 }),
        MirType::Place(ListTypeRef { elem: 2 }),
        MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        }),
    ];
    m.consts = vec![MirConst::I32(0)];
    m.funcs.push(MirFunction {
        owner,
        name: "d_main".into(),
        sig: 5,
        slots: vec![MirSlot {
            ty: 0,
            kind: SlotKind::Temp,
            name: None,
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: 0 }],
            term: Terminator::Return { value: 0 },
        }],
    });
    m.is_unit = true;
    let error = ace_native::collections_lower::admit_unit(&m).unwrap_err();
    assert!(
        matches!(error, ace_native::MnError::Unsupported("place", _)),
        "{error}"
    );
    assert!(error.message().contains("must be Copy"), "{error}");
}

#[test]
fn place_set_commits_and_retires_same_element_sibling() {
    // The set commits; a fresh token reads the new value.
    let legal = run_place("legal-place-set");
    assert_eq!(
        legal.exit_code,
        Some(2),
        "{}",
        String::from_utf8_lossy(&legal.stderr)
    );
    common::collections::assert_balanced(&legal);
    // A sibling minted on the SAME element before the set is retired.
    let stale = run_place("set-retires-same-elem-sibling");
    assert_eq!(
        trap_code(&stale),
        Some(27),
        "{}",
        String::from_utf8_lossy(&stale.stderr)
    );
    let text = String::from_utf8_lossy(&stale.stderr);
    for field in [
        "MN-COLL live=0",
        "MN-COLL bytes=0",
        "MN-COLL invalid=0",
        "MN-COLL arena_live=0",
    ] {
        assert!(
            text.contains(&format!("{field}\n")),
            "missing {field}: {text}"
        );
    }
}

#[test]
fn place_stale_set_traps_before_any_buffer_write() {
    let run = run_place("stale-set-after-push");
    assert_eq!(
        trap_code(&run),
        Some(27),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let text = String::from_utf8_lossy(&run.stderr);
    for field in [
        "MN-COLL live=0",
        "MN-COLL bytes=0",
        "MN-COLL invalid=0",
        "MN-COLL arena_live=0",
    ] {
        assert!(
            text.contains(&format!("{field}\n")),
            "missing {field}: {text}"
        );
    }
}

#[test]
fn place_failed_grow_traps_named_injection_and_cleans_tokens() {
    // Seam 3 (grow): the base fixture's own push already consumed grow #1,
    // so the injected failure lands on the ListInsert's grow (#2). The
    // insert traps INJECTED before any commit; the sweep cleans the minted
    // place token (live=0), never a dangling registry entry. The token's
    // atomicity across the failure (same process, fault cleared, retry
    // reads the old element) is the runtime probe's seam-retry case.
    let knobs = WrapperKnobs {
        fail_seam: Some((ace_native::collections_lower::SEAM_GROW, 2)),
        ..Default::default()
    };
    let run = run_place_with_knobs("failed-grow-keeps-token", &knobs);
    assert_eq!(
        trap_code(&run),
        Some(25),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let text = String::from_utf8_lossy(&run.stderr);
    assert!(text.contains("MN-COLL-INJECT seam=3"), "{text}");
    for field in [
        "MN-COLL live=0",
        "MN-COLL bytes=0",
        "MN-COLL invalid=0",
        "MN-COLL arena_live=0",
    ] {
        assert!(
            text.contains(&format!("{field}\n")),
            "missing {field}: {text}"
        );
    }
}

#[test]
fn place_cross_slot_siblings_stay_independent() {
    // Replacing elem[1] retires only the elem[1] place; the elem[0] place
    // on its own slot reads on (no whole-list generation bump).
    let run = run_place("cross-slot-sibling-legal");
    assert_eq!(
        run.exit_code,
        Some(1),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    common::collections::assert_balanced(&run);
}

#[test]
fn place_foreign_token_is_refused_before_any_dereference() {
    // A slot whose place was retired by PlaceSet holds EMPTY bits — not a
    // registry token. Reading it must trap INVALID(23) (find_place miss),
    // never dereference a dangling pointer; the sweep stays clean.
    let run = run_place("foreign-token-get");
    assert_eq!(
        trap_code(&run),
        Some(23),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let text = String::from_utf8_lossy(&run.stderr);
    for field in [
        "MN-COLL live=0",
        "MN-COLL bytes=0",
        "MN-COLL invalid=0",
        "MN-COLL arena_live=0",
    ] {
        assert!(
            text.contains(&format!("{field}\n")),
            "missing {field}: {text}"
        );
    }
}

#[test]
fn place_static_verify_rejects_illegal_uses_before_runtime() {
    // The static front gate (verify_mir) rejects malformed place uses; the
    // runtime stale trap only ever sees verified MIR. These two layers must
    // stay separate (V-05 static/runtime refusal split).
    let mut get_bad_src = place_fixture("legal-mint-get");
    for f in &mut get_bad_src.funcs {
        for b in &mut f.blocks {
            for i in &mut b.instrs {
                if let MirInstr::PlaceGet { src, .. } = i {
                    *src = 0; // List, not a Place
                }
            }
        }
    }
    assert!(
        ace_av_core::verify_mir(&get_bad_src).is_err(),
        "PlaceGet src must be a Place type"
    );

    let mut set_bad_place = place_fixture("legal-place-set");
    for f in &mut set_bad_place.funcs {
        for b in &mut f.blocks {
            for i in &mut b.instrs {
                if let MirInstr::PlaceSet { place, .. } = i {
                    *place = 0; // List, not a Place
                }
            }
        }
    }
    assert!(
        ace_av_core::verify_mir(&set_bad_place).is_err(),
        "PlaceSet place must be a Place type"
    );

    let mut mint_bad_dst = place_fixture("legal-mint-get");
    for f in &mut mint_bad_dst.funcs {
        for b in &mut f.blocks {
            for i in &mut b.instrs {
                if let MirInstr::ListPlace { dst, .. } = i {
                    *dst = 6; // I32 slot, not a Place
                }
            }
        }
    }
    assert!(
        ace_av_core::verify_mir(&mint_bad_dst).is_err(),
        "ListPlace dst must be a Place type"
    );
}
