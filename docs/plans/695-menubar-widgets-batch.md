---
plan_id: PLAN-695
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: menubar-widgets-batch
author: [agent]
created_at: 2026-09-23
updated_at: 2026-09-23

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [widgets/menubar-family]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-lang/ui-gen, widgets, auto-edit]
current_step: 14
total_steps: 14
---

# [PLAN-695] menubar-widgets-batch

## 变更摘要

把 AutoUI 的 menubar 从"七件主干 + 硬编码深色"补全为 shadcn-vue 对齐的**完整 MenuBar 组件族**，
并在 widgets-gallery 落完整示例、最终回灌 auto-edit：

1. **W1（auto-lang，核心）**：schema/registry/双臂发射器三层补齐缺失的 8 个 menubar 元素
   （label / shortcut / radio-group / radio-item / sub / sub-trigger / sub-content / group）；
   VM 臂（解释态 + a2r 静态降层）根修浅色配色（硬编码 `#16171B`/zinc → popover 语义 token）、
   disabled 置灰、开态 trigger 高亮、跨菜单 hover 切换、submenu 嵌套弹层；键盘导航按时间盒裁定。
2. **W2（auto-os，widgets-gallery）**：menubar 页从裸壳 `menubar {}` 重写为 shadcn 官方 demo
   等价的完整示例（四菜单 + checkbox/radio/submenu/shortcut/label/disabled/separators + 状态绑定），
   双臂实机走查。
3. **W4（auto-edit）**：消费升级——双轨重生成、自有 ctx_menu/app.at 硬编码色改 token、
   "行尾"分组改用 submenu、probe_menu.py 回归。
4. **W3（滚动批次协议）**：gallery 其余未完善组件（49 组件盘点结论）作为滚动工作包在册，
   用户点名一件消化一件，均走本计划的 revision 增量。

非目标：widgets-gallery 其余 40 页的交互化改造不在本轮执行（仅登记）；dropdown-menu /
context-menu 的 VM 臂实现（`iced:none` 断供）不在本轮（登记为滚动批次候选）；Vue 臂 menubar
脚手架 16 件资产不动（已齐备且 token 化）。

## 目标

- **G1 DSL 完备**：menubar 族 16 元素全部可在 .at 声明，schema 三端注记（web/iced/gpui）如实反映实现面。
- **G2 双臂保真**：Vue 臂全部发射 reka-ui 标准件（零手写 div、零硬编码 zinc 色）；VM 臂双解释态
  （`aura_view_builder` 解释态 + `rust.rs` a2r 降层）行为与配色对齐 shadcn 语义。
- **G3 主题正确**：菜单面板/项/分隔线/快捷键配色消费 popover 语义 token，浅色模式下可读
  （根修 auto-edit"浅色模式下颜色不对"）。
- **G4 gallery 完整示例**：widgets-gallery menubar 页达到 datatable/alertdialog 级的完整度
  （多 preview、真实交互、状态绑定、属性表、源码串）。
- **G5 消费方回灌**：auto-edit 双轨重生成后浅色可读、menubar-label 不再被静默丢弃、
  "行尾"分组升级为 submenu。
- **G6 滚动跟踪**：gallery 未完善组件需求进一件登记一件，单一计划承载，避免碎片化立项。

成功形态：gallery menubar 页在 Vue/VM 双臂与 shadcn-vue 官方 demo 视觉等价、交互等价；
auto-edit 菜单在浅色主题下全部可读；后续"把 X 组件也做完善"类需求直接追加到本计划。

## 架构方案

**跨仓布局（PLAN-692 先例）**：主仓 auto-lang（生成器/schema/VM 臂），兄弟 worktree 承载
auto-os（widgets-gallery 语料真身，PLAN-590 已迁出）与 auto-edit（消费方）。分组平铺：

```
D:/autostack/.wt/lang-695/
├── auto-lang    # branch plan-695-dev（主实现面）
├── auto-os      # widgets-gallery 语料（branch plan-695-dev）
└── auto-edit    # 消费方回灌（branch plan-695-dev）
```

gallery 发现链 `resolve_os_top_dir`（env → 组内 `../auto-os` → 主检出）在分组布局下天然解析到
worktree 内的 auto-os，无需 junction（worktree 红线）。

