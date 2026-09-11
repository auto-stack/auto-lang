---
plan_id: PLAN-609
status: reviewed              # drafting → executing → execution_done → reviewed → archived
feature_name: theme-capability-closeout（PLAN-601 后续：theme{} 双端消费对齐 + 包组件 SFC 发射修复）
author: [zhaopuming]
created_at: 2026-09-11
updated_at: 2026-09-11
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-007]     # AutoUI 跨端视觉一致（主题/令牌面）

affects: [auto-lang/ui, auto-man]
current_step: 6
total_steps: 6
---

# [PLAN-609] theme-capability-closeout（PLAN-601 后续：theme{} 双端消费对齐 + 包组件 SFC 发射修复）

## 0. 变更摘要

收敛 [PLAN-601](archive/601-theme-declaration-hot-switch.md)（Design 29 Phase 2）
复审/收口期发现的两个独立缺口为一条收口计划（双任务轨，AC 各自独立）：

- **T-A 轨（VM 腿 theme{} 声明消费接线）**：pac.at `theme: {}` 声明目前
  **只有 vue 腿消费**（PLAN-601 T-06：auto-man `compose` → index.css 双 mode
  块 + index.html 运行时种子）；VM/iced 腿完全无视——`set_theme_composed`
  全仓零生产调用方，VM boot 不读 `theme_decl`。同一份声明双端表现分歧，
  正是 GOAL-007 要杜绝的形态。本轨把声明合成体接入 VM boot（Plan 458 种子
  块旁），激活优先级遵从既有链 CLI > os-config > pac.at > 内置。
- **T-B 轨（包组件 SFC 发射链修复）**：`use settings: SettingsPopover`
  包组件的 SFC 发射断链——**006-hero-section/015-notes（桌面策展应用）全新
  检出 `auto run`（vue 模式）vite 解析断链起不来**（601 复审实机复现）；
  charts 裸名组件发射 `<div :data>` 占位同族。master gen 残留的
  SettingsPopover.vue（2026-09-02 真生成件）证明发射能力历史存在——属
  584/590 资产搬迁期的回归。本轨先勘验断点再修复，恢复包组件 → SFC 落盘。
- **明确排除（非目标）**：P601-T11 本体 SVG 属性 token 通道（随主题切换
  示范面另案）；Phase 2b per-app color context（T-01 已裁定出栈）；
  settings 选择器 UI（auto-os 资产面移交件）。

## 1. 目标

1. **T-A（双端消费对齐）**：pac.at `theme: {}` 声明的 app 在 VM/iced 腿
   boot 时以合成主题激活（`theme_name()` 返回声明名，resolve 查表走合成
   体）；无声明 app 零变化（内置缺省链不动）。
2. **T-B（包组件 SFC 发射）**：`use <name>: <Component>` 引用的包组件在
   vue 生成时落盘真实 SFC（非缺文件）；006-hero-section/015-notes 全新
   检出 `auto run`（vue 模式）可启动（vite 零解析错误）。
3. **示范面回归**：修复后示例 app 双端冒烟通过，theme 单测面零回归。

## 2. 架构方案

```
T-A: pac.at theme{} ──▶ auto-man Pac.theme_decl ──▶ decl::compose
                                                          │ Arc<ComposedTheme>
                                                          ▼
        rust_ui.rs run_vm_ui boot（Plan 458 种子块旁,首帧前）
          ──▶ style::theme::set_theme_composed（ACTIVE_THEME 槽+THEME_EPOCH）
                                                          │
        vue 腿（601 T-06 已接）：index.css 装配+运行时种子 ←── 同一 compose
T-B: use <name>: <Component> ──▶ 包组件发射链勘验（T-B1 定位断点）
        ──▶ vue gen components/<Name>.vue 落盘恢复（+charts 裸名占位按勘验结论处置）
```

