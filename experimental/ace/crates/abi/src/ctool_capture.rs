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
    let Some(resources) = resources else {
        return ace_core::link::run_deadlined_captured(cmd, deadline, caps)
            .map(|value| Captured {
                value,
                _allocation: None,
            })
            .map_err(CaptureError::Subprocess);
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
    let value = ace_core::link::run_deadlined_captured_into(cmd, deadline, caps, stdout, stderr)
        .map_err(CaptureError::Subprocess)?;
    Ok(Captured {
        value,
        _allocation: Some(allocation),
    })
}
