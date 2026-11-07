//! PLAN-923 V-01: per-seam allocation-fault sweep over real native PEs.
//! t02-design.md §2 seams: S1 node-header / S2 data-buffer / S3 grow-buffer
//! / S4 frame-cells / S5 clone-work (collections seams, trap code 25) and
//! S6 text-backing / S7 text-meta (embedded text runtime seams, ERR_ENV_OOM
//! → trap code 22). Each seam is swept 1..N with the N+1 positive control
//! and the no-injection control; every injected point must trap with the
//! correct code and leave the runtime clean.
#[path = "../common/mod.rs"]
mod common;
use ace_av_core::mir::*;
use ace_native::collections_lower::WrapperKnobs;
use ace_native::collections_lower::{SEAM_CLONEWORK, SEAM_DATA, SEAM_FRAME, SEAM_GROW, SEAM_NODE};

fn driver_run(source: &str, tag: &str, knobs: WrapperKnobs) -> (Option<i32>, Vec<u8>) {
    assert!(
        common::tools_available(),
        "native tools are required; no skip pass"
    );
    let dir = std::env::temp_dir().join(format!("plan923-{tag}-{}", std::process::id()));
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let res = driver.request_resource_events(source, "main", &dir, None, knobs);
    (res.exit_code, res.stderr)
}

fn trapped_code(exit: &Option<i32>, stderr: &[u8]) -> Option<i64> {
    if *exit != Some(70) {
        return None;
    }
    let text = String::from_utf8_lossy(stderr);
    text.lines().find_map(|line| {
        let rest = line.strip_prefix("MN-COLL-TRAP code=")?;
        rest.parse::<i64>().ok()
    })
}

fn clean_after_trap(stderr: &[u8]) {
    let text = String::from_utf8_lossy(stderr);
    for field in [
        "MN-COLL live=0",
        "MN-COLL bytes=0",
        "MN-COLL invalid=0",
        "MN-COLL arena_live=0",
        "MN-COLL arena_invalid=0",
    ] {
        assert!(
            text.contains(&format!("{field}\n")),
            "missing {field}: {text}"
        );
    }
}

/// Sweep one collections seam 1..N over a real source->PE driver run; assert
/// every injected point traps with `code` and cleans up, and that the first
/// non-injected run (the N+1 point) returns exactly the baseline exit.
/// S4 (frame-open) collapses into the generic null path, so its trap code is
/// RESOURCE(22) — the MN-COLL-INJECT fact line distinguishes it from a real
/// budget refusal.
fn expected_code(seam: u32) -> i64 {
    if seam == SEAM_FRAME {
        22
    } else {
        25
    }
}

fn sweep_collections_seam(source: &str, tag: &str, seam: u32, base_exit: Option<i32>) {
    let code = expected_code(seam);
    let mut n = 1u64;
    let mut injected = 0usize;
    loop {
        let knobs = WrapperKnobs {
            fail_seam: Some((seam, n)),
            ..Default::default()
        };
        let run = driver_run(source, &format!("{tag}-s{seam}-n{n}"), knobs);
        match trapped_code(&run.0, &run.1) {
            Some(actual) => {
                assert_eq!(
                    actual,
                    code,
                    "seam {seam} n={n}: wrong trap code ({})",
                    String::from_utf8_lossy(&run.1)
                );
                // The injection fact names the seam and the hit ordinal.
                let text = String::from_utf8_lossy(&run.1);
                assert!(
                    text.contains(&format!("MN-COLL-INJECT seam={seam}\n")),
                    "missing injection fact: {text}"
                );
                clean_after_trap(&run.1);
                injected += 1;
                n += 1;
            }
            None => {
                assert_eq!(
                    run.0, base_exit,
                    "seam {seam} n={n}: N+1 run differs from the no-injection baseline"
                );
                break;
            }
        }
        assert!(n < 64, "seam {seam} runaway injection");
    }
    assert!(
        injected >= 1,
        "seam {seam}: no allocation point was injected"
    );
}

/// The no-injection positive control: the program result on an unmodified run.
fn base_of(source: &str, tag: &str) -> Option<i32> {
    let run = driver_run(source, &format!("{tag}-base"), WrapperKnobs::default());
    assert!(
        trapped_code(&run.0, &run.1).is_none(),
        "baseline must not trap: {}",
        String::from_utf8_lossy(&run.1)
    );
    clean_after_trap(&run.1);
    run.0
}

#[test]
fn per_seam_fault_sweep_l04_real_source_pe() {
    let source = std::fs::read_to_string(
        common::repo_root().join("crates/auto-lang/test/cases/ace-m1/src/L04.at"),
    )
    .unwrap();
    let base = base_of(&source, "l04");
    assert_eq!(base, Some(42), "L04 baseline");
    // L04 (List<Item> new/push/get with Str field): cap=4 so no growth;
    // hits node/data/frame allocations. Growth is swept on its own source.
    for seam in [SEAM_NODE, SEAM_DATA, SEAM_FRAME] {
        sweep_collections_seam(&source, "l04", seam, base);
    }
}

#[test]
fn per_seam_fault_sweep_grow_seam_real_source_pe() {
    // Six pushes exceed the List.new() cap of 4: the fifth push grows.
    let source = "fn main() int { var xs = List<int>.new()\n xs.push(10)\n xs.push(11)\n xs.push(12)\n xs.push(13)\n xs.push(14)\n return xs.get(0) + xs.get(4)\n }";
    let base = base_of(source, "grow");
    assert_eq!(base, Some(24), "grow baseline");
    for seam in [SEAM_NODE, SEAM_DATA, SEAM_GROW, SEAM_FRAME] {
        sweep_collections_seam(source, "grow", seam, base);
    }
}

