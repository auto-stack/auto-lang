---
plan_id: PLAN-619
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 015-notes-vm-vue-parity
author: [zhaopuming]
created_at: 2026-09-12
updated_at: 2026-09-12
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [auto-lang/ui, auto-man]
current_step: 0
total_steps: 5
---

# [PLAN-619] 015-notes-vm-vue-parity

## 0. 变更摘要

PLAN-616 交付 015-notes 清爽化重做后，**同一份 `.at` 源码在 Vue 与 VM(iced) 两端肉眼可见地不一致**。
本计划把这些差异**量化、定位到引擎侧根因并修掉**，让「双端一致」从口号变成有像素预算的门禁。

用户实测指出的 4 项（均已在 `docs/plans/attachments/616/` 的前后对比截图上复核）：

| # | 现象 | 量化证据（逻辑坐标，`tools/parity_shot_diff.py` 可复跑） |
|---|---|---|
| P1 | 所有面板/表面背景色两端不同 | 页面/编辑区 vue `(9,14,26)` vs vm `(20,26,41)`；侧栏 vue `(14,21,37)` vs vm `(26,34,53)` |
| P2 | 图标明显偏小、未行内居中 | 同一 glyph 同一窗口 ink：vue `17.5×19.5` vs vm `8.2×9.8`（≈47%/50%） |
| P3 | 搜索行左右内边距缺失（VM） | 行左边界 12 → 图标 ink 起始：vue `24` vs vm `16.5`（少 ≈8px） |
| P4 | 侧栏列表左侧内边距两端不同 | 分组标签左起 vue `16` vs vm `8`；笔记行文本左起 vue `24` vs vm `16.5`（少 8px） |

另：**纵向节奏两端一致（地标带 68/112 vs 69/113，差 1px）**——初判的「VM 行更紧」不成立，不列入本计划。

## 1. 目标

- G1：语义 token 在**两端解析为同一色值**（P1）——消除「同一 token 两套色板」。
- G2：`icon` 元素的尺寸在两端**按 `size` 生效**且**光学居中**（P2）。
- G3：容器（row/col 包装件）的 `px-*`/`py-*` 在两端**同等生效**（P3/P4）。
- G4：把「双端一致性」变成**可复跑的门禁**：`tools/parity_shot_diff.py` + 采样点/像素预算写进 acceptance。

**非目标**：不重做 015-notes 的视觉设计（PLAN-616 已定稿）；不放宽 acceptance.atd 既有验收项；
不改后端/数据模型。

## 2. 架构方案

三个根因分别落在不同层，修法不同：

| 现象 | 根因层 | 处置方向 |
|---|---|---|
| P1 色板 | **VM 语义解析单源** vs **Vue index.css 生成源** | 二选一并让另一端跟随：①（推荐）VM 默认活动主题与「scaffold 类应用」的 Vue CSS 同源；②或 app 用 `theme{}` 显式钉住（PLAN-601 已双端消费）。需先定「谁是权威」——见 §10 待澄清① |
| P2 图标 | **`size` 未贯通**（Vue 走 lucide 默认 24；VM 实际 ≈12） | Vue 生成器补 `:size`；VM 侧查清 12px 来源（`icon` 臂设了 Width/Height=size，实测却 ≈12 → 疑落 PUA/字号路径）并使其按 size 绘制 + 盒内居中 |
| P3/P4 内边距 | **容器 padding 解析**（`IcedStyle` 的 per-axis 与 uniform 覆盖次序 / sidebar 契约类的合并） | 先做**有界调查**（T-01）定位到具体函数再改，避免盲改 |

## 3. 技术栈

Auto 语言 `.at`（示例侧，仅作验证载体）；Rust 引擎侧：`crates/auto-lang/src/ui/style/{class.rs,iced_adapter.rs}`、
`crates/auto-lang/src/ui/iced/renderer.rs`、`crates/auto-lang/src/ui/aura_view_builder.rs`、
`crates/auto-lang/src/ui_gen/{vue.rs,sidebar_contract.rs}`；生成侧：`crates/auto-man/src/`（index.css 模板/scaffold）。
验证：`tools/parity_shot_diff.py`（本计划新增，git 跟踪）+ 双端截图 + `cargo tv`/`cargo t`。

## 4. 需求分析与背景调查

### 4.1 证据链（可复跑）

```bash
python tools/parity_shot_diff.py docs/plans/attachments/616/after_vue_initial.png \
                                 docs/plans/attachments/616/after_vm_initial.png
# → 语义面色差 + 地标带 + 图标 ink 三组数字（P1/P2/P4 的机器可读版）
```

截图：`docs/plans/attachments/616/{before,after}_{vue,vm}_initial.png`（PLAN-616 归档，git 跟踪）。

### 4.2 根因定位（已核实部分）

