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
    assert_eq!(ok.caps_version, "ac-source-front/4");
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
        other => panic!(
            "plain face must reject `#`, got {:?}",
            matches!(other, FrontOutcome::Ok(_))
        ),
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
    let out = parse_front_with_caps(req(SUM_HASH), &[INSTANCES_CAPABILITY.to_string()]);
    assert!(
        matches!(out, FrontOutcome::Diag(_)),
        "the instances face must not admit `#`"
    );
}

/// 未服务 HashExpr：绑定成功（inner 完整绑定、HashExpr 类型=inner 类型），
/// typecheck 走处受控拒（精确 code、Verify 阶段）——绝不静默当运行时
/// 表达式下跑（服务/回灌是 T-06/T-07 的 CT 车道职责）。无 CT 能力的
/// 绑定车道在 Bind 阶段拒（能力门复核）。
#[test]
fn unserved_hash_binds_then_rejects_at_typecheck() {
    let FrontOutcome::Ok(ok) = with_ct(SUM_HASH) else {
        panic!("parse under CT capability");
    };
    // 绑定成功（携带 CT 能力的车道）。
    let resolved = resolve_with(
        &ok.program,
        &["ac.source.cte.scalar/v1".to_string()],
        SourceProfile::Scalar,
    )
    .expect("HashExpr binds fully under the CT capability");
    // typecheck 受控拒。
    let ds = match ace_core::source::typecheck_with(&resolved, SourceProfile::Scalar) {
        Err(ds) => ds,
        Ok(()) => panic!("unserved HashExpr must be rejected before emit"),
    };
    let d = &ds[0];
    assert_eq!(d.code, "source.cte.unserved");
    assert!(matches!(d.stage, ace_core::atom_text::Stage::Verify));
    // 无 CT 能力的绑定车道：Bind 阶段能力门拒。
    let ds = match resolve_with(&ok.program, &[], SourceProfile::Scalar) {
        Err(ds) => ds,
        Ok(_) => panic!("caps-less bind lane rejects HashExpr"),
    };
    assert_eq!(ds[0].code, "source.cte.capability");
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

// -- PLAN-936（T-04）：有限 meta selector（ct_field_count/ct_has_field）----

const META_OK: &str = "\
type Pair {
    x int
    y int
}
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(ct_field_count(Pair), 40) }
}
";

fn with_ct_meta(src: &str) -> FrontOutcome {
    parse_front_with_caps(
        req(src),
        &[CT_CAPABILITY.to_string(), "cte.meta.fields/v1".to_string()],
    )
}

/// 完整合法链：record 声明 + meta selector + HashExpr 绑定成功（inner 的
/// MetaQuery/Call 均完整绑定；类型 I32）。执行（meta_query 计数与 AV 实参
/// 回灌）属 T-05/T-06 查询层。
#[test]
fn meta_field_count_binds_inside_hash() {
    let FrontOutcome::Ok(ok) = with_ct_meta(META_OK) else {
        panic!("meta/hash module must parse under CT+meta capabilities");
    };
    let resolved = resolve_with(
        &ok.program,
        &[
            "ac.source.cte.scalar/v1".to_string(),
            "cte.meta.fields/v1".to_string(),
        ],
        SourceProfile::Scalar,
    )
    .expect("meta selectors bind inside a HashExpr");
    let main = resolved
        .fns
        .iter()
        .find(|f| f.name == "main")
        .expect("main resolved");
    // main 的 return 语句值是 HashExpr（类型 I32）。
    let ret = &main.body.stmts[0];
    let stmt_value = match ret {
        ace_core::source::resolve::RStmt::Return { value, .. } => value,
        _ => panic!("first stmt must be the return"),
    };
    assert!(matches!(
        stmt_value.kind,
        ace_core::source::resolve::RExprKind::HashExpr { .. }
    ));
    assert_eq!(stmt_value.vt.as_str(), "i32");
}

