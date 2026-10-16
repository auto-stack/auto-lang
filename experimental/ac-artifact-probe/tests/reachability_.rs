//! V-06 reference/lifecycle tests (§5.4): shared chunks survive deleting
//! one project; provenance (not required) upstream HIR is reclaimable while
//! the object stays linkable; leases pin artifacts against GC and are
//! released by OS truth, not TTL.

#![cfg(windows)]

mod common;

use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::store::{run_gc, GcConfig, ManagedStore};
use common::{first, run_node, scratch, NODE_EXE};
use std::process::Command;
use std::time::Duration;

fn trigger_gc(store: &ManagedStore) -> ac_artifact_probe::store::GcReceipt {
    // Hair-trigger watermark for lifecycle tests (the watermark VALUES are
    // exercised in gc_budget_); here we need candidates processed now.
    run_gc(store, GcConfig { high: 1, low: 0 }).unwrap()
}

#[test]
fn reachability_deleting_one_project_keeps_shared_chunks_of_the_other() {
    let root = scratch("reach-delete");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 5 + 5 }").unwrap();
    // Two projects, identical content.
    for p in ["proj-a", "proj-b"] {
        let out = vertical_and_run(
            &store,
            &VerticalCfg {
                project: p.into(),
                source: src.clone(),
                work_dir: root.join(p),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(out.exit_code, Some(10));
    }
    let chunks_before = store.space().blocks_count;

    // Project A closes: its roots go away.
    store.remove_project_roots("proj-a").unwrap();
    let receipt = trigger_gc(&store);
    assert!(
        receipt.ran
            || receipt.total_after < receipt.total_before
            || store.space().blocks_count <= chunks_before
    );

    // Project B still hits everything (shared chunks were NOT deleted).
    let out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "proj-b".into(),
            source: src,
            work_dir: root.join("proj-b"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(10));
    assert_eq!(
        out.counters.parse, 0,
        "project B must still hit after A's deletion"
    );
    assert_eq!(out.counters.lower, 0);
}

#[test]
fn reachability_object_linkable_after_upstream_hir_reclaimed() {
    let root = scratch("reach-obj");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 40 + 2 }").unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src.clone(),
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(42));
    let a09 = out
        .artifacts
        .iter()
        .find(|(l, _)| l == "A09")
        .unwrap()
        .1
        .clone();

    // Keep ONLY the object rooted; the upstream HIR/snapshot/summary lose
    // their roots (provenance edges are NOT required-data — §5.1/A09).
    store.remove_project_roots("p").unwrap();
    store.register_root("delivery", &a09, "soft").unwrap();
    trigger_gc(&store);

    // The object still links + runs with the correct semantics, in a fresh
    // process, without any upstream HIR.
    let out_json = run_node(&[
        "vertical",
        "--store",
        root.join("store").to_str().unwrap(),
        "--project",
        "delivery",
        "--source",
        src.to_str().unwrap(),
        "--work",
        root.join("w").to_str().unwrap(),
    ]);
    assert_eq!(out_json.code, 0, "{}", out_json.raw);
    let result = first(&out_json, "RESULT");
    assert_eq!(result["exit_code"], 42);
    assert_eq!(result["hits"]["object"], "hit");
    assert_eq!(result["counters"]["lower"], 0);
    assert_eq!(result["counters"]["parse"], 0);
}

#[test]
fn reachability_lease_pins_against_gc_and_release_is_os_truth() {
    let root = scratch("reach-lease");
    let store_root = root.join("store");
    let store = ManagedStore::open(&store_root).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 9 }").unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src.clone(),
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(9));
    let a07 = out
        .artifacts
        .iter()
        .find(|(l, _)| l == "A07")
        .unwrap()
        .1
        .clone();
    let a09 = out
        .artifacts
        .iter()
        .find(|(l, _)| l == "A09")
        .unwrap()
        .1
        .clone();

    // Unroot everything except the delivered object; a live lease holds
    // the HIR artifact even with no roots at all.
    store.remove_project_roots("p").unwrap();
    store.register_root("keep", &a09, "soft").unwrap();
    let lease = store.acquire_lease("reader", 1, Some(&a07)).unwrap(); // TTL 1s — liveness must NOT be TTL-decided
    std::thread::sleep(Duration::from_millis(1500)); // TTL long expired
    trigger_gc(&store);
    assert!(
        store.lease_live(&lease.lease_id).unwrap(),
        "lease past TTL is still live while held"
    );
    let still_object = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src.clone(),
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(still_object.exit_code, Some(9));
    assert_eq!(still_object.counters.lower, 0, "rooted object survived GC");
    // The leased HIR must also have survived: force the HIR-hit path.
    store.purge_layer("A09").unwrap();
    store.remove_project_roots("keep").unwrap();
    let hir_out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src,
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        hir_out.counters.hir_restore, 1,
        "leased HIR must not have been deleted"
    );
    assert_eq!(hir_out.exit_code, Some(9));

    // Holder releases (drop in THIS process) → sweep + GC reclaim it.
    drop(lease);
    store.sweep_stale_leases().unwrap();
    trigger_gc(&store);
    // After reclaim, the object (rebuilt above) still runs.
    let final_out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: root.join("m.at"),
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(final_out.exit_code, Some(9));
}

