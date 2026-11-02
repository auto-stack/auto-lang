# AV 资源故障注入与 Place 代际正式证据（scenario R02/R03，PLAN-915）

> **Status**: delivered（PLAN-915，2026-11-02）。本文件是 `ns=scenario` 两格 R02/R03 的
> **AV 侧证据合同**，不是新 profile、不是新 wire：两个场景都跑既有 profile
> （`av.collections/1` 与 `av.closures/1`）与既有 seam（`ExecLimits::fault_after_allocs`、
> `StrArena::set_fault_at`、`ListPlace` 运行期校验）。原始 corpus 源/expected/分母不变。
>
> 实现：`experimental/ace/crates/av/src/resource_915.rs`（fixture/扫描）、
> `av-worker case-serve` 的 `ac-av/resource-run-v1` 配方、
> `crates/auto-lang-core/src/test_cases/ac_av.rs` 的父判与冻结表。
> 正式材料：`crates/auto-lang/test/cases/ace-m1/resource-915/`，
> 固定组 `case-ace-m1-resource-915`。

## 1. 两格义务（与 `ace-m1-scenario-oracles.md` 一致）

- **R02**：构造/clone/扩容的**每个实际分配 attempt** 均可注入；失败不提交，已构造部分回收，
  源/旧值/所有权/epoch 不变，错误分类不伪装成语言错误；故障清除后同会话真重跑得到正确结果；
  N+1 正控成功。
- **R03**：List 扩容前取得的 view/token 在结构性变更后使用必须**受控拒绝**，危险访问 = 0；
  合法近邻（新 token、未变元素、活 Share/Static 缓冲）保持可用；静态借用违规仍在前门拒绝。
  两者是两条独立义务：静态拒绝不能冒充运行期 stale 证据，反之亦然。

## 2. R02：逐 attempt 注入合同（值 seam + arena seam）

两类 seam 互相独立，分别扫描：

| seam | 覆盖的实际分配 | 计数/失败点 |
|---|---|---|
| 值 seam（`fault_after_allocs`） | 槽写正增量（构造/CLONE/聚合/Variant pack/Place 铸造）、闭包环境物化、**List 容量扩容** | 每次 attempt 递增一个序号；命中即 `Infra`，先于写入/变异 |
| owned arena seam（`set_fault_at`） | owned Str 缓冲、List 缓冲、capture-clone 拷贝 | 全局 attempt 序（1-based）；命中时无 id、无 `AllocEvent`、无 live 项 |

- **先冻结 N**：无注入运行一次得到结果与 ledger；再以 `fault_after_allocs = Some(u64::MAX)`
  运行取得真实值 allocation 总数；随后 1..=N 与 1..=N' 逐点注入，`N+1`/`N'+1` 为正控，
  另有“无注入对照”与“故障清除后重跑”两次运行。
- **每个失败点断言**：outcome 是带序号的可辨识 `Infra`（`fault-injected at …#<n>`），
  live owned 缓冲 = 0（部分产物已回收），失败点之前的 attempt 成功计数 = n-1（arena seam），
  语言结果不产生 PASS/半产物。
- **口径**：record/Variant 的 `Rc` 头不单列 attempt——它们的构造以"槽写正增量"计入值
  seam（与 821/822 的 V-07 既有定义一致）；owned Str/List 缓冲才是 arena attempt。
  这两类合起来覆盖本域运行时真正记账的每一次分配尝试。
- **支持域**：原 corpus `L04`（owned record 字段 view）、`L05`（remove 移出 owner）、
  `V05`（Variant owned payload）、`LSTR`（List<Str> push/remove）、`K02`（创建者退出后
  owned 闭包，capture/env 物化），外加 typed 层的 capture-clone 控制：AutoLang 源码
  无法产生 clone 捕获（返回借用捕获闭包被 `source.closure-borrow-escape` 拒），故按
  executor 自用的 prepare 边界直接见证——两字段 owned record 经 `EnvPlan::prepare`
  clone，逐 attempt 失败均报 `env.capture.faulted-clone`（带序号），源保持可读/仍在
  owner，N+1 控件成功。
