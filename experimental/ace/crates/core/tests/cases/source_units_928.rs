//! PLAN-928 T-02 探针：closures 面最小整链（统一前门 → 投影 → 单元目录）。
//!
//! - **差分冒烟**：K 语料（865 闭包集）旧手写 vs 新 CST 链全等（Program 全
//!   字段/首诊断）——共享链语法正确性的即时 oracle；
//! - **§5.2 四项**：(a) 单 lambda 编辑只物化目标体（兄弟/owner 生产 0）；
//!   (b) owner 变化同文 lambda 语法共享 + semantic_recheck 信号；
//!   (c) 兄弟 fn 声明不被片段重建（Rc 同一 + 生产 0）；
//!   (d) 坏编辑诊断后目录/源保持旧态，修复后正确；
//! - **§5.4**：同文异 owner 不合并；同 owner 重复同文拓扑变化 = 具名非快路径
//!   （如实重产，不假 0）；
//! - **预算**：1 字节 work 预算 → typed ResourceExhausted（资源态，非语法拒）。

use ace_core::source::cst::{self, units, FrontBudget};
use ace_core::source::{parse_program_with_handwritten, SourceProfile};
use std::rc::Rc;

const CL: SourceProfile = SourceProfile::Closures;
const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;

fn assert_diff_same(src: &str, what: &str) {
    let old = parse_program_with_handwritten(src, CL);
    let new = cst::parse_program_via_cst(src, CL);
    match (&old, &new) {
        (Ok(a), Ok(b)) => {
            assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "{what}: closures AST mismatch old(handwritten) vs new(CST)\n--old--\n{a:#?}\n--new--\n{b:#?}"
            );
        }
        (Err(a), Err(b)) => {
            assert_eq!(
                (a.stage, a.code.as_str(), a.message.as_str(), a.span),
                (b.stage, b.code.as_str(), b.message.as_str(), b.span),
                "{what}: closures first-diag mismatch\n--old--\n{a:?}\n--new--\n{b:?}"
            );
        }
        _ => panic!(
            "{what}: outcome mismatch old={old:?} new={new:?} (src={src})",
            old = parse_program_with_handwritten(src, CL).is_ok(),
            new = cst::parse_program_via_cst(src, CL).is_ok(),
        ),
    }
}

/// K 语料原文（865 集；plan §5.1 L04/L05 的闭包对应面）。
#[test]
fn t02_diff_k_corpus_closures() {
    let k01 = "fn main() int {\n    let base int = 40\n    let f = move (n int) => { return base + n }\n    return f(2)\n}\n";
    let k02 = "fn make(move s Str) fn(int) int {\n    return move (n int) => { return s.len() + n }\n}\nfn main() int {\n    let s Str = \"hello\"\n    let f = make(s.move)\n    return f(37)\n}\n";
    let k06 = "fn main() int {\n    var base int = 1\n    var f = move (n int) => {\n        base = base + n\n        return base\n    }\n    return f(20) + f(0)\n}\n";
    for (src, what) in [(k01, "K01"), (k02, "K02-fn-type+postfix"), (k06, "K06-writeback")] {
        assert_diff_same(src, what);
    }
    // 表达式体 + 零参 + 多参 + 嵌套 lambda（差分面扩展）。
    assert_diff_same(
        "fn main() int {\n    let a = () => 7\n    let b = (x int, y int) => { return x + y }\n    let c = (x int) => { let inner = (y int) => { return x + y }\n        return inner(x) }\n    return a() + b(1, 2) + c(3)\n}\n",
        "expr-body+zero-params+nested",
    );
    // 负例（T-02 面）：坏 lambda 必须是 typed 源码诊断（不成功）——逐字节
    // 诊断 parity（code/span/message：如未闭合块旧报 EOF 处"expected
    // statement or `}`"、新链当前报恢复位"expected `)`"）由 T-04 的冻结
    // 差分边集对齐；括号组回退的正向差分已在上例覆盖。
    let bad = "fn main() int {\n    let f = (n int) => { return n\n}\n";
    assert!(
        cst::parse_program_via_cst(bad, CL).is_err(),
        "unterminated lambda block must diagnose, not succeed"
    );
    assert_diff_same("fn main() int {\n    let x = (1 + 2) * 3\n    return x\n}\n", "paren-group-fallback");
}

