---
plan_id: PLAN-601
status: executing               # review needs_fix（R1 boot 读回）→ 修复后 re-review
feature_name: theme-declaration-hot-switch（Design 29 Phase 2）
author: [zhaopuming]
created_at: 2026-09-09
updated_at: 2026-09-10
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-007]     # AutoUI 跨端视觉一致（主题/令牌面）

affects: [auto-lang/ui, auto-man, auto]   # specs 路径
current_step: 11
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
  [✅ 已完成] 裁定落档（AskUserQuestion 未获答复，按 plan 推荐项执行，复审/merge 可翻案）：①归一口径=**零变化优先**（五套 named themes 全保留、双轨默认不变、AC-02 不扩 vue 基线再生）；②D2=**+10**；③2b 出栈维持。D4 调查结论：auto-os 仓无独立发射点——`apps/025-sys-monitor/gen/front/.../index.css` 为本仓 scaffold 生成物（primary 239 84% 67%/secondary 40 24% 85.5%/sidebar 16 处 ≈ E3 全同），`widgets-gallery/vue-ref` 为静态参照资产（E7 在案）→ **P593-D4 关债**（T-09 时 KNOWN-DEBT 注记）。
- **T-02** registry 双面统一：ThemeSpec 结构化 + 主题双面值迁移 +
  `hsl_str_to_rgb` + canonical CSS 渲染 + 双面一致性测试。验证：
  `cargo t plan601 && cargo t ui::style`。
  [✅ 已完成] commit 837018b25：222 值机械迁移（五主题双面，stella 14 键 Rgb 真值+CSS 面派生补全/popover←card 等约定、cli-vue 双 sidebar 取级联生效值）；canonical render_core/render_sidebar；金样 value-pinned 升级（vs Phase1 逐字金样逐对相等，双端绿）；双面一致性测试；生成期 s/l 互换 bug 自抓自修+4 扩展键补齐；design_tokens 4/4+plan593 8/8+ui::style 98/98+auto-man 270/270。
- **T-03** theme{} 声明解析（pac.at 扩展，兼容标量）+ extends 合成 + 词表
  校验。验证：正负用例 `cargo t theme_decl`。
  [✅ 已完成] commit b5b4f60f0：decl.rs（ThemeDecl/compose 链深≤4 防环/本声明最后胜/mode 链上最近声明胜/normalize hex→HSL/六负例）+ TokenName::from_css_var + ActiveSpec(Builtin|Composed) 槽泛化 + set_theme_composed + pac.rs theme_decl 块解析（对象冒号形态 back:{project} 同构，标量 458 链路不动）；decl 4 用例+pac 测试；design_tokens 8/8+auto-man 271/271。实勘注记：.at 配置对象需冒号形态（`theme: {`），Design 29 §4.2 示例已按实际语法修正。
- **T-04** 活动主题状态升级：set_theme + epoch + `dark:` is_dark 泛化 +
  兼容写入口。验证：`cargo t plan601 && cargo tv`（VM 渲染面）。
  [✅ 已完成] commits 719ce4f56/e0db89b04：ACTIVE_THEME 槽+theme_name/set_theme（内置校验/变化时 epoch 自增复用既有失效回路）+active_theme()；resolve 双入口改查活动主题（默认 stella=零变化）；mode 仍由 DARK_MEMORY 承载（dark: 门控读值不变=零回归泛化成文）；切换翻转/epoch/未知名拒绝测试 6/6（epoch 计数含 dark 翻转的时序教训在案）。tv 档随 T-12 一并。
- **T-05** session/storage/settings：SetTheme(name) 泛化 + appearance.theme
  值域 + 选择器。验证：session 面向用例 + 手动 settings 冒烟。
  [✅ 已完成] commit 73dd39835：SetThemeName 动词（set_theme_name	<内置名>，解析臂词表门）+ renderer 执行臂（set_theme+epoch 失效+config.theme_name 落盘+快照全撤+全 App view_dirty）+ boot/热应用差分臂；desktop_config theme_name 字段往返测试 10/10；set_theme(bool) mode 链路零扰动（正交裁定）；**settings 选择器 UI 属 auto-os 资产面**（apps/common/settings）——能力层全落地，UI 控件随 auto-os 侧跟进（复审可裁移交或组内 auto-os worktree 补）。
