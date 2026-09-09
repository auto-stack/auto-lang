---
plan_id: PLAN-601
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: theme-declaration-hot-switch（Design 29 Phase 2）
author: [zhaopuming]
created_at: 2026-09-09
updated_at: 2026-09-09
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-man, auto]   # specs 路径
current_step: 0
total_steps: 12
---

# [PLAN-601] theme-declaration-hot-switch（Design 29 Phase 2：主题声明与热切换）

## 0. 变更摘要

落地 [Design 29](../design/29-autoui-style-theme-system.md) Phase 2：主题从
「dark/light 二值 + accent 单槽」升级为**可命名、可派生、可热切换的整套色板**。

- **A 声明面**：pac.at `theme {}` 块（extends/mode/colors）解析与链式合成，
  封闭词表编译期校验（未知 token = 错误）；
- **B 切换面**：`SetTheme(name)` 泛化（session.rs WmCommand）+ os-config
  `appearance.theme` 值域扩主题名（旧 `dark|light` 读回兼容）+ settings 面板
  主题选择器 + Vue 臂 `applyTheme` 运行时整套变量写入（applyAccent 先例泛化）；
- **C 结构面**：registry 双面统一（每主题 CSS 面 + VM 面——Phase 1 的
  zinc/scaffold CSS-only / stella RGB-only 升级为结构化 ThemeSpec 双渲染）；
  `dark:` 门控经 `active_theme().is_dark` 泛化；
- **D 债收口**：P593-D1（accent 投影坍缩，81 处在用的双端对齐）、D2（accent
  dark 提亮 +4/+10 归一）、D3（ui_gen base_css 测试专用遗留退役）、D5（auto
  CLI cmd_tauri/cmd_vue 双副本收编）、D4（auto-os 跨仓对账，bounded 调查）；
- **E 归一裁定**：zinc/stella 双轨分叉定性——推荐「零变化优先：named themes
  全保留、双轨默认不变、切换能力本身即归一手段」（裁定项见 §10）。

非目标（Design 29 §4.5 分期）：per-app color context 2b（渲染时按 App 路由
各自主题）——出栈对齐 RenderQueue 色彩上下文重构或独立 plan；style recipe
语言层 = Phase 3。

## 1. 目标

1. 任何 token 化书写的 app 可在 ≥3 套内置主题间热切换，token 化区域 100%
   跟随（含 code_editor 高亮层 = Design 29 §4.4 V4 完整兑现）。
2. 主题可声明、可派生：app 用 `theme { extends/colors }` 局部覆盖内置主题。
3. 旧配置零破坏：`theme: "dark"|"light"` 与 `accent:` 5 预设照常工作。
4. 五条 P593 债全部收口或带结论关闭。
5. Phase 1 的「逐字块」形态升级为结构化 ThemeSpec（CSS 值与 RGB 值同源
   双渲染）——金样口径从 byte-pinned 升级 value-pinned（布局归一后合法再生）。

## 2. 架构方案

```
pac.at theme{} ──解析──▶ ThemeSpec(声明态) ──extends 合成──▶ ResolvedTheme
                                                            │ 双面
                     ┌──────────────────────────────────────┤
                     ▼ CSS 面                                ▼ VM 面
        vue 脚手架 index.css 生成（canonical 布局）      resolve_semantic_rgb 查表
        + applyTheme() 运行时整套写入（不重建工程）      + code_editor 派生（V4）
                     │                                        │
                     └──────── THEME_EPOCH 失效回路（既有）───┘
   SetTheme(name)（session WmCommand）→ registry 活动槽 + epoch bump + view 重建
```

- **活动主题状态**：theme state 从 (dark_mode: bool, accent: str) 升级为
  `(theme: ThemeId, mode: bool, accent: Option<str>)`——dark_mode/accent 保留
  为兼容读写视图（`dark_mode()` 委托活动主题 mode，`dark:` 门控零扰动）。
- **内置主题集**：`zinc`（Phase 1 E2 值，退役生成器后仅存表）、`scaffold`
  （E3 真实 Vue 路径值）、`stella`（VM 现行，双 mode 对）——各自补齐双面；
  D5 收编后 `tauri`/`cli-vue` 入集（五套）。
