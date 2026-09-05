---
plan_id: PLAN-552
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-app-curation
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-man]   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 8
---

# [PLAN-552] 桌面应用策展——desktop 字段 opt-in + 测试探针清退

## 变更摘要

虚拟桌面的 app 清单（boot `aggregate_scan`）目前对 `examples/ui` **无过滤
全收**（`renderer.rs` boot 注册表段，~10689 行），46 个条目全部上架桌面——
其中 10 个 Tier1/2 教学 demo、3 个前端克隆演示、8 个测试探针都与"操作系统
默认应用集"的定位冲突。ui-gallery（Plan 549）上线后，"画廊收录全部 demo"与
"桌面上架策展子集"已成为两个不同诉求，但两者共用同一扫描面。

本计划两层解耦：

1. **pac.at 增 `desktop:` 字段**（主根 opt-in / 外部根 opt-out）——注册表
   条目带 `desktop_visible`，boot 期把 **展示清单**（`registry_entries`，
   launcher/图标格/dock 消费）过滤为策展集，**启动解析器**（`app_resolver`）
   保留全量（自定义图标/按名启动不受策展限制）。
2. **8 个测试探针目录物理迁出** `examples/ui` → `examples/capability-tests/`
   （026–040 先例）——它们连画廊都不该收录（裸 id、无教程价值）。

用户级运行时隐藏（`shell.desktop.hidden`，右键移除图标）保持正交，不受影响。

## 目标

1. 桌面三消费面（桌面图标格 `inject_desktop_surface`、launcher 注入、
   dock `inject_dock_pinned`）只显示策展 app 集（C 档 20 个 + 外部根）。
2. 教学 demo（001–010 等）保留在 `examples/ui` 供画廊收录，但默认不上桌面
   ——**零改动达成**（opt-in 语义下不加字段即隐藏）。
3. `examples/ui` 回归 README 自己的"只放应用性质"契约：探针清退后画廊
   不再出现裸 id 条目。
4. 外部根（os-config、未来 jade-garden/auto-musk/auto-shell）**缺席字段即
   可见**（opt-out）——外部根本来就是显式注册进 OS 的 app。
5. `examples/ui` README 总览表回写"桌面可见性"列与空洞历史注记。

## 架构方案

```
boot（renderer.rs ~10689）
  full = aggregate_scan(apps_dir, extra_roots, ScanOptions::default())
       ├─ app_resolver ← full          // 按名启动能力全量保留
       └─ registry_entries ← full.filter(desktop_visible)   // 展示清单
                                    │
    ┌─────────────────────────────────┼──────────────────────┐
    ▼                                 ▼                      ▼
inject_desktop_surface        launcher 注册表快照      inject_dock_pinned
（__desktop_icons 注入）      （grid/palette 数据）    （icon 查表，缺省回退）
```

- **字段解析**（`app_registry.rs` `entry_for_dir`）：pac `desktop: "true"/"false"`
  （大小写不敏感，与 `window: "fit"` 同风格）；`AppRegistryEntry` 增
  `desktop_visible: bool`。
- **缺省语义按扫描根区分**：`scan_apps`（主根 examples/ui——混合目录）
  缺席 = false（**opt-in**，新教学 demo 不再自动污染桌面）；`scan_app_root`
  （外部自含根——显式注册的整 app）缺席 = true（**opt-out**）。实现：
  `entry_for_dir` 增 `default_visible: bool` 参数。
- **画廊生成器不受影响**：`auto-man/src/vue.rs` `generate_gallery_host`
  读同一 `scan_apps` 但不消费 `desktop_visible`——画廊仍收录全部 demo。

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

- **GOAL-009**（虚拟桌面与桌面 Shell）/ **GOAL-010**（示例应用轨道·AutoOS
  默认应用集）：本计划是两目标交点——桌面从"扫描什么就上架什么"转为策展制。
- 现状扫描面（2026-09-05 实测）：`examples/ui` 43 个可启动条目全部入注册表；
  boot 无 render 过滤（`ScanOptions::default()`）。
- 三档清单（用户 2026-09-05 讨论定案）：
  - **C 档·桌面保留（20）**：011 012 013 014 015 016 017 018 020 022 024
    025 026 027 028 029 030 038 041 045 → pac 加 `desktop: "true"`。
  - **B 档·留画廊不上桌面（14，零改动）**：001–010、019、021、023、043、044。
  - **A 档·探针迁出（8）**：`overlay-probe`、`p051-min-ta`、`p493-color-check`、
    `p507-tier-coverage`、`p515-scroll-overflow`、`p518-glass-sample`、
    `459-dual-app`、`042-two-inputs-child` → `examples/capability-tests/`。
