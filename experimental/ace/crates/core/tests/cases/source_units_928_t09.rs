//! PLAN-928 T-09：V 面正式化（文件化 suite 之外的 in-process 反控面）。
//!
//! - V-03 零编辑重放：同字节切片 ⇒ 生产 0、身份 Rc 同一、无重验义务；
//! - V-05 stale/foreign 候选 typed 拒绝（零变异）+ collections 半 token 坏
//!   候选生命周期；
//! - V-06 前缀漂移（坐标平移、mint/指纹/Rc 不变）；
//! - V-08 清单 codec 读回全等 + 伪造/未知结构消费前拒绝 + 合法近邻；
//! - V-09 λ 重排/删除（序-文错位如实重产；拓扑事件）；
//! - V-04 locals 顺序/类型变化仍带语义重验义务；
//! - V-07 大 source 目录构建有界。

use ace_core::source::cst::{self, units, FrontBudget};
use ace_core::source::SourceProfile;
use std::rc::Rc;

const CL: SourceProfile = SourceProfile::Closures;
const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;

/// V-03：零编辑重放——同字节切片重放 ⇒ 全单元生产 0、身份 Rc 同一、
/// 无语义重验义务；owner 面与暂存面同合同。
#[test]
fn t09_zero_edit_replay_zero_production() {
    let src = "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n }\n    let g = (m int) => { return f(m) + 1 }\n    return f(2) + g(3)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let cold = cat.meter.clone();
    let f_ptr = Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize;
    let g_ptr = Rc::as_ptr(&cat.lambda("main", 1).unwrap().body) as usize;
    let owner_ptr = Rc::as_ptr(&cat.fns["main"].item) as usize;
    // λ 面零编辑：同字节重放。
    let span = cat.lambda("main", 0).unwrap().span;
    let same = cat.src()[span.start as usize..span.end as usize].to_string();
    let r = cat.edit_lambda("main", 0, &same, BUDGET).expect("zero-edit lambda replay");
    assert!(r.produced.is_empty(), "零编辑：生产 0");
    assert_eq!(r.reused.len(), 2, "目标+兄弟都计复用");
    assert!(!r.semantic_recheck_required, "源未变 ⇒ 无语义重验义务");
    assert_eq!(cat.meter.produced, cold.produced, "生产计数不动");
    assert_eq!(cat.meter.reused, cold.reused + 2, "复用如实 +2");
    assert_eq!(Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize, f_ptr, "身份 Rc 同一");
    assert_eq!(Rc::as_ptr(&cat.lambda("main", 1).unwrap().body) as usize, g_ptr, "兄弟不动");
    assert_eq!(Rc::as_ptr(&cat.fns["main"].item) as usize, owner_ptr, "owner 不重产");
    // owner 面零编辑。
    let ospan = cat.fns["main"].span;
    let same_fn = cat.src()[ospan.start as usize..ospan.end as usize].to_string();
    let r2 = cat.edit_owner("main", &same_fn, BUDGET).expect("zero-edit owner replay");
    assert!(r2.produced.is_empty() && r2.reused.len() == 3, "owner+2λ 全复用");
    assert!(!r2.semantic_recheck_required);
    // 暂存面零编辑 + 提交诚实回执。
    let cand = cat.stage_owner_edit("main", &same_fn, BUDGET).expect("zero-edit stage");
    assert_eq!(cand.produced(), 0);
    assert!(cand.semantic_recheck().is_empty(), "零编辑候选不携带重验义务");
    let r3 = cat.commit_staged(cand).expect("commit zero-edit");
    assert!(r3.produced.is_empty() && r3.reused.len() == 3);
    assert_eq!(cat.meter.produced, cold.produced, "全程生产 0");
}

