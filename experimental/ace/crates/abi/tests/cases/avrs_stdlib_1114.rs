//! PLAN-1114 (IF-ACE-AV-RS-STDLIB/1) — V-01: the AV/Rust stdlib core
//! batch. The REAL `auto.str` public byte face and the math integer face
//! execute through the assembled `.rs.at` provider (one composed `avrs/1`
//! unit built from every selected Rs layer) with REAL foreign dispatch on
//! the ordinary AV chain: `A中😀\0` byte length 9 (UTF-8 bytes, NUL is
//! content). The app source never changes while the SELECTED Rust
//! implementation edit flips the provider identity AND the observable
//! value; an UNSELECTED `.py.at` sibling leaves the identity untouched.
//! The public `.at` contract faces carry the same signatures the app
//! calls (the consistency anchor). Named rejections: a py-only candidate
//! set (`assembly.provider-backend`) and a closure with no rs layer at
//! all (`load.no-provider-impl`).
#![cfg(windows)]
use std::path::{Path, PathBuf};

use ace_abi::rust_provider::open_assembled_rs_stdlib_session;
use ace_av_core::contract::{ExecOutcome, Value};
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, ProviderKind, TargetOs,
};

/// The repository's delivered AV/RS stdlib tree (the canonical sources —
/// the test never re-spells them; a source edit IS the mutation probe).
fn avrs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../stdlib/ace/avrs")
}

/// T-05 F3: transient scratch under the OS temp dir, removed on drop.
struct Scope(PathBuf);
impl std::ops::Deref for Scope {
    type Target = std::path::Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn scope(tag: &str) -> Scope {
    let root = std::env::temp_dir().join(format!("p1114-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    Scope(root)
}

/// Stage the delivered stdlib files into one flat assembly root (the
/// 1043 discovery contract: one flat directory; the app source is the
/// single-module entry of the ordinary AV chain). Every delivered module
/// stages — the composed provider is a function of the whole tree.
fn stage(root: &Path, extra: &[(&str, &str)]) {
    let mut names: Vec<String> = std::fs::read_dir(avrs_dir())
        .expect("the avrs stdlib tree exists")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert!(!names.is_empty(), "the avrs tree carries modules");
    for name in &names {
        let bytes = std::fs::read(avrs_dir().join(name))
            .unwrap_or_else(|e| panic!("stage {name}: {e}"));
        std::fs::write(root.join(name), bytes).unwrap();
    }
    // The entry module's public source (the flat-dir assembly requires
    // it): the ordinary single-module app importing the composed provider.
    std::fs::write(root.join("main.at"), APP).unwrap();
    for (name, content) in extra {
        std::fs::write(root.join(name), content).unwrap();
    }
}

/// The app: the ordinary single-module source importing the composed
/// provider and calling the module's PUBLIC names.
const APP: &str = r#"use.rs avrs
fn main() int {
    let text = "A中😀\0"
    var acc = str_byte_len(text)
    acc = acc * 100 + char_at(text, 1)
    acc = acc * 10 + abs(0 - 7)
    return acc
}"#;

fn rs_ctx() -> PackageAssemblyContext {
    PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Kind(ProviderKind::Rs),
        TargetOs::Windows,
    )
}

/// Compile the app on the ordinary AV chain against the session's
/// admitted provider and execute it through the foreign session.
fn run_avrs_app(source: &str, session: &mut ace_abi::rust_provider::F0Session) -> ExecOutcome {
    // The delivered ordinary-language foreign cap set (PLAN-1093 corpus
    // spelling): the language lane admission (which keys the foreign
    // module domain — without it mir_verify names the resource capability
    // unknown) + the foreign grants (the manifest check requires every
    // provider capability the composed unit declares).
    let caps = [
        "ac.source.language.av/v1".to_string(),
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let program = ace_av::foreign_source::compile_foreign_program(
        source, "main.at", "d_main", &caps, &session.admitted,
    )
    .expect("the app compiles against the admitted avrs provider");
    let decoded = ace_av_core::abc::decode_abc(&program.abc).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(
        decoded,
        &program.owner,
        session.session.table_shared(),
    )
    .expect("the program admits against the session");
    session
        .session
        .execute(
            &loaded,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
        )
        .expect("authorized run")
        .0
}

fn expect_i32(out: ExecOutcome, want: i32, what: &str) {
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(v, want, "{what}"),
        other => panic!("{what}: expected Success(I32({want})), got {other:?}"),
    }
}

/// V-01: the core batch really runs. byte_len("A中😀\0") = 9 by UTF-8
/// bytes; char_at(text, 1) = '中' = 20013; abs rides the same provider on
/// the calling-thread i32 lane. acc = ((9*100 + 20013)*10 + 7) = 209137.
#[test]
fn v01_avrs_core_values_via_rust_provider() {
    let root = scope("v01-core");
    stage(&root, &[]);
    let (mut session, inputs) =
        open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 1)
            .expect("the avrs stdlib session opens");
    // The composed closure carries the two provider-backed modules and
    // the app entry (the flat-dir assembly shape).
    assert!(inputs.iter().any(|i| i.id == "str" && i.provider.is_some()));
    assert!(inputs.iter().any(|i| i.id == "math" && i.provider.is_some()));
    expect_i32(run_avrs_app(APP, &mut session), 209137, "core values");
    assert!(
        session.session.table_shared().callee_count() >= 3,
        "every provider op really dispatched"
    );
    let digest_before = session.build.provider_digest.clone();

    // ---- SELECTED-input edit: the str fragment body moves -> a new
    // provider identity through the SAME entry, and the observable value
    // moves with it (byte_len 9 -> 10: acc +100). The app never changes.
    drop(session);
    let original = std::fs::read_to_string(root.join("str.rs.at")).unwrap();
    let edited = original.replace(
        "    s.as_bytes().len() as i32",
        "    s.as_bytes().len() as i32 + 1",
    );
    assert_ne!(&edited, &original);
    std::fs::write(root.join("str.rs.at"), &edited).unwrap();
    let (mut session2, _) =
        open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 2)
            .expect("the edited provider rebuilds");
    assert_ne!(
        session2.build.provider_digest, digest_before,
        "a selected-input edit flips the provider identity"
    );
    expect_i32(run_avrs_app(APP, &mut session2), 210137, "edited value");

