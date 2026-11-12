//! PLAN-935（T-02）：有限模板语法面前门事实。
//!
//! 覆盖：能力门控（无能力仍原拒绝）、`<T>` 投影、类型位裸 ident=类型变量、
//! 有限形态具名拒（重名/模板入口；`<T, U>` 由 PLAN-963 放开为合法声明）、
//! 未匹配 ident 不猜类型、plain 面比较算子不受 fn-tail `<` 歧义影响。

use ace_core::source::ast::{Item, TypeRef};
use ace_core::source::cst::{
    parse_front, parse_front_with_caps, FrontBudget, FrontKind, FrontOutcome, FrontRequest,
    INSTANCES_CAPABILITY,
};
use ace_core::source::SourceProfile;

const CHOOSE: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
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

fn with_caps(src: &str) -> FrontOutcome {
    parse_front_with_caps(req(src), &[INSTANCES_CAPABILITY.to_string()])
}

fn fn_of<'a>(
    p: &'a ace_core::source::ast::Program,
    name: &str,
) -> &'a ace_core::source::ast::FnItem {
    p.items
        .iter()
        .find_map(|i| match i {
            Item::Fn(f) if f.name.name == name => Some(f),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no fn `{name}`"))
}

/// 能力在：choose 投影为模板（type_params=[T]，参数/返回类型变量），
/// main 无类型参数；模板体/调用点原样保留（span 指向原源）。
#[test]
fn parse_choose_template_with_capability() {
    let FrontOutcome::Ok(ok) = with_caps(CHOOSE) else {
        panic!("choose must parse under the instances capability");
    };
    // PLAN-936（T-04）：前门能力面加 scalar-ct → caps 键升 /3（语法/
    // 能力/映射面变更必须换键；本面行为不变）。
    assert_eq!(ok.caps_version, "ac-source-front/4");
    let choose = fn_of(&ok.program, "choose");
    assert_eq!(
        choose
            .type_params
            .iter()
            .map(|t| t.name.name.as_str())
            .collect::<Vec<_>>(),
        vec!["T"],
        "type params project in declaration order"
    );
    assert!(matches!(choose.params[0].ty.kind, TypeRef::TyVar(_)));
    assert!(matches!(choose.params[1].ty.kind, TypeRef::TyVar(_)));
    assert!(matches!(choose.params[2].ty.kind, TypeRef::Builtin(_)));
    assert!(matches!(choose.ret.kind, TypeRef::TyVar(_)));
    let main = fn_of(&ok.program, "main");
    assert!(main.type_params.is_empty());
    assert!(matches!(main.ret.kind, TypeRef::Builtin(_)));
}

/// 无能力：同一源码在 plain scalar 裁剪下仍是语法拒绝（原拒绝面不变，
/// 不因新增规则静默放行）。
#[test]
fn without_capability_still_rejected() {
    assert!(
        matches!(parse_front(req(CHOOSE)), FrontOutcome::Diag(_)),
        "no capability: `<T>` must stay a parse rejection"
    );
    // 能力面只在 Scalar 生效：其他 profile 不因携带该能力改变行为。
    let other = parse_front_with_caps(
        FrontRequest {
            src: CHOOSE,
            profile: SourceProfile::EntryUnit,
            kind: FrontKind::WholeModule,
            budget: FrontBudget::WORK_OWNED_64MIB,
        },
        &[INSTANCES_CAPABILITY.to_string()],
    );
    assert!(matches!(other, FrontOutcome::Diag(_)));
}

/// 有限形态具名拒：重名类型参数、模板入口。
///
/// PLAN-963：`<T, U>` 不再拒绝——多类型参数是通用实例面的合法声明（见
/// `parse_multi_type_params_963`）；其余有限面拒绝保持不变。重名走 fn/type
/// 统一的 `source.duplicate`（PLAN-963 把两者投影到同一 `TypeParamDecl`
/// 形状，诊断码随之统一）。
#[test]
fn finite_surface_named_rejections() {
    let cases: [(&str, &str, &str); 2] = [
        (
            "fn pick<T, T>(a T, b T) T { return a }\nfn main() int { return 0 }",
            "source.duplicate",
            "duplicate type parameter",
        ),
        (
            "fn main<T>() int { return 0 }",
            "source.instances-unsupported",
            "entry function cannot be a template",
        ),
    ];
    for (src, code, needle) in cases {
        match with_caps(src) {
            FrontOutcome::Diag(ds) => {
                assert!(
                    ds.iter()
                        .any(|d| d.code == code && d.message.contains(needle)),
                    "{src}: expected named rejection `{needle}`, got {ds:?}"
                );
            }
            other => panic!("{src}: expected rejection, got {other:?}"),
        }
    }
}

/// PLAN-963：多类型参数 `<T, U>` 投影为声明序 `["T","U"]`，各类型位
/// 裸 ident 仍是类型变量（不因放开数量而丢位序/混淆参数名）。
#[test]
fn parse_multi_type_params_963() {
    let src = "\
fn pick<T, U>(a T, b U, flag bool) T {
    if flag { return a }
    return a
}
fn main() int { return 0 }
";
    let FrontOutcome::Ok(ok) = with_caps(src) else {
        panic!("multi type parameters must parse under the instances capability");
    };
    let pick = fn_of(&ok.program, "pick");
    assert_eq!(
        pick.type_params
            .iter()
            .map(|t| t.name.name.as_str())
            .collect::<Vec<_>>(),
        vec!["T", "U"],
        "type params project in declaration order"
    );
    assert!(matches!(pick.params[0].ty.kind, TypeRef::TyVar(_)));
    assert!(matches!(pick.params[1].ty.kind, TypeRef::TyVar(_)));
    assert!(matches!(pick.params[2].ty.kind, TypeRef::Builtin(_)));
    assert!(matches!(pick.ret.kind, TypeRef::TyVar(_)));
}

