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
        // Full pipeline-style session setup (the missing piece: resolve_deps
        // triggers compile_dep -> shim metadata registers imported fn
        // signatures into the type store), then parse the minimal repro.
        let code = std::fs::read_to_string(src_dir.join("__repro.at")).unwrap();
        let mut session = crate::compile::CompileSession::new();
        let _ = session.collect_rust_imports(&code);
        let _ = session.resolve_deps(&code);
        let _ = session.resolve_uses(&code);
        let mut parser = crate::Parser::new_with_type_store(code.as_str(), session.type_store());
        match parser.parse() {
            Ok(ast) => eprintln!("FULLPIPE PARSE OK ({} stmts)", ast.stmts.len()),
            Err(e) => eprintln!("FULLPIPE PARSE ERR
{}", miette::Report::new(e)),
        }    }
}