    // ---- UNSELECTED sibling: a py candidate joins the directory; the AV
    // configuration still selects the Rs layers, the composed provider
    // identity (and the value) is untouched.
    std::fs::write(root.join("str.py.at"), "# python adapter (unselected on av)\n").unwrap();
    let (mut session3, _) =
        open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 3)
            .expect("the unselected sibling changes nothing");
    assert_eq!(
        session3.build.provider_digest, session2.build.provider_digest,
        "an unselected py body leaves the provider identity untouched"
    );
    expect_i32(run_avrs_app(APP, &mut session3), 210137, "stable value");
}

/// V-01 contract anchor: every provider export the app can call has a
/// same-signature public declaration in the module's `.at` contract face
/// (the single public API record).
#[test]
fn v01_avrs_public_contract_faces_match_provider_exports() {
    let root = scope("v01-contract");
    stage(&root, &[]);
    let (session, _) = open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 1)
        .expect("the avrs stdlib session opens");
    let mut contracts = String::new();
    for entry in std::fs::read_dir(avrs_dir()).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if name.ends_with(".at") && !name.contains(".rs.") && !name.contains(".py.") {
            contracts.push_str(&std::fs::read_to_string(avrs_dir().join(&name)).unwrap());
        }
    }
    for declaration in session.admitted.source_declarations_ref() {
        let name = &declaration.name;
        assert!(
            contracts.contains(&format!("pub spec fn {name}(")),
            "the provider export `{name}` has no matching public contract declaration"
        );
    }
}

/// V-01 negatives: only py candidates for the provider-backed modules is
/// the assembly's own NAMED backend rejection (never a silent fallback);
/// removing every rs layer is `load.no-provider-impl`.
#[test]
fn v01_avrs_named_rejections() {
    let root = scope("v01-neg");
    let remove_rs_layers = |dir: &Path| {
        for entry in std::fs::read_dir(dir).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            if name.ends_with(".rs.at") {
                std::fs::remove_file(dir.join(&name)).unwrap();
            }
        }
    };

    // py-only on the AV backend: the assembly's named selection error.
    let proj = root.join("py-only");
    std::fs::create_dir_all(&proj).unwrap();
    stage(&proj, &[]);
    remove_rs_layers(&proj);
    std::fs::write(proj.join("str.py.at"), "# python adapter\n").unwrap();
    let err = match open_assembled_rs_stdlib_session(&proj, "app::main", &mut rs_ctx(), 1) {
        Err(e) => e,
        Ok(_) => panic!("a py-only candidate set must reject"),
    };
    assert!(
        err.contains("assembly.provider-backend") || err.contains("load.assembly"),
        "named: {err}"
    );

    // no rs layer at all: pure-public closure is legal assembly, but the
    // AV/RS stdlib lane names the missing provider.
    let proj = root.join("no-rs");
    std::fs::create_dir_all(&proj).unwrap();
    stage(&proj, &[]);
    remove_rs_layers(&proj);
    let err = match open_assembled_rs_stdlib_session(&proj, "app::main", &mut rs_ctx(), 1) {
        Err(e) => e,
        Ok(_) => panic!("a closure without rs layers must reject"),
    };
    assert!(err.contains("load.no-provider-impl"), "named: {err}");
}

