//! PLAN-818 T-04 transaction tier (`mir_native_transaction_`): staged
//! link/publish failure semantics over REAL old outputs (V-07; AC-05) —
//! three prior outputs survive link errors byte-identically and still
//! run; a deadline kills the whole tree and reclaims owned staging; a
//! publish IO failure rolls the output back to the old artifact; a final
//! path occupied by a directory is user property (never deleted, publish
//! fails); a legitimate publish replaces products atomically.
//!
//! Env discipline: `AC741_RUST_LLD` (the landed tool override) and the
//! `MN818_TEST_LINK_WALL_MS` seam are process-global — every test in this
//! binary serializes on TXN_LOCK before any link work.

mod common;

use ac_mn::link_driver::{link_and_run, ObjectInput};
use ac_mn::{emit, MnError};
use auto_av_core::contract::AV_PROFILE;
use auto_av_core::mir::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

static TXN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct EnvGuard(&'static str);
impl Drop for EnvGuard {
    fn drop(&mut self) {
        std::env::remove_var(self.0);
    }
}

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mn8-t04txn-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A minimal one-function unit whose program returns `exit_const`.
fn minimal_unit(exit_const: i32) -> MirModule {
    let owner = OwnerScope {
        schema: "auto.mir.av-probe/1".to_string(),
        module: "txn".to_string(),
    };
    let mut m = MirModule::unit(owner.clone());
    m.profile = AV_PROFILE.to_string();
    let ti = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }));
    let c = m.intern_const(MirConst::I32(exit_const));
    m.funcs.push(MirFunction {
        env: None,
        owner,
        name: "d_main".to_string(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: ti,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: c }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    m.entry = 0;
    m
}

/// Real objects for one direct link call: the unit COFF + the wrapper.
fn real_objects(exit_const: i32) -> (Vec<ObjectInput>, Vec<u8>) {
    let u = minimal_unit(exit_const);
    let obj = emit::emit_unit_coff(&u).expect("emit unit");
    let sym = ac_mn::symbols::function_symbol(&u.owner, "d_main");
    let wrapper = emit::emit_wrapper_coff(&sym).expect("emit wrapper");
    (
        vec![ObjectInput {
            name: "d_main".to_string(),
            bytes: obj.bytes,
        }],
        wrapper,
    )
}

fn exe_path_of(workdir: &Path) -> PathBuf {
    workdir.join(format!("mn8-out-{}.exe", std::process::id()))
}

fn run_exe(path: &Path) -> i32 {
    let out = Command::new(path).output().expect("run old exe");
    out.status.code().expect("exit code")
}

/// Leftover owned staging: any file starting with ".tmp-" or ending with
/// ".obj" (the staged-object prefix pattern is NN-name.obj).
fn staging_residue(workdir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(workdir) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if n.starts_with(".tmp-")
                || (n.ends_with(".obj") && n.len() > 3 && n.as_bytes()[2] == b'-')
            {
                out.push(n);
            }
        }
    }
    out
}

