---
plan_id: PLAN-672
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: ui-gallery-vue-fix-batch
author: [zhaopuming]
created_at: 2026-09-21
updated_at: 2026-09-21

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/auto-man]  # 主修面；auto-os 侧仅参照副本同步（不涉 specs）
current_step: 0
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

- [ ] T-00 master 提交计划骨架与 .next-id（簿记类，master 允许）。
- [ ] T-01 auto-lang worktree（`D:/autostack/.wt/lang-672/auto-lang @ plan-672-dev`）
      修 `crates/auto-man/assets/gallery/AppViewport.vue` + 模板测试。
- [ ] T-02 同 worktree 调序 vue.rs W6 块 + 补时序断言。
- [ ] T-03 门禁：`cargo check -p auto-man` + `cargo t -p auto-man` +
      `cargo t plan633`（embed 断言面）。
- [ ] T-04 重建 auto 二进制 → auto-os 侧同步参照副本（兄弟 worktree 或主检出
      受控落地）→ 重启画廊 → playwright 截图四例 → 用户复核。

## 复审记录

## 待澄清事项

- auto-os 工作区现存他方 WIP（registry.at / AppViewport.vm.at / demos 012·013·020 /
  两 store，2026-09-21 在册）——与本计划改动面（auto-lang 模板+vue.rs）不重叠；
  os 侧参照副本同步时注意避开其未提交状态（只动 AppViewport.vue 一文件）。