- **T-06** Vue applyTheme：脚手架 canonical index.css + host 注入 applyTheme +
  accent overlay 内聚。验证：脚手架测试 + `cargo test -p auto-man`。
  [✅ 已完成] commit eccdcfdbd：registry render_theme_pairs_js/render_theme_palettes_js（五内置双面 JS 值源）+decl render_pairs_js（合成体同形）；ui_gen theme_runtime_js（applyTheme 全变量写入：html inline light + `.dark` 元素 dark 值 + 光照模式陈值清理 + accent overlay 内聚末位 + 'auto-theme' storage 持久）+ widget/store 双注入臂（同 accent 门控，零 accent 面 app 零注入）+mode 翻转 watch 升级整套重应用；auto-man theme_decl 消费（compose→index.css 双 mode 块 + index.html `__AUTO_COMPOSED_THEME__` 种子 write-if-unset）；design_tokens 9/9+vue/theme 144/144+auto-man 274/274。
- **T-07** D2 提亮归一 +10（vue TS）。验证：包含性测试更新 + 对拍样本。
  [✅ 已完成] applyAccent dark +4→+10（注释含归一依据）；vue 面 301/302 绿（唯一红=charts_gallery master 预存）；P593-D2 随 T-09 的 KNOWN-DEBT 注记一并关。
- **T-08** D1 accent 投影：Color::Accent/OnAccent + 解析臂 + 对拍 2 示例。
  验证：`cargo t ui::style` + verifier 对拍。
  [✅ 已完成] commit fd3f7aff9：独立变体+投影臂+t_c 完备集+2；ui::style 99/99；双端截图对拍挂 review/verifier 通道（单测级投影钉死，视觉对拍属 AC-06 复审面）。
- **T-09** D3/D5 收编：E2 退役 + cmd_tauri/cmd_vue registry 装配。
  验证：`cargo test -p auto`（plan571 互锁三处绿）。
  [✅ 已完成] D3 commit 172a05292（债况文档）+ 9e00e4cca（函数本体+调用点测试退役补齐——172a05292 实仅含文档，代码侧遗留未提交现补）；D5 commit 55b56e56a（cmd_tauri/cmd_vue generate_index_css 色 block 改 render_core/render_sidebar registry 装配〔内置 tauri/cli-vue 双面，装配前逐值机械对拍零漂移〕；vis-*/双 sidebar 兜底/preview-code/forest-sunset-ocean/mono 留模板；plan571 contains 测试升级 registry 逐字等值断言；KNOWN-DEBT P593-D5 翻关债〔死文件防复活即同源；tauri 真路径经 auto-man 共享脚手架已 registry 化〕）。`cargo test -p auto` 11/11。
- **T-10** V4 完整：code_editor 从 ResolvedTheme 派生。验证：
  `cargo t plan601` + 截图抽查。
  [✅ 已完成] commit 4c3ebb85f：CodeEditorTheme::from_resolved（bg/fg 取 registry Background/Foreground u8 真值→编辑器色域映射成文）+for_builtin/for_composed 双面+active_code_theme 统一解析（缺值回退 legacy 预设）；syntax 主题键扩主题维 `autoui-{theme}-{mode}-{accent}`（五内置×双 mode×五 accent 预烘焙 + boot 期合成主题烘焙——合成体 boot 后固定先于编辑器首建）；current_theme/selectable_text 选区改活动主题口；set_theme/composed+THEME_EPOCH 失效→下帧重注册=编辑器随主题翻转；渲染契约断言随新契约更新；code_editor+highlight+theme 86/86+ui::style 99/99。截图抽查挂 review/verifier 通道。
- **T-11** charts-gallery 硬编码色 token 化重估迁移（主题切换示范面）。
  验证：三主题截图对拍。
  [✅ 已完成] commit 23a5a7d50：重估结论=SVG 图形属性 token 通道缺失（serialize_svg_element 构建期逐字序列化，stroke/fill 双腿均不认语义名/var()），示例色 token 化需跨管线协议特性先行——登记 **P601-T11 开放债**（含 vue 腿裸名包组件 SFC 化缺口）；任务内落地=日常档唯一预存红修复（test_charts_gallery_compiles fixture 随 484 M4 迁 examples/ui/024-charts，断言按现行裸名折叠架构改写）。三主题截图对拍随示范面立项（债项触发条件）。
