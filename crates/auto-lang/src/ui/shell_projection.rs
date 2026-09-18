//! PLAN-027 T-05：ShellProjection —— 桌面 shell 投影 typed 快照载体
//! （设计 docs/design/autoui/desktop-shell-a2r.md §6-S2；PLAN-027 §5.1
//! D1/D2 定案）。
//!
//! **定位**：投影协议 v1.8 的类型化增量通道（SD-02）。解释态 `__wm_*`
//! write_state 通道双轨期原样保留（I1）；本模块是**同一投影语义的
//! plain-data 载体**——宿主侧 `build_shell_projection`（renderer.rs
//! sync_shell_windows 同一派生逻辑单源化）构建，解释轨经
//! [`ShellProjection::interpreted_writes`] lowering 回写状态变量（字段
//! 集与 wire 形逐字节一致），编译轨（T-08 ShellSurface 装配）直接消费
//! typed 载体。
//!
//! **B-ready（I3）**：全部 plain data，无进程内假设；wire 叶面保形原则
//! （focused 等布尔在载体为 bool，lowering 单点译 "1"/""——零漂移锚）。
//!
//! **懒挂载面**：launcher/switcher/notification_center/dashboard 的召唤
//! 注入各有独立 payload（注入时点 = 召唤期，非指纹门控帧拍）——见
//! [`SwitcherSnapshot`] 等。cursor/drag 等"只写不置脏"字段**不入**任何
//! 快照（逐事件写语义保持）。

/// 召唤事件位——宿主写状态不触发 handler，随快照显式携带（替换
/// call_handler 直调；PLAN-014 语义零漂移）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellEvent {
    /// switcher 召唤/Init：重建 rows（renderer summon_switcher）。
    RebuildMru,
    /// notification_center 召唤/活更新：重建 rows。
    RebuildNotes,
    /// desktop.at：launching ack 求差（PLAN-014 W-04；随 __wm_running）。
    RunningSync,
    /// launcher 召唤：重算 ranked/网格行。
    ApplyFilter,
    /// dashboard 召唤/活更新：空态判据刷新（face_ids.len）。
    RebuildFaces,
}

/// 单窗投影条目（`__wm_wins`/`__wm_mru` 共用形态）。native 槽位条目
/// （Plan 486 v1.3）以 `native == true` 表达：workspace/app/pager/
/// pinned/dup_app 无义（lowering 按分支发出与现状逐字段一致的 Obj）。
#[derive(Debug, Clone, PartialEq)]
pub struct ShellWin {
    /// App 窗 = Wid 数字串；native 槽位 = "N<slot_id>"（独立编码空间）。
    pub wid: String,
    pub title: String,
    pub focused: bool,
    /// App 窗 = 分区 id；native = None。
    pub workspace: Option<usize>,
    /// Plan 486 v1.3：native 槽位条目。
    pub native: bool,
    /// registry_id（App 窗；native 恒空）。
    pub app: String,
    /// 注册表图标（缺省 app-window；native = 缓存 HICON 字段 hicon:<slot>）。
    pub icon: String,
    /// Plan 505 B2 v1.5：本窗属其分区缩略前 4（mru/native 条目恒 false）。
    pub pager: bool,
    /// PLAN-012 O2：本窗 app 已固定（dock 跳过判据）。
    pub pinned: bool,
    /// PLAN-012 O2：同 app 已有更前位窗。
    pub dup_app: bool,
}

/// 分区投影条目（协议 v1 §2.2；v1.1 label、v1.5 more）。
#[derive(Debug, Clone, PartialEq)]
pub struct ShellWorkspace {
    pub id: usize,
    pub name: String,
    pub current: bool,
    /// 1 基人读标签（宿主投影——避开 .at 字符串算术）。
    pub label: String,
    /// 缩略溢出标签 "+N"（分区窗数 >4；无溢出空串）。
    pub more: String,
}

/// 通知历史条目（协议 v1.2；v1.8 app 来源位）。
#[derive(Debug, Clone, PartialEq)]
pub struct ShellNote {
    pub id: u64,
    pub kind: String,
    pub msg: String,
    pub at: String,
    /// 来源 app id（宿主内部通知/历史恢复为 ""）。
    pub app: String,
}

/// dock 固定条目（config.dock_pinned 单源；icon 宿主自注册表解析）。
#[derive(Debug, Clone, PartialEq)]
pub struct DockPin {
    pub id: String,
    pub icon: String,
    /// 运行窗在场（PLAN-012 O2 灰条判据）。
    pub running: bool,
}

/// shell（任务栏）面指纹门控快照——`sync_shell_windows` 每帧写集的
/// typed 载体。`fp` 随载体（门控判定留宿主 apply 侧）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShellProjection {
    /// 跨分区非隐藏窗（z_order 序）∪ native Docked 槽位条目。
    pub wins: Vec<ShellWin>,
    /// 常规分区（负一屏保留分区除外）。
    pub workspaces: Vec<ShellWorkspace>,
    /// 当前分区 MRU 序（switcher 合同面；dock 不消费）。
    pub mru: Vec<ShellWin>,
    /// 通知历史全量（合同面）。
    pub notes: Vec<ShellNote>,
    /// `"layout\tfocused_wid"`（布局钮态）。
    pub meta: String,
    /// `",id1,id2,"` 运行中 app 集合派生串。
    pub running_csv: String,
    /// 聚焦窗 registry_id（"" = 无聚焦/native 聚焦）。
    pub focused_app: String,
    pub notes_unread: u64,
    /// 宿主派生 badge 串（>9 → "9+"、0 → ""）。
    pub notes_badge: String,
    pub notes_visible: bool,
    /// `",id1,id2,"` 固定集合派生串。
    pub dock_pinned_csv: String,
    pub dock_pinned: Vec<DockPin>,
    pub settings_open: bool,
    pub showdesk: bool,
    pub dashboard_visible: bool,
    /// free | grid | master-stack。
    pub layout: String,
    /// 指纹（协议 v1 §2.3 段式）——门控判定留在宿主 apply 侧。
    pub fp: String,
}

