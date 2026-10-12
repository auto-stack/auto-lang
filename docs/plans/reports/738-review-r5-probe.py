"""Read-only review probes: temporarily add a test, run it, restore exact bytes.

The injected test exercises the official generator and actual freshness gate.
It changes only isolated temporary project/workspace inputs, not implementation.
"""
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path("D:/autostack/.wt/lang-738/auto-lang")
SOURCE = ROOT / "crates/auto-man/src/rust_ui.rs"
LOG = ROOT / ".review738-r5-lock-probe.log"

PROBE = r'''
    #[test]
    fn review738_r5_lock_drift_after_first_materialization() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("project");
        let isolated_workspace = dir.path().join("workspace");
        std::fs::create_dir_all(project.join("src/back")).unwrap();
        std::fs::write(project.join("pac.at"),
            "{ name: \"review738-r5-lock\", scene: \"workspace\" }").unwrap();
        std::fs::write(project.join("src/back/api.at"),
            "#[api(method = \"GET\", path = \"/api/answer\")]\npub fn answer() int { return 42 }\n").unwrap();
        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(value) => std::env::set_var("AUTO_RUST_WORKSPACE", value),
                    None => std::env::remove_var("AUTO_RUST_WORKSPACE"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("AUTO_RUST_WORKSPACE"));
        std::env::set_var("AUTO_RUST_WORKSPACE", &isolated_workspace);
        crate::api_gen::generate_api(&project, "rust").unwrap();
        let ws = ensure_shared_workspace(&project);
        let receipt_path = ws.join(back_member_name(&project)).join("generation.json");
        let receipt: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&receipt_path).unwrap()).unwrap();
        assert_eq!(receipt["workspace_lock"], "absent");
        assert!(backend_generation_is_fresh(&project));
        let lock_path = ws.join("Cargo.lock");
        std::fs::write(&lock_path,
            "version = 4\n\n[[package]]\nname = \"review-runtime\"\nversion = \"0.1.0\"\n").unwrap();
        assert!(backend_generation_is_fresh(&project), "initial materialization");
        let receipt_after = std::fs::read_to_string(&receipt_path).unwrap();
        assert_eq!(receipt_after, std::fs::read_to_string(&receipt_path).unwrap());
        std::fs::write(&lock_path,
            "version = 4\n\n[[package]]\nname = \"review-runtime\"\nversion = \"0.2.0\"\n").unwrap();
        let drift_is_fresh = backend_generation_is_fresh(&project);
        eprintln!("R5_PROBE recorded_lock={} first_materialization_fresh=true later_lock_drift_fresh={}",
            receipt["workspace_lock"], drift_is_fresh);
        assert!(!drift_is_fresh, "later lock dependency drift must reject old receipt");
    }
'''


def main():
    dirty = subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=ROOT, text=True
    )
    if dirty.strip():
        raise RuntimeError("Probe requires clean implementation worktree")
    original = SOURCE.read_bytes()
    text = original.decode("utf-8")
    match = re.search(r"mod tests \{\r?\n    use super::\*;\r?\n", text)
    if not match:
        raise RuntimeError("Probe insertion point unavailable")
    newline = "\r\n" if "\r\n" in text else "\n"
    patched = text[:match.end()] + PROBE.replace("\n", newline) + text[match.end():]
    command = ["cargo", "test", "-p", "auto-man", "--lib",
               "review738_r5_lock_drift_after_first_materialization", "--",
               "--test-threads=1", "--nocapture"]
    result = None
    try:
        SOURCE.write_bytes(patched.encode("utf-8"))
        with LOG.open("w", encoding="utf-8") as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log,
                                    stderr=subprocess.STDOUT, timeout=600)
    finally:
        SOURCE.write_bytes(original)
        if SOURCE.read_bytes() != original:
            raise RuntimeError("Exact implementation restoration failed")
    output = LOG.read_text(encoding="utf-8")
    highlights = [line for line in output.splitlines()
                  if any(token in line for token in ["R5_PROBE", "test result:",
                                                     "later lock", "panicked at"])]
    print(json.dumps({"command": command, "exit_code": result.returncode,
                      "restored_exact_bytes": True, "highlights": highlights},
                     ensure_ascii=False))


if __name__ == "__main__":
    main()
