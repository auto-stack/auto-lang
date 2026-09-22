---
plan_id: PLAN-688
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: sidebar-shell-collapse-responsive
author: [agent]
created_at: 2026-09-22
updated_at: 2026-09-22
plan_revision: 1
current_step: 0
total_steps: 7

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [blueprint]          # docs/specs/blueprint/{contract,project}.md
---

# [PLAN-688] sidebar-shell BP 交互化：三段式 + 运行时收缩 + 平板竖屏自动收缩

> **用户裁定（2026-09-22，需求走查会话，截图两张为验收锚）**：
> ①三段式（上=标题icon+收缩钮 / 中=nav 列表 hover+选中高亮 / 下=settings+附加 nav 项）、
> ②运行时收缩切换（宽栏 ↔ icon rail，收缩态 hover 出 tooltip）、
> ③平板竖屏自动收缩（**768x1024 口径**）——**单计划装下**（用户原话"现在可以用一个计划就装下了"）。
> ④mobile ☰ 悬浮菜单（窗口上方标题栏/下方 tab 栏 ☰ → 弹出悬浮菜单）**只记录不实现**：
> 落 `blueprints/navigation/sidebar-shell/spec.md` Non-goals + `gotchas.md`，待 mobile milestone 立项时
> 显式翻案 `docs/design/autoui/sidebar-family-and-nav-retirement.md` §2.2 的"mobile 机制整批不做"非目标。
> ⑤**RQ 轨未实现完毕、非缺省轨**（用户勘正 2026-09-22）——VM 轨优先验收，本计划零 RQ 面改动。

## 0. 变更摘要

`navigation/sidebar-shell` BP 从**静态双变体参考**（default 宽栏 / compact icon-rail 两个独立 widget，
变体在装配期选定）升级为**单一受态驱动的交互 widget**：

1. sidebar 内部改三段式：上段（标题 icon + 收缩钮）/ 中段（`nav_items` EDIT 区，hover 高亮 +
   选中高亮）/ 下段（`footer` EDIT 区：settings 按钮 + 附加 nav 项）。
2. 新增 bp-private `collapsed` 态 + `ToggleCollapsed` 消息：点击收缩钮 → icon rail（w-14，仅 icon，
   `title:` tooltip）；点击标题 icon → 回宽栏。宽度/标签可见性全部走
   **`style: if` 条件样式 × `lg:` 响应式类组合**（§5 D1），零新增 VM 原语。
3. 平板竖屏自动收缩：窗口宽 < 1024（`lg` 断点以下，min-width 语义）强制 rail 形态——
   768x1024 竖屏首渲染即 rail；拉宽 ≥1024 自动展宽（Plan 527 T7 的 resize→view 重建→重解析回路）。
4. 文档面：spec.md（acceptance 增补 + Non-goals mobile 记录）+ gotchas.md（三条新条目）；
   `bps-gallery/src/front/registry.at` 按 SD-01/PLAN-676 再生；plan649/640/657 测试锚同步。

**不做**：mobile ☰/悬浮菜单/isMobile（记录不实现）；竖屏下手动展开覆盖（需窗口信号原语，随
mobile milestone 评估）；收缩宽度动画（VM 轨无动画通道）；RQ 轨适配；`sidebar_*` widget 族与
`sidebar-nav` BP 的改动（本计划只动 sidebar-shell 包）。

## 1. 目标

### Goals

- **G1（三段式）**：裸实例化 sidebar-shell 时 sidebar 呈上中下三段：上段标题 icon + 收缩钮；
  中段 nav 列表（每行 icon + label + 可选 badge）；下段 settings + 附加项。VM 轨截图可验。
- **G2（选中/hover 高亮）**：nav 行选中态 `variant: if .active_nav == node.id` 切换（sidebar-nav
  先例同款）；hover 高亮经 `hover:` 变体类（Plan 527 T6，按钮族 VM 真消费）。双端一致。
- **G3（运行时收缩）**：`.collapsed` bool + `ToggleCollapsed`（`=!` 切换，form/login
  ToggleRemember 先例）。宽栏点收缩钮 → rail；rail 态点标题 icon → 宽栏。VM 轨两态截图可验。
- **G4（竖屏自动收缩）**：窗口宽 <1024 时 rail 形态不依赖任何手动态（纯断点类门控）；
  768x1024 实测首渲染即 rail；≥1024 恢复宽栏（或 `.collapsed` 态），resize 实时生效。
