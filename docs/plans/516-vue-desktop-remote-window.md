---
plan_id: PLAN-516
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: vue-desktop-remote-window
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-02

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 修改——桌面线增 Plan 516 条目：vue 桌面远程窗 remote_window（508 渲染器包经 auto-man wm_assets 编译期 include_str! 拷贝物化 src/wm/remote-renderer/，包零改动）——远程 App 作为 kind:remote 虚拟窗进 465 WM 同权混排（布局/任务栏/键盘路由/断线保留）；<apps_dir>/remote-apps.json 配置 + URL 注入（?remote/app/title/rbudget）双通道，boot 建连失败降级状态面不阻断桌面启动"
  - "docs/design/autoui/desktop-protocol-v1.md: 修改——RenderQueue 线增 vue 桌面远程窗条目 + 尺寸协商注记（v1 以 vue 侧为源不回传 resize，Welcome.rect 为 canvas 位图源空间；远端跟随语义协议号位预留未占）"
new_spec_components:
  - "crates/auto-man/assets/wm/remote.ts: 新增——远程会话切片（reactive 会话状态/帧缓存 rAF 末帧合帧/指针与字符输入回发/bootRemoteApps 降级建连/断线窗口保留/关闭经 setRemoteCleanup 钩子回收）"
  - "crates/auto-man/assets/wm/RemoteWindow.vue: 新增——远程窗 wm 叶（canvas 位图=Welcome 尺寸×DPR/四态连接状态面/指针 HitTable 命中回发/可打印字符回发/__p516 e2e 探针）"
  - "crates/auto-man/wm-test/: 新增——vue WM 运行时资产 vitest 测试位（T1 组件 5 + T2 store 5 + T3 输入 4 = 14 绿；resolveId 直指渲染器包源，与物化拷贝双向钉漂移；FakeWebSocket+服务器消息字节构造 helpers）"
touched_goals:             # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "GOAL-009: 虚拟桌面与桌面 Shell——508 远程渲染解锁的用户出口兑现：远程 App 进 vue 虚拟桌面 WM 同权混排（远程访问形态），Playwright 全链闭环（帧渲染/点击回发/拖动/任务栏聚焦/断线保留）"

affects: [auto-lang/ui]
current_step: 8
total_steps: 8
---

# [PLAN-516] vue 桌面 remote_window 集成——远程 App 进虚拟桌面

## 变更摘要

兑现 508 非目标预留的"vue 桌面深度集成"：把 Stage 6 落地的
`packages/drawlist-renderer/`（TS/Canvas2D 渲染器：codec/messages/render/
connect 四模块，Rust↔TS golden 对拍）接进 **vue 虚拟桌面宿主（465）**——
远程 App（桌面协议 WS 会话）作为**一种虚拟窗类型**进 vue 桌面的 WM
（store/layout/键盘/任务栏），而非 508 demo 页的独立窗口形态。

场景定位：**远程访问形态**——用户在浏览器/tauri 壳里打开 vue 虚拟桌面，
桌面里的某个"App"实际运行在远端（或本机另进程）的桌面宿主上，帧以
DrawList command 流过来，输入经 HitTable 回发。这是 508"远程渲染解锁"
面向用户的出口。

四块：

1. **RemoteWindow 组件**（`packages/widgets/` 的 wm 族，VirtualWindow
   同级）：canvas + drawlist-renderer connect + 会话状态面（连接中/
   在线/断线重连）。
2. **WM 集成**：remote 会话注册进 vue 桌面 store（窗口条目/任务栏图标/
   布局参与/键盘路由透传），与本地 App 虚拟窗同权混排。
3. **会话配置 v1**：桌面配置声明远程条目（url/token/目标 app）——配置
   注入形态（URL 参数或桌面配置文件），设置 UI 后置。
4. **端到端验收**：vue 桌面开一个远程窗渲染 002-counter，点击闭环
   （Playwright），且窗口受 vue 桌面 WM 管理（拖动/任务栏聚焦）。

## 目标

- **G1 组件**：`RemoteWindow.vue`（wm 族）——canvas 尺寸随虚拟窗 rect、
  DPR 处理、帧渲染、连接状态面（508 ReconnectPolicy 语义）。
