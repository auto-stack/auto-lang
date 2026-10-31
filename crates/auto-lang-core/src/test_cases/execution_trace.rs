//! Opt-in test governance receipts at the original comparison boundaries.
//! An absent terminal event is incomplete execution, never a passed child.
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
        let path = std::env::temp_dir().join(format!(
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
