# AutoShell 状态投影协议 v1.4（S2 接缝合同）

> **版本**：v1.4（2026-08-30，Plan 487 M4 落码；486 先合占 v1.3，487 按
> 并行协调叠 v1.4——v1/v1.1/v1.2/v1.3 见 §6
> 变更记录）。双端同
> 版本：vm 端（auto-lang `ui/iced/renderer.rs::sync_shell_windows`，本版实现
> 方）与 vue 端（465/shell-track 后续，按本文档实现同版本对拍）。
> **定位**：Design 25 §3 S2 的正式化——宿主把驱动事实投影为 shell App 的
> 响应式状态；本文件是与 `schema/aura.at` 并列的机器可对拍合同。
> **上位词表**：S1 命令下行（DesktopBus）的动词词表见本文 §4。

## 1. 命名与权属

- 宿主注入 shell 的响应式状态一律 `__wm_*` 前缀（双下划线 = 宿主特权命名
  空间）。shell .at 声明这些变量并**只读消费**；写权属在宿主。
- shell 出向唯一写 `__desktop_cmd`（§4 命令总线）。
- 类型约束：平铺可 for 循环数组（宿主 `write_state_vec` 注入 Obj 数组；
  消费侧在 **view** 的 for 循环——VM handler 内对注入 Obj 数组的字段读有
  B12 族已知缺陷（464 探针），需要 handler 消费时用平行字符串列表）。

## 2. 字段表（shell.at model 声明同款）

| 字段 | 类型 | 语义 | 权属 | 引入 |
|---|---|---|---|---|
| `__wm_wins` | Obj 数组 `{wid:str, title:str, focused:str, workspace:str, app:str, icon:str, native:str}` | **全部**虚拟窗（跨 workspace 全集，dock 运行指示消费；可见性由宿主绘制层自过滤）。`focused` = `"1"/""`；`workspace` = 分区下标串；`app` = 注册表 id（boot 窗缺省 `""`）；`icon` = lucide 名（注册表实时查 → 缺省 `"app-window"`）。**v1.3 native 槽位条目**：Docked 原生窗口追加在尾部，字段集 `{wid, title, focused, native, icon}`（workspace/app 不适用省略）——`wid` = `"N<slot_id>"` **独立编码空间**（`N` 前缀 + 十进制槽位 id，与 App wid 的纯数字空间隔离；shell 侧零解析成本区分两类条目）；`native` = `"1"`（App 条目恒空串，分支判据统一）；`focused` 恒空（native 焦点域在 OS 层，WM 不代管）；`icon` 占位 `"app-window"`（HICON 提取为增强候选） | 宿主写 | v1（native 条目/`native` 字段：v1.3） |
| `__wm_meta` | str `"layout\tfocused_wid"` | 布局名（free/grid/master-stack）+ 焦点窗 wid（无焦点空串） | 宿主写 | v1 |
| `__wm_workspaces` | Obj 数组 `{id:str, name:str, current:str, label:str}` | 分区清单；`id` = 下标串；`name` = pack 默认 "Desktop N"（M4 settings 可覆盖）；`current` = `"1"/""`；`label` = 1 基人读标签（= id+1 十进制串；**宿主投影**，避开 .at 字符串算术——pager 按钮文本消费） | 宿主写 | v1（label：v1.1） |
| `__wm_mru` | Obj 数组（条目同 `__wm_wins` 六字段） | **当前分区**的窗口按 MRU 序（front = 最近聚焦；退役 Ctrl+Tab 焦点环语义延续——焦点环不跨分区，472 定案）。switcher overlay 专用消费面，dock 消费不受影响。switcher **handler** 侧消费走宿主召唤时的伴随平行字符串列表（`mru_wids`/`mru_titles`/`mru_icons` + `call_handler("RebuildMru")` 建 handler 自有 rows，B12 规避——464 launcher `apps_*`/`ranked` 同型；`__wm_mru` 本体保持合同面对拍形态） | 宿主写 | v1.1 |
| `__wm_running` | str `",id1,id2,"` | 运行中 app id 集合的**派生串**（pinned 运行指示的 view 条件消费——.at 无法跨列表聚合，宿主派生保持 I9 单一事实源；T4 增补） | 宿主写 | v1 |
| `__wm_notes` | Obj 数组 `{id:str, kind:str, msg:str, at:str}` | **通知历史全量**（MRU 序 front=最新；容量 50 FIFO）。shell 侧为合同面（dock 不直接消费）；通知中心面板 handler 消费走召唤/活更新时的伴随平行字符串列表（`note_ids`/`note_kinds`/`note_msgs`/`note_ats` + `call_handler("RebuildNotes")` 建 handler 自有 rows，B12 规避——`__wm_mru` 同型）。`kind` ∈ success/error/info（约定值，未知宿主侧已兜底）；`at` = 入史时刻 `HH:MM` 本地时间串（宿主投影） | 宿主写 | v1.2 |
| `__wm_notes_unread` | str | 未读通知计数十进制串（dock 铃铛 badge 条件消费：`!= "0"` 且非空串渲染）；开面板即清零；不落盘——boot 恢复后恒 `"0"` | 宿主写 | v1.2 |
| `__wm_fp` | str | 投影指纹（§3）；shell 不消费，仅门控 | 宿主写 | v1 |
| `__wm_clock` | str `"HH:MM"` | dock 时钟本地时间串（497 S3）。**非门控字段**：不进 `__wm_fp` 指纹、不走 §3 投影组换装——ServiceTick 帧泵独立注入（分钟变化才写，稳态零重建；本地时钟非驱动事实，避免每分钟全组换装抖动） | 宿主写 | v1.4 内（497） |
| `__desktop_cmd` | str | 出向命令记录串（§4）；宿主**读+清** | shell 写 | v1 |

