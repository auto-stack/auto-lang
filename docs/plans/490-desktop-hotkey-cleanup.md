---
plan_id: PLAN-490
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-hotkey-cleanup
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 7
---

# [PLAN-490] 桌面热键清理——宿主共存键位表 + 数据级可配置 + VM 行点击 parity

## 变更摘要

463-479 沉淀的桌面热键表（Esc/Ctrl+Tab/Alt+Tab/Ctrl+Space/Ctrl+Alt+G·L·F/
Ctrl+Alt+←→/Ctrl+Alt+Shift+←→）在设计与宿主 Win11 共存时遗留三个冲突点
和一个焦点盲区（用户 2026-08-30 会话提出 Super 系被吞问题后盘点确认）：

1. **Alt+Tab 窗口循环**：Win11 系统保留（OS 任务切换器），焦点在桌面窗时
   应用侧大概率收不到——死键位；功能已被 478 Ctrl+Tab switcher 实质覆盖。
2. **Ctrl+Space launcher**：中文 IME 经典开关，抢键环境下不可靠；464 已留
   Ctrl+Alt+Space 兜底但无任何用户可见性。**用户 2026-08-30 实测确证**
   （028-launcher 使用反馈：本机 Win 中文 IME 下 Ctrl+Space 完全无效）。
3. **Ctrl+Alt+方向键 分区切换**：Intel 核显驱动"屏幕旋转"快捷键，装驱
   机器被驱动层吞掉。
4. **焦点盲区**：键盘订阅仅桌面窗持焦点时有效——焦点落在 docked 原生窗
   （486 触发面之后常态）时所有桌面热键失灵（T5 实测 Esc 需 AppActivate
   才生效）。
5. **VM 轨布局件 onclick 静默丢弃（用户 2026-08-30 实测：launcher 候选
   列表只能键盘选、鼠标点不中）**：`.at` 的 `row { onclick: .Launch(...) }`
   在 VM 轨转换层 `aura_view_builder.rs:1697 convert_row_tracked_ctx` 不提取
   onclick，且 `View::Row`（view.rs:254）无事件字段——声明被静默丢弃，
   iced 渲染为纯布局行不可点。**同族缺口**：`View::Column/Container` 亦无
   事件字段、对应转换不提取（div 形态实证：auto-musk specs 树
   `specs_category.at:239`）。Vue 轨正常（`ui_gen/vue.rs:12249` 泛映射
   onclick→@click，div 可点）→ VM/Vue parity 缺口（GOAL-007 域），影响
   仓内一切布局/容器元素 onclick 写法，不止 028。

本期做四件事：**退役死键位 + 冲突键位改宿主安全默认 + 键位表数据级可配置
+ VM 轨布局件点击 parity 修复**（前三件为热键域；第四件随用户同日反馈
并入——launcher 可用性修复，方案 b：视图模型带事件 + iced mouse_area
包装）。键位可配置（`shell.keys.*` storage 覆盖，472 §6 dock 配置同型
先例）。Super 系组合与 RegisterHotKey 全局层（解决焦点盲区）**不在本期**
——作为 Phase 2 增强候选登记待澄清。

## 目标

- **G1 Alt+Tab 退役**：删除/降级 `renderer.rs` 键盘订阅的 Alt+Tab→CycleWindow
  臂（463 键位 v1 的遗留），窗口循环语义由 Ctrl+Tab switcher 承担；文档同步。
- **G2 冲突键位宿主安全化**：分区切换默认键从 Ctrl+Alt+←/→ 迁至
  Ctrl+Alt+[ / ]（无系统/驱动冲突族）；launcher 召唤保持 Ctrl+Space 但把
  Ctrl+Alt+Space 兜底**显性化**——协议文档 + 帮助面之外，launcher 入口
  UI（028 召唤按钮文案/空态提示）同步标注实际可用键位（用户实测 Ctrl+Space
  在中文 IME 机为死键，兜底必须可被发现；不换默认——肌肉记忆友好 +
  shell.keys.* 可覆盖兜底）。
