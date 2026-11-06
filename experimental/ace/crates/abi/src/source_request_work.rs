//! Request-local host ports, kept separate from per-source producer ports.
//! Only the explicit source certification entry installs this observation.
use ace_source_query::{
    resources::{Charge, Reservation, Resources},
    work::{Ledger, Phase, Work},
};
use serde::{Serialize, Serializer};
use std::{cell::RefCell, fmt, sync::Arc};

thread_local! {
    static CURRENT: RefCell<Option<Ledger>> = const { RefCell::new(None) };
}
struct Owner {
    ledger: Ledger,
    _allocation: Reservation,
}
#[derive(Clone)]
pub struct RequestWork(Arc<Owner>);
impl fmt::Debug for RequestWork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.ledger.fmt(f)
    }
}
impl Serialize for RequestWork {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.ledger.serialize(serializer)
    }
}
impl RequestWork {
    pub(crate) fn new(resources: &Resources) -> Result<Self, String> {
        // At most eight fixed host rows, one '*' owner, and two live port
        // tickets. Includes the Arc and BTree node capacities, before any of
        // those owners is allocated. No source-dependent name enters here.
        let allocation = resources
            .reserve(Charge::index(16 << 10))
            .map_err(|e| e.to_string())?;
        Ok(Self(Arc::new(Owner {
            ledger: Ledger::default(),
            _allocation: allocation,
        })))
    }
    pub(crate) fn enter(&self) -> Scope {
        let previous = CURRENT.with(|slot| slot.replace(Some(self.0.ledger.clone())));
        Scope {
            previous,
            _owner: self.clone(),
        }
    }
}
pub(crate) struct Scope {
    previous: Option<Ledger>,
    _owner: RequestWork,
}
impl Drop for Scope {
    fn drop(&mut self) {
        CURRENT.with(|slot| {
            slot.replace(self.previous.take());
        });
    }
}
pub(crate) fn begin(stage: &'static str, phase: Phase) -> Option<Work> {
    // The fixed set is also the bound used by the pre-allocation reservation.
    assert!(matches!(
        stage,
        "provider-cache"
            | "provider-compile"
            | "tool-process"
            | "crt-context"
            | "reference-compile"
            | "actual-run"
            | "reference-run"
            | "tool-input"
    ));
    CURRENT.with(|slot| slot.borrow().as_ref().map(|l| l.begin("*", stage, phase)))
}
pub(crate) fn io(stage: &'static str, read: u64, written: u64) {
    CURRENT.with(|slot| {
        if let Some(l) = slot.borrow().as_ref() {
            l.io("*", stage, read, written);
        }
    });
}
pub(crate) fn reuse(stage: &'static str) {
    CURRENT.with(|slot| {
        if let Some(l) = slot.borrow().as_ref() {
            l.reuse("*", stage, 1);
        }
    });
}
pub(crate) fn complete(work: Option<Work>) {
    if let Some(work) = work {
        work.complete(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::process::CommandExt;
    use std::{process::Command, time::Duration};
    #[test]
    fn actual_tool_refusal_attempt_io_and_scope_lifetime_are_reported_without_a_row_copy() {
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let limited = Resources::limited(ace_source_query::resources::Limits {
            index: (16 << 10) - 1,
            ..Default::default()
        })
        .unwrap();
        let limited_baseline = limited.stats().current;
        assert!(RequestWork::new(&limited).is_err());
        assert_eq!(limited.stats().current, limited_baseline);
        let work = RequestWork::new(&resources).unwrap();
        let scope = work.enter();
        let path = std::path::PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("request-work-{}.marker", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut command = Command::new("cmd.exe");
        command.args(["/D", "/C"]);
        command.raw_arg(format!(
            "echo measured>\"{}\" & echo observed",
            path.display()
        ));
        let pressure = resources.reserve(Charge::io((8 << 20) - 127)).unwrap();
        let caps = ace_core::link::CaptureCaps {
            stdout_max: 64,
            stderr_max: 64,
        };
        assert!(matches!(
            crate::ctool_capture::run(&mut command, Duration::from_secs(5), caps, Some(&resources)),
            Err(crate::ctool_capture::CaptureError::Resource(_))
        ));
        assert!(!path.exists(), "a refused invocation reached the real tool");
        drop(pressure);
        let captured =
            crate::ctool_capture::run(&mut command, Duration::from_secs(5), caps, Some(&resources))
                .unwrap();
        assert!(captured.value.stdout.starts_with(b"observed"));
        assert!(path.is_file());
        let totals = work.0.ledger.totals("*", "tool-process");
        assert_eq!(
            (
                totals.attempted,
                totals.p,
                totals.produced,
                totals.accepted,
                totals.pending
            ),
            (2, 2, 1, 1, 0)
        );
        assert_eq!(totals.read_bytes, captured.value.stdout.len() as u64);
        drop(captured);
        let bytes = serde_json::to_vec(&work).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()[0]["totals"]["attempted"],
            2
        );
        drop(work);
        assert_eq!(
            resources.stats().current.index,
            baseline.index + (16 << 10),
            "scope still owns the observed ledger"
        );
        drop(scope);
        assert_eq!(resources.stats().current, baseline);
        assert!(begin("actual-run", Phase::Production).is_none());
        std::fs::remove_file(path).unwrap();
        let work = RequestWork::new(&resources).unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _scope = work.enter();
            let _pending = begin("actual-run", Phase::Production);
            panic!("request unwound");
        }))
        .is_err());
        assert!(begin("actual-run", Phase::Production).is_none());
        drop(work);
        assert_eq!(resources.stats().current, baseline);
    }
}
