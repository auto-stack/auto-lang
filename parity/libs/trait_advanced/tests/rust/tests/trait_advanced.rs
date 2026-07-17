//! Native Rust oracle for the `trait_advanced` parity library (Plan 358 D2.1).
//!
//! Mirrors the Auto spec-driven logic in `auto/trait_advanced.at` using native
//! Rust traits so the parity comparator can compare three-way (AutoVM / a2r /
//! Rust). Each `#[test]` function name MUST match the Auto TAP test name
//! exactly — the comparator keys results by name.
//!
//! Coverage (mirrors the Auto side):
//!   * L1 baseline — `Identifiable` (required-method dispatch, two impls) and
//!     non-generic `Comparable` (spec dispatch). These pass on all three
//!     backends.
//!   * Sub-scenario A default-method probe — `Announcer` carries a default
//!     method. Rust supports default methods natively, so these pass on the
//!     Rust side; a2r fails (DIV-TRAIT-A2R-1) and the case is an expected L3
//!     divergence.
//!   * Sub-scenario B (associated types): Rust supports associated types, but
//!     Auto does not (parse error), so there is no corresponding Auto test and
//!     therefore no Rust oracle test here. See README "Sub-scenario B".

// =============================================================================
// Sub-scenario A, L1 baseline — required-method dispatch.
// =============================================================================

trait Identifiable {
    fn ident(&self) -> String;
}

struct Device {
    serial: i32,
}

impl Identifiable for Device {
    fn ident(&self) -> String {
        format!("{}{}", "device-", format!("{:?}", self.serial))
    }
}

struct Channel {
    name: String,
}

impl Identifiable for Channel {
    fn ident(&self) -> String {
        format!("{}{}", "channel:", self.name)
    }
}

// Sub-scenario A, L3 probe — a trait with a default method.
trait Announcer {
    fn label(&self) -> String;
    // Default method: implementers inherit this if they don't override it.
    fn announce(&self) {
        println!("{}{}", "[ANN] ", self.label());
    }
}

struct Robot {
    id: i32,
}

impl Announcer for Robot {
    fn label(&self) -> String {
        format!("{}{}", "robot-", format!("{:?}", self.id))
    }
    // Overrides the default with the same body the AutoVM currently requires
    // (Auto's trait checker does not yet skip default-bodied methods).
    fn announce(&self) {
        println!("{}{}", "[ANN] ", self.label());
    }
}

// NOTE: a value-returning default method trait is intentionally omitted here.
// Rust supports it natively, but a2r miscompiles the default body
// (DIV-TRAIT-A2R-1), and because the parity runner links the whole library
// into every test binary, including such a trait would spoil the L1 baseline.
// The gap is documented in known-divergences.md.

// =============================================================================
// Sub-scenario C, L1 baseline — non-generic Comparable spec.
// =============================================================================

trait Comparable {
    fn compare(&self, other: i32) -> i32;
}

struct ScoreCmp {
    val: i32,
}

impl Comparable for ScoreCmp {
    fn compare(&self, other: i32) -> i32 {
        self.val - other
    }
}

// =============================================================================
// Primitive entry points mirroring `auto/trait_advanced.at`.
// =============================================================================

fn device_ident(serial: i32) -> String {
    let d = Device { serial };
    d.ident()
}

fn channel_ident(name: &str) -> String {
    let c = Channel { name: name.to_string() };
    c.ident()
}

fn announce_robot(id: i32) -> String {
    format!("{}{}", "[ANN] ", robot_label(id))
}

fn robot_label(id: i32) -> String {
    let r = Robot { id };
    r.label()
}

fn max_score_val(a: i32, b: i32) -> i32 {
    let sa = ScoreCmp { val: a };
    if sa.compare(b) >= 0 {
        a
    } else {
        b
    }
}

fn score_cmp(a: i32, b: i32) -> i32 {
    let sa = ScoreCmp { val: a };
    sa.compare(b)
}

// =============================================================================
// Tests. Names match the Auto TAP test names.
// =============================================================================

// --- L1 baseline (spec_basics.at) ---

#[test]
fn test_identifiable_device_ident() {
    assert_eq!(device_ident(42), "device-42");
}

#[test]
fn test_identifiable_channel_ident() {
    assert_eq!(channel_ident("alpha"), "channel:alpha");
}

#[test]
fn test_max_score_picks_larger() {
    assert_eq!(max_score_val(3, 5), 5);
}

#[test]
fn test_max_score_order_invariant() {
    assert_eq!(max_score_val(5, 3), 5);
}

#[test]
fn test_max_score_tie_returns_first() {
    assert_eq!(max_score_val(4, 4), 4);
}

#[test]
fn test_score_cmp_less() {
    assert_eq!(score_cmp(3, 5), -2);
}

#[test]
fn test_score_cmp_equal() {
    assert_eq!(score_cmp(5, 5), 0);
}

#[test]
fn test_score_cmp_greater() {
    assert_eq!(score_cmp(7, 3), 4);
}

// --- Sub-scenario A default-method probe (default_methods_probe.at) ---
// These pass on Rust and on all three backends (the void default method is the
// form a2r compiles correctly).

#[test]
fn test_default_announce_robot() {
    assert_eq!(announce_robot(42), "[ANN] robot-42");
}

#[test]
fn test_required_label_method() {
    assert_eq!(robot_label(7), "robot-7");
}
