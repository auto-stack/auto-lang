// Plan 020 T-02 —— 协议客户端入口抽壳：`--autodesk-*` 客户端形态中不依赖
// 组件构造方式的部分（端点解析 + 帧二态分派），供两轨共用：
// - 解释轨：`auto run` 的 cmd_autodesk.rs 薄壳化（.at 装载 + 三态裁决留在
//   壳内，行为零变化——Plan 480/500 既有流程逐步同型搬迁）；
// - native 轨：a2r 生成 exe 的客户端臂（Plan 020 T-03/T-04/T-05，Component
//   泛型入口在本模块追加，`DynamicComponent` 专属臂不动）。
//
// 端点语义（v1.1/v1.3 既有口径）：① spawn 注入直连（`--autodesk-client=
// <pipe>`，单 client 测试机件，模式位随 Hello 协商缺省 Commands）② broker
// 孵化（`request_incubation_render` 记录第三字段携带二态模式 + auto 降级
// 标记）。预算 5000ms 同 cmd_autodesk 既有值。

use crate::ui::component::Component;
use crate::ui::desktop_protocol::broker::{self, RequestedRender};
use crate::ui::desktop_protocol::client_runtime::{self, ClientConfig, ReconnectPolicy};
use crate::ui::desktop_protocol::coverage::{Coverage, RenderMode, Verdict};
use crate::ui::desktop_protocol::message::FrameMode;
use crate::ui::desktop_protocol::native_projector::NativeProjector;
use crate::ui::desktop_protocol::pixels;
use crate::ui::desktop_protocol::transport;
use crate::ui::dynamic::DynamicComponent;

/// 客户端启动参数（组件由各轨自行装载后送入——本入口不关心来源）。
pub struct ClientOpts {
    pub app_name: String,
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub frame_mode: FrameMode,
    /// auto 裁决降级标记（broker 孵化记录携带 `pixels:auto`，宿主观测留痕）。
    pub auto_downgraded: bool,
}

/// 端点目标：① 直连 per-app 管道 ② broker 孵化（`--autodesk-broker` 可改）
/// ③ rqhost 采纳（PLAN-031——rendezvous 内建 + exit-on-EOF 策略档）。
pub enum ClientTarget {
    Direct(String),
    Broker { broker_pipe: String },
    /// PLAN-031 D5：`auto run -q` 形态。connect = rendezvous 采纳
    /// （连 well-known → `adopt␟<app_name>` → 转连 per-app 管道——
    /// 采纳与使用同点，不预连不烧一次性管道实例）；差异在**宿主亡
    /// 策略**——reconnect=None（`ClientExit::HostLost` 即退 + 观测行），
    /// 区别于桌面档 30s 重连（原生 app 心智：宿主没了就干净退出）。
    Rqhost { wellknown: String, app_name: String },
}

/// 宿主亡策略选择（PLAN-031 T-05）：Rqhost 档 = exit-on-EOF（None）；
/// 桌面档（Direct/Broker）= 既有 30s/50ms 重连不变（I2）。
pub(crate) fn reconnect_for(target: &ClientTarget, per_app_pipe: String) -> Option<ReconnectPolicy> {
    match target {
        ClientTarget::Rqhost { .. } => None,
        ClientTarget::Direct(_) | ClientTarget::Broker { .. } => Some(ReconnectPolicy {
            pipe: per_app_pipe,
            budget_ms: 30_000,
            interval_ms: 50,
        }),
    }
}

/// 端点解析：直连 / broker 孵化 / rqhost 采纳。返回
/// `(per_app_pipe, app_end)`——per_app_pipe 供 Commands 臂 ReconnectPolicy
/// 重连同管道。
pub fn connect(
    target: &ClientTarget,
    app_name: &str,
    render: RequestedRender,
) -> Result<(String, Box<dyn transport::Transport + Send>), String> {
    match target {
        ClientTarget::Direct(p) => {
            let end = transport::connect(p, 5000).map_err(|e| format!("连 {p}: {e:?}"))?;
            Ok((p.clone(), end))
        }
        ClientTarget::Rqhost { wellknown, app_name } => {
            super::rqhost::adopt(wellknown, app_name, 5000)
                .map_err(|e| format!("rqhost 采纳失败: {e:?}"))
        }
        ClientTarget::Broker { broker_pipe } => {
            broker::request_incubation_render(broker_pipe, app_name, render, 5000)
                .map_err(|e| format!("broker 孵化失败: {e:?}"))
        }
    }
}