- **G5（rail 态 tooltip）**：rail 的 icon 按钮 hover 显示 label tooltip。VM 轨现状勘定后
  或"已接线仅记录"或"补 `title:`→iced tooltip 臂"（T-01(c) 定性，T-04 兜底）。
- **G6（文档与同步）**：mobile ☰ 需求落 spec Non-goals + gotchas（翻案留待声明）；
  registry.at 再生；测试锚全绿。

### 非目标

- mobile ☰ / Sheet 悬浮菜单 / isMobile 检测（VM 原语 PLAN-533/534 已在，纯接线——留给 mobile milestone）。
- 竖屏下手动展开覆盖 portrait 强制 rail（需要 `.at` 可读窗口尺寸的原语；v1 竖屏恒 rail，
  收缩钮在竖屏下视觉无操作——gotchas 记录该取舍）。
- 收缩/展开宽度过渡动画。
- RQ 轨（用户裁定：未完成不碰；新增视觉面在 RQ 落地时须过 DisplayList 消费者全扫描——683 在途，此处仅留注记）。
- compact 变体退役（保留静态纯 rail 参考，向后兼容；registry 双变体同页不互踩有 SandwichShellFull 先例）。

### 约束与依赖

- 单仓（auto-lang）；实现走 worktree `D:/autostack/.wt/lang-688/auto-lang`（branch `plan-688-dev`），
  master 只落计划簿记（AGENTS.md L1 流程）。
- widget 签名保持不变：`SidebarShell(nav_tree, user_name, user_email, on_nav, on_sign_out)`
  ——plan649 t09 直连导入 props 串零改动。
- nav 节点 schema 扩展为**全键** `{id, label, badge, icon}`（icon 可空串；VM 缺键访问=硬错，
  catalog.at 纪律）——t09 的 `.nav` fixture 须同步补 `icon` 键（T-07）。

### 成功样态

bps-gallery 双臂打开 sidebar-shell 页：宽栏三段式 → 点收缩钮变 rail（hover 出 tooltip）→
点标题 icon 回宽栏；把窗口拉到 768 宽 → 自动 rail；拉回 ≥1024 → 自动展宽。Vue 臂同页行为一致。

## 2. 架构方案

### D1：单树 class 组合（核心裁定）——不用双树可见性，不加窗口信号原语

nav 行常驻渲染 icon + label 两元素，"label 面"可见性组合手动态与断点：

```
label/标题文字/chevron/badge（label 面）:
  style: if .collapsed { "hidden" } else { "hidden lg:flex" }
sidebar 列宽:
  style: if .collapsed { "w-14 items-center" } else { "w-14 items-center lg:w-56" }
icon（icon 面）: 常驻
```

语义 = `可见 ∧(¬collapsed ∧ width≥1024)`，一个 `if` 嵌套即完成"手动收缩 × 朝向"的 AND 组合，
无需在 `.at` 里读窗口尺寸。依赖三件已落地基建（起草时已实勘）：

1. Style 变体/响应式管道：`hover:`/`lg:` 等前缀解析期门控，`theme::window_width` 回填，
   resize→view 重建→重解析实时生效（Plan 527 T6/T7，`crates/auto-lang/src/ui/style/mod.rs:20,188-195`；
   断点阈值 sm 640 / md 768 / **lg 1024** / xl 1280 / 2xl 1536）。
2. `hidden` + display 变体：响应式 display 覆盖 Hidden 正是设计用途（Plan 409 §10 续，
   `crates/auto-lang/src/ui/style/class.rs:1242,1270`）。
3. `style: if` 条件样式：gallery-shell accent 切换先例（`blueprints/layout/gallery-shell/reference/default.at:99-115`）。

**已知取舍**：竖屏（<1024）下收缩钮视觉无操作（rail 恒定）；`lg:flex` 与 `hidden` 同进 base 的
冲突消解按 class.rs display 变体设计（响应式覆盖 Hidden）。**T-01 探针验证三前提，任一破即回退
双树方案（T-03）**：宽/rail 两棵子树都渲染，可见性 `hidden lg:flex` / `flex lg:hidden` 互斥。

### D2：三段式结构（reference/default.at 重写）