/// clock/date 独立通道（PLAN-014 W-06'：ServiceTick 帧泵、独立变化才写
/// ——不入指纹门控组，独立脏帧语义保位）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShellClock {
    /// HH:MM 本地时。
    pub time: String,
    /// "M月D日 周X"。
    pub date: String,
}

/// switcher overlay 召唤注入快照（mru 平行字符串列表 B12 规避形态 +
/// 合同面 `__wm_mru`；召唤时点定序，打开期间不重注）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SwitcherSnapshot {
    pub hosted: bool,
    pub visible: bool,
    pub mru_wids: Vec<String>,
    pub mru_titles: Vec<String>,
    pub mru_icons: Vec<String>,
    /// Plan 497 G5：快照就绪标记（"1"/""）。
    pub mru_thumbs: Vec<String>,
    /// 合同面 Obj 数组（typed 载体）。
    pub wm_mru: Vec<ShellWin>,
    pub events: Vec<ShellEvent>,
}

/// notification_center overlay 注入快照（note_* 平行列表 + 面板几何 +
/// 合同面；打开期间活更新同 payload）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NotesSnapshot {
    pub hosted: bool,
    pub visible: bool,
    /// 面板列表最大高 px（PLAN-012 O3 宿主按视口注入）。
    pub panel_max_h: u32,
    pub note_ids: Vec<String>,
    pub note_kinds: Vec<String>,
    pub note_msgs: Vec<String>,
    pub note_ats: Vec<String>,
    pub note_apps: Vec<String>,
    pub wm_notes: Vec<ShellNote>,
    pub wm_notes_unread: u64,
    pub events: Vec<ShellEvent>,
}

/// launcher overlay 注入快照（apps_* 七平行列表 B12 规避形态；
/// PLAN-015 locale 解析 display 链在宿主）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LauncherSnapshot {
    pub hosted: bool,
    pub visible: bool,
    pub app_ids: Vec<String>,
    pub app_titles: Vec<String>,
    pub app_icons: Vec<String>,
    pub app_cats: Vec<String>,
    /// id 小写键（中英两可搜）。
    pub app_lns: Vec<String>,
    pub app_lts: Vec<String>,
    /// Plan 503 M4 品牌色（stella 粉彩系 6 位 hex）。
    pub app_colors: Vec<String>,
    pub events: Vec<ShellEvent>,
}

/// dashboard overlay 注入快照（face_* 平行列表 + 面板几何镜像 + 合同面）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DashboardFace {
    pub id: String,
    pub title: String,
    pub icon: String,
    /// running | hatched | placeholder。
    pub status: String,
    /// "1" | "2"。
    pub span: String,
    /// "main" | "system"（R9 双 tab 面）。
    pub tab: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DashboardSnapshot {
    pub hosted: bool,
    pub visible: bool,
    /// 面板几何 px（宿主单一事实，本面镜像）。
    pub panel_w: u32,
    pub panel_h: u32,
    pub panel_top: u32,
    pub faces: Vec<DashboardFace>,
    pub events: Vec<ShellEvent>,
}

/// desktop 本体面从 sync 路径收到的增量：`__wm_running` + RunningSync
/// 召唤（PLAN-014 W-04 launching ack 数据面）。其余字段（cells/bg/wp/
/// cursor/drag）各有独立写点，不入快照。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DesktopSurfaceSync {
    pub running_csv: String,
    pub events: Vec<ShellEvent>,
}

/// desktop 本体面 outproc 全量快照（PLAN-030：in-proc 写集的 typed 化
/// ——`inject_desktop_surface` 八键 + `__wm_running`；cursor/drag 维持
/// 事件通道[ShellCursorMove]不入快照，与解释轨"只写不置脏"语义同册）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DesktopSurfaceSnapshot {
    /// `__desktop_icons`（F2 自定义列表 − hidden）。
    pub icons: Vec<DesktopIconEntry>,
    /// `__desktop_cells`（含 spacer 填充，行主序）。
    pub cells: Vec<DesktopCellEntry>,
    /// 平行字符串列表（B12 规避——handler 下标读）。
    pub cell_ids: Vec<String>,
    pub cell_cs: Vec<String>,
    pub cell_rs: Vec<String>,
    /// `__desktop_bg`（"#RRGGBB" → 消费侧拼 bg-[..]；纯色分支空串）。
    pub bg: String,
    /// `__desktop_label_dark`（暗壁纸白字旗标）。
    pub label_dark: bool,
    /// `__desktop_hidden` csv。
    pub hidden: String,
    /// `__wm_running`（launching ack 求差数据面）。
    pub running_csv: String,
    pub events: Vec<ShellEvent>,
    /// 指纹门（per-face 宿主侧缓存比较——与 ShellProjection.fp 同册）。
    pub fp: String,
}

/// `__desktop_icons` 条目（typed 叶）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DesktopIconEntry {
    pub id: String,
    pub icon: String,
    pub label: String,
    /// custom（F2 后唯一来源）。
    pub src: String,
    pub color: String,
}

/// `__desktop_cells` 条目（spacer 与图标位同形；spacer = true 时仅 c/r
/// 有效——与 lowering 键集逐字段一致）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DesktopCellEntry {
    pub spacer: bool,
    pub id: String,
    pub icon: String,
    /// 满幅 tile 旗标（iconfile: 真位图）。
    pub full: bool,
    pub label: String,
    pub src: String,
    pub color: String,
    pub c: usize,
    pub r: usize,
}

// ====================== wire 编解码（PLAN-030 T-02）======================
//
// 027 休眠 typed 载体的 wire 激活：LE 原语直用 desktop_protocol::codec，
// 叶面保形（bool 载体 bool、字符串载体 string——"1"/"" lowering 单点
// 留在解释轨/child 侧 apply）。载荷走 ControlMsg::ShellProjectionPush
// {face, payload}；clock/cursor 独立变体字段直载（不入快照语义）。

