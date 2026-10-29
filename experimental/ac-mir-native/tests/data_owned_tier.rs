//! PLAN-870 T-03/T-04 differential tier (`mn_data_owned_`): the frozen
//! ACE-M1 T*/R* executable rows run through THREE real paths — AV (the
//! semantic baseline, exact value/trap + the owned arena event ledger),
//! the landed old HIR-native reference, and the NEW mn pipeline — each
//! checked against the INDEPENDENT frozen oracle first (V-01..V-04), then
//! the alloc/free event streams compared event-by-event (freeze §5).

mod common;

use auto_av_core::contract::{ExecOutcome, TrapKind};

/// One row's independent oracle: a value, or a trap kind.
enum Want {
    V(i32),
    Trap(TrapKind),
}

struct Case {
    id: &'static str,
    /// av.data/1 rows take the record pipeline; av.owned/1 the owned one.
    data: bool,
    want: Want,
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            id: "T01",
            data: false,
            want: Want::V(38),
        },
        Case {
            id: "T02",
            data: false,
            want: Want::V(27),
        },
        Case {
            id: "T04",
            data: false,
            want: Want::V(128512),
        },
        Case {
            id: "T08",
            data: false,
            want: Want::Trap(TrapKind::StrIndex),
        },
        Case {
            id: "R01",
            data: true,
            want: Want::V(19),
        },
        Case {
            id: "R02",
            data: true,
            want: Want::V(9),
        },
        Case {
            id: "R03",
            data: true,
            want: Want::V(23),
        },
        Case {
            id: "R06",
            data: true,
            want: Want::V(21),
        },
        Case {
            id: "R04",
            data: false,
            want: Want::V(51),
        },
        Case {
            id: "R05",
            data: false,
            want: Want::V(42),
        },
    ]
}

