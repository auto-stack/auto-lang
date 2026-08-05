//! Non-Linux fallback: re-export the W2 mock impls under Linux-named types.
//!
//! This lets Windows dev/test code reference `auto_cosmic_ports_linux::*`
//! without conditional `use` paths. The real Linux adapters live in
//! [`crate::linux`] (compiled only on `target_os = "linux"`).

pub use auto_cosmic_ports::mock::{MockClockPort as LinuxClockPort, MockNotificationsPort as LinuxNotificationsPort, MockPowerPort as LinuxPowerPort};
