//! In-process memory accounting for the semantic probe (PLAN-796
//! §5.4, contract-decision D10).
//!
//! Three ledgers with explicit byte budgets:
//! * **hot pages** — an LRU of decoded pages, ≤8 MiB;
//! * **scratch** — solver/index staging bytes, ≤8 MiB (RAII returned);
//! * **managed total** — hot + scratch, ≤16 MiB.
//!
//! Pressure pages are generated and published STREAMED: exactly one
//! page is materialized at a time, so ≥32 MiB of on-disk semantic pages
//! never implies ≥32 MiB of process memory. Budget violations are
//! controlled rejections (`BudgetRejected` — an infrastructure state,
//! never a type error).

use ac_artifact_probe::store::{ManagedStore, PutRequest, StoreError};
use ac_sema_contract::{SemaPage, SEMA_PAGE_SCHEMA_V1};

pub const HOT_BUDGET: usize = 8 * 1024 * 1024;
pub const SCRATCH_BUDGET: usize = 8 * 1024 * 1024;
pub const MANAGED_BUDGET: usize = 16 * 1024 * 1024;

/// Controlled budget rejection (infrastructure state).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BudgetRejected {
    pub ledger: &'static str,
    pub wanted: usize,
    pub budget: usize,
}

impl std::fmt::Display for BudgetRejected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "resource: {} ledger wants {} bytes over budget {}",
            self.ledger, self.wanted, self.budget
        )
    }
}

impl std::error::Error for BudgetRejected {}

/// LRU of decoded pages with byte accounting. Probe-scale O(n) scan is
/// fine (pages: hundreds, not millions).
pub struct HotPages {
    order: Vec<String>, // front = most recently used
    pages: std::collections::HashMap<String, (SemaPage, usize)>,
    budget: usize,
    bytes: usize,
    evictions: u64,
}

impl Default for HotPages {
    fn default() -> Self {
        Self::new(HOT_BUDGET)
    }
}

impl HotPages {
    pub fn new(budget: usize) -> Self {
        HotPages {
            order: Vec::new(),
            pages: std::collections::HashMap::new(),
            budget,
            bytes: 0,
            evictions: 0,
        }
    }

    fn page_bytes(p: &SemaPage) -> usize {
        std::mem::size_of::<SemaPage>() + format!("{:?}", p).len() // conservative live estimate
    }

    pub fn insert(&mut self, key: String, page: SemaPage) -> Result<(), BudgetRejected> {
        let sz = Self::page_bytes(&page);
        if sz > self.budget {
            return Err(BudgetRejected {
                ledger: "hot",
                wanted: sz,
                budget: self.budget,
            });
        }
        if let Some((_, old)) = self.pages.remove(&key) {
            self.bytes -= old;
            self.order.retain(|k| k != &key);
        }
        while self.bytes + sz > self.budget {
            // evict LRU (last in order)
            let Some(victim) = self.order.pop() else {
                break;
            };
            if let Some((_, (page, vsz))) = self.pages.remove_entry(&victim) {
                self.bytes -= vsz;
                drop(page);
                self.evictions += 1;
            }
        }
        self.bytes += sz;
        self.order.insert(0, key.clone());
        self.pages.insert(key, (page, sz));
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<&SemaPage> {
        if self.pages.contains_key(key) {
            self.order.retain(|k| k != key);
            self.order.insert(0, key.to_string());
            Some(&self.pages.get(key).unwrap().0)
        } else {
            None
        }
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn evictions(&self) -> u64 {
        self.evictions
    }
    pub fn len(&self) -> usize {
        self.pages.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pages.len() == 0
    }
}

/// RAII scratch allocation: bytes return to the ledger on drop.
pub struct ScratchGuard<'a> {
    meter: &'a mut SessionMeter,
    n: usize,
}

impl Drop for ScratchGuard<'_> {
    fn drop(&mut self) {
        self.meter.scratch_bytes = self.meter.scratch_bytes.saturating_sub(self.n);
    }
}

/// The session meter: hot pages + scratch, managed total ≤16 MiB.
pub struct SessionMeter {
    pub hot: HotPages,
    scratch_bytes: usize,
    scratch_peak: usize,
    rejected: u64,
}