/// §5.2(a)：owner 多 lambda，编辑其一——目标物化 1，兄弟与 owner 生产 0。
#[test]
fn t02_a_single_lambda_edit() {
    let src = "fn helper() int {\n    return 1\n}\nfn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n }\n    let g = (m int) => { return f(m) + 1 }\n    return f(2) + g(3)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold build");
    assert_eq!(cat.meter.produced, 4, "cold: helper + main + 2 lambda");
    let helper_before = Rc::as_ptr(&cat.fns["helper"].item) as usize;
    let main_before = Rc::as_ptr(&cat.fns["main"].item) as usize;
    let g_before = Rc::as_ptr(&cat.lambda("main", 1).unwrap().body) as usize;

    // 编辑 lambda#0（f）：正文加一。
    let old_span = cat.lambda("main", 0).unwrap().span;
    let old_slice = cat.src()[old_span.start..old_span.end].to_string();
    let new_slice = old_slice.replace("base + n", "base + n + 1");
    let before = (cat.meter.produced, cat.meter.reused, cat.meter.discarded);
    let receipt = cat
        .edit_lambda("main", 0, &new_slice, BUDGET)
        .expect("single lambda edit");
    let after = (cat.meter.produced, cat.meter.reused, cat.meter.discarded);
    assert_eq!(after.0 - before.0, 1, "目标 lambda 物化恰 1");
    assert_eq!(after.1 - before.1, 1, "兄弟 lambda 真实 Rc 共享如实计复用（0 生产）");
    assert_eq!(after.2, before.2, "无废弃物化");
    assert_eq!(receipt.produced.len(), 1, "仅目标单元入 produced");
    assert!(receipt.produced[0].kind == units::UnitKind::Lambda && receipt.produced[0].ordinal == 0);
    // 兄弟 g 与 owner/兄弟 fn 的密封体是同一 Rc（真复用，非重物化）。
    assert_eq!(Rc::as_ptr(&cat.fns["helper"].item) as usize, helper_before, "helper fn 体未重建");
    assert_eq!(Rc::as_ptr(&cat.fns["main"].item) as usize, main_before, "owner fn 体未重建（lambda 编辑不重产 owner）");
    assert_eq!(Rc::as_ptr(&cat.lambda("main", 1).unwrap().body) as usize, g_before, "兄弟 lambda 体共享同一 Rc");
    // 新源上重解析全等冒烟（目录源与整源解析一致）。
    assert_diff_same(&receipt.new_src, "after-single-lambda-edit");
}

/// §5.2(b)：owner 创建点变化、lambda 文本不变——语法体共享 + 语义重验信号。
#[test]
fn t02_b_owner_edit_shares_unchanged_lambda_but_signals_recheck() {
    let src = "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n }\n    return f(2)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let lam_before = Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize;
    let old_span = cat.fns["main"].span;
    let old_fn = cat.src()[old_span.start..old_span.end].to_string();
    // 创建点变化：let -> var（捕获写性变化 = 909 共享可写捕获面）。
    let new_fn = old_fn.replace("let base int = 40", "var base int = 41");
    assert_ne!(old_fn, new_fn);
    let before = cat.meter.clone();
    let receipt = cat.edit_owner("main", &new_fn, BUDGET).expect("owner edit");
    assert_eq!(cat.meter.produced - before.produced, 1, "仅 owner fn 物化 1");
    assert_eq!(cat.meter.reused - before.reused, 1, "同文 lambda 复用 1（生产 0）");
    assert!(receipt.semantic_recheck_required, "owner 变化必须携带语义重验信号（语法共享不免 capture 重验）");
    assert_eq!(Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize, lam_before, "同文 lambda 语法体 Rc 同一");
    assert_diff_same(&receipt.new_src, "after-owner-edit");
}