**接入模式（PLAN-692 command 家族模板照抄）**：
schema 后端注记翻转 → registry spec+kebab 别名 → 专用发射器 kind 分支 → scaffold 资产为真身
（Vue 臂）→ 双单测 → gallery 实机验证。menubar 与 command 的差异：menubar 的 VM 臂本轮**实现**
（command 是显式 `iced:none` 另立），故 schema iced 注记随实现翻转。

**VM 臂三路同源收敛**：menubar 在 VM 侧有三条 lowering 路径（actions DSL 合成 Plan-418、
解释态声明式族 Plan-630、a2r 静态降层 Plan-674），本轮改配色/行为时三处同步，共用
`menu_item_button_view` 与 Popover 原语；开合态仍走 `action_config::menubar_open()` 全局注册表
（含 TimeSource 泵豁免规则，PLAN-664），不引入新状态通道。

**submenu 机制**：复用 Popover 原语嵌套（父菜单面板内 item 再挂 `EndTop` Popover），
开合仍走 `menubar_open` 键空间（`menu_id::sub_id` 复合键），不新发明 overlay 通道——
规避 `renderer.rs:7665` "Overlay 走 `_ => Empty` 兜底"的已知差异面。

**滚动批次协议（W3）**：新组件需求进入本计划的唯一入口 = 追加 `W5.N` 任务 + `plan_revision`
+1（契约文本变更须递增）；audit 盘点表（§5.3）作为候选池，执行顺序由用户点名驱动，
plan 文件是唯一跟踪面，不另立计划。

## 需求分析与背景调查

### 4.1 授权记录

- 用户（2026-09-23）：以 shadcn-vue menubar 为参照，结合 auto-edit 既有 menu，做一套 AutoUI
  完整 MenuBar 组件；在 gallery 展示完整示例；最后应用到 auto-edit。该需求与后续其他
  gallery 未完善组件需求合并用一个新计划跟踪。先分析后起草（本计划即产物）。
- 允许仓库：auto-lang（主）、auto-os（widgets-gallery）、auto-edit（消费方）。
- 预算/自动续跑限制：未指定；执行仍走 /auto-plan:work → /auto-plan:review → /auto-plan:merge 范式。

### 4.2 现状勘定（2026-09-23 三路探查，file:line 已核）

**auto-lang 生成器/双臂**：
- registry 七件已注册（`crates/auto-lang/src/ui_gen/widget/registry.rs:1390-1449`，PLAN-677 T-03
  补齐三形态别名）；未注册：RadioItem/RadioGroup/Label/Shortcut/Sub/SubTrigger/SubContent/Group。
- schema（`schema/aura.at`）：root `menubar` :873-886（web:component / **iced:unknown**）；
  子元素 :4482-4595（content/item/checkbox_item/label/menu/separator/trigger），label 为
  **web:none/iced:none**，其余 iced:none；schema 镜像 `crates/auto-lang/src/aura/schema.rs:2515-2641`。
- Vue 发射器 `crates/auto-lang/src/ui_gen/vue.rs:6393-6549` `generate_menubar_view_node`：
  kind 白名单仅六件；shortcut 硬编码 `text-zinc-500`（:6539）；调度点 :7137-7139。
- VM 解释态 `crates/auto-lang/src/ui/aura_view_builder.rs:7486-7694` `convert_menubar_component`
  + `:7412-7476` `menu_item_button_view` + `:7696+` actions 合成；a2r 降层
  `crates/auto-lang/src/ui_gen/rust.rs:5339-5474`（预扫描推送 `__MenubarToggle/__MenubarClose`
  :1247-1269）；消息处理 `crates/auto-lang/src/ui/iced/renderer.rs:17077-17142`。
- **浅色翻车根因**：面板 `w-44 bg-[#16171B] border border-zinc-700` 三处硬编码
  （`aura_view_builder.rs:7650、7826`、`rust.rs:5401`），分隔线 `bg-zinc-700`（rust.rs:5412），
  文字 zinc-200/300/500（`aura_view_builder.rs:7424-7455`）；主题系统 `TokenName::Popover`
  早已备好 light/dark 默认值（`crates/auto-lang/src/design_tokens/registry.rs:26-35,233-255`）
  但 menubar 零消费（`ui/style/theme/mod.rs:324-326` 将 popover 投影收敛为 Card）。
- disabled 项 VM v1 **隐藏而非置灰**（rust.rs:5347-5349 注）；menubar-label 双侧 lower 均不消费
  （rust.rs 匹配臂 `_ => {}`、vue.rs 白名单缺失）——auto-edit "行尾"标签被静默吞。
