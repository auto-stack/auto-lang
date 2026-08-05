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
//! ## Cross-platform build
//!
//! The real D-Bus/UPower adapters use `zbus` (Linux-only). On non-Linux
//! targets, this crate compiles to a stub that re-exports the mock impls from
//! `auto-cosmic-ports` — so the same `use` paths work everywhere and Windows
//! builds/tests are unaffected. On Linux, `mod linux` provides the real
//! adapters; switch with the `port_impl!` macro or feature flag per app.

/// Real Linux adapters (zbus/UPower/D-Bus). Only compiled on `target_os = "linux"`.
#[cfg(target_os = "linux")]
pub mod linux;

/// On non-Linux, re-export the mock impls so downstream code that references
/// `auto_cosmic_ports_linux::*` still compiles (for dev/test on Windows).
/// Production Linux builds use the real `linux` module above.
#[cfg(not(target_os = "linux"))]
pub mod fallback;

#[cfg(target_os = "linux")]
pub use linux::{LinuxClockPort, LinuxNotificationsPort, LinuxPowerPort};

#[cfg(not(target_os = "linux"))]
pub use fallback::{LinuxClockPort, LinuxNotificationsPort, LinuxPowerPort};
