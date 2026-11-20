use crate::protocol::{read_frame, write_frame, Frame, REQUEST_MAGIC, RESPONSE_MAGIC};
use crate::{Failure, ForeignPermit, Outcome, Request, Response, REQUEST_CAP, RESPONSE_CAP};
use auto_tool_execution::{
    run_bounded, run_bounded_cancellable, BoundedError, BoundedPolicy, ExecutionSpec, ExitKind,
    StdioPolicy, TreeScope,
};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

const STDERR_CAP: usize = 64 * 1024;
const CLEANUP_BUDGET: Duration = Duration::from_secs(1);

/// One request on an owned pipe. Partial or malformed framing cannot produce
/// a success reply and the transport closes after a single exchange.
pub fn serve(input: impl Read, output: impl Write) -> Result<(), Failure> {
    serve_inner(input, output, None)
}

pub fn serve_with_foreign(
    input: impl Read,
    output: impl Write,
    permit: &ForeignPermit,
) -> Result<(), Failure> {
    serve_inner(input, output, Some(permit))
}

fn serve_inner(
    input: impl Read,
    mut output: impl Write,
    permit: Option<&ForeignPermit>,
) -> Result<(), Failure> {
    let started = Instant::now();
    let frame = read_frame::<Request>(input, REQUEST_MAGIC, REQUEST_CAP)?;
    let id = frame.metadata.id;
    let (outcome, payload) = crate::execution::run(frame.metadata, &frame.bytes, started, permit)
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
    provider: Option<(PathBuf, [u8; 32])>,
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
        Ok(Self {
            bridge,
            provider: None,
        })
    }

    pub fn with_foreign_provider(mut self, config: &Path, hash: [u8; 32]) -> Result<Self, Failure> {
        ForeignPermit::read(config, hash)?;
        self.provider = Some((
            config
                .canonicalize()
                .map_err(|error| Failure::tool("bridge.foreign-config-path", error.to_string()))?,
            hash,
        ));
        Ok(self)
    }

    pub fn run(&self, request: &Request, abc: &[u8]) -> Result<(Response, Vec<u8>), Failure> {
        self.run_inner(request, abc, None)
    }

    pub fn run_cancellable(
        &self,
        request: &Request,
        abc: &[u8],
        cancel: &AtomicBool,
    ) -> Result<(Response, Vec<u8>), Failure> {
        self.run_inner(request, abc, Some(cancel))
    }

    fn run_inner(
        &self,
        request: &Request,
        abc: &[u8],
        cancel: Option<&AtomicBool>,
    ) -> Result<(Response, Vec<u8>), Failure> {
        let started = Instant::now();
        let mut encoded = Vec::new();
        let hash = write_frame(&mut encoded, REQUEST_MAGIC, REQUEST_CAP, request, abc)?;
        self.run_encoded(request, encoded, hash, started, cancel)
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
        let (response, payload) = self.run_encoded(&metadata, encoded, hash, started, None)?;
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
        cancel: Option<&AtomicBool>,
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
        let mut argv = vec![self.bridge.clone().into_os_string()];
        if let Some((path, digest)) = self
            .provider
            .as_ref()
            .filter(|_| request.external_provider.is_some())
        {
            argv.push("--worker-foreign".into());
            argv.push(path.clone().into_os_string());
            argv.push(blake3::Hash::from(*digest).to_hex().to_string().into());
        } else {
            argv.push("--worker".into());
        }
        let mut spec = ExecutionSpec::new(argv);
        spec.stdin = StdioPolicy::Piped;
        spec.stdin_data = Some(encoded);
        spec.tree_scope = TreeScope::ProcessTree;
        let policy = BoundedPolicy {
            exec_deadline: remaining,
            cleanup_deadline: CLEANUP_BUDGET,
            stdout_cap: Some(RESPONSE_CAP),
            stderr_cap: Some(STDERR_CAP),
        };
        let response = match cancel {
            Some(cancel) => run_bounded_cancellable(&spec, &policy, cancel),
            None => run_bounded(&spec, &policy),
        }
        .map_err(transport_failure)?;
        if response.kind == ExitKind::Cancelled {
            return Err(Failure::new(
                crate::FailureKind::Cancelled,
                "bridge.cancelled",
                "owned bridge request was cancelled and reaped",
            ));
        }
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
        accept_response(request, hash, &stdout)
    }
}

