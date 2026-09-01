# ui（AURA / UI 引擎 / 桌面运行时）

> **Status**: active（主战场：vue 轨 codegen 成熟化 + VM 轨视觉 parity + 虚拟桌面线推进中）
> 最近刷新：2026-09-01（Plan 508 归档回写：桌面协议 Stage 6 收官——默认策略裁定 shell.apps.process_model + 远程 command 流〔WsTransport/:17800 镜像会话/drawlist-renderer TS 包/浏览器点击闭环〕）

## 职责

Auto 的 UI 子系统，围绕 **AURA**（UI-IR）组织，2026-08 起扩展为**桌面运行时**：

- **前端解析**：UI 方言（`widget`/`msg`/`model`/`view`/`on`/`setup`/`actions`，dialect 机制）。
- **AURA 提取与校验**：WidgetDecl → 视图树/状态/事件三元 IR，按 `schema/aura.at`（唯一契约源，plan-435）校验。
- **代码生成**（`ui_gen/`）：a2vue（主力）、a2rust、a2jet/a2ark、ts adapter、widget 契约表、block 层。
- **VM 运行时渲染**（`ui/`）：vnode/事件路由/VM 桥接、iced/gpui/headless 后端、code_editor 与
  autodown_editor 内建 widget、热重载、MCP 调试服务。
- **桌面运行时**（2026-08 新增）：`session.rs`（DesktopSession/AppSession 双层会话 + WmState/
  WmCommand 桌面消息）、`ui/iced/virtual_window.rs`（VirtualWindow 渲染）、排布纯函数与
  DesktopBus v0——单 OS 窗口内多 App 虚拟桌面。
- **a2ui 协议** 与 **`#[api]` 前后端契约**（`src/api/`）。

## 现状（2026-08-28）

**组件线（plan-437 落地，ADR-18/19）**：chart 族（line/bar/area/donut）schema 契约落库
+ vue 发射 spec 驱动化（特判臂退役）；四类图官方 Auto 组件（AutoLineChart 等四件，
widget-parens props + Init 几何 + 段记录打包，载体 widgets-gallery components/）；
VM 轨子组件 Init 渲染期补发（props 播种→Init→build，vue onMounted 对齐）——
派生计算型组件双轨可用的地基。契约细节见 [design/chart-components.md](design/chart-components.md)。
**交互态（plan-498 落地）**：四图族 emphasis 二态（line/area 图例悬停高亮+转折点浮现/
bar 分组描边/donut 扇区中角外移）+ legend onclick 点击显隐（mouse-area on_click 引擎臂，
iced on_press/vue @click）；悬停态字段图族专属（hovLn/hovAr/hovBr/hovDn 无悬停哨兵 9——
负数字面量 view 比较缺陷 P498-1 与 VM 单态串扰 P498-2 均已挂账）。
**交互 v2——指针移动流（plan-499 落地）**：mouse-area `onmousemove`+`coords:"WxH"` 通用
指针原语（事件携带 viewBox 逻辑坐标,引擎层完成屏幕→逻辑换算;VM 臂 PointerArea 自定义
widget 33ms 时间闸+0.5px 量化限频,vue 臂 DOM 原生不限频）;line axisPointer 十字线索引
吸附+tooltip 跟随,donut 极坐标扇区直接命中（svgdoc 静态限制经代码命中解除）,hover
动画双轨（timer 数值插值/CSS transition 类）;统一原语设计与 P-list 图元协议草案见
[design/autoui/canvas-pointer-events.md](../../../design/autoui/canvas-pointer-events.md)。
**diagram 家族开篇（plan-502 落地）**：flow-diagram v1（widgets-gallery Diagrams 分组
/flow-diagram 页）——数据轨 props `nodes`/`edges`/`direction`（td/lr 经转置）+
**Sugiyama-lite 分层布局纯 Auto**（DFS 回边剥离开环→最长路径分层→barycenter 双向
2 轮降交叉→层内等距/父居中）+ v1 SVG 渲染（484 charts 同通路）+ **svg `text` 直通
标签**（M1 对照定案胜出:vue 轨 in_svg_subtree 上下文分流/vm 轨 svgdoc text 序列化臂,
resvg 原生栅格化——svg 无 text 约束自此解除）+ hover emphasis/锚定 tooltip（498
三段式,哨兵 999）+ 边路由 bbox 交点直线 + head/tail 字形（arrow/diamond/circle）+
line dash/thick。契约见 [design/diagram-components.md](design/diagram-components.md)；
group 平铺/focus 模型归 Phase 2a，DSL 静态糖归 Phase 3。

**导航组件线（plan-482 落地）**：nav-item/nav-group/nav(search:) 组件族——
class 契约单一来源（`ui_gen/nav_contract.rs` ↔ 脚手架 NavItem/NavGroup 镜像，
单测锁死双端不可漂移）；hover/active/disabled 三态、icon（lucide svg/emoji 双
通道）/label+desc/badge 槽、路由 to:（vue RouterLink / VM `__navigate`+历史栈
`router.back()`）与 onclick 状态双模式、data-active 与 nav-name/nav-desc 语义
锚；顺修 lucide_svg 裸片段缺陷（此前全部 VM lucide 图标空渲染）。三 app 落地：
015-notes/018-book-reader/widgets-gallery + 外部 auto-musk(052)/auto-os-config(012)。

**slot 替换（plan-476 落地）**：VM 轨 widget 插座/填充与 vue 轨语义对齐——调用位
`slot(name:X){..}`/裸子节点渲染到子 widget outlet，父作用域求值+父事件路由+逐帧重求值；
机制为构建期 `SlotFills` 父 builder 捕获 + 五容器×双胎兄弟拼接，
详见 [design/slot-substitution.md](design/slot-substitution.md)。

