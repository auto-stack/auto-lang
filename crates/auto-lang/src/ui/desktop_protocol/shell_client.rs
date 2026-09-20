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
use crate::ui::desktop_protocol::native_projector::RqProjector;
use crate::ui::desktop_protocol::codec::Reader;
use crate::ui::desktop_protocol::endpoint::FrameSource;
use crate::ui::desktop_protocol::message::{
    shell_face, surface_role, ControlMsg, FrameMsg, FrameMode, HandshakeMsg, InputMsg,
    ProtocolMsg, SurfaceDecl,
};
use crate::ui::desktop_protocol::transport;
use crate::ui::desktop_protocol::PROTOCOL_VERSION;
use crate::ui::shell_projection::{
    shell_event_name, DashboardSnapshot, DesktopSurfaceSnapshot, LauncherSnapshot, NotesSnapshot,
    ShellProjection, ShellWrite, SwitcherSnapshot,
};
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

/// 编译组件的外部写态面（PLAN-036 D8——codegen 生成实现；解释臂
/// `DynamicComponent` 桥接既有 write_state/call_handler，两臂同体）。
pub trait ShellStateAccess {
    /// 标量写（`__wm_*`/`__desktop_*` 家族；返回 false = 组件无此键）。
    fn shell_write(&mut self, key: &str, value: auto_val::Value) -> bool;
    /// 数组写（平行列表家族）。
    fn shell_write_vec(&mut self, key: &str, values: Vec<auto_val::Value>) -> bool;
    /// 召唤事件（宿主写状态不触发 handler——随快照显式携带，027 语义；
    /// 编译组件 = 事件名 → Msg variant `on()` 派发）。
    fn shell_dispatch(&mut self, event: &str);
    /// 读字符串态（`__desktop_cmd` 读走面）。
    fn shell_read_str(&self, key: &str) -> Option<String>;
    /// 键盘 bind 路由（PLAN-036 T-07 D5——规范键名 → 组件 Msg 派发；
    /// 返回 false = 组件无此 bind。解释臂 = key_bindings 表 →
    /// call_handler；编译臂 = codegen key_message 直派[launcher 记债]）。
    fn shell_key(&mut self, _key: &str) -> bool {
        false
    }
}

/// 壳面装配接缝（PLAN-036 D8）：`ShellFaces` 消费面的 trait 化——
/// 解释臂（`FaceProjector<DynamicComponent>`）与编译臂（a2r 生成组件
/// 经 `FaceProjector<C>`）同接口互换（SC:10-14 替换点注记的落地）。
pub trait ShellSurface {
    /// 投影 writes 应用（`interpreted_writes()` lowering 落点——叶面
    /// 保形：bool → "1"/"" 已在宿主侧 lowering 完成，wire 上是 bool）。
    fn apply_writes(&mut self, writes: Vec<ShellWrite>);
    /// 召唤事件（`ShellEvent` 名——child 侧消费单点）。
    fn dispatch_event(&mut self, name: &str);
    /// 标量写（时钟/光标低频面）。
    fn write_scalar(&mut self, key: &str, value: &str);
    /// 读字符串态（`__desktop_cmd` 读走——c4 语义：读 + 清空配对）。
    fn read_state_str(&self, key: &str) -> Option<String>;
    /// 清字符串态（读走配对清空）。
    fn clear_state_str(&mut self, key: &str);
    /// 输入路由（坐标已由宿主平移为面局部系）。
    fn on_input(&mut self, input: &InputMsg);
    /// 渲染一面（queue 臂 DrawList）。
    fn render_frame(&mut self) -> Option<crate::ui::desktop_protocol::message::DrawList>;
    fn revision(&self) -> u64;
    fn bump_revision(&mut self);
    /// 面命中区矩形快照（e2e 点击钩子消费）。
    fn hit_rects(&self) -> Vec<crate::ui::desktop_protocol::message::WRect>;
    /// 键盘 bind 路由（PLAN-036 T-07 D5：VK → 规范名 → 组件 bind 表
    /// 派发；返回 true = 消费[revision 由调用方前进]）。
    fn dispatch_key(&mut self, vk: u32, modifiers: u8) -> bool;
    /// 自主聚焦首输入槽（D5：`__focus_input` 的 child 等价）。
    fn focus_input(&mut self) -> bool;
}

