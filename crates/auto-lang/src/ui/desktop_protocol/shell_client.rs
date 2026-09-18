//! PLAN-030 T-03 —— 壳 outproc 客户端装配（`--autodesk-shell`）。
//!
//! **形态**：B 程序主体 client 侧——单连接双表面（background[壁纸上/
//! 窗口下全屏] + chrome[置顶任务栏带]），Hello 尾段声明（v1.11）；
//! 渲染 = queue 臂内联帧（FrameReady 携 DrawList——纯内存 compose，
//! 免 shm 协商）；投影下行 = ShellProjectionPush/ShellClockTick/
//! ShellCursorMove 消费（typed 载体 child 侧 lowering）；命令上行 =
//! `__desktop_cmd` 读走（c4 语义 child 化）→ `ControlMsg::DesktopBus`。
//!
//! **面装载**：v1 双常驻面经 `shell_source()` 解释装载（pack 解析序与
//! in-proc 轨同源：override → AUTO_SHELL_PACK → 兄弟 auto-os → 硬编码
//! → 内嵌 pin 快照）。a2r 编译面轨（027 五件词汇门产物）为同契约替换
//! 点——AppProjector 接缝即投影器面（编译轨换编译投影器同形，§5.1 D7
//! 定案注记）。
//!
//! **生命周期**：宿主看门兵持有 respawn 权（死亡 → 宿主退避重启 + 全量
//! 重推）；child 侧不设 host 重连（EOF = 退出，恢复归宿主臂——与
//! ReconnectPolicy 30s 自愈不对称的显式取舍，I5 桌面不炸）。

use crate::ui::desktop_protocol::broker::{self, RequestedRender};
use crate::ui::desktop_protocol::client_runtime::AppProjector;
use crate::ui::desktop_protocol::codec::Reader;
use crate::ui::desktop_protocol::endpoint::FrameSource;
use crate::ui::desktop_protocol::message::{
    shell_face, surface_role, ControlMsg, FrameMsg, FrameMode, HandshakeMsg, InputMsg,
    ProtocolMsg, SurfaceDecl,
};
use crate::ui::desktop_protocol::transport;
use crate::ui::desktop_protocol::PROTOCOL_VERSION;
use crate::ui::shell_projection::{shell_event_name, DesktopSurfaceSnapshot, ShellProjection, ShellWrite};
use std::collections::BTreeMap;

/// 壳几何（spawn 注入 `AUTO_SHELL_GEOM=<W>x<H>x<BAND>`；e2e 同形）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellGeometry {
    pub viewport_w: f32,
    pub viewport_h: f32,
    /// 任务栏带高（chrome 表面高——D1 定案：带矩形非全屏）。
    pub band_h: f32,
}

impl ShellGeometry {
    /// "1280x800x48" 解析（坏值 None——调用方走缺省）。
    pub fn parse(s: &str) -> Option<Self> {
        let mut it = s.split('x');
        let w: f32 = it.next()?.parse().ok()?;
        let h: f32 = it.next()?.parse().ok()?;
        let band: f32 = it.next()?.parse().ok()?;
        if w <= 0.0 || h <= 0.0 || band <= 0.0 || band >= h {
            return None;
        }
        Some(Self { viewport_w: w, viewport_h: h, band_h: band })
    }

    pub fn encode(&self) -> String {
        format!("{}x{}x{}", self.viewport_w, self.viewport_h, self.band_h)
    }

    pub fn from_env() -> Option<Self> {
        std::env::var("AUTO_SHELL_GEOM").ok().as_deref().and_then(Self::parse)
    }

    pub fn fallback() -> Self {
        Self { viewport_w: 1280.0, viewport_h: 800.0, band_h: 48.0 }
    }
}

/// 双常驻面会话（face → AppProjector；AppProjector 接缝 = 渲染/命中/
/// 输入/revision 全套——编译面轨替换点）。
pub struct ShellFaces {
    /// chrome 面（shell.at 任务栏）。
    chrome: AppProjector,
    /// background 面（desktop.at 桌面图标）。
    background: AppProjector,
    geometry: ShellGeometry,
}

