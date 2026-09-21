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
current_step: 6
total_steps: 6
---

# [PLAN-672] ui-gallery-vue-fix-batch（画廊修复批·滚动跟踪）

## 变更摘要

**本计划为滚动跟踪批**（用户裁定 2026-09-21）：此后对 ui-gallery 的所有修改建议
统一记入本计划——**每次先更新本文件（新增条目/任务/AC），再进 worktree 修复**。
批内条目独立验收，全部清偿后归档。

- **条目 1（用户提出 2026-09-21）**：Vue 臂画廊中 app 未在视口内居中（VM 臂已居中）。
- **条目 2（本会话发现 2026-09-21）**：027-file-manager 触发 vite `vue-sonner`
  解析失败 → dev server 整站崩溃。根因 = PLAN-528 W6 宿主自愈重写冲掉画廊合并依赖。
- **条目 3（用户提出 2026-09-21）**：内容超高出现滚动条时（滚动行为本身 ✓），
  Vue 臂用的是浏览器默认滚动条（宽轨+箭头，暗色主题下突兀），应为 AutoUI
  风格滚动条（细/半透明/圆角，与 reka-ui ScrollArea 观感一致）。
- **条目 4（用户提出 2026-09-21，诊断+修法定稿）**：013-todo 在 Vue 臂显示
  「全栈/原生应用」独立运行提示，用户预期可内嵌。**用户追问 proxy 机制裁定
  修法方向：Vue 臂接线 PLAN-658 back-proxy**。诊断补充：PLAN-658 多后端
  proxy 已实现且在档（delivered/archived）——`start_gallery_back_proxy`
  （vue.rs:6710）按 `src/back/api.at` needs_session 为 013/015 等建 VM 会话，
  按 `/apps/<app_id>/api/*` 前缀分派——但**唯一调用点在 rust_ui.rs:3444
  （VM 臂宿主）**。Vue 臂三缺：① proxy 未启动；② vite `/api` 代理指向
  宿主自身 api 端口（画廊无 api → 死端口），且无 `/apps` 路由；③ fullstack
  语料未发射（generate_gallery_host 仅 `row.loadable` 档写 App/stores/
  components，画廊 `lib/` 连 api.ts 都没有）。VM 臂源侧前缀化
  （prefix_api_url_literals）不适用于浏览器 fetch（跨源）；Vue 臂走
  vite 代理透传（同源）→ 保持 demo 语料相对路径风格，api.ts 内路径前缀
  改写为 `/apps/<id>/api/`。

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

### T-05 视口滚动条 AutoUI 化（条目 3）

现状：`.demo-mount-root` 的 `overflow-auto` 出浏览器默认滚动条。宿主
`generate_index_css`（vue.rs:1710-1720，Plan 053 后续）已内置 AutoUI 风格
滚动条类 `.ash-scroll`（8px/透明轨/`--border` 圆角拇指/hover `--muted-foreground`，
Firefox `scrollbar-width: thin`+`scrollbar-color`）——注释载明其视觉对齐
reka-ui ScrollArea 且随明暗主题。画廊 gen index.css 已含全套（实测 16 处），
os 仓无 handmade 覆盖。

修法：模板 `AppViewport.vue` 的 demo-mount-root div 追加 `ash-scroll` 类
（零 CSS 重复，纯复用宿主样式）。范围限视口自身滚动条；demo 内部滚动容器
为 demo 代码自身职责，不在本条目。

（`.ash-scroll-fade` 悬浮淡入变体已否决：VM 臂 iced ScrollArea 滚动条常显，
demo 展示窗保留滚动可供性更利于两臂观感一致。）

同步 os 参照副本 + gen ext 副本字节对齐（同 T-04 收口路径，P672-D1 债仍在册）。

### T-07/T-08 Vue 臂 back-proxy 接线（条目 4）

复用 PLAN-658 全套（proxy 会话分派/行缓存/降级语义），三处接线全在 auto-man：

**T-07 proxy 启动 + vite 路由**
- `run_vue_project`（vue.rs:5694）gallery 分支（`is_ui_gallery || gallery_mode`）：
  在 `generate_gallery_host` **之后**（行缓存热）调 `start_gallery_back_proxy`，
  拿端口后为 vite 子进程注入 env `AUTO_GALLERY_BACK_PROXY=http://127.0.0.1:<port>`；
  proxy 启动失败照 658 语义降级不阻断（env 不注入，fullstack demo 走错误横幅）。
- `generate_vite_config`（vue.rs:790）：追加条件条目——env 存在时
  `'/apps': { target: env, changeOrigin: true }`（spread 写法，未设=零条目，
  standalone 项目 vite.config 零变化）。

