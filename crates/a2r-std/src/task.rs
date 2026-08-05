//! Task module — actor runtime for transpiled Auto `task` definitions.
//!
//! Plan 387: provides the runtime that a2r-generated Rust code links against
//! when translating Auto's actor model (`task Name { ... on { ... } }`).
//! Mirrors the AutoVM actor semantics (Plan 317 path B):
//!   - unbounded mailbox (send never blocks, matching VM's `Vec`-backed queue)
//!   - FIFO dispatch
//!   - actors process all in-flight messages before the program exits
//!
//! ## §16 first-class-citizen model (RAII)
//!
//! `TaskRef<M>` is the SOLE owner of an actor's mailbox sender. Dropping a
//! `TaskRef` (anywhere — in `main`, in a helper fn, as a struct field) closes
//! the mailbox: the actor's `rx.recv().await` returns `None` and it exits. There
//! is no separate `ActorRuntime` holding a sender clone, and no `__rt` local
//! binding threaded through `main`.
//!
//! Spawned actors' `JoinHandle`s are tracked in a thread-local registry via
//! `track_join` (called by every generated `spawn_<name>` helper). `drain_all`
//! (called at the end of generated `main`) waits — polling a per-mailbox
//! pending-message counter — until every already-sent message has been fully
//! handled by its actor, then returns. It does NOT await the join handles: when
//! `main` returns, the Tokio runtime tears down remaining tasks. This avoids the
//! deadlock where `drain_all` would join an actor that is waiting for a
//! `TaskRef` drop that only happens when `main` returns (i.e. after the join).
//!
//! ## In-flight tracking (no fixed yield count)
//!
//! Every message `send`t through a `TaskRef` from `channel()` increments an
//! atomic pending counter; the generated actor loop calls `rx.mark_processed()`
//! after each `handle_msg` completes. `drain_all` loops (yielding between
//! checks) until all registered counters reach zero — so it works regardless of
//! how many messages were sent or how many times handlers `await` internally.
//! Earlier iterations drained with a fixed 16 yields, which silently dropped
//! messages when a handler awaited (each yield only schedules one message's
//! worth of progress on a single-threaded runtime).
//!
//! ## Generated-code integration
//!
//! ```ignore
//! // spawn helper — no __rt parameter; any function can call it
//! pub fn spawn_counter() -> a2r_std::task::TaskRef<i64> {
//!     let (taskref, mut rx) = a2r_std::task::channel::<i64>();
//!     a2r_std::task::track_join(tokio::spawn(async move {
//!         let mut actor = Counter::new();
//!         let _ = actor.start().await;
//!         while let Some(msg) = rx.recv().await {
//!             let _ = actor.handle_msg(msg, a2r_std::task::NopReply).await;
//!             rx.mark_processed();   // tell drain_all this message is done
//!         }
//!     }));
//!     taskref
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let h = spawn_counter();   // TaskRef — works in any fn, not just main
//!     h.send(1);
//!     a2r_std::task::drain_all().await;  // waits until all sent messages handled
//!     // h drops here (end of main) → mailbox closes → actor exits naturally
//! }
//! ```

use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Safety cap for `drain_all`'s wait loop. Only reachable if a handler never
/// finishes (spins forever); prevents tests from hanging in that case.
const MAX_DRAIN_SPINS: usize = 1_000_000;

thread_local! {
    /// Registry of spawned actor JoinHandles, populated by `track_join` and
    /// drained by `drain_all`. Thread-local because the generated `main` runs on
    /// a single runtime thread; `track_join`/`drain_all` are called from there.
    static JOIN_HANDLES: RefCell<Vec<JoinHandle<()>>> = RefCell::new(Vec::new());
    /// Registry of pending-message counters (one per live `channel()`), polled
    /// by `drain_all`. Weak refs: a dropped mailbox leaves a dead entry that is
    /// skipped on upgrade failure.
    static PENDING: RefCell<Vec<Weak<AtomicUsize>>> = RefCell::new(Vec::new());
}

/// Track a spawned actor's JoinHandle so `drain_all` can wait for in-flight
/// messages. Called by every generated `spawn_<name>` helper right after
/// `tokio::spawn`. The handle is held until `drain_all` runs.
pub fn track_join(join: JoinHandle<()>) {
    JOIN_HANDLES.with(|h| h.borrow_mut().push(join));
}

