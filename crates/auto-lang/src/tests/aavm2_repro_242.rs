// 242 / plan-432 D26 复现:VM 字符串池 RC 回归。
//
// 现象:循环体内以运行期字符串(拼接/函数返回值;字面量与循环外均正常)调
// List.push,事后 get 读回即 UAF:`[RC canary] string tombstone access: pool
// index N was freed`(debug 断言,engine.rs:1385 get_string)。master 上
// conformance_bootstrap 亦同类 canary(heap 4000001,rc.rs:482),99_bootstrap
// parser 系列测试 ignored——疑似 Plan 419/423 RC 改造的存量回归。
//
// 诊断线索(P419_TRACE_POOL=<N> 追踪):池槽 retain(0→1)/release(1→0)/FREE
// 循环复用后,复活槽的 tombstone 未清(canary 按 tombstone 判定,不看 rc);
// 见 engine.rs add_string 的 dedup 命中路径与 pool_free_idx 的键删除交互。
// 已验证不可绕:提升临时变量、for/while 形态等价、仅字面量 push 正常。
//
// 阻断:plan 432 S2 的 parse_dump 必经"语句循环内 push dump 串"路径,M2 闸门
// (aavm2_m2)因此挂 ignore;VM 修复后:①本测试去 ignore 应转绿;②aavm2_m2
// 闸门去 ignore 验闸。

use crate::run_with_capture;

const REPRO: &str = r#"fn f() str {
    var l = List.new()
    var i = 0
    while i < 3 {
        l.push("(s" + i.str() + ")")
        i = i + 1
    }
    return l.get(2)
}
fn main() {
    print(f())
    print("ok")
}
"#;

#[test]
fn repro_242_string_pool_uaf() {
    // Plan 432 D26 修复(ListData<i32> 字符串哨兵容器侧份额)后的常驻回归:
    // 修复前 canary panic,修复后输出 (s2)/ok。
    let (_r, stdout) = run_with_capture(REPRO).expect("repro should run");
    assert_eq!(stdout.trim_end(), "(s2)\nok");
}