### 2.1 桌面本体面字段（`assets/desktop.at`，v1.4 内字段扩展——不升版本段）

Plan 496 M5 的第五面（常驻不召唤，boot 装载挂桌面层 z 槽）。命名族
`__desktop_*`（与 `__wm_*` 同为宿主特权命名空间；boot 期一次注入，无指纹
门控——数据源 storage 键 boot 读一次，会话内不变）：

| 字段 | 类型 | 语义 | 权属 | 引入 |
|---|---|---|---|---|
| `__desktop_bg` | str | 壁纸色值类片段：`shell.desktop.wallpaper` 为 `#hex` 时注入 `"bg-[#hex]"`（面根 bg 实铺）；图片路径/缺省时注入 `""`（图片壁纸由宿主在面之下推壁纸图层——DSL 无重叠布局，z 序宿主侧兑现） | 宿主写 | v1.4 内（496） |
| `__desktop_icons` | Obj 数组 `{id:str, icon:str, label:str, src:str}` | 桌面条目 = pinned ∪ 自定义合并去重（pinned 先列；`shell.desktop.icons` 逗号串）再排除 hidden（`shell.desktop.hidden` 逗号串，pinned/custom 通用移除位）。`icon`/`label` 注册表解析（缺省 `app-window`/id）；`src` = `pinned`\|`custom` | 宿主写 | v1.4 内（496） |
| `__desktop_hidden` | str | 排除 id 逗号串（移除臂续写 `shell.desktop.hidden` 的当前值底稿） | 宿主写 | v1.4 内（496） |

## 3. 更新语义与指纹门控（协议条款）

- 宿主每 update 周期在 DesktopBus 排空点邻位重算投影（O(窗数) 串接）。
- `__wm_fp` = 逐窗 `"{wid}:{focused},{workspace};"` 串接（**v1.3**：native
  槽位条目并入同段，`"N{slot}:{focused},"` 同型追加在 App 窗之后——槽位
  增删/瞬时态转 Docked 必翻指纹）+ `"|{__wm_meta}"` +
  `"|"` + 逐分区 `"{id}:{current},{label};"` 串接 + `"|"` + 逐 mru 窗
  `"{wid};"` 串接 + `"|notes:{len}:{front_id}:{unread};"`（v1.1：分区段扩
  label、尾接 mru 段；v1.2：尾接 notes 段——len/front_id 双段覆盖容量环绕
  与 dismiss 组合、unread 独立第三段，任何历史/未读变化必翻其一）。
- **指纹未变 → 整组跳过写**（防每帧 churn，不置 dirty）；**有变 → 整组原子
  换装**（wins/meta/workspaces/mru/fp 全写）+ shell `view_dirty` 置位触发
  重渲染。投影无部分更新。
- 宿主写状态不触发 shell handler（464 实证）；投影消费一律在 view 侧，
  需要 handler 参与的表面（switcher RebuildMru 等）由宿主显式 `call_handler`。

