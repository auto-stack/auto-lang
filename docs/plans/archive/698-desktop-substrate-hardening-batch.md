---
plan_id: PLAN-698
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-substrate-hardening-batch
author: [zcode]
created_at: 2026-09-23
updated_at: 2026-09-23
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 ui-cache 生成器指纹契约, SD-02 VM bus.subscribe runtime 契约, SD-03 menubar submenu 形态终态, SD-04 desktop 端点管道命名与 auto-os 配对契约]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN（无已登记 GOAL 覆盖本批领域）

affects: [docs/specs/auto-lang/runtime/overview.md, docs/specs/stdlib/design/http-server.md, docs/specs/widgets/menubar-family.md, docs/specs/auto-lang/ui/overview.md]
current_step: 5
total_steps: 5
---

# [PLAN-698] desktop-substrate-hardening-batch——债册 medium 清偿批 + 基建小件

> 来源：PLAN-697 归档后盘点（2026-09-23 用户裁定"打包一个项目"）。
> 四件无相互依赖，四轨并行：P693-D2（desktop 跨仓配对，medium）+
> P695-D2（VM submenu 浮动式，medium）+ P696-D1（VM pubsub/SSE，
> medium）+ P692 ui-cache 生成器指纹（infra 高杠杆小件）。

## 变更摘要

四轨清偿，一计划打包：

1. **ui-cache 生成器指纹**（P692 债务候选#1）：`.auto/ui-cache.json`
   缓存键仅含源文件哈希——生成器升级后旧产物被判"新鲜"复用（692 W-1
   实证：code_editor 修复后页面不重生成）。生成器身份进缓存键，指纹
   变化即全失效。
2. **VM bus.subscribe runtime**（P696-D1）：`auto.bus.subscribe()` 目前
   是 compile seam（stdlib.rs:6473 压 -1，执行即响亮失败）——generated
   Axum 的 publisher SSE 绿不外推到 VM。补 VM 侧 subscribe bridge，
   publisher SSE 按真实 VM server topology 复验。
3. **menubar submenu 浮动式**（P695-D2）：浮动面板（嵌套 Popover
   overlay）走查三次复现 screenshot 恒超时+一次进程静默死亡，VM 臂已
   降级内联展开。bounded spike 勘定 iced overlay 嵌套可行性：feasible
   → 浮动式恢复（RightTop 落位变体已备三处）；infeasible → 内联式收口
   为契约终态。
4. **desktop 端点跨仓配对**（P693-D2）：auto-os 桌面 shell 内嵌 rqhost
   daemon（库形态优先）+ 管道命名约定裁定（693 待澄清#1：固定 wellknown
   vs 桌面注册表发布）+ 003 级交互实机互连录证——RQ 线最后一块跨仓
   拼图。

**非目标**：P695-D1 menubar 键盘导航 / P696-D2 graceful shutdown API /
P684-D1/D2 027 写端点 / P683-D1..D4 remote 词汇面（各留债册，另批）；
不做 launcher VM 轨输入面（P694-D2 归 auto-os/shell 线但属输入路由，
非本批配对面）。

**成功标准**：四轨各一 AC 全绿（生成器升级自动失效 / 017 VM 臂
subscribe 全环 / submenu 形态决策工件+终态落地 / auto-os 桌面内拉起
003 exe 全环录证）；既有门禁零新增红。

## 目标

- **G1（ui-cache 指纹）**：生成器身份（fingerprint）进 UICache 缓存键，
  指纹不匹配全失效自然再生——692 W-1 场景（生成器修复后页面自动重生
  成）回归绿。顺带核 `version: u32` 迁移语义（1→2）与"缓存是否应入库"
  评估结论（auto-os 侧注记）。
- **G2（VM bus.subscribe）**：subscribe 从 compile seam → 真实执行面
  （iterator 服务），§8.1 表更新为 VM publisher SSE 复验后口径。