- **T-A 激活点**：`crates/auto-man/src/rust_ui.rs::run_vm_ui` 的 boot 种子块
  （`rust_ui.rs:2622` 附近，Plan 458 的 AUTO_UI_THEME/AUTO_UI_ACCENT 应用处
  旁）——首帧前激活零重建成本；env（CLI 层）模式/accent 语义不动，声明提供
  色板，优先级链成文。
- **T-B 发射链**：`crates/auto-lang/src/ui_gen/vue.rs` 的包组件消费面
  （`known_sub_widgets` 折叠 + `use` 块注册）——App.vue 侧 import/标签发射
  存在，`components/<Name>.vue` 落盘缺失；master gen 残留
  （2026-09-02 SettingsPopover.vue 4015B 真生成件）证明发射能力存在，
  584/590 资产搬迁期为回归窗口（T-B1 勘验定位精确断点后定修复形）。

## 3. 技术栈

Rust（auto-man：`rust_ui.rs`/`vue.rs`；auto-lang：`ui_gen/vue.rs`、
`ui/style/theme/`、`design_tokens/decl.rs`——已有面，仅接线/修复）；
测试：nextest（`cargo t` 日常档 + scoped）、`auto run` 双端冒烟
（vue vite 启动零解析错误 + VM MCP snapshot）。

## 4. 需求分析与背景调查

（授权：用户 2026-09-11 指令——601 后续缺口 B+A 合并一条计划。取材：
601 复审记录 R1/R2/R3、601 收口审计实勘、KNOWN-DEBT P601-T11、
Design 29 §4.5/§7。）

- **T-A 实勘（2026-09-11）**：`grep -rn theme_decl` 生产消费仅
  `crates/auto-man/src/vue.rs:2300`（vue 腿，PLAN-601 T-06）；
  `set_theme_composed` 定义于 `crates/auto-lang/src/ui/style/theme/mod.rs:79`
  ，零生产调用方。VM boot 种子块 = `crates/auto-man/src/rust_ui.rs:2622`
  （Plan 458：env 主题/accent 首帧前应用先例）。当前无任何 app 使用
  theme{}（examples 全量扫描零命中）——能力补全，无现场事故。
- **T-B 实勘（2026-09-10/11）**：601 复审探针——worktree 全新生成
  006-hero-section，App.vue L6 `import SettingsPopover from
  '@/components/SettingsPopover.vue'` 存在而文件缺失 → vite
  "Failed to resolve import" 断链；015-notes 同症状。master gen 残留
  `SettingsPopover.vue`（4015B，mtime 2026-09-02，"Auto-generated" 头）
  证明历史发射能力。charts（601 T-11）：裸名 `line-chart` 等发射
  `<div :data=...>` 占位非组件 SFC——同一发射链族。
- **优先级链（458/504/506）**：CLI > os-config per-app > pac.at > 内置。
  T-A 的声明= pac.at 层；env（CLI）与 desktop_config（os-config/宿主，
  601 T-05）在其上，语义互补（mode/accent vs 色板）。
- **依赖**：无外部仓依赖。T-B 的修复若涉及 auto-os 侧组件源解析（590
  os_paths 解析序），按既有 `resolve_os_top_dir` 序（env→兄弟→主检出）
  处理，不新增跨仓写。

## 5. 详细设计

### T-A 轨：theme{} VM 消费接线

1. **激活点**：`run_vm_ui`（`crates/auto-man/src/rust_ui.rs`）boot 块——
   `Pac::new(AutoConfig::from_file(pac_path))` 取 `theme_decl` →
   `decl::compose(&decl, &BTreeMap::new())`（与 vue 腿 T-06 同口径：pac 单
   块解析无具名声明集）→ `Arc<ComposedTheme>` → `set_theme_composed`。
   compose 失败告警回退缺省（与 vue 腿同款容错）。
2. **优先级成文**：env 模式/accent（CLI）＞ desktop_config theme_name
   （os-config/宿主，601 T-05）＞ pac.at 声明色板 ＞ 内置缺省。声明激活在
   env 种子后执行（色板覆盖、mode/accent 不被声明清写——`set_theme_composed`
   只换槽不动 DARK_MODE/ACCENT_NAME）。