## 4. S1 命令下行：DesktopBus 动词词表

传输载体 = 463 实装候选 B（`__desktop_cmd` 状态变量总线；T1 施工图对账定案，
Design 25 §3 原"候选 A 转正"修订为词表规范，builtin 语法化留 v2——触发条件：
命令需返回值/类型化参数）。编码：`verb\u{1F}arg` 单记录，`\n` 或 `\u{1E}`
连多条；shell.at 控件字符串只可直书 `\t`/`\n`，宿主两套分隔符等价接受；
未知动词/坏记录跳过不 panic（前向兼容）。

| 动词 | 载荷 | 宿主语义 | 引入 |
|---|---|---|---|
| `launch` | app id | 注册表启动新实例 | 463 |
| `close` | wid | 关虚拟窗 + App | 463 |
| `focus` | wid | 聚焦置顶 | 463 |
| `layout` | free/grid/master-stack | 全场重排（当前分区） | 463 |
| `summon` | launcher | 召唤 launcher overlay | 464 |
| `workspace` | 分区下标 n | 切换当前分区（窗口随分区隐现，全保留） | 472 |
| `workspace_next` | （无参记录） | (current+1)%N 环切 | 472 |
| `activate` | app id | dock 固定图标点击：运行中 →（窗在隐藏分区先切分区）聚焦其窗；未运行 → launch | 472 |
| `workspace_add` | （无参记录） | 新增分区（追加尾部）并即入新分区（pager `+`） | 478 |
| `workspace_close` | 分区下标 n | 删除分区（窗口重排相邻前驱 + 下标压实 + current clamp）；**宿主策略门**：非空分区 toast 提示不删、末分区保底（≥1） | 478 |
| `send_to` | wid、分区下标 n | 跨分区发送窗口（归属迁移、焦点让渡、随分区隐现） | 478 |
| `notify` | kind、msg | App 主动请求通知——入史 + 未读 +1（面板可见时不加）+ toast 浮现三联动（`push_notification` 单入口，持久化落 `shell.notes.0..9`）。约束：**msg 单行**（记录层按 `\n` 切分）；kind 约定 success/error/info，未知值浮现面按默认配色、面板按 info 兜底 | 479 |
| `notes_toggle` | （无参记录） | 通知中心面板开合（dock 铃铛钮；宿主臂落 `toggle_notification_center`：懒挂载 → 快照注入 + RebuildNotes → 打开即未读清零；可见再拨即自隐） | 479 |
| `notes_clear` | （无参记录） | 清空通知历史 + 落盘（面板「全部清除」） | 479 |
| `notes_dismiss` | 通知 id | 按 id 删除单条通知 + 落盘（面板「逐条 ×」） | 479 |
| `focus_native` | slot id 或 wid `"N<slot>"` | 任务栏 native 条目点击：聚焦槽位原生窗——最小化先 `SW_RESTORE` 再 `SetForegroundWindow`（best-effort，前台锁拒绝不视为错误）；arg 容收两形态（shell 直传条目 wid，宿主剥 `N` 前缀归一） | 486 |
| `close_native` | slot id 或 wid `"N<slot>"` | 任务栏 native 条目 ×：`PostMessageW(WM_CLOSE)` 正常关闭机会；槽位由 DESTROY WinEvent 自然回收（B7 路径），动词本身不移除槽位 | 486 |
| `open_settings` | （无参记录） | 设置面板开合（dock 齿轮钮；宿主臂落 `toggle_settings`：懒挂载 → 配置快照注入（cfg_*/pinned_ids/about_*）+ RebuildPinned；**二态翻转**——可见再拨即自隐，Esc 同效。发件面 = shell.at 齿轮） | 487 |
| `set_dock_position` | `top`/`bottom` | dock 位置**热切换**（I7：几何是驱动事实，settings 面板只是 UI——发件面 = settings.at 位置单选）。宿主臂 `execute_set_dock_position`：storage 键 `shell.dock.position` 写回 → `dock_edges` 键重推导（boot 同函数）→ `apply_layout` relayout + 槽位排水 → shell `__dock_*` 投影热同步 | 487 |
| `set_dock_enabled` | `1`/`0` | dock 启用开关**热切换**（发件面 = settings.at 开关；`0` = 零预留，位置键保留——重开按原位置恢复）。宿主臂 `execute_set_dock_enabled` 同上三联动，写回键 `shell.dock.enabled`（`"true"/"false"`） | 487 |

