//! Opt-in test governance receipts at the original comparison boundaries.
//! An absent terminal event is incomplete execution, never a passed child.
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// PLAN-941: original IPC bytes and process terminal, never a test verdict.
/// Opt-in at the existing parent boundary; no runner, parser, or meter lives
/// here. Each frame stays separate so recording does not copy full payloads
/// into a second JSON envelope. START without TERMINAL is incomplete evidence.
pub struct WorkerExchange {
    directory: PathBuf,
    identity: String,
    start_sha256: String,
}

impl WorkerExchange {
    pub fn enabled() -> Result<bool, String> {
        match std::env::var("AUTO_CASE_WIRE_EVIDENCE").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("0") => Ok(false),
            Ok("1") => Ok(true),
            _ => Err("AUTO_CASE_WIRE_EVIDENCE must be absent, 0 or 1".into()),
        }
    }

    pub fn start(
        artifact_dir: &std::path::Path,
        protocol: &str,
        recipe: &str,
        worker_sha256: &str,
        request: &[u8],
    ) -> Result<Option<Self>, String> {
        if Self::enabled()? {
            Self::start_at(artifact_dir, protocol, recipe, worker_sha256, request).map(Some)
        } else {
            Ok(None)
        }
    }

    fn start_at(
        artifact_dir: &std::path::Path,
        protocol: &str,
        recipe: &str,
        worker_sha256: &str,
        request: &[u8],
    ) -> Result<Self, String> {
        auto_test_case::admission::check_no_reparse_ancestors(artifact_dir)
            .map_err(|e| format!("exchange path: {e}"))?;
        let directory = artifact_dir.join("worker-exchanges");
        std::fs::create_dir_all(&directory).map_err(|e| format!("exchange directory: {e}"))?;
        auto_test_case::admission::check_no_reparse_ancestors(&directory)
            .map_err(|e| format!("exchange path: {e}"))?;
        let identity = claim_identity(&directory, std::process::id(), &SEQUENCE)
            .map_err(|e| format!("exchange identity: {e}"))?;
        let request = exchange_file(&directory, &identity, "request", request)?;
        let start = serde_json::json!({
            "schema":"case-worker-exchange/1", "exchange":identity, "phase":"start",
            "protocol":protocol, "recipe":recipe, "worker_sha256":worker_sha256,
            "request":request,
        });
        let start_sha256 = exchange_file(
            &directory,
            &identity,
            "start.json",
            &serde_json::to_vec(&start).map_err(|e| e.to_string())?,
        )?["sha256"]
            .as_str()
            .unwrap()
            .to_string();
        Ok(Self {
            directory,
            identity,
            start_sha256,
        })
    }

    pub fn finish(
        self,
        stdout: &[u8],
        stderr: &[u8],
        exit_code: Option<i32>,
        transport_error: Option<&str>,
    ) -> Result<(), String> {
        let stdout = exchange_file(&self.directory, &self.identity, "stdout", stdout)?;
        let stderr = exchange_file(&self.directory, &self.identity, "stderr", stderr)?;
        let terminal = serde_json::json!({
            "schema":"case-worker-exchange/1", "exchange":self.identity, "phase":"terminal",
            "start_sha256":self.start_sha256, "exit_code":exit_code,
            "stdout":stdout, "stderr":stderr, "transport_error":transport_error,
        });
        exchange_file(
            &self.directory,
            &self.identity,
            "terminal.json",
            &serde_json::to_vec(&terminal).map_err(|e| e.to_string())?,
        )?;
        Ok(())
    }
}

fn exchange_file(
    directory: &std::path::Path,
    identity: &str,
    role: &str,
    bytes: &[u8],
) -> Result<serde_json::Value, String> {
    use auto_test_case::sha256::{hex, sha256};
    use std::io::Write;
    // Mirrors the largest existing bounded ABI/AV transport frame including
    // its envelope allowance; individual domains retain their stricter caps.
    if bytes.len() > (4 << 20) + (64 << 10) {
        return Err("exchange frame exceeds original transport cap".into());
    }
    let name = format!("{identity}-{role}");
    auto_test_case::admission::check_no_reparse_ancestors(&directory.join(&name))
        .map_err(|e| format!("exchange path: {e}"))?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(&name))
        .map_err(|e| format!("exchange file: {e}"))?;
    file.write_all(bytes)
        .map_err(|e| format!("exchange write: {e}"))?;
    Ok(serde_json::json!({"path":name,"sha256":hex(&sha256(bytes)),"bytes":bytes.len()}))
}