#[cfg(feature = "ui-iced")]
pub mod wire {
    use super::*;
    use crate::ui::desktop_protocol::codec::{put_bool, put_string, put_u32, put_u64, put_u8, Reader};

    type CResult<T> = Result<T, crate::ui::desktop_protocol::CodecError>;

    fn put_usize(out: &mut Vec<u8>, v: usize) {
        put_u64(out, v as u64);
    }

    fn usize_of(r: &mut Reader<'_>) -> CResult<usize> {
        Ok(r.u64()? as usize)
    }

    fn put_str_vec(out: &mut Vec<u8>, v: &[String]) {
        put_u32(out, v.len() as u32);
        for s in v {
            put_string(out, s);
        }
    }

    fn str_vec_of(r: &mut Reader<'_>) -> CResult<Vec<String>> {
        let n = r.u32()? as usize;
        let mut v = Vec::with_capacity(n.min(4096));
        for _ in 0..n {
            v.push(r.string()?);
        }
        Ok(v)
    }

    fn shell_event_tag(e: &ShellEvent) -> u8 {
        match e {
            ShellEvent::RebuildMru => 1,
            ShellEvent::RebuildNotes => 2,
            ShellEvent::RunningSync => 3,
            ShellEvent::ApplyFilter => 4,
            ShellEvent::RebuildFaces => 5,
        }
    }

    fn shell_event_of(tag: u8) -> CResult<ShellEvent> {
        Ok(match tag {
            1 => ShellEvent::RebuildMru,
            2 => ShellEvent::RebuildNotes,
            3 => ShellEvent::RunningSync,
            4 => ShellEvent::ApplyFilter,
            5 => ShellEvent::RebuildFaces,
            other => {
                return Err(crate::ui::desktop_protocol::CodecError::UnknownTag(other))
            }
        })
    }

    fn put_events(out: &mut Vec<u8>, events: &[ShellEvent]) {
        put_u32(out, events.len() as u32);
        for e in events {
            put_u8(out, shell_event_tag(e));
        }
    }

    fn events_of(r: &mut Reader<'_>) -> CResult<Vec<ShellEvent>> {
        let n = r.u32()? as usize;
        let mut v = Vec::with_capacity(n.min(64));
        for _ in 0..n {
            v.push(shell_event_of(r.u8()?)?);
        }
        Ok(v)
    }

