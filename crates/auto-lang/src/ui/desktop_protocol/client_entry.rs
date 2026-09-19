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
use crate::ui::desktop_protocol::client_runtime::{
    self, AppProjector, ClientConfig, ReconnectPolicy,
};
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

/// 端点目标：① 直连 per-app 管道 ② broker 孵化（`--autodesk-broker` 可改）。
pub enum ClientTarget {
    Direct(String),
    Broker { broker_pipe: String },
}

/// 端点解析：直连 / broker 孵化。返回 `(per_app_pipe, app_end)`——
/// per_app_pipe 供 Commands 臂 ReconnectPolicy 重连同管道。
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
        ClientTarget::Broker { broker_pipe } => {
            broker::request_incubation_render(broker_pipe, app_name, render, 5000)
                .map_err(|e| format!("broker 孵化失败: {e:?}"))
        }
    }
}

/// 解释轨客户端（`DynamicComponent`）：
/// - `Commands` → [`AppProjector`] 投影 + [`client_runtime::run_client`] 命令帧；
/// - `Pixels` → [`pixels::run_independent_child`] 隐藏 iced 窗自渲 + screenshot。
pub fn run_dynamic_client(
    component: DynamicComponent,
    opts: ClientOpts,
    target: ClientTarget,
) -> Result<(), String> {
    let render =
        RequestedRender { mode: opts.frame_mode, auto_downgraded: opts.auto_downgraded };
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
            let config = ClientConfig {
                app_name: opts.app_name.clone(),
                title: opts.title,
                width: opts.width,
                height: opts.height,
            };
            let reconnect =
                ReconnectPolicy { pipe: per_app_pipe, budget_ms: 30_000, interval_ms: 50 };
            let projector = AppProjector::new(component, config.width, config.height);
            let (exit, projector) =
                client_runtime::run_client(app_end, projector, config, Some(reconnect));
            println!("[autodesk-client] exit={exit:?} revision={}", projector.revision());
            Ok(())
        }
    }
}

/// native 轨三态 → 二态分派（Plan 020 T-04；PLAN-026 T-06 复测后裁定
/// **维持不翻**；PLAN-029 T-08 复测同裁定）：native `Auto` 缺省 =
/// **independent**（025 语义不变），升级点 = 观测行携带**真扫描缺项
/// 清单**（原 v1 恒定文案 → 逐 App 缺项载荷）+ queue-covered 命名
///（queue 化可行性逐 App 可见）。**翻转点已备**：三闸数据门 =
/// examples 全量 Covered ≥95%（029 复测行 overall 44.4% / judged
/// 72.7%——样本 36 扩容稀释 + 046-tabs 入分母；009 opacity 放行翻绿、
/// 041 popover 半句清偿；报告 `docs/plans/reports/
/// p029-native-flip-retest-row.md`）；达标时 Covered 臂改返值即为翻转
///（one-line，随 ramp v3 复评）。显式 `Queue` 不在此裁决（覆盖门在
/// [`run_native_client`] 消费 [`NativeProjector::ensure_covered`]——
/// 拒绝退出留痕）；`Independent` 直通。
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
                    // 翻转点：数据门达标时本臂改返 Commands——026
                    // 数据未达标（报告 p026-native-flip-data-row.md），
                    // 维持 Auto→independent 缺省。
                    FrameMode::Pixels,
                    true,
                    Some(format!(
                        "[render] native auto -> independent ({widget_name}; \
                         queue-covered, default flip pending ramp v3 data gate)"
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
            let reconnect =
                ReconnectPolicy { pipe: per_app_pipe, budget_ms: 30_000, interval_ms: 50 };
            let (exit, projector) = client_runtime::run_client_session(
                app_end,
                projector,
                config,
                Some(reconnect),
            );
            println!("[autodesk-client] exit={exit:?} revision={}", projector.revision());
            Ok(())
        }
    }
}
