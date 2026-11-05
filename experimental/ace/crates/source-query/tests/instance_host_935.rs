//! PLAN-935 T-05：实例宿主——单飞/集合候选原子切换/实例页持久化。
//!
//! * V-03 面：同键单产（productions=1, attempts=N）、异键隔离；
//! * V-04 面：A→B→A 历史复用须再验证（页信封/键/内容等值才回放）；
//! * V-05 面：commit 重验（取消/late/集合漂移保旧）；集合指纹随模板
//!   body/签名/需求漂移而变；
//! * 持久化：页 codec 往返 + 信封拒（他域字节/坏版本）+ ManagedStore 层
//!   `S935` 真落盘与键查询。

use ace_core::source::cst::{
    parse_front_with_caps, FrontBudget, FrontKind, FrontOutcome, FrontRequest, INSTANCES_CAPABILITY,
};
use ace_core::source::SourceProfile;
use ace_source_query::{
    set_fingerprint, InstanceHost, InstancePage, InstancePageError, MemoryInstanceSink,
    INSTANCES_ALGO, INSTANCES_PRODUCER, INSTANCES_SCHEMA,
};

const CHOOSE: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
}
";
const CHOOSE_SWAPPED: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return b }
    return a
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
}
";
const PKG: &str = "plan935";
const MOD: &str = "choose.at";

fn caps() -> Vec<String> {
    vec![INSTANCES_CAPABILITY.to_string()]
}

fn parse(src: &str) -> ace_core::source::ast::Program {
    match parse_front_with_caps(
        FrontRequest {
            src,
            profile: SourceProfile::Scalar,
            kind: FrontKind::WholeModule,
            budget: FrontBudget::WORK_OWNED_64MIB,
        },
        &caps(),
    ) {
        FrontOutcome::Ok(ok) => ok.program,
        other => panic!("{other:?}"),
    }
}

#[test]
fn page_codec_roundtrip_and_envelope_refusals() {
    let mut host = InstanceHost::new(MemoryInstanceSink::default());
    let cand = host.prepare_set(&parse(CHOOSE), PKG, MOD, &caps()).expect("prepare");
    assert_eq!(cand.instances.len(), 2);
    let page = InstancePage::from(&cand.instances[0]);
    let bytes = ace_source_query::encode_instance_page(&page).expect("encode");
    assert!(bytes.len() <= 512 * 1024);
    let back = ace_source_query::decode_instance_page(&bytes).expect("decode");
    assert_eq!(back, page);
    // 坏 magic / 坏版本 → 拒（不重释）。
    let mut bad = bytes.clone();
    bad[0] = b'X';
    assert_eq!(
        ace_source_query::decode_instance_page(&bad),
        Err(InstancePageError::BadMagic)
    );
    let mut bad = bytes.clone();
    bad[6] = 9;
    assert!(matches!(
        ace_source_query::decode_instance_page(&bad),
        Err(InstancePageError::BadVersion { found: 9 })
    ));
    // 他域信封（伪造 producer）→ 拒。
    let mut foreign = page.clone();
    foreign.producer = "plan934/source-sq";
    let bytes = ace_source_query::encode_instance_page(&foreign).expect("encode");
    assert!(matches!(
        ace_source_query::decode_instance_page(&bytes),
        Err(InstancePageError::BadEnvelope { .. })
    ));
    assert_eq!(INSTANCES_PRODUCER, "plan935/source-instances");
    assert_eq!(INSTANCES_SCHEMA, "ac935/instances/v1");
    assert_eq!(INSTANCES_ALGO, "ac935/instances-discovery/2");
}

/// V-03：同键单飞——两个调用点/两次请求同一实例，productions=1；
/// 异键隔离——另一实例独立计数。commit 原子安装整个集合。
#[test]
fn single_flight_dedup_and_atomic_install() {
    let mut host = InstanceHost::new(MemoryInstanceSink::default());
    let prog = parse(CHOOSE);
    let mut cand = host.prepare_set(&prog, PKG, MOD, &caps()).expect("prepare");
    let i32_key = cand
        .instances
        .iter()
        .find(|i| i.concrete_name.contains("i32"))
        .unwrap()
        .key
        .clone();
    // commit（含全部实例页的单飞生产）。
    host.commit(cand, &prog, PKG, MOD, &caps(), None)
        .expect("commit");
    let cur_fp = host.current().expect("installed").set_fp.clone();
    assert_eq!(host.current().unwrap().instances.len(), 2);
    assert_eq!(host.revision(), 1);
    assert_eq!(host.productions(), 2, "one production per distinct key");
    // 同键再请求 → 复用（attempts 增、productions 不增）。
    let a2 = host.attempts();
    let page = InstancePage::from(
        host.current()
            .unwrap()
            .instances
            .iter()
            .find(|i| i.key == i32_key)
            .unwrap(),
    );
    let (_p, fresh) = host.produce(&page).expect("produce");
    assert!(!fresh, "same key reuses the single production");
    assert_eq!(host.productions(), 2);
    assert_eq!(host.attempts(), a2 + 1);
    // 集合指纹对实例集合区分（V-03：不同 T/集合不混淆）。
    let other = host.prepare_set(&parse(CHOOSE_SWAPPED), PKG, MOD, &caps()).expect("p2");
    assert_ne!(set_fingerprint(&other.instances), cur_fp);
}

