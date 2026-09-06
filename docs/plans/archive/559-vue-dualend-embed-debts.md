---
plan_id: PLAN-559
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: vue-dualend-embed-debts
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: vue desktop-host v1 守卫契约修订——api-client 类守卫在粘合可安装时放开（gen 树生成粘合或项目 src/back/api.ts 择先，run 内先到先得+每次覆写）；extra roots 兄弟探测（../auto-os-config/auto，id=os-config 与 vm extra_roots_from 对齐，AUTO_DESKTOP_APPS_EXTRA 可覆）"
  - "docs/specs/auto-lang/ui/overview.md: vue codegen 三契约——事件参数 $event.target.{value,checked} 收窄（vue_event_param 单点）；store 组合式跨 store 限定调用 facade 化（AuraStore.sibling_stores+ts_adapter store_bare_heads 自限定裸发）；use back.api 只发 @/lib/api import 行（排除 Plan 522 use-fn 拉取）"
  - "docs/specs/auto-lang/ui/architecture.md: 验收通道 autoui_desktop handler 增 (app, widget) 子组件定位维度——namespaced call_handler_for onclick 同管线（Plan 320 单 VM 统一态下恒根 state id），DesktopPage/WallpaperPicker 等子组件可直驱"
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 项目供给 TS 粘合安装（auto-man api_gen install_project_api_glue）——契约抽取零端点且项目带 src/back/api.ts 手写 web 实现时装入 gen/front/vue/src/lib/api.ts+dist（os-config 首试点，孪生 88+6 导出签入 auto/src/back/api.ts 为真源，regen.sh 镜像 host）"
  - "docs/specs/auto-lang/ui/overview.md: 验收场景 p559——W6 夹具模块（modules.d drop-in，widgets 声明无 view_name）通用编辑器挂载 WallpaperPicker，(app,widget) 直驱 Pick→config.at 断言→已应用态；场景幂等（daemon PUT 基线重置）"
touched_goals:
  - "GOAL-007: 双端一致收尾面——os-config vue 轨构建全绿+Desktop 页/WallpaperPicker/通用编辑器字段级挂载双端同源，Desktop 页双端三 shots 对拍"
  - "GOAL-009: 桌面 Shell——vue 桌面任务栏 ⚙️ 直开 os-config（聚焦-或-启动，vm 551 T2 对齐）+api-client app 以普通窗嵌入 desktop-host（daemon 数据经代理全链）"

affects: [auto-lang/vm, auto-lang/autoui, auto-man/vue, auto-os-config]
current_step: 9
total_steps: 9
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

- [x] master `cargo tf` 全绿（schema_drift_fence + docs_gen 转绿）。
      （T1/T2 实测：双红在主检出已消，scoped 对全绿记档销债；`cargo tf`
      全量复证归 fold 前门禁——skill 纪律全量门只在全量门跑。）
- [x] os-config `auto build` vue 轨全绿（tsc+vite，含 lib/api 粘合生成）。
      （T3 实测 gen 树 tsc+vite 全绿；host `npm run build` 同绿。）
- [x] vue desktop-host 嵌入 os-config：窗口渲染 + 模块数据来自 daemon。
      （T4 Playwright 实证：窗口 System Overview 渲染真实 daemon 数据，
      经 AUTO_HTTP_PROXY 代理 :17701。）
- [x] vue desktop-host 任务栏 ⚙️ → os-config 窗。（T5 Playwright 两次
      点击实证：首击开窗、再击聚焦不重复开窗。）
- [x] DesktopPage vue/vm 对拍三 shots 一致性通过。（T6：vue
      t6_vue_01–03 × vm 559-vm-01–03，顶部标签/外观壁纸卡/Settings 卡
      双端同构；第三交互点口径注记见 T6 标记。）
- [x] ConfigEditor 打开带 widgets 声明的模块 → picker 内联渲染（非平铺
      输入框），点选落盘与手输同构。（T7 drop-in 夹具双端实证；vue 端
      点选 PUT 落盘 daemon GET 断言。dir_picker 声明回退平铺，登记后续。）
- [x] 验收通道 p559 场景：picker 点选 → config.at → 宿主热应用全链绿。
      （T8 实跑 PASS：config.at 断言+「已应用」态 shot。）