#[test]
fn per_seam_fault_sweep_lstr_real_source_pe() {
    let source = std::fs::read_to_string(
        common::repo_root().join("crates/auto-lang/test/cases/ace-m1/src/LSTR864.at"),
    )
    .unwrap();
    let base = base_of(&source, "lstr");
    assert_eq!(base, Some(51), "LSTR864 baseline");
    // LSTR cap=4, two pushes: no growth (swept separately).
    for seam in [SEAM_NODE, SEAM_DATA, SEAM_FRAME] {
        sweep_collections_seam(&source, "lstr", seam, base);
    }
    // S6 text-backing seam (embedded text runtime): injections surface as
    // env-oom -> trap 22.
    let mut n = 1u64;
    let mut injected = 0usize;
    loop {
        let knobs = WrapperKnobs {
            fail_text_nth: Some(n),
            ..Default::default()
        };
        let run = driver_run(&source, &format!("lstr-s6-n{n}"), knobs);
        match trapped_code(&run.0, &run.1) {
            Some(actual) => {
                assert_eq!(
                    actual,
                    22,
                    "text n={n}: {}",
                    String::from_utf8_lossy(&run.1)
                );
                clean_after_trap(&run.1);
                injected += 1;
                n += 1;
            }
            None => {
                assert_eq!(run.0, base, "text N+1 differs");
                break;
            }
        }
        assert!(n < 64, "text seam runaway");
    }
    assert!(injected >= 1, "text seam: no injection point");
    // S7 ledger-metadata seam: first ledger growth must inject (trap 22).
    let knobs = WrapperKnobs {
        fail_ledger_nth: Some(1),
        ..Default::default()
    };
    let run = driver_run(&source, "lstr-s7-n1", knobs);
    assert_eq!(
        trapped_code(&run.0, &run.1),
        Some(22),
        "{}",
        String::from_utf8_lossy(&run.1)
    );
    clean_after_trap(&run.1);
}

/// Minimal typed List<Str> deep clone: real verify_mir -> emit -> PE with a
/// per-seam injection (S5 clone-work plus the S1/S2 node/data of the clone
/// root and its text backing).
fn typed_list_str_clone() -> MirModule {
    let owner = OwnerScope {
        schema: ace_av_core::contract::MIR_COLLECTIONS_SCHEMA.into(),
        module: "collections923".into(),
    };
    let mut m = MirModule::new_collections(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Bool,
        MirType::Str,
        MirType::List(ListTypeRef { elem: 2 }),
        MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        }),
        MirType::StrView,
    ];
    m.consts = vec![MirConst::Str("中".as_bytes().into()), MirConst::I32(0)];
    let slot_types = [2, 3, 3, 2, 0, 0, 5, 0];
    let slots = slot_types
        .iter()
        .map(|ty| MirSlot {
            ty: *ty,
            kind: SlotKind::Temp,
            name: None,
        })
        .collect();
    use MirInstr::*;
    let instrs = vec![
        Const { dst: 0, value: 0 },
        Const { dst: 5, value: 1 },
        ListNew {
            dst: 1,
            list_ty: 3,
            cap: 1,
        },
        ListPush { root: 1, value: 0 },
        Clone { dst: 2, src: 1 },
        ListRemove {
            dst: 3,
            root: 2,
            index: 5,
        },
        View { dst: 6, src: 3 },
        StrLen { dst: 4, view: 6 },
        BorrowEnd { token: 6 },
    ];
    m.funcs.push(MirFunction {
        owner,
        name: "d_main".into(),
        sig: 4,
        slots,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 4 },
        }],
    });
    m
}

#[test]
fn typed_list_str_clone_seam_sweep() {
    let m = typed_list_str_clone();
    ace_av_core::verify_mir(&m).unwrap();
    let units = ace_av::cache::split_units(&m).unwrap();
    let entry = "d_main";
    let base = common::collections::run_units_with_knobs(
        &units,
        entry,
        "typed-clone-base",
        None,
        &WrapperKnobs::default(),
    );
    assert_eq!(
        base.exit_code,
        Some(1),
        "{}",
        String::from_utf8_lossy(&base.stderr)
    );
    common::collections::assert_balanced(&base);
    for seam in [SEAM_NODE, SEAM_DATA, SEAM_CLONEWORK, SEAM_FRAME] {
        let mut n = 1u64;
        let mut injected = 0usize;
        loop {
            let knobs = WrapperKnobs {
                fail_seam: Some((seam, n)),
                ..Default::default()
            };
            let run = common::collections::run_units_with_knobs(
                &units,
                entry,
                &format!("typed-clone-s{seam}-n{n}"),
                None,
                &knobs,
            );
            match trapped_code(&run.exit_code, &run.stderr) {
                Some(actual) => {
                    assert_eq!(actual, expected_code(seam), "typed seam {seam} n={n}");
                    let text = String::from_utf8_lossy(&run.stderr);
                    assert!(
                        text.contains(&format!("MN-COLL-INJECT seam={seam}\n")),
                        "missing injection fact: {text}"
                    );
                    clean_after_trap(&run.stderr);
                    injected += 1;
                    n += 1;
                }
                None => {
                    assert_eq!(run.exit_code, base.exit_code, "typed seam {seam} N+1");
                    break;
                }
            }
            assert!(n < 64, "typed seam {seam} runaway");
        }
        assert!(injected >= 1, "typed seam {seam}: no injection point");
    }
}
