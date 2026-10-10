"""Post-landing probe: real generator/receipt/freshness, exact-byte restore."""
import importlib.util
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "review738_probe", Path(__file__).with_name("738-review-r5-probe.py")
)
harness = importlib.util.module_from_spec(spec)
spec.loader.exec_module(harness)
harness.ROOT = Path("D:/autostack/.wt/review-738-r11/auto-lang")
harness.SOURCE = harness.ROOT / "crates/auto-man/src/rust_ui.rs"
harness.LOG = harness.ROOT / ".review738-r11-lock.log"
harness.PROBE = r'''
    #[test]
    fn review738_r5_lock_drift_after_first_materialization() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("project");
        let workspace = dir.path().join("workspace");
        std::fs::create_dir_all(project.join("src/back")).unwrap();
        std::fs::write(project.join("pac.at"),
            "{ name: \"review738-r11-lock\", scene: \"workspace\" }").unwrap();
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
        let lock_path = ws.join("Cargo.lock");
        assert!(backend_generation_is_fresh(&project));
        std::fs::create_dir(&lock_path).unwrap();
        assert_ne!(std::fs::read(&lock_path).unwrap_err().kind(), std::io::ErrorKind::NotFound);
        assert!(!backend_generation_is_fresh(&project));
        std::fs::remove_dir(&lock_path).unwrap();
        assert!(backend_generation_is_fresh(&project));
        assert_eq!(std::fs::read_to_string(&receipt_path).unwrap(), receipt);
        std::fs::create_dir(&lock_path).unwrap();
        let error = crate::api_gen::generate_api(&project, "rust").unwrap_err();
        assert!(error.to_string().contains("workspace Cargo.lock unreadable"));
        std::fs::remove_dir(&lock_path).unwrap();
        // Regeneration recreates the member directory before writing its receipt.
        crate::api_gen::generate_api(&project, "rust").unwrap();
        assert!(backend_generation_is_fresh(&project));
        std::fs::write(&lock_path, "version = 4\n").unwrap();
        assert!(backend_generation_is_fresh(&project));
        let bound = std::fs::read_to_string(&receipt_path).unwrap();
        std::fs::remove_file(&lock_path).unwrap();
        assert!(!backend_generation_is_fresh(&project));
        std::fs::create_dir(&lock_path).unwrap();
        assert!(!backend_generation_is_fresh(&project));
        std::fs::remove_dir(&lock_path).unwrap();
        std::fs::write(&lock_path, "version = 4\n").unwrap();
        assert!(backend_generation_is_fresh(&project));
        assert_eq!(std::fs::read_to_string(&receipt_path).unwrap(), bound);
        eprintln!("R5_PROBE R11 read-error generator+unbound+bound+recovery PASS");

        std::fs::remove_file(&lock_path).unwrap();
        crate::api_gen::generate_api(&project, "rust").unwrap();
        std::fs::write(&lock_path, b"").unwrap();
        let first_empty_fresh = backend_generation_is_fresh(&project);
        let first_receipt: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&receipt_path).unwrap()).unwrap();
        crate::api_gen::generate_api(&project, "rust").unwrap();
        let empty_receipt: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&receipt_path).unwrap()).unwrap();
        let generated_empty_fresh = backend_generation_is_fresh(&project);
        eprintln!("R5_PROBE R11 empty-lock first_fresh={} first_receipt={} regenerated_receipt={} regenerated_fresh={}",
            first_empty_fresh, first_receipt["workspace_lock"],
            empty_receipt["workspace_lock"], generated_empty_fresh);
        assert_eq!(first_receipt["workspace_lock"], empty_receipt["workspace_lock"],
            "empty lock must have one identity in generator and freshness consumer");
        assert!(generated_empty_fresh, "unchanged successfully generated lock must be fresh");
    }
'''

if __name__ == "__main__":
    harness.main()
