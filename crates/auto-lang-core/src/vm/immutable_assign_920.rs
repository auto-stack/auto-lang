//! PLAN-920 旧 AVM 不可变赋值拒绝的 typed 事实回归锁（V-01/02/03 源级）。
//!
//! 修复前：三处 codegen 拒绝分支返回白文本 `AutoError::Msg("Cannot
//! reassign to immutable variable …")`——父侧 `classify_staged` 按类型把裸
//! Msg 判为 Infrastructure，B18 avm 语言负例从未成立（gap-audit F-INFRA-3，
//! 2026-11-01 908T04 重探针实录）。修复后：复用既有
//! `NameError::ImmutableAssignment`（miette code `auto_name_E0203`），
//! staged 与非 staged 生产入口同型传播。
//!
//! 覆盖面（§6 V-01/02/03）：
//! - 裸赋值 `x = 2`（expr_ops.rs 表达式分支 = B18 命中位）；
//! - 复合赋值 `x += 1`（expr_ops.rs 复合分支）；
//! - Store 语句防御位（stmt_emit.rs，AST 级证人——当前语法仅
//!   StoreKind::Field 可入该分支且 Field 语句位由 TypeDecl 路径编译，
//!   生产不可达，故 D-02 以 Codegen API 级证人锁族一致性，不伪造源级用例）；
//! - 合法近邻不误拒：NB18（var 重赋值得 2）、var 复合赋值、内层 var 遮蔽
//!   外层 let 的块内合法写；
//! - span 精度（D-01）：fn 体内与顶层脚本的重赋值语句均拿到行级 span
//!   （实证：fn 体 (34,9)、顶层 (14,5)）；(0,1) 退化仅出现在既无行号
//!   也无源文本的路径（AST 级直接调 Codegen API 的证人）；
//! - 反控（V-04 单元位）：携带与 B18 原文本完全相同的 `Msg` 仍判
//!   Infrastructure——判面靠 typed 通道，不靠文本匹配。

#![cfg(test)]

use crate::error::{AutoError, NameError};
use crate::test_cases::{classify_staged, Failure, StageOutcome};

/// 原封不动的 corpus 源（crates/auto-lang/test/cases/ace-m1/src/B18.at）。
const B18: &str = "fn main() int {\n    let x int = 1\n    x = 2\n    return x\n}\n";

/// NB18 合法近邻（var 版，同目录 NB18.at）。
const NB18: &str = "fn main() int {\n    var x int = 1\n    x = 2\n    return x\n}\n";

/// 复合赋值变体（`let` + `+=`，expr_ops.rs 复合分支）。
const COMPOUND: &str = "fn main() int {\n    let x int = 1\n    x += 2\n    return x\n}\n";

/// 断言错误为 typed Name(ImmutableAssignment) 并解构出 (name, span)。
fn expect_immutable_assignment(err: &AutoError) -> (&str, &miette::SourceSpan) {
    match err {
        AutoError::Name(NameError::ImmutableAssignment { name, span }) => (name, span),
        other => panic!("expected Name(ImmutableAssignment), got: {:?}", other),
    }
}

#[test]
fn b18_plain_reassign_nonstaged_typed_name_error() {
    // V-01 非staged：真实生产入口 run_autovm_capture 拒绝为 typed Name。
    let err = crate::run_autovm_capture(B18).expect_err("B18 must be rejected");
    let (name, span) = expect_immutable_assignment(&err);
    assert_eq!(name, "x");
    // D-01 行级精度：fn 体 Block 循环在 compile_stmt 前经 emit_source_line
    // 记录行号，span = 第 3 行（"    x = 2"，9 字节）的字节范围。
    // 行1 "fn main() int {\n"=16B，行2 "    let x int = 1\n"=18B → 起点 34。
    assert_eq!(span.offset(), 34, "span must start at line 3");
    assert_eq!(span.len(), 9, "span must cover the whole `    x = 2` line");
    // miette 稳定码（父侧判面的事实来源）。
    use miette::Diagnostic as _;
    let code = match &err {
        AutoError::Name(ne) => ne.code().map(|c| c.to_string()),
        _ => None,
    };
    assert_eq!(code.as_deref(), Some("auto_name_E0203"));
}

