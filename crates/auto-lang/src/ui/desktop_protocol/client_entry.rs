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
use crate::ui::desktop_protocol::endpoint::FrameSource;
use crate::ui::desktop_protocol::message::FrameMode;
use crate::ui::desktop_protocol::native_projector::RqProjector;
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
    /// PLAN-683（remote 模式）：headless iced 宿主 + DisplayList v2 产线
    /// 帧（Commands 家族内载荷 tag 分派）。
    pub remote: bool,
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
    /// PLAN-693：虚拟桌面合成器端点——连接语义与 [`ClientTarget::Rqhost`]
    /// 完全同构（adopt rendezvous 协议零变化），差异仅端点来源（CLI
    /// `--desktop-endpoint` 参数指定，非 well-known 常量）与**不孵化
    /// 语义**（桌面进程先于 app 在，端点缺席 = 报错提示先启动桌面，
    /// 不代孵——孵化分支结构性不存在，[`super::rqhost::adopt`] 本就只
    /// 连不孵）；宿主亡策略同 rqhost（exit-on-EOF——桌面宿主没了，app
    /// 跟随退出）。
    Desktop { endpoint: String, app_name: String },
}

/// 宿主亡策略选择（PLAN-031 T-05）：Rqhost/桌面端点档 = exit-on-EOF
///（None）；桌面档（Direct/Broker）= 既有 30s/50ms 重连不变（I2）。
pub(crate) fn reconnect_for(target: &ClientTarget, per_app_pipe: String) -> Option<ReconnectPolicy> {
    match target {
        ClientTarget::Rqhost { .. } | ClientTarget::Desktop { .. } => None,
        ClientTarget::Direct(_) | ClientTarget::Broker { .. } => Some(ReconnectPolicy {
            pipe: per_app_pipe,
            budget_ms: 30_000,
            interval_ms: 50,
        }),
    }
}

/// 端点解析：直连 / broker 孵化 / rqhost 采纳 / 桌面端点采纳。返回
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
        ClientTarget::Desktop { endpoint, app_name } => {
            // 不孵化语义（PLAN-693）：adopt 本就只连不孵——端点缺席在此
            // 干净报错，提示先启动虚拟桌面（rq 模式的 ensure 孵化分支
            // 不进入）。
            super::rqhost::adopt(endpoint, app_name, 5000).map_err(|e| {
                format!("desktop endpoint {endpoint} 不可达——请先启动虚拟桌面（{e:?}）")
            })
        }
        ClientTarget::Broker { broker_pipe } => {
            broker::request_incubation_render(broker_pipe, app_name, render, 5000)
                .map_err(|e| format!("broker 孵化失败: {e:?}"))
        }
    }
}

