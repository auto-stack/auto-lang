//! PLAN-1033 (1024 F25 → AXTX-01/AXTX-02): warm-edit LATE-FAILURE
//! publication atomicity over the REAL session/store/anchor chain.
//!
//! Closing predicate (plan §5): when prepare/assemble/store/anchor fails
//! LATE in one warm edit, the source SHA, residents, file set and
//! reachable root are NOT partially published — the session stays wholly
//! at the OLD accepted state; the next read/run after a restart still
//! serves the OLD product; a retry after the fault clears succeeds.
//! Rootless CAS byproducts of the failed attempt are allowed (named).
//!
//! Fault seams are REAL, not cfg(test) (T-01 freeze):
//! * late assemble failure — the §5 delegating provider: full
//!   delegation, only the Nth `assemble_fileset` returns a named
//!   rejection (swapped in through `set_target`);
//! * late anchor-publication failure — a pre-seeded CONFLICTING payload
//!   under the NEXT anchor key (same layer + QueryKey, different bytes):
//!   the real `publish_anchor` put deterministically fails with
//!   `StoreError::Conflict` (never overwrites) as a controlled
//!   `a2x.resource`;
//! * prepare-stage quota failures stay covered by
//!   `store_cache::quota_gate_is_controlled_and_preserves_state`.
//!
//! Coverage rows (plan §6 r2): C/Rust Scalar (AXP1, A=41/B=61, restore
//! runs the old product); C RecordText/Collections (AXP2/AXP3 seed
//! anchors, resource/seed/required closure, old-anchor restore);
//! Rust RecordText/Collections keep the frozen no-anchor contract;
//! read-only, other-target residents, early parse rejection, same-source
//! retry. A→B→A and the quota gates stay pinned by the existing roots in
//! this same conformance binary.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::process::Command;

use ace_a2x_c::CTargetProvider;
use ace_a2x_core::reject::A2xReject;
use ace_a2x_core::semantic_view::{FnFacts, SemanticView};
use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_core::target::{Family, FileSet, FilesetRequest, TargetDescriptor, TargetProvider};
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::{
    A2xRejectOrResource, A2xSession, LAYER_FRAGMENT, LAYER_FRAGMENT_C_COLL, LAYER_FRAGMENT_C_RT,
    LAYER_SOURCE_ANCHOR, LAYER_SOURCE_ANCHOR_COLL, LAYER_SOURCE_ANCHOR_RT,
};
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::{QueryKey, RefKind};
use ace_artifacts::schema::LayerSpec;
use ace_artifacts::store::{ManagedStore, PutRequest};
use ace_core::source::SourceProfile;

// ---- frozen fixtures (byte-exact reuse of the store_cache family) ----

const COLD: &str = "fn helper(x int) int { return x * 2 }\nfn main() int { return helper(20) + 1 }";
const HELPER_B: &str = "fn helper(x int) int { return x * 3 }";

/// PLAN-925/933 record-text corpus shape on the C channel (BASE from
/// a2c_rt_method_warm: methods + free fn + caller).
const RT_SRC: &str = "type Counter {\n    value int\n    static fn new(n int) Counter { return Counter(n) }\n    fn read() int { return .value }\n    fn add(delta int) int { return .value + delta }\n}\nfn read() int { return 7 }\nfn main() int {\n    let c Counter = Counter.new(20)\n    return c.read() * 100 + c.add(1) * 10 + read()\n}";
const READ_B: &str = "fn read() int { return 8 }";

/// PLAN-926 collections corpus (a2c_coll_restore SRC): A=44.
const COLL_SRC: &str = "fn left_value() int { return 20 }\nfn right_value() int { return 22 }\ntype Item { name Str; code int }\nenum Choice { Data(Item)\nEnd }\nfn main() int {\n    var xs = List<Choice>.new()\n    xs.push(Choice.Data(Item(\"ab\", left_value() + right_value())))\n    let t = xs.remove(0)\n    is t {\n        Choice.Data(item) -> { return item.name.len() + item.code }\n        Choice.End -> { return 0 }\n    }\n}";
const LEFT_B: &str = "fn left_value() int { return 25 }";

/// PLAN-902 text-family corpus (store_rt_902 TEXT_A_FULL) for the Rust
/// no-anchor record-text row.
const RT_TEXT_A: &str = "fn main() int {\n    let a Str = \"ab\"\n    let b Str = a.clone()\n    var c Str = b.move\n    c = c + \"中\"\n    return a.len() * 10 + c.len()\n}\n";
const RT_TEXT_B_FRAG: &str = "fn main() int {\n    let a Str = \"ab\"\n    let b Str = a.clone()\n    var c Str = b.move\n    c = c + \"国\"\n    return a.len() * 10 + c.len()\n}";

