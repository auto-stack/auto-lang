//! Plan 442 Phase C1 probe (auto-musk backend corpus, VM runtime smoke).
//!
//! The musk backend (`backend/crates/musk/auto-src/*.at`) is Auto-sourced and
//! today reaches production via a2r transpilation. Phase C asks whether the
//! SAME sources can run directly on the AutoVM. This probe drives the purest
//! module (`app_config.at`) through `run_file` and records the first runtime
//! gap (use.rust FFI surface) as test output — the C1 gap ledger starter.
//!
//! `#[ignore]` — sibling-checkout assumption, manual-only:
//!   cargo test -p auto-lang --lib musk_backend -- --ignored --nocapture

#[cfg(test)]
mod plan442_musk_backend_probe {
    fn locate_auto_src() -> Option<std::path::PathBuf> {
        let rel = "../../../auto-musk/backend/crates/musk/auto-src";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
        ];
        candidates.into_iter().flatten().find(|p| p.is_dir())
    }

    /// C1 smoke: VM-direct run of a driver importing app_config.at.
    #[test]
    #[ignore = "requires sibling auto-musk checkout; manual Phase-C gate"]
    fn musk_backend_app_config_vm_run() {
        let Some(src_dir) = locate_auto_src() else {
            eprintln!("plan442 musk backend probe: SKIPPED — auto-musk not found");
            return;
        };
        // Driver lives inside auto-src so `use app_config` resolves file-locally.
        let driver = src_dir.join("__plan442_trial_driver.at");
        std::fs::write(
            &driver,
            "// Plan 442 C1 probe driver (generated; delete freely)\n\
             use app_config\n\n\
             fn main() {\n    print(\"plan442-c1-ok\")\n}\n",
        )
        .expect("write driver");
        let out = std::panic::catch_unwind(|| {
            let r = crate::run_file(driver.to_string_lossy().as_ref());
            r.map(|_| String::new())
        });
        let _ = std::fs::remove_file(&driver);
        match out {
            Ok(Ok(_)) => {
                // run_file prints directly; treat clean return as the smoke pass.
                eprintln!("plan442-c1: VM run of app_config driver returned Ok");
            }
            Ok(Err(e)) => panic!("VM run failed (C1 gap): {e}"),
            Err(_) => panic!("VM run panicked (C1 gap)"),
        }
    }
}