- [x] 「tag 表改动必须重生成 schema」流程约定沉淀（设计文档/AGENTS 注记）。
      （沉淀于 KNOWN-DEBT P551-D1 销词条目 + 本 plan T1 笔记①；复审存档
      时随 merge 落 specs。）

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- T1 复核与定位：master tf 双红现状复核（548 是否已修）；vue codegen
  back.api 粘合缺失环节定位（resolve_back_api / codegen.rs import
  处理）；desktop_apps_dir/extra roots 现状盘点。产出：三份定位笔记
  回填本 plan 待澄清。[✅ 已完成] 三笔记已回填待澄清节：双红已消
  （schema_drift 2/2+docs_gen 4/4 主检出实测）/粘合缺失=api_gen
  lenient 抽取零端点（os-config api.at 为 VM 实现式）+形态修正为项目
  供给 TS 粘合/extra roots=aggregate_scan 可复用+auto-down 第三兄弟
  仓实建。
- T2 W1：schema 重生成（SCHEMA_DRIFT_GENERATE_AT=1）+ 复核 diff +
  `cargo tf` 双红转绿（与 548 协调，冲突则降级为仅 docs_gen 侧修复
  + schema 漂移挂账移交）。[✅ 已完成] 按 D1 判定「已绿则记档销债」：
  双红在主检出已绿（schema_drift 2/2+docs_gen 4/4，无 crates/schema
  未提交改动），免重生成；cargo tf 全量复证归 fold 前门禁（skill 纪律：
  全量门只在全量门跑）。
- T3 W2：vue 轨 lib/api 粘合层生成（crates/auto-man/src/vue.rs 或
  auto-lang codegen 相应环节）+ os-config gen 树再生成 →
  `auto build` tsc+vite 绿。[✅ 已完成] 四件上收+粘合安装：a)
  $event.target 收窄（vue_event_param 单点）；b) store 跨 store 限定
  调用 facade 化（sibling_stores+store_bare_heads）；c) 项目供给 TS
  粘合安装（api_gen install_project_api_glue：抽取零端点时
  src/back/api.ts→gen lib/api.ts+dist）+use back.api 排除出 use-fn
  拉取（TS2440/TS2304 根修）；os-config 侧 api.ts 孪生落位
  auto/src/back/api.ts（+5 fn 移植/types 内联/pickModule 补齐）+
  regen.sh 镜像行。实测：os-config `auto build` tsc+vite 全绿、host
  `npm run build` 绿、部署树补齐 DesktopPage/WallpaperPicker/
  useDesktopCfgStore。
- T4 W3：generate_desktop_host 守卫放开（api-client 类）+ 粘合层
  注入 + extra roots（crates/auto-man/src/vue.rs）→ desktop-host
  嵌入 os-config 实机（Playwright）。[✅ 已完成] 守卫放开（gen 粘合
  /项目 src/back/api.ts 择先，run 内先到先得每次覆写）+ extra roots
  （默认探测 ../auto-os-config/auto，id=os-config 对齐 vm 轨，
  AUTO_DESKTOP_APPS_EXTRA 可覆）；实机：组目录 scratch 宿主项目
  desktop-host，39 扫描/32 嵌入/os-config 粘合安装，Playwright
  summon→search→launch 全链，窗口渲染 System Overview+真实 daemon
  数据（AUTO_HTTP_PROXY 代理 :17701；注：auto run 的 vite 子进程不吃
  该 env，实证用产物目录裸 vite 等价服务）。
- T5 W4：Taskbar.vue ⚙️ 按钮 + wm store launch 接线
  （crates/auto-man/assets/wm/Taskbar.vue + store.ts）→ Playwright
  点击验证。[✅ 已完成] ⚙️ 按钮（emit settings，os-config 条目在场
  性门控）+ 宿主 App launchSettings 聚焦既有窗否则启动（vm 551 T2
  对齐）；Playwright 两次点击实证：首击开窗（daemon 数据）、再击单窗
  聚焦。
- T6 W5：Desktop 页双端对拍三 shots（vm: p551 驱动复用；vue:
  Playwright）→ 对拍判读（autoui-verifier）。[✅ 已完成] vue 侧三
  shots（t6_vue_01_dock/02_appearance/03_picker_scan，desktop-host
  Playwright）；vm 侧经 W7 注入面补拍（559-vm-01/02/03：dock/外观/
  light 切换）——裸 settings 槽 Nav 不可达子组件为历史缺口（551-02
  证据实为 dock，本案 W7 收口后 DesktopPage.Nav 首次生效）。判读：顶
  部横向标签导航、外观页壁纸卡结构、侧栏 Settings 卡（THEME/ACCENT）
  双端同源一致；第三交互点 vm=主题翻转 / vue=扫描目录（vue 端 Pick 链
  已由 p559 场景独立实证），登记为口径注记非缺口。