/// 解释轨客户端（`DynamicComponent`）：
/// - `Commands` → [`NativeProjector`] 投影（PLAN-033 T-02 改接：View 全
///   展开渲染 + 启动覆盖门，与 native 轨同臂——`-q` 对两轨一视同仁，
///   解释组件免每 app iced/wgpu 后端）+ 泛型泵命令帧；
/// - `Pixels` → [`pixels::run_independent_child`] 隐藏 iced 窗自渲 + screenshot。
pub fn run_dynamic_client(
    component: DynamicComponent,
    opts: ClientOpts,
    target: ClientTarget,
) -> Result<(), String> {
    let render =
        RequestedRender { mode: opts.frame_mode, auto_downgraded: opts.auto_downgraded };
    let reconnect_pipe_target = matches!(target, ClientTarget::Rqhost { .. });
    let (per_app_pipe, app_end) = connect(&target, &opts.app_name, render)?;
    match opts.frame_mode {
        FrameMode::Pixels => pixels::run_independent_child(
            app_end,
            component,
            &opts.app_name,
            &opts.title,
            opts.width,
            opts.height,
        )
        .map(|_| ()),
        FrameMode::Commands => {
            let mut projector = NativeProjector::new(component, opts.width, opts.height);
            if let Err(gate) = projector.ensure_covered() {
                eprintln!("[render] {gate}");
                return Err(gate);
            }
            let config = ClientConfig {
                app_name: opts.app_name.clone(),
                title: opts.title,
                width: opts.width,
                height: opts.height,
            };
            let reconnect = reconnect_for(&target, per_app_pipe);
            let (exit, projector) =
                client_runtime::run_client_session(app_end, projector, config, reconnect);
            if reconnect_pipe_target && matches!(exit, client_runtime::ClientExit::HostLost) {
                eprintln!("[rqhost-client] host lost → exit（exit-on-EOF 策略档）");
            }
            println!("[autodesk-client] exit={exit:?} revision={}", projector.revision());
            Ok(())
        }
    }
}

// ================================ 测试 ================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::endpoint::FrameSource;

    fn example_source(dir: &str) -> Option<String> {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/ui/");
        let path = format!("{base}{dir}/src/front/app.at");
        std::fs::read_to_string(&path).ok()
    }

    /// PLAN-033 T-02 冒烟：VM 源（`DynamicComponent`）经 Commands 臂新装配
    /// （NativeProjector + ensure_covered + 产帧）——001/003 两载体 Covered
    /// 且帧非空（AC-01 单元级证据；全链 e2e 在 T-07 p033_rq_unify_arm）。
    #[test]
    fn vm_queue_arm_assembly_covered() {
        for dir in ["001-helloworld", "003-converter"] {
            let Some(src) = example_source(dir) else {
                eprintln!("[p033] skip: {dir} 载体缺席");
                return;
            };
            let component =
                crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{dir}: {e}"));
            let mut projector = NativeProjector::new(component, 480.0, 320.0);
            projector
                .ensure_covered()
                .unwrap_or_else(|gate| panic!("{dir} 未过覆盖门: {gate}"));
            let frame = projector.render_frame();
            assert!(!frame.ops.is_empty(), "{dir} 空帧");
            assert!(projector.revision() >= 1);
        }
    }
}

