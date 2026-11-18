//! PLAN-982（④ S2-D）：**端到端**宽标量——`ratio: 1.5` / `big: 18446744073709551615`。
//!
//! 链条：门控数值面（`AUTO_ACE_NUMBER_SURFACE`）⇒ 投影器把"恰好一个 affix-num 叶"
//! 的值升成 `ConfigPairValue::Scalar` ⇒ 物化器发射 `cfg_put_f64_text` /
//! `cfg_put_u64_text`（十进制文本传值）⇒ 会话写 `StoredValue::{F64,U64}` ⇒
//! 投影成 `ConfigValue::{F64(bits),U64}`。
//!
//! 默认（门控关）仍是 `source.unsupported` —— 与今天的表面逐字一致。

use std::sync::Mutex;

use auto_lang_core::ace_config::{AceConfigOutcome, AceConfigRequest, ConfigValue};
use auto_lang_core::eval_config_with_ace;

const GATE: &str = "AUTO_ACE_NUMBER_SURFACE";

/// 门控是**进程级**环境变量，用例必须串行。
static LOCK: Mutex<()> = Mutex::new(());

fn outcome_of(source: &str) -> AceConfigOutcome {
    let _ = auto_lang_ace_host::install();
    let request = AceConfigRequest::simple(source);
    eval_config_with_ace(&request).outcome
}

#[test]
fn p982_wide_scalars_are_refused_by_default() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var(GATE);
    match outcome_of("ratio: 1.5\n") {
        AceConfigOutcome::Success { .. } => {
            panic!("the default surface must not accept float literals")
        }
        other => {
            let text = format!("{other:?}");
            assert!(
                text.contains("source.unsupported") || text.contains("unsupported"),
                "the default refusal is named: {text}"
            );
        }
    }
}

#[test]
fn p982_wide_scalars_land_in_the_document_under_the_gate() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var(GATE, "1");
    let outcome = outcome_of("ratio: 1.5\nbig: 18446744073709551615\n");
    std::env::remove_var(GATE);

    let document = match outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("the gate must accept both wide scalars: {other:?}"),
    };
    assert_eq!(
        document.root.prop("ratio"),
        Some(&ConfigValue::F64(1.5f64.to_bits())),
        "the float lands bit-exact"
    );
    assert_eq!(
        document.root.prop("big"),
        Some(&ConfigValue::U64(u64::MAX)),
        "the wide integer lands as u64"
    );
}