- **G3 键位表数据级可配置**：热键表提升为 `shell.keys.<action>` storage 键
  可覆盖（缺省 = 内置默认表；覆盖链 运行时配置 > 内置，472 dock 配置同型），
  桌面 Init 读入。**非目标**：热键自定义 UI（settings M4 接管）；Super 系
  组合；RegisterHotKey 全局热键层（焦点盲区根治，Phase 2 候选）。
- **G4 VM 轨布局件点击 parity（方案 b，用户 2026-08-30 选定）**：
  `View::Row/Column/Container` 增可选 `onclick: Option<M>` 事件字段 +
  转换层（aura_view_builder tracked/untracked 两族）提取 onclick + iced 侧
  mouse_area 包装发射消息——`.at` 单一真源语义不动、Vue 轨零改动（已通），
  VM 补齐即双端 parity 闭合。覆盖面=布局/容器族三节点（`row`/`col`/
  `container|div`；028 候选行为 row 形态，auto-musk specs 树为 div 形态
  ——同族缺口两实证）。**非目标**：hover/右键等其它布局件事件
  （button 的 on_right_click 先例在案，需要时同型扩展）；`tab` 等余下
  元素经 T5 盘点归族（挂 Container 或另立跟进）。

## 架构方案

```
renderer.rs 键盘订阅（纯函数段） ──读取──▶ HotkeyTable
                                          ▲
storage shell.keys.<action>（Init 读入） ──┘（缺省回退内置表）
```

- **HotkeyTable 纯逻辑层**（`ui/session.rs` 或独立小模块）：`action →
  (modifiers, key)` 映射 + 序列化形态（"ctrl+alt+bracketleft" 风格字符串）；
  缺省表 = G1/G2 清理后的新默认；storage 覆盖解析失败静默回退默认（坏配置
  不炸桌面）。
- 订阅处逐臂改查表判定（维持既有 Some(DM) 返回形状，零事件流改动）。
- shell Init（.at 侧）不参与——键位消费在宿主侧，shell.at 只读投影不变
  （I9 不受影响）。

**G4 布局件点击 parity 链**（方案 b，三层各加一环，沿既有先例）：

```
.at row/col/div{onclick} ─▶ View::Row/Column/Container（view.rs，
                              +onclick: Option<M>）
                        │ convert_row/column/container_tracked_ctx
                        │ + untracked 三点（aura_view_builder 提取
                        │   onclick —— 沿 :2076 text onclick→Button 的
                        │   事件提取先例 event_to_message_with）
                        ▼
                AbstractView::Row/Column/Container（renderer.rs，+on_click）
                        │ render_dynamic_view 布局臂：on_click 有值时以
                        │ mouse_area(el).on_release(消息) 包装（右键先例
                        │   renderer.rs:3134；无值 = 原样，零行为变化）
                        ▼
                iced Element（布局件可点，点击发射声明消息）
```

- Vue 轨零改动（`ui_gen/vue.rs:12249` 泛映射已通）；`.at` 单一真源不动。
- `collect_input_ids`/debug 包装等对三节点的既有 match 用 `..` 通配，加字段
  不破坏（执行期以 `cargo check` 锚定）。
- ViewBuilder（view.rs 链式构造）加 `.on_click(M)` 入口，构造面与
  `onclick: Option<M>` 字段对齐。

## 技术栈

既有 iced 键盘订阅 + storage 读写管线（018/025/472 先例）；G4 用
`iced::widget::mouse_area`（renderer.rs 已 import，3134 右键先例）+
`iced_test` 机制测门面（p483/p491 同款 `--features iced-layout-tests`）。
零新依赖。

## 需求分析与背景调查

（取材 docs/specs/auto-lang/ui/overview.md §shell 段 + 会话盘点 2026-08-30）

- **直接依据**：用户会话提问"super+X 被宿主 Win11 吞掉，虚拟桌面有宿主时
  是否需要另一套快捷键"——评估结论：463 起 keymap 已避开 Super（Ctrl+Alt
  系，"不依赖 Win 键；T6 实测定案"注释在 renderer.rs:6413），无需两套键位，
  但存在上述三冲突点 + 焦点盲区。
