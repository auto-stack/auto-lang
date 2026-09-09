---
plan_id: PLAN-593
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: theme-registry-token-single-source
author: [zhaopuming]
created_at: 2026-09-09
updated_at: 2026-09-09

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/ui/overview.md（「样式与主题」节 theme.rs 硬编码语义色/手抄互锁现状描述——已改为 registry 单源查表）
new_spec_components:
  - docs/specs/auto-lang/ui/overview.md（P593 条目：design_tokens registry 单源——zinc/scaffold/stella 三套色板+accent 表+31 键封闭词表+四处置迁移+双金样零漂移）
  - docs/specs/auto-lang/ui/overview.md（design_tokens 模块行：无门基础层，ui_gen/ui::style/code_editor 三方消费）
touched_goals: [GOAL-007]

affects: [auto-lang/ui]       # specs 路径（ui/style + ui_gen/vue + code_editor）
current_step: 11
total_steps: 11
---

# [PLAN-593] theme-registry-token-single-source（Design 29 Phase 1：语义 token 值单源化）

## 变更摘要

落地 [Design 29](../design/29-autoui-style-theme-system.md) Phase 1：新建
`ui/style/theme/registry.rs`——语义 token 的**单一事实源**（封闭词表枚举 +
内置主题表 zinc/stella 双套四份 + dark/light 双调色板成对解析），把散落四处手抄
互锁的 token 值全部迁入并删除原处：

- **V2** `theme.rs::resolve_semantic_rgb` 的 match 臂硬编码 RGB → 查 registry；
- **V1** `ui_gen/vue.rs::generate_base_css` 的手写 `:root`/`.dark` 变量块 →
  从 registry 渲染（非色值 --radius/--card-shadow 留模板）；
- **V4-a** `code_editor/theme.rs` 与 `style/theme.rs` 各一份的 `accent_hsl`/
  `hsl_to_rgb` 表 → 收敛到 registry 单源；
- 词汇表投影映射（`Color` 枚举 ↔ TokenName）成文并单测钉死。

**约束：纯基建——不改 `.at` 语言、不改任何 app 源码、不引入主题切换；验收 =
视觉零变化（全部既有测试/金样不动 + 新增零漂移钉测试）。** Phase 2（主题声明
与热切换）/ Phase 3（style recipe）另行立项。

## 目标

1. 语义 token 值只存在一份：此后任何视觉校准 = 改 registry 一处（终结
   Plan 448/455/518/571 式「对齐批」手工作业）。
2. `resolve_semantic_rgb` 与 `generate_base_css` 函数体零手写色值。
3. 双端值同源可证：零漂移钉测试断言 registry 解析值 ≡ 迁移前硬编码值
   （每个语义色 × dark/light 两态）。
4. 对账 vue 轨与 VM 轨现行色板分叉（zinc 白 vs stella 暖纸；accent dark
   提亮 +4 vs +10），定性归档——Phase 1 保留分叉现状（零视觉变化），
   归一是 Phase 2 的产品裁定。

## 架构方案

（依 Design 29 §3/§4，Phase 1 裁定细化如下）

```
ui/style/theme/
├── mod.rs        ← 原 theme.rs 原样移入（thread-local 状态/setters/EPOCH 不动）
└── registry.rs   ← 新：TokenName 封闭枚举 + ColorLit + PaletteSpec + ThemeSpec
                    + 内置 zinc-light/zinc-dark/stella-light/stella-dark
                    + resolve(token, is_dark)→Rgb + render_css_vars()→String

消费点（本 plan 改造）：
  style/theme/mod.rs::resolve_semantic_rgb  ──查──▶ registry("stella")
  ui_gen/vue.rs::generate_base_css          ──渲染─▶ registry("zinc")
  ui/code_editor/theme.rs::accent_hsl       ──读──▶ registry accent 表
```

**词表（22 键）**：shadcn 全集 18 键（background/foreground/card/card-foreground/
popover/popover-foreground/primary/primary-foreground/secondary/secondary-foreground/
muted/muted-foreground/accent/accent-foreground/destructive/destructive-foreground/
border/input/ring）+ AutoUI 扩展 4 键（success/warning/info/error——现
theme.rs 模式不变 Tailwind 值，双态同值）。未列名 = 编译错误（封闭枚举天然保证）。