    impl ShellWin {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_string(out, &self.wid);
            put_string(out, &self.title);
            put_bool(out, self.focused);
            match self.workspace {
                Some(ws) => {
                    put_bool(out, true);
                    put_usize(out, ws);
                }
                None => put_bool(out, false),
            }
            put_bool(out, self.native);
            put_string(out, &self.app);
            put_string(out, &self.icon);
            put_bool(out, self.pager);
            put_bool(out, self.pinned);
            put_bool(out, self.dup_app);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self {
                wid: r.string()?,
                title: r.string()?,
                focused: r.bool()?,
                workspace: if r.bool()? { Some(usize_of(r)?) } else { None },
                native: r.bool()?,
                app: r.string()?,
                icon: r.string()?,
                pager: r.bool()?,
                pinned: r.bool()?,
                dup_app: r.bool()?,
            })
        }
    }

    impl ShellWorkspace {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_usize(out, self.id);
            put_string(out, &self.name);
            put_bool(out, self.current);
            put_string(out, &self.label);
            put_string(out, &self.more);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self {
                id: usize_of(r)?,
                name: r.string()?,
                current: r.bool()?,
                label: r.string()?,
                more: r.string()?,
            })
        }
    }

    impl ShellNote {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_u64(out, self.id);
            put_string(out, &self.kind);
            put_string(out, &self.msg);
            put_string(out, &self.at);
            put_string(out, &self.app);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self {
                id: r.u64()?,
                kind: r.string()?,
                msg: r.string()?,
                at: r.string()?,
                app: r.string()?,
            })
        }
    }

    impl DockPin {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_string(out, &self.id);
            put_string(out, &self.icon);
            put_bool(out, self.running);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self { id: r.string()?, icon: r.string()?, running: r.bool()? })
        }
    }

    fn put_wins(out: &mut Vec<u8>, wins: &[ShellWin]) {
        put_u32(out, wins.len() as u32);
        for w in wins {
            w.wire_encode(out);
        }
    }

    fn wins_of(r: &mut Reader<'_>) -> CResult<Vec<ShellWin>> {
        let n = r.u32()? as usize;
        let mut v = Vec::with_capacity(n.min(512));
        for _ in 0..n {
            v.push(ShellWin::wire_decode(r)?);
        }
        Ok(v)
    }

    impl ShellProjection {
        /// shell 面 payload（`ShellProjectionPush{face: SHELL, ..}`）。
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_wins(out, &self.wins);
            put_u32(out, self.workspaces.len() as u32);
            for w in &self.workspaces {
                w.wire_encode(out);
            }
            put_wins(out, &self.mru);
            put_u32(out, self.notes.len() as u32);
            for n in &self.notes {
                n.wire_encode(out);
            }
            put_string(out, &self.meta);
            put_string(out, &self.running_csv);
            put_string(out, &self.focused_app);
            put_u64(out, self.notes_unread);
            put_string(out, &self.notes_badge);
            put_bool(out, self.notes_visible);
            put_string(out, &self.dock_pinned_csv);
            put_u32(out, self.dock_pinned.len() as u32);
            for p in &self.dock_pinned {
                p.wire_encode(out);
            }
            put_bool(out, self.settings_open);
            put_bool(out, self.showdesk);
            put_bool(out, self.dashboard_visible);
            put_string(out, &self.layout);
            put_string(out, &self.fp);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            let wins = wins_of(r)?;
            let n = r.u32()? as usize;
            let mut workspaces = Vec::with_capacity(n.min(64));
            for _ in 0..n {
                workspaces.push(ShellWorkspace::wire_decode(r)?);
            }
            let mru = wins_of(r)?;
            let n = r.u32()? as usize;
            let mut notes = Vec::with_capacity(n.min(1024));
            for _ in 0..n {
                notes.push(ShellNote::wire_decode(r)?);
            }
            let meta = r.string()?;
            let running_csv = r.string()?;
            let focused_app = r.string()?;
            let notes_unread = r.u64()?;
            let notes_badge = r.string()?;
            let notes_visible = r.bool()?;
            let dock_pinned_csv = r.string()?;
            let n = r.u32()? as usize;
            let mut dock_pinned = Vec::with_capacity(n.min(256));
            for _ in 0..n {
                dock_pinned.push(DockPin::wire_decode(r)?);
            }
            let settings_open = r.bool()?;
            let showdesk = r.bool()?;
            let dashboard_visible = r.bool()?;
            let layout = r.string()?;
            let fp = r.string()?;
            Ok(Self {
                wins,
                workspaces,
                mru,
                notes,
                meta,
                running_csv,
                focused_app,
                notes_unread,
                notes_badge,
                notes_visible,
                dock_pinned_csv,
                dock_pinned,
                settings_open,
                showdesk,
                dashboard_visible,
                layout,
                fp,
            })
        }
    }

    impl DesktopIconEntry {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_string(out, &self.id);
            put_string(out, &self.icon);
            put_string(out, &self.label);
            put_string(out, &self.src);
            put_string(out, &self.color);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self {
                id: r.string()?,
                icon: r.string()?,
                label: r.string()?,
                src: r.string()?,
                color: r.string()?,
            })
        }
    }

    impl DesktopCellEntry {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_bool(out, self.spacer);
            put_string(out, &self.id);
            put_string(out, &self.icon);
            put_bool(out, self.full);
            put_string(out, &self.label);
            put_string(out, &self.src);
            put_string(out, &self.color);
            put_usize(out, self.c);
            put_usize(out, self.r);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self {
                spacer: r.bool()?,
                id: r.string()?,
                icon: r.string()?,
                full: r.bool()?,
                label: r.string()?,
                src: r.string()?,
                color: r.string()?,
                c: usize_of(r)?,
                r: usize_of(r)?,
            })
        }
    }

    impl DesktopSurfaceSnapshot {
        /// desktop 面 payload（`ShellProjectionPush{face: DESKTOP_SURFACE,
        /// ..}`）——解释轨 inject_desktop_surface 写集的 typed 全量。
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_u32(out, self.icons.len() as u32);
            for e in &self.icons {
                e.wire_encode(out);
            }
            put_u32(out, self.cells.len() as u32);
            for e in &self.cells {
                e.wire_encode(out);
            }
            put_str_vec(out, &self.cell_ids);
            put_str_vec(out, &self.cell_cs);
            put_str_vec(out, &self.cell_rs);
            put_string(out, &self.bg);
            put_bool(out, self.label_dark);
            put_string(out, &self.hidden);
            put_string(out, &self.running_csv);
            put_events(out, &self.events);
            put_string(out, &self.fp);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            let n = r.u32()? as usize;
            let mut icons = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                icons.push(DesktopIconEntry::wire_decode(r)?);
            }
            let n = r.u32()? as usize;
            let mut cells = Vec::with_capacity(n.min(8192));
            for _ in 0..n {
                cells.push(DesktopCellEntry::wire_decode(r)?);
            }
            let cell_ids = str_vec_of(r)?;
            let cell_cs = str_vec_of(r)?;
            let cell_rs = str_vec_of(r)?;
            let bg = r.string()?;
            let label_dark = r.bool()?;
            let hidden = r.string()?;
            let running_csv = r.string()?;
            let events = events_of(r)?;
            let fp = r.string()?;
            Ok(Self {
                icons,
                cells,
                cell_ids,
                cell_cs,
                cell_rs,
                bg,
                label_dark,
                hidden,
                running_csv,
                events,
                fp,
            })
        }
    }

    impl DesktopSurfaceSync {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_string(out, &self.running_csv);
            put_events(out, &self.events);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self { running_csv: r.string()?, events: events_of(r)? })
        }
    }

    impl ShellClock {
        /// 仅供参考——生产 clock 走 `ControlMsg::ShellClockTick` 变体
        /// 字段直载（不入 payload）。
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_string(out, &self.time);
            put_string(out, &self.date);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self { time: r.string()?, date: r.string()? })
        }
    }

    impl SwitcherSnapshot {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_bool(out, self.hosted);
            put_bool(out, self.visible);
            put_str_vec(out, &self.mru_wids);
            put_str_vec(out, &self.mru_titles);
            put_str_vec(out, &self.mru_icons);
            put_str_vec(out, &self.mru_thumbs);
            put_wins(out, &self.wm_mru);
            put_events(out, &self.events);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self {
                hosted: r.bool()?,
                visible: r.bool()?,
                mru_wids: str_vec_of(r)?,
                mru_titles: str_vec_of(r)?,
                mru_icons: str_vec_of(r)?,
                mru_thumbs: str_vec_of(r)?,
                wm_mru: wins_of(r)?,
                events: events_of(r)?,
            })
        }
    }

    impl NotesSnapshot {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_bool(out, self.hosted);
            put_bool(out, self.visible);
            put_u32(out, self.panel_max_h);
            put_str_vec(out, &self.note_ids);
            put_str_vec(out, &self.note_kinds);
            put_str_vec(out, &self.note_msgs);
            put_str_vec(out, &self.note_ats);
            put_str_vec(out, &self.note_apps);
            put_u32(out, self.wm_notes.len() as u32);
            for n in &self.wm_notes {
                n.wire_encode(out);
            }
            put_u64(out, self.wm_notes_unread);
            put_events(out, &self.events);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            let hosted = r.bool()?;
            let visible = r.bool()?;
            let panel_max_h = r.u32()?;
            let note_ids = str_vec_of(r)?;
            let note_kinds = str_vec_of(r)?;
            let note_msgs = str_vec_of(r)?;
            let note_ats = str_vec_of(r)?;
            let note_apps = str_vec_of(r)?;
            let n = r.u32()? as usize;
            let mut wm_notes = Vec::with_capacity(n.min(1024));
            for _ in 0..n {
                wm_notes.push(ShellNote::wire_decode(r)?);
            }
            let wm_notes_unread = r.u64()?;
            let events = events_of(r)?;
            Ok(Self {
                hosted,
                visible,
                panel_max_h,
                note_ids,
                note_kinds,
                note_msgs,
                note_ats,
                note_apps,
                wm_notes,
                wm_notes_unread,
                events,
            })
        }
    }

    impl LauncherSnapshot {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_bool(out, self.hosted);
            put_bool(out, self.visible);
            put_str_vec(out, &self.app_ids);
            put_str_vec(out, &self.app_titles);
            put_str_vec(out, &self.app_icons);
            put_str_vec(out, &self.app_cats);
            put_str_vec(out, &self.app_lns);
            put_str_vec(out, &self.app_lts);
            put_str_vec(out, &self.app_colors);
            put_events(out, &self.events);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self {
                hosted: r.bool()?,
                visible: r.bool()?,
                app_ids: str_vec_of(r)?,
                app_titles: str_vec_of(r)?,
                app_icons: str_vec_of(r)?,
                app_cats: str_vec_of(r)?,
                app_lns: str_vec_of(r)?,
                app_lts: str_vec_of(r)?,
                app_colors: str_vec_of(r)?,
                events: events_of(r)?,
            })
        }
    }

    impl DashboardFace {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_string(out, &self.id);
            put_string(out, &self.title);
            put_string(out, &self.icon);
            put_string(out, &self.status);
            put_string(out, &self.span);
            put_string(out, &self.tab);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            Ok(Self {
                id: r.string()?,
                title: r.string()?,
                icon: r.string()?,
                status: r.string()?,
                span: r.string()?,
                tab: r.string()?,
            })
        }
    }

    impl DashboardSnapshot {
        pub fn wire_encode(&self, out: &mut Vec<u8>) {
            put_bool(out, self.hosted);
            put_bool(out, self.visible);
            put_u32(out, self.panel_w);
            put_u32(out, self.panel_h);
            put_u32(out, self.panel_top);
            put_u32(out, self.faces.len() as u32);
            for f in &self.faces {
                f.wire_encode(out);
            }
            put_events(out, &self.events);
        }

        pub fn wire_decode(r: &mut Reader<'_>) -> CResult<Self> {
            let hosted = r.bool()?;
            let visible = r.bool()?;
            let panel_w = r.u32()?;
            let panel_h = r.u32()?;
            let panel_top = r.u32()?;
            let n = r.u32()? as usize;
            let mut faces = Vec::with_capacity(n.min(256));
            for _ in 0..n {
                faces.push(DashboardFace::wire_decode(r)?);
            }
            let events = events_of(r)?;
            Ok(Self { hosted, visible, panel_w, panel_h, panel_top, faces, events })
        }
    }
}

