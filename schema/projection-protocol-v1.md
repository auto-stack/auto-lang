# AutoShell 状态投影协议 v1.9（S2 接缝合同）

> **版本**：v1.9（2026-09-18，双增量并行协调叠号——486/487 先例）：
> v1.8 = auto-os PLAN-014 shell-ux-polish-v2（纯字段 + 总线语义增量、
> **零新动词**——`__wm_date`、`__wm_running` desktop 扩注、
> `__wm_notes[].app`/`note_apps`、`__wm_notes_badge`、`__desktop_cmd`
> 追加语义、`__desktop_icons[].color`/`__desktop_cells[].full` 审计
> 补记，详见 §6 v1.8 节）；
> v1.9 = auto-os PLAN-024 dashboard 常驻小组件层（入向
> `__dashboard_faces`/`__wm_dashboard`/几何注入 + 出向
> `__dashboard_cmd` 六动词词表（toggle/close/pin/unpin/span/launch）+
> `__dashboard_open` 合成消息 + `shell.dashboard.*` storage 键空间，
> 详见 §6 v1.9 节）。
> v1.7（2026-09-14，双增量并行落码——PLAN-016：`open_with` 打开
> 文件动词（普通注册表窗上行排空面 + pac `opens:` 关联校验面）；auto-os
> PLAN-019：负一屏显示桌面（保留分区 + origin 往返）与壁纸选择 carousel
> （pick/close 组合簿记 + 候选注入面）。详见 §6 v1.7 节）。
> v1.6（2026-09-12，PLAN-012 落码；dock 固定/聚焦/通知可见派生面
> + 单枚固定与格子重注入动词 + 整桌面预览合同面，详见 §6 v1.6 节）。
> v1.5（2026-08-31，Plan 505 B2 落码；pager 派生面（≤4 截断 + "+N"）。
> v1.4（2026-08-30，Plan 487 M4；486 先合占 v1.3，487 按并行协调叠 v1.4）——v1/v1.1/v1.2/v1.3/v1.4 见 §6
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
| `__wm_wins` | Obj 数组 `{wid:str, title:str, focused:str, workspace:str, app:str, icon:str, native:str, pager:str, pinned:str, dup_app:str}` | **全部**虚拟窗（跨 workspace 全集，dock 运行指示消费；可见性由宿主绘制层自过滤）。`focused` = `"1"/""`；`workspace` = 分区下标串；`app` = 注册表 id（boot 窗缺省 `""`）；`icon` = lucide 名（注册表实时查 → 缺省 `"app-window"`）。**v1.3 native 槽位条目**：Docked 原生窗口追加在尾部，字段集 `{wid, title, focused, native, icon}`（workspace/app 不适用省略）——`wid` = `"N<slot_id>"` **独立编码空间**（`N` 前缀 + 十进制槽位 id，与 App wid 的纯数字空间隔离；shell 侧零解析成本区分两类条目）；`native` = `"1"`（App 条目恒空串，分支判据统一）；`focused` 恒空（native 焦点域在 OS 层，WM 不代管）；`icon` 占位 `"app-window"`（HICON 提取为增强候选）。**v1.5 `pager` 旗标**：`"1"` = 本窗属其分区 z_order 前 4（pager 缩略网格≤4 截断消费；溢出窗⊕空串，与 `ws.more` 标签配套——.at 无过滤后截断原语，宿主派生保持 I9；mru/native 条目恒空串）。**F2 走查增补 `pinned`/`dup_app`**：`"1"/""`——本窗 app 已固定 / 同 app 已有更前位（z_order 序首见之外）非隐藏窗；dock 条目跳过判据（**等式消费**——原 `__dock_pinned_csv.contains(...)` 为 view 条件方法调用死点恒 false，实机固定图标+运行图标并存实证；`dup_app` 兼承"同类 app 共享一图标"用户裁定；mru 条目恒空串；窗开/关/hide 翻 win 指纹段随写刷新） | 宿主写 | v1（native 条目/`native` 字段：v1.3；pager：v1.5；pinned/dup_app：F2 走查增补） |
| `__wm_meta` | str `"layout\tfocused_wid"` | 布局名（free/grid/master-stack）+ 焦点窗 wid（无焦点空串） | 宿主写 | v1 |
| `__wm_workspaces` | Obj 数组 `{id:str, name:str, current:str, label:str, more:str}` | 分区清单；`id` = 下标串；`name` = pack 默认 "Desktop N"（M4 settings 可覆盖）；`current` = `"1"/""`；`label` = 1 基人读标签（= id+1 十进制串；**宿主投影**，避开 .at 字符串算术——pager 按钮文本消费）；**v1.5 `more`**：溢出标签 `"+N"`（分区窗数 >4 时；无溢出/空分区空串——pager 网格≤4 截断配套消费） | 宿主写 | v1（label：v1.1；more：v1.5） |
| `__wm_mru` | Obj 数组（条目同 `__wm_wins` 六字段） | **当前分区**的窗口按 MRU 序（front = 最近聚焦；退役 Ctrl+Tab 焦点环语义延续——焦点环不跨分区，472 定案）。switcher overlay 专用消费面，dock 消费不受影响。switcher **handler** 侧消费走宿主召唤时的伴随平行字符串列表（`mru_wids`/`mru_titles`/`mru_icons` + `call_handler("RebuildMru")` 建 handler 自有 rows，B12 规避——464 launcher `apps_*`/`ranked` 同型；`__wm_mru` 本体保持合同面对拍形态） | 宿主写 | v1.1 |
| `__wm_running` | str `",id1,id2,"` | 运行中 app id 集合的**派生串**（.at view 条件无法 `contains` 消费——方法调用死点，O2 实证；保留为对拍/审计面 + handler 侧可用。pinned 灰条判据改 `__dock_pinned` 条目 `running` 字段）。**v1.8 注入面扩展**：同步扩注 desktop 本体面（`assets/desktop.at` 声明同款）——W-04 启动中反馈的 launching ack 判据数据面（handler 侧 contains 消费合法）；宿主写点 = `sync_shell_windows` 投影组内，随写显式召唤 desktop 层 `RunningSync` handler | 宿主写 | v1（desktop 层扩注：v1.8） |
| `__wm_notes` | Obj 数组 `{id:str, kind:str, msg:str, at:str, app:str}` | **通知历史全量**（MRU 序 front=最新；容量 50 FIFO）。shell 侧为合同面（dock 不直接消费）；通知中心面板 handler 消费走召唤/活更新时的伴随平行字符串列表（`note_ids`/`note_kinds`/`note_msgs`/`note_ats` + `call_handler("RebuildNotes")` 建 handler 自有 rows，B12 规避——`__wm_mru` 同型）。`kind` ∈ success/error/info（约定值，未知宿主侧已兜底）；`at` = 入史时刻 `HH:MM` 本地时间串（宿主投影）。**v1.8 `app`**：来源 app id（notify 动词发件方 registry_id；宿主内部通知/历史槽恢复缺省 `""`——面板行跳来源臂以空串判不可跳）；平行列表同型增 `note_apps`（W-14 B12 落地后消参，挂账 auto-os §10-Q5） | 宿主写 | v1.2（app/note_apps：v1.8） |
| `__wm_notes_unread` | str | 未读通知计数十进制串（dock 铃铛 badge 条件消费：`!= "0"` 且非空串渲染）；开面板即清零；不落盘——boot 恢复后恒 `"0"` | 宿主写 | v1.2 |
| `__wm_notes_badge` | str | **v1.8** badge **显示串**（宿主派生：unread >9 → `"9+"`、1..9 → 十进制串、0 → `""`）——W-07 圆形角标文本面（.at 视图无数值比较/截断原语，I9 宿主派生单点）；等式/空串消费在 shell.at，`__wm_notes_unread` 本体保持计数合同不变 | 宿主写 | v1.8 |
| `__wm_focused_app` | str | **v1.6** 聚焦窗 registry_id 派生串（"" = 无聚焦或聚焦在 native 槽位）——pinned 图标聚焦底条 + 底色高亮的**标量判据面**（.at 无法跨列表表达"存在聚焦窗"量词，宿主派生保持 I9；聚焦变化必经 meta 段 focused_wid 翻转触发重写，本字段随写同步） | 宿主写 | v1.6 |
| `__dock_pinned_csv` | str `",id1,id2,"` | **v1.6** 固定集合派生串（前后逗号封边；空表 = 单纯 `","`）——dock 去重的**原判据面，已被 `__wm_wins.pinned`/`dup_app` 派生字段取代**（view 条件 contains 死点，F2 走查实证）；保留为对拍/审计面 | 宿主写 | v1.6 |
| `__dock_pinned` | Obj 数组 `{id:str, icon:str, running:str}` | **v1.6** 固定集合（config.dock_pinned 单源；icon 注册表解析缺省 `app-window`）。**F2 走查增补 `running`**：`"1"/""` 该 app 当前有非隐藏运行窗——pinned 图标灰条判据（等式消费）；inject_dock_pinned（pin/unpin 即时臂）与 sync 投影（fp 差分刷新臂，窗开合即刷新）双写者同形幂等 | 宿主写 | v1.6（running：F2 走查增补） |
| `__wm_notes_visible` | str `"1"/""` | **v1.6** 通知中心面板可见性（宿主 overlay 组件 visible 直读投影；外点/×/Esc 任意关闭路径自隐后同步翻转）——铃铛打开态高亮的唯一事实源；指纹并入 notes 段尾 `:v`（见 §3） | 宿主写 | v1.6 |
| `__wm_settings_open` | str `"1"/""` | **F2 走查增补** 设置窗在场标量（存在非隐藏 os-config 窗 = "1"；W1 close→hide 后隐藏即回落）——⚙️ 高亮判据（原 `__wm_running.contains(",os-config,")` view 条件方法调用死点，实机探针定性：设置窗在场齿轮仍无高亮）；窗开/关/hide 翻 win 指纹段随写同步 | 宿主写 | F2 走查增补 |
| `__wm_fp` | str | 投影指纹（§3）；shell 不消费，仅门控 | 宿主写 | v1 |
| `__wm_clock` | str `"HH:MM"` | dock 时钟本地时间串（497 S3）。**非门控字段**：不进 `__wm_fp` 指纹、不走 §3 投影组换装——ServiceTick 帧泵独立注入（分钟变化才写，稳态零重建；本地时钟非驱动事实，避免每分钟全组换装抖动） | 宿主写 | v1.4 内（497） |
| `__wm_date` | str `"M月D日 周X"` | **v1.8** dock 日期行（中文周几宿主格式化——.at 无日期算术；`%-m`/`%-d` 不补零）。与 `__wm_clock` 同一 ServiceTick 帧泵、**独立变化才写**（两字段独立脏帧——另一字段不变时稳态零重建口径维持）；同不进指纹门控 | 宿主写 | v1.8 |
| `__wm_showdesk` | str `"1"/""` | **v1.7** 当前分区 = 负一屏（保留空分区，PLAN-019）——任务栏 sliver 两态高亮与 toggle 判据（`"1"` = 点击发 `showdesk_return`，`""` = 发 `show_desktop`；等式消费）。负一屏簿记（origin/picker 簿记）在宿主 WmState，不投影 | 宿主写 | v1.7 |
| `__desktop_cmd` | str | 出向命令记录串（§4）；宿主**读+清** | shell 写 | v1 |
| `__desktop_cmd` | str | 出向命令记录串（§4）；宿主**读+清**。**v1.7 发件面扩宽**：普通注册表窗联合排空（pump 全窗 drain；特权四窗原有专属排空保留，二次空读无害）——`open_with` 首个普通窗动词（027 文件管理器发件）。**v1.8 追加语义（PLAN-014 W-02）**：shell 侧写点一律 `.SendCmd(rec)` 单点追加（非空先接换行符再拼记录）——同排空周期多命令累积不互相覆盖；宿主 `parse_records` 按换行/REC_SEP 切分逐条执行，排空点每 update 周期读+清（追加语义以此为止） | shell 写（v1.7 起：普通注册表窗亦可） | v1（发件面扩宽：v1.7；追加语义：v1.8） |

