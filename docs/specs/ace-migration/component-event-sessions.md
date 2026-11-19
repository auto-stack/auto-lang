# ACE 组件事件会话（增量执行/事务/关闭）

> 来源：PLAN-1106（2026-11-20 交付，复审/交付身份见计划 §9，归档于
> `docs/plans/archive/1106-ase-incremental-event-sessions-and-replay-audit-repair.md`）。
> 本文件记录组件面**增量事件会话**的已验能力与边界（修复
> `D-ASE-1023-REPLAY`）；行为/窗口/隔离证据的历史见
> [ASE M0 交互会话](../ase/m0-interactive-sessions.md)。

## 能力（已验）

普通 Node 源经共同 Schema 投影与**唯一 verify** 一次编译为**可保留事件
程序**；每个应用/实例 = 一个独立**会话**（独立全局 cell 状态根，共享已验
代码 `Arc`）；每个被接纳的事件 = 恰一次目标 handler 真实派发；状态读取 =
cell 纯观察（零派发）；关闭 = 释放状态根。

### 编译入口（`ace-core` `source::age_component`）

- `compile_component_event_program(src, module_path, alias, caps, node_index,
  emit_texts)`：与场景车道同一形状门/字段门/重复门/绑定检查与
  fail-closed 边界（body_foreign/宿主通道调用具名拒）。脱糖产物：
  - 每字段一个 `store` cell（`__age_ev_g_<field>`）；
  - `__age_ev_init()`：写声明初值（会话开启每代一次）；
  - `__age_ev_seed(<字段形参…>)`：props 种子全量覆盖；
  - 每 handler `__age_ev_on_<msg>(<载荷形参…>)`：开头 cell 载入局部 →
    `hook_stmts_lower` 同一脱糖跑业务语句 → **末尾统一提交全部 cell**
    （业务中途 trap ⇒ 零提交，旧态保留——编译期事务性）；
  - 每文本叶 `__age_ev_text_<ti>()`（`emit_texts`）：纯读 cell。
- str 载荷/种子按**借用 `str` 形参 + 一次 `.clone()` owned 局部**铸造
  （执行器入口准入只收 `StrView`，owned `Str` 非法入口实参）。
- `emit_texts=false`（states-only）：f-string 叶在 av.owned/1 不可 lowering
  的既有边界内状态面照常（会话层带文本→states-only 重试并把原因留痕，
  `texts_available=false`）。

### 会话运行时（`ace-av` `component_event_sessions`）

- `ComponentEventSessionProgram::compile`（一次；`compile_count` 留痕）→
  `open_session(&seeds)`（link + init 恰一次 + 可选 seed 覆盖；每会话独立
  cell 状态根与代次）。
- `dispatch[_seq]`：声明存在性/载荷 arity/类型在效果前验证（错误码保持
  旧场景道合同：`source.age.component-event` / `source.type-mismatch`）；
  成功 = 恰一次 `execute_program_entry_args` 真实派发（av-core def-id 定向
  入口，init 链每代一次；未链函数具名 Infra 拒）。
- 事务：编译期统一提交 + 会话层派发前 cell 备份/失败
  `restore_global_cells`（预算边界残余半提交兜底）；失败不发布半状态，
  失败后合法事件直接处理（不回放恢复）。
- 序号：宿主生成单调 sequence；`dispatch_seq(Some(seq))` ≤ 水位 = 已成功
  事件重投——不重复执行（返回 `Duplicate` + 上次已提交状态）；新序号正常
  执行；同名合法新事件各有各的序号不去重（有界水位，不累积业务历史）。
- 读取：`read_states`（cell 直读零派发）/`read_state`/`read_texts`（每叶
  一次纯读派发，不执行业务 handler）/`last_committed_states`。
- 关闭：`close()`/Drop 标记 released 并释放状态根；关闭后事件/读取具名拒。

### 观察口（执行层 oracle）

`SessionAudit`：`handler_entries`（只在真实执行器派发成功后增长）、
`entry_seqs`（执行器进程序号——真实派发证据，非 wrapper 自增）、
`init_runs`、`links`、`rejected`、`duplicate_ignored`、`released`。
UI 桥级 `EngineExecutionAudit` 汇总（含跨 unmount 的 retired 折入——增量
不变量 `events_delivered == handler_entries` 在生产道全程可对账）。

## 消费合同（auto-ui 桥）

`AceStateEngine` 内部持一次编译的程序；mount=open_session、dispatch=恰
一次 handler 派发、读取/切换零派发、unmount=close 释放；spawn_async/
await_async 沿 968 scheduler 与世代纪律。负控旋钮（`debug_force_replay_lane`
[带状态基线]/`debug_force_rebuild_lane`/`debug_force_getter_dispatch_lane`）
为测试专用真实 producer 注入道，生产路径恒关；旧场景观察链
（`observe_component_scenario_states`）保留为显式离线 oracle，在线无
fallback。

## 边界

- 单 node 源（多单元/跨引用 = IF-981-MODULE；包面 `PackageStateEngine`
  不变，另有 owner）；`MAX_INSTANCES=4`/引擎。
- body_foreign handler（检查函数道承载）与宿主通道调用不进本车道。
- f-string 文本叶 states-only（上文）；外部不可逆 IO 跨进程 exactly-once
  不在域；测试审计效果是非业务观测。
- 已验入口：av `cargo test -p ace-av --test conformance
  component_event_sessions_1106`；UI
  `cargo test -p auto-ui-runtime --features ace-bridge --test
  component_event_sessions_1106`（V-06..08 负控）。
