---
plan_id: PLAN-593
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: theme-registry-token-single-source
author: [zhaopuming]
created_at: 2026-09-09
updated_at: 2026-09-09

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # specs 路径（ui/style + ui_gen/vue + code_editor）
current_step: 0
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
- **S2 钉基线（先钉后改）**：新建 `crates/auto-lang/src/plan593_theme_registry_tests.rs`
  ——T-a 期望表（抄现行 match 臂 RGB）+ T-b golden（现行 `generate_base_css()`
  全文）。此时断言目标函数尚是旧实现——测试先行**直接绿**（旧实现即真值）。
  lib.rs 注册 mod。验证：`cargo t plan593` 全绿。
- **S3 registry 落地**：`git mv crates/auto-lang/src/ui/style/theme.rs …/theme/mod.rs`
  （module 路径 `ui::style::theme` 不变，零调用点扰动）；新建
  `…/ui/style/theme/registry.rs`：TokenName/ColorLit/PaletteSpec/ThemeSpec/
  builtin（zinc-light/dark 值逐字抄 vue.rs 15570-15622，stella-light/dark 逐字抄
  theme.rs match 臂）/resolve/render_css_vars。验证：`cargo check -p auto-lang`
  + `cargo t ui::style::theme`。
- **S4 V2 改造**：`theme/mod.rs::resolve_semantic_rgb` match 臂 → 投影查表
  （D2；Primary 臂改读 registry 的 accent 表，逻辑不变）。验证：
  `cargo t plan593 && cargo t ui::style`（T-a 零漂移钉 + 既有 stella 钉全绿）。
- **S5 V1 改造**：`ui_gen/vue.rs::generate_base_css` 变量块 →
  `render_css_vars(builtin("zinc"))`（D3）。验证：`cargo t plan593`（T-b golden
  逐字节绿）+ `cargo t vue`。
- **S6 V4-a 收敛**：`ui/code_editor/theme.rs` 删本地 `accent_hsl`/`hsl_to_rgb`，
  改引 `ui::style::theme` 单源；修 stale 注释（D4）。验证：
  `cargo check -p auto-lang && cargo t plan593`（T-d accent 单源断言）。
- **S7 P571-D1 证据门**：跑爆炸半径 grep（D5）；零命中 → `color.rs` 补
  `Color::Accent`/`OnAccent` 变体 + `"accent"`/`"accent-foreground"` 解析臂 +
  registry 投影；有命中 → KNOWN-DEBT 登记钉 Phase 2。验证：
  `cargo t ui::style && cargo t plan593`。
- **S8 词表封闭性测试**：T-c 断言补进 plan593 测试文件。验证：`cargo t plan593`。
- **S9 健康检查**：`cargo check -p auto-lang` 零警告；无遗留 debug 输出；
  `grep -n "互锁\|改任一须同步" crates/auto-lang/src` 确认 V1/V2 处互锁注释
  已随迁移消解或改写为指向 registry。
- **S10 全量门禁**：`cargo t` 全量（预期 = 现行基线红名单零新增）。
- **S11 review/fold 前**：`cargo tf`（Category B 全量档）；本档执行步骤勾记 +
  复审记录待 `/auto-plan:review`。

## 复审记录

（S1 对账表回填区）

## 待澄清事项

1. **vue 桌面宿主 CSS 路径归属**（S1 解决）：本仓 vue.rs base_css 供脚手架
   app；桌面 vue 宿主（465 wm）的 index.css 是否另有发射点（auto-os 仓？）——
   若在仓外，zinc/stella 归一联动需跨仓协调，Phase 2 立项时确认。
2. **accent dark 提亮 +4/+10 分叉**（S1 定性，Phase 2 裁定归一值）。
3. **三键收敛（card/popover/surface→Surface）**：本 plan 只成文不扩枚举
   （Design 29 §8-3）；Phase 2 视内置主题是否需要三键分色再定。
