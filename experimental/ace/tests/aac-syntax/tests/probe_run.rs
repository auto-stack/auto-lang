//! PLAN-1050 probe: run Auto-produced ABC through the independent Rust
//! consumer. PROBE_ABC points at the ABC file, PROBE_EXPECT the expected
//! i32 value.
use ace_av_core::mir::OwnerScope;

#[test]
fn auto_abc_decodes_verifies_and_executes() {
    let bytes = std::fs::read(std::env::var("PROBE_ABC").unwrap()).unwrap();
    let decoded = ace_av_core::decode_abc(&bytes).expect("decode Auto ABC");
    let schema = std::env::var("PROBE_SCHEMA").unwrap_or_else(|_| {
        if bytes.len() >= 6 && bytes[4] == 1 && bytes[5] == 0 {
            "auto.mir.av-data/1"
        } else {
            "auto.mir.av-owned/1"
        }
    }
    .to_string());
    let loaded = ace_av_core::verify_abc(
        decoded,
        &OwnerScope {
            schema,
            module: "app.main".into(),
        },
    )
    .expect("independent consumer admission");
    let (out, _) = ace_av_core::execute(&loaded, &Default::default(), &Default::default());
    println!("outcome: {out:?}");
    let want: i32 = std::env::var("PROBE_EXPECT")
        .unwrap()
        .parse()
        .unwrap();
    assert!(matches!(
        out,
        ace_av_core::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(v),
            ..
        } if v == want
    ));
}