### 2.0.1 通知面板接缝字段（`assets/notification_center.at`，召唤/活更新注入）

| 字段 | 类型 | 语义 | 权属 | 引入 |
|---|---|---|---|---|
| `__panel_h` | int | **F2 走查增补** 面板根列显式像素高（视口 - dock 预留）——真实链中 Stack 子层 `h-full`（Fill）约束传递失效（headless 全链复刻通过、实机 mt-auto 填充条塌缩，O3 活体 diff 实证），显式像素高使贴底锚定不依赖约束传递；注入点 = 召唤/活更新（resize 开着面板留旧值，重开生效——v1 可接受） | 宿主写 | F2 走查增补 |
| `__panel_max_h` | int | **F2 走查增补** 历史列表最大高（= `__panel_h` - 底垫 60 - 标题行 ~52 - 余量 12）——条目多时列表 `max-h` 滚动，卡片恒有界（实机 6 条 ~110px 条目 ≈ 790px > 744 可用 → mt-auto 空间归零卡片贴顶，用户截图复现） | 宿主写 | F2 走查增补 |

### 2.1 桌面本体面字段（`assets/desktop.at`，v1.4 内字段扩展——不升版本段）

Plan 496 M5 的第五面（常驻不召唤，boot 装载挂桌面层 z 槽）。命名族
`__desktop_*`（与 `__wm_*` 同为宿主特权命名空间；boot 期一次注入，无指纹
门控——数据源 storage 键 boot 读一次，会话内不变）：