#[test]
fn b18_staged_classified_language_type_e0203() {
    // V-01 staged：同一生产路径的 staged 孪生 + classify_staged 判面。
    let staged =
        crate::run_with_capture_and_path_staged(B18, "main.at").expect_err("B18 staged must fail");
    assert!(
        staged.run_trap_code.is_none(),
        "compile-stage rejection is not a VM trap"
    );
    match classify_staged(&staged.error, staged.run_trap_code) {
        StageOutcome::Language(Failure { stage, code, .. }) => {
            assert_eq!(stage.as_str(), "type");
            assert_eq!(code.as_deref(), Some("auto_name_E0203"));
        }
        other => panic!("B18 must classify as a language negative, got: {:?}", other),
    }
    // staged 与非 staged 同型（同一错误对象族，非测试入口特供）。
    let (name, _) = expect_immutable_assignment(&staged.error);
    assert_eq!(name, "x");
}

#[test]
fn compound_reassign_same_typed_family() {
    // V-02：复合赋值（+=）走 expr_ops.rs 复合分支，同族 typed 拒绝。
    let err = crate::run_autovm_capture(COMPOUND).expect_err("compound must be rejected");
    let (name, span) = expect_immutable_assignment(&err);
    assert_eq!(name, "x");
    assert_eq!(
        span.offset(),
        34,
        "span must start at the `    x += 2` line"
    );
    assert_eq!(span.len(), 10, "span covers `    x += 2` (10 bytes)");
}

#[test]
fn store_stmt_guard_ast_level_typed_family() {
    // V-02/D-02：stmt_emit.rs Store 语句防御位。当前语法生产不可达（仅
    // StoreKind::Field 可入且 Field 语句位由 TypeDecl 编译），以 Codegen
    // API 级证人锁族一致：先 let 声明（mutability=false + 入域），再以
    // Store{kind:Field} 语句重赋值 → 同一 typed 错误。
    use crate::ast::{Stmt, Store, StoreKind};
    let let_x = Stmt::Store(Store {
        kind: StoreKind::Let,
        name: "x".into(),
        ty: crate::ast::Type::Int,
        expr: crate::ast::Expr::Int(1),
        attrs: vec![],
        is_pub: false,
    });
    let field_reassign = Stmt::Store(Store {
        kind: StoreKind::Field,
        name: "x".into(),
        ty: crate::ast::Type::Unknown,
        expr: crate::ast::Expr::Int(2),
        attrs: vec![],
        is_pub: false,
    });
    let mut codegen = crate::vm::codegen::Codegen::new();
    codegen
        .compile_stmt(&let_x)
        .expect("let declaration compiles");
    let err = codegen
        .compile_stmt(&field_reassign)
        .expect_err("Field-kind store reassignment must be rejected");
    let (name, span) = expect_immutable_assignment(&err);
    assert_eq!(name, "x");
    // Codegen::new() 无 source_text/行号 → 退化 span（D-01 约定）。
    assert_eq!((span.offset(), span.len()), (0, 1));
}

#[test]
fn top_level_script_reassign_line_span() {
    // D-01 精度边界：顶层脚本重赋值同样拿到行级 span（实证 (14,5) =
    // 第 2 行 "x = 2"）；退化 span (0,1) 仅出现在既无行号也无源文本的
    // 路径（如上方 AST 级直接调 Codegen API 的证人）。
    let src = "let x int = 1\nx = 2\nprint(x)\n";
    let err = crate::run_autovm_capture(src).expect_err("top-level reassign must be rejected");
    let (name, span) = expect_immutable_assignment(&err);
    assert_eq!(name, "x");
    // 行1 "let x int = 1\n"=14B → 第 2 行 "x = 2" 起点 14、长 5。
    assert_eq!((span.offset(), span.len()), (14, 5));
}

#[test]
fn legal_neighbors_not_rejected() {
    // V-03：合法近邻全部可执行，无一误拒。程序入口的返回值走 result
    // 通道（legacy-display，与 corpus NB18 avm oracle 同一判面）。
    let (result, _) = crate::run_autovm_capture(NB18).expect("NB18 (var) must run");
    assert_eq!(result.trim(), "2", "NB18 main must return 2");

    let (result, out) = crate::run_autovm_capture(
        "fn main() int {\n    var x int = 1\n    x += 1\n    print(x)\n    return x\n}\n",
    )
    .expect("compound on var must run");
    assert_eq!(out.trim(), "2");
    assert_eq!(result.trim(), "2");

    // 内层 var 遮蔽外层 let：块内对内层 var 的写合法（不同作用域不误判）。
    let (_, out) = crate::run_autovm_capture(
        "fn main() int {\n    let x int = 1\n    if true {\n        var x int = 5\n        x = 6\n        print(x)\n    }\n    return x\n}\n",
    )
    .expect("inner var shadow write must run");
    assert_eq!(out.trim(), "6");

    // 不同域的 let 互不影响：第二个 let 是新声明（shadow），不是重赋值。
    // 仅锁"可执行 + 内层声明可见"——块后外层 print 的旧 VM 内层槽显示
    // 怪癖（实测 "5\n5"）是既有 let/var 槽语义，本计划不改（非目标）。
    let (_, out) = crate::run_autovm_capture(
        "fn main() int {\n    let x int = 1\n    if true {\n        let x int = 5\n        print(x)\n    }\n    print(x)\n    return x\n}\n",
    )
    .expect("inner let shadow must run");
    assert_eq!(out.lines().count(), 2, "both prints must execute");
    assert_eq!(out.lines().next(), Some("5"), "inner let shadow is visible");
}

