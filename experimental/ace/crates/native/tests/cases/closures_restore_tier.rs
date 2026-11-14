//! PLAN-924 V-08 restore tiers: a persisted closures manifest restores on
//! a FRESH driver into a WARM session — the first run links the restored
//! objects with ZERO codegen (P0) over R>0 restored bytes, the next lambda
//! edit rides the ordinary warm routes with exactly-one-unit production —
//! and the refusal faces (forged ids, tampered store bytes) plus the
//! readonly no-publication leg behave as named errors, never as wrong
//! results. No live env, cell or code pointer is ever restored.

#[path = "../common/mod.rs"]
mod common;

use ace_native::driver::{CacheMode, MnDriver};

fn corpus(name: &str) -> String {
    let path = common::repo_root()
        .join("crates/auto-lang/test/cases/ace-m1/src")
        .join(format!("{name}.at"));
    std::fs::read_to_string(&path).expect("corpus source")
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "plan924-restore-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn store_snapshot(root: &std::path::Path) -> Vec<(std::path::PathBuf, u64)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                // The store's transient read-gate locks are not payload:
                // a readonly restore may take them, never write artifacts.
                if path.file_name().and_then(|n| n.to_str()) == Some("locks") {
                    continue;
                }
                stack.push(path);
            } else {
                out.push((path, entry.metadata().unwrap().len()));
            }
        }
    }
    out.sort();
    out
}

/// The full V-08 closure: publish K02, restore on a fresh driver into a
/// warm session (P0 first run, R>0 restored bytes), take the NEXT lambda
/// edit (42 -> 43, exactly the closure body unit), then a hit (zero
/// production over the installed PE).
#[test]
fn closures_restore_session_takes_next_lambda_edit_with_zero_first_production() {
    assert!(common::tools_available());
    let dir = scratch("session");
    let cache = dir.join("store");
    let source = corpus("K02");
    let mut publisher = MnDriver::with_store(&cache, "mn-closures-case").unwrap();
    let build = publisher
        .build_closures(&source, "main", &dir.join("publish-out"), None)
        .expect("publish K02");
    let manifest = build
        .manifest_artifact_id
        .clone()
        .expect("published manifest id");

    // A FRESH driver (a new "process") restores into a warm session.
    let mut fresh = MnDriver::with_store(&cache, "mn-closures-restore").unwrap();
    fresh.forbid_codegen = true;
    let (mut session, run) =
        match fresh.restore_closures_session(&manifest, &dir.join("restored-out"), None) {
            Ok(pair) => pair,
            Err(e) => panic!("restore session: {e:?}"),
        };
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.work.route, "restore");
    // P0: ZERO codegen over the restored objects.
    assert!(
        run.work.codegen_units.is_empty(),
        "restore codegen: {:?}",
        run.work.codegen_units
    );
    // R>0: the restored COFF bytes were actually read back.
    let read = run
        .work
        .obs
        .counters
        .get("restored_coff_bytes_read")
        .copied()
        .unwrap_or(0);
    assert!(read > 0, "no restored bytes read");
    assert_eq!(
        run.work
            .obs
            .counters
            .get("cache_hit_coff")
            .copied()
            .unwrap_or(0),
        3,
        "K02 has three restored units"
    );
    assert_eq!(session.manifest_id(), Some(manifest.as_str()));

    // The NEXT lambda edit rides the ordinary warm route: exactly the
    // closure body unit re-codegens, the value hand-models to 43.
    let edited = source.replace("+ n }", "+ n + 1 }");
    let edit = session
        .apply_source(&edited)
        .expect("warm edit after restore");
    assert_eq!(
        edit.exit_code,
        Some(43),
        "{}",
        String::from_utf8_lossy(&edit.stderr)
    );
    assert_eq!(
        edit.work.codegen_units,
        vec!["d___closure_make_0".to_string()],
        "route: {}",
        edit.work.route
    );
    // The hit after the edit is zero-production over the installed PE.
    let hit = session.apply_source(&edited).expect("hit");
    assert_eq!(hit.exit_code, Some(43));
    assert!(hit.work.codegen_units.is_empty());
    assert_eq!(hit.work.route, "hit");
    let _ = session.close();
    let _ = std::fs::remove_dir_all(&dir);
}

