//! PLAN-935 T-02：实例面**恢复可行性**（V-06 有界）+ 四方向反控。
//!
//! 恢复（沿 934 事实恢复缝，实例层只重确定性推导，不重跑求解器）：
//! * 冷编译产出具体函数事实页；新会话（= 新进程形状）重推导脱糖程序 →
//!   逐页 restore_verify → pages-only bind → emit → **真实 verify_scalar**，
//!   HIR 与冷编译语义相等；
//! * seed 后产品编译 solver_runs=0（恢复不重产实例/不重推导任何 body）；
//! * 恢复→首有效编辑：恰 main 一页重推导（实例页 hit）；
//! * 恢复→首坏编辑（异质实参）：具名拒，零求解。
//!
//! 反控（实例轴四方向，沿 934 模式移植）：
//! 1. 冷 miss + 求解器禁 → 红（真实需求）；2. 暖 hit + 求解器禁 → 绿
//! （无隐藏生产）；3. 篡改实例事实类型 → 真 bind 红（source.fact-mismatch）；
//! 4. 异 owner 页 → seed 拒（逻辑域不匹配）。

use ace_core::source::cst::{
    parse_front_with_caps, FrontBudget, FrontKind, FrontOutcome, FrontRequest, INSTANCES_CAPABILITY,
};
use ace_core::source::resolve::resolve_with_supply;
use ace_core::source::{emit_resolved_parts, SourceProfile};
use ace_core::verify::verify_scalar;
use ace_source_query::{
    compile_scalar_product, instantiate_program, restore_bind, restore_verify, BodyTypeSupply,
    InferBodyFacts, PortableTy, SourceQuerySession,
};
use std::collections::BTreeMap;

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
const PACKAGE: &str = "plan935";
const MODULE: &str = "choose.at";

fn caps() -> Vec<String> {
    vec![INSTANCES_CAPABILITY.to_string()]
}

/// 确定性推导：源 → 能力面前门 → 脱糖具体程序（两个进程形状各跑一次，
/// 结果逐字节同——实例调度是纯函数）。
fn derive(src: &str) -> ace_core::source::ast::Program {
    match parse_front_with_caps(
        FrontRequest {
            src,
            profile: SourceProfile::Scalar,
            kind: FrontKind::WholeModule,
            budget: FrontBudget::WORK_OWNED_64MIB,
        },
        &caps(),
    ) {
        FrontOutcome::Ok(ok) => {
            instantiate_program(&ok.program, PACKAGE, MODULE, &caps())
                .expect("instantiate")
                .program
        }
        other => panic!("parse failed: {other:?}"),
    }
}

fn fn_names(p: &ace_core::source::ast::Program) -> Vec<String> {
    p.items
        .iter()
        .filter_map(|i| match i {
            ace_core::source::ast::Item::Fn(f) => Some(f.name.name.clone()),
            _ => None,
        })
        .collect()
}

/// 冷事实页（具体函数逐个体）。
fn cold_pages(program: &ace_core::source::ast::Program) -> BTreeMap<String, InferBodyFacts> {
    let mut session = SourceQuerySession::new(PACKAGE, MODULE).with_capabilities(caps());
    let mut pages = BTreeMap::new();
    for name in fn_names(program) {
        let facts = session
            .infer_body(program, &name)
            .expect("cold facts")
            .facts;
        pages.insert(name, facts);
    }
    pages
}

