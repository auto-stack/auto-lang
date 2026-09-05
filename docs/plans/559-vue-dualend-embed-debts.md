---
plan_id: PLAN-559
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vue-dualend-embed-debts
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 0
---

# [PLAN-559] vue-dualend-embed-debts

## 变更摘要

PLAN-551 归档后的 follow-up 合并计划（用户 2026-09-05 裁定：Vue 对拍链与
P551-D1/D4/D5 三债并入同一计划解决）。六个工作流：

1. **W1（P551-D1）master tf 双红收口**：`schema_drift_fence` +
   `docs_gen kitchen_sink_page_in_sync`——shadcn tag 表（Plan 530 引入，
   548 会话扩展）未被 schema/aura.at 覆盖。T1 先复核 548 会话是否已随行
   修复（主检出 schema/aura.at 曾有未提交修改）；未修则
   `SCHEMA_DRIFT_GENERATE_AT=1` 重生成 + 复核 diff + tf 双红转绿。
2. **W2（P551-D3）os-config vue 构建生成偏斜**：`auto build` vue 轨
   tsc 红（`Cannot find module '@/lib/api'` 等，gen 树缺 lib/api.ts 粘合
   层；pristine 同红=auto.exe 与 os-config 已提交源码版本偏斜）——修
   vue 轨 codegen 的 back.api 粘合层生成，os-config `auto build` 全绿。
3. **W3 desktop-host 嵌入 api-client app**：465 v1 生成器跳过
   needs-API-client app——扩展 `generate_desktop_host`：为声明 back.api
   的 app 生成宿主侧 api 粘合（daemon_base 注入），扫描根增相邻仓
   `../auto-os-config/auto`，os-config 作为首个嵌入试点。
4. **W4 vue desktop-host 齿轮入口**：Taskbar.vue 新增 ⚙️ 按钮 → wm store
   launch os-config 窗（vm 轨 551 T2 语义的 vue 面对齐；551 T7 时因 W3
   架构边界未做，本 plan 补齐）。
5. **W5 Desktop 页双端对拍**：DesktopPage（标签导航四子页+壁纸选择器）
   vue/vm 同源截图对拍（autoui-verifier 双轨）。
6. **W6+W7（P551-D4/D5）**：ConfigEditor 字段级 widget 挂载（widgets
   映射按模块装载缓存 + entryAt kind 覆盖 + picker 渲染分支）+
   验收通道子组件注入面（autoui_desktop handler 支持 app+widget 定位，
   picker 点选 click-through e2e 场景）。

跨仓：auto-os-config（W2/W5/W6/W7）与 auto-lang（W1/W3/W4，W7 两侧），
worktree 组平铺（`.wt/lang-559/{auto-lang,auto-os-config}`）。

## 目标

- **G1（W1）全量门回绿**：master `cargo tf` schema_drift_fence 与
  docs_gen 双红清零（与 548 会话协调，避免 schema/aura.at 双头改）。
- **G2（W2）vue 轨构建绿**：os-config `auto build` 过 tsc+vite（含
  gen 树 lib/api 粘合层生成），pristine 与工作树行为一致。
- **G3（W3）desktop-host 能装 api-client app**：os-config 以普通窗口
  嵌入 vue 桌面（数据走 daemon，AUTOOS_DAEMON/base 注入）。
- **G4（W4）vue 齿轮入口**：vue 桌面任务栏 ⚙️ → os-config 窗。
- **G5（W5）Desktop 页双端对拍绿**：四子页+选择器 vue/vm 截图对拍。
- **G6（W6）通用编辑器字段级挂载**：widgets 声明在 ConfigEditor 生效
  （wallpaper_picker/dir_picker 内联渲染，写路径仍单源 PUT）。
- **G7（W7）picker 点选 e2e**：验收通道可注入 DesktopPage 子组件，
  点选→config.at→热应用全链自动化场景。

## 架构方案

### 现状勘证（2026-09-05，PLAN-551 实测）