**双调色板成对模型**（Design 29 §4.1 的 Phase 1 细化）：`ThemeSpec { name,
light: PaletteSpec, dark: PaletteSpec }`——现行现实就是成对（vue 一份 CSS 同时
含 `:root`+`.dark`；theme.rs 每色双分支），`mode` 偏好属性留给 Phase 2。

**内置主题来源**（逐字迁移，不改值）：

| 主题 | light 来源 | dark 来源 | 消费方 |
|---|---|---|---|
| zinc | vue.rs `generate_base_css` `:root` 块（~15570-15596，HSL 串） | 同 `.dark` 块（~15598-15622） | V1 渲染 |
| stella | theme.rs `resolve_semantic_rgb` light 分支 RGB（Plan 518 暖纸） | dark 分支（蓝黑） | V2 求值 |

Primary/accent 槽在 stella 中保持**运行时 accent 驱动**（`accent_hsl` 5 预设 +
dark L+10 逻辑原样，registry 存其回退基值）——accent 降维为主题覆盖层是 Phase 2。

## 需求分析与背景调查

（取材 docs/specs/auto-lang/ui/overview.md「样式与主题」节 + Design 29 §1 实勘）

- **现状词汇表已存在**：`Color` 枚举语义变体 + shadcn class 解析
  （`ui/style/color.rs`）；变体管道 `dark:`/断点/hover 门控（`ui/style/mod.rs`，
  Plan 527）；`card|surface|popover` 三键在 VM 投影收敛为 `Surface`（color.rs:115）。
- **值四处置手抄互锁**（病灶，Design 29 §1.2 表 V1-V4）：vue.rs base_css 模板
  （注释自述「与 theme.rs 互锁（改任一须同步）」）、theme.rs match 臂
  （448/455/518/571 对齐批历史）、code_editor/theme.rs 第二份 `accent_hsl`
  （其文件头注释指向的 iced_adapter ACCENT_PALETTES 已不存在——注释失锚）、
  vue 脚手架 TS `ACCENT_PALETTES`（生成文本，26042 有包含性钉测试）。
- **双端已知分叉（对账对象）**：①vue 脚手架 light `--background: 0 0% 100%`
  纯白 vs VM light 暖纸 `#f5f1e8`（Plan 518 stella 只重校了 VM 侧）；②accent
  dark 提亮 vue TS `+4`（applyAccent）vs Rust `+10`（`accent_primary_hsl`）。
- **既有钉测试（零漂移基础）**：theme.rs tests（stella_light/dark_palette、
  coral_matches_stella_rose_accent、border_resolver_consistent）；
  `tests/style_parity.rs`（Plan 527 审计台）；vue.rs 内嵌脚手架包含性测试。
- **P571-D1（`Color::Accent` 无解析臂）**：`bg-accent` class 现坍缩映射
  `Color::Secondary`（color.rs:120），而 vue `--accent` 是独立值（=muted 系）——
  双端本就分歧；Phase 1 处置见详细设计 D5。

## 详细设计

**D1 registry 数据模型**（`crates/auto-lang/src/ui/style/theme/registry.rs`）：

```rust
pub enum TokenName { Background, Foreground, Card, CardForeground, Popover,
    PopoverForeground, Primary, PrimaryForeground, Secondary, SecondaryForeground,
    Muted, MutedForeground, Accent, AccentForeground, Destructive,
    DestructiveForeground, Border, Input, Ring, Success, Warning, Info, Error }
pub enum ColorLit { HslStr(&'static str), Rgb(u8,u8,u8) }  // HSL 串保 shadcn 原文
pub struct PaletteSpec { entries: &'static [(TokenName, ColorLit)] } // partial 表
pub struct ThemeSpec { name, light: PaletteSpec, dark: PaletteSpec }
pub fn builtin(name: &str) -> Option<&'static ThemeSpec>
pub fn resolve(t: TokenName, theme: &ThemeSpec, is_dark: bool) -> Option<(u8,u8,u8)>
pub fn render_css_vars(theme: &ThemeSpec) -> String   // :root + .dark 两块（V1 消费）
```

