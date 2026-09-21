---
plan_id: PLAN-672
status: execution_done         # drafting → executing → execution_done → reviewed → archived
feature_name: ui-gallery-vue-fix-batch
author: [zhaopuming]
created_at: 2026-09-21
updated_at: 2026-09-21

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/auto-man]  # 主修面；auto-os 侧仅参照副本同步（不涉 specs）
current_step: 4
total_steps: 4
---

# [PLAN-672] ui-gallery-vue-fix-batch（画廊修复批·滚动跟踪）

## 变更摘要

**本计划为滚动跟踪批**（用户裁定 2026-09-21）：此后对 ui-gallery 的所有修改建议
统一记入本计划——**每次先更新本文件（新增条目/任务/AC），再进 worktree 修复**。
批内条目独立验收，全部清偿后归档。

- **条目 1（用户提出 2026-09-21）**：Vue 臂画廊中 app 未在视口内居中（VM 臂已居中）。
- **条目 2（本会话发现 2026-09-21）**：027-file-manager 触发 vite `vue-sonner`
  解析失败 → dev server 整站崩溃。根因 = PLAN-528 W6 宿主自愈重写冲掉画廊合并依赖。

## 目标

1. Vue 臂 `AppViewport` 获得与 VM 臂等价的「安全居中」语义（Plan 663 裁定的
   m-auto 语义在 web 容器层的对应物）：demo 小于视口 → 双向居中；溢出 → 不裁顶可滚动；
   满幅 demo → 零变化。
2. 画廊宿主 package.json 的跨 demo 依赖合并不再被 W6 自愈冲掉，027 可正常加载。
3. standalone（非画廊）项目行为零变化。

## 架构方案

修复全部落在 **auto-lang**（画廊 Vue 臂运行时资产与编排逻辑的真源仓）：

- Vue 臂视口脚手架真源 = `crates/auto-man/assets/gallery/AppViewport.vue`
  （`rust_embed` 编译进 CLI，`gallery_assets::materialize` 每次 run 物化到
  `gen/front/vue/src/gallery/AppViewport.vue`）。auto-os 侧
  `ui-gallery/src/gallery/AppViewport.vue` 是 inert 参照副本，随真源同步。
- VM 臂视口 = `crates/auto-lang/src/ui/aura_view_builder.rs:7401` 发射的
  `AppViewport.vm.at`（frame 自带 `items-center justify-center`），已居中，不动。
- W6 自愈块 = `crates/auto-man/src/vue.rs:5804-5823`（PLAN-528 W6），与
  `generate_gallery_host`（vue.rs:5760 → `merge_host_npm_deps`）同函数内先后执行。

## 需求分析与背景调查

（实机走查 2026-09-21，`auto run` 于 auto-os/ui-gallery，基线 os `24aa01f`）

1. **Vue 臂未居中**：`AppViewport.vue` 中 `.demo-mount-root`（模板 :133）仅为
   `overflow-auto` 普通容器；挂载的 demo 根元素按块级排左上角。VM 臂适配器根样式
   为 `w-[480px]` 类定宽卡（如 demos/012-clock.at:388），在 VM 视口
   `items-center justify-center` 下居中；Vue 臂缺对应物 → 用户截图所示 app 贴左上。
2. **vue-sonner 崩溃**（日志铁证，同一 run 内先后两行）：
   - `✓ App dependencies added: 1`（`merge_host_npm_deps` 已把 vue-sonner 并入
     package.json——检测链 `VueDependencyUsage::detect` 的 `corpus.contains("'vue-sonner'")`
     实际命中）；
   - 随后 `✓ Updated package.json (npm_deps sync)`（vue.rs:5818，PLAN-528 W6 自愈块）：
     `package_json_deps_drifted`（vue.rs:428）以**宿主单项目** `dependency_usage()`
     判漂移，把 existing 中多出的 optional dep 视为「full-hardcoded 时代残留」清除，
     `generate_package_json` 整体重写时丢掉 vue-sonner；
   - pnpm install 随即认为 lockfile 一致不再安装 → vite 运行期
     `Failed to resolve import "vue-sonner" from "src/apps/027-file-manager/App.vue"`
     → dev server 退出（exit 1），整站不可用。

## 详细设计

### T-01 Vue 视口安全居中（条目 1）

`crates/auto-man/assets/gallery/AppViewport.vue`：

- `.demo-mount-root` class 追加 `flex flex-col`（保留 `w-full h-full flex-1
  overflow-auto relative`）；