- coverage：`crates/auto-lang/src/aura/element_coverage.rs:307-314` menubar = `NotYet`。
- 既有测试：`rust.rs:14098` `menubar_family_lowers_to_popover_row`；Vue 臂 PLAN-677 T-03 测试；
  golden 门 `crates/auto-lang/tests/gallery_golden.rs`；schema 漂移门 schema_drift 测试。

**auto-edit（消费方）**：
- menu = PLAN-630 声明式族（`D:/autostack/auto-edit/specs/auto-edit/src/front/app.at:92-142`，
  文件/编辑/视图/帮助四菜单）+ actions 块（:39-83）；走标准生成管线（regen_vue.py / vm 轨），
  无手写组件；右键 `EditorCtxMenu` 为 popover 原语（`src/front/ctx_menu.at:13-41`）。
- **自有硬编码色**：`ctx_menu.at:19` 面板 `bg-[#16171B]` + `:12` 菜单项 `text-foreground`——
  浅色下黑字深板不可读；`app.at:161` 激活 tab `bg-[#1C1D24]` 同型问题。
- 两轨生成物 stale（main.rs 2026-09-22 / App.vue 2026-09-21 均落后 app.at）；
  回归探针 `specs/auto-edit/tests/probe_menu.py` 在案可复用。
- 他方 WIP 在案：auto-edit main 有未跟踪 plan010 草稿与 stylekit 删除态——本计划 W4 全部
  在独立 worktree 进行，不碰主检出 WIP。

**widgets-gallery（auto-os 仓）**：
- 语料真身 `D:/autostack/auto-os/widgets-gallery/src/front/`（70 页）；auto-lang 仓内
  `examples/widgets-gallery/` 是残壳 pac（render=vm，front_port 3024，relative_path="/.."）。
- `pages/menubar.at:17` 仅 `menubar {}` 裸壳一行，零菜单项。
- 49 组件盘点：缺页 2（input-otp/resizable）、合格 6（alertdialog/datatable/toast/sonner/
  carousel/code-editor）、其余 41 页零受控态零事件的静态文档页；menu 系姊妹件
  dropdown-menu（缺 disabled/shortcut/checkbox/sub 演示）、context-menu（VM 臂全家族
  `iced:none` 断供）同样简陋——详表 §5.3 滚动批次候选池。
- VM 侧已知差异：动态路径 Overlay 类视图走 `_ => Empty` 兜底（`renderer.rs:7663-7665` 注），
  四次同坑在案（Grid/video/Popover/WindowThumbnail）——本轮 submenu 显式加臂规避。

### 4.3 依赖与前置

- **前置 1（硬）**：PLAN-692（plan-692-dev 在途，execution_done 待 review）先合入 master——
  其改动与 W1 同文件（schema/aura.at、registry.rs、vue.rs），并行必撞 rebase；且其
  `apply_schema_vue_mappings` schema-only 补建机制与 kebab 别名模式是 T-02 的模板。
- **前置 2（软）**：PLAN-693/694（a2r exe 桌面链）与本计划文件交集小（生成器 remote 字段侧），
  若在途须在开工会上对齐 rebase 顺序。
- 假设：widgets-gallery 语料继续住 auto-os 仓（PLAN-590 裁定不回迁）。

## 详细设计

### 5.1 W1 —— auto-lang：menubar 族补全 + VM 根修

**T-01 schema 扩展**（`schema/aura.at` + `crates/auto-lang/src/aura/schema.rs` 镜像再生）
- 新增 7 元素：`menubar_radio_group`、`menubar_radio_item`、`menubar_shortcut`、`menubar_sub`、
  `menubar_sub_trigger`、`menubar_sub_content`、`menubar_group`——tier `web_component`，
  `vue` 映射指 `@/components/ui/menubar`（资产已在），props 对齐 shadcn
  （radio-group: model-value；radio-item: value/disabled；shortcut: 无（文本子）；sub 族: 无）。
- `menubar_label`：web:none → component（scaffold `MenubarLabel.vue` 在）。
- iced 注记如实翻转：root/menu/trigger/content/item/checkbox-item/separator/label 由
  `unknown`/`none` → `component`（T-04~T-07 落地后）；sub/radio/shortcut 族随对应任务翻转。
- root `menubar` 的 sub_widgets 列表扩至全族。
- 锚点：既有块 schema/aura.at:873-886、:4482-4595。