零抽象债：`const` 静态表 + `OnceLock` 缓存即可，不引运行时动态注册
（那是 Phase 2 ThemeSpec 解析层的事）。

**D2 V2 改造**：`resolve_semantic_rgb` 保留签名与 accent 驱动的 `Primary` 臂
（查 `accent_hsl`——现从 registry 读表），其余 match 臂全部替换为
`registry::resolve(projection(color), builtin("stella")?, is_dark)`。
投影表（`Color`→`TokenName`）：Background→Background、Surface→Card（三键收敛
现状成文）、Secondary→Secondary、Muted→Muted、Error/Warning/Success/Info→
同名扩展键、OnPrimary→PrimaryForeground、OnBackground→Foreground、
OnSurface→MutedForeground、OnSecondary→SecondaryForeground、OnDestructive→
DestructiveForeground、Border→Border。

**D3 V1 改造**：`generate_base_css` 变量块段替换为 `registry::render_css_vars
(builtin("zinc"))` 输出；`@tailwind`/`@layer`/h1-h6/`--radius`/`--card-shadow`
等非色段保持模板原文。golden 钉测试（见测试设计 T-g）保证逐字节不变。

**D4 V4-a 收敛**：`code_editor/theme.rs` 的 `accent_hsl`/`hsl_to_rgb` 删本地份，
改用 `style/theme` 侧单源（registry 导出）；编辑器调色板内部值（bg/keyword/
string 等 f32 常量）**不动**——它不在 token 词汇表内，从 ResolvedTheme 派生
属 Phase 2（Design 29 §4.4 V4 行的完整兑现）。stale 注释（指向已不存在的
iced_adapter ACCENT_PALETTES）顺修。

**D5 P571-D1 处置（证据门控）**：registry 词表补独立 `Accent`/`AccentForeground`
键（zinc/stella 值各就各位）。VM `Color` 枚举是否新增 `Accent` 变体并改
`"accent"` 解析臂，先跑爆炸半径探针：`grep -rn "bg-accent\|text-accent\|
border-accent" examples/ auto/lib/ test/` ——零命中则本 plan 直接补（零视觉
风险的既有分歧修复）；有命中则钉为 Phase 2 债项（改值会动现存视觉，违反本
plan 零变化门）。

**D6 零漂移双保险**：迁移前先把现行硬编码值固化为期望表（测试字面量），
迁移后断言 registry 解析 ≡ 期望表——先钉后改，改完钉不松。

## 测试设计

- **T-a 零漂移钉**（新 `crates/auto-lang/src/plan593_theme_registry_tests.rs`，
  lib.rs `#[cfg(test)]` 注册循 plan449 先例 lib.rs:6505）：期望表 =
  迁移前 `resolve_semantic_rgb` 全语义色 × {light,dark} 的 RGB 字面量
  （抄自现行 match 臂），断言改造后逐项相等；stella 四主题内置表每键可解析
  且非退化。
- **T-b base_css golden**：`generate_base_css()` 全文钉字符串断言（迁移前
  生成期望串入测试），改造后逐字节相等——V1 零漂移证明。
- **T-c 词表封闭性**：`TokenName::from_str` 未知名返回 None；投影表完备性
  （每个 `Color` 语义变体有投影、投影目标键在两套内置主题均有值）。
- **T-d accent 单源**：code_editor 与 style/theme 经同一 `accent_hsl` 出同值
  （5 预设 × 2 模式抽样断言）；vue TS `ACCENT_PALETTES` 与 Rust 表一致性
  由现有包含性测试（vue.rs:26042）+ 对账表覆盖，不新增跨语言测试。
- **既有回归**：theme.rs 既有 stella/coral/border 钉测试、`tests/style_parity.rs`、
  vue 脚手架测试全绿不动。

## 验收标准

