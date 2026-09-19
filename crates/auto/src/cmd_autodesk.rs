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
// Plan 500 步骤 6 —— 三态渲染开关（裁决链：spawn `--autodesk-render=` >
// pac.at `desktop_render:` > auto）。spawn 参数名带 autodesk 前缀：CLI
// `run` 已有具名 `--render`（前端后端，clap 具名先吞）——撞名第二处，
// 与 --autodesk-client/-broker 同族避让：
// - `queue` → ClientPump（DrawList 命令帧，NativeProjector 投影——
//   PLAN-033 T-02 改接，与 native 轨同臂）；
// - `independent` → **解释轨已退役**（PLAN-033 T-04：装载后报错留痕；
//   a2r 轨像素兜底 run_independent_native_child 不受影响）；
// - `auto` → queue（启动覆盖门在 client_entry::run_dynamic_client 权威
//   裁决——native 门拒即拒绝渲染退出留痕，禁静默错绘）。
//
// Plan 020 T-02 —— 端点解析 + 帧二态分派抽壳至
// [`client_entry::run_dynamic_client`]（本文件保留解释轨专属的 .at 装载与
// 三态裁决，行为零变化；native 轨 a2r exe 客户端臂复用同一 client_entry
// 骨架）。

use auto_lang::ui::desktop_protocol::broker::BROKER_PIPE;
use auto_lang::ui::desktop_protocol::client_entry::{
    self, ClientOpts, ClientTarget,
};
use auto_lang::ui::desktop_protocol::coverage::{self, RenderMode};

/// Run 分支入口裁决：孵化标记在册 → 协议 client 循环（走完即返回）；
/// `None` = ③ 独立形态，调用方继续现行 Run 流程。
pub fn run_if_client_entry(args: &[String]) -> Option<Result<(), String>> {
    let has_client = args.iter().any(|a| a.starts_with("--autodesk-client="));
    let has_incubate = args.iter().any(|a| a == "--autodesk-incubate");
    let has_shell = args.iter().any(|a| a == "--autodesk-shell");
    if !has_client && !has_incubate && !has_shell {
        return None;
    }
    Some(run_client_entry(args))
}

/// 协议 client 形态全流程：三态裁决 → 装载 App → 端点 → 主循环/像素臂
/// → 出口落 stdout。PLAN-030：`--autodesk-shell` = 壳 outproc 客户端
/// （双表面 + 投影下行 + DesktopBus 上行——broker 管道必填）。
fn run_client_entry(args: &[String]) -> Result<(), String> {
    let mut pipe: Option<String> = None;
    let mut app_name: Option<String> = None;
    let mut broker_pipe = BROKER_PIPE.to_string();
    let mut render_arg: Option<String> = None;
    let mut shell_entry = false;
    for arg in args {
        if let Some(v) = arg.strip_prefix("--autodesk-client=") {
            pipe = Some(v.to_string());
        } else if let Some(v) = arg.strip_prefix("--app386=") {
            app_name = Some(v.to_string());
        } else if let Some(v) = arg.strip_prefix("--autodesk-broker=") {
            broker_pipe = v.to_string();
        } else if let Some(v) = arg.strip_prefix("--autodesk-render=") {
            render_arg = Some(v.to_string());
        } else if arg == "--autodesk-shell" {
            shell_entry = true;
        }
    }
    if shell_entry {
        return auto_lang::ui::desktop_protocol::shell_client::run_shell_outproc(&broker_pipe);
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
            eprintln!(
                "[autodesk-client] 未知 --autodesk-render={arg}（auto|queue|independent），回退裁决链"
            );
        }
    }
    let (frame_mode, downgrade) = coverage::effective_frame_mode(mode, &component);
    if let Some(line) = &downgrade {
        eprintln!("[autodesk-client] {line}");
    }

    // 端点：① spawn 注入直连（模式位随 Hello 协商缺省 Commands——直连
    // 宿主为单 client 测试机件，v1.3 像素臂走 ② broker 孵化记录带模式）/
    // ② broker 孵化（记录第三字段携带二态模式 + auto 降级标记）。
    // Plan 020 T-02：此后的端点解析与帧二态分派在 client_entry（零行为差）。
    let target = match pipe {
        Some(p) => ClientTarget::Direct(p),
        None => ClientTarget::Broker { broker_pipe },
    };
    let opts = ClientOpts {
        app_name: app_name.clone(),
        title: app_name,
        width: 480.0,
        height: 320.0,
        frame_mode,
        auto_downgraded: downgrade.is_some(),
    };
    client_entry::run_dynamic_client(component, opts, target)
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