| 字段 | 类型 | 语义 | 权属 | 引入 |
|---|---|---|---|---|
| `__desktop_bg` | str | 壁纸色值类片段：`shell.desktop.wallpaper` 为 `#hex` 时注入 `"bg-[#hex]"`（面根 bg 实铺）；图片路径/缺省时注入 `""`（图片壁纸由宿主在面之下推壁纸图层——DSL 无重叠布局，z 序宿主侧兑现） | 宿主写 | v1.4 内（496） |
| `__desktop_icons` | Obj 数组 `{id:str, icon:str, label:str, src:str, color:str}` | 桌面条目 = 自定义条目（`shell.desktop.icons` 逗号串）排除 hidden（`shell.desktop.hidden` 逗号串；PLAN-012 F2 起桌面图标 = 仅自定义列表，pinned 不并入）。`icon`/`label` 注册表解析（缺省 `app-window`/id）；`src` = `pinned`\|`custom`。**v1.8 `color` 审计补记**：per-app 徽标色（`badge_color_for` 哈希分配）——v1.4 起实注入而字段表漏记（PLAN-014 W-01/W-12' 补记），非 full 满幅条目的 chip 底色消费 | 宿主写 | v1.4 内（496；color 补记：v1.8） |
| `__desktop_hidden` | str | 排除 id 逗号串（移除臂续写 `shell.desktop.hidden` 的当前值底稿） | 宿主写 | v1.4 内（496） |
| `__desktop_cells` | Obj 数组 `{id,icon,label,src,color,c,r,spacer,full}`（spacer 条目仅 `{spacer:"1",c,r}`） | **v1.6** 格子化图标表——`shell.desktop.positions`（追加式 `"id=c:r,..."` csv，last-wins 解析）定位优先 + 未定位**列主序**填首个空格（022 SD-01；rows = 视口高/80px 行距扣任务栏，clamp 4..24）+ 空位 spacer 填充；view 渲染消费（handler 消费走下列平行字符串列表，B12 规避）。拖拽落子 = shell 追加写 positions + `refresh_desktop_icons` 触发重注入。**v1.8 `full` 审计补记**：满幅位图旗标（"1" = 48px 满幅 iconfile 渲染臂；缺省 = 徽标色 chip）——022 起实注入而字段表漏记（PLAN-014 补记） | 宿主写 | v1.6（full 补记：v1.8） |
| `__desktop_cell_ids` / `__desktop_cell_cs` / `__desktop_cell_rs` | 平行字符串列表（与 `__desktop_cells` 同序；spacer 格 id = 空串） | **v1.6** 格子平行字符串列表——`IconPress` 拖拽落子的 handler 下标读数据面（按 id 检索 (c,r)，B12 规避——`note_ids` 同型） | 宿主写 | v1.6 |
| （DSL 合同面）`workspace_preview` | 布局件 `workspace_preview (ws: <分区id>, fallback: <icon>)` | **v1.6** 整桌面等比预览 leaf——宿主渲染臂合成（壁纸 `#hex` 基色底 / 缺省主色占位 → 该分区逐窗 snapshot 按 usable 矩形 Contain 等比贴片；miss = 占位块 + fallback icon 居中 + request_capture 预抓，window_thumbnail 同款 SWR）。**协议零字段增量**：数据面直连 wm 几何 + snapshot 缓存（`iced::workspace_preview` 发布/消费），不入 VM 状态。消费面 = switcher 分区卡 | 宿主渲染 | v1.6 |
| `__wp_picker` | str `"1"/""` | **v1.7** 壁纸选择 carousel 可见性（PLAN-019；desktop.at 全屏层条件渲染唯一事实源）。开合簿记在宿主（`picker_open` + `picker_return_on_close`——`wallpaper_pick`/`wallpaper_close` 组合臂单点维护，.at 侧零分支）；随写置 dirty | 宿主写 | v1.7 |
| `__wp_dir` | str | **v1.7** 当前壁纸目录（picker 头部展示；`wallpapers_dir` 解析链当前值——config → env `AUTO_DESKTOP_WALLPAPERS_DIR` → 探测目录） | 宿主写 | v1.7 |
| `__wp_current` | str | **v1.7** 当前壁纸路径（boot 壁纸解析值——`#hex`/`builtin:`/图片路径；picker 缩略图高亮的**等式判据面**；`set_wallpaper` 臂随写） | 宿主写 | v1.7 |
| `__wp_items` | Obj 数组 `{name:str}` | **v1.7** 候选壁纸清单全量合同面（`scan_wallpapers_dir` 供源——jpg/jpeg/png 文件名升序）。渲染消费走 `__wp_visible` 滑窗切片（FU7）；handler 侧下标读走伴随平行字符串列表 `wp_paths`（与 items 同序；B12 规避），picker open/目录变更/点选后全量重注入 | 宿主写 | v1.7 |
| `__wp_visible` | Obj 数组 `{name:str,path:str,src:str}` | **v1.7** carousel 可见窗口切片（`picker_win` 起 5 枚；`__wp_items` 同构元素）——desktop.at 缩略栅格渲染面；`wallpaper_nav` 滑窗后随写 | 宿主写 | v1.7 |
| `__wp_x` / `__wp_y` | float | **v1.7** picker 面板锚点坐标（宿主按可用区算好注入：水平居中、贴任务栏留 8px——.at 无算术）。坐标锚 popover 消费 | 宿主写 | v1.7 |
| `__wp_preview` | str | **v1.7** 大图预览态（"" = 栅格态；非空 = 预览图**路径**——视图零算术直渲染）。宿主持有（`picker_preview` 游标簿记投影，路径自 `picker_paths` 解析）——`wallpaper_preview`/`wallpaper_nav` 移动，`wallpaper_close`/目录变更复位；desktop.at 双态渲染判据 | 宿主写 | v1.7 |

