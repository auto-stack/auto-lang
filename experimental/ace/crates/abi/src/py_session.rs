//! PLAN-1026 T-04: the public Python training session — the product entry
//! for running a `use.py` source through the full py lane (compile via the
//! aggregates foreign entry → closures lowering → foreign ABC admission →
//! combined console+foreign execution against the py-bridge CPython
//! adapter).
//!
//! The adapter IS this crate's py-bridge provider (967 B-01 build closure,
//! pinned pyo3); the session is serial (one interpreter, one owner thread
//! — the frozen v1 support boundary) and cheap to reopen (the prepared
//! cdylib is cached in the OS temp dir and shared across processes). The
//! interpreter is host infrastructure and is NEVER finalized per session
//! (R-06); dropping the session only tears down the binding generation.

use crate::rust_provider::{open_py_bridge_session, F0Session};
use ace_av_core::av::{execute_console_foreign, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::foreign_exec::ForeignStubTable;
use ace_av_core::host::{ConsoleHost, HostError, Stream};
use ace_foreign_contract::manual::AdmittedForeign;

/// A minimal in-memory console capture (the py lane's stdout observer).
#[derive(Default)]
pub struct PyConsole {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub dispatches: u64,
}

impl ConsoleHost for PyConsole {
    fn retained_bytes(&self) -> usize {
        self.stdout.len().saturating_add(self.stderr.len())
    }
    fn write(
        &mut self,
        stream: Stream,
        bytes: &[u8],
        _cancel: &CancelFlag,
    ) -> Result<(), HostError> {
        self.dispatches += 1;
        match stream {
            Stream::Stdout => self.stdout.extend_from_slice(bytes),
            Stream::Stderr => self.stderr.extend_from_slice(bytes),
        }
        Ok(())
    }
}

/// One serial py training session.
pub struct PySession {
    inner: F0Session,
}

/// The captured result of one run.
pub struct PyRun {
    pub outcome: ExecOutcome,
    pub console: PyConsole,
}

impl PyRun {
    pub fn success_i32(&self) -> Option<i32> {
        match &self.outcome {
            ExecOutcome::Success {
                value: Value::I32(v),
                ..
            } => Some(*v),
            _ => None,
        }
    }
    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.console.stdout).into_owned()
    }
}

impl PySession {
    /// Open the py adapter at binding generation 1.
    pub fn open() -> Result<Self, String> {
        Ok(Self {
            inner: open_py_bridge_session(1)?,
        })
    }

    /// The admitted foreign environment.
    pub fn admitted(&self) -> &AdmittedForeign {
        &self.inner.admitted
    }

    /// Compile + lower + admit + execute one `use.py` source end-to-end
    /// with the default budgets (the f64 reconstruction's pow2 loop is
    /// instruction-heavy; training runs want [`PySession::run_with`]).
    pub fn run(&mut self, src: &str) -> Result<PyRun, String> {
        self.run_with(src, &ExecLimits::default())
    }

    /// The same run with explicit limits.
    pub fn run_with(&mut self, src: &str, limits: &ExecLimits) -> Result<PyRun, String> {
        let caps = vec![
            ace_foreign_contract::manual::SOURCE_CAP.to_string(),
            ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
        ];
        let compiled = ace_core::source::compile_aggregates_source_with_foreign(
            src,
            "py-session.at",
            &caps,
            &self.inner.admitted,
        )
        .map_err(|ds| format!("compile: {ds:?}"))?;
        let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0)
            .map_err(|e| format!("lower: {e:?}"))?;
        ace_av_core::verify_mir(&mir).map_err(|e| format!("mir: {e:?}"))?;
        let encoded = ace_av_core::abc::encode_abc(&mir)
            .map_err(|e| format!("encode: {e:?}"))?
            .to_flat();
        let decoded =
            ace_av_core::abc::decode_abc(&encoded).map_err(|e| format!("decode: {e:?}"))?;
        let table: &mut ForeignStubTable = self.inner.session.table();
        let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, table)
            .map_err(|e| format!("admit: {e:?}"))?;
        let mut console = PyConsole::default();
        let (outcome, _) = execute_console_foreign(
            &loaded,
            limits,
            &CancelFlag::new(false),
            &["console.stdout".to_string()],
            &mut console,
            table,
        )
        .map_err(|e| format!("execute: {e:?}"))?;
        Ok(PyRun { outcome, console })
    }

    /// Tear down the binding generation (the interpreter stays host-owned).
    pub fn close(&mut self) {
        self.inner.session.teardown();
    }
}