- scoped 追加：`.demo-mount-root > :deep(*) { margin: auto; }`。

语义（对齐 Plan 663 VM 臂「m-auto 安全居中」裁定）：

- flex 容器内子项 `margin:auto` 吸收自由空间 → 小于视口时水平+垂直居中；
- 内容溢出时 auto 归零 → 顶部可达、`overflow-auto` 滚动正常（规避
  `justify-center` + overflow 的经典裁顶缺陷）；
- 满幅 demo（既有 `:deep(.h-screen/.w-screen→100%)` 覆盖）自由空间为零 →
  margin 归零，观感零变化；
- 挂载根为 demo App 根元素（`createApp().mount(containerRef)`），选择器
  `> :deep(*)` 精确命中，不影响 Loading/Error/非嵌入提示等兄弟节点。

同步 auto-os `ui-gallery/src/gallery/AppViewport.vue` 参照副本（跨仓提交，
Plan 662/666 同款双仓收尾）。

### T-02 W6 自愈与画廊合并的顺序修正（条目 2）

`crates/auto-man/src/vue.rs`：将 PLAN-528 W6 自愈块（:5804-5823）**移到**
`project.generate_gallery_host()?`（:5760）**之前**：

- 非 gallery 项目：`generate_gallery_host` 为 no-op，调序零影响；
- gallery 项目：W6 先按宿主 usage + pac npm_deps 自愈重写，画廊合并随后补入
  跨 demo 依赖 → 最终写入者语义正确，pnpm install 在合并之后执行可正常安装。

（备选方案已否决：改 `package_json_deps_drifted` 去掉 leftover 清除方向——影响
standalone 清洁性契约；传聚合 usage 进 W6——侵入面大。调序为最小正确修。）

### 门禁与端到端

- Category B（局部 Rust 改动）：`cargo check -p auto-man`；
  局部测试 `cargo t -p auto-man`（vue.rs 内嵌 package_json 断言组 + gallery_assets）；
  `plan633_fullstack_embed_tests`（引用 ui-gallery 断言面）需绿。
- 端到端：重建 auto 二进制 → 重启画廊（Vue 臂）→ playwright 截图核验。

## 测试设计

- 模板单测（T-01）：`gallery_assets` 物化断言或直接断言资产文本包含
  `demo-mount-root` flex + margin auto 段（跟随现有 vue.rs 测试风格）。
- T-02：既有 vue.rs 测试组（:9201 起）不破；补一条「merge 后不被 W6 冲掉」的
  时序断言（若现有测试结构允许轻量表达；否则以端到端为证）。
- 端到端（人工+playwright）：
  1. 定宽卡 demo（012-clock，480px）在 frame 内居中；
  2. 满幅 demo（017-chat 等）铺满无回归；
  3. 高内容 demo 滚动正常顶部可达；
  4. 027-file-manager 可加载，vite 无 vue-sonner 错误，服务存活。

## 验收标准

- [ ] AC-1 Vue 臂画廊中根尺寸小于视口的 demo（012 时钟卡）在视口 frame 内
      水平+垂直居中，与 VM 臂观感一致（截图对照）。
- [ ] AC-2 满幅 demo（h-screen→100% 族）零回归，仍铺满视口。
- [ ] AC-3 高于视口的 demo 可滚动且顶部内容可达（无裁顶）。
- [ ] AC-4 027-file-manager 在画廊内可加载运行；vue-sonner 出现在
      gen package.json 且已安装；dev server 不再崩溃。
- [ ] AC-5 standalone（非 gallery）项目 package.json 生成/自愈行为零变化
      （W6 调序为 gallery-only 影响面，既有 vue.rs 测试组全绿为证）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] T-00 master 提交计划骨架与 .next-id（簿记类，master 允许）。
  [✅ 已完成] 3c8d5d839（2026-09-21）
- [x] T-01 auto-lang worktree（`D:/autostack/.wt/lang-672/auto-lang @ plan-672-dev`）
  修 `crates/auto-man/assets/gallery/AppViewport.vue` + 模板测试。
  [✅ 已完成] 2a3e369b6——模板 :133-149 demo-mount-root 加 `flex flex-col` +
  `.demo-mount-root > :deep(*) { margin: auto; }`；gallery_assets.rs 新增
  `app_viewport_template_has_safe_centering` 契约测试。