fn source_of(id: &str) -> String {
    let p = common::repo_root().join(format!("crates/auto-lang/test/cases/ace-m1/src/{id}.at"));
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// The AV leg under the SAME shared stack the corpus freezes: the data
/// pipeline for av.data/1 rows, the owned pipeline (arena attached) for
/// av.owned/1 rows.
fn run_av(c: &Case, src: &str) -> (ExecOutcome, Vec<(u64, u64)>, Vec<u64>) {
    if c.data {
        // The data profile is pure-value records — no owned buffers, no
        // arena: the ledger axes are structurally empty.
        let out = auto_av_probe::data_driver::request_data(None, "p870-diff", src, "d_main", &[])
            .expect("av data leg");
        (out.av.clone().expect("data outcome"), vec![], vec![])
    } else {
        let out = auto_av_probe::owned_driver::request_owned(None, "p870-diff", src, "d_main", &[])
            .expect("av owned leg");
        let arena_allocs = out
            .arena
            .as_ref()
            .map(|a| {
                a.alloc_events()
                    .into_iter()
                    .map(|e| (e.id, e.bytes as u64))
                    .collect()
            })
            .unwrap_or_default();
        let arena_frees = out
            .arena
            .as_ref()
            .map(|a| a.free_events().into_iter().map(|e| e.id).collect())
            .unwrap_or_default();
        (
            out.av.clone().expect("owned outcome"),
            arena_allocs,
            arena_frees,
        )
    }
}

/// The old HIR-native reference leg (782 chain, supervised worker).
fn run_old(c: &Case, src: &str) -> auto_av_probe::native_reference::NativeRun {
    let workdir =
        std::env::temp_dir().join(format!("mn8-p870-old-{}-{}", c.id, std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let run = if c.data {
        auto_av_probe::native_reference::build_and_run_record(src, &[], "d_main", &workdir, None)
    } else {
        auto_av_probe::native_reference::build_and_run_text_record(
            src,
            &[],
            "d_main",
            &workdir,
            None,
        )
    }
    .expect("old-native leg");
    let _ = std::fs::remove_dir_all(&workdir);
    run
}

/// Native-side ledger event stream parsed from the shim's stderr traces
/// (freeze §5): `LIT len=N`/`CLONE len=`/`CONCAT a=` pair with the
/// following `... id=N` line; `FREE id=N` is a free event.
fn parse_events(stderr: &[u8]) -> (Vec<(u64, u64)>, Vec<u64>) {
    let text = String::from_utf8_lossy(stderr);
    let mut allocs: Vec<(u64, u64)> = Vec::new();
    let mut frees: Vec<u64> = Vec::new();
    let mut pending_len: Option<u64> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("LIT len=") {
            pending_len = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("CLONE len=") {
            pending_len = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("CONCAT a=") {
            // CONCAT traces `a=<alen> b=<blen>`; the total is not directly
            // the alloc size — recompute from a+b.
            let parts: Vec<&str> = rest.split_whitespace().collect();
            let a: u64 = parts
                .first()
                .and_then(|p| p.trim().parse().ok())
                .unwrap_or(0);
            let b: u64 = parts
                .get(1)
                .and_then(|p| p.strip_prefix("b=").and_then(|v| v.trim().parse().ok()))
                .unwrap_or(0);
            pending_len = Some(a + b);
        } else if let Some(rest) = line.strip_prefix("LIT id=") {
            let id: u64 = rest.trim().parse().unwrap_or(0);
            allocs.push((id, pending_len.take().unwrap_or(0)));
        } else if let Some(rest) = line.strip_prefix("CLONE id=") {
            let id: u64 = rest.trim().parse().unwrap_or(0);
            allocs.push((id, pending_len.take().unwrap_or(0)));
        } else if let Some(rest) = line.strip_prefix("CONCAT id=") {
            let id: u64 = rest.trim().parse().unwrap_or(0);
            allocs.push((id, pending_len.take().unwrap_or(0)));
        } else if let Some(rest) = line.strip_prefix("FREE id=") {
            frees.push(rest.trim().parse().unwrap_or(0));
        }
    }
    (allocs, frees)
}

#[test]
#[cfg(windows)]
fn mn_data_owned_three_path_differential() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    for c in cases() {
        let src = source_of(c.id);
        // ---- Leg 1 — AV (semantic baseline + arena ledger) -------------
        let (av, av_allocs, av_frees) = run_av(&c, &src);
        match &c.want {
            Want::V(v) => match &av {
                ExecOutcome::Success { value, .. } => {
                    let got = match value {
                        auto_av_core::contract::Value::I32(i) => *i,
                        other => panic!("{}: non-i32 AV result {other:?}", c.id),
                    };
                    assert_eq!(got, *v, "{}: AV value", c.id);
                }
                other => panic!("{}: AV expected value, got {other:?}", c.id),
            },
            Want::Trap(kind) => match &av {
                ExecOutcome::Trap { kind: k, .. } => {
                    assert_eq!(k, kind, "{}: AV trap kind", c.id);
                }
                other => panic!("{}: AV expected trap, got {other:?}", c.id),
            },
        }

        // ---- Leg 2 — old HIR-native reference ---------------------------
        let old = run_old(&c, &src);
        match &c.want {
            Want::V(v) => {
                assert_eq!(old.exit_code, *v, "{}: old-native exit", c.id);
            }
            Want::Trap(_) => {
                assert_eq!(old.exit_code, 70, "{}: old-native trap exit", c.id);
            }
        }

        // ---- Leg 3 — NEW mn ---------------------------------------------
        let mn = common::run_mn(&src, &[], &format!("p870-{}", c.id));
        assert!(mn.ok, "{}: mn request failed: {:?}", c.id, mn.error);
        match &c.want {
            Want::V(v) => {
                assert_eq!(
                    mn.exit_code,
                    Some(*v),
                    "{}: mn exit (stderr {:?})",
                    c.id,
                    String::from_utf8_lossy(&mn.stderr)
                );
            }
            Want::Trap(_) => {
                assert_eq!(mn.exit_code, Some(70), "{}: mn trap exit", c.id);
            }
        }

        // ---- Ledger axis: mn event stream vs AV arena (owned rows) ------
        // IDs are normalized to CREATION ORDINALS (AV ids are 0-based,
        // the shim's 1-based — the semantic content is the byte-size
        // sequence and the release order of creation ordinals).
        if !c.data {
            let (mn_allocs, mn_frees) = parse_events(&mn.stderr);
            let normalize = |allocs: &[(u64, u64)], frees: &[u64]| -> (Vec<u64>, Vec<u64>) {
                let id_to_ord: std::collections::BTreeMap<u64, u64> = allocs
                    .iter()
                    .enumerate()
                    .map(|(i, (id, _))| (*id, i as u64))
                    .collect();
                let sizes = allocs.iter().map(|(_, b)| *b).collect();
                let ord_frees = frees
                    .iter()
                    .map(|id| *id_to_ord.get(id).unwrap_or(&u64::MAX))
                    .collect();
                (sizes, ord_frees)
            };
            let (mn_sizes, mn_free_ords) = normalize(&mn_allocs, &mn_frees);
            let (av_sizes, av_free_ords) = normalize(&av_allocs, &av_frees);
            assert_eq!(
                mn_sizes,
                av_sizes,
                "{}: alloc byte-size stream vs AV arena (mn stderr {:?})",
                c.id,
                String::from_utf8_lossy(&mn.stderr)
            );
            // Freeze §5 tier ②/③: free MULTISET + exactly-once per buffer
            // (AV's within-exit firing order on clone-derived rows is a
            // host Rc-drop artifact — coverage §4.6 decision item).
            let mut mn_free_sorted = mn_free_ords.clone();
            mn_free_sorted.sort();
            let mut av_free_sorted = av_free_ords.clone();
            av_free_sorted.sort();
            assert_eq!(
                mn_free_sorted,
                av_free_sorted,
                "{}: free multiset (creation ordinals) vs AV arena (mn stderr {:?})",
                c.id,
                String::from_utf8_lossy(&mn.stderr)
            );
            let dup = |v: &[u64]| {
                let mut seen = std::collections::BTreeSet::new();
                v.iter().any(|x| !seen.insert(*x))
            };
            assert!(
                !dup(&mn_free_ords),
                "{}: a buffer freed more than once (double free)",
                c.id
            );
            // The old reference's stream is compared as a REGISTERED
            // decision item carrier (coverage §4.2: its record-field
            // release order is reverse-declaration where AV/mn are
            // declaration-order) — only the multiset must agree today.
            let (old_allocs, old_frees) = parse_events(old.stderr.as_bytes());
            // The old path's shim ids are 1-based like mn's — compare the
            // normalized byte-size multisets and free counts.
            let mut old_sizes: Vec<u64> = old_allocs.iter().map(|(_, b)| *b).collect();
            old_sizes.sort();
            let mut av_sizes_sorted = av_sizes.clone();
            av_sizes_sorted.sort();
            assert_eq!(
                old_sizes, av_sizes_sorted,
                "{}: old-native alloc size multiset vs AV",
                c.id
            );
            assert_eq!(
                old_frees.len(),
                av_frees.len(),
                "{}: old-native free count vs AV",
                c.id
            );
        }
    }
}

/// V-04 negative controls: the named rejection surface rejects with
/// codegen 0 and no side effects (freeze §7).
#[test]
#[cfg(windows)]
fn mn_data_owned_rejection_surface() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    // T03: the frozen corpus's old-770 `=` reassignment — the shared
    // frontend's OWNED verify rejects it (verify.resource-assign); mn
    // reports the same diagnostic family, never a PE.
    let mn = common::run_mn(&source_of("T03"), &[], "p870-t03-reject");
    assert!(!mn.ok, "T03 must be diagnosed");
    let err = format!("{:?}", mn.error.expect("error"));
    assert!(
        err.contains("verify.resource-assign"),
        "T03 diagnostic carries the shared code: {err}"
    );
    assert_eq!(
        mn.obs
            .counters
            .get("codegen_emit_units")
            .copied()
            .unwrap_or(0),
        0,
        "T03: no codegen on diagnosis"
    );

    // T06/T07/R10: the diagnostic rows reject through the same shared
    // stack with codegen 0.
    for id in ["T06", "T07", "R10"] {
        let mn = common::run_mn(&source_of(id), &[], &format!("p870-{id}-reject"));
        assert!(!mn.ok, "{id} must be diagnosed");
        assert_eq!(
            mn.obs
                .counters
                .get("codegen_emit_units")
                .copied()
                .unwrap_or(0),
            0,
            "{id}: no codegen on diagnosis"
        );
    }

    // Entry-shape control (freeze §3): an owned entry returning Str is
    // not the frozen zero-param i32 wrapper contract.
    let src = "fn main() Str {\n    return \"x\"\n}\n";
    let mn = common::run_mn(src, &[], "p870-entry-shape");
    assert!(!mn.ok, "non-i32 entry must reject");
    let err = format!("{:?}", mn.error.expect("error"));
    assert!(err.contains("entry"), "entry rejection named: {err}");
}

/// V-04 unit-level rejection controls (no PE needed): wide constants and
/// out-of-scope profiles reject by name at the codegen door.
#[test]
fn mn_data_owned_unit_rejections() {
    use ac_mn::MnError;
    use auto_av_core::mir::*;

    // Wide numeric const in a scalar-shaped unit → mn.unsupported.const.
    let mut m = MirModule::unit(OwnerScope {
        schema: "auto.mir.av-probe/1".into(),
        module: "m_test".into(),
    });
    m.profile = "av.core-i32/1".into();
    let t_i32 = m.intern_type(MirType::I32);
    let c_wide = m.intern_const(MirConst::I64(7));
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t_i32,
    }));
    let f = MirFunction {
        env: None,
        source_snapshot: 0,
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Temp,
            ty: t_i32,
            name: None,
        }],
        entry: 0,
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const {
                dst: 0,
                value: c_wide,
            }],
            term: Terminator::Return { value: 0 },
        }],
        spans: vec![],
    };
    m.funcs.push(f);
    match ac_mn::emit::emit_unit_coff(&m) {
        Err(MnError::Unsupported(code, _)) => assert_eq!(code, "const"),
        other => panic!("wide const must reject by name, got {other:?}"),
    }

    // Collections profile → mn.unsupported.profile (the family stays
    // closed per the 2026-10-28 ruling).
    let mut m2 = MirModule::unit(OwnerScope {
        schema: "auto.mir.av-probe/1".into(),
        module: "m_test".into(),
    });
    m2.profile = "av.collections/1".into();
    let t = m2.intern_type(MirType::I32);
    let sig2 = m2.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t,
    }));
    m2.funcs.push(MirFunction {
        env: None,
        source_snapshot: 0,
        owner: m2.owner.clone(),
        name: "d_main".into(),
        sig: sig2,
        slots: vec![],
        entry: 0,
        blocks: vec![MirBlock {
            instrs: vec![],
            term: Terminator::Return { value: 0 },
        }],
        spans: vec![],
    });
    match ac_mn::emit::emit_unit_coff(&m2) {
        Err(MnError::Unsupported(code, _)) => assert_eq!(code, "profile"),
        other => panic!("collections profile must reject, got {other:?}"),
    }
}

