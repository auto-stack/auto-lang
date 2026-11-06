//! Cumulative bytes of newly materialized artifacts in one explicit request.
//! This account is separate from live heap ownership: a produced artifact
//! remains spent after its last owner drops or its candidate is discarded.
use super::{ResourceError, Resources};

#[derive(Clone, Copy, Debug)]
pub struct OutputLimits {
    pub artifact: u64,
    pub request: u64,
}
impl Default for OutputLimits {
    fn default() -> Self {
        Self {
            artifact: 64 << 20,
            request: 128 << 20,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutputStats {
    pub produced: u64,
    pub pending: u64,
    pub peak_reserved: u64,
    pub refused: bool,
}
pub(super) struct Budget {
    limits: OutputLimits,
    stats: OutputStats,
}
impl Resources {
    /// The host installs this once, before request production. Long-lived
    /// library sessions retain their existing live-memory account; their host
    /// chooses request boundaries rather than treating lifetime churn as one
    /// cumulative production request. Payloads cannot install or reset limits.
    pub fn enable_output_budget(&self, limits: OutputLimits) -> Result<(), ResourceError> {
        let maximum = OutputLimits::default();
        let mut state = self.0.lock().expect("resource account");
        if limits.artifact > maximum.artifact
            || limits.request > maximum.request
            || state.production.is_some()
        {
            state.stats.rejections += 1;
            return Err(ResourceError::Budget(
                "cannot increase or reset request output budget".into(),
            ));
        }
        state.production = Some(Budget {
            limits,
            stats: OutputStats::default(),
        });
        Ok(())
    }
    pub fn output_stats(&self) -> Option<OutputStats> {
        self.0
            .lock()
            .expect("resource account")
            .production
            .as_ref()
            .map(|b| b.stats)
    }
    /// Reserve a measured size or proven bound BEFORE allocating the output.
    /// An unfinished permit returns only pending capacity. Completed bytes
    /// never return to the cumulative allowance, even on later failure.
    pub fn reserve_output(&self, bytes: usize) -> Result<OutputPermit, ResourceError> {
        let mut state = self.0.lock().expect("resource account");
        let Some(budget) = state.production.as_mut() else {
            return Ok(OutputPermit {
                resources: None,
                bytes: bytes as u64,
            });
        };
        let bytes = bytes as u64;
        let projected = budget
            .stats
            .produced
            .checked_add(budget.stats.pending)
            .and_then(|n| n.checked_add(bytes));
        if budget.stats.refused
            || bytes > budget.limits.artifact
            || projected.is_none_or(|n| n > budget.limits.request)
        {
            budget.stats.refused = true;
            state.stats.rejections += 1;
            return Err(ResourceError::Budget(
                "request output reservation exceeds artifact/request allowance".into(),
            ));
        }
        budget.stats.pending += bytes;
        budget.stats.peak_reserved = budget.stats.peak_reserved.max(projected.unwrap());
        Ok(OutputPermit {
            resources: Some(self.clone()),
            bytes,
        })
    }
    /// External compilers have no exact size preflight. Reserve the remaining
    /// request allowance for their whole output batch before starting them;
    /// the host must observe each named artifact and finish after tree reap.
    /// This is admission plus sampled enforcement, not a filesystem quota.
    pub fn reserve_external_output(&self) -> Result<ExternalOutputPermit, ResourceError> {
        let mut state = self.0.lock().expect("resource account");
        let (resources, bytes, artifact) = if let Some(budget) = state.production.as_mut() {
            let remaining = budget
                .limits
                .request
                .saturating_sub(budget.stats.produced.saturating_add(budget.stats.pending));
            if budget.stats.refused || remaining == 0 || budget.limits.artifact == 0 {
                budget.stats.refused = true;
                state.stats.rejections += 1;
                return Err(ResourceError::Budget("no external output allowance".into()));
            }
            budget.stats.pending += remaining;
            budget.stats.peak_reserved = budget.limits.request;
            (Some(self.clone()), remaining, budget.limits.artifact)
        } else {
            let limits = OutputLimits::default();
            (None, limits.request, limits.artifact)
        };
        Ok(ExternalOutputPermit {
            permit: Some(OutputPermit { resources, bytes }),
            artifact,
            observed: 0,
        })
    }
}
/// One sequential external producer batch. Completed and failed producers
/// both spend their observed output; only unused pending capacity is refunded.
pub struct ExternalOutputPermit {
    permit: Option<OutputPermit>,
    artifact: u64,
    observed: u64,
}
impl ExternalOutputPermit {
    /// The host retains each artifact's high-water length across observations.
    pub fn observe(&mut self, total: u64, largest: u64) -> Result<(), ResourceError> {
        self.observed = self.observed.max(total);
        let permit = self.permit.as_ref().expect("unfinished external output");
        if largest > self.artifact || self.observed > permit.bytes {
            if let Some(resources) = &permit.resources {
                let mut state = resources.0.lock().expect("resource account");
                state
                    .production
                    .as_mut()
                    .expect("installed output budget")
                    .stats
                    .refused = true;
                state.stats.rejections += 1;
            }
            return Err(ResourceError::Budget(
                "external artifact/request output allowance exceeded".into(),
            ));
        }
        if permit
            .resources
            .as_ref()
            .is_some_and(|r| r.output_stats().unwrap().refused)
        {
            return Err(ResourceError::Budget(
                "request output already refused".into(),
            ));
        }
        Ok(())
    }
    pub fn finish(mut self) -> Result<(), ResourceError> {
        self.permit
            .take()
            .unwrap()
            .complete_inner(self.observed as usize, false)
    }
}
impl Drop for ExternalOutputPermit {
    fn drop(&mut self) {
        if let Some(permit) = self.permit.take() {
            let _ = permit.complete_inner(self.observed as usize, false);
        }
    }
}
pub struct OutputPermit {
    resources: Option<Resources>,
    bytes: u64,
}
impl OutputPermit {
    pub fn complete(self, actual: usize) -> Result<(), ResourceError> {
        self.complete_inner(actual, true)
    }
    fn complete_inner(mut self, actual: usize, per_artifact: bool) -> Result<(), ResourceError> {
        let actual = actual as u64;
        if let Some(resources) = self.resources.take() {
            let mut state = resources.0.lock().expect("resource account");
            let budget = state.production.as_mut().expect("installed output budget");
            budget.stats.pending -= self.bytes;
            budget.stats.produced = budget.stats.produced.saturating_add(actual);
            if budget.stats.refused
                || actual > self.bytes
                || (per_artifact && actual > budget.limits.artifact)
                || budget.stats.produced > budget.limits.request
            {
                budget.stats.refused = true;
                state.stats.rejections += 1;
                return Err(ResourceError::Budget(
                    "producer exceeded its request output reservation".into(),
                ));
            }
        } else if actual > self.bytes {
            return Err(ResourceError::Budget(
                "producer exceeded its output reservation".into(),
            ));
        }
        Ok(())
    }
}
impl Drop for OutputPermit {
    fn drop(&mut self) {
        if let Some(resources) = self.resources.take() {
            let mut state = resources.0.lock().expect("resource account");
            state
                .production
                .as_mut()
                .expect("installed output budget")
                .stats
                .pending -= self.bytes;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_batch_shares_remaining_allowance_and_spends_failed_or_deleted_outputs() {
        let resources = Resources::default();
        resources
            .enable_output_budget(OutputLimits {
                artifact: 8,
                request: 20,
            })
            .unwrap();
        resources.reserve_output(4).unwrap().complete(4).unwrap();
        let mut batch = resources.reserve_external_output().unwrap();
        assert_eq!(resources.output_stats().unwrap().pending, 16);
        batch.observe(12, 7).unwrap(); // Two artifacts; their sum may exceed 8.
        batch.observe(0, 0).unwrap(); // Removal never refunds produced bytes.
        batch.finish().unwrap();
        assert_eq!(resources.output_stats().unwrap().produced, 16);
        assert_eq!(resources.output_stats().unwrap().pending, 0);
        let mut failed = resources.reserve_external_output().unwrap();
        assert!(failed.observe(5, 5).is_err());
        drop(failed); // The failure path spends too, without an explicit finish.
        let stats = resources.output_stats().unwrap();
        assert_eq!(stats.produced, 21);
        assert_eq!(stats.pending, 0);
        assert!(stats.refused);
        assert!(resources.reserve_external_output().is_err());

        let resources = Resources::default();
        resources
            .enable_output_budget(OutputLimits {
                artifact: 8,
                request: 20,
            })
            .unwrap();
        let mut batch = resources.reserve_external_output().unwrap();
        assert!(
            batch.observe(9, 9).is_err(),
            "artifact cap bypassed by the batch reservation"
        );
        assert!(batch.finish().is_err());
        assert_eq!(resources.output_stats().unwrap().produced, 9);
        assert_eq!(resources.output_stats().unwrap().pending, 0);
    }
    #[test]
    fn request_output_reserves_pending_keeps_discarded_spend_and_never_resets_a_refusal() {
        let resources = Resources::default();
        resources
            .enable_output_budget(OutputLimits {
                artifact: 8,
                request: 12,
            })
            .unwrap();
        let cancelled = resources.reserve_output(8).unwrap();
        assert_eq!(resources.output_stats().unwrap().pending, 8);
        drop(cancelled);
        assert_eq!(resources.output_stats().unwrap().pending, 0);
        resources.reserve_output(8).unwrap().complete(7).unwrap();
        assert_eq!(resources.output_stats().unwrap().produced, 7);
        let second = resources.reserve_output(5).unwrap();
        assert_eq!(resources.output_stats().unwrap().peak_reserved, 12);
        second.complete(5).unwrap();
        assert!(resources.reserve_output(1).is_err());
        assert!(
            resources.reserve_output(0).is_err(),
            "a refused request resumed production"
        );
        assert!(resources
            .enable_output_budget(OutputLimits::default())
            .is_err());
        assert_eq!(resources.output_stats().unwrap().produced, 12);
        assert_eq!(resources.output_stats().unwrap().pending, 0);
        let next = Resources::default();
        next.enable_output_budget(OutputLimits {
            artifact: 8,
            request: 12,
        })
        .unwrap();
        next.reserve_output(8).unwrap().complete(8).unwrap();
        assert!(!next.output_stats().unwrap().refused);
    }
    #[test]
    fn output_limits_cannot_expand_and_pending_outputs_share_the_same_request() {
        let resources = Resources::default();
        assert!(resources
            .enable_output_budget(OutputLimits {
                request: (128 << 20) + 1,
                ..Default::default()
            })
            .is_err());
        resources
            .enable_output_budget(OutputLimits {
                artifact: 8,
                request: 12,
            })
            .unwrap();
        let held = resources.reserve_output(8).unwrap();
        assert!(resources.clone().reserve_output(5).is_err());
        assert!(
            held.complete(8).is_err(),
            "a pending output was accepted after request refusal"
        );
        assert_eq!(resources.output_stats().unwrap().produced, 8);
        assert_eq!(resources.output_stats().unwrap().pending, 0);
        assert!(resources.reserve_output(1).is_err());
        let bad = Resources::default();
        bad.enable_output_budget(OutputLimits {
            artifact: 8,
            request: 12,
        })
        .unwrap();
        assert!(bad.reserve_output(8).unwrap().complete(9).is_err());
        assert_eq!(bad.output_stats().unwrap().produced, 9);
        assert!(bad.output_stats().unwrap().refused);
    }
}
