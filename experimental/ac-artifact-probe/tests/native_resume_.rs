//! V-04 real vertical line (§5.3): the four oracle fixtures through the
//! real pipeline, cross-PROCESS restart resume (object hit and HIR hit),
//! skip-proof via counters + source-absence injection. Windows native gate.

#![cfg(windows)]

mod common;

use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::store::ManagedStore;
use ac_artifact_probe::subproc::run_deadlined;
use common::{fixture_path, first, run_node, scratch, NODE_EXE, ORACLE};
use std::process::Command;
use std::time::Duration;

#[test]
fn native_resume_four_fixtures_fresh_restart_hirhit_and_source_absent() {
    let root = scratch("native-resume");
    let store_root = root.join("store");
    for (fixture, want) in ORACLE {
        let work = root.join(fixture);
        std::fs::create_dir_all(&work).unwrap();
        let src = fixture_path(fixture);

        // -- fresh, in-process: full pipeline ---------------------------------
        let store = ManagedStore::open(&store_root).unwrap();
        let out = vertical_and_run(
            &store,
            &VerticalCfg {
                project: "proj".into(),
                source: src.clone(),
                work_dir: work.clone(),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(out.exit_code, Some(*want), "{} fresh", fixture);
        assert_eq!(out.stdout, "");
        assert_eq!(out.stderr, "");
        assert_eq!(out.counters.parse, 1);
        assert_eq!(out.counters.verify, 1);
        assert_eq!(out.counters.lower, 1);
        assert_eq!(out.counters.link, 1);
        assert_eq!(out.counters.run, 1);

        // -- restart in a NEW PROCESS with the source REMOVED: object hit
        //    is the only path that can succeed (failure injection — the
        //    front end cannot run at all) --------------------------------------
        let hidden = root.join("hidden-src");
        std::fs::create_dir_all(&hidden).unwrap();
        let moved = hidden.join(fixture);
        std::fs::copy(&src, &moved).unwrap(); // copy+remove: temp may be on another drive
        std::fs::remove_file(&src).unwrap();
        let out_json = run_node(&[
            "vertical",
            "--store",
            store_root.to_str().unwrap(),
            "--project",
            "proj",
            "--source",
            src.to_str().unwrap(),
            "--snapshot-id",
            &out.snapshot_id,
            "--work",
            work.to_str().unwrap(),
        ]);
        assert_eq!(out_json.code, 0, "{} restart: {}", fixture, out_json.raw);
        let result = first(&out_json, "RESULT");
        assert_eq!(result["exit_code"], *want, "{} restart exit", fixture);
        assert_eq!(result["counters"]["parse"], 0, "{}: front end must be skipped", fixture);
        assert_eq!(result["counters"]["verify"], 0, "{}: verifier must be skipped on object hit", fixture);
        assert_eq!(result["counters"]["lower"], 0, "{}: lowering must be skipped on object hit", fixture);
        assert_eq!(result["counters"]["link"], 1, "{}: link is still real", fixture);
        assert_eq!(result["counters"]["run"], 1, "{}: run is still real", fixture);
        assert_eq!(result["hits"]["object"], "hit");

        // -- HIR hit in a new process (object layer purged): binary restore
        //    → real verify → lower → link → run --------------------------------
        let store = ManagedStore::open(&store_root).unwrap();
        store.purge_layer("A09").unwrap();
        store.purge_layer("A10").unwrap();
        drop(store);
        let out_json = run_node(&[
            "vertical",
            "--store",
            store_root.to_str().unwrap(),
            "--project",
            "proj",
            "--source",
            src.to_str().unwrap(),
            "--snapshot-id",
            &out.snapshot_id,
            "--work",
            work.to_str().unwrap(),
        ]);
        assert_eq!(out_json.code, 0, "{} hir-hit: {}", fixture, out_json.raw);
        let result = first(&out_json, "RESULT");
        assert_eq!(result["exit_code"], *want, "{} hir-hit exit", fixture);
        assert_eq!(result["counters"]["hir_restore"], 1, "{}: binary restore happened", fixture);
        assert_eq!(result["counters"]["verify"], 1, "{}: restored bundle re-verified", fixture);
        assert_eq!(result["counters"]["lower"], 1);
        assert_eq!(result["counters"]["parse"], 0, "{}: no source needed for HIR hit", fixture);

        // -- restore the source for the next fixture ---------------------------
        std::fs::copy(&moved, &src).unwrap();
        std::fs::remove_file(&moved).unwrap();
    }
}

#[test]
fn native_resume_binary_restore_is_not_a_text_roundtrip() {
    // The HIR chunk payload contains no Atom text form: the restore is the
    // frozen binary codec (binary feasibility proof, §5.3).
    let root = scratch("native-binary");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 2 + 2 }").unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg { project: "p".into(), source: src, work_dir: root.join("w"), ..Default::default() },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(4));
    let hir_art = store
        .get_artifact(
            "A07",
            &ac_artifact_probe::identity::hir_bundle_key(
                "auto.source.core-i32.draft",
                "core-i32-draft",
                &[],
                &ac_artifact_probe::identity::SourceSnapshotId(out.snapshot_id.clone()),
                ac_artifact_probe::codec::HIR_CODEC_VERSION,
            ),
        )
        .unwrap()
        .expect("HIR artifact");
    let text = String::from_utf8_lossy(&hir_art.payload);
    assert!(!text.contains("hir("), "payload must be binary, found Atom text form");
    assert!(!text.contains("module("));
}

#[test]
fn native_resume_deadlined_exe_run() {
    // The produced exe runs under the probe deadline machinery (a hung exe
    // is a controlled run error, not a test hang).
    std::env::set_var("AC_PROBE_RUN_DEADLINE_MS", "3000");
    let root = scratch("native-deadline");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src = root.join("m.at");
    // Infinite loop: the verifier accepts it (termination is not a verify
    // rule); the run must hit the tightened deadline.
    std::fs::write(&src, "fn main() int { while 1 < 2 { }; return 0 }").unwrap();
    let r = vertical_and_run(
        &store,
        &VerticalCfg { project: "p".into(), source: src, work_dir: root.join("w"), ..Default::default() },
    );
    match r {
        Err(ac_artifact_probe::driver::DriverError::Run(msg)) => {
            assert!(msg.contains("deadline") || msg.contains("exceeded"), "{}", msg);
        }
        other => panic!("expected run-deadline error, got {:?}", other.map(|o| o.exit_code)),
    }
    // Sanity: the deadline executor itself works for a fast command.
    let mut fast = Command::new("cmd");
    fast.args(["/C", "exit /b 7"]);
    let ok = run_deadlined(fast, Duration::from_secs(10)).unwrap();
    assert_eq!(ok.code, Some(7));
    let _ = NODE_EXE;
}
