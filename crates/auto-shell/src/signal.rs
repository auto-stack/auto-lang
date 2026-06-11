//! Platform-specific Ctrl+C protection during external command execution.
//!
//! When ASH runs an external command, Ctrl+C should:
//! - Reach the child process (so the user can interrupt it)
//! - NOT kill ASH itself (so the shell continues after the child exits)
//!
//! This module provides a RAII guard (`CtrlCGuard`) that temporarily
//! protects ASH from Ctrl+C while allowing the child to receive it.

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag: when true, ASH ignores Ctrl+C.
static IGNORE_CTRL_C: AtomicBool = AtomicBool::new(false);

// ── Platform-specific implementation ────────────────────────────

#[cfg(windows)]
mod platform {
    use super::IGNORE_CTRL_C;
    use std::sync::atomic::Ordering;

    // Raw Windows API types
    type BOOL = i32;
    type DWORD = u32;

    // Handler routine callback signature
    type PHANDLER_ROUTINE = Option<unsafe extern "system" fn(DWORD) -> BOOL>;

    extern "system" {
        fn SetConsoleCtrlHandler(HandlerRoutine: PHANDLER_ROUTINE, Add: BOOL) -> BOOL;
    }

    const CTRL_C_EVENT: DWORD = 0;
    const CTRL_BREAK_EVENT: DWORD = 1;

    /// Console handler that swallows Ctrl+C when IGNORE_CTRL_C is set.
    ///
    /// Returns TRUE (handled) during command execution → ASH survives.
    /// Returns FALSE otherwise → default handler (or crossterm's) runs.
    unsafe extern "system" fn ctrl_handler(ctrl_type: DWORD) -> BOOL {
        if IGNORE_CTRL_C.load(Ordering::SeqCst)
            && (ctrl_type == CTRL_C_EVENT || ctrl_type == CTRL_BREAK_EVENT)
        {
            return 1; // TRUE — handled, don't terminate
        }
        0 // FALSE — let next handler decide
    }

    /// One-time initialization. Call once at startup before any commands.
    pub fn init() {
        unsafe {
            SetConsoleCtrlHandler(Some(ctrl_handler), 1);
        }
    }

    /// RAII guard: while alive, ASH ignores Ctrl+C.
    pub struct CtrlCGuard;

    impl CtrlCGuard {
        pub fn new() -> Self {
            IGNORE_CTRL_C.store(true, Ordering::SeqCst);
            CtrlCGuard
        }
    }

    impl Drop for CtrlCGuard {
        fn drop(&mut self) {
            IGNORE_CTRL_C.store(false, Ordering::SeqCst);
        }
    }
}

#[cfg(unix)]
mod platform {
    use super::IGNORE_CTRL_C;
    use std::sync::atomic::Ordering;

    // POSIX signal constants (all Unix flavors)
    const SIGINT: i32 = 2;

    // Raw libc FFI — avoids adding `libc` as a dependency
    extern "C" {
        fn signal(sig: i32, handler: usize) -> usize;
    }

    const SIG_DFL: usize = 0;

    /// SIGINT handler: catches Ctrl+C during command execution.
    ///
    /// When IGNORE_CTRL_C is true, the signal is silently caught
    /// (ASH survives). Otherwise, it restores default behavior and
    /// re-raises so the process terminates as usual.
    extern "C" fn sigint_handler(_sig: i32) {
        if IGNORE_CTRL_C.load(Ordering::SeqCst) {
            // Silently catch — ASH survives, child handles its own SIGINT
        } else {
            // Not during command execution — restore default and re-raise
            unsafe {
                signal(SIGINT, SIG_DFL);
                raise(SIGINT);
            }
        }
    }

    extern "C" {
        fn raise(sig: i32) -> i32;
    }

    /// One-time initialization. Call once at startup.
    pub fn init() {
        unsafe {
            signal(SIGINT, sigint_handler as usize);
        }
    }

    /// RAII guard: while alive, ASH catches and ignores SIGINT.
    pub struct CtrlCGuard;

    impl CtrlCGuard {
        pub fn new() -> Self {
            IGNORE_CTRL_C.store(true, Ordering::SeqCst);
            CtrlCGuard
        }
    }

    impl Drop for CtrlCGuard {
        fn drop(&mut self) {
            IGNORE_CTRL_C.store(false, Ordering::SeqCst);
        }
    }
}

pub use platform::{CtrlCGuard, init};