#[test]
fn instances_restore_pages_only_bind_and_first_edits() {
    // ---- 冷编译（产品路径）作为对照 HIR -------------------------------
    let mut cold_session = SourceQuerySession::new(PACKAGE, MODULE).with_capabilities(caps());
    let cold = compile_scalar_product(CHOOSE, MODULE, &caps(), PACKAGE, MODULE, &mut cold_session)
        .expect("cold compile");
    assert_eq!(cold.instances.len(), 2);
    let cold_runs = cold_session.counters().solver_runs;

    // ---- 新进程形状：重推导 + 逐页深验 + pages-only bind + 真 verify ----
    let program = derive(CHOOSE);
    assert_eq!(
        fn_names(&program),
        vec![
            "choose_i32".to_string(),
            "choose_bool".to_string(),
            "main".to_string()
        ],
        "desugared program shape is deterministic"
    );
    let pages = cold_pages(&program);
    for (name, page) in &pages {
        restore_verify(&program, PACKAGE, MODULE, name, Some(page))
            .unwrap_or_else(|e| panic!("page `{name}` must admit: {e}"));
    }
    assert!(
        restore_verify(&program, PACKAGE, MODULE, "choose_i32", None).is_err(),
        "a missing required instance page refuses the lane"
    );
    let (resolved, applied, supply) = restore_bind(
        &program,
        &caps(),
        PACKAGE,
        MODULE,
        &pages,
        SourceProfile::Scalar,
    )
    .expect("pages-only bind");
    assert_eq!(applied.len(), 3, "main + two concrete instances");
    assert_eq!(supply.admitted_bodies(), 3);
    let parts = emit_resolved_parts(&resolved, CHOOSE, MODULE, SourceProfile::Scalar)
        .expect("emit restored");
    let checked = verify_scalar(parts.bundle).expect("restored HIR passes real verification");
    // 语义等价：与冷编译 HIR 同（语义相等由 bundle 的 Debug 面承载）。
    assert_eq!(
        format!("{:?}", checked.bundle()),
        format!("{:?}", cold.compiled.checked.bundle()),
        "restored HIR is semantically the cold HIR"
    );

    // ---- seed → 产品编译 0 求解（恢复不重产）--------------------------
    let mut warm = SourceQuerySession::new(PACKAGE, MODULE).with_capabilities(caps());
    for page in pages.values() {
        warm.seed(&program, page).expect("seed page");
    }
    let restored_product =
        compile_scalar_product(CHOOSE, MODULE, &caps(), PACKAGE, MODULE, &mut warm)
            .expect("restored compile");
    assert_eq!(restored_product.instances.len(), 2);
    assert_eq!(
        warm.counters().solver_runs,
        0,
        "a restored compile solves nothing (R permits re-verify, not re-solve)"
    );

    // ---- 恢复 → 首有效编辑（caller 常量）：恰 main 一页重推导 ----------
    let edited = CHOOSE.replace("40, 42", "41, 42");
    let program2 = derive(&edited);
    let p2 = compile_scalar_product(&edited, MODULE, &caps(), PACKAGE, MODULE, &mut warm)
        .expect("first good edit compiles");
    assert_eq!(p2.instances.len(), 2);
    assert_eq!(
        warm.counters().solver_runs,
        1,
        "exactly the edited caller re-solves; both instance pages hit"
    );
    let _ = program2;

    // ---- 恢复 → 首坏编辑（异质实参）：具名拒、零求解 -------------------
    let bad = CHOOSE.replace("choose(40, 42, false)", "choose(40, true, false)");
    match compile_scalar_product(&bad, MODULE, &caps(), PACKAGE, MODULE, &mut warm) {
        Err(ds) => assert!(
            ds.iter().any(|d| d.code == "type-mismatch"),
            "expected the real solver heterogeneous rejection, got {ds:?}"
        ),
        Ok(_) => panic!("first bad edit must be rejected"),
    }
    assert_eq!(
        warm.counters().solver_runs,
        1,
        "a discovery rejection runs no solver"
    );
    let _ = cold_runs;
}

/// 反控 1：冷 miss + 求解器禁 → 红（实例体真需要求解器生产）。
#[test]
fn anti_cold_miss_with_solver_disabled_is_red() {
    let mut session = SourceQuerySession::new(PACKAGE, MODULE)
        .with_capabilities(caps())
        .with_solver(false);
    match compile_scalar_product(CHOOSE, MODULE, &caps(), PACKAGE, MODULE, &mut session) {
        Err(ds) => assert!(
            ds.iter().any(|d| d.code == "source.fact-resource"),
            "expected the solver-disabled resource rejection, got {ds:?}"
        ),
        Ok(_) => panic!("a cold compile with the solver disabled must be red"),
    }
}