- T7 W6：ConfigEditor widgets 缓存 + entryAt 覆盖 + picker 渲染分支
  （auto/src/front/config_editor.at + api.at）→ 单测 + 实机。[✅ 已完成]
  entryAtW×2（vm merged 真源 auto-os-config-back/api.at 漏改即 launch
  不可用——实测定位后同步）+ConfigEditor widgets prop（app.at 传
  Modules.active_widgets，装载一次零额外 HTTP，优于计划「一次 HTTP」
  字面）+wallpaper_picker 分支（dir_picker 回退平铺登记）；TS 孪生
  entryAtW。验收夹具 modules.d/p559-fixture.at（drop-in 热注册+两枚
  生成 PNG）双端实证：通用编辑器 picker 栅格渲染+vue 点选 PUT 落盘
  （daemon GET 断言 aqua.png）。
- T8 W7：autoui_desktop handler 子组件定位（crates/auto-lang/src/
  ui/mcp_server.rs + renderer.rs 注入消费）+ acceptance 场景 p559
  （picker 点选→config.at→热应用）→ 全链绿。[✅ 已完成] DesktopInject
  .Handler 增 widget 维度+DynamicComponent.call_widget_handler
  （Plan 320 单 VM 统一态下恒根 state id 的 namespaced 派发）+mcp
  schema；验收通道 handler_widget+场景 p559 实跑 PASS（Pick plum→
  config.at 断言绿→「已应用」态 shot）。
- T9 双端收口：对拍终判 + 文档/spec 回写（465 守卫矩阵更新、551 债
  销号 D1/D3/D4/D5）+ execution_done。[✅ 已完成] 对拍终判见 T6；
  KNOWN-DEBT P551-D1/D3/D4/D5 销号（D2 附 559 worktree 组解法注记）、
  「tag 表改动必须重生成 schema」流程约定沉淀于 D1 销词条目+本 plan
  T1 笔记①；465 守卫矩阵更新以 W3 实现注记形式落在桌面 scaffold 段
  （api-client 类守卫=粘合可安装时放开，ext/i18n/router 三类不动）。
  scoped 验证：cargo check auto-lang/auto-man 0 error、ui_gen
  744/745（唯一红=master 既有 charts 存量，stash 复核与本计划无关）、
  auto-man api_gen 27/27。

## 复审记录

- **复审人**：ZCode（/auto-plan:review，独立复验——verify, don't trust）
- **复审时间**：2026-09-05
- **复审基点**：worktree `.wt/lang-559/auto-lang`@plan-559-dev（merge-base
  5ff92f364，7 提交 +640/−17，13 文件）与 `.wt/lang-559/auto-os-config`
  @auto-lang-dev（2 提交 +2346/−108，15 文件）。注：执行期间 master 前进
  （548/552/555/062 折叠+560/561/562 立项），两-dot diff 会混入 master 反向
  ——真实 diff 以 merge-base 口径为准；本分支与 folds 无交叠冲突。

### 逐条验收复验