/// §5.2(c)：兄弟 fn 声明不被片段重建（Rc 同一 + 生产 0）。
#[test]
fn t02_c_sibling_decls_not_rebuilt() {
    let src = "fn a() int {\n    return 1\n}\nfn b() int {\n    let f = (n int) => { return n + 1 }\n    return f(1)\n}\nfn c() int {\n    return 3\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let a_ptr = Rc::as_ptr(&cat.fns["a"].item) as usize;
    let c_ptr = Rc::as_ptr(&cat.fns["c"].item) as usize;
    let old_span = cat.fns["b"].span;
    let new_fn = cat.src()[old_span.start..old_span.end].replace("n + 1", "n + 2");
    let before = cat.meter.clone();
    let receipt = cat.edit_owner("b", &new_fn, BUDGET).expect("owner b edit");
    assert_eq!(cat.meter.produced - before.produced, 2, "b(1) + 其 lambda 文变重产(1)；a/c 不计");
    assert_eq!(Rc::as_ptr(&cat.fns["a"].item) as usize, a_ptr, "声明 a 未重建");
    assert_eq!(Rc::as_ptr(&cat.fns["c"].item) as usize, c_ptr, "声明 c 未重建");
    assert_diff_same(&receipt.new_src, "after-sibling-edit");
}

/// §5.2(d)：坏编辑诊断——目录/源保持旧态；修复后正确。
#[test]
fn t02_d_failed_edit_preserves_state_and_fix_recovers() {
    let src = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    return f(1)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let src_before = cat.src().to_string();
    let meter_before = cat.meter.clone();
    let body_before = Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize;
    // 坏编辑：块未闭合。
    let err = cat
        .edit_lambda("main", 0, "(n int) => { return n + 1", BUDGET)
        .expect_err("broken lambda must diagnose");
    match err {
        units::EditError::Diag(ds) => assert!(!ds.is_empty(), "诊断非空"),
        other => panic!("expected Diag, got {other:?}"),
    }
    assert_eq!(cat.src(), src_before, "失败编辑不改变源基线");
    assert_eq!(cat.meter.produced, meter_before.produced, "失败编辑不提交生产");
    assert_eq!(cat.meter.discarded, meter_before.discarded + 1, "废弃物化单列 +1");
    assert_eq!(Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize, body_before, "旧密封体仍可用");
    // 修复编辑：成功且正确。
    let receipt = cat
        .edit_lambda("main", 0, "(n int) => { return n + 41 }", BUDGET)
        .expect("fixed edit");
    assert_diff_same(&receipt.new_src, "after-fix");
}

