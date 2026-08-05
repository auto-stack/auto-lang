//! Task module — actor runtime for transpiled Auto `task` definitions.
//!
//! Plan 387: provides the runtime that a2r-generated Rust code links against
//! when translating Auto's actor model (`task Name { ... on { ... } }`).
//! Mirrors the AutoVM actor semantics (Plan 317 path B):
//!   - unbounded mailbox (send never blocks, matching VM's `Vec`-backed queue)
//!   - FIFO dispatch
//!   - actors run until all senders are dropped, then drain in-flight messages
//!     and exit (matching VM's "process all in-flight before exit")
//!
//! ## Shutdown model (Plan 387 D1)
//!
//! Each `spawn_<Task>` helper hands the freshly-spawned actor's `TaskHandle`
//! (mailbox sender + join handle) to the `ActorRuntime` via `register`, and gets
//! back a lightweight `TaskRef` carrying a sender clone for `h.send(...)`.
//!
//! The mailbox channel closes only when the LAST sender clone is dropped. After
//! `register`, TWO clones exist: the runtime's (inside the `ActorEntry.closer`)
//! and the user's (inside the `TaskRef`). `run_to_completion` drops the
//! runtime's clone via `closer`, but that ALONE does not close the channel while
//! the user's `TaskRef` lives — so the **generated `main` must drop every
//! `TaskRef` before calling `run_to_completion`**. The a2r transpiler emits
//! `drop(<handle>);` for each variable assigned from `Task.spawn(...)` right
//! before `__rt.run_to_completion().await;`. With both clones dropped,
//! `rx.recv()` returns `None`, the actor exits, and `join.await` resolves.
//!
//! If a `TaskRef` is NOT dropped before `run_to_completion`, `join.await` will
//! hang forever (the actor waits for a close that never comes). There is no
//! runtime timeout/abort fallback — correctness relies on the generated
//! `drop()` calls.
//!
//! ## Generated-code integration
//!
//! ```ignore
//! pub fn spawn_counter(__rt: &mut a2r_std::task::ActorRuntime) -> a2r_std::task::TaskRef<i64> {
//!     let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<i64>();
//!     let join = tokio::spawn(async move {
//!         let mut actor = Counter::new();
//!         let _ = actor.start().await;
//!         while let Some(msg) = rx.recv().await {
//!             let _ = actor.handle_msg(msg, a2r_std::task::NopReply).await;
//!         }
//!     });
//!     __rt.register(a2r_std::task::TaskHandle::new(tx, join))
//! }
//! ```

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// A lightweight reference to a spawned actor for sending messages.
///
/// `send` is non-blocking (unbounded channel) to match VM semantics where
/// `h.send(msg)` enqueues immediately and returns. This does NOT own the actor
/// lifecycle — the `ActorRuntime` does, so dropping a `TaskRef` does not abort
/// the actor; it only drops one sender clone.
pub struct TaskRef<M: Send + 'static> {
    tx: mpsc::UnboundedSender<M>,
}

impl<M: Send + 'static> TaskRef<M> {
    /// Send a message to the actor's mailbox. Non-blocking; matches VM `h.send`.
    /// Errors (receiver dropped) are silently ignored to match VM's fire-and-forget send.
    pub fn send(&self, msg: M) {
        let _ = self.tx.send(msg);
    }
}

/// Internal handle stashed in `ActorRuntime`: holds the last sender clone (so
/// dropping it closes the mailbox) and the actor's JoinHandle. Type-erased via
/// the closer closure so the runtime can store actors of different message types.
struct ActorEntry {
    join: JoinHandle<()>,
    /// Dropping this closes the mailbox channel (drops the last sender held by
    /// the runtime). The user's `TaskRef` holds its own clone.
    closer: Box<dyn FnOnce() + Send>,
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

/// Collects spawned actors so `main` can wait for all in-flight messages to
/// drain before exiting — matching the VM's "process all in-flight messages
/// then exit" liveness contract.
pub struct ActorRuntime {
    entries: Vec<ActorEntry>,
}

impl ActorRuntime {
    /// Create an empty runtime.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Register a spawned actor. The `TaskHandle` carries the mailbox sender and
    /// the JoinHandle; this fn takes ownership, stashes a closer that drops the
    /// sender, and returns a `TaskRef` (with a sender clone) for the caller to
    /// keep sending. The runtime dropping the original sender on shutdown is
    /// what closes the mailbox and lets the actor exit.
    pub fn register<M: Send + 'static>(&mut self, h: TaskHandle<M>) -> TaskRef<M> {
        let TaskHandle { tx, join } = h;
        // Give the caller a sender clone for `h.send(...)`.
        let user_tx = tx.clone();
        // The closer drops the runtime's original sender; once the caller's
        // TaskRef (user_tx) is also dropped, the channel closes.
        let closer: Box<dyn FnOnce() + Send> = Box::new(move || drop(tx));
        self.entries.push(ActorEntry { join, closer });
        TaskRef { tx: user_tx }
    }