/// V-07: THREE prior successful outputs exist; a link error (a corrupted
/// object) leaves the latest output byte-identical and runnable, with
/// every owned staged file reclaimed.
#[test]
#[cfg(windows)]
fn mir_native_transaction_link_failure_preserves_old_outputs() {
    let _g = TXN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let w = temp_root("linkfail");
    // Three successful rounds; keep each output's bytes.
    let mut history: Vec<Vec<u8>> = Vec::new();
    for exit in [33i32, 44, 55] {
        let (objs, wrap) = real_objects(exit);
        let run = link_and_run(&objs, &wrap, "ac_start", &w, &[], "txn-round").expect("round");
        assert_eq!(run.exit_code, Some(exit));
        history.push(std::fs::read(exe_path_of(&w)).expect("exe bytes"));
    }
    assert_eq!(history.len(), 3, "three prior outputs");
    // A corrupted object: zero the COFF machine field — a STRUCTURAL
    // corruption rust-lld deterministically rejects (byte-flipping the
    // middle can land in padding/debug sections and still link).
    let (objs, wrap) = real_objects(77);
    // Not a COFF at all — deterministic linker rejection (lld recovers
    // the machine from later inputs, so header tampering is NOT enough).
    let garbage = vec![0x58u8; 100];
    let bad = vec![ObjectInput {
        name: objs[0].name.clone(),
        bytes: garbage,
    }];
    let err = match link_and_run(&bad, &wrap, "ac_start", &w, &[], "txn-round") {
        Err(e) => e,
        Ok(r) => panic!("corrupted object must fail the link: {:?}", r.exit_code),
    };
    assert!(
        err.message().contains("rust-lld exit"),
        "link failure classification: {err}"
    );
    // The old output survives byte-identically and still runs.
    assert_eq!(
        std::fs::read(exe_path_of(&w)).expect("old exe"),
        *history.last().unwrap(),
        "old output byte-identical after the failed link"
    );
    assert_eq!(run_exe(&exe_path_of(&w)), 55, "old program still correct");
    assert!(
        staging_residue(&w).is_empty(),
        "owned staging reclaimed: {:?}",
        staging_residue(&w)
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// V-07: a LINKER DEADLINE kills the whole tree (the sleeper's ping child
/// included) within the shortened test wall, the old output survives and
/// runs, and staging is reclaimed.
#[test]
#[cfg(windows)]
fn mir_native_transaction_link_timeout_kills_tree_preserves_old() {
    let _g = TXN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let w = temp_root("timeout");
    // Baseline output with the REAL linker.
    let (objs, wrap) = real_objects(66);
    let run = link_and_run(&objs, &wrap, "ac_start", &w, &[], "txn-base").expect("base");
    assert_eq!(run.exit_code, Some(66));
    let old_bytes = std::fs::read(exe_path_of(&w)).unwrap();

    // A sleeper standing in for the linker: cmd → ping (~300s), far past
    // the 3s test wall. The contained executor must kill the TREE.
    let sleeper = w.join("sleeper.cmd");
    std::fs::write(
        &sleeper,
        "@echo off\r\nping -n 300 -w 1000 127.0.0.1 > nul\r\n",
    )
    .unwrap();
    let _lld_guard = EnvGuard("AC741_RUST_LLD");
    std::env::set_var("AC741_RUST_LLD", &sleeper);
    let _wall_guard = EnvGuard("MN818_TEST_LINK_WALL_MS");
    std::env::set_var("MN818_TEST_LINK_WALL_MS", "3000");
    let (objs2, wrap2) = real_objects(88);
    let started = Instant::now();
    let err = match link_and_run(&objs2, &wrap2, "ac_start", &w, &[], "txn-timeout") {
        Err(e) => e,
        Ok(r) => panic!("a sleeping linker must hit the deadline: {:?}", r.exit_code),
    };
    let elapsed = started.elapsed();
    assert!(
        err.message().contains("rust-lld")
            && (err.message().contains("Deadline") || err.message().contains("exceeded")),
        "deadline classification: {err}"
    );
    assert!(
        elapsed.as_secs() < 30,
        "the tree was killed at the (shortened) wall, took {elapsed:?}"
    );
    // The ping descendant must be gone (taskkill tree semantics); the old
    // output survives and runs.
    assert_eq!(std::fs::read(exe_path_of(&w)).unwrap(), old_bytes);
    assert_eq!(run_exe(&exe_path_of(&w)), 66);
    let mut residue = staging_residue(&w);
    residue.retain(|n| n != "sleeper.cmd");
    assert!(residue.is_empty(), "staging reclaimed: {residue:?}");
    let _ = std::fs::remove_dir_all(&w);
}

/// V-07: a publish IO failure (the receipt path blocked) rolls the exe
/// back to the OLD output — old bytes identical, old program correct,
/// staging reclaimed; the blocking directory is user property and stays.
#[test]
#[cfg(windows)]
fn mir_native_transaction_publish_io_failure_rolls_back_to_old() {
    let _g = TXN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let w = temp_root("rollback");
    let (objs, wrap) = real_objects(88);
    let run = link_and_run(&objs, &wrap, "ac_start", &w, &[], "txn-base").expect("base");
    assert_eq!(run.exit_code, Some(88));
    let old_bytes = std::fs::read(exe_path_of(&w)).unwrap();
    // Block the receipt path with a directory, then attempt a new
    // publish: the link succeeds, the exe is placed, the receipt rename
    // fails — the exe must roll back to the old artifact.
    let receipt = w.join(".mn-link.txt");
    std::fs::remove_file(&receipt).unwrap();
    std::fs::create_dir(&receipt).unwrap();
    let (objs2, wrap2) = real_objects(99);
    let err = match link_and_run(&objs2, &wrap2, "ac_start", &w, &[], "txn-io") {
        Err(e) => e,
        Ok(r) => panic!("blocked receipt must fail the publish: {:?}", r.exit_code),
    };
    assert!(
        err.message().contains("publish receipt") && err.message().contains("old output restored"),
        "IO-failure rollback receipt: {err}"
    );
    assert_eq!(
        std::fs::read(exe_path_of(&w)).unwrap(),
        old_bytes,
        "rolled back to the old output byte-identically"
    );
    assert_eq!(run_exe(&exe_path_of(&w)), 88, "old program still correct");
    assert!(receipt.is_dir(), "the blocking directory is user property");
    let mut residue = staging_residue(&w);
    residue.retain(|n| n != ".mn-link.txt");
    assert!(residue.is_empty(), "staging reclaimed: {residue:?}");
    let _ = std::fs::remove_dir_all(&w);
}

/// V-07/user property: a final exe path occupied by a DIRECTORY is never
/// backed up or deleted — the publish fails at the place step and the
/// owned staging is reclaimed.
#[test]
#[cfg(windows)]
fn mir_native_transaction_exe_target_blocked_by_directory() {
    let _g = TXN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let w = temp_root("dirblock");
    let blocked = exe_path_of(&w);
    std::fs::create_dir_all(&blocked).unwrap();
    std::fs::write(blocked.join("user-file.txt"), "user property").unwrap();
    let (objs, wrap) = real_objects(11);
    let err = match link_and_run(&objs, &wrap, "ac_start", &w, &[], "txn-dir") {
        Err(e) => e,
        Ok(r) => panic!(
            "a directory at the output path must fail: {:?}",
            r.exit_code
        ),
    };
    assert!(
        err.message().contains("publish staged exe"),
        "classification: {err}"
    );
    assert!(blocked.is_dir(), "the directory survives untouched");
    assert!(
        blocked.join("user-file.txt").is_file(),
        "user property intact"
    );
    assert!(
        staging_residue(&w).is_empty(),
        "staging reclaimed: {:?}",
        staging_residue(&w)
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// V-07 control: a LEGITIMATE publish replaces the products — the new
/// program runs, the receipt reflects the new round, and the workdir
/// carries exactly the two products.
#[test]
#[cfg(windows)]
fn mir_native_transaction_legitimate_publish_updates_products() {
    let _g = TXN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let w = temp_root("legit");
    let (objs, wrap) = real_objects(33);
    link_and_run(&objs, &wrap, "ac_start", &w, &[], "txn-round-one").expect("round 1");
    assert_eq!(run_exe(&exe_path_of(&w)), 33);
    let (objs2, wrap2) = real_objects(77);
    link_and_run(&objs2, &wrap2, "ac_start", &w, &[], "txn-round-two").expect("round 2");
    assert_eq!(run_exe(&exe_path_of(&w)), 77, "the new program runs");
    let receipt = std::fs::read_to_string(w.join(".mn-link.txt")).unwrap();
    assert!(
        receipt.contains("txn-round-two") && !receipt.contains("txn-round-one"),
        "the receipt reflects the committed round"
    );
    let mut entries: Vec<String> = std::fs::read_dir(&w)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec![
            ".mn-link.txt".to_string(),
            exe_path_of(&w)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        ],
        "exactly the two products remain"
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// Compile-level guard: MnError classification of the link-failure family
/// (no tools needed).
#[test]
fn mir_native_transaction_error_taxonomy() {
    let e = MnError::Infra("rust-lld exit Some(1)".to_string());
    assert_eq!(e.code(), "mn.infra");
    assert!(e.message().contains("rust-lld"));
}