- 已知硬引用（迁移时必改）：
  - `app_registry.rs` 测试 `scan_examples_ui_finds_at_least_27_apps`：显式
    断言 `459-dual-app` 回退条目存在；
  - `examples/ui_desktop.rs`：`include_str!` 内嵌 `459-dual-app/app.at` +
    直挂窗引用（~12/41 行）；
  - `auto-man/src/vue.rs` 画廊分类 if 链硬编码 `042` 前缀（02-components 臂）。
- 既有用户级机制保持不动：`shell.desktop.hidden`（右键移除，storage 持久）、
  `shell.desktop.icons`（自定义图标）——自定义图标按 id 经 **全量 resolver**
  解析，仍可把隐藏 app 钉上桌面（显式用户意图优先于策展默认）。

## 详细设计

### 1. 注册表字段（app_registry.rs）

```rust
pub struct AppRegistryEntry {
    ...
    /// PLAN-552：桌面展示可见性（pac `desktop:`；主根缺省 false=opt-in，
    /// 外部自含根缺省 true=opt-out）。仅过滤展示清单，不影响启动解析。
    pub desktop_visible: bool,
}
```

- `entry_for_dir(dir, id, opts, default_visible)`；`desktop` 字段显式值
  覆盖缺省：`"true"` → true，`"false"` → false，其他值按缺省（坏值静默
  回退，与 `window:` 字段容错风格一致）。
- `scan_apps` 传 `false`；`scan_app_root` 传 `true`。

### 2. boot 两分（renderer.rs boot 注册表段）

- `let full = aggregate_scan(...)` 后：
  `let curated: Vec<_> = full.iter().filter(|e| e.desktop_visible).cloned().collect();`
- `app_resolver` 闭包捕获 `full`（现状捕获 `entries` 的 clone——换捕获源即可）；
- `session.desktop.registry_entries = curated;`
- eprintln 日志改双计数：`"app registry: {full} entries ({curated} desktop-visible) from {dir}"`。
- `arm_boot_fit_windows` / `launcher_entry` 查找均在 curated 上（028 属 C 档，
  字段加齐后无回归）。

### 3. pac.at 批量加字段（C 档 20 目录）

每目录 pac.at 追加一行 `desktop: "true"`（027/028/029/030/045 已有
`category:` 行，追加在其后保持分组）。

### 4. 探针迁移（A 档 8 目录）

- `git mv examples/ui/<id> examples/capability-tests/<id>`（保留历史）。
- 同步修复三处硬引用（见需求分析）+ T4 全仓 grep 清余波。
- `examples/capability-tests/` 侧 README（如无则该目录总 README 补一行）。

## 测试设计

- **单元（app_registry.rs tests）**：
  - `desktop_field_parse_matrix`：临时根四象限——主根×(true/false/缺席)、
    外部根×(true/false/缺席) 的 `desktop_visible` 断言；
  - `scan_examples_ui_curation_set`：真实 `examples/ui` 扫描，断言
    `desktop_visible == true` 的 id 集 **恰好等于** C 档 20 id（多一个/
    少一个都 fail——防止今后新 demo 悄悄上架或 C 档掉字段）；
  - 既有 `scan_examples_ui_finds_at_least_27_apps`：删除 459 专项断言、
    计数改 `>= 34`（43 - 8 迁出 + 注记）；`render_filter_keeps_only_matching`
    不受影响。
- **局部回归**：`cargo t app_registry`；若 `cargo t plan503` 引用被迁目录则
  一并跑（T4 grep 定）。
- **手动冒烟**（worktree 实机，非门禁）：`cargo run --example ui_desktop`
  boot 日志双计数；桌面图标格与 launcher（Ctrl+Space）无 001–010/019/021/
  023/043/044；右键移除仍生效。
- **门禁分级**：Category B（局部 Rust 改动）——`cargo check -p auto-lang`
  + 局部 `cargo t`；不动编译器/VM 核心，不跑 `cargo tf`（reviewer 可裁定加）。

## 验收标准

1. `scan_examples_ui_curation_set` 绿：策展集恰为 C 档 20 id。
2. 桌面图标格/launcher grid/palette 无教学 demo、克隆演示与探针。
3. 8 个探针目录位于 `examples/capability-tests/`；全仓 grep 八个 id 仅剩
   capability-tests 路径与历史文档引用，无代码/测试悬空。
