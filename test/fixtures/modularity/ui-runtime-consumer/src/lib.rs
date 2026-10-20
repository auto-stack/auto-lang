//! PLAN-814 T-08：UI runtime 外部消费方探针（canonical 双直依赖形态）。
//!
//! 仅 `auto_ui_runtime::` + `auto_lang_core::` import（旧 auto-lang 包
//! 缺席）——V-02/V-03 消费侧证明：装配单点真实生效、根再导出类型与 SDK
//! 同一、runtime 自有资产经动态管线真实装载。

use auto_ui_runtime::ui::{auto_val, Component, View};
use auto_ui_runtime::{install_ui_host_capabilities, AutoResult, Parser};

/// V-03：canonical Component/View 类型可用（compile + run 形态）。
#[derive(Debug)]
pub struct ProbeCounter {
    pub count: i64,
}

impl Component for ProbeCounter {
    type Msg = i64;

    fn on(&mut self, msg: Self::Msg) {
        self.count += msg;
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().child(View::text(format!("count:{}", self.count))).build()
    }
}

/// V-02：runtime 根再导出 Parser 与 SDK 直接消费同一类型（签名互通）。
pub fn parser_identity_pin(code: &str) -> AutoResult<auto_lang_core::ast::Code> {
    // runtime::Parser 再导出与 auto_lang_core::Parser 是同一类型——本函数
    // 签名（返回 SDK 类型）接受 runtime 构造结果即编译期身份证明。
    let session = auto_lang_core::session::CompilerSession::core();
    let mut p: Parser = Parser::from(code).with_session(session);
    p.parse()
}

/// V-03：装配单点真实生效——安装后 VM 管线可用（frame/native 缺席语义
/// 由 A 期探针族覆盖；此处钉"安装后 run 管线不 panic + stdout 可达"）。
pub fn assembled_vm_stdout(code: &str) -> String {
    install_ui_host_capabilities();
    let (_r, out) = auto_lang_core::run_with_capture(code).expect("vm pipeline runs");
    out
}

/// V-03：runtime 自有资产（assets/desktop.at）经 build_dynamic_component
/// 真实装载——canonical 生成轨入口在新包闭包内自足。
pub fn desktop_asset_loads() -> bool {
    let src = auto_ui_runtime::ui::shell::DESKTOP_AT;
    auto_ui_runtime::build_dynamic_component(src, None).is_ok()
}

/// auto_val 再导出同源（V-02：ui::auto_val 路径保持）。
pub fn auto_val_reexport_pin(v: auto_val::Value) -> auto_val::Value {
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan814_v03_canonical_component_builds() {
        let c = ProbeCounter { count: 7 };
        // view 构建不 panic 即 Component/View canonical 面工作。
        let _ = c.view();
    }

    #[test]
    fn plan814_v02_parser_identity() {
        let code = parser_identity_pin("fn main() int { return 1 }")
            .expect("core parse via runtime re-export");
        assert!(!code.stmts.is_empty());
    }

    #[test]
    fn plan814_v03_assembly_and_vm_stdout() {
        let out = assembled_vm_stdout("fn main() { print(\"runtime-consumer-ok\") }");
        assert!(out.contains("runtime-consumer-ok"), "stdout: {out}");
    }

    #[test]
    fn plan814_v03_desktop_asset_dynamic_build() {
        assert!(
            desktop_asset_loads(),
            "desktop.at 经 runtime 动态管线装载失败（canonical 资产自足性破坏）"
        );
    }

    #[test]
    fn plan814_v02_auto_val_reexport() {
        let v = auto_val_reexport_pin(auto_val::Value::Int(3));
        assert_eq!(v, auto_val::Value::Int(3));
    }
}
