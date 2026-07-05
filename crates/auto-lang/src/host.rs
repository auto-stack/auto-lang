// Plan 011 (MS3-B): Shell host bridge.
//
// AutoLang functions `system()`, `system_status()`, `export()`, `exit()` need
// to invoke the embedding shell. AutoVM lives in a pure-logic crate with no
// shell dependency, so the shell registers a `ShellHost` implementation via
// `AutoVM::set_host(...)`. Native shims for the four functions read
// `vm.host` and forward.
//
// The host is `Send + Sync` so it can live in `Arc<dyn ShellHost>` on the VM
// (whose native shims are `Send + Sync`). Implementations are expected to be
// single-threaded in practice (ash scripts run on one thread); the trait
// uses interior mutability (`&self`) via the implementation's own locking.

use std::sync::Arc;

/// A bridge from AutoLang to the embedding shell.
///
/// All methods take `&self`; implementations provide their own interior
/// mutability (e.g. a `Mutex<HostState>` holding a raw `*mut Shell` that the
/// host owner sets/clears around each `session.run()`).
pub trait ShellHost: Send + Sync {
    /// Execute a shell command and return its stdout (trailing newline
    /// trimmed). On failure, returns the empty string — use
    /// [`system_status`] to inspect the exit code.
    fn system(&self, cmd: &str) -> String;

    /// Return the exit code of the most recently executed command
    /// (0 = success, non-zero = failure).
    fn system_status(&self) -> i32;

    /// Set an environment variable, equivalent to `export key=val`.
    fn export(&self, key: &str, val: &str);

    /// Request that the script exit with `code`. Sets a "pending exit" flag
    /// that the host's run loop checks between statements.
    fn exit(&self, code: i32);

    /// True after `exit()` has been called — the host's script loop should
    /// stop processing further lines and propagate the exit code.
    fn exit_requested(&self) -> bool;

    /// The exit code requested by the last `exit()` call (0 if none).
    fn requested_exit_code(&self) -> i32;
}

/// Type alias for the shared host handle stored on the VM.
pub type SharedHost = Arc<dyn ShellHost>;
