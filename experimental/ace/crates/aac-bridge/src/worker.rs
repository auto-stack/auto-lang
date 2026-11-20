use crate::protocol::{read_frame, write_frame, Frame, REQUEST_MAGIC, RESPONSE_MAGIC};
use crate::{Failure, Outcome, Request, Response, REQUEST_CAP, RESPONSE_CAP};
use auto_tool_execution::{
    run_bounded, BoundedError, BoundedPolicy, ExecutionSpec, ExitKind, StdioPolicy, TreeScope,
};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const STDERR_CAP: usize = 64 * 1024;
const CLEANUP_BUDGET: Duration = Duration::from_secs(1);

/// One request on an owned pipe. Partial or malformed framing cannot produce
/// a success reply and the transport closes after a single exchange.
pub fn serve(input: impl Read, mut output: impl Write) -> Result<(), Failure> {
    let started = Instant::now();
    let frame = read_frame::<Request>(input, REQUEST_MAGIC, REQUEST_CAP)?;
    let id = frame.metadata.id;
    let (outcome, payload) = crate::execution::run(frame.metadata, &frame.bytes, started)
        .unwrap_or_else(|error| (Outcome::Failure { error }, Vec::new()));
    let response = Response {
        id,
        request_hash: frame.hash,
        outcome,
    };
    write_frame(
        &mut output,
        RESPONSE_MAGIC,
        RESPONSE_CAP,
        &response,
        &payload,
    )?;
    Ok(())
}

/// A client selects the bridge executable from trusted SDK configuration.
/// A program's IR cannot supply executables, environment overrides or paths.
pub struct Client {
    bridge: PathBuf,
}

impl Client {
    pub fn new(bridge: &Path) -> Result<Self, Failure> {
        if !bridge.is_absolute() {
            return Err(Failure::admission(
                "bridge.executable",
                "expected an absolute SDK bridge executable path",
            ));
        }
        let bridge = bridge
            .canonicalize()
            .map_err(|error| Failure::tool("bridge.executable", error.to_string()))?;
        if !bridge.is_file() {
            return Err(Failure::tool(
                "bridge.executable",
                "configured bridge executable is not a file",
            ));
        }
        Ok(Self { bridge })
    }

    pub fn run(&self, request: &Request, abc: &[u8]) -> Result<(Response, Vec<u8>), Failure> {
        let mut encoded = Vec::new();
        let started = Instant::now();
        let hash = write_frame(&mut encoded, REQUEST_MAGIC, REQUEST_CAP, request, abc)?;
        self.run_encoded(request, encoded, hash, started)
    }

    /// File transport into the same contained worker exchange. The caller
    /// owns the stage directory and passes absolute paths for this request.
    /// A valid response is published atomically without replacing a result
    /// that already occupies the requested path.
    pub fn exchange_file(&self, input: &Path, target: &Path) -> Result<(), Failure> {
        let started = Instant::now();
        if !input.is_absolute() || !target.is_absolute() {
            return Err(Failure::admission(
                "bridge.stage",
                "request and response must be absolute paths",
            ));
        }
        if target
            .try_exists()
            .map_err(|error| Failure::tool("bridge.stage", error.to_string()))?
        {
            return Err(Failure::admission(
                "bridge.stage",
                "response already exists",
            ));
        }
        let file = std::fs::File::open(input)
            .map_err(|error| Failure::tool("bridge.stage", error.to_string()))?;
        let mut encoded = Vec::new();
        file.take((REQUEST_CAP + 1) as u64)
            .read_to_end(&mut encoded)
            .map_err(|error| Failure::tool("bridge.stage", error.to_string()))?;
        if encoded.len() > REQUEST_CAP {
            return Err(Failure::resource(
                "bridge.frame-size",
                "request file exceeds the input cap",
            ));
        }
        let Frame { metadata, hash, .. } =
            read_frame::<Request>(encoded.as_slice(), REQUEST_MAGIC, REQUEST_CAP)?;
        let (response, payload) = self.run_encoded(&metadata, encoded, hash, started)?;
        let parent = target.parent().ok_or_else(|| {
            Failure::admission("bridge.stage", "response path has no parent directory")
        })?;
        let mut staged = tempfile::NamedTempFile::new_in(parent)
            .map_err(|error| Failure::tool("bridge.stage", error.to_string()))?;
        write_frame(
            staged.as_file_mut(),
            RESPONSE_MAGIC,
            RESPONSE_CAP,
            &response,
            &payload,
        )?;
        staged
            .persist_noclobber(target)
            .map_err(|error| Failure::tool("bridge.publish", error.error.to_string()))?;
        Ok(())
    }

    fn run_encoded(
        &self,
        request: &Request,
        encoded: Vec<u8>,
        hash: [u8; 32],
        started: Instant,
    ) -> Result<(Response, Vec<u8>), Failure> {
        let max = request.mode.budget().min(Duration::from_millis(u64::from(
            request.options.remaining_ms,
        )));
        let remaining = max.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Err(Failure::resource(
                "bridge.deadline",
                "the request has no remaining wall budget",
            ));
        }
        let mut spec =
            ExecutionSpec::new([self.bridge.as_os_str(), std::ffi::OsStr::new("--worker")]);
        spec.stdin = StdioPolicy::Piped;
        spec.stdin_data = Some(encoded);
        spec.tree_scope = TreeScope::ProcessTree;
        let response = run_bounded(
            &spec,
            &BoundedPolicy {
                exec_deadline: remaining,
                cleanup_deadline: CLEANUP_BUDGET,
                stdout_cap: Some(RESPONSE_CAP),
                stderr_cap: Some(STDERR_CAP),
            },
        )
        .map_err(transport_failure)?;
        if response.kind != ExitKind::Success {
            return Err(Failure::tool(
                "bridge.worker-exit",
                format!(
                    "bridge worker exited {:?}: {}",
                    response.kind,
                    String::from_utf8_lossy(response.stderr.as_deref().unwrap_or_default())
                ),
            ));
        }
        let stdout = response.stdout.ok_or_else(|| {
            Failure::tool(
                "bridge.worker-stdout",
                "worker did not produce a response stream",
            )
        })?;
        let frame = read_frame::<Response>(stdout.as_slice(), RESPONSE_MAGIC, RESPONSE_CAP)?;
        if request.id != frame.metadata.id || hash != frame.metadata.request_hash {
            return Err(Failure::admission(
                "bridge.response-identity",
                "response belongs to a different request",
            ));
        }
        match (&frame.metadata.outcome, request.mode) {
            (Outcome::Failure { .. }, _)
            | (Outcome::Objects { .. }, crate::Mode::NativeObjects)
            | (Outcome::Value { .. }, crate::Mode::CompileTime) => {}
            _ => {
                return Err(Failure::admission(
                    "bridge.response-mode",
                    "response belongs to a different operation",
                ))
            }
        }
        frame.metadata.outcome.verify_payload(&frame.bytes)?;
        Ok((frame.metadata, frame.bytes))
    }
}

fn transport_failure(error: BoundedError) -> Failure {
    match &error {
        BoundedError::ExecDeadline { cleanup, .. } if cleanup.completed => {
            Failure::resource("bridge.deadline", error.to_string())
        }
        BoundedError::StreamCap { cleanup, .. } if cleanup.completed => {
            Failure::resource("bridge.output", error.to_string())
        }
        _ => Failure::tool("bridge.worker", error.to_string()),
    }
}