- **G3（submenu 形态）**：浮动式可行性 spike 决策工件 + 终态落地
  （浮动恢复或内联收口）——走查期静默死亡经 697 足迹桩武装后若再现
  可判位。
- **G4（desktop 配对）**：auto-os 桌面 shell 成为真实桌面端点（内嵌
  daemon），003 exe 交互全环录证；管道命名约定有裁定结论入契约。

## 架构方案

四轨各自独立，共用一个 worktree 组（跨仓计划：`lang-698` 组挂
auto-lang + auto-os 双 worktree）：

```
PLAN-698
  ├─ 轨A ui-cache（auto-lang 单仓）
  │   build.rs hash src/ui_gen/** → AUTO_UI_GEN_FINGERPRINT env
  │   → UICache.generator_fingerprint 不匹配即全失效
  ├─ 轨B VM bus.subscribe（auto-lang 单仓）
  │   勘定 publisher 拓扑 → subscribe bridge（iterator 注册表形态
  │   倾向）→ 017 VM 臂全环复验 → §8.1 落表
  ├─ 轨C submenu 浮动式（auto-lang 单仓）
  │   iced overlay 嵌套 spike（带 697 足迹桩复现死亡判位）
  │   → feasible: 浮动恢复+RightTop 接线 / infeasible: 内联收口
  └─ 轨D desktop 配对（跨仓 auto-os）
      管道命名裁定 → auto-os shell 内嵌 run_daemon → 003 互连录证
```

关键既有件（实勘在案）：
- `crates/auto-lang/src/database/ui_cache.rs`——UICache 已有
  `version: u32`（const 1）+ `api_functions_hash` 先例（API 配置变化
  全失效），指纹字段同型扩列；
- `crates/auto-lang/src/vm/ffi/stdlib.rs:6473`
  shim_bus_subscribe_stub（编译面 seam）；VM SSE producer 回收臂已由
  696 交付（http_server.rs）；
- `crates/auto-lang/src/ui/aura_view_builder.rs`
  build_menu_panel_items 内联臂 + view.rs/popover.rs/
  native_projector.rs 三处 RightTop 备位；menubar 契约档
  `docs/specs/widgets/menubar-family.md`；
- `crates/auto-lang/src/ui/desktop_protocol/rqhost.rs:1266`
  run_daemon(wellknown) 库形态入口 + client_entry.rs
  ClientTarget::Desktop 采纳臂（P693 交付，exit-on-EOF）。

## 需求分析与背景调查

- **授权记录**：用户 2026-09-23 「OK，打包里一个项目里吧」——对
  697 收官盘点中建议的「三个 medium 债 + P692 ui-cache 键小件」打包
  立项的显式批准。
- **债册原文**：P693-D2（:22）/ P695-D2（:45）/ P696-D1（:55）+
  P692 债务候选#1（:27，含"评估该缓存是否应入库"子问）。
- **spec 现状**：http-server.md §8.1 明文 "`auto.bus.subscribe()` 仍是
  VM compile seam"；menubar-family.md 现契约为内联展开形态（P695 D2
  降级后）；ui/overview.md 桌面端点节（PLAN-697 SD-01 相邻）无跨仓
  配对契约；runtime/overview.md 提及 ui-cache 无指纹契约。
- **PLAN-693 遗留**：待澄清#1（管道命名：固定 wellknown vs 桌面注册
  表发布）在 693 归档时未裁定——本计划 T-04 内呈报裁定。

## 详细设计

（各轨勘定后回填精确点位；下为现行设计意向。）

### 轨A ui-cache 指纹

- `crates/auto-lang/build.rs`（新增）：对 `src/ui_gen/**` 生成器源
  集做稳定内容 hash → `cargo:rustc-env=AUTO_UI_GEN_FINGERPRINT=<h>`；
  `rerun-if-changed` 指向该目录。
- `ui_cache.rs`：UICache 增 `generator_fingerprint: Option<String>`，
  写侧 stamp `env!("AUTO_UI_GEN_FINGERPRINT")`；load 时与现 env 不
  匹配 → 整缓存失效（同 version 语义）；`VERSION` 1→2 顺带迁移
  （旧缓存无指纹字段，version 不匹配即弃）。
