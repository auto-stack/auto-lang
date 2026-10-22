//! PLAN-817 CTE probe crate: the executable side of the frozen contract.
//! `av_driver` runs real AV calls through the merged 798 services,
//! `store` persists frozen envelopes in the 783 ManagedStore, `driver`
//! orchestrates one query step end-to-end, `deck` is the strict
//! `cte817/1` atom-data deck parser and `worker` speaks `cteworker/1`.

pub mod av_driver;
pub mod deck;
pub mod driver;
pub mod observe;
pub mod store;
pub mod worker;

pub use deck::{parse_deck, Deck};
pub use driver::{CteDriver, CteStepOutcome};

/// FNV-1a 64 (same family as the 798 driver's source snapshots — a local
/// revision binding hash, not a security hash).
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

pub fn fnv1a64_hex(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a64(bytes))
}