/// V-06 pre-wired unit-level counter-controls (freeze §8): the frozen
/// orderings and identity axes that a plausible wrong implementation
/// breaks.
#[test]
fn mn_data_owned_static_countercontrols() {
    use auto_av_core::mir::*;

    let mut m = MirModule::unit(OwnerScope {
        schema: "auto.mir.av-probe/1".into(),
        module: "m_test".into(),
    });
    let t_str = m.intern_type(MirType::Str);
    let t_i32 = m.intern_type(MirType::I32);
    let inner = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "m_src".into(),
        name: "Inner".into(),
        fields: vec![
            MirFieldDef {
                name: "a".into(),
                ty: t_str,
            },
            MirFieldDef {
                name: "b".into(),
                ty: t_str,
            },
        ],
    }));
    let outer = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "m_src".into(),
        name: "Outer".into(),
        fields: vec![
            MirFieldDef {
                name: "head".into(),
                ty: t_str,
            },
            MirFieldDef {
                name: "inner".into(),
                ty: inner,
            },
        ],
    }));

    // Layout control: declaration-order offsets. Outer = {Str@0, Inner@12}
    // (Str 12B/8); Inner = {Str@0, Str@12}, size 24 align 8; Outer size 36
    // rounded to 40 (align 8).
    let mut cache = std::collections::BTreeMap::new();
    let r = m.types.get(outer as usize).unwrap();
    let l = ac_mn::layout::record_layout(
        &m,
        outer,
        match r {
            MirType::Record(x) => x,
            _ => unreachable!(),
        },
        &mut cache,
    )
    .unwrap();
    assert_eq!(
        l.field_off,
        vec![0, 16],
        "Outer head@0, inner (8-aligned)@16"
    );
    // Inner = {Str@0, Str@16} (Str 12B/align8 → second field at 16), size 32;
    // Outer = {Str@0, Inner@16}, size 48.
    assert_eq!(l.size, 48, "Outer size = 16+32");
    assert_eq!(l.align_log2, 3);

    // Str-path control: DECLARATION order (a plausible wrong
    // implementation emits reverse-declaration — the old reference's
    // probe-P3 order — and this control fires).
    let paths = ac_mn::layout::str_field_paths(&m, outer).unwrap();
    assert_eq!(
        paths,
        vec![vec![0], vec![1, 0], vec![1, 1]],
        "Str paths enumerate depth-first in declaration order"
    );

    // Fingerprint identity controls (freeze §6): nominal names and string
    // VALUES must move the unit fingerprint — the pre-870 discriminant
    // labels collided.
    let unit_of = |m: &MirModule| {
        let mut u = MirModule::unit(m.owner.clone());
        u.profile = "av.owned/1".into();
        u.types = m.types.clone();
        u.consts = m.consts.clone();
        let sig = u.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: t_i32,
        }));
        u.funcs.push(MirFunction {
            env: None,
            source_snapshot: 0,
            owner: u.owner.clone(),
            name: "d_main".into(),
            sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Temp,
                    ty: t_str,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Temp,
                    ty: outer,
                    name: None,
                },
            ],
            entry: 0,
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::Const { dst: 0, value: 0 }],
                term: Terminator::Return { value: 0 },
            }],
            spans: vec![],
        });
        u
    };
    m.consts
        .push(MirConst::Str(b"zzz".to_vec().into_boxed_slice()));
    let fp1 = ac_mn::fingerprint::unit_fingerprint(&unit_of(&m)).unwrap();
    // Same everything except the record NAME → different fp (nominal
    // identity; a structural-width-only implementation collides).
    let mut m2 = m.clone();
    if let Some(MirType::Record(r)) = m2.types.get_mut(outer as usize) {
        r.name = "Outer2".into();
    }
    let fp2 = ac_mn::fingerprint::unit_fingerprint(&unit_of(&m2)).unwrap();
    assert_ne!(fp1, fp2, "nominal identity moves the fingerprint");
    // Same names, different string const VALUE → different fp.
    let mut m3 = m.clone();
    m3.consts[0] = MirConst::Str(b"other".to_vec().into_boxed_slice());
    let fp3 = ac_mn::fingerprint::unit_fingerprint(&unit_of(&m3)).unwrap();
    assert_ne!(fp1, fp3, "string const value moves the fingerprint");
}