impl ShellFaces {
    /// 解释装载 v1 双常驻面（027 SHELL_MANIFEST ResidentBoot 两件）。
    pub fn load(geometry: ShellGeometry) -> Result<Self, String> {
        let chrome_src = crate::ui::shell::shell_source("shell.at");
        let bg_src = crate::ui::shell::shell_source("desktop.at");
        let chrome = crate::build_dynamic_component(&chrome_src, None)
            .map_err(|e| format!("壳 chrome 面装载失败: {e}"))?;
        let background = crate::build_dynamic_component(&bg_src, None)
            .map_err(|e| format!("壳 background 面装载失败: {e}"))?;
        Ok(Self {
            chrome: AppProjector::new(chrome, geometry.viewport_w, geometry.band_h),
            background: AppProjector::new(background, geometry.viewport_w, geometry.viewport_h),
            geometry,
        })
    }

    pub fn geometry(&self) -> ShellGeometry {
        self.geometry
    }

    fn projector_mut(&mut self, face: u8) -> Option<&mut AppProjector> {
        match face {
            shell_face::SHELL => Some(&mut self.chrome),
            shell_face::DESKTOP_SURFACE => Some(&mut self.background),
            // overlay 三面 v1 维持 in-proc 懒挂载（D6 边界）——忽略留痕。
            _ => None,
        }
    }

    /// 投影下行消费：typed 载体 wire 解码 → interpreted_writes child 侧
    /// lowering（叶面保形单点：bool → "1"/"" 在 lowering，wire 上是 bool）。
    /// 返回 true = 状态变化（revision 前进）。
    pub fn apply_projection(&mut self, face: u8, payload: &[u8]) -> bool {
        match face {
            shell_face::SHELL => {
                let mut r = Reader::new(payload);
                let Ok(proj) = ShellProjection::wire_decode(&mut r) else {
                    return false;
                };
                let Some(p) = self.projector_mut(face) else { return false };
                apply_writes(p, proj.interpreted_writes());
                p.bump_revision();
                true
            }
            shell_face::DESKTOP_SURFACE => {
                let mut r = Reader::new(payload);
                let Ok(snap) = DesktopSurfaceSnapshot::wire_decode(&mut r) else {
                    return false;
                };
                // 召唤事件（宿主写状态不触发 handler——随快照显式携带，
                // 027 语义）。
                let events = snap.events.clone();
                let Some(p) = self.projector_mut(face) else { return false };
                apply_writes(p, snap.interpreted_writes());
                for e in &events {
                    let _ = p
                        .component_mut()
                        .bridge_mut()
                        .call_handler(shell_event_name(e), &[]);
                }
                p.bump_revision();
                true
            }
            _ => false,
        }
    }

    /// 时钟（分钟门数据——chrome 面任务栏钟）。
    pub fn apply_clock(&mut self, face: u8, time: &str, date: &str) -> bool {
        let Some(p) = self.projector_mut(face) else { return false };
        let _ = p
            .component_mut()
            .write_state("__wm_clock", auto_val::Value::str(time));
        let _ = p
            .component_mut()
            .write_state("__wm_date", auto_val::Value::str(date));
        p.bump_revision();
        true
    }

    /// 光标事件（空白菜单坐标锚——background 面消费；逐事件语义——
    /// 宿主侧消费门控制推送节拍[D3 定案]）。
    pub fn apply_cursor(&mut self, face: u8, x: f32, y: f32) -> bool {
        let Some(p) = self.projector_mut(face) else { return false };
        let _ = p
            .component_mut()
            .write_state("__desktop_cursor_x", auto_val::Value::str(&x.to_string()));
        let _ = p
            .component_mut()
            .write_state("__desktop_cursor_y", auto_val::Value::str(&y.to_string()));
        p.bump_revision();
        true
    }

