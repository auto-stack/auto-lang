//! Plan 365 W4: Linux port adapters — real system-service implementations.
//!
//! These are the "Linux adapter" half of D3's port model. Each implements a
//! `auto-cosmic-ports` port trait against the real Linux system service:
//!
//! | Port                  | Linux service                          |
//! |-----------------------|----------------------------------------|
//! | `PowerPort`           | UPower (D-Bus `org.freedesktop.UPower`)|
//! | `ClockPort`           | `clock_gettime` (CLOCK_REALTIME)       |
//! | `NotificationsPort`   | FreeDesktop Notifications (D-Bus)      |
//!
//! ## Status
//!
//! **Scaffold** — the crate structure and adapter signatures are in place;
//! the real D-Bus/UPower calls require `zbus` (Linux-only) and are implemented
//! incrementally per COSMIC component. The W2 mock impls remain the primary
//! path on Windows.
//!
//! ## Build
//!
//! Linux/WSL2 only. Excluded from the workspace by default.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(not(target_os = "linux"))]
compile_error!(
    "auto-cosmic-ports-linux is Linux-only (zbus/UPower/D-Bus). \
     It is excluded from the workspace by default; do not build on Windows."
);