/// V-05/§5.4：stale 候选——staging 后目录推进 ⇒ typed 拒绝、零变异；
/// 重新暂存后提交正常。
#[test]
fn t09_stale_candidate_rejected_zero_mutation() {
    let src = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    let g = (m int) => { return m + 2 }\n    return f(1) + g(2)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    // 对同一基修订暂存两个候选。
    let new_fn_a = "fn main() int {\n    let f = (n int) => { return n + 11 }\n    let g = (m int) => { return m + 2 }\n    return f(1) + g(2)\n}\n";
    let new_fn_b = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    let g = (m int) => { return m + 22 }\n    return f(1) + g(2)\n}\n";
    let cand_a = cat.stage_owner_edit("main", new_fn_a, BUDGET).expect("stage a");
    let cand_b = cat.stage_owner_edit("main", new_fn_b, BUDGET).expect("stage b");
    // 提交 a：推进修订。
    cat.commit_staged(cand_a).expect("commit a");
    let after_a = (cat.src().to_string(), cat.meter.clone());
    // 提交 b（基修订已过）：stale 拒绝，目录零变异。
    match cat.commit_staged(cand_b) {
        Err(units::EditError::StaleForeign(msg)) => assert!(msg.contains("stale"), "stale 具名：{msg}"),
        other => panic!("stale candidate must be rejected, got {:?}", other.map(|_| "Ok")),
    }
    assert_eq!(cat.src(), after_a.0, "stale 拒绝不改源");
    assert_eq!(cat.meter, after_a.1, "stale 拒绝不动计量");
    // 以当前修订重新暂存同内容 b：提交成功。
    let cand_b2 = cat.stage_owner_edit("main", new_fn_b, BUDGET).expect("restage b");
    cat.commit_staged(cand_b2).expect("commit restaged b");
}

/// V-05/§5.4：foreign 候选——另一目录实例铸造的候选提交到本目录 ⇒
/// typed 拒绝（同源文本也不行——instance 区分）、零变异。
#[test]
fn t09_foreign_candidate_rejected_zero_mutation() {
    let src = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    return f(1)\n}\n";
    let mut cat_a = units::UnitCatalog::build(src, CL, BUDGET).expect("cold a");
    let cat_b = units::UnitCatalog::build(src, CL, BUDGET).expect("cold b");
    assert_eq!(cat_a.module, cat_b.module, "同源文本 module 相同（这正是需要 instance 的原因）");
    assert_ne!(cat_a.instance, cat_b.instance);
    let new_fn = "fn main() int {\n    let f = (n int) => { return n + 41 }\n    return f(1)\n}\n";
    let foreign = cat_b.stage_owner_edit("main", new_fn, BUDGET).expect("stage in b");
    let before = (cat_a.src().to_string(), cat_a.meter.clone());
    match cat_a.commit_staged(foreign) {
        Err(units::EditError::StaleForeign(msg)) => assert!(msg.contains("foreign"), "foreign 具名：{msg}"),
        other => panic!("foreign candidate must be rejected, got {:?}", other.map(|_| "Ok")),
    }
    assert_eq!(cat_a.src(), before.0);
    assert_eq!(cat_a.meter, before.1);
}

/// V-05：collections 坏候选（半 token）→ 诊断 + 目录零变异；修复后提交正确。
#[test]
fn t09_collections_bad_candidate_half_token() {
    let src = "type Item { name Str; code int }\nfn make(x Item) int {\n    return x.code\n}\nfn main() int {\n    let a Item = Item(\"k\", 41)\n    return make(a.move)\n}\n";
    let mut cat = units::UnitCatalog::build(src, SourceProfile::Collections, BUDGET).expect("cold");
    let old = (cat.src().to_string(), cat.meter.clone(), cat.events.clone());
    // 坏 owner（字符串半 token 未闭合）。
    let broken = "fn main() int {\n    let a Item = Item(\"k, 41)\n    return make(a.move)\n}\n";
    match cat.stage_owner_edit("main", broken, BUDGET) {
        Err(units::EditError::Diag(_)) => {}
        other => panic!("broken candidate must diagnose, got {:?}", other.map(|_| "Ok")),
    }
    assert_eq!(cat.src(), old.0, "坏候选不改源");
    assert_eq!(cat.meter, old.1, "坏候选不动计量");
    assert_eq!(cat.events, old.2, "坏候选不发事件");
    // 修复后提交。
    let fixed = "fn main() int {\n    let a Item = Item(\"k\", 41)\n    return make(a.move) + 1\n}\n";
    let cand = cat.stage_owner_edit("main", fixed, BUDGET).expect("stage fixed");
    cat.commit_staged(cand).expect("commit fixed");
    assert!(cat.src().contains("+ 1"));
}