## 3. 更新语义与指纹门控（协议条款）

- 宿主每 update 周期在 DesktopBus 排空点邻位重算投影（O(窗数) 串接）。
- `__wm_fp` = 逐窗 `"{wid}:{focused},{workspace};"` 串接（**v1.3**：native
  槽位条目并入同段，`"N{slot}:{focused},"` 同型追加在 App 窗之后——槽位
  增删/瞬时态转 Docked 必翻指纹）+ `"|{__wm_meta}"` +
  `"|"` + 逐分区 `"{id}:{current},{label};"` 串接 + `"|"` + 逐 mru 窗
  `"{wid};"` 串接 + `"|notes:{len}:{front_id}:{unread}:{visible};"`（v1.1：
  分区段扩 label、尾接 mru 段；v1.2：尾接 notes 段——len/front_id 双段覆盖
  容量环绕与 dismiss 组合、unread 独立第三段，任何历史/未读变化必翻其一；
  **v1.6**：notes 段尾扩 `:v` 可见性位 + 尾接 `"|pinned:{id1,id2};"`
  固定集合段——pin/unpin 落子与面板开合必翻其一）。
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
| `dock_pin` | app id | **v1.6** 单枚固定——`config.dock_pinned` Vec 增删去重之增（已在表 = 幂等保持）→ 单源落盘 → `inject_dock_pinned` + `inject_desktop_surface` 双投影热同步（发件面 = shell.at 窗口条目右键菜单「固定到任务栏」）。窄动词缘由：shell 读不到 `__dock_pinned` Obj 数组全集做 csv 手术（B12 同族），单枚增删收口宿主侧 | v1.6 |
| `dock_unpin` | app id | **v1.6** 单枚取消固定——同上之删（不在表 = 幂等跳过；发件面 = pinned 图标右键菜单「取消固定」） | v1.6 |
| `refresh_desktop_icons` | （无参记录） | **v1.6** 桌面图标格子重注入——shell 拖拽落子写 `shell.desktop.positions` 后触发，宿主重读 storage 重算 `__desktop_cells`/平行列表（`inject_desktop_surface` 重跑；拖拽结果即时可见 + boot 同链） | v1.6 |
| `show_desktop` | （无参记录） | **v1.7** 显示桌面（负一屏）——切换到宿主保留的空分区（懒建：首次到达 `add_workspace` 并记录 `showdesk_ws`）；`showdesk_origin` 记录进入前 current（current 已是负一屏则**不覆盖**——幂等）。发件面 = shell.at 任务栏右缘 sliver | v1.7 |
| `showdesk_return` | （无参记录） | **v1.7** 返回 origin 分区——picker 若开着先关（簿记幂等清零）→ current = `showdesk_origin` → origin 清零。发件面 = sliver 再点（`__wm_showdesk == "1"` 臂） | v1.7 |
| `wallpaper_pick` | （无参记录） | **v1.7** 更换壁纸组合入口 = `show_desktop` 幂等臂 → `picker_open=1` → `picker_return_on_close = (到达前 current != showdesk_ws)`（**归属规则单点**：谁切屏谁负责切回——他分区进入组合调用 = 自动返回；负一屏自入 = 关闭只关 picker 不代管返回）→ `__wp_*` 五面注入。发件面 = desktop.at 图标/空白右键菜单「更换壁纸…」 | v1.7 |
| `wallpaper_close` | （无参记录） | **v1.7** 关闭 picker——`picker_open=0` → 若 `picker_return_on_close` 则走 `showdesk_return` 臂（origin 消费后清零）。发件面 = desktop.at picker 关闭钮/遮罩/Esc 栅格态 | v1.7 |
| `wallpaper_browse_dir` | （无参记录） | **v1.7** 弹原生目录对话框（rfd `pick_folder`，`ui-dialog` feature）——选定后走 `SetWallpapersDir` 同一执行臂（目录写路径单一：config 落盘 + mtime 热轮询）+ `__wp_items`/`__wp_dir` 重注入。发件面 = desktop.at picker 头部「浏览…」 | v1.7 |
| `wallpaper_nav` | `prev`/`next` | **v1.7** picker 导航（宿主按态分派；FU7 语义修订）——预览态 = 大图游标环绕移动（`__wp_preview` 随写）；栅格态 = **carousel 滑窗**（`picker_win` 可见 5 枚窗口起点的 ±1 滑动 + `__wp_visible` 重注入；端点 clamp）。导航不应用——点选缩略图才应用。数学生宿主的缘由：.at 无列表下标/算术（B12 族） | v1.7 |
| `wallpaper_preview` | 图片路径（空参 = 退栅格） | **v1.7** 大图预览进出——带参按 path 反查游标进入预览（缺席/关态 no-op）；空参退栅格态。`__wp_preview` 载荷 = 预览图路径（.at 零算术直渲染）。发件面 = desktop.at 缩略图「预览」钮 / 预览态「返回」钮 | v1.7 |
| `open_with` | app id、绝对路径 | **v1.7** 用注册表 App 打开文件——执行臂：注册表校验（未知 app 拒绝；pac `opens:` 声明面非空且未含目标扩展名即拒，toast 反馈）→ 未运行 launch 后向目标 App state 写 `auto_open_path` / 已运行聚焦 + 同款写入 → 目标 App Tick 消费（041-auto-edit `ConsumeOpen`、031-image-viewer SettleTick 消费臂为首批接收面；未声明 `auto_open_path` 的 App 写入静默无效 = capability 自声明）。约束：**path 单行**（记录层按 `
` 切分，notify msg 同款）；发件面 = 027 双击/右键打开（PLAN-016） | v1.7 |

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
  双形态 arg）+ `dock_pin_unpin_verbs_parse_and_encode`（v1.6）+
  `w5_refresh_desktop_icons_verb_parse`（v1.6）。
