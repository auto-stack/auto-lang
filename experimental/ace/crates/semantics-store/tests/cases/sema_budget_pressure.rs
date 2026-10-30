//! PLAN-796 T-04 budget/pressure tests (`sema_budget_` filter, V-06):
//! ≥32 MiB streamed pages under the 16 MiB managed budget, hot-LRU
//! caps, 4096-level deep chains full-path, 512-revision churn with GC
//! convergence and sentinel survival, and the 512 KiB payload cap.

#![cfg(windows)]

use ace_artifacts::identity::QueryKey;
use ace_artifacts::store::{run_gc, GcConfig};
use ace_core::source::SourceProfile;
use ace_semantics::adapter::parse_into_input;
use ace_semantics::query::{fn_body_canonical, QueryKind};
use ace_semantics::{InferOutcome, InferSession};
use ace_semantics_contract::{
    ContentVersion, DeclKind, DeclRef, FieldDesc, PackageId, PageHeader, PageKind, RecordMeta,
    RecordShape, SemaPage, TypeDesc, MAX_PAGE_BODY,
};
use ace_semantics_store::budget::{
    publish_streamed, HotPages, SessionMeter, HOT_BUDGET, MANAGED_BUDGET, SCRATCH_BUDGET,
};
use ace_semantics_store::{open_store, publish_infer_result, run_inference, FetchOutcome};

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
    let group = ace_semantics_contract::RecGroup {
        members: vec![shape.clone()],
    };
    SemaPage {
        header: PageHeader {
            schema: ace_semantics_contract::SEMA_PAGE_SCHEMA_V2.into(),
            producer: ace_semantics_contract::SEMA_PRODUCER_ID.into(),
            codec_version: ace_semantics_contract::SEMA_CODEC_VERSION,
            kind: PageKind::RecordMeta,
            content: group.group_content(),
        },
        body: ace_semantics_contract::PageBody::RecordMeta(RecordMeta {
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
    let mem_before = ace_artifacts::sysinfo::imp::current_process_memory().private_bytes;

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
                let now = ace_artifacts::sysinfo::imp::current_process_memory().private_bytes;
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
                ace_semantics_contract::PageBody::RecordMeta(m) => m.shape.decl.name.clone(),
                _ => "x".into(),
            };
            (
                ace_semantics_store::SEMA_LAYER,
                QueryKey {
                    layer: ace_semantics_store::SEMA_LAYER,
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

/// The 512 KiB page-payload cap is enforced at encode: big-but-legal
/// passes, oversized is a controlled rejection (never an OOM write).
#[test]
fn sema_budget_small_query_payload_cap() {
    let ok = filler_page(0, 3000); // ~140 KiB
    let bytes = ace_semantics_contract::encode_page(&ok).unwrap();
    assert!(bytes.len() < MAX_PAGE_BODY);
    let too = filler_page(1, 12000); // ~560 KiB > cap
    match ace_semantics_contract::encode_page(&too) {
        Err(ace_semantics_contract::CodecError::TooLarge { what, .. }) => {
            assert_eq!(what, "page body")
        }
        other => panic!("expected TooLarge, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// F-796-R7-02: conservative pre-allocation accounting
// ---------------------------------------------------------------------------

fn lets_input(n: usize, name_len: usize) -> ace_semantics::SemaInput {
    let mut src = String::from("fn main() int {\n");
    for i in 0..n {
        let mut name = format!("v{i}");
        while name.len() < name_len {
            name.push('x');
        }
        src.push_str(&format!("let {name} = {i}\n"));
    }
    src.push_str("return 1\n}");
    parse_into_input(&src, "r7budget", "m", SourceProfile::Core).unwrap()
}

fn run_lets(
    n: usize,
    name_len: usize,
    family: &str,
) -> (ace_semantics_store::QueryReport, SessionMeter) {
    let root = scratch(family);
    let store = open_store(&root.join("store")).unwrap();
    let input = lets_input(n, name_len);
    let q = QueryKind::InferBody {
        fn_name: "main".into(),
    };
    let mut meter = SessionMeter::new();
    let report =
        ace_semantics_store::run_query_flow(&store, &input, &q, None, Some(&mut meter)).unwrap();
    (report, meter)
}

/// 10k facts publish inside the 8MiB scratch budget; 20k exceed the
/// contract and are a typed `ResourceExhausted` BEFORE any solve or
/// publish (the R7 allocator probe measured 13.2MiB live for 20k while
/// the old ledger said 2.5MiB and still published).
#[test]
fn sema_budget_large_inference_rejects_before_publish() {
    let (ok, ok_meter) = run_lets(10_000, 4, "r7-budget-10k");
    assert_eq!(
        ok.disposition,
        ace_semantics_store::Disposition::PublishedMiss
    );
    assert!(
        ok_meter.managed_bytes() <= MANAGED_BUDGET,
        "10k managed ledger {} > {MANAGED_BUDGET}",
        ok_meter.managed_bytes()
    );
    assert!(
        ok_meter.scratch_peak() >= 6 * 1024 * 1024,
        "10k must reserve conservatively (peak {} bytes)",
        ok_meter.scratch_peak()
    );

    let (rej, rej_meter) = run_lets(20_000, 4, "r7-budget-20k");
    assert_eq!(
        rej.disposition,
        ace_semantics_store::Disposition::ResourceExhausted,
        "20k must be rejected by the budget front door: {:?}",
        rej.reason
    );
    assert!(
        rej_meter.rejections() >= 1,
        "the ledger must record the rejection"
    );
    assert!(
        rej_meter.scratch_peak() <= SCRATCH_BUDGET,
        "peak {} exceeds the scratch budget",
        rej_meter.scratch_peak()
    );
    // nothing was published by the rejected query
    let root = std::env::temp_dir().join("ac796-r7-budget-20k");
    let store = open_store(&root.join("store")).unwrap();
    assert!(matches!(
        ace_semantics_store::lookup_only(
            &store,
            &lets_input(20_000, 4),
            &QueryKind::InferBody {
                fn_name: "main".into()
            }
        )
        .unwrap(),
        FetchOutcome::Miss
    ));
}

/// Long identifier names are part of the reservation (the old per-node
/// constant ignored them).
#[test]
fn sema_budget_long_names_are_accounted() {
    let (_, short) = run_lets(2_000, 4, "r7-budget-short-names");
    let (_, long) = run_lets(2_000, 200, "r7-budget-long-names");
    assert!(
        long.scratch_peak() >= short.scratch_peak().saturating_add(700_000),
        "200-char names over 2000 lets must add ≥700KB to the reservation (short {}, long {})",
        short.scratch_peak(),
        long.scratch_peak()
    );
}

/// A cache HIT still reserves the read/decode window BEFORE the payload
/// is allocated, and is charged on the ledger.
#[test]
fn sema_budget_cache_readback_reserves_before_decode() {
    let root = scratch("r7-budget-readback");
    let store = open_store(&root.join("store")).unwrap();
    let input = lets_input(10_000, 4);
    let q = QueryKind::InferBody {
        fn_name: "main".into(),
    };
    let first = ace_semantics_store::run_query_flow(&store, &input, &q, None, None).unwrap();
    assert_eq!(
        first.disposition,
        ace_semantics_store::Disposition::PublishedMiss
    );
    let mut meter = SessionMeter::new();
    let second =
        ace_semantics_store::run_query_flow(&store, &input, &q, None, Some(&mut meter)).unwrap();
    assert_eq!(second.disposition, ace_semantics_store::Disposition::Hit);
    assert_eq!(second.compute_calls, 0);
    assert!(
        meter.scratch_peak() >= 1_000_000,
        "the read/decode reservation must precede the store read (peak {})",
        meter.scratch_peak()
    );
}

/// F-796-R8-01: 1000-char names are live in MULTIPLE copies (binding,
/// summary, clone, manifest, encode). 2000 occurrences exceed the
/// 8MiB solve reservation; 1000 occurrences exceed it in the publish
/// window. Both must be typed `ResourceExhausted` BEFORE the encode —
/// the R8 probe measured 9.26MB/4.63MB actual live with a 5.57MB/2.79MB
/// ledger and a late codec error.
#[test]
fn sema_budget_long_names_reject_before_encode() {
    let (rej2k, meter2k) = run_lets(2_000, 1_000, "r8-long-2000");
    assert_eq!(
        rej2k.disposition,
        ace_semantics_store::Disposition::ResourceExhausted,
        "2000 long names must be rejected before solve/publish: {:?}",
        rej2k.reason
    );
    assert!(meter2k.rejections() >= 1);
    let reason2k = rej2k.reason.clone().unwrap_or_default();
    assert!(
        reason2k.contains("scratch") || reason2k.contains("page"),
        "typed resource reason must name the ledger: {reason2k}"
    );

    let (rej1k, meter1k) = run_lets(1_000, 1_000, "r8-long-1000");
    assert_eq!(
        rej1k.disposition,
        ace_semantics_store::Disposition::ResourceExhausted,
        "1000 long names must be rejected in the publish window: {:?}",
        rej1k.reason
    );
    assert!(meter1k.rejections() >= 1);
    let reason1k = rej1k.reason.clone().unwrap_or_default();
    assert!(
        reason1k.contains("scratch") || reason1k.contains("page"),
        "{reason1k}"
    );

    // no page was published by either rejected query
    let root = std::env::temp_dir().join("ac796-r8-long-2000");
    let store = open_store(&root.join("store")).unwrap();
    assert!(matches!(
        ace_semantics_store::lookup_only(
            &store,
            &lets_input(2_000, 1_000),
            &QueryKind::InferBody {
                fn_name: "main".into()
            }
        )
        .unwrap(),
        FetchOutcome::Miss
    ));
}

/// The page-size front door is SEPARATE from the memory ledger: 1000
/// names of 600 chars fit in memory but their encoded body exceeds the
/// 512KiB small-query cap — typed `ResourceExhausted` (ledger=page),
/// never a late codec error.
#[test]
fn sema_budget_oversized_page_rejected_as_resource_before_encode() {
    let (rej, meter) = run_lets(1_000, 600, "r8-page-front-door");
    assert_eq!(
        rej.disposition,
        ace_semantics_store::Disposition::ResourceExhausted,
        "{:?}",
        rej.reason
    );
    let reason = rej.reason.clone().unwrap_or_default();
    assert!(
        reason.contains("page"),
        "the pre-encode page ledger must fire: {reason}"
    );
    assert!(meter.rejections() >= 1);
}

/// A long-name page that FITS the budgets is still served on readback
/// with the pre-decode reservation charged (cold compute vs readback
/// accounting is symmetric).
#[test]
fn sema_budget_long_name_readback_is_charged() {
    let root = scratch("r8-long-readback");
    let store = open_store(&root.join("store")).unwrap();
    let input = lets_input(400, 1_000);
    let q = QueryKind::InferBody {
        fn_name: "main".into(),
    };
    let first = ace_semantics_store::run_query_flow(&store, &input, &q, None, None).unwrap();
    assert_eq!(
        first.disposition,
        ace_semantics_store::Disposition::PublishedMiss
    );
    let mut meter = SessionMeter::new();
    let second =
        ace_semantics_store::run_query_flow(&store, &input, &q, None, Some(&mut meter)).unwrap();
    assert_eq!(second.disposition, ace_semantics_store::Disposition::Hit);
    assert_eq!(second.compute_calls, 0);
    assert!(
        meter.scratch_peak() >= 1_000_000,
        "readback must reserve the decode window (peak {})",
        meter.scratch_peak()
    );
}

/// F-796-R9-02: the FINAL page/manifest includes the target function
/// name as the `fn-body` positive origin. A 600k-char target name is a
/// typed `ResourceExhausted` on the page ledger BEFORE clone/encode
/// (the R9 probe previously got a late codec error); 400k still
/// publishes below the 512KiB cap.
#[test]
fn sema_budget_target_name_is_part_of_the_page_budget() {
    fn target_name_input(suffix: usize) -> (String, ace_semantics::SemaInput) {
        let name = format!("f{}", "x".repeat(suffix));
        let src = format!("fn {name}() int {{ return 1 }}");
        let input = parse_into_input(&src, "r9", "m", SourceProfile::Core).unwrap();
        (name, input)
    }

    let root = scratch("r9-target-400k");
    let store = open_store(&root.join("store")).unwrap();
    let (name, input) = target_name_input(400_000);
    let q = QueryKind::InferBody { fn_name: name };
    let mut meter = SessionMeter::new();
    let ok =
        ace_semantics_store::run_query_flow(&store, &input, &q, None, Some(&mut meter)).unwrap();
    assert_eq!(
        ok.disposition,
        ace_semantics_store::Disposition::PublishedMiss,
        "400k target name page is under the cap: {:?}",
        ok.reason
    );

    let root2 = scratch("r9-target-600k");
    let store2 = open_store(&root2.join("store")).unwrap();
    let (name2, input2) = target_name_input(600_000);
    let q2 = QueryKind::InferBody { fn_name: name2 };
    let mut meter2 = SessionMeter::new();
    let rej = ace_semantics_store::run_query_flow(&store2, &input2, &q2, None, Some(&mut meter2))
        .unwrap();
    assert_eq!(
        rej.disposition,
        ace_semantics_store::Disposition::ResourceExhausted,
        "600k target name must be a typed page rejection: {:?}",
        rej.reason
    );
    let reason = rej.reason.clone().unwrap_or_default();
    assert!(
        reason.contains("page"),
        "the page ledger must name the oversized final page: {reason}"
    );
    assert!(meter2.rejections() >= 1);
    assert!(
        matches!(
            ace_semantics_store::lookup_only(&store2, &input2, &q2).unwrap(),
            FetchOutcome::Miss
        ),
        "no page may be published by the rejected query"
    );
}

// Explicit formal benchmark or sustained scale/stress contract.
mod performance {
    use super::*;

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
                Some((_, ace_semantics::Ann::Record(next))) => cursor = next,
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
        let scc_before = ace_semantics_store::SCC_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
        let t_pub = std::time::Instant::now();
        ace_semantics_store::publish_record_meta(&store, &input, "R0", Some("deep")).unwrap();
        let publish_ms = t_pub.elapsed().as_millis();
        let scc_after = ace_semantics_store::SCC_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
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
        match ace_semantics_store::lookup_only(
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
        let sentinel_art = ace_semantics_store::publish_module_index(
            &store,
            &sentinel_input,
            Some("sentinel-proj"),
        )
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
        match ace_semantics_store::lookup_only(
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
}