/// F-870-R1-01: the MutRet wrapper write-back machine path (field move +
/// write-through zeroing) — the corpus's only mut row (R07) is rejected by
/// the shared stack, so the delivered surface is proven on a non-corpus
/// source with an INDEPENDENTLY hand-computed oracle (r=2+5=7; a.n=7
/// written back; a.body="q" len 1 → 7+7+1=15) across the legs that can
/// run it (AV owned baseline + mn; the old 782 path via text-record).
#[test]
#[cfg(windows)]
fn mn_owned_mut_writethrough_wrapper_three_paths() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let src = "type Note { body Str; n int }\nfn bump(mut x Note, d int) int {\n    x.n = x.n + d\n    return x.n\n}\nfn main() int {\n    var a Note = Note { body: \"q\", n: 2 }\n    let r int = bump(a.mut, 5)\n    return r + a.n + a.body.len()\n}\n";
    // AV leg (the semantic baseline).
    let out = auto_av_probe::owned_driver::request_owned(None, "p870-mut", src, "d_main", &[])
        .expect("av owned leg");
    match out.av.clone().expect("outcome") {
        ExecOutcome::Success {
            value: auto_av_core::contract::Value::I32(v),
            ..
        } => assert_eq!(v, 15, "AV mut write-back value"),
        other => panic!("AV expected value, got {other:?}"),
    }
    let arena = out.arena.expect("arena");
    let av_allocs: Vec<(u64, u64)> = arena
        .alloc_events()
        .into_iter()
        .map(|e| (e.id, e.bytes as u64))
        .collect();
    let av_frees: Vec<u64> = arena.free_events().into_iter().map(|e| e.id).collect();
    // Old HIR-native reference leg.
    let old = run_old(
        &Case {
            id: "MUT",
            data: false,
            want: Want::V(15),
        },
        src,
    );
    assert_eq!(old.exit_code, 15, "old-native mut write-back exit");
    // mn leg.
    let mn = common::run_mn(src, &[], "p870-mut");
    assert!(mn.ok, "mn mut write-back failed: {:?}", mn.error);
    assert_eq!(
        mn.exit_code,
        Some(15),
        "mn mut write-back exit (stderr {:?})",
        String::from_utf8_lossy(&mn.stderr)
    );
    // Ledger (three-tier): one "q" byte allocated, freed at main exit.
    let (mn_allocs, mn_frees) = parse_events(&mn.stderr);
    let mn_sizes: Vec<u64> = mn_allocs.iter().map(|(_, b)| *b).collect();
    let av_sizes: Vec<u64> = av_allocs.iter().map(|(_, b)| *b).collect();
    assert_eq!(
        mn_sizes,
        vec![1],
        "mn alloc stream (stderr {:?})",
        String::from_utf8_lossy(&mn.stderr)
    );
    assert_eq!(av_sizes, vec![1], "AV alloc stream");
    assert_eq!(mn_frees.len(), 1, "the single buffer frees exactly once");
    let _ = av_frees;
}

