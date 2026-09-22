// PLAN-693 —— a2r exe 三模式 CLI（G1）：渲染底座参数化——同一编译 exe
// 以三种参数启动得到三种正确形态：
// - `independent`（缺省）：本地 iced wgpu 自窗（现状）；
// - `rq`：连接/孵化 rqhost 共享合成器（`auto run -q` 语义内联到 exe）；
// - `desktop`：连接虚拟桌面进程的合成器端点（不孵化——桌面已在前，
//   端点缺席 = 报错提示先启动桌面）。
//
// CLI 面：`--render-mode independent|rq|desktop` + `-q`/`--rq` 语义糖
//（= rq）+ `--desktop-endpoint <pipe>`（desktop 必需）+ `--window <WxH>` /
// `--title <t>`（既有 AUTO_VM_WINDOW/TITLE 的 CLI 形）。解析手写轻量
//（不引 clap——a2r exe 依赖面克制）；未知参数容错透传（v1 语义），
// 显式已知参数畸形 = Err（用户意图在册，静默吞成缺省反而背离）。
//
// 优先级链（F-3 关闭口径，设计档 rq-remote-renderer §8）：CLI >
// `AUTO_VM_RENDER` env > pac `desktop_render` > independent——remote 族
// 恒显式选用，缺省恒独立轨。env/pac 值域映射：`rq|remote` → Rq（rqhost
// 底座 v1 恒 DisplayList v2 remote 宿主，remote 即 rq 的帧载荷侧面）；
// `desktop` → Desktop；`independent` → Independent；`queue|auto`（帧覆盖
// 声明，非底座选择）→ 跳过降下一级；未知值 → 跳过 + 观测行留痕。

/// 渲染底座三模式（PLAN-693；帧载荷维度仍是
/// [`super::coverage::RenderMode`]——两维度正交，本枚举只裁"窗在哪"）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBase {
    /// 本地 iced wgpu 自窗（缺省，独立轨）。
    Independent,
    /// rqhost 共享合成器（探测→孵化，`-q` 语义）。
    Rq,
    /// 虚拟桌面合成器端点（探测即连，不孵化）。
    Desktop,
}

impl RenderBase {
    /// 值域串（观测行/报错消息用）。
    pub fn name(self) -> &'static str {
        match self {
            Self::Independent => "independent",
            Self::Rq => "rq",
            Self::Desktop => "desktop",
        }
    }

    /// 底座声明串解析（CLI/env/pac 三来源共用）。`remote` = Rq 别名
    ///（rqhost 底座 v1 恒 remote 帧宿主）；`queue|auto` = None（帧覆盖
    /// 声明非底座选择，调用方降下一优先级）。
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "independent" => Some(Self::Independent),
            "rq" | "remote" | "rqhost" => Some(Self::Rq),
            "desktop" => Some(Self::Desktop),
            _ => None,
        }
    }
}

/// 三模式 CLI 解析产物。`mode` = CLI 显式底座（None = CLI 未选——env/pac
/// 腿接手）；`desktop_endpoint` 仅 CLI 面供给（desktop 模式缺它 = 调用方
/// 报错，列出探测过的 wellknown）。
#[derive(Debug, Clone, Default)]
pub struct RenderCli {
    pub mode: Option<RenderBase>,
    pub desktop_endpoint: Option<String>,
    /// `--window <WxH>`（既有 `AUTO_VM_WINDOW` 的 CLI 形；调用方覆写 env
    /// 后经 [`super::rqhost::vm_window_size`] 统一消费——边界校验同式）。
    pub window: Option<(f32, f32)>,
    pub title: Option<String>,
}

/// 窗尺寸解析（`AUTO_VM_WINDOW` 同式：`<W>x<H>`；边界 200..=7680 /
/// 200..=4320）。畸形 = Err（CLI 显式面不静默吞）。
fn parse_window_spec(spec: &str) -> Result<(f32, f32), String> {
    let Some((w, h)) = spec.trim().split_once(['x', 'X']) else {
        return Err(format!("--window 畸形（{spec:?}，期望 <W>x<H>，如 800x600）"));
    };
    let (w, h) = (
        w.trim().parse::<f32>(),
        h.trim().parse::<f32>(),
    );
    match (w, h) {
        (Ok(w), Ok(h))
            if (200.0..=7680.0).contains(&w) && (200.0..=4320.0).contains(&h) =>
        {
            Ok((w, h))
        }
        _ => Err(format!(
            "--window 越界或非数（{spec:?}；合法 200..=7680 × 200..=4320）"
        )),
    }
}

