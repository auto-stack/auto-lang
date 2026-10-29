//! Opt-in test governance receipts at the original comparison boundaries.
//! An absent terminal event is incomplete execution, never a passed child.
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
        let identity = format!(
            "{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
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