/// V-09/V-06：同 owner λ 重排（异文）= 序-文错位 ⇒ 如实重产（不假 0）；
/// 删除一个 λ = 拓扑变化（具名事件）。
#[test]
fn t09_lambda_reorder_and_delete() {
    let src = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    let g = (m int) => { return m + 2 }\n    return f(1) + g(2)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    // 重排：g 在前 f 在后（异文错位）。
    let reordered = "fn main() int {\n    let g = (m int) => { return m + 2 }\n    let f = (n int) => { return n + 1 }\n    return f(1) + g(2)\n}\n";
    let cand = cat.stage_owner_edit("main", reordered, BUDGET).expect("stage reorder");
    assert_eq!(cand.produced(), 1 + 2, "owner + 两个 λ 都重产（序-文错位，不假 0）");
    assert_eq!(cand.reused(), 0);
    cat.commit_staged(cand).expect("commit reorder");
    // 删除 g：拓扑变化。
    let deleted = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    return f(1)\n}\n";
    let cand2 = cat.stage_owner_edit("main", deleted, BUDGET).expect("stage delete");
    // 拓扑变化：owner(1) + 剩余 λ 重产（直路 edit_owner 的非快路径合同在
    // t02_same_text_identity_rules 已证；此处为暂存提交面计数）。
    assert_eq!(cand2.produced(), 1 + 1, "拓扑变化：owner + 剩余 λ 重产");
    let events_before = cat.events.clone();
    cat.commit_staged(cand2).expect("commit delete");
    let new_events: Vec<_> = cat.events.iter().skip(events_before.len()).collect();
    assert!(
        new_events.iter().any(|e| matches!(e, units::CatalogEvent::LambdaTopologyChanged { owner } if owner == "main")),
        "删除 λ 必发拓扑事件（暂存提交面）"
    );
    // λ 目录只剩 1 个。
    assert!(cat.lambda("main", 0).is_some() && cat.lambda("main", 1).is_none());
}

/// V-06：前缀漂移——上方 owner 变长 ⇒ 下方 owner 坐标平移，但内容指纹与
/// mint 身份不变、Rc 同一（坐标漂移不换身份，§5.4）。
#[test]
fn t09_prefix_drift_identity_stable() {
    let src = "fn top() int {\n    return 1\n}\nfn below() int {\n    let f = (n int) => { return n + 2 }\n    return f(1)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let below = &cat.fns["below"];
    let (old_span, old_mint, old_sha) = (below.span, below.mint_span, below.content_sha);
    let below_ptr = Rc::as_ptr(&cat.fns["below"].item) as usize;
    let lam = cat.lambda("below", 0).unwrap();
    let (lam_old_sha, lam_old_span) = (lam.content_sha, lam.span);
    let lam_ptr = Rc::as_ptr(&lam.body) as usize;
    // top 加一行（更长）⇒ below 整体后移。
    let top_span = cat.fns["top"].span;
    let new_top = cat.src()[top_span.start as usize..top_span.end as usize]
        .replace("return 1", "let pad int = 7\n    return 1 + pad");
    cat.edit_owner("top", &new_top, BUDGET).expect("edit top");
    let below2 = &cat.fns["below"];
    assert!(below2.span.start > old_span.start, "below 坐标后移");
    assert_eq!(below2.mint_span, old_mint, "mint 身份不随漂移变");
    assert_eq!(below2.content_sha, old_sha, "切片内容指纹不变");
    assert_eq!(Rc::as_ptr(&cat.fns["below"].item) as usize, below_ptr, "below Rc 同一");
    assert_eq!(Rc::as_ptr(&cat.lambda("below", 0).unwrap().body) as usize, lam_ptr, "below λ Rc 同一");
    // λ 当前坐标随后移（span 更新），指纹不变。
    let lam2 = cat.lambda("below", 0).unwrap();
    assert!(lam2.span.start > lam_old_span.start, "λ 当前坐标漂移");
    assert_eq!(lam2.content_sha, lam_old_sha, "λ 指纹不变");
}

