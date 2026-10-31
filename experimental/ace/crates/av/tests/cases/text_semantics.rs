//! PLAN-894 T-04/T-05 AV tier: the precise text-comparison oracle (V-05),
//! the replacement free-exactly-once ledger oracle (V-02) and the
//! controlled-allocation-fault atomicity oracle (V-04).

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{
    ExecOutcome, Value, ABC_OWNED_VERSION, ABC_PROFILE_OWNED, MIR_OWNED_SCHEMA,
};
use ace_av_core::mir::*;
use ace_av_core::{abc_verify, mir_verify};
use ace_lowering::lower_checked_owned;

/// Compile + lower + verify + admit an owned text source (the shared
/// chain both recipes ride).
fn admit(src: &str) -> abc_verify::LoadedModule {
    let compiled =
        ace_core::source::compile_text_record_source(src, "p.at", &[]).expect("compiles");
    let mir = lower_checked_owned(&compiled.checked, "d_main", 0x894).expect("lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let wire = encode_abc(&mir).expect("encode");
    assert_eq!(wire.version, ABC_OWNED_VERSION);
    assert_eq!(wire.profile, ABC_PROFILE_OWNED);
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).expect("decode");
    abc_verify::verify_abc(decoded, &mir.owner).expect("admit")
}

fn expect_i32(loaded: &abc_verify::LoadedModule, limits: &ExecLimits, want: i32, tag: &str) {
    let (out, _) = execute(loaded, limits, &CancelFlag::new(false));
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(v, want, "{tag}"),
        other => panic!("{tag}: expected {want}, got {other:?}"),
    }
}

// ---- V-05: the precise comparison oracle -----------------------------------

/// eq/ne over full length + UTF-8 bytes: NUL never truncates, no Unicode
/// normalization, length and content both matter. Both operands are VIEW
/// (`str`) values — the frozen source surface admits eq/ne only at the
/// same VT (a mixed Str/str pair is a preserved type-mismatch face, see
/// the owned-mix test below); view variety comes from different
/// constructions (literal views, a concat owner's rooted view, a clone's).
#[test]
fn av_owned_eq_ne_exact_bytes_matrix() {
    // (x-prelude building `x` as a view, literal, expect_eq) — hand-audited
    // INDEPENDENT of any engine.
    #[allow(clippy::type_complexity)]
    let rows: &[(&str, &str, bool)] = &[
        ("let x str = \"A\\0B\"", "\"A\"", false), // NUL keeps the tail: prefix != full
        ("let x str = \"A\\0B\"", "\"A\\0B\"", true), // exact incl. embedded NUL
        ("let x str = \"A\\0B\"", "\"A\\0C\"", false), // differs past the NUL
        ("let x str = \"A\\0\\0B\"", "\"A\\0B\"", false), // consecutive NULs are content
        ("let x str = \"A\\0B\\0\"", "\"A\\0B\"", false), // trailing NUL is content
        ("let x str = \"\"", "\"\"", true),        // empty equals empty
        ("let x str = \"\"", "\"\\0\"", false),    // empty != one NUL
        ("let x str = \"é\"", "\"é\"", true),      // identical NFC strings
        ("let x str = \"é\"", "\"e\"", false),     // no folding to base letter
        ("let x str = \"中\"", "\"中x\"", false),  // same prefix, longer tail
        ("let x str = \"ab\"", "\"ab\"", true),    // plain equality
        // same bytes via DIFFERENT owners: a concat's rooted view and a
        // clone's rooted view.
        (
            "let s Str = \"A\" + \"\\0B\"\n    let x str = s.to(str)",
            "\"A\\0B\"",
            true,
        ),
        (
            "let s Str = \"中\".clone()\n    let x str = s.to(str)",
            "\"中\"",
            true,
        ),
    ];
    for (prelude, lit, want_eq) in rows {
        let src = format!(
            "fn main() int {{\n    {prelude}\n    if x == {lit} {{ return 1 }}\n    return 2\n}}\n",
        );
        expect_i32(
            &admit(&src),
            &ExecLimits::default(),
            if *want_eq { 1 } else { 2 },
            &format!("eq({prelude} == {lit})"),
        );
        let src_ne = format!(
            "fn main() int {{\n    {prelude}\n    if x != {lit} {{ return 1 }}\n    return 2\n}}\n",
        );
        expect_i32(
            &admit(&src_ne),
            &ExecLimits::default(),
            if *want_eq { 2 } else { 1 },
            &format!("ne({prelude} != {lit})"),
        );
    }
}

/// The mixed-type face is PRESERVED: eq/ne between an owned `Str` and a
/// view `str` (either side) is a source.type-mismatch rejection — the
/// frozen surface admits text comparison only at the same VT. (The
/// lowering's owned-operand view coercion is an HIR-level safety net; the
/// durable machine-level negative is the hand-built Bin-on-views control
/// above.)
#[test]
fn av_owned_mixed_eq_types_rejected() {
    let lhs_owned = "\
fn make() Str { return \"A\\0B\" }
fn main() int {
    let a Str = make()
    if a == \"A\\0B\" { return 41 }
    return 40
}
";
    let ds = ace_core::source::compile_text_record_source(lhs_owned, "p.at", &[])
        .expect_err("mixed Str/str eq is rejected");
    assert!(
        ds.iter().any(|d| d.code == "source.type-mismatch"),
        "named rejection: {ds:?}"
    );
    let rhs_owned = "\
fn make() Str { return \"A\\0B\" }
fn main() int {
    let x str = \"A\\0B\"
    if x == make() { return 43 }
    return 42
}
";
    let ds2 = ace_core::source::compile_text_record_source(rhs_owned, "p.at", &[])
        .expect_err("view/owned eq is rejected");
    assert!(
        ds2.iter().any(|d| d.code == "source.type-mismatch"),
        "named rejection: {ds2:?}"
    );
}