#[test]
fn reachability_cross_process_lease_pinning() {
    let root = scratch("reach-lease-proc");
    let store_root = root.join("store");
    let store = ManagedStore::open(&store_root).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 8 }").unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src,
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap();
    let a07 = out
        .artifacts
        .iter()
        .find(|(l, _)| l == "A07")
        .unwrap()
        .1
        .clone();
    store.remove_project_roots("p").unwrap();
    drop(store);

    // Another PROCESS holds the lease (detached spawn; id lands in a file).
    // `--crash`: the holder ABORTS without releasing — a crashed session.
    let id_file = root.join("lease-id.txt");
    let mut lease_cmd = Command::new(NODE_EXE);
    lease_cmd.args([
        "lease",
        "--store",
        store_root.to_str().unwrap(),
        "--purpose",
        "remote-holder",
        "--hold-ms",
        "800",
        "--artifact",
        &a07,
        "--id-file",
        id_file.to_str().unwrap(),
        "--crash",
    ]);
    lease_cmd.stdin(std::process::Stdio::null());
    lease_cmd.stdout(std::process::Stdio::null());
    lease_cmd.stderr(std::process::Stdio::null());
    let mut holder = lease_cmd.spawn().expect("spawn lease holder");
    let mut lease_id = String::new();
    for _ in 0..100 {
        if let Ok(s) = std::fs::read_to_string(&id_file) {
            lease_id = s.trim().to_string();
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!lease_id.is_empty(), "lease holder did not report its id");

    let store = ManagedStore::open(&store_root).unwrap();
    assert!(
        store.lease_live(&lease_id).unwrap(),
        "cross-process lease is live"
    );
    trigger_gc(&store);
    assert!(
        store.lease_live(&lease_id).unwrap(),
        "GC must not delete a live foreign lease's artifacts (or its lock)"
    );
    drop(store);

    // Wait out the crashed holder: OS lock released at process death even
    // though the row was never marked released.
    let status = holder.wait().unwrap();
    assert!(
        status.code().is_none() || status.code() != Some(0),
        "holder must have aborted"
    );
    let store = ManagedStore::open(&store_root).unwrap();
    assert!(
        !store.lease_live(&lease_id).unwrap(),
        "holder exited → lock released by OS"
    );
    let n = store.sweep_stale_leases().unwrap();
    assert_eq!(
        n, 1,
        "crashed lease reclaimed by sweep (row was never released)"
    );
}

/// F787-R6-03（787 复审移交的上游缺陷回归）：lease_id 携带进程内全局序列
/// ——同毫秒**顺序**申请-释放循环不得撞 leases 主键。修复前 `ls-{ms}-{pid}`
/// 在已释放行仍残留表内时，128 连发 91 次 UNIQUE constraint 失败；liveness
/// 语义不变（OS 锁文件裁定，TTL/PID 不参与）。
#[test]
fn reachability_sequential_same_millisecond_leases_do_not_collide() {
    let root = scratch("reach-lease-seq");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    for i in 0..128 {
        let lease = store
            .acquire_lease("seq-probe", 3600, None)
            .unwrap_or_else(|e| panic!("grant #{i} failed: {e}"));
        assert!(store.lease_live(&lease.lease_id).unwrap());
        // 释放后行仍留在 leases 表——同毫秒内的下一发正是旧实现的碰撞点。
        drop(lease);
    }
}
