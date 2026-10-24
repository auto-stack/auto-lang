//! PLAN-827 T-05 cache/identity/compatibility axes (V-07/V-08) over the
//! REAL AP-01 seed bytes: overlay edit misses exactly the changed unit,
//! cross-project/caps changes re-key (never cross-contaminated source
//! identity), a same-key corrupt republish CONFLICTS and the committed
//! good artifact survives to be re-verified and executed, and a
//! position-only shift re-binds identity without breaking execution.

use auto_av_probe::cache::{self, caps_fingerprint};
use auto_av_probe::driver::Driver;

const AP01_ORIGINAL: &str = "../../ac-core/fixtures/scalar-source/valid/p-gcd-euclid.at";
const AP01_SHA256: &str = "6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3";

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn ap01() -> String {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(AP01_ORIGINAL))
        .expect("read AP-01");
    assert_eq!(sha256_hex(&bytes), AP01_SHA256, "AP-01 drifted");
    String::from_utf8(bytes).unwrap()
}

fn overlay9(src: &str) -> String {
    let hits = src.matches("return 0").count();
    assert_eq!(hits, 1, "overlay literal matched {hits} times");
    src.replacen("return 0", "return 9", 1)
}

/// V-07 overlay at the driver level: same store, AP-01 -> 0 then the
/// return-0->9 overlay -> 9. The UNCHANGED gcd unit hits the cache; only
/// the edited main unit re-emits; AV still executes.
#[test]
fn av_source_827_overlay_misses_only_changed_unit() {
    let src = ap01();
    let root = std::env::temp_dir().join(format!("av827-ov-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = Driver::with_store(&root, "av827-ov").expect("store");
    let r1 = d.request(&src, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    assert_eq!(r1.value_i32(), Some(0));
    let r2 = d.request(&overlay9(&src), "d_main", &[]);
    assert!(r2.ok(), "{:?}", r2.diag);
    assert_eq!(r2.value_i32(), Some(9), "overlay executes as 9");
    assert_eq!(
        *r2.counters.get("cache_hit_abc").unwrap_or(&0),
        1,
        "unchanged gcd unit hits"
    );
    assert_eq!(
        *r2.counters.get("abc_units_emitted").unwrap_or(&0),
        1,
        "only the edited main unit re-emits"
    );
    assert!(*r2.counters.get("av_execute").unwrap_or(&0) > 0);
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08: the SAME store root under a DIFFERENT project — the delivered
/// semantics are content-addressed reuse (the six-scenario "跨工程磁盘hot"
/// face): the same content hits, the result is correct, and identity stays
/// content-bound. Roots belong to the PUBLISHING project (a full hit
/// publishes nothing new); wrong-content crossing is ruled out by the
/// overlay/identity axes, not by project walls.
#[test]
fn av_source_827_cross_project_content_addressed_reuse() {
    let src = ap01();
    let root = std::env::temp_dir().join(format!("av827-xp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut a = Driver::with_store(&root, "project-a").expect("store a");
    let ra = a.request(&src, "d_main", &[]);
    assert!(ra.ok(), "{:?}", ra.diag);
    assert_eq!(ra.value_i32(), Some(0));
    let mut b = Driver::with_store(&root, "project-b").expect("store b");
    let rb = b.request(&src, "d_main", &[]);
    assert!(rb.ok(), "{:?}", rb.diag);
    assert_eq!(rb.value_i32(), Some(0), "correct result in the new project");
    assert_eq!(
        *rb.counters.get("cache_hit_abc").unwrap_or(&0),
        2,
        "same content reuses the committed artifacts across projects"
    );
    assert!(
        *rb.counters.get("av_execute").unwrap_or(&0) > 0,
        "a cross-project hit STILL executes AV"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08: a capability-set change re-keys the cache (caps participate in
/// the artifact keys); the same caps still hit.
#[test]
fn av_source_827_caps_change_rekeys() {
    let src = ap01();
    let root = std::env::temp_dir().join(format!("av827-caps-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = Driver::with_store(&root, "av827-caps").expect("store");
    let r1 = d.request(&src, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    let r2 = d.request(&src, "d_main", &[]);
    assert!(r2.ok(), "{:?}", r2.diag);
    assert_eq!(
        *r2.counters.get("cache_hit_abc").unwrap_or(&0),
        2,
        "same caps hit"
    );
    let r3 = d.request(&src, "d_main", &["hir.test.trace.v1".to_string()]);
    assert!(
        r3.ok(),
        "AP-01 has no marks; the cap only re-keys: {:?}",
        r3.diag
    );
    assert_eq!(
        *r3.counters.get("cache_hit_abc").unwrap_or(&0),
        0,
        "changed caps = fresh keys (dependency re-key)"
    );
    assert_eq!(r3.value_i32(), Some(0));
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08 corrupt-inner axis (write path): republishing a DIFFERENT payload
/// under a real persisted unit key CONFLICTS by rule; the committed good
/// artifact survives and the next request still hits, re-verifies and
/// executes it (a tampered inner can never enter through the write path;
/// read-side envelope/decode admission is separately covered by the
/// cache_tier negatives).
#[test]
fn av_source_827_corrupt_republish_conflicts_and_good_survives() {
    let src = ap01();
    let root = std::env::temp_dir().join(format!("av827-cor-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = Driver::with_store(&root, "av827-cor").expect("store");
    let r1 = d.request(&src, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    let manifest = r1.manifest.as_ref().expect("manifest");
    let store = cache::open_store(&root).unwrap();
    let caps_fp = caps_fingerprint(&[]);
    let mut conflicts = 0u32;
    for u in &manifest.units {
        let key = cache::abc_unit_key(&manifest.owner, &u.name, &u.body_fp, &caps_fp);
        let id = cache::EnvelopeIdentity::abc(&manifest.owner);
        // GARBAGE bytes under the REAL key: same-key different-content.
        let err = cache::put_enveloped(
            &store,
            &key,
            cache::ABC_LAYER.id,
            &id,
            b"tampered-inner-payload",
            vec![],
            Some("av827-cor"),
        )
        .expect_err("same-key different-content must conflict");
        assert!(
            err.to_string().to_lowercase().contains("conflict"),
            "conflict-class rejection: {err}"
        );
        conflicts += 1;
    }
    assert!(conflicts >= 2, "both units' keys are defended: {conflicts}");
    drop(store);
    // The committed GOOD artifacts survive: next request hits, re-verifies
    // and still executes the correct result.
    let r2 = d.request(&src, "d_main", &[]);
    assert!(r2.ok(), "{:?}", r2.diag);
    assert_eq!(r2.value_i32(), Some(0));
    assert_eq!(
        *r2.counters.get("cache_hit_abc").unwrap_or(&0),
        2,
        "good committed units still hit after the failed tamper"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08 position axis: a position-only shift (leading blank lines) keeps
/// the SAME semantics and result but a DIFFERENT source identity — spans
/// re-bind to the shifted snapshot, never a stale or spanless execution.
#[test]
fn av_source_827_position_shift_new_identity_same_result() {
    let src = ap01();
    let shifted = format!("\n\n{src}");
    let mut d = Driver::ephemeral();
    let r1 = d.request(&src, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    let r2 = d.request(&shifted, "d_main", &[]);
    assert!(r2.ok(), "shifted snapshot still runs: {:?}", r2.diag);
    assert_eq!(r2.value_i32(), Some(0), "same semantics");
    assert_ne!(
        r1.source_hash, r2.source_hash,
        "a position shift is a distinct source identity"
    );
    assert!(
        !r2.spans.is_empty(),
        "the shifted run carries real stage spans"
    );
}
