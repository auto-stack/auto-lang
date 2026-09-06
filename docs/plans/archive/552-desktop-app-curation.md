---
plan_id: PLAN-552
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-app-curation
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05   # 2026-09-05 execution start → execution_done → reviewed（复审 PASS，tf 3424/3425 唯一红=master 存量）

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: R10 应用注册表增 desktop 策展语义——AppRegistryEntry.desktop_visible（pac `desktop:` 字段，主根缺省 false=opt-in / 外部自含根缺省 true=opt-out，坏值回退）+ boot 两分（app_resolver 捕获全量、registry_entries=curated、双计数日志）"
  - "docs/specs/auto-lang/ui/overview.md: 探针语料面迁移——8 测试探针自 examples/ui 迁 examples/capability-tests（stage3 e2e example_source 双根解析；ui_desktop/ui_dual_app include_str 改指 capability-tests；scan 计数断言 27→34 口径）"
  - "docs/specs/auto-man/project.md: 画廊生成器——02-components 分类臂摘除 042 前缀；画廊收录面 43→35（探针清退，画廊不再出现裸 id 条目）"
new_spec_components: []
touched_goals:
  - "GOAL-009: 桌面上架从'扫描即上架'转策展制——图标格/launcher/dock 三消费面只见 desktop_visible 策展集，按名启动与自定义图标启动解析保持全量"
  - "GOAL-010: examples/ui 回归应用契约（8 探针迁 capability-tests，画廊收录 43→35）+ C 档 19 策展子集（desktop: \"true\"；045 已由 551 退役不计）+ README 桌面可见性列与迁移注记"

affects: [auto-lang/ui, auto-man]   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 8
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

- [x] **T1 注册表字段与解析** [✅ 已完成] `desktop_visible` 字段 + `entry_for_dir(default_visible)` 解析（scan_apps→false / scan_app_root→true，"true"/"false" 大小写不敏感坏值回退）+ renderer.rs 5 处测试字面量补字段；`cargo check -p auto-lang` 通过（44s，161 存量警告均为未触碰文件）
  `crates/auto-lang/src/ui/app_registry.rs`：`AppRegistryEntry` 增
  `desktop_visible: bool`；`entry_for_dir` 增 `default_visible` 参数并解析
  pac `desktop` 字段（"true"/"false" 大小写不敏感，坏值回退缺省）；
  `scan_apps` 传 false、`scan_app_root` 传 true。
  验证：`cargo check -p auto-lang`
- [x] **T2 解析矩阵单元测试** [✅ 已完成] `desktop_field_parse_matrix`（主根/外部根 × true/false/缺席 六格 + 坏值/大小写两臂）PASS；`cargo t app_registry` 12/12 绿
  同文件 tests：`desktop_field_parse_matrix`（临时根六格：主根/外部根 ×
  true/false/缺席）。
  验证：`cargo t app_registry`
- [x] **T3 boot 两分** [✅ 已完成] resolver 捕获源换 `full`（原 `entries.clone()`）、`registry_entries = curated`（`desktop_visible` 过滤 + PLAN-552 注记）、eprintln 双计数 `{} entries ({} desktop-visible)`；`cargo check -p auto-lang` 通过
  `crates/auto-lang/src/ui/iced/renderer.rs` boot 注册表段（`aggregate_scan`
  调用处）：resolver 捕获 full、`registry_entries` 赋 curated、eprintln 双计数。
  验证：`cargo check -p auto-lang`