- **G2 WM 同权**：远程窗进 store 窗口表（布局/任务栏/键盘路由/最小化），
  与本地窗无差别管理；断线时窗口保留+状态面可见。
- **G3 输入回发**：canvas 指针/键盘事件 → HitTable 命中 → InputMsg
  编码回发（渲染器包既有能力，接线到 vue 桌面事件路由）。
- **G4 配置**：v1 桌面配置声明远程条目（url/token/app 名），boot 时建连；
  无配置零行为变化。
- **G5 端到端**：Playwright——vue 桌面 + 远程窗渲染 002-counter，点击
  button → 远端 revision 递增 → 帧文本变化断言；拖动窗口/任务栏聚焦
  留痕。
- **非目标**：远程会话设置 UI；多远程窗性能优化；`remote_window` 的
  .at widget 登记（v1 宿主内置面——待澄清①评估升格）；TLS/跨网鉴权
  （508 边界沿用：回环+token）；移动端。

## 架构方案

```
vue 虚拟桌面宿主（465：store/layout/keyboard/Taskbar/VirtualWindow）
  └─ RemoteWindow.vue（新，wm 族）
       ├─ packages/drawlist-renderer（508：connect/renderFrame/hitTest/InputMsg）
       ├─ WS ──▶ 桌面宿主 :17800（508 listener，回环+token）
       └─ 远端 App（queue 臂 DrawList 帧，507 Tier1+2 覆盖集）
WM 集成：store 窗口表增 kind:"remote" 条目（title/icon/rect 状态同本地窗）
```

- **落点**：`packages/widgets/src/wm/RemoteWindow.vue` + store 扩展
  （remote sessions 状态切片）+ 桌面宿主入口配置消费；渲染器包零改动
  （预期——若需扩 API 走其包自身测试链）。
- **形态判定**：v1 为**宿主内置面**（与 Taskbar/dock 同级），不做 .at
  widget 登记——I4/I8 纪律针对 shell 表面与 widget 声明源，宿主能力面
  不强制同源；升格 .at widget 的条件记待澄清①。

## 技术栈

既有（drawlist-renderer 包 + vue 桌面宿主 + Playwright）。零新依赖。

## 需求分析与背景调查

（取材 508 归档交付 + 465 宿主机制 + 现场核验 2026-09-01）

- **508 交付**：`packages/drawlist-renderer/`（codec/messages/render/
  connect；Hello/Welcome/FrameReady/HitTable tag9/InputMsg 编解码；重连
  ReconnectPolicy 对齐；Rust↔TS golden 双侧对拍防漂移）+ 宿主 WS listener
  （:17800 回环+token）+ demo 页与 002-counter 远程闭环（Playwright 已有
  一条）。
- **465 宿主机制**：vue 虚拟桌面 = store/layout/keyboard/VirtualWindow/
  Taskbar（`@/wm/VirtualWindow`，aura.at:5070 登记）；桌面宿主配置/装载
  机制承载新窗口类型扩展点。
- **与 508 demo 的差异**（本计划的增量本质）：demo 页是独立单页；本期把
  远程窗**纳入 WM**——虚拟窗 rect 驱动 canvas、任务栏条目、布局混排、
  键盘路由、断线保留。
- **前置**：508 ✅（渲染器包与 listener 就绪）；507 ✅（Tier1+2 覆盖集
  即远程可渲染集）。
- **排程**：vue 线——与 509（Smithay）/513（清理）/515（DEBT 二期）
  零交叠，可并行。

## 详细设计

### 1. RemoteWindow.vue（G1）

- props：会话配置（url/token/appId）+ 虚拟窗 rect（store 注入）；
- 生命周期：mount 建连 → 帧到达渲染（canvas，DPR 缩放）→ unmount 优雅
  断连；连接状态面（spinner/重连倒计时/失败原因）覆盖在 canvas 上。
- 帧节奏：requestAnimationFrame 合帧（多帧取末帧，渲染器包若有节流则复用）。

### 2. store/WM 集成（G2）

- 窗口表条目 `kind:"remote"`（title/icon/rect/minimized 与本地窗同构）；
  布局算法零改动（rect 由 store 统一分配，RemoteWindow 消费）；