// ---------------------------------------------------------------------------
// V-02 (T-03): real file / console / loopback I/O with exact bytes, EOF,
// named errors and close; owned-clock handle safety across generations.
// ---------------------------------------------------------------------------

/// V-02 file leg: exact bytes (UTF-8 + NUL content), EOF at len, named
/// -8 for the missing path, remove and the post-remove existence flip —
/// every observation a REAL Rust dispatch through the composed provider.
#[test]
fn v02_avrs_file_exact_bytes_eof_and_errors() {
    let root = scope("v02-file");
    stage(&root, &[]);
    let (mut session, _) =
        open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 1)
            .expect("the avrs stdlib session opens");
    let path = root.join("avrs-v02.txt").to_string_lossy().replace('\\', "/");
    // "A中😀\0" by exact UTF-8 bytes: 41 E4 B8 AD F0 9F 98 80 00.
    let app = format!(
        r#"use.rs avrs
fn main() int {{
    let path = "{path}"
    var acc = file_write_byte(path, 65)
    acc = acc + file_append_byte(path, 228)
    acc = acc + file_append_byte(path, 184)
    acc = acc + file_append_byte(path, 173)
    acc = acc + file_append_byte(path, 240)
    acc = acc + file_append_byte(path, 159)
    acc = acc + file_append_byte(path, 152)
    acc = acc + file_append_byte(path, 128)
    acc = acc + file_append_byte(path, 0)
    var bytes = file_read_len(path)
    var ok = 0
    if file_exists(path) {{ ok = 1 }}
    var eof = file_read_byte(path, 9)
    var b0 = file_read_byte(path, 0)
    var b8 = file_read_byte(path, 8)
    var miss = file_read_len("{path}.missing")
    var rm = file_remove(path)
    var gone = 0
    if file_exists(path) {{ gone = 1 }}
    return acc * 1000000 + bytes * 10000 + ok * 1000 + (eof + 1) * 100 + b0 * 10 + b8 + miss + rm + gone
}}"#
    );
    // acc=0 (all writes ok) | bytes=9 | ok=1 | (eof+1)=0 | b0=65 | b8=0
    // (NUL is content) | miss=-8 | rm=0 | gone=0
    // => 0 + 90000 + 1000 + 0 + 650 + 0 - 8 + 0 + 0 = 91642.
    expect_i32(run_avrs_app(&app, &mut session), 91642, "file face");
    assert!(
        session.session.table_shared().callee_count() >= 17,
        "every file op really dispatched"
    );
    assert!(!root.join("avrs-v02.txt").exists(), "remove really removed");
}

/// V-02 console leg: the REAL stdout write returns the exact UTF-8 byte
/// count — the count oracle is distinguishing (`A中😀\0` = 9 by bytes; a
/// C-strlen truncation at the first NUL would answer 1, an ASCII-only
/// count 4; the byte identity is independently pinned by V-01's
/// str_byte_len oracle over the SAME literal per operations.toml).
#[test]
fn v02_avrs_console_exact_bytes() {
    let root = scope("v02-console");
    stage(&root, &[]);
    let (mut session, _) =
        open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 1)
            .expect("the avrs stdlib session opens");
    expect_i32(
        run_avrs_app(
            "use.rs avrs\nfn main() int { return console_write(\"A中😀\0\") }",
            &mut session,
        ),
        9,
        "the console write count is the exact UTF-8 byte len",
    );
    expect_i32(
        run_avrs_app(
            "use.rs avrs\nfn main() int { return console_write(\"abc\") }",
            &mut session,
        ),
        3,
        "the plain-ASCII control",
    );
}