fn claim_identity(
    directory: &std::path::Path,
    pid: u32,
    sequence: &AtomicU64,
) -> std::io::Result<String> {
    loop {
        let identity = format!("{}-{}", pid, sequence.fetch_add(1, Ordering::Relaxed));
        // Keep this reservation until the complete trace directory is retired.
        // Windows can reuse a PID while earlier events remain in that directory.
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(format!("{identity}.claim")))
        {
            Ok(_) => return Ok(identity),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

pub struct Invocation {
    directory: PathBuf,
    identity: String,
    finished: bool,
}

fn event(directory: &std::path::Path, identity: &str, phase: &str, data: serde_json::Value) {
    use std::io::Write;
    let path = directory.join(format!("{identity}-{phase}.json"));
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("test trace must not omit or overwrite an execution event");
    let value = serde_json::json!({"schema":"test-execution-trace/1","invocation":identity,
        "phase":phase,"module":std::env::var("AUTO_TEST_TRACE_MODULE").ok(),
        "rust_test":std::env::var("NEXTEST_TEST_NAME").ok().or_else(||std::thread::current().name().map(str::to_owned)),"data":data});
    output
        .write_all(&serde_json::to_vec(&value).unwrap())
        .expect("test trace write");
}

impl Invocation {
    pub fn start(metadata: impl FnOnce() -> serde_json::Value) -> Option<Self> {
        let directory = PathBuf::from(std::env::var_os("AUTO_TEST_TRACE_DIR")?);
        assert!(
            directory.is_absolute() && directory.is_dir(),
            "explicit trace directory required"
        );
        let identity = claim_identity(&directory, std::process::id(), &SEQUENCE)
            .expect("test trace identity reservation must succeed");
        event(&directory, &identity, "start", metadata());
        Some(Self {
            directory,
            identity,
            finished: false,
        })
    }

    pub fn finish(mut self, observation: serde_json::Value) {
        event(&self.directory, &self.identity, "finish", observation);
        self.finished = true;
    }
}

impl Drop for Invocation {
    fn drop(&mut self) {
        if !self.finished {
            // Avoid a second panic while unwinding; START without FINISH is
            // itself the durable evidence of an incomplete child invocation.
            if !std::thread::panicking() {
                event(
                    &self.directory,
                    &self.identity,
                    "incomplete",
                    serde_json::json!({"status":"ERROR"}),
                );
            }
        }
    }
}

pub fn file_identity(path: &std::path::Path) -> serde_json::Value {
    use auto_test_case::sha256::{hex, sha256};
    match std::fs::read(path) {
        Ok(bytes) => serde_json::json!({"path":path,"sha256":hex(&sha256(&bytes))}),
        Err(error) => serde_json::json!({"path":path,"read_error":error.to_string()}),
    }
}

pub fn observe<T, E>(
    metadata: impl FnOnce() -> serde_json::Value,
    operation: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let trace = Invocation::start(metadata);
    let result = operation();
    if let Some(trace) = trace {
        trace.finish(serde_json::json!({"status":if result.is_ok(){"PASS"}else{"ERROR"}}));
    }
    result
}

pub fn legacy_files(
    manifest: &str,
    base: &str,
    case: &str,
    oracle_suffixes: &[&str],
) -> serde_json::Value {
    let stem = case
        .rsplit('/')
        .next()
        .unwrap()
        .split_once('_')
        .map(|(_, name)| name)
        .unwrap_or(case);
    let directory = std::path::Path::new(manifest)
        .join("test")
        .join(base)
        .join(case);
    serde_json::json!({"family":format!("legacy-golden-{base}"),"id":case,
        "source":file_identity(&directory.join(format!("{stem}.at"))),
        "oracles":oracle_suffixes.iter().map(|suffix|file_identity(&directory.join(format!("{stem}{suffix}")))).collect::<Vec<_>>()})
}

pub fn legacy_error_identity(manifest: &str, case: &str, expected: &str) -> serde_json::Value {
    use auto_test_case::sha256::{hex, sha256};
    let mut value = legacy_files(manifest, "a2r", case, &[]);
    value["family"] = serde_json::json!("legacy-a2r-error");
    value["expected_error_substring_sha256"] = serde_json::json!(hex(&sha256(expected.as_bytes())));
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    fn directory() -> PathBuf {
        let root = std::env::var_os("AUTO_PLAN_ARTIFACTS")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!(
            "trace-claim-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn wire_bytes_failed_exit_and_identity_are_preserved() {
        use auto_test_case::sha256::{hex, sha256};
        let dir = directory();
        let request = b"{\"nonce\":\"actual-request\"}";
        let exchange =
            WorkerExchange::start_at(&dir, "proto/1", "recipe/1", &"a".repeat(64), request)
                .unwrap();
        let identity = exchange.identity.clone();
        let start_sha = exchange.start_sha256.clone();
        exchange
            .finish(
                &[0xff, 0, b'\n'],
                b"original stderr",
                Some(7),
                Some("worker exit"),
            )
            .unwrap();
        let base = dir.join("worker-exchanges");
        assert_eq!(
            std::fs::read(base.join(format!("{identity}-request"))).unwrap(),
            request
        );
        assert_eq!(
            std::fs::read(base.join(format!("{identity}-stdout"))).unwrap(),
            [0xff, 0, b'\n']
        );
        let terminal: serde_json::Value = serde_json::from_slice(
            &std::fs::read(base.join(format!("{identity}-terminal.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(terminal["start_sha256"], start_sha);
        assert_eq!(terminal["exit_code"], 7);
        assert_eq!(terminal["transport_error"], "worker exit");
        assert_eq!(
            terminal["stdout"]["sha256"],
            hex(&sha256(&[0xff, 0, b'\n']))
        );
        assert!(terminal.get("verdict").is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn wire_early_error_has_no_terminal_and_does_not_overwrite() {
        let dir = directory();
        let first = WorkerExchange::start_at(&dir, "p", "r", "hash", b"original").unwrap();
        let first_id = first.identity.clone();
        drop(first);
        let second = WorkerExchange::start_at(&dir, "p", "r", "hash", b"second").unwrap();
        assert_ne!(first_id, second.identity);
        let base = dir.join("worker-exchanges");
        assert!(!base.join(format!("{first_id}-terminal.json")).exists());
        assert_eq!(
            std::fs::read(base.join(format!("{first_id}-request"))).unwrap(),
            b"original"
        );
        drop(second);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn wire_oversize_is_not_truncated_into_a_valid_frame() {
        let dir = directory();
        let result =
            WorkerExchange::start_at(&dir, "p", "r", "hash", &vec![0; (4 << 20) + (64 << 10) + 1]);
        assert!(result.is_err());
        assert!(std::fs::read_dir(dir.join("worker-exchanges"))
            .unwrap()
            .all(|p| !p
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with("-request")));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn recycled_pid_reserves_new_identity_without_overwriting_prior_events() {
        let dir = directory();
        let first = claim_identity(&dir, 42, &AtomicU64::new(0)).unwrap();
        event(&dir, &first, "start", serde_json::json!({"original":true}));
        let bytes = std::fs::read(dir.join(format!("{first}-start.json"))).unwrap();
        // A fresh process counter with the same recycled PID must collide.
        let second = claim_identity(&dir, 42, &AtomicU64::new(0)).unwrap();
        assert_eq!(first, "42-0");
        assert_eq!(second, "42-1");
        event(&dir, &second, "start", serde_json::json!({"second":true}));
        assert_eq!(
            std::fs::read(dir.join(format!("{first}-start.json"))).unwrap(),
            bytes
        );
        assert!(dir.join(format!("{first}.claim")).is_file());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn claimed_identity_does_not_weaken_duplicate_json_event_rejection() {
        let dir = directory();
        let id = claim_identity(&dir, 42, &AtomicU64::new(0)).unwrap();
        event(&dir, &id, "finish", serde_json::json!({"original":true}));
        let path = dir.join(format!("{id}-finish.json"));
        let original = std::fs::read(&path).unwrap();
        assert!(std::panic::catch_unwind(|| event(
            &dir,
            &id,
            "finish",
            serde_json::json!({"wrong":true})
        ))
        .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
