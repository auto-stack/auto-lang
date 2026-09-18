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
    pub fn interpreted_writes(&self) -> Vec<ShellWrite> {
        let badge: auto_val::Value = s(self.notes_badge.clone());
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
}