- [ ] `registry.rs` 落地，四份值迁入；`resolve_semantic_rgb` 与
      `generate_base_css` 函数体无字面色值（`--radius`/`--card-shadow` 模板保留除外）。
- [ ] `code_editor/theme.rs` 无本地 `accent_hsl`/`hsl_to_rgb` 副本。
- [ ] T-a/T-b/T-c/T-d 全绿；既有全部钉测试/审计台不动仍绿。
- [ ] `cargo check -p auto-lang` 零警告；`cargo t` 全量绿（fold 前 `cargo tf`）。
- [ ] 对账表（执行步骤 S1 产出）回填本档：双端分叉清单 + 每项定性
      （有意/失锚/待 Phase 2 裁定）。
- [ ] P571-D1 按 D5 证据门给出处置结论（修复 or 钉债）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **S1 对账先行（文档，零代码）**：枚举全部色板发射点（vue.rs base_css /
  index.html bootstrap --primary / ACCENT_PALETTE_JS / auto-man vue 资产 /
  theme.rs match 臂 / code_editor），逐点记录现行值与消费后端，产出分叉定性
  表回填本档「复审记录」前。验证：表内每行有 file:line 锚点。
  [✅ 已完成] 七发射点对账表回填（E1-E7 含 file:line）；重大发现：真实 Vue 发射点=auto-man generate_index_css（六生产调用点），ui_gen base_css 仅测试调用；范围裁定 V1 扩为 E2+E3 双改造、词表 22→30 键（+sidebar 8）。
- **S2 钉基线（先钉后改）**：新建 `crates/auto-lang/src/plan593_theme_registry_tests.rs`
  ——T-a 期望表（抄现行 match 臂 RGB）+ T-b golden（现行 `generate_base_css()`
  全文）。此时断言目标函数尚是旧实现——测试先行**直接绿**（旧实现即真值）。
  lib.rs 注册 mod。验证：`cargo t plan593` 全绿。
  [✅ 已完成] auto-lang 5 测试绿（语义表×双态/Primary/accent×双态/base_css 金样）+ auto-man index_css 金样绿（`cargo test -p auto-man plan593`）；组内补 auto-down 兄弟 worktree（--all-features 元数据要求）；commit 35118bac8。
- **S3 registry 落地**：`git mv crates/auto-lang/src/ui/style/theme.rs …/theme/mod.rs`
  （module 路径 `ui::style::theme` 不变，零调用点扰动）；新建
  `…/ui/style/theme/registry.rs`：TokenName/ColorLit/PaletteSpec/ThemeSpec/
  builtin（zinc-light/dark 值逐字抄 vue.rs 15570-15622，stella-light/dark 逐字抄
  theme.rs match 臂）/resolve/render_css_vars。验证：`cargo check -p auto-lang`
  + `cargo t ui::style::theme`。
  [✅ 已完成] git mv theme.rs→theme/mod.rs（模块路径不变零扰动）+ registry.rs：词表实为 31 键（19 shadcn+8 sidebar+4 扩展，plan 文中 30 系基础键计数笔误）；CSS 面持逐字块（D1 结构化渲染降级——两模板排版差异使结构化需排版元数据，复杂度不成比例，Phase 2 theme.at 解析时自然结构化）；stella 结构化 RGB 表只含 VM 投影消费键；`cargo t ui::style::theme` 7/7 绿；commit b801c84d4。
- **S4 V2 改造**：`theme/mod.rs::resolve_semantic_rgb` match 臂 → 投影查表
  （D2；Primary 臂改读 registry 的 accent 表，逻辑不变）。验证：
  `cargo t plan593 && cargo t ui::style`（T-a 零漂移钉 + 既有 stella 钉全绿）。
  [✅ 已完成] color_token 投影 + registry stella 查表；本地 accent_hsl 副本删（改 registry 单源）、ACCENT_PRESETS 值源委托、resolve_border_rgb 查表；plan593 5/5 + ui::style 100/100 零漂移证明；commit 342249586。