**T-02 registry 补全**（`crates/auto-lang/src/ui_gen/widget/registry.rs`）
- 为 8 个新/改元素补 spec（MenuBarRadioGroup/MenuBarRadioItem/MenuBarLabel/MenuBarShortcut/
  MenuBarSub/MenuBarSubTrigger/MenuBarSubContent/MenuBarGroup），每个带 `menu-bar-*` /
  `menubar_*` / `menubar-*` 三形态别名（PLAN-677 别名纪律）；沿用 PLAN-692
  `apply_schema_vue_mappings` 的 schema 映射回填位次。

**T-03 Vue 发射器扩展**（`crates/auto-lang/src/ui_gen/vue.rs:6393-6549`）
- kind 白名单六件 → 全族：label → `MenubarLabel`；shortcut → `MenubarShortcut`
  （替换 :6539 硬编码 `text-zinc-500` span——**浅色修正的 Vue 侧**）；radio-group/radio-item →
  `MenubarRadioGroup`/`MenubarRadioItem`（model-value/value 绑定）；sub/sub-trigger/sub-content →
  三件套嵌套发射；group → `MenubarGroup`。
- checkbox-item 的 checked 若已用 `MenubarCheckboxItem` 则不动；item 顺序组装保持
  icon/title/shortcut 槽序。
- 同步 PLAN-677 T-03 既有测试 + 新增每 kind 断言；golden 更新（gallery_golden）。

**T-04 VM 配色主题化 + disabled 置灰**（根修，三路同改）
- `aura_view_builder.rs:7650、7826`（解释态面板）、`rust.rs:5401、5412`（a2r 面板/分隔线）、
  `:7424-7455`（项文字/图标/快捷键色）：字面 `#16171B`/zinc-* → popover 语义 token。
  实现路径：`design_tokens` 已有 `TokenName::Popover/PopoverForeground`（registry.rs:26-35）；
  需打通 `ui/style/theme/mod.rs:324-326` 的投影收敛（popover 不再折到 Card）+ dark_mode 感知
  解析；a2r 静态降层侧若拿不到运行期主题，则发射 token 类名（与 Vue `bg-popover` 同词汇），
  由 VM 渲染层解析。
- disabled：隐藏 → 置灰（降透明度/前景色 muted），三路一致。
- 验收锚：浅色主题下截图（T-11）。

**T-05 VM 开态差分 + 跨菜单 hover 切换**
- trigger 开态高亮（`menubar_open()==Some(id)` 时 accent/muted 底色，对齐 shadcn
  `data-[state=open]:bg-accent`）。
- hover-switch：开态下指针进入其他 trigger 直接切换（on_hover/MouseArea 包裹 + 判
  `menubar_open()` 非空；关闭态悬停不开——对齐 shadcn menubar 语义）；不破坏
  popover.rs "外点 dismiss 不捕获保留切换语义"的既有契约（menubar-snapshot 四规则）。

**T-06 VM submenu**（双解释态）
- `menubar-sub` 面板内三件：sub-trigger 挂 `View::Popover`（EndTop 放置 + 嵌套面板样式同 token）；
  开合键 = `<menu_id>::<sub_id>` 复合键进 `menubar_open` 注册表；Esc/外点层级关闭语义随
  Popover 原语既有捕获规则。
- a2r 臂（rust.rs 降层）同构实现；**显式新增 renderer match 臂**，不走 Overlay 兜底。
- 消息通道沿用 `__MenubarToggle`（复合键字符串），无需新消息变体。

**T-07 VM radio / label / shortcut 渲染**（双解释态）
- radio-item：group 绑定 model 字段（model-value 表达式）+ item `value`；选中态渲染
  lucide `circle-dot`（Vue 臂 reka-ui 自带）；onclick 回写 store。
- label：分组标题渲染（muted 小字，非可点）——修复 auto-edit "行尾"被吞。
- shortcut：独立子元素形态渲染右对齐 muted（与 item 的 shortcut prop 殊途同归）。

**T-08 VM 键盘方向键导航（时间盒裁定任务）**
- 目标：开态内 ↑/↓ 移项、→ 进 submenu、← 回父级、Enter 激活、Esc 关闭（Esc 已有）。
- 裁定点：若 iced 焦点模型下成本超时间盒（≥1 天量级），则实现 Enter/Esc 最小集 +
  落债 `P695-D1`（方向键导航）并在计划注记，不阻塞 AC-04 主体验收。