/// Create a `TaskRef` + its paired actor-side receiver, sharing one in-flight
/// message counter.
///
/// This is the canonical construction path used by generated `spawn_<name>`
/// helpers: `TaskRef::send` increments the counter, the actor loop calls
/// `ActorReceiver::mark_processed` after each message, and `drain_all` waits
/// until the counter hits zero.
pub fn channel<M: Send + 'static>() -> (TaskRef<M>, ActorReceiver<M>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let pending = Arc::new(AtomicUsize::new(0));
    PENDING.with(|reg| reg.borrow_mut().push(Arc::downgrade(&pending)));
    (
        TaskRef {
            tx,
            pending: pending.clone(),
        },
        ActorReceiver { rx, pending },
    )
}

/// Let every already-sent message be processed by its actor, then return.
///
/// Generated `main` calls this as its last statement. It yields the runtime and
/// re-checks the registered pending counters until every sent-but-unhandled
/// message has been processed (i.e. until no live mailbox has `pending > 0`),
/// then returns. It does NOT await the join handles (doing so would deadlock
/// when a `TaskRef` is still alive in `main`'s scope — the actor waits for the
/// mailbox to close, which only happens when `main` returns and drops the
/// `TaskRef`). By the time it returns, all already-sent messages have been
/// handled; the Tokio runtime tears down the (now idle) actor tasks when `main`
/// exits.
pub async fn drain_all() {
    let mut spins = 0usize;
    loop {
        // Let queued messages get scheduled and processed before checking.
        tokio::task::yield_now().await;
        let any_pending = PENDING.with(|reg| {
            reg.borrow().iter().any(|w| {
                w.upgrade()
                    .map(|c| c.load(Ordering::SeqCst) > 0)
                    .unwrap_or(false)
            })
        });
        if !any_pending {
            return;
        }
        spins += 1;
        // Defensive cap: only hit if a handler never finishes (a fixed yield
        // count would otherwise lose messages; this loop waits properly but
        // must not hang a test forever on a stuck handler).
        if spins >= MAX_DRAIN_SPINS {
            return;
        }
    }
}

/// The actor-side half of `channel()`: the mailbox receiver plus the shared
/// in-flight counter. Generated actor loops own this and call
/// `mark_processed` after each `handle_msg` so `drain_all` knows the message is
/// fully handled.
pub struct ActorReceiver<M> {
    rx: mpsc::UnboundedReceiver<M>,
    pending: Arc<AtomicUsize>,
}

impl<M> ActorReceiver<M> {
    /// Receive the next message; `None` once the mailbox is closed and drained.
    pub async fn recv(&mut self) -> Option<M> {
        self.rx.recv().await
    }

    /// Mark one message as fully handled. Must be called exactly once per
    /// message returned by `recv`, after the handler has completed.
    pub fn mark_processed(&self) {
        self.pending.fetch_sub(1, Ordering::SeqCst);
    }
}

/// A reference to a spawned actor for sending messages. SOLE owner of the
/// mailbox sender (RAII): dropping it closes the mailbox, letting the actor exit.
///
/// `send` is non-blocking (unbounded channel) to match VM semantics where
/// `h.send(msg)` enqueues immediately and returns. `TaskRef` is a first-class
/// type — it can be stored in struct fields, passed to functions, returned, etc.
/// (Plan 387 §16 P0-2). It is NOT `Clone`; to share an actor, move the `TaskRef`
/// or wrap it in `Arc` at the call site.
pub struct TaskRef<M: Send + 'static> {
    tx: mpsc::UnboundedSender<M>,
    pending: Arc<AtomicUsize>,
}