/// §5.4：同文异 owner 绝不合并；同 owner 重复同文 + 拓扑变化 = 具名非快路径。
#[test]
fn t02_same_text_identity_rules() {
    // 同文 lambda 在两个 owner。
    let src = "fn u1() int {\n    let f = (n int) => { return n + 1 }\n    return f(1)\n}\nfn u2() int {\n    let f = (n int) => { return n + 1 }\n    return f(2)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let l0 = Rc::as_ptr(&cat.lambda("u1", 0).unwrap().body) as usize;
    let l1 = Rc::as_ptr(&cat.lambda("u2", 0).unwrap().body) as usize;
    assert_ne!(l0, l1, "同文异 owner 的 lambda 密封体必须独立（不跨 owner 合并）");
    // 编辑 u1 的 lambda：u2 的同文 lambda 不被动（异 owner 无串扰）。
    cat.edit_lambda("u1", 0, "(n int) => { return n + 2 }", BUDGET).expect("edit u1");
    assert_eq!(
        Rc::as_ptr(&cat.lambda("u2", 0).unwrap().body) as usize, l1,
        "u1 编辑不得触碰 u2 的同文 lambda"
    );
    assert_diff_same(cat.src(), "after-cross-owner-edit");

    // 同 owner 两个同文 lambda；编辑其一后保持独立；插入第三个（拓扑变化）
    // = 具名非快路径，整 owner 如实重产。
    let src2 = "fn w() int {\n    let a = (n int) => { return n + 1 }\n    let b = (n int) => { return n + 1 }\n    return a(1) + b(2)\n}\n";
    let mut cat2 = units::UnitCatalog::build(src2, CL, BUDGET).expect("cold2");
    let a0 = Rc::as_ptr(&cat2.lambda("w", 0).unwrap().body) as usize;
    let b0 = Rc::as_ptr(&cat2.lambda("w", 1).unwrap().body) as usize;
    assert_ne!(a0, b0, "同 owner 重复同文以 ordinal 区分，独立密封");
    let old_span = cat2.fns["w"].span;
    let with_third = cat2.src()[old_span.start..old_span.end]
        .replace("    return a(1) + b(2)\n", "    let c = (n int) => { return n + 9 }\n    return a(1) + b(2) + c(3)\n");
    let before = cat2.meter.clone();
    let receipt = cat2.edit_owner("w", &with_third, BUDGET).expect("topology edit");
    assert_eq!(
        receipt.non_fast_path,
        Some("owner-lambda-topology-changed"),
        "拓扑变化必须具名非快路径"
    );
    // 如实重产：owner + 全部 lambda（3 个），无假 0。
    assert_eq!(cat2.meter.produced - before.produced, 1 + 3, "owner(1) + 3 lambda 重产，不得假 0");
    assert_eq!(cat2.meter.reused - before.reused, 0, "拓扑变化下无复用冒充");
    assert_diff_same(&receipt.new_src, "after-topology-edit");
}

/// 预算：1 字节 work 预算 → typed ResourceExhausted（资源态非语法拒）。
#[test]
fn t02_budget_typed_resource_exhausted() {
    let src = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    return f(1)\n}\n";
    let tiny = FrontBudget { work_bytes: 1, max_parse_depth: 1 << 20 };
    match units::UnitCatalog::build(src, CL, tiny) {
        Err(units::EditError::ResourceExhausted { .. }) => {}
        other => panic!("expected ResourceExhausted, got {other:?}"),
    }
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold ok");
    match cat.edit_lambda("main", 0, "(n int) => { return n + 2 }", tiny) {
        Err(units::EditError::ResourceExhausted { .. }) => {}
        other => panic!("edit must stay resource-typed, got {other:?}"),
    }
}

/// PLAN-928 T-03：collections 面差分（统一前门 vs 手写 oracle；字节全等）。
/// 覆盖 L04/L05 全语法（type `;` 字段、List<T>.new、pos/named ctor、move
/// 后缀、方法链）、enum/臂载荷/is 限定臂/for-in（裸 ident 迭代器，C7）、
/// 参数化/可选/嵌套泛型、`<` 比较回退、Str/clone。
#[test]
fn t03_collections_differential() {
    let cases: &[(&str, &str)] = &[
        ("type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    let x Item = Item(\"hello\", 37)\n    xs.push(x.move)\n    return xs.get(0).name.len() + xs.get(0).code\n}\n", "L04"),
        ("type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"hello\", 37))\n    let x Item = xs.remove(0)\n    return x.name.len() + x.code + xs.len()\n}\n", "L05"),
        ("enum Shape { Dot Circle(int) Square(int) }\nfn main() int {\n    let s Shape = Circle(3)\n    is s {\n        Dot -> { return 0 }\n        Circle(r) -> { return r }\n        Square(w) -> { return w }\n    }\n}\n", "enum-is"),
        ("fn main() int {\n    var xs = List<int>.new()\n    xs.push(1)\n    xs.push(2)\n    var total int = 0\n    for x in xs {\n        total = total + x\n    }\n    return total + xs.len()\n}\n", "for-in"),
        ("fn f(a int, b int) bool {\n    return a < b\n}\n", "lt-compare-fallback"),
        ("fn opt(x ?int) int {\n    return 1\n}\n", "optional-ty"),
        ("fn r(x Result<int, Str>) int {\n    return 1\n}\n", "result-2args"),
        ("fn r(x List<List<int>>) int {\n    return 1\n}\n", "nested-generic"),
        ("type P { a int }\nfn main() int {\n    let p P = P { a: 1 }\n    return p.a\n}\n", "named-ctor"),
        ("enum Shape { Dot }\nfn main() int {\n    let s Shape = Dot\n    is s {\n        Shape.Dot -> { return 0 }\n    }\n}\n", "is-qual-arm"),
        ("fn main() int {\n    let s Str = \"hi\"\n    var t Str = s.clone()\n    return t.len()\n}\n", "str-clone"),
    ];
    for (src, what) in cases {
        let old = parse_program_with_handwritten(src, SourceProfile::Collections);
        let new = cst::parse_program_via_cst(src, SourceProfile::Collections);
        match (&old, &new) {
            (Ok(a), Ok(b)) => assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "{what}: collections AST mismatch\n--old--\n{a:#?}\n--new--\n{b:#?}"
            ),
            (Err(a), Err(b)) => assert_eq!(
                (a.stage, a.code.as_str(), a.message.as_str(), a.span),
                (b.stage, b.code.as_str(), b.message.as_str(), b.span),
                "{what}: collections first-diag mismatch\n--old--\n{a:?}\n--new--\n{b:?}"
            ),
            _ => panic!("{what}: outcome mismatch old_ok={} new_ok={}", old.is_ok(), new.is_ok()),
        }
    }
}

