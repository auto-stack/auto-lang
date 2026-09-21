# Shell a2r 接缝面（S1 生成域 + S2 typed 接缝）

> **Status**: S1 词汇门节已转正（PLAN-674 T-03/T-04 per-kind 词汇表化，
> 2026-09-21）；其余 provisional（PLAN-027 rev2 交付面；B 形态程序落地后
> 随其 review 升格/修订）
> 来源：PLAN-027（2026-09-18，auto-os `docs/plans/027-desktop-shell-a2r.md`
> rev 2）；设计依据 `docs/design/autoui/desktop-shell-a2r.md`（裁定落定
> = B 形态 + 双轨常驻）。
> 提交锚：plan-027-dev 97d0bb75d / 274265345 / 611fbff2f / T-05 / d6e8da838；
> 词汇门扩容 = PLAN-674（auto-lang docs/plans/674-rq-codeeditor-a2r-vocab.md）。

## 范围

shell pack 五件（shell/desktop/switcher/notification_center/dashboard.at，
~1800 行）a2r 编译化的**形态无关**资产面：代码生成域补面（S1）与
宿主↔shell typed 接缝（S2）。B 形态（shell = outproc 编译 exe、渲染走
RenderQueue）直接承继本面——typed 载体即 wire payload 词汇。

## S1：codegen 生成域（`ui_gen/rust.rs`）

- **裸 popover 臂**：`popover` → `View::Popover`（坐标锚 `x/y` 双全 =
  `PopoverAnchor::Point`、placement 缺省 BottomStart；否则首 plain 子 =
  锚件、缺省 Bottom）；`class` 落面板列（缺 Width 类注入 `w-auto`，
  解释臂 `Width(Auto)` 同语义）；`ondismiss` 走 events 桶 → `on_dismiss`。
  契约测试 `test_bare_popover_point_anchor_codegen` /
  `test_bare_popover_widget_anchor_codegen`。
- **宿主合成件直发**：`window_thumbnail` / `workspace_preview` → 既有
  `View::WindowThumbnail/WorkspacePreview` 变体（iced 消费臂与解释轨同
  一 `into_iced` 面——双形态快照渲染/fallback 语义同源）。
  `test_host_synth_slot_codegen`。
- **mouse-area 臂**（普查修正 C：设计 §3a-a5"shell 未用"误记，实勘
  26 处）：事件映射与解释臂 `convert_mouse_area_untracked` 全同源
  （onmouseenter|onhover→on_enter、ondblclick→on_double_click、
  onmousedown|onclick→on_click、onmouseup→on_release、
  oncontextmenu(.prevent)→on_context_menu）。
- **tag 映射补齐**：`div`→container、`taskbar`→row（解释臂同源）。
- **显式拒绝门（per-kind 词汇表化，PLAN-674 T-03/T-04）**：
  `add_prop_to_builder`/`add_event_to_builder` 未知键 emit `compile_error!`
  （静默丢弃退役）。识别面 = **per-kind 词汇表**（单源 =
  `ui_gen/vocab.rs::view_prop_vocab`/`view_event_vocab`——语义源 = View
  IR builder 方法面（`ui/view.rs` 各 `View*Builder`；表体落 ui_gen 是
  组合律：view 域整树挂 `ui` feature 门后，a2r codegen 须在无 ui 组合
  （`test-trans` 档）可编译；新 builder setter 扩容同步表；aura schema
  对齐随收口另立）：命中发射 `.{prop}({coerced})`（值形
  Str/Bool/F32/U16/Usize/Flag 强转；事件槽 Closure/Msg 双态），真未知
  仍显式拒绝且文案含 `on <kind>` 上下文。复合组件族（menubar 族
  value/title/icon/shortcut/enabled/checked）由族降层臂整体消费
  （折叠契约镜像解释态 `convert_menubar_component`；开合态走
  `action_config::MENUBAR_OPEN` 全局面 + 合成变体 `__MenubarToggle(String)`
  /`__MenubarClose`——与 VM 轨 renderer 同机制），不经词汇门。认知且
  双轨同弃层 = 布局 hover 事件（View IR 布局件无 hover 槽，解释
  `set_layout_events` 同弃）。
- **shell 全量词汇门**：`test_shell_pack_codegen_vocabulary_gate`——
  真源五件每对 (tag, prop/event-base) 对表断言；表即合同，pack 新词汇
  须同步扩臂 + 扩表。

## S2：typed 接缝

- **投影载体** `ui/shell_projection.rs`：`ShellProjection`（shell 面指纹
  门控组 typed 载体——wins/workspaces/mru/notes/标量派生面 + `fp`）、
  `ShellEvent`（RebuildMru/RebuildNotes/RunningSync/ApplyFilter/
  RebuildFaces）、`ShellClock`（独立脏帧通道）、懒挂载 payload 四件
  （Switcher/Notes/Launcher/DashboardSnapshot）、`DesktopSurfaceSync`、
  `ShellManifest`（五件两常驻三懒挂载）。**wire 叶面保形**：bool→"1"/""
  等 lowering 单点（`interpreted_writes`），Obj 字段集与解释轨逐字段
  一致。plain-data 可序列化（B-ready）。
- **sync 单源化**：`renderer.rs::build_shell_projection`（派生逻辑单点）
  + `apply_shell_projection_interpreted`（指纹门控保留、写集逐字节
  一致；desktop 层 `__wm_running`/RunningSync 随行）。cursor/drag 等
  "只写不置脏"字段不入快照组（逐事件写语义保持）。
- **命令接缝** `ui/session.rs`：`DesktopBusHandle`（枚举载荷单方法
  `send(DesktopCommand)` + provided `send_record`——与解释轨
  `__desktop_cmd` 同一 `parse_records` 单点分型，双轨零分叉）；
  `DesktopBusQueue`（进程内队列，outproc 演进时换线载体）。
- **storage 接缝**：a2r codegen 译 `vm::ffi::stdlib::shim_storage_*`
  （与解释轨原生位同一 KV 后端）；`HostStorage`/`ShimHostStorage` 为
  装配层显式注入/测试替身面。
- **对拍门**：`desktop_command_roundtrip_full_vocabulary`——52 变体
  显式枚举 encode→parse 恒等（新增动词漏臂即红；曾捕获 `SetThemeName`
  encode 死词并修复）。

## 零回归面（双轨前提）

解释装载路径行为零变化：sync 重构为逐行平移（投影门控族 8 测 +
`ui::session::tests` 71 全绿）；解释轨 `__desktop_cmd` 字符串通道原样。
全量失败集与改前基线逐一全等（本机 41 项预存红在册）。

## 已知限制（→ B 程序/后续计划）

- 按钮动态 style + variant：preset 不注入（PLAN-571 文档化先例）——
  双轨视觉对拍线需复核（shell 按钮 ×38 携 variant）。
- `shell_packs_compile` 冒烟缺 dashboard.at（历史遗漏，PLAN-024 引入）。
- 基线红 `p010_popover_ondismiss_extracted_from_events`（疑因：desktop.at
  拖拽幽灵 popover 无 ondismiss → 解释臂落 widget 形态
  `__popover_close` 兜底；VM 态无此处理语义）——KNOWN-DEBT 在册。