impl Default for SessionMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionMeter {
    pub fn new() -> Self {
        SessionMeter {
            hot: HotPages::default(),
            scratch_bytes: 0,
            scratch_peak: 0,
            rejected: 0,
        }
    }

    /// Reserve `n` scratch bytes (solver staging, decode buffers).
    pub fn scratch(&mut self, n: usize) -> Result<ScratchGuard<'_>, BudgetRejected> {
        if self.hot.bytes() + self.scratch_bytes + n > MANAGED_BUDGET
            || self.scratch_bytes + n > SCRATCH_BUDGET
        {
            self.rejected += 1;
            return Err(BudgetRejected {
                ledger: "scratch",
                wanted: self.scratch_bytes + n,
                budget: SCRATCH_BUDGET,
            });
        }
        self.scratch_bytes += n;
        self.scratch_peak = self.scratch_peak.max(self.scratch_bytes);
        Ok(ScratchGuard { meter: self, n })
    }

    /// Record a NON-scratch budget rejection (the pre-encode page
    /// ledger) so the session receipt counts it like every other
    /// resource rejection (F-796-R8-01).
    pub fn record_rejection(
        &mut self,
        ledger: &'static str,
        wanted: usize,
        budget: usize,
    ) -> BudgetRejected {
        self.rejected += 1;
        BudgetRejected {
            ledger,
            wanted,
            budget,
        }
    }

    pub fn managed_bytes(&self) -> usize {
        self.hot.bytes() + self.scratch_bytes
    }
    pub fn scratch_peak(&self) -> usize {
        self.scratch_peak
    }
    pub fn rejections(&self) -> u64 {
        self.rejected
    }
}

/// Stream pages into the store ONE AT A TIME (§5.4 流式生成落盘): the
/// iterator yields pages lazily; each page is encoded and published
/// before the next is demanded. Returns (published_count, total_bytes).
pub fn publish_streamed<I>(
    store: &ManagedStore,
    pages: I,
    layer_key_of: impl Fn(&SemaPage) -> (&'static str, ac_artifact_probe::identity::QueryKey),
    project: Option<&str>,
    mut meter: Option<&mut SessionMeter>,
) -> Result<(usize, u64), StreamError>
where
    I: Iterator<Item = SemaPage>,
{
    let mut count = 0usize;
    let mut total = 0u64;
    for page in pages {
        let payload = ac_sema_contract::encode_page(&page)?;
        total += payload.len() as u64;
        // F-796-R6-05: the ONE materialized page is charged while it is
        // encoded + written — a streamed 32MiB corpus never implies
        // 32MiB of unaccounted process memory.
        let mut guard = None;
        if let Some(m) = meter.as_deref_mut() {
            guard = Some(m.scratch(payload.len())?);
        }
        let (layer, key) = layer_key_of(&page);
        let req = PutRequest {
            layer,
            key,
            kind: crate::store::SEMA_CARRIER,
            payload,
            edges: Vec::new(),
            project: project.map(|s| s.to_string()),
        };
        let art = store.put_artifact(req)?;
        drop(guard);
        if let Some(p) = project {
            // idempotent republication inserts no root row — register the
            // current project explicitly (same rule as every reuse path)
            store.register_root(p, &art, "soft")?;
        }
        count += 1;
    }
    Ok((count, total))
}

#[derive(Debug)]
pub enum StreamError {
    Store(StoreError),
    Codec(ac_sema_contract::CodecError),
    Budget(BudgetRejected),
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::Store(e) => write!(f, "store: {e}"),
            StreamError::Codec(e) => write!(f, "codec: {e}"),
            StreamError::Budget(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for StreamError {}

impl From<StoreError> for StreamError {
    fn from(e: StoreError) -> Self {
        StreamError::Store(e)
    }
}

impl From<ac_sema_contract::CodecError> for StreamError {
    fn from(e: ac_sema_contract::CodecError) -> Self {
        StreamError::Codec(e)
    }
}

impl From<BudgetRejected> for StreamError {
    fn from(e: BudgetRejected) -> Self {
        StreamError::Budget(e)
    }
}

/// The schema id re-export used by generators building pages.
pub const PAGE_SCHEMA: &str = SEMA_PAGE_SCHEMA_V1;