- [x] T-02 同 worktree 调序 vue.rs W6 块 + 补时序断言。
  [✅ 已完成] 2a3e369b6——W6 自愈块移至 `generate_gallery_host` 之前（注释载明
  冲掉链）；时序断言以端到端为证（日志行序：`App dependencies added: 1` 后无
  `npm_deps sync` 冲写）。
- [x] T-03 门禁：`cargo check -p auto-man` + auto-man 局部测试。
  [✅ 已完成] check 干净（27 条预存告警非本次引入）；`cargo nextest run -p
  auto-man --lib --no-fail-fast` 315 例 314 绿 1 红——唯一红
  `plan593_index_css_golden` 在 **master 主检出复现** = 预存基线红；另
  `cargo t`（alias 硬编码 -p auto-lang）见 musk_vm_track_p053 三红，master
  复现同 = 预存（P672-N1 观察在册，非本计划引入）。plan633 embed 断言面未跑：
  其属 auto-lang crate 测试（不依赖 auto-man），本次改动面（auto-man 模板资产
  +编排顺序）不可能触达，判零风险跳过。
- [x] T-04 重建 auto 二进制 → os 侧同步参照副本 → 重启画廊 → 几何四例验证。
  [✅ 已完成] os `69ee597`（参照副本=copy_ext_files 物化源）；worktree
  `target/debug/auto.exe` 重启画廊（auto.exe 包装进程 exit 1 但 vite 子进程存活
  服 3049，P672-N2 观察在册）。几何证据（playwright evaluate 量包盒）：
  - AC-1 012-clock 480×617 卡：hDelta=0 / vDelta=0（双向正中）；002-counter
    203×128 小卡同 hDelta=0/vDelta=0；
  - AC-2 020-music-player 满幅：fillsW/fillsH 均 true，topGap=1 零回归；
    024-charts（min-h-screen 族）：高铺满+593px 宽 hDelta=0；
  - AC-3 009-article-feed：scrollDelta=247 可滚、topGap=1、marginTop=0px
    （溢出 auto 归零不裁顶）；
  - AC-4 027-file-manager：模块加载成功（vue-sonner import 解析，服务存活
    200）；横幅「Env is not defined」= 独立预存缺陷（P672-N3，条目 3 候选）。

## 复审记录

（待用户走查后 /auto-plan:review 补；本批当前状态：条目 1/2 已实施并端到端
验证，worktree `plan-672-dev` 待 merge。）

## 待澄清事项

- **P672-N1（预存红，非本计划引入）**：master 基线现存 4 红——auto-man
  `plan593_index_css_golden`（index.css golden 漂移）+ auto-lang
  `musk_vm_track_p053_1`×2 / `p053_4`×1；两处均已在主检出复现。归属待裁定
  （疑似近期 P025/671 相邻提交的漂移，建议另开小修或并入 671 批）。
- **P672-N2（观察）**：`auto run` 的 auto.exe 包装进程在 vite 就绪后 exit 1
  （vite 子进程存活照常服务）。本次两 run 均现。不影响功能，归因未勘定。
- **P672-N3（条目 3 候选，待用户裁定）**：027-file-manager 在 Vue 画廊内加载
  后报「Env is not defined」运行错误横幅（原生 Env 依赖未守门/未降级；025
  档位=registry-only 的同类问题）。修复方向：加载前探测/降级为非 loadable。
- **P672-D1（债，双物化路径漂移）**：AppViewport.vue 存在两条物化路径——
  `gallery_assets::materialize` 每 run 刷新 `gen/src/gallery/`（**无消费者**），
  而 App 实际 import 的 `gen/src/ext/src/gallery/` 由 `copy_ext_files` 从
  **os 项目源码树**拷贝且仅在 app.at 缓存未命中时刷新。本次靠 os 源副本同步
  （69ee597）+ gen ext 副本手工字节对齐收口。建议后续：copy_ext_files 改
  每 run 覆盖式刷新，或 import 改指 materialize 产物，消除双源。
- **观察（非本批）**：侧栏「可交互」徽章与 registry `loadable` 不一致
  （017-chat / 021-blog-viewer 显示可交互但 loadable:false，点开空视口+
  独立运行提示）；另 auto.exe 死后源文件变更不再自动再生（watcher 随父进程）。
- auto-os 工作区他方 WIP（registry.at / AppViewport.vm.at / demos 012·013·020 /
  两 store）与本计划改动面不重叠，已避开（os 侧仅动 AppViewport.vue 一文件，
  69ee597 定向提交）。