/// Windows VK → .at bind 规范键名（launcher bind 表词汇：方向/Enter/
/// Tab/Space/Esc；Ctrl 组合键宿主热键表保留[D5 边界]——裸键Only）。
pub fn vk_to_bind_name(vk: u32) -> Option<&'static str> {
    Some(match vk {
        0x25 => "ArrowLeft",
        0x26 => "ArrowUp",
        0x27 => "ArrowRight",
        0x28 => "ArrowDown",
        0x0D => "Enter",
        0x09 => "Tab",
        0x20 => " ",
        0x1B => "Escape",
        _ => return None,
    })
}

/// 泛型面装配（D8）：`RqProjector<C>` + `ShellStateAccess` →
/// `ShellSurface`——投影器提供渲染/命中/输入/revision，状态访问 trait
/// 提供写态/事件/读走（解释/编译两臂的同一适配体）。
pub struct FaceProjector<C: crate::ui::component::Component + ShellStateAccess> {
    inner: RqProjector<C>,
}

impl<C: crate::ui::component::Component + ShellStateAccess> FaceProjector<C> {
    pub fn new(component: C, width: f32, height: f32) -> Self {
        Self { inner: RqProjector::new(component, width, height) }
    }

    /// 029 覆盖门（装载期过门——027 五件 Covered 前提）。
    pub fn ensure_covered(&self) -> Result<(), String> {
        self.inner.ensure_covered()
    }

    pub fn component_mut(&mut self) -> &mut C {
        self.inner.component_mut()
    }
}

impl<C: crate::ui::component::Component + ShellStateAccess> ShellSurface
    for FaceProjector<C>
{
    fn apply_writes(&mut self, writes: Vec<ShellWrite>) {
        for w in writes {
            match w {
                ShellWrite::Scalar(k, v) => {
                    let _ = self.inner.component_mut().shell_write(&k, v);
                }
                ShellWrite::Array(k, vs) => {
                    let _ = self.inner.component_mut().shell_write_vec(&k, vs);
                }
            }
        }
    }

    fn dispatch_event(&mut self, name: &str) {
        self.inner.component_mut().shell_dispatch(name);
    }

    fn write_scalar(&mut self, key: &str, value: &str) {
        let _ = self
            .inner
            .component_mut()
            .shell_write(key, auto_val::Value::str(value));
    }

    fn read_state_str(&self, key: &str) -> Option<String> {
        self.inner.component().shell_read_str(key)
    }

    fn clear_state_str(&mut self, key: &str) {
        let _ = self
            .inner
            .component_mut()
            .shell_write(key, auto_val::Value::str(""));
    }

    fn on_input(&mut self, input: &InputMsg) {
        self.inner.on_input(input);
    }

    fn render_frame(&mut self) -> Option<crate::ui::desktop_protocol::message::DrawList> {
        Some(self.inner.render_frame())
    }

    fn revision(&self) -> u64 {
        self.inner.revision()
    }

    fn bump_revision(&mut self) {
        self.inner.bump_revision();
    }

    fn hit_rects(&self) -> Vec<crate::ui::desktop_protocol::message::WRect> {
        self.inner.hit_rects()
    }

    fn dispatch_key(&mut self, vk: u32, modifiers: u8) -> bool {
        let Some(name) = vk_to_bind_name(vk) else { return false };
        if modifiers != 0 {
            // Ctrl/Shift/Alt 组合 = 宿主热键域（D5 边界）——child 不消费。
            return false;
        }
        self.inner.component_mut().shell_key(name)
    }

    fn focus_input(&mut self) -> bool {
        self.inner.focus_first_input()
    }
}

impl ShellStateAccess for crate::ui::dynamic::DynamicComponent {
    fn shell_write(&mut self, key: &str, value: auto_val::Value) -> bool {
        self.write_state(key, value).is_ok()
    }

    fn shell_write_vec(&mut self, key: &str, values: Vec<auto_val::Value>) -> bool {
        self.write_state_vec(key, values).is_ok()
    }

    fn shell_dispatch(&mut self, event: &str) {
        let _ = self.bridge_mut().call_handler(event, &[]);
    }

    fn shell_read_str(&self, key: &str) -> Option<String> {
        match self.read_state(key) {
            Ok(auto_val::Value::Str(s)) => Some(s.as_str().to_string()),
            _ => None,
        }
    }

    fn shell_key(&mut self, key: &str) -> bool {
        // D5：解释臂键盘 bind 路由——.at bind 表（规范键名 → 事件名）
        // 命中即 handler 派发（宿主窗 key_message 消费面的 child 等价）。
        let event = self.key_bindings().get(key).cloned();
        match event {
            Some(name) => {
                let _ = self.bridge_mut().call_handler(&name, &[]);
                true
            }
            None => false,
        }
    }
}