**桌面视觉体系（plan-503 落地）**：stella-os 风格移植——accent 玫瑰粉（coral 校准
#c4706a）、dock 图标格/激活竖条/运行圆点、弹层 glass 三件套（bg-card/80+细边+柔影，
无 blur 降格 parity 条款）、壁纸 scrim、窗口 chrome（36px 标题栏/16px 圆角/三色圆点）、
launcher 品牌色图标底块；引擎补齐 style 串循环成员插值 `${member.field}` 双端。
详见 [design/desktop-shell.md](design/desktop-shell.md)。

**桌面线（452→459→462→463→465 落地，464 未开工，386 暂缓）**：
452 翻转"Windows 非 compositor"裁定并验证 IME/焦点分区可行 → 459 iced daemon 多 OS 窗口 +
会话化（453 的 DesktopSession/AppSession 拆分）→ **462 路线 A 地基**：VirtualWindow widget
（Stack/clip/mouse_area 组合）+ WmState/Wid + DM::Wm 消息 + 桌面级键盘路由 → 463 桌面 shell：
全屏 borderless 宿主 + 任务栏 overlay + free/grid/master-stack 排布纯函数 + pac.at 应用注册表 +
DesktopBus v0。**465 vue 端虚拟桌面已落地**（M4）：`auto run --desktop` 宿主 scaffold
（auto-man `generate_desktop_host`：scan_apps 过滤 + `src/apps-registry.ts` 静态 import 注册表 +
子组件/stores 合并）+ WM 运行时（auto-man `assets/wm/`：store.ts WmStore、layout.ts 布局 TS
直译〔I6 与 layout.rs 共享期望值表 layout_cases.json〕、keyboard.ts R12 热键捕获段、
VirtualWindow/Taskbar.vue DOM 叶——schema/aura.at `vue:` 登记源两端同源）；E1 `(AppId,event)`
注入形状与 E2 AppWindow 叶枚举成文（`docs/plans/reports/465-t4-wm-dom-leaf.md`）；
tauri 全屏壳复用同一宿主页。**464 launcher 已落地**（SummonLauncher 懒挂载 + 真注册表平行串列注入 + windowless 特权 App 拆借垫片）。**472 AutoShell 地基已落地（shell-track M1）**：DesktopBus v1 对账定案（候选 B 传输 + `desktop.*` 动词词表 8 动词，Design 25 §3 注记回写）+ 投影协议 v1 合同 （`schema/projection-protocol-v1.md`：`__wm_*` 六字段全集/`__wm_workspaces`/指纹门控，双端对拍基线）+ workspace 分区驱动（WmState 加法增域、默认 2 分区、过滤六点）+ shell.at 升格 `widget Desktop` dock（图标化/pinned activate/切换条/`shell.dock.*` 数据级配置，pinned 宿主解析 {id,icon} 注入）。M2（switcher/pager）消费面就绪。**478 shell-track M2 已落地**：投影协议升版 v1.1（`__wm_mru` 当前分区 MRU 序投影 + `__wm_workspaces.label` 1 基标签 + 指纹扩段 + 动词词表增 `workspace_add`/`workspace_close`/`send_to`，vue 端对拍基线）+ switcher overlay（`assets/switcher.at` 进程内嵌第二枚 overlay 槽：Ctrl+Tab 召唤/推进、MRU 快照平行串列注入 + RebuildMru、Tab/←→/Enter/Esc 键盘流、点击聚焦、键盘独占 + Esc 仲裁 + 仅 visible 推层）+ dock 切换条升格 pager（1 基 label、当前分区高亮、每分区 × 删除——宿主 toast 门（非空不删/末分区保底）、尾部 + 增分区即入）+ send_to 跨区发送（Ctrl+Alt+Shift+←/→，WmCommand::SendFocusedTo）+ 驱动配套（remove_workspace 重排相邻前驱/下标压实/clamp/焦点让渡、move_win_to_workspace、mru_in_workspace）。热键表：Ctrl+Tab 改道 switcher（**490 起 Alt+Tab 退役**——Win11 系统保留死键位；分区切换迁 Ctrl+Alt+[ / ]，launcher Ctrl+Space+Ctrl+Alt+Space 双收，见下 490 条目）。**479 shell-track M3 已落地**：463 瞬时 toast 升格「浮现+历史聚合」双面（`push_notification` 单入口——入史+未读+落盘+浮现+面板活更新，既有 8 处 toast 调用点改道）+ 通知中心（dock 铃铛+未读 badge——`__wm_notes_unread` 条件消费空串/零双守卫 + 第三枚 overlay 槽 `assets/notification_center.at` 右下锚定面板：懒挂载/快照注入 note_ids·kinds·msgs·ats+RebuildNotes、开面板未读清零、逐条 ×/全部清除/Esc 关、键盘独占+Esc 仲裁+仅 visible 推层）+ `notify`/`notes_toggle`/`notes_clear`/`notes_dismiss` 动词（词表 v1.2）+ storage 定长槽 `shell.notes.0..9` 持久化（persist_notes 全量重写/restore_notifications boot 恢复，NOTES_CAP=50 内存 FIFO MRU）+ 投影协议升版 v1.2（`__wm_notes` 全量 {id,kind,msg,at} + `__wm_notes_unread` 未读串 + 指纹尾接 `|notes:{len}:{front_id}:{unread};` 段，vue 端对拍基线）。**487 shell-track M4 已落地（系统设置面板 S7）**：设置面板 overlay（`assets/settings.at` 第四枚 overlay 槽：左列 Dock/通知/关于三分区导航 + 右列内容卡；DesktopState.settings_app + HostCtx.settings_fields + split_mut windowless 第五路/split_ref_settings/settings_visible + toggle_settings 懒挂载/二态翻转/配置快照注入（cfg_dock_position/cfg_dock_enabled/cfg_notes_enabled + pinned_ids 平行列表〔B12 规避〕+ about_host/about_version 常量）+ call_handler RebuildPinned + 仅 visible 推层装配 + Esc 仲裁链第五路/键盘独占/订阅第五块）+ dock 几何驱动动词与执行臂（`open_settings`/`set_dock_position`/`set_dock_enabled` 词表 v1.4——execute_set_dock_position/enabled → apply_dock_edges_now 三联动：storage 键写回 → dock_edges 键重推导〔boot 同函数，I9〕→ apply_layout relayout + 槽位排水 + shell `__dock_*` 投影热同步；enabled=false 零预留位置键保留，重开按原位恢复）+ dock 配置链升格读写闭环（`shell.dock.*` 由 472 boot 单向读 → 驱动写回 + pinned UI 写手——面板行内增删 storage.set 直写逗号拼接 = load_dock_pinned 格式，boot 生效）+ 通知持久化开关（`shell.notes.enabled` "false"=关——479 消费链 push_notification 单点门控，notify 全链路短路；缺席/其余=开向后兼容）+ 投影协议升版 **v1.4**（486 先合占 v1.3、487 按并行协调叠 v1.4——纯增量动词/storage 键，零新投影字段零指纹变化）+ shell.at 双任务栏分支齿轮入口（OpenSettingsPanel → open_settings，铃铛邻位）。os-config 跨仓深桥/主题分区待续（壁纸已由 M5 落地，见下）。**496 shell-track M5 已落地（桌面本体 S9）**：第五面 `assets/desktop.at`（常驻不召唤——boot 装载挂 463 预留桌面层 z 槽：view 装配 Stack 先于 z_order 虚拟窗推层=壁纸之上/App 窗口之下；DesktopState.desktop_app/desktop_wallpaper + HostCtx.desktop_fields + windowless 第六路拆借/split_ref_desktop）+ 壁纸双径（#hex 由 desktop.at 根 bg 插值实铺〔`__desktop_bg` 注入〕/图片路径由宿主壁纸图层铺底——DSL 无重叠布局 z 序宿主兑现；boot 解析回退 #090e1a 默认色）+ 图标网格（pinned ∪ 自定义合并去重 hidden 排除，icon/label 注册表解析；storage 三键 shell.desktop.wallpaper/icons/hidden——487 非几何无动词判定同款，boot 生效）+ ondblclick VM 全链（View::MouseArea.on_double_click 双映射→iced mouse_area；convert_view_messages 补 MouseArea 显式臂——此前 VM 动态路径落 Empty 兜底，484 图表族经 Rust codegen 未暴露）+ 右键三项本地面板（打开=activate 472 两臂/移除=hidden 直写/更换壁纸=open_settings）+ 空白点击 463 语义+ settings 四分区（+外观：壁纸输入 storage 直写 + cfg_wallpaper 召唤快照）+ 投影协议 v1.4 内字段扩展（§2.1 `__desktop_*` 三字段族，boot 一次注入无指纹门控，零新动词）+ a2vue 真资产同源金样（插值 class 缺口修复：`${.field}` 静态段+`:class` 拼接表达式五点落码）。**497 shell-track S3 已落地（Status 栏——桌面特性线收官）**：每窗口真缩略三件套——①快照核心 `ui/iced/snapshot.rs`（**T1 定案裁剪式整窗快照**：headless no-op/iced 无公开子树离屏 API 双证伪 → `iced::window::screenshot` 整窗 RGBA〔自带 scale_factor，官方支持 widget-bounds 裁剪语义〕按 `VWinState.rect×scale_factor` 裁剪 + box 降采样长边≤256；进程级 TTL 2s 缓存〔惰性过期〕+ 抓取请求队列〔500ms 冷却防风暴〕+ 事件失效三点接线〔CloseWindow/SetLayout/apply_dock_edges_now〕）②`window_thumbnail` widget 七表登记（aura.at〔vue: @/wm/WindowThumbnail I4 同源〕+registry+schema.rs+view_builder 双臂+View 变体+渲染臂+render_support Full；渲染臂命中 `Handle::from_rgba` 直绘/miss→lucide fallback+request_capture——native "N<slot>" parse 失败天然回退）③消费者三面（switcher 行缩略〔`mru_thumbs` 平行就绪标记合同面〕+ dock 条目 hover popover〔422 先例 mouse-area+open 表达式〕+ pager 分区hover 网格〔该区窗口缩略+标题〕）+ 宿主抓取编排闭环（ServiceTick 排空队列→一次整窗截图服务全部请求〔`SnapshotShot` 事件回调按 pending wid 集裁剪入缓存+switcher/shell dirty 驱动 miss→真缩略一帧升级〕）；dock 时钟（`__wm_clock` HH:MM 本地——ServiceTick 分钟变化才写，**唯一非门控注入字段**不进指纹〔投影协议 v1.4 内字段扩展注记〕）+ 托盘组右置（挂载点容器 v1 占位+铃铛+齿轮+时钟两态）；**缺陷修复**：untracked convert_element 补 popover 臂（tracked 兜底委托路径下 popover 落容器直通锚/overlay 语义全失）+ invalidate_all 清冷却表 + SnapshotShot 补 shell dirty；T5 实机六项 PASS（switcher 真像素缩略/dock hover/pager 网格/时钟走字/顶底两态/冷缓存升级链）；tf 3316+desktop_mcp 3+t2_snapshot 4+a2vue 14 绿；债务 P497-1（pager ≤4 截断——.at 无过滤后截断原语）/P497-2（a2vue props 不透传，465 先例一致）。**501 shell-track S7 已落地（系统设置接通统一 settings center——487 预留的 os-config 跨仓深桥兑现）**：daemon 生命周期管理器 `ui/osconfig_daemon.rs`（DaemonStatus 三态 Running/Spawning/Offline + DaemonIo 注入式检活〔std TCP 裸 HTTP `/api/health`——reqwest blocking 在 tokio 上下文 panic 故弃〕+ detached spawn〔Win DETACHED_PROCESS|CREATE_NEW_PROCESS_GROUP + stdio null，桌面退出不杀——共享服务语义，待澄清② v1〕+ 就绪轮询 ≤5s + badge_projection 三态徽标投影；端口约定 17701，spawn 期 `AUTOOS_BACK_PORT` 覆盖 daemon 缺省 17901；发现序 storage `shell.osconfig.daemon` > 相邻仓 `auto-os-config-back/target/release`〔二进制实名 `auto-os-config-back-server`，计划原文路径现场核验修正〕> PATH 留扩展位〔P501-1——安装态立项时接宿主 which 语义〕）+ app 注册表多扫描根（`aggregate_scan`：主根 examples 优先按 id 去重；extra 根 = storage `shell.apps.extra_dirs`〔分号分隔 `id=path`/`path`，`parse_extra_dirs`〕+ 相邻仓探测缺省 `../auto-os-config/auto` → id `os-config`〔`shell.apps.scan_siblings=false` 可关；boot 期 `host_extra_roots` 包装〕）+ launch 执行臂依赖面（pac 可选字段 `daemon: autoos`〔跨仓 os-config 0e81196〕→ `ensure_ready` 检活/懒起 → `AUTOOS_DAEMON` env 进程注入〔VM Env.get 同源〕；**Offline 不阻断 launch**〔App 自带 daemon_view 连接测试 UX〕，原因记 `DesktopState.osconfig_status` 供徽标；pac `back: { project }` 外部 back cdylib 桩桥装载〔Plan 061 链桌面补齐——`set_external_back_root` + `load_back_cdylib` auto-man rust_ui 同型，句柄驻 `DesktopState.back_keepalive` 丢弃即卸载〕）+ settings.at 五分区（+系统：「系统设置（全部模块）」入口卡 `OpenSystemSettings` → `launch	os-config`；offline 置灰「重试并打开」双态——launch 每次重新探活零额外动词；召唤注入 `osconfig_state`/`osconfig_hint`）；T3 集成档 `tests/osconfig_integration.rs`（材料门控跳过 + 90s 看门狗 + 六段面包屑：daemon 起〔USERPROFILE/HOME 重定向配置根零污染——待澄清③跨仓 config root env 因此非必需〕→ 就绪 ping → 真相邻仓条目 launch → App Init 真数据 sys_host → GET /api/modules ≥7 → PUT ai-daemon.at 落盘断言）；合并后 master 全量 3323/3323 + scoped 189/189 + T3 1/1 绿；债务 P501-1（PATH 级）/P501-2（人手点击链残差）；P501-1..6 台账。**473 native dock 假洞 Phase 1 已落地**：`ui/native_dock/`（NativeSlot 模型+状态机+策略纯逻辑 + Win32 适配层 #[cfg(windows)]/非 Windows no-op——EnumWindows+PID 发现/DWM ext-frame 几何写读回/GWL_STYLE 剥离还原/DWMWCP_DONOTROUND 直角/sink_desktop_below z 序〔insertAfter 语义实测勘误：对 desktop 调 SetWindowPos 沉到 slot 之下〕/WinEventHook 五事件 OUTOFCONTEXT 钩子线程〔RwLock 数据槽×占用 AtomicBool 分立防回调死锁〕）；DesktopBus 动词增 `dock_native`（pid=/hwnd=）/`undock_native`；WmState.native_slots 注册表 + 槽位伪 Wid 进 apply_layout 同轮排布（min-size 扩张 C3/free 恒等）+ sync_native_geometry DPI 排水（CoordMapper 桌面原点×GetDpiForWindow+标题条客户区内缩）+ 槽位框 chrome（标题条 min/close）+ C2 独占全屏拒绝 + C4 拖走/B7 回收事件臂 + B8 退出批量恢复；feature 阶梯 native-dock（windows optional+target 双门控，默认档零开销）/test-native-dock + tools/native-fixture 夹具（JSON-lines start/bounds/close）+ 真第三方进程 E2E 六测试；**486 native dock Phase 1.5 触发面已落地**：DragWatch 拖入手势会话（`ui/native_dock/mod.rs` 纯逻辑状态机 Idle→Watching→Over——注入式落点计算〔含 free-cell 命中/最近〕+30Hz 节流+rect 变化即时重发+T1 七测；win32 钩子增 MOVESIZESTART 六事件+GetCursorPos 光标采样）+ session 接线（`DesktopEvent::NativeDragOver` 物理域消息面〔E2E/headless 注入用〕+`DesktopSession.native_drag_watch/native_drag_over` 字段+renderer `drive_drag_watch`〔START 起会话/被拖窗 LOCATIONCHANGE 采样/END 终态→dock 执行臂或清 overlay〕+`native_candidate_logical` 级联占位抽取〔高亮即落点不变量〕）+ 落点高亮 overlay（`native_drag_over_element` 主色 18% 半透明+2px 描边，view 层栈槽位 chrome 之上）+ 投影协议升版 **v1.3**（`__wm_wins` 纖入 native 槽位条目 {wid:"N<slot>",title,native,icon,focused}——仅 Docked 态/App 条目 native 恒空串统一/指纹窗段扩 "N{slot}:0,"，vue 端对拍基线）+ 任务栏动词 `focus_native`/`close_native`（三处落点+N 前缀 arg 双形态容收；执行臂 SW_RESTORE+SetForegroundWindow best-effort / WM_CLOSE→DESTROY 自然回收 B7）+ shell.at dock 区 native 分支（title 文本按钮 max-w-32 truncate+×）+ T4 E2E 拖入/拖出（`drag_sim` 合成拖拽——SendInput caption 真拖主路径〔TOPMOST 置顶+AttachThreadInput 前台化+激活结算+70% 宽大窗抓点四要素，4K@200% 小窗 caption 按钮占宽过半实测教训〕+SC_MOVE|HTCAPTION 注入退路〔同入真实 move-size 循环〕）+ T5 实机冒烟（t5_smoke #[ignore] 手动驱动：B1/B5/B8 实机留痕、D1 ◐ Chrome 自移触发 C4、G2 附带实证；B6/C1/B9 仍待用户）+ P473 债务行清偿回写；债务 P486-1 事件泵吞吐（16ms 单事件/拍，系统噪声下 dock 落位秒级延迟）；**485 原生互操作 Phase 2 剪贴板已落地（三族全通）**：`ui/clipboard_native.rs`（纯 codec 层 DROPFILES/DIBV5↔RGBA 全平台可测 + Win32 双门控层——files_get/files_set（CF_HDROP，DragQueryFileW/GMEM_MOVEABLE DROPFILES）/image_get（CF_DIBV5→CF_DIB→registered PNG 三退路→temp PNG，64MP 防爆）/image_set（DIBV5+PNG 双挂）；feature native-clipboard=[dep:windows,dep:image]（ui-iced 隐含，windows dep features 与 native-dock 共条目扩列）；四 VM natives auto.clipboard.files_get/files_set/image_get/image_set（catalog 2934-2937，降级臂空表/false/null——.at 零平台分支）+ codegen bare-name intrinsics；GlobalClipboardTestLock 跨进程命名互斥（nextest 多进程剪贴板测试互清根治）；示例 043-clipboard-bridge（三卡实机演示，Explorer/截图工具/画图往返留痕）；OLE 拖放 Phase 3/真洞 Phase 4 待续（真人清单顺延 KD-P473-2；P481-6 实机 Ctrl+C 末步复验受阻于合成输入不达 winit raw-input 流，债务开放）。**494 原生真洞 Phase 4 已落地（Region 机制替换形态）**：双 spike 证伪原设计——透明 swapchain 本机不可行（wgpu HWND surface 仅 [Opaque] alpha；DxgiFromVisual/DirectComposition 能力解锁但内容不上屏〔疑 ToDesk 远程显示环境〕；色键分层破坏 flip-model 呈现）+ HTTRANSPARENT 跨进程证伪（MSDN 同线程文义——WindowFromPoint 不跳层、真实点击被丢弃）→ 机制替换为 **SetWindowRgn 洞排除**：`raise_desktop_above`（SetWindowPos(slot,desktop) 单步 z 翻转，473 sink 参数对调）+ `apply_hole_regions`（CreateRectRgn+RGN_DIFF 逐洞扣除+SetWindowRgn(berase=false)，空表复位）——洞区窗口不存在=视觉透出 z 下层+点击直达（OS 区域语义，无同线程限制）；模式位 `shell.native.hole` storage（默认 off，DesktopOptions 程序位取或）+ sync_native_geometry hole 分支（z 翻转+refresh_hole_regions 洞集重建=全部 Docked 槽位 slot_rect）+ 失败自动回退假洞（hole_mode 翻 off+473 z 全量重申+日志，`refresh_hole_regions_at` 可测核心+stale hwnd 实测）+ 退出清 Region；T1 纯逻辑（window_local_holes 裁剪换算+z 序插入模型）+ 真实测试（z 不变量〔首可见邻居断言——IME 伴随窗楔位实测〕/Region 穿透/复位）+ T3 E2E 铁证（洞心 SendInput 跨进程精确穿透 ±6/洞外零泄漏；夹具增 click 坐标日志+win32::test_support scratch 设施）；G4 覆盖层洞边裁剪（Region 代价）+T5 实机清单+透明路径复验=已批准债务 P494-1/2/3（物理机复验，AUTO_DESKTOP_HOLE=1 钩子）。**386 路线 B 桌面协议 Stage 1+2 已落地（v1.1）**：`ui/desktop_protocol/`
（五通道消息 + 二进制编解码 + 双端状态机 + 命名管道/共享内存传输 + broker
入口裁决 + L2 detach-attach，re-exec 两进程集成验证，状态保持经 revision
连续性证明）；spawn-client 双态启动 + broker 孵化中转就绪，live-iced 渲染
消费面换接与 Stage 3 多 App 内存实测归 shell-track/后续。设计源：
[Design 23/24/25](../../../design/autoui/README.md)、[桌面协议 v1.1](../../../design/autoui/desktop-protocol-v1.md)。。**480 路线 B Stage 3 已落地（v1.2，收官）**：真桌面壳孵化通道（`ui/desktop_protocol/client_runtime.rs` AppProjector 投影器 v1——AuraNode→DrawList text/button+线性堆叠+button 命中区+prop/FStr 插值代入 VM 状态；ClientPump 协议泵 step/run 双形态；`auto run --autodesk-client=<pipe>`/`--autodesk-incubate`/`--app386=<name>` 双模入口，无标记行为零改动；`DesktopSession::enable_broker` serve 线程 + ServiceTick 帧泵周期落地）+ BrokerClient 驻留多 App 宿主（`broker_clients` 表，N=3/5 压测全 Active/逐 App 点击帧递增/30s 存活）+ 弹性重连（EOF→预算内重试连回，VM 状态/revision 原地）+ L1 换窗（`detach_surface_to_os_window`/`attach_surface_back` 登记翻转，App/VM 原地）+ L3 v2a 快照迁移（`ControlMsg::StateSnapshot` tag 11 注入恢复，count/revision 连续）+ 内存边际增量基线（Private 4.81MiB/App 临界达标·WS 23.17MiB/App 未达标——度量+判定形态，`docs/plans/reports/480-memory-baseline.md`）+ 修复 recv_wait 丢消息/shm 段名跨进程撞名两真缺陷（`autodesk-shm-<pid>-<surface>`）。像素级投影保真与 live-iced 渲染器换接归后续。**500 Stage 4 已落地（v1.3，RenderQueue 命令帧臂）**：帧载荷二态（queue=DrawList 命令帧宿主栅格化 / independent=shm 像素帧纹理上传，同宿主同屏并存）+ per-App 三态开关 `desktop_render:`（spawn > manifest > auto 覆盖探测降级+观测行）+ `coverage.rs` 能力表/judge——详见 [desktop-protocol-v1](../../../design/autoui/desktop-protocol-v1.md) §1.3 与 specs.json P500-x。**507 Stage 5 已落地（覆盖爬坡 + parity 债闸门）**：aura.at 388 element 三级分级定稿（Tier1 40/Tier2 29/Tier3 not-yet 73 逐项裁定/n-a 246），AppProjector 爬坡至 Tier1+2 全量（**covered 69/388=17.8%**）——display 七员（icon/badge/avatar/progress/divider/separator/spacer）、form 四员（checkbox/switch/radio/textarea——Toggle 命中区〔handler 在场=handler 拥有状态变更〕+焦点 accent 描边+禁用乘暗不登记）、grid cols 等宽网格+card 表面缺省档、typography 缺省档（pre 族底盒/blockquote 引用条/small/heading 档；bold/italic 字重边界）+语义容器 16 员块流（折叠键贯通 normalize_kind/layout_node）；**第三端 parity 债自动闸门**：`aura/element_coverage.rs` 元素级登记表（单一事实源，无 feature 门）+ schema_drift 双向同步围栏（未登记即红/陈旧即红/覆盖率输出）+ covered⊆target_set 一致性钉 + parity 金样矩阵（覆盖表驱动 5 夹具×两阶段+防漏钉夹具并集⊇target_set，`test/parity/matrix/`）；**500 逃逸修正二枚**：容器 z 序（bg 先于子级——顺序栅格化下 bg 盖子级）与 Toggle 双翻对冲（实机 e2e 实证）；**日常门禁**：`cargo t` 携 `--features ui-iced`（tf 盲区收口）；构造示例 `examples/ui/p507-tier-coverage` 入实机 queue e2e。specs.json P507-1..6；债务 P507-1..3。**508 Stage 6 已落地（默认策略裁定 + 远程 command 流，RenderQueue 线收官）**：`shell.apps.process_model` 配置位（inproc|outproc，缺省 inproc 零变化；outproc=launch_app broker 孵化真子进程——re-exec spawn→broker 受理→同步泵 attach→registry_id 回填）+ G2 对比实测**裁定维持 inproc 缺省**（总边际 0.86 vs 7.64MiB/App、启动 2–17ms vs 25–250ms、交互 0.07 vs 1.54ms；outproc 留隔离选项，翻转三闸 T-覆盖/T-稳定性/T-远程，`docs/plans/reports/508-process-model-verdict.md`）+ `WsTransport`（Transport 第五实现：tokio-tungstenite 0.30 no-default-features，WS Binary=codec 信封原样，token 升级期 query 校验 401 终态拒收）+ 远程镜像会话（`remote.rs`：`127.0.0.1:17800` 缺省不监听、`shell.remote.token` 缺省拒绝；Hello→Welcome+HitTable〔tag9 纯追加〕→帧推送，输入路由同 broker 收尾；`PROTOCOL_VERSION` 仍 1，pipe/loopback 零改动）+ `packages/drawlist-renderer/`（TS/Canvas2D：codec/messages/render/connect 四模块，Rust↔TS golden 双侧对拍防漂移，vitest 20 绿）+ `examples/remote/viewer/`（vite demo + e2e：T4 Playwright 浏览器渲染 002-counter 点击闭环 PASS、T6 真桌面 outproc↔headed Chromium 双向闭环截图×3）。specs.json P508-1..6。

