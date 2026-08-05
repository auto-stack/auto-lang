//! Mock implementations of system ports — scriptable fakes for Windows dev.
//!
//! Each mock holds a `Mutex`-protected queue of scripted events/states. Tests
//! and the dev host push fixtures in; the app reads them out.

use std::sync::Mutex;
use crate::{BatteryState, ClockPort, Notification, NotificationsPort, PowerPort, SystemPort};

/// A mock power port. Push battery states in; `battery()` returns the latest.
#[derive(Debug)]
pub struct MockPowerPort {
    state: Mutex<BatteryState>,
}

impl MockPowerPort {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(BatteryState {
                level: 1.0,
                on_ac: true,
            }),
        }
    }

    /// Script the next battery state the app will see.
    pub fn set_battery(&self, level: f64, on_ac: bool) {
        *self.state.lock().unwrap() = BatteryState { level, on_ac };
    }
}

impl Default for MockPowerPort {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPort for MockPowerPort {
    fn name(&self) -> &str {
        "power"
    }
}

impl PowerPort for MockPowerPort {
    fn battery(&self) -> BatteryState {
        self.state.lock().unwrap().clone()
    }
}

/// A mock clock port. Set a fixed time; `now_secs()` returns it.
#[derive(Debug)]
pub struct MockClockPort {
    time: Mutex<u64>,
}

impl MockClockPort {
    pub fn new() -> Self {
        Self {
            time: Mutex::new(0),
        }
    }

    /// Script the wall-clock time the app will see (seconds since epoch).
    pub fn set_time(&self, secs: u64) {
        *self.time.lock().unwrap() = secs;
    }

    /// Advance the mock clock by `delta` seconds (simulate a tick).
    pub fn advance(&self, delta: u64) {
        *self.time.lock().unwrap() += delta;
    }
}

impl Default for MockClockPort {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPort for MockClockPort {
    fn name(&self) -> &str {
        "clock"
    }
}

impl ClockPort for MockClockPort {
    fn now_secs(&self) -> u64 {
        *self.time.lock().unwrap()
    }
}

/// A mock notifications port. Push notifications in; `poll_notification()`
/// pops them FIFO.
#[derive(Debug)]
pub struct MockNotificationsPort {
    queue: Mutex<std::collections::VecDeque<Notification>>,
}

impl MockNotificationsPort {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(std::collections::VecDeque::new()),
        }
    }

    /// Script a notification the app will receive.
    pub fn push(&self, n: Notification) {
        self.queue.lock().unwrap().push_back(n);
    }
}

impl Default for MockNotificationsPort {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPort for MockNotificationsPort {
    fn name(&self) -> &str {
        "notifications"
    }
}

impl NotificationsPort for MockNotificationsPort {
    fn poll_notification(&self) -> Option<Notification> {
        self.queue.lock().unwrap().pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_power_roundtrip() {
        let p = MockPowerPort::new();
        assert_eq!(p.battery().level, 1.0);
        assert!(p.battery().on_ac);
        p.set_battery(0.42, false);
        assert!((p.battery().level - 0.42).abs() < 1e-9);
        assert!(!p.battery().on_ac);
    }

    #[test]
    fn mock_clock_advance() {
        let c = MockClockPort::new();
        c.set_time(3661); // 01:01:01
        assert_eq!(c.hms(), (1, 1, 1));
        c.advance(60);
        assert_eq!(c.hms(), (1, 2, 1));
    }

    #[test]
    fn mock_notifications_fifo() {
        let n = MockNotificationsPort::new();
        assert!(n.poll_notification().is_none());
        n.push(Notification {
            app_name: "a".into(),
            summary: "first".into(),
            body: "".into(),
        });
        n.push(Notification {
            app_name: "b".into(),
            summary: "second".into(),
            body: "".into(),
        });
        assert_eq!(n.poll_notification().unwrap().summary, "first");
        assert_eq!(n.poll_notification().unwrap().summary, "second");
        assert!(n.poll_notification().is_none());
    }
}