/// V-08：清单 codec——编码确定、读回全等；伪造/未知结构消费前拒绝；
/// 合法近邻（另一目录）可解码但身份比对不等。
#[test]
fn t09_manifest_codec_roundtrip_and_forgery() {
    // Collections 复合面（type/enum/双 fn；collections 顶层无 lambda——
    // T-04 合同，λ 单元的清单面由 closures 目录覆盖）。
    let src = "type Item { name Str; code int }\nenum Shape { Dot Circle(int) }\nfn make(x Item) int {\n    return x.code\n}\nfn main() int {\n    let a Item = Item(\"k\", 1)\n    return make(a.move)\n}\n";
    let cat = units::UnitCatalog::build(src, SourceProfile::Collections, BUDGET).expect("cold");
    let m = cat.to_manifest(cst::FRONT_CAPS_VERSION);
    let enc1 = units::encode_manifest(&m);
    let enc2 = units::encode_manifest(&m);
    assert_eq!(enc1, enc2, "编码确定");
    let dec = units::decode_manifest(&enc1).expect("roundtrip decode");
    assert!(dec.matches_manifest(&m), "读回与当次导出全等");
    assert_eq!(dec.profile, "collections", "profile 用来源合同 slug（非 grammar id）");
    // λ 单元的清单面：closures 目录。
    let cl_src = "fn main() int {\n    let f = (n int) => { return n + 1 }\n    return f(1)\n}\n";
    let cat_l = units::UnitCatalog::build(cl_src, CL, BUDGET).expect("cold-closures");
    let m_l = cat_l.to_manifest(cst::FRONT_CAPS_VERSION);
    assert!(m_l.units.iter().any(|u| u.kind == units::UnitKind::Lambda), "λ 入清单");
    let dec_l = units::decode_manifest(&units::encode_manifest(&m_l)).expect("decode closures");
    assert!(dec_l.matches_manifest(&m_l));
    assert_eq!(dec_l.profile, "closures");
    // 读回重验后继续单元编辑（V-08：目录仍可用）。
    let mut cat2 = cat;
    let main_span = cat2.fns["main"].span;
    let new_main = cat2.src()[main_span.start..main_span.end].replace("make(a.move)", "make(a.move) + 1");
    cat2.edit_owner("main", &new_main, BUDGET).expect("edit after readback");
    let m2 = cat2.to_manifest(cst::FRONT_CAPS_VERSION);
    assert!(!dec.matches_manifest(&m2), "编辑后旧读回不再全等（身份面过时）");
    // 伪造：坏 schema。
    let bad_schema = enc1.replacen("ac-source-units/1", "ac-source-units/9", 1);
    assert!(units::decode_manifest(&bad_schema).is_err(), "坏 schema 拒绝");
    // 未知 profile。
    let bad_profile = enc1.replacen("profile collections", "profile mega-profile", 1);
    assert!(units::decode_manifest(&bad_profile).is_err(), "未知 profile 拒绝");
    // 未知 unit kind。
    let bad_kind = enc1.replacen("unit fn ", "unit mystery ", 1);
    assert!(units::decode_manifest(&bad_kind).is_err(), "未知 kind 拒绝");
    // 坏数字（ordinal）。
    let bad_num = enc1.replacen("unit fn ", "unit fn x ", 1);
    assert!(units::decode_manifest(&bad_num).is_err(), "坏 ordinal 拒绝");
    // 中段空行。
    let blank = enc1.replacen("\nunit", "\n\nunit", 1);
    assert!(units::decode_manifest(&blank).is_err(), "空行拒绝");
    // 数字合法但内容指纹被篡改：可解码，身份比对拒绝（不冒信任）。
    let mut lines: Vec<String> = enc1.trim_end_matches('\n').split('\n').map(String::from).collect();
    for ln in lines.iter_mut() {
        if ln.starts_with("unit fn ") {
            let mut p: Vec<String> = ln.split(' ').map(String::from).collect();
            p[4] = "12345".to_string();
            *ln = p.join(" ");
            break;
        }
    }
    let forged = lines.join("\n");
    let dec_forged = units::decode_manifest(&forged).expect("结构合法可解码");
    assert!(!dec_forged.matches_manifest(&m), "篡改指纹必须被身份比对拒绝");
    // 合法近邻：另一目录（不同源）解码成功但比对不等。
    let other = units::UnitCatalog::build("fn main() int {\n    return 1\n}\n", SourceProfile::Collections, BUDGET).expect("cold other");
    let m_other = other.to_manifest(cst::FRONT_CAPS_VERSION);
    let dec_other = units::decode_manifest(&units::encode_manifest(&m_other)).expect("neighbor decodes");
    assert!(!dec_other.matches_manifest(&m), "近邻不误判为本目录");
}

