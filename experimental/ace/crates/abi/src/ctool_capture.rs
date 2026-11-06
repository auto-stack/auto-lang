//! Capture ownership for the source C-tool producer. The core executor still
//! owns containment, deadlines, reader collection and stream-cap rejection.
use ace_core::link::{CaptureCaps, CapturedRun, SubprocError};
use ace_source_query::resources::{Charge, Reservation, Resources};
use std::{process::Command, time::Duration};

#[derive(Debug)]
pub(crate) enum CaptureError {
    Resource(String),
    Subprocess(SubprocError),
}

pub(crate) struct Captured {
    // Bytes must drop before their reservation, including early failures.
    pub(crate) value: CapturedRun,
    pub(crate) _allocation: Option<Reservation>,
}

pub(crate) fn run(
    cmd: &mut Command,
    deadline: Duration,
    caps: CaptureCaps,
    resources: Option<&Resources>,
) -> Result<Captured, CaptureError> {
    run_observed(cmd, deadline, caps, resources, || Ok(()))
}

pub(crate) fn run_observed(
    cmd: &mut Command,
    deadline: Duration,
    caps: CaptureCaps,
    resources: Option<&Resources>,
    mut observe_output: impl FnMut() -> Result<(), String>,
) -> Result<Captured, CaptureError> {
    let observation_error = std::cell::Cell::new(None);
    let observe = || {
        crate::source_request_work::observe_scratch()
            .map_err(str::to_string)
            .and_then(|()| observe_output())
            .map_err(|message| {
                observation_error.set(Some(message));
                std::io::Error::other("source tool resource observation")
            })
    };
    let work = crate::source_request_work::begin(
        "tool-process",
        ace_source_query::work::Phase::Production,
    );
    let Some(resources) = resources else {
        return ace_core::link::run_deadlined_captured_observed(cmd, deadline, caps, observe)
            .map(|value| Captured {
                value,
                _allocation: None,
            })
            .map(|captured| {
                crate::source_request_work::io(
                    "tool-process",
                    (captured.value.stdout.len() + captured.value.stderr.len()) as u64,
                    0,
                );
                crate::source_request_work::complete(work);
                captured
            })
            .map_err(|error| match observation_error.take() {
                Some(message) => CaptureError::Resource(message),
                None => CaptureError::Subprocess(error),
            });
    };
    let bytes = caps
        .stdout_max
        .checked_add(caps.stderr_max)
        .filter(|_| caps.stdout_max != usize::MAX && caps.stderr_max != usize::MAX)
        .ok_or_else(|| CaptureError::Resource("C capture capacity overflow".into()))?;
    let allocation = resources
        .reserve(Charge::io(bytes))
        .map_err(|e| CaptureError::Resource(e.to_string()))?;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    stdout
        .try_reserve_exact(caps.stdout_max)
        .map_err(|e| CaptureError::Resource(format!("C stdout allocation: {e}")))?;
    stderr
        .try_reserve_exact(caps.stderr_max)
        .map_err(|e| CaptureError::Resource(format!("C stderr allocation: {e}")))?;
    let value = ace_core::link::run_deadlined_captured_into_observed(
        cmd, deadline, caps, stdout, stderr, observe,
    )
    .map_err(|error| match observation_error.take() {
        Some(message) => CaptureError::Resource(message),
        None => CaptureError::Subprocess(error),
    })?;
    crate::source_request_work::io(
        "tool-process",
        (value.stdout.len() + value.stderr.len()) as u64,
        0,
    );
    crate::source_request_work::complete(work);
    Ok(Captured {
        value,
        _allocation: Some(allocation),
    })
}
