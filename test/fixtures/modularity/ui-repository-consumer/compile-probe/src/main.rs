//! PLAN-831 T-09/F831-09-02 编译探针——旧公开 API 实编 + 类型单源身份。
//!
//! 覆盖 compat-touchpoints.toml 冻结的 16 条 `auto_lang::` 路径（TYP-C01..C16）：
//! - 轻入口**真调用**（parse 族/回调安装/has_ui_keywords/类型转换）；
//! - 重入口（VM 捕获/执行、trans）以 fn item 引用钉住符号与可调用形态；
//! - pub 字段（`Parser::{scope,db,cur}`）以字段读取编译证明；
//! - `cargo run` 执行运行时断言：新旧路径函数指针同一（单源身份）、
//!   SDK 轻数据类型流入 UI 消费（跨包类型传递）。
//!
//! feature 组合（见 Cargo.toml）：default=full；`--no-default-features
//! --features lang-core`=核心面；`... lang-core,ui-face`=显式 UI 组合。
//! 三组合的编译/运行为 F831-09-02 的闭合证据。

fn noop_install_hook() {}

/// C01/C02/C03/C04/C05/C06/C15/C16——核心面（三种组合全部编译）。
fn core_face() -> usize {
    let mut proofs = 0usize;

    // C02：SDK 解析入口族——真实调用
    let code = auto_lang::parse("let a = 1").expect("auto_lang::parse");
    let _ = code;
    let preserved = auto_lang::parse_preserve_error("let a = 1").expect("parse_preserve_error");
    let _ = preserved;
    proofs += 2;

    // C06：公开回调 seam——真实安装 no-op 回调（探针自有进程内全局钩子）
    auto_lang::set_ui_host_install_hook(noop_install_hook);
    proofs += 1;

    // C03/C04/C05：VM 捕获/执行与 trans 出口——fn item 引用（重入口不执行）
    let _capture = auto_lang::run_with_capture_and_bytecode;
    let _exec = auto_lang::execute_autovm_with_deadline_and_input;
    let _trans = auto_lang::trans_rust_with_session;
    let _ = (&_capture, &_exec, &_trans);
    proofs += 3;

    // C01：Parser 类型 + pub 字段读取（`scope`/`db`/`cur`——类型检查即证据）
    fn touch_parser_pub_fields(p: &auto_lang::Parser) {
        let _scope = &p.scope;
        let _db = &p.db;
        let _cur = &p.cur;
    }
    let _ = touch_parser_pub_fields;
    proofs += 1;

    // C15/C16：auto-lang crate 本地 pub 类型（非 SDK 转发）
    fn _c15(_: Option<&auto_lang::InferenceContext>) {}
    fn _c16(_: Option<&auto_lang::SharedTypeRegistry>) {}
    let _ = (&_c15, &_c16);
    proofs += 2;

    proofs
}

/// C08..C13 + 跨包类型传递/单源身份——UI 面（default 与显式 UI 组合编译）。
#[cfg(feature = "ui-face")]
fn ui_face() -> usize {
    let mut proofs = 0usize;

    // C11：UIC 指纹（真调用）
    let _fingerprint = auto_lang::generator_fingerprint();
    proofs += 1;

    // C08/C09/C10/C12/C13：模块面 pub 项类型引用（编译即证据）
    fn _c08(_: Option<auto_lang::ui_gen::shared::SharedGen>) {}
    fn _c09(_: Option<auto_lang::aura::AuraWidget>) {}
    fn _c10(_: Option<auto_lang::a2ui::A2UIError>) {}
    fn _c12(_: Option<auto_lang::back_proxy::BackProxyConfig>) {}
    fn _c13(_: Option<auto_lang::host_adapters::dialog::ParentedOsDialogProvider>) {}
    let _ = (&_c08, &_c09, &_c10, &_c12, &_c13);
    proofs += 5;

    // 跨 SDK/UI 轻数据类型传递：auto_val::Node（SDK 单源轻数据）流入
    // UI runtime 的 node_converter——同一 nominal 类型跨包可用。
    let node = auto_val::Node::new("text").with_arg("hello");
    let view = auto_ui_runtime::ui::node_converter::convert_node(&node);
    assert!(view.is_ok(), "convert_node 应接受 SDK auto_val::Node");
    proofs += 1;

    // nominal 类型身份：runtime 再导出的 Parser 与 SDK 原始 Parser 同一类型
    // （编译期即约束——不同源的同名类型在此无法通过）。
    fn same_type<T>(_: &T, _: &T) {}
    fn parser_identity(p: &auto_lang_core::Parser) {
        let via_runtime: &auto_ui_runtime::Parser = p;
        same_type(p, via_runtime);
    }
    let _ = parser_identity;
    proofs += 1;

    proofs
}

/// C07/C14——旧路径面的 iced/interpreter 门子集：`auto_lang::has_ui_keywords`
/// 的转发在 auto-lang `ui-iced` 门、`auto_lang::ui::i18n_lookup` 在 runtime
/// `ui-interpreter` 门（随 ui-iced 激活）；`ui` 门组合下这两项按 feature
/// 条件性缺席（门名与迁移前同名保持——探针分面如实记录该矩阵）。
#[cfg(feature = "iced-face")]
fn iced_face() -> usize {
    let mut proofs = 0usize;

    // C14：动态构建管线族——has_ui_keywords 真调用
    assert!(
        auto_lang::has_ui_keywords("widget W { view { col { text \"hi\" } } }"),
        "has_ui_keywords 应识别 widget 源"
    );
    proofs += 1;

    // C07：ui 再导出面——新旧路径同一函数体（i18n_lookup 单源=compiler，
    // 指针级同一性是运行时可观察的单源 oracle）
    let old_path_lookup: fn(&str) -> Option<String> = auto_lang::ui::i18n_lookup::lookup;
    let direct_lookup: fn(&str) -> Option<String> = auto_ui_compiler::i18n_lookup::lookup;
    assert!(
        std::ptr::eq(old_path_lookup as *const (), direct_lookup as *const ()),
        "auto_lang::ui::i18n_lookup::lookup 与 auto_ui_compiler 直依赖必须是同一函数体"
    );
    proofs += 1;

    proofs
}

fn main() {
    let core = core_face();
    #[cfg(feature = "ui-face")]
    let ui = ui_face();
    #[cfg(not(feature = "ui-face"))]
    let ui = 0;
    #[cfg(feature = "iced-face")]
    let iced = iced_face();
    #[cfg(not(feature = "iced-face"))]
    let iced = 0;
    println!(
        "plan831-compile-probe: core face {} 项 + ui face {} 项 + iced face {} 项实编通过",
        core, ui, iced
    );
}