## 5. 对拍与验收（I8/I9）

- vm 端实现金样：`ui/iced/renderer.rs` tests `projection_*` 七测（v1 往返/
  指纹门控/分区切换反射/registry icon + v1.1 mru 序与 label/mru 过滤与
- vm 端实现金样：`ui/iced/renderer.rs` tests `projection_*` 七测（v1 往返/
  指纹门控/分区切换反射/registry icon + v1.1 mru 序与 label/mru 过滤与
  指纹段 + v1.3 native 条目与指纹段）+ `notif_*` 八测（v1.2 动词往返/
  历史 FIFO/未读语义/持久化槽
  round-trip/双面一体/面板召唤/notes 投影与指纹段）+ `settings_*` 七测
  （v1.4 动词往返/装载导航/执行臂热应用/召唤注入/Dock 派发与 pinned
  落键/通知门控/齿轮链路）+ `desktop_surface_*` 三测（496：T2 storage
  往返+壁纸解析 / T2 合并去重注入 / T1 装载派发——§2.1 字段族）+
  iced-layout-tests `desktop_surface_z_slot_window_covers_icons`（T3 层级）。
- a2vue 双端同源金样：`ui_gen/vue.rs` tests `test_a2vue_desktop_surface_asset`
  （496 I8：真资产 desktop.at → SFC 对拍，ondblclick/@contextmenu 事件面）。
- 动词编码往返金样：`ui/session.rs` tests `native_dock_verbs_parse_and_
  encode`（dock_native/undock_native + v1.3 focus_native/close_native
  双形态 arg）。
- vue 端（465 后续）按本表实现同版本投影 + 同指纹规则，对拍项登记后
  消费本文件作基线；版本升级 = 文件名/版本号 + 双端同步 + 对拍重跑。
- I7（shell 无几何操作）、I9（窗口/分区列表唯一事实来自本投影）随行。

## 6. 变更记录

### v1.4 内字段扩展（2026-08-31，Plan 497 S3——不升版本段）

- **§2 字段表增 `__wm_clock`**：dock 时钟本地 `HH:MM` 串——唯一**非门控**
  注入字段（不进指纹、不走投影组换装；ServiceTick 分钟变化才写）。
- **零新动词/零指纹变化**；快照缩略数据不经投影（`mru_thumbs` 平行
  字符串列表为召唤快照注入，同 `mru_icons` 通道——像素资产宿主侧
  `ui/iced/snapshot.rs` 缓存直取，T1 定案裁剪式整窗快照）。

### v1.4 内字段扩展（2026-08-31，Plan 496 M5——不升版本段）

- **第五面**：`assets/desktop.at`（桌面本体——壁纸/图标网格/入口；常驻
  不召唤，boot 装载挂桌面层 z 槽：壁纸层之上、App 虚拟窗口之下）。
- **§2.1 字段族** `__desktop_bg`/`__desktop_icons`/`__desktop_hidden`
  （boot 一次注入，无指纹门控）。**零新动词**——复用 `activate`（双击/
  菜单打开，472 两臂）与 `open_settings`（v1.4，更换壁纸入口）。
- **storage 键增量**：`shell.desktop.wallpaper`（路径|#hex，settings 外观
  分区写手）、`shell.desktop.icons`（自定义条目 id 逗号串）、
  `shell.desktop.hidden`（排除 id 逗号串）。三者均 boot 读一次生效
  （487 非几何无动词判定同款）。

### v1.4（2026-08-30，Plan 487 M4）
- **新增动词** `open_settings` / `set_dock_position` / `set_dock_enabled`
  （§4，词表 v1.4）：设置面板召唤（dock 齿轮，二态翻转）+ dock 位置/开关
  驱动动词（I7：几何是驱动事实——宿主臂热改 `dock_edges` + relayout +
  storage 键写回三联动，boot 读路径同键保持一致）。
- **第四枚 overlay 槽**：`assets/settings.at`（Dock/通知/关于三分区）。
  召唤时快照注入 `cfg_dock_position`/`cfg_dock_enabled`/`pinned_ids`/
  `cfg_notes_enabled`/`about_host`/`about_version`（B12 规避平行列表 +
  常量，挂召唤注入通道——**无新 `__wm_*` 投影字段**）。