```
col (w-full h-screen)
  row (flex-1 min-h-0)
    sidebar                                    ← 三段式
      col (style: 条件宽度，见 D1)
        // 上段：标题 icon + 收缩钮
        row: button(title_icon)[ToggleCollapsed] + label 面(标题文字) + button(chevron, label 面)[ToggleCollapsed]
        // 中段：nav_items EDIT 区（可滚动）
        for node in .nav_tree:
          row (key: node.id):
            button f"${node.icon}" (w-9 h-9, title: node.label)[.NavSelected(node.id)]
            button f"${node.label}" (flex-1 justify-start, label 面, variant: if .active_nav == node.id {"secondary"} else {"ghost"}, hover: 类)
            if node.badge != "": badge (label 面)
        // 下段：footer EDIT 区
        separator + button(⚙ settings, icon+label 同 D1 规则) + 附加项示例
    col (flex-1 min-h-0)
      header (h-12 border-b): brand "Acme" + user dropdown-menu（维持现状，迁入 content 侧）
      slot: content EDIT 区（fallback 占位文案保留——t09 needle 兼容）
```

- widget 签名/msg 面增量：`msg` 增 `ToggleCollapsed`；`model` 增 `var collapsed bool = false`。
  `NavSelected/MenuSignOut` 语义不变。
- 结构性变更：原全宽 header 迁入 content 列（对齐截图形态：sidebar 满高三段，头部属内容侧）。
  needle 兼容："Acme"/"Home"/"App content mounts here" 全保留。
- nav 节点全键 `{id,label,badge,icon}`；`icon` 为文本字形（compact.at `⌂▦⚙` 先例），空串时
  icon 钮显示 label 首字符（f 表达式切片或消费方烘焙——T-02 实现时择一，倾向消费方烘焙，
  bp 不做切片逻辑）。

### D3：tooltip（G5）

rail 态 icon 按钮带 `title: node.label`。VM 轨现状：素按钮 `title:`→tooltip 疑似未接线
（`render_support.rs:201` sidebar_menu_button tooltip 明确 deferred=P548-D2；EE03 tooltip 只覆盖
合成工具栏钮，`iced/renderer.rs:4479`；`iced::widget::tooltip` 已在 renderer.rs:15 导入）。
T-01(c) 勘定：已接线→仅记录；缺失→T-04 在按钮转换臂加 tooltip 包裹（EE03 先例，小改）。

### D4：不变式

- `sidebar` tag 走 VM 契约 Column 子集（Plan 561），本计划不改词汇层。
- compact 变体（`SidebarShellCompact`）零改动。
- Vue 臂同源免费：reference .at 经 codegen 生成，`lg:`/`hidden`/`hover:` 是 Tailwind 原生；
  `style: if` 条件样式 vue 臂有 gallery-shell 在库先例。

## 3. 技术栈

- AutoLang `.at`（BP 资产层：reference/spec/gotchas）——主体改动
- Rust（仅 T-04 兜底臂：`crates/auto-lang/src/ui/aura_view_builder.rs` 按钮转换 + 可能的
  `iced/renderer.rs` tooltip 消费；`cargo check -p auto-lang` + `cargo t iced` 门禁）
- `auto bp list --format at`（registry.at 再生）
- 测试：`plan640_bp_tests`（扫描表）/ `plan649_bp_tests`（t09 直连导入）/ `plan657_bp_admin_tests`
  （admin 路径表）+ autoui-verifier 双臂走查

## 4. 需求分析与背景调查

### 授权记录（2026-09-22 用户裁定）

- 范围：§0 四条裁定（①②③实现 + ④记录）；单计划；VM 轨优先；RQ 零接触。
- 仓：auto-lang 单仓。预算/自动续跑：未特别限定（按 AGENTS.md 标准流程）。

### 实勘记录（起草时，master ba7477aa8 工作树）

