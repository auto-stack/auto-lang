//! PLAN-1000 R05:avm — `move` 闭包按值捕获的源级回归锁。
//!
//! 修复前：`compile_closure` 无视 `Closure.is_move`（Plan 364 W5 D4），
//! move 闭包仍发射 Plan 385 by-ref 槽位；creator 帧返回后调用即触发
//! PLAN-667 (F-01) 帧存活守卫 —— K02 creator-exit 源（native-closures-924
//! /resource-915/legacy-a2r-resources-998 的 `return move (n int) => …`）
//! 在 legacy VM 上以 `RuntimeError(captured variable … creator frame has
//! already returned …)` 拒绝，R05:avm 无法闭合。
//!
//! 修复后：`is_move` 闭包全部捕获发射 slot_offset=0xFFFF（env 按值副本，
//! 与 Rust `move |…|` 语义一致）；非 move 闭包保持 Plan 385 by-ref 槽位
//! 及其帧存活守卫。
//!
//! 覆盖面：
//! - K02 creator-exit 正例：move 闭包随返回值逃逸 creator 帧，存活且值
//!   正确（42）；
//! - 反控：同一源去掉 `move`（by-ref 闭包逃逸 creator）仍被帧守卫拒绝
//!   ——判面是捕获方式，不是闭包语法本身；
//! - 生存期内调用（corpus K01/K06 形态）在 base 组 case 检查回归
//!  （legacy-composite autovm 29/29），不在此重复。

#![cfg(test)]

/// K02-924.at 原封源（native-closures-924 冻结 fixture，R02/R05 共享）。
const K02_MOVE: &str = "fn make(move s Str) fn(int) int {\n    return move (n int) => { return s.len() + n }\n}\nfn main() int {\n    let s Str = \"hello\"\n    let f = make(s.move)\n    return f(37)\n}\n";

/// 反控：同源去掉闭包前缀 `move`（by-ref 捕获逃逸 creator 帧）。
const K02_BYREF: &str = "fn make(move s Str) fn(int) int {\n    return (n int) => { return s.len() + n }\n}\nfn main() int {\n    let s Str = \"hello\"\n    let f = make(s.move)\n    return f(37)\n}\n";

#[test]
fn k02_move_closure_survives_creator_return() {
    let (result, _) = crate::run_with_capture_and_path_staged(K02_MOVE, "main.at")
        .expect("move closure must run after the creator frame returned");
    assert_eq!(result.trim(), "42", "s.len()+n = 5+37");
}

#[test]
fn byref_closure_escape_still_refused() {
    // 反控：非 move 闭包的 by-ref 捕获在 creator 帧返回后必须仍被
    // PLAN-667 (F-01) 帧存活守卫拒绝——证明修复只放宽 move 语义，
    // 没有削弱 by-ref 逃逸防线。
    let staged = crate::run_with_capture_and_path_staged(K02_BYREF, "main.at")
        .expect_err("by-ref closure escape must be refused");
    let msg = format!("{}", staged.error);
    assert!(
        msg.contains("creator frame has already returned"),
        "frame-alive guard must fire, got: {msg}"
    );
}