- 双保险：`api_functions_hash` 既有臂不动。
- auto-os 侧：`.auto/ui-cache.json` 为生成数据——评估结论（入库与否）
  注记归 auto-os（本计划只保证失效正确性；入库策略不阻塞）。

### 轨B VM bus.subscribe

- T-02a 勘定（决策工件）：VM 侧 ~Stream/publisher 现行拓扑（
  http_server.rs SSE producer 如何被宿主服务、topic 寻址形态）→
  subscribe bridge 三案定一：
  ①进程内 iterator 注册表（subscribe 返回 iterator_id，事件泵线程
  写入，.at 侧既有 iterator 家族消费——与 stub 签名零破坏）；
  ②直连 SSE URL 的 HTTP iterator（复用既有 async 结果面）；
  ③bus topic 注册表+内核态发布订阅（重，仅当①②证明不足）。
  倾向①（进程内拓扑与 back-proxy 同构、断连回收复用 696 臂）。
- T-02b 落地：替换 stub（保留 -1 失败面为查无此 topic 时的响亮错误）；
  断连/停端语义对齐 696 回收臂；017-chat（或最小语料）VM 臂
  subscribe→收事件→UI 联动全环。

### 轨C submenu 浮动式

- T-03a spike（有界）：iced 0.14 overlay 嵌套（嵌套 overlay 树/分层
  合成）可行性与局限勘定；同场以 697 足迹桩跑浮动式开态复现（screenshot
  超时位面 + 死亡判位）。
- feasible → T-03b：浮动式恢复（RightTop 三处备位接线：view.rs/
  popover.rs/native_projector.rs）+ 开态快照/截图验证 + 内联臂降为
  降级路径；infeasible → 内联臂收口为契约终态（SD-03 modify）+
  P695-D2 降级划线。screenshot 通道超时若独立存在（P695-D5 041/
  render 时序族）不在本计划修复面，只判位留痕。

### 轨D desktop 配对（跨仓）

- T-04a 命名裁定（呈报项，见待澄清2）：注册表发布+wellknown 兜底
  （倾向）vs 固定 wellknown。
- T-04b：auto-os 桌面 shell 内嵌 rqhost daemon——库形态（auto-os 已
  依赖 auto-lang 的话直接 `rqhost::run_daemon(pipe)`；依赖不成立则
  子进程 `auto rqhost --pipe` 形态，勘定后定）；管道名来源=裁定结论。