- 键盘路由：焦点在远程窗时按键转 InputMsg（KeyExited/CharTyped），本地
  窗语义不变；
- 断线：窗口保留 + 状态面"重连中"；会话终止（远端 Exit）→ 窗口关闭走
  既有销毁链。

### 3. 配置（G4）

- 桌面配置（465 装载机制）增 `remote_apps: [{id, url, token?, title,
  icon?}]`；boot 建连失败 → 窗口仍出现（状态面可见），不阻断桌面启动。
- token 来源：v1 同配置明文（回环场景），跨网安全沿 508 边界外置。

### 4. 端到端（G5）

- 环境：本机桌面宿主（WS listener）+ 002-counter（queue 臂 attach）+
  vue 桌面（dev server 或 tauri 壳）；
- Playwright：断言帧渲染（文本可见）→ 点击 → 文本变化 → 拖动窗口
  rect 变化 → 任务栏聚焦切换；复用 508 e2e 脚本基础设施
  （autoui-verifier 入口）。

## 测试设计

1. **T1 组件测**（vitest）：RemoteWindow 状态机（连接中/在线/重连/失败
   面）、rect→canvas 尺寸、配置缺省零渲染。
2. **T2 store 测**：remote 条目注册/断线保留/关闭销毁；布局混排（本地+
  远程各一）。
3. **T3 输入路由测**：焦点远程窗按键 → InputMsg 编码断言（mock 连接）。
4. **T4 Playwright 端到端**（G5 全链）。
5. **T5 回归**：无远程配置时 vue 桌面行为零变化；508 demo 页与渲染器包
   测试不回归。

## 验收标准

1. T1–T4 绿；T5 零回归。
2. 远程窗与本地窗在 vue 桌面内同权（布局/任务栏/键盘/最小化）演示留痕。
3. 断线重连状态面可见且窗口不消失（演示留痕）。
4. 无配置零行为变化（回归门禁）；渲染器包零改动或改动带其自身测试绿。
5. vue 档测试绿；零新依赖。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **store 扩展**：`packages/widgets/src/wm/`（store 所在）增 remote
   sessions 切片 + `kind:"remote"` 窗口条目 + T2 单测。
   验证：`pnpm -C packages/widgets test`（或既有 vue 测试入口）。
   [✅ 已完成] 路径适配：wm 宿主实际在 `crates/auto-man/assets/wm/`（465 生成器资产，计划括注"(store 所在)"锚定）；store.ts 增 kind/openWindow/setRemoteCleanup，新 remote.ts 会话切片；测试位 `crates/auto-man/wm-test/`（vitest，resolveId 直指渲染器包源）；T2 5 绿（注册/断线保留/关闭销毁/混排/空配置）。
2. **RemoteWindow 组件**：新建 `packages/widgets/src/wm/RemoteWindow.vue`
   （canvas/connect/状态面）+ T1。
   验证：同上。
   [✅ 已完成] `crates/auto-man/assets/wm/RemoteWindow.vue`（canvas 位图=Welcome 尺寸×DPR、四态状态面、指针命中回发、字符回发、__p516 探针）+ T1 5 绿（状态机/dead 面/缺会话静默/位图与末帧渲染/探针 buttonCenters）。
3. **WM 接线**：布局/任务栏/键盘路由消费 remote 条目 + T2/T3。
   验证：同上。
   [✅ 已完成] 布局/任务栏零改动即同权（rect/z/focused 经 wm.wins 统一）；键盘=组件 canvas 回发 + 桌面热键原样（捕获段）；T3 4 绿（指针同字节回发+聚焦/空白不回发/字符同字节/焦点语义）；wm-test 全套 14/14 绿。
4. **配置装载**：桌面宿主入口消费 `remote_apps` 配置（465 装载机制）+
   boot 建连 + 失败降级。
   验证：无配置零变化断言（T5 部分）。
   [✅ 已完成] wm_assets.rs 物化渲染器运行时（include_str! 五件→src/wm/remote-renderer/）；vue.rs：apps-registry 增 REMOTE_APPS（`<apps_dir>/remote-apps.json`，token 并入 url、坏 JSON 降级空表）+ host App.vue 三接线（kind:'remote' 分流/setClient $el/boot=[配置+URL 注入]）；rust 单测 remote_apps_registry_config 绿（空表门禁）+ auto-man lib 243/243 绿。