/// 三模式 CLI 解析（a2r 生成 main 调用；args = `std::env::args()` 全量）。
/// 未知参数容错透传忽略；重复参数后者覆盖（last-wins）；`-q`/`--rq` =
/// `--render-mode rq` 语义糖。
pub fn parse_render_cli(args: &[String]) -> Result<RenderCli, String> {
    let mut rc = RenderCli::default();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let mut next = |i: &mut usize, flag: &str| -> Result<String, String> {
            // `--flag value` 与 `--flag=value` 双形。
            if let Some(v) = a.strip_prefix(&format!("{flag}=")) {
                return Ok(v.to_string());
            }
            if a == flag {
                *i += 1;
                return args.get(*i).cloned().ok_or_else(|| {
                    format!("{flag} 缺值（期望后随参数或 = 值形）")
                });
            }
            Err(format!("{flag} 未匹配"))
        };
        if a == "-q" || a == "--rq" {
            rc.mode = Some(RenderBase::Rq);
        } else if a.starts_with("--render-mode=") || a == "--render-mode" {
            let v = next(&mut i, "--render-mode")?;
            rc.mode = Some(
                RenderBase::parse(&v)
                    .ok_or_else(|| format!("未知 --render-mode={v:?}（independent|rq|desktop）"))?,
            );
        } else if a.starts_with("--desktop-endpoint=") || a == "--desktop-endpoint" {
            rc.desktop_endpoint = Some(next(&mut i, "--desktop-endpoint")?);
        } else if a.starts_with("--window=") || a == "--window" {
            rc.window = Some(parse_window_spec(&next(&mut i, "--window")?)?);
        } else if a.starts_with("--title=") || a == "--title" {
            rc.title = Some(next(&mut i, "--title")?);
        }
        // 其余参数容错透传忽略（既有 `--autodesk-*` 族 / 未知旗标 v1）。
        i += 1;
    }
    Ok(rc)
}

/// 底座优先级链（F-3 关闭口径）：CLI > env > pac > independent。返回
/// (底座, Option<观测行>)——观测行仅在非缺省腿生效/未知值跳过时给出
///（缺省零噪音）。env/pac 未选底座（None/queue/auto/未知）逐级降。
pub fn resolve_render_base(
    cli: Option<RenderBase>,
    env: Option<&str>,
    pac: Option<&str>,
) -> (RenderBase, Option<String>) {
    if let Some(m) = cli {
        return (m, Some(format!("[render] render-mode: {} (CLI)", m.name())));
    }
    if let Some(v) = env {
        match RenderBase::parse(v) {
            Some(m) => {
                return (m, Some(format!("[render] render-mode: {} (AUTO_VM_RENDER env)", m.name())))
            }
            None if v.trim().eq_ignore_ascii_case("queue")
                || v.trim().eq_ignore_ascii_case("auto") => {}
            // 未知 env 值留痕（queue/auto = 帧覆盖声明，静默降级正常态）。
            None => {
                return (
                    RenderBase::Independent,
                    Some(format!("[render] AUTO_VM_RENDER={v:?} 不识——跳过降下级（independent）")),
                )
            }
        }
    }
    if let Some(v) = pac {
        match RenderBase::parse(v) {
            Some(m) => {
                return (m, Some(format!("[render] render-mode: {} (pac desktop_render)", m.name())))
            }
            None if v.trim().eq_ignore_ascii_case("queue")
                || v.trim().eq_ignore_ascii_case("auto") => {}
            None => {
                return (
                    RenderBase::Independent,
                    Some(format!("[render] pac desktop_render={v:?} 不识——跳过（independent）")),
                )
            }
        }
    }
    (RenderBase::Independent, None)
}