| # | 验收 | 判定 | 复验证据 |
|---|---|---|---|
| 1 | master tf 全绿（双红收口） | **PASS** | 复审全量门禁 `cargo tf`（worktree）：**3427 跑 3426 绿**，唯一红=`test_charts_gallery_compiles`=master 存量甄别（552/555 复审同判；执行期 stash 对照证实与本计划无关）。首轮 tf 因 nextest fail-fast 少跑 838，已 `--no-fail-fast` 补全量读数 |
| 2 | os-config `auto build` vue 轨全绿 | **PASS** | 执行期 gen 树 tsc+vite 全绿 + host `npm run build` 绿；工作树此后未变更，读数有效 |
| 3 | desktop-host 嵌入 os-config（daemon 数据） | **PASS** | 复审活体重演：daemon :17701 + desktop-host + 代理 vite，⚙️→窗口 System Overview 渲染真实 daemon 数据（r_vue_gear_window.png） |
| 4 | 任务栏 ⚙️ → os-config 窗 | **PASS** | 同上活体重演；按钮在场性门控+聚焦-或-启动语义见代码 Taskbar.vue/generate_host_app_vue |
| 5 | Desktop 页双端三 shots 一致 | **PASS** | vue 复审新证 r_vue_desktop_dock.png（dock 段真实数据）+ 执行期 t6_vue_01–03 × vm 559-vm-01–03；结构判读：顶部标签/外观壁纸卡/Settings 卡双端同源。口径注记：第三交互点 vm=主题翻转、vue=扫描目录（vue 端 Pick 链由 p559 独立覆盖） |
| 6 | ConfigEditor picker 内联渲染+落盘同构 | **PASS（口径注记）** | p559-01 shot：夹具模块通用编辑器内 WallpaperPicker 栅格（非平铺）；Pick→PUT 落盘 daemon GET 断言。注①：W6「单测」腿——entryAtW 为 .at 层代码无 rust 单测面，以 p559 e2e+双端夹具覆盖替代；注②：dir_picker 声明回退平铺（无 DirPicker widget，登记后续波次） |
| 7 | p559 场景全链绿 | **PASS** | 复审重跑 PASS（含幂等修复：场景先经 daemon PUT 重置基线再 Pick，修复了复跑 before==after 假绿——该缺陷为本复审发现并当场修复） |
| 8 | 「tag 表改动必须重生成 schema」约定沉淀 | **PASS** | KNOWN-DEBT P551-D1 销词条目+本 plan T1 笔记① 在案（主检出 9804a388b） |

### 补充门禁（改动面专项）

- `cargo t desktop_protocol --features ui-iced`（Plan 507/531 复审清单项，
  W7 触碰 renderer/session/dynamic）：**120/120 绿**。
- `cargo nextest run -p auto-man`（api_gen 粘合安装+vue.rs 守卫/extra roots）：
  **245/245 绿**。
- ui_gen 家族（T3a/T3b 新测试在内）：tf 全量内全绿；唯一红 charts 与本计划
  改动面（ui/layout、icons、strip_html、c2_param、d8）零交集。

### 遗漏/延后/workaround 猎查

- **遗漏**：无。13+15 文件 diff 与 9 任务逐一对应；无任务丢子项。
- **延后（已登记，无未授权项）**：①ext/i18n/router 三守卫类放开——计划文字
  明示范围外（待澄清第 2 条）；②DirPicker widget——计划未承诺，夹具的
  dir_picker 声明按回退平铺处理并在 T7 标记登记；③013/015 等 Notes 族
  api-client app 因无粘合诚实跳过（T4 日志在案）。
- **workaround/债候**：
  - **P559-D1（新债候）**：`auto run` 在 Windows 上不把 AUTO_HTTP_PROXY 透传
    给其 vite 子进程——desktop-host 嵌入 api-client app 后数据面开箱需手动
    以 env 裸起 vite（复审活体复验即此形态）。上游属 env 注入臂缺口，建议
    后续波次收口（vm 轨有 AUTOOS_DAEMON 注入先例）。
  - **P559-D2（债候）**：regen.sh 部署侧 sed 中事件 cast/Collection.Init
    重写两族已被 codegen 上收，现冗余（幂等无害）；建议下轮 os-config 清理。
  - p559 场景幂等缺陷（before==after 假绿）——本复审发现即修（PUT 基线
    重置），非遗留。

### 结论

七验收 + 补充专项全 PASS（第 6 条带两条口径注记、第 1 条以 scoped+全量门禁
复合判定），零未授权延后、零本计划引入回归；两债候（P559-D1/D2）均为
非阻断上游/清理项。**路由：`reviewed`** —— 可交 `/auto-plan:merge`。
（fold 提示：master 已前进（548/552/555/062 折叠），merge 时需先同步
master 再折，预计无实质冲突；tf 基线唯一红=charts 存量与 555 复审读数
3428/3429 同源。）

### 债务收口附记（2026-09-05，merge 前用户裁定先清两债）

- **P559-D1 复验证伪**：干净环境（杀净全部 node/auto，daemon 独占 17701）
  下 env 内联 `auto run --desktop` 的 vite 代理 `/api` 实测 200 + ⚙️→
  os-config→daemon 数据端到端活体（scratch/p559/d1_e2e_osconfig_data.png）。
  pkg.rs run_script_live 零 env 操纵、std 默认继承——「不透传」不成立；
  原 404 为旧无 env vite 残留占 3000 + vite auto-increment 静默漂移的假象。
  台账改判细节见 KNOWN-DEBT P559-D1（含 strictPort/daemon 默认端口两条
  小口径注记，不立案）。