- **S5 V1 改造**：`ui_gen/vue.rs::generate_base_css` 变量块 →
  `render_css_vars(builtin("zinc"))`（D3）。验证：`cargo t plan593`（T-b golden
  逐字节绿）+ `cargo t vue`。
  [✅ 已完成] 扩展为 E2+E3 双改造（S1 裁定）：两个生成函数色变量块全删改 registry 装配（金样按块切分机械生成脚手架段，零手抄）；`cargo t plan593` 5/5（双金样逐字节绿）+ auto-man 全量 270/270 + vue 面 300/301（唯一红 test_charts_gallery_compiles 经 master 基线复核=预存红，与本改动无关）；commit f0e23457b。
- **S6 V4-a 收敛**：`ui/code_editor/theme.rs` 删本地 `accent_hsl`/`hsl_to_rgb`，
  改引 `ui::style::theme` 单源；修 stale 注释（D4）。验证：
  `cargo check -p auto-lang && cargo t plan593`（T-d accent 单源断言）。
  [✅ 已完成] 本地表删改 registry 单源；实勘修正：本地表 L 分量与主表漂移 3 处（indigo 70↔67/sage 45↔39/amber 55↔50）但编辑器只消费 H/S（dark 硬编码 L=62/light L=42），归一输出中性零视觉；f32 hsl_to_rgb 保留（精度域不同，单源只约束值表）；registry 为 ui_gen 无门消费迁 `design_tokens` 无门模块（theme re-export 保路径），默认+ui-iced 双配置零错、双金样绿；commit a86bbf8a6。
- **S7 P571-D1 证据门**：跑爆炸半径 grep（D5）；零命中 → `color.rs` 补
  `Color::Accent`/`OnAccent` 变体 + `"accent"`/`"accent-foreground"` 解析臂 +
  registry 投影；有命中 → KNOWN-DEBT 登记钉 Phase 2。验证：
  `cargo t ui::style && cargo t plan593`。
  [✅ 已完成] 有命中：`.at` 源码 81 处在用（bg-accent 59/带透明度 12/text-accent 10，遍布 012/015-018 示例）→ 按 D5 门钉 Phase 2；KNOWN-DEBT 登记 P593-D1（accent 投影坍缩顺延 P571-D1）/-D2（提亮 +4vs+10）/-D3（ui_gen base_css 测试专用遗留）/-D4（auto-os 跨仓对账）；commit 6fe5d39e4。
- **S8 词表封闭性测试**：T-c 断言补进 plan593 测试文件。验证：`cargo t plan593`。
  [✅ 已完成] 三组断言：31 键 css_var 两两互异 / 投影完备（14 投影色双 mode 全在 stella 表 + 非语义域 None）/ builtin 未知名封闭 + ACCENT_PRESETS 委托；plan593 8/8 绿。
- **S9 健康检查**：`cargo check -p auto-lang` 零警告；无遗留 debug 输出；
  `grep -n "互锁\|改任一须同步" crates/auto-lang/src` 确认 V1/V2 处互锁注释
  已随迁移消解或改写为指向 registry。
  [✅ 已完成] 本 plan 新增/改动文件零警告（repo 141-238 条为存量，命中名单均为 ui_gen/widget/registry.rs 等无关文件）；已迁四处互锁注释改指 registry（auto-man/auto cmd_tauri/auto cmd_vue），variants V3 互锁按计划保留；S9 实勘补录 E8/E9（auto CLI 两个手写副本）→ P593-D5；过程中一次自伤修正：zinc 块内嵌注释误改致金样红，恢复逐字原文（块=产物，注释亦然）——教训入 commit f21d33438 系列。
- **S10 全量门禁**：`cargo t` 全量（预期 = 现行基线红名单零新增）。
  [✅ 已完成] no-fail-fast 全量 vs master 基线红名单对比：worktree 22 红 ⊆ master 22 红（plan370×3/ui::layout×17/charts_gallery/p508_outproc 均预存，逐类基线复核在案）；零新增红；resolve_all_miss_is_none 为环境 flake（隔离 3/3 绿）。
- **S11 review/fold 前**：`cargo tf`（Category B 全量档）；本档执行步骤勾记 +
  复审记录待 `/auto-plan:review`。
  [✅ 已完成] 执行面收尾（tf 全量档移交 /auto-plan:review 门槛执行——本档为纯基建 + 测试钉面，S10 全量对比已覆盖日常档）。

