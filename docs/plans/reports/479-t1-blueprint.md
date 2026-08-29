# Plan 479 T1 施工图 — 通知中心（S6）+ desktop.notify 动词

> 状态：定案（六项待澄清全部收口 + 五项实施细节补全）。
> 勘察基线：worktree `plan-479-dev` @ c43928307。
> 本报告按计划 D1–D5 顺序对账，只做「定案 + 落点」，不改设计方向。

## 0. 六项待澄清定案（计划文「待澄清事项」逐条收口）

| # | 待澄清项 | 定案 | 依据 |
|---|---|---|---|
| 1 | `notes_toggle` 路由 | **总线无参动词 + update 执行臂**（计划倾向，采纳）：`DesktopCommand::NotesToggle` 新臂，`execute_desktop_commands` 臂调 `toggle_notification_center`（renderer.rs，478 召唤臂同型）。dock 钮 onclick `.NotificationToggle` → `__desktop_cmd = "notes_toggle"`。词表 v1.2 实为**四动词**（notify / notes_toggle / notes_clear / notes_dismiss） | dock 钮路径必须经总线；面板自持翻转会造成两条开关路径语义分叉；478 `SummonSwitcher` 事件臂已证执行体形态 |
| 2 | 面板锚位 | **右下锚定卡片**（w-80）：col `w-full h-full` → spacer `flex-1`（顶压）→ row 内 spacer `flex-1`（左压）+ 卡片 + 右缘 spacer `w-3` → 底部 spacer `h-16`（让位 h-12 dock）。避开 bottom-center toast 区（3500ms 浮层），通知中心惯例位 | 计划「无强偏好则右下」；switcher 已证 col+spacer 布局形态；dock flex-1 spacer 已证 |
| 3 | `at` 串格式 | **`HH:MM` 本地时间**，宿主侧 `chrono::Local::now().format("%H:%M")` 入史时定格 | 计划倾向 HH:MM；chrono 0.4 已是 auto-lang 依赖（vm/native.rs:6913 `chrono::Local::now` 先例）；478 label「宿主投影避开 .at 算术」同型 |
| 4 | 容量与槽数 | **内存 `NOTES_CAP = 50`（FIFO，MRU 序 front=最新）／落盘 10 槽 `shell.notes.0`..`shell.notes.9`**。每槽一条目 JSON（serde_json 序列化，引号/换行安全），空槽 = `""`；slot0 = MRU front；写入 = 变更后全量重写 10 槽；boot = 读 10 槽 parse，坏槽跳过 | 计划建议 50/10 采纳；launcher.recent_apps.N 定长槽同型；`storage_host_publish/read` 对偶（stdlib.rs:582/591）已证 |
| 5 | kind→图标映射 | **success→`check`、error→`x`、info→`info`**（嵌套 if/else 双层，switcher 同型——.at 无 else-if 链）。`check`/`x`/`bell` 已在内嵌 lucide 表（renderer.rs:3915/3912/3877）；**`info` 缺席 → T3 补一枚**（472 T5 补表先例），path `<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>`。未知 kind 回退 info | renderer.rs lucide_svg 表实测核对；ToastReq kind 集合含 warning/default，v1 面板只按三枚渲染，其余走 info 兜底 |
| 6 | 指纹 notes 段格式 | **`|notes:{len}:{front_id}:{unread};`**（计划草案 `notes:<len>:<unread>` 的精化：插入 front_id 段）。覆盖性论证：任意入史改变 len 或（cap 满环绕时）front_id；任意 dismiss 改变 len 或（删 front 时）front_id；clearAll 改变 len；未读变化翻第三段。仅 `{len}:{unread}` 双段在「面板开着 dismiss 一条 + 同周期 notify 一条」（len 与 unread 均不变、内容变）场景下会漏翻，故须 front_id | 计划留「精确格式 T1 定案」；指纹段式与 v1.1 逐段 `;` 连接一致 |

## 1. D1 对账：数据与历史（T2 落码）

- `NotificationEntry { id: u64, kind: String, msg: String, at: String }` — session.rs（DesktopState 邻位）。
- `DesktopState` 增三域：`notifications: RefCell<Vec<NotificationEntry>>`（front=最新）、
  `notes_next_id: Cell<u64>`（从 1 起）、`notes_unread: Cell<u64>`。`DesktopState::new` 补初值。