    /// Wait for all tracked actors to finish.
    ///
    /// First drops the runtime-held sender clones (closing mailboxes → actors
    /// observe `recv() == None` and exit), then awaits each JoinHandle. The
    /// caller's `TaskRef`s should be dropped by end of `main`'s scope; if any
    /// are still alive they keep the channel open and the actor never exits.
    pub async fn run_to_completion(mut self) {
        // Close every mailbox by dropping the runtime's sender clones.
        for e in self.entries.drain(..) {
            (e.closer)();
            let _ = e.join.await;
        }
    }
}

impl Default for ActorRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Constructed by spawn helpers from `(sender, joinhandle)`. Consumed by
/// `ActorRuntime::register`. Not used directly by generated user code.
pub struct TaskHandle<M: Send + 'static> {
    tx: mpsc::UnboundedSender<M>,
    join: JoinHandle<()>,
}

impl<M: Send + 'static> TaskHandle<M> {
    pub fn new(tx: mpsc::UnboundedSender<M>, join: JoinHandle<()>) -> Self {
        Self { tx, join }
    }
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

    // Mirror the generated spawn-helper + main pattern for one actor.
    async fn run_one_actor(messages: &[i64]) -> (bool, Vec<i64>) {
        let started = Arc::new(Mutex::new(false));
        let log: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = mpsc::unbounded_channel::<i64>();
        let started_c = started.clone();
        let log_c = log.clone();
        let mut rt = ActorRuntime::new();
        let join = tokio::spawn(async move {
            actor_loop(rx, log_c, started_c).await;
        });
        let h = rt.register(TaskHandle::new(tx, join));
        for m in messages {
            h.send(*m);
        }
        drop(h); // drop the TaskRef so only the runtime holds a sender
        rt.run_to_completion().await;
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
    async fn register_then_run_drains_all() {
        // Full generated-code pattern: register + send + drop + run.
        // Mirrors a Counter that sums its messages (like VM 006_state_increment).
        let sum = Arc::new(Mutex::new(0i64));
        let sum_c = sum.clone();
        let (tx, mut rx) = mpsc::unbounded_channel::<i64>();
        let mut rt = ActorRuntime::new();
        let join = tokio::spawn(async move {
            while let Some(m) = rx.recv().await {
                *sum_c.lock().unwrap() += m; // state persists across messages (VM contract item 7)
            }
        });
        let h = rt.register(TaskHandle::new(tx, join));
        h.send(1);
        h.send(1);
        h.send(1);
        drop(h);
        rt.run_to_completion().await;
        assert_eq!(*sum.lock().unwrap(), 3, "state must persist across 3 messages");
    }

    #[tokio::test]
    async fn drop_handle_then_run_completes_without_hang() {
        // The generated-code shutdown contract (Plan 387 D1): main drops every
        // TaskRef BEFORE calling run_to_completion. This test exercises exactly
        // that — drop(h) then run — and asserts it completes (no hang). The
        // closer inside run_to_completion drops the runtime's own sender clone;
        // combined with the user's drop(h), both clones are gone, recv() returns
        // None, the actor exits, and join resolves.
        // NOTE: if drop(h) were removed, run_to_completion would deadlock (the
        // user's clone keeps the channel open). That deadlock case is intentionally
        // NOT tested because it would hang the test runner.
        let (tx, rx) = mpsc::unbounded_channel::<i64>();
        let mut rt = ActorRuntime::new();
        let join = tokio::spawn(async move {
            let mut rx = rx;
            while rx.recv().await.is_some() {}
        });
        let h = rt.register(TaskHandle::new(tx, join));
        h.send(1);
        drop(h);
        rt.run_to_completion().await;
        // Reaching here means no hang — the actor observed the channel close and
        // exited cleanly. (No stdout to assert; completion is the assertion.)
    }
}
