//! PLAN-969 AUDIT-03 (1024 F03): the escalation ladder routes on the
//! STRUCTURED surface fact (`Diagnostic::surface`), never on message
//! text. Permanent classes (compile-only — no link, no toolchain):
//! - wider-profile-owned sources still climb (numeric operator chars,
//!   aggregates shapes) and compile at the owning rung;
//! - a REAL parse error stays terminal at the producing rung with the
//!   original diagnostic (same `source.syntax` code as a tagged surface
//!   shape, but no structured fact, no climb);
//! - the message-independence property itself is unit-pinned in
//!   `native/src/driver.rs` (tagged + arbitrary text climbs; untagged +
//!   legacy-matching text stays terminal).

#[path = "../common/mod.rs"]
mod common;

/// The numeric-owned operator characters escalate from the scalar face
/// and compile at the numeric rung.
#[test]
fn audit03_numeric_owned_characters_climb() {
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    assert!(
        driver
            .check_source("fn main() int {\n    return 3 & 5\n}", &[])
            .is_ok(),
        "numeric-owned `&` climbs the ladder and compiles"
    );
}

/// The aggregates-owned shapes (tuple literal, named ctor) escalate and
/// compile at the aggregates rung (PLAN-962 spellings).
#[test]
fn audit03_aggregates_shapes_climb() {
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let src = "fn main() int {\n    let t = (19, 23)\n    let (x, y) = t\n    let p = Pair { right: 0, left: y }\n    return x + p.left\n}\ntype Pair { left int; right int }";
    assert!(
        driver.check_source(src, &[]).is_ok(),
        "aggregates tuple/ctor shapes climb the ladder and compile"
    );
}

/// A REAL parse error is terminal: it keeps the producing rung's
/// original diagnostic — the same `source.syntax` code as a tagged
/// surface shape, but no structured fact, so no climb.
#[test]
fn audit03_real_parse_error_stays_terminal() {
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let err = driver
        .check_source("fn main() int { return 2 + }", &[])
        .expect_err("a real parse error never climbs");
    let text = format!("{err:?}");
    assert!(
        text.contains("expected expression"),
        "the ORIGINAL diagnostic survives (no reroute): {text}"
    );
}