// ====================== 解释轨 lowering（单点构造）======================

/// 解释轨回写项——write_state（Scalar）/ write_state_vec（Array）双形态。
#[derive(Debug, Clone, PartialEq)]
pub enum ShellWrite {
    Scalar(&'static str, auto_val::Value),
    Array(&'static str, Vec<auto_val::Value>),
}

fn s(v: impl Into<String>) -> auto_val::Value {
    auto_val::Value::Str(v.into().into())
}

impl ShellWin {
    /// Obj lowering——App/native 分支字段集与 sync_shell_windows 现状
    /// 逐字段一致（native 分支无 workspace/app/pinned/dup_app，判据统一
    /// 不缺字段的字段恒空串照发）。
    pub fn to_value(&self) -> auto_val::Value {
        if self.native {
            return auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                ("wid", s(self.wid.clone())),
                ("title", s(self.title.clone())),
                ("focused", s(String::new())),
                ("native", s("1")),
                ("icon", s(self.icon.clone())),
                ("pager", s(String::new())),
            ])));
        }
        auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
            ("wid", s(self.wid.clone())),
            ("title", s(self.title.clone())),
            ("focused", s(if self.focused { "1" } else { "" })),
            ("workspace", s(self.workspace.unwrap_or(0).to_string())),
            ("native", s(String::new())),
            ("app", s(self.app.clone())),
            ("icon", s(self.icon.clone())),
            ("pager", s(if self.pager { "1" } else { "" })),
            ("pinned", s(if self.pinned { "1" } else { "" })),
            ("dup_app", s(if self.dup_app { "1" } else { "" })),
        ])))
    }
}

impl ShellWorkspace {
    pub fn to_value(&self) -> auto_val::Value {
        auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
            ("id", s(self.id.to_string())),
            ("name", s(self.name.clone())),
            ("current", s(if self.current { "1" } else { "" })),
            ("label", s(self.label.clone())),
            ("more", s(self.more.clone())),
        ])))
    }
}

impl ShellNote {
    pub fn to_value(&self) -> auto_val::Value {
        auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
            ("id", s(self.id.to_string())),
            ("kind", s(self.kind.clone())),
            ("msg", s(self.msg.clone())),
            ("at", s(self.at.clone())),
            ("app", s(self.app.clone())),
        ])))
    }
}

impl DockPin {
    pub fn to_value(&self) -> auto_val::Value {
        auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
            ("id", s(self.id.clone())),
            ("icon", s(self.icon.clone())),
            ("running", s(if self.running { "1" } else { "" })),
        ])))
    }
}

