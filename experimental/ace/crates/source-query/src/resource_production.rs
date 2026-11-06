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
}
pub struct OutputPermit {
    resources: Option<Resources>,
    bytes: u64,
}
impl OutputPermit {
    pub fn complete(mut self, actual: usize) -> Result<(), ResourceError> {
        let actual = actual as u64;
        if let Some(resources) = self.resources.take() {
            let mut state = resources.0.lock().expect("resource account");
            let budget = state.production.as_mut().expect("installed output budget");
            budget.stats.pending -= self.bytes;
            budget.stats.produced = budget.stats.produced.saturating_add(actual);
            if budget.stats.refused
                || actual > self.bytes
                || actual > budget.limits.artifact
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
