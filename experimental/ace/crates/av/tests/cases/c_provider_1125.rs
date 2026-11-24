//! PLAN-1125: the in-repo 1093 C provider (`parity1093_c`) reaches the
//! ordinary AV worker through the explicit prepare lane. The positive
//! control proves the call lands in the REAL C `strlen`; the identity and
//! authority countercontrols tamper a COPY of the prepared closure while
//! the DLL itself stays callable, so a named rejection is the gate that
//! stood between the source and the provider — never a failed call.

use ace_av::language_source_worker::serve;
use base64::Engine as _;
use serde_json::{json, Value as Json};
use std::path::{Path, PathBuf};

const CAPS: [&str; 5] = [
    "ac.source.language.av/v1",
    "foreign.scalar",
    "foreign.resource",
    "console.stdout",
    "console.stderr",
];

/// The prepare-av-providers bin writes the closures into its own exe
/// directory (the shared ACE_AV_PROVIDER_ROOT the harness forwards).
fn prepared_root() -> PathBuf {
    if let Ok(root) = std::env::var("ACE_AV_PROVIDER_ROOT") {
        return PathBuf::from(root);
    }
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo manifest dir");
    PathBuf::from(manifest).join("../../target/debug")
}

fn closure() -> PathBuf {
    let descriptor = prepared_root().join("parity1093_c.descriptor.json");
    assert!(
        descriptor.is_file(),
        "foreign.prepared: parity1093_c closure missing at {} (run the av-providers prepare lane)",
        descriptor.display()
    );
    prepared_root()
}

fn request(source: &str, caps: &[&str], root: &Path) -> Json {
    json!({
        "recipe": "ac-av/language-source-run-v1",
        "nonce": "0".repeat(64),
        "source_b64": base64::engine::general_purpose::STANDARD.encode(source),
        "steps": [{"op": "run"}],
        "source_path": "main.at",
        "entry": "d_main",
        "caps": caps,
        "foreign": {"kind": "c", "provider": "parity1093_c", "artifact_root": root},
    })
}

/// One worker run: the step as returned (a rejection is `outcome: diag`
/// with its `diag` carrying the named stage/code).
fn run(source: &str, caps: &[&str], root: &Path) -> Json {
    serve(&request(source, caps, root)).expect("ordinary worker response")["steps"][0].clone()
}

/// A scratch copy of the prepared closure: `edit` mutates the descriptor
/// in place, the DLL is copied byte-for-byte.
fn tampered(edit: impl FnOnce(&mut Json)) -> (tempfile::TempDirGuard, PathBuf) {
    let root = closure();
    let guard = tempfile::TempDirGuard::new();
    let copy = guard.path().to_path_buf();
    std::fs::copy(root.join("parity1093_c.dll"), copy.join("parity1093_c.dll")).unwrap();
    let mut descriptor: Json =
        serde_json::from_slice(&std::fs::read(root.join("parity1093_c.descriptor.json")).unwrap())
            .unwrap();
    edit(&mut descriptor);
    std::fs::write(
        copy.join("parity1093_c.descriptor.json"),
        serde_json::to_vec(&descriptor).unwrap(),
    )
    .unwrap();
    (guard, copy)
}

mod tempfile {
    //! A drop-guarded scratch directory (the closure copies never touch
    //! the prepared root).
    pub struct TempDirGuard(std::path::PathBuf);