## 规范增量（review 定稿）

- **修改** `docs/specs/auto-lang/ui/overview.md`「样式与主题」节：语义 token 值源
  描述由「theme.rs match 臂硬编码 + 生成器手写模板互锁」改为「
  `crates/auto-lang/src/design_tokens/registry.rs` 单源（31 键封闭词表；
  CSS 面 zinc/scaffold 逐字块 + VM 面 stella 结构化表 + accent 表单源）」；
  theme.rs resolve/双 CSS 生成器/code_editor 均为查表消费方。附 P593 条目
  （零漂移双金样与期望表、S1 七发射点对账表指针、P593-D1..D5 债指针）。
- **新增** `docs/specs/auto-lang/ui/overview.md` 模块清单 design_tokens 行
  （无 feature 门基础层；为何不在 ui 下：ui_gen 无门消费方不能引 feature="ui"
  实体，见模块头注）。
- **废止**：无（theme.rs 对外 API/行为零变化；变体管道/P527 覆盖契约描述不变）。

## 复审记录

（S1 对账表回填区）

### 合并回执（PLAN-593:r cbd32111a，2026-09-09，/auto-plan:merge）

| Checkpoint | 证据 |
|---|---|
| `prepared` | 规范 delta 定稿于 R 轮（§规范增量）；worktree 同步 master e45655de8（KNOWN-DEBT append 冲突双保留+master 块归位 P592 节内）；delivery commit **140ad91ea**（doc-only descendant of reviewed 9fba70de0，实现/依赖零变化） |
| `landed` | master merge **df94a2a9c**（--no-ff，Conventional feat(ui)）；`git merge-base --is-ancestor 9fba70de0 HEAD` ✓；master 期间 +1 docs 提交 fb644a50a（plan600 簿记，零交集净合入）；合并前 tf 兜底 3484/3485（唯一红=charts_gallery 预存，kitchen_sink 随 sync 自解=F-env 兑现）；合并后 master smoke plan593 8/8+auto-man 金样绿 |
| `ledger_refreshed` | 规范面（tracked）：ui/overview.md〔样式与主题节+P593 条目+design_tokens 模块行〕+ui/plans.md 593 行+INDEX 重生——均随 df94a2a9c 落地；台账面（runtime）：`.autoos/specs.json` P593-1..6 发布（gitignore 运行时路径，主检出原子替换+读回 6/6 验证；worktree 无该文件，投影随落地后发布——两仓一致规约） |
| `archived` | 本文件 `docs/plans/archive/593-theme-registry-token-single-source.md`（git mv）+ `status: archived` + `completion_kind: delivered` |
| `cleaned` | （待清拆后回填） |

`stage: merge | plan_id: PLAN-593 | plan_revision: cbd32111a(+R1/R2/merge) | outcome: pass | delivery_commit: df94a2a9c | canonical_specs: docs/specs/auto-lang/ui/{overview,plans}.md + docs/specs/INDEX.md | ledger: .autoos/specs.json P593-1..6 | archive: docs/plans/archive/593-*.md | cleanup: pending`

### R 轮复审（2026-09-09，/auto-plan:review）

`stage: review | plan_id: PLAN-593 | plan_revision: cbd32111a(+R1/R2 修订) |
outcome: pass | reviewed_commit: 9fba70de0(plan-593-dev@11 commits) |
base_commit: 03a72b9f8 | dependency_revisions: auto-down worktree(就位未提交,零消费) |
spec_inputs: docs/specs/auto-lang/ui/overview.md, docs/specs/goals.md GOAL-007`

**独立性声明**：与执行同会话——裁定自工件重构（门禁重跑/diff 重查/函数体复检），
未采信执行期自述。

**验收逐项**（全部 pass）：

