//! PLAN-937 CA-D1: the bridge CRT toolchain closure is the A2C channel's
//! frozen flags — pinned to `crun::C_FLAGS` so the two spellings can
//! never drift (the bridge compiles with the FULL CRT closure, never
//! the freestanding probe flags).

#![cfg(windows)]

#[test]
fn bridge_crt_flags_match_the_a2c_channel() {
    let bridge: Vec<&str> = ace_abi::bridge::BRIDGE_CRT_FLAGS.to_vec();
    let crun: Vec<&str> = ace_a2x_worker::crun::C_FLAGS.to_vec();
    assert_eq!(
        bridge, crun,
        "bridge CRT flags must equal the frozen A2C flags"
    );
    assert!(
        bridge.contains(&"-MT"),
        "static CRT closure is part of the identity"
    );
}