| 项 | 现状 | 根因/差距 |
|---|---|---|
| D1 tf 双红 | schema_drift_fence 列 alert-dialog/dialog/dropdown 族 tag 未被 schema/aura.at 覆盖（93d933a62 Plan 530 引入、548 扩展）；docs_gen kitchen_sink 同步红 | 重生成 schema 即可；548 会话 schema/aura.at 曾有未提交修改（疑似修复中）——T1 复核 |
| D3 vue 构建 | os-config `auto build` tsc：`@/lib/api`、`Env`、`http`、`json` 全缺——gen/front/vue 无 src/lib/api.ts；vm merged 轨不受影响（back.api 解析根=back 桩） | vue 轨 codegen 未为 back/api-at 生成粘合层；版本偏斜细节 T1 定位 |
| desktop-host 嵌入 | `generate_desktop_host` v1 守卫跳过 `needs API client`（`@/lib/api`/`@/api`/ext/i18n/router 五类） | api 粘合层就绪后（W2）守卫可放开 api 类；宿主需 daemon_base 注入 |
| vue 齿轮 | Taskbar.vue 仅 summon/窗按钮/布局/alt-tab，无设置入口 | 新增 ⚙️ → launch os-config |
| D4 挂载 | entryAt 无 widgets 感知；ConfigEditor 视图 per-render 取映射不可接受（每次 HTTP） | widgets 映射随模块装载一次入编辑器 state，entryAt 增参或调用点改写 |
| D5 注入 | autoui_desktop handler 的 app 枚举固定五槽（551 已重绑 settings→os-config root），子 widget 不可达 | handler 动作增 widget 定位维度（app+widget→子组件 msg 直调） |

### 方案要点

1. **W2 先行是 W3/W5 的前置**（粘合层是嵌入与对拍的地基）。
2. **宿主 api 粘合设计**：desktop-host 的 api 层 = 生成的 `lib/api.ts`
   （fn 签名自 back/api.at 提取，实现 = fetch(daemonBase + url) 映射）；
   daemonBase 构建期注入（515 G3 壁纸注入同款常量机制）或运行期
   `AUTOOS_DAEMON` env。签名提取复用 W2 的 codegen 粘合层——一处修复
   两处受益（独立 vue 前端 + desktop-host 嵌入）。
3. **扫描根扩展**：desktop-host 生成器 scan_apps 增 extra roots 参数
   （vm 轨 `host_extra_roots` 同构：`../auto-os-config/auto`），id 去重
   主根优先。
4. **D4 缓存面**：ConfigEditor 装载模块时同步取 widgets 映射（一次
   HTTP），存 widget state；entryAt 增可选 widgets 参数（现有调用点
   传缓存），命中字段 kind=widget 名 → 视图分支挂 WallpaperPicker/
   DirPicker（写路径自包含 fresh GET→editField→PUT，551 T5 语义）。
5. **D5 注入面**：`autoui_desktop` handler 动作 payload 增 `widget`
   可选字段——注入层按 (app, widget) 定位 DynamicComponent 子实例调
   handler；DesktopPage/ConfigEditor 子组件即可驱动（picker Pick、
   Nav、SelectModule 全链自动化）。

## 需求分析与背景调查

（勘证源自 PLAN-551 执行期实测与债登记 KNOWN-DEBT-AND-RISKS.md
P551-D1..D5，2026-09-05）

- GOAL-007（AutoUI 跨端一致）：Desktop 页 vm 轨已绿（551 实机三连证），
  vue 轨因本 plan 的 W2/W3 缺位无法对拍——双端一致性义务的收尾面。
- GOAL-009（虚拟桌面与桌面 Shell）：⚙️ 直开 os-config 的 vue 面缺失
  （W4），vm 面已收官。
- 465（vue 虚拟桌面宿主）：v1 五类跳过守卫中 `needs API client` 一类
  的解除条件即为 W2 粘合层；`ext/i18n/router` 三类守卫不在本 plan。
- 501/551（os-config 嵌入 vm 桌面）：vm 轨链路全绿；vue 轨为对称面。
- D1 债主：Plan 530/548 的 shadcn tag 扩展未走 schema 重生成流程——
  本 plan 收口并沉淀「tag 表改动必须重生成 schema」的流程约定。

## 详细设计

- D1 W1 收口判定：`cargo test -p auto-lang --test schema_drift --test
  docs_gen` 当前态 → 已绿则记档销债；红则重生成（548 协调窗口：其
  schema/aura.at 未提交修改若仍在，先与其对齐再动）。
- D2 W2 定位：vue codegen 对 `use back.api` 的处理（codegen.rs
  import_scope）生成调用点，但 lib/api.ts 粘合模块的生成缺失——
  T1 用最小 os-config 副本定位缺失生成的确切环节（resolve_back_api）。
- D3 W3 粘合复用：desktop-host 生成器调用 W2 的粘合层产出入
  `src/lib/api.ts` + app 级 `daemonBase` 注入；守卫放开仅限
  api-client 类（ext/i18n/router 仍跳）。
