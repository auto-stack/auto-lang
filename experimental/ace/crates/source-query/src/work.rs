//! Actual port observations. Production, metadata and recovery are distinct;
//! an attempted call is recorded before entering it, including failed calls.
//! A candidate's produced output remains pending until its owner accepts it.
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Production,
    Metadata,
    Recovery,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Totals {
    pub attempted: u64,
    pub produced: u64,
    pub reused: u64,
    pub accepted: u64,
    pub discarded: u64,
    pub pending: u64,
    pub p: u64,
    pub m: u64,
    pub r: u64,
    /// Logical payload bytes crossing the observed I/O port, not an estimate
    /// of kernel physical disk traffic (the parent Job measures that separately).
    pub read_bytes: u64,
    pub written_bytes: u64,
}
#[derive(Clone, Debug, Default)]
pub struct Ledger(Arc<Mutex<BTreeMap<String, BTreeMap<&'static str, Totals>>>>);
#[derive(Clone, Debug, Serialize)]
pub struct Row {
    pub owner: String,
    pub stage: &'static str,
    pub totals: Totals,
}
impl Ledger {
    pub fn observe_foreign<T>(&self, f: impl FnOnce() -> T) -> (T, Vec<Work>) {
        let outputs: Arc<Mutex<Vec<Work>>> = Arc::default();
        let captured = outputs.clone();
        let ledger = self.clone();
        let observer = Arc::new(move |stage, produced| {
            let mut outputs = captured.lock().expect("foreign work");
            if produced {
                if let Some(output) = outputs
                    .iter_mut()
                    .rev()
                    .find(|w| w.stage == stage && w.produced == 0)
                {
                    output.produced(1);
                }
            } else {
                outputs.push(ledger.begin("*", stage, Phase::Production));
            }
        });
        let result = ace_foreign_contract::work::with_observer(observer, f);
        let outputs = match Arc::try_unwrap(outputs) {
            Ok(m) => m.into_inner().expect("foreign work"),
            Err(_) => unreachable!("observer released"),
        };
        (result, outputs)
    }
    /// Observe the existing core producer hooks without resetting any enclosing
    /// test window. The returned tickets belong to the source candidate.
    pub fn observe_core<T>(&self, f: impl FnOnce() -> T) -> (T, Vec<Work>) {
        let outputs: Arc<Mutex<Vec<Work>>> = Arc::default();
        let captured = outputs.clone();
        let ledger = self.clone();
        let observer = Arc::new(move |stage, name: &str, produced| {
            let mut outputs = captured.lock().expect("core work");
            if produced {
                if let Some(output) = outputs
                    .iter_mut()
                    .rev()
                    .find(|w| w.stage == stage && w.owner == name && w.produced == 0)
                {
                    output.produced(1);
                }
            } else {
                outputs.push(ledger.begin(name, stage, Phase::Production));
            }
        });
        let result = ace_core::body_work::with_body_observer(observer, f);
        let outputs = match Arc::try_unwrap(outputs) {
            Ok(m) => m.into_inner().expect("core work"),
            Err(_) => unreachable!("observer released"),
        };
        (result, outputs)
    }
    fn update(&self, owner: &str, stage: &'static str, f: impl FnOnce(&mut Totals)) {
        f(self
            .0
            .lock()
            .expect("work ledger")
            .entry(owner.into())
            .or_default()
            .entry(stage)
            .or_default());
    }
    pub fn begin(&self, owner: &str, stage: &'static str, phase: Phase) -> Work {
        self.update(owner, stage, |t| {
            t.attempted += 1;
            match phase {
                Phase::Production => t.p += 1,
                Phase::Metadata => t.m += 1,
                Phase::Recovery => t.r += 1,
            }
        });
        Work {
            ledger: self.clone(),
            owner: owner.into(),
            stage,
            produced: 0,
            accepted: false,
        }
    }
    pub fn reuse(&self, owner: &str, stage: &'static str, count: u64) {
        self.update(owner, stage, |t| t.reused += count);
    }
    pub fn io(&self, owner: &str, stage: &'static str, read: u64, written: u64) {
        self.update(owner, stage, |t| {
            t.read_bytes += read;
            t.written_bytes += written;
        });
    }
    pub fn totals(&self, owner: &str, stage: &'static str) -> Totals {
        self.0
            .lock()
            .expect("work ledger")
            .get(owner)
            .and_then(|s| s.get(stage))
            .copied()
            .unwrap_or_default()
    }
    pub fn rows(&self) -> Vec<Row> {
        self.0
            .lock()
            .expect("work ledger")
            .iter()
            .flat_map(|(owner, stages)| {
                stages.iter().map(|(&stage, &totals)| Row {
                    owner: owner.clone(),
                    stage,
                    totals,
                })
            })
            .collect()
    }
}
pub struct Work {
    ledger: Ledger,
    owner: String,
    stage: &'static str,
    produced: u64,
    accepted: bool,
}
impl Work {
    pub fn produced(&mut self, count: u64) {
        self.produced += count;
        self.ledger.update(&self.owner, self.stage, |t| {
            t.produced += count;
            t.pending += count;
        });
    }
    pub fn accept(mut self) {
        self.ledger.update(&self.owner, self.stage, |t| {
            t.accepted += self.produced;
            t.pending -= self.produced;
        });
        self.accepted = true;
    }
    pub fn complete(mut self, count: u64) {
        self.produced(count);
        self.accept();
    }
}
impl Drop for Work {
    fn drop(&mut self) {
        if !self.accepted {
            self.ledger.update(&self.owner, self.stage, |t| {
                t.discarded += self.produced;
                t.pending -= self.produced;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unwinding_releases_observers_and_pending_candidate_outputs() {
        let ledger = Ledger::default();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ledger.observe_core(|| {
                ace_core::body_work::record_resolve("failed");
                ace_core::body_work::record_produced("resolve", "failed");
                panic!("candidate producer unwound");
            });
        }));
        assert!(panic.is_err());
        let t = ledger.totals("failed", "resolve");
        assert_eq!(
            (t.attempted, t.produced, t.discarded, t.pending),
            (1, 1, 1, 0)
        );
        ace_core::body_work::record_resolve("outside");
        assert_eq!(ledger.totals("outside", "resolve"), Totals::default());
        let (_, outputs) = ledger.observe_core(|| {
            ace_core::body_work::record_resolve("repaired");
            ace_core::body_work::record_produced("resolve", "repaired");
        });
        for output in outputs {
            output.accept();
        }
        assert_eq!(ledger.totals("repaired", "resolve").accepted, 1);
    }
    #[test]
    fn failed_attempts_pending_cancel_reuse_and_accept_are_distinct() {
        let l = Ledger::default();
        drop(l.begin("f", "hir", Phase::Production));
        {
            let mut w = l.begin("f", "hir", Phase::Production);
            w.produced(1);
        }
        l.begin("f", "hir", Phase::Production).complete(1);
        l.reuse("f", "hir", 1);
        let t = l.totals("f", "hir");
        assert_eq!(
            (
                t.attempted,
                t.produced,
                t.discarded,
                t.accepted,
                t.pending,
                t.reused,
                t.p
            ),
            (3, 2, 1, 1, 0, 1, 3)
        );
        l.begin("f", "restore", Phase::Recovery).accept();
        assert_eq!(l.totals("f", "restore").p, 0);
        assert_eq!(l.totals("f", "restore").r, 1);
    }
}