#[test]
fn plain_msg_with_b18_text_stays_infrastructure() {
    // V-04 单元反控：携带与原 B18 完全相同文本的裸 Msg 必须仍判
    // Infrastructure——语言负例判定只认 typed 通道，不做文本特判。
    let forged = AutoError::Msg(
        "Cannot reassign to immutable variable 'x' (declared with 'let')".to_string(),
    );
    match classify_staged(&forged, None) {
        StageOutcome::Infrastructure(_) => {}
        other => panic!(
            "forged Msg text must never be a language negative: {:?}",
            other
        ),
    }
}

#[test]
fn other_family_typed_name_keeps_its_own_code() {
    // V-04 异族 typed 反控：不同 NameError 变体（E0201）判 Language/type
    // 但码各归其位——B18 的 E0203 判定靠错误类型身份，不是 Name 族群
    // 整体放行，更不是文本匹配。
    let e = AutoError::Name(NameError::UndefinedVariable {
        name: "y".into(),
        span: miette::SourceSpan::new(miette::SourceOffset::from(0usize), 1),
        suggested: None,
    });
    match classify_staged(&e, None) {
        StageOutcome::Language(Failure { stage, code, .. }) => {
            assert_eq!(stage.as_str(), "type");
            assert_eq!(
                code.as_deref(),
                Some("auto_name_E0201"),
                "each typed variant keeps its own stable code"
            );
        }
        other => panic!("typed Name must classify as language, got: {:?}", other),
    }
}

#[test]
fn run_trap_channel_does_not_cross_into_b18_compile_stage() {
    // V-04 run trap 反控：带 typed trap 的运行期失败判 Run；B18 的编译期
    // 拒绝（无 trap，b18_staged… 已证）判 Type——两条 typed 通道不串道。
    let e =
        AutoError::Msg("Cannot reassign to immutable variable 'x' (declared with 'let')".into());
    match classify_staged(&e, Some("ImmutableWriteTrap")) {
        StageOutcome::Language(Failure { stage, code, .. }) => {
            assert_eq!(stage.as_str(), "run");
            assert_eq!(code.as_deref(), Some("ImmutableWriteTrap"));
        }
        other => panic!("typed trap must classify run-stage, got: {:?}", other),
    }
}

#[test]
fn negative_repair_recheck_cycle_on_same_fixture() {
    // V-06 失败修复循环：同一夹具 负→合法修复→负，真实走生产入口。
    // 类型/原因不串（修复期合法运行得 2，再破坏仍回到同一 typed 事实），
    // 不报伪 P0；每次断言都由真实执行产生。
    let b18 = "fn main() int {\n    let x int = 1\n    x = 2\n    return x\n}\n";
    let repaired =
        "fn main() int {\n    let x int = 1\n    var x2 int = x\n    x2 = 2\n    return x2\n}\n";

    // 负：typed E0203。
    let err1 = crate::run_autovm_capture(b18).expect_err("cycle step 1: negative must reject");
    let (name, _) = expect_immutable_assignment(&err1);
    assert_eq!(name, "x");

    // 修复：合法 body 真实运行得 2（合法重验）。
    let (result, _) =
        crate::run_autovm_capture(repaired).expect("cycle step 2: repaired body must run");
    assert_eq!(result.trim(), "2");

    // 再破坏：同一 typed 事实重现（重复执行/串行回归不因上一步合法而漂移）。
    let err3 =
        crate::run_autovm_capture(b18).expect_err("cycle step 3: re-broken must reject again");
    let (name2, _) = expect_immutable_assignment(&err3);
    assert_eq!(name2, "x");
}