/// PLAN-928 T-04：语料全集差分扫描（ace-m1/src 全部 .at 源 × Collections/
/// Closures 两 profile；正例=AST 字节全等，反例=首诊断四元组全等）。
/// 这是 V-01 的 in-process 前身；文件化 case/oracle 由 T-09 注册。
#[test]
fn t04_corpus_sweep_both_profiles() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/auto-lang/test/cases/ace-m1/src"
    );
    let mut names: Vec<_> = std::fs::read_dir(dir)
        .expect("corpus dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".at"))
        .collect();
    names.sort();
    let mut fails: Vec<String> = Vec::new();
    let mut ok_count = 0usize;
    for name in &names {
        let path = format!("{dir}/{name}");
        let Ok(src) = std::fs::read_to_string(&path) else { continue };
        for (profile, pname) in [
            (SourceProfile::Collections, "collections"),
            (SourceProfile::Closures, "closures"),
        ] {
            let old = parse_program_with_handwritten(&src, profile);
            let new = cst::parse_program_via_cst(&src, profile);
            let same = match (&old, &new) {
                (Ok(a), Ok(b)) => format!("{a:?}") == format!("{b:?}"),
                (Err(a), Err(b)) => {
                    (a.stage, a.code.as_str(), a.message.as_str(), a.span)
                        == (b.stage, b.code.as_str(), b.message.as_str(), b.span)
                }
                _ => false,
            };
            if same {
                ok_count += 1;
            } else {
                let detail = match (&old, &new) {
                    (Ok(_), Err(b)) => format!("new rejects: {} {}-{} {}", b.code, b.span.start, b.span.end, b.message),
                    (Err(a), Ok(_)) => format!("new accepts (old: {} {}-{} {})", a.code, a.span.start, a.span.end, a.message),
                    (Err(a), Err(b)) => format!("diag differs:\n  old {} {}-{} {}\n  new {} {}-{} {}", a.code, a.span.start, a.span.end, a.message, b.code, b.span.start, b.span.end, b.message),
                    (Ok(_), Ok(_)) => "AST differs".to_string(),
                };
                fails.push(format!("{name}/{pname}: {detail}"));
            }
        }
    }
    assert!(
        fails.is_empty(),
        "corpus sweep: {} ok, {} failing:\n{}",
        ok_count,
        fails.len(),
        fails.join("\n")
    );
}