/// V-05：commit 重验——取消、late（宿主已前进）、集合漂移（模板体已改）
/// 都保旧集合完整。
#[test]
fn commit_revalidation_keeps_old_set() {
    let prog = parse(CHOOSE);
    // 1) 取消。
    {
        let mut host = InstanceHost::new(MemoryInstanceSink::default());
        let cand = host.prepare_set(&prog, PKG, MOD, &caps()).expect("p");
        let flag = std::sync::atomic::AtomicBool::new(true);
        let err = host
            .commit(cand, &prog, PKG, MOD, &caps(), Some(&flag))
            .unwrap_err();
        assert_eq!(err, ace_source_query::InstanceCommitError::Cancelled);
        assert!(host.current().is_none(), "nothing half-published");
    }
    // 2) late：prepare 后宿主 revision 前进（另一次 commit 成功）→ 旧候选拒。
    {
        let mut host = InstanceHost::new(MemoryInstanceSink::default());
        let stale = host.prepare_set(&prog, PKG, MOD, &caps()).expect("p1");
        let fresh = host.prepare_set(&prog, PKG, MOD, &caps()).expect("p2");
        host.commit(fresh, &prog, PKG, MOD, &caps(), None).expect("c1");
        let err = host
            .commit(stale, &prog, PKG, MOD, &caps(), None)
            .unwrap_err();
        assert!(matches!(
            err,
            ace_source_query::InstanceCommitError::LateResult { current: 1, candidate: 0 }
        ));
        assert_eq!(host.current().unwrap().revision, 1, "old set intact");
    }
    // 3) 集合漂移：prepare 后模板体换版 → 当前源集合指纹不同 → 拒。
    {
        let mut host = InstanceHost::new(MemoryInstanceSink::default());
        let cand = host.prepare_set(&prog, PKG, MOD, &caps()).expect("p");
        let swapped = parse(CHOOSE_SWAPPED);
        let err = host
            .commit(cand, &swapped, PKG, MOD, &caps(), None)
            .unwrap_err();
        assert!(matches!(
            err,
            ace_source_query::InstanceCommitError::SetChanged { .. }
        ));
        assert!(host.current().is_none());
    }
}

/// V-04 面：A→B→A——历史键可复用，但复用页必须与当前推导**内容等值**
/// （A 的页在 B 时代不能冒充）。
#[test]
fn aba_history_reuse_requires_equal_content() {
    let mut host = InstanceHost::new(MemoryInstanceSink::default());
    let prog_a = parse(CHOOSE);
    let cand_a = host.prepare_set(&prog_a, PKG, MOD, &caps()).expect("A");
    let key_a = cand_a.instances[0].key.clone();
    let page_a = InstancePage::from(&cand_a.instances[0]);
    host.commit(cand_a, &prog_a, PKG, MOD, &caps(), None).expect("install A");
    // B 时代：换版生产+安装（A 的页仍在 sink——历史保留）。
    let prog_b = parse(CHOOSE_SWAPPED);
    let cand_b = host.prepare_set(&prog_b, PKG, MOD, &caps()).expect("B");
    host.commit(cand_b, &prog_b, PKG, MOD, &caps(), None).expect("install B");
    // 回 A：同源推导 → 同键同内容 → 复用（productions 不增）。
    let prod_before = host.productions();
    let (p, fresh) = host.produce(&page_a).expect("A again");
    assert_eq!(p.key, key_a);
    assert!(!fresh, "A→B→A reuses the historical byte-identical page");
    assert_eq!(host.productions(), prod_before);
    // 但**不同内容**（伪改 demands 的页）不得复用——键同内容异 → 重产。
    let mut forged = page_a.clone();
    forged.demands += 99;
    let (_p2, fresh2) = host.produce(&forged).expect("produce forged");
    assert!(fresh2, "a different payload under the same key must not replay");
}

/// 持久化：ManagedStore 层 S935 真落盘 + 键查询 + 跨 sink 恢复。
#[test]
fn managed_store_s935_persistence() {
    let dir = std::env::temp_dir().join(format!("s935-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    {
        let mut sink = ace_source_query::ManagedInstanceSink::open(&dir, Some("p935"))
            .expect("open store");
        let cand = {
            let mut tmp = InstanceHost::new(ace_source_query::MemoryInstanceSink::default());
            let c = tmp.prepare_set(&parse(CHOOSE), PKG, MOD, &caps()).expect("p");
            c
        };
        for inst in &cand.instances {
            let page = InstancePage::from(inst);
            let art = ace_source_query::InstanceSink::put_page(&mut sink, &page).expect("put");
            assert!(!art.is_empty());
        }
    }
    {
        let sink = ace_source_query::ManagedInstanceSink::open(&dir, Some("p935"))
            .expect("reopen");
        // 键查询：页在（重开后的同键 get_by_key）。
        let mut tmp = InstanceHost::new(ace_source_query::MemoryInstanceSink::default());
        let cand = tmp.prepare_set(&parse(CHOOSE), PKG, MOD, &caps()).expect("p");
        for inst in &cand.instances {
            let got = ace_source_query::InstanceSink::get_by_key(&sink, &inst.key)
                .expect("page persisted");
            assert_eq!(got, InstancePage::from(inst));
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