**T-09 coverage/文档/测试收口**（auto-lang 侧）
- `element_coverage.rs:307-314` menubar `NotYet` → 按实实现面更新；
- `docs/components/core.md` 再生（schema 变更 → Category C 门禁
  `cargo test -p auto-lang --test docs_gen`）；
- schema_drift 测试同步；grep corpus/examples 确认无存量 menubar 语料被 golden 影响
  （预期无——auto-edit 语料不在 auto-lang corpus）。

### 5.2 W2 —— auto-os：widgets-gallery menubar 完整示例

**T-10 重写 `auto-os/widgets-gallery/src/front/pages/menubar.at`**（worktree 组内 auto-os）
- 对标 shadcn-vue 官方 demo 四菜单（File/Edit/View/Profiles）：shortcut（⌘T/⌘N/⌘P…）、
  disabled 项、Share/Find 两个 **submenu**、View 菜单 **checkbox-item**（受控绑定）、
  Profiles **radio-group**、separator、**label**——一次覆盖全族 kind。
- 加一个状态绑定 preview（checked/radio 绑 store，点击回显）+ 属性表更新（全族 props）
  + `auto:` 源码串，形态对齐 alertdialog（受控范本）/code-editor（交互金标准）。

**T-11 双臂实机走查 + 截图**（autoui-verifier 技能）
- Vue 臂：`auto run`（widgets-gallery，端口 3024）；VM 臂：`auto run -r vm`；
  走查点 = 四菜单展开/submenu 二级/checkbox 勾选/radio 切换/浅色模式配色/hover 切换/disabled 置灰；
  截图入 `docs/plans/` 证据位（VM 臂注意 MCP 端口争用礼仪：`AUTOUI_MCP_PORT` 钉独立端口）。

### 5.3 W3 —— 滚动批次协议与候选池（登记，不执行）

49 组件盘点结论（2026-09-23）进入本计划候选池，**执行由用户点名驱动**，每次进入 =
追加 W5.N 任务 + plan_revision +1：

| 优先级 | 候选 | 缺口 |
|---|---|---|
| P0 | dropdown-menu 补齐 | disabled/shortcut/label/checkbox/sub 演示 + VM 臂现状勘定 |
| P0 | context-menu | demo 补 checkbox/radio/sub/destructive + **VM 臂全家族 iced:none 断供须立项** |
| P0 | 浮层受控化 | dialog/drawer/sheet/popover 受控 open 演示（对齐 alertdialog 范本）+ VM Overlay 兜底差异收口 |
| P1 | 缺页 | resizable、input-otp（schema 元素已备，补 demo 页+路由+侧栏） |
| P1 | 表单交互化长尾 | slider/switch/checkbox/radiogroup/select/combobox/tabs/collapsible/accordion/pagination/toggle-group/progress/date-picker/calendar/form 按金标准补受控态+事件 |
| P2 | 基建 | demo 发现机制单一事实源（bps-gallery registry.at 模式）、kitchen-sink 视觉回归（DEBTS #435）、widgets-gallery 纳入 P674 同口径 not-yet 勘定 |

### 5.4 W4 —— auto-edit 回灌

**T-12 auto-edit 语料修**（worktree 组内 auto-edit）
- `src/front/ctx_menu.at:19` 面板 `bg-[#16171B]` → `bg-popover` 系 token；`:12` 菜单项
  `text-foreground` 配 popover 前景；`app.at:161` 激活 tab `bg-[#1C1D24]` → token。
- "行尾"label + EOL 三项升级为 `menubar-sub`（消费 T-06 能力；若 T-08 键盘导航落债不影响）。

**T-13 auto-edit 双轨重生成 + 回归**
- `python specs/auto-edit/scripts/regen_vue.py --build` + vm 轨重生成（清 stale main.rs/App.vue/dist）；
  `tests/probe_menu.py` 回归绿；浅色模式走查截图（对照修复前"黑字深板"）。

