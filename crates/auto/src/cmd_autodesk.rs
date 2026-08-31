// Plan 480 S2 —— 双模 exe 入口：`auto run` 的协议 client 形态。
//
// `auto run --autodesk-client=<pipe> --app386=<name>` = 孵化通道 ①（直连
// per-app 管道）；`auto run --autodesk-incubate --app386=<name>` = ②（向
// broker 请求孵化，`--autodesk-broker=<pipe>` 可改 broker 管道名）；
// 无标记 = ③ 独立形态，现行 `auto run` 行为零改动。
//
// App 材料装载：`examples/ui/<name>/src/front/app.at`（根目录可用
// `AUTO_386_APP_ROOT` 覆盖——测试用临时目录注入最小 App）。装载后走
// [`client_runtime::run_client`] 阻塞主循环（重连策略在册：host 断连
// 后原地等待重连 30s，VM 状态/revision 保持——S7 弹性语义）。
//
// Plan 500 步骤 6 —— 三态渲染开关（裁决链：spawn `--render=` >
// pac.at `desktop_render:` > auto）：
// - `queue` → 既有 ClientPump（DrawList 命令帧，AppProjector 投影）；
// - `independent` → [`pixels::run_independent_child`]（自带 iced 隐藏窗
//   自渲染 + screenshot 像素帧）；
// - `auto` → 装载期覆盖度探测（coverage::effective_frame_mode）：
//   Covered → queue；NotCovered → 降级 independent（孵化记录带
//   `pixels:auto` 标记，宿主观测行留痕）。

use auto_lang::ui::desktop_protocol::broker::{self, BROKER_PIPE};
use auto_lang::ui::desktop_protocol::client_runtime::{
    self, AppProjector, ClientConfig, ReconnectPolicy,
};
use auto_lang::ui::desktop_protocol::coverage::{self, RenderMode};
use auto_lang::ui::desktop_protocol::message::FrameMode;
use auto_lang::ui::desktop_protocol::pixels;
use auto_lang::ui::desktop_protocol::transport;

/// Run 分支入口裁决：孵化标记在册 → 协议 client 循环（走完即返回）；
/// `None` = ③ 独立形态，调用方继续现行 Run 流程。
pub fn run_if_client_entry(args: &[String]) -> Option<Result<(), String>> {
    let has_client = args.iter().any(|a| a.starts_with("--autodesk-client="));
    let has_incubate = args.iter().any(|a| a == "--autodesk-incubate");
    if !has_client && !has_incubate {
        return None;
    }
    Some(run_client_entry(args))
}

/// 协议 client 形态全流程：三态裁决 → 装载 App → 端点 → 主循环/像素臂
/// → 出口落 stdout。
fn run_client_entry(args: &[String]) -> Result<(), String> {
    let mut pipe: Option<String> = None;
    let mut app_name: Option<String> = None;
    let mut broker_pipe = BROKER_PIPE.to_string();
    let mut render_arg: Option<String> = None;
    for arg in args {
        if let Some(v) = arg.strip_prefix("--autodesk-client=") {
            pipe = Some(v.to_string());
        } else if let Some(v) = arg.strip_prefix("--app386=") {
            app_name = Some(v.to_string());
        } else if let Some(v) = arg.strip_prefix("--autodesk-broker=") {
            broker_pipe = v.to_string();
        } else if let Some(v) = arg.strip_prefix("--render=") {
            render_arg = Some(v.to_string());
        }
    }
    let app_name = app_name
        .ok_or("--app386=<name> 必填（孵化 App 名 = <app-root>/<name>）")?
        .clone();

    // App 源装载（AUTO_386_APP_ROOT > ./examples/ui）。
    let root = std::env::var("AUTO_386_APP_ROOT").unwrap_or_else(|_| "examples/ui".to_string());
    let app_dir = std::path::Path::new(&root).join(&app_name);
    let path = app_dir.join("src/front/app.at");
    let src = std::fs::read_to_string(&path)
        .map_err(|e| format!("装载 {}: {e}", path.display()))?;
    let component = auto_lang::build_dynamic_component(&src, Some(&path.to_string_lossy()))
        .map_err(|e| format!("编译 {}: {e}", path.display()))?;

    // 三态裁决：spawn 参数 > pac.at desktop_render > auto 探测
    //（与进程形态裁决 adjudicate() 正交——那条链定 Client/Broker/
    // Standalone，本链定帧载荷形态）。
    let manifest = read_manifest_render(&app_dir);
    let mode = RenderMode::resolve(
        render_arg.as_deref(),
        manifest.as_deref(),
    );
    if let Some(arg) = render_arg.as_deref() {
        if RenderMode::parse(arg).is_none() {
            eprintln!("[autodesk-client] 未知 --render={arg}（auto|queue|independent），回退裁决链");
        }
    }
    let (frame_mode, downgrade) = coverage::effective_frame_mode(mode, &component);
    if let Some(line) = &downgrade {
        eprintln!("[autodesk-client] {line}");
    }
    let auto_downgraded = downgrade.is_some();

    // 端点：① spawn 注入直连（模式位随 Hello 协商缺省 Commands——直连
    // 宿主为单 client 测试机件，v1.3 像素臂走 ② broker 孵化记录带模式）/
    // ② broker 孵化（记录第三字段携带二态模式 + auto 降级标记）。
    let (per_app_pipe, app_end) = match pipe {
        Some(p) => {
            let end = transport::connect(&p, 5000).map_err(|e| format!("连 {p}: {e:?}"))?;
            (p, end)
        }
        None => {
            let render = broker::RequestedRender { mode: frame_mode, auto_downgraded };
            broker::request_incubation_render(&broker_pipe, &app_name, render, 5000)
                .map_err(|e| format!("broker 孵化失败: {e:?}"))?
        }
    };

    match frame_mode {
        FrameMode::Pixels => {
            // independent 臂：自带 iced 隐藏窗自渲染 + screenshot 像素帧
            //（阻塞至宿主 Close；VM 状态在渲染宿主会话内）。
            pixels::run_independent_child(
                app_end,
                component,
                &app_name,
                &app_name,
                480.0,
                320.0,
            )
            .map(|_| ())
        }
        FrameMode::Commands => {
            let config = ClientConfig {
                app_name: app_name.clone(),
                title: app_name,
                width: 480.0,
                height: 320.0,
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

/// pac.at `desktop_render:` 声明读取（auto-man Pac 解析规则的最小本地
/// 复刻——.at 顶层字符串 prop；auto-man 侧 Pac::desktop_render 为工程
/// 装配面，child 进程不引 auto-man）。
fn read_manifest_render(app_dir: &std::path::Path) -> Option<String> {
    let pac = std::fs::read_to_string(app_dir.join("pac.at")).ok()?;
    for line in pac.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("desktop_render:") {
            let v = rest.trim().trim_matches('"').trim_matches('\'').trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}