4. 外部根语义：`scan_app_root` fixture 缺席 `desktop` 字段 → visible=true
   （os-config 上架不受影响）。
5. `shell.desktop.hidden` 右键移除、`shell.desktop.icons` 自定义图标行为
   不回归（注入段 `inject_desktop_surface` 逻辑未动，仅上游清单变短）。
6. `cargo check -p auto-lang` 零警告；`cargo t app_registry` 绿。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T1 注册表字段与解析**
  `crates/auto-lang/src/ui/app_registry.rs`：`AppRegistryEntry` 增
  `desktop_visible: bool`；`entry_for_dir` 增 `default_visible` 参数并解析
  pac `desktop` 字段（"true"/"false" 大小写不敏感，坏值回退缺省）；
  `scan_apps` 传 false、`scan_app_root` 传 true。
  验证：`cargo check -p auto-lang`
- [ ] **T2 解析矩阵单元测试**
  同文件 tests：`desktop_field_parse_matrix`（临时根六格：主根/外部根 ×
  true/false/缺席）。
  验证：`cargo t app_registry`
- [ ] **T3 boot 两分**
  `crates/auto-lang/src/ui/iced/renderer.rs` boot 注册表段（`aggregate_scan`
  调用处）：resolver 捕获 full、`registry_entries` 赋 curated、eprintln 双计数。
  验证：`cargo check -p auto-lang`
- [ ] **T4 探针引用盘点**
  全仓 grep 八个 A 档 id（排除 node_modules/gen/archive）：`grep -rn
  "overlay-probe\|p051-min-ta\|p493-color-check\|p507-tier-coverage\|
  p515-scroll-overflow\|p518-glass-sample\|459-dual-app\|042-two-inputs-child"
  --include="*.rs" --include="*.toml" --include="*.md" --include="*.at"`，
  产出引用清单贴回本节。
  验证：清单覆盖 app_registry tests / ui_desktop.rs / vue.rs 三已知点。
- [ ] **T5 探针迁移与引用修复**
  `git mv` 八目录 → `examples/capability-tests/`；修
  `app_registry.rs::scan_examples_ui_finds_at_least_27_apps`（删 459 断言、
  计数 ≥34 + 迁移注记）；修 `examples/ui_desktop.rs`（include_str 路径
  `../../..` 深度 + 注释）；修 `auto-man/src/vue.rs` 分类链（摘除 042 前缀）；
  T4 清单内其余引用逐一处理。
  验证：`cargo check -p auto-lang && cargo t app_registry`
- [ ] **T6 C 档 pac 加字段**
  20 个 pac.at（011–018 除 019、020、022、024–030、038、041、045）各追加
  `desktop: "true"`。
  验证：`cd examples/ui && for d in 011-calculator 012-stopwatch 013-todo
  014-weather 015-notes 016-calendar 017-chat 018-book-reader 020-music-player
  022-kanban 024-charts 025-dashboard 026-database 027-file-manager
  028-launcher 029-photo-gallery 030-video-player 038-minesweeper
  041-auto-edit 045-desktop-settings; do grep -L 'desktop:' $d/pac.at; done`
  （输出为空 = 齐）
- [ ] **T7 策展集合断言测试**
  `app_registry.rs` tests 增 `scan_examples_ui_curation_set`（C 档 20 id
  恰等断言）。
  验证：`cargo t app_registry`
- [ ] **T8 文档回写与终检**
  `examples/ui/README.md`：总览表加"桌面"列（✓=C 档）；编号说明节补
  探针迁移历史注记（8 目录去向 capability-tests）；`examples/
  capability-tests/` README 补目录行（如存在）。终检：
  `cargo check -p auto-lang`（零警告）+ `cargo t app_registry` +
  `cargo t plan503`（若 T4 清单命中）。
  验证：命令全绿 + README diff 人工过目

## 复审记录

## 待澄清事项

1. **后续 backlog（本计划不做，登记备查）**：回收站（027+桌面集成）、
   App Store（auto-man GUI）、截图工具（需宿主抓屏）、压缩文件管理器、
   记事本 lite、通讯录——来源 2026-09-05 桌面应用盘点讨论。
2. p518-glass-sample 若被 Plan 518 文档当作"样张资产"路径引用（docs/plans/
   518 系），迁移后文档链接是否需要留重定向注记——T4 grep 结果定。
3. 真画布原语（笔迹/pointer 路径 canvas）另立设计文档，与本计划无关
   （PLAN-553 像素形态绕开）。