/// exe 旁 pac.at 的 `desktop_render:`（auto build 部署形态随行时生效；
/// 缺席 = None——生成物 pac 位置感知的运行期最小面，解析规则 =
/// cmd_autodesk `read_manifest_render` 同式复刻）。
pub fn sidecar_pac_desktop_render() -> Option<String> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    sidecar_pac_desktop_render_from(&dir)
}

/// [`sidecar_pac_desktop_render`] 的目录注入形（可测性缝——测试以临时
/// 目录覆盖，不依赖 current_exe）。
pub fn sidecar_pac_desktop_render_from(dir: &std::path::Path) -> Option<String> {
    let pac = std::fs::read_to_string(dir.join("pac.at")).ok()?;
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

// ================================ 测试 ================================

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    // ---------------- 解析矩阵（T-01） ----------------

    #[test]
    fn parse_three_modes_and_sugar() {
        let rc = parse_render_cli(&args(&["app", "--render-mode", "rq"])).unwrap();
        assert_eq!(rc.mode, Some(RenderBase::Rq));
        let rc = parse_render_cli(&args(&["--render-mode=independent"])).unwrap();
        assert_eq!(rc.mode, Some(RenderBase::Independent));
        let rc = parse_render_cli(&args(&["--render-mode=desktop"])).unwrap();
        assert_eq!(rc.mode, Some(RenderBase::Desktop));
        // 语义糖。
        let rc = parse_render_cli(&args(&["-q"])).unwrap();
        assert_eq!(rc.mode, Some(RenderBase::Rq));
        let rc = parse_render_cli(&args(&["--rq"])).unwrap();
        assert_eq!(rc.mode, Some(RenderBase::Rq));
        // 无模式旗标 = CLI 未选。
        let rc = parse_render_cli(&args(&["--autodesk-incubate", "positional"])).unwrap();
        assert_eq!(rc.mode, None, "未知/既有参数容错透传，mode 不受扰");
    }

    #[test]
    fn parse_desktop_endpoint_window_title() {
        let rc = parse_render_cli(&args(&[
            "--render-mode",
            "desktop",
            "--desktop-endpoint",
            "my-desktop-pipe",
            "--window",
            "800x600",
            "--title=部署面板",
        ]))
        .unwrap();
        assert_eq!(rc.mode, Some(RenderBase::Desktop));
        assert_eq!(rc.desktop_endpoint.as_deref(), Some("my-desktop-pipe"));
        assert_eq!(rc.window, Some((800.0, 600.0)));
        assert_eq!(rc.title.as_deref(), Some("部署面板"));
        // = 形单 token。
        let rc = parse_render_cli(&args(&["--desktop-endpoint=p2", "--window=1024x768"]))
            .unwrap();
        assert_eq!(rc.desktop_endpoint.as_deref(), Some("p2"));
        assert_eq!(rc.window, Some((1024.0, 768.0)));
    }

    #[test]
    fn parse_malformed_known_args_err() {
        // 显式已知参数畸形 = Err（不静默吞成缺省）。
        assert!(parse_render_cli(&args(&["--render-mode", "rqq"])).is_err());
        assert!(parse_render_cli(&args(&["--render-mode"])).is_err(), "缺值");
        assert!(parse_render_cli(&args(&["--desktop-endpoint"])).is_err(), "缺值");
        assert!(parse_render_cli(&args(&["--window", "800"])).is_err(), "缺 H");
        assert!(parse_render_cli(&args(&["--window", "100x600"])).is_err(), "越界下");
        assert!(parse_render_cli(&args(&["--window", "800x9999"])).is_err(), "越界上");
        assert!(parse_render_cli(&args(&["--window", "axb"])).is_err(), "非数");
    }

    #[test]
    fn parse_last_wins_on_repeat() {
        let rc =
            parse_render_cli(&args(&["--render-mode", "rq", "--render-mode", "desktop"])).unwrap();
        assert_eq!(rc.mode, Some(RenderBase::Desktop), "重复参数 last-wins");
        let rc = parse_render_cli(&args(&["-q", "--render-mode", "independent"])).unwrap();
        assert_eq!(rc.mode, Some(RenderBase::Independent), "糖与全名同位竞争");
    }

    // ---------------- 优先级链（AC-02：CLI 压 env 压 pac） ----------------

    #[test]
    fn priority_chain_cli_beats_env_beats_pac() {
        let (b, _) = resolve_render_base(
            Some(RenderBase::Independent),
            Some("remote"),
            Some("desktop"),
        );
        assert_eq!(b, RenderBase::Independent, "CLI 顶优先（含显式 independent 压 env/pac）");
        let (b, log) =
            resolve_render_base(Some(RenderBase::Rq), Some("desktop"), Some("desktop"));
        assert_eq!(b, RenderBase::Rq);
        assert!(log.unwrap().contains("CLI"));
        let (b, log) = resolve_render_base(None, Some("desktop"), Some("rq"));
        assert_eq!(b, RenderBase::Desktop, "env 压 pac");
        assert!(log.unwrap().contains("AUTO_VM_RENDER"));
        let (b, log) = resolve_render_base(None, None, Some("rq"));
        assert_eq!(b, RenderBase::Rq, "pac 腿兜底");
        assert!(log.unwrap().contains("pac"));
    }

    #[test]
    fn priority_chain_defaults_and_skips() {
        // 全缺席 = independent 零噪音（F-3：缺省恒独立轨）。
        let (b, log) = resolve_render_base(None, None, None);
        assert_eq!(b, RenderBase::Independent);
        assert!(log.is_none(), "缺省无观测行");
        // queue/auto = 帧覆盖声明非底座——跳过降级（静默，正常态）。
        let (b, log) = resolve_render_base(None, Some("queue"), Some("auto"));
        assert_eq!(b, RenderBase::Independent);
        assert!(log.is_none());
        // remote = rq 别名（env 腿——`-q --render=remote` 既有透传链）。
        let (b, _) = resolve_render_base(None, Some("remote"), None);
        assert_eq!(b, RenderBase::Rq);
        // 未知值 = 跳过 + 观测行留痕。
        let (b, log) = resolve_render_base(None, Some("weird"), None);
        assert_eq!(b, RenderBase::Independent);
        assert!(log.unwrap().contains("不识"));
    }

    // ---------------- 旁车 pac（部署形态） ----------------

    #[test]
    fn sidecar_pac_read_and_absent() {
        let dir = std::env::temp_dir().join(format!("p693-sidecar-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(sidecar_pac_desktop_render_from(&dir).is_none(), "无 pac.at = None");
        std::fs::write(dir.join("pac.at"), "name = \"demo\"\ndesktop_render: \"remote\"\n")
            .unwrap();
        assert_eq!(
            sidecar_pac_desktop_render_from(&dir).as_deref(),
            Some("remote"),
            "exe 旁 pac.at desktop_render 读取"
        );
        // 值域外透传（解析在 resolve 腿）+ 单引号形 + 空值跳过。
        std::fs::write(dir.join("pac.at"), "desktop_render: 'desktop'\n").unwrap();
        assert_eq!(sidecar_pac_desktop_render_from(&dir).as_deref(), Some("desktop"));
        std::fs::write(dir.join("pac.at"), "desktop_render: \"\"\n").unwrap();
        assert!(sidecar_pac_desktop_render_from(&dir).is_none(), "空值 = 未声明");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---------------- 底座值域 ----------------

    #[test]
    fn base_parse_value_domain() {
        assert_eq!(RenderBase::parse("independent"), Some(RenderBase::Independent));
        assert_eq!(RenderBase::parse("rq"), Some(RenderBase::Rq));
        assert_eq!(RenderBase::parse("REMOTE"), Some(RenderBase::Rq), "别名大小写不敏感");
        assert_eq!(RenderBase::parse("rqhost"), Some(RenderBase::Rq));
        assert_eq!(RenderBase::parse("desktop"), Some(RenderBase::Desktop));
        assert_eq!(RenderBase::parse("queue"), None, "帧覆盖声明非底座");
        assert_eq!(RenderBase::parse("auto"), None);
        assert_eq!(RenderBase::parse("bogus"), None);
    }
}
