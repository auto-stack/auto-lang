"""Run the formal service witness with the existing shared build cache.

Only the cfg(test) fixture's hardcoded target-directory selection is adapted
to CARGO_TARGET_DIR; all production generation, build, ready, business and
regeneration assertions are unchanged. Restore exact bytes afterwards.
"""
import json
import pathlib
import subprocess

ROOT = pathlib.Path("D:/autostack/.wt/review-751-r2/auto-lang")
SOURCE = ROOT / "crates/auto-man/src/api_gen.rs"
LOG = ROOT / ".review751-r2-service-adapted.log"

def main():
    if subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True).strip():
        raise RuntimeError("Requires clean verification worktree")
    original = SOURCE.read_bytes()
    text = original.decode("utf-8").replace("\r\n", "\n")
    before = '''        let target = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .canonicalize()
            .unwrap();'''
    after = '''        let target = std::path::PathBuf::from(
            std::env::var_os("CARGO_TARGET_DIR").expect("review build cache"))
            .canonicalize().unwrap();'''
    if text.count(before) != 1:
        raise RuntimeError("Unique cfg(test) fixture target unavailable")
    patched = text.replace(before, after, 1)
    if b"\r\n" in original:
        patched = patched.replace("\n", "\r\n")
    command = ["cargo", "test", "-p", "auto-man", "--lib", "--features",
               "test-http-e2e", "http_e2e_plan738", "--", "--ignored",
               "--test-threads=1"]
    try:
        SOURCE.write_bytes(patched.encode("utf-8"))
        with LOG.open("w", encoding="utf-8") as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log,
                                    stderr=subprocess.STDOUT, timeout=600)
    finally:
        SOURCE.write_bytes(original)
        assert SOURCE.read_bytes() == original
    highlights = [s for s in LOG.read_text(encoding="utf-8").splitlines()
                  if "test result:" in s or "panicked at" in s]
    print(json.dumps({"cargo_exit": result.returncode,
        "fixture_change":"cfg(test) target path only; CARGO_TARGET_DIR",
        "restored_exact_bytes":True,"highlights":highlights},ensure_ascii=False))

if __name__ == "__main__":
    main()
