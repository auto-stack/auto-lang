//! Source-import uses the original managed-store collector and ABI watermarks.
//! This policy supplies no storage, reachability or liveness implementation.
use ace_artifacts::store::{run_gc, GcConfig, GcReceipt, ManagedStore};

pub const HOT_HIGH: u64 = 8 << 20;
pub const HOT_LOW: u64 = 6 << 20;

#[derive(Clone, Copy, Debug)]
pub struct StorePolicy {
    high: u64,
    low: u64,
}
impl Default for StorePolicy {
    fn default() -> Self {
        Self {
            high: HOT_HIGH,
            low: HOT_LOW,
        }
    }
}
impl StorePolicy {
    /// Test/host pressure may only tighten the original ABI policy. A zero
    /// low watermark is the original collector's explicit reclaim-dead mode;
    /// live roots and leases still remain outside its deletion set.
    pub fn tightened(high: u64, low: u64) -> Result<Self, String> {
        if high == 0 || high > HOT_HIGH || low > HOT_LOW || low > high.saturating_mul(3) / 4 {
            return Err("foreign.resource.gc-policy".into());
        }
        Ok(Self { high, low })
    }
    pub(crate) fn worker() -> Self {
        let value = |key: &str| std::env::var(key).ok().and_then(|s| s.parse::<u64>().ok());
        let high = value("ABI_TEST_GC_HIGH_BYTES")
            .unwrap_or(HOT_HIGH)
            .clamp(1, HOT_HIGH);
        let low = value("ABI_TEST_GC_LOW_BYTES")
            .unwrap_or(HOT_LOW)
            .min(HOT_LOW)
            .min(high.saturating_mul(3) / 4);
        Self { high, low }
    }
    /// The returned original receipt includes bounded backoff when the live
    /// floor prevents convergence. Callers must not label that a reclaimed set.
    pub fn collect(&self, store: &ManagedStore) -> Result<GcReceipt, String> {
        run_gc(
            store,
            GcConfig {
                high: self.high,
                low: self.low,
            },
        )
        .map_err(|error| format!("foreign.store.gc: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_hot_policy_preserves_original_caps_and_only_tightens() {
        let policy = StorePolicy::default();
        assert_eq!((policy.high, policy.low), (8 << 20, 6 << 20));
        assert!(StorePolicy::tightened(HOT_HIGH + 1, HOT_LOW).is_err());
        assert!(StorePolicy::tightened(HOT_HIGH, HOT_LOW + 1).is_err());
        assert!(StorePolicy::tightened(0, 0).is_err());
        assert!(StorePolicy::tightened(1024, 769).is_err());
        assert!(StorePolicy::tightened(1024, 768).is_ok());
        assert!(StorePolicy::tightened(1024, 0).is_ok());
    }
}