/// PLAN-935（T-03）+ PLAN-963（T-02/T-04）：`<T has Bound>` 与 `<const N>`
/// 都是语法形状（词法 ident+ident），不能当普通 token 跳过。PLAN-963 起两者
/// 都是 fn 模板的合法声明：约束投影到 `TypeParamDecl.bound`，值参数投影到
/// `TypeParamKind::Const`；只有**非 u32 声明类型**在投影期具名拒。无能力时
/// 同一形状仍是原语法拒绝。
#[test]
fn constraint_and_const_shapes_named_rejections() {
    // 约束：合法（投影出 bound，不丢位序）。
    let bound_src = "fn sort<T has Comparable>(a T) T { return a }
fn main() int { return 0 }";
    let FrontOutcome::Ok(ok) = with_caps(bound_src) else {
        panic!("`<T has S>` must be a legal fn template declaration (PLAN-963)");
    };
    let sort = fn_of(&ok.program, "sort");
    assert_eq!(sort.type_params.len(), 1);
    assert_eq!(sort.type_params[0].name.name, "T");
    assert_eq!(
        sort.type_params[0].bound.as_ref().map(|b| b.name.as_str()),
        Some("Comparable"),
        "约束投影到声明（不丢、不错位）"
    );

    // 值参数：PLAN-963（T-04）起是合法声明（kind=Const，缺省 u32）。
    let const_src = "fn log2<const N>(n int) int { return n }
fn main() int { return 0 }";
    let FrontOutcome::Ok(ok) = with_caps(const_src) else {
        panic!("`<const N>` must be a legal value-parameter declaration (PLAN-963 T-04)");
    };
    let log2 = fn_of(&ok.program, "log2");
    assert_eq!(log2.type_params.len(), 1);
    assert_eq!(log2.type_params[0].name.name, "N");
    assert_eq!(
        log2.type_params[0].kind,
        ace_core::source::ast::TypeParamKind::Const,
        "值参数按 kind 区分（不与类型参数混位）"
    );

    // 非 u32 声明类型：仍然是具名拒（不静默当 u32）。
    let bad_ty_src = "fn log2<const N bool>(n int) int { return n }
fn main() int { return 0 }";
    match with_caps(bad_ty_src) {
        FrontOutcome::Diag(ds) => assert!(
            ds.iter()
                .any(|d| d.code == "source.instances-unsupported" && d.message.contains("u32")),
            "{bad_ty_src}: expected the non-u32 named rejection, got {ds:?}"
        ),
        other => panic!("{bad_ty_src}: expected rejection, got {other:?}"),
    }

    // 无能力：两者都是原语法拒绝（parse 面）。
    for src in [bound_src, const_src] {
        assert!(
            matches!(parse_front(req(src)), FrontOutcome::Diag(_)),
            "{src}: no-cap must stay rejected"
        );
    }
}

/// 类型位裸 ident 只有匹配声明的类型变量才是 TyVar；未匹配的 ident 走
/// 原 unsupported_ty 拒绝（不猜默认类型、不当 record）。
#[test]
fn unmatched_ident_type_word_rejected() {
    let src = "fn pick(x Foo) int { return 0 }\nfn main() int { return 0 }";
    match with_caps(src) {
        FrontOutcome::Diag(ds) => {
            assert!(
                ds.iter()
                    .any(|d| d.code == "source.unsupported" && d.message.contains("Foo")),
                "unmatched bare ident must keep the unsupported-type rejection: {ds:?}"
            );
        }
        other => panic!("expected rejection, got {other:?}"),
    }
    // 匹配的拼写（局部变量/参数名恰为 T 的大小写变体不误绑）：
    // `T` 只在声明它的模板内可见；main 里的裸 `T` 不被当类型变量。
    let src2 = "fn pick<T>(a T) T { return a }\nfn main() int { let t = 1\n return t }";
    assert!(matches!(with_caps(src2), FrontOutcome::Ok(_)));
}

/// plain 面回归：比较算子/泛型内建形态不受 fn-tail `<T>` 步骤影响
/// （fn-tail 仅在声明名后消费 `<`，表达式层 pratt 不变）。
#[test]
fn plain_face_comparison_unaffected() {
    let src = "\
fn cmp(a int, b int) bool { return a < b }
fn main() int {
    if cmp(1, 2) { return 1 }
    return 0
}
";
    // 无能力（plain scalar）：照旧 Ok。
    assert!(matches!(parse_front(req(src)), FrontOutcome::Ok(_)));
    // 有能力（scalar-instances 裁剪）：同一源码仍 Ok（超集面）。
    assert!(matches!(with_caps(src), FrontOutcome::Ok(_)));
}
