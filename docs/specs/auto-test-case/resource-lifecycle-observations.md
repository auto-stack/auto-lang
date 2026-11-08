# 资源生命周期观察合同（resource-lifecycle-observations）

> 状态：delivered（PLAN-919，2026-11-03）。
> 性质：scenario R01/R04/R06 × ac/av 六格的**逐 owner 生命周期事件**规范正文——
> 通道词表、有界准入、父侧冻结不变式与逐例表。语义 oracle 不变
> （[ace-m1-scenario-oracles.md](ace-m1-scenario-oracles.md) §1 R 族）；
> 本文件冻结"怎么真实产生并观察"。

## 1. 通道与身份模型（冻结）

- **AV（`av.lifecycle/1`）**：`execute_observed` 在现有 Checked HIR→共享 MIR→
  ABC→AV 执行路径上记录纯数据事件；单一连续 `seq`（从 0）覆盖 frame/own/disown/
  free/end，arena free 钩子使 host 侧 drop 结果后的 free 事实继续进入同一序。
  owner 身份：`str`/`list`＝arena buffer id；`record`/`variant`＝无内在 id
  （LIFO 序键＝min buffer id，与执行器既有 sweep 键一致）；`closure-env`＝env
  store 句柄。槽级责任键＝`(frame, slot, gen)`。
- **native（`MN-EVT`）**：no_std collections runtime 在 PE stderr 上逐行输出；
  `seq` 从 1；frame 激活 seq、node 创建 id（即 LIFO 键）；槽级键＝`(frame, slot)`
  （own/disown 交替允许多对）。`frame-leave` 无 sweep 标志（返回与 unwind 同门）；
  disown 词表含 `dropped`（显式 drop）。
- 观察是**纯数据**：不持有 Value/handle、不延长寿命、不深 clone、不改变 drop 顺序。
  开关关闭＝零事件、语言结果逐字节一致（两侧对照测试常驻）。
- **有界（先限额再分配）**：AV 上限 16384 事件→具名拒绝
  `av.resource.lifecycle-events`（dispatch 循环下一拍，绝不静默截断）；
  native 上限 8192 行→`MN-COLL-EVT cap=` + abort 22。观察预算：AV 序列化形态
  <4MiB/操作；native ≤8192×≤256B < 1MiB stderr 上限；源/流/值/进程增量门全部沿用原合同。

## 2. 请求面（只给操作与预算）

- `ac-av/resource-events-v1`：`av.resource { profile: owned|collections, max_steps?,
  wall_ms?, cancel_after_ms? }`——两个**既有生产前端**（text-record/owned 与
  collections；不造第三前端），R06 退出旋钮由请求显式给出。
- `ac/mir-native-resource-events-v1`：`mn.resource { budget_bytes?, cancel_after_work?,
  fail_nth? }`——旋钮由生成入口 wrapper 预置（`mn_coll_set_budget` /
  `mn_coll_cancel_after_work` / `mn_coll_fail_nth` / `mn_coll_observe_enable`），
  绝不靠父侧杀进程冒充取消。
- 预期事件**永不进请求**；父侧以冻结规则与逐例表重判，不信 worker 自报。

## 3. 父侧冻结不变式（独立重判，全通道适用）

1. seq 连续（AV 0 基/native 1 基）；结构键集按动作精确匹配，未知/重复/缺字段/
   坏类型/越界/词表外值＝红。
2. frame 嵌套：enter/leave 平衡、leave 只关最内层；首事件＝根 enter。
3. 槽纪律：AV 每 `(frame,slot,gen)` 恰一 own 后一终态 disown；native 每
   `(frame,slot)` own/disown 交替且以 own 开终态收尾——**漏释放/重复释放**在此红。
4. buffer：每个 id free 恰一次（双 free 红）；free 流序与 ledger 序一致；
   分配集与 free 集核销（泄漏红）；`end` 后仅允许 free（结果存活性）；drop 结果
   后 live=0。
5. LIFO：同一 frame-leave sweep 内 swept disown 的 key 严格降序（错序红）；
   native 同门（槽倒序）。
6. 结束标记：执行过的流恰一个 `end`/`MN-EVTEND total=`，计数与行数一致
   （截断/重放红）；cap 标记＝具名拒绝，永不绿。
7. 源→受检产物→执行产物身份链：AV `source_hash`=fnv1a64(当前源) 回显核对；
   native 沿 collections manifest/运行时/link 身份链（abort 面不发布 candidate，
   不要求 manifest_fp）。

## 4. 逐例冻结表与反控

- 父侧表（`ac_av/resource_events.rs` / `ac_mn/resource_events.rs`）按
  `case/check` 键冻结：outcome/value/trap/resource 面、frame 数、created 计数
  （按 kind）、精确 free 序——数值来自真实父侧运行转储，非 token 手改。
- 区分反控（必须红）：只报总数（缺事件流）、漏 end、双 free/双 disown、泄漏、
  sweep 键升序、伪造词表、重放行、cap 截断、非零 live——19 条 admission 控制
  常驻（auto-lang-core 单测）+ native 7 条 runtime/driver 事实测试。