/// F-870-R1-02: the StrCmp and CharFromCodepoint machine faces have no
/// runnable corpus row (T05 is the named upstream exclusion) — hand-built
/// av.owned/1 units drive them through the REAL emit+link+run pipeline
/// with hand-computed exits.
#[test]
#[cfg(windows)]
fn mn_owned_strcmp_and_codepoint_machine_faces() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    use auto_av_core::mir::*;
    let owner = || OwnerScope {
        schema: "auto.mir.av-probe/1".into(),
        module: "m_test".into(),
    };
    // (tag, (lhs const, rhs const), eq?, want) — StrCmp over static views.
    let cmp_cases = [
        ("cmp-eq-ab", ("A", "B"), true, 0),
        ("cmp-ne-ab", ("A", "B"), false, 1),
        ("cmp-eq-aa", ("A", "A"), true, 1),
    ];
    // The owned units reference the text runtime — locate it exactly the
    // way the driver does.
    let text_lib = auto_av_probe::text_runtime::text_runtime_lib().expect("text runtime");
    let support = std::path::PathBuf::from(&text_lib);
    let run_cmp = |tag: &str, lhs: &str, rhs: &str, eq: bool| -> Option<i32> {
        let mut m = MirModule::unit(owner());
        m.profile = "av.owned/1".into();
        let t_i32 = m.intern_type(MirType::I32);
        let t_bool = m.intern_type(MirType::Bool);
        let t_view = m.intern_type(MirType::StrView);
        let c_l = m.intern_const(MirConst::Str(lhs.as_bytes().to_vec().into_boxed_slice()));
        let c_r = m.intern_const(MirConst::Str(rhs.as_bytes().to_vec().into_boxed_slice()));
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: t_i32,
        }));
        m.funcs.push(MirFunction {
            env: None,
            source_snapshot: 0,
            owner: m.owner.clone(),
            name: "d_main".into(),
            sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Temp,
                    ty: t_view,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Temp,
                    ty: t_view,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Temp,
                    ty: t_bool,
                    name: None,
                },
            ],
            entry: 0,
            blocks: vec![MirBlock {
                instrs: vec![
                    MirInstr::Const { dst: 0, value: c_l },
                    MirInstr::Const { dst: 1, value: c_r },
                    MirInstr::StrCmp {
                        eq,
                        dst: 2,
                        lhs: 0,
                        rhs: 1,
                    },
                ],
                term: Terminator::Return { value: 2 },
            }],
            spans: vec![],
        });
        let obj = ac_mn::emit::emit_unit_coff(&m).expect("emit");
        let wrapper = ac_mn::emit::emit_wrapper_coff(&obj.symbol).expect("wrapper");
        let workdir =
            std::env::temp_dir().join(format!("mn8-p870-r1-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&workdir);
        let run = ac_mn::link_driver::link_and_run(
            &[ac_mn::link_driver::ObjectInput {
                name: "d_main".into(),
                bytes: obj.bytes,
            }],
            &wrapper,
            ac_mn::config::START_SYMBOL,
            &workdir,
            &[support.clone()],
            tag,
        )
        .expect("link+run");
        let _ = std::fs::remove_dir_all(&workdir);
        run.exit_code
    };
    for (tag, (l, r), eq, want) in cmp_cases {
        let got = run_cmp(tag, l, r, eq);
        assert_eq!(got, Some(want), "{tag}: StrCmp machine face");
    }
    // An invalid codepoint traps (av.trap.char-invalid family → exit 70):
    // CharFromCodepoint over -1 never returns.
    let mut m = MirModule::unit(owner());
    m.profile = "av.owned/1".into();
    let t_i32 = m.intern_type(MirType::I32);
    let t_char = m.intern_type(MirType::Char);
    let c_neg = m.intern_const(MirConst::I32(-1));
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t_i32,
    }));
    m.funcs.push(MirFunction {
        env: None,
        source_snapshot: 0,
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Temp,
                ty: t_i32,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Temp,
                ty: t_char,
                name: None,
            },
        ],
        entry: 0,
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_neg,
                },
                MirInstr::CharFromCodepoint { dst: 1, code: 0 },
            ],
            term: Terminator::Return { value: 1 },
        }],
        spans: vec![],
    });
    let obj = ac_mn::emit::emit_unit_coff(&m).expect("emit");
    let wrapper = ac_mn::emit::emit_wrapper_coff(&obj.symbol).expect("wrapper");
    let workdir = std::env::temp_dir().join(format!("mn8-p870-r1-cpinval-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let run = ac_mn::link_driver::link_and_run(
        &[ac_mn::link_driver::ObjectInput {
            name: "d_main".into(),
            bytes: obj.bytes,
        }],
        &wrapper,
        ac_mn::config::START_SYMBOL,
        &workdir,
        &[support.clone()],
        "cpinval",
    )
    .expect("link+run");
    let _ = std::fs::remove_dir_all(&workdir);
    assert_eq!(run.exit_code, Some(70), "invalid codepoint traps (exit 70)");
}