- **入史入口 `push_notification(state, kind, msg)`**（renderer.rs，`push_desktop_toast` 邻位），
  双面一体顺序：
  1. 入史（front 插入 + 超过 `NOTES_CAP` 尾部弹出）；
  2. 未读：面板**不可见**时 +1（`notification_visible()` 判定，session.rs 新访问器）；
  3. 落盘（`persist_notes`，全量重写 10 槽）；
  4. `push_desktop_toast(state, kind, msg)` 浮现（内部不动，8 调用点改道后它只剩本入口一个调用者 + 无关的
     `__toast_tick` 到期路径）；
  5. 面板已挂载且可见 → 快照重注入 + `call_handler("RebuildNotes")` + 面板 view_dirty（打开期间列表活更新）。
- **改道**：renderer.rs 现存 8 处 `push_desktop_toast` 调用点（6339/6343/6352 launcher 装载三处、
  6417 switcher 装载、6572/6582 分区删除门、6597/6599 LaunchApp 成败）→ 全部改 `push_notification`。
  行为增量 = 多入史 + 落盘，浮现不变。
- **持久化读写**：`persist_notes(state)`（写侧）/ `restore_notifications(state)`（boot 读侧，填 §5 接线）。
  JSON 形态：`{"id":1,"kind":"success","msg":"已启动 x","at":"14:30"}`。
  未读计数**不落盘**（boot 恢复后恒 0——会话概念，重启无「错过」语义）。

## 2. D2 对账：动词与执行臂（T2/T3/T5 落码）

- `DesktopCommand` 四新臂（session.rs）：
  - `Notify(String, String)` — encode `notify\u{1F}{kind}\u{1F}{msg}`；parse 对 arg 二次
    `split_once([FIELD_SEP, '\t'])` → (kind, msg)；**约束 v1：msg 不得含 `\n`**（parse 记录层按
    `[REC_SEP, '\n']` 切分会截断——宿主侧既有 msg 全部单行，成文词表约束）；坏载荷（无第二分符）跳过。
  - `NotesClear` / `NotesToggle` — 无参动词，parse 于 `split_once` 前置直判（workspace_next/`_add` 先例）。
  - `NotesDismiss(u64)` — arg parse u64，坏 id 跳过。
- 宿主执行臂（renderer.rs `execute_desktop_commands`）：
  - `Notify(kind, msg)` → `push_notification(state, &kind, &msg)`；
  - `NotesClear` → 历史清空 + 落盘 + 面板可见则重注入（§1 第 5 步同款）；
  - `NotesDismiss(id)` → 按 id 移除 + 落盘 + 面板可见则重注入；
  - `NotesToggle` → `toggle_notification_center(state)`（§3）。
  - NotesClear/Dismiss 臂显式置**面板** view_dirty（重注入路径已含）；shell 侧投影由臂尾/下周期
    `sync_shell_windows` 指纹翻转承接。
- 排空：`drain_and_execute_desktop_commands` 增第四路（`notification_app`，switcher 6480 同型）。

## 3. D3 对账：面板 overlay（T3 落码）

- **`assets/notification_center.at`**（内嵌，widget `NotificationCenter`，switcher.at 同构）：
  - msg：`Init, RebuildNotes, Escape, Dismiss(str), ClearAll`（无键盘导航——无 Advance/Pick）。
  - model 接缝：`visible`/`hosted`/`__desktop_cmd` + 合同面 `__wm_notes`/`__wm_notes_unread`
    （声明不 handler 消费，switcher `__wm_mru` 同型）+ 快照平行串列
    `note_ids/note_kinds/note_msgs/note_ats` + handler 自建 `rows`（{i,id,kind,msg,at}）+ `nrows`。
  - bind 仅 `"Escape" -> .Escape`；handler：`.Dismiss(id)` 写 `notes_dismiss\t<id>`、
    `.ClearAll` 写 `notes_clear`、`.Escape` visible 门控自隐（宿主侧不隐面板层——装配层按
    visible 推层，下一周期视图重建生效）。
  - 视图（visible 门控）：右下卡片（§0 定案 2）= 标题行「通知」+「全部清除」钮 + 空态行
    （`nrows == 0`）+ 条目行（kind 图标 §0 定案 5 / msg / at / `×` 钮 onclick `.Dismiss(r.id)`）。