| # | 事实 | 出处 |
|---|---|---|
| 1 | sidebar-shell 现状 = 静态双 widget（default 参数化 / compact 纯静态），变体装配期选定 | `blueprints/navigation/sidebar-shell/reference/{default,compact}.at`、`spec.md` |
| 2 | 断点阈值 sm 640/md 768/lg 1024/xl 1280/2xl 1536，min-width 语义；768 竖屏命中 md 不命中 lg | `crates/auto-lang/src/ui/style/mod.rs:188-195,542-552` |
| 3 | 响应式类解析期门控 + resize→view 重建→重解析（实时生效）；`hover:` 按钮族 VM 真消费 | Plan 527 T6/T7，`style/mod.rs:20,205-247` |
| 4 | `hidden` + 响应式 display 覆盖为设计用途 | Plan 409 §10 续，`style/class.rs:1242,1270` |
| 5 | `style: if` 条件样式在库先例 | `blueprints/layout/gallery-shell/reference/default.at:99-115` |
| 6 | bool 切换 `=. !.x` 在库先例 | `blueprints/form/login/reference/minimal.at:73`（ToggleRemember） |
| 7 | 素按钮 `title:`→tooltip VM 疑似未接线（P548-D2 同族；EE03 仅合成工具栏钮） | `render_support.rs:201`、`iced/renderer.rs:4479`——T-01(c) 定性 |
| 8 | 悬浮层 VM 真实现（dropdown-menu/sheet/drawer/popover，PLAN-533/534）——本计划不用，记录给 mobile milestone | `ui/aura_view_builder.rs:1771,1786-1810` |
| 9 | bps-gallery registry.at = `auto bp list --format at` 生成产物，BP 改动须再生（CI diff 校验） | `examples/bps-gallery/README.md`（PLAN-676 SD-01） |
| 10 | 测试锚点：t09 直连导入 needle `Home/Acme/App content mounts here`；props 串 `(nav_tree: .nav, user_name: "u", user_email: "e", on_nav: .NavSel, on_sign_out: .Primary)`；扫描表 13 包 | `plan649_bp_tests.rs:318-360`、`plan640_bp_tests.rs:26,42-43`、`plan657_bp_admin_tests.rs:145,176` |
| 11 | t09 的 `.nav` fixture 节点现无 `icon` 键——全键纪律下 T-02 改 reference 读 `node.icon` 前必须补 | `plan649_bp_tests.rs` e2e_host 播种 |

### 规格现状

`docs/specs/blueprint/{contract.md,project.md}` 为 BP 层 spec。交互态 BP（bp-private 态承载
交互行为 + 断点组合语法）目前无登记——SD-01 增补；sidebar-shell 包条目注记——SD-02。

## 5. 详细设计

（D1-D4 见 §2；此处补执行级细节。T-01 探针结论须回写本节。）

### T-01 探针设计（bounded investigation，决策工件）

临时 example（`examples/capability-tests/` 惯例位或 `cargo t iced` 单测）验证三前提：

- **(a) 组合语法**：`style: if .collapsed {"hidden"} else {"hidden lg:flex"}` 在 768 宽窗口
  label 不可见、1280 宽可见；`w-14 items-center lg:w-56` 同窗宽切换生效。
  预期：行为符合 D1 语义。
- **(b) resize 实时性**：窗口 768→1280 拉伸，不重启进程，label/宽度随断点翻转。
  预期：Plan 527 T7 回路真实生效（若此处破，D1 整体不成立，走 T-03 双树回退）。
- **(c) title tooltip 现状**：`button "⌂" { title: "Home" }` 在 VM 轨 hover 是否出 tooltip。
  预期输出：二值定性（已接线/未接线）写回 §2 D3 与本节。

### reference/default.at 骨架（T-02 实现 baseline）

见 §2 D2。补充约定：

- 上段标题 icon 复用 brand 首字形（reference 用 "▲" 类中性字形 + "Acme" 文字，label 面）；
  收缩钮 chevron 用 "◂/▸" 文本字形（lucide 面不进本计划——词汇层零改动）。
- 中段滚动：nav 列表容器 `overflow-y-auto`（Plan 656 scroll 语义）。
- 下段 settings 行 = icon 钮 + label 钮同 D1 规则；附加项示例 1 条（"选择工作目录"形态，
  纯 reference 样例无业务语义）。
- 全部 EDIT 区注释保持 `// EDIT: <point>` 惯例（logo/nav_items/footer/content/header_actions）。

### spec.md 变更（T-05）

- `acceptance` 增三条：collapse toggle 双向 / 竖屏(<1024)自动 rail / rail tooltip。
- `props` 注记 nav 节点全键 schema（+`icon`）。
- 新 `# Non-goals` 节：mobile ☰ 需求描述（截图语义：窗口上标题栏/下 tab 栏 ☰ → 悬浮菜单切换栏目）
  + "待 mobile milestone 翻案 sidebar-family-and-nav-retirement §2.2"声明 + 竖屏手动展开覆盖
  同批评估注记。