/// 反控 2：暖 hit + 求解器禁 → 绿（页缓存复用，无隐藏生产）。
#[test]
fn anti_warm_hit_with_solver_disabled_is_green() {
    let mut session = SourceQuerySession::new(PACKAGE, MODULE).with_capabilities(caps());
    let _ = compile_scalar_product(CHOOSE, MODULE, &caps(), PACKAGE, MODULE, &mut session)
        .expect("cold");
    session.set_allow_solver(false);
    let p = compile_scalar_product(CHOOSE, MODULE, &caps(), PACKAGE, MODULE, &mut session)
        .expect("warm compile with the solver disabled is green");
    assert_eq!(
        p.instances.len(),
        2,
        "instances re-derive without any solve"
    );
    assert!(session.counters().body_hits >= 3);
}

/// 反控 3：篡改实例事实类型（main 页上 choose_i32 调用结果 int→bool）→
/// 真 **bind** 红（不是辅助 checker 红）。
#[test]
fn anti_tampered_instance_page_type_fails_real_bind() {
    let program = derive(CHOOSE);
    let mut session = SourceQuerySession::new(PACKAGE, MODULE).with_capabilities(caps());
    let mut supply = BodyTypeSupply::new();
    for name in fn_names(&program) {
        let ast_body = program
            .items
            .iter()
            .find_map(|it| match it {
                ace_core::source::ast::Item::Fn(f) if f.name.name == name => Some(&f.body),
                _ => None,
            })
            .expect("ast body");
        let mut facts = session.infer_body(&program, &name).expect("facts").facts;
        if name == "main" {
            // 调用结点声明 i32 结果；结构恒等仍成立——只有核心的
            // fact-mismatch 检查能抓到。
            let call = facts
                .nodes
                .iter_mut()
                .find(|n| n.kind == "expr.call" && n.ty == PortableTy::I32)
                .expect("the choose_i32 call fact");
            call.ty = PortableTy::Bool;
        }
        supply
            .add_body(ast_body, &facts)
            .expect("shape-valid page admits");
    }
    let err = match resolve_with_supply(&program, &caps(), SourceProfile::Scalar, &supply) {
        Ok(_) => panic!("a bool fact on the int instance call must fail the bind"),
        Err(e) => e,
    };
    assert!(
        err.iter().any(|d| d.code == "source.fact-mismatch"),
        "expected a bind-stage fact mismatch, got {:?}",
        err.iter()
            .map(|d| (&d.code, &d.message))
            .collect::<Vec<_>>()
    );
}

/// 预算可行性（T-02 整链锚）：三个具体函数的事实页各 ≤512KiB 页体上限，
/// 总编码量远低于 16MiB 受管语义预算——实例不另享预算份额。
#[test]
fn instance_pages_fit_the_frozen_budget_anchors() {
    let program = derive(CHOOSE);
    let mut session = SourceQuerySession::new(PACKAGE, MODULE).with_capabilities(caps());
    let mut total = 0usize;
    for name in fn_names(&program) {
        let facts = session.infer_body(&program, &name).expect("facts").facts;
        let key = session.body_key(&program, &name).expect("key");
        let bytes = ace_source_query::encode_page(&key, &facts).expect("encode page");
        assert!(
            bytes.len() <= 512 * 1024,
            "page `{name}` exceeds the 512KiB page anchor: {}",
            bytes.len()
        );
        total += bytes.len();
    }
    assert!(
        total < 1024 * 1024,
        "the whole choose module's pages ({total}B) must sit far below the 16MiB managed budget"
    );
}

/// 反控 4：异 owner（异包）页 → seed 逻辑域拒。
#[test]
fn anti_foreign_owner_page_refused() {
    let program = derive(CHOOSE);
    // 在另一个包下生产的 choose_i32 页。
    let mut foreign =
        ace_source_query::SourceQuerySession::new("other-pkg", MODULE).with_capabilities(caps());
    let page = foreign
        .infer_body(&program, "choose_i32")
        .expect("foreign facts")
        .facts;
    let mut home = SourceQuerySession::new(PACKAGE, MODULE).with_capabilities(caps());
    assert!(
        home.seed(&program, &page).is_err(),
        "a foreign-owner page must be refused at seed (logical-domain mismatch)"
    );
}