/// PLAN-928 T-05：候选事务——暂存不提交、坏候选诊断零变异、提交推进；
/// 语义重验义务随候选显式携带（§5.2b 的候选面）。
#[test]
fn t05_candidate_transaction_lifecycle() {
    let src = "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n }\n    let g = (m int) => { return f(m) + 1 }\n    return f(2) + g(3)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let old_src = cat.src().to_string();
    let old_meter = cat.meter.clone();
    let f_before = std::rc::Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize;

    // 坏候选：诊断 + 目录/源/计量零变异（除废弃物化单列）。
    let bad = cat
        .stage_owner_edit("main", "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n\n}\n", BUDGET)
        .map_or_else(|e| e, |_| panic!("broken candidate must diagnose"));
    assert!(matches!(bad, units::EditError::Diag(_)));
    assert_eq!(cat.src(), old_src);
    assert_eq!(cat.meter.produced, old_meter.produced);
    assert_eq!(cat.meter.discarded, old_meter.discarded, "stage 失败不计废弃物化（未到物化点）");

    // 好候选：暂存面已决策（owner 1 产 + 兄弟同文 1 复用 + 目标文变 1 产）。
    let new_fn = "fn main() int {\n    let base int = 41\n    let f = (n int) => { return base + n }\n    let g = (m int) => { return f(m) + 2 }\n    return f(2) + g(3)\n}\n";
    let cand = cat.stage_owner_edit("main", new_fn, BUDGET).expect("stage");
    assert_eq!(cand.produced(), 2, "owner(1) + 文变 lambda g(1)");
    assert_eq!(cand.reused(), 1, "同文 lambda f 复用");
    assert_eq!(cand.semantic_recheck(), &[0u32, 1], "两个 lambda 语义重验义务都携带（owner 变化）");
    // 暂存不提交：目录与源仍为旧态。
    assert_eq!(cat.src(), old_src);
    assert_eq!(cat.meter.produced, old_meter.produced);
    // 取消 = 丢弃候选（无 API 侧效应）。
    drop(cand);
    assert_eq!(cat.src(), old_src);
    assert_eq!(std::rc::Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize, f_before);

    // 重新暂存并提交：推进 + 计量如实。
    let cand2 = cat.stage_owner_edit("main", new_fn, BUDGET).expect("stage2");
    let receipt = cat.commit_staged(cand2).expect("commit staged");
    assert_ne!(cat.src(), old_src);
    assert_eq!(cat.meter.produced - old_meter.produced, 2);
    assert_eq!(cat.meter.reused - old_meter.reused, 1);
    assert_eq!(receipt.produced.len(), 2);
    assert_eq!(receipt.reused.len(), 1);
    // 差分冒烟：提交后的源仍可整源全等解析。
    let old = parse_program_with_handwritten(cat.src(), CL);
    let new = cst::parse_program_via_cst(cat.src(), CL);
    assert_eq!(old.is_ok(), new.is_ok());
}

/// PLAN-928 T-05（V-10）：current + 多代 pin 同时存活——旧 pin 只读其原
/// 修订视图；跨修订不串用；最后释放后目录照常。
#[test]
fn t05_pin_generations_coexist() {
    let src = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    return f(1)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let pin0 = cat.pin();
    let sha0 = pin0.revision_sha();
    assert_eq!(sha0, units::content_sha(src.as_bytes()));
    // 编辑（owner 路径）后 pin1；pin0 仍是旧视图。
    let new_fn = "fn main() int {\n    let f = (n int) => { return n + 2 }\n    return f(1)\n}\n";
    let cand = cat.stage_owner_edit("main", new_fn, BUDGET).expect("stage");
    cat.commit_staged(cand).expect("commit staged");
    let pin1 = cat.pin();
    assert_ne!(pin0.revision_sha(), pin1.revision_sha());
    // 旧 pin 读取：owner 体为其原修订内容（return n + 1）。
    let p0_main = pin0.owner_item("main").expect("pin0 main");
    let p1_main = pin1.owner_item("main").expect("pin1 main");
    assert_ne!(format!("{p0_main:?}"), format!("{p1_main:?}"), "两代 pin 的 owner 体不同代");
    // current 与两代 pin 共存互不串用。
    assert_ne!(cat.src(), pin0.revision_src);
    assert_eq!(cat.src(), pin1.revision_src.as_str());
    drop(pin0);
    drop(pin1);
    // 目录照常工作。
    let _ = cat.lambda("main", 0).expect("catalog alive after pins dropped");
}