- **storage 键增量**：`shell.notes.enabled`（通知持久化开关，`"false"` =
  关——479 消费链 `push_notification` 单点门控；缺席/其余 = 开，向后
  兼容）。`shell.dock.pinned` 获 UI 写手（settings 面板行内增删直写，
  格式不变）。
- **向后兼容声明**：纯增量动词/storage 键，零新投影字段、零指纹变化——
  v1/v1.1/v1.2/v1.3 消费者零破坏；旧 store 键缺席时行为全同（通知门控
  缺席即开）。
- **版本协调注记（Plan 486 并行）**：486（触发面）与 487 并行加动词——
  协调规则先合者占 v1.3 后合者叠 v1.4；**合并实况：486 先合占 v1.3
  （focus_native/close_native + native 条目），487 叠 v1.4**（本条目
  即该规则执行结果，落码时预写的「487 占 v1.3」按实况改编）。


### v1.3（2026-08-30，Plan 486）

- **`__wm_wins` 纳入 native 槽位条目**（§2）：Docked 原生窗口以
  `{wid:"N<slot>", title, focused, native, icon}` 五字段集追加在 App 窗
  之后；`N` 前缀 wid 编码空间与 App wid 隔离；App 条目统一增 `native`
  恒空串字段（shell 分支判据 `w.native == "1"`，避免缺失字段访问）。
- **新增动词** `focus_native` / `close_native`（§4，任务栏 native 条目
  点击/×；arg 双形态容收）。
- **指纹扩展**（§3）：窗段并入 `"N{slot}:{focused},"`（native 条目同型）。
- **向后兼容声明**：纯增量——既有消费者零破坏（App 条目仅多一个恒空串
  字段，for 循环字段读不受影响；native 条目仅 Windows 宿主产生）。vue
  端（未实现）以本版为对拍基线；文件名不变，双端同步对拍在 vue 端落地
  时执行（§5）。

### v1.2（2026-08-29，Plan 479 T4）

- **新增投影** `__wm_notes` / `__wm_notes_unread`（§2）：通知历史全量
  {id,kind,msg,at} Obj 数组 + 未读计数串（dock 铃铛 badge 消费）；面板
  handler 侧消费走伴随平行字符串列表（`note_*` + `RebuildNotes`，B12
  规避注记同 `__wm_mru`）。
- **新增动词** `notify` / `notes_toggle` / `notes_clear` / `notes_dismiss`
  （§4，词表 v1.2；notify 含 msg 单行约束）。
- **指纹扩展**（§3）：尾接 notes 段 `"|notes:{len}:{front_id}:{unread};"`。
- **向后兼容声明**：纯增量字段/动词——v1/v1.1 消费者（dock 任务栏、布局
  键、pinned 运行指示、pager、switcher）零破坏；`__wm_notes` 对 shell
  为合同面（dock 不消费），`__wm_fp` 为门控内部串。vue 端（未实现）以
  本版为对拍基线；文件名不变，双端同步对拍在 vue 端落地时执行（§5）。

### v1.1（2026-08-29，Plan 478 T3）

- **新增投影** `__wm_mru`（§2）：当前分区 MRU 序窗口清单，switcher overlay
  专用；伴随平行字符串列表形态注记（B12 规避）。
- **`__wm_workspaces` 条目增可选字段 `label`**（1 基人读标签，宿主投影）。
- **新增动词** `workspace_add` / `workspace_close` / `send_to`（§4）。
- **指纹扩展**（§3）：分区段 `"{id}:{current};"` → `"{id}:{current},{label};"`；
  尾接 mru 段逐窗 `"{wid};"`。
- **向后兼容声明**：纯增量字段/动词——v1 消费者（dock 任务栏、布局键、
  pinnned 运行指示）零破坏；指纹串整体换装对 v1 消费面透明（`__wm_fp`
  为门控内部串）。vue 端（未实现）以本版为对拍基线；文件名不变，双端
  同步对拍在 vue 端落地时执行（§5）。

### v1（2026-08-29，Plan 472 T3）

- 首版：`__wm_wins`（v1 六字段）/`__wm_meta`/`__wm_workspaces`/`__wm_running`/
  `__wm_fp` 字段表 + 指纹门控 + DesktopBus v1 动词词表（launch/close/focus/
  layout/summon/workspace/workspace_next/activate）。