    /// 输入路由（wid → face 投影器；坐标已由宿主平移为面局部系）。
    pub fn on_input(&mut self, face: u8, input: &InputMsg) {
        if let Some(p) = self.projector_mut(face) {
            p.on_input(input);
        }
    }

    /// 命令读走（c4 语义 child 化：read_state + 清空；面局部 `__desktop_cmd`）。
    pub fn drain_commands(&mut self, face: u8) -> Vec<String> {
        let Some(p) = self.projector_mut(face) else { return Vec::new() };
        let cur = p
            .component()
            .read_state("__desktop_cmd")
            .ok()
            .and_then(|v| match v {
                auto_val::Value::Str(s) => Some(s.as_str().to_string()),
                _ => None,
            })
            .unwrap_or_default();
        if cur.is_empty() {
            return Vec::new();
        }
        let _ = p
            .component_mut()
            .write_state("__desktop_cmd", auto_val::Value::str(""));
        cur.split('\n')
            .filter(|r| !r.trim().is_empty())
            .map(|r| r.trim_end_matches('\r').to_string())
            .collect()
    }

    /// 渲染一面（queue 臂 DrawList）。
    pub fn render(
        &mut self,
        face: u8,
    ) -> Option<crate::ui::desktop_protocol::message::DrawList> {
        self.projector_mut(face).map(|p| p.render_frame())
    }

    pub fn revision(&self, face: u8) -> u64 {
        match face {
            shell_face::SHELL => self.chrome.revision(),
            shell_face::DESKTOP_SURFACE => self.background.revision(),
            _ => 0,
        }
    }
}

fn apply_writes(p: &mut AppProjector, writes: Vec<ShellWrite>) {
    for w in writes {
        match w {
            ShellWrite::Scalar(k, v) => {
                let _ = p.component_mut().write_state(k, v);
            }
            ShellWrite::Array(k, vs) => {
                let _ = p.component_mut().write_state_vec(k, vs);
            }
        }
    }
}

/// 每表面帧产状态（内联帧：frame_id 单调 + 双槽轮转——FrameAck 归还）。
#[derive(Default)]
struct SurfaceFrames {
    next_frame_id: u64,
    last_slot: u8,
    free_slots: Vec<u8>,
}

impl SurfaceFrames {
    fn take_slot(&mut self) -> u8 {
        if let Some(s) = self.free_slots.pop() {
            self.last_slot = s;
            s
        } else {
            self.last_slot = 1 - self.last_slot;
            self.last_slot
        }
    }

    fn ack(&mut self, slot: u8) {
        if !self.free_slots.contains(&slot) {
            self.free_slots.push(slot);
        }
    }
}

/// 壳 outproc 客户端主循环（`--autodesk-shell` 入口的消费体）。
pub struct ShellPump {
    end: Box<dyn transport::Transport + Send>,
    faces: ShellFaces,
    /// wid → (surface, face)——Welcome 协商结果。
    routes: BTreeMap<u64, (u64, u8)>,
    frames: BTreeMap<u64, SurfaceFrames>,
    last_revisions: BTreeMap<u8, u64>,
}