**T-08 fullstack 语料发射 + 注册表翻转**
- `generate_gallery_host` 发射循环（vue.rs:4605 `if row.loadable`）→
  `if row.loadable || row.fullstack`；fullstack demo 追加两步：
  - 语料改写：`from '@/lib/api'` → `from '@/apps/<id>/lib_api'`（app/store/
    components 三处语料同改）；
  - api.ts 落盘：读 `<app_root>/gen/front/vue/src/lib/api.ts`（api_gen 产物，
    桌面宿主循环 :4376 同源先例；缺则回落 `src/back/api.ts` 胶水，再缺则
    该 demo 跳过并告警），fetch 路径改写 `` `/api/ `` → `` `/apps/<id>/api/ ``
    （生成模板定格式，单replace；写入 `apps_src/<id>/lib_api.ts`，per-demo
    隔离避免共享池撞名）。
- `generate_demos_registry`：TS registry `loadable` 字段写
  `row.loadable || row.fullstack`（侧栏「可交互」徽章随之翻转，AppViewport
  mountApp 放行）。registry.at / AppViewport.vm.at 的 VM 侧语义**不动**
  （:6441 VM 早已按 loadable||fullstack||route_stub）。
- 边界：本条目验证面 = 013/015（纯 CRUD）。SSE 流 demo（017-chat 族）随档位
  一并发射，但流消费在 Vue 臂未经 PLAN-658 的 Tick 注入改造——若运行期断流
  降级为错误横幅（与 658「失败降级不阻断」同语义），不强保。

### T-09/T-10 门禁与端到端（条目 4）

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
- [x] AC-6 内容超高时视口滚动条为 AutoUI 风格（细 8px/圆角/`--border` 主题色
      拇指），不再是浏览器默认宽轨；滚动行为本身不回归（AC-3 复验）。——
      证据：T-06 计算样式+截图。
- [ ] AC-7（条目 4）013-todo 在 Vue 臂画廊内加载且 CRUD 可用（新增/勾选待办
      经 vite `/apps` 代理 → back-proxy 会话持久生效）；015-notes 同面加载。
      —— **部分达成**：加载/读/创建/持久 ✓（T-10 证据）；删除与更新受阻于
      P672-N5（proxy 会话路径参数绑定，非本计划回归），修复后闭环。
- [ ] AC-8（条目 4）代理未运行时（env 未注入）fullstack demo 降级为错误横幅
      或空态，不崩溃、不影响其他 demo；standalone 项目 vite.config 零变化
      （env 条目未设不出现）。—— 结构性满足：/apps 条目 env 门控（未设即
      零条目）；无 api 源 fullstack 直接回退独立提示（047 实测）；代理死时
      fetch 失败走错误横幅（实测窗口期现过 500 横幅，页面不崩）。

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
    200）；横幅「Env is not defined」= 独立预存缺陷（P672-N3，条目 3 候选，
    后用户提出条目 3 为滚动条样式，Env 守门改列 P672-N3 观察）。
- [x] T-05（条目 3）模板 demo-mount-root 追加 `ash-scroll` 类 + 契约测试扩面。
  [✅ 已完成] worktree 9d84d40a6——模板 :139-142 挂类+注释；契约测试改为单断言
  `demo-mount-root ash-scroll w-full h-full flex flex-col`（首次提交断言子串
  失配被自家测试抓出，修正后过）。
- [x] T-06（条目 3）os 参照副本/gen ext 副本对齐 → 浏览器复验 AC-6（滚动条
  计算样式+截图）→ 门禁复跑 → 提交。
  [✅ 已完成] os `dcbeca4`（src 参照）+ gen ext/src/gallery 双副本字节对齐
  （vite HMR 生效路径）。复验证据：`getComputedStyle(demo-mount-root)` =
  `scrollbar-width: thin` / `scrollbar-color: rgb(30,41,59) transparent`
  （=主题 `--border` 解析值，非浏览器默认）；009 滚动行为零回归
  （scrollDelta=247、topGap=1）；009 满宽子项 centerHDelta=-5px=细滚动条占位
  半宽（预期几何，非缺陷）；截图右缘细圆角拇指在档。门禁复跑：gallery_assets
  契约测试绿，auto-man 315 例唯一红仍为预存 plan593_index_css_golden。
  附带证据：HMR 全量重载后 002-counter 居中保持（条目 1 修复对重载鲁棒）。
- [x] T-07（条目 4）run_vue_project 画廊分支启 proxy + env 注入；
  generate_vite_config 加 /apps 条目；vite.config 每 run 自愈。
  [✅ 已完成] worktree 46abb6565——gallery 分支 `generate_gallery_host` 后调
  `start_gallery_back_proxy`（行缓存热），端口 set_var
  `AUTO_GALLERY_BACK_PROXY`（失败 remove_var 降级）；vite.config 追加
  env 门控 `/apps` 条目；自愈块（Plan 548 同律）补 vite.config.ts 每 run
  重写（实测增量路径不重写导致 `/apps` 丢失 → proxy 起了也 404）。
