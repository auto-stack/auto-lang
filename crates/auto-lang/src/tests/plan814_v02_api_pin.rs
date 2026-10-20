//! PLAN-814 T-05：V-02 旧公开 API 编译 pin（AC-02）。
//!
//! 旧 `auto_lang::*` 消费面的**编译期锁**：Parser pub 字段/构造族/方言注册、
//! ParserBridge 方法面、AutoVM pub 字段/构造、NativeInterface 注册族、
//! feature 门控根再导出、UI 动态管线入口签名、以及新旧包**类型身份同一**
//! （旧路径与 auto_ui_runtime/auto_lang_core 是同一类型，无双 Parser/VM）。
//! 任一旧公开项被删除/改签名 = 本模块编译红（比运行断言更强的 pin 形态）。
//! 运行期仅做最小语义冒烟（默认组合装配在 plan806/scene/dialect 族另测）。

#![allow(dead_code)]

/// Parser 17 个 pub 字段 + 构造/装配族编译 pin。
#[test]
fn plan814_v02_parser_public_surface_pins() {
    use crate::parser::Parser;
    use crate::session::CompilerSession;

    let src = "fn main() int { return 1 }";
    let mut cs = crate::compile::CompileSession::new();
    let mut p =
        Parser::new_with_type_store(src, cs.type_store()).with_session(CompilerSession::ui());

    // 17 pub 字段逐名触达（读/写各至少一字段，其余取引用钉存在性）。
    let _ = &p.scope;
    let _ = &p.db;
    let _ = &p.cur;
    p.skip_check = p.skip_check;
    let _ = &p.compile_dest;
    let _ = &p.errors;
    let _ = &p.warnings;
    let _ = &p.error_limit;
    let _ = &p.infer_ctx;
    let _ = &p.type_registry;
    let _ = &p.type_store;
    let _ = &p.module_tracker;
    let _ = &p.lambda_id_gen;
    let _ = &p.session;
    let _ = &p.script_pragma;
    let _ = &p.rust_pragma;
    let _ = &p.saw_bare_null;

    // 构造/装配族签名 pin。
    let _ = Parser::from(src);
    let _ = Parser::new(src);
    let _ = Parser::new_with_note(src, 'p');
    let _ = Parser::new_with_note_and_first_token(
        src,
        'p',
        p.cur.clone(),
        crate::lexer::Lexer::new(src),
    );
    p.set_dest(p.compile_dest.clone());
    if let Some(reg) = p.type_registry.clone() {
        p.set_type_registry(reg);
    }
    if let Some(db) = p.db.clone() {
        p.set_database(db);
    }
    // 方言注册族（Box<dyn> + DialectPriority 真实签名 pin）。
    fn _pin_register<'p, 'a>(p: &'p mut Parser<'a>, d: Box<dyn crate::dialect::Dialect>) {
        p.register_dialect(d, crate::dialect::DialectPriority::Override);
    }
    fn _pin_consume<'a>(p: Parser<'a>, d: Box<dyn crate::dialect::Dialect>) -> Parser<'a> {
        p.with_dialect(d, crate::dialect::DialectPriority::Override)
    }
    fn _pin_register_ext<'p, 'a>(
        p: &'p mut Parser<'a>,
        d: Box<dyn crate::dialect::ExternalDialect>,
    ) {
        p.register_external_dialect(d, crate::dialect::DialectPriority::Append);
    }
    fn _pin_consume_ext<'a>(
        p: Parser<'a>,
        d: Box<dyn crate::dialect::ExternalDialect>,
    ) -> Parser<'a> {
        p.with_external_dialect(d, crate::dialect::DialectPriority::Append)
    }
    let _ = Parser::from(src).with_session(CompilerSession::core());
}

/// ParserBridge 公开方法面 pin（17 方法存在性 + 双端口实现）。
#[test]
fn plan814_v02_parser_bridge_surface_pins() {
    use crate::parser_bridge::ParserBridge;

    let src = "let x = 1";
    let mut cs = crate::compile::CompileSession::new();
    let mut p = Parser::new_with_type_store(src, cs.type_store());
    let mut b = ParserBridge::new(&mut p);
    let _: crate::token::TokenKind = b.cur_kind();
    let _: String = b.cur_text();
    let _: crate::token::Pos = b.cur_pos();
    let _ = b.is_kind(crate::token::TokenKind::Ident);
    let _: crate::token::Token = b.advance();
    let _: crate::error::AutoResult<()> = b.expect(crate::token::TokenKind::Ident);
    let _: crate::error::AutoResult<()> = b.expect_keyword("pin");
    let _: Option<crate::token::TokenKind> = b.probe_next_kind();
    let _: usize = b.skip_empty_lines();
    let cp = b.checkpoint();
    b.restore(cp);
    let _: crate::error::AutoResult<crate::ast::Type> = b.parse_type();
    let _: crate::error::AutoResult<crate::ast::Expr> = b.parse_expr();
    let _: crate::session::Scenario = b.scenario();
    let _: bool = b.is_ui_scenario();
    let _: bool = b.in_on_body();
    let _: bool = b.in_fn_body();
    let _: crate::error::AutoError = b.syntax_error("pin");
    use crate::Parser;
}

