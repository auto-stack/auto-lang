# 闭包源码来源绑定恢复（R3 关闭）

> **Status**: active（PLAN-913 交付，2026-11-02）。实现面：
> `experimental/ace/crates/av/src/closures_source_recovery.rs`（manifest/发布/恢复）、
> `experimental/ace/crates/core/src/source/closures_context.rs`（可移植声明上下文）、
> `av/src/bin/av_closures_913_restore.rs`（新进程恢复探针）、worker recipe
> `ac-av/closures-recovery-run-v1`、正式 suite `ace-m1-closures-recovery-913`。
> 承接 [闭包单函数前端生产](closures-function-production.md)（PLAN-912）交付的
> accepted source state，关闭其明示的 **R3（新进程恢复前置生产）**。

## 问题与边界

旧路径（823/865/912）在任何 store 命中之前必须先 `verify_mir` + `encode_abc`：
托管 store 的内容键 `unit-content` 是 canonical wire（`av-closures-unit/v1`）的
domain hash，因此"恢复"实际是先生产再查表。R3 的关闭标准是**零源生产**：
同一 source/context 的授权恢复 P 轴（parse/resolve/typecheck/Atom plan/bind/
HIR 构造/MIR lower/ABC encode）必须为 0，同时真实执行原 source 语义。

## 发布（A 面：真实生产后）

一次已提交的 source 生产可发布为项目**当前来源 root**：

- **schema** `ace.closures.source-origin/1`，私有层 id `A23-R`（不占全局
  A00–A23 层号，不改既有 A23 unit 层契约），root kind `source-recovery-current`。
- manifest 记录：source SHA-256/长度、逻辑 owner（module path）、MIR owner
  module/schema、entry/profile/capabilities、producer 三元组
  （`MIR_LOWERING_VERSION`/`MIR_CLOSURES_SCHEMA`/`av-closures-unit/v1`）+
  ABC codec/语义版本、accepted generation/commit point/context revision、
  每单元 `unit/logical/body_digest/dependency_digest/env_layout_identity/
  mir_identity/body_fp/wire_bytes/artifact id/spans` 与**spanless identity wire**
  （base64 + fnv，必须等于 accepted state 的 `mir_identity`）。不含 guest 值/
  PASS/活 env/cell/宿主地址。
- **required 闭包**：manifest 以 `RefKind::RequiredData` 指向每个单元 ABC 与
  声明上下文 artifact；可见 root 因此始终闭包在存活 required 数据上。
- **原子发布顺序**：写声明上下文 → 写 manifest → 取 OS 租约 → 注册**新**
  root → 摘旧 root → 返回 ticket。任何间隙失败（可注入的 `PublishSeam`：
  after-context/after-manifest/before-root/after-root/before-return）都不交
  ticket；写前失败只留旧 root，写后失败至多旧+新并存，退役后仅新 root——
  三者都是完整闭包，旧 binding 仍按 caller 的 id+digest 可读。
- **CAS**：`accepted_generation` 不得倒退；陈旧发布在任何写之前以
  `recovery.stale-generation` 拒绝（当前 root 的 generation 经**校验和验证的
  提示读**获得，只能导致拒绝不会导致接受）。

## 恢复（B 面：P=0）

新进程只用 (expected manifest id, expected digest, 源全文, 配置) 入：

1. 按 id **有界**读取 manifest（`read_bounded`，超预算在读前拒 →
   `recovery.oversized`），用请求侧事实核 producer/schema/owner/语义/快照；
2. 重算 payload SHA-256 与 caller 传入的强摘要常数时间比较；
3. 绑定 source SHA/长度、logical owner/entry/profile/capabilities 与 producer
   身份；
4. 按 id 读声明上下文（同一有界入口 + 摘要 + 字节数），解码
   `ace.closures.context-dump/1`，并要求它与 manifest 描述同一 accepted state；
5. 逐单元按 id 读 ABC wire（有界）→ AVEN envelope 身份（producer/schema/
   owner/unit/body_fp）→ **payload 自身 fnv 必须等于记录指纹** → R 轴
   `verify_abc_bytes` 深验 → host capability 授权 → `FunctionCode::admit_unit`；