/// V-08 refusal and readonly faces: a forged manifest id refuses the whole
/// chain; a tampered store byte (the runtime blob) refuses the
/// runtime/content identity check; a readonly driver assembles a usable
/// session WITHOUT publication (store bytes untouched); the good manifest
/// then restores and takes the next edit.
#[test]
fn closures_restore_rejects_forged_and_tampered_and_stays_readonly() {
    assert!(common::tools_available());
    let dir = scratch("reject");
    let cache = dir.join("store");
    let source = corpus("K02");
    let mut publisher = MnDriver::with_store(&cache, "mn-closures-case").unwrap();
    let build = publisher
        .build_closures(&source, "main", &dir.join("publish-out"), None)
        .expect("publish K02");
    let manifest = build.manifest_artifact_id.clone().unwrap();

    // Forged identity: an unknown manifest id refuses before any link.
    let mut forged = MnDriver::with_store(&cache, "mn-closures-forged").unwrap();
    let error = match forged.restore_closures_session("forged-id", &dir.join("forged-out"), None) {
        Err(e) => e,
        Ok(_) => panic!("forged manifest id must refuse the restore chain"),
    };
    assert!(
        error.code().contains("artifact"),
        "forged id error: {error:?}"
    );

    // Tampered store bytes: flipping a byte in the largest stored blob
    // (the runtime library page) refuses the runtime/content identity.
    let tampered_root = dir.join("tampered");
    fn copy_tree(src: &std::path::Path, dst: &std::path::Path) {
        std::fs::create_dir_all(dst).unwrap();
        for entry in std::fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let to = dst.join(entry.file_name());
            if entry.path().is_dir() {
                copy_tree(&entry.path(), &to);
            } else {
                std::fs::copy(entry.path(), &to).unwrap();
            }
        }
    }
    copy_tree(&cache, &tampered_root);
    // Flip one byte in EVERY stored file (manifest, blobs, pages): the
    // chain must refuse somewhere — a corrupted payload never runs.
    for (path, len) in store_snapshot(&tampered_root) {
        if len == 0 {
            continue;
        }
        let mut bytes = std::fs::read(&path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x40;
        std::fs::write(&path, bytes).unwrap();
    }
    // The refusal fires wherever the corruption is caught — even at store
    // open when the store's own marker/TOC byte was the one flipped.
    let error = match MnDriver::with_store(&tampered_root, "mn-closures-tampered") {
        Err(open) => open,
        Ok(mut tampering) => {
            match tampering.restore_closures_session(&manifest, &dir.join("tampered-out"), None) {
                Err(e) => e,
                Ok(_) => panic!("tampered store bytes must refuse the restore chain"),
            }
        }
    };
    // The refusal is NAMED wherever the corruption is caught — the store
    // open (broken TOC/marker), the manifest/blob layer, or the identity
    // chain — and never a wrong result.
    assert!(
        error.code().contains("mn.admit.closures")
            || error.code().contains("artifact")
            || error.code().contains("resource")
            || error.code().contains("mn.infra"),
        "tamper error: {error:?}"
    );

    // Readonly: the restored session works, never publishes, and the store
    // bytes are untouched.
    let before = store_snapshot(&cache);
    let mut readonly = MnDriver::with_store(&cache, "mn-closures-readonly").unwrap();
    readonly.cache_mode = CacheMode::ReadOnly;
    let (mut session, run) =
        match readonly.restore_closures_session(&manifest, &dir.join("readonly-out"), None) {
            Ok(pair) => pair,
            Err(e) => panic!("readonly restore: {e:?}"),
        };
    assert_eq!(run.exit_code, Some(42));
    assert_eq!(session.manifest_id(), None);
    let edit = session
        .apply_source(&source.replace("+ n }", "+ n + 1 }"))
        .expect("readonly warm edit");
    assert_eq!(edit.exit_code, Some(43));
    assert_eq!(session.manifest_id(), None);
    assert_eq!(before, store_snapshot(&cache), "readonly wrote store bytes");
    session.close();

    // Recovery: the good manifest restores on a fresh writable driver and
    // the next lambda edit produces the same value.
    let mut recover = MnDriver::with_store(&cache, "mn-closures-recover").unwrap();
    recover.forbid_codegen = true;
    let (mut recovered, run) =
        match recover.restore_closures_session(&manifest, &dir.join("recover-out"), None) {
            Ok(pair) => pair,
            Err(e) => panic!("recovery restore: {e:?}"),
        };
    assert_eq!(run.exit_code, Some(42));
    let edit = recovered
        .apply_source(&source.replace("+ n }", "+ n + 1 }"))
        .expect("recovery warm edit");
    assert_eq!(edit.exit_code, Some(43));
    let _ = recovered.close();
    let _ = std::fs::remove_dir_all(&dir);
}

