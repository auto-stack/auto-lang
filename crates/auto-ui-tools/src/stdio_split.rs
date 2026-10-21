//! PLAN-815 T-05: control-stream / log-stream separation at the OS handle
//! level for the tool binary.
//!
//! The tool's stdout IS the framed control stream, but the migrated run
//! paths (and every child they spawn with inherited stdio) print
//! human-readable output via `println!`/inherited stdout. Swapping the
//! process stdout handle onto stderr **before the first print** routes all
//! of that to the log stream by construction, while the control writer
//! keeps the original handle:
//!
//! - in-crate `println!` (step banners, cargo/npm noise): Rust's global
//!   stdout resolves the OS handle lazily on first use, so a swap at bin
//!   start makes them land on stderr;
//! - children spawned with `Stdio::inherit()` (legacy arms) inherit the
//!   redirected handles: stdout bytes go to the log stream too;
//! - hosted children are piped by [`crate::proc_host`] regardless.
//!
//! Must be called before anything writes to stdout. The returned `File`
//! owns a duplicate of the original stdout handle; writing framed messages
//! through it is the only thing that ever reaches the client's control
//! stream.

use std::fs::File;

/// Take over the process stdio: returns a writer bound to the ORIGINAL
/// stdout (control stream) after pointing the process stdout handle at
/// stderr (log stream).
///
/// # Safety
///
/// Call exactly once, at bin start, before any stdout write. The returned
/// `File` owns the duplicated handle; the process-wide stdout handle is
/// redirected for the remainder of the process lifetime.
#[cfg(windows)]
pub unsafe fn split_control_from_stdout() -> File {
    use std::os::windows::io::{FromRawHandle, OwnedHandle};
    use std::os::windows::raw::HANDLE;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> HANDLE;
        fn SetStdHandle(nStdHandle: u32, hHandle: HANDLE) -> i32;
        fn DuplicateHandle(
            hSourceProcessHandle: HANDLE,
            hSourceHandle: HANDLE,
            hTargetProcessHandle: HANDLE,
            lpTargetHandle: *mut HANDLE,
            dwDesiredAccess: u32,
            bInheritHandle: i32,
            dwOptions: u32,
        ) -> i32;
        fn GetCurrentProcess() -> HANDLE;
    }

    const STD_OUTPUT_HANDLE: u32 = (-11i32) as u32;
    const STD_ERROR_HANDLE: u32 = (-12i32) as u32;
    const DUPLICATE_SAME_ACCESS: u32 = 2;

    let original = GetStdHandle(STD_OUTPUT_HANDLE);
    assert!(
        !original.is_null(),
        "GetStdHandle(STD_OUTPUT_HANDLE) failed"
    );
    let mut duplicate: HANDLE = std::ptr::null_mut();
    let ok = DuplicateHandle(
        GetCurrentProcess(),
        original,
        GetCurrentProcess(),
        &mut duplicate,
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    );
    assert!(ok != 0, "DuplicateHandle for the control stream failed");

    let stderr_handle = GetStdHandle(STD_ERROR_HANDLE);
    assert!(
        !stderr_handle.is_null(),
        "GetStdHandle(STD_ERROR_HANDLE) failed"
    );

    // Route the process stdout handle onto stderr: every later println!
    // and inherited-stdio child lands on the log stream.
    let swapped = SetStdHandle(STD_OUTPUT_HANDLE, stderr_handle);
    assert!(swapped != 0, "SetStdHandle(stdout -> stderr) failed");

    // SAFETY: `duplicate` is a fresh owned handle from DuplicateHandle.
    File::from(OwnedHandle::from_raw_handle(duplicate))
}

#[cfg(not(windows))]
pub unsafe fn split_control_from_stdout() -> File {
    use std::os::unix::io::{FromRawFd, IntoRawFd};

    #[link(name = "c")]
    extern "C" {
        fn dup(fd: i32) -> i32;
        fn dup2(oldfd: i32, newfd: i32) -> i32;
    }

    // SAFETY: raw fd duplication at bin start, before any stdout use.
    let control_fd = dup(1);
    assert!(control_fd >= 0, "dup(stdout) for the control stream failed");
    // Route fd 1 onto fd 2 (stderr): later println!s and inherited-stdio
    // children land on the log stream.
    let ok = dup2(2, 1);
    assert!(ok >= 0, "dup2(stderr, stdout) failed");
    // SAFETY: control_fd is a fresh descriptor from dup(1).
    File::from_raw_fd(control_fd)
}