### 5.5 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | schema/aura.at（经 aura/schema.rs 编译面） | menubar 族 7 元素新增 + label web:none→component + iced 注记按实现翻转 | shadcn 对齐 API 面；schema 如实反映双臂实现（692"后端翻转是总根源"纪律） | AC-01 |
| SD-02 | add | docs/specs/widgets/menubar-family.md | 新组件族契约：16 kind 表、双臂能力矩阵、主题 token 规则、VM 键盘面裁定 | 照 command-family.md 先例沉淀组件族 spec | AC-02/03/04 |
| SD-03 | modify | docs/specs/widgets/project.md | registry 补 menubar 行（16 资产件在库但 spec 无行） | 包 spec 如实 | AC-02 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/menubar-snapshot.md | 四硬规则扩注：hover-switch 语义、popover token 配色、submenu 复合键 | 开合契约演进留痕 | AC-03/04 |
| SD-05 | modify | docs/specs/auto-lang/ui/overview.md | 现状节 menubar 段更新（族完备/主题化） | 现状如实 | AC-01 |
| SD-06 | modify | docs/components/core.md | docs_gen 再生（schema 变更伴生） | 生成文档门禁 | AC-07 |

## 测试设计

- **门禁分级（AGENTS.md Category B/C）**：迭代期 `cargo check -p auto`（ui/aura 侧；注：`-p auto-lang`
  不编 ui lib，两包都查）+ 定向单测；fold 前 `cargo tf`（全量档，生成器横切改动）+
  `cargo test -p auto-lang --test docs_gen`（schema 定义文件变更触发 Category C）。
- **定向单测**：`cargo t menubar`（rust.rs `menubar_family_lowers_to_popover_row` 扩展 +
  vue.rs 677 系测试扩展 + 新 kind 断言）、`cargo t schema_drift`、`cargo t gallery_golden`
  （SFC 金样更新）、registry 别名测试。
- **不改 aavm 触发面**：本计划不触碰 `auto/lib/*.at`/`test/vm/aavm2/**`/`parity/**`，
  零 `taa` 义务；`cargo tv` 仅在 T-09 确认 corpus 有 menubar 语料时才需要（预期不需要）。
- **实机双臂**：autoui-verifier 双端走查（T-11/T-13），VM 臂 MCP 端口钉 `AUTOUI_MCP_PORT`；
  浅色模式切换截图为 AC-03 的实证载体。
- **auto-edit 回归**：`probe_menu.py`（既有）+ 浅色走查截图。

## 验收标准

- **AC-01 DSL 完备**：menubar 族 16 元素在 schema 注册且三端注记如实；含全族的 .at 语料
  双臂编译/发射绿。验证：`cargo t schema_drift` + `cargo t menubar` + docs_gen 门。
- **AC-02 Vue 臂保真**：gallery menubar 页 Vue SFC 全部 reka-ui 标准件（零手写 div、
  零硬编码 zinc/Hex 色，shortcut 走 MenubarShortcut/muted token）。验证：golden 更新 +
  T-11 截图。
- **AC-03 VM 浅色正确**：双解释态面板/项/分隔线/快捷键配色消费 popover 语义 token，
  浅色主题下截图可读；disabled 置灰非隐藏。验证：T-11 VM 截图（浅/深对照）+ 单测断言
  token 引用。
- **AC-04 VM 行为面**：保留开合四关闭路径（外点/Esc/失焦/handler）与 TimeSource 豁免规则
  不回归；新增 hover-switch、开态 trigger 高亮、radio 状态、submenu 打开（键盘导航按
  T-08 裁定，落债须在案）。验证：T-11 VM 走查 + renderer 消息单测。
- **AC-05 gallery 完整示例**：menubar 页含 shadcn 官方 demo 等价四菜单 + 状态绑定 preview +
  属性表 + 源码串；双臂截图入档。验证：T-11。
- **AC-06 auto-edit 回灌**：双轨生成物新鲜、浅色下菜单/ctx 菜单可读（对照截图）、
  "行尾"为 submenu、`probe_menu.py` 绿。验证：T-13。
- **AC-07 门禁全绿**：`cargo check -p auto`、定向单测、`cargo tf`（fold 前）、docs_gen
  零新增红（预存红按基线对勘）。
- **AC-08 滚动协议在册**：§5.3 候选池 + 进入协议（点名 → W5.N + revision+1）可执行，
  INDEX 已挂本计划。

## 执行步骤

前置门：PLAN-692 合入 master 后开工（§4.3 前置 1）；开工时创建分组 worktree
`git worktree add D:/autostack/.wt/lang-695/auto-lang -b plan-695-dev`（主检出）+
auto-os/auto-edit 兄弟 worktree（各自仓，branch plan-695-dev）。

