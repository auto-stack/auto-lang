//! PLAN-981（`IF-981-GFX-HOST-PORT`）：AGE 图形的 AV 宿主派发接面。
//!
//! 语言侧（`AGE_GFX_SERVICE_OPS`）把 **13 个标量结果 op**（input/tick/row/
//! render2d）降到普通 import 调用；本模块提供运行期的**注册式宿主**：
//! 调用点按 symbol 分流到 [`AgeGfxHost::dispatch`]，未注册 provider 时具名拒
//! （`age-gfx.host-unavailable`），provider 返回的 `Value` 必须与调用槽的
//! `MirType` 对得上（`age-gfx.result-type`）。
//!
//! **未覆盖（具名，下一片）**：返回端口 record 的 3 个 op（`query2d.segment`/
//! `query2d.sweep`/`image2d.png`）需要多字段宿主返回 ABI（当前 MIR 一条指令
//! 只写一个槽）；它们仍在 lowering 侧以 `lower.age-gfx-port-result` 具名拒。
//!
//! 预算/生命周期（每 Tick 信用、sticky 第 33 次拒、Run 级注册与停止）在
//! `ace_av` 的会话层（`AgeGfxSession`），这里只负责**派发**——费率表与语言侧
//! 同源（`age_resources`），不在此重复。

use std::cell::RefCell;

use crate::contract::Value;

/// 一次宿主派发的具名失败（fault）。`code` 是稳定串（如 `age-gfx.credits`），
/// 供调用方/测试断言；`message` 携带上下文（不参与身份）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgeGfxFault {
    pub code: &'static str,
    pub message: String,
}

impl AgeGfxFault {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// 宿主派发的返回值：标量 op 直接给 `Value`；**端口 op**（查询/图像身份）一次
/// 调用返回**完整字段序列**（按 `AGE_PORT_FIELDS` 声明序），由派发层按调用槽
/// 的 nominal record 类型核对后组装成 `Value::Record` 一次发布（字段读 0 credit，
/// 不拆成 N 次收费调用）。
#[derive(Clone, Debug, PartialEq)]
pub enum AgeGfxReturn {
    Scalar(Value),
    Record(Vec<Value>),
}

/// 图形宿主 provider：按受检 symbol 派发，参数/结果就是 AV 的 `Value`。
///
/// provider 不接触编译器、不重解析源；它只实现算法（JUDGE：查询/行状态/精灵
/// 算法归 UI058 中立单元，见 PLAN-938 host authority）。返回 `Err` 即具名
/// fault：AV 以 `ExecOutcome::Infra("age-gfx.<code>: <message>")` 终止本次运行
/// （不写结果槽 → 对状态零效果）。
pub trait AgeGfxHost {
    fn dispatch(&mut self, symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault>;
}

/// 是否为 AGE 图形服务的**宿主派发** symbol（标量结果那 13 个）。
///
/// 只认 `age.<group>.*.v1` 且 group ∈ query2d/input/tick/row/render2d/image2d；
/// `frame2d.at`（纯值构造）、`material2d/mesh2d/model3d`（资源构造）不在此列
/// ——它们在 lowering 里已经走 Aggregate/资源身份，不会以调用到达这里。
pub fn is_host_symbol(symbol: &str) -> bool {
    let Some(rest) = symbol.strip_prefix("age.") else {
        return false;
    };
    let Some((group, _)) = rest.split_once('.') else {
        return false;
    };
    matches!(
        group,
        "query2d" | "input" | "tick" | "row" | "render2d" | "image2d"
    )
}

/// 宿主返回值与调用槽类型的**双向**核对（typed 派发的运行时半边；语言侧
/// 签名已在 HIR/描述符核过，这里防 provider 撒谎）。
pub fn value_matches_type(v: &Value, want: Option<&crate::mir::MirType>) -> bool {
    match (want, v) {
        (Some(crate::mir::MirType::I32), Value::I32(_)) => true,
        (Some(crate::mir::MirType::Bool), Value::Bool(_)) => true,
        (Some(crate::mir::MirType::F32), Value::F32(_)) => true,
        (Some(crate::mir::MirType::F64), Value::F64(_)) => true,
        (Some(crate::mir::MirType::Str), Value::Str(_)) => true,
        (Some(crate::mir::MirType::StrView), Value::Str(_)) => true,
        _ => false,
    }
}

thread_local! {
    /// 本线程当前 run 注册的图形宿主。执行是同步的（单线程 AV），注册/卸载
    /// 由调用方（`ace_av::age_gfx::AgeGfxSession`）在同一线程成对完成。
    static REGISTERED: RefCell<Option<Box<dyn AgeGfxHost>>> = const { RefCell::new(None) };
}

/// 注册本线程的图形宿主；返回被替换的旧宿主（正常应为 `None`）。
pub fn install(host: Box<dyn AgeGfxHost>) -> Option<Box<dyn AgeGfxHost>> {
    REGISTERED.with(|r| r.borrow_mut().replace(host))
}

/// 卸载本线程的图形宿主（run 结束/会话停止）。
pub fn uninstall() -> Option<Box<dyn AgeGfxHost>> {
    REGISTERED.with(|r| r.borrow_mut().take())
}

/// 派发给本线程注册的宿主：`None` = 未注册（调用点据此具名拒）。
pub(crate) fn dispatch_current(
    symbol: &str,
    args: &[Value],
) -> Option<Result<AgeGfxReturn, AgeGfxFault>> {
    REGISTERED.with(|r| {
        r.borrow_mut()
            .as_mut()
            .map(|host| host.dispatch(symbol, args))
    })
}

/// AGE 图形能力键（`age2d.query/input/scene/sprite`、`age3d.*`）：模块的
/// `requires` 里带这些键时，admission 承认它们（真正授权仍由调用点/provider
/// 决定）。
pub fn is_gfx_capability(c: &str) -> bool {
    c.starts_with("age2d.") || c.starts_with("age3d.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_symbols_are_the_graphics_service_families_only() {
        assert!(is_host_symbol("age.query2d.segment.v1"));
        assert!(is_host_symbol("age.input.space.v1"));
        assert!(is_host_symbol("age.tick.dt.v1"));
        assert!(is_host_symbol("age.row.place.v1"));
        assert!(is_host_symbol("age.render2d.spawn.v1"));
        assert!(is_host_symbol("age.image2d.png.v1"));
        // 值构造/资源身份不是宿主派发（lowering 另有面）。
        assert!(!is_host_symbol("age.frame2d.at.v1"));
        assert!(!is_host_symbol("age.material2d.color.v1"));
        assert!(!is_host_symbol("age.model3d.glb.v1"));
        assert!(!is_host_symbol("auto.console.print.i32.v1"));
        assert!(!is_host_symbol("age"));
    }
}
