//! PLAN-936（T-04）：有限 CT 表达式面（`#{ expr }`）前门与绑定事实。
//!
//! 覆盖：能力门控（无能力仍原拒绝面）、HashExpr 投影（span 指向原源、
//! inner 原样）、scalar-ct 面包含模板表面（CT 可作用于模板实例）、
//! 未服务 HashExpr 在绑定处受控拒（绝不静默当运行时表达式）、畸形近邻。

use ace_core::source::ast::{ExprKind, Item, Stmt};
use ace_core::source::cst::{
    parse_front, parse_front_with_caps, FrontBudget, FrontKind, FrontOutcome, FrontRequest,
    CT_CAPABILITY, INSTANCES_CAPABILITY,
};
use ace_core::source::{resolve_with, SourceProfile};

const SUM_HASH: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    let n = #{ sum(20, 22) }
    return n
}
";

const CHOOSE_HASH: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    return #{ choose(40, 42, false) }
}
";

fn req(src: &str) -> FrontRequest<'_> {
    FrontRequest {
        src,
        profile: SourceProfile::Scalar,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    }
}

fn with_ct(src: &str) -> FrontOutcome {
    parse_front_with_caps(req(src), &[CT_CAPABILITY.to_string()])
}

fn main_body<'a>(p: &'a ace_core::source::ast::Program) -> &'a ace_core::source::ast::Block {
    p.items
        .iter()
        .find_map(|i| match i {
            Item::Fn(f) if f.name.name == "main" => Some(&f.body),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no fn main"))
}

/// 能力在：`#{ sum(20, 22) }` 投影为 HashExpr 原子，inner 调用原样
/// （span 指向原源），caps_version 换键为 ac-source-front/3。
#[test]
fn parse_hash_expr_with_capability() {
    let FrontOutcome::Ok(ok) = with_ct(SUM_HASH) else {
        panic!("sum/hash must parse under the CT capability");
    };
    assert_eq!(ok.caps_version, "ac-source-front/3");
    let body = main_body(&ok.program);
    let Stmt::Let { value, .. } = &body.stmts[0] else {
        panic!("first stmt is the let");
    };
    let ExprKind::HashExpr { expr } = &value.kind else {
        panic!("initializer must be a HashExpr, got {:?}", value.kind);
    };
    let ExprKind::Call { callee, args } = &expr.kind else {
        panic!("inner must be the plain call, got {:?}", expr.kind);
    };
    assert_eq!(callee.name, "sum");
    assert_eq!(args.len(), 2);
    assert!(value.span.start <= expr.span.start && expr.span.end <= value.span.end);
    // 第二条语句原样（return n）。
    assert!(matches!(&body.stmts[1], Stmt::Return { .. }));
}

/// 无能力：同一源码在 plain scalar 裁剪下仍是语法拒绝（`#` 不是词面；
/// 原拒绝面不变，不因新增规则静默放行）。
#[test]
fn hash_rejected_without_capability() {
    match parse_front(req(SUM_HASH)) {
        FrontOutcome::Diag(_) => {}
        other => panic!("plain face must reject `#`, got {:?}", matches!(other, FrontOutcome::Ok(_))),
    }
}

/// scalar-ct 面包含 scalar-instances 全表面：CT 能力（未携 instances
/// 能力）同时解析模板 `<T>` 与 `#{ ... }`——CT 可作用于模板实例（V-04）。
#[test]
fn ct_face_subsumes_the_instances_surface() {
    let FrontOutcome::Ok(ok) = with_ct(CHOOSE_HASH) else {
        panic!("choose/hash must parse under the CT capability alone");
    };
    let choose = ok
        .program
        .items
        .iter()
        .find_map(|i| match i {
            Item::Fn(f) if f.name.name == "choose" => Some(f.type_params.len()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no fn choose"));
    assert_eq!(choose, 1, "the template type parameter projects");
    let body = main_body(&ok.program);
    let Stmt::Return { value, .. } = &body.stmts[0] else {
        panic!("first stmt is the return");
    };
    assert!(matches!(value.kind, ExprKind::HashExpr { .. }));
}

/// instances 能力（无 CT 能力）：`#{ ... }` 仍是语法拒绝——面不叠加。
#[test]
fn instances_capability_alone_does_not_admit_hash() {
    let out = parse_front_with_caps(
        req(SUM_HASH),
        &[INSTANCES_CAPABILITY.to_string()],
    );
    assert!(
        matches!(out, FrontOutcome::Diag(_)),
        "the instances face must not admit `#`"
    );
}

/// 未服务 HashExpr 在绑定处受控拒：精确 code、Bind 阶段——绝不静默当
/// 运行时表达式下跑（服务/回灌是 T-06/T-07 的 CT 车道职责）。
#[test]
fn unserved_hash_is_a_controlled_bind_rejection() {
    let FrontOutcome::Ok(ok) = with_ct(SUM_HASH) else {
        panic!("parse under CT capability");
    };
    let err = match resolve_with(&ok.program, &[], SourceProfile::Scalar) {
        Err(ds) => ds,
        Ok(_) => panic!("unserved HashExpr must be rejected at bind"),
    };
    let d = &err[0];
    assert_eq!(d.code, "source.cte.unserved");
    assert!(matches!(d.stage, ace_core::atom_text::Stage::Bind));
}

/// 畸形近邻：`#` 后非 `{`、`#{` 体非单表达式/未闭合 —— 精确语法诊断。
#[test]
fn malformed_hash_neighbors_are_syntax_rejections() {
    for src in [
        "fn main() int {\n    return # sum(1, 2)\n}\n",
        "fn main() int {\n    return #{ let x = 1 }\n}\n",
        "fn main() int {\n    return #{ sum(1, 2)\n}\n",
    ] {
        let out = with_ct(src);
        assert!(
            matches!(out, FrontOutcome::Diag(_)),
            "malformed hash neighbor must be a syntax rejection: {src:?}"
        );
    }
}