- **accent 降维**：accent = 覆盖层（primary/primary-foreground/ring 三槽 +
  dark 提亮），叠加于任意活动主题之上；`pac.at accent:` 键保留。
- **CLI 两副本（D5）**：cmd_tauri/cmd_vue 生成器改 registry 装配（默认各
  保真为内置主题 `tauri`/`cli-vue`，或按 §10 裁定直接对齐 scaffold）。

## 3. 技术栈

Rust（auto-lang：design_tokens/ui/style/ui_gen；auto-man、auto CLI）；.at
parser 扩展（pac.at theme 块）；Vue 脚手架 TS（applyTheme）；测试 = nextest
（t/tf 档 + 改 VM 渲染面 → `cargo tv`）+ autoui-verifier 双端截图对拍（主题
切换验收）。

## 4. 需求分析与背景调查

（取材：Design 29 §4/§7-Phase 2；PLAN-593 归档件〔S1 对账表 E1-E9、R 裮审、
合并回执〕；KNOWN-DEBT P593-D1..D5；docs/specs/auto-lang/ui/overview.md
P593 条目；本会话实勘。授权：用户 2026-09-09 指令——Phase 2 = 主题声明与
热切换，含 P593-D1..D5 收口与 zinc/stella 归一裁定；§10 两项产品裁定待确认。）

- **既有回路可直接复用**：THEME_EPOCH 失效（StreamCache 消费）+ Plan 518
  `SetTheme(bool)` 热切换动词 + storage `shell.appearance.theme` 持久化 + boot
  读回——「换值→失效→重建→构建期重解析」全链已验证；vue `applyAccent`
  （`root.style.setProperty` + .dark 双写）证明运行时变量写入可行。
- **优先级链继承**：CLI > os-config per-app > pac.at > 内置（Plan 458/504/506）。
- **P593-D1 爆炸半径在案**：`bg-accent` 81 处（59+12+10），改投影=VM 视觉
  变化（向 vue `--accent` 值对齐）——本 plan 为**有意的视觉对齐**，非零变化
  门（AC-06 双端对拍验收）。
- **D2 分叉细节**：vue TS applyAccent dark `+4`（ui_gen ACCENT_PALETTE_JS）vs
  Rust `+10`（theme::accent_primary_hsl/Primary 臂）。
- **D4 范围**：auto-os 仓已知 `widgets-gallery/vue-ref/src/assets/index.css`
  为静态参照；是否存在活的桌面宿主 index.css 发射点 = 本 plan bounded 调查。

## 5. 详细设计

### 5.1 registry 双面统一（T-02）

`ThemeSpec` 重构：逐字块 → 结构化 `&[(TokenName, ColorLit)]`（HSL 串保原文，
RGB 由 HSL 解析派生——`hsl_str_to_rgb` 工具入 registry）。CSS 渲染升级为
canonical 布局（固定键序、无注释）——**金样口径变更**：base_css/index_css
金样从逐字节升级为「变量名→值」对等断言（P593 金样退役为对拍样本），布局
归一属合法再生（593-R 轮裁定先例：块=产物，值=契约）。zinc/scaffold/stella/
tauri/cli-vue 各补齐 CSS+VM 双面（值逐字保真，来源=Phase 1 迁移值+两 CLI
模板现值）。

### 5.2 theme 声明（T-03）

pac.at 扩展（兼容既有标量 `theme: "dark"`）：

```auto
theme {
    extends: "stella"        // 缺省按轨：vue 轨 scaffold 系 / VM 轨 stella
    mode: "auto"             // light|dark|auto（缺省继承）
    colors: { primary: "#8b5cf6" }   // partial，未知键=编译错误
}
```

解析入运行期 `ThemeSpec`（与静态内置表同类型）；extends 链深度上限 4（防
环）；词表校验复用 `TokenName` 封闭性。

### 5.3 切换面（T-04/T-05/T-06）

- `set_theme(name)`：registry 活动槽换值 + `THEME_EPOCH` bump（回路既有）；
  `set_dark_mode`/`set_accent_name` 保留为兼容写入口（改写活动主题属性）。