/// 解释轨客户端（`DynamicComponent`）：
/// - `Commands` → [`RqProjector`] 投影（PLAN-033 T-02 改接：View 全
///   展开渲染 + 启动覆盖门，与 native 轨同臂——`-q` 对两轨一视同仁，
///   解释组件免每 app iced/wgpu 后端）+ 泛型泵命令帧；
/// - `Pixels` → **已退役**（PLAN-033 T-04：解释态两合法形态 = inproc
///   直挂 / `-q` 经 native 臂——拒绝退出留痕；a2r 轨像素兜底不受影响）。
pub fn run_dynamic_client(
    component: DynamicComponent,
    opts: ClientOpts,
    target: ClientTarget,
) -> Result<(), String> {
    let render =
        RequestedRender { mode: opts.frame_mode, auto_downgraded: opts.auto_downgraded };
    // exit-on-EOF 策略档观测（rqhost/桌面端点两 adopt 形同款）。
    let reconnect_pipe_target =
        matches!(target, ClientTarget::Rqhost { .. } | ClientTarget::Desktop { .. });
    match opts.frame_mode {
        FrameMode::Pixels => Err(
            "[render] 解释轨 pixels 臂已退役（PLAN-033）——VM 轨两合法形态 = inproc 直挂 / -q 经 native 臂".to_string(),
        ),
        FrameMode::Commands if opts.remote => {
            // PLAN-683（remote 模式）：headless iced 宿主——组件树照常
            // 渲染（tiny_skia 记录层截获），DisplayList v2 产线帧；覆盖
            // 门不适用（组件覆盖 = iced 全集，结构保证）。
            let (per_app_pipe, app_end) = connect(&target, &opts.app_name, render)?;
            let source = super::headless::HeadlessFrameSource::new(
                component,
                opts.width,
                opts.height,
            );
            let config = ClientConfig {
                app_name: opts.app_name.clone(),
                title: opts.title,
                width: opts.width,
                height: opts.height,
            };
            let reconnect = reconnect_for(&target, per_app_pipe);
            let (exit, source) =
                client_runtime::run_client_session_v2(app_end, source, config, reconnect);
            if reconnect_pipe_target && matches!(exit, client_runtime::ClientExit::HostLost) {
                eprintln!("[rqhost-client] host lost → exit（exit-on-EOF 策略档）");
            }
            println!("[autodesk-client] exit={exit:?} revision={}", source.revision());
            Ok(())
        }
        FrameMode::Commands => {
            let (per_app_pipe, app_end) = connect(&target, &opts.app_name, render)?;
            let mut projector = RqProjector::new(component, opts.width, opts.height);
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
    /// （RqProjector + ensure_covered + 产帧）——001/003 单文件载体 Covered
    /// 且帧非空（AC-01 单元级证据；027 多文件 front 的 Covered/真渲由
    /// T-07 e2e 启动门承载——ensure_covered 即门，VM 全构建需真装载器
    /// 路径语义，naive 合并不通）。
    #[test]
    fn vm_queue_arm_assembly_covered() {
        for dir in ["001-helloworld", "003-converter"] {
            let Some(src) = example_source(dir) else {
                eprintln!("[p033] skip: {dir} 载体缺席");
                return;
            };
            let component =
                crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{dir}: {e}"));
            let mut projector = RqProjector::new(component, 480.0, 320.0);
            projector
                .ensure_covered()
                .unwrap_or_else(|gate| panic!("{dir} 未过覆盖门: {gate}"));
            let frame = projector.render_frame();
            assert!(!frame.ops.is_empty(), "{dir} 空帧");
            assert!(projector.revision() >= 1);
        }
    }

    // ---------------- PLAN-693 T-02：Desktop 连接语义 ----------------

    fn ce_pipe(tag: &str) -> String {
        format!(
            "{}-ce-{tag}-{}",
            crate::ui::desktop_protocol::rqhost::RQHOST_PIPE,
            std::process::id()
        )
    }

    fn ce_render() -> broker::RequestedRender {
        broker::RequestedRender { mode: FrameMode::Commands, auto_downgraded: false }
    }

    /// AC-03（不孵化语义·缺度态）：desktop 端点缺席 = 干净报错提示先启动
    /// 虚拟桌面——`adopt` 本就只连不孵（无 spawn 分支可进），报错即全部
    /// 语义；孵化不发生在此结构性可见（connect 无 spawner 注入点）。
    #[test]
    fn desktop_connect_absent_endpoint_errs_clean() {
        let endpoint = ce_pipe("absent");
        let target = ClientTarget::Desktop {
            endpoint: endpoint.clone(),
            app_name: "t".to_string(),
        };
        // Ok 侧 Transport 非 Debug——match 取 Err（勿 unwrap_err）。
        let err = match connect(&target, "t", ce_render()) {
            Ok(_) => panic!("缺席端点不应连上（不孵化语义）"),
            Err(e) => e,
        };
        assert!(
            err.contains("请先启动虚拟桌面") && err.contains(&endpoint),
            "报错须含端点名与先启动桌面提示：{err}"
        );
        // 宿主亡策略同 rqhost：exit-on-EOF（reconnect=None）。
        assert!(
            reconnect_for(&target, "per-app".into()).is_none(),
            "Desktop 档 = exit-on-EOF（None）"
        );
    }

    /// AC-03（不孵化语义·在度态）：进程内 RqServe 替身（rqhost 测试同款
    /// 形态）——端点在位 = adopt 全链通（rendezvous→per-app 转连），连接
    /// 语义与 Rqhost 同构的单元级证据。
    #[test]
    fn desktop_connect_adopts_live_serve() {
        let endpoint = ce_pipe("live");
        let (serve, _claim) =
            crate::ui::desktop_protocol::rqhost::RqServe::start(&endpoint)
                .unwrap_or_else(|e| panic!("serve 替身启动失败: {e:?}"));
        let target =
            ClientTarget::Desktop { endpoint: endpoint.clone(), app_name: "t".to_string() };
        let (per_app, _app_end) = match connect(&target, "t", ce_render()) {
            Ok(v) => v,
            Err(e) => panic!("adopt 失败: {e}"),
        };
        assert!(!per_app.is_empty(), "per-app 管道名回传");
        assert!(per_app != endpoint, "转连 per-app 管道（非 well-known 本名）");
        serve.stop(&endpoint);
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
/// [`RqProjector::ensure_covered`]——拒绝退出留痕）；`Independent`
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
        // PLAN-683（remote 模式）：Commands 家族 + remote 标记（调用侧
        // ClientOpts.remote=true → headless 宿主分岔）；覆盖门不适用。
        RenderMode::Remote => (FrameMode::Commands, false, None),
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
/// - `Commands` → [`RqProjector`] 投影 + 泛型泵命令帧（启动覆盖门：
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
    // exit-on-EOF 策略档观测（rqhost/桌面端点两 adopt 形同款）。
    let rqhost_target =
        matches!(target, ClientTarget::Rqhost { .. } | ClientTarget::Desktop { .. });
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
        FrameMode::Commands if opts.remote => {
            // PLAN-683（remote 模式）：native 轨同享 headless 宿主
            //（HeadlessFrameSource 泛型于 Component——a2r 编译组件同臂）。
            let source = super::headless::HeadlessFrameSource::new(
                component,
                opts.width,
                opts.height,
            );
            let config = ClientConfig {
                app_name: opts.app_name.clone(),
                title: opts.title,
                width: opts.width,
                height: opts.height,
            };
            let reconnect = reconnect_for(&target, per_app_pipe);
            let (exit, source) =
                client_runtime::run_client_session_v2(app_end, source, config, reconnect);
            if rqhost_target && matches!(exit, client_runtime::ClientExit::HostLost) {
                eprintln!("[rqhost-client] host lost → exit（exit-on-EOF 策略档）");
            }
            println!("[autodesk-client] exit={exit:?} revision={}", source.revision());
            Ok(())
        }
        FrameMode::Commands => {
            let projector = RqProjector::new(component, opts.width, opts.height);
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