- **v1.6 金样**：`projection_v16_dock_pinned_csv_and_fingerprint`（csv
  三态 + 指纹 pinned 段 + `__wm_focused_app` 派生翻转）/
  `projection_v16_notes_visible_fingerprint`（:v 尾标翻转）/
  `w1_osconfig_close_hides_and_focus_unhides`（close→hide 投影排除全链）/
  `w5_desktop_icon_cells_assignment` + `w5_desktop_positions_last_wins`
  （格子分配/持久解析）+ `w8_workspace_preview_materializes_in_popover_for`
  （合同面物化 fence）+ iced-layout-tests `w2_notification_panel_anchor_*`/
  `w7_icon_*`（headless 布局探针——面板锚定与字形居中，非投影面但同批）。
- vue 端（465 后续）按本表实现同版本投影 + 同指纹规则，对拍项登记后
  消费本文件作基线；版本升级 = 文件名/版本号 + 双端同步 + 对拍重跑。
- I7（shell 无几何操作）、I9（窗口/分区列表唯一事实来自本投影）随行。

## 6. 变更记录

### v1.8（2026-09-17，auto-os PLAN-014 shell-ux-polish-v2——纯字段 + 总线语义，零新动词）

- **`__wm_date` 新字段（§2，W-06'）**：dock 日期行 `"M月D日 周X"`——与
  `__wm_clock` 同一 ServiceTick 帧泵独立注入（各自变化才写，独立脏帧），
  不进指纹门控（497 时钟先例同口径）。
