//! PLAN-734：API 契约与生成一致性。
//!
//! T-01 原型探针（VM i64 语义/JSON 往返/编组现状）随任务推进扩充为
//! §6.1 支持与负向矩阵（五形态对拍在 http_e2e_plan734 族）。

// ============================================================================
// T-01 探针：VM int 的实际宽度语义（bind 截断修复的依据）
// ============================================================================

mod plan734_probe {
    /// VM 字面量/算术/JSON 往返在超 i32 范围 int 上的真实行为：
    /// - 字面量 5_000_000_000（> i32::MAX）压栈后 print 输出什么；
    /// - 大数加法；
    /// - json.parse 大整数 → get_int；
    /// - json 序列化往返。
    #[test]
    fn plan734_probe_vm_i64_literal_arith_json() {
        let code = r#"
fn main() {
    let x = 5000000000
    print(x)
    let y = x + 1
    print(y)
    let s = json.encode(y)
    print(s)
}
"#;
        let (_, out) = crate::run_with_capture(code).expect("run");
        eprintln!("--- i64 probe output ---\n{out}\n--- end ---");
        // 记录现状（T-01 决策表输入）：断言留到决策冻结后收紧。
        assert!(!out.is_empty());
    }

/// VM 中 int 值的字符串化边界（E1 后续）：字面量拼接走 str 转换路径。
    #[test]
    fn plan734_probe_vm_i64_roundtrip() {
        let code = r#"
fn main() {
    let x = 5000000000
    let s = "id=" + x
    print(s)
}
"#;
        let (_, out) = crate::run_with_capture(code).expect("run");
        eprintln!("--- roundtrip output ---\n{out}\n--- end ---");
        assert!(!out.is_empty());
    }
}