- gotchas.md 新三条：①竖屏强制 rail 取舍与收缩钮无操作语义；②`style: if`×响应式类组合语法
  （label 面/icon 面规则，可复制到其他 BP）；③mobile ☰ 记录指针（悬浮层原语 PLAN-533/534 已在，
  缺的是 isMobile 信号与翻案裁定）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/ 目标 | before/after 规则 | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/blueprint/contract.md | before：BP 参考实现皆静态、交互变体装配期选定；after：登记"交互态 BP"惯例——bp-private model var + toggle msg + `style: if`×响应式类组合（label 面/icon 面规则），sidebar-shell 为锚例 | 首个交互态 BP，组合语法可复用 | AC-03, AC-04 |
| SD-02 | modify | docs/specs/blueprint/project.md | sidebar-shell 条目注记交互化（三段式/收缩/竖屏）+ mobile ☰ Non-goals 指针 | 消费方发现面 | AC-06 |

## 6. 测试设计

| 层 | 内容 | 命令 | 预期 |
|---|---|---|---|
| 探针 | T-01 三前提（组合语法/resize 实时/title 现状） | 临时 example + `auto run -r vm` 实测，或 `cargo t iced` 单测 | §5 T-01 预期 |
| BP 结构 | reference 解析 + 消息/态面（ToggleCollapsed/collapsed/三段 EDIT 注释在位） | `cargo t plan649`（或对应 bp 测试滤串） | 绿 |
| 直连导入 | t09 needle 兼容（`Home/Acme/App content mounts here`）+ `.nav` fixture 补 `icon` 键 | `cargo t plan649 t09`（滤串） | 绿 |
| 注册表 | 13 包扫描/palette 零漂移/admin 路径表 | `cargo t plan640`、`cargo t plan657` | 绿 |
| 门禁 | 若 T-04 触 Rust：类型检查 + 局部模块 | `cargo check -p auto-lang` + `cargo t iced` | 零 error、零新 warning、绿 |
| 实机 | bps-gallery sidebar-shell 页三态走查（宽/rail/768 竖屏）+ Vue 臂同页 | `cd examples/bps-gallery && auto run -r vm` / `auto run`（autoui-verifier 截图流程） | §1 G1-G4 样态 |

门禁映射（AGENTS.md）：改动主体为资产（Category A 不跑 cargo t）；T-04 触 Rust 升 Category B
（`cargo check -p auto-lang` + `cargo t iced`）。零 aavm 触发面（不跑 taa）；无 docs_gen/schema
改动（不跑 docs_gen 档）。

## 7. 验收标准

| ID | 可观察行为 | 验证方法 |
|---|---|---|
| AC-01 | 裸实例化 sidebar-shell：sidebar 三段式（上=标题icon+收缩钮 / 中=nav icon+label+badge / 下=settings+附加项），无应用特定文案 | bps-gallery VM 轨截图（`auto run -r vm`） |
| AC-02 | nav 行 hover 高亮（`hover:` 类生效）+ 选中高亮（`variant: if .active_nav` 切换） | 双臂各一张 hover/选中态截图 |
| AC-03 | 宽栏点收缩钮 → rail（w-14 仅 icon）；rail 点标题 icon → 回宽栏；`.collapsed` 态真实驱动 | VM 轨两态截图 + 切换前后 nav 选中态保持 |
| AC-04 | 窗口宽 <1024（含 768x1024 竖屏）首渲染即 rail；拉宽 ≥1024 自动展宽（不重启）；label 面随断点翻转 | VM 轨 768/1280 两窗宽截图 + resize 录程叙述 |
| AC-05 | rail 态 icon 按钮 hover 显示 label tooltip（VM 轨真显示；若 T-01(c) 勘定已接线则勘定记录即满足） | VM 轨 hover 截图或勘定记录 |
| AC-06 | spec.md Non-goals 记录 mobile ☰ 需求与翻案留待声明；gotchas 三条新条目在位；compact 变体零回归 | 文件 diff + `cargo t plan640`（扫描含 compact） |
| AC-07 | registry.at 再生落盘（`auto bp list --format at` diff 为空）；plan640/649/657 全绿；若触 Rust：`cargo check -p auto-lang` 零 error + `cargo t iced` 绿 | 命令输出 |
| AC-08 | Vue 臂 bps-gallery 同页断点行为一致（768 rail / ≥1024 宽栏），无双轨漂移 | `auto run` 走查截图 |

## 8. 执行步骤

> 实现在 worktree `D:/autostack/.wt/lang-688/auto-lang`（branch `plan-688-dev`）；
> 计划簿记（[✅] 标记/frontmatter 翻转）留 master 默认检出。

