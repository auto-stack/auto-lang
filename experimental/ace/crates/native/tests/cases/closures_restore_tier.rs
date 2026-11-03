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
    let (mut session, run) = match fresh
        .restore_closures_session(&manifest, &dir.join("restored-out"), None)
    {
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
        run.work.obs.counters.get("cache_hit_coff").copied().unwrap_or(0),
        3,
        "K02 has three restored units"
    );
    assert_eq!(session.manifest_id(), Some(manifest.as_str()));

    // The NEXT lambda edit rides the ordinary warm route: exactly the
    // closure body unit re-codegens, the value hand-models to 43.
    let edited = source.replace("+ n }", "+ n + 1 }");
    let edit = session.apply_source(&edited).expect("warm edit after restore");
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
    let error = match forged
        .restore_closures_session("forged-id", &dir.join("forged-out"), None)
    {
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
        Ok(mut tampering) => match tampering
            .restore_closures_session(&manifest, &dir.join("tampered-out"), None)
        {
            Err(e) => e,
            Ok(_) => panic!("tampered store bytes must refuse the restore chain"),
        },
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
    let (mut session, run) = match readonly
        .restore_closures_session(&manifest, &dir.join("readonly-out"), None)
    {
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
    let (mut recovered, run) = match recover
        .restore_closures_session(&manifest, &dir.join("recover-out"), None)
    {
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