- session.rs `SetTheme(bool)` → `SetTheme(ThemeId)`（WmCommand 新形态 + 旧
  动词串兼容窗口）；storage `appearance.theme` 值域 `dark|light|<name>`（读
  回映射 dark/light→缺省主题+mode 位）；settings.at Appearance 分区二值开关
  → 主题选择器（枚举内置主题）。
- Vue：脚手架 index.css 生成升级（canonical 布局，双 mode 块全集）+ host
  注入 `applyTheme(name)`（写全部变量 + `.dark` class 翻转——applyAccent
  泛化，accent overlay 逻辑内聚同一函数）。

### 5.4 债收口（T-07/T-08/T-09）

- **D2**：提亮归一 **+10**（推荐值：Rust 侧已过 coral 校准测试与 stella 对齐
  实证）——vue TS `+4`→`+10`，vue 暗色 accent 微调在案（AC-06 附件）。
- **D1**：`Color::Accent`/`OnAccent` 变体 + `"accent"`/`"accent-foreground"`
  解析臂 + registry Accent 槽（各主题值已就位）；VM `bg-accent` 视觉向
  `--accent` 值对齐——81 处使用面双端对拍验收。
- **D3**：`ui_gen::generate_base_css` 退役（唯一调用方=自身测试）——函数与
  其金样删除，zinc 主题表保留为可切换内置；PLAN-571 contains 互锁测试归
  auto-man 侧既有件。
- **D5**：cmd_tauri/cmd_vue 生成器色变量块 → registry 装配（保真为内置
  `tauri`/`cli-vue`；vis-* 图表 token 族与双 sidebar 块随装配细化处置，非色
  token 留模板）。
- **D4**：bounded 调查——auto-os 仓桌面宿主 CSS 发射点枚举，结论=关债或
  移交（写入复审记录）。

### 5.5 V4 完整兑现（T-10）

code_editor 主题（bg/fg/caret/syntax 派生）从 `ResolvedTheme` 取值（现硬编码
f32 常量翻译为 registry 值的派生函数，编辑器色域映射成文）；切主题编辑器
高亮同步翻转。

### 5.6 规范增量

| delta_id | op | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md（P593 条目后） | 「主题=dark/light+accent」→「named themes（zinc/scaffold/stella/tauri/cli-vue）+theme{} 声明+热切换+accent 覆盖层」 | 本 plan 主体 | AC-01/02/04 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md（design_tokens 模块行） | 「CSS 面逐字块/VM 面结构化」→「双面结构化 ThemeSpec，canonical CSS 渲染」 | 5.1 | AC-05 |
| SD-03 | add | docs/specs/auto-lang/ui/overview.md | code_editor 主题派生（V4 完整）条目 | 5.5 | AC-08 |
| SD-04 | modify | docs/plans/KNOWN-DEBT-AND-RISKS.md | P593-D1..D5 → 收口/关闭注记 | 5.4 | AC-06 |

## 6. 测试设计

- **T-金样升级**：双 CSS 生成器金样改 value-pinned（解析变量对断言）+ 新
  canonical 布局金样；registry 双面一致性测试（每主题每 token：CSS 值解析
  RGB ≡ VM 面 RGB）。
- **声明解析**：theme{} 合法/非法（未知 token/extends 环/深链）正负用例；
  旧标量 theme 兼容。
- **切换**：set_theme 后 resolve_semantic_rgb/base_css/编辑器派生全翻转
  （单测级）+ THEME_EPOCH 断言。
- **对拍（autoui-verifier）**：≥3 主题 × 双端截图对拍（VM/iced 与 vue 模式
  同 app 同主题）；bg-accent 对齐专项（D1，任选 2 个在用示例）。
- **门禁**：日常 `cargo t` + 改 VM 渲染面 `cargo tv` + fold 前 `cargo tf`；
  auto-man/auto 显式 `-p` 跑。

## 7. 验收标准

- **AC-01** theme{} 声明：extends 合成/局部覆盖/未知 token 编译错误/旧标量
  兼容，正负用例绿。
- **AC-02** 热切换：≥3 内置主题切换，token 化区域 100% 跟随——双端截图
  对拍通过；切换不重启进程（VM）/不重建工程（vue applyTheme）。