**vue 轨（codegen 正确性 + 工程化）**：444 修五类 vue-tsc 缺陷（回调通道/emits 名册派生/变体断言）；
443 defineModel 降级收窄（bound_model_channels 预扫）；451 actions/menubar/toolbar/快捷键消费补进
codegen；457 ~60 个 shadcn 组件编译期内嵌 auto-man（冷启动离线化）；437 chart 族声明驱动发射
（SVG 直通三端同源，借 442 A4）。

**VM 轨（视觉 parity + 运行时对齐）**：450/451-image/452-login 逐项拉平 shadcn 语义（圆角 SDF 掩膜、
Text 盒模型、按钮 14px/500、input 透明背景）；455 双端 parity 跟踪器把标准下沉为引擎规范（focus
ring 2px、margin 盒模型），矩阵约 9 绿 / 8+ 待审计；458 theme/accent 一等配置（CLI/pac.at/env
三通道，双端默认 dark+indigo）；446 清偿实战上报的渲染薄弱点（A1 多 store 消歧编译期报错、
J1/J2 渲染器子树，批五转正中）。

**DSL 与内建 widget**：425 component fn 糖化退役双轨（widget 单轨）；426/436 setup{} 三相位语义
（vue 落地、解释器 L1、a2r 显式报错）；448 msg 去名 + 内联 lambda 简写；413–428 code_editor 全链
（自研 cosmic-text ViEditor、折叠逐 run 管线、多 tab、vue CodeMirror 契约对齐）；449 确立 VM 组件
写法边界（三缺口登记：回调 props 退化/快照子树不可见/片段条件不求值）。

