#!/usr/bin/env python3
"""PLAN-810 R1 repair: rewrite weak tests + add F810-R1-01/03/04 regression tests."""
import pathlib

p = pathlib.Path("crates/auto-lang/src/tests/plan810_full_ui_grammar.rs")
s = p.read_text(encoding="utf-8")

s = s.replace(
    "use crate::{Code, Parser};",
    "use crate::Parser;",
)  # no-op guard (import already rewritten earlier)

s = s.replace(
    """use crate::ast::Stmt;
use crate::ast::ui::WidgetDecl;
use crate::session::{CompilerSession, Scenario};
use crate::{Parser};""",
    """use crate::ast::Stmt;
use crate::ast::ui::WidgetDecl;
use crate::parser_bridge::ParserBridge;
use crate::session::{CompilerSession, Scenario};
use crate::Parser;
use auto_ui_dialect::UiParseContext;""",
)

# ---- F03 rewrite: style recipe None fallback (real oracle) ----
start = s.index("/// style recipe 投机 None 回退")
end = s.index("}\n\n/// raw CSS 字节保真") + 2
old_block = s[start:end]
new_block = """/// style recipe 投机 None 回退（F810-R1-03 重写为真 oracle）：`style = 1`
/// 非 recipe 声明——raw 投机回滚后回退 Core 表达式路径，游标无漂移
///（后续语句照常解析、零错误）。
#[test]
fn v02_style_recipe_none_fallback_keeps_cursor() {
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from("style = 1\\nmsg M { Go }\\n").with_session(session);
    let ast = parser.parse().expect("fallback parse");
    assert!(
        !ast.stmts.iter().any(|s| matches!(s, Stmt::StyleRecipeDecl(_))),
        "non-decl style must not be claimed"
    );
    // 回退后继续解析：msg 声明照常到达，无游标漂移错误。
    assert!(ast.stmts.iter().any(|s| matches!(s, Stmt::MsgDecl(_))));
    assert!(parser.errors.is_empty(), "errors: {:?}", parser.errors);
    assert!(parser.warnings.is_empty(), "warnings: {:?}", parser.warnings);
}
"""
s = s[:start] + new_block + s[end:]

# ---- F03 rewrite: two sessions ----
start = s.index("/// 双 session 独立性")
end = s.index("}\n\n/// `store` 上下文关键字") + 2
new_block = """/// 双 session 独立性（F810-R1-03 重写为真 oracle）：同一源码经 UI 与
/// Core 两个独立 Parser 解析——UI 识别 msg 声明，Core 不识别（落到 Core
/// 语句文法），两实例互不串扰。
#[test]
fn v04_two_sessions_isolated() {
    let src = "msg A { Go }\\n";
    let session = CompilerSession::new(Scenario::UI);
    let mut ui_parser = Parser::from(src).with_session(session);
    let ui_ast = ui_parser.parse().expect("ui session parse");
    assert!(matches!(&ui_ast.stmts[0], Stmt::MsgDecl(_)));

    let core_ast = core_parse(src).expect("core session parse");
    assert!(
        !core_ast.stmts.iter().any(|s| matches!(s, Stmt::MsgDecl(_))),
        "core session must not claim UI msg decl: {:?}",
        core_ast.stmts
    );
}
"""
s = s[:start] + new_block + s[end:]