3. **桌面宿主路径**：宿主 launch 链若独立于 `run_vm_ui`（app_registry
   launch），随 T-A3 勘验结论补同款激活（如有第二入口）。

### T-B 轨：包组件 SFC 发射链修复

1. **T-B1 勘验（bounded investigation）**：定位 `use settings: X` 与
   charts 裸名两条消费路径的 SFC 落盘断点——对比 2026-09-02 生成件与当前
   codegen（git log `ui_gen/vue.rs` 发射函数/落盘调用；包扫描
   `collect_dep_front_dirs`/registry 解析链）；产出断点结论+修复形裁定
   （决策工件入本节追记）。
2. **修复**：按勘验结论恢复发射（源解析补 590 解析序 / 落盘臂修复 /
   import-发射与文件发射一致性守卫）。验收形态：全新 gen 后
   `components/SettingsPopover.vue` 存在且 vite 零解析错误。
3. **charts 裸名占位**：按 T-B1 结论同刀处置（若同根）；若异根（占位为
   484 M4 有意形态）则在勘验工件中裁定并留债注记，不强改。

> **T-B1 勘验结论（2026-09-11 执行期追记）**
>
> - **发射链本体完好**：`use <pkg>: <Comp>` 的 import 发射（ui_gen/vue.rs
>   ExtImportKind::Component 空 path 臂 → `@/components/<Sym>.vue`）、
>   Plan 475 dep 编译循环（auto-man/vue.rs from_workspace）、写盘名
>   （widget_name，非 pages 通道）与 import 名天然一致。
> - **断点 = dep 源解析双路悬空**：584/590 把 `examples/ui/common/*` 七源
>   git rm 迁 auto-os（commit 047743158 → auto-os 8a91761，落点
>   `apps/common/settings`）后，006/010/015/016 四 pac.at 的
>   `dep settings { path: "../common/settings" }` 全部死指：①`deps/settings`
>   junction（gitignore，09-02 旧物）悬空被 `is_dir` 门跳过；②pac.at
>   path 扫描 `local_path.is_dir()` 假静默跳过。SFC 无源可编而 import 照发
>   → vite "Failed to resolve import"（601 复审实勘复现）。master 残留
>   SettingsPopover.vue（09-02）非"发射能力证据"，而是 590 删源前的旧物。
> - **修复形（两刀，均在本仓 auto-man，零跨仓写）**：
>   ①`collect_dep_front_dirs` 死指回退 `resolve_dep_os_mirror`——沿
>   `resolve_os_top_dir` 解析序（env AUTO_OS_ROOT 即权威 → 兄弟 → 主检出）
>   在 auto-os `apps/` 下三候选探测（剥 `../` 搬迁形状 → common/<dep> →
>   <dep>），只读不物化不建链接（Worktree 红线）；
>   ②from_workspace import-发射/文件发射一致性守卫——App.vue 的
>   `@/components/<X>.vue` 导入逐一比对编译集，缺者 strict 硬错/非 strict
>   告警（CodeEditor shell 与 ui/ 深路径豁免）。
> - **执行期新发现（根因之二，冒烟暴露）**：`run_vue_project` 冷启动
>   `incremental_compile_changed` 无 dep 阶段（仅写 App.vue），changed>0
>   走 `generate_scaffolding_only`（刻意保留组件目录）→ 全新检出仍无 SFC。
>   补 **Phase 1c**（与 Phase 1b components/ 包目录 Plan 484 同疾同构）：
>   dep .at 编译 write-if dirty/missing 直写 components/，widget 名并入
>   sub_widget_names（与 from_workspace Phase-1 扫描同口径）。
> - **charts 裸名裁定：异根，不改**。占位折叠为 484 M4 有意形态
>   （test_charts_gallery_compiles 显式断言），债记 KNOWN-DEBT P601-T11
>   （SFC 化随 SVG 属性 token 通道示范面另案）。

