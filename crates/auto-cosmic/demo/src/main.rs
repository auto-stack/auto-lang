//! Plan 365 W2: demo clock+battery applet.
//!
//! A COSMIC-style applet (clock + battery indicator) driven by mock system
//! ports. Runs on Windows via the headless dev host (Plan 365 Host ①). The
//! same component would run under the libcosmic host (W3) on Linux with real
//! ports (W4) — the app core is platform-neutral.
//!
//! This binary exercises the full mock loop: script events → render VTree via
//! `HostBackend::Headless`.

use auto_cosmic_ports::mock::{MockClockPort, MockPowerPort};
use auto_cosmic_ports::{ClockPort, PowerPort};
use auto_lang::ui::host::HostBackend;
use auto_lang::ui::{Component, View};
use std::sync::Arc;

/// Messages the applet responds to.
#[derive(Clone, Debug)]
enum AppletMsg {
    /// Fired every second by the clock port (mocked).
    Tick,
}

/// Clock + battery applet state.
#[derive(Debug)]
struct ClockBatteryApplet {
    clock: Arc<MockClockPort>,
    power: Arc<MockPowerPort>,
}

impl Default for ClockBatteryApplet {
    fn default() -> Self {
        let clock = Arc::new(MockClockPort::new());
        let power = Arc::new(MockPowerPort::new());
        // Script a realistic starting state: 12:00:00, 78% battery, not charging.
        clock.set_time(12 * 3600);
        power.set_battery(0.78, false);
        Self { clock, power }
    }
}

impl Component for ClockBatteryApplet {
    type Msg = AppletMsg;

    fn on(&mut self, _msg: Self::Msg) {
        // On each tick, advance the mock clock by 1 second (simulating real time).
        self.clock.advance(1);
    }

    fn view(&self) -> View<AppletMsg> {
        let (h, m, s) = self.clock.hms();
        let bat = self.power.battery();
        let pct = (bat.level * 100.0).round() as u32;
        let label = if bat.on_ac {
            format!("{:02}:{:02}:{:02}  ⚡ {}%", h, m, s, pct)
        } else {
            format!("{:02}:{:02}:{:02}  🔋 {}%", h, m, s, pct)
        };
        View::Text {
            content: label,
            style: None,
        }
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    let app = ClockBatteryApplet::default();

    // Print initial state (proves the mock ports work on Windows).
    let (h, m, s) = app.clock.hms();
    let bat = app.power.battery();
    println!(
        "ClockBatteryApplet: {:02}:{:02}:{:02}, battery {:.0}%, AC={}",
        h, m, s, bat.level * 100.0, bat.on_ac
    );

    // Script a few ticks + a battery change to exercise the mock loop.
    app.clock.advance(120); // +2 minutes
    app.power.set_battery(0.76, false);

    let (h2, m2, s2) = app.clock.hms();
    let bat2 = app.power.battery();
    println!(
        "After 2 min: {:02}:{:02}:{:02}, battery {:.0}%",
        h2, m2, s2, bat2.level * 100.0
    );

    // Render via the unified HostBackend (W1 seam) — headless path on Windows.
    HostBackend::Headless.run::<ClockBatteryApplet>()
}
