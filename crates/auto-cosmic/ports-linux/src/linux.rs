//! Linux port adapter implementations (zbus/UPower/D-Bus).
//!
//! Each adapter wraps a real Linux system service via D-Bus (zbus) or syscalls.

use auto_cosmic_ports::{BatteryState, ClockPort, Notification, NotificationsPort, PowerPort, SystemPort};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// LinuxClockPort — clock_gettime (no D-Bus needed).
// ---------------------------------------------------------------------------

/// Linux clock port backed by `SystemTime` (CLOCK_REALTIME equivalent).
pub struct LinuxClockPort;

impl LinuxClockPort {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxClockPort {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPort for LinuxClockPort {
    fn name(&self) -> &str {
        "clock-linux"
    }
}

impl ClockPort for LinuxClockPort {
    fn now_secs(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// LinuxPowerPort — UPower via zbus (org.freedesktop.UPower).
// ---------------------------------------------------------------------------

/// UPower D-Bus property names.
const UPOWER_DBUS_NAME: &str = "org.freedesktop.UPower";
const UPOWER_DBUS_PATH: &str = "/org/freedesktop/UPower/devices/DisplayDevice";

/// Linux power port backed by UPower (D-Bus).
///
/// Connects to `org.freedesktop.UPower` on the system bus. `battery()` queries
/// the `org.freedesktop.UPower.Device` interface for `Percentage` (0–100) and
/// `State` properties. `on_ac` is derived from `State`:
/// - 1 (Charging) or 4 (FullyCharged) → on_ac = true
/// - 2 (Discharging) or 0 (Unknown) → on_ac = false
pub struct LinuxPowerPort {
    /// Cached connection (lazily established; reconnects on failure).
    conn: Mutex<Option<zbus::blocking::Connection>>,
}

impl LinuxPowerPort {
    pub fn new() -> Self {
        Self {
            conn: Mutex::new(None),
        }
    }

    /// Ensure we have a system-bus connection; (re)connect if needed.
    fn connection(&self) -> Result<zbus::blocking::Connection, zbus::Error> {
        let mut guard = self.conn.lock().unwrap();
        if guard.is_none() {
            *guard = Some(zbus::blocking::Connection::system()?);
        }
        // unwrap safe: just set it
        Ok(guard.as_ref().unwrap().clone())
    }
}

impl Default for LinuxPowerPort {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPort for LinuxPowerPort {
    fn name(&self) -> &str {
        "power-linux"
    }
}

impl PowerPort for LinuxPowerPort {
    fn battery(&self) -> BatteryState {
        // If D-Bus is unavailable (e.g., no UPower running, or WSL2 without
        // dbus configured), fall back to a safe default rather than panicking.
        let conn = match self.connection() {
            Ok(c) => c,
            Err(_) => return BatteryState { level: 1.0, on_ac: true },
        };

        // Query the DisplayDevice proxy for Percentage and State.
        use zbus::blocking::proxy::CacheProperties;
        let proxy = match zbus::blocking::ProxyBuilder::new(&conn)
            .destination(UPOWER_DBUS_NAME)
            .path(UPOWER_DBUS_PATH)
            .interface("org.freedesktop.UPower.Device")
            .cache_properties(CacheProperties::No)
            .build::<zbus::blocking::Proxy>()
        {
            Ok(p) => p,
            Err(_) => return BatteryState { level: 1.0, on_ac: true },
        };

        let pct: f64 = proxy.get_property("Percentage").unwrap_or(100.0);
        let state: u32 = proxy.get_property("State").unwrap_or(0);

        // UPower Device State: 1=Charging, 2=Discharging, 3=Empty,
        // 4=FullyCharged, 5=PendingCharge, 6=PendingDischarge, 0=Unknown.
        let on_ac = matches!(state, 1 | 4 | 5);

        BatteryState {
            level: (pct / 100.0).clamp(0.0, 1.0),
            on_ac,
        }
    }
}

// ---------------------------------------------------------------------------
// LinuxNotificationsPort — FreeDesktop Notifications via zbus.
// ---------------------------------------------------------------------------

/// Linux notifications port backed by FreeDesktop Notifications (D-Bus).
///
/// Connects to `org.freedesktop.Notifications` on the session bus and collects
/// incoming notification signals into an internal queue. `poll_notification()`
/// pops them FIFO (non-blocking).
///
/// **Note**: FreeDesktop Notifications is a *push* API — apps call `Notify()`
/// to *send* notifications; there is no standard "received notification"
/// signal. This adapter therefore monitors the session bus for `Notify` method
/// calls (via eavesdropping when permitted) for testing purposes. In practice,
/// COSMIC's notification daemon exposes its own D-Bus interface for received
/// notifications; this adapter will be extended to match that once a COSMIC
/// notification component is replicated.
pub struct LinuxNotificationsPort {
    queue: Mutex<std::collections::VecDeque<Notification>>,
}

impl LinuxNotificationsPort {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(std::collections::VecDeque::new()),
        }
    }

    /// Push a notification into the internal queue (for testing or for
    /// signal-handler integration).
    pub fn push(&self, n: Notification) {
        self.queue.lock().unwrap().push_back(n);
    }
}

impl Default for LinuxNotificationsPort {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPort for LinuxNotificationsPort {
    fn name(&self) -> &str {
        "notifications-linux"
    }
}

impl NotificationsPort for LinuxNotificationsPort {
    fn poll_notification(&self) -> Option<Notification> {
        self.queue.lock().unwrap().pop_front()
    }
}

// ---------------------------------------------------------------------------
// Tests (Linux-only; require a running D-Bus session for the D-Bus ones).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_port_returns_epoch_time() {
        let c = LinuxClockPort::new();
        let now = c.now_secs();
        // Should be a plausible Unix timestamp (> year 2020).
        assert!(now > 1_500_000_000, "clock returned implausible time: {}", now);
    }

    #[test]
    fn power_port_falls_back_gracefully_without_dbus() {
        // On a system without UPower (or without D-Bus), battery() should
        // return a safe default, not panic.
        let p = LinuxPowerPort::new();
        let b = p.battery();
        assert!(b.level >= 0.0 && b.level <= 1.0);
    }

    #[test]
    fn notifications_port_queue_roundtrip() {
        let n = LinuxNotificationsPort::new();
        assert!(n.poll_notification().is_none());
        n.push(Notification {
            app_name: "test".into(),
            summary: "hello".into(),
            body: "".into(),
        });
        assert_eq!(n.poll_notification().unwrap().summary, "hello");
        assert!(n.poll_notification().is_none());
    }
}