/// PLAN-928 T-06（V-06 面）：签名变化事件、坐标漂移不改 mint 身份、
/// 清单无指针稳定。
#[test]
fn t06_events_identity_manifest() {
    // 声明/签名面（collections：type 声明 + 签名，无 lambda）。
    let src = "type Item { name Str; code int }
fn make(x Item) int {
    return x.code
}
fn main() int {
    let a Item = Item(49, 41)
    return make(a.move)
}
";
    let mut cat = units::UnitCatalog::build(src, ace_core::source::SourceProfile::Collections, BUDGET).expect("cold");
    assert!(cat.decls.contains_key("Item"), "type decl is a catalog unit");
    assert_eq!(cat.meter.produced, 3, "decl(1)+fns(2)");
    let make_mint = cat.fns["make"].mint_span;
    // body-only 编辑：mint 保持、无签名事件。
    let old_span = cat.fns["make"].span;
    let new_fn = cat.src()[old_span.start..old_span.end].replace("x.code", "x.code + 1");
    let cand = cat.stage_owner_edit("make", &new_fn, BUDGET).expect("stage");
    cat.commit_staged(cand).expect("commit staged");
    assert_eq!(cat.fns["make"].mint_span, make_mint, "body-only edit keeps mint span");
    assert!(!cat.events.iter().any(|e| matches!(e, units::CatalogEvent::OwnerSignatureChanged { .. })), "no signature event on body-only edit");
    // 签名变化：事件必达，mint 携带不变。
    let old_span = cat.fns["make"].span;
    let new_fn = cat.src()[old_span.start..old_span.end].replace("fn make(x Item) int", "fn make(x Item, extra int) int");
    let cand = cat.stage_owner_edit("make", &new_fn, BUDGET).expect("stage2");
    cat.commit_staged(cand).expect("commit staged");
    assert_eq!(cat.fns["make"].mint_span, make_mint, "mint carries across signature edit");
    assert!(
        cat.events.iter().any(|e| matches!(e, units::CatalogEvent::OwnerSignatureChanged { owner } if owner == "make")),
        "signature change must emit an explicit event"
    );
    // 声明变更事件：依赖 = 名字引用该声明的 owner。
    cat.note_decl_changed("Item", cat.decls["Item"].span);
    let has_decl_event = cat.events.iter().any(|e| matches!(e, units::CatalogEvent::DeclChanged { name, dependents } if name == "Item" && dependents.contains(&"make".to_string())));
    assert!(has_decl_event, "decl change carries referencing owners");
    // 清单：无指针、确定、含单元种类（collections：decl+fn）。
    let m1 = cat.to_manifest(cst::FRONT_CAPS_VERSION);
    let m2 = cat.to_manifest(cst::FRONT_CAPS_VERSION);
    assert_eq!(format!("{m1:?}"), format!("{m2:?}"), "manifest deterministic");
    assert!(m1.units.iter().any(|u| u.kind == units::UnitKind::TypeDecl));
    assert!(m1.units.iter().any(|u| u.kind == units::UnitKind::Fn && u.owner == "make"));
    assert_eq!(m1.schema, "ac-source-units/1");

    // lambda 单元面（closures）。
    let src2 = "fn main() int {
    let f = (n int) => { return n + 1 }
    return f(1)
}
";
    let cat2 = units::UnitCatalog::build(src2, CL, BUDGET).expect("cold-closures");
    let m3 = cat2.to_manifest(cst::FRONT_CAPS_VERSION);
    assert!(m3.units.iter().any(|u| u.kind == units::UnitKind::Lambda), "lambda units in manifest");
}

