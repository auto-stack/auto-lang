//! Request observation: stage spans (wall ms), named counters, and the
//! managed-data charge ledger (T-05). The stage contract (every request
//! records prep/frontend/MIR/cache/codegen/link/run windows) is wired here
//! from the first case on; memory peaks live in `resources.rs`.

use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Default, Debug, Clone)]
pub struct Obs {
    pub spans: BTreeMap<String, u128>,
    pub counters: BTreeMap<String, u64>,
    /// Managed-data partitions charged over this request (T-05, §11).
    pub ledger: crate::resources::ManagedLedger,
}

impl Obs {
    pub fn span<T>(&mut self, name: &str, f: impl FnOnce(&mut Self) -> T) -> T {
        let started = Instant::now();
        let out = f(self);
        let e = self.spans.entry(name.to_string()).or_insert(0);
        *e += started.elapsed().as_millis();
        out
    }

    pub fn counter(&mut self, name: &str, n: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += n;
    }

    /// One line of the request trace (stable, human-greppable).
    pub fn render(&self) -> String {
        let spans: Vec<String> = self
            .spans
            .iter()
            .map(|(k, v)| format!("{k}={v}ms"))
            .collect();
        let counters: Vec<String> = self
            .counters
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!(
            "spans[{}] counters[{}]",
            spans.join(" "),
            counters.join(" ")
        )
    }
}
