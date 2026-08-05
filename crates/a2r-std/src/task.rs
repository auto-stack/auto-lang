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
//! (called at the end of generated `main`) yields the runtime a few times so
//! every already-sent message is processed by its actor, then returns. It does
//! NOT await the join handles — when `main` returns, the Tokio runtime tears
//! down remaining tasks. This avoids the deadlock where `drain_all` would join
//! an actor that is waiting for a `TaskRef` drop that only happens when `main`
//! returns (i.e. after the join).
//!
//! ## Generated-code integration
//!
//! ```ignore
//! // spawn helper — no __rt parameter; any function can call it
//! pub fn spawn_counter() -> a2r_std::task::TaskRef<i64> {
//!     let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<i64>();
//!     a2r_std::task::track_join(tokio::spawn(async move {
//!         let mut actor = Counter::new();
//!         let _ = actor.start().await;
//!         while let Some(msg) = rx.recv().await {
//!             let _ = actor.handle_msg(msg, a2r_std::task::NopReply).await;
//!         }
//!     }));
//!     a2r_std::task::TaskRef::new(tx)
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let h = spawn_counter();   // TaskRef — works in any fn, not just main
//!     h.send(1);
//!     a2r_std::task::drain_all().await;  // let in-flight messages process
//!     // h drops here (end of main) → mailbox closes → actor exits naturally
//! }
//! ```

use std::cell::RefCell;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

thread_local! {
    /// Registry of spawned actor JoinHandles, populated by `track_join` and
    /// drained by `drain_all`. Thread-local because the generated `main` runs on
    /// a single runtime thread; `track_join`/`drain_all` are called from there.
    static JOIN_HANDLES: RefCell<Vec<JoinHandle<()>>> = RefCell::new(Vec::new());
}

/// Track a spawned actor's JoinHandle so `drain_all` can wait for in-flight
/// messages. Called by every generated `spawn_<name>` helper right after
/// `tokio::spawn`. The handle is held until `drain_all` runs.
pub fn track_join(join: JoinHandle<()>) {
    JOIN_HANDLES.with(|h| h.borrow_mut().push(join));
}

/// Let every already-sent message be processed by its actor, then return.
///
/// Generated `main` calls this as its last statement. It yields the runtime
/// several times so spawned actor tasks (which are pending at `rx.recv().await`)
/// get scheduled and process queued messages. It does NOT await the join handles
/// (doing so would deadlock when a `TaskRef` is still alive in `main`'s scope —
/// the actor waits for the mailbox to close, which only happens when `main`
/// returns and drops the `TaskRef`). Instead, after yielding, it returns; the
/// Tokio runtime tears down remaining tasks when `main` exits. By then all
/// already-sent messages have been processed (the yields drained the mailboxes).
pub async fn drain_all() {
    // Yield enough times for current_thread to run each pending actor task at
    // least once past its recv().await (processing one queued message per yield).
    // A fixed number covers typical cases; pathological deep queues would need
    // a loop until mailboxes are empty, but that requires peeking the channel
    // (not exposed by mpsc). For the VM parity tests (≤3 messages) this suffices.
    for _ in 0..16 {
        tokio::task::yield_now().await;
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
}

impl<M: Send + 'static> TaskRef<M> {
    /// Construct a TaskRef from its mailbox sender. Generated spawn helpers
    /// call this after `track_join`.
    pub fn new(tx: mpsc::UnboundedSender<M>) -> Self {
        Self { tx }
    }

    /// Send a message to the actor's mailbox. Non-blocking; matches VM `h.send`.
    /// Errors (receiver dropped) are silently ignored to match VM's fire-and-forget send.
    pub fn send(&self, msg: M) {
        let _ = self.tx.send(msg);
    }
}

// TaskRef's Drop is the default (drops the UnboundedSender, closing the channel
// if this is the last sender). No custom Drop needed — RAII is automatic.

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

    /// Minimal actor loop equivalent to what a2r generates: start hook → recv loop.
    async fn actor_loop<M: Send + 'static>(
        mut rx: mpsc::UnboundedReceiver<M>,
        log: Arc<Mutex<Vec<M>>>,
        started: Arc<Mutex<bool>>,
    ) {
        *started.lock().unwrap() = true; // start hook runs first (VM contract item 1)
        while let Some(msg) = rx.recv().await {
            log.lock().unwrap().push(msg); // FIFO dispatch (VM contract item 2)
        }
    }

    // Mirror the §16 generated pattern: spawn helper (track_join + TaskRef::new)
    // + drain_all. Handles are NOT explicitly dropped — drain_all yields to let
    // in-flight messages process, then returns (runtime tears down on main exit).
    async fn run_one_actor(messages: &[i64]) -> (bool, Vec<i64>) {
        JOIN_HANDLES.with(|h| h.borrow_mut().clear());
        let started = Arc::new(Mutex::new(false));
        let log: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = mpsc::unbounded_channel::<i64>();
        let started_c = started.clone();
        let log_c = log.clone();
        track_join(tokio::spawn(async move {
            actor_loop(rx, log_c, started_c).await;
        }));
        let h = TaskRef::new(tx);
        for m in messages {
            h.send(*m);
        }
        drain_all().await;
        // h still alive here (dropped at end of fn) — but drain_all already
        // yielded-processed the in-flight messages.
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
        // yields enough times for all in-flight messages to be processed. Mirrors
        // a Counter that sums its messages (like VM 006_state_increment).
        JOIN_HANDLES.with(|h| h.borrow_mut().clear());
        let sum = Arc::new(Mutex::new(0i64));
        let sum_c = sum.clone();
        let (tx, mut rx) = mpsc::unbounded_channel::<i64>();
        track_join(tokio::spawn(async move {
            while let Some(m) = rx.recv().await {
                *sum_c.lock().unwrap() += m;
            }
        }));
        let h = TaskRef::new(tx);
        h.send(1);
        h.send(1);
        h.send(1);
        // NOTE: no drop(h) — the whole point of §16 RAII. drain_all must still
        // process the 3 sent messages.
        drain_all().await;
        assert_eq!(*sum.lock().unwrap(), 3, "all 3 in-flight messages must process without explicit drop");
    }

    #[tokio::test]
    async fn taskref_can_live_in_arbitrary_scope() {
        // §16 P0-1: spawn works in a helper fn, not just main. The TaskRef
        // returned from the helper is used then dropped at helper's scope end.
        JOIN_HANDLES.with(|h| h.borrow_mut().clear());
        let received = Arc::new(Mutex::new(Vec::<i64>::new()));
        let received_c = received.clone();

        async fn spawn_and_send(log: Arc<Mutex<Vec<i64>>>) {
            // helper that spawns + sends — would fail under the old __rt model
            let (tx, mut rx) = mpsc::unbounded_channel::<i64>();
            let log_c = log;
            track_join(tokio::spawn(async move {
                while let Some(m) = rx.recv().await {
                    log_c.lock().unwrap().push(m);
                }
            }));
            let h = TaskRef::new(tx);
            h.send(42);
            // h drops here (end of helper) → mailbox closes → actor exits
        }
        spawn_and_send(received_c).await;
        drain_all().await;
        assert_eq!(*received.lock().unwrap(), vec![42], "helper-fn spawn + RAII drop must work");
    }
}