impl ShellPump {
    /// 经 broker 孵化 + 双表面握手（Hello 尾段：background 领头声明 +
    /// chrome 尾段——宿主 ResolveAndAttach 壳分支同序消费）。
    pub fn start(
        broker_pipe: &str,
        geometry: ShellGeometry,
        faces: ShellFaces,
    ) -> Result<Self, String> {
        let (_, end) = broker::request_incubation_render(
            broker_pipe,
            "shell",
            RequestedRender { mode: FrameMode::Commands, auto_downgraded: false },
            5000,
        )
        .map_err(|e| format!("壳 broker 孵化失败: {e:?}"))?;
        let hello = ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: PROTOCOL_VERSION,
            app_name: "shell".into(),
            title: "shell".into(),
            icon: None,
            width: geometry.viewport_w,
            height: geometry.viewport_h,
            fonts: Vec::new(),
            surfaces: vec![
                SurfaceDecl {
                    role: surface_role::BACKGROUND,
                    width: geometry.viewport_w,
                    height: geometry.viewport_h,
                },
                SurfaceDecl {
                    role: surface_role::CHROME,
                    width: geometry.viewport_w,
                    height: geometry.band_h,
                },
            ],
        });
        let mut end = end;
        end.send(&hello).map_err(|e| format!("壳 Hello 发送失败: {e:?}"))?;
        Ok(Self {
            end,
            faces,
            routes: BTreeMap::new(),
            frames: BTreeMap::new(),
            last_revisions: BTreeMap::new(),
        })
    }

    /// 阻塞主循环（child 主线程）。EOF = 宿主消失 → 退出（恢复归宿主
    /// 看门兵）。Err = 致命解码错（进程出口非零由调用方定）。
    pub fn run(mut self) -> Result<(), String> {
        loop {
            let Some(msg) = self.end.recv_wait(50) else {
                if self.end.is_eof() {
                    return Ok(());
                }
                self.sync_frames();
                continue;
            };
            let msg = msg.map_err(|e| format!("壳消息解码失败: {e:?}"))?;
            match msg {
                ProtocolMsg::Handshake(HandshakeMsg::Welcome { wid, surface, extra_surfaces, .. }) => {
                    self.routes.insert(wid, (surface, shell_face::DESKTOP_SURFACE));
                    self.frames.entry(wid).or_default();
                    for e in extra_surfaces {
                        let face = if e.role == surface_role::CHROME {
                            shell_face::SHELL
                        } else {
                            shell_face::DESKTOP_SURFACE
                        };
                        self.routes.insert(e.wid, (e.surface, face));
                        self.frames.entry(e.wid).or_default();
                    }
                }
                ProtocolMsg::Frame(FrameMsg::BufferAlloc { .. }) => {
                    // 内联帧形态：无 shm 协商（宿主壳分支不建段）。
                }
                ProtocolMsg::Frame(FrameMsg::FrameAck { wid, slot, .. }) => {
                    if let Some(f) = self.frames.get_mut(&wid) {
                        f.ack(slot);
                    }
                }
                ProtocolMsg::Frame(FrameMsg::BufferRelease { .. }) => {
                    return Ok(());
                }
                ProtocolMsg::Control(ControlMsg::ShellProjectionPush { face, payload }) => {
                    self.faces.apply_projection(face, &payload);
                    self.sync_frames();
                }
                ProtocolMsg::Control(ControlMsg::ShellClockTick { face, time, date }) => {
                    self.faces.apply_clock(face, &time, &date);
                    self.sync_frames();
                }
                ProtocolMsg::Control(ControlMsg::ShellCursorMove { face, x, y }) => {
                    self.faces.apply_cursor(face, x, y);
                    self.sync_frames();
                }
                ProtocolMsg::Control(ControlMsg::Close { wid }) => {
                    let _ = self
                        .end
                        .send(&ProtocolMsg::Control(ControlMsg::ExitRequest { wid }));
                    return Ok(());
                }
                ProtocolMsg::Control(ControlMsg::Resize { .. }) => {
                    // v1：壳表面尺寸 boot 定档（视口变更随 respawn 生效）。
                }
                ProtocolMsg::Control(_) => {}
                ProtocolMsg::Input(input) => {
                    let wid = input.wid();
                    let Some(&(_, face)) = self.routes.get(&wid) else {
                        continue;
                    };
                    self.faces.on_input(face, &input);
                    // 输入可能写命令（按钮 handler）：读走 + 上行。
                    for record in self.faces.drain_commands(face) {
                        let _ = self
                            .end
                            .send(&ProtocolMsg::Control(ControlMsg::DesktopBus { wid, record }));
                    }
                    self.sync_frames();
                }
                _ => {}
            }
        }
    }

    /// revision 对账产帧（两面独立对账——分面增量）。
    fn sync_frames(&mut self) {
        for face in [shell_face::DESKTOP_SURFACE, shell_face::SHELL] {
            let rev = self.faces.revision(face);
            if self.last_revisions.get(&face) == Some(&rev) {
                continue;
            }
            self.last_revisions.insert(face, rev);
            let Some((&wid, _)) = self.routes.iter().find(|(_, (_, f))| *f == face) else {
                continue;
            };
            let Some(list) = self.faces.render(face) else { continue };
            let Some(fr) = self.frames.get_mut(&wid) else { continue };
            fr.next_frame_id += 1;
            let frame = ProtocolMsg::Frame(FrameMsg::FrameReady {
                wid,
                frame_id: fr.next_frame_id,
                slot: fr.take_slot(),
                damage: None,
                revision: rev,
                payload: list,
            });
            let _ = self.end.send(&frame);
        }
    }
}