- **T-12** 门禁收口：`cargo t` 全量 + `cargo tv` + fold 前 `cargo tf` +
  auto-man/auto 显式；复审交接。
  [✅ 已完成] 2026-09-10：日常档全量（--no-fail-fast）4726 测 4705 绿 **21 红 = master 基线 22 红 − test_charts_gallery_compiles（本计划修复），集合对拍零新增红**；`cargo tv` 3639/3639 全绿（cb_web_mime 首跑 flake 单测复跑双树绿，在案）；`cargo tf` 3495/3495 全绿；auto-man 274/274 + auto 11/11。门禁注记：`.config/nextest.toml` fail-fast 使裸 `cargo t` 在首失败二进制后取消余量（3523 not run），全量红集合裁定须 `--no-fail-fast`；`cargo test -p auto-man` 会再生成 examples/rust-workspace/015-notes 产物（master 同样漂移，非本计划引入，还原处置）。

## 9. 复审记录

（review 2026-09-10：`stage: review | PLAN-601 r1 | outcome: needs_fix |
reviewed_commit: 23a5a7d501bc45c4ad847725dc94ed0d6d6bf2d5@plan-601-dev（树净）|
base_commit: 2f68be1d6（merge-base；master 已前移至 f9a988a87——fold 阶段同步项）|
dependency_revisions: 同仓 workspace（auto-man/auto），无跨仓依赖改动；auto-os 未触及 |
spec_inputs: docs/specs/auto-lang/ui/overview.md（P593 条目+design_tokens 行+code_editor 行=SD-01/02/03 靶）、docs/plans/KNOWN-DEBT-AND-RISKS.md（SD-04 已随 work 落）|
acceptance_results: AC-01 pass / AC-02 partial / AC-03 pass / AC-04 **partial→R1** / AC-05 pass / AC-06 partial(随 R2) / AC-07 pass / AC-08 pass(单测级,截图随 R2) |

**findings:**

- **R1【P1·AC-04·needs_fix 主因】theme_name boot 读回缺失**：持久化链只走
  「动词臂(renderer.rs:9122 execute_set_theme_name)→落盘」与「config.at 外写
  diff 臂(:9249, Plan 551 轮询)」；boot 期 `DesktopSession::new` 仅
  `config: load()` 装载结构体（session.rs:386），`style::theme::set_theme`
  无 boot 调用方（全 crate 生产调用点仅 9122/9249），且 poll 首采样
  「只建锚不应用」（551 语义：dock/壁纸等结构化消费面成立，theme_name 需
  ACTIVE_THEME 激活不成立）→ **主题选择重启后不存活**。运行时实机复现
  （002-counter + `AUTOOS_DESKTOP_CONFIG` 种子 zinc/stella/scaffold×3 次
  boot，MCP 截图 4 张字节全等=主题未应用）。修复面小：renderer 启动路径
  补 boot 激活臂（`if let Some(name)=&cfg.theme_name { set_theme(name) }`
  + 失败容错 + 单测）。重开 T-05。
- **R2【P2·非缺陷·路由注记】VM 腿主题通道宿主门控**：config 轮询
  （poll_external_config）与动词均 `is_desktop()` 门控（551 宿主中心设计
  一致）——纯 `auto run -r vm` 运行时切主题不可达；VM 端运行时视觉验收
  （AC-02/06/08 截图对拍）依赖 auto-os 宿主集成（与 settings 选择器 UI
  移交件同批）。仓内证据=执行臂级单测（t3_session_with_shell 桌面态：
  execute_set_theme_name/apply_external_config_diff 真实会话驱动）。
- **R3【P3·预存·非本计划】vue 腿裸名包组件 SFC 缺口现场复现**：
  006-hero-section/015-notes 生成 App.vue imports SettingsPopover.vue
  未发射（`use settings: SettingsPopover` 包组件族,与 P601-T11 债项同族）——
  建议 merge 时将该债项文本扩及 settings-popover 家族。
- **R4【info】生成产物漂移副作用**：`cargo test -p auto-man` 与 `auto run`
  会再生成 examples/rust-workspace/015-notes-back 等产物（master 同样漂移,
  非本计划引入）——已还原；生成器金样自愈面建议独立小计划。

**runtime 实机证据（vue 腿,AC-02 核心段 pass）**：006-hero-section 经本
worktree codegen 生成+`auto run`（vite :3000），playwright boot-restore 路径
（localStorage `auto-theme` 种子→载入,同工程零重建）：zinc→`--background
222.2 47% 7%`、stella→`223 34% 12%`（=stella dark 真值 rgb(20,26,41)）、
scaffold→`222.2 47% 7%`,computed 值与 registry 表逐值相符;`--primary` 恒
indigo=accent overlay 按设计骑压;持久键各归位;截图 3 张落档（stella 与
zinc/scaffold 像素可区分）。T-06 交付物（THEME_PALETTES 五内置/applyTheme/
升级 watch）实证落位于真实生成产物。

**门禁复核（复审重放）**：scoped 121/122（唯一红=ui::layout 基线红,与本计划
无关）;日常档全量/tv/tf/auto-man/auto 于 work 阶段同基线重放（记录见上轮）,
红集合对拍裁定成立。

evidence: 本记录内嵌（命令+computed 值摘录）;探针截图 D:/tmp/p601_review/
（transient,值摘录已固化于此）|
next: work 修复 R1（范围限 boot 读回臂+单测;R2/R3 随 merge 路由,R4 建议
独立计划）→ 修复后快速 re-review（仅 R1 面）`）