5. **Playwright 端到端**：T4 全链脚本（复用 508 e2e 基建）。
   验证：脚本绿 + 留痕。
   [✅ 已完成] `examples/remote/viewer/e2e/p516-desktop-remote-e2e.mjs`（508 harness + `auto run --render vue --desktop --apps` + Chromium）七断言 PASS：Welcome+首帧 Counter: 0 → 点击 '+' → Counter: 1（revision 2）→ 拖动 rect 80,80→220,140 → 任务栏聚焦切换（本地+远程混排）→ kill 宿主 → dead 状态面 + 窗口保留。考古注记：vite5 win 绑 localhost(::1)，探测须用 localhost 而非 127.0.0.1。
6. **演示与断线场景**：拖动/聚焦/断线重连状态面留痕。
   验证：截图/录屏留痕。
   [✅ 已完成] 五张截图入 `docs/plans/reports/assets/516/`：01-在线帧渲染、02-点击闭环、03-拖动、04-任务栏聚焦（本地+远程并排）、05-断线 dead 状态面+窗口保留。
7. **回归**：T5 全量（vue 桌面无配置行为 + 508 demo/渲染器包）。
   验证：相关套件绿。
   [✅ 已完成] 四路全绿：e2e 阶段 0（裸桌面零远程窗 + launcher 可用）+ rust 空表门禁（remote_apps_registry_config）；508 demo 页 e2e 复跑 PASS（Welcome/首帧/点击闭环，508 链路零回归）；渲染器包 vitest 20/20；wm-test 14/14。
8. **收尾**：健康检查；状态翻 execution_done。
   验证：vue 档测试绿 + `cargo check -p auto-lang`（如涉 rust 侧零改动
   则仅确认）。
   [✅ 已完成] 健康检查过：cargo check -p auto-man 无本次新增警告（226 行均为 crate 既有）；console.log/warn 均为刻意会话诊断（508 demo 同惯例）；auto-lang 零改动（确认式）；工作树净（5 提交在 plan-516-dev）。

## 复审记录

**复审人**：zhaopuming（/auto-plan:review 独立复审 pass，2026-09-02）
**复审基线**：工作树 `.worktrees/plan-516-dev`（6 提交，merge-base 95aa137b6；master 已被并行 514 会话推进至 11916efc5——`master..HEAD` 裸 diff 会混入 514 反向幻影，已按 merge-base 校正，真实 516 diff = 25 文件 +2762/−10）。

### 验收标准逐条裁定（verify, don't trust——全部复审期独立复跑）

| # | 标准 | 裁定 | 证据 |
|---|------|------|------|
| 1 | T1–T4 绿；T5 零回归 | **PASS** | T1/T2/T3 = wm-test 14/14（复审复跑）；T4 = p516 e2e 复审复跑 PASS（八标记：无配置零变化/在线帧渲染 Counter: 0/点击闭环 Counter: 1 revision 2/拖动 80,80→220,140/任务栏聚焦切换/断线 dead 状态面+窗口保留）；T5 = 渲染器包 vitest 20/20（复跑）+ 508 demo 页 e2e 复审复跑 PASS（welcome/首帧/点击闭环 clicks=1） |
| 2 | 远程/本地窗同权演示留痕（布局/任务栏/键盘/最小化） | **PASS（注记）** | e2e 拖动 rect 变化 + 任务栏聚焦切换 + T2 grid 混排（各占一格不重叠）+ 截图 03/04；键盘路由 = 可打印字符回发（T3 同字节断言）。**注记**：最小化在 465 v1 store 无动词（chrome 黄/绿点为视觉位预留——VirtualWindow.vue:84 注释），远程/本地同等无此能力 = 同权成立，属 465 继承边界非 516 延后 |
| 3 | 断线重连状态面可见且窗口不消失 | **PASS** | e2e kill 宿主 → data-remote-status=dead + `.virtual-window:has(canvas.remote-canvas)` count=1；截图 05 |
| 4 | 无配置零行为变化；渲染器包零改动 | **PASS** | e2e 阶段 0（裸桌面零 canvas.remote-canvas + launcher 可用）+ rust 空表门禁 `remote_apps_registry_config`（REMOTE_APPS 空表断言）+ `git diff 95aa137b6..HEAD -- packages/drawlist-renderer/src/` = **零改动**（复审期核验） |
| 5 | vue 档测试绿；零新依赖 | **PASS（口径注记）** | wm-test 14/14；宿主 scaffold 零新增 npm 依赖（渲染器为编译期拷贝物化，非 npm 依赖）；wm-test devDeps（vitest/vue/@vue/test-utils/jsdom）为测试基建，与 508 渲染器包 vitest 同性质，不计入产品依赖 |
| — | 全量门禁（review 期唯一 full-suite 位） | **PASS** | `cargo tf` 3350/3350 绿（96 skipped 常规位）；tv/tt/tb 未触（无 VM 文件/转译器/book 改动） |