impl ShellProjection {
    /// 解释轨回写序列——与 sync_shell_windows 现行写集逐一对应
    /// （调用方按序 write_state[_vec] 后置 view_dirty）。
    pub fn interpreted_writes(&self) -> Vec<ShellWrite> {        let badge: auto_val::Value = s(self.notes_badge.clone());
        vec![
            ShellWrite::Array("__wm_wins", self.wins.iter().map(|w| w.to_value()).collect()),
            ShellWrite::Array(
                "__wm_workspaces",
                self.workspaces.iter().map(|w| w.to_value()).collect(),
            ),
            ShellWrite::Array("__wm_mru", self.mru.iter().map(|w| w.to_value()).collect()),
            ShellWrite::Array("__wm_notes", self.notes.iter().map(|n| n.to_value()).collect()),
            ShellWrite::Scalar("__wm_meta", s(self.meta.clone())),
            ShellWrite::Scalar("__wm_running", s(self.running_csv.clone())),
            ShellWrite::Scalar("__wm_focused_app", s(self.focused_app.clone())),
            ShellWrite::Scalar("__wm_notes_unread", s(self.notes_unread.to_string())),
            ShellWrite::Scalar("__wm_notes_badge", badge),
            ShellWrite::Scalar(
                "__wm_notes_visible",
                s(if self.notes_visible { "1" } else { "" }),
            ),
            ShellWrite::Scalar("__dock_pinned_csv", s(self.dock_pinned_csv.clone())),
            ShellWrite::Array(
                "__dock_pinned",
                self.dock_pinned.iter().map(|p| p.to_value()).collect(),
            ),
            ShellWrite::Scalar(
                "__wm_settings_open",
                s(if self.settings_open { "1" } else { "" }),
            ),
            ShellWrite::Scalar("__wm_showdesk", s(if self.showdesk { "1" } else { "" })),
            ShellWrite::Scalar(
                "__wm_dashboard",
                s(if self.dashboard_visible { "1" } else { "" }),
            ),
            ShellWrite::Scalar("__wm_layout", s(self.layout.clone())),
            ShellWrite::Scalar("__wm_fp", s(self.fp.clone())),
        ]
    }
}

impl DesktopIconEntry {
    fn to_value(&self) -> auto_val::Value {
        auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
            ("id", s(self.id.clone())),
            ("icon", s(self.icon.clone())),
            ("label", s(self.label.clone())),
            ("src", s(self.src.clone())),
            ("color", s(self.color.clone())),
        ])))
    }
}

impl DesktopCellEntry {
    fn to_value(&self) -> auto_val::Value {
        if self.spacer {
            return auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
                ("spacer", s("1")),
                ("c", s(self.c.to_string())),
                ("r", s(self.r.to_string())),
            ])));
        }
        auto_val::Value::Obj(Box::new(auto_val::Obj::from_pairs([
            ("id", s(self.id.clone())),
            ("icon", s(self.icon.clone())),
            ("full", s(if self.full { "1" } else { "" })),
            ("label", s(self.label.clone())),
            ("src", s(self.src.clone())),
            ("color", s(self.color.clone())),
            ("c", s(self.c.to_string())),
            ("r", s(self.r.to_string())),
        ])))
    }
}

impl DesktopSurfaceSnapshot {
    /// 解释轨回写序列——与 inject_desktop_surface 现行写集逐一对应
    /// （outproc child 侧 apply：按序 write_state[_vec] 后置 view_dirty；
    /// `__desktop_cursor_*` 事件通道独立，不在本组）。
    pub fn interpreted_writes(&self) -> Vec<ShellWrite> {
        vec![
            ShellWrite::Array("__desktop_icons", self.icons.iter().map(|e| e.to_value()).collect()),
            ShellWrite::Array("__desktop_cells", self.cells.iter().map(|e| e.to_value()).collect()),
            ShellWrite::Array(
                "__desktop_cell_ids",
                self.cell_ids.iter().map(|v| s(v.clone())).collect(),
            ),
            ShellWrite::Array(
                "__desktop_cell_cs",
                self.cell_cs.iter().map(|v| s(v.clone())).collect(),
            ),
            ShellWrite::Array(
                "__desktop_cell_rs",
                self.cell_rs.iter().map(|v| s(v.clone())).collect(),
            ),
            ShellWrite::Scalar("__desktop_bg", s(self.bg.clone())),
            ShellWrite::Scalar("__desktop_label_dark", s(if self.label_dark { "1" } else { "0" })),
            ShellWrite::Scalar("__desktop_hidden", s(self.hidden.clone())),
            ShellWrite::Scalar("__wm_running", s(self.running_csv.clone())),
        ]
    }
}

// =========================== ShellManifest（D5）===========================

/// 面装载形态（定案记录 D5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMount {
    /// boot 期常驻装载（shell 任务栏 / desktop 本体面）。
    ResidentBoot,
    /// 召唤期懒挂载 overlay（switcher/notification_center/dashboard）。
    LazyOverlay,
}

/// 装配清单单面。
#[derive(Debug, Clone, PartialEq)]
pub struct ShellFace {
    /// 面 id（= pack 文件 stem；装配工厂 mount_face 的键）。
    pub id: &'static str,
    /// widget 名（pack `.at` 根声明；对拍/诊断消费）。
    pub widget: &'static str,
    pub mount: ShellMount,
}

/// shell pack 装配清单——**五件**（定案记录修正 A：PLAN-024 增
/// dashboard.at；launcher overlay 走注册表装载路径，非 pack 源）。
pub const SHELL_MANIFEST: ShellManifest = ShellManifest {
    crate_name: "shell-pack",
    faces: &[
        ShellFace { id: "shell", widget: "Desktop", mount: ShellMount::ResidentBoot },
        ShellFace { id: "desktop", widget: "DesktopSurface", mount: ShellMount::ResidentBoot },
        ShellFace { id: "switcher", widget: "Switcher", mount: ShellMount::LazyOverlay },
        ShellFace {
            id: "notification_center",
            widget: "NotificationCenter",
            mount: ShellMount::LazyOverlay,
        },
        ShellFace { id: "dashboard", widget: "DashboardPanel", mount: ShellMount::LazyOverlay },
    ],
};

/// 装配清单（a2r "无窗组件库"产物面；T-07 生成物同形 const）。
#[derive(Debug, Clone, PartialEq)]
pub struct ShellManifest {
    pub crate_name: &'static str,
    pub faces: &'static [ShellFace],
}

impl ShellManifest {
    pub fn face(&self, id: &str) -> Option<&ShellFace> {
        self.faces.iter().find(|f| f.id == id)
    }
}

