//! Live ownership and pre-allocation reservations for the source composition.
//! The original store owns disk quota/GC. These are in-process byte accounts;
//! native MIR/staging/I/O shares and the ABI index cap remain simultaneous
//! constraints on the 48MiB managed total. Upstream syntax work keeps its own
//! FrontBudget. No serialized value can grant itself a larger allowance.
use std::{
    any::Any,
    sync::{Arc, Mutex, Weak},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Charge {
    pub resident: usize,
    pub mir: usize,
    pub staging: usize,
    pub io: usize,
    pub index: usize,
}
impl Charge {
    pub fn resident(bytes: usize) -> Self {
        Self {
            resident: bytes,
            ..Self::default()
        }
    }
    pub fn staging(bytes: usize) -> Self {
        Self {
            staging: bytes,
            ..Self::default()
        }
    }
    pub fn io(bytes: usize) -> Self {
        Self {
            io: bytes,
            ..Self::default()
        }
    }
    pub fn index(bytes: usize) -> Self {
        Self {
            index: bytes,
            ..Self::default()
        }
    }
    pub fn mir(bytes: usize) -> Self {
        Self {
            resident: bytes,
            mir: bytes,
            ..Self::default()
        }
    }
    pub fn total(self) -> Option<usize> {
        self.resident
            .checked_add(self.staging)?
            .checked_add(self.io)?
            .checked_add(self.index)
    }
    fn array(self) -> [usize; 5] {
        [self.resident, self.mir, self.staging, self.io, self.index]
    }
    fn from_array(a: [usize; 5]) -> Self {
        Self {
            resident: a[0],
            mir: a[1],
            staging: a[2],
            io: a[3],
            index: a[4],
        }
    }
    fn plus(self, other: Self) -> Option<Self> {
        let a = self.array();
        let b = other.array();
        let mut sum = [0; 5];
        for i in 0..5 {
            sum[i] = a[i].checked_add(b[i])?;
        }
        Some(Self::from_array(sum))
    }
    fn minus(self, other: Self) -> Self {
        let a = self.array();
        let b = other.array();
        Self::from_array(std::array::from_fn(|i| {
            a[i].checked_sub(b[i]).expect("balanced resource ownership")
        }))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub total: usize,
    pub mir: usize,
    pub staging: usize,
    pub io: usize,
    pub index: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            total: 48 << 20,
            mir: 16 << 20,
            staging: 24 << 20,
            io: 8 << 20,
            index: 8 << 20,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceError {
    Budget(String),
    Decode(String),
}
impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Budget(s) => write!(f, "foreign.resource: {s}"),
            Self::Decode(s) => write!(f, "foreign.wire.decode: {s}"),
        }
    }
}
impl std::error::Error for ResourceError {}
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub current: Charge,
    pub peak: Charge,
    pub peak_total: usize,
    pub rejections: u64,
    pub live_objects: usize,
}
struct Retained {
    object: Weak<dyn Any + Send + Sync>,
    charge: Charge,
}
struct State {
    limits: Limits,
    stats: Stats,
    retained: Vec<Retained>,
}
impl State {
    fn collect(&mut self) {
        let mut freed = Charge::default();
        self.retained.retain(|r| {
            if r.object.strong_count() > 0 {
                true
            } else {
                freed = freed.plus(r.charge).expect("bounded retained charges");
                false
            }
        });
        self.stats.current = self.stats.current.minus(freed);
        self.stats.live_objects = self.retained.len();
    }
    fn add(&mut self, charge: Charge) -> Result<(), ResourceError> {
        let next = self.stats.current.plus(charge);
        let admissible = charge.mir <= charge.resident
            && next.is_some_and(|n| {
                n.total().is_some_and(|total| total <= self.limits.total)
                    && n.mir <= self.limits.mir
                    && n.staging <= self.limits.staging
                    && n.io <= self.limits.io
                    && n.index <= self.limits.index
            });
        if !admissible {
            self.stats.rejections += 1;
            return Err(ResourceError::Budget(format!(
                "reservation {:?} over current {:?} and limits {:?}",
                charge, self.stats.current, self.limits
            )));
        }
        let next = next.expect("admissible charge");
        self.stats.current = next;
        let old = self.stats.peak.array();
        let new = next.array();
        self.stats.peak = Charge::from_array(std::array::from_fn(|i| old[i].max(new[i])));
        self.stats.peak_total = self
            .stats
            .peak_total
            .max(next.total().expect("admitted total"));
        Ok(())
    }
    fn retain(
        &mut self,
        object: Weak<dyn Any + Send + Sync>,
        charge: Charge,
    ) -> Result<(), ResourceError> {
        if self.retained.len() == self.retained.capacity() {
            // Account the index expansion BEFORE allocating it. Fixed-size
            // records have no per-entry strings or unbounded hidden payloads.
            let count = 64usize;
            let previous = self.retained.capacity() * std::mem::size_of::<Retained>();
            let bytes = previous + count * std::mem::size_of::<Retained>();
            self.add(Charge::index(bytes))?;
            if let Err(e) = self.retained.try_reserve_exact(count) {
                self.stats.current = self.stats.current.minus(Charge::index(bytes));
                return Err(ResourceError::Budget(format!(
                    "ownership index allocation: {e}"
                )));
            }
            self.stats.current = self.stats.current.minus(Charge::index(previous));
        }
        self.retained.push(Retained { object, charge });
        self.stats.live_objects = self.retained.len();
        Ok(())
    }
}
#[derive(Clone)]
pub struct Resources(Arc<Mutex<State>>);
impl std::fmt::Debug for Resources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.stats().fmt(f)
    }
}
impl Default for Resources {
    fn default() -> Self {
        Self::limited(Limits::default()).expect("default limits")
    }
}
impl Resources {
    /// A host/testing seam may only tighten the frozen limits.
    pub fn limited(limits: Limits) -> Result<Self, ResourceError> {
        let max = Limits::default();
        if limits.total > max.total
            || limits.mir > max.mir
            || limits.staging > max.staging
            || limits.io > max.io
            || limits.index > max.index
        {
            return Err(ResourceError::Budget(
                "host cannot increase frozen limits".into(),
            ));
        }
        let mut state = State {
            limits,
            stats: Stats::default(),
            retained: Vec::new(),
        };
        state.add(Charge::index(
            std::mem::size_of::<State>() + 2 * std::mem::size_of::<usize>(),
        ))?;
        Ok(Self(Arc::new(Mutex::new(state))))
    }
    pub fn reserve(&self, charge: Charge) -> Result<Reservation, ResourceError> {
        let mut state = self.0.lock().expect("resource account");
        state.collect();
        state.add(charge)?;
        Ok(Reservation {
            resources: self.clone(),
            charge: Some(charge),
        })
    }
    /// Provider-only interface production uses the existing shared account,
    /// before the neutral contract allocates its new binding page or digest.
    pub fn rebind_foreign(
        &self,
        foreign: &ace_foreign_contract::manual::AdmittedForeign,
        host: &ace_foreign_contract::manual::HostAuthority,
    ) -> Result<ace_foreign_contract::manual::AdmittedForeign, String> {
        let mut allocation = None;
        let next = foreign.rebind_reserved(host, |bytes, metadata| {
            let page = self
                .reserve(Charge::resident(bytes))
                .map_err(|e| format!("foreign.resource.binding: {e}"))?;
            let metadata = self
                .reserve(Charge::index(metadata))
                .map_err(|e| format!("foreign.resource.binding: {e}"))?;
            allocation = Some((page, metadata));
            Ok(())
        })?;
        let (page, metadata) = allocation.expect("successful binding encoder reserved its output");
        page.retain(&next.pages().binding)
            .map_err(|e| e.to_string())?;
        metadata
            .retain(next.binding_metadata_identity())
            .map_err(|e| e.to_string())?;
        Ok(next)
    }
    pub fn stats(&self) -> Stats {
        let mut state = self.0.lock().expect("resource account");
        state.collect();
        state.stats
    }
    /// Consume an existing caller-owned payload without cloning its bytes.
    /// Its capacity and Arc/Vec header are admitted before allocating the
    /// shared owner. This does not reserve the caller's earlier Vec producer.
    pub fn own_payload(&self, bytes: Vec<u8>) -> Result<Arc<Vec<u8>>, ResourceError> {
        let allocation = self.reserve(Charge::resident(
            bytes.capacity() + std::mem::size_of::<Vec<u8>>() + 2 * std::mem::size_of::<usize>(),
        ))?;
        let bytes = Arc::new(bytes);
        allocation.retain(&bytes)?;
        Ok(bytes)
    }
    pub fn available_io(&self) -> usize {
        let mut state = self.0.lock().expect("resource account");
        state.collect();
        state.limits.io.saturating_sub(state.stats.current.io).min(
            state
                .limits
                .total
                .saturating_sub(state.stats.current.total().unwrap_or(usize::MAX)),
        )
    }
    /// Reserve the remaining MIR partition atomically with checking the shared
    /// live total. A bounded decoder consumes this allowance, rather than
    /// independently assuming that the entire original 16MiB is free.
    pub fn reserve_remaining_mir(&self) -> Result<(usize, Reservation), ResourceError> {
        let mut state = self.0.lock().expect("resource account");
        state.collect();
        let available = state
            .limits
            .mir
            .saturating_sub(state.stats.current.mir)
            .min(
                state
                    .limits
                    .total
                    .saturating_sub(state.stats.current.total().unwrap_or(usize::MAX)),
            );
        if available == 0 {
            state.stats.rejections += 1;
            return Err(ResourceError::Budget(
                "no remaining MIR decode allowance".into(),
            ));
        }
        let charge = Charge::mir(available);
        state.add(charge)?;
        Ok((
            available,
            Reservation {
                resources: self.clone(),
                charge: Some(charge),
            },
        ))
    }
    /// Original ManagedStore read, with room for its framed read and returned
    /// payload BEFORE dispatch. The returned immutable page inherits the same
    /// reservation; there is no uncharged adoption gap or extra payload copy.
    pub fn load_payload(
        &self,
        maximum: usize,
        expected: ace_artifacts::codec::ChunkKind,
        read: impl FnOnce(
            u64,
        ) -> Result<
            Option<ace_artifacts::store::LoadedArtifact>,
            ace_artifacts::store::StoreError,
        >,
    ) -> Result<Option<Arc<Vec<u8>>>, ResourceError> {
        let header = ace_artifacts::codec::CHUNK_HEADER_LEN;
        let available = self.available_io();
        let bound = available.saturating_sub(256) / 2;
        let bound = bound.min(
            maximum
                .checked_add(header)
                .ok_or_else(|| ResourceError::Budget("read size overflow".into()))?,
        );
        if bound < header {
            self.0.lock().expect("resource account").stats.rejections += 1;
            return Err(ResourceError::Budget(
                "read buffers do not fit I/O allowance".into(),
            ));
        }
        let metadata = self.reserve(Charge::io(256))?;
        let mut payload_charge = self.reserve(Charge::io(bound * 2))?;
        let Some(loaded) = read(bound as u64).map_err(|e| ResourceError::Decode(e.to_string()))?
        else {
            return Ok(None);
        };
        if loaded.kind != expected {
            return Err(ResourceError::Decode(
                "artifact carrier kind mismatch".into(),
            ));
        }
        let ace_artifacts::store::LoadedArtifact {
            artifact_id,
            chunk_id,
            kind: _,
            payload,
        } = loaded;
        drop(artifact_id);
        drop(chunk_id);
        drop(metadata);
        payload_charge.resize(Charge::resident(
            payload.capacity() + std::mem::size_of::<Vec<u8>>() + 2 * std::mem::size_of::<usize>(),
        ))?;
        let bytes = Arc::new(payload);
        payload_charge.retain(&bytes)?;
        Ok(Some(bytes))
    }
    pub fn decode_json<T: serde::de::DeserializeOwned>(
        &self,
        bytes: &[u8],
    ) -> Result<(T, Reservation), ResourceError> {
        self.decode_json_profile(bytes, ace_foreign_contract::codec::source_json_allocation)
    }
    pub fn decode_foreign_request<T: serde::de::DeserializeOwned>(
        &self,
        bytes: &[u8],
    ) -> Result<(T, Reservation), ResourceError> {
        self.decode_json_profile(
            bytes,
            ace_foreign_contract::codec::foreign_request_json_allocation,
        )
    }
    /// Finite publication/identity records contain metadata, not source text
    /// or byte-array payloads. Retain the returned index ticket with the model.
    pub fn decode_metadata_json<T: serde::de::DeserializeOwned>(
        &self,
        bytes: &[u8],
    ) -> Result<(T, Reservation), ResourceError> {
        self.decode_json_profile(bytes, ace_foreign_contract::codec::metadata_json_allocation)
    }
    fn decode_json_profile<T: serde::de::DeserializeOwned>(
        &self,
        bytes: &[u8],
        allocation: fn(
            &[u8],
            usize,
        ) -> Result<
            ace_foreign_contract::codec::JsonAllocation,
            ace_foreign_contract::codec::AllocationError,
        >,
    ) -> Result<(T, Reservation), ResourceError> {
        let _scratch =
            self.reserve(Charge::io(bytes.len().checked_mul(2).ok_or_else(|| {
                ResourceError::Budget("JSON scratch size overflow".into())
            })?))?;
        let bound = allocation(bytes, Limits::default().total).map_err(|e| match e {
            ace_foreign_contract::codec::AllocationError::Budget(s) => {
                self.0.lock().expect("resource account").stats.rejections += 1;
                ResourceError::Budget(s)
            }
            ace_foreign_contract::codec::AllocationError::Decode(s) => ResourceError::Decode(s),
        })?;
        let reservation = self.reserve(Charge {
            index: bound.metadata,
            staging: bound
                .bytes
                .checked_add(bound.source)
                .ok_or_else(|| ResourceError::Budget("JSON allocation overflow".into()))?,
            ..Charge::default()
        })?;
        let decoded =
            serde_json::from_slice(bytes).map_err(|e| ResourceError::Decode(e.to_string()))?;
        Ok((decoded, reservation))
    }
    /// Hold both parser scratch and the finite foreign metadata representation
    /// before entering its strict typed codec. Caller retains the typed share
    /// for the admitted environment's complete shared lifetime.
    pub fn reserve_foreign_decode(&self, bytes: &[u8]) -> Result<Reservation, ResourceError> {
        let io = bytes
            .len()
            .checked_mul(2)
            .ok_or_else(|| ResourceError::Budget("foreign JSON scratch overflow".into()))?;
        let mut reservation = self.reserve(Charge::io(io))?;
        let bound =
            ace_foreign_contract::codec::metadata_json_allocation(bytes, Limits::default().total)
                .map_err(|e| match e {
                ace_foreign_contract::codec::AllocationError::Budget(s) => ResourceError::Budget(s),
                ace_foreign_contract::codec::AllocationError::Decode(s) => ResourceError::Decode(s),
            })?;
        reservation.resize(Charge {
            io,
            index: bound.metadata,
            ..Charge::default()
        })?;
        Ok(reservation)
    }
    /// Charge an already-owned immutable input when the consumer adopts it.
    /// Its weak identity preserves the charge across external Arc pins and
    /// snapshots, and it is charged once per account even through aliases.
    pub fn adopt<T: Any + Send + Sync>(
        &self,
        object: &Arc<T>,
        charge: Charge,
    ) -> Result<(), ResourceError> {
        let erased: Arc<dyn Any + Send + Sync> = object.clone();
        let weak = Arc::downgrade(&erased);
        let mut state = self.0.lock().expect("resource account");
        state.collect();
        if let Some(old) = state
            .retained
            .iter()
            .find(|r| Weak::ptr_eq(&r.object, &weak))
        {
            if old.charge != charge {
                return Err(ResourceError::Budget(
                    "inconsistent immutable object charge".into(),
                ));
            }
            return Ok(());
        }
        state.add(charge)?;
        if let Err(e) = state.retain(weak, charge) {
            state.stats.current = state.stats.current.minus(charge);
            return Err(e);
        }
        Ok(())
    }
}
pub struct Reservation {
    resources: Resources,
    charge: Option<Charge>,
}
impl Reservation {
    /// Join two already admitted allocations in the SAME live account before
    /// moving their payloads into one immutable owner. No release/reacquire gap.
    pub fn merge(mut self, mut other: Self) -> Result<Self, ResourceError> {
        if !Arc::ptr_eq(&self.resources.0, &other.resources.0) {
            return Err(ResourceError::Budget(
                "cannot combine different resource accounts".into(),
            ));
        }
        let combined = self
            .charge
            .expect("live reservation")
            .plus(other.charge.expect("live reservation"))
            .ok_or_else(|| ResourceError::Budget("reservation merge overflow".into()))?;
        self.charge = Some(combined);
        other.charge = None;
        Ok(self)
    }
    pub fn retain_foreign_metadata<T: Any + Send + Sync>(
        mut self,
        object: &Arc<T>,
    ) -> Result<(), ResourceError> {
        let mut charge = self.charge.expect("live reservation");
        charge.io = 0;
        self.resize(charge)?;
        self.retain(object)
    }
    pub fn resize(&mut self, next: Charge) -> Result<(), ResourceError> {
        let old = self.charge.expect("live reservation");
        let mut state = self.resources.0.lock().expect("resource account");
        let current = state.stats.current;
        state.stats.current = current.minus(old);
        if let Err(e) = state.add(next) {
            state.stats.current = current;
            return Err(e);
        }
        self.charge = Some(next);
        Ok(())
    }
    /// Transfer a PRE-allocation reservation to the immutable object. No
    /// hidden strong reference is kept; the account notices its last owner.
    pub fn retain<T: Any + Send + Sync>(mut self, object: &Arc<T>) -> Result<(), ResourceError> {
        let erased: Arc<dyn Any + Send + Sync> = object.clone();
        let charge = self.charge.expect("live reservation");
        let retained = {
            self.resources
                .0
                .lock()
                .expect("resource account")
                .retain(Arc::downgrade(&erased), charge)
        };
        retained?;
        self.charge = None;
        Ok(())
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if let Some(charge) = self.charge.take() {
            let mut state = self.resources.0.lock().expect("resource account");
            state.stats.current = state.stats.current.minus(charge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foreign_metadata_profile_reserves_structured_facts_and_releases_scratch_after_admission() {
        let resources = Resources::default();
        let bytes = br#"{"facts":{"source":{"Typed":{"id":"manual"}},"functions":[{"form":{"Pure":{"params":["I32","Bool"],"ret":"I32"}}}]}}"#;
        // Source-session facts are BYTE pages; foreign-interface facts are
        // STRUCTURED objects. Accidentally sharing the former profile rejects
        // this valid wire before its typed codec can enter.
        assert!(ace_foreign_contract::codec::source_json_allocation(bytes, 1 << 20).is_err());
        let allocation = resources.reserve_foreign_decode(bytes).unwrap();
        assert!(resources.stats().current.index > 0);
        assert_eq!(resources.stats().current.io, bytes.len() * 2);
        let identity = Arc::new(());
        allocation.retain_foreign_metadata(&identity).unwrap();
        assert_eq!(resources.stats().current.io, 0);
        let pinned = resources.stats().current.index;
        assert!(pinned > 0);
        let pin = identity.clone();
        drop(identity);
        assert_eq!(resources.stats().current.index, pinned);
        drop(pin);
        assert!(resources.stats().current.index < pinned);
    }
    #[test]
    fn managed_read_reserves_before_dispatch_and_keeps_the_returned_page_charged() {
        use ace_artifacts::{codec::ChunkKind, store::LoadedArtifact};
        let r = Resources::limited(Limits {
            io: 1024,
            ..Limits::default()
        })
        .unwrap();
        let bytes = r
            .load_payload(64 << 20, ChunkKind::SnapshotRecord, |bound| {
                assert!(bound <= (1024 - 256) / 2);
                Ok(Some(LoadedArtifact {
                    artifact_id: String::new(),
                    chunk_id: String::new(),
                    kind: ChunkKind::SnapshotRecord,
                    payload: vec![0; 100],
                }))
            })
            .unwrap()
            .unwrap();
        assert_eq!(r.stats().current.io, 0);
        assert_eq!(r.stats().current.resident, 140);
        drop(bytes);
        assert_eq!(r.stats().current.resident, 0);
        assert!(matches!(
            r.load_payload(1024, ChunkKind::SnapshotRecord, |_| Ok(Some(
                LoadedArtifact {
                    artifact_id: String::new(),
                    chunk_id: String::new(),
                    kind: ChunkKind::Object,
                    payload: vec![0; 100],
                }
            ))),
            Err(ResourceError::Decode(_))
        ));
        assert_eq!(r.stats().current.io, 0);
        let tight = Resources::limited(Limits {
            io: 128,
            ..Limits::default()
        })
        .unwrap();
        assert!(matches!(
            tight.load_payload(1024, ChunkKind::SnapshotRecord, |_| panic!(
                "unreserved store read"
            )),
            Err(ResourceError::Budget(_))
        ));
    }
    #[test]
    fn typed_decode_is_not_entered_when_preflight_exceeds_the_shared_index() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static ENTERED: AtomicUsize = AtomicUsize::new(0);
        struct Decoded;
        impl<'de> serde::Deserialize<'de> for Decoded {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                ENTERED.fetch_add(1, Ordering::Relaxed);
                let _ = <serde::de::IgnoredAny as serde::Deserialize>::deserialize(d)?;
                Ok(Self)
            }
        }
        let r = Resources::limited(Limits {
            index: 1024,
            ..Limits::default()
        })
        .unwrap();
        let payload = serde_json::to_vec(
            &serde_json::json!({"rows":(0..100).map(|i| (i,"x")).collect::<Vec<_>>()}),
        )
        .unwrap();
        assert!(r.decode_json::<Decoded>(&payload).is_err());
        assert_eq!(ENTERED.load(Ordering::Relaxed), 0);
        assert!(r.decode_foreign_request::<Decoded>(&payload).is_err());
        assert_eq!(ENTERED.load(Ordering::Relaxed), 0);
        assert!(r.decode_metadata_json::<Decoded>(&payload).is_err());
        assert_eq!(ENTERED.load(Ordering::Relaxed), 0);
        let (_value, guard) = r.decode_json::<Decoded>(b"{}").unwrap();
        assert_eq!(ENTERED.load(Ordering::Relaxed), 1);
        drop(guard);
        let (_value, guard) = r.decode_foreign_request::<Decoded>(b"{}").unwrap();
        assert_eq!(ENTERED.load(Ordering::Relaxed), 2);
        drop(guard);
        let (_value, guard) = r.decode_metadata_json::<Decoded>(b"{}").unwrap();
        assert_eq!(ENTERED.load(Ordering::Relaxed), 3);
        drop(guard);
        assert!(matches!(
            r.decode_json::<Decoded>(b"{"),
            Err(ResourceError::Decode(_))
        ));
    }
    #[test]
    fn joined_output_tickets_transfer_without_reacquiring_or_mixing_accounts() {
        let r = Resources::default();
        let a = r.reserve(Charge::staging(4096)).unwrap();
        let b = r.reserve(Charge::io(2048)).unwrap();
        let before = r.stats().current;
        let mut joined = a.merge(b).unwrap();
        assert_eq!(r.stats().current, before);
        joined.resize(Charge::resident(6144)).unwrap();
        let payload = Arc::new(vec![0u8; 6144]);
        joined.retain(&payload).unwrap();
        assert_eq!(r.stats().current.resident, 6144);
        assert_eq!(r.stats().current.staging, 0);
        assert_eq!(r.stats().current.io, 0);
        let pin = payload.clone();
        drop(payload);
        assert_eq!(r.stats().current.resident, 6144);
        drop(pin);
        assert_eq!(r.stats().current.resident, 0);
        let other = Resources::default();
        let a = r.reserve(Charge::staging(100)).unwrap();
        let b = other.reserve(Charge::staging(200)).unwrap();
        assert!(a.merge(b).is_err());
        assert_eq!(r.stats().current.staging, 0);
        assert_eq!(other.stats().current.staging, 0);
    }

    #[test]
    fn mir_decode_reservation_obeys_both_remaining_partition_and_live_total() {
        let r = Resources::limited(Limits {
            total: 32 << 10,
            mir: 24 << 10,
            ..Limits::default()
        })
        .unwrap();
        let baseline = r.stats().current;
        let old_mir = r.reserve(Charge::mir(4 << 10)).unwrap();
        let (limit, decoding) = r.reserve_remaining_mir().unwrap();
        assert_eq!(limit, 20 << 10);
        assert!(r.reserve_remaining_mir().is_err());
        drop(decoding);
        let other = r.reserve(Charge::resident(12 << 10)).unwrap();
        let (limit, decoding) = r.reserve_remaining_mir().unwrap();
        assert_eq!(limit, (16 << 10) - baseline.index);
        assert!(r.reserve(Charge::io(1)).is_err());
        drop(decoding);
        drop(other);
        drop(old_mir);
        assert_eq!(r.stats().current, baseline);
    }

    #[test]
    fn reservations_share_caps_before_allocation_and_drop_on_rejection() {
        let r = Resources::default();
        let baseline = r.stats().current;
        let resident = r.reserve(Charge::resident(20 << 20)).unwrap();
        let staging = r.reserve(Charge::staging(24 << 20)).unwrap();
        assert!(r.reserve(Charge::io(5 << 20)).is_err());
        assert_eq!(r.stats().current.total(), Some((44 << 20) + baseline.index));
        assert!(r.reserve(Charge::mir(17 << 20)).is_err());
        drop(staging);
        drop(resident);
        assert_eq!(r.stats().current, baseline);
        assert!(Resources::limited(Limits {
            total: 49 << 20,
            ..Limits::default()
        })
        .is_err());
    }
    #[test]
    fn external_pins_and_aliases_keep_exact_immutable_ownership_alive() {
        let r = Resources::default();
        let reservation = r.reserve(Charge::resident(4096)).unwrap();
        let bytes = Arc::new(vec![0u8; 4096]);
        reservation.retain(&bytes).unwrap();
        let pinned = bytes.clone();
        r.adopt(&pinned, Charge::resident(4096)).unwrap();
        assert_eq!(r.stats().current.resident, 4096);
        drop(bytes);
        assert_eq!(r.stats().current.resident, 4096);
        drop(pinned);
        assert_eq!(r.stats().current.resident, 0);
        assert_eq!(r.stats().live_objects, 0);
        assert!(r.stats().current.index > 0); // index backing capacity is real
    }
}