impl<M: Send + 'static> TaskRef<M> {
    /// Construct a TaskRef from a bare mailbox sender, with no in-flight
    /// tracking. Only for hand-written actor loops that pair with `track_join`
    /// but never call `mark_processed`; `drain_all` cannot wait on such mailboxes
    /// (it would spin until the safety cap). Generated code should use
    /// [`channel`] instead so `drain_all` can guarantee delivery.
    pub fn new(tx: mpsc::UnboundedSender<M>) -> Self {
        Self {
            tx,
            pending: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Send a message to the actor's mailbox. Non-blocking; matches VM `h.send`.
    /// Errors (receiver dropped) are silently ignored to match VM's fire-and-forget send.
    pub fn send(&self, msg: M) {
        if self.tx.send(msg).is_ok() {
            // Only count messages actually queued; a failed send (mailbox
            // already closed) must not leave a phantom pending message.
            self.pending.fetch_add(1, Ordering::SeqCst);
        }
    }
}

// TaskRef's Drop is the default (drops the UnboundedSender, closing the channel
// if this is the last sender). No custom Drop needed — RAII is automatic.

impl<M: Send + 'static> std::fmt::Debug for TaskRef<M> {
    /// Handles are move-only; Debug exists so generated structs that hold a
    /// `TaskRef` field can still derive `Debug` (Plan 387 follow-up).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TaskRef")
    }
}

/// No-op reply channel for Tier 1 (Plan 387 §12.3).
///
/// The a2r `Stmt::Reply` emitter produces `let _ = reply_tx.send(expr);` and so
/// every generated `handle_msg` takes a `reply_tx` parameter. In Tier 1 there is
/// no real ask/reply round-trip (the VM itself does not wire `current_msg_context`),
/// so handlers receive a `NopReply` that swallows the value. Tier 3 will replace
/// this with a real `oneshot::Sender`.
pub struct NopReply;