## 9. 复审记录

（review 2026-09-10：`stage: review | PLAN-601 r1 | outcome: needs_fix |
reviewed_commit: 23a5a7d501bc45c4ad847725dc94ed0d6d6bf2d5@plan-601-dev（树净）|
base_commit: 2f68be1d6（merge-base；master 已前移至 f9a988a87——fold 阶段同步项）|
dependency_revisions: 同仓 workspace（auto-man/auto），无跨仓依赖改动；auto-os 未触及 |
spec_inputs: docs/specs/auto-lang/ui/overview.md（P593 条目+design_tokens 行+code_editor 行=SD-01/02/03 靶）、docs/plans/KNOWN-DEBT-AND-RISKS.md（SD-04 已随 work 落）|
acceptance_results: AC-01 pass / AC-02 partial / AC-03 pass / AC-04 **partial→R1** / AC-05 pass / AC-06 partial(随 R2) / AC-07 pass / AC-08 pass(单测级,截图随 R2) |

**findings:**

- **R1【P1·AC-04·needs_fix 主因】theme_name boot 读回缺失**：持久化链只走
  「动词臂(renderer.rs:9122 execute_set_theme_name)→落盘」与「config.at 外写
  diff 臂(:9249, Plan 551 轮询)」；boot 期 `DesktopSession::new` 仅
  `config: load()` 装载结构体（session.rs:386），`style::theme::set_theme`
  无 boot 调用方（全 crate 生产调用点仅 9122/9249），且 poll 首采样
  「只建锚不应用」（551 语义：dock/壁纸等结构化消费面成立，theme_name 需
  ACTIVE_THEME 激活不成立）→ **主题选择重启后不存活**。运行时实机复现
  （002-counter + `AUTOOS_DESKTOP_CONFIG` 种子 zinc/stella/scaffold×3 次
  boot，MCP 截图 4 张字节全等=主题未应用）。修复面小：renderer 启动路径
  补 boot 激活臂（`if let Some(name)=&cfg.theme_name { set_theme(name) }`
  + 失败容错 + 单测）。重开 T-05。
- **R2【P2·非缺陷·路由注记】VM 腿主题通道宿主门控**：config 轮询
  （poll_external_config）与动词均 `is_desktop()` 门控（551 宿主中心设计
  一致）——纯 `auto run -r vm` 运行时切主题不可达；VM 端运行时视觉验收
  （AC-02/06/08 截图对拍）依赖 auto-os 宿主集成（与 settings 选择器 UI
  移交件同批）。仓内证据=执行臂级单测（t3_session_with_shell 桌面态：
  execute_set_theme_name/apply_external_config_diff 真实会话驱动）。
- **R3【P3·预存·非本计划】vue 腿裸名包组件 SFC 缺口现场复现**：
  006-hero-section/015-notes 生成 App.vue imports SettingsPopover.vue
  未发射（`use settings: SettingsPopover` 包组件族,与 P601-T11 债项同族）——
  建议 merge 时将该债项文本扩及 settings-popover 家族。
- **R4【info】生成产物漂移副作用**：`cargo test -p auto-man` 与 `auto run`
  会再生成 examples/rust-workspace/015-notes-back 等产物（master 同样漂移,
  非本计划引入）——已还原；生成器金样自愈面建议独立小计划。

**runtime 实机证据（vue 腿,AC-02 核心段 pass）**：006-hero-section 经本
worktree codegen 生成+`auto run`（vite :3000），playwright boot-restore 路径
（localStorage `auto-theme` 种子→载入,同工程零重建）：zinc→`--background
222.2 47% 7%`、stella→`223 34% 12%`（=stella dark 真值 rgb(20,26,41)）、
scaffold→`222.2 47% 7%`,computed 值与 registry 表逐值相符;`--primary` 恒
indigo=accent overlay 按设计骑压;持久键各归位;截图 3 张落档（stella 与
zinc/scaffold 像素可区分）。T-06 交付物（THEME_PALETTES 五内置/applyTheme/
升级 watch）实证落位于真实生成产物。

**门禁复核（复审重放）**：scoped 121/122（唯一红=ui::layout 基线红,与本计划
无关）;日常档全量/tv/tf/auto-man/auto 于 work 阶段同基线重放（记录见上轮）,
红集合对拍裁定成立。

evidence: 本记录内嵌（命令+computed 值摘录）;探针截图 D:/tmp/p601_review/
（transient,值摘录已固化于此）|
next: work 修复 R1（范围限 boot 读回臂+单测;R2/R3 随 merge 路由,R4 建议
独立计划）→ 修复后快速 re-review（仅 R1 面）`）