**P1（色板）——已定案**：
- VM：`crates/auto-lang/src/ui/style/theme/mod.rs` 的 `resolve_semantic_rgb()` → `active_theme().resolve(token, is_dark)`
  （注释明确「PLAN-593 V2：静态语义色查 registry（stella 单源）」）；
  `crates/auto-lang/src/design_tokens/registry.rs` 的 `STELLA.dark` 表：`Background = Rgb(20,26,41)`、
  `Card = Rgb(26,34,53)` —— **与 VM 截图像素完全相等**。
- Vue：`gen/front/vue/src/assets/index.css` 的 `:root/.dark`（auto-man scaffold 模板）：
  `--background: 222.2 47% 7%` = `(9,14,26)`、`--card: 222.2 47% 10%` = `(14,21,37)` —— **与 Vue 截图完全相等**。
- 结论：**同一语义 token 在两端来自不同色板**（stella vs scaffold/shadcn），非渲染误差。

**P2（图标）——半定案**：
- `aura_view_builder.rs`：`icon` 臂 `size` 默认 16，并注入 `Width(Pixels(size))`/`Height(Pixels(size))` 类；
- `ui/iced/renderer.rs`（`lucide:` 臂）：svg widget 宽高取 `IcedStyle.width/height`，**缺省 16**；
- `ui_gen/vue.rs:8431`：`icon` 追加 `w-5 h-5`（20px），但 `size` 未出现在发射里 → Vue 实际按 lucide 默认 **24** 渲染
  （实测 ink 17.5×19.5 ≈ 24 盒的 73%/81%）；
- VM 实测 ink 8.2×9.8 ≈ **12px 盒**，既不是请求的 18 也不是缺省 16 → 疑走「按钮内嵌 PUA/字号」路径或 size 未达 svg 臂，**需 T-02 钉死**。

**P3/P4（内边距）——待定位**：
- `renderer.rs:1824` `apply_row_style(..., padding, style, ...)` → `iced_padding(padding, style)`（`:1168`）
  → `iced_padding_from_is()`（`:1145`）：**uniform `padding` 命中即 early-return，per-axis 覆盖被丢弃**；
  该 early-return 与「契约类 `p-2` + 用户 `px-2 pt-2` 合并」组合是 P3/P4 的头号嫌疑；
- 侧栏族在 VM 端由 `ui_gen/sidebar_contract.rs` 的契约常量（如
  `CONTENT_BASE = flex min-h-0 flex-1 flex-col gap-2 overflow-auto`）+ 用户类合并生成；
  `015-notes/src/front/sidebar.at` 的 `sidebar_content (style: "px-2 pb-2")`、搜索行 `px-2.5`
  是实测少掉的两处 → **T-01 有界调查**给出确切函数与覆盖次序。

### 4.3 影响面（为何值得单独立项）

P1 影响**所有** gallery/示例应用（任何未声明 `theme{}` 的 scaffold 类应用在 VM 上都会偏 stella），
是 GOAL-007「跨端视觉一致」的正面违例；P2/P3 影响所有用 `icon` 与容器 padding 的页面。
属 **L2 级引擎改动**，需独立计划承载门禁与回归对拍。

### 4.4 授权与约束

- 用户授权（2026-09-12）：指出两图差异（面板底色 / search bar 左右 padding / Notes 列表左 padding /
  图标尺寸与居中）并指示「继续洗」→ 即继续收敛两端差异。
- 约束：不引入新的视觉设计变更；不得以「已知差异」名义放宽既有 parity 契约；
  P1 的「谁是权威色板」若涉及跨示例统一，须在 §10 记录裁决依据。

## 5. 详细设计

### 5.1 分任务设计要点

- **T-01（有界调查，产出决策件）**：定位 P3/P4 的确切丢失点。方法：最小 `.at` 夹具
  （row/col + `p-2`⊕`px-2`、`bg-card` + `px-2.5`）+ VM dump 或单元级断言 `iced_padding_from_is`，
  给出「uniform 覆盖 per-axis」还是「契约类合并丢类」的定论与覆盖次序规则。
- **T-02**：P2 尺寸贯通。确定 VM 端 12px 来源（走查 `icon` 臂 → svg 臂 / PUA 路径），
  使 VM 按 `size` 绘制并**盒内居中**；Vue 生成器为 `icon` 发射 `:size`。
- **T-03**：P1 色板单源。按 §10① 裁决落地（VM 默认主题对齐 scaffold 类应用 / 或示例侧 `theme{}` 钉住 +
  规范写明适用范围），并补**跨端色值对拍断言**。
- **T-04**：把 `tools/parity_shot_diff.py` 纳入门禁（登记到 `autoui-verifier` 技能的双端检查清单），
  为 015-notes 加「两端语义面/图标 ink 在预算内」的 acceptance 条目。
- **T-05**：015-notes 实机复验 + 归档新截图（补充/替换 attachments 基线）。