- **`__wm_running` 注入面扩展（§2，W-04）**：字段本体 v1 不变，同步扩注
  desktop 本体面（desktop.at 同款声明）——启动中反馈 launching ack 判据
  数据面；宿主随写召唤 desktop 层 `RunningSync` handler（写状态不触发
  handler 律）；启动失败残态由 Init/重注入求差自愈（shell 侧防御）。
- **`__wm_notes[].app` / `note_apps`（§2，W-08）**：notify 落库记录来源
  app id（联合排空泵注册表窗段**按 app 分段执行**——段前置置
  `notify_source`，命令顺序与原扁平 concat 逐一相同）；宿主内部通知/
  历史槽恢复缺省 `""`（面板行跳来源臂空串判不可跳，`activate` 两臂复用
  零新动词）。W-14 B12 落地后 `note_apps`/平行列表族消参（挂账）。
- **`__wm_notes_badge` 新字段（§2，W-07）**：badge 显示串宿主派生
  （>9 → `"9+"`、0 → `""`）——.at 视图无数值比较/截断原语，I9 单点；
  `__wm_notes_unread` 计数合同不变。
- **`__desktop_cmd` 追加语义（§2/§4 头注，W-02）**：shell 侧写点 `.SendCmd`
  单点追加（四 pack 统一），同排空周期多命令以换行累积；宿主排空/解析
  语义不变（`parse_records` 换行切分既有支持）。
- **字段表审计补记（§2.1，W-01/W-12'）**：`__desktop_icons[].color`
  （badge_color_for 徽标色，v1.4 起实注入漏记）+ `__desktop_cells[].full`
  （022 满幅位图旗标漏记）；`__desktop_icons` 语义行同步修正为 PLAN-012
  F2 后实况（仅自定义列表，pinned 不并入）；`__desktop_cells` 未定位
  填充序更正为列主序（022 SD-01 实况）。
