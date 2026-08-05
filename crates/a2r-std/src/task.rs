//! Task module — actor runtime for transpiled Auto `task` definitions.
//!
//! Plan 387: provides the runtime that a2r-generated Rust code links against
//! when translating Auto's actor model (`task Name { ... on { ... } }`).
//! Mirrors the AutoVM actor semantics (Plan 317 path B):
//!   - unbounded mailbox (send never blocks, matching VM's `Vec`-backed queue)
//!   - FIFO dispatch
//!   - actors run until all senders are dropped, then drain in-flight messages
//!     and exit (matching VM's "process all in-flight before exit")
//!   - single-threaded cooperative scheduling (generated `main` uses
//!     `#[tokio::main(flavor = "current_thread")]`)
//!
//! ## Generated-code integration
//!
//! The a2r transpiler emits, per `task` definition, a spawn helper. Because the
//! task name is statically known at transpile time, the helper is a plain
//! function named `spawn_<TaskName>`:
//!
//! ```ignore
//! pub fn spawn_counter(rt: &mut a2r_std::task::ActorRuntime) -> a2r_std::task::TaskHandle<i64> {
//!     let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<i64>();
//!     let join = tokio::spawn(async move {
//!         let mut actor = Counter::new();
//!         let _ = actor.start().await;                 // start hook first
//!         while let Some(msg) = rx.recv().await {
//!             let _ = actor.handle_msg(msg, a2r_std::task::NopReply).await;
//!         }
//!         let _ = actor.stop().await;                  // stop hook on mailbox close (Tier 2)
//!     });
//!     rt.track(join);
//!     a2r_std::task::TaskHandle::new(tx)
//! }
//! ```
//!
//! Generated `main` keeps the runtime and calls `run_to_completion` last:
//! ```ignore
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     let mut __rt = a2r_std::task::ActorRuntime::new();
//!     let h = spawn_counter(&mut __rt);
//!     h.send(1i64);
//!     __rt.run_to_completion().await;
//! }
//! ```

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// A handle to a spawned actor, carrying only the mailbox sender.
///
/// `send` is non-blocking (unbounded channel) to match VM semantics where
/// `h.send(msg)` enqueues immediately and returns. The actor's `JoinHandle`
/// is owned by `ActorRuntime`, not by this handle — so dropping a handle does
/// NOT abort the actor; it only (eventually) closes the channel when the last
/// sender clone is gone.
pub struct TaskHandle<M: Send + 'static> {
    tx: mpsc::UnboundedSender<M>,
}

impl<M: Send + 'static> TaskHandle<M> {
    /// Construct a handle from its mailbox sender. Generated spawn helpers
    /// call this after registering the join handle with `ActorRuntime::track`.
    pub fn new(tx: mpsc::UnboundedSender<M>) -> Self {
        Self { tx }
    }

    /// Send a message to the actor's mailbox. Non-blocking; matches VM `h.send`.
    /// Errors (receiver dropped) are silently ignored to match VM's fire-and-forget send.
    pub fn send(&self, msg: M) {
        let _ = self.tx.send(msg);
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

/// Collects spawned actors' join handles so `main` can wait for all in-flight
/// messages to drain before exiting — matching the VM's "process all in-flight
/// messages then exit" liveness contract.
///
/// Generated `main` instantiates this, passes `&mut` to each `spawn_<Task>`
/// helper, and calls `run_to_completion` as its last statement.
pub struct ActorRuntime {
    handles: Vec<JoinHandle<()>>,
}

impl ActorRuntime {
    /// Create an empty runtime.
    pub fn new() -> Self {
        Self { handles: Vec::new() }
    }

    /// Track a spawned actor's join handle. Generated spawn helpers call this
    /// right after `tokio::spawn`, then return a fresh `TaskHandle` to the caller.
    pub fn track(&mut self, join: JoinHandle<()>) {
        self.handles.push(join);
    }

    /// Wait for all tracked actors to finish.
    ///
    /// IMPORTANT: all `TaskHandle`s (and any `UnboundedSender` clones) must be
    /// dropped before this returns — otherwise the mailbox channel never closes,
    /// `rx.recv()` never returns `None`, and actors never exit. Generated `main`
    /// drops handles at end of scope, so this is naturally satisfied when
    /// `run_to_completion` is the last statement.
    pub async fn run_to_completion(mut self) {
        for h in self.handles.drain(..) {
            let _ = h.await;
        }
    }
}

impl Default for ActorRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Minimal actor loop equivalent to what a2r generates: start hook → recv loop.
    /// Each message is appended to `log`; start hook sets `started` first.
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
        rt.track(tokio::spawn(async move {
            actor_loop(rx, log_c, started_c).await;
        }));
        let h = TaskHandle::new(tx);
        for m in messages {
            h.send(*m);
        }
        drop(h);
        rt.run_to_completion().await;
        let was_started = *started.lock().unwrap();
        let received = log.lock().unwrap().clone();
        (was_started, received)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn start_hook_runs_before_messages() {
        // VM contract item 1: start hook runs once at spawn, before any message.
        let (started, received) = run_one_actor(&[1]).await;
        assert!(started, "start hook must run");
        assert_eq!(received, vec![1]);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fifo_dispatch_order() {
        // VM contract item 2: messages dispatched in send order (mirrors
        // test/vm/23_actor/003_multi_message: send 1,2,1 → got one\ngot two\ngot one).
        let (_, received) = run_one_actor(&[1, 2, 1]).await;
        assert_eq!(received, vec![1, 2, 1]);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn empty_mailbox_exits_cleanly() {
        // VM contract item 8: program exits cleanly when main returns + mailbox empty.
        // No messages sent → drop sender → recv None → actor exits; must not hang.
        let (_, received) = run_one_actor(&[]).await;
        assert!(received.is_empty(), "no messages → no dispatch");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn nop_reply_swallows_value() {
        // Stmt::Reply emits `let _ = reply_tx.send(expr);` — NopReply must no-op.
        let r = NopReply;
        let _ = r.send(42i64);
        let _ = r.send("hello");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn register_then_run_drains_all() {
        // Full generated-code pattern: track + TaskHandle::new + send + drop + run.
        // Mirrors a Counter that sums its messages (like VM 006_state_increment).
        let sum = Arc::new(Mutex::new(0i64));
        let sum_c = sum.clone();
        let (tx, mut rx) = mpsc::unbounded_channel::<i64>();
        let mut rt = ActorRuntime::new();
        rt.track(tokio::spawn(async move {
            while let Some(m) = rx.recv().await {
                *sum_c.lock().unwrap() += m; // state persists across messages (VM contract item 7)
            }
        }));
        let h = TaskHandle::new(tx);
        h.send(1);
        h.send(1);
        h.send(1);
        drop(h);
        rt.run_to_completion().await;
        assert_eq!(*sum.lock().unwrap(), 3, "state must persist across 3 messages");
    }
}