/// 壳面会话（face → ShellSurface；解释装载/编译组件两轨的装配
/// 面——PLAN-036 D8 替换点；T-04 扩 overlay 懒装：解释轨首推送时
/// 装载，编译轨装配点五面预给）。
pub struct ShellFaces {
    /// 已装面（shell_face 值 → 装配体；常驻两面 boot 装，overlay
    /// 懒装/预装）。
    faces: BTreeMap<u8, Box<dyn ShellSurface>>,
    geometry: ShellGeometry,
}

impl ShellFaces {
    /// 解释装载 v1 双常驻面（027 SHELL_MANIFEST ResidentBoot 两件）+
    /// ensure_covered 过门（029 五件 Covered 前提——AC-02 证据）。
    /// overlay 两面（switcher/notification）懒装——首次投影推送时
    /// [`Self::ensure_overlay`]（PLAN-036 T-04）。
    pub fn load(geometry: ShellGeometry) -> Result<Self, String> {
        let chrome_src = crate::ui::shell::shell_source("shell.at");
        let bg_src = crate::ui::shell::shell_source("desktop.at");
        let chrome = crate::build_dynamic_component(&chrome_src, None)
            .map_err(|e| format!("壳 chrome 面装载失败: {e}"))?;
        let background = crate::build_dynamic_component(&bg_src, None)
            .map_err(|e| format!("壳 background 面装载失败: {e}"))?;
        let mut chrome =
            FaceProjector::new(chrome, geometry.viewport_w, geometry.band_h);
        let mut background =
            FaceProjector::new(background, geometry.viewport_w, geometry.viewport_h);
        if let Err(gate) = chrome.ensure_covered() {
            return Err(format!("壳 chrome 面 {gate}"));
        }
        if let Err(gate) = background.ensure_covered() {
            return Err(format!("壳 background 面 {gate}"));
        }
        let mut faces = BTreeMap::new();
        faces.insert(shell_face::SHELL, Box::new(chrome) as Box<dyn ShellSurface>);
        faces.insert(shell_face::DESKTOP_SURFACE, Box::new(background) as Box<dyn ShellSurface>);
        Ok(Self { faces, geometry })
    }

    /// 编译装配（PLAN-036 D8）：外部供面（a2r 生成 crate 的
    /// `mount_face` 工厂产物——crates/auto cmd_autodesk 装配点注入；
    /// T-04 起五面全给，overlay 预装免懒装）。面已由工厂
    /// ensure_covered 过门。
    pub fn from_faces(
        geometry: ShellGeometry,
        faces: Vec<(u8, Box<dyn ShellSurface>)>,
    ) -> Self {
        let mut map = BTreeMap::new();
        for (face, surface) in faces {
            map.insert(face, surface);
        }
        Self { faces: map, geometry }
    }

