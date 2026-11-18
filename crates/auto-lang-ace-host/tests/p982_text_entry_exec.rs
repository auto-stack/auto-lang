//! PLAN-982（983 文本面 · 步骤 1）：**受检文本入口的宿主层实测**。
//!
//! `fn main(q str) int { return q.len_bytes() }` 经
//! Text profile 编译 → 受检 lowering → MIR/ABC 验证 → AV 真执行，
//! 传入 `中文`（6 字节 / 2 标量）返回 **6**。
//!
//! 负例：旧入口（无 `ac.runtime.text.v1`）拒绝文本实参；错签名与非法 UTF‑8
//! 进入 handler 前就被拒绝；执行后 arena 收支归零。

use ace_av_core::av::{execute_entry, execute_entry_text, CancelFlag, ExecLimits};
use ace_av_core::mir::{MirModule, MirType};

fn compile_text_entry(src: &str) -> MirModule {
    let caps = vec!["ac.runtime.text.v1".to_string()];
    let product = ace_core::source::compile_text_source(src, "p982-text-entry.at", &caps)
        .expect("the Text profile compiles the handler");
    ace_lowering::hir_adapter::lower_checked_owned(&product.checked, "d_main", 0)
        .expect("the text entry lowers")
}

fn admit(mir: &MirModule) -> ace_av_core::abc_verify::LoadedModule {
    let encoded = ace_av_core::abc::encode_abc(mir).expect("encode").to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).expect("decode");
    ace_av_core::abc_verify::verify_abc(decoded, &mir.owner).expect("admit")
}

#[test]
fn text_entry_returns_the_byte_length_of_chinese_text() {
    let mir = compile_text_entry("fn main(q str) int {\n  return q.len_bytes()\n}\n");
    assert!(
        mir.has_text_entry_capability(),
        "the module carries the text-entry capability: {:?}",
        mir.capabilities
    );
    let loaded = admit(&mir);
    let limits = ExecLimits::default();
    let cancel = CancelFlag::new(false);
    let (outcome, trace) = execute_entry_text(&loaded, &limits, &cancel, &["中文".as_bytes()]);
    let live = trace.arena.as_ref().map(|a| a.live()).unwrap_or(0);
    eprintln!("p982 text entry: {outcome:?} live={live}");
    assert!(
        format!("{outcome:?}").contains("I32(6)"),
        "`中文` is 6 UTF-8 bytes: {outcome:?}"
    );
    assert_eq!(live, 0, "no text allocation outlives the entry");
}

#[test]
fn the_generic_entry_still_refuses_text_arguments() {
    let mir = compile_text_entry("fn main(q str) int {\n  return q.len_bytes()\n}\n");
    let loaded = admit(&mir);
    let limits = ExecLimits::default();
    let cancel = CancelFlag::new(false);
    // 通用入口只收标量实参；`Value::I32` 填 `StrView` 槽必须被拒。
    let (outcome, _trace) = execute_entry(
        &loaded,
        &limits,
        &cancel,
        &[ace_av_core::contract::Value::I32(6)],
    );
    let rendered = format!("{outcome:?}");
    eprintln!("p982 generic entry with text slot: {rendered}");
    assert!(
        rendered.contains("admission violation") || rendered.contains("Infra"),
        "the generic entry keeps its scalar-only contract: {rendered}"
    );
}

#[test]
fn invalid_utf8_is_refused_before_the_handler_runs() {
    let mir = compile_text_entry("fn main(q str) int {\n  return q.len_bytes()\n}\n");
    let loaded = admit(&mir);
    let limits = ExecLimits::default();
    let cancel = CancelFlag::new(false);
    let (outcome, _trace) = execute_entry_text(&loaded, &limits, &cancel, &[&[0xffu8, 0xfe]]);
    eprintln!("p982 invalid utf8: {outcome:?}");
    assert!(
        format!("{outcome:?}").contains("UTF-8"),
        "invalid UTF-8 is refused before execution: {outcome:?}"
    );
}

#[test]
fn the_text_param_is_a_borrowed_view() {
    let mir = compile_text_entry("fn main(q str) int {\n  return q.len_bytes()\n}\n");
    let params = mir.sig_of(mir.funcs[mir.entry as usize].sig).expect("sig");
    let names: Vec<String> = params
        .params
        .iter()
        .map(|p| format!("{:?}", mir.types.get(*p as usize)))
        .collect();
    eprintln!("p982 text entry params: {names:?}");
    assert!(
        names.iter().any(|n| n.contains("StrView")),
        "the entry param is a borrowed view, not owned Str: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n == "Some(Str)"),
        "owned Str is not the entry param: {names:?}"
    );
    let _ = MirType::StrView;
}
#[test]
fn a_budget_exhausted_run_is_reclaimed_and_leaves_no_text() {
    let mir = compile_text_entry("fn main(q str) int {\n  return q.len_bytes()\n}\n");
    let loaded = admit(&mir);
    let limits = ExecLimits {
        max_steps: 1,
        ..ExecLimits::default()
    };
    let cancel = CancelFlag::new(false);
    let (outcome, trace) = execute_entry_text(&loaded, &limits, &cancel, &["中文".as_bytes()]);
    let live = trace.arena.as_ref().map(|a| a.live()).unwrap_or(0);
    eprintln!("p982 over budget: {outcome:?} live={live}");
    assert!(
        format!("{outcome:?}").contains("Resource"),
        "the budget is enforced: {outcome:?}"
    );
    assert_eq!(live, 0, "a reclaimed run leaves no text behind");
}

#[test]
fn an_owned_str_param_is_not_the_text_entry_contract() {
    // `Str`（拥有值）不是受检文本入口的形参类型：这条签名必须在 lowering/验证
    // 阶段被拒，而不是"跑起来再算"。
    let caps = vec!["ac.runtime.text.v1".to_string()];
    let product = ace_core::source::compile_text_source(
        "fn main(q Str) int {\n  return 0\n}\n",
        "p982-owned.at",
        &caps,
    )
    .expect("the Text profile parses the source");
    let lowered = ace_lowering::hir_adapter::lower_checked_owned(&product.checked, "d_main", 0);
    let message = match lowered {
        Ok(mir) => {
            // 验证关口必须拒绝持有值 `Str` 形参（只有 `StrView` 才受检放行）。
            match ace_av_core::mir_verify::verify_mir(&mir) {
                Ok(_) => "verify admitted an owned Str param (unexpected)".to_string(),
                Err(e) => format!("verify rejected: {e:?}"),
            }
        }
        Err(e) => format!("lowering rejected: {e:?}"),
    };
    eprintln!("p982 owned Str param: {message}");
    assert!(
        message.contains("rejected"),
        "an owned Str param is not admitted as the borrowed text entry: {message}"
    );
}