- **装载四件套扩第四路**（session.rs）：
  - `DesktopState.notification_app: Option<AppId>` + `HostCtx.notification_fields: ShellFields`
    + `open_desktop`/`DesktopState::new` 补初值；
  - `split_mut` windowless 分支 `is_notification` 判定 + 四路 fields 分支；
  - `split_ref_notification()` / `notification_visible()`（launcher/switcher 同型）。
- **召唤执行体 `toggle_notification_center(state)`**（renderer.rs，summon_switcher 同构）：
  懒挂载（`build_notification_center_component()`，shell.rs 新函数 + `NOTIFICATION_CENTER_AT` 常量；
  失败 `push_notification(state, "error", …)` 降级）→ 可见时翻转自隐（visible="0" + view_dirty），
  隐时打开：visible="1" + hosted="1" + 快照注入四串列 + `call_handler("RebuildNotes")` +
  **未读清零**（`notes_unread.set(0)`）+ view_dirty。无输入控件——无 `__focus_input`/聚焦任务。
- **装配层**：switcher 层邻位追加第四枚 overlay，**仅 `notification_visible()` 时推层**
  （9559 switcher 块同款，隐匿渲染零成本）。
- **Esc 仲裁链**：`WmCommand::ExitDesktop` 门（9251）扩 `|| notification_visible()`；
  **键盘独占**：per-App focused 门（9701–9702）扩 `&& !notification_visible()`；
  **键盘订阅**：switcher 块（9736–9752）同型第四块，`escape_forward=true`（Esc Captured 转发，
  幂等由 handler visible 门控保证）。

## 4. D4/D5 对账：dock 铃铛 + 投影 v1.2（T4 落码）

- **shell.at**（两分支各一处，置于行尾布局钮之后）：
  ```text
  row { items-center gap-0
    button (icon: "bell") { onclick: .NotificationToggle  style: "h-9 w-9 px-0 text-sm" }
    if .__wm_notes_unread != "0" && .__wm_notes_unread != "" {
      text .__wm_notes_unread { style: "text-xs text-error" }
    }
  }
  ```
  （`""` 守卫：宿主未写入时 model 缺省空串不该出空 badge；`&&` 已证——calculator/calendar 使用。
  msg 块增 `NotificationToggle`；on 臂 `.__desktop_cmd = "notes_toggle"`；头注接缝清单升 v1.2。）
- **投影（`sync_shell_windows`）**：
  - `__wm_notes`：`state.desktop.notifications` 全量 → Obj 数组 `{id,kind,msg,at}`（全串值，
    协议条目同型）；**写 shell 一处**（badge/合同面）；面板走快照注入不消费本投影。
  - `__wm_notes_unread`：`notes_unread` 十进制串。
  - 指纹尾段：`fp.push_str(&format!("|notes:{}:{}:{};", len, front_id, unread))`（§0 定案 6），
    写侧随整组原子换装（wins/meta/workspaces/mru/notes/fp）。
  - 借用注意：`notifications` 为 `RefCell`，构建 Obj 数组前先 clone 出快照再写 app 状态。
- **协议文档 `schema/projection-protocol-v1.md` 文内升版 v1.2**（文件名不动，478 先例）：
  标题/版本行、§2 增 `__wm_notes`/`__wm_notes_unread` 两行、§3 指纹式补 notes 段、
  §4 词表增四动词行（notify/notes_toggle/notes_clear/notes_dismiss，载荷与宿主语义含
  「msg 单行约束」「NotesToggle 召唤执行体」）、§6 v1.2 变更记录（纯增量向后兼容声明）。
- shell.at / notification_center.at 头注接缝清单同步 v1.2。

## 5. T5 接线收口落点

- `NotesToggle` 执行臂已在 T3 随执行体落——T5 验证**接线**：dock 钮 → `__desktop_cmd` →
  drain → 臂 → 面板开合 → badge 归零，无头端到端一测串起。