- **oncontextmenu 坐标事件臂**：默认**后置**（auto-os §10-Q2——022 拖拽
  幽灵臂依赖 `__desktop_cursor_x/y` 泵，改造前需裁定交互归属），不在
  本版强捆，泵与坐标锚照旧。
- **向后兼容声明**：纯增量字段/注入面——`app`/`note_apps` 缺席消费方
  零破坏（notification_center.at 域外防御读：缺列表 app = ""）；
  `__wm_notes_badge` 为新只读面；追加语义对既有单写点行为等价（单命令
  串不变）；`__wm_running` desktop 层扩注对 shell 层零影响。vue 端以
  本版为对拍基线（§5）。
### v1.9（2026-09-18，auto-os PLAN-024：dashboard 常驻小组件层——v1.8 并行协调叠号）

- **dashboard 面板 = 第四 overlay 槽**（设置面板退役后继任；**常驻语义**（用户裁定 R2/R21）：z 高于
  桌面图标层、低于全部 app 窗，boot 常显、× 隐藏 / dock ▦ 切换，
  无 scrim/外点关闭；**face 卡双击 = 三态打开**（升格开窗原语
  open_window_for_session / activate 聚焦 / launch））。面板本体 = 特权面
  `dashboard.at`（shell pack 第五件，hash-lock 双写）；face 卡 = 各 App
  `view mini` 命名视图的**宿主拆借渲染**（`SessionViewRef.view_name` 选择
  器 + `DynamicComponent.view_named`——活渲染面：与主窗同 component/同
  VM 桥，输入/Tick/重渲染全通，非截图/缩放）。
- **入向 `__dashboard_faces`**（面板 App 合同面 Obj 数组，声明不 handler
  消费——handler 侧走 `face_ids/face_titles/face_icons/face_statuses/
  face_spans` 平行字符串列表 + `RebuildFaces`，B12 规避同族）：face 快照
  `{id,title,icon,status,span}`，`status` ∈ running/hatched/placeholder
  （D4 门：daemon/back_root/exe 缺一才孵化——inproc 合并 VM 内
  back_port 不构成外部依赖；tab 派生 = 注册表 category，system →
  系统页）。
- **入向 `__wm_dashboard`**（shell 标量 "1"/""）：面板可见性投影——dock
  Dashboard 钮两态高亮判据（`__wm_notes_visible` 同型）。
- **入向几何注入** `__panel_w/__panel_h/__panel_top`（px）：面板布局单一
  事实在宿主（`dashboard_layout` 行主序 next-fit：等宽 3 列 + span 1|2 宽卡），
  面板 .at 经 style 插值镜像（`__panel_max_h` 同型）。
- **出向 `__dashboard_cmd`**（面板 App 上行总线，宿主读+清）：六动词
  `dashboard_toggle`/`dashboard_close`/`dashboard_pin <id>`/
  `dashboard_unpin <id>`/`dashboard_span <id> <1|2>`/`dashboard_launch
  <id>`（`__desktop_cmd` 同一 parse_records 解析，DesktopCommand 六新变体）。
  shell.at dock 钮走 `__desktop_cmd` 的 `dashboard_toggle` 无参动词。
- **配置键空间 `shell.dashboard.*`**：`enabled`（csv 纳入清单；缺席 =
  未配置 = 首次召唤自动纳入全部候选）+`span.<app>`（"1"|"2"）。
- **降耗（R5）**：非活动 tab/面板隐藏时孵化会话 `.Tick` 停订（订阅随
  消息周期重评估）。
- **栅格进度条三态（R17d）**：接近即占位（空白块） / 段内过半点亮 /
  未到不渲染；段色四分位（≤25 绿 / ≤50 蓝 / ≤75 黄 / >75 红）。宿主侧
  `storage_host_read`/`storage_host_publish` 直读写（非几何无动词，boot
  生效——既定判定）。
- **新动词（§4 词表扩）**：上述六 dashboard 动词。

### v1.7（2026-09-14，双增量并行落码：PLAN-016 + auto-os PLAN-019）

- **（PLAN-016）新动词 `open_with`（app id、绝对路径）**：桌面级文件打开互操作。执行臂
- **（PLAN-016）`__desktop_cmd` 发件面扩宽**：pump 联合排空从特权四窗扩展到全部注册表
- **（PLAN-016）pac `opens:` 关联键**：`AppRegistryEntry.opens`/`LaunchSpec.opens`
- **（auto-os PLAN-019）负一屏显示桌面**：§4 +`show_desktop`/`showdesk_return` 两动词——宿主保留
- **（auto-os PLAN-019）壁纸选择 carousel**：§4 +`wallpaper_pick`/`wallpaper_close`/
- **（auto-os PLAN-019）vue 端注记**：本版实现方 = vm 端（renderer.rs）；vue 端沿 v1.6 先例，