/// V-04：owner 创建点 locals 顺序/类型变化、λ 文本不变 ⇒ 语法体共享 +
/// 语义重验义务照带（顺序/类型面与写性面同信号；义务面不精简）。
#[test]
fn t09_owner_locals_order_and_type_change_signal() {
    let src = "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n }\n    return f(2)\n}\n";
    let mut cat = units::UnitCatalog::build(src, CL, BUDGET).expect("cold");
    let lam_ptr = Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize;
    // 顺序变化：在 base 前插入新 local。
    let reordered = "fn main() int {\n    let pad Str = \"x\"\n    let base int = 40\n    let f = (n int) => { return base + n }\n    return f(2)\n}\n";
    let cand = cat.stage_owner_edit("main", reordered, BUDGET).expect("stage order");
    assert_eq!(cand.produced(), 1, "仅 owner 生产");
    assert_eq!(cand.reused(), 1, "同文 λ 语法共享");
    assert_eq!(cand.semantic_recheck(), &[0], "顺序变化仍携带重验义务");
    let receipt = cat.commit_staged(cand).expect("commit");
    assert!(receipt.semantic_recheck_required);
    assert_eq!(Rc::as_ptr(&cat.lambda("main", 0).unwrap().body) as usize, lam_ptr, "λ 体 Rc 同一");
    // 类型变化（λ 未引用的独立 local）：语法共享照常，重验义务不精简。
    let typed = "fn main() int {\n    let pad Str = \"xy\"\n    let base int = 40\n    let f = (n int) => { return base + n }\n    return f(2)\n}\n";
    let cand2 = cat.stage_owner_edit("main", typed, BUDGET).expect("stage type");
    assert_eq!(cand2.reused(), 1, "λ 文本未变仍共享");
    assert!(!cand2.semantic_recheck().is_empty(), "类型变化仍带重验义务");
}

/// V-07：大 source 目录构建有界（几千单元；工作栈迭代，深度只受预算约束）。
#[test]
fn t09_large_source_catalog_bounded() {
    let mut src = String::new();
    let n = 1500usize;
    for i in 0..n {
        src.push_str(&format!("fn f{i}() int {{\n    return {i}\n}}\n"));
    }
    let cat = units::UnitCatalog::build(&src, CL, BUDGET).expect("large cold build");
    assert_eq!(cat.fns.len(), n);
    assert_eq!(cat.meter.produced, n, "每 fn 计产 1");
    let m = cat.to_manifest(cst::FRONT_CAPS_VERSION);
    assert_eq!(m.units.len(), n);
    assert!(units::decode_manifest(&units::encode_manifest(&m)).unwrap().matches_manifest(&m));
}

/// T-10（真缺陷回归）：括号组后随点步 ⇒ Method 接收者基座，非间接调用。
/// T-07 首版把 `(a + "z").len()` 误投 CallValue（Text profile a2c_rt
/// ledger 面暴露）；`(f)(3)`（无点步）仍是间接调用。差分 vs 手写。
#[test]
fn t10_paren_group_receiver_vs_indirect_callee() {
    use ace_core::source::cst::parse_program_via_cst;
    use ace_core::source::parse_program_with_handwritten;
    let text_src = "fn main() int {
    let a Str = \"xy\"
    let hit bool = a.to(str) == \"x\" || (a + \"z\").len() == 3
    if hit {
        return 1
    }
    return a.len()
}
";
    for (src, profile, what) in [
        (text_src, SourceProfile::Text, "text (group).method()"),
        ("fn main() int {
    let f = (n int) => { return n + 1 }
    return (f)(3)
}
", CL, "closures (f)(3) indirect"),
        ("fn main() int {
    let g = (n int) => { return n * 2 }
    return (g)(3) + (g)(1).max2(2)
}
", CL, "closures mixed"),
    ] {
        let old = parse_program_with_handwritten(src, profile);
        let new = parse_program_via_cst(src, profile);
        match (&old, &new) {
            (Ok(a), Ok(b)) => assert_eq!(
                format!("{a:?}"), format!("{b:?}"),
                "{what}: AST mismatch old(handwritten) vs new(CST)"
            ),
            (Err(a), Err(b)) => assert_eq!(
                (a.stage, a.code.as_str(), a.message.as_str(), a.span),
                (b.stage, b.code.as_str(), b.message.as_str(), b.span),
                "{what}: diag mismatch"
            ),
            _ => panic!("{what}: outcome mismatch"),
        }
    }
}