- **AC-03** 兼容：`theme: "dark"|"light"`、`accent:` 5 预设、优先级链
  CLI>os-config>pac.at>内置 照常（既有 458/504/506 测试面零回归）。
- **AC-04** 配置面：settings 主题选择器 + `appearance.theme` 主题名持久化 +
  boot 读回 + 旧值映射。
- **AC-05** registry 双面：五主题每 token 双面一致断言绿；金样 value-pinned
  口径成文。
- **AC-06** 债收口：D1 双端 bg-accent 对齐对拍绿；D2 提亮统一 +10（vue 暗色
  accent 变化在案）；D3 E2 退役（函数+金样删，zinc 表留）；D5 两 CLI 生成器
  registry 化；D4 调查结论落档。
- **AC-07** `dark:` 门控泛化零回归（既有 dark: 测试面 + 518 SetTheme 链路）。
- **AC-08** 编辑器高亮随主题翻转（单测 + 截图对拍抽查）。

## 8. 执行步骤

- **T-01** 裁定前置（文档）：§10 两项产品裁定请用户确认（归一口径/默认值；
  D2 归一值）；D4 auto-os 调查启动。验证：裁定落档/调查结论行。
- **T-02** registry 双面统一：ThemeSpec 结构化 + 主题双面值迁移 +
  `hsl_str_to_rgb` + canonical CSS 渲染 + 双面一致性测试。验证：
  `cargo t plan601 && cargo t ui::style`。
- **T-03** theme{} 声明解析（pac.at 扩展，兼容标量）+ extends 合成 + 词表
  校验。验证：正负用例 `cargo t theme_decl`。
- **T-04** 活动主题状态升级：set_theme + epoch + `dark:` is_dark 泛化 +
  兼容写入口。验证：`cargo t plan601 && cargo tv`（VM 渲染面）。
- **T-05** session/storage/settings：SetTheme(name) 泛化 + appearance.theme
  值域 + 选择器。验证：session 面向用例 + 手动 settings 冒烟。
- **T-06** Vue applyTheme：脚手架 canonical index.css + host 注入 applyTheme +
  accent overlay 内聚。验证：脚手架测试 + `cargo test -p auto-man`。
- **T-07** D2 提亮归一 +10（vue TS）。验证：包含性测试更新 + 对拍样本。
- **T-08** D1 accent 投影：Color::Accent/OnAccent + 解析臂 + 对拍 2 示例。
  验证：`cargo t ui::style` + verifier 对拍。
- **T-09** D3/D5 收编：E2 退役 + cmd_tauri/cmd_vue registry 装配。
  验证：`cargo test -p auto`（plan571 互锁三处绿）。
- **T-10** V4 完整：code_editor 从 ResolvedTheme 派生。验证：
  `cargo t plan601` + 截图抽查。
- **T-11** charts-gallery 硬编码色 token 化重估迁移（主题切换示范面）。
  验证：三主题截图对拍。
- **T-12** 门禁收口：`cargo t` 全量 + `cargo tv` + fold 前 `cargo tf` +
  auto-man/auto 显式；复审交接。

## 9. 复审记录

（draft 交接：`stage: new | PLAN-601 r1 | outcome: pass——两项产品裁定
（§10-1/§10-2）待用户确认，T-01 即裁定落档步，确认后可转 work | next: work`）

## 10. 待澄清事项

1. **归一裁定（T-01a，推荐已给）**：zinc/scaffold/stella 双轨分叉——推荐
   「零变化优先」：五套 named themes 全保留、双轨默认不变（vue=scaffold 系、
   VM=stella）、归一=用户切换能力本身；若要求视觉归一（如 vue 默认翻
   stella）则本 plan 增一条 AC 与对拍面（vue 全示例截图基线再生）。
2. **D2 归一值（T-01b，推荐 +10）**：vue 暗色 accent 提亮 4%→10%（暗色下
   primary 更亮，与 VM/coral 校准一致）；或反向 +4（vue 现状为准，VM 侧改）。
3. per-app color context（2b）确认出栈至 RenderQueue 期/独立 plan（默认出栈）。