### 遗漏/延后/workaround 猎查

- **遗漏**：无——执行步骤 1–8 均有对应 diff 与验证；G1–G5 全落地。
- **延后**：无未经批准的缩减。④ 尺寸协商 v1（vue 侧为源）与 ② 一窗一会话均为计划内待澄清明示形态；协议文档注记已补（ed22dda56）。
- **Workaround/债候选**（登记，不阻断）：
  - **P516-1 候选** `remote.ts:125`——onLog 行文耦合：渲染器包 connect() 无状态回调 API，以 log 前缀 `disconnected` 推断 reconnecting 态（包零改动约束的代价，代码注释已注明）。包未来若扩 onStatus API 则改订阅。
  - **P516-2 候选（=待澄清⑤）**——键盘回发 printable-only：计划 G3 所引 KeyExited 不在渲染器包 TS 编码子集（仅 PointerPressed/CharTyped）；特殊键/组合键需包扩 encode API（其自身测试链+golden 维护）。
- **裁定注记**：① 落点路径适配（计划 `packages/widgets/src/wm/` → 实际 `crates/auto-man/assets/wm/`，括注"store 所在"锚定，留痕于步骤 1）；② 508 遗失 package.json 重建 + gitignore 反忽略 = 测试位缺件考古修复（fresh clone 可复现性），非行为改动。

### 结论

**通过（pass）**——全部验收标准复验绿，无阻断债；2 项债候选已登记。分支 plan-516-dev（6 提交）待 `/auto-plan:merge` 折叠。

## 待澄清事项

- **⑤ 键盘回发子集（执行注记，复审裁）**：G3 所写"KeyExited/CharTyped"
  中 KeyExited 不在渲染器包 TS 镜像的编码子集（仅
  encodePointerPressed/encodeCharTyped）；v1 落线 = 可打印字符回发
  （508 demo 页同口径，渲染器包零改动）。特殊键/组合键回发需渲染器包
  扩 encode API（走其自身测试链 + golden 维护）——债候选。
- **① .at widget 升格条件**：v1 宿主内置面；若 .at 应用需要声明式内嵌
  远程窗（`remote_window` widget），升格走 aura.at+registry 登记（vm 臂
  n/a/web_component 形态）——触发=出现真实声明式需求。
- **② 会话与窗口的生命周期绑定**：v1 一窗一会话；多窗共享一会话（同一
  远程桌面多窗映射）列为后续形态，需求出现再扩。
- **③ 帧节奏参数**：rAF 合帧为 v1；显式节流/背压（远端产帧快于渲染）
  以实测数据定，必要时渲染器包加 API（带其 golden 维护）。
- **④ 远端窗口语义映射**：远程会话的 Welcome.wh/rect 与 vue 虚拟窗 rect
  的协商（远端按 vue 窗 resize 跟随，v1 以 vue 侧为源）——与 512 fit
  动态重测的远端联动语义在协议文档补一行注记。
- **排程提示**：508/507 均已合入，无前置阻塞；但若 515 的 scissor 算子
  先合入，远程窗的 scroll 内容裁剪自动受益（渲染器包 golden 同步关系），
  领取顺序不设硬约束。