- **T-01 机制勘定探针**（bounded investigation → 决策工件写回 §5）
  - 文件：临时 example（`examples/capability-tests/` 或独立探针目录，合并前可弃）或
    `crates/auto-lang/src/tests/` 单测
  - 操作：(a) 组合语法双窗宽验证；(b) resize 实时性；(c) title tooltip 现状定性
  - 产出：三前提结论写回 §5 D1/D3；D1 破则 T-03 转执行
  - 验证：`auto run -r vm` 实测或 `cargo t iced`；预期见 §5
  - 关联：AC-04/AC-05 前置
- **T-02 reference/default.at 三段式重写**（依赖 T-01(a) 通过）
  - 文件：`blueprints/navigation/sidebar-shell/reference/default.at`
  - 操作：§2 D2 骨架落地——三段式/`collapsed`+`ToggleCollapsed`/D1 组合类/nav 全键 icon/
    header 迁 content 侧/needle 三文本保留/EDIT 注释惯例
  - 验证：`auto run -r vm` 裸实例化走查（宽/rail 两态）
  - 关联：AC-01/02/03
- **T-03 双树回退**（**条件任务**：仅 T-01(a)/(b) 判 D1 不可行时执行；否则标注
  skipped+原因，不计入完成度缺口）
  - 文件：同 T-02
  - 操作：宽/rail 两棵子树 + `hidden lg:flex`/`flex lg:hidden` 互斥可见性
  - 关联：AC-03/AC-04
- **T-04 VM 轨 button title→tooltip 臂**（**条件任务**：仅 T-01(c) 判未接线时执行）
  - 文件：`crates/auto-lang/src/ui/aura_view_builder.rs`（按钮转换臂读 `title` →
    tooltip 包裹；EE03 先例 `iced/renderer.rs:4479`，`iced::widget::tooltip` 已导入）
  - 验证：`cargo check -p auto-lang` 零 error；`cargo t iced` 绿；探针 (c) 复跑出 tooltip
  - 关联：AC-05；债册 P548-D2 同族清偿注记（T-07 落账）
- **T-05 spec.md + gotchas.md**（依赖 T-02 定稿）
  - 文件：`blueprints/navigation/sidebar-shell/spec.md`、`gotchas.md`
  - 操作：§5 spec 变更三条 + Non-goals mobile 记录 + gotchas 三条
  - 关联：AC-06
- **T-06 registry.at 再生 + 双臂走查**（依赖 T-02/T-05）
  - 文件：`examples/bps-gallery/src/front/registry.at`
  - 操作：`auto bp list --format at > examples/bps-gallery/src/front/registry.at`；
    bps-gallery VM 轨三态走查（宽/rail/768）+ Vue 臂 `auto run` 走查（autoui-verifier 截图）
  - 关联：AC-01/02/03/04/08
- **T-07 测试收口 + 门禁 + 落账**（依赖 T-02..T-06 全部定稿）
  - 文件：`crates/auto-lang/src/plan649_bp_tests.rs`（`.nav` fixture 补 `icon` 键 + needle 复核）、
    `docs/plans/KNOWN-DEBT-AND-RISKS.md`（P548-D2 处置注记，若 T-04 执行）
  - 操作与验证：`cargo t plan640` + `cargo t plan649` + `cargo t plan657` 绿；
    若触 Rust：`cargo check -p auto-lang` + `cargo t iced` 绿；扫描零漂移
  - 关联：AC-07

（每步完成后在对应任务下追加 `[✅ 已完成] <证据>` 一行。）

## 9. 复审记录

### draft handoff（2026-09-22）

- stage: new，PLAN-688 revision 1。
- outcome: pass——任务面覆盖全部 AC 与 SD；路径/命令均实勘在案（§4 表）；无阻塞待澄清。
- 关键裁定已锁：D1 单树组合（T-01 可证伪，T-03 回退在案）；mobile ☰ 记录不实现（用户裁定）；
  VM 轨优先（用户勘正 RQ 非缺省）。
- next: work（用户确认本契约后建 worktree `lang-688` 执行）。

## 10. 待澄清事项

无阻塞项。随行注记（不阻塞执行）：

1. 竖屏下手动展开覆盖（需 `.at` 窗口尺寸原语）——随 mobile milestone 与 ☰ 同批评估；
   届时若立窗口信号原语，D1 组合语法可整体简化。
2. compact 变体长期去留（交互化后 default 已覆盖 rail 形态；是否有消费方依赖 standalone
   compact 待普查）——退役走独立评审，不在本计划。