- **用户 2026-08-30 二次反馈（同日吸收）**：028-launcher 实用两缺陷——
  ①本机中文 IME 下 Ctrl+Space 完全无效（冲突实证，强化 G2 兜底可见性）；
  ②候选列表鼠标点不中（根因=VM 轨转换层丢弃 row onclick，静态证据：
  `aura_view_builder.rs:1697` 不提取 onclick + `View::Row` 无事件字段 +
  Vue 侧 `ui_gen/vue.rs:12249` 泛映射对照；修复方案 b 由用户当场选定）。
- **键表现状**（renderer.rs:6376-6440）：见变更摘要列表；478 已把 switcher
  改道 Ctrl+Tab 并留注"Alt+Tab 保留窗口循环（463 键位 v1 不动）"——该注
  即本期退役对象。
- **storage 配置先例**：shell.dock.pinned/position/enabled（472 §6）——
  Init 读入 + 缺省回退 pack 默认 + 宿主侧解析；键位表复用同型。
- **排程**：489（ui-iced suite reds，并行会话）与本期零文件交叠预期
  （489 动测试基建，本期动 renderer 订阅段 + session/storage）；**491**
  （VM Tab 焦点环遍历，reviewed 待合）与本期间文件（renderer.rs）但异段
  （491 动 keyboard_event_message 分派臂 ~6576 与 update 遍历臂 ~8935，
  本期动桌面热键订阅段 ~6376-6440 与 render_dynamic_view 布局臂 ~14199）
  ——后合者 rebase 一次即可，无语义冲突。执行前核对。

## 详细设计

### 1. HotkeyTable（`ui/session.rs` 增段）

```rust
/// 桌面动作键（可配置面；字符串形态 = storage 覆盖与文档共用）
pub enum HotkeyAction { ExitDesktop, CycleSwitcher, SummonLauncher,
    SetLayoutGrid, SetLayoutStack, SetLayoutFree, WorkspaceNext, WorkspacePrev,
    SendToNext, SendToPrev }
pub struct HotkeyTable { map: HashMap<HotkeyAction, KeySpec> }
```

- `KeySpec { ctrl: bool, alt: bool, shift: bool, key: KeyName }`，`KeyName`
  枚举（Escape/Tab/Space/Left/Right/BracketLeft/BracketRight/G/L/F）。
- `HotkeyTable::builtin()` = 清理后默认表；`from_storage_overrides(
  &[(action, str)])` 解析 "ctrl+alt+bracketleft" 形态（大小写不敏感、
  '+' 分隔、未知词丢弃该条回退默认）。
- 匹配 API：`fn matches(&self, action, &modifiers, &key) -> bool`（订阅处
  逐臂调用，替代硬编码布尔式）。

### 2. 默认表变更（G1/G2）

| 动作 | 旧 | 新 |
|---|---|---|
| CycleWindow (Alt+Tab) | alt+tab | **删除**（switcher 承担） |
| WorkspaceNext/Prev | ctrl+alt+right/left | **ctrl+alt+bracketright / bracketleft** |
| 其余 | 不变 | 不变（含 launcher ctrl+space + ctrl+alt+space 双收） |

### 3. storage 覆盖 + Init 读入

- 键形态：`shell.keys.workspace_next` = "ctrl+alt+right"（用户可把分区切回
  方向键——Intel 驱动冲突机的逃生舱）。
- 读入点：desktop boot（`open_desktop` 邻位或 shell Init 同拍）——具体落点
  执行期定（storage 读取在宿主侧有 018/025 先例）。
- 投影/协议面零改动（键位是宿主侧消费，不进 shell.at）。

### 4. 文档同步

- `schema/projection-protocol-v1.md`：不涉及（键位不在协议面）。
- `docs/specs/auto-lang/ui/overview.md` 热键表段 + 键位说明（含 Ctrl+Space
  IME 兜底显性化）随 merge 回写。

### 5. G4 布局件点击 parity（方案 b 三层落点）

- **view.rs**：`View::Row/Column/Container` 增 `onclick: Option<M>`；
  ViewBuilder 链式 `.on_click(M)`；既有构造点（含 tests）以 `None` 补位——
  字段命名对齐 `View::Button.onclick` 先例。
- **aura_view_builder.rs**：`convert_row_tracked_ctx` / `convert_column_
  tracked_ctx` / `convert_container_tracked_ctx` 与 untracked 三点共六处提取
  `onclick`（`aura_events_get_base(events, "onclick")` +
  `event_to_message_with`，沿 :2076 text 提升先例；未提取 = None，行为
  持平）。**注意**：onclick 提取需在 props/events 进 children 转换前完成
  （消息在子节点 move 前构造）。