/// native 轨三态 → 二态分派（Plan 020 T-04）。**PLAN-032 T-06 翻转**
///（ramp v3 数据门达标——2026-09-19 复测 judged 22/22 = 100% ≥ 95%，
/// 六缺项全清偿：012 SelfCenter / 018·041 hidden / 018·021 定位族
///（absolute 真渲 + fixed/sticky 降级放行）/ 024 样式 grid / 046
/// tabs kind；报告 `docs/plans/reports/p032-native-flip-row.md`）：
/// native `Auto` 缺省 = **queue**（Covered → Commands；翻转前 025-032
/// 期间缺省 independent——026/029 两复测未达标的在案裁定由 p032 收
/// 束）。未覆盖降级路径不变（Pixels + 观测行携带真扫描缺项清单 +
/// auto 降级标记）。防漏钉 = `native_flip_coverage_data_row` 断言反转
///（judged < 95% 即红——跌破门需显式裁定，禁静默回归）。显式
/// `Queue` 不在此裁决（覆盖门在 [`run_native_client`] 消费
/// [`NativeProjector::ensure_covered`]——拒绝退出留痕）；`Independent`
/// 直通。
/// 返回 `(帧模式, auto 降级标记, Option<观测行>)`。
pub fn resolve_native_frame_mode<M: Clone + std::fmt::Debug>(
    mode: RenderMode,
    widget_name: &str,
    view: &crate::ui::view::View<M>,
) -> (FrameMode, bool, Option<String>) {
    match mode {
        RenderMode::Queue => (FrameMode::Commands, false, None),
        RenderMode::Independent => (FrameMode::Pixels, false, None),
        RenderMode::Auto => {
            let scan = crate::ui::desktop_protocol::coverage::scan_native_view(view);
            match crate::ui::desktop_protocol::coverage::judge(&scan, &Coverage::native_queue_set()) {
                Verdict::Covered => (
                    // PLAN-032 T-06 翻转执行（ramp v3 数据门达标——judged
                    // 22/22 = 100%；翻转前本臂返 Pixels：026/029 两复测
                    // 未达标维持不翻的在案裁定由 p032 报告收束）。
                    FrameMode::Commands,
                    true,
                    Some(format!(
                        "[render] native auto -> queue ({widget_name}; \
                         queue-covered, default flipped@ramp3: judged 22/22 = 100%)"
                    )),
                ),
                Verdict::NotCovered(missing) => (
                    FrameMode::Pixels,
                    true,
                    Some(format!(
                        "[render] native auto -> independent downgrade ({widget_name}; \
                         missing: {})",
                        missing.join(", ")
                    )),
                ),
            }
        }
    }
}

/// native 轨客户端（a2r 编译 `Component`，Plan 020 T-04）：
/// - `Commands` → [`NativeProjector`] 投影 + 泛型泵命令帧（启动覆盖门：
///   not-yet = 拒绝退出留痕，AC-04）；
/// - `Pixels` → [`pixels::run_independent_native_child`] 隐藏窗自渲
///   （T-03 入口）。
///
/// T-05 生成 main 消费：`--autodesk-incubate` 分派至此。
pub fn run_native_client<C>(
    component: C,
    opts: ClientOpts,
    target: ClientTarget,
) -> Result<(), String>
where
    C: Component + 'static,
    C::Msg: Clone + std::fmt::Debug + Send + 'static,
{
    let render =
        RequestedRender { mode: opts.frame_mode, auto_downgraded: opts.auto_downgraded };
    let rqhost_target = matches!(target, ClientTarget::Rqhost { .. });
    let (per_app_pipe, app_end) = connect(&target, &opts.app_name, render)?;
    match opts.frame_mode {
        FrameMode::Pixels => pixels::run_independent_native_child(
            app_end,
            component,
            &opts.app_name,
            &opts.title,
            opts.width,
            opts.height,
        ),
        FrameMode::Commands => {
            let projector = NativeProjector::new(component, opts.width, opts.height);
            if let Err(gate) = projector.ensure_covered() {
                eprintln!("[render] {gate}");
                return Err(gate);
            }
            let config = ClientConfig {
                app_name: opts.app_name.clone(),
                title: opts.title,
                width: opts.width,
                height: opts.height,
            };
            let reconnect = reconnect_for(&target, per_app_pipe);
            let (exit, projector) = client_runtime::run_client_session(
                app_end,
                projector,
                config,
                reconnect,
            );
            if rqhost_target && matches!(exit, client_runtime::ClientExit::HostLost) {
                eprintln!("[rqhost-client] host lost → exit（exit-on-EOF 策略档）");
            }
            println!("[autodesk-client] exit={exit:?} revision={}", projector.revision());
            Ok(())
        }
    }
}
