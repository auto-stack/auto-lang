//! PLAN-975 T-02 coverage increment (AC-COVERAGE follow-up) — the B1
//! `c.to(Str)` primitive on the A2PY end: the same public source shape
//! as the AV/A2R/AC/native legs (`json975_char_to_str_a2x_ends` in the
//! av crate), lowered through `ace-a2x-python` onto REAL OS Python.
//! Hand oracle: materialized 'A'(1B) + U+4E2D(3B) + U+1F600(4B) give
//! lens 134; the concatenation equals "A中😀" (+7) -> 134*10+7 = 1347.
use std::path::PathBuf;

use ace_a2x_python::{
    python_runtime_source, verify_python_fileset_manifest, PythonTargetProvider, PY_PRODUCER,
    PY_RT_FILESET_SCHEMA,
};
use ace_a2x_store::A2xSession;
use ace_a2x_worker::pyrun::{resolve_python, run_py_fileset};
use ace_core::runtime_contract::sha256_hex;
use ace_core::source::SourceProfile;

fn out_root(sub: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2x-out-p975c2s")
        .join(format!("{sub}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

const SRC: &str = r#"fn mk(cp int) Str {
    return char.from_codepoint(cp).to(Str)
}
fn main() int {
    let a Str = mk(65)
    let zh Str = mk(20013)
    let em Str = mk(128512)
    let lens int = a.len_bytes() * 100 + zh.len_bytes() * 10 + em.len_bytes()
    let all Str = a + zh + em
    let lit Str = "A中😀"
    if all != lit { return 9000 + lens }
    if all.len() != 3 { return 8000 + lens }
    return lens * 10 + 7
}"#;

#[test]
fn char975_to_str_on_python_end() {
    let rt = python_runtime_source(&[]);
    let rt_hash = sha256_hex(rt.as_bytes());
    let py = resolve_python(&rt_hash)
        .expect("OS Python interpreter required (missing Python cannot PASS)");

    let store = out_root("store");
    let mut session = A2xSession::open(
        &store,
        "p975c2s",
        "probe",
        64 * 1024 * 1024,
        PythonTargetProvider,
    )
    .expect("open a2x session");
    let asm = session
        .cold_assemble_profile(SRC, SourceProfile::TextRecord, &[], "main", false)
        .expect("assembles the to(Str) source");

    let run_dir = out_root("run");
    for (rel, content) in &asm.files {
        std::fs::write(run_dir.join(rel), content).unwrap();
    }
    std::fs::write(run_dir.join("manifest.json"), &asm.manifest).unwrap();
    verify_python_fileset_manifest(&asm.manifest, &asm.files, PY_PRODUCER, PY_RT_FILESET_SCHEMA)
        .expect("manifest verifies");
    let run = run_py_fileset(&py, &run_dir, &[]).expect("the real Python run");
    assert_eq!(
        run.stdout.as_slice(),
        b"value:1347\n".as_slice(),
        "A2PY end stdout (exit {:?}, stderr {})",
        run.exit_code,
        String::from_utf8_lossy(&run.stderr)
    );
    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&run_dir);
}