/// 各拒绝轴精确具名（§5.2：缺cap/CT外/未知类型/非字面量字段/裸值选择子/
/// 同名声明冲突），互不混淆。
#[test]
fn meta_rejections_are_precise() {
    let cases: &[(&str, &str, usize)] = &[
        // CT 外使用（无 HashExpr 包裹）。
        (
            "type P {
    x int
}
fn main() int {
    return ct_field_count(P)
}
",
            "source.cte.meta-outside-ct",
            2,
        ),
        // 缺 meta 能力（只有 CT 能力）。
        (
            "type P {
    x int
}
fn main() int {
    return #{ ct_field_count(P) }
}
",
            "source.cte.meta-capability",
            1,
        ),
        // 未知类型。
        (
            "fn main() int {
    return #{ ct_field_count(Nope) }
}
",
            "source.cte.meta-unknown-type",
            2,
        ),
        // 类型选择子不是裸 record 名（字面量值）。
        (
            "type P {
    x int
}
fn main() int {
    return #{ ct_field_count(1) }
}
",
            "source.cte.meta-type-selector",
            2,
        ),
        // 字段串非字面量。
        (
            "type P {
    x int
}
fn main() int {
    let f = \"x\"
    return #{ ct_has_field(P, f) }
}
",
            "source.cte.meta-field-selector",
            2,
        ),
        // 元数错误。
        (
            "type P {
    x int
}
fn main() int {
    return #{ ct_field_count(P, P) }
}
",
            "source.argument",
            2,
        ),
        // 用户同名声明（meta 能力在时受保护）。
        (
            "fn ct_field_count() int {
    return 1
}
fn main() int {
    return 0
}
",
            "source.cte.meta-name-conflict",
            2,
        ),
    ];
    for (src, code, cap_idx) in cases {
        let caps_ct_only = vec!["ac.source.cte.scalar/v1".to_string()];
        let caps_both = vec![
            "ac.source.cte.scalar/v1".to_string(),
            "cte.meta.fields/v1".to_string(),
        ];
        let caps = if *cap_idx == 1 {
            caps_ct_only
        } else {
            caps_both
        };
        let FrontOutcome::Ok(ok) = with_ct_meta(src) else {
            panic!("parse failed for {code}");
        };
        let ds = match resolve_with(&ok.program, &caps, SourceProfile::Scalar) {
            Err(ds) => ds,
            Ok(_) => panic!("{code} must be rejected"),
        };
        assert!(
            ds.iter().any(|d| d.code == *code),
            "expected {code}, got {:?}",
            ds.iter().map(|d| d.code.clone()).collect::<Vec<_>>()
        );
    }
}

/// 无 meta 能力时同名用户 fn 合法（trace 模式：无能力=名字不劫持）；
/// meta 能力在时才冲突。
#[test]
fn meta_names_are_ordinary_without_the_capability() {
    let src = "\
fn ct_field_count() int {
    return 1
}
fn main() int {
    return ct_field_count()
}
";
    let FrontOutcome::Ok(ok) =
        parse_front_with_caps(req(src), &["ac.source.cte.scalar/v1".to_string()])
    else {
        panic!("parse");
    };
    resolve_with(
        &ok.program,
        &["ac.source.cte.scalar/v1".to_string()],
        SourceProfile::Scalar,
    )
    .expect("without the meta capability the name is an ordinary fn");
}

/// has_field 绑定成 Bool 类型；record 声明只允许字段形（方法在
/// scalar-ct 面仍语法拒绝——元信息面不扩方法）。
#[test]
fn has_field_binds_bool_and_records_stay_field_only() {
    let src = "\
type Pair {
    x int
    y int
}
fn main() int {
    if #{ ct_has_field(Pair, \"x\") } { return 1 }
    return 0
}
";
    let FrontOutcome::Ok(ok) = with_ct_meta(src) else {
        panic!("parse");
    };
    let resolved = resolve_with(
        &ok.program,
        &[
            "ac.source.cte.scalar/v1".to_string(),
            "cte.meta.fields/v1".to_string(),
        ],
        SourceProfile::Scalar,
    )
    .expect("has_field binds");
    let main = resolved.fns.iter().find(|f| f.name == "main").unwrap();
    let cond = match &main.body.stmts[0] {
        ace_core::source::resolve::RStmt::If { cond, .. } => cond,
        _ => panic!("first stmt must be the if"),
    };
    assert!(matches!(
        cond.kind,
        ace_core::source::resolve::RExprKind::HashExpr { .. }
    ));
    assert_eq!(cond.vt.as_str(), "bool");

    // 方法成员在 scalar-ct 面仍语法拒（fields-only 元信息面）。
    let with_method = "\
type P {
    x int
    fn get(self) int {
        return self.x
    }
}
fn main() int {
    return 0
}
";
    assert!(
        matches!(with_ct_meta(with_method), FrontOutcome::Diag(_)),
        "methods stay outside the scalar-ct record form"
    );
}