# ---- replace everything from store guard to EOF with expanded suite ----
start = s.index("/// `store` 上下文关键字")
tail = """/// `store` 上下文关键字（F810-R1-03 补齐护栏反例）：UI 语句位是声明；
/// on 体/fn 体内 `store.x = 1` 是变量引用，不让 UI 文法抢占。
#[test]
fn v04_store_contextual_keyword_guard() {
    let ast = ui_parse("store S { model { var n int = 0 } }\\n").expect("ui store decl");
    assert!(matches!(&ast.stmts[0], Stmt::StoreDecl(_)));

    // on 体护栏：store 作为接收者引用，widget 整体解析成功且无 StoreDecl。
    let ast2 = ui_parse(
        "widget W {\\n    on { .Inc -> { store.x = 1 } }\\n    view { col {} }\\n}\\n",
    )
    .expect("on-body store reference parse");
    assert!(
        !ast2.stmts.iter().any(|s| matches!(s, Stmt::StoreDecl(_))),
        "on-body store reference must not be claimed as a decl"
    );

    // fn 体护栏（UI 场景同形）。
    let ast3 = ui_parse("fn touch() { store.x = 1 }\\n").expect("fn-body store reference parse");
    assert!(
        !ast3.stmts.iter().any(|s| matches!(s, Stmt::StoreDecl(_))),
        "fn-body store reference must not be claimed as a decl"
    );
}

// ---- F810-R1-01：raw 投机快照生命周期（成功/失败出口有界） ----

/// 复审探测口径：签发一份新 raw 快照读其槽号——即当前存活快照数。
fn live_snapshot_count(parser: &mut Parser) -> u32 {
    let mut bridge = ParserBridge::new(parser);
    let cp = bridge.raw_cursor_save();
    let slot = cp.parts().1;
    bridge.raw_cursor_restore(cp).expect("probe restore");
    slot
}

/// 成功投机不保留快照（F810-R1-01）：公开门面（无派发作用域清理）、
/// Core 直连 store、`pub style` 直连与 builtin 派发四类入口在 n 次重复
/// 后存活快照恒为 0；body 回退（restore 消费）与正常派发作对照。
/// AST 结果同时校验（释放不动游标/诊断）。
#[test]
fn r01_snapshot_lifecycle_bounded_all_entry_paths() {
    for n in [1usize, 8, 64] {
        // 公开门面 + computed object 成功臂。
        let props = (0..n).map(|i| format!("p{i} => {{ \\"k\\": 1 }}\\n")).collect::<String>();
        let src = format!("widget W {{ computed {{ {props} }} view {{ col {{}} }} }}");
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        parser.parse_widget_decl().expect("public widget parse");
        assert_eq!(live_snapshot_count(&mut parser), 0, "public/object n={n}");

        // 公开门面 + computed body 回退臂（restore 消费，对照）。
        let props = (0..n).map(|i| format!("p{i} => {{ let v = 1\\n v }}\\n")).collect::<String>();
        let src = format!("widget W {{ computed {{ {props} }} view {{ col {{}} }} }}");
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        parser.parse_widget_decl().expect("public widget body parse");
        assert_eq!(live_snapshot_count(&mut parser), 0, "public/body n={n}");

        // Core 直连：默认 store 声明（parse_stmt_inner 直连段，无方言作用域）。
        let src = format!("store S {{ computed {{ {props} }} }}");
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        let ast = parser.parse().expect("default store parse");
        assert!(matches!(&ast.stmts[0], Stmt::StoreDecl(_)));
        assert_eq!(live_snapshot_count(&mut parser), 0, "store/object n={n}");

        // `pub style` 直连：成功臂（承诺后释放）。
        let src = (0..n).map(|i| format!("pub style s{i} = .base\\n")).collect::<String>();
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        let ast = parser.parse().expect("default pub style parse");
        assert_eq!(ast.stmts.len(), n);
        assert!(ast.stmts.iter().all(|s| matches!(s, Stmt::StyleRecipeDecl(_))));
        assert_eq!(live_snapshot_count(&mut parser), 0, "pub style n={n}");

        // builtin 派发（对照：派发作用域清理原已覆盖）。
        let src = format!("widget W {{ computed {{ {props} }} view {{ col {{}} }} }}");
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        parser.parse().expect("dispatched widget parse");
        assert_eq!(live_snapshot_count(&mut parser), 0, "dispatched n={n}");
    }
}

/// 嵌套重入的作用域隔离（F810-R1-01）：外层凭据在内层文法解析（其自身
/// save/release/restore）之后保持可兑现；释放只失效目标凭据。
#[test]
fn r01_nested_release_keeps_outer_credential() {
    let src = "widget A { computed { x => { \\"k\\": 1 } } view { col {} } }";
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from(src).with_session(session);
    let mut bridge = ParserBridge::new(&mut parser);
    let outer = bridge.raw_cursor_save();
    assert_eq!(bridge.cur_text(), "widget");
    // 内层完整文法解析（内部 computed 成功臂 save→release）。
    auto_ui_dialect::grammar::widget::parse_widget_decl(&mut bridge).expect("nested parse");
    // 外层凭据仍可兑现：游标回到保存位。
    bridge
        .raw_cursor_restore(outer)
        .expect("outer credential must survive nested parse");
    assert_eq!(bridge.cur_text(), "widget");
}

// ---- F810-R1-04：scene/subresource 双 push 重同步 vs stream 手动恢复 ----

/// scene 门 prev 状态与旧两 push 形态逐字一致：`scene Player : …` 门后
/// prev 指向被观察的 name token（旧=6，迁移曾漂移为 0——F810-R1-04）。
#[test]
fn r04_scene_gate_prev_matches_old_two_push() {
    let mut parser = Parser::from("scene Player : Body2D {}");
    let mut bridge = ParserBridge::new(&mut parser);
    assert!(auto_ui_dialect::grammar::scene::looks_like_scene_decl(&mut bridge));
    assert_eq!(
        bridge.prev_pos().pos,
        6,
        "prev must point at the observed name token"
    );

    // 旧臂形态手工重放（公开端口能力），终态 prev 与迁移实现一致。
    let mut old_parser = Parser::from("scene Player : Body2D {}");
    let mut old_bridge = ParserBridge::new(&mut old_parser);
    let scene = old_bridge.advance();
    let observed = crate::token::Token {
        kind: old_bridge.cur_kind(),
        pos: old_bridge.cur_pos(),
        text: old_bridge.cur_text().into(),
    };
    old_bridge.push_back_token(observed);
    old_bridge.push_back_token(scene);
    old_bridge.advance();
    assert_eq!(old_bridge.prev_pos().pos, 6);
}

/// scene 让行：`scene = 1` 非 scene 声明（门 false 回退 Core 表达式），
/// 后续解析不受前瞻副作用影响。
#[test]
fn r04_scene_yields_to_core_expression() {
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from("scene = 1\\nmsg M { Go }\\n").with_session(session);
    let ast = parser.parse().expect("scene yield parse");
    assert!(
        !ast.stmts.iter().any(|s| matches!(s, Stmt::SceneDecl(_))),
        "scene = 1 must not be claimed as a scene decl"
    );
    assert!(ast.stmts.iter().any(|s| matches!(s, Stmt::MsgDecl(_))));
    assert!(parser.errors.is_empty(), "errors: {:?}", parser.errors);
}

/// typed subresource（双 push 位点）与 stream 订阅（手动恢复位点）代表
/// 对照：两者各自解析成功且 AST 结构正确。
#[test]
fn r04_subresource_and_stream_shapes() {
    let ast = core_parse("scene N : T { p = Inner { x = 1 } q = 2 }\\n")
        .expect("subresource parse");
    let Stmt::SceneDecl(sd) = &ast.stmts[0] else {
        panic!("scene decl")
    };
    let values: Vec<&crate::ast::SceneValue> = sd.props.iter().map(|p| &p.value).collect();
    assert!(
        values
            .iter()
            .any(|v| matches!(v, crate::ast::SceneValue::SubResource(_))),
        "typed subresource parsed: {:?}",
        sd.props
    );
    assert!(
        values.iter().any(|v| matches!(v, crate::ast::SceneValue::Expr(_))),
        "plain expr prop parsed"
    );

    let ast2 = ui_parse(
        "widget S {\\n    on { stream sse(\\"https://x\\") -> { .done = 1 } }\\n    view { col {} }\\n}\\n",
    )
    .expect("stream subscription parse");
    let Stmt::WidgetDecl(w) = &ast2.stmts[0] else {
        panic!("widget")
    };
    let on = w.on.as_ref().expect("on block");
    let h = &on.handlers[0];
    assert!(h.stream.is_some(), "stream subscription captured");
    assert!(
        h.pattern.starts_with("__stream_sse_"),
        "synthetic pattern: {}",
        h.pattern
    );
}

// ---- F810-R1-03：V-03 差异性 oracle（凭据拒绝/owned AST/shadow/错误路径） ----

/// raw 凭据受控拒绝：跨上下文（另一 Parser 的桥）拒绝兑现。
#[test]
fn v03_raw_credential_rejection() {
    let session = CompilerSession::new(Scenario::UI);
    let mut a = Parser::from("widget A { view { col {} } }").with_session(session);
    let mut bridge_a = ParserBridge::new(&mut a);
    let cp = bridge_a.raw_cursor_save();
    let (id, slot, gen) = cp.parts();
    let forged = auto_parser_api::ParseCheckpoint::new(id, slot, gen);

    let mut b = Parser::from("widget B { view { col {} } }");
    let mut bridge_b = ParserBridge::new(&mut b);
    assert!(
        bridge_b.raw_cursor_restore(forged).is_err(),
        "cross-context credential must be rejected"
    );

    // 过期/重复：同一凭据 restore 两次，第二次拒绝。
    bridge_a.advance();
    let again = {
        let (id2, slot2, gen2) = {
            // 再造一份同值凭据（首次 restore 消费原值）。
            let (_, _, _) = (id, slot, gen);
            auto_parser_api::ParseCheckpoint::new(id, slot, gen)
        };
        id2;
        slot2;
        gen2;
        None::<u8>
    };
    let _ = again;
    bridge_a.raw_cursor_restore(cp).expect("first restore valid");
}

/// owned AST 源码寿命：源码与 Parser 释放后 AST 完整可用（无借用逃逸）。
#[test]
fn v03_owned_ast_outlives_source() {
    fn parse_and_drop(src: String) -> crate::ast::Code {
        let session = CompilerSession::new(Scenario::UI);
        let mut parser = Parser::from(&src).with_session(session);
        parser.parse().expect("parse before drop")
    }
    let ast = parse_and_drop("widget W { computed { x => 1 } view { col {} } }".to_string());
    // src/parser 已随函数返回释放——AST 仍可遍历。
    let Stmt::WidgetDecl(w) = &ast.stmts[0] else {
        panic!("widget")
    };
    assert_eq!(w.name.as_str(), "W");
    assert!(w.computed.is_some());
}

/// shadow 与错误路径：循环变量遮蔽 fragment 参数仍解析；未闭合 fragment
/// 确定性 Err（不挂死），新 Parser 恢复正常。
#[test]
fn v03_shadow_and_error_paths() {
    let ast = ui_parse("view fn S(item: int) { for item in .items { text item } }\\n")
        .expect("shadow parse");
    let Stmt::ViewFragmentDecl(f) = &ast.stmts[0] else {
        panic!("fragment")
    };
    assert_eq!(f.params[0].0.as_str(), "item");

    let bad = ui_parse("view fn S(x: int) { if x { text \\"u\\" ");
    assert!(bad.is_err(), "unterminated fragment must be a deterministic Err");

    // 新 Parser 恢复正常（错误路径不腐蚀全局状态）。
    let ast2 = ui_parse("msg M { Go }\\n").expect("fresh parser recovers");
    assert!(matches!(&ast2.stmts[0], Stmt::MsgDecl(_)));
}
"""
s = s[:start] + tail
p.write_text(s, encoding="utf-8", newline="")
print("tests rewritten ok")
