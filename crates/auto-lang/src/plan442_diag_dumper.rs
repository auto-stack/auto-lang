// TEMPORARY diagnostic dumper (removed before commit): parse each blocked
// auto-src module with the VM parser scenario and print the Debug form of
// errors (variant fields: expected/found/span) for the divergence hunt.
#[cfg(test)]
mod plan442_diag_dumper {
    #[test]
    fn dump_blocked_module_diagnostics() {
        let src_dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .map(|d| std::path::PathBuf::from(d).join("../../../auto-musk/backend/crates/musk/auto-src"))
                .unwrap_or_else(|_| "../../../auto-musk/backend/crates/musk/auto-src".into()),
        );
        if !src_dir.is_dir() {
            eprintln!("diag: auto-src not found, skipping");
            return;
        }
        let r = std::panic::catch_unwind(|| {
            crate::run_file(src_dir.join("__repro.at").to_string_lossy().as_ref()).map(|_| String::new())
        });
        match r {
            Ok(Ok(_)) => eprintln!("REPRO RUN CLEAN"),
            Ok(Err(e)) => eprintln!("REPRO RUN ERR
{}", miette::Report::new(e)),
            Err(_) => eprintln!("REPRO RUN PANIC"),
        }    }
}