/// AutoVM pub 字段 + 构造族 + NativeInterface 注册族 pin。
#[test]
fn plan814_v02_vm_public_surface_pins() {
    use crate::vm::engine::AutoVM;
    use crate::vm::native::NativeInterface;

    let (vm, _stdout, _entry, _ty) = crate::create_vm_from_source("fn main() int { return 1 }")
        .expect("default composition compiles + runs");
    let _ = &vm.flash;
    let _ = &vm.native_interface;
    let _ = &vm.strings;
    let _ = &vm.string_dedup;
    let _ = &vm.tasks;
    let _ = &vm.id_gen;

    // NativeInterface 注册族 pin（真实调用形态——泛型 F 三族 + 具名三族）。
    let mut ni = NativeInterface::production();
    ni.register_static(1u16, |_t: &mut crate::vm::task::AutoTask, _v: &AutoVM| {
        Ok(())
    });
    let _dyn_id: u16 =
        ni.register_dynamic(|_t: &mut crate::vm::task::AutoTask, _v: &AutoVM| Ok(()));
    ni.register_dynamic_with_id(
        10002u16,
        |_t: &mut crate::vm::task::AutoTask, _v: &AutoVM| Ok(()),
    );
    // 具名三族：真实名须在 BIGVM_NATIVES——以不被调用的帮助函数体钉签名
    // （编译 pin；运行语义由 native 族各自测试覆盖）。
    fn _pin_by_name_family(ni: &mut NativeInterface) {
        let _: u16 = ni.register_typed_shim_by_name("pin_typed", |_t, _v| Ok(()), &["i32"], "i32");
        let _: u16 =
            ni.register_method_shim_by_name("pin_method", |_t, _v| Ok(()), "Pin", &["i32"], "i32");
        ni.register_name("pin_name", 3u16);
    }
    // AutoVM 构造面（fn 项类型 pin——签名漂移即红）。
    let _: fn(crate::vm::virt_memory::VirtualFlash, usize) -> AutoVM = AutoVM::new;
    let _: fn(
        crate::vm::virt_memory::VirtualFlash,
        usize,
    ) -> (AutoVM, std::sync::Arc<std::sync::RwLock<String>>) = AutoVM::new_with_capture;
    let _ = &vm.native_interface;
}

/// feature 门控根再导出 + UI 动态管线入口签名 + 新旧类型身份同一 pin。
#[cfg(feature = "ui-iced")]
#[test]
fn plan814_v02_root_reexports_and_type_identity_pins() {
    use crate::{
        App, AppResult, Component, Style, VNode, VNodeId, VNodeKind, VNodeProps, VTree, View,
        ViewBuilder,
    };

    // 类型身份同一：旧 auto_lang 路径与 auto_ui_runtime 是同一类型
    // （无双 Parser/VM/View——AC-02"无双状态"的编译期证据）。
    fn takes_runtime_view<V: Clone + std::fmt::Debug>(_v: &auto_ui_runtime::ui::View<V>) {}
    fn takes_runtime_component<C: auto_ui_runtime::ui::Component>(_c: &C) {}
    fn identity_bridge(v: &View<u8>, c: &impl Component) {
        takes_runtime_view(v);
        takes_runtime_component(c);
    }

    // 全部门控根名经上方 use 编译期钉存在性；类型身份同一经 identity_bridge
    // 编译期钉"无双状态"（旧路径与新包是同一类型）。

    // UI 动态管线入口签名 pin（实装在 auto_ui_runtime::dynamic_build，
    // 旧 crate 根路径经再导出保持）。
    let _: fn(&str, Option<&str>) -> crate::error::AutoResult<String> = crate::run_file_dynamic_ui;
    let _: fn(
        &str,
        Option<&str>,
    ) -> crate::error::AutoResult<auto_ui_runtime::ui::dynamic::DynamicComponent> =
        crate::build_dynamic_component;
    let _: fn(&str) -> bool = crate::has_ui_keywords;
}
