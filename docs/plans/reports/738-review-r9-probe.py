"""Reproduce unbound lock read failure through the real freshness gate.

Reuse the R5 harness's clean-tree check and exact-byte restoration. Only a
temporary cfg(test) probe and isolated project/workspace inputs are changed.
"""
import importlib.util
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "review738_probe", Path(__file__).with_name("738-review-r5-probe.py")
)
harness = importlib.util.module_from_spec(spec)
spec.loader.exec_module(harness)
harness.LOG = harness.ROOT / ".review738-r9-lock-read-error.log"
harness.PROBE = r'''
    #[test]
    fn review738_r5_lock_drift_after_first_materialization() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("project");
        let workspace = dir.path().join("workspace");
        std::fs::create_dir_all(project.join("src/back")).unwrap();
        std::fs::write(project.join("pac.at"),
            "{ name: \"review738-r9-lock\", scene: \"workspace\" }").unwrap();
        std::fs::write(project.join("src/back/api.at"),
            "#[api(method = \"GET\", path = \"/api/answer\")]\npub fn answer() int { return 42 }\n").unwrap();
        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(v) => std::env::set_var("AUTO_RUST_WORKSPACE", v),
                    None => std::env::remove_var("AUTO_RUST_WORKSPACE"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("AUTO_RUST_WORKSPACE"));
        std::env::set_var("AUTO_RUST_WORKSPACE", &workspace);
        crate::api_gen::generate_api(&project, "rust").unwrap();
        let ws = ensure_shared_workspace(&project);
        let receipt_path = ws.join(back_member_name(&project)).join("generation.json");
        let receipt = std::fs::read_to_string(&receipt_path).unwrap();
        let record: serde_json::Value = serde_json::from_str(&receipt).unwrap();
        assert_eq!(record["workspace_lock"], "absent");
        assert!(backend_generation_is_fresh(&project), "genuine absence may be fresh");
        let lock_path = ws.join("Cargo.lock");
        std::fs::create_dir(&lock_path).unwrap();
        let error = std::fs::read(&lock_path).unwrap_err();
        assert_ne!(error.kind(), std::io::ErrorKind::NotFound);
        let is_fresh = backend_generation_is_fresh(&project);
        assert_eq!(std::fs::read_to_string(&receipt_path).unwrap(), receipt);
        eprintln!("R5_PROBE R9 recorded=absent lock_read_error={:?} is_fresh={}",
            error.kind(), is_fresh);
        assert!(!is_fresh, "unreadable lock must not be treated as genuine absence");
    }
'''

if __name__ == "__main__":
    harness.main()