// ================================ 测试 ================================

#[cfg(test)]
mod tests {
    use super::*;

    /// T-05：ShellWin lowering——App/native 分支字段集与 sync_shell_windows
    /// 现状逐字段一致（native 分支无 workspace/app/pinned/dup_app 键）。
    #[test]
    fn shell_win_lowering_branch_field_sets() {
        let app = ShellWin {
            wid: "3".into(),
            title: "Editor".into(),
            focused: true,
            workspace: Some(1),
            native: false,
            app: "041-auto-edit".into(),
            icon: "lucide:app-window".into(),
            pager: true,
            pinned: true,
            dup_app: false,
        }
        .to_value();
        let auto_val::Value::Obj(obj) = app else { panic!("obj") };
        let keys: Vec<String> = obj.key_names().iter().map(|k| k.to_string()).collect();
        assert_eq!(
            keys,
            vec![
                "wid", "title", "focused", "workspace", "native", "app", "icon", "pager",
                "pinned", "dup_app"
            ]
        );
        let focused = obj.get("focused").expect("focused key");
        assert!(matches!(&focused, auto_val::Value::Str(s) if s.as_str() == "1"));

        let native = ShellWin {
            wid: "N7".into(),
            title: "Console".into(),
            focused: false,
            workspace: None,
            native: true,
            app: String::new(),
            icon: "hicon:7".into(),
            pager: false,
            pinned: false,
            dup_app: false,
        }
        .to_value();
        let auto_val::Value::Obj(obj) = native else { panic!("obj") };
        let keys: Vec<String> = obj.key_names().iter().map(|k| k.to_string()).collect();
        assert_eq!(
            keys,
            vec!["wid", "title", "focused", "native", "icon", "pager"]
        );
    }

    /// T-05：interpreted_writes 覆盖 sync_shell_windows 现行写集——键序
    /// 即写序（指纹尾写，门控期间不触达）。
    #[test]
    fn interpreted_writes_cover_legacy_write_set() {
        let proj = ShellProjection {
            fp: "3:1,|grid\t3|0:1,1;|mru:|notes:0:0:0:;|pinned:;".into(),
            ..Default::default()
        };
        let writes = proj.interpreted_writes();
        let keys: Vec<&str> = writes.iter().map(|w| match w {
            ShellWrite::Scalar(k, _) | ShellWrite::Array(k, _) => *k,
        }).collect();
        assert_eq!(
            keys,
            vec![
                "__wm_wins",
                "__wm_workspaces",
                "__wm_mru",
                "__wm_notes",
                "__wm_meta",
                "__wm_running",
                "__wm_focused_app",
                "__wm_notes_unread",
                "__wm_notes_badge",
                "__wm_notes_visible",
                "__dock_pinned_csv",
                "__dock_pinned",
                "__wm_settings_open",
                "__wm_showdesk",
                "__wm_dashboard",
                "__wm_layout",
                "__wm_fp",
            ]
        );
        // 布尔 lowering wire 形（"1"/""）。
        let proj2 = ShellProjection {
            notes_visible: true,
            settings_open: false,
            showdesk: true,
            dashboard_visible: false,
            ..Default::default()
        };
        for w in proj2.interpreted_writes() {
            if let ShellWrite::Scalar(k, v) = w {
                let expect = |name: &str| -> &str {
                    match name {
                        "__wm_notes_visible" | "__wm_showdesk" => "1",
                        "__wm_settings_open" | "__wm_dashboard" => "",
                        _ => "",
                    }
                };
                if matches!(k, "__wm_notes_visible" | "__wm_showdesk" | "__wm_settings_open" | "__wm_dashboard") {
                    assert!(
                        matches!(&v, auto_val::Value::Str(sv) if sv.as_str() == expect(k)),
                        "{k} wire form"
                    );
                }
            }
        }
    }

    /// T-05（D5）：装配清单——五件、两常驻三懒挂载、face 查键。
    #[test]
    fn shell_manifest_faces() {
        assert_eq!(SHELL_MANIFEST.faces.len(), 5);
        assert_eq!(SHELL_MANIFEST.face("shell").unwrap().mount, ShellMount::ResidentBoot);
        assert_eq!(SHELL_MANIFEST.face("desktop").unwrap().mount, ShellMount::ResidentBoot);
        for lazy in ["switcher", "notification_center", "dashboard"] {
            assert_eq!(SHELL_MANIFEST.face(lazy).unwrap().mount, ShellMount::LazyOverlay);
        }
        assert!(SHELL_MANIFEST.face("nope").is_none());
    }