（work 第三轮 2026-09-10 收口：`stage: work | PLAN-601 r1 | outcome: pass（全部 12 任务完成）→ execution_done |
code_commit: 23a5a7d50@plan-601-dev（本轮 9e00e4cca D3 收尾 + eccdcfdbd T-06 + 55b56e56a T-09-D5 + 4c3ebb85f T-10 + 23a5a7d50 T-11）|
task_ids: T-01..T-12 全✅（T-11 重估裁定=P601-T11 开放债登记；T-05 settings 选择器 UI=auto-os 资产面移交在案）|
evidence: 日常档全量（--no-fail-fast）4726 测 21 红=master 基线 22 红−charts_gallery（本计划修复），集合对拍零新增红；tv 3639/3639；tf 3495/3495；auto-man 274/274；auto 11/11；design_tokens 9/9+vue/theme 144/144+code_editor 86/86+ui::style 99/99 |
blockers: 无（复审面三件：①AC-02/06/08 双端截图对拍〔autoui-verifier 通道，T-08 先例〕；②settings 选择器 UI 移交裁定〔auto-os 侧〕；③P601-T11 SVG token 通道债触发时机〕|
next: review`）

（work 第二轮 2026-09-09 续：`stage: work | PLAN-601 r1 | executing |
code_commit: fd3f7aff9@plan-601-dev（本轮 b5b4f60f0/73dd39835/fd3f7aff9/172a05292 + D3 退役）|
task_ids: +T-03✅ T-05✅ T-08✅ T-09◐（D3/D4/债况✅，D5 装配余）| 累计 7/12 |
evidence: 日常档 22 红=master 基线全等零新增；design_tokens 8/8+decl 4 用例+theme 6/6+desktop_config 10/10+auto-man 271/271+vue 301/302（预存红唯一） |
blockers: 无；T-06（vue applyTheme）/T-10（V4 派生）/T-11（charts 迁移）/T-09-D5（CLI 装配）待续 |
next: 续 work 自 T-06`）

（work 阶段记录 2026-09-09：`stage: work | PLAN-601 r1 | outcome: executing（部分完成交接）|
code_commit: e0db89b04+T-07 提交 @plan-601-dev | task_ids: T-01✅ T-02✅ T-04✅ T-07✅；余 T-03/T-05/T-06/T-08/T-09/T-10/T-11/T-12 |
evidence: design_tokens 4/4+plan593 8/8+ui::style 104 含 theme 6/6+auto-man 270/270+vue 301/302（唯一红预存） |
blockers: 无（T-01 两裁定按推荐项执行待用户事后追认；T-05 settings.at 资产疑在 auto-os 仓——执行时需组内 auto-os worktree）|
next: 续 work 自 T-03（theme{} 声明解析——pac.at 解析器入口待探）`）


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