- **P559-D2 已清**：regen.sh 删两族已上游化的 sed（事件 cast 四行 ×2 处、
  plan010 R10 与 plan446 VG16 两块）；清理后全量 regen 重跑部署树零漂
  （git status 仅剩 regen.sh 自身），host `npm run build` 绿。

复审结论维持 `reviewed` 不变。

## 待澄清事项

- W1 与 548 会话的 schema/aura.at 协调窗口：其未提交修改仍在时,
  T2 先对齐再动（避免双头改）；若 548 已落地修复，W1 缩为验证销债。
- W3 守卫放开仅限 api-client 类——ext/i18n/router 三类仍跳过（后续
  波次），需确认无异议。
- daemonBase 注入形态（构建期常量 vs 运行期 env）T1 定稿。

### T1 定位笔记（2026-09-05 实测回填）

1. **W1 双红现状：已消，记档销债。** 主检出（master HEAD 5ff92f364 +
   无 crates/schema 未提交改动）`cargo test -p auto-lang --test
   schema_drift --test docs_gen` 全绿（schema_drift 2/2、docs_gen 4/4）。
   551 复审档 5428d7f2b 所记 tf 2 红为当时主检出残留 548 会话未提交
   改动的瞬时态；现 548 分支（plan-548-dev，未合 master）工作树亦无
   schema 改动。W1 缩为验证销债；cargo tf 全量复证归 fold 前门禁。
2. **W2 粘合缺失确切环节 + 形态修正。** `auto build -d . --gen-only`
   实证：API client 生成跑了但 `⚠ No API endpoints or types found`
   （api_gen.rs generate_vue_api → extract_api_lenient 在 os-config
   back/api.at 上抽出零端点——该文件是 **VM 实现式**（80 个 fn、
   http/json/Env 内建配方），非 015-notes 式契约式），故
   gen/front/vue/src/lib/api.ts 未写入，gen 树 tsc 8 处
   `Cannot find module '@/lib/api'`。**形态修正**：方案要点 2 的
   「签名提取 + fetch 映射」不足以覆盖 35+ 个纯文本/JSON 逻辑 fn；
   改为**项目供给 TS 粘合**：vue 轨在契约抽取为空但 front 有 back.api
   导入时，安装 `<auto>/src/back/api.ts`（项目手写 TS 孪生）到 gen 树
   lib/api.ts。os-config 的孪生已存在 88 导出（host src/lib/api.ts,
   1094 行），仅缺 551 壁纸面 5 fn（fetchDesktopCfgSafe/cfgField/
   listImagesSafe/imageCount/imageAt，VM 体均为配方形，人工移植）。
   同轮 upstream 三件（gen 树 tsc 其余红）：①`$event.target` 未收窄
   （TS2339/TS18047，regen.sh 部署侧 sed 既有补偿→上收 codegen）；
   ②跨 store 裸名调用（TS2552/TS2304，Collection/DesktopCfg，部署侧
   sed 同族→上收 codegen）；③粘合安装机制本身。daemonBase 定稿：
   粘合 TS 内 base=相对 `/api`（vite proxy 同源）+ `AUTOOS_DAEMON`
   构建期常量覆盖（desktop-host 注入用，515 G3 同款）。
3. **desktop-host/extra roots 盘点。** `generate_desktop_host`
   （auto-man/vue.rs:3406）守卫四类：api-client（`@/lib/api`、
   `from '@/api`）/router/ext/i18n，首类即本 plan 放开对象；
   `desktop_apps_dir`（:5134）= AUTO_DESKTOP_APPS env → examples/ui
   单根；scan_apps 单目录扫描；vm 轨已有 `aggregate_scan`+
   `host_extra_roots`（app_registry.rs:247，主根 examples 优先 id
   去重，`../auto-os-config/auto` 探测）可同构复用到 vue 轨。
   Taskbar.vue（auto-man/assets/wm/）按钮族 summon/窗控/布局/alt-tab，
   ⚙️ 槽位清晰，store.ts launchWindow(appId,title,comp) 动态 import
   即落点。worktree 组实建需第三兄弟仓 auto-down（workspace 成员
   a2r-actor-tests→autodown-core 路径解析），分支 auto-lang-559-dev。