/// PLAN-1027 (UTF-01): real native session + restore UTF-8 single-region edit
/// fidelity across shared leading/trailing multi-byte codepoints, comments,
/// independent fresh PE controls, same-B zero-production hits, and controlled
/// non-ASCII identifier / out-of-domain rejections.
#[test]
fn audit1027_utf_01() {
    assert!(
        common::tools_available(),
        "native tools are required; no skip pass"
    );

    // 1. Scalar-count distinguishing PE exit-code oracle:
    // A (`s.len()` = 1, `f(41)` -> 42) -> B (`s.len()` = 2, `f(40)` -> 42)
    // Wait: keep `f(40)` fixed so only the string literal edits (1 -> 2 scalars,
    // A=41 cold -> B=42 warm & fresh). Pre-fix U+FFFD corruption produced 43+ or
    // span UTF-8 boundary failure.
    for (tag, a_ch, b_ch) in [
        ("e_ae", "é", "äé"),
        ("zh_chuanzh", "中", "串中"),
        ("emoji2", "😀", "😁😀"),
    ] {
        let src_a = format!(
            "fn main() int {{\n    let f = (x int) => {{\n        let s Str = \"{a_ch}\"\n        return s.len() + x\n    }}\n    return f(40)\n}}"
        );
        let src_b = src_a.replace(&format!("\"{a_ch}\""), &format!("\"{b_ch}\""));
        let dir_warm = scratch(&format!("utf01-warm-{tag}"));
        let (mut session, cold) = ace_native::closures_incremental::ClosuresNativeSession::new(
            &src_a, "main", &dir_warm, None,
        )
        .expect("cold start");
        assert_eq!(cold.exit_code, Some(41));
        let cold_hashes = session.object_hashes();

        let warm = session.apply_source(&src_b).expect("warm UTF-8 edit");
        assert_eq!(
            warm.exit_code,
            Some(42),
            "{tag}: stderr={}",
            String::from_utf8_lossy(&warm.stderr)
        );
        assert_eq!(
            warm.work.codegen_units,
            vec!["d___closure_main_0".to_string()],
            "{tag}: route={}",
            warm.work.route
        );
        assert!(
            warm.work.route.starts_with("fast-unit:"),
            "{tag}: route={}",
            warm.work.route
        );
        assert_eq!(
            session.object_hashes().get("d_main"),
            cold_hashes.get("d_main"),
            "{tag}: d_main object changed on lambda literal edit"
        );

        // Same-B hit: proves accepted source is byte-identical to B.
        let hit = session.apply_source(&src_b).expect("same-B hit");
        assert_eq!(hit.exit_code, Some(42));
        assert_eq!(hit.work.route, "hit");
        assert!(hit.work.codegen_units.is_empty());

        // Independent fresh session on B matches warm exit code.
        let dir_fresh = scratch(&format!("utf01-fresh-{tag}"));
        let (mut fresh_b, fresh_run) =
            ace_native::closures_incremental::ClosuresNativeSession::new(
                &src_b, "main", &dir_fresh, None,
            )
            .expect("fresh B");
        assert_eq!(fresh_run.exit_code, Some(42));
        let _ = fresh_b.close();
        let _ = session.close();
        let _ = std::fs::remove_dir_all(&dir_warm);
        let _ = std::fs::remove_dir_all(&dir_fresh);
    }

    // 2. Same-scalar-count literals (`é->ê`, `中->串`, `😀->😁`) and UTF-8 comment:
    // same-B hit proves accepted source == B (U+FFFD corruption missed `hit` pre-fix).
    for (tag, a_ch, b_ch) in [
        ("e_e", "é", "ê"),
        ("zh_chuan", "中", "串"),
        ("emoji", "😀", "😁"),
    ] {
        let src_a = format!(
            "fn main() int {{\n    // 注释 {a_ch}\n    let f = (x int) => {{\n        let s Str = \"{a_ch}\"\n        return s.len() + x\n    }}\n    return f(41)\n}}"
        );
        // Single-region literal edit first (`é` -> `ê`), then comment edit.
        let src_b = src_a.replacen(&format!("\"{a_ch}\""), &format!("\"{b_ch}\""), 1);
        let src_c = src_b.replacen(&format!("注释 {a_ch}"), &format!("注释 {b_ch}"), 1);
        let dir = scratch(&format!("utf01-same-{tag}"));
        let (mut session, cold) = ace_native::closures_incremental::ClosuresNativeSession::new(
            &src_a, "main", &dir, None,
        )
        .expect("cold");
        assert_eq!(cold.exit_code, Some(42));

        let warm_b = session.apply_source(&src_b).expect("warm literal");
        assert_eq!(warm_b.exit_code, Some(42));
        assert_eq!(
            warm_b.work.codegen_units,
            vec!["d___closure_main_0".to_string()]
        );
        let hit_b = session.apply_source(&src_b).expect("hit literal");
        assert_eq!(hit_b.work.route, "hit");
        assert!(hit_b.work.codegen_units.is_empty());

        let warm_c = session.apply_source(&src_c).expect("warm comment");
        assert_eq!(warm_c.exit_code, Some(42));
        let hit_c = session.apply_source(&src_c).expect("hit comment");
        assert_eq!(hit_c.work.route, "hit");
        assert!(hit_c.work.codegen_units.is_empty());

        let dir_fresh = scratch(&format!("utf01-fresh-same-{tag}"));
        let (mut fresh_c, fresh_run) =
            ace_native::closures_incremental::ClosuresNativeSession::new(
                &src_c, "main", &dir_fresh, None,
            )
            .expect("fresh C");
        assert_eq!(fresh_run.exit_code, Some(42));
        let _ = fresh_c.close();
        let _ = session.close();
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&dir_fresh);
    }

    // 3. Restored closures session takes a UTF-8 literal warm edit and subsequent hit cleanly.
    {
        let dir = scratch("utf01-restore");
        let cache = dir.join("store");
        let src_a = "fn main() int {\n    let f = (x int) => {\n        let s Str = \"é\"\n        return s.len() + x\n    }\n    return f(41)\n}";
        let src_b = src_a.replace("\"é\"", "\"ê\"");
        let mut publisher = MnDriver::with_store(&cache, "mn-closures-utf8").unwrap();
        let build = publisher
            .build_closures(src_a, "main", &dir.join("publish-out"), None)
            .expect("publish UTF-8 closures");
        let manifest = build.manifest_artifact_id.clone().unwrap();

        let mut fresh = MnDriver::with_store(&cache, "mn-closures-utf8-restore").unwrap();
        fresh.forbid_codegen = true;
        let (mut session, run) = fresh
            .restore_closures_session(&manifest, &dir.join("restored-out"), None)
            .expect("restore UTF-8 session");
        assert_eq!(run.exit_code, Some(42));
        assert!(run.work.codegen_units.is_empty());

        let warm = session
            .apply_source(&src_b)
            .expect("warm edit after restore");
        assert_eq!(warm.exit_code, Some(42));
        assert_eq!(
            warm.work.codegen_units,
            vec!["d___closure_main_0".to_string()]
        );
        let hit = session.apply_source(&src_b).expect("hit after restore");
        assert_eq!(hit.work.route, "hit");
        assert!(hit.work.codegen_units.is_empty());

        let _ = session.close();
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 4. Domain-boundary controls: non-ASCII identifier & out-of-profile text methods stay rejected.
    {
        let dir = scratch("utf01-controls");
        let err_ident = ace_native::closures_incremental::ClosuresNativeSession::new(
            "fn main() int {\n    let café int = 1\n    return café\n}",
            "main",
            &dir.join("ident"),
            None,
        )
        .err()
        .expect("non-ASCII identifier must reject");
        assert_eq!(err_ident.code(), "mn.unsupported.frontend");

        let err_len_bytes = ace_native::closures_incremental::ClosuresNativeSession::new(
            "fn main() int {\n    let f = (x int) => {\n        let s Str = \"é\"\n        return s.len_bytes() + x\n    }\n    return f(40)\n}",
            "main",
            &dir.join("lenbytes"),
            None,
        )
        .err()
        .expect("s.len_bytes() stays outside av.closures descriptor");
        assert_eq!(err_len_bytes.code(), "mn.unsupported.frontend");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