- R06 面：av `av.resource.instr-steps`/`av.cancelled`/`av.trap.*`；native
  `mn.resource.execution-budget|allocation-injected|cancelled` 与 `mn.trap.*`
  （真实下溢面按编译产物冻结）。

## 5. 边界（不宣称）

- AV 集合前端 whole-source 计量边界、avm/a2r 两列（NA-6）、R02/R03/R05（915/921
  线）、native loader/console（NA-3/F-NAT-1）不变。闭包逃逸（返回闭包）env 事件
  属 R05 线；本包 R06 用深集合树。
- 观察 OFF 的既有 recipe 行为逐字节不变（`resource-events` 是新 recipe，不覆写
  旧通道）；暖路径/恢复（913/889）零额外重编或重装（组级复跑绿）。

## 6. PLAN-923 增量：逐 seam 故障事实与 Place 代际

- 新事实面与 MN-EVT 并存：`MN-COLL-INJECT seam=<N>`+`nth=<N>`（成对严格解码，
  seam 0..=8；无 seam 的 nth / 无 nth 的 seam 是坏事实）、`MN-COLL-TRAP code=`
  （白名单扩至 27 = STALE）、`MN-COLL live=/bytes=/invalid=/arena_live=`。
  recipe `ac/mir-native-resource-guards-v1`（固定组 case-ace-m1-native-resource-923；
  父侧冻结表 ac_mn/resource_guards.rs：trap 码 + seam/nth 对 + zero-live 清理）。
- Place 代际：结构变更 bump node.epoch（退役全部旧 token）、in-place set/
  PlaceSet bump 目标元素 gen（废本 place 与兄弟）；stale 在 buffer 读写前
  trap STALE(27)；预算/注入失败未 commit 时 epoch/gen/有效 token 不变。
- 旧 919/915 边界：919 的 MN-EVT 通道与 R06 旋钮不因 923 变化；915 的
  av `resource-run-v1` 通道与冻结表不变；R03 ac 现由 923 关闭、R02（K02/env）
  与 R05 完整闭包唯一 owner = 924（本包只关 R03，不拿部分证据勾 R02）。

## 7. 证据锚点

- `experimental/ace/crates/av-core/src/lifecycle.rs`（通道）+
  `av.rs` 钩子（commit_slot_write/owned_kill_slot/两 sweep/call 转移/orphan env/
  溢出轮询）+ `owned_text.rs` free 钩子。
- `experimental/ace/crates/collections-runtime/src/lib.rs`（MN-EVT/cap/end）；
  `collections_lower.rs`（WrapperKnobs、Frame 40B 槽）；
  `native/src/collections_observe.rs`（严格解码）。
- `experimental/ace/crates/av/tests/cases/resource_events_919.rs`（8）与
  `native/tests/cases/collections_resource_events_919.rs`（8）。
- 套件 `crates/auto-lang/test/cases/ace-m1/resource-events-919/`（10 checks，
  固定组 `case-ace-m1-resource-events-919`）；binding 六格 `ok:P919-events`。

## PLAN-924：闭包 env 生命周期观测

- MN-EVT 通道上闭包 env 的 `act=own kind=closure` 出现在 creator 帧内；
  creator `frame-leave` 后 env 仍活（间接调用读得到）；最后一次 pin/owner
  释放后 `act=free`，终态 live=0/bytes=0/arena_live=0。
- 错误退出：body 内陷阱经 abort 清扫 parked env + 空闲文本元数据后同样达
  全零基线（修复前 trap 前已 StrLen 的程序泄漏 288B arena——见 924 归档
  计划 T-04/V-12 证据）。
- 注入分类学不变：S9 env 节点注入 trap INJECTED(25)；S4 帧注入沿 923 冻结
  的 frame-open 空路径（trap 22 + INJECT 事实行）。

## PLAN-998：a2r 面资源生命周期观察（2026-11-08，SD-04）

- a2r 侧 per-owner 生命周期事件（stdout `auto.res:<kind>[:<arg>]`，
  kind ∈ create|transfer|release|mut|read|view|env|fault|budget|cancel|
  stale|retry|heap|result）随 `legacy-a2r.resources-observe/1` 落地：
  release 由 RAII guard 的真实 Drop 观测（逐帧 LIFO=Rust drop 序），
  transfer 解除源 guard（转移≠释放），enum 赋值走代际 replace（旧 payload
  在真实替换点释放恰一次），结构变异先求参数再检查增长点。与 av/mn 面
  （av.lifecycle/1、MN-EVT，PLAN-919）分属不同引擎通道，互不借用。
- fault/budget/cancel/stale 的 typed 终态：stderr `auto.res:fail:<kind>`
  + exit 71（组合 profile 的 trap 仍 exit 70），unwind 过真实 Drop——
  无剩余预算时清理仍完成；合作取消点与父 Job kill 分态，kill 不充资源
  释放。分配计数：各侧只计各自真实 fresh alloc()（realloc 增长不计，
  方法级对称：各侧以 N+1 对照证明自身计数精确），观察机制自身分配
  （事件打印/环境读取）被 IN 旗标排除。
- 历史边界记录：R03-static 旧前端门不静态拒迭代中变异（发射产物由
  rustc 编译期拒），typed stale 面经 reference helper 面交付。