    /// PLAN-030 T-02：wire 编解码 round-trip——全家族（叶面保形：bool
    /// 载体 bool、字符串载体 string）。overlay 三面 v1 不下行（D6 边界）
    /// 但编码在册（AC-01 五面载体）。
    #[cfg(feature = "ui-iced")]
    #[test]
    fn wire_round_trip_full_family() {
        use crate::ui::desktop_protocol::codec::Reader;
        let win = ShellWin {
            wid: "3".into(),
            title: "编辑器".into(),
            focused: true,
            workspace: Some(2),
            native: false,
            app: "041-auto-edit".into(),
            icon: "lucide:app-window".into(),
            pager: true,
            pinned: false,
            dup_app: true,
        };
        let proj = ShellProjection {
            wins: vec![win.clone()],
            workspaces: vec![ShellWorkspace {
                id: 1,
                name: "w1".into(),
                current: true,
                label: "2".into(),
                more: "+7".into(),
            }],
            mru: vec![win],
            notes: vec![ShellNote {
                id: 9,
                kind: "toast".into(),
                msg: "你好".into(),
                at: "12:00".into(),
                app: "002-counter".into(),
            }],
            meta: "grid\t3".into(),
            running_csv: ",002-counter,".into(),
            focused_app: "002-counter".into(),
            notes_unread: 12,
            notes_badge: "9+".into(),
            notes_visible: true,
            dock_pinned_csv: ",a,".into(),
            dock_pinned: vec![DockPin { id: "a".into(), icon: "i".into(), running: false }],
            settings_open: false,
            showdesk: true,
            dashboard_visible: false,
            layout: "master-stack".into(),
            fp: "3:1,|grid\t3|".into(),
        };
        let mut buf = Vec::new();
        proj.wire_encode(&mut buf);
        let mut r = Reader::new(&buf);
        let back = ShellProjection::wire_decode(&mut r).expect("decode");
        assert!(r.remaining() == 0, "载荷恰好耗尽");
        assert_eq!(back, proj);

        let desk = DesktopSurfaceSnapshot {
            icons: vec![DesktopIconEntry {
                id: "002-counter".into(),
                icon: "iconfile:x.ico".into(),
                label: "计数器".into(),
                src: "custom".into(),
                color: "#ff00aa".into(),
            }],
            cells: vec![
                DesktopCellEntry {
                    spacer: true,
                    c: 0,
                    r: 1,
                    ..Default::default()
                },
                DesktopCellEntry {
                    spacer: false,
                    id: "002-counter".into(),
                    icon: "app-window".into(),
                    full: true,
                    label: "计数器".into(),
                    src: "custom".into(),
                    color: "#ff00aa".into(),
                    c: 1,
                    r: 0,
                },
            ],
            cell_ids: vec![String::new(), "002-counter".into()],
            cell_cs: vec!["0".into(), "1".into()],
            cell_rs: vec!["1".into(), "0".into()],
            bg: "#101014".into(),
            label_dark: true,
            hidden: "003-x".into(),
            running_csv: ",a,".into(),
            events: vec![ShellEvent::RunningSync],
            fp: "desk-fp".into(),
        };
        let mut buf = Vec::new();
        desk.wire_encode(&mut buf);
        let mut r = Reader::new(&buf);
        let back = DesktopSurfaceSnapshot::wire_decode(&mut r).expect("decode");
        assert!(r.remaining() == 0, "载荷恰好耗尽");
        assert_eq!(back, desk);

        let sync = DesktopSurfaceSync {
            running_csv: ",x,".into(),
            events: vec![ShellEvent::RunningSync, ShellEvent::RebuildMru],
        };
        let mut buf = Vec::new();
        sync.wire_encode(&mut buf);
        let mut r = Reader::new(&buf);
        assert_eq!(DesktopSurfaceSync::wire_decode(&mut r).unwrap(), sync);

        let clock = ShellClock { time: "09:05".into(), date: "9月19日 周六".into() };
        let mut buf = Vec::new();
        clock.wire_encode(&mut buf);
        let mut r = Reader::new(&buf);
        assert_eq!(ShellClock::wire_decode(&mut r).unwrap(), clock);

        let sw = SwitcherSnapshot {
            hosted: true,
            visible: true,
            mru_wids: vec!["3".into()],
            mru_titles: vec!["t".into()],
            mru_icons: vec!["i".into()],
            mru_thumbs: vec!["thumbnail://3!app-window".into()],
            wm_mru: vec![],
            events: vec![ShellEvent::RebuildMru],
        };
        let mut buf = Vec::new();
        sw.wire_encode(&mut buf);
        let mut r = Reader::new(&buf);
        assert_eq!(SwitcherSnapshot::wire_decode(&mut r).unwrap(), sw);

        let notes = NotesSnapshot {
            hosted: true,
            visible: false,
            panel_max_h: 600,
            note_ids: vec!["9".into()],
            note_kinds: vec!["toast".into()],
            note_msgs: vec!["m".into()],
            note_ats: vec!["12:00".into()],
            note_apps: vec![String::new()],
            wm_notes: vec![],
            wm_notes_unread: 1,
            events: vec![],
        };
        let mut buf = Vec::new();
        notes.wire_encode(&mut buf);
        let mut r = Reader::new(&buf);
        assert_eq!(NotesSnapshot::wire_decode(&mut r).unwrap(), notes);

        let launcher = LauncherSnapshot {
            hosted: false,
            visible: false,
            app_ids: vec!["a".into()],
            app_titles: vec!["A".into()],
            app_icons: vec!["i".into()],
            app_cats: vec!["tools".into()],
            app_lns: vec!["a".into()],
            app_lts: vec!["a".into()],
            app_colors: vec!["#fff".into()],
            events: vec![],
        };
        let mut buf = Vec::new();
        launcher.wire_encode(&mut buf);
        let mut r = Reader::new(&buf);
        assert_eq!(LauncherSnapshot::wire_decode(&mut r).unwrap(), launcher);

        let dash = DashboardSnapshot {
            hosted: true,
            visible: true,
            panel_w: 320,
            panel_h: 480,
            panel_top: 60,
            faces: vec![DashboardFace {
                id: "f".into(),
                title: "F".into(),
                icon: "i".into(),
                status: "running".into(),
                span: "1".into(),
                tab: "main".into(),
            }],
            events: vec![ShellEvent::RebuildFaces],
        };
        let mut buf = Vec::new();
        dash.wire_encode(&mut buf);
        let mut r = Reader::new(&buf);
        assert_eq!(DashboardSnapshot::wire_decode(&mut r).unwrap(), dash);
    }

    /// PLAN-030 T-02：DesktopSurfaceSnapshot 解释轨 lowering 键集 = 
    /// inject_desktop_surface 现行写集（`__desktop_cursor_*` 除外——事件
    /// 通道独立）。
    #[test]
    fn desktop_surface_snapshot_lowering_keys() {
        let desk = DesktopSurfaceSnapshot::default();
        let keys: Vec<&str> = desk
            .interpreted_writes()
            .iter()
            .map(|w| match w {
                ShellWrite::Scalar(k, _) | ShellWrite::Array(k, _) => *k,
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                "__desktop_icons",
                "__desktop_cells",
                "__desktop_cell_ids",
                "__desktop_cell_cs",
                "__desktop_cell_rs",
                "__desktop_bg",
                "__desktop_label_dark",
                "__desktop_hidden",
                "__wm_running",
            ]
        );
    }
}