    impl TempDirGuard {
        pub fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let unique = format!(
                "ace-av-c1125-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            );
            let path = std::env::temp_dir().join(unique);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        pub fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

fn positive(step: &Json) {
    assert_eq!(step["outcome"], "success", "{step}");
    assert_eq!(step["console"]["stdout_b64"], "", "{step}");
    assert_eq!(step["console"]["stderr_b64"], "", "{step}");
}

/// A named rejection: the diag carries the code, no run value was
/// produced and no call reached a provider or the host.
fn rejected<'a>(step: &'a Json, code: &str) -> &'a Json {
    assert_eq!(step["outcome"], "diag", "{code}: {step}");
    let error = &step["diag"];
    assert_eq!(error["code"], code, "{error}");
    assert!(step["value"].is_null(), "{step}");
    assert_eq!(step["counters"]["runtime_calls"], 0, "{step}");
    assert_eq!(step["counters"]["host_dispatches"], 0, "{step}");
    error
}

#[test]
fn plan1125_c_provider_calls_the_real_c_strlen() {
    for (source, value) in [
        ("fn main() int { return c_strlen(\"hello\") }", 5),
        ("fn main() int { return c_strlen(\"\") }", 0),
        // byte length would be 11 — only the C strlen stops at the NUL
        ("fn main() int { return c_strlen(\"hello\\0world\") }", 5),
        // UTF-8 bytes, not code points (6): é is 2 bytes, 😀 is 4
        ("fn main() int { return c_strlen(\"héllo😀\") }", 10),
    ] {
        let step = run(&format!("use.c parity1093_c\n{source}"), &CAPS, &closure());
        positive(&step);
        assert_eq!(step["value"], value, "{source}: {step}");
    }
}

#[test]
fn plan1125_c_header_spelling_reaches_the_source_rejection() {
    let step = run("use.c <string.h>\nfn main() int { return c_strlen(\"hello\") }", &CAPS, &closure());
    let error = rejected(&step, "source.use-rs.provider");
    assert!(error["message"].as_str().unwrap().contains("string.h"), "{error}");
}

#[test]
fn plan1125_c_closure_is_bound_to_the_in_repo_sources() {
    let descriptor: Json = serde_json::from_slice(
        &std::fs::read(closure().join("parity1093_c.descriptor.json")).unwrap(),
    )
    .unwrap();
    let project = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../foreign-contract/c/parity1093_c");
    let recorded = descriptor["source_sha256"].as_object().expect("source_sha256 recorded");
    let mut header = Vec::new();
    for file in ["parity1093_c.h", "parity1093_c.c", "parity1093_c_impl.c"] {
        let bytes = std::fs::read(project.join(file)).unwrap();
        if file.ends_with(".h") {
            header = bytes.clone();
        }
        assert_eq!(
            recorded.get(file).and_then(Json::as_str),
            Some(ace_core::runtime_contract::sha256_hex(&bytes).as_str()),
            "{file}: the closure was not prepared from the in-repo source"
        );
    }
    // the admitted header digest is the in-repo header's own
    assert_eq!(
        descriptor["provider_digest"],
        json!(ace_foreign_contract::blake3_hex(&header))
    );
}

#[test]
fn plan1125_c_wrong_artifact_identity_is_rejected() {
    let (_guard, root) = tampered(|descriptor| {
        let mut declared = descriptor["artifact_sha256"].as_str().unwrap().to_owned();
        declared.replace_range(0..1, if declared.starts_with('0') { "1" } else { "0" });
        descriptor["artifact_sha256"] = json!(declared);
    });
    rejected(&run("use.c parity1093_c\nfn main() int { return c_strlen(\"hello\") }", &CAPS, &root), "foreign.artifact");
}

#[test]
fn plan1125_c_foreign_header_source_is_rejected() {
    // the DLL and digest stay as prepared; only the vouched header differs
    let (_guard, root) = tampered(|descriptor| {
        let header = base64::engine::general_purpose::STANDARD
            .decode(descriptor["source_b64"].as_str().unwrap())
            .unwrap();
        let mut other = header;
        other.extend_from_slice(b"\n// not the admitted header\n");
        descriptor["source_b64"] = json!(base64::engine::general_purpose::STANDARD.encode(&other));
    });
    rejected(&run("use.c parity1093_c\nfn main() int { return c_strlen(\"hello\") }", &CAPS, &root), "foreign.identity");
}

#[test]
fn plan1125_c_foreign_manifest_provider_is_rejected() {
    let (_guard, root) = tampered(|descriptor| {
        let mut manifest: Json = serde_json::from_slice(
            &base64::engine::general_purpose::STANDARD
                .decode(descriptor["manifest_b64"].as_str().unwrap())
                .unwrap(),
        )
        .unwrap();
        manifest["provider"] = json!("parity1093_other");
        descriptor["manifest_b64"] = json!(
            base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(&manifest).unwrap())
        );
    });
    rejected(&run("use.c parity1093_c\nfn main() int { return c_strlen(\"hello\") }", &CAPS, &root), "foreign.identity");
}

#[test]
fn plan1125_c_missing_closure_is_a_preparation_failure() {
    let guard = tempfile::TempDirGuard::new();
    rejected(
        &run(
            "use.c parity1093_c\nfn main() int { return c_strlen(\"hello\") }",
            &CAPS,
            guard.path(),
        ),
        "foreign.prepared",
    );
}

#[test]
fn plan1125_c_revoked_capability_rejects_before_the_call() {
    for revoked in ["foreign.scalar", "foreign.resource"] {
        let caps: Vec<&str> = CAPS.iter().copied().filter(|cap| *cap != revoked).collect();
        let step = run("use.c parity1093_c\nfn main() int { return c_strlen(\"hello\") }", &caps, &closure());
        rejected(&step, "source.foreign-capability");
    }
}