- **renderer.rs**：`AbstractView::Row/Column/Container` 增 `on_click:
  Option<IcedMessage>`（convert_view_messages 传递）+ `render_dynamic_view`
  布局臂有值时 `mouse_area(el).on_release(...)` 包装（3134 右键先例同型；
  None = 原样直出，无包装开销）。
- **受影响面预判（T5 盘点核）**：028 候选行（row 形态，`app.at:120/135/
  181/191`）；auto-musk specs 树（div 形态，`specs_category.at:239` 等）；
  余下 `tab` 等元素形态归族盘点。仓内 grep `onclick` × 布局/容器元素清单
  入执行记录，逐条核 VM 轨点击恢复或列残留跟进。

## 测试设计

1. **T1 纯单测**：HotkeyTable 内置表匹配矩阵（每动作正反例）；storage
   覆盖解析（合法串/坏词/空串回退）；新默认表锁定（Alt+Tab 无臂、
   bracket 键有臂）。
2. **T2 订阅行为测**：键盘订阅纯函数逐臂过新表（Alt+Tab → None；
   Ctrl+Alt+] → NextWorkspace；覆盖表后 Ctrl+Alt+Right 恢复生效）。
3. **T3 storage 往返**：Init 读入覆盖键 → 表生效；坏配置 → 默认表 +
   不 panic（472 dock 配置测试同型）。
4. **T4 实机冒烟**：全屏 ui_desktop + t5_smoke 驱动点按验证新键位分区
   切换（AppActivate 聚焦后发键——T5 的 Esc 经验复用）；IME 开启态
   Ctrl+Alt+Space 召唤 launcher 实机一次（用户实测场景直证）。
5. **T5 布局件点击机制测（G4，先红后绿）**：iced_test（`--features
   iced-layout-tests`，沿 p483/p491 门面）——
   - `p490_row_onclick_fires`（红）：`row { onclick }` 形态视图经
     render_dynamic_view 后，Simulator 坐标点击行矩形 → 断言收到声明的
     IcedMessage（现状：无消息=红）。
   - `p490_col_onclick_fires`：col 同型。
   - `p490_container_div_onclick_fires`：div（Container）同型（musk specs
     树形态）。
   - `p490_layout_without_onclick_inert`（锚）：无 onclick 的三节点点击
     零消息、无包装行为变化。
   - `p490_launcher_row_launches`：028 候选行形态（for r in ranked 的
     row{onclick:.Launch}）点击 → Launch 消息（应用形态直证）。
6. **T6 实机/MCP 回归**：028-launcher MCP `autoui_action press` 候选行
   launch 照常 + 真鼠标点击实机一次（P483-3 同通道受阻则 MCP 代验留档，
   沿 491 先例）。

## 验收标准

1. Alt+Tab 不再产生窗口循环（订阅返回 None）；Ctrl+Tab switcher 行为
   不回归。
2. Ctrl+Alt+[ / ] 切分区默认生效；storage `shell.keys.workspace_next=
   "ctrl+alt+right"` 覆盖后方向键恢复生效（T2+T3 测试锁）。
3. 坏配置不炸：任意非法 shell.keys.* 值 → 默认表照常工作（T3）。
4. `cargo tf` 全绿；`cargo check -p auto-lang` 触碰文件零新增警告；
   ui feature 档不回归（i18n 既有红除外——489 接管）。
5. 文档：overview 热键段更新（含 IME 兜底说明）；G1 退役在 486/490 交接
   注记可追；launcher 入口 UI 标注 Ctrl+Alt+Space 兜底键位（G2 显性化）。
6. **G4**：VM 轨 `row/col/div(container) { onclick }` 点击发射声明消息
   （T5 机制测 p490 三形态转绿）；无 onclick 布局件零行为变化（锚测）；
   028 候选行鼠标可选（T6 实机/MCP 留档）；Vue 轨零改动。