- [x] **T4 探针引用盘点** [✅ 已完成] 全仓 grep（排除 node_modules/gen/archive/target/历史计划档）+ 042 前缀专项补查，清单如下：
  **代码/测试级（T5 必改 9 点）**：
  1. `crates/auto-lang/src/ui/app_registry.rs` tests `scan_examples_ui_finds_at_least_27_apps`（459 断言+计数）
  2. 同文件 `launch_three_real_apps_via_registry_resolver`（第三 id=459，**计划清单外新发现**——迁移后不在 examples/ui 扫描面）
  3. `crates/auto-lang/examples/ui_desktop.rs:12,41`（include_str + 直挂 source_path）
  4. `crates/auto-lang/examples/ui_dual_app.rs:18`（include_str，**新发现**）
  5. `crates/auto-lang/src/ui/desktop_protocol/stage3.rs:296-315`（T3_EXAMPLES 含 p507/p515 + example_source 路径模板硬编码 examples/ui，**新发现**；530+/679+ 的 app_name 断言串随路径修复自动有效）
  6. `crates/auto-man/src/vue.rs:3636`（02-components 臂 `starts_with("042")`，全 id grep 不命中——前缀硬编码，专项补查命中）
  7. `.agents/skills/autoui-verifier/SKILL.md:140`（p518 样张路径）
  8. `docs/plans/KNOWN-DEBT-AND-RISKS.md:83`（P518 条目样张路径——活台账，随迁更新）
  9. 探针目录自身文件（pac.at/app.at/README 随 git mv 整体迁移）
  **注释/合成数据/历史文档级（保留不动）**：app_registry.rs:7,291（459 形态命名注释）；plan536_t1_reactive_probe_tests.rs:477 与 test/ui/plan536_absolute（p051 先例注释）；renderer.rs:23630（042 同构注释）、:19740-19741（boot_entry_matches 合成路径测试数据，非真实路径）；docs/plans/*、docs/specs/*（历史叙述）；renderer.rs 头部文档（459 panic 探针运行说明随 ui_desktop.rs 更新）
  验证：清单覆盖计划三已知点（app_registry tests / ui_desktop.rs / vue.rs）✓ 且多出 4 个计划外代码级引用点
- [x] **T5 探针迁移与引用修复** [✅ 已完成] `git mv` 八目录（R 状态历史保留）；修 `scan_examples_ui_finds_at_least_27_apps`（删 459 断言、计数 ≥34 + 迁移注记）、`launch_three_real_apps_via_registry_resolver`（第三 App 换 041-auto-edit，title "AutoEdit" 断言）；修 `ui_desktop.rs`/`ui_dual_app.rs` include_str+source_path+注释；修 `stage3.rs` `example_source` 双根解析（examples/ui → capability-tests 兜底）；修 `vue.rs` 02-components 臂摘除 `starts_with("042")`；SKILL.md/KNOWN-DEBT p518 样张路径更新；验证：`cargo check -p auto-lang -p auto-man` 零 error + `--example ui_desktop/ui_dual_app`（ui-iced）编译过 + `cargo t app_registry` 12/12 绿
  `git mv` 八目录 → `examples/capability-tests/`；修
  `app_registry.rs::scan_examples_ui_finds_at_least_27_apps`（删 459 断言、
  计数 ≥34 + 迁移注记）；修 `examples/ui_desktop.rs`（include_str 路径
  `../../..` 深度 + 注释）；修 `auto-man/src/vue.rs` 分类链（摘除 042 前缀）；
  T4 清单内其余引用逐一处理。
  验证：`cargo check -p auto-lang && cargo t app_registry`
- [x] **T6 C 档 pac 加字段** [✅ 已完成] 19 个 pac.at 追加 `desktop: "true"`（027/028/029/030 带 `category:` 的插其后，其余 EOF 追加）；`grep -L` 复查输出为空、带字段 pac 总数恰 19。**执行偏差**：`045-desktop-settings` 已被 Plan 551 T7 退役（commit cfcd534ff 删目录，设置改走 os-config daemon 单源；本计划起草时 045 尚在故 C 档记 20）——无法给已删目录加字段，C 档 20→**19**，T7 断言集相应 19 元。详见待澄清④
- [x] **T7 策展集合断言测试** [✅ 已完成] `scan_examples_ui_curation_set`——`desktop_visible` id 集恰等断言（19 id 双向 fail：多一=新 demo 悄悄上架、少一=C 档掉字段；注释含 045 退役注记）；`cargo t app_registry` 13/13 绿
  `app_registry.rs` tests 增 `scan_examples_ui_curation_set`（C 档 20 id
  恰等断言）。
  验证：`cargo t app_registry`
- [x] **T8 文档回写与终检** [✅ 已完成] `examples/ui/README.md`：总览表加"桌面"列（✓=C 档 19）+ 补 030/044 两缺行（030 属 C 档原表漏行）+ 编号说明节探针迁移注记（8 目录去向 + 042 空洞恢复可填）+ 桌面可见性语义段（opt-in/opt-out/resolver 全量/hidden 正交）；`examples/capability-tests/README.md` 补 Test probes 节（8 探针角色/来历表 + 消费面注记）。终检：`cargo check -p auto-lang -p auto-man` 通过且触及文件零警告归因（renderer 命中行 122/980/6123 均为预存、ui_gen/vue.rs 非本文件）+ `cargo t app_registry` 13/13 绿 + `cargo t plan503` 跳过（T4 清单未命中该套件）；README diff 人工过目通过；验收③ grep 复查仅剩历史文档与一处合成测试数据（boot_entry_matches 假路径，T4 裁定保留）
  `examples/ui/README.md`：总览表加"桌面"列（✓=C 档）；编号说明节补
  探针迁移历史注记（8 目录去向 capability-tests）；`examples/
  capability-tests/` README 补目录行（如存在）。终检：
  `cargo check -p auto-lang`（零警告）+ `cargo t app_registry` +
  `cargo t plan503`（若 T4 清单命中）。
  验证：命令全绿 + README diff 人工过目

## 复审记录

**复审人**：ZCode（/auto-plan:review）· **时间**：2026-09-05 · **worktree**：`.wt/lang-552/auto-lang`（plan-552-dev，4 commits：2f7c80068/6e63db44e/fec86f841/债务登记）· **裁定**：**PASS → reviewed**

**逐条验收重验**（verify, don't trust——全部在 worktree 重跑/重读代码）：

1. **策展集恰等断言** ✅ PASS（含偏差裁定）——`scan_examples_ui_curation_set` 断言集为 C 档 **19** id（计划原文 20）：`045-desktop-settings` 已被 Plan 551 T7 退役（cfcd534ff 删目录，起草时点差），无法给不存在目录加字段。偏差非执行遗漏，已记待澄清④；测试在 tf 全量中绿（双向恰等断言照常生效）。
2. **三消费面只见策展集** ✅ PASS（代码证据）——launcher 快照 `renderer.rs:10210`、图标格 `inject_desktop_surface`（10031 reg 查表）、dock `inject_dock_pinned`（9911）均读 `registry_entries`，boot 赋 curated（10858）；resolver 捕获 full（10811-10842）。实机冒烟（boot 双计数日志/Ctrl+Space 目检）按计划口径为**非门禁**项，留用户实机过目。
3. **8 目录迁移 + 引用清干净** ✅ PASS——git R 状态历史保留；全仓 grep 复查：代码/测试级零悬空，残余仅历史文档（docs/plans、docs/specs）与 `renderer.rs:19740` 合成路径测试数据（boot_entry_matches 假路径，非真实引用）。
4. **外部根 opt-out 语义** ✅ PASS——`desktop_field_parse_matrix` 外部根×缺席=true 断言绿；`scan_app_root` 公开签名未变，`tests/osconfig_integration.rs` 无 AppRegistryEntry 字面量（编译无影响）。
5. **hidden/icons 用户级机制不回归** ✅ PASS（含边界注记）——注入段逻辑零改动（diff 仅 boot 段+5 测试字面量）；`desktop_surface_merge_dedupe_and_injection` 等 tf 全量绿。边界：customs 槽位的非策展 id icon/label 回退 app-window/裸 id（计划注记"经全量 resolver 解析"仅启动成立、元数据查表不走 resolver——代码为准）→ 已登记 KNOWN-DEBT P552 行。
6. **check 零警告 + 局部测试绿** ✅ PASS——`cargo check -p auto-lang -p auto-man` 通过，触及文件（app_registry/stage3/auto-man vue/ui_desktop/ui_dual_app + renderer 编辑区）警告归因为零（renderer 命中行 122/980/6123 均预存）；`cargo t app_registry` 13/13。

**全量门禁（review 唯一一次 tf）**：`cargo tf` 3425 跑 **3424 绿 / 1 红**；`--no-fail-fast` 补跑确认唯一红为 `ui_gen::vue::tests::test_charts_gallery_compiles`——**在 master（5ff92f364）复现同败，存量、未登记、非本计划引入**（diff 未触及 charts-gallery/ui_gen）→ 已登记 KNOWN-DEBT「master 存量红」行，建议独立修复立项。tv/tt/tb 未跑：vm_file_tests 语料根为专用 vm 目录（非 examples/）、transpiler/book 未触及。

**遗漏/延后/workaround 排查**：
- 遗漏：无——T1-T8 每步在 diff 有对应改动；计划外发现并修复 4 个代码级引用（launch 测试/ui_dual_app/stage3/SKILL+DEBT 路径）已在 T4 清单留痕。
- 延后：无未批准拆分。待澄清①backlog 为用户预登记；②已在 T4 裁定闭环；④045 偏差为上游退役所致（见上）。
- Workaround：stage3 `example_source` 双根解析为正式设计（T5 注记），非临时补丁。

**债务候选**（已入 KNOWN-DEBT-AND-RISKS.md）：P552 customs 非策展元数据回退边界；master 存量红 test_charts_gallery_compiles（552 复审发现，归属 master 基线）。

## 待澄清事项

1. **后续 backlog（本计划不做，登记备查）**：回收站（027+桌面集成）、
   App Store（auto-man GUI）、截图工具（需宿主抓屏）、压缩文件管理器、
   记事本 lite、通讯录——来源 2026-09-05 桌面应用盘点讨论。
2. p518-glass-sample 若被 Plan 518 文档当作"样张资产"路径引用（docs/plans/
   518 系），迁移后文档链接是否需要留重定向注记——T4 grep 结果定。
   〔T4 裁定：命中两处——KNOWN-DEBT P518 条目（活台账，已随迁更新路径）
   与 autoui-verifier SKILL.md 对拍白名单（已随迁更新）；archive 历史计划
   引用保留原路径不加重定向（历史叙述，非活链接）。〕
3. 真画布原语（笔迹/pointer 路径 canvas）另立设计文档，与本计划无关
   （PLAN-553 像素形态绕开）。
4. **〔执行期已自行处置，请复审确认〕045-desktop-settings 退役致 C 档
   20→19**：本计划起草（2026-09-05）时 `examples/ui/045-desktop-settings`
   尚在（扫描面 43 条目含 045）；执行时发现其已被并行合入的 Plan 551 T7
   退役（commit cfcd534ff 删目录——设置走 os-config daemon 配置单源，非
   registry App 形态）。处置：T6 跳过 045（19 目录加字段）；T7 策展断言
   集为 19 id。无用户决策面（目录不存在即无字段可加）；若复审不认可
   此口径，需在 045 以新形态回归时补 `desktop: "true"`。

## spec-sync 回写记录（merge）

- 2026-09-05 `/auto-plan:merge`：P552-1..6 六节入账 `.autoos/specs.json`（reports/goals/
  architecture/designs/tests/reviews，file 指本归档件）；`docs/specs/auto-lang/ui/overview.md`
  增 552 段（512 段后）、`docs/specs/auto-lang/ui/plans.md` 与 `docs/specs/auto-man/plans.md`
  增 552 行、`docs/specs/goals.md` GOAL-009/010 追加 552（画廊计数 43→35 注记）、
  KNOWN-DEBT master 红行加 P555-D4 同源注记；`scripts/spec-index.py` 再生。
  worktree `.wt/lang-552/`（auto-lang + auto-down 只读依赖位）经 wt-guard clean 后移除，
  分支 plan-552-dev 已删；merge commit 7e7dba5ca，主检出验收 `cargo t app_registry` 13/13。
