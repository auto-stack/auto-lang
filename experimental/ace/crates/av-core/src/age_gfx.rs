//! PLAN-981（`IF-981-GFX-HOST-PORT`）：AGE 图形的 AV 宿主派发接面。
//!
//! 语言侧（`AGE_GFX_SERVICE_OPS`）把 **13 个标量结果 op**（input/tick/row/
//! render2d）降到普通 import 调用；本模块提供运行期的**注册式宿主**：
//! 调用点按 symbol 分流到 [`AgeGfxHost::dispatch`]，未注册 provider 时具名拒
//! （`age-gfx.host-unavailable`），provider 返回的 `Value` 必须与调用槽的
//! `MirType` 对得上（`age-gfx.result-type`）。
//!
//! 端口 record 的 3 个 op（`query2d.segment/sweep`、`image2d.png`）已随端口
//! ABI 交付：provider 一次调用返回**完整字段序列**（[`AgeGfxReturn::Record`]，
//! `AGE_PORT_FIELDS` 声明序），派发层按调用槽 nominal record 逐字段核对后
//! 组装 `Value::Record` 一次发布（字段读 0 credit）。
//!
//! PLAN-981（宿主动态 token/世代）：[`AgeGfxHost::dispatch_with_ctx`] 携带
//! 会话事实（tick/callback/query_epoch/credits）——宿主侧 token/句柄世代的
//! 记账原料；默认转发 `dispatch`，既有 provider 不破。
//!
//! 预算/生命周期（每 Tick 信用、sticky 超额拒、Run 级注册与停止）在
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

    /// PLAN-981（宿主动态 token/世代，2026-11-18 用户裁定）：带**会话事实**的
    /// 派发。默认转发 [`AgeGfxHost::dispatch`]——既有 provider（UI058
    /// `NeutralGfxHost` 等）不破；宿主车道按需消费：
    /// - `query_epoch`：本 callback（一次 `run_def`）内第几次 `query2d.*` 派发
    ///   （1 基、单调）。**第二次查询 ⇒ epoch 前进 ⇒ 先前 token 失效**——语言侧
    ///   源序走查（`source.age.query-token-stale`）的**宿主侧动态事实**：宿主可
    ///   用它给自家 token 记账打真实 fault-13 载荷（021 语义；判罚点在宿主
    ///   算法车道，本层只交事实、不重复判）。
    /// - `tick`/`callback`/`credits_used`：世代与账本事实（宿主实现 spawn/
    ///   despawn 代次 fault-33、每 Tick 预算对账用同一份事实，不建第二账）。
    fn dispatch_with_ctx(
        &mut self,
        symbol: &str,
        args: &[Value],
        ctx: &AgeGfxCall<'_>,
    ) -> Result<AgeGfxReturn, AgeGfxFault> {
        let _ = ctx;
        self.dispatch(symbol, args)
    }
}

/// 一次宿主派发携带的会话事实（PLAN-981 宿主动态 token/世代）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgeGfxCall<'a> {
    /// 会话 Tick 序号（`begin_tick` 单调递增；未 begin 即 0）。
    pub tick: u64,
    /// callback 序号（每 `run_def` 一次，会话内单调；世代边界）。
    pub callback: u64,
    /// 本 callback 内第几次 `query2d.*`（1 基、单调；非查询 op 原样带当前值）。
    pub query_epoch: u32,
    /// 本 Tick 已累计信用（本次派发**记账后**的值，与 `credits_used()` 同口径）。
    pub credits_used: u32,
    /// 本次派发的 symbol（便于宿主单入口记账）。
    pub symbol: &'a str,
}

/// 是否为 AGE 图形服务的**宿主派发** symbol（标量结果那 13 个）。
///
/// 只认 `age.<group>.*.v1` 且 group ∈ query2d/input/tick/row/render2d/image2d；
/// `frame2d.at`（纯值构造）、`material2d/mesh2d/model3d`（资源构造）不在此列
/// ——它们在 lowering 里已经走 Aggregate/资源身份，不会以调用到达这里。
///
/// av-core 是独立 VM，不依赖编译器 crate——本谓词与语言侧受检表
/// （`ace_core::source::age_resources::is_gfx_host_symbol`，HostAlgorithm/
/// HostIdentity 16 op）镜像同一集合；两边任何一表变更须同步（a2x 车道按
/// 受检表收符号面）。
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