> **T-A3 勘验结论（2026-09-11 执行期追记，§10-2 闭环）**
>
> 桌面宿主 launch 链第二入口勘验：`DesktopSession::launch_app` 两臂——
> - **outproc 臂**：`spawn_outproc_child` 执行 **`auto run
>   --autodesk-incubate`**（session.rs:2244）→ VM render app 经
>   `run_vm_ui` → **T-A1 激活已覆盖**（子进程独立 boot，声明在其
>   pac.at 生效）。
> - **inproc 臂**：`build_dynamic_component` 直构组件，不经 run_vm_ui、
>   不消费 pac theme{}。补同款激活需把 theme{} 块解析（现居 auto-man
>   Pac，auto-lang 只持有 ThemeDecl/compose/主题运行时）引入 session
>   域——跨 crate 解析面移动，超出本计划设计锚（§2 T-A 激活点仅锚
>   run_vm_ui boot）；且实勘零 app 声明 theme{}，无可观察消费面。
>   **裁定：不接线，留债**——首个桌面 inproc app 声明 theme{} 时随
>   消费面立项（候注记于本节）。AC-01 不受影响（§10-2 明文）。

### 规范增量

| delta_id | op | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md（601 条目） | 「theme{} 声明…经 auto-man 消费（vue 腿）」→「双端消费：vue index.css+运行时种子；VM boot set_theme_composed 激活（优先级链 CLI>os-config>pac.at>内置）」 | T-A 落地后的现状成文 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md（关键入口/ui_gen 行） | 包组件消费注记补「use 引用的包组件必须落盘 components/*.vue（import-发射一致性）」 | T-B 修复的约束成文 | AC-03/04 |

## 6. 测试设计

- **T-A**：单测——theme{} 声明的临时工程经 `run_vm_ui` boot 路径后
  `theme_name()`=声明名/`resolve` 走合成体（隔离 env 下驱动，循 601
  `desktop_boot_activates` 形态）；无声明零变化负例；未知名/坏值容错负例。
- **T-B**：生成面测试——包组件 app 全新生成后 `components/<Name>.vue`
  存在 + App.vue import 与文件一致性断言；006-hero-section/015-notes
  `auto run`（vue 模式）vite 启动零解析错误冒烟（手测/MCP 探针留痕）。
- **回归**：`cargo t` 日常档红集合与基线全等；`cargo test -p auto-man`。

## 7. 验收标准

- **AC-01**（T-A）：theme{} 声明 app 在 VM 腿 boot 后活动主题=合成主题
  （`theme_name()`/resolve 可断言），无声明/坏声明回退缺省，env 优先级
  语义不变。验证：新单测 + 既有 theme 面 6/6 不回归。
- **AC-02**（T-A）：双端同源——同一 theme{} 工程 vue index.css 值与 VM
  resolve 值逐键相符（既有双面一致性测试口径延展到声明合成体）。
- **AC-03**（T-B）：全新检出 006-hero-section `auto run`（vue 模式）启动
  成功（vite 零解析错误），SettingsPopover.vue 为真生成件（非手工桩）。
- **AC-04**（T-B）：charts 裸名消费按 T-B1 裁定落地（同根则发射修复；
  异根则裁定+债注记成文），024-charts vue 面测试维持绿。
- **AC-05**：日常档红集合与 master 基线全等，零新增红。

## 8. 执行步骤

- [x] **T-01**（T-B1）包组件 SFC 发射链勘验：git log/代码走读定位断点
  （发射函数、落盘调用、源解析），对比 09-02 生成件；产出断点结论与
  修复形裁定（追记本计划 §5）。验证：勘验工件（结论段落）。依赖：无。
  `[✅ 2026-09-11]` 断点=dep 源双路悬空（590 迁 auto-os 后 pac.at path 死
  指+junction 悬空），发射链本体完好；结论工件见 §5 追记。证据：
  commit 596eb30b7；047743158（590 git rm）。
- [x] **T-02**（T-B2）按 T-01 结论修复发射链。验证：新生成含
  `components/SettingsPopover.vue`；生成面一致性测试绿。依赖：T-01。
  `[✅ 2026-09-11]` 镜像回退+一致性守卫双刀落地 auto-man/vue.rs。证据：
  test_plan609_dead_dep_resolves_via_auto_os_mirror /
  test_plan609_unresolved_dep_import_guard 双绿（nextest 2 passed）；
  test_plan475 回归绿；commit 596eb30b7。
- [x] **T-03**（T-B3）006-hero-section/015-notes `auto run` vue 冒烟 +
  charts 裸名处置落地。验证：vite 零解析错误留痕；024-charts 测试绿。
  依赖：T-02。（AC-03/04）`[✅ 2026-09-11]` 全新检出差温（删 gen/.auto）
  `auto run`：006 SettingsPopover.vue 落盘且与 09-02 残留逐字节一致，
  vite ready 零 resolve 错误，root/App.vue/组件 HTTP 探针全 200；015 三件
  （EditorPanel/NavTree/SettingsPopover）落盘同字节一致、全 200（后端
  panic=8080 AddrInUse 环境碰撞，非本修复面）。执行期暴露冷启动增量路径
  无 dep 阶段——补 Phase 1c（§5 追记）。charts 裁定异根不改，债记
  KNOWN-DEBT P601-T11；test_charts_gallery_compiles 绿。证据：
  commit 86acdd516。
- [x] **T-04**（T-A1）`run_vm_ui` boot 接线：theme_decl→compose→
  set_theme_composed + 容错 + 优先级成文。验证：新单测
  （声明激活/无声明零变化/坏值容错三段）。依赖：无（与 T-01..03 并行）。
  `[✅ 2026-09-11]` `apply_pac_theme_decl` helper + run_vm_ui env 种子后
  调用（声明只换色板槽，DARK_MODE/ACCENT_NAME 不清写）。三段单测绿：
  vm_boot_pac_theme_decl_activates_composed / vm_boot_no_theme_decl_
  zero_change / vm_boot_bad_theme_decl_falls_back。证据：commit 596a87141。
- [x] **T-05**（T-A2）双端同源验证：声明合成体 vue 值 vs VM resolve 值逐键
  断言（测试或对拍工件）。验证：断言绿。依赖：T-04。（AC-02）
  `[✅ 2026-09-11]` plan609_theme_decl_dual_face_same_source 绿：同一
  ComposedTheme 喂 index.css 与 VM 槽，词表内键逐键断言（扩展 4 键
  VM 面承载边界成文），VM resolve==CSS 值串解析。证据：commit 596a87141。
- [x] **T-06** 门禁收口：`cargo t` 全量（红集合对拍基线）+ auto-man 显式 +
  簿记交接。依赖：T-03/T-05。（AC-05）`[✅ 2026-09-11]` 日常档
  （--no-fail-fast 全量 4746）红集合 22 条与 master 同命令逐条全等
  （零新增红）；`cargo nextest run -p auto-man` 279/279 绿；theme 面
  36/36 绿；fmt 漂移计数与 master 全等（零新增）。T-A3 第二入口勘验
  闭合（§5 追记）。

## 9. 复审记录

（draft 2026-09-11：`stage: new | PLAN-609 r1 | outcome: pass——双轨任务
齐备,落点已实勘锚定（rust_ui.rs:2622 种子块/ui_gen 发射链），T-01 为
bounded investigation 带决策工件 | blockers: 无 | next: work 自 T-01/T-04
（两轨可并行）`）

（work 2026-09-11：`stage: work | PLAN-609 r1 | outcome: pass | code_commit:
596eb30b7/86acdd516/596a87141（worktree plan-609-dev，基点 1e834db38；
auto-down 兄弟 worktree detached 1557a39 仅作构建依赖）| task_ids: T-01..06
全闭环 | evidence: T-B 双刀（镜像回退 resolve_dep_os_mirror + 一致性守卫 +
增量 Phase 1c）实测 006/015 全新检出 auto run vite 零解析错误、
SettingsPopover/EditorPanel/NavTree 与 09-02 已知良好生成件逐字节一致；
charts 异根裁定债记 KNOWN-DEBT P601-T11；T-A 三段 boot 单测 + 双端同源
逐键对拍绿；门禁：日常档红集合 22 与 master 全等、auto-man 279/279、
theme 面 36/36、fmt 零新增 | blockers: 无（015 后端 panic=8080 AddrInUse
环境碰撞，非本修复面）| next: review`）

（review 2026-09-11：`stage: review | PLAN-609 r1 | outcome: pass |
reviewed_commit: 596a871417a28f0af20151e0019dc6cdcbe4f1ae（worktree 干净）
| base_commit: 1e834db38 | dependency_revisions: auto-down detached
1557a39（仅构建依赖，零修改）| spec_inputs:
docs/specs/auto-lang/ui/overview.md（SD-01 落点 line196「经 auto-man
消费（index.css+运行时种子）」、SD-02 落点 line212 关键入口 ui_gen 行，
before 均核实）| acceptance_results: AC-01 pass（三段 boot 单测重跑绿：
合成名上槽/resolve==合成体/mode+accent 不清写；无声明与坏 extends 负例
绿；theme 面 36/36）| AC-02 pass（plan609_theme_decl_dual_face_same_source
重跑绿：CSS 逐键携带+VM resolve==CSS 值串解析+覆盖键规范化落位）|
AC-03 pass（复审重执行：006 删 gen/.auto 全新 auto run——vite ready 零
Failed to resolve import，root/App.vue/SettingsPopover.vue 三探针 200，
生成件与 09-02 已知良好件 diff 空；015 三组件 vite transform 全 200）|
AC-04 pass（charts 异根裁定工件在案 §5+KNOWN-DEBT P601 债务节；
test_charts_gallery_compiles 绿）| AC-05 pass（复审重跑双侧日常档：
稳定红核 21 条 diff 空；两侧各现一次互异瞬时单红
〔plan425@master / ffi_dual_019@worktree〕单跑均过不跨侧复现=资源抖动；
另 cargo tf 复审档 3506/3506 全绿、auto-man 279/279）| findings:
R-1 规范注记（非阻断，合并沉淀时处理）——overview.md line205-207
P601-T11 开放债句「settings-popover 等裸名包组件 SFC 化缺口」应收窄：
609 实证 settings-popover 断链根因=dep 源死指（本计划已收口），
开放债仅余 charts 裸名占位（484 M4 有意形态）；R-2 观察项（非阻断）——
运行 015 邻接测试套/冒烟会以现行 api_gen 输出重写
examples/rust-workspace/ 入库样例（生成器与入库副本预存漂移），复审期
已两次还原，建议后续独立小债登记 | evidence: 本记录命令与结果均为复审
会话独立重执行；同会话复审限制已声明，以工件重建裁决 |
supersedes_spec_components: []（两笔 SD 均为 overview.md 条目内 modify，
无 retire/新增组件——空集说明）| new_spec_components: []（同前）|
touched_goals: [GOAL-007] | next: merge`）

## 10. 待澄清事项

1. T-B 若勘验结论指向组件源已迁 auto-os（584/590 搬迁），修复形在
   「解析序补 auto-os 源」与「本仓恢复源资产」之间择一——按 590 既有
   解析序裁定倾向前者，执行期如需跨仓改动升级为待澄清升级用户。
   **〔已闭合 2026-09-11〕**勘验证实源迁 auto-os（§5 追记）；修复形=
   解析序镜像回退（`resolve_dep_os_mirror`），零跨仓写，未升级。
2. T-A 桌面宿主 launch 链若存在独立于 `run_vm_ui` 的第二入口，随勘验
   补同款激活；不影响 AC-01 判定（以 `run_vm_ui` 主路径为准）。
   **〔已闭合 2026-09-11〕**勘验见 §5 T-A3 追记：outproc 臂经
   `auto run`→`run_vm_ui` 已被 T-A1 覆盖；inproc 臂裁定不接线留债
   （跨 crate 解析面移动 + 零声明消费者）。