6. spanless identity wire 的 fnv 必须等于 accepted `mir_identity`；
7. 用**已验单元的 MIR 事实**作交叉核对重建安全前端上下文（见下），再
   `restore_install`：与 `apply_revision` **同一安装事务**，不跑 verify_mir/
   encode、不写 store、不取读侧租约；`close` 亦不触 store（只读正例）。

失败分态（不静默 cold、不重写 PASS）：`recovery.miss`／`oversized`／
`envelope`／`kind`／`unit-fingerprint`／`digest`／`source-mismatch`／
`context-mismatch`／`capability`／`stale`／`evidence`／`identity`。

## 可移植声明上下文与下一次编辑

`ace.closures.context-dump/1`（`core/src/source/closures_context.rs`）保存暖路径
真正读取的**声明**：每函数签名、resolver 五表（records/lists/variants/
function_types/sigs/rec_index）、单元目录（wire 身份、kind、逻辑 ID、站点事实、
span/body_digest/mint_span）、registry 声明与 creator→sites 索引、
generation/context_revision/next_logical/trace/profile/caps。**不含任何 body**。

`restore_context` 把 dump 当作未授信输入：schema/source SHA/module/caps/bounds/
空上下文/catalog↔函数配对/重复项逐项拒，并**与已解码单元 cross-check**
（lambda 的 MIR env arity == registry env 长度；creator 的 `CreateClosure`
站点 callee 顺序与 capture-mode 序列 == registry `sites_of`+`site_modes`；
owner 单元不得带 env）。registry 由 `ClosuresVerifyContext::from_declarations`
重建，并重跑冷构造的跨 body 不变量（每 lambda 恰一处创建点、声明的 creator
必须列出它、site_modes 与 env 等长）。

恢复上下文里未改函数是**只有签名的条目**（空 body/locals），只作文档脚手架：
单 lambda body 编辑沿 912 暖路径（fragment parse → 上下文中重解析 → 单体
typecheck → 作用域文档 → 凭证 → 单单元 lower/encode/安装），未改单元 P/IO 为 0
且对象/代际保持；需要保留 body 的 **owner 路由**在恢复上下文里是具名非快
`NonFastReason::RestoredContext`（调用方走整模块 truth 路径，whole 轴如实非零）。

等价 oracle：恢复上下文与冷整模块路径对同一编辑必须产出**逐字节相同**的
scoped Atom 文档（`closures_context_913` 用例）。

## 预算、轴与观测

- source ≤4MiB、单 manifest ≤4MiB、声明上下文 ≤4MiB、单 wire ≤8MiB、
  单元数 ≤4096（全部在读前判）；恢复自有 namespace ≤128MiB。
- 轴：P（源生产，正例必须 0）、R（manifest/unit decode、`verify_abc_bytes`、
  上下文重建；按单元记 `Stage::RestartDeepVerify`）、M（hash/元数据/坐标）。
  IO 分 read/write/root/lease；只读恢复数据/root 写 0（store 自身 `locks/`
  存活锁被 open 刷新属边界，非数据写）。
- 正式数字（68 顶层函数/70 单元，30 样本×4 态）：fresh P90 634ms、hit 5ms、
  restart（新进程，caller 可见）53ms（恢复执行腿 39ms）、restore+edit 42ms，
  全 ≤2000ms；required data manifest ≤45 165B、wire ≤14 749B。

## 相关

[闭包单函数生产](closures-function-production.md)（912：accepted source state）、
[闭包环境](closure-environments.md)、[常驻函数安装](resident-function-install.md)、
[源码闭包域](../auto-ac/source-closures.md)、[AV 缓存与验收](cache-and-conformance.md)、
[产物契约](../auto-ac/artifact-contract.md)、[ACE M1 一致性语料](../auto-test-case/ace-m1-conformance.md)。

## PLAN-929：恢复发布的版本化 required 边（推广面）

- 913 深重验模型推广为**声明面环境清单**（`core-decl-env/1`：签名/
  参数 mode/receiver/记录形状/VT 池；无 body/指针/trusted 位）——
  恢复侧从清单装配 stub-body `CheckedEnv`（`recheck_swap` 单体事务
  自带给定 body，未改函数 body 暖路径永不被读），恢复后首个暖编辑
  **整源 parse/resolve=0**（whole 轴观察）。
- 身份对账（profile/caps/source sha/池索引越界）使用前受控拒；发布
  幂等（同键同字节不再 put；只读冷装零发布）。