- boot 恢复接线：boot Desktop 分支 `dock_edges` 赋值邻位（renderer.rs:7111 后）调
  `restore_notifications(&mut session)`（desktop 模式限定；独立模式不读 storage）。
- 接线后无头端到端（TDD 测试映射 §7 `notif_end_to_end_toggle_dismiss_restore`）：
  notify → 投影 unread=1 → toggle（可见+清零）→ dismiss → 落盘断言 → 新会话 restore 读回。

## 6. 运行时序与已知边界

- **badge 刷新时序**：投影同步在每 update 周期**头部**（renderer.rs:9113）而排空在臂尾——
  ServiceTick 期到达的 notify 其 badge 于下一周期（≤400ms）显现；DM::App 路径（dock 钮、
  LaunchApp 臂尾 9231 同步）同周期即达。toast 层既有节奏同款，不改。
- **storage 测试隔离**：`STORAGE_MAP` 进程级全局 + `storage_file()` 按 cwd 哈希回退——
  持久化单测**必须**先设 `AUTO_VM_STORAGE_FILE` 指向本测试独占临时文件再触发首次 load
  （nextest 每测独立进程，天然隔离；plain `cargo test` 同进程并发下该测自证边界，成文注释）。
- **notify 动词到达面**：任何特权 App 的 `__desktop_cmd`（shell/launcher/switcher/面板自身）
  均经排空管线，`notify` 天然四路可达——App 主动通知 v1 由测试以面板/shell 记录驱动验证。
- **面板不拦截 toast**：通知浮层（bottom-center）与面板（右下）视觉并存，双面一体语义即如此，
  不做「面板开着抑制 toast」（非目标外，留 M4 settings）。

## 7. TDD 测试映射（全部含 `notif`/`note` 命中 `-E 'test(notif or note)'` 过滤器）

| 测试名（renderer.rs tests 模块） | 步 | 断言面 |
|---|---|---|
| `notif_command_roundtrip_encodes_and_parses` | T2 | 四动词 encode/parse 往返（\u{1E} 与 \t 双轨分符）；坏载荷（notify 缺段/notes_dismiss 坏 id）跳过不 panic |
| `notif_history_fifo_capacity_and_mru_order` | T2 | front 插入序、NOTES_CAP=50 环绕（最旧弹出） |
| `notif_unread_semantics_panel_visibility` | T2 | 面板关 +1 / 面板开不加 / 开面板清零 / boot 恢复后 0 |
| `notif_storage_roundtrip_slots` | T2 | push → persist 10 槽形态 → 新会话 restore 读回 MRU 序；坏槽跳过 |
| `notif_push_dual_face_history_and_toast` | T2 | push_notification 单调用 = 入史 + toast + 未读三联动 |
| `notif_center_summon_headless` | T3 | toggle 懒挂载/快照 rows 序=MRU/未读清零/Dismiss 写 `notes_dismiss\t<id>`/ClearAll 写 `notes_clear`/Escape 自隐（switcher roundtrip 同型） |
| `notif_projection_notes_and_fingerprint` | T4 | `__wm_notes` 条目四字段、`__wm_notes_unread` 串、指纹 `notes:` 段翻转门控 |
| `notif_end_to_end_toggle_dismiss_restore` | T5 | §5 端到端链 |

## 8. 步骤-落点对照（执行期勾选用）

- T2：session.rs（NotificationEntry/三域/四臂 encode+parse）+ renderer.rs（push_notification/
  persist_notes/8 改道/Notify+NotesClear+NotesDismiss 三臂）+ 测试 1–5。
- T3：assets/notification_center.at 新建 + shell.rs 装载函数 + session.rs 四路 + renderer.rs
  toggle 执行体/drain 四路/装配层/Esc/订阅 + lucide `info` 补表 + 测试 6。
- T4：shell.at 铃铛两分支 + sync_shell_windows 投影/指纹 + 协议文档 v1.2 + 两 .at 头注 + 测试 7。
- T5：boot restore 接线 + 测试 8（NotesToggle 臂 T3 已落，本步只验接线）。
- T6/T7：按计划文原样执行。