    /// overlay 面懒装（解释轨；已装 = 幂等 true）。编译轨 overlay 由
    /// `from_faces` 预装——本方法对缺席面返回 false（推送拒收留痕）。
    fn ensure_overlay(&mut self, face: u8) -> bool {
        if self.faces.contains_key(&face) {
            return true;
        }
        let (src, w, h) = match face {
            shell_face::SWITCHER => (
                crate::ui::shell::shell_source("switcher.at"),
                self.geometry.viewport_w,
                self.geometry.viewport_h,
            ),
            shell_face::NOTIFICATION_CENTER => (
                crate::ui::shell::shell_source("notification_center.at"),
                self.geometry.viewport_w,
                self.geometry.viewport_h,
            ),
            shell_face::DASHBOARD => (
                crate::ui::shell::shell_source("dashboard.at"),
                self.geometry.viewport_w,
                self.geometry.viewport_h,
            ),
            // PLAN-036 T-07（D3-C）：launcher 源 = 注册表条目（宿主 spawn
            // 注入 AUTO_LAUNCHER_ENTRY 路径 env——非 pack 源；缺席 = 报
            /// 错退出，宿主 respawn 链兜底）。
            shell_face::LAUNCHER => {
                let Some(entry) = std::env::var_os("AUTO_LAUNCHER_ENTRY") else {
                    eprintln!("[p036-child] launcher 面：AUTO_LAUNCHER_ENTRY 缺席");
                    return false;
                };
                match std::fs::read_to_string(&entry) {
                    Ok(src) => (
                        std::borrow::Cow::Owned(src),
                        self.geometry.viewport_w,
                        self.geometry.viewport_h,
                    ),
                    Err(e) => {
                        eprintln!(
                            "[p036-child] launcher 面注册表源读取失败（{}）: {e}",
                            entry.to_string_lossy()
                        );
                        return false;
                    }
                }
            }
            _ => return false,
        };
        let comp = match crate::build_dynamic_component(src.as_ref(), None) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[p036-child] overlay 面 {face} 解释装载失败: {e}");
                return false;
            }
        };
        let mut p = FaceProjector::new(comp, w, h);
        if let Err(gate) = p.ensure_covered() {
            eprintln!("[p036-child] overlay 面 {face} 覆盖门拒收: {gate}");
            return false;
        }
        p.write_scalar("hosted", "1");
        self.faces.insert(face, Box::new(p));
        true
    }

    pub fn geometry(&self) -> ShellGeometry {
        self.geometry
    }

    /// PLAN-036 T-07：空会话构造（launcher 单面 exe 入口）。
    pub fn empty(geometry: ShellGeometry) -> Self {
        Self { faces: BTreeMap::new(), geometry }
    }

    /// 公共装载面（ensure_overlay 包装——launcher 入口单面装配）。
    pub fn mount_overlay(&mut self, face: u8) -> bool {
        self.ensure_overlay(face)
    }

    fn projector_mut(&mut self, face: u8) -> Option<&mut dyn ShellSurface> {
        match self.faces.get_mut(&face) {
            Some(b) => Some(&mut **b),
            None => None,
        }
    }

    /// 投影下行消费：typed 载体 wire 解码 → interpreted_writes child 侧
    /// lowering（叶面保形单点：bool → "1"/"" 在 lowering，wire 上是 bool）。
    /// 返回 true = 状态变化（revision 前进）。
    /// PLAN-036 T-04：SWITCHER/NOTIFICATION_CENTER 两臂激活（懒装 +
    /// 写集 + 召唤/键盘事件派发——D6）。
    pub fn apply_projection(&mut self, face: u8, payload: &[u8]) -> bool {
        match face {
            shell_face::SHELL => {
                let mut r = Reader::new(payload);
                let Ok(proj) = ShellProjection::wire_decode(&mut r) else {
                    return false;
                };
                let Some(p) = self.projector_mut(face) else { return false };
                p.apply_writes(proj.interpreted_writes());
                if std::env::var("AUTO030_TRACE").is_ok() {
                    eprintln!("[p030-child] applied shell proj: proj.wins={}", proj.wins.len());
                }
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
                p.apply_writes(snap.interpreted_writes());
                for e in &events {
                    p.dispatch_event(shell_event_name(e));
                }
                p.bump_revision();
                true
            }
            shell_face::SWITCHER => {
                let mut r = Reader::new(payload);
                let Ok(snap) = SwitcherSnapshot::wire_decode(&mut r) else {
                    return false;
                };
                let events = snap.events.clone();
                if !self.ensure_overlay(face) {
                    return false;
                }
                let Some(p) = self.projector_mut(face) else { return false };
                p.apply_writes(snap.interpreted_writes());
                for e in &events {
                    p.dispatch_event(shell_event_name(e));
                }
                p.bump_revision();
                true
            }
            shell_face::NOTIFICATION_CENTER => {
                let mut r = Reader::new(payload);
                let Ok(snap) = NotesSnapshot::wire_decode(&mut r) else {
                    return false;
                };
                let events = snap.events.clone();
                if !self.ensure_overlay(face) {
                    return false;
                }
                let Some(p) = self.projector_mut(face) else { return false };
                p.apply_writes(snap.interpreted_writes());
                for e in &events {
                    p.dispatch_event(shell_event_name(e));
                }
                p.bump_revision();
                true
            }
            shell_face::LAUNCHER => {
                let mut r = Reader::new(payload);
                let Ok(snap) = LauncherSnapshot::wire_decode(&mut r) else {
                    return false;
                };
                let events = snap.events.clone();
                if !self.ensure_overlay(face) {
                    return false;
                }
                let Some(p) = self.projector_mut(face) else { return false };
                p.apply_writes(snap.interpreted_writes());
                for e in &events {
                    p.dispatch_event(shell_event_name(e));
                }
                // D5：ApplyFilter 后自主聚焦首输入（__focus_input 等价）。
                if events.iter().any(|e| matches!(e, crate::ui::shell_projection::ShellEvent::ApplyFilter)) && snap.visible {
                    p.focus_input();
                }
                p.bump_revision();
                true
            }
            shell_face::DASHBOARD => {
                let mut r = Reader::new(payload);
                let Ok(snap) = DashboardSnapshot::wire_decode(&mut r) else {
                    return false;
                };
                let events = snap.events.clone();
                if !self.ensure_overlay(face) {
                    return false;
                }
                let Some(p) = self.projector_mut(face) else { return false };
                p.apply_writes(snap.interpreted_writes());
                for e in &events {
                    p.dispatch_event(shell_event_name(e));
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
        p.write_scalar("__wm_clock", time);
        p.write_scalar("__wm_date", date);
        p.bump_revision();
        true
    }

    /// 光标事件（空白菜单坐标锚——background 面消费；逐事件语义——
    /// 宿主侧消费门控制推送节拍[D3 定案]）。
    pub fn apply_cursor(&mut self, face: u8, x: f32, y: f32) -> bool {
        let Some(p) = self.projector_mut(face) else { return false };
        p.write_scalar("__desktop_cursor_x", &x.to_string());
        p.write_scalar("__desktop_cursor_y", &y.to_string());
        p.bump_revision();
        true
    }

    /// 输入路由（wid → face 投影器；坐标已由宿主平移为面局部系）。
    /// PLAN-036 T-07（D5）：KeyPressed 先经面 bind 路由（VK → 组件
    /// Msg——方向/Enter/Tab/Esc），未消费再落投影器既有键处理（退格/
    /// select Esc）；消费即 revision 前进（渲染随对账泵刷新）。
    pub fn on_input(&mut self, face: u8, input: &InputMsg) {
        if let Some(p) = self.projector_mut(face) {
            if let InputMsg::KeyPressed { key, modifiers, .. } = input {
                if p.dispatch_key(*key, *modifiers) {
                    p.bump_revision();
                    return;
                }
            }
            p.on_input(input);
        }
    }

    /// 命令读走（c4 语义 child 化：read_state + 清空；面局部 `__desktop_cmd`）。
    pub fn drain_commands(&mut self, face: u8) -> Vec<String> {
        let Some(p) = self.projector_mut(face) else { return Vec::new() };
        let cur = p.read_state_str("__desktop_cmd").unwrap_or_default();
        if cur.is_empty() {
            return Vec::new();
        }
        p.clear_state_str("__desktop_cmd");
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
        self.projector_mut(face).and_then(|p| p.render_frame())
    }

    pub fn revision(&self, face: u8) -> u64 {
        self.faces.get(&face).map(|p| p.revision()).unwrap_or(0)
    }

    /// 面命中区矩形快照（e2e 点击钩子消费——宿主 pointer 路由的等价载荷）。
    pub fn hit_rects(&self, face: u8) -> Vec<crate::ui::desktop_protocol::message::WRect> {
        self.faces.get(&face).map(|p| p.hit_rects()).unwrap_or_default()
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
    /// 投影应用后钩子（e2e 点击注入——真按钮命中区坐标自注入，宿主
    /// pointer 路由的等价载荷；生产 None）。
    on_applied: Option<Box<dyn FnMut(&mut ShellFaces) + Send>>,
    /// wid → (surface, face)——Welcome 协商结果。
    routes: BTreeMap<u64, (u64, u8)>,
    frames: BTreeMap<u64, SurfaceFrames>,
    last_revisions: BTreeMap<u8, u64>,
    /// PLAN-036 T-07：Welcome 领头面（shell=DESKTOP_SURFACE /
    /// launcher=LAUNCHER）。
    leading_face: u8,
}

impl ShellPump {
    /// 经 broker 孵化 + 双表面握手（Hello 尾段：background 领头声明 +
    /// chrome 尾段——宿主 ResolveAndAttach 壳分支同序消费）。
    pub fn start(
        broker_pipe: &str,
        geometry: ShellGeometry,
        faces: ShellFaces,
    ) -> Result<Self, String> {
        Self::start_as(broker_pipe, "shell", geometry, faces, shell_surfaces(&geometry))
    }

    /// PLAN-036 T-07（D3-C）：泛化入口——launcher 独立 exe 单面握手
    ///（app_name="launcher"、单 OVERLAY 声明、leading face=LAUNCHER——
    /// 宿主 attach launcher 分支同序消费）。
    #[allow(clippy::too_many_arguments)]
    pub fn start_as(
        broker_pipe: &str,
        app_name: &str,
        geometry: ShellGeometry,
        faces: ShellFaces,
        surfaces: Vec<SurfaceDecl>,
    ) -> Result<Self, String> {
        let leading_face = if app_name == "launcher" {
            shell_face::LAUNCHER
        } else {
            shell_face::DESKTOP_SURFACE
        };
        let geometry_ref = geometry;
        let (_, end) = broker::request_incubation_render(
            broker_pipe,
            "shell",
            RequestedRender { mode: FrameMode::Commands, auto_downgraded: false },
            5000,
        )
        .map_err(|e| format!("壳 broker 孵化失败: {e:?}"))?;
        let hello = ProtocolMsg::Handshake(HandshakeMsg::Hello {
            version: PROTOCOL_VERSION,
            app_name: app_name.to_string(),
            title: app_name.to_string(),
            icon: None,
            width: geometry_ref.viewport_w,
            height: geometry_ref.viewport_h,
            fonts: Vec::new(),
            surfaces,
        });
        let mut end = end;
        end.send(&hello).map_err(|e| format!("壳 Hello 发送失败: {e:?}"))?;
        Ok(Self {
            end,
            faces,
            on_applied: None,
            routes: BTreeMap::new(),
            frames: BTreeMap::new(),
            last_revisions: BTreeMap::new(),
            leading_face,
        })
    }

    /// 注册投影应用钩子（e2e 专用——builder）。
    pub fn with_on_applied(
        mut self,
        f: Box<dyn FnMut(&mut ShellFaces) + Send>,
    ) -> Self {
        self.on_applied = Some(f);
        self
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
                    self.routes.insert(wid, (surface, self.leading_face));
                    self.frames.entry(wid).or_default();
                    // PLAN-036 T-04：OVERLAY 依序映射（Hello 声明序约定
                    ///——首 OVERLAY=switcher、次=notification_center）。
                    let mut overlay_idx = 0usize;
                    for e in extra_surfaces {
                        let face = if e.role == surface_role::CHROME {
                            shell_face::SHELL
                        } else if e.role == surface_role::DASHBOARD {
                            shell_face::DASHBOARD
                        } else if e.role == surface_role::OVERLAY {
                            overlay_idx += 1;
                            match overlay_idx {
                                1 => shell_face::SWITCHER,
                                2 => shell_face::NOTIFICATION_CENTER,
                                _ => shell_face::DASHBOARD,
                            }
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
                    if self.faces.apply_projection(face, &payload) {
                        if let Some(hook) = self.on_applied.as_mut() {
                            hook(&mut self.faces);
                        }
                        // PLAN-036 T-04（D6）：投影携带事件（Pick/Escape 等）
                        /// 可产命令（SendCmd 总线写点）——与 wire 输入同路：
                        /// 命令读走 + DesktopBus 上行（e2e 钩子与生产同径）。
                        for record in self.faces.drain_commands(face) {
                            let wid = self
                                .routes
                                .iter()
                                .find(|(_, (_, f))| *f == face)
                                .map(|(w, _)| *w)
                                .unwrap_or(0);
                            let _ = self.end.send(
                                &ProtocolMsg::Control(ControlMsg::DesktopBus { wid, record }),
                            );
                        }
                    }
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

    /// revision 对账产帧（已装面独立对账——分面增量；PLAN-036 T-04：
    /// overlay 懒装面随装随对账）。
    fn sync_frames(&mut self) {
        let mounted: Vec<u8> = self.faces.faces.keys().copied().collect();
        for face in mounted {
            let rev = self.faces.revision(face);
            if self.last_revisions.get(&face) == Some(&rev) {
                continue;
            }
            self.last_revisions.insert(face, rev);
            let Some((&wid, _)) = self.routes.iter().find(|(_, (_, f))| *f == face) else {
                continue;
            };
            let Some(list) = self.faces.render(face) else {
                if std::env::var("AUTO030_TRACE").is_ok() {
                    eprintln!("[p030-child] sync_frames face={face} render-none");
                }
                continue;
            };
            let Some(fr) = self.frames.get_mut(&wid) else {
                if std::env::var("AUTO030_TRACE").is_ok() {
                    eprintln!("[p030-child] sync_frames face={face} wid={wid} no-frames");
                }
                continue;
            };
            fr.next_frame_id += 1;
            let frame = ProtocolMsg::Frame(FrameMsg::FrameReady {
                wid,
                frame_id: fr.next_frame_id,
                slot: fr.take_slot(),
                damage: None,
                revision: rev,
                payload: list,
            });
            if std::env::var("AUTO030_TRACE").is_ok() {
                eprintln!("[p030-child] send frame face={face} wid={wid} rev={rev} ops={}", frame_ops(&frame));
            }
            if self.end.send(&frame).is_err() {
                eprintln!("[p030-child] frame send FAILED face={face} wid={wid}");
            }
        }
    }
}

#[cfg(feature = "ui-iced")]
fn frame_ops(frame: &ProtocolMsg) -> usize {
    match frame {
        ProtocolMsg::Frame(FrameMsg::FrameReady { payload, .. }) => payload.ops.len(),
        _ => 0,
    }
}

/// 壳五面 Hello 声明（background 领头 + chrome + overlay×2 + dashboard
/// ——T-04/T-05 契约原样）。
fn shell_surfaces(geometry: &ShellGeometry) -> Vec<SurfaceDecl> {
    vec![
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
        // PLAN-036 T-04：overlay 两面全屏声明（OVERLAY 档——面区分按
        // 声明序 0=switcher 1=notification_center，Welcome 路由同序约定）。
        SurfaceDecl {
            role: surface_role::OVERLAY,
            width: geometry.viewport_w,
            height: geometry.viewport_h,
        },
        SurfaceDecl {
            role: surface_role::OVERLAY,
            width: geometry.viewport_w,
            height: geometry.viewport_h,
        },
        // PLAN-036 T-05（D2-A）：dashboard 面——中间 z 档显式声明（表面
        // 尺寸 = 035 固定外框 696×232）。
        SurfaceDecl {
            role: surface_role::DASHBOARD,
            width: 696.0,
            height: 232.0,
        },
    ]
}

/// `auto run --autodesk-launcher` 产品入口（PLAN-036 T-07 D3-C：launcher
/// 一面一 exe——独立进程单 OVERLAY 面；源 = 注册表条目 env
/// `AUTO_LAUNCHER_ENTRY`（宿主 spawn 注入），v1 解释装载（编译轨
/// launcher exe 记 P036 债）。
pub fn run_launcher_outproc(broker_pipe: &str) -> Result<(), String> {
    let geometry = ShellGeometry::from_env().unwrap_or_else(ShellGeometry::fallback);
    let mut faces = ShellFaces::empty(geometry);
    if !faces.mount_overlay(shell_face::LAUNCHER) {
        return Err("launcher 面装载失败（AUTO_LAUNCHER_ENTRY 源/覆盖门）".into());
    }
    let surfaces = vec![SurfaceDecl {
        role: surface_role::OVERLAY,
        width: geometry.viewport_w,
        height: geometry.viewport_h,
    }];
    let pump = ShellPump::start_as(broker_pipe, "launcher", geometry, faces, surfaces)?;
    pump.run()
}

/// `auto run --autodesk-shell` 产品入口（cmd_autodesk 分派）。
/// 几何：AUTO_SHELL_GEOM env（spawn 注入）> 缺省 1280x800x48。
/// 解释装载缺省（开发态——显式 AUTO_SHELL_PACK 路径；编译轨经
/// `run_shell_outproc_with` 由 cmd_autodesk 装配，PLAN-036 D4）。
pub fn run_shell_outproc(broker_pipe: &str) -> Result<(), String> {
    run_shell_outproc_with(broker_pipe, ShellFaces::load)
}

/// 供面工厂形态（PLAN-036 D8）：编译轨装配点（crates/auto
/// cmd_autodesk——shell_pack::mount_face 工厂注入；auto-lang 不引
/// 生成 crate，无环）。
pub fn run_shell_outproc_with(
    broker_pipe: &str,
    faces_factory: impl FnOnce(ShellGeometry) -> Result<ShellFaces, String>,
) -> Result<(), String> {
    let geometry = ShellGeometry::from_env().unwrap_or_else(ShellGeometry::fallback);
    let faces = faces_factory(geometry)?;
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

    /// PLAN-036 T-04（B1）：overlay 两面投影应用——解释轨懒装载路径
    ///（load 只装双常驻；首 SWITCHER/NOTIFICATION_CENTER 投影触发
    /// 懒装 + 写集应用 + 事件派发；solo 检出兜底内嵌 pin 快照）。
    #[test]
    fn shell_faces_overlay_lazy_mount_apply() {
        use crate::ui::shell_projection::{ShellEvent, SwitcherSnapshot};
        let Ok(mut faces) = ShellFaces::load(ShellGeometry::fallback()) else {
            eprintln!("shell faces load 失败（环境）");
            return;
        };
        let sw = SwitcherSnapshot {
            hosted: true,
            visible: true,
            mru_wids: vec!["3".into()],
            mru_titles: vec!["A".into()],
            mru_icons: vec!["app-window".into()],
            mru_thumbs: vec!["".into()],
            wm_mru: vec![],
            events: vec![ShellEvent::RebuildMru, ShellEvent::Advance],
        };
        let mut payload = Vec::new();
        sw.wire_encode(&mut payload);
        assert!(
            faces.apply_projection(shell_face::SWITCHER, &payload),
            "switcher 懒装 + 应用 + 事件派发"
        );
        assert!(faces.revision(shell_face::SWITCHER) > 0);
        assert!(faces.render(shell_face::SWITCHER).is_some(), "懒装面渲染");
        let notes = crate::ui::shell_projection::NotesSnapshot {
            hosted: true,
            visible: true,
            ..Default::default()
        };
        let mut np = Vec::new();
        notes.wire_encode(&mut np);
        assert!(
            faces.apply_projection(shell_face::NOTIFICATION_CENTER, &np),
            "notes 懒装 + 应用"
        );
        // 坏 payload 拒收不炸。
        assert!(!faces.apply_projection(shell_face::SWITCHER, &[0xFF, 0xFF]));
    }

    /// PLAN-036 T-07（D5）：键盘 bind 路由——面 KeyPressed 先经 bind 表
    ///（switcher.at `ArrowRight -> .Advance` 在册）派发 handler；未消费
    /// 键落投影器既有臂（不炸）。VK→规范名映射钉住。
    #[test]
    fn shell_faces_key_bind_routing() {
        use crate::ui::desktop_protocol::message::InputMsg;
        assert_eq!(vk_to_bind_name(0x27), Some("ArrowRight"));
        assert_eq!(vk_to_bind_name(0x0D), Some("Enter"));
        assert_eq!(vk_to_bind_name(0x51), None, "字母键无 bind 面");
        let Ok(mut faces) = ShellFaces::load(ShellGeometry::fallback()) else {
            eprintln!("shell faces load 失败（环境）");
            return;
        };
        let mut payload = Vec::new();
        crate::ui::shell_projection::SwitcherSnapshot {
            hosted: true,
            visible: true,
            ..Default::default()
        }
        .wire_encode(&mut payload);
        assert!(faces.apply_projection(shell_face::SWITCHER, &payload));
        let rev = faces.revision(shell_face::SWITCHER);
        // 方向键：bind 命中 → handler 派发 → revision 前进。
        faces.on_input(
            shell_face::SWITCHER,
            &InputMsg::KeyPressed { wid: 0, key: 0x27, modifiers: 0 },
        );
        assert!(
            faces.revision(shell_face::SWITCHER) > rev,
            "ArrowRight bind 路由派发"
        );
        // 修饰键组合 = 宿主热键域——child 不消费（revision 不动）。
        let rev2 = faces.revision(shell_face::SWITCHER);
        faces.on_input(
            shell_face::SWITCHER,
            &InputMsg::KeyPressed { wid: 0, key: 0x27, modifiers: 2 },
        );
        assert!(faces.revision(shell_face::SWITCHER) >= rev2);
        // 无 bind 键（字母）落投影器 no-op——不炸不前进。
        faces.on_input(
            shell_face::SWITCHER,
            &InputMsg::KeyPressed { wid: 0, key: 0x51, modifiers: 0 },
        );
    }

    /// PLAN-036 T-05（B2）：dashboard 面投影应用——解释轨懒装载路径
    ///（D2-A 中间 z 档面；faces 清单 + 几何写集 + RebuildFaces 事件）。
    #[test]
    fn shell_faces_dashboard_lazy_mount_apply() {
        use crate::ui::shell_projection::{DashboardFace, DashboardSnapshot, ShellEvent};
        let Ok(mut faces) = ShellFaces::load(ShellGeometry::fallback()) else {
            eprintln!("shell faces load 失败（环境）");
            return;
        };
        let dash = DashboardSnapshot {
            hosted: true,
            visible: true,
            panel_w: 696,
            panel_h: 232,
            panel_top: 12,
            faces: vec![DashboardFace {
                id: "012-clock".into(),
                title: "时钟".into(),
                icon: "app-window".into(),
                status: "hatched".into(),
                span: "2".into(),
                tab: "main".into(),
            }],
            events: vec![ShellEvent::RebuildFaces],
        };
        let mut payload = Vec::new();
        dash.wire_encode(&mut payload);
        assert!(
            faces.apply_projection(shell_face::DASHBOARD, &payload),
            "dashboard 懒装 + 应用"
        );
        assert!(faces.revision(shell_face::DASHBOARD) > 0);
        assert!(faces.render(shell_face::DASHBOARD).is_some(), "懒装面渲染");
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
        // PLAN-036 T-04：SWITCHER 臂已激活（懒装 + 应用）——跨面 payload
        /// 按可解码即应用（wire 解码无判别位；生产宿主按 face 定载体
        /// 不混发，D6 注记）。
        assert!(faces.apply_projection(shell_face::SWITCHER, &payload));
        // 命令读走：初始为空（幂等清空）。
        assert!(faces.drain_commands(shell_face::SHELL).is_empty());
        // 渲染可产帧。
        assert!(faces.render(shell_face::SHELL).is_some());
    }
}