const RT_CAP: &str = "ac.runtime.text.v1";

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/audit1033")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = out_base().join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Write an assembly, compile with rustc, run, return trimmed stdout.
fn run_rust(tag: &str, asm: &ace_a2x_store::Assembly) -> String {
    let dir = out_base().join(format!("run-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let exe = dir.join("case.exe");
    let mut cc = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()));
    cc.args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir);
    let out = auto_tool_execution::run_command_output(cc).expect("spawn rustc");
    assert!(
        out.status.success(),
        "rustc failed for {tag}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&exe).current_dir(&dir).output().expect("run");
    assert!(run.status.success());
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

/// Write a C assembly, compile with the frozen cl chain, run, return
/// trimmed stdout.
fn run_c(tag: &str, asm: &ace_a2x_store::Assembly) -> String {
    let dir = out_base().join(format!("crun-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let (out, _) = ace_a2x_worker::crun::compile_and_run_c(&dir, "case.exe").unwrap();
    out.trim().to_string()
}

fn generated_of<'a>(asm: &'a ace_a2x_store::Assembly, name: &str) -> &'a str {
    &asm.files
        .iter()
        .find(|(p, _)| p == name)
        .unwrap_or_else(|| panic!("file {name} present"))
        .1
}

/// Byte range of one TOP-LEVEL `fn <name>(` declaration (preceded
/// directly by a newline or start-of-source — indented method fns of a
/// `type` block never match; the range runs to the next top-level `\nfn `
/// or EOF).
fn range_of_decl(src: &str, name: &str) -> (usize, usize) {
    let pat = format!("fn {name}(");
    let bytes = src.as_bytes();
    let mut search_from = 0;
    let start = loop {
        let at = src[search_from..]
            .find(&pat)
            .map(|rel| search_from + rel)
            .unwrap_or_else(|| panic!("decl {name} present"));
        if at == 0 || bytes[at - 1] == b'\n' {
            break at;
        }
        search_from = at + 1;
    };
    let end = src[start + 1..]
        .find("\nfn ")
        .map(|rel| start + 1 + rel)
        .unwrap_or(src.len());
    (start, end)
}

/// The NEXT full source of one ranged edit.
fn next_source(base: &str, range: (usize, usize), fragment: &str) -> String {
    format!("{}{}{}", &base[..range.0], fragment, &base[range.1..])
}

// ---- the §5 delegating fault provider ----

/// PLAN-1033 §5: delegates descriptor/key/runtime/fragment/prelude
/// behavior COMPLETELY to the wrapped provider; fails ONLY the Nth
/// `assemble_fileset` call with a named rejection (the late assemble
/// failure, after residents are prepared).
struct FaultyAssembleProvider<P> {
    inner: P,
    fail_on: usize,
    calls: Cell<usize>,
}

impl<P: TargetProvider> FaultyAssembleProvider<P> {
    fn fail_on_nth(inner: P, n: usize) -> Self {
        Self {
            inner,
            fail_on: n,
            calls: Cell::new(0),
        }
    }
}

impl<P: TargetProvider> TargetProvider for FaultyAssembleProvider<P> {
    fn descriptor(&self) -> TargetDescriptor {
        self.inner.descriptor()
    }
    fn fragment_key_inputs(&self, runtime_sha256: &str) -> Vec<(String, String)> {
        self.inner.fragment_key_inputs(runtime_sha256)
    }
    fn fragment_layer_id(&self, family: Family) -> Result<&'static str, A2xReject> {
        self.inner.fragment_layer_id(family)
    }
    fn runtime_source(&self, view: &SemanticView<'_>, family: Family) -> Result<String, A2xReject> {
        self.inner.runtime_source(view, family)
    }
    fn produce_fragments(
        &self,
        view: &SemanticView<'_>,
        fns: &[&FnFacts<'_>],
        family: Family,
    ) -> Result<Vec<String>, A2xReject> {
        self.inner.produce_fragments(view, fns, family)
    }
    fn capture_prelude_seed(
        &self,
        view: &SemanticView<'_>,
        family: Family,
    ) -> Result<Option<String>, A2xReject> {
        self.inner.capture_prelude_seed(view, family)
    }
    fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject> {
        let n = self.calls.get() + 1;
        self.calls.set(n);
        if n == self.fail_on {
            return Err(A2xReject::new(
                "a2x.fault",
                "PLAN-1033 injected late assemble failure",
            ));
        }
        self.inner.assemble_fileset(req)
    }
}

// ---- the T-01 anchor-conflict seam ----

/// Pre-seed a CONFLICTING payload under the NEXT anchor key (same layer
/// + the same 5 key components `anchor_key` builds). The session's real
/// `publish_anchor` put then deterministically fails with
/// `StoreError::Conflict` (the store never overwrites) — a controlled
/// `a2x.resource` exactly at the anchor publication point.
/// The anchor-layer QueryKey of one assembly state — exactly the five
/// components `A2xSession::anchor_key` builds (QueryKey canonicalizes
/// the order itself).
fn anchor_query_key(
    layer: &'static LayerSpec,
    owner: &str,
    profile: &str,
    source_sha: &str,
    backend: &str,
    entry: &str,
) -> QueryKey {
    QueryKey::new(
        layer,
        vec![
            ("owner".into(), owner.to_string()),
            ("profile".into(), profile.to_string()),
            ("source_sha256".into(), source_sha.to_string()),
            ("backend".into(), backend.to_string()),
            ("entry".into(), entry.to_string()),
        ],
    )
}

fn seed_anchor_conflict(
    root: &Path,
    layer: &'static LayerSpec,
    owner: &str,
    profile: &str,
    next_sha: &str,
    backend: &str,
    entry: &str,
) {
    let store = ManagedStore::open(root).expect("open store for the seed");
    let key = anchor_query_key(layer, owner, profile, next_sha, backend, entry);
    store
        .put_artifact(PutRequest {
            layer: layer.id,
            key,
            kind: ChunkKind::ObjectManifest,
            payload: b"plan-1033 anchor conflict seed".to_vec(),
            edges: vec![],
            project: None,
        })
        .expect("seed the conflicting anchor row");
}

/// Resolve the fragment-layer artifact id of one anchored function under
/// a published anchor — the EXACT key the restore path rebuilds
/// (owner/decl/sem_fp + the provider's target-identity inputs).
fn fragment_artifact_id(
    root: &Path,
    layer: &'static LayerSpec,
    owner: &str,
    decl: &str,
    sem_fp: &str,
    runtime_sha256: &str,
    provider: &dyn TargetProvider,
) -> String {
    let store = ManagedStore::open(root).expect("open store read-only");
    let mut comps = vec![
        ("owner".into(), owner.to_string()),
        ("decl".into(), decl.to_string()),
        ("sem_fp".into(), sem_fp.to_string()),
    ];
    comps.extend(provider.fragment_key_inputs(runtime_sha256));
    let key = QueryKey::new(layer, comps);
    store
        .get_artifact(layer.id, &key)
        .expect("fragment lookup")
        .unwrap_or_else(|| panic!("fragment for `{decl}` is persisted"))
        .artifact_id
}

/// RV2 oracle (review round 1): the published anchor's REQUIRED edges
/// follow the EDITED fragment's real artifact id — the pre-edit id must
/// not be required (the stale-edge defect would exactly revert this).
#[track_caller]
fn assert_required_edge_follows_edited_fragment(
    root: &Path,
    anchor_layer: &'static LayerSpec,
    anchor_sha: &str,
    profile: &str,
    backend: &str,
    entry: &str,
    owner: &str,
    frag_layer: &'static LayerSpec,
    decl: &str,
    edited_fp: &str,
    pre_edit_fp: &str,
    runtime_sha256: &str,
    provider: &dyn TargetProvider,
) {
    let store = ManagedStore::open(root).expect("open store read-only");
    let anchor_id = store
        .get_artifact(
            anchor_layer.id,
            &anchor_query_key(anchor_layer, owner, profile, anchor_sha, backend, entry),
        )
        .expect("anchor lookup")
        .unwrap_or_else(|| panic!("anchor for {anchor_sha:?} is published"))
        .artifact_id;
    let edges = store.artifact_edges(&anchor_id).expect("anchor edges");
    let edited_id = fragment_artifact_id(
        root,
        frag_layer,
        owner,
        decl,
        edited_fp,
        runtime_sha256,
        provider,
    );
    assert!(
        edges
            .iter()
            .any(|(dst, k)| dst == &edited_id && *k == RefKind::RequiredData),
        "the rebound anchor requires the EDITED fragment `{decl}`"
    );
    if edited_fp != pre_edit_fp {
        let pre_edit_id = fragment_artifact_id(
            root,
            frag_layer,
            owner,
            decl,
            pre_edit_fp,
            runtime_sha256,
            provider,
        );
        assert!(
            edges.iter().all(|(dst, _)| dst != &pre_edit_id),
            "the rebound anchor must NOT require the pre-edit fragment of `{decl}` (stale edge)"
        );
    }
}

/// Assert the rejection is the named injected fault.
#[track_caller]
fn expect_fault_reject(err: &Result<ace_a2x_store::Assembly, A2xRejectOrResource>) {
    match err {
        Err(A2xRejectOrResource::Reject(r)) => {
            assert_eq!(r.code, "a2x.fault", "the injected assemble rejection");
        }
        other => panic!("expected the injected assemble rejection, got {:?}", other),
    }
}

/// Assert the rejection is the controlled store conflict at the anchor
/// publication point.
#[track_caller]
fn expect_anchor_conflict_resource(err: &Result<ace_a2x_store::Assembly, A2xRejectOrResource>) {
    match err {
        Err(A2xRejectOrResource::Resource(r)) => {
            assert_eq!(r.code, "a2x.resource");
            assert!(
                r.detail.to_lowercase().contains("conflict"),
                "the anchor put must fail as a store conflict, got: {r}"
            );
        }
        other => panic!(
            "expected anchor-conflict resource rejection, got {:?}",
            other
        ),
    }
}

fn helper_fp(anchor: &ace_a2x_store::Anchor, decl: &str) -> String {
    anchor
        .functions
        .iter()
        .find(|f| f.decl == decl)
        .unwrap_or_else(|| panic!("decl {decl} anchored"))
        .sem_fp
        .clone()
}

// ===========================================================================
// AXTX-01 — late-failure atomicity on the anchored scalar channels
// ===========================================================================

/// Rust scalar (AXP1): a late assemble failure and a late anchor-
/// publication failure in one ranged warm edit (A=41 → B=61) leave the
/// session wholly at A — old anchor identity/sem_fps, old binding (a B
/// base is stale), old residents (the retry reuses the byproduct store
/// hit); the restart restore of A is pure and RUNS the old product; a
/// retry after the fault clears succeeds (B=61, anchor rebound).
#[test]
fn audit1033_axtx_01() {
    // ---- axis 1: late assemble failure (delegating provider) ----
    let root = scratch("axtx01-assemble");
    let mut s = A2xSession::open(
        &root,
        "p1033",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    let cold = s.cold_assemble(COLD, false, &[], "main", false).unwrap();
    assert_eq!(run_rust("cold41", &cold), "value:41");
    let a_sha = sha256_hex(COLD.as_bytes());
    let anchor_a = s.last_anchor().expect("cold publishes AXP1").clone();
    assert_eq!(anchor_a.source_sha256, a_sha);
    let helper_a_fp = helper_fp(&anchor_a, "helper");

    let range = range_of_decl(COLD, "helper");
    let full_after = next_source(COLD, range, HELPER_B);
    let next_sha = sha256_hex(full_after.as_bytes());

    s.set_target(FaultyAssembleProvider::fail_on_nth(
        RustTargetProvider::v1(),
        1,
    ));
    let prints_before = s.stats().print;
    let err = s.warm_edit_ranged("helper", COLD, range, HELPER_B, "main", false);
    expect_fault_reject(&err);
    assert_eq!(
        s.stats().print,
        prints_before + 1,
        "the fragment WAS produced (and persisted as a byproduct) before the late failure"
    );

    // OLD acceptance state: the anchor keeps A's binding + helper sem_fp
    let anchor_now = s.last_anchor().expect("anchor survived").clone();
    assert_eq!(
        anchor_now.source_sha256, a_sha,
        "late assemble failure: the anchor keeps the OLD source binding"
    );
    assert_eq!(anchor_now, anchor_a, "the whole anchor is the OLD one");
    assert_eq!(helper_fp(&anchor_now, "helper"), helper_a_fp);
    // ...and the session binding is still A: a B-based edit is STALE
    let stale = s.warm_edit_ranged("helper", &full_after, range, HELPER_B, "main", false);
    match stale {
        Err(A2xRejectOrResource::Reject(r)) => assert_eq!(r.code, "a2x.stale-base"),
        other => panic!("B base must be stale after the failed edit, got {other:?}"),
    }

    // retry after the fault clears: the byproduct fragment is a REAL
    // store hit (zero print), the edit commits, the anchor rebinds to B
    s.set_target(RustTargetProvider::v1());
    let hits_before = s.stats().store_hit_fragment;
    let b = s
        .warm_edit_ranged("helper", COLD, range, HELPER_B, "main", false)
        .expect("retry after fault clears succeeds");
    assert_eq!(
        s.stats().print,
        prints_before + 1,
        "retry reuses the byproduct fragment (store hit, no re-print)"
    );
    assert!(s.stats().store_hit_fragment > hits_before);
    assert_eq!(run_rust("retry61", &b), "value:61");
    let anchor_b = s.last_anchor().expect("anchor rebound").clone();
    assert_eq!(anchor_b.source_sha256, next_sha);
    assert_ne!(helper_fp(&anchor_b, "helper"), helper_a_fp);
    // RV2: the rebound anchor's REQUIRED edges follow the EDITED helper
    // fragment's real artifact id — the pre-edit id must not be required
    // (the stale-edge defect reverted exactly this).
    assert_required_edge_follows_edited_fragment(
        &root,
        &LAYER_SOURCE_ANCHOR,
        &next_sha,
        "core",
        "auto-a2x-rust",
        "main",
        "probe",
        &LAYER_FRAGMENT,
        "helper",
        &helper_fp(&anchor_b, "helper"),
        &helper_a_fp,
        &anchor_b.runtime_sha256,
        &RustTargetProvider::v1(),
    );
    // the file set really switched (helper fragment changed, main kept)
    assert_ne!(
        generated_of(&cold, "generated.rs"),
        generated_of(&b, "generated.rs")
    );

    // ---- axis 2: late anchor-publication failure (seeded conflict) ----
    let root2 = scratch("axtx01-anchorconflict");
    let mut s2 = A2xSession::open(
        &root2,
        "p1033",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    let cold2 = s2.cold_assemble(COLD, false, &[], "main", false).unwrap();
    assert_eq!(run_rust("cold41b", &cold2), "value:41");
    seed_anchor_conflict(
        &root2,
        &LAYER_SOURCE_ANCHOR,
        "probe",
        "core",
        &next_sha,
        "auto-a2x-rust",
        "main",
    );
    let err2 = s2.warm_edit_ranged("helper", COLD, range, HELPER_B, "main", false);
    expect_anchor_conflict_resource(&err2);

    // OLD acceptance state: anchor wholly A; binding still A (stale B)
    let anchor_keep = s2.last_anchor().expect("anchor survived").clone();
    assert_eq!(anchor_keep.source_sha256, a_sha);
    assert_eq!(helper_fp(&anchor_keep, "helper"), helper_a_fp);
    let stale2 = s2.warm_edit_ranged("helper", &full_after, range, HELPER_B, "main", false);
    match stale2 {
        Err(A2xRejectOrResource::Reject(r)) => assert_eq!(r.code, "a2x.stale-base"),
        other => panic!("B base must be stale after the anchor conflict, got {other:?}"),
    }

    // restart: a FRESH session's next read still serves the OLD product
    // (pure restore of A — P0 — and the old assembly RUNS)
    drop(s2);
    let mut r2 = A2xSession::open(
        &root2,
        "p1033",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    let before = r2.stats().clone();
    let restored = r2
        .restore_source_bound(COLD, SourceProfile::Core, &[], "main")
        .expect("pure restore of the OLD state");
    let after = r2.stats().clone();
    assert_eq!(after.lower, before.lower, "restore is pure: no lowering");
    assert_eq!(after.print, before.print, "restore is pure: no printing");
    assert_eq!(after.persist, before.persist, "restore is pure: no writes");
    assert_eq!(
        restored.files, cold2.files,
        "restored files == cold A files"
    );
    assert_eq!(run_rust("restore41", &restored), "value:41");

    // and the SAME edit succeeds in a fault-free namespace (retry B=61)
    let root3 = scratch("axtx01-retry");
    let mut s3 = A2xSession::open(
        &root3,
        "p1033",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    s3.cold_assemble(COLD, false, &[], "main", false).unwrap();
    let b3 = s3
        .warm_edit_ranged("helper", COLD, range, HELPER_B, "main", false)
        .expect("clean namespace retries fine");
    assert_eq!(run_rust("cleanretry61", &b3), "value:61");
    assert_eq!(
        s3.last_anchor().unwrap().source_sha256,
        next_sha,
        "the retry's anchor binds B"
    );
}

/// C scalar (AXP1, the other end): the same two late-failure axes on the
/// C channel (profile Scalar, ranged authorization) — the old anchor and
/// binding survive byte-exactly, the restart restore of A is pure, and
/// the fault-free retry rebinds.
#[test]
fn audit1033_axtx_01_c_scalar() {
    let root = scratch("axtx01-c-assemble");
    let mut s = A2xSession::open(
        &root,
        "p1033c",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap();
    let cold = s
        .cold_assemble_profile(COLD, SourceProfile::Scalar, &[], "main", false)
        .unwrap();
    // (the cold VALUE oracle rides the restore run below — restored files
    // are byte-identical to cold's, keeping this case at 2 cl compiles)
    let a_sha = sha256_hex(COLD.as_bytes());
    let anchor_a = s.last_anchor().expect("C cold publishes AXP1").clone();
    assert_eq!(anchor_a.source_sha256, a_sha);

    let range = range_of_decl(COLD, "helper");
    let full_after = next_source(COLD, range, HELPER_B);
    let next_sha = sha256_hex(full_after.as_bytes());

    // axis 1: late assemble failure
    s.set_target(FaultyAssembleProvider::fail_on_nth(
        CTargetProvider::v1(),
        1,
    ));
    let err = s.warm_edit_ranged("helper", COLD, range, HELPER_B, "main", false);
    expect_fault_reject(&err);
    assert_eq!(
        s.last_anchor().unwrap().source_sha256,
        a_sha,
        "C late assemble failure keeps the OLD anchor"
    );

    // retry: rebind + real C value oracle
    s.set_target(CTargetProvider::v1());
    let b = s
        .warm_edit_ranged("helper", COLD, range, HELPER_B, "main", false)
        .expect("C retry succeeds");
    assert_eq!(run_c("c-retry61", &b), "value:61");
    assert_eq!(s.last_anchor().unwrap().source_sha256, next_sha);

    // axis 2: late anchor-publication failure (seeded conflict) in a
    // fresh namespace + the restart restore of A
    let root2 = scratch("axtx01-c-anchorconflict");
    let mut s2 = A2xSession::open(
        &root2,
        "p1033c",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap();
    let cold2 = s2
        .cold_assemble_profile(COLD, SourceProfile::Scalar, &[], "main", false)
        .unwrap();
    seed_anchor_conflict(
        &root2,
        &LAYER_SOURCE_ANCHOR,
        "probe",
        "scalar",
        &next_sha,
        "auto-a2x-c",
        "main",
    );
    let err2 = s2.warm_edit_ranged("helper", COLD, range, HELPER_B, "main", false);
    expect_anchor_conflict_resource(&err2);
    assert_eq!(
        s2.last_anchor().unwrap().source_sha256,
        a_sha,
        "C anchor conflict keeps the OLD anchor"
    );

    drop(s2);
    let mut r2 = A2xSession::open(
        &root2,
        "p1033c",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap();
    let before = r2.stats().clone();
    let restored = r2
        .restore_source_bound(COLD, SourceProfile::Scalar, &[], "main")
        .expect("C pure restore of the OLD state");
    let after = r2.stats().clone();
    assert_eq!(after.lower, before.lower);
    assert_eq!(after.print, before.print);
    assert_eq!(after.persist, before.persist);
    assert_eq!(restored.files, cold2.files);
    assert_eq!(run_c("c-restore41", &restored), "value:41");
}

/// C record-text (AXP2, review round-1 split of the anchored-families
/// case for the ≤10s per-case gate): the seed-carrying anchor — a late
/// assemble failure keeps the old anchor WITH ITS SEED and the whole old
/// closure; the retry rebinds and its REQUIRED edges follow the EDITED
/// fragment (RV2 oracle); the restart restore of A is pure (blind,
/// seed-driven) and byte-identical.
#[test]
fn audit1033_axtx_01_c_rt() {
    let root = scratch("axtx01-crt");
    let mut s = A2xSession::open(
        &root,
        "p1033rt",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap();
    let cold = s
        .cold_assemble_profile(RT_SRC, SourceProfile::Record, &[], "main", false)
        .unwrap();
    // (the cold VALUE oracle rides the restore run below — restored files
    // are byte-identical to cold's, keeping this case at 2 cl compiles)
    let a_sha = sha256_hex(RT_SRC.as_bytes());
    let anchor_a = s.last_anchor().expect("C rt cold publishes AXP2").clone();
    assert_eq!(anchor_a.source_sha256, a_sha);
    let seed_a = anchor_a.prelude_seed.clone().unwrap_or_default();

    let range = range_of_decl(RT_SRC, "read");
    let full_after = next_source(RT_SRC, range, READ_B);
    let next_sha = sha256_hex(full_after.as_bytes());

    s.set_target(FaultyAssembleProvider::fail_on_nth(
        CTargetProvider::v1(),
        1,
    ));
    let err = s.warm_edit_ranged("read", RT_SRC, range, READ_B, "main", false);
    expect_fault_reject(&err);
    let anchor_now = s.last_anchor().expect("anchor survived").clone();
    assert_eq!(anchor_now.source_sha256, a_sha);
    assert_eq!(
        anchor_now.prelude_seed.clone().unwrap_or_default(),
        seed_a,
        "the OLD seed-carrying anchor is intact"
    );

    // retry: rebind (+1 value) — the required closure follows the edited fn
    s.set_target(CTargetProvider::v1());
    let b = s
        .warm_edit_ranged("read", RT_SRC, range, READ_B, "main", false)
        .expect("rt retry succeeds");
    assert_eq!(run_c("rt-retry", &b), "value:2218");
    let anchor_b = s.last_anchor().expect("anchor rebound").clone();
    assert_eq!(
        anchor_b.source_sha256, next_sha,
        "the rt retry's anchor rebinds to B"
    );
    // RV2: the AXP2 anchor's required edges follow the EDITED `read`
    // fragment (record profile label is "record")
    assert_required_edge_follows_edited_fragment(
        &root,
        &LAYER_SOURCE_ANCHOR_RT,
        &next_sha,
        "record",
        "auto-a2x-c",
        "main",
        "probe",
        &LAYER_FRAGMENT_C_RT,
        "read",
        &helper_fp(&anchor_b, "read"),
        &helper_fp(&anchor_a, "read"),
        &anchor_b.runtime_sha256,
        &CTargetProvider::v1(),
    );

    // restart: fresh session restore of A is pure and byte-identical
    drop(s);
    let mut r = A2xSession::open(
        &root,
        "p1033rt",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap();
    let before = r.stats().clone();
    let restored = r
        .restore_source_bound(RT_SRC, SourceProfile::Record, &[], "main")
        .expect("pure restore of the OLD rt state");
    let after = r.stats().clone();
    assert_eq!(after.lower, before.lower);
    assert_eq!(after.print, before.print);
    assert_eq!(after.persist, before.persist);
    assert_eq!(restored.files, cold.files, "blind re-assembly == cold A");
    assert_eq!(run_c("rt-restore", &restored), "value:2217");
}

/// C collections (AXP3, review round-1 split of the anchored-families
/// case): the same late-failure contract on the collections seed anchor.
#[test]
fn audit1033_axtx_01_c_coll() {
    let root2 = scratch("axtx01-ccoll");
    let mut s2 = A2xSession::open(
        &root2,
        "p1033coll",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap();
    let cold2 = s2
        .cold_assemble_profile(COLL_SRC, SourceProfile::Collections, &[], "main", false)
        .unwrap();
    // (the cold VALUE oracle rides the restore run below — restored files
    // are byte-identical to cold's, keeping this case at 2 cl compiles)
    let a_sha2 = sha256_hex(COLL_SRC.as_bytes());
    let anchor_a2 = s2
        .last_anchor()
        .expect("C coll cold publishes AXP3")
        .clone();
    assert_eq!(anchor_a2.source_sha256, a_sha2);
    let seed_a2 = anchor_a2.prelude_seed.clone().unwrap_or_default();

    let range2 = range_of_decl(COLL_SRC, "left_value");
    let full_after2 = next_source(COLL_SRC, range2, LEFT_B);
    let next_sha2 = sha256_hex(full_after2.as_bytes());
    s2.set_target(FaultyAssembleProvider::fail_on_nth(
        CTargetProvider::v1(),
        1,
    ));
    let err2 = s2.warm_edit_ranged("left_value", COLL_SRC, range2, LEFT_B, "main", false);
    expect_fault_reject(&err2);
    let anchor_now2 = s2.last_anchor().expect("anchor survived").clone();
    assert_eq!(anchor_now2.source_sha256, a_sha2);
    assert_eq!(
        anchor_now2.prelude_seed.clone().unwrap_or_default(),
        seed_a2,
        "the OLD seed-carrying collections anchor is intact"
    );

    s2.set_target(CTargetProvider::v1());
    let b2 = s2
        .warm_edit_ranged("left_value", COLL_SRC, range2, LEFT_B, "main", false)
        .expect("coll retry succeeds");
    // "ab".len()=2 + code(25+22=47) = 49
    assert_eq!(run_c("coll-retry", &b2), "value:49");
    // RV2: the AXP3 anchor's required edges follow the EDITED left_value
    // fragment (collections profile label is "unsupported" — frozen)
    assert_required_edge_follows_edited_fragment(
        &root2,
        &LAYER_SOURCE_ANCHOR_COLL,
        &next_sha2,
        "unsupported",
        "auto-a2x-c",
        "main",
        "probe",
        &LAYER_FRAGMENT_C_COLL,
        "left_value",
        &helper_fp(s2.last_anchor().unwrap(), "left_value"),
        &helper_fp(&anchor_a2, "left_value"),
        &s2.last_anchor().unwrap().runtime_sha256,
        &CTargetProvider::v1(),
    );

    drop(s2);
    let mut r2 = A2xSession::open(
        &root2,
        "p1033coll",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap();
    let before2 = r2.stats().clone();
    let restored2 = r2
        .restore_source_bound(COLL_SRC, SourceProfile::Collections, &[], "main")
        .expect("pure restore of the OLD coll state");
    let after2 = r2.stats().clone();
    assert_eq!(after2.lower, before2.lower);
    assert_eq!(after2.print, before2.print);
    assert_eq!(after2.persist, before2.persist);
    assert_eq!(restored2.files, cold2.files, "blind re-assembly == cold A");
    assert_eq!(run_c("coll-restore", &restored2), "value:44");
}

// ===========================================================================
// AXTX-02 — no-anchor families, boundaries and isolation
// ===========================================================================

/// Rust record-text/collections keep the frozen NO-ANCHOR contract: a
/// late assemble failure leaves no partial publication (no anchor exists
/// to strand), the session still serves/retries, and the restart
/// re-assembles purely from store hits, byte-identical.
#[test]
fn audit1033_axtx_02() {
    // ---- Rust record text (fragment path, no anchor) ----
    let root = scratch("axtx02-rt");
    let mut s =
        A2xSession::open(&root, "p1033x", "owner", 8 << 20, RustTargetProvider::v1()).unwrap();
    let caps = [RT_CAP.to_string()];
    let cold = s
        .cold_assemble_profile(RT_TEXT_A, SourceProfile::TextRecord, &caps, "main", false)
        .unwrap();
    assert!(
        s.last_anchor().is_none(),
        "Rust record text anchors nothing"
    );
    let cold_files = cold.files.clone();

    s.set_target(FaultyAssembleProvider::fail_on_nth(
        RustTargetProvider::v1(),
        1,
    ));
    let err = s.warm_edit("main", RT_TEXT_B_FRAG, "main", false);
    expect_fault_reject(&err);
    assert!(
        s.last_anchor().is_none(),
        "no anchor appeared through the failure"
    );

    // resident preservation on the no-anchor channel: a whitespace-only
    // variant of the A body has the SAME sem_fp — it must come from the
    // SURVIVING A resident (zero production), not a re-produced B state
    s.set_target(RustTargetProvider::v1());
    let ws_a = "fn main() int {\n    let a Str = \"ab\"\n    let b Str = a.clone()\n    var c Str = b.move\n    c = c + \"中\"\n    return a.len() * 10 + c.len()\n }";
    let prints_after_failure = s.stats().print;
    let reuse_before = s.stats().fragments_reused_resident;
    let ws = s
        .warm_edit("main", ws_a, "main", false)
        .expect("ws edit after failure");
    assert_eq!(
        s.stats().print,
        prints_after_failure,
        "the A resident survived the failed edit (whitespace edit reuses it)"
    );
    assert!(s.stats().fragments_reused_resident > reuse_before);
    assert_eq!(
        ws.files, cold_files,
        "the surviving resident serves A bytes"
    );

    // retry (fault cleared) and A-return both work; residents/store serve
    let b = s
        .warm_edit("main", RT_TEXT_B_FRAG, "main", false)
        .expect("rt retry");
    assert_ne!(
        b.files, cold_files,
        "the retry really switched the file set"
    );
    let a2 = s
        .warm_edit("main", RT_TEXT_A_FRAG_FULL, "main", false)
        .expect("A return");
    assert_eq!(a2.files, cold_files, "A return reuses the historical bytes");

    // restart over the same store: pure hits, byte-identical
    drop(s);
    let mut r =
        A2xSession::open(&root, "p1033x", "owner", 8 << 20, RustTargetProvider::v1()).unwrap();
    let prints_before = r.stats().print;
    let restarted = r
        .restart_assemble_profile(RT_TEXT_A, SourceProfile::TextRecord, &caps, "main")
        .expect("restart re-admission");
    assert_eq!(
        r.stats().print,
        prints_before,
        "restart is pure store hits (no anchor channel)"
    );
    assert_eq!(restarted.files, cold_files);

    // ---- Rust collections (ranged path, no anchor) ----
    let root2 = scratch("axtx02-coll");
    let mut s2 =
        A2xSession::open(&root2, "p1033y", "owner", 8 << 20, RustTargetProvider::v1()).unwrap();
    let cold2 = s2
        .cold_assemble_profile(COLL_SRC, SourceProfile::Collections, &[], "main", false)
        .unwrap();
    assert!(
        s2.last_anchor().is_none(),
        "Rust collections keep the frozen no-anchor behavior"
    );
    let cold2_files = cold2.files.clone();
    let range = range_of_decl(COLL_SRC, "left_value");

    s2.set_target(FaultyAssembleProvider::fail_on_nth(
        RustTargetProvider::v1(),
        1,
    ));
    let err2 = s2.warm_edit_ranged("left_value", COLL_SRC, range, LEFT_B, "main", false);
    expect_fault_reject(&err2);
    assert!(s2.last_anchor().is_none());
    // the source binding never moved: a B-based follow-up is stale
    let full_after = next_source(COLL_SRC, range, LEFT_B);
    match s2.warm_edit_ranged("left_value", &full_after, range, LEFT_B, "main", false) {
        Err(A2xRejectOrResource::Reject(r2)) => assert_eq!(r2.code, "a2x.stale-base"),
        other => panic!("B base must be stale, got {other:?}"),
    }

    s2.set_target(RustTargetProvider::v1());
    // resident preservation (no-anchor channel): the failed edit left no
    // published B resident — the retry must be a REAL store hit on the
    // byproduct fragment, not a resident reuse
    let prints_failed = s2.stats().print;
    let hits_failed = s2.stats().store_hit_fragment;
    let b2 = s2
        .warm_edit_ranged("left_value", COLL_SRC, range, LEFT_B, "main", false)
        .expect("coll retry");
    assert_eq!(
        s2.stats().print,
        prints_failed,
        "the coll retry reuses the byproduct fragment (no re-print)"
    );
    assert!(
        s2.stats().store_hit_fragment > hits_failed,
        "a REAL store hit — the failed edit published no resident"
    );
    assert_ne!(b2.files, cold2_files);
    // the manifest rebinds to the NEXT source (the ranged contract)
    let next_sha = sha256_hex(full_after.as_bytes());
    assert!(
        b2.manifest
            .contains(&format!("\"source_sha256\": \"{next_sha}\"")),
        "the retry's manifest binds the replaced full source"
    );

    drop(s2);
    let mut r2 =
        A2xSession::open(&root2, "p1033y", "owner", 8 << 20, RustTargetProvider::v1()).unwrap();
    let prints2 = r2.stats().print;
    let restarted2 = r2
        .restart_assemble_profile(COLL_SRC, SourceProfile::Collections, &[], "main")
        .expect("coll restart re-admission");
    assert_eq!(r2.stats().print, prints2, "coll restart is pure hits");
    assert_eq!(restarted2.files, cold2_files);
}

/// A semantically identical copy of TEXT_A_FULL as the A-return fragment
/// (same body, the fragment contract's exact text).
const RT_TEXT_A_FRAG_FULL: &str = "fn main() int {\n    let a Str = \"ab\"\n    let b Str = a.clone()\n    var c Str = b.move\n    c = c + \"中\"\n    return a.len() * 10 + c.len()\n}";

/// Boundaries: an early parse rejection leaves the accepted state
/// untouched; OTHER TARGETS' residents survive a failed edit on this
/// target (mixed-target isolation — the failed Rust edit's binding probe
/// is stale, the C namespace serves from residents with zero production,
/// and the Rust namespace restarts to the original A bytes); read-only
/// writes nothing and publishes nothing (but is NOT session-invariance —
/// the plan says so explicitly); a same-source retry after the fault
/// clears succeeds.
#[test]
fn audit1033_axtx_02_boundaries() {
    let root = scratch("axtx02-bounds");
    let mut s = A2xSession::open(
        &root,
        "p1033z",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    let cold = s.cold_assemble(COLD, false, &[], "main", false).unwrap();
    let a_sha = sha256_hex(COLD.as_bytes());
    let cold_generated = generated_of(&cold, "generated.rs").to_string();

    // ---- early parse rejection: nothing changed ----
    let bad_frag = "fn helper(x int) int { return }";
    let range = range_of_decl(COLD, "helper");
    match s.warm_edit_ranged("helper", COLD, range, bad_frag, "main", false) {
        Err(A2xRejectOrResource::Reject(r)) => {
            assert_eq!(r.code, "a2x.parse", "early parse rejection is named")
        }
        other => panic!("malformed fragment must reject, got {other:?}"),
    }
    assert_eq!(s.last_anchor().unwrap().source_sha256, a_sha);

    // ---- mixed target: C cold, then a FAILED Rust edit on the same session ----
    s.set_target(CTargetProvider::v1());
    let cold_c = s
        .cold_assemble_profile(COLD, SourceProfile::Scalar, &[], "main", false)
        .unwrap();
    let cold_c_generated = generated_of(&cold_c, "generated.c").to_string();
    let full_after = next_source(COLD, range, HELPER_B);

    s.set_target(FaultyAssembleProvider::fail_on_nth(
        RustTargetProvider::v1(),
        1,
    ));
    let err = s.warm_edit_ranged("helper", COLD, range, HELPER_B, "main", false);
    expect_fault_reject(&err);
    // the Rust binding is still A: a B-based edit rejects as stale
    // (named rejection BEFORE the injected assemble fault)
    match s.warm_edit_ranged("helper", &full_after, range, HELPER_B, "main", false) {
        Err(A2xRejectOrResource::Reject(r)) => assert_eq!(r.code, "a2x.stale-base"),
        other => panic!("B base must be stale after the failed Rust edit, got {other:?}"),
    }

    // back on C: the C residents serve with ZERO production (a
    // whitespace-equivalent edit reuses BOTH residents)
    s.set_target(CTargetProvider::v1());
    let main_range = range_of_decl(COLD, "main");
    let ws_main = "fn main() int { return helper(20) +  1 }";
    let reuse_before = s.stats().fragments_reused_resident;
    let c_ws = s
        .warm_edit_ranged("main", COLD, main_range, ws_main, "main", false)
        .expect("C residents survived the Rust failure");
    assert_eq!(
        s.stats().fragments_reused_resident,
        reuse_before + 1,
        "the whitespace-equivalent main edit reuses its surviving C resident"
    );
    // the C file set still carries helper(x*2) — the OLD A body (the C
    // scalar channel spells it through the checked runtime helper)
    assert!(
        c_ws.files
            .iter()
            .any(|(p, c)| p == "generated.c" && c.contains("a2x_mul(l0_x, (int32_t)2)")),
        "the surviving C residents serve the A helper body"
    );

    // ---- read-only: writes NOTHING, publishes nothing. The ws edit
    // rebound the binding to the ws source, so the read-only edit rides
    // THAT base (read-only is zero-write, not session-invariance).
    let ws_full = next_source(COLD, main_range, ws_main);
    let ro_range = range_of_decl(&ws_full, "helper");
    let space_before = s.space().total;
    let anchor_before = s.last_anchor().map(|a| a.source_sha256.clone());
    let _ro = s
        .warm_edit_ranged("helper", &ws_full, ro_range, HELPER_B, "main", true)
        .expect("read-only edit still assembles");
    assert_eq!(
        s.space().total,
        space_before,
        "read-only writes zero store bytes"
    );
    assert_eq!(
        s.last_anchor().map(|a| a.source_sha256.clone()),
        anchor_before,
        "read-only publishes no anchor"
    );

    // ---- same-source retry after the fault clears (this target: C).
    // The read-only edit rebound the source binding to B, so the real
    // retry takes the read-only result's base.
    let ro_full = next_source(&ws_full, ro_range, HELPER_B);
    let b_range = range_of_decl(&ro_full, "helper");
    let b = s
        .warm_edit_ranged("helper", &ro_full, b_range, HELPER_B, "main", false)
        .expect("same-source retry succeeds on C");
    let b_generated = generated_of(&b, "generated.c");
    assert!(
        b_generated.contains("a2x_mul(l0_x, (int32_t)3)"),
        "the retry's C file set carries the B body"
    );
    assert_ne!(b_generated, cold_c_generated);

    // and the RUST namespace is untouched by everything above: a fresh
    // Rust session over the same store restarts to the ORIGINAL A bytes
    drop(s);
    let mut rr = A2xSession::open(
        &root,
        "p1033z",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    let prints = rr.stats().print;
    let restarted = rr.restart_assemble(COLD, false, &[], "main").unwrap();
    assert_eq!(rr.stats().print, prints, "Rust restart is pure hits");
    assert_eq!(generated_of(&restarted, "generated.rs"), cold_generated);
}
