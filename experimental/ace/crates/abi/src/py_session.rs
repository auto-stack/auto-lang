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

use crate::rust_provider::{
    ensure_python_dll_path, open_py_bridge_session, open_session_from_build, F0Session,
    RustProviderBuild,
};
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
    pub trace: ace_av_core::av::ExecTrace,
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

    /// Open the current binding with an explicitly prepared provider.
    pub fn from_prepared(build: &RustProviderBuild) -> Result<Self, String> {
        ensure_python_dll_path();
        Ok(Self {
            inner: open_session_from_build(build, 1)?,
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
        self.run_source(src, limits, false)
    }

    /// Run ordinary Auto with the current, source-bound Python provider.
    pub fn run_language_with(&mut self, src: &str, limits: &ExecLimits) -> Result<PyRun, String> {
        self.run_source(src, limits, true)
    }

    fn run_source(
        &mut self,
        src: &str,
        limits: &ExecLimits,
        ordinary: bool,
    ) -> Result<PyRun, String> {
        let caps = vec![
            ace_foreign_contract::manual::SOURCE_CAP.to_string(),
            ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
        ];
        let compile = if ordinary {
            ace_core::source::compile_language_source_with_foreign
        } else {
            ace_core::source::compile_aggregates_source_with_foreign
        };
        let lower = if ordinary {
            ace_lowering::lower_checked_auto_language
        } else {
            ace_lowering::lower_checked_closures
        };
        let compiled = compile(src, "py-session.at", &caps, &self.inner.admitted)
            .map_err(|ds| format!("compile: {ds:?}"))?;
        let mir = lower(&compiled.checked, "d_main", 0).map_err(|e| format!("lower: {e:?}"))?;
        ace_av_core::verify_mir(&mir).map_err(|e| format!("mir: {e:?}"))?;
        let encoded = ace_av_core::abc::encode_abc(&mir)
            .map_err(|e| format!("encode: {e:?}"))?
            .to_flat();
        let decoded =
            ace_av_core::abc::decode_abc(&encoded).map_err(|e| format!("decode: {e:?}"))?;
        let table: &mut ForeignStubTable = self.inner.session.table();
        if ordinary {
            table
                .bind_python_error_observers(&self.inner.admitted)
                .map_err(|error| format!("{}: {}", error.code, error.message))?;
        }
        let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, table)
            .map_err(|e| format!("admit: {e:?}"))?;
        let mut console = PyConsole::default();
        let (outcome, trace) = execute_console_foreign(
            &loaded,
            limits,
            &CancelFlag::new(false),
            &["console.stdout".to_string()],
            &mut console,
            table,
        )
        .map_err(|e| format!("execute: {e:?}"))?;
        Ok(PyRun {
            outcome,
            console,
            trace,
        })
    }

    /// Tear down the binding generation (the interpreter stays host-owned).
    pub fn close(&mut self) {
        self.inner.session.teardown();
    }
}