### 5.2 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-lang/ui/overview.md`（主题/token 节） | 写明「语义 token 的**权威色板**及其两端适用范围」；退役「VM 单源 stella 静态色」在 scaffold 类应用上的隐式适用 | P1 的定案必须成文，否则新示例继续踩 | AC-01 |
| SD-02 | add | `docs/specs/auto-lang/ui/overview.md`（已知坑） | 记录「容器 per-axis padding 与 uniform padding 的覆盖规则」「`icon` 尺寸权威来源」两条 | P2/P3 的语言层规则 | AC-02, AC-03 |

## 6. 测试设计

- 单元：`iced_adapter`/`class.rs` 的 padding 与 icon 尺寸解析用例（覆盖 `p-2`+`px-2`、`size` 缺省/显式）。
- 对拍：`tools/parity_shot_diff.py` 采集两端语义面色值 + 图标 ink；**预算**：语义面色值**逐通道 ≤2**
  （同一 token 同值），图标 ink 长宽比 **≥0.9**（VM/Vue），列表/搜索行左内边距差 **≤1px**。
- 端到端：`cargo tv`（语料 golden 不回归）+ `cargo t`；015-notes 19 条 MCP 场景与 18 条 Vue 场景**保持全绿**。
- 回归锚：把上述预算写成 015-notes acceptance 的 T14（两端对拍）条目。

## 7. 验收标准

- **AC-01**：015-notes 双端截图中，页面/编辑区/侧栏三处语义面色**逐通道差 ≤2**（当前 P1 差 (11,12,15)）。
- **AC-02**：同一 `icon (size: N)` 在两端 ink 尺寸比 ≥0.9，且图标盒在行内**光学居中**（目验 + ink 中心与文本对齐）。
- **AC-03**：搜索行左内边距与侧栏列表左缩进两端差 **≤1px**（当前差 8px）。
- **AC-04**：`tools/parity_shot_diff.py` 入库并在 acceptance / `autoui-verifier` 清单中登记为常驻检查。
- **AC-05**：`cargo tv` + `cargo t` 零新增红；015-notes 19 MCP 场景 + 18 Vue 场景保持全绿。

## 8. 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 `[✅ 已完成]` 一行证据）

- **T-01 P3/P4 有界调查 → 决策件**：最小夹具 + 单元断言定位 padding 丢失点（uniform 覆盖 per-axis？契约类合并丢类？），
  产出覆盖次序规则与修复点。验证：夹具上新旧行为对比（VM dump / 单元测试）。
- **T-02 P2 图标尺寸与居中**：`aura_view_builder.rs` / `ui/iced/renderer.rs` / `ui_gen/vue.rs` 三处按 T-01 结论统一 `size` 权威。
  验证：`tools/parity_shot_diff.py` 的 ink 比 ≥0.9。
- **T-03 P1 色板单源**：按 §10① 裁决落地 + 跨端色值断言。验证：语义面逐通道差 ≤2。
- **T-04 门禁化**：`tools/parity_shot_diff.py` 登记到 `.agents/skills/autoui-verifier/` 清单与 015-notes acceptance T14。
- **T-05 实机复验与归档**：双端新截图 + `cargo tv`/`cargo t` 对拍 + 更新 attachments。

**依赖**：T-01 → T-02/T-03 → T-04 → T-05。

## 9. 复审记录

### 9.1 /auto-plan:new 起草移交（2026-09-12）

- stage: new；Plan ID: PLAN-619；plan_revision: 1。
- 依据：用户实测 4 项现象 + 本计划 §4.1/§4.2 的像素级复核（`tools/parity_shot_diff.py` 可复跑）。
- outcome: **blocked**——§10① 的「权威色板」裁决会改变跨示例视觉基线，需用户定夺后再进入 work；
  其余任务（T-01/T-02/T-04/T-05）在裁决前已可开工（T-03 依赖裁决）。
- next: 用户裁定 §10① 后 → work（T-01 起）。

## 10. 待澄清事项

1. **P1 权威色板（必须裁决）**：语义 token 应以哪套色板为准？
   - **选项 A（推荐）**：**scaffold/shadcn 为权威**——VM 的语义解析对「未声明 `theme{}` 的 scaffold 类应用」
     与 Vue CSS 同源（改 `active_theme()` 缺省或生成侧对齐）。影响：桌面 shell/stella 类应用需显式声明 `theme{}` 才能保持 stella。
   - **选项 B**：**stella 为权威**——改 auto-man 的 scaffold `index.css` 模板走 stella 值。影响：Web 侧全部示例/gallery 的
     视觉基线整体变化（面大），且与 shadcn 组件默认观感脱钩。
   - **选项 C**：示例侧 `theme{}` 钉住（015-notes 先落地），引擎默认不动、差异长期保留为「已知差异」。
     代价：每个 scaffold 类应用都要手抄一遍色板，GOAL-007 的「一致」只在小范围成立。
2. **P2 的 `size` 权威**：以 `.at` 的 `size:` 为准（则 `w-5 h-5` 需让位），还是以 class 为准（则 `size:` 退化）？
   建议前者（语言层 prop 优先），需在 SD-02 写明。