impl NopReply {
    /// Swallow the reply value. Returns `()` so `let _ = reply_tx.send(x);` compiles.
    pub fn send<T>(&self, _msg: T) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Minimal actor loop equivalent to what a2r generates: start hook → recv loop
    /// with `mark_processed` after each handled message.
    async fn actor_loop<M: Send + 'static>(
        mut rx: ActorReceiver<M>,
        log: Arc<Mutex<Vec<M>>>,
        started: Arc<Mutex<bool>>,
    ) {
        *started.lock().unwrap() = true; // start hook runs first (VM contract item 1)
        while let Some(msg) = rx.recv().await {
            log.lock().unwrap().push(msg); // FIFO dispatch (VM contract item 2)
            rx.mark_processed();
        }
    }

    // Mirror the §16 generated pattern: spawn helper (channel + track_join) +
    // drain_all. Handles are NOT explicitly dropped — drain_all waits for all
    // pending messages to process, then returns (runtime tears down on main exit).
    async fn run_one_actor(messages: &[i64]) -> (bool, Vec<i64>) {
        JOIN_HANDLES.with(|h| h.borrow_mut().clear());
        PENDING.with(|p| p.borrow_mut().clear());
        let started = Arc::new(Mutex::new(false));
        let log: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
        let (h, rx) = channel::<i64>();
        let started_c = started.clone();
        let log_c = log.clone();
        track_join(tokio::spawn(async move {
            actor_loop(rx, log_c, started_c).await;
        }));
        for m in messages {
            h.send(*m);
        }
        drain_all().await;
        // h still alive here (dropped at end of fn) — but drain_all already
        // waited until all in-flight messages were processed.
        let was_started = *started.lock().unwrap();
        let received = log.lock().unwrap().clone();
        (was_started, received)
    }

    #[tokio::test]
    async fn start_hook_runs_before_messages() {
        // VM contract item 1: start hook runs once at spawn, before any message.
        let (started, received) = run_one_actor(&[1]).await;
        assert!(started, "start hook must run");
        assert_eq!(received, vec![1]);
    }

    #[tokio::test]
    async fn fifo_dispatch_order() {
        // VM contract item 2: messages dispatched in send order (mirrors
        // test/vm/23_actor/003_multi_message: send 1,2,1 → got one\ngot two\ngot one).
        let (_, received) = run_one_actor(&[1, 2, 1]).await;
        assert_eq!(received, vec![1, 2, 1]);
    }

    #[tokio::test]
    async fn empty_mailbox_exits_cleanly() {
        // VM contract item 8: program exits cleanly when main returns + mailbox empty.
        let (_, received) = run_one_actor(&[]).await;
        assert!(received.is_empty(), "no messages → no dispatch");
    }

    #[tokio::test]
    async fn nop_reply_swallows_value() {
        // Stmt::Reply emits `let _ = reply_tx.send(expr);` — NopReply must no-op.
        let r = NopReply;
        let _ = r.send(42i64);
        let _ = r.send("hello");
    }

    #[tokio::test]
    async fn drain_processes_all_inflight_without_explicit_drop() {
        // §16 RAII: TaskRef is NOT explicitly dropped before drain_all. drain_all
        // waits until all in-flight messages are processed. Mirrors a Counter
        // that sums its messages (like VM 006_state_increment).
        JOIN_HANDLES.with(|h| h.borrow_mut().clear());
        PENDING.with(|p| p.borrow_mut().clear());
        let sum = Arc::new(Mutex::new(0i64));
        let sum_c = sum.clone();
        let (h, mut rx) = channel::<i64>();
        track_join(tokio::spawn(async move {
            while let Some(m) = rx.recv().await {
                *sum_c.lock().unwrap() += m;
                rx.mark_processed();
            }
        }));
        h.send(1);
        h.send(1);
        h.send(1);
        // NOTE: no drop(h) — the whole point of §16 RAII. drain_all must still
        // wait until all 3 sent messages are processed.
        drain_all().await;
        assert_eq!(*sum.lock().unwrap(), 3, "all 3 in-flight messages must process without explicit drop");
    }

    #[tokio::test]
    async fn drain_waits_for_async_handlers() {
        // Regression (Plan 387 archive fix): the old fixed-16-yield drain_all
        // lost messages when a handler awaited internally — 30 sends of an
        // async-yielding handler processed only 15. drain_all must keep waiting
        // until the pending counter reaches zero.
        JOIN_HANDLES.with(|h| h.borrow_mut().clear());
        PENDING.with(|p| p.borrow_mut().clear());
        let count = Arc::new(Mutex::new(0i64));
        let count_c = count.clone();
        let (h, mut rx) = channel::<i64>();
        track_join(tokio::spawn(async move {
            while let Some(_) = rx.recv().await {
                tokio::task::yield_now().await; // async work inside the handler
                *count_c.lock().unwrap() += 1;
                rx.mark_processed();
            }
        }));
        for i in 0..30 {
            h.send(i);
        }
        drain_all().await;
        assert_eq!(
            *count.lock().unwrap(),
            30,
            "all 30 messages must process even though the handler awaits (no fixed-yield cap)"
        );
    }

    #[tokio::test]
    async fn drain_waits_for_many_actors() {
        // Many actors each with queued messages: drain_all must wait for all of
        // them (a fixed yield count would only cover one scheduling round each).
        JOIN_HANDLES.with(|h| h.borrow_mut().clear());
        PENDING.with(|p| p.borrow_mut().clear());
        let total = Arc::new(Mutex::new(0i64));
        let mut handles = Vec::new();
        for _ in 0..20 {
            let t = total.clone();
            let (h, mut rx) = channel::<i64>();
            track_join(tokio::spawn(async move {
                while let Some(_) = rx.recv().await {
                    tokio::task::yield_now().await; // async handler work
                    *t.lock().unwrap() += 1;
                    rx.mark_processed();
                }
            }));
            h.send(1);
            handles.push(h);
        }
        drain_all().await;
        assert_eq!(*total.lock().unwrap(), 20, "all 20 actors must process their message");
    }

    #[tokio::test]
    async fn taskref_can_live_in_arbitrary_scope() {
        // §16 P0-1: spawn works in a helper fn, not just main. The TaskRef
        // returned from the helper is used then dropped at helper's scope end.
        JOIN_HANDLES.with(|h| h.borrow_mut().clear());
        PENDING.with(|p| p.borrow_mut().clear());
        let received = Arc::new(Mutex::new(Vec::<i64>::new()));
        let received_c = received.clone();

        async fn spawn_and_send(log: Arc<Mutex<Vec<i64>>>) {
            // helper that spawns + sends — would fail under the old __rt model
            let (h, mut rx) = channel::<i64>();
            let log_c = log;
            track_join(tokio::spawn(async move {
                while let Some(m) = rx.recv().await {
                    log_c.lock().unwrap().push(m);
                    rx.mark_processed();
                }
            }));
            h.send(42);
            // h drops here (end of helper) → mailbox closes → actor exits
        }
        spawn_and_send(received_c).await;
        drain_all().await;
        assert_eq!(*received.lock().unwrap(), vec![42], "helper-fn spawn + RAII drop must work");
    }
}
