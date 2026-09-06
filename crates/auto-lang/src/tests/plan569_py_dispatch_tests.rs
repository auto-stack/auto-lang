//! Plan 569 D2 (P539 根治): py 返回值方法分派动态化——codegen 决策核单测。
//!
//! 侧表三源（py-ffi 调用点 / 变量落盘 / 用户 fn 返回传播）→ obj_len(1863)
//! /obj_call(1862) 组合子发射断言（字节流反汇编扫描）；Auto 值零变化红线
//! 对照臂（str.len 静态路由不受侧表影响）。
//!
//! 编译期纯静态断言——py_native_map 的填充（handle_py_import）与组合子
//! 发射均不触 pyo3，无需 python feature。

use crate::parser::Parser;
use crate::vm::codegen::Codegen;
use crate::vm::interop::{NATIVE_INTEROP_OBJ_CALL, NATIVE_INTEROP_OBJ_LEN};
use crate::vm::opcode::OpCode;

/// 编译脚本片段，返回 codegen（供 .code 字节流与侧表检查）。
fn compile_for_test(src: &str) -> Codegen {
    let mut parser = Parser::new(src);
    let code = parser.parse().expect("parse failed");
    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    for stmt in &code.stmts {
        codegen.compile_stmt(stmt).expect("compile failed");
    }
    codegen
}

/// 字节流中是否出现 `CALL_NAT_COUNTED + native_id(u16 LE) + count`。
fn has_counted_native(codegen: &Codegen, id: u16) -> bool {
    let code = &codegen.code;
    let op = OpCode::CALL_NAT_COUNTED as u8;
    let idb = id.to_le_bytes();
    code.windows(3).any(|w| w[0] == op && w[1] == idb[0] && w[2] == idb[1])
}

/// T03-① py-ffi 调用结果变量 `.len()` → obj_len(1863)。
#[test]
fn py_var_len_routes_obj_len() {
    let cg = compile_for_test(
        "use.py numpy: arange\nvar t = arange(6)\nvar n = t.len()\n",
    );
    assert!(cg.py_typed_vars.contains("t"), "侧表变量落盘：py_typed_vars 应含 t");
    assert!(
        has_counted_native(&cg, NATIVE_INTEROP_OBJ_LEN),
        "py 接收者 .len() 应发射 obj_len(1863) CALL_NAT_COUNTED"
    );
}

/// T03-② py 接收者其他方法 → obj_call(1862)，且结果续传侧表。
#[test]
fn py_var_method_routes_obj_call() {
    let cg = compile_for_test(
        "use.py numpy: arange\nvar t = arange(6)\nvar u = t.reshape(2, 3)\n",
    );
    assert!(
        has_counted_native(&cg, NATIVE_INTEROP_OBJ_CALL),
        "py 接收者 .reshape(...) 应发射 obj_call(1862) CALL_NAT_COUNTED"
    );
    assert!(
        cg.py_typed_vars.contains("u"),
        "obj_call 结果 may_py 续传：u 应入 py_typed_vars"
    );
}

/// T03-③ Auto str `.len()` 零变化——不进组合子（仍 str.len 静态路由）。
#[test]
fn auto_str_len_unchanged() {
    let cg = compile_for_test("var s = \"abc\"\nvar n = s.len()\n");
    assert!(!cg.py_typed_vars.contains("s"));
    assert!(
        !has_counted_native(&cg, NATIVE_INTEROP_OBJ_LEN),
        "Auto str .len() 不得改发 obj_len（零回归红线）"
    );
    assert!(
        !has_counted_native(&cg, NATIVE_INTEROP_OBJ_CALL),
        "Auto str 不得误入 obj_call"
    );
}

/// T03-④ Auto fn 返回 str 的 `.len()` 不受影响（fn_may_py_returns 空）。
#[test]
fn auto_fn_return_len_unchanged() {
    let cg = compile_for_test(
        "fn get() -> str { return \"abc\" }\nvar n = get().len()\n",
    );
    assert!(cg.fn_may_py_returns.is_empty(), "Auto fn 不入 fn_may_py_returns");
    assert!(
        !has_counted_native(&cg, NATIVE_INTEROP_OBJ_LEN),
        "Auto fn 返回 str 的 .len() 不得改发 obj_len（零回归红线）"
    );
}

/// T02 用户 fn 返回 py 桥值的跨函数传导：fn 尾表达式 may_py → 调用点
/// 落盘 → `.len()` 路由 obj_len（验收标准 2 的编译期形态）。
#[test]
fn fn_return_py_propagates_to_obj_len() {
    let cg = compile_for_test(
        "use.py numpy: arange\nfn mk() -> str { return arange(6) }\nvar t = mk()\nvar n = t.len()\n",
    );
    assert!(
        cg.fn_may_py_returns.contains("mk"),
        "fn 尾表达式 may_py 应回填 fn_may_py_returns"
    );
    assert!(cg.py_typed_vars.contains("t"), "调用点传导：t 应入 py_typed_vars");
    assert!(
        has_counted_native(&cg, NATIVE_INTEROP_OBJ_LEN),
        "经用户 fn 返回的 py 值 .len() 应路由 obj_len"
    );
}

/// T02 Call 形态接收者直查：`arange(6).len()`（不经变量）。
#[test]
fn py_call_receiver_len_routes_obj_len() {
    let cg = compile_for_test("use.py numpy: arange\nvar n = arange(6).len()\n");
    assert!(
        has_counted_native(&cg, NATIVE_INTEROP_OBJ_LEN),
        "Call 接收者（py-ffi 调用点源）.len() 应路由 obj_len"
    );
}