// ---------------------------------------------------------------------------
// PLAN-031 T-06 —— `-q/--render-queue` gate（main.rs Run 臂消费）
// ---------------------------------------------------------------------------

/// -q gate 校验：出界组合显式报错（vue/tauri/jet/arkts 前端不支持
/// rqhost 形态；`--rq-host` 参数面预留未实现——desktop 形态归 PLAN-030
/// 线）。缺省（render 缺席）= vm 轨合法。
pub fn rqhost_gate_validate(render: Option<&str>, args: &[String]) -> Result<(), String> {
    if let Some(r) = render {
        if !matches!(r, "vm" | "rust") {
            return Err(format!(
                "--render-queue 仅支持 vm/rust 前端目标（当前 --render={r}）"
            ));
        }
    }
    if let Some(a) = args.iter().find(|a| a.starts_with("--rq-host=")) {
        return Err(format!("{a} 预留未实现（desktop 形态归 PLAN-030 线）"));
    }
    Ok(())
}

/// rust 轨注入 args（cargo `--` 透传 → 生成 gate 消费）：
/// - `--autodesk-rqhost`：策略档标记——新 gate 选 `ClientTarget::Rqhost`
///   （rendezvous 采纳 + exit-on-EOF）；旧生成物不识未知旗标（安全忽略）
///   退化为 broker 孵化直连（渲染通、宿主死 30s 挂等——重生成后全语义）；
/// - `--autodesk-render=queue`：定档 queue（跳过 auto 覆盖扫描裁决）；
/// - `--autodesk-incubate --autodesk-broker=<wellknown>`：孵化入场
///   （rqhost serve 双动词兼容 incubate 记录）。
pub fn rqhost_injected_args(wellknown: &str, args: &[String]) -> Vec<String> {
    let mut injected = vec![
        "--autodesk-rqhost".to_string(),
        "--autodesk-render=queue".to_string(),
        "--autodesk-incubate".to_string(),
        format!("--autodesk-broker={wellknown}"),
    ];
    injected.extend_from_slice(args);
    injected
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T-06：gate 校验——vm/rust/缺省合法；vue/jet 等出界显式报错；
    /// `--rq-host` 预留位显式拒绝。
    #[test]
    fn rqhost_gate_validate_variants() {
        assert!(rqhost_gate_validate(None, &[]).is_ok(), "缺省 = vm 轨合法");
        assert!(rqhost_gate_validate(Some("vm"), &[]).is_ok());
        assert!(rqhost_gate_validate(Some("rust"), &[]).is_ok());
        for bad in ["vue", "tauri", "jet", "arkts"] {
            let err = rqhost_gate_validate(Some(bad), &[]).unwrap_err();
            assert!(err.contains(bad), "报错含目标名: {err}");
        }
        let arg = "--rq-host=desktop".to_string();
        let err = rqhost_gate_validate(Some("vm"), &[arg.clone()]).unwrap_err();
        assert!(err.contains("--rq-host"), "预留位拒绝: {err}");
    }

    /// T-06：注入形状——四旗标前注 + 原尾参透传保序。
    #[test]
    fn rqhost_injected_args_shape() {
        let args = vec!["--foo=1".to_string(), "positional".to_string()];
        let injected = rqhost_injected_args("autodesk-rqhost-t1", &args);
        assert_eq!(
            injected,
            vec![
                "--autodesk-rqhost".to_string(),
                "--autodesk-render=queue".to_string(),
                "--autodesk-incubate".to_string(),
                "--autodesk-broker=autodesk-rqhost-t1".to_string(),
                "--foo=1".to_string(),
                "positional".to_string(),
            ]
        );
    }
}