/// V-02 env + time + loopback legs: the unset sentinel (-203) vs the
/// existing value, the calling-thread sleep, the owned clock lifecycle
/// (borrow lo/hi, close) and the REAL loopback TCP echo with exact bytes.
#[test]
fn v02_avrs_env_time_and_loopback() {
    let root = scope("v02-sys");
    stage(&root, &[]);
    std::env::set_var("AVRS_V02_PROBE", "abc");
    let (mut session, _) =
        open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 1)
            .expect("the avrs stdlib session opens");
    let app = r#"use.rs avrs
fn main() int {
    var acc = 0
    if env_has("AVRS_V02_PROBE") { acc = acc + 1 }
    acc = acc + env_get_len("AVRS_V02_PROBE")
    var unset = env_get_len("AVRS_V02_PROBE_UNSET")
    var clock = clock_open()
    var lo = clock_ms_lo(clock)
    var hi = clock_ms_hi(clock)
    acc = acc + clock_close(clock)
    if lo > 0 { if hi > 0 { acc = acc + 10 } }
    acc = acc + net_loopback_echo("A中😀\0") * 100
    var mismatch = net_loopback_echo("x") * 100
    return acc * 10 + unset + 203 + mismatch - 100
}"#;
    // env: has(+1) + len(3) = 4; close 0; clock plausible +10;
    // loopback exact "A中😀\0" = 9 bytes -> +900; mismatch leg "x" = 1
    // byte echo (exact) -> +100 on the mismatch slot; unset -203 +203.
    // acc = 4 + 10 + 900 = 914; return 914*10 + 0 + (100 - 100) = 9140.
    expect_i32(run_avrs_app(app, &mut session), 9140, "env/time/loopback face");
    std::env::remove_var("AVRS_V02_PROBE");
}

/// V-02 handle safety: the owned clock handle is provider-private and
/// generation-bound — a handle from a STALE generation is rejected before
/// execution in the new session while the OLD session still answers its
/// own legal handle.
#[test]
fn v02_avrs_owned_handle_generation_and_close_safety() {
    let root = scope("v02-handles");
    stage(&root, &[]);
    let (mut s1, _) = open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 1)
        .expect("generation 1 opens");
    // The handle escapes as the app's value (an opaque i32 to the guest).
    let handle = match run_avrs_app(
        "use.rs avrs\nfn main() int { return clock_open() }",
        &mut s1,
    ) {
        ExecOutcome::Success {
            value: Value::I32(h),
            ..
        } => h,
        other => panic!("clock_open: {other:?}"),
    };
    // A NEW generation opens (same sources, generation 2): the stale
    // handle from generation 1 is rejected BEFORE execution in it.
    let (mut s2, _) = open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 2)
        .expect("generation 2 opens");
    let stale = format!("use.rs avrs\nfn main() int {{ return clock_ms_lo({handle}) }}");
    let rejected = run_avrs_app(&stale, &mut s2);
    match &rejected {
        ExecOutcome::Infra(message)
            if message.contains("foreign.handle") => {}
        other => panic!(
            "a stale-generation handle is rejected before the callee runs: {other:?}"
        ),
    }

    // The OLD session STILL answers its own legal handle afterwards.
    let legal = format!("use.rs avrs\nfn main() int {{ return clock_ms_lo({handle}) }}");
    let still = run_avrs_app(&legal, &mut s1);
    assert!(
        matches!(still, ExecOutcome::Success { .. }),
        "the old session still answers its legal handle: {still:?}"
    );
}

/// V-02 close safety: after the app closes the clock, the same session
/// rejects the closed handle with the owned model's named failure (the
/// callee never runs); the rejection is a structured failure, not a
/// value.
#[test]
fn v02_avrs_closed_handle_rejected() {
    let root = scope("v02-closed");
    stage(&root, &[]);
    let (mut session, _) =
        open_assembled_rs_stdlib_session(&root, "app::main", &mut rs_ctx(), 1)
            .expect("session opens");
    let out = run_avrs_app(
        "use.rs avrs\nfn main() int {\n    var c = clock_open()\n    var dead = clock_close(c)\n    return clock_ms_lo(c)\n}",
        &mut session,
    );
    match &out {
        ExecOutcome::Infra(message) if message.contains("foreign.handle") => {}
        other => panic!("a closed handle is rejected before the callee runs: {other:?}"),
    }
}

