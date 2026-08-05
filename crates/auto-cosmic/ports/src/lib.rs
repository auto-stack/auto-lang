//! Plan 365 W2: System port interfaces + mock framework.
//!
//! Every Linux-only system dependency (D-Bus services, Wayland protocols,
//! /proc, UPower, etc.) is accessed through a port trait with two
//! implementations:
//!
//! - **Mock** (`mock` module): scriptable fake driven by recorded fixtures /
//!   synthetic events, so app logic is fully exercisable on Windows.
//! - **Linux adapter** (W4): real zbus / wayland-client / /proc code.
//!
//! Initial port list (from the COSMIC component analysis):
//! NotificationsPort, PowerPort (UPower/logind), ClockPort (timed events),
//! AudioPort, DisplayPort, NetworkPort, BluetoothPort, SessionPort,
//! PortalPort, SecretsPort.
//!
//! W2 delivers the trait definitions + mock impls + a demo app
//! (clock+battery applet) that runs on Windows driven by scripted events.

pub mod mock;

/// A system port — an abstract interface to a platform service.
///
/// Each port is a small trait so the app core stays platform-neutral. The mock
/// impls live in [`mock`]; real Linux adapters land in W4.
pub trait SystemPort: Send {
    /// Human-readable port name (e.g. "power", "notifications").
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// PowerPort — battery / AC status (UPower on Linux).
// ---------------------------------------------------------------------------

/// Battery state reported by the power service.
#[derive(Debug, Clone)]
pub struct BatteryState {
    /// 0.0–1.0 state of charge.
    pub level: f64,
    /// True when on AC power.
    pub on_ac: bool,
}

/// Port for power/battery queries (UPower / logind on Linux).
pub trait PowerPort: SystemPort {
    /// Current battery state. `level = 1.0` and `on_ac = true` when no battery.
    fn battery(&self) -> BatteryState;
}

// ---------------------------------------------------------------------------
// ClockPort — wall-clock time + tick events.
// ---------------------------------------------------------------------------

/// Port for wall-clock time (used by clock applets, timers, etc.).
pub trait ClockPort: SystemPort {
    /// Current wall-clock time in seconds since Unix epoch (mockable).
    fn now_secs(&self) -> u64;

    /// Current hour/minute/second in local time (mockable).
    fn hms(&self) -> (u8, u8, u8) {
        let secs = self.now_secs();
        let h = ((secs / 3600) % 24) as u8;
        let m = ((secs / 60) % 60) as u8;
        let s = (secs % 60) as u8;
        (h, m, s)
    }
}

// ---------------------------------------------------------------------------
// NotificationsPort — desktop notifications (D-Bus FreeDesktop on Linux).
// ---------------------------------------------------------------------------

/// A notification event received from the system.
#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub summary: String,
    pub body: String,
}

/// Port for receiving desktop notifications.
pub trait NotificationsPort: SystemPort {
    /// Pop the next pending notification (non-blocking). Returns `None` if empty.
    fn poll_notification(&self) -> Option<Notification>;
}