- D4 D5 依赖序：W7 注入面在 W6 之后（先有挂载点才有可驱动的子组件）。
- D6 兼容面：Taskbar.vue 属 auto-man assets（生成资产）——版本化
  影响 desktop-host 再生成；对既有已生成 desktop-host 无破坏（按钮
  纯增量）。

## 测试设计

- W1：`cargo tf` 双红转绿（全量门，本 plan 唯一全量跑）。
- W2：os-config `auto build` 过 tsc+vite；pristine 复跑同绿；
  gen 树含 src/lib/api.ts。
- W3：desktop-host 再生成含 os-config 窗口；Playwright 打开 →
  os-config 前端渲染 + 数据来自 daemon。
- W4：Taskbar ⚙️ 点击 → os-config 窗（Playwright）。
- W5：Desktop 页 vue/vm 截图对拍（autoui-verifier 双轨，dock 页 +
  外观页 + picker 三 shots）。
- W6：单测（registry widgets→entryAt kind 覆盖）+ 实机（ConfigEditor
  打开 desktop 模块 → picker 内联渲染）。
- W7：acceptance 新场景 `p559`：picker Pick → config.at 断言 →
  宿主热应用（551-10/11 同款对照）。

## 验收标准

- [ ] master `cargo tf` 全绿（schema_drift_fence + docs_gen 转绿）。
- [ ] os-config `auto build` vue 轨全绿（tsc+vite，含 lib/api 粘合生成）。
- [ ] vue desktop-host 嵌入 os-config：窗口渲染 + 模块数据来自 daemon。
- [ ] vue desktop-host 任务栏 ⚙️ → os-config 窗。
- [ ] DesktopPage vue/vm 对拍三 shots 一致性通过。
- [ ] ConfigEditor 打开带 widgets 声明的模块 → picker 内联渲染（非平铺
      输入框），点选落盘与手输同构。
- [ ] 验收通道 p559 场景：picker 点选 → config.at → 宿主热应用全链绿。
- [ ] 「tag 表改动必须重生成 schema」流程约定沉淀（设计文档/AGENTS 注记）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- T1 复核与定位：master tf 双红现状复核（548 是否已修）；vue codegen
  back.api 粘合缺失环节定位（resolve_back_api / codegen.rs import
  处理）；desktop_apps_dir/extra roots 现状盘点。产出：三份定位笔记
  回填本 plan 待澄清。
- T2 W1：schema 重生成（SCHEMA_DRIFT_GENERATE_AT=1）+ 复核 diff +
  `cargo tf` 双红转绿（与 548 协调，冲突则降级为仅 docs_gen 侧修复
  + schema 漂移挂账移交）。
- T3 W2：vue 轨 lib/api 粘合层生成（crates/auto-man/src/vue.rs 或
  auto-lang codegen 相应环节）+ os-config gen 树再生成 →
  `auto build` tsc+vite 绿。
- T4 W3：generate_desktop_host 守卫放开（api-client 类）+ 粘合层
  注入 + extra roots（crates/auto-man/src/vue.rs）→ desktop-host
  嵌入 os-config 实机（Playwright）。
- T5 W4：Taskbar.vue ⚙️ 按钮 + wm store launch 接线
  （crates/auto-man/assets/wm/Taskbar.vue + store.ts）→ Playwright
  点击验证。
- T6 W5：Desktop 页双端对拍三 shots（vm: p551 驱动复用；vue:
  Playwright）→ 对拍判读（autoui-verifier）。
- T7 W6：ConfigEditor widgets 缓存 + entryAt 覆盖 + picker 渲染分支
  （auto/src/front/config_editor.at + api.at）→ 单测 + 实机。
- T8 W7：autoui_desktop handler 子组件定位（crates/auto-lang/src/
  ui/mcp_server.rs + renderer.rs 注入消费）+ acceptance 场景 p559
  （picker 点选→config.at→热应用）→ 全链绿。
- T9 双端收口：对拍终判 + 文档/spec 回写（465 守卫矩阵更新、551 债
  销号 D1/D3/D4/D5）+ execution_done。

## 复审记录

## 待澄清事项

- W1 与 548 会话的 schema/aura.at 协调窗口：其未提交修改仍在时,
  T2 先对齐再动（避免双头改）；若 548 已落地修复，W1 缩为验证销债。
- W3 守卫放开仅限 api-client 类——ext/i18n/router 三类仍跳过（后续
  波次），需确认无异议。
- daemonBase 注入形态（构建期常量 vs 运行期 env）T1 定稿。