- T-04c：003 级 exe 在 auto-os 桌面内拉起→v2 帧+输入回环→关闭回收
  全环录证（P693 三形态表的 auto-os 桌面臂补行）；desktop 端点
  `--desktop-endpoint` 指向配对管道。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/runtime/overview.md | 无 ui-cache 指纹契约 → UICache 缓存键含生成器指纹（src/ui_gen/** 内容 hash），不匹配全失效；version 迁移语义 | G1 沉淀；692 W-1 根修 | AC-01 |
| SD-02 | modify | docs/specs/stdlib/design/http-server.md §8.1 | "auto.bus.subscribe() 仍是 VM compile seam" → VM subscribe runtime 形态+真实覆盖口径（勘定后的 bridge 案） | G2 沉淀 | AC-02 |
| SD-03 | add/modify | docs/specs/widgets/menubar-family.md | submenu 形态按 spike 结论定终态（浮动式恢复含 RightTop 变体 / 内联式为契约终态+降级注记） | G3 沉淀 | AC-03 |
| SD-04 | add | docs/specs/auto-lang/ui/overview.md | 无跨仓配对契约 → auto-os 桌面 shell=桌面端点（内嵌 daemon 库/子进程形态+管道命名约定+003 互录证口径） | G4 沉淀 | AC-04 |

## 测试设计

- 轨A：单测（指纹不匹配全失效/匹配保留/version 迁移弃旧）+ 692 W-1
  场景实机回归（改生成器→页面自动重生成，免手工清缓存）。
- 轨B：bridge 单测（topic 查无响亮失败/正常收流/断连回收）+ 017 VM
  臂全环（实机走查，VM server topology 下 publisher→subscribe→UI）。
- 轨C：spike 决策工件（reports/p698-batch/）+ 浮动式开态快照/截图
  （或内联收口的契约对勘）+ 死亡判位留痕（若复现）。
- 轨D：auto-os 桌面内 003 exe 全环录证（截图+观测行，reports/
  p698-batch/）；desktop 臂门禁（render_cli/client_entry 既有测试组
  零回归）。
- 既有门禁零新增红（cargo t 同作用域全量对账；触及 VM/生成器按
  Category B 加 cargo tv / tt 作用域）。

## 验收标准

- [ ] AC-01 生成器指纹落地：单测三态绿 + 692 W-1 场景实机回归
      （生成器改动→缓存自动失效重生成）。
- [ ] AC-02 VM bus.subscribe runtime：bridge 单测绿 + 017（或等价
      语料）VM 臂 subscribe→收事件→UI 联动全环录证 + §8.1 落表。
- [ ] AC-03 submenu 形态终态：spike 决策工件在档 + 浮动式恢复
      （开态快照/截图可达+RightTop 变体）或内联收口（契约 modify+
      债册划线）——二选一兑现且与 SD-03 一致。
- [ ] AC-04 desktop 配对：管道命名裁定结论在档 + auto-os 桌面内嵌
      daemon + 003 exe 全环录证（拉起/交互/回收）。
- [ ] AC-05 既有门禁零新增红（同作用域 no-fail-fast 对账，预存红
      逐名对照 697 基线名册）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] T-01 轨A ui-cache 指纹：build.rs（hash src/ui_gen/** → env）+
      ui_cache.rs 指纹字段+失效臂+VERSION 迁移 + 单测三态 + 692 W-1
      实机回归。[关联 AC-01] 验证：cargo t ui_cache + 实机两轮（改
      生成器前后）。[✅ 已完成]（2026-09-23）单测 9/9 绿（match 保留/
      mismatch 清空重签/appeared-disappeared/v1 迁移弃旧）；实机三轮：
      F1 生成（fp 8dbb7c2f1ac61eb0）→ 无改动复跑零再生+缓存 md5 稳+
      零 warning → 改 ui_gen/vocab.rs 重建 F2 → warning
      "generator fingerprint changed" ×2 + App.vue 重写（mtime 变）+
      缓存重签 4fd2e932fcca040d + "(changed)" 标记；探针已还原。
      worktree 提交 5785d532d。
- [x] T-02 轨B VM bus.subscribe：T-02a 勘定（publisher 拓扑+bridge
      三案决策工件）→ T-02b 落地（stub 替换+回收臂对齐+017 全环）+
      §8.1 落表。[关联 AC-02] 验证：cargo t bus_subscribe + 017 实机
      走查录证。[✅ 已完成]（2026-09-23）T-02a 裁定案①：VM serve 循环
      本就按 iterator_id 拉 ~Stream handler；生成 events.rs=无 topic
      broadcast::channel(256)；AsyncHttpStream 臂=现成适配器（696 回收
      语义）——三案中唯一零破坏。T-02b：shim_bus_subscribe（EVENT_BUS
      256+转发线程+AsyncHttpStream）+publisher 臂 publish_post_broadcast
      （双 serve 循环插桩；Typing/New{Type} 对齐 api_gen；has_sse 门控
      经 codegen record_api_return_type 新侧信道，unique_name 防 s-expr
      倾泻）。单测 bus_subscribe 2/2+plan698_publisher 3/3。017 VM 臂
      全环：SSE 帧 `{"event":"Typing","name":"Alice"}` +
      `{...,"event":"NewMessage"}` 与 Axum 形态逐字节一致；UI 联动：
      带外 curl POST typing（Carol）→ 浏览器渲染 "Carol is typing…"
      （截图+帧录证 docs/reports/p698-batch/）。§8.1 落表+seam 注记
      划线。worktree 提交 见 T-02 提交（feat(vm-bus)）。勘定坑二枚：
      Display 对 User 类型吐 type-decl s-expr（unique_name 根修）；
      DashMap 读守卫内 tasks.remove 自死锁（测试首跑挂 49min 实证，
      守卫作用域纪律）。
- [x] T-03 轨C submenu 浮动式：T-03a spike（overlay 嵌套勘定+足迹桩
      死亡判位，决策工件）→ T-03b 按结论浮动恢复或内联收口 + SD-03
      对应面。[关联 AC-03] 验证：spike 工件 + 快照/截图或契约对勘。
      [✅ 已完成]（2026-09-23）T-03a 判定 **feasible**：根因勘定=iced
      0.14 官方嵌套协议（overlay::Nested 递归 Overlay::overlay 钩子）
      存在而 Panel 未实现（默认 None）——嵌套 Popover overlay 永不注册
      即 P695-D2 真死因。T-03b 浮动式恢复：Panel::overlay 收集钩子
      （绝对坐标零平移）+ 视图臂嵌套 Popover（RightTop T-06 变体）+
      内联降级路径（AUTO_MENU_SUBMENU_INLINE=1）。契约测试更新浮动形态
      6/6+popover 18/18 绿；实机 widgets-gallery File→Share：开态快照
      可达+全程零死亡（6+ 次实验）。判位留痕：screenshot 通道超时在
      基线（零改动）单层菜单开态同现——D5 预存与形态无关，按待澄清4
      不修另线；像素级确认被该通道阻塞为残余边界（工件 T03-submenu-
      floating.md 在档）。worktree 提交 feat(ui-menubar)。
- [x] T-04 轨D desktop 配对（跨仓 auto-os）：T-04a 命名裁定 → T-04b
      内嵌 daemon（库/子进程按依赖勘定）→ T-04c 003 全环录证 +
      SD-04。[关联 AC-04] 验证：auto-os 侧构建 + 实机录证三件
      （拉起/交互/回收）。[✅ 已完成]（2026-09-23）T-04a 定稿：注册表
      发布+pid 派生隔离（`autodesk-rqhost-desktop-<pid>` +
      AUTO_DESKTOP_ENDPOINT env 子进程继承；wellknown 兜底=桌面外 -q
      轨不变）。T-04b 形态勘定：**库形态被结构性否决**（winit Windows
      每进程单事件循环——专线程第二 iced 循环 RecreationAttempt panic
      双栈实锤，留痕 t04_desktop.log）→ 按待澄清3 预授权落子进程形态
      （`auto rqhost --pipe` 孵化，spawn_desktop_daemon，5s 就绪探测
      放行）；启用门 DesktopOptions.rqhost_endpoint ∨ env
      AUTO_DESKTOP_RQHOST=1（缺席零变化）；auto-os desktop.ps1 缺省置
      1（os 侧提交 0db4c17）。T-04c 全环（003 converter.exe，
      auto build -r rust）：spawn→publish→客户端"采纳桌面合成器端点
      （不孵化）"→adopt→开窗 480x320→首帧→perf 帧行→EOF 窗回收→
      末窗 daemon 自退——拉起/交互/回收三件录证齐（log×2+屏摄，
      docs/reports/p698-batch/T04-desktop-pairing.md）。SD-04 落表
      ui/overview.md 桌面端点契约节。lang 提交 3d3db11a3；render_cli
      8/8+client_entry 3/3。
- [x] T-05 收口：门禁对账（AC-05）+ SD-01..04 落表核验 + 债册对账
      （P693-D2/P695-D2/P696-D1 处置+P692 候选#1 销号）+ 复审。
      [关联 AC-05] [✅ 已完成]（2026-09-23）门禁对账：`cargo tv`
      no-fail-fast 全量 5652 跑 10 唯一红 + `cargo t` 日档全量 5504
      跑 10 红——两档同名册，逐名定责全预存/漂移零新增：musk×6（在册
      预存，.at 双源漂移修归 auto-os sync）、projector_counter
      （PLAN-678/679 主检出同败实证在档）、native_gate_runtime_views
      （018 语料链接失败 chapter_from_progress undefined——.at 漂移
      族/他方 auto-os 语料 WIP，与本 diff 零交集）、plan358 预算+
      a2vue 金样（**detached 基线 543eccdc4 双双复现同败**——基线
      worktree 定责后 guard-clean 移除）。SD-01（runtime/overview.md
      指纹契约节）+SD-02（§8.1，T-02 时已落）+SD-03（menubar-family
      浮动终态+边界更新）+SD-04（ui/overview.md 桌面端点契约节）
      四件全落表。债册：P693-D2/P695-D2/P696-D1 三债销号划线+P692
      候选#1 销号+新债 P698-D1（浮动像素确认被 D5 阻塞+交互细节）/
      P698-D2（daemon 生命周期无级联+launch 面接线）在册。worktree
      簿记提交 2461bbf39。

## 复审记录

- draft 交付（2026-09-23）：stage=new，PLAN-698 rev1。outcome=pass。
  next=work（四轨无相互依赖可并行；T-02a/T-03a 为有界勘定门——
  infeasible/inadequate 结论均为合法出口且已在契约内授权分流臂；
  T-04a 命名裁定呈报待澄清2，默认注册表发布+wellknown 兜底）。
- work 交付（2026-09-23）：stage=work | plan_id=PLAN-698 | rev=1 |
  outcome=**pass（execution_done）** | code_commit=auto-lang worktree
  plan-698-dev 5785d532d→4281a7678→c435fe7be→3d3db11a3→2461bbf39（五
  提交）+ auto-os plan-698-dev 0db4c17 | task_ids=T-01..T-05 全清 |
  evidence=AC-01（指纹三态单测 9/9+W-1 实机三轮：F1 生成→稳态零再生
  →F2 全量再生重签）/AC-02（bridge 案①落地：bus_subscribe 2/2+
  publisher 3/3；017 VM 臂 SSE 帧与 Axum 形态逐字节一致+带外 POST
  typing UI 渲染 "Carol is typing" 截图录证）/AC-03（T-03a feasible：
  Nested 协议钩子+浮动恢复，契约单测 6/6+popover 18/18；开态快照
  可达+全程零死亡；截图超时经基线对照判位 D5 预存）/AC-04（T-04a
  命名定稿+子进程形态勘定〔库形态 winit 单事件循环否决，实锤留痕〕+
  003 converter 全环：adopt→开窗→v2 帧→EOF 回收→末窗自退，log×2+
  屏摄）/AC-05（tv+日档双档同名册 10 红，逐名预存定责，基线
  detached 复现两陌生红，零新增） | blockers=无 |
  next=review（/auto-plan:review；worktree lang-698 双仓保留）。

- review #1（2026-09-24）：stage=review | plan_id=PLAN-698 | rev=1 |
  outcome=**needs_fix（F-1）** | reviewed_commit=2461bbf39（lang）+
  0db4c17（os） | base=543eccdc4/eb93848 | 证据=diff 全量审计（10 .rs）
  +四 AC 工件链在档+§8.1/SD 文本对现行行为核验 | 独立性声明=与执行
  同会话，裁定由工件/diff 重建 |
  **F-1（medium，AC-02 面）**：bus.subscribe 转发线程在订阅端断连后
  永生——`cleanup_sse_iterator` 只移除 iterator 不回收 ASYNC_STREAMS
  句柄，rx 不 drop → 转发线程 `blocking_send` 永不失败（broadcast
  静态永活）；每次弃订泄漏线程+句柄+Receiver。既有各流（http/io）
  线程自然终止故仅句柄累积（预存面），bus 线程无自然终点（本计划
  引入面）。修法=cleanup 对 AsyncHttpStream 臂回收句柄（rx drop→
  线程退出），test 断言句柄移除。 | next=work（F-1 修复）→re-review。

- review #2（2026-09-24）：stage=review | plan_id=PLAN-698 | rev=1 |
  outcome=**pass** | reviewed_commit=b6e037cbf（lang F-1 终形）+
  0db4c17（os） | base=543eccdc4/eb93848 | acceptance=AC-01..05 全过
  （AC-02 面含 F-1 修复） | findings=F-1 已闭环 |
  evidence=①F-1 终形（cleanup 回收句柄 + Weak 有界轮询转发线程）：
  单测 cleanup_reclaims_async_stream_handle+bus 3/3、作用域 17/17、
  **实机 4 轮订阅/断连线程数恒基线（7→7，RECLAIMED）**；②终码 tv
  全档 5653 跑 11 红=已知 10+back_provision（OS bind 10013 端口
  排除环境红，diff 零触面）；ffi_dual_019/c1_future_all 单跑双绿=
  负载抖动（ffi flaky 留档先例同族）；③tf 全档（中间形）12 红同名册
  定责；④SD-01..04 文本终态逐节核验与终码一致（§8.1 补 F-1 回收
  语义句） | next=merge。

- merge（2026-09-24）：stage=merge | PLAN-698:r1 | outcome=**pass
  （delivered）** | 落库=lang master b7ee6f302（rebase 6/6 range-diff
  全等 ff-only）+os main a574e31（rebase 1/1 全等 ff-only）|
  账本=specs.json P698-1（architecture）/P698-2（reviews）+ui/plans.md
  698 行+INDEX 再生 | 归档=docs/plans/archive/698-...md status
  archived | 清理=lang-698 组（auto-lang+auto-os worktree+auto-down
  依赖位+分支）guard clean 后移除 | checkpoint=prepared/landed/
  ledger_refreshed/archived/cleaned 五章全验。

### 执行期勘定与坑（沉淀）

1. winit/iced Windows 每进程单事件循环——同进程第二 iced 循环
   （库形态内嵌 daemon）= RecreationAttempt panic（结构性约束，
   子进程形态为 rqhost 嵌入唯一形）。
2. iced 0.14 官方嵌套 overlay 协议（overlay::Nested 递归
   Overlay::overlay 钩子）——P695-D2 真死因=钩子未实现而非 iced
   缺能力；screenshot 通道开态超时与形态无关（基线同现）。
3. DashMap 读守卫作用域内 tasks.remove 同 shard 自死锁（测试首跑
   挂 49min 实证）——守卫 drop 后再 remove（http_server Plan 346
   3c 注记同款陷阱）。
4. Type::Display 对 User 类型吐 type-decl s-expr——事件名侧信道须
   unique_name；`*.log` 全局 gitignore 再度吞报告附件（P697 同款，
   报告附件一律 .txt）。
5. musk×6/counter/a2vue 金样/plan358 预算/native_gate 018 语料链——
   五族预存红在 detached 基线逐名复现定责，AC-05 口径可复核。

## 待澄清事项

1. T-02b bridge 三案（进程内 iterator 注册表 vs 直连 SSE URL vs 内核
   pubsub）——T-02a 勘定后呈报；默认案①。
2. T-04a 管道命名约定（桌面注册表发布+wellknown 兜底 vs 固定
   wellknown）——PLAN-693 待澄清#1 遗留；默认前者（多桌面实例隔离），
   执行期呈报一次定稿。
3. T-04b auto-os→auto-lang 库依赖方向是否已成立（拆仓调研称方向天然
   成立但 auto-os 实际 Cargo 依赖未核）——不成立则子进程形态兜底，
   勘定后即定不阻塞。
4. T-03 spike 若浮动式 feasible 但 screenshot 通道仍超时（P695-D5
   041/render 时序族独立问题）——本计划 AC-03 口径=渲染+快照面，
   截图超时只判位留痕不修复（另线）。