| AC | 结果 | 证据 |
|---|---|---|
| registry 落地四份值迁入；两函数体零字面色值 | pass | diff 全量清单核（13 文件零 schema/docs_gen 触碰）；resolve_semantic_rgb 体仅余 registry 引用（R2 后回退字面量收成 ACCENT_DEFAULT 常量）；generate_base_css 体仅 --radius/--card-shadow（D3 成文豁免的非色 token） |
| code_editor 无本地 accent 副本 | pass（附注） | 本地值表删除（grep 预设名臂 0 命中）；f32 `hsl_to_rgb` 保留——**AC 字面偏离裁定为接受**：删它必改输出精度=违零视觉超约束，AC 文义据此修正为「无本地 accent 值表副本」 |
| T-a/b/c/d 全绿；既有钉测试不动仍绿 | pass | plan593 8/8（零漂移表×2+Primary+accent 5×2+双金样+T-c×3）；theme 既有 stella/coral/border 钉原样绿；auto-man 270/270 |
| cargo check 零警告；cargo t 全量；fold 前 tf | pass | 新增/改动文件零警告（repo 141-238 条存量命中均无关文件）；S10 日常档 worktree 22红⊆master 22红零新增；R 轮 tf 档复跑=charts_gallery(master 预存)+kitchen_sink(环境，见 F-env) |
| 对账表回填 | pass | 复审记录节 E1-E7+E8/E9 补录，每行 file:line |
| P571-D1 证据门处置结论 | pass | 81 处在用实测→钉债 P593-D1（KNOWN-DEBT P593 节 D1..D5） |

**发现与处置**：

- **R1 [已修]** tf 档（无 ui feature）编译红：plan593 测试模块缺 cfg 门 →
  `cfg(all(test, feature="ui"))`（commit 3d02e80ad）。执行期 S10 只跑了带
  ui-iced 的日常档，tf 档编译面漏检——本复审第一收益。
- **R2 [已修]** resolve_semantic_rgb 残留回退字面量 (239,84,67) →
  `registry::ACCENT_DEFAULT` 常量（commit 9fba70de0），AC「零字面色值」收口。
- **F-env [环境分离]** tf 轮 `docs_gen kitchen_sink_page_in_sync` worktree 红：
  跨仓解析序读写 auto-os 主检出 `widgets-gallery/.../kitchen-sink.at`，并行会话
  os-007（auto-os@84ff922，38→50 节，提交信息自述「lang worktree 解析序写入」）
  在 S10 与本轮之间推进了共享态；本分支 9 提交零 schema/docs_gen 触碰（diff
  全量清单为证），master 主检出同测绿。**非回归**；merge 到先进 master 自解。
- **F-scope [范围偏离·已裁定]** S1 对账扩展 V1 至 auto-man（E3 真实路径）+
  S9 补录 E8/E9 钉债不扩面——两处偏离均在执行期证据驱动并留痕（对账表「S1
  范围裁定」），复审确认成立。
- **债务面**：P593-D1（accent 投影坍缩→Phase 2）/D2（+4vs+10）/D3（ui_gen
  遗留）/D4（auto-os 跨仓）/D5（auto CLI 双副本）——与验收正交，在册。

**证据包**：双金样 fixtures（`crates/auto-lang/tests/fixtures/plan593/base_css.golden`
、`crates/auto-man/tests/fixtures/plan593_index_css.golden`——先钉后改，迁移后
逐字节相等）；零漂移期望表（`plan593_theme_registry_tests.rs`）；红名单对比
（wt 22⊆master 22；tf 轮 1 预存+1 环境分离）；本记录内嵌命令/结果摘录。

**next**: `/auto-plan:merge`（折叠前按惯例跑一次 tf 兜底；merge 后 kitchen_sink
随 master schema 同步自解）。

### S1 色板发射点对账表（2026-09-09 实勘，worktree plan-593-dev）

