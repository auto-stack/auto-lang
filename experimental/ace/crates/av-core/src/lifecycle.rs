//! PLAN-919: the bounded per-owner lifecycle observation channel.
//!
//! Frozen contract (`av.lifecycle/1`):
//! - Pure data only: an event never holds a `Value`/handle, never extends
//!   a lifetime, never deep-clones payload and never changes drop order.
//! - One monotonic `seq` per execution covers every event, INCLUDING the
//!   buffer free facts surfaced through the arena free hook (so a free can
//!   be ordered against the slot disowns that preceded it) and the frees
//!   that fire after the execution ends (the host dropping the result).
//! - Hard cap [`MAX_EVENTS`]: when the log is full the executor stops
//!   appending and flags `overflowed`; the dispatch loop turns that flag
//!   into a named `av.resource.lifecycle-events` refusal. There is NO
//!   silent truncation — a full log is a red outcome, never a green one
//!   with missing events.
//! - The channel is OFF unless explicitly enabled; with the switch off no
//!   event is produced and the language result is identical.
//!
//! Owner identity model (frozen):
//! - `Str`/`List` owners carry the arena buffer id (ids come from the one
//!   per-execution arena counter, so `(domain, id)` is unique; the free
//!   facts share the same id space).
//! - `Record`/`Variant` owners carry no intrinsic id (`id == 0`); their
//!   LIFO order key is the minimum reachable buffer id (`u64::MAX` for a
//!   composite with no buffers), exactly the sweep ordering key the
//!   executor already uses.
//! - `ClosureEnv` owners carry the closure-env store handle.
//! - A slot-level own/disown pair is keyed by `(frame, slot, gen)` — the
//!   logical-owner responsibility unit. Transfers are `disown(moved-out)`
//!   followed by `own(received)` of the same `(kind, id)`.

/// The wire schema id of this channel.
pub const SCHEMA: &str = "av.lifecycle/1";

/// Hard per-execution event cap (bounded observation; `16_384 * ~48 B` is
/// far below the 4 MiB observation budget of PLAN-919 §5.2).
pub const MAX_EVENTS: usize = 16_384;

/// The not-slot-bound marker for closure-env store events.
pub const NO_SLOT: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    FrameEnter,
    FrameLeave,
    Own,
    Disown,
    Free,
    End,
}

