//! Explicit collections consumer. Expected values never cross this boundary.
//! Transport is the existing bounded MN rounds shape; recipe/ABI are distinct.
use ac_mn::{
    collections_incremental::{CollectionRun, CollectionsIncrementalSession},
    driver::MnDriver,
    MnError,
};
use base64::Engine as _;
use serde_json::{json, Value};
use std::{
    io::{BufRead, Write},
    path::Path,
    process::ExitCode,
};
#[global_allocator]
static ALLOCATOR: ac_mn::collections_memory::CollectionsAllocator =
    ac_mn::collections_memory::CollectionsAllocator;
const RECIPE: &str = ac_mn::collections_layout::RECIPE;
const LINE_CAP: usize = (4 << 20) * 4 / 3 + (64 << 10);
static PRIVATE_BASE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn process_memory() -> (u64, u64, bool) {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::{
            ProcessStatus::{
                K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
            },
            Threading::GetCurrentProcess,
        };
        let mut counters: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        let size = std::mem::size_of_val(&counters) as u32;
        counters.cb = size;
        if K32GetProcessMemoryInfo(
            GetCurrentProcess(),
            (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX).cast::<PROCESS_MEMORY_COUNTERS>(),
            size,
        ) != 0
        {
            return (
                counters.PrivateUsage as u64,
                counters.PeakPagefileUsage as u64,
                true,
            );
        }
    }
    (0, 0, false)
}
fn worker_memory() -> Value {
    let (current, peak, ok) = process_memory();
    let base = PRIVATE_BASE.load(std::sync::atomic::Ordering::Relaxed);
    json!({"base":base,"current":current,"peak":peak,"delta":peak.saturating_sub(base),"ok":u32::from(ok)})
}
fn read_line(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    reader
        .take((LINE_CAP + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() > LINE_CAP || bytes.last() != Some(&b'\n') {
        return Err("missing/over-cap request line".into());
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| e.to_string())
}
fn respond(echo: &Value, result: Result<Value, String>) -> Result<(), String> {
    let mut response = echo.clone();
    match result {
        Ok(ok) => response["ok"] = ok,
        Err(error) => response["error"] = json!({"code":"mn.collections.worker","message":error}),
    }
    let bytes = serde_json::to_vec(&response).map_err(|e| e.to_string())?;
    if bytes.len() > LINE_CAP {
        return Err("response exceeds line cap".into());
    }
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    output
        .write_all(&bytes)
        .and_then(|_| output.write_all(b"\n"))
        .and_then(|_| output.flush())
        .map_err(|e| e.to_string())
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn identity() -> Value {
    json!({"recipe":RECIPE,"worker_version":"mnworker/1.0","mn_abi":ac_mn::config::MN_ABI,"worker_memory":worker_memory(),
    "mn_target":ac_mn::config::MN_TARGET,"collections_abi":ac_mn::collections_lower::ABI,
    "collections_codegen":ac_mn::collections_layout::codegen_build(),"counters":{},"spans":{}})
}
fn run_json(
    run: CollectionRun,
    op: &str,
    changed: Option<&str>,
    runtime: Option<&ac_mn::collections_runtime::Identity>,
    link: Option<&[String; 3]>,
) -> Value {
    let mut out = identity();
    out["allocation_meter"] = json!(ac_mn::collections_memory::snapshot());
    out["runtime_identity"] = json!(runtime);
    out["link_identity"] = json!(link);
    out["op"] = json!(op);
    out["exit_code"] = json!(run.exit_code);
    out["stderr_hex"] = json!(hex(&run.stderr));
    out["stdout_hex"] = json!(hex(&run.stdout));
    out["manifest_fp"] = json!(run.build.manifest_fp);
    out["manifest_id"] = json!(run.build.manifest_artifact_id);
    out["wall_ms"] = json!(run.work.elapsed_ms);
    out["changed_fn"] = json!(changed);
    out["codegen_units"] = json!(run
        .work
        .obs
        .counters
        .get("codegen_emit_units")
        .copied()
        .unwrap_or(0));
    out["cache_hits"] = json!(run
        .work
        .obs
        .counters
        .get("cache_hit_coff")
        .copied()
        .unwrap_or(0));
    out["reused_history_unit"] = json!(
        changed.is_some()
            && run
                .work
                .obs
                .counters
                .get("codegen_emit_units")
                .copied()
                .unwrap_or(0)
                == 0
    );
    out["refused"] = Value::Null;
    out["infra"] = Value::Null;
    // Session preparation structurally checks actual stage observations and Rc
    // identities before publication; include the raw producer observations too.
    out["unaffected_stage_delta"] = json!(run
        .work
        .bodies
        .iter()
        .filter(|(name, _)| Some(name.as_str()) != changed)
        .map(|(_, w)| (w.resolve + w.typecheck + w.plan + w.bind) as u64)
        .sum::<u64>());
    out["whole_module_stage_delta"] = json!(0);
    out["fallback_whole_lower"] = json!(0);
    out["parsed_functions"] = json!(run.work.syntax.parsed_functions);
    out["bodies"] = json!(run
        .work
        .bodies
        .iter()
        .map(|(name, w)| (name, json!([w.resolve, w.typecheck, w.plan, w.bind])))
        .collect::<std::collections::BTreeMap<_, _>>());
    out["ledger_peaks"] = json!(run.work.obs.ledger.peaks());
    out["counters"] = json!(run.work.obs.counters);
    out["spans"] = json!(run.work.obs.spans);
    out
}
fn error_json(error: MnError) -> Result<Value, String> {
    if matches!(
        error,
        MnError::Admit("collections-observation" | "runtime-value", _)
    ) {
        return Err(format!("{}: {}", error.code(), error.message()));
    }
    match error {
        MnError::Unsupported(..) | MnError::Admit(..) => {
            let mut out = identity();
            out["diagnostic"] = json!({"code":error.code(),"message":error.message()});
            Ok(out)
        }
        _ => Err(format!("{}: {}", error.code(), error.message())),
    }
}
fn str_field<'a>(request: &'a Value, key: &str) -> Result<&'a str, String> {
    request
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("missing {key}"))
}
fn serve() -> Result<(), String> {
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    ac_mn::collections_memory::reset_peaks();
    let mut request = read_line(&mut input)?.ok_or("missing request")?;
    let echo = json!({"schema":"mnworker/1","nonce":request["nonce"],"request_hash":request["request_hash"]});
    let mut execute = || -> Result<(Option<CollectionsIncrementalSession>, Value), String> {
        if str_field(&request, "recipe")? != RECIPE {
            return Err("wrong collections recipe".into());
        }
        // Binding is recomputed from the actual request, not merely echoed.
        use sha2::{Digest, Sha256};
        // Map::remove may swap the final key into the removed position when
        // preserve_order is enabled. Reconstruct the unsigned object so a
        // legitimate hash key in the middle cannot reorder signed fields.
        let unsigned = Value::Object(
            request
                .as_object()
                .ok_or("request object")?
                .iter()
                .filter(|(key, _)| key.as_str() != "request_hash")
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        );
        let actual_request_hash = hex(&Sha256::digest(unsigned.to_string().as_bytes()));
        if str_field(&request, "request_hash")? != actual_request_hash {
            return Err(format!(
                "request hash mismatch (render_bytes={}, actual={}, supplied={})",
                unsigned.to_string().len(),
                actual_request_hash,
                str_field(&request, "request_hash")?
            ));
        }
        drop(unsigned);
        if request.get("mode").and_then(Value::as_str) == Some("identity") {
            let runtime = ac_mn::collections_runtime::prepare(None).map_err(|e| e.message())?;
            let mut out = identity();
            out["runtime_identity"] = json!(runtime.identity);
            let (lld, sdk, kernel) =
                ac_mn::link_driver::collections_link_identities().map_err(|e| e.message())?;
            out["link_identity"] = json!([lld, sdk, kernel]);
            return Ok((None, out));
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(str_field(&request, "source_b64")?)
            .map_err(|e| e.to_string())?;
        if bytes.len() > ac_mn::config::MAX_INPUT_BYTES {
            return Err("source cap".into());
        }
        let source = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        // The signed wire image is no longer a producer input. Keep the decoded
        // source, not two base64 copies, throughout the retained frontend build.
        request.as_object_mut().unwrap().remove("source_b64");
        let store = Path::new(str_field(&request, "store_root")?);
        let workdir = Path::new(str_field(&request, "workdir")?);
        if let Some(parent) = store.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            ac_mn::link_driver::set_tool_disk_memo(parent.join("mn-collections-tool-hints.json"));
        }
        let entry = str_field(&request, "entry")?;
        if std::env::var("AC_MN_COLLECTIONS_FAULT").as_deref() == Ok("omit-frame-free") {
            // Actual incorrect native code, with no persistent cache available.
            ac_mn::collections_lower::test_omit_frame_free(true);
            let mut isolated = MnDriver::ephemeral();
            let run = isolated.request_collections(&source, entry, workdir, None);
            let omitted = ac_mn::collections_lower::test_omitted_returns();
            ac_mn::collections_lower::test_omit_frame_free(false);
            std::fs::create_dir_all(workdir).map_err(|e| e.to_string())?;
            std::fs::write(workdir.join(".mn889-omitted-free.stderr"), &run.stderr)
                .map_err(|e| e.to_string())?;
            std::fs::write(workdir.join(".mn889-omitted-free.json"),json!({"omitted_native_returns":omitted,"exit_code":run.exit_code,"codegen_units":run.obs.counters.get("codegen_emit_units"),"source_hash":ac_mn::cache::content_hash(source.as_bytes())}).to_string()).map_err(|e|e.to_string())?;
            let mut out = identity();
            out["allocation_meter"] = json!(ac_mn::collections_memory::snapshot());
            out["exit_code"] = json!(run.exit_code);
            out["stderr_hex"] = json!(hex(&run.stderr));
            out["stdout_hex"] = json!(hex(&run.stdout));
            out["counters"] = json!(run.obs.counters);
            out["spans"] = json!(run.obs.spans);
            out["omitted_native_returns"] = json!(omitted);
            return Ok((None, out));
        }
        let mut driver =
            MnDriver::with_store(store, "mn-collections-case").map_err(|e| e.message())?;
        if let Some(id) = request.get("manifest_id").and_then(Value::as_str) {
            if request.get("mode").and_then(Value::as_str) == Some("rounds") {
                return Err("restore cannot create a retained source session".into());
            }
            let persisted = ac_artifact_probe::store::ManagedStore::open_existing(store)
                .map_err(|e| e.to_string())?;
            let manifest = ac_mn::collections_artifacts::get_manifest(&persisted, id)
                .map_err(|e| e.message())?;
            let normalized = if entry.starts_with("d_") {
                entry.to_owned()
            } else {
                format!("d_{entry}")
            };
            if manifest.source_hash != ac_mn::cache::content_hash(source.as_bytes())
                || manifest.entry != normalized
            {
                return Err("restore source/entry binding mismatch".into());
            }
            drop(persisted);
            driver.forbid_codegen = true;
            let run = driver.request_restored_collections(id, workdir, None);
            if !run.ok {
                return Err(run
                    .error
                    .map(|e| e.message())
                    .unwrap_or("restore failed".into()));
            }
            let mut out = identity();
            out["allocation_meter"] = json!(ac_mn::collections_memory::snapshot());
            out["exit_code"] = json!(run.exit_code);
            out["stderr_hex"] = json!(hex(&run.stderr));
            out["stdout_hex"] = json!(hex(&run.stdout));
            out["manifest_fp"] = json!(run.manifest_fp);
            out["runtime_identity"] = json!(manifest.runtime);
            out["link_identity"] = json!([manifest.lld, manifest.sdk, manifest.kernel32]);
            out["manifest_id"] = json!(run.manifest_artifact_id);
            out["counters"] = json!(run.obs.counters);
            out["spans"] = json!(run.obs.spans);
            return Ok((None, out));
        }
        match CollectionsIncrementalSession::new(&source, entry, driver, workdir, None) {
            Ok((session, run)) => {
                let out = run_json(
                    run,
                    if request.get("mode").and_then(Value::as_str) == Some("rounds") {
                        "warmup"
                    } else {
                        "run"
                    },
                    None,
                    session.runtime_identity(),
                    session.link_identity(),
                );
                Ok((Some(session), out))
            }
            Err(error) => Ok((None, error_json(error)?)),
        }
    };
    let (session, out) = match execute() {
        Ok(value) => value,
        Err(error) => {
            respond(&echo, Err(error))?;
            return Ok(());
        }
    };
    respond(&echo, Ok(out))?;
    let Some(mut session) = session else {
        return Ok(());
    };
    match std::env::var("AC_MN_COLLECTIONS_FAULT").as_deref() {
        Ok("whole-front") => session.negative_controls.whole_frontend = true,
        Ok("whole-lower") => session.negative_controls.whole_lower = true,
        Ok("copy-unchanged") => session.negative_controls.copy_unchanged_code = true,
        _ => {}
    }
    if request.get("mode").and_then(Value::as_str) != Some("rounds") {
        session.close();
        return Ok(());
    }
    loop {
        ac_mn::collections_memory::reset_peaks();
        let Some(step) = read_line(&mut input)? else {
            break;
        };
        let echo = json!({"schema":"mnworker/1","nonce":step["nonce"]});
        if step.get("op").and_then(Value::as_str) == Some("close") {
            session.close();
            respond(
                &echo,
                Ok(
                    json!({"op":"close","allocation_meter":ac_mn::collections_memory::snapshot(),"worker_memory":worker_memory()}),
                ),
            )?;
            return Ok(());
        }
        let result = (|| -> Result<Value, String> {
            let op = str_field(&step, "op")?;
            let before = session.identities();
            match op {
                "edit-source" => {
                    let from = str_field(&step, "from")?;
                    let to = step.get("to").and_then(Value::as_str).ok_or("missing to")?;
                    if session.source().matches(from).count() != 1 {
                        return Err("edit must match exactly once".into());
                    }
                    let source = session.source().replacen(from, to, 1);
                    let candidate = session.prepare_source(&source).map_err(|e| {
                        format!(
                            "{}; actual parsed={:?}; actual bodies={:?}; actual counters={:?}",
                            e.message(),
                            session.last_work().syntax.parsed_functions,
                            session.last_work().bodies,
                            session.last_work().obs.counters
                        )
                    })?;
                    let changed = candidate.changed_function().map(str::to_string);
                    let run = session
                        .commit_prepared(candidate)
                        .map_err(|e| e.message())?;
                    let mut out = run_json(
                        run,
                        op,
                        changed.as_deref(),
                        session.runtime_identity(),
                        session.link_identity(),
                    );
                    let after = session.identities();
                    out["identities_before"] = json!(before);
                    out["identities_after"] = json!(after);
                    Ok(out)
                }
                "run" => {
                    let source = session.source().to_owned();
                    let run = session.apply_source(&source).map_err(|e| e.message())?;
                    let mut out = run_json(
                        run,
                        op,
                        None,
                        session.runtime_identity(),
                        session.link_identity(),
                    );
                    out["identities_before"] = json!(before);
                    out["identities_after"] = json!(session.identities());
                    Ok(out)
                }
                _ => Err("unsupported collections operation".into()),
            }
        })();
        respond(&echo, result)?;
    }
    session.close();
    Ok(())
}
fn main() -> ExitCode {
    PRIVATE_BASE.store(process_memory().0, std::sync::atomic::Ordering::Relaxed);
    match serve() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("mn-collections-worker: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod allocation_controls {
    #[test]
    fn exact_backing_alignment_realloc_unwind_and_partition_boundary() {
        use ac_mn::{collections_memory as meter, resources::Partition};
        use std::alloc::{alloc, dealloc, realloc, Layout};
        let initial = meter::snapshot().current;
        unsafe {
            let layout = Layout::from_size_align(37, 4096).unwrap();
            let ptr = meter::scoped(Partition::Io, || alloc(layout));
            assert!(!ptr.is_null());
            assert_eq!((ptr as usize) % 4096, 0);
            for i in 0..37 {
                ptr.add(i).write(i as u8);
            }
            assert_eq!(meter::snapshot().current[2] - initial[2], 4133);
            let resized = realloc(ptr, layout, 80);
            assert!(!resized.is_null());
            assert_eq!((resized as usize) % 4096, 0);
            for i in 0..37 {
                assert_eq!(resized.add(i).read(), i as u8);
            }
            assert_eq!(meter::snapshot().current[2], initial[2]);
            dealloc(resized, Layout::from_size_align(80, 4096).unwrap());
            meter::reset_peaks();
            let size = (8 << 20) - 4 * std::mem::size_of::<usize>();
            let layout = Layout::from_size_align(size, 8).unwrap();
            let boundary = meter::scoped(Partition::Io, || alloc(layout));
            assert!(!boundary.is_null());
            assert_eq!(meter::snapshot().current[2] - initial[2], 8 << 20);
            assert!(meter::check().is_ok());
            let one = Layout::from_size_align(1, 1).unwrap();
            let excess = meter::scoped(Partition::Io, || alloc(one));
            assert!(!excess.is_null());
            assert!(meter::check().is_err());
            dealloc(excess, one);
            dealloc(boundary, layout);
            assert_eq!(meter::snapshot().current[2], initial[2]);
            meter::reset_peaks();
            assert!(meter::check().is_ok());
        }
        // Scope restoration runs during unwinding too; the original partition
        // owns its eventual frees even when the caller changes scopes.
        let _ = std::panic::catch_unwind(|| {
            meter::scoped(Partition::MirSymbol, || panic!("allocator scope control"))
        });
        let mut after = std::vec::Vec::<u8>::with_capacity(1024);
        after.push(7);
        assert_eq!(meter::snapshot().current[0], initial[0]);
        drop(after);
    }
}