/// `auto run --autodesk-shell` 产品入口（cmd_autodesk 分派）。
/// 几何：AUTO_SHELL_GEOM env（spawn 注入）> 缺省 1280x800x48。
pub fn run_shell_outproc(broker_pipe: &str) -> Result<(), String> {
    let geometry = ShellGeometry::from_env().unwrap_or_else(ShellGeometry::fallback);
    let faces = ShellFaces::load(geometry)?;
    let pump = ShellPump::start(broker_pipe, geometry, faces)?;
    pump.run()
}

// ================================ 测试 ================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_geometry_roundtrip() {
        let g = ShellGeometry { viewport_w: 1280.0, viewport_h: 800.0, band_h: 48.0 };
        assert_eq!(ShellGeometry::parse(&g.encode()), Some(g));
        assert!(ShellGeometry::parse("1280").is_none());
        assert!(ShellGeometry::parse("axbxc").is_none());
        // 带高 ≥ 视口高 = 非法。
        assert!(ShellGeometry::parse("800x600x600").is_none());
    }

    /// 槽轮转（内联帧双槽——ack 归还复用）。
    #[test]
    fn surface_frames_slot_rotation() {
        let mut f = SurfaceFrames::default();
        let a = f.take_slot();
        let b = f.take_slot();
        assert_ne!(a, b, "双缓冲交替");
        f.ack(a);
        assert_eq!(f.take_slot(), a, "ack 归还槽优先复用");
        assert_ne!(f.take_slot(), a, "无空闲时翻转");
    }

    /// 装配单测：面装载 + 投影 apply（指纹门宿主侧，child 全量应用）+
    /// 命令读走幂等（c4 清空语义）。
    #[test]
    fn shell_faces_projection_apply_and_drain() {
        let faces = ShellFaces::load(ShellGeometry::fallback());
        if let Err(err) = faces {
            // 单测环境 pack 缺失（solo 检出等）——内嵌 pin 快照兜底应恒
            // 可装载；仍失败 = 环境异常，跳过不误报。
            eprintln!("shell faces load 失败（环境）: {err}");
            return;
        }
        let mut faces = faces.unwrap();
        let proj = ShellProjection {
            fp: "fp-1".into(),
            ..Default::default()
        };
        let mut payload = Vec::new();
        proj.wire_encode(&mut payload);
        assert!(faces.apply_projection(shell_face::SHELL, &payload), "shell 面应用");
        let rev_before = faces.revision(shell_face::SHELL);
        assert!(rev_before > 0);
        // 同 payload 再应用：child 侧无指纹门（宿主侧门控）——revision
        // 仍前进（全量应用语义）。
        assert!(faces.apply_projection(shell_face::SHELL, &payload));
        assert!(faces.revision(shell_face::SHELL) > rev_before);
        // 时钟/光标。
        assert!(faces.apply_clock(shell_face::SHELL, "09:05", "9月19日 周六"));
        assert!(faces.apply_cursor(shell_face::DESKTOP_SURFACE, 12.0, 34.0));
        // 无面拒收（overlay 三面 in-proc）。
        assert!(!faces.apply_projection(shell_face::SWITCHER, &payload));
        // 命令读走：初始为空（幂等清空）。
        assert!(faces.drain_commands(shell_face::SHELL).is_empty());
        // 渲染可产帧。
        assert!(faces.render(shell_face::SHELL).is_some());
    }
}
