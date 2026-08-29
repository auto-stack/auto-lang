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

use auto_lang::ui::desktop_protocol::broker::{self, BROKER_PIPE};
use auto_lang::ui::desktop_protocol::client_runtime::{
    self, AppProjector, ClientConfig, ReconnectPolicy,
};
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

/// 协议 client 形态全流程：装载 App → 端点 → 主循环 → 出口落 stdout。
fn run_client_entry(args: &[String]) -> Result<(), String> {
    let mut pipe: Option<String> = None;
    let mut app_name: Option<String> = None;
    let mut broker_pipe = BROKER_PIPE.to_string();
    for arg in args {
        if let Some(v) = arg.strip_prefix("--autodesk-client=") {
            pipe = Some(v.to_string());
        } else if let Some(v) = arg.strip_prefix("--app386=") {
            app_name = Some(v.to_string());
        } else if let Some(v) = arg.strip_prefix("--autodesk-broker=") {
            broker_pipe = v.to_string();
        }
    }
    let app_name = app_name
        .ok_or("--app386=<name> 必填（孵化 App 名 = <app-root>/<name>）")?
        .clone();

    // App 源装载（AUTO_386_APP_ROOT > ./examples/ui）。
    let root = std::env::var("AUTO_386_APP_ROOT").unwrap_or_else(|_| "examples/ui".to_string());
    let path = std::path::Path::new(&root).join(&app_name).join("src/front/app.at");
    let src = std::fs::read_to_string(&path)
        .map_err(|e| format!("装载 {}: {e}", path.display()))?;
    let component = auto_lang::build_dynamic_component(&src, Some(&path.to_string_lossy()))
        .map_err(|e| format!("编译 {}: {e}", path.display()))?;

    // 端点：① spawn 注入直连 / ② broker 孵化。
    let (per_app_pipe, app_end) = match pipe {
        Some(p) => {
            let end = transport::connect(&p, 5000).map_err(|e| format!("连 {p}: {e:?}"))?;
            (p, end)
        }
        None => broker::request_incubation(&broker_pipe, &app_name, 5000)
            .map_err(|e| format!("broker 孵化失败: {e:?}"))?,
    };

    let config = ClientConfig {
        app_name: app_name.clone(),
        title: app_name,
        width: 480.0,
        height: 320.0,
    };
    let reconnect = ReconnectPolicy { pipe: per_app_pipe, budget_ms: 30_000, interval_ms: 50 };
    let projector = AppProjector::new(component, config.width, config.height);
    let (exit, projector) = client_runtime::run_client(app_end, projector, config, Some(reconnect));
    println!("[autodesk-client] exit={exit:?} revision={}", projector.revision());
    Ok(())
}