fn accept_response(
    request: &Request,
    hash: [u8; 32],
    stdout: &[u8],
) -> Result<(Response, Vec<u8>), Failure> {
    let frame = read_frame::<Response>(stdout, RESPONSE_MAGIC, RESPONSE_CAP)?;
    if request.id != frame.metadata.id || hash != frame.metadata.request_hash {
        return Err(Failure::admission(
            "bridge.response-identity",
            "response belongs to a different request",
        ));
    }
    use crate::{Mode, NativeRuntime};
    match (&frame.metadata.outcome, request.mode) {
        (Outcome::Failure { .. }, _)
        | (
            Outcome::Objects {
                runtime: NativeRuntime::None | NativeRuntime::Text,
                ..
            },
            Mode::NativeObjects,
        )
        | (
            Outcome::Objects {
                runtime: NativeRuntime::Collections,
                ..
            },
            Mode::NativeCollections,
        )
        | (
            Outcome::Objects {
                runtime: NativeRuntime::Scheduler,
                ..
            },
            Mode::NativeScheduler,
        )
        | (Outcome::Value { .. }, Mode::CompileTime) => {}
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

#[cfg(all(test, windows, target_arch = "x86_64", target_env = "msvc"))]
mod tests {
    use super::*;
    use crate::execution::EXECUTION_ENTRIES;
    use crate::{Domains, InputIdentity, Mode, Options, Value};
    use ace_av_core::contract::MIR_SCHEMA;
    use ace_av_core::mir::{
        FnSig, MirBlock, MirConst, MirFunction, MirInstr, MirModule, MirSlot, MirType, OwnerScope,
        SlotKind, Terminator,
    };

    fn fixture(mode: Mode) -> (Request, MirModule) {
        let owner = OwnerScope {
            schema: MIR_SCHEMA.into(),
            module: "framing".into(),
        };
        let mut mir = MirModule::new(owner.clone());
        let scalar = mir.intern_type(MirType::I32);
        let sig = mir.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: scalar,
        }));
        mir.consts.push(MirConst::I32(7));
        mir.funcs.push(MirFunction {
            owner: owner.clone(),
            name: "main".into(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Temp,
                ty: scalar,
                name: None,
            }],
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::Const { dst: 0, value: 0 }],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        let request = Request {
            id: [1; 16],
            mode,
            domains: Domains::windows(mode),
            inputs: InputIdentity {
                source: [1; 32],
                grammar: [2; 32],
                passes: [3; 32],
                stdlib: [4; 32],
                dependencies: [5; 32],
            },
            owner: owner.module,
            mir_schema: owner.schema,
            options: Options { remaining_ms: 2000 },
            capabilities: vec![],
            external_provider: None,
            globals: vec![],
            initializers: vec![],
        };
        (request, mir)
    }

    fn frame(request: &Request, mir: &MirModule) -> (Vec<u8>, [u8; 32]) {
        let abc = ace_av_core::encode_abc(mir).unwrap().to_flat();
        let mut encoded = Vec::new();
        let hash = write_frame(&mut encoded, REQUEST_MAGIC, REQUEST_CAP, request, &abc).unwrap();
        (encoded, hash)
    }

    #[test]
    fn gate_rejects_bad_frames_and_owner_reference_before_either_executor() {
        let start = Instant::now();
        for mode in [Mode::CompileTime, Mode::NativeObjects] {
            EXECUTION_ENTRIES.set(0);
            let (req, mut original) = fixture(mode);
            let (valid, hash) = frame(&req, &original);

            let mut wrong_version = valid.clone();
            wrong_version[3] = b'0';
            let mut oversize = valid.clone();
            oversize[8..12].copy_from_slice(&(REQUEST_CAP as u32 + 1).to_le_bytes());
            let mut bad_digest = valid.clone();
            bad_digest[12] ^= 1;
            let mut extra = valid.clone();
            extra.push(0);
            let mut forged_metadata = serde_json::to_value(&req).unwrap();
            forged_metadata["checked"] = true.into();
            let abc = ace_av_core::encode_abc(&original).unwrap().to_flat();
            let mut forged = Vec::new();
            write_frame(
                &mut forged,
                REQUEST_MAGIC,
                REQUEST_CAP,
                &forged_metadata,
                &abc,
            )
            .unwrap();

            for (bytes, code) in [
                (&wrong_version[..], "bridge.frame-version"),
                (&oversize, "bridge.frame-size"),
                (&bad_digest, "bridge.frame-checksum"),
                (&extra, "bridge.frame-trailing"),
                (&forged, "bridge.frame-metadata"),
            ] {
                let mut out = Vec::new();
                assert_eq!(serve(bytes, &mut out).unwrap_err().code, code);
                assert!(out.is_empty());
                assert_eq!(EXECUTION_ENTRIES.get(), 0);
            }

            let mut out = Vec::new();
            serve(valid.as_slice(), &mut out).unwrap();
            let (response, payload) = accept_response(&req, hash, &out).unwrap();
            assert_executor_output(mode, response.outcome, payload);
            assert_eq!(EXECUTION_ENTRIES.get(), 1);

            original.funcs[0].blocks[0].instrs[0] = MirInstr::Const { dst: 0, value: 63 };
            let (bad_ref, bad_hash) = frame(&req, &original);
            out.clear();
            serve(bad_ref.as_slice(), &mut out).unwrap();
            let (response, payload) = accept_response(&req, bad_hash, &out).unwrap();
            assert!(
                matches!(response.outcome, Outcome::Failure { error } if error.code == "mir.const-ref")
            );
            assert!(payload.is_empty());
            assert_eq!(EXECUTION_ENTRIES.get(), 1);

            out.clear();
            serve(valid.as_slice(), &mut out).unwrap();
            let (response, payload) = accept_response(&req, hash, &out).unwrap();
            assert_executor_output(mode, response.outcome, payload);
            assert_eq!(EXECUTION_ENTRIES.get(), 2);
        }
        eprintln!(
            "gate_rejects_bad_frames_and_owner_reference_before_either_executor {:?}",
            start.elapsed()
        );
    }

    fn assert_executor_output(mode: Mode, outcome: Outcome, payload: Vec<u8>) {
        if mode == Mode::CompileTime {
            assert!(
                matches!(outcome, Outcome::Value { value: Value::I32(7), facts } if facts.root_frames == 1 && facts.live_text == 0)
            );
            assert!(payload.is_empty());
        } else {
            assert!(matches!(outcome, Outcome::Objects { .. }));
            assert_eq!(payload.get(..2), Some(&[0x64, 0x86][..]));
        }
    }

    #[test]
    fn a_previous_valid_response_cannot_bind_to_a_new_id_or_request_digest() {
        let start = Instant::now();
        let (mut req, mir) = fixture(Mode::CompileTime);
        let (original, hash) = frame(&req, &mir);
        let mut response = Vec::new();
        serve(original.as_slice(), &mut response).unwrap();
        assert_executor_output(
            req.mode,
            accept_response(&req, hash, &response).unwrap().0.outcome,
            vec![],
        );
        req.id[0] ^= 1;
        assert_eq!(
            accept_response(&req, hash, &response).unwrap_err().code,
            "bridge.response-identity"
        );
        req.id[0] ^= 1;
        req.inputs.source[0] ^= 1;
        let (next, next_hash) = frame(&req, &mir);
        assert_ne!(hash, next_hash);
        assert_eq!(
            accept_response(&req, next_hash, &response)
                .unwrap_err()
                .code,
            "bridge.response-identity"
        );
        response.clear();
        serve(next.as_slice(), &mut response).unwrap();
        assert_executor_output(
            req.mode,
            accept_response(&req, next_hash, &response)
                .unwrap()
                .0
                .outcome,
            vec![],
        );
        eprintln!(
            "a_previous_valid_response_cannot_bind_to_a_new_id_or_request_digest {:?}",
            start.elapsed()
        );
    }
}