impl Act {
    pub fn wire(&self) -> &'static str {
        match self {
            Act::FrameEnter => "frame-enter",
            Act::FrameLeave => "frame-leave",
            Act::Own => "own",
            Act::Disown => "disown",
            Act::Free => "free",
            Act::End => "end",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerKind {
    Str,
    List,
    Record,
    Variant,
    ClosureEnv,
}

impl OwnerKind {
    pub fn wire(&self) -> &'static str {
        match self {
            OwnerKind::Str => "str",
            OwnerKind::List => "list",
            OwnerKind::Record => "record",
            OwnerKind::Variant => "variant",
            OwnerKind::ClosureEnv => "closure-env",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnVia {
    Created,
    Received,
}

impl OwnVia {
    pub fn wire(&self) -> &'static str {
        match self {
            OwnVia::Created => "created",
            OwnVia::Received => "received",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisownVia {
    /// The value left this slot alive (call argument, aggregate operand,
    /// variant payload move) — responsibility continues elsewhere.
    MovedOut,
    /// A newer value replaced the slot (re-assignment, VariantSwitch's
    /// old live arm) — the old value is destroyed.
    Replaced,
    /// The frame sweep released the slot (return path or unwind cleanup).
    Swept,
    /// An orphaned closure env reclaimed after every frame unwound.
    Orphan,
}

impl DisownVia {
    pub fn wire(&self) -> &'static str {
        match self {
            DisownVia::MovedOut => "moved-out",
            DisownVia::Replaced => "replaced",
            DisownVia::Swept => "swept",
            DisownVia::Orphan => "orphan",
        }
    }
}

/// The execution's terminal marker (the parent refuses an executed run
/// whose event stream has no `end`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndInfo {
    /// success | trap | resource | cancelled | infra
    pub outcome: &'static str,
    pub value: Option<i32>,
    pub trap: Option<String>,
    pub resource: Option<String>,
    pub steps: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleEvent {
    pub seq: u64,
    pub act: Act,
    /// own/disown only
    pub kind: Option<OwnerKind>,
    /// own/disown (buffer id / env handle / 0 for composites) and free
    /// (buffer id).
    pub id: u64,
    /// own/disown: the LIFO order key (buffer id, composite min-buffer-id,
    /// or env handle).
    pub key: u64,
    pub via_owned: Option<OwnVia>,
    pub via_disowned: Option<DisownVia>,
    pub frame: u64,
    pub slot: u32,
    pub gen: u64,
    /// frame-leave: the release path (false = the Return arm, true = the
    /// unwind/cleanup sweep).
    pub sweep: bool,
    /// free only
    pub bytes: usize,
    /// end only
    pub end: Option<EndInfo>,
}

#[derive(Debug)]
struct Inner {
    enabled: bool,
    events: Vec<LifecycleEvent>,
    overflowed: bool,
    ended: bool,
    /// The multiset of identities that left a slot alive and have not yet
    /// been received elsewhere (`moved-out` matching for `received`).
    in_flight: Vec<(OwnerKind, u64)>,
    /// Buffer ids that have been owned and not yet freed: a later `own` of
    /// such an id is a transfer (`received`), never a second creation. A
    /// slot-level `swept` does NOT end a buffer (a shared handle may keep
    /// it alive — e.g. the Return arm's `rv`); only the `free` fact does.
    alive_buffers: std::collections::BTreeSet<u64>,
    /// Closure-env store handles owned and not yet released (the terminal
    /// `swept`/`orphan` disown releases the env).
    alive_envs: std::collections::BTreeSet<u64>,
}

/// The shared, cloneable lifecycle log. Cloning shares the same
/// underlying events (the arena free hook keeps appending after the
/// execution returns, exactly like the arena ledger itself).
#[derive(Clone, Debug)]
pub struct LifecycleLog {
    inner: std::rc::Rc<std::cell::RefCell<Inner>>,
}

impl Default for LifecycleLog {
    fn default() -> Self {
        LifecycleLog {
            inner: std::rc::Rc::new(std::cell::RefCell::new(Inner {
                enabled: false,
                events: Vec::new(),
                overflowed: false,
                ended: false,
                in_flight: Vec::new(),
                alive_buffers: std::collections::BTreeSet::new(),
                alive_envs: std::collections::BTreeSet::new(),
            })),
        }
    }
}

impl LifecycleLog {
    pub fn is_enabled(&self) -> bool {
        self.inner.borrow().enabled
    }

    /// Turn the channel on. Enabling twice is a caller bug — the log must
    /// be fresh per execution.
    pub fn enable(&self) {
        let mut g = self.inner.borrow_mut();
        debug_assert!(!g.enabled, "lifecycle log re-enabled");
        debug_assert!(g.events.is_empty(), "lifecycle log re-enabled");
        g.enabled = true;
    }

    pub fn overflowed(&self) -> bool {
        self.inner.borrow().overflowed
    }

    /// A copy-once snapshot for the worker response.
    pub fn events(&self) -> Vec<LifecycleEvent> {
        self.inner.borrow().events.clone()
    }

    pub fn ended(&self) -> bool {
        self.inner.borrow().ended
    }

    fn push_raw(&self, ev: LifecycleEvent) {
        let mut g = self.inner.borrow_mut();
        if !g.enabled || g.ended {
            return;
        }
        if g.events.len() >= MAX_EVENTS {
            g.overflowed = true;
            return;
        }
        let seq = g.events.len() as u64;
        g.events.push(LifecycleEvent { seq, ..ev });
    }

    pub fn push_frame_enter(&self, frame: u64) {
        self.push_raw(LifecycleEvent {
            seq: 0,
            act: Act::FrameEnter,
            kind: None,
            id: 0,
            key: 0,
            via_owned: None,
            via_disowned: None,
            frame,
            slot: NO_SLOT,
            gen: 0,
            sweep: false,
            bytes: 0,
            end: None,
        });
    }

    pub fn push_frame_leave(&self, frame: u64, sweep: bool) {
        self.push_raw(LifecycleEvent {
            seq: 0,
            act: Act::FrameLeave,
            kind: None,
            id: 0,
            key: 0,
            via_owned: None,
            via_disowned: None,
            frame,
            slot: NO_SLOT,
            gen: 0,
            sweep,
            bytes: 0,
            end: None,
        });
    }

    /// Record that `(kind, id)` became owned by slot `(frame, slot, gen)`.
    /// `via` is resolved here: an in-flight identity (a prior `moved-out`
    /// that has not landed), a buffer still alive (owned before, not yet
    /// freed) or an env still in the store is a `received` transfer;
    /// anything else is a fresh `created` owner.
    pub fn push_own(&self, kind: OwnerKind, id: u64, key: u64, frame: u64, slot: u32, gen: u64) {
        let via = {
            let mut g = self.inner.borrow_mut();
            if !g.enabled {
                return;
            }
            let pos = g.in_flight.iter().position(|(k, i)| *k == kind && *i == id);
            match pos {
                Some(p) => {
                    g.in_flight.remove(p);
                    OwnVia::Received
                }
                None => match kind {
                    OwnerKind::Str | OwnerKind::List => {
                        if g.alive_buffers.insert(id) {
                            OwnVia::Created
                        } else {
                            OwnVia::Received
                        }
                    }
                    OwnerKind::ClosureEnv => {
                        if g.alive_envs.insert(id) {
                            OwnVia::Created
                        } else {
                            OwnVia::Received
                        }
                    }
                    OwnerKind::Record | OwnerKind::Variant => OwnVia::Created,
                },
            }
        };
        self.push_raw(LifecycleEvent {
            seq: 0,
            act: Act::Own,
            kind: Some(kind),
            id,
            key,
            via_owned: Some(via),
            via_disowned: None,
            frame,
            slot,
            gen,
            sweep: false,
            bytes: 0,
            end: None,
        });
    }

    /// Record the end of slot `(frame, slot, gen)`'s responsibility.
    /// `moved-out` also parks the identity in the in-flight multiset.
    pub fn push_disown(
        &self,
        kind: OwnerKind,
        id: u64,
        key: u64,
        via: DisownVia,
        frame: u64,
        slot: u32,
        gen: u64,
    ) {
        if via == DisownVia::MovedOut {
            let mut g = self.inner.borrow_mut();
            if g.enabled {
                g.in_flight.push((kind, id));
            }
        } else if kind == OwnerKind::ClosureEnv {
            // The store entry releases exactly at the terminal disown.
            self.inner.borrow_mut().alive_envs.remove(&id);
        }
        self.push_raw(LifecycleEvent {
            seq: 0,
            act: Act::Disown,
            kind: Some(kind),
            id,
            key,
            via_owned: None,
            via_disowned: Some(via),
            frame,
            slot,
            gen,
            sweep: false,
            bytes: 0,
            end: None,
        });
    }

    /// A buffer free fact (from the arena free hook; also fires after the
    /// execution ends, ordering the result drop against the run).
    pub fn push_free(&self, id: u64, bytes: usize) {
        // `push_raw` refuses after `end`; free facts AFTER the end marker
        // are legitimate (the host dropping the result) and must still be
        // recorded, so this path bypasses the ended gate explicitly.
        let mut g = self.inner.borrow_mut();
        if !g.enabled {
            return;
        }
        g.alive_buffers.remove(&id);
        if g.events.len() >= MAX_EVENTS {
            g.overflowed = true;
            return;
        }
        let seq = g.events.len() as u64;
        g.events.push(LifecycleEvent {
            seq,
            act: Act::Free,
            kind: None,
            id,
            key: id,
            via_owned: None,
            via_disowned: None,
            frame: 0,
            slot: NO_SLOT,
            gen: 0,
            sweep: false,
            bytes,
            end: None,
        });
    }

    /// Seal the log with the execution's terminal marker. After the `end`
    /// event only `free` facts may follow.
    pub fn push_end(&self, info: EndInfo) {
        let mut g = self.inner.borrow_mut();
        if !g.enabled || g.ended {
            return;
        }
        if g.events.len() >= MAX_EVENTS {
            g.overflowed = true;
            return;
        }
        let seq = g.events.len() as u64;
        g.events.push(LifecycleEvent {
            seq,
            act: Act::End,
            kind: None,
            id: 0,
            key: 0,
            via_owned: None,
            via_disowned: None,
            frame: 0,
            slot: NO_SLOT,
            gen: 0,
            sweep: false,
            bytes: 0,
            end: Some(info),
        });
        g.ended = true;
    }

    /// The free facts recorded so far, as the arena-style `(id, bytes)`
    /// pairs (for the parent's cross-check against the arena ledger).
    pub fn free_pairs(&self) -> Vec<(u64, usize)> {
        self.inner
            .borrow()
            .events
            .iter()
            .filter(|e| e.act == Act::Free)
            .map(|e| (e.id, e.bytes))
            .collect()
    }
}