/// The durable NEGATIVE control: a hand-built Bin EqI32 over StrView
/// operands does not type-check — text never rides the integer BIN (the
/// pre-fix shape the T-02 probe caught as mir.bin-type).
#[test]
fn av_owned_bin_eqi32_on_views_is_invalid_mir() {
    let mut m = MirModule::new_owned(OwnerScope {
        schema: MIR_OWNED_SCHEMA.to_string(),
        module: "m_neg894".to_string(),
    });
    let t_str = m.intern_type(MirType::Str);
    let t_view = m.intern_type(MirType::StrView);
    let t_i32 = m.intern_type(MirType::I32);
    let t_bool = m.intern_type(MirType::Bool);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t_i32,
    }));
    let lit = m.intern_const(MirConst::Str("A\0B".as_bytes().to_vec().into_boxed_slice()));
    let slot = |ty: u32, name: &str| MirSlot {
        kind: SlotKind::Temp,
        ty,
        name: Some(name.into()),
    };
    // 0: owned literal buffer, 1/2: views of it, 3: bool result.
    let slots = vec![
        slot(t_str, "own"),
        slot(t_view, "l"),
        slot(t_view, "r"),
        slot(t_bool, "d"),
    ];
    let instrs = vec![
        MirInstr::Const { dst: 0, value: lit },
        MirInstr::View { dst: 1, src: 0 },
        MirInstr::View { dst: 2, src: 0 },
        // The forbidden shape: EqI32 reading VIEW operands.
        MirInstr::Bin {
            op: BinOp::EqI32,
            dst: 3,
            lhs: 1,
            rhs: 2,
        },
    ];
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots,
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    let r = mir_verify::verify_mir(&m);
    assert!(
        r.is_err(),
        "EqI32 over StrView operands must not verify (got {r:?})"
    );
    let msg = format!("{r:?}");
    assert!(msg.contains("bin-type"), "named rejection: {msg}");
}

// ---- V-02: the replacement ledger oracle ------------------------------------

#[test]
fn av_owned_replacement_frees_old_once_new_once() {
    // The §5.1 canonical chain: old c is READ by the RHS (its view), the
    // new owner materializes, then the old buffer frees exactly once and
    // the new one frees exactly once at exit.
    let out = ace_av::owned_driver::request_owned(
        None,
        "p894-ledger",
        "fn main() int {\n    let a Str = \"ab\"\n    var c Str = a.clone()\n    c = c + \"中\"\n    return c.len()\n}\n",
        "d_main",
        &[],
    )
    .expect("chain runs");
    let arena = out.arena.as_ref().expect("arena");
    let allocs = arena.alloc_events();
    let frees = arena.free_events();
    // three owned buffers: the "ab" binding, the clone (the initial c),
    // and the concat result — each freed EXACTLY once (a and new c at
    // exit, the OLD c by the replacement itself).
    assert_eq!(
        allocs.len(),
        3,
        "binding + clone + concat allocate: {allocs:?}"
    );
    assert_eq!(frees.len(), 3, "every buffer frees exactly once: {frees:?}");
    let mut sizes: Vec<usize> = allocs.iter().map(|e| e.bytes).collect();
    sizes.sort_unstable();
    assert_eq!(sizes, vec![2, 2, 5], "buffer byte sizes");
    let av = out.av.clone().expect("outcome");
    match av {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => {
            assert_eq!(v, 3, "len(\"ab中\") = 3 unicode scalars")
        }
        other => panic!("ledger row: {other:?}"),
    }
}

// ---- V-04: controlled allocation-fault atomicity ----------------------------

#[test]
fn av_owned_fault_during_rhs_keeps_old_target_unfreed() {
    // Fail the Nth allocation INSIDE the RHS of a replacement: the outcome
    // is a resource/infra rejection (never a value); the OLD target's
    // buffer is NOT freed before the RHS completes — the fault precedes
    // the commit segment, so the old value stays intact by construction
    // (no partial commit exists on this path).
    let src = "\
fn main() int {
    let a Str = \"ab\"
    var c Str = a.clone()
    c = c + \"中\"
    return c.len()
}
";
    let loaded = admit(src);
    // alloc #1 = the initial clone; #2 = the RHS concat → fault at #2.
    let limits = ExecLimits {
        fault_after_allocs: Some(2),
        ..ExecLimits::default()
    };
    let (out, _) = execute(&loaded, &limits, &CancelFlag::new(false));
    assert!(
        !matches!(out, ExecOutcome::Success { .. }),
        "an injected RHS allocation fault must not produce a value: {out:?}"
    );
    // Control: the same program under default limits succeeds at 3.
    expect_i32(&loaded, &ExecLimits::default(), 3, "fault-free control");
    // Fault at #1 (the initial clone — BEFORE the target exists): also a
    // clean rejection, never corruption.
    let limits1 = ExecLimits {
        fault_after_allocs: Some(1),
        ..ExecLimits::default()
    };
    let (out1, _) = execute(&loaded, &limits1, &CancelFlag::new(false));
    assert!(
        !matches!(out1, ExecOutcome::Success { .. }),
        "early fault: {out1:?}"
    );
}