- **T-01** [x] schema/aura.at 族扩展（7 新元素+label 双面翻转+iced 注记翻转；schema.rs fallback 照 checkbox_item 先例不镜像——围栏 rs 维度裁定）+ element_coverage 7 件登记 → AC-01
  （证据：e659208d2；`cargo check -p auto` 绿 + schema_drift 2/2 绿）
- **T-02** [x] registry.rs 8 spec + 三形态别名 + p695 单测 → AC-01
  （证据：9f3f75c86；p695_menubar_family_registry_complete 绿）
- **T-03** [x] vue.rs 发射器全族 kind + shortcut 组件化 + 静态 disabled → AC-02
  （证据：6ede43b02；p695_view_menubar_full_family + plan677 更新绿；golden 基线漂移勘定=预存红（auto-os master 演进未重基线，menubar.at 哈希两侧一致），重基线随 fold 门）
- **T-04** [x] VM 配色主题化根修（Color::Popover/PopoverForeground 独立投影 registry 双盘；三路字面深色全退役）+ disabled 置灰 → AC-03
  （证据：6275eaf98；p695_menubar_panel_tokens_and_disabled_dim + p695_menubar_a2r_popover_tokens_and_dimmed 绿；plan593 投影完备集 +2）
- **T-05** [x] VM 开态 accent 高亮 + hover-switch（MouseArea on_enter 条件包裹）→ AC-04
  （证据：55e5c1b19；p695_menubar_hover_switch_and_open_highlight 绿；menubar-snapshot 规则 3 相容核对在案）
- **T-06** [x] VM submenu 双解释态（复合键注册表+前缀感知外层开态+内联展开裁定）→ AC-04
  （证据：bc35012ce + dd129995a；p695_menubar_submenu_radio_label_shortcut 绿；RightTop 变体保留；嵌套 overlay 挂渲染通道留债 P695-D2）
- **T-07** [x] VM radio/label/shortcut（circle-dot/muted 静态文本/group 透传）→ AC-04
  （证据：bc35012ce；单测同上绿）
- **T-08** [x] 键盘导航时间盒裁定：Esc 已有（Popover 既有捕获+menubar-snapshot 验证锚）；方向键/Enter 需 iced 焦点基建超时间盒 → 落债 P695-D1，AC-04 最小集满足
  （证据：c342a1717 提交注记；renderer Esc 订阅链在档）
- **T-09** [x] coverage menubar root→Covered + core.md 再生（docs_gen 4/4）+ corpus 零 menubar 语料确认 → AC-01/07
  （证据：c342a1717）
- **T-10** [x] [auto-os] menubar.at 重写（四菜单全族+状态回显+全族属性表+auto: 源码串）→ AC-05
  （证据：auto-os 8e098bc；docs_gen gallery_properties schema 符合门绿）
- **T-11** [x] 实机走查：VM 臂三修（untracked 臂补缺 dd129995a/复合键前缀/内联展开）+ 深色三截图（initial/file_open/view_open/submenu_inline 环境受限部分取得）+ EOL 端到端（auto-edit 编辑→行尾→CRLF 三段全真）；Vue 臂活体走查环境受阻（worktree 无 front workspace，unblock=宿主窗交互启动或 692 同款全量再生成流程）——SFC 发射面由单测+schema 符合门确定性覆盖 → AC-02/03/04/05 部分实证（D5/D6 在册）
- **T-12** [x] [auto-edit] ctx_menu/app.at token 修 + 行尾 submenu 化 → AC-06
  （证据：auto-edit d71a396）
- **T-13** [x] [auto-edit] 双轨重生成（regen_vue 33 组件零 S002 + vm 轨=解释态）+ probe_menu.py 两轮 exit 0 四时点全开 + EOL 活体三段；对照截图不可得（auto-edit 1s Tick 与截图通道争用，预存特征）→ AC-06（结构链承载）
- **T-14** [x] 滚动协议落册：候选池核对（§5.3 表即协议面，进入=点名+W5.N+revision+1）+ INDEX 性质核对（Stage-B 指针页，非活跃清单——695 非随迁计划无行）+ specs.json 预填核对（P-NNN-1 条目形态已核，落库随 merge）→ AC-08

依赖链：T-01→T-02→T-03/T-04~T-07（可并行）；T-08 依赖 T-06；T-09 收口 W1；T-10 依赖
T-03/T-07（语料用全族）；T-11 依赖 T-04~T-10；T-12 依赖 T-04/T-06/T-07；T-13 依赖 T-12。