### v1.6（2026-09-13 F2 走查增补，PLAN-012 O1-O3 收口）

- **判据面迁移（view 条件方法调用死点清偿）**：`.at` view 条件求值器
  （`eval_condition_with_inner`）无方法调用臂——`.contains(...)` 等
  **静默塌缩恒 false**（T9 发现②实锤升级：实机 dock 固定图标+运行图
  标并存、pinned 灰条缺失、⚙️ 无高亮三处活体定性）。判据面全部改宿主
  派生 + 等式消费：
  - `__wm_wins` 条目增 `pinned`/`dup_app`（"1"/""）——dock 去重 + 同类
    app 共享一图标（用户裁定）；`__dock_pinned` 条目增 `running`
    （pinned 灰条）；新标量 `__wm_settings_open`（⚙️ 高亮）。
  - `__dock_pinned_csv`/`__wm_running` 降级为对拍/审计面（handler 侧
    contains 仍可用——限制仅 view 条件）。
- **通知面板几何注入**：`__panel_h`（根列显式像素高）+ `__panel_max_h`
  （列表 max-h）——条目多时卡片贴顶（6 条 ~110px ≈ 790 > 744 可用，
  用户截图复现）+ 真实链 Stack 子层 h-full 约束传递失效（headless 全链
  复刻通过、实机塌缩，O3 活体 diff 实证）双因子清偿。
- **desktop 模式 primary 锚点迁移**：特权 shell 先于直挂 comps 分配
  （boot 序调整）+ shell 层 MCP 同步开启——autoui_state/autoui_vtree/
  截图通道自此观测 shell 投影面（O1"真实链路 bounds 探针"前置条件；
  旧序 primary = 首个直挂窗，shell 面不可观测）。配套：shell_fields.
  window_size 随 tick 镜像宿主 viewport（层 App 无窗事件，截图守卫曾
  拒死 MCP 截图）。
- **金样增补**：iced-layout-tests `p012_o3_notification_layer_in_stack_
  anchor`（Stack 装配锚定守卫）+ `p012_o3_notification_real_component_
  in_stack`（真组件链少条目贴底 + 多条目 max-h 有界双场景）。

### v1.6（2026-09-12，PLAN-012）

- **动词**：`dock_pin	<id>` / `dock_unpin	<id>`（单枚固定/取消固定；
  执行臂 Vec 增删去重 → `desktop_config::save` → 投影热同步）+
  `refresh_desktop_icons`（无参；桌面图标格子重注入）。
- **投影面**：`__dock_pinned_csv`（",id1,id2," 去重判据串，指纹尾接
  `|pinned:` 段）；`__wm_focused_app`（聚焦窗 registry_id 标量串——
  聚焦判据从窗口条目 `focused` 旗标升级出标量面）；`__wm_notes_visible`
  （"1"/""，notes 指纹段尾扩 `:v`）。
- **配置语义**：`dock_pinned` 缺省三枚退役——缺键 = 显式空 = 空表
  （`DEFAULT_DOCK_PINNED = []`；`set_dock_pinned` 空 csv 不再回退缺省）。
- **DSL 合同面**：`workspace_preview (ws, fallback)` 布局件（宿主合成，
  协议零字段增量——SD-02）；桌面本体面增 `__desktop_cells` 格子表与
  `__desktop_cell_ids/cs/rs` 平行列表（拖拽换位，`shell.desktop.positions`
  追加式持久）。
- **隐藏窗语义**（宿主内聚，协议面体现为投影排除）：os-config 窗
  close→hide（`VWinState.hidden`），常驻隐藏窗不入 `__wm_wins`/运行集/
  MRU/格子表——"真关了"感知 + 齿轮高亮（`__wm_running` 判据）随投影
  回落。

### v1.5（2026-08-31，Plan 505 B2）

- **pager 派生面（≤4 截断 + "+N"，债 P497-1 清偿）**：
  `__wm_wins` 条目增 `pager` 旗标（"1"/""，本窗属其分区
  z_order 前 4；mru/native 条目恒空串）+ `__wm_workspaces`
  条目增 `more` 溢出标签（"+N"/空串）。根因：.at 无
  "过滤后截断"原语（for+if 无局部计数器、数组无
  take/slice），派生保持宿主侧 I9 单一事实源（`__wm_running`
  先例）。
- **指纹不扩段**：旗标/标签均为既有指纹 win 段的纯函数
  （逐窗 workspace 已入段）——无需新段。
- **向后兼容声明**：纯增量字段——既有消费者零破坏
  （pager 网格消费面由全量改为≤4 截断，为本意
  修复；他窗 for 循环字段读不受影响）。vue 端（未实现）
  以本版为对拍基线；文件名不变，双端同步对拍在 vue
  端落地时执行（§5）。

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
