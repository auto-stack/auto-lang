//! PLAN-796 T-04 budget/pressure tests (`sema_budget_` filter, V-06):
//! ≥32 MiB streamed pages under the 16 MiB managed budget, hot-LRU
//! caps, 4096-level deep chains full-path, 512-revision churn with GC
//! convergence and sentinel survival, and the 512 KiB payload cap.

#![cfg(windows)]

use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::store::{run_gc, GcConfig};
use ac_sema_contract::{
    ContentVersion, DeclKind, DeclRef, FieldDesc, PackageId, PageHeader, PageKind, RecordMeta,
    RecordShape, SemaPage, TypeDesc, MAX_PAGE_BODY,
};
use ac_sema_core::adapter::parse_into_input;
use ac_sema_core::query::{fn_body_canonical, QueryKind};
use ac_sema_core::{InferOutcome, InferSession};
use ac_sema_probe::budget::{publish_streamed, HotPages, SessionMeter, HOT_BUDGET, MANAGED_BUDGET};
use ac_sema_probe::{open_store, publish_infer_result, run_inference, FetchOutcome};
use auto_ac_prototype::source::SourceProfile;

fn scratch(family: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("ac796-{}", family));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// A filler RecordMeta page of ~`target_bytes` payload.
fn filler_page(i: usize, fields: usize) -> SemaPage {
    let decl = DeclRef::new(
        &PackageId::new("pressure"),
        "m",
        DeclKind::Record,
        &format!("Big{i}"),
        ContentVersion::of_canonical(format!("big{i}").as_bytes()),
    );
    let shape = RecordShape {
        decl: decl.clone(),
        fields: (0..fields)
            .map(|k| FieldDesc {
                name: format!("field_{k:06}_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
                ty: TypeDesc::I32,
            })
            .collect(),
    };
    let group = ac_sema_contract::RecGroup {
        members: vec![shape.clone()],
    };
    SemaPage {
        header: PageHeader {
            schema: ac_sema_contract::SEMA_PAGE_SCHEMA_V1.into(),
            producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
            codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
            kind: PageKind::RecordMeta,
            content: group.group_content(),
        },
        body: ac_sema_contract::PageBody::RecordMeta(RecordMeta {
            shape,
            methods: vec![],
            group_content: group.group_content(),
        }),
    }
}

/// V-06: ≥32 MiB of real semantic pages, generated and published
/// STREAMED (one page live at a time), with the in-process managed
/// ledger never exceeding 16 MiB and the private-bytes increment
/// staying within the 64 MiB target.
#[test]
fn sema_budget_streamed_32mib_under_managed_budget() {
    let root = scratch("budget-32mib");
    let store = open_store(&root.join("store")).unwrap();
    let mut meter = SessionMeter::new();

    // 256 pages × ~3000 fields ≈ 140 KiB payload each → ≥32 MiB total
    let fields = 3000;
    let pages = (0..256).map(|i| filler_page(i, fields));
    let mem_before = ac_artifact_probe::sysinfo::imp::current_process_memory().private_bytes;

    // F-796-R6-05: sample the OS private bytes DURING the publish window
    // (a thread at ~2ms), not just before/after — the peak is what the
    // 64MiB target constrains, and the streamed page must be charged to
    // the SAME meter that the flow consumes.
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let peak = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let sampler = {
        let stop = stop.clone();
        let peak = peak.clone();
        std::thread::spawn(move || {
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                let now = ac_artifact_probe::sysinfo::imp::current_process_memory().private_bytes;
                peak.fetch_max(now, std::sync::atomic::Ordering::Relaxed);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        })
    };

    let (count, total_bytes) = publish_streamed(
        &store,
        pages,
        |p| {
            let name = match &p.body {
                ac_sema_contract::PageBody::RecordMeta(m) => m.shape.decl.name.clone(),
                _ => "x".into(),
            };
            (
                ac_sema_probe::SEMA_LAYER,
                QueryKey {
                    layer: ac_sema_probe::SEMA_LAYER,
                    components: vec![("page".into(), name)],
                },
            )
        },
        None,
        Some(&mut meter),
    )
    .unwrap();
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    sampler.join().unwrap();

    assert!(
        total_bytes >= 32 * 1024 * 1024,
        "streamed pages must reach ≥32MiB, got {}",
        total_bytes
    );
    assert_eq!(count, 256);
    assert!(
        meter.managed_bytes() <= MANAGED_BUDGET,
        "managed ledger must stay ≤16MiB, got {}",
        meter.managed_bytes()
    );
    assert!(
        meter.scratch_peak() >= 100 * 1024,
        "the streamed page must actually be charged (peak {} bytes)",
        meter.scratch_peak()
    );
    let peak_private = peak.load(std::sync::atomic::Ordering::Relaxed);
    let delta = peak_private.saturating_sub(mem_before);
    assert!(
        delta <= 64 * 1024 * 1024,
        "private-bytes PEAK increment must stay ≤64MiB, got {}",
        delta
    );
    // pages really are on disk
    assert!(store.space().blocks >= 32 * 1024 * 1024);
}

/// Hot LRU honors its 8 MiB cap (evictions) and the scratch ledger
/// returns bytes on drop; over-budget asks are controlled rejections.
#[test]
fn sema_budget_hot_lru_and_scratch_ledgers() {
    let mut meter = SessionMeter::new();
    // insert pages until well past 8 MiB — the LRU must evict
    let mut i = 0usize;
    while meter.hot.evictions() == 0 && i < 4096 {
        let page = filler_page(i, 1500); // ~130 KiB estimated live each
        meter.hot.insert(format!("p{i}"), page).unwrap();
        i += 1;
    }
    assert!(meter.hot.evictions() > 0, "LRU must have evicted by now");
    assert!(meter.hot.bytes() <= HOT_BUDGET, "hot ledger capped at 8MiB");

    // scratch: reserved and returned (RAII)
    {
        let _g = meter.scratch(1024 * 1024).unwrap();
    }
    assert_eq!(meter.scratch_peak(), 1024 * 1024);
    assert_eq!(
        meter.managed_bytes(),
        meter.hot.bytes(),
        "scratch returned on drop"
    );
    // a page bigger than the whole budget is a controlled rejection
    let mut strict = HotPages::new(1024);
    let big = filler_page(0, 5000);
    assert!(strict.insert("big".into(), big).is_err());
}

/// V-06 deep chains: a 4096-term operator spine AND a 4096-deep record
/// reference chain go parse → project → infer → canonical → publish →
/// read-back with no native-recursion failure anywhere on the path.
#[test]
fn sema_budget_deep_chain_4096_full_path() {
    // (a) deep expression spine in one body
    let mut expr = String::from("1");
    for _ in 0..4096 {
        expr.push_str(" + 1");
    }
    let src = format!("fn main() int {{ return {expr} }}");
    let input = parse_into_input(&src, "deep", "m", SourceProfile::Core).unwrap();
    match InferSession::new(&input).infer_function("main") {
        InferOutcome::Ok(sum) => {
            assert!(sum.facts.expr_types.len() >= 4096, "every spine node typed");
            let f = input.lookup_fn("main").unwrap();
            let canonical = fn_body_canonical(f);
            assert!(canonical.len() > 4096);
        }
        other => panic!("deep spine must infer, got {other:?}"),
    }

    // (b) deep record chain R0.next: R1 … R4095 — full-path closure walk
    let mut src = String::new();
    for i in 0..4095 {
        src.push_str(&format!("type R{i} {{ v int; next R{} }}\n", i + 1));
    }
    src.push_str("type R4095 { v int }\n");
    let input = parse_into_input(&src, "deep", "m", SourceProfile::TextRecord).unwrap();
    // walk the whole chain iteratively: every link must resolve
    let mut cursor = "R0";
    let mut depth = 0usize;
    loop {
        let r = input.lookup_record(cursor).expect("chain link resolves");
        depth += 1;
        match r.fields.iter().find(|(n, _)| n == "next") {
            Some((_, ac_sema_core::Ann::Record(next))) => cursor = next,
            Some(_) => panic!("next field must be a record ref"),
            None => break,
        }
    }
    assert_eq!(depth, 4096, "full chain walked");
    // publish + read back the head of the chain. F-796-R6 V-06 cost
    // attribution: publishing a 4096-record DAG must partition the
    // module graph ONCE — the old per-record miss rebuilt the full SCC
    // twice per record (O(V^2)), which is what made this case take
    // ~400s. The counter is the permanent algorithmic control.
    let root = scratch("budget-deep");
    let store = open_store(&root.join("store")).unwrap();
    let scc_before = ac_sema_probe::SCC_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    let t_pub = std::time::Instant::now();
    ac_sema_probe::publish_record_meta(&store, &input, "R0", Some("deep")).unwrap();
    let publish_ms = t_pub.elapsed().as_millis();
    let scc_after = ac_sema_probe::SCC_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    eprintln!(
        "deep-chain 4096: publish {publish_ms}ms, scc builds {}, per-record {:.1}ms",
        scc_after - scc_before,
        publish_ms as f64 / 4096.0
    );
    assert!(
        scc_after - scc_before <= 2,
        "4096-chain publication must partition the graph once, saw {} builds",
        scc_after - scc_before
    );
    match ac_sema_probe::lookup_only(
        &store,
        &input,
        &QueryKind::RecordFields {
            record: "R0".into(),
        },
    )
    .unwrap()
    {
        FetchOutcome::Hit { .. } => {}
        other => panic!("deep chain head page must hit, got {other:?}"),
    }
}

/// V-06 churn: 512 revision publish/release cycles; unrooted history is
/// reclaimed by GC (space converges), while the rooted+leased sentinel
/// survives untouched.
#[test]
fn sema_budget_churn_512_gc_converges_sentinel_survives() {
    let root = scratch("budget-churn");
    let store = open_store(&root.join("store")).unwrap();

    // sentinel: a module-index page under its own project + a lease
    let sentinel_src = "fn sentinel() int { return 42 }";
    let sentinel_input =
        parse_into_input(sentinel_src, "sentinel", "s", SourceProfile::Core).unwrap();
    let sentinel_art =
        ac_sema_probe::publish_module_index(&store, &sentinel_input, Some("sentinel-proj"))
            .unwrap();
    let lease = store
        .acquire_lease("sentinel-keep", 600, Some(&sentinel_art))
        .unwrap();

    // churn: 512 revisions of a body edit, each published UNROOTED
    // except the current one (previous current-root dropped each step)
    for i in 0..512u32 {
        let src = format!("fn main() int {{ return {i} + 1 }}");
        let input = parse_into_input(&src, "churn", "m", SourceProfile::Core).unwrap();
        let q = QueryKind::InferBody {
            fn_name: "main".into(),
        };
        let sum = match run_inference(&input, "main") {
            InferOutcome::Ok(s) => s,
            other => panic!("rev {i}: {other:?}"),
        };
        let art = publish_infer_result(&store, &input, &q, &sum, None).unwrap();
        // move the "current" soft root to this revision
        store.remove_roots("churn-current", "soft").unwrap();
        store.register_root("churn-current", &art, "soft").unwrap();
    }

    let before_gc = store.space().total;
    let _ = run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    let after_gc = store.space().total;
    assert!(
        after_gc < before_gc,
        "GC must reclaim unrooted churn history (before {before_gc}, after {after_gc})"
    );

    // sentinel still readable and live
    match ac_sema_probe::lookup_only(
        &store,
        &sentinel_input,
        &QueryKind::ListCandidates { domain: "fn" },
    )
    .unwrap()
    {
        FetchOutcome::Hit { .. } => {}
        other => panic!("sentinel must survive churn GC, got {other:?}"),
    }
    assert!(store.lease_live(&lease.lease_id).unwrap());
    lease.release();
}

/// The 512 KiB page-payload cap is enforced at encode: big-but-legal
/// passes, oversized is a controlled rejection (never an OOM write).
#[test]
fn sema_budget_small_query_payload_cap() {
    let ok = filler_page(0, 3000); // ~140 KiB
    let bytes = ac_sema_contract::encode_page(&ok).unwrap();
    assert!(bytes.len() < MAX_PAGE_BODY);
    let too = filler_page(1, 12000); // ~560 KiB > cap
    match ac_sema_contract::encode_page(&too) {
        Err(ac_sema_contract::CodecError::TooLarge { what, .. }) => {
            assert_eq!(what, "page body")
        }
        other => panic!("expected TooLarge, got {other:?}"),
    }
}