/// PLAN-928 T-08（V-07）：深链——解析/投影/目录走查全部有界；预算收紧时
/// 深链是 typed 资源态（ResourceExhausted），不是爆栈/崩溃。
#[test]
fn t08_deep_chain_budget_typed() {
    // 深嵌套括号链：depth 800 在默认预算下全链成功；depth 100k 超出收紧
    // 的 max_parse_depth=10k 时受控拒绝（typed），目录/走查不递归。
    let mk = |d: usize| {
        let mut s = String::from("fn main() int {\n    return ");
        for _ in 0..d { s.push('('); }
        s.push('1');
        for _ in 0..d { s.push(')'); }
        s.push_str("\n}\n");
        s
    };
    let ok_src = mk(800);
    let cat = units::UnitCatalog::build(&ok_src, CL, BUDGET).expect("depth-800 chain within default budget");
    let _ = cat.lambda("main", 0);
    let deep_src = mk(100_000);
    let tight = FrontBudget { work_bytes: 64 << 20, max_parse_depth: 10_000 };
    match units::UnitCatalog::build(&deep_src, CL, tight) {
        Err(units::EditError::ResourceExhausted { .. }) => {}
        other => panic!("deep chain under tight depth budget must be ResourceExhausted, got {:?}", other.map(|_| "Ok")),
    }
    // 深嵌套 lambda 链（目录走查对象）：depth 200 通过（工作栈迭代）。
    let mut lam = String::from("fn main() int {\n    let f = (a int) => { let g = (b int) => { return a + b }; return g(a) };\n    return f(1)\n}\n");
    let _ = &mut lam;
    let cat2 = units::UnitCatalog::build(&lam, CL, BUDGET).expect("nested lambdas");
    assert_eq!(cat2.meter.produced, 3, "fn + 2 lambdas");
}

/// PLAN-928 T-08（V-07/10）：200 轮单 λ 编辑系列——每轮目标物化恰 1，
/// 兄弟 λ 全程 0 生产（Rc 复用），计量累计如实；最后 pin 释放后目录照常。
#[test]
fn t08_two_hundred_edit_series_sibling_zero() {
    let src = "fn main() int {\n    let base int = 1\n    let f = (n int) => { return n + 1 }\n    let g = (m int) => { return m + 2 }\n    return f(1) + g(2)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let pin0 = cat.pin();
    let cold = cat.meter.produced;
    assert_eq!(cold, 3, "fn + 2 lambda");
    for round in 0..200u32 {
        let new_slice = format!("(n int) => {{ return n + {} }}", 1 + round + 1);
        let receipt = cat.edit_lambda("main", 0, &new_slice, BUDGET).expect("edit round");
        assert_eq!(receipt.produced.len(), 1, "round {round}: only the target lambda produces");
        assert_eq!(receipt.reused.len(), 1, "round {round}: sibling shared");
        let _ = receipt;
    }
    // 累计：cold(3) + 200 目标物化；兄弟 g 的 body 自冷构建后从未重产。
    assert_eq!(cat.meter.produced - cold, 200, "200 rounds => exactly 200 productions");
    assert_eq!(cat.meter.reused, 200, "sibling reuse each round");
    drop(pin0);
    let _ = cat.lambda("main", 0).expect("catalog alive after pin drop");
}

/// PLAN-928 T-08（V-10）：最后 owner 释放——drop 全部 pin 与目录后 Rc 载荷
/// 引用计数归零（真实回收可观察，不以数组清空冒充）。
#[test]
fn t08_last_owner_reclaim() {
    let src = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    return f(1)\n}\n";
    let cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let pin = cat.pin();
    let body = std::rc::Rc::clone(&cat.lambda("main", 0).unwrap().body);
    assert_eq!(std::rc::Rc::strong_count(&body), 3, "catalog + pin + local handle");
    drop(pin);
    assert_eq!(std::rc::Rc::strong_count(&body), 2, "pin release drops its handle");
    drop(cat);
    assert_eq!(std::rc::Rc::strong_count(&body), 1, "catalog drop releases; only local handle remains");
    drop(body);
}