## 复审记录

- 2026-09-23 起草（stage: new，PLAN-695 rev1）：三路探查（auto-lang 生成器/双臂、auto-edit
  消费现状、widgets-gallery 49 组件盘点）+ shadcn-vue menubar API 对勘完成；W1-W4 任务与
  SD-01..06 就位；开工前置门（692 先折）在案。outcome: pass（待用户确认后 /auto-plan:work）。
- 2026-09-23 执行完毕（stage: work，PLAN-695 rev1，outcome: pass → 待独立复审）：
  三仓 plan-695-dev（auto-lang base d59bd9fbb→349574529+tf 复跑批 / auto-os base
  20ce5c8→8e098bc / auto-edit base cf56891→d71a396）。T-01..T-14 全勾证据随任务行；
  新单测 9 枚全绿、schema_drift 2/2、docs_gen 4/4、registry/vue/VM 解释态/a2r 定向
  全绿。**执行期四增量**（均在案带证据）：①untracked 派发表缺 menubar 臂（画廊
  preview 实锤 Empty，baseline 预存漂移随之裁剪）；②复合键注册表外层开态须前缀
  感知（子键曾连带关外层）；③嵌套 Popover overlay 挂 iced 渲染/截图通道→VM
  submenu 改内联展开（浮动式留债 P695-D2）；④menubar_shortcut 补 text prop 声明
  （positional text 落 text prop，S001 消除）。fold 前门：cargo tf --no-fail-fast
  全量对勘（首轮 2 musk 预存 + gallery 重测并发内存挤兑中止——单跑 1289s 绿）+
  gallery_golden 重基线（理由随提交）。auto-edit 侧：worktree 二进制 env 覆盖序
  regen（PATH 解析落主检出旧 exe 的坑在案）；Vue 臂活体走查环境受阻（D6）。
  blockers: 无。next: /auto-plan:review。
- 2026-09-23 fold 门对勘补录（stage: work 续）：cargo tf --no-fail-fast
  全量 5465 测，红册 12→定谳 11 预存（musk×6/counter×1/a2vue 金样×1 =
  基线在档；style_if stress + native_gate 018 两枚 master 同测实锤预存）
  + 1 本计划（plan674 menubar_family_lowers_to_popover_row 断言随 a2r
  前缀感知发射更新，73de4c9ef）。执行期增量 ⑤：menubar_shortcut 补
  text prop 声明（S001 消除）；⑥desktop_protocol coverage 语义勘定——
  Covered=协议 target_set 可投影非本地实现完备，menubar root 回退
  NotYet（covered fence 实锤）。gallery_pages_compile tf 首轮栈溢出中止
  = 重测并发内存挤兑，单跑 1289s 绿非缺陷。gallery_golden 重基线随本门
  执行（P695-D4 消化）。
- 2026-09-23 work 终态（outcome: pass，status → execution_done）：
  gallery_golden 重基线落定（25cb53fa7，理由随提交：auto-os master 自
  614 基线点演进 + 本计划 menubar.at 语料定稿；复跑对账 1108s exit 0，
  P695-D4 销号）。全任务+验收映射收口，代码全提交，无阻塞问题。
  next: /auto-plan:review（worktree 保留）。

## 待澄清事项

- **Q-1 键盘导航深度**：T-08 默认带时间盒裁定（超成本落债 P695-D1），若用户要求全量键盘面
  为硬验收，请在开工前注明（影响 AC-04 口径）。
- **Q-2 开工时序**：PLAN-692 在途（execution_done 待 review），其 schema/registry/vue.rs
  改动与 W1 同文件——默认等 692 折叠后开工；若要求并行需接受 rebase 成本。
- **Q-3 auto-edit 他方 WIP**：auto-edit 主检出有未跟踪 plan010 草稿与 stylekit 删除态（他方
  会话）；W4 已定为独立 worktree 隔离，不碰主检出——若该会话仍在写 auto-edit，T-13 重生成
  前需与其协调空窗。

## 待澄清处置（2026-09-23 执行期）

- Q-1 键盘导航深度：按预案时间盒裁定——Esc 既有满足最小集，方向键/Enter 落债
  P695-D1（AC-04 口径不受阻）。
- Q-2 692 时序：692 已折（archived/cleaned），未发生 rebase 冲突。
- Q-3 auto-edit 他方 WIP：主检出 WIP 未被触碰；本计划全在 worktree（cf56891 基线）。