7. **G4 受影响面**：仓内 `.at` 布局/容器元素 onclick 用法盘点清单入执行
   记录（含 auto-musk 侧 specs 树 div 形态），逐条核或列残留跟进
   （`tab` 等余族归并或另立）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **HotkeyTable 纯逻辑**：`ui/session.rs` 增 HotkeyAction/KeySpec/
   HotkeyTable（builtin + from_storage_overrides + matches）+ T1 单测。
   验证：`cargo nextest run -p auto-lang --lib --features ui-iced hotkey`。
2. **默认表迁移**：builtin 表按 G1/G2 新默认；renderer.rs:6376-6440 订阅
   段逐臂改查表 + T2 订阅行为测（Alt+Tab 无臂/bracket 臂/覆盖恢复）。
   验证：`cargo nextest run -p auto-lang --lib --features ui-iced hotkey`。
3. **storage 覆盖读入**：boot 侧读 `shell.keys.*` → 构建会话表 + T3 往返
   测试（合法/坏配置）。
   验证：`cargo nextest run -p auto-lang --lib --features ui-iced hotkey storage`。
4. **G4 红测试**：renderer.rs `line_edit_tests`（p483/p491 相邻）新增
   p490 布局件点击五测（row/col/div 先红、无 onclick 锚、launcher 形态），
   跑 `cargo test -p auto-lang --lib --features iced-layout-tests p490`
   确认红（onclick 提取/字段未落地）。
5. **G4 实现三层**：view.rs（View::Row/Column/Container +onclick 字段 +
   ViewBuilder .on_click）→ aura_view_builder.rs（tracked/untracked 六转换
   点提取）→ renderer.rs（AbstractView 传递 + mouse_area 包装）。受影响面
   grep 盘点清单记入计划。
   验证：p490 五测全绿 + `cargo check -p auto-lang` 零新警告
   + p483/p491 相邻测试不回归。
6. **文档 + overview 注记**：热键表说明（新默认/IME 兜底/覆盖键形态）+
   G2 launcher 入口键位标注 + overview 行点击 parity 段。
   验证：`cargo test -p auto-lang --test docs_gen`（若触发生成器）。
7. **实机冒烟 + 收尾**：T4 两项（AppActivate 后键位实机、IME 态
   Ctrl+Alt+Space）+ T6（028 候选行真鼠标/MCP 点击 launch）；`cargo tf`
   全量门；状态翻 execution_done。
   验证：`cargo tf`。

## 复审记录

## 待澄清事项

- **G2 默认键定案复核（2026-08-30 用户实测吸收）**：用户确证中文 IME 机
  Ctrl+Space 为死键后，本计划仍取"不换默认 + Ctrl+Alt+Space 兜底显性化 +
  shell.keys.* 可覆盖"（吸收用户反馈时的隐含默认——未要求换键）；若实机
  验收仍感不可发现/不可用，Phase 2 重议默认键位。
- **G4 并入注记（2026-08-30 用户指定）**：行点击 parity 原属独立缺陷，
  用户指定并入本期（方案 b 当场选定）；域相邻（launcher 可用性），
  文件面与热键段无交叠（renderer.rs 异段）。

- **与 488（OLE 拖放 P3）的热键域协调（2026-08-30 调度补记）**：488 T7 将在
  桌面级热键段增 Ctrl+V 粘贴臂，与本计划 G3 键位表化同域——后合者适配：
  490 先合，则 488 的 Ctrl+V 臂挂入 `shell.keys.paste` 键位表；488 先合，则
  本计划收编该臂进表。领取顺序不设硬约束。
- Alt+Tab 退役是否保留 storage 逃生舱（`shell.keys.cycle_window` 显式配置
    可复活 alt+tab）——**默认保留解析能力但内置表不配**，成本零（表驱动
    天然支持）。
- Ctrl+Alt+[ / ] 在非美式键盘布局的可达性（欧陆布局 bracket 需 AltGr——
    与 ctrl+alt 组合可能冲突）——中文/美式布局无碍；欧陆用户走 storage
    覆盖逃生。执行期在文档注记。
- RegisterHotKey 全局层（Super 系 + 焦点盲区根治）为 Phase 2 候选——
    本期不做，需求出现立项。
- 事件泵吞吐（P486-1 债务）不并入本期——域不同（native dock 事件泵 vs
    桌面键位），独立清偿。
