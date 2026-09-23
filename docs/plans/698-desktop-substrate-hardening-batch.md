---
plan_id: PLAN-698
status: drafting               # drafting → executing → execution_done → reviewed → archived
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
current_step: 0
total_steps: 4
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

- [ ] T-01 轨A ui-cache 指纹：build.rs（hash src/ui_gen/** → env）+
      ui_cache.rs 指纹字段+失效臂+VERSION 迁移 + 单测三态 + 692 W-1
      实机回归。[关联 AC-01] 验证：cargo t ui_cache + 实机两轮（改
      生成器前后）。
- [ ] T-02 轨B VM bus.subscribe：T-02a 勘定（publisher 拓扑+bridge
      三案决策工件）→ T-02b 落地（stub 替换+回收臂对齐+017 全环）+
      §8.1 落表。[关联 AC-02] 验证：cargo t bus_subscribe + 017 实机
      走查录证。
- [ ] T-03 轨C submenu 浮动式：T-03a spike（overlay 嵌套勘定+足迹桩
      死亡判位，决策工件）→ T-03b 按结论浮动恢复或内联收口 + SD-03
      对应面。[关联 AC-03] 验证：spike 工件 + 快照/截图或契约对勘。
- [ ] T-04 轨D desktop 配对（跨仓 auto-os）：T-04a 命名裁定 → T-04b
      内嵌 daemon（库/子进程按依赖勘定）→ T-04c 003 全环录证 +
      SD-04。[关联 AC-04] 验证：auto-os 侧构建 + 实机录证三件
      （拉起/交互/回收）。
- [ ] T-05 收口：门禁对账（AC-05）+ SD-01..04 落表核验 + 债册对账
      （P693-D2/P695-D2/P696-D1 处置+P692 候选#1 销号）+ 复审。
      [关联 AC-05]

## 复审记录

- draft 交付（2026-09-23）：stage=new，PLAN-698 rev1。outcome=pass。
  next=work（四轨无相互依赖可并行；T-02a/T-03a 为有界勘定门——
  infeasible/inadequate 结论均为合法出口且已在契约内授权分流臂；
  T-04a 命名裁定呈报待澄清2，默认注册表发布+wellknown 兜底）。

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