- **父侧冻结**（worker 只报事实，不带期望）：每例的结果值与两个 attempt 总数写死在父表
  `RESOURCE_SWEEP_CASES`。当前冻结值：L04=42/6/2、L05=42/5/2、V05=42/6/1、LSTR=51/5/3、
  K02=42/5/1（值 / 值 seam attempt / arena attempt）；capture-clone 冻结为
  attempt 3/4 失败、attempt 5 成功、live 恒为 2。任一 seam 少一点即红。
- **不声称**：随机主机 OOM、非本域容器（List 嵌 List 等）、native/旧 AVM 的对应格、
  以及把预算拒绝当作 allocator 故障（预算 precheck 先于 attempt，attempt=0）。

## 3. R03：运行期 stale 见证合同（typed、但真授信）

AutoLang 源码当前**不能**铸造 List Place（`xs.get(i)` 的 owned 元素是只读受检 view，
跨结构变更的持有被 `source.view-escape`/`source.iterate-mutate` 静态拒绝），因此运行期
见证使用 typed MIR，但必须通过**与源码相同的三道 gate**：
`verify_mir` → `encode_abc` → `verify_abc_bytes`（未授信/手改 MIR 不作为合法 stale 输入）。

- **序列**：`LIST_NEW` → push → `LIST_PLACE` 铸 token → `PLACE_GET`（合法读）→
  结构性变更（push / insert / remove / 元素替换 `LIST_SET`）→ 旧 token 的 `PLACE_GET`。
- **拒绝点**：`resolve_place_elem` 逐次校验 frame（激活序）→ slot → `buf_id` → `epoch` →
  `index` 界 → `elem_gen`，任一不符即 `av.trap.stale-place`，且该校验**先于任何 buffer 访问**。
- **可观察事实**：`ExecLimits::observe_places`（默认关闭，生产零成本）启用时，
  `ExecTrace.place_facts` 报告 mints/get_ok/get_stale/set_ok/set_stale 与
  `buffer_reads`（**只统计通过全部校验后真正发生的访问**）。场景断言
  `get_stale=1` 且 `buffer_reads` 只等于变更前那次合法读的数量——stale 使用不产生任何访问。
- **正控**：变更后新铸 token 读到当前元素（fresh）；无变更的 mint→read 读回原值（legal）；
  两者都要求 `get_stale=0`。
- **父侧冻结**（`PLACE_SEQUENCES`）：四种变更都必须是 `av.trap.stale-place`；
  fresh 值与 legal 值按变更类型冻结（push/insert=30、remove=20、replace=99；legal=10）。
- **静态面另记**：`for x in xs { xs.push(...) }` 在源码前门以 `bind/source.iterate-mutate`
  拒绝（父侧以 stage=resolve、该 code 判定），不与运行期 stale 混为一谈。
- **不声称**：`MutRefV{frame,slot,gen}` 的通用运行期失效句柄校验（821 明确不交付，
  本计划不扩）；全局句柄/借用语言重构；native/旧 AVM 的对应格。

## 4. 观测、预算与生产隔离

- 新观测只在显式开关下产生：值/arena seam 仅在 `fault_after_allocs` / `set_fault_at` 配置时
  计数（生产默认零开销）；place facts 仅在 `observe_places=true` 时随 trace 返回。
- 失败的 arena attempt 不落地任何 id/事件；`alloc_events` 序号随 `ExecTrace` 返回，
  供父侧发现“少扫一点”。
- 观测材料按原更严上限：请求 ≤4MiB、单次执行值 ≤8MiB/受管 ≤32MiB；扫描不引入并发写、
  不改变默认路径的指令/布局/结果。
- 与 913 恢复窗互不阻塞：本场景不发布、不恢复、不缓存运行结果；故障后重跑是同一进程内
  重新执行，不写历史缓存。

## 5. 证据边界

- 两格只关闭 scenario R02/R03 的 **av** 列；ac/avm/a2r 列各自的 owner 与状态不变。
- M1 总门仍由其余格与 `ace-m1-gap-audit.md` 的剩余 owner 决定，本文件不签 M1。
- 证据以 `case-ace-m1-resource-915` 的可复跑结果 + 父侧冻结表为准；worker 的自述、单点
  注入或“只测第 1 点”不构成闭合证据。