- [x] T-08（条目 4）fullstack 语料发射 + api.ts 路径改写 + TS registry 翻转。
  [✅ 已完成] worktree 46abb6565——发射循环 `row.loadable || row.fullstack`；
  fullstack 语料 `@/lib/api` → `@/apps/<id>/lib_api`（单/双引号双改写）；
  api.ts 源=gen api.ts（回落 src/back/api.ts 胶水），fetch 路径
  `` `/api/ `` → `` `/apps/<id>/api/ ``，落盘 apps/<id>/lib_api.ts；
  **无 api 源的 fullstack demo 回退独立提示**（实测 047-bp-admin 无源，
  首版发射致 vite build lib_api 断链，改判不可内嵌后消解）；TS registry
  在发射成功分支置 `row.loadable = true`（generate_demos_registry 单看
  loadable；registry.at VM 侧 loadable||fullstack 并集语义不变）。
  proxy 会话准入扩 fullstack（`back_needs_session || fullstack`——普通
  CRUD back 此前不建 session 因 VM 臂走 inproc；Vue 臂 fetch 必须有会话）。
- [x] T-09（条目 4）门禁复跑 + 重建二进制。
  [✅ 已完成] `cargo check -p auto-man` 干净；auto-man 315 例 314 绿
  （唯一红=预存 plan593）；plan633_fullstack 4/4 绿（--features ui-iced）；
  back_proxy_tests（th 档真 TCP）未跑——back_proxy.rs 本体零改动，proxy
  行为由 T-10 实机覆盖。
- [x] T-10（条目 4）端到端：013/015 经 vite /apps 代理 CRUD 实测。
  [✅ 已完成] 实测（worktree 二进制重启画廊）：`Gallery back-proxy:
  http://127.0.0.1:3358 (33 apps, 6 sessions)`（013-todo 5 路由/015-notes
  8 路由/017-chat/047/025/031 会话全起）；vite.config 带 `/apps` env 条目；
  `POST /apps/013-todo/api/todos → 200` 且持久（后续 GET/Init 返回探针
  条目）；013 内嵌渲染种子+居中 0/0+交互底栏（3 items left/筛选器）；
  015-notes GET 200 种子返回。**AC-7 部分达成**：加载/读/创建 ✓，删除/
  更新受阻=P672-N5（proxy 会话路径参数 `:id` 疑 str 绑定，DELETE 200 但
  无效——PLAN-658 会话面从未被真实消费过，非本计划回归）；UI Enter 新增
  待 standalone 差分=P672-N4（IAB 只发 input 不发 keyup，合成事件亦未触
  发 Vue handler，疑 codegen 层 keyup 绑定问题，与嵌入无关）。

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
- **P672-N3（观察，待用户裁定）**：027-file-manager 在 Vue 画廊内加载
  后报「Env is not defined」运行错误横幅（原生 Env 依赖未守门/未降级；025
  档位=registry-only 的同类问题）。修复方向：加载前探测/降级为非 loadable。
- **P672-N4（观察，条目 4 伴随）**：013 UI 的 Enter 新增在嵌入态未触发
  （IAB 自动化只发 `input` 不发键盘事件为已知限制，但合成 keyup 亦未触发
  Vue handler、CUA 真实键盘同）——疑 codegen 层 `@keyup.enter` 绑定问题，
  与画廊嵌入无关。待 standalone 差分定位后决定归属（可能并入 671 批
  vue 生成器缺口账）。
- **P672-N5（观察，条目 4 伴随，阻断 AC-7 闭环）**：back-proxy 会话的
  路径参数路由（`/api/todos/:id` DELETE/PUT）返回 200 但变更不生效——
  `:id` 疑以 str 参与比较（同 PLAN-669 vm-server 实参绑定同族问题）；
  POST/GET（无路径参数）完全正常。PLAN-658 会话面此前无真实消费方，
  条目 4 首次点亮即暴露。修复方向：back_proxy 会话分派的路径参数按
  #[api] 签名类型绑定（669 先例）。
- **P672-N2（升级）**：托管后台下 `auto run` 包装进程不定时 exit（本次
  多轮复现，exit 1/0xC00004），**in-process back-proxy 随之死亡**——
  条目 4 前包装进程退出无害（仅丢 watcher），此后 N2 升级为「代理随宿主
  亡」的结构性弱点。用户终端前台跑不受影响（进程常驻）；根治方向=proxy
  独立子进程化或 N2 归因，另账处理。
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