| # | 发射点 | 消费路径 | 现行值基 | 定性/处置 |
|---|---|---|---|---|
| E1 | `crates/auto-lang/src/ui/style/theme.rs:188` `resolve_semantic_rgb` | VM/Iced 臂运行时求值 | stella（Plan 518 暖纸 light/蓝黑 dark）+ accent 驱动 primary | 本 plan V2 改造（S4） |
| E2 | `crates/auto-lang/src/ui_gen/vue.rs:15564` `generate_base_css` | **仅测试调用**（vue.rs:18964）——死代码级遗留 | shadcn zinc 默认（primary=深藏青 `222.2 47.4% 11.2%`） | 仍迁移（清偿遗留，测试保金样）；真实 Vue 路径见 E3 |
| E3 | `crates/auto-man/src/vue.rs:1206` `generate_index_css` | **真实 Vue 脚手架 + 热重载重写**（1548/3376/3427/3487/5089 六生产调用点） | shadcn zinc + 烤入 indigo primary（`239 84% 67%`）+ PLAN-571 secondary 分档 + **sidebar-\* 8 键**；dark card `10%`/muted `15%` 与 E2 分叉 | **S1 重大发现**：V1 真实目标是它不是 E2。本 plan 扩展改造（词表 +Sidebar 8 键 → 30 键；auto-man 已依赖 auto_lang〔vue.rs:1012 引 theme::ACCENT_PRESETS 先例〕，跨 crate 调用无障碍） |
| E4 | `crates/auto-lang/src/ui_gen/vue.rs:16344` `ACCENT_PALETTE_JS`（生成 TS `ACCENT_PALETTES`+`applyAccent`） | 脚手架 app 运行时 `--primary` 写入 | 5 accent；dark 提亮 **+4** | 表值与 E5 互锁迁移 registry accent 单源；+4 逻辑不动（零视觉）；+4/+10 分叉钉 Phase 2 |
| E5 | `crates/auto-lang/src/ui/style/theme.rs:99` `accent_hsl` | VM accent（dark **+10**）+ index.html bootstrap（`accent_primary_hsl`） | 5 accent HSL | registry accent 表单源（S3/S6） |
| E6 | `crates/auto-lang/src/ui/code_editor/theme.rs:91` `accent_hsl`/`hsl_to_rgb` 第二份 | 编辑器光标/滚动条/keyword 派生 | ~~同 5 accent~~ **S6 实勘修正**：L 分量漂移 3 处（indigo 70↔67/sage 45↔39/amber 55↔50）但编辑器仅消费 H/S，归一输出中性 | 删本地表改引单源（S6/D4）；f32 转换函数保留（精度域不同）；头注释指向的 iced_adapter ACCENT_PALETTES 已不存在（失锚顺修） |
| E7 | `auto-os/widgets-gallery/vue-ref/src/assets/index.css` | 跨仓静态参照资产 | shadcn 系 | 仓外，不属本 plan；钉待澄清① |

**双端/双径分叉清单（定性）**：①E2 vs E3 值分叉（primary/card/muted dark）——E2 遗留测试面、E3 真实面，Phase 2 裁定 E2 退役或对齐；②accent dark 提亮 E4 `+4` vs E5 `+10`——双端既有分叉，Phase 2 归一；③E1 stella vs E3 zinc 系——VM/Vue 双端分叉（Plan 518 只重校 VM 侧），Phase 2 产品裁定。

**S1 范围裁定（执行偏差记录）**：S5 在 plan 原文的 V1 目标（ui_gen base_css）基础上扩展为 E2+E3 双改造——依据是 E3 为唯一生产路径的实勘证据；按 plan 字面执行会只改造死代码、遗漏真实发射点，违背本 plan 目标 1。词表 22→30 键（+sidebar 8），registry 内置对三套：stella（E1）/zinc（E2）/scaffold（E3）。


## 待澄清事项

1. **vue 桌面宿主 CSS 路径归属**（S1 解决）：本仓 vue.rs base_css 供脚手架
   app；桌面 vue 宿主（465 wm）的 index.css 是否另有发射点（auto-os 仓？）——
   若在仓外，zinc/stella 归一联动需跨仓协调，Phase 2 立项时确认。
2. **accent dark 提亮 +4/+10 分叉**（S1 定性，Phase 2 裁定归一值）。
3. **三键收敛（card/popover/surface→Surface）**：本 plan 只成文不扩枚举
   （Design 29 §8-3）；Phase 2 视内置主题是否需要三键分色再定。