**示例轨道**：examples/ui 024-charts（437/445）、025-dashboard（438）已交付；026-database（439）、
027-file-manager（440）草案可领取；028-launcher 归 464。

**481 展示型文字选择/复制已落地**：text/label 增 `selectable` 属性（bool,默认 false——opt-in,缺省渲染路径逐行零变化）;VM 端自研 `ui/iced/selectable_text.rs` SelectableText widget(advanced Widget——绘制复用 iced `text` 同参同路径保证逐像素一致,命中走 iced_graphics Paragraph 公开 `buffer()`;手势集 v1=拖选/双击词选(字符类分段词界,UAX#29 默认 CJK 连字)/Ctrl+C 写剪贴板(有选区才捕获)/Esc 清除不夺全局流;选区为 widget 本地状态,零桌面集成改动) + `selection.rs` 选区纯逻辑(全平台单测);vue 端显式化 `style="user-select: text"`(plain/shadcn 双路径);a2vue 金样 011 锁 prop 往返;001-helloworld/004-profile-card 点亮。边界:font-mono 代码文本 v1 保持 Rich 高亮不可选;arboard 兜底未启用(iced 剪贴板恒可用)。

**504 示例桌面化三件套（011-calculator 样板）**：pac.at `window: "fit"` 自适应窗口——独立 VM 窗首帧按内容 shrink 测量后 resize（clamp [200, 可用区]）、桌面虚拟窗以测量值替代"可用区 60%"写死初值（`register_window` 覆盖保留 fit_pending 为关键修复）；title/settings 上移 os-config per-app 配置（`~/.config/autoos/apps/<app>/config.at`，launch 直读文件注入 theme/accent——启动期 daemon 可能未起，不经 daemon；`modules.d/<id>.at` 注册后通用编辑器零手写获得设置 UI）；应用内 `ExampleHeader` 退役，标题由 pac.at `title:` + 桌面 chrome 提供。债项 P504-1..4 见 KNOWN-DEBT。
**506 示例桌面化批一（011 样板批量展开 7 例）**：三件套向首批示例批量兑现——header 退役线（008/009/010）：`common/header.at`（ExampleHeader 组件包）整体删除，app.at 无 header/settings，`dark_mode`/`accent_color` 声明保留作 os-config/env 播种挂钩（宿主 `seed_app_config` 与 renderer env 播种均要求已声明变量，删声明即静默失效）；theme/accent 注册 os-config per-app 配置（`~/.config/autoos/modules.d/auto-<app>.at` + `apps/<app>/config.at`，shape 循 calculator 先例 theme/accent 两键），优先级链 CLI > os-config > pac.at > 内置。fit/title 线（002/003/012/038）：pac.at 补 `title:` + `window: "fit"`，根容器 `center` 居中外壳拆除改"内容即页面"（四 app 实测均有 center，012 另删 min-h-screen），VM 独立窗实测收缩 400x400/400x720/550x774/647x878（默认 1293x836），fit 断言范式 = VM 截图 PNG 像素尺寸 < 900。测试改法范式：双端脚本删 settings 交互、增"无 header 元素 + 内容标记"断言（循 504 test_011）。债项 P506-1（038 Reveal 触发 VM RC use-after-free，master 预存疑 511 回归）/P506-2（MCP rendered-vtree 快照无事件注记，038 改 label 定位法）见 KNOWN-DEBT；批二 = 剩余 20 例 title 债务（001/004-007/013-025/028/042）。
## 关键入口

- `dialect/ui.rs:UiDialect` · `aura/extract.rs` · `aura/schema_loader.rs`（契约源自 `schema/aura.at`）
- `ui_gen/vue.rs:VueGenerator` · `ui_gen/api.rs` · `ui_gen/widget/registry.rs`（widget/chart 契约表）
- `ui/widget_registry.rs` · `ui/render_support.rs` · `ui/event_router.rs` · `ui/aura_view_builder.rs`
- 桌面线：`ui/session.rs`（DesktopSession/AppSession + WmState/WmCommand/DM::Wm）·
  `ui/iced/virtual_window.rs`（VirtualWindow）· `ui/iced/renderer.rs`（view_desktop_fn/run_dynamic_iced_multi）·
  `ui/desktop_protocol/`（路线 B 桌面协议 v1.1：五通道/传输/shm/broker/状态机）
- 样式与主题：`ui/style/`（class/theme/iced_adapter）· `ui/action_config.rs`（actions 配置层，
  热重载/OS keymap/表达式条件）
- 内建编辑器：`ui/code_editor/` · `ui/autodown_editor/` · `ui/handler_codegen.rs` · `ui/hot_reload.rs`
- `ui/mcp_server.rs`（AutoUI MCP 调试服务）· `a2ui/schema.rs:A2UIMessage`

## 使用示例

```auto
widget Counter {
    setup { greet = fn() { print("hi") } }   // 每实例前导（426）
    msg { Inc, Dec }                          // 448 起可去名
    model { count int = 0 }
    view { col { button + { onclick: .Inc } h2 > Count: ${.count} } }
    on { .Inc => { .count += 1 } }
}
```

## 已知坑

- **测试 storage/管道全局态卫生（Plan 489，P487-2 收敛）**：两条铁律——
  ①凡断言「键缺席回退默认」或落盘链的测试，一律 `t2_isolate_storage` 隔离
  （`AUTO_VM_STORAGE_FILE` 指临时文件；`storage_raw_remove` 只清内存，
  `storage_load` 会把盘上键并回——实机桌面用过设置面板后 store 必含
  `shell.dock.*`，487 写回链生效即打破无隔离测试的前提）；②broker/管道类
  测试一律 pid 后缀管道（`adjudicate_on` 参数化缝 / `Broker::on_pipe`），
  禁止依赖生产固定管道 `autodesk-broker` 的全局命名空间状态（本机任何
  桌面宿主 listen 即打穿 Standalone 断言——间歇红根源）。i18n corpus 的
  `front/i18n/{lang}.json` 属必备资产（.gitignore `*.json` 母规则需
  `!test/**/i18n/*.json` 否定）——落测不入库则 fresh clone 必红。
- **VM input 焦点寻址（Plan 483）**：VM 轨 text_input 每框派生唯一稳定 Id
  （`derive_input_id`：on_change 的 widget+event 主键、placeholder/width/
  password 三元组兜底），渲染期 `collect_input_ids` 依 DFS 序登记进
  per-App `devtools.input_ids`；自动聚焦路径（`__focus_input` 约定/未捕获
  Tab→`__focus_prompt`/PromptBar refocus/launcher 召唤）一律按登记表取首个
  input Id 寻址——**严禁**回退「全窗固定 `prompt_input`」写法（iced Focus
  operation 对同 Id 的全部 focusable 一次全置焦 ⇒ 多 input 视图双焦点+
  键盘双投递，上游 auto-musk 011）。焦点跨重建由 iced Tree 槽位 diff 保持
  （Plan 047 on_submit 语义不受影响）。
- **VM input 焦点环遍历（Plan 491，483 登记表之上的扩展）**：未捕获 Tab/
  Shift+Tab 由 `keyboard_event_message` 按 `modifiers.shift()` 分派
  `__focus_next_input`/`__focus_prev_input`（Named 臂无修饰前缀——Shift+Tab
  与 Tab 同命中 "Tab"，必须在此分流）；update 遍历臂走两段链
  `operate(FindFocusedInput).then(focus_traverse→内建 focus)`——探针经
  Operation 遍历读**实际**持焦者（含点击直聚；无聚焦恒出 `Some(None)`，
  异于内建 `find_focused` 的 `Outcome::None` 断链），`focus_traverse` 按登记
  表 DFS 序回环求址（不在表内/无聚焦→首项，空表 None）。**登记表空（纯
  textarea 视图，ash-gui/028）回落 057 `__focus_prompt` textarea 优先链**。
  机制级 p491 七测（iced_test）；真键盘实机复验在 P483-3 真人清单（环境
  OS 键盘注入对 winit 不可达）。
- **桌面热键表数据级可配置（Plan 490）**：桌面级热键改查 `HotkeyTable`
  （session.rs——HotkeyAction 11 动作/KeySpec 解析/builtin 新默认 +
  `shell.keys.<action>` storage 覆盖，472 dock 配置同型；boot 期
  load_hotkey_overrides 读入，坏值静默回退）。**新默认**：Alt+Tab 退役
  （G1，Win11 系统保留；`shell.keys.cycle_window` 显式配置可复活）；
  分区切换 Ctrl+Alt+←/→ 迁 **Ctrl+Alt+[ / ]**（G2，Intel 核显旋转键
  冲突族；方向键可覆盖恢复）；launcher 主键 Ctrl+Space + **别名
  Ctrl+Alt+Space 双收**（中文 IME 抢键机实测——入口按钮文案标注）。
  订阅臂链 = 纯函数 `desktop_hotkey_message`（可测化内核）。
- **VM 轨布局件点击 parity（Plan 490 G4）**：`row/col/div(container)`
  挂 `onclick` 在 VM 轨此前被转换层**静默丢弃**（Vue 轨 onclick→@click
  泛映射已通）——028 launcher 候选行鼠标点不中的根因。修复三层：
  `View::Row/Column/Container` +`onclick: Option<M>`（map/convert 语义
  穿透）+ aura 分发点 `set_layout_onclick` 提取（tracked/untracked 六臂，
  沿 text onclick→Button 先例）+ `wrap_layout_onclick` mouse_area 包装
  （on_release 发射；inspect 模式自守卫）。**严禁**在转换层丢弃布局件
  事件声明——新增布局事件（hover/右键）走 View::MouseArea（484）或
  同型扩展。
- VM 组件三缺口（449 实测）：回调 props 退化（组件一律无 props 读 store 规避）、快照组件子树不可见、
  片段参数化条件不求值——修好前 041 组件化受限。
- 446 批五（U2–U6）在 worktree 待 merge；463 合入后需实测内存给 386 提供数据。
- 455 矩阵多数示例（008–025）仍 Pending Audit；示例矩阵编号与目录有漂移（todo 在计划内）。
- 桌面线由并行会话活跃推进：WM 无独立 wm.rs 文件（WmState/WmCommand 实体在 session.rs，Plan 471 实测
  校正）；virtual_window 的 schema/aura.at 契约登记（462-I4）master 暂未见，随桌面线收尾合入。
- 465 未落地前，vue codegen 的 modal `position:fixed`/teleport-to-body 假设与虚拟桌面容器冲突（改造点已定位 vue.rs:6310/3599/4057）。
- a2vue 工程庞大（vue.rs 万行级），缺陷修复走 444 式"五类分簇"模式；013/015/011 构建失败系 master 预存（R006/R007）。
- router 双语法并存（Plan 105/106，见 docs/router.md）。

**505 桌面 DEBT 批处理一期（四族清偿）**：A 交互时序——原生槽位事件泵单发
16ms try_recv 改 `drain_slot_events` 每拍排空 + MoveSizeStart/End 稳定分区
前置（快甩同批即判；`NativeSlotEvents` 批消息形态）；B shell 面五瑕疵——
shell.at 任务栏双分支收敛 flex-col-reverse 单份 + `__dock_border` 宿主投影
边线、投影协议 v1.5（`pager` 旗标 + `more` "+N" 派生面）、a2vue 注册件
props 透传、daemon 发现序三级 PATH、`shutdown_broker` 五退出点；C 实机
验收通道——`autoui_desktop` MCP 注入（DesktopInject 队列走真实按钮同一
消费臂，AUTOUI_ACCEPTANCE=1 门控）+ ADR 规程 + acceptance_channel.py 统一
入口，P487-1/P496-1/P501-2 三债实机照补拍归档；D P488-D4 on_dnd_finished
发起方锚定 + 壁纸热切换定案（天然支持）。债项 P505-1/2 见 KNOWN-DEBT。

## 蒸馏来源

- 本模块 spec 于 2026-08-28 由 Plan 471 刷新：蒸馏 437–465 活跃计划 + 4xx 归档计划 + 365–428 早期 UI 计划。
- 设计层：[Design 20（分离架构）](../../../design/20-autoui-separation-architecture.md)、
  [Design 16（App 生成战略）](../../../design/16-app-generation-and-ai-authoring.md)、
  [design/autoui/](../../../design/autoui/README.md)（虚拟桌面三部曲）。
- 过程记录：`docs/plans/plans.md 索引表` + `docs/plans/KNOWN-DEBT-AND-RISKS.md`（445/449/414/422/444 条目）。
