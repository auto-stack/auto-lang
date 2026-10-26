# PLAN-823 contract-decision.md — T-01 冻结决策（av.closures/1 模块子集与集成边界）

2026-10-26，work stage T-01（用户以 /auto-plan:work 明确实施授权）。本文件冻结
[PLAN-823](../../823-av-owned-closure-captures.md) §5.5/r3、r6 并行合同（[审计](../reports/ace-parallel-dependency-audit-2026-10-26.md)）
所要求的支持矩阵、profile/codec 命名、访问面、预算与 oracle 映射；未知项不留给
下游猜。任何变更须回到本文件并记 revision。

> **r1（2026-10-26 T-10 落地）**：§3 签名按实现定稿——`commit` 只收 **move 源**
> （第 k 个绑定第 k 个 Move spec，clone 源始终留创建者，符合 §2.1"clone 不消费源"）；
> 错误变体携带上下文字段；`EnvPlan`/`ClosureEnv` 带 Drop（逆序清理即回滚律）。
> 其余决策不变。
>
> **r2（2026-10-27 T-02 集成窗口，D822 已合入）**：见 §8。

## 1. 调查基面与依赖门核实

| 项 | 实际状态（2026-10-26 核实） |
|---|---|
| 主检出 | master@ff7547f9d（822 needs_fix 修复簿记，非代码面；`crates/**`、`test/**` 零未提交改动） |
| D821 | **满足** — archive/821 r6 9/9，delivery 92b8daf60，门禁全绿（裸 t 3185/3186 + tv 163/163 + acceptance 八 Scope） |
| D828 | **满足** — archive/828 r3 11/11（函数级增量编译/装载/snapshot） |
| D822 | **未满足** — executing r5，R1 needs_fix 修复已落地（ff7547f9d）再送审 execution_done；review/merge 未收口。**T-02 集成门继续阻塞**，本轮仅 T-01→T-10 |
| 817/818 | 均已 archived；其兼容面（V-09 重放）入口在 probe `native_admission`/`native_reference` 与既有 recipe，T-05 冻结实际清单 |
| probe workspace | `experimental/ac-av-probe/`（798 系独立 workspace，members=core+probe）；`parallel/` 目录不存在，`core/src/closures.rs` 不存在（目标均为新文件） |

**公开 API 指纹**（auto-av-core @ ff7547f9d，SHA-256 前 16 位，逐文件独立复跑核实）：
lib.rs `fe57a8f58bd87dc2`、contract.rs `1b04fc6f8d2a892d`、owned_text.rs `b8846a7e90045057`、
place.rs `c7de6e9e56441e33`、mir.rs `8d4ec17ebc560c91`、ownership.rs `d64b067fac1ec0b1`、
av.rs `a0f2e2305b3df91b`、mir_verify.rs `69542107acf6c2f4`、instr_spec.rs `85f133ff02e3b145`。

## 2. 支持矩阵

### 2.1 T-10 独立模块子集（本轮交付，无公共写面）

单一生产源 `experimental/ac-av-probe/core/src/closures.rs`（新增、**未在 core/lib.rs
注册**——core 编译面零变化），实现：

| 面 | 冻结决策 |
|---|---|
| 捕获源类型（821 已交付值面） | 支持：`I32/Bool/I64/U64/F32/F64/Char`（Copy 标量，move=按值复制）、`Str`（owned 句柄整体转移）、`Record`（纯值 record 及含 Str 闭包的资源 record，按 `ownership::record_reaches_str` 语义） |
| 拒绝的捕获源 kind | `StrView`（借用不可逃逸，§5.1.3）、`MutRef`（独占写 token）——prepare 期 `env.capture.unsupported-kind`，且发生在任何 clone 分配尝试**之前**（attempts 不增） |
| 捕获模式 | `Move`（整体转移，提交期发生）/ `Clone`（显式深 clone：每个可达 owned Str 分配独立新缓冲，prepare 期执行、可失败） |
| 借用捕获 | **不在模块子集**（静态可证非逃逸分析属 verifier+executor，T-03/T-04） |
| 生命周期 | `prepare`（可失败：arena 故障/预算/kind 拒绝）→ `commit`（纯验证 + 无分配转移；失败原样返还 sources）→ `ClosureEnv`（owned 聚合，字段按捕获序）→ `Drop` 逆捕获序释放（821 逆创建序同律） |
| 失败保源 | prepare 全程只借引 sources（`&[CaptureSpec]`）；任何 prepare 失败零消费待 move 源；已 staging 的 clone 逆序清理（drop → arena 记 free） |
| List | 不可表达——`Value` 无 List 变体（D822 未合入）。**诚实边界**：这不是本模块的拒绝规则，是依赖接缝；D822 合入后 T-02 刷新 |
| 间接调用 / ClosureRef / env store / generation | **全部不做**（"不实现第二套间接调用器"）；env 身份/generation/句柄失效属集成（T-02..T-04） |

### 2.2 T-02+ 完整 profile 面（窗口后，仅冻结命名与请求，不实现）

| 项 | 冻结/请求 |
|---|---|
| 执行 profile 名 | **`av.closures/1`**（字符串确认与已交付 `av.core-i32/1`/`av.data/1`/`av.owned/1` 及计划命名族无冲突，冻结） |
| MIR schema（请求） | `auto.mir.av-closures/1`；语义版本 `MIR/ABC/AV_CLOSURES_SEMANTICS` 均 (1)（请求，窗口登记时生效） |
| ABC wire（请求） | AVAD 魔数族下一 free wire version 与 numeric profile-id；当前观察 free = v3/id4，但 822 在途可能先占——**编号由核心 owner 在 T-02 窗口统一登记，本计划不自行占号** |
| 指令（请求） | `CREATE_ENV/CAPTURE_FIELD/CALL_CLOSURE/DROP_CLOSURE` + InstructionSpec 行（含 owner/清理语义列）；opcode 由核心 owner 分配。§5.4 合同：环境所有权/调用合同入 spec 行，语义元数据不得被 strip debug 移除 |
| 注册（请求） | recipe `av-data-823` 编号、`test/cases/av-data/823/`、脚本 `scripts/verify-av-closures-823.ps1`（均 T-02 建） |

**版本/窗口负控（V-14 冻结）**：当前 `mir_verify::run` 对 `av.closures/1` 声明模块的
拒绝是**真实既有行为**（`mir.profile` unsupported——profile 白名单三成员）。T-10 harness
以此作诚实边界测试：能力未注册前，任何 closures profile 模块不得通过 verify_mir。

## 3. T-10 访问面（API 冻结）

```rust
// core/src/closures.rs —— 模块子集公开面（同源文件，窗口后原样进 core）
pub const MODULE_SUBSET: &str = "plan823/closures-module-subset/1"; // ≠ 任何已注册 profile

pub enum CaptureMode { Move, Clone }
pub struct CaptureSpec<'a> { pub mode: CaptureMode, pub src: &'a Value }

pub struct EnvLimits { pub max_fields: usize, pub max_env_bytes: usize } // Default 取 §4 冻结值

pub enum EnvPrepareError { TooManyFields{got, max}, EnvBytesBudget{would, max},
                           UnsupportedKind{kind}, FaultedClone{attempt} }
pub enum EnvCommitError  { SourceCountMismatch{expected, got}, SourceKind{index, expected, got},
                           EnvBytesBudget{would, max} }

pub struct EnvAccounting { pub bytes: usize, pub fields: u64 } // 计量回传，executor 记 fuel

pub struct EnvPlan { limits: EnvLimits, planned: Vec<PlannedCapture> } // PlannedCapture 私有：Move{kind}/Clone{staged}
impl EnvPlan {
    pub fn prepare(arena: &StrArena, specs: &[CaptureSpec<'_>], limits: &EnvLimits)
        -> Result<(EnvPlan, EnvAccounting), EnvPrepareError>;   // 借引、可失败、失败逆序清理（Drop 即回滚律）
    pub fn commit(mut self, move_sources: Vec<Value>)           // 只收 move 源：第 k 个绑定第 k 个 Move spec；
        -> Result<(ClosureEnv, EnvAccounting),                  //   clone 源始终留创建者，不提交
           (EnvCommitError, Vec<Value>)>;                       // 无分配；失败原样返还 move 源
}

pub struct ClosureEnv { fields: Vec<Value> }                    // 私有；按捕获序
impl ClosureEnv {
    pub fn field_count(&self) -> usize;
    pub fn field(&self, i: usize) -> Option<&Value>;
    pub fn deep_size(&self, visited: &mut u64) -> usize;   // 与 Value::deep_size 同规则（按字段计）
    pub fn into_fields(self) -> Vec<Value>;                 // 集成期责任转出缝
} // Drop：逆捕获序逐字段释放（pop+drop），arena free 事件即逆创建序 oracle

/// 深克隆规范实现（显式栈、无语言深度截断；Str→arena 新缓冲，record 逐字段重建）
pub fn deep_clone_value(val: &Value, arena: &StrArena)
    -> Result<(Value, EnvAccounting), u64>;                 // Err = arena 故障 attempt#
```

错误码（阶段/码冻结）：`env.capture.too-many-fields`、`env.capture.bytes-budget`、
`env.capture.unsupported-kind`、`env.capture.faulted-clone`、`env.commit.source-count`、
`env.commit.source-kind`、`env.commit.bytes-budget`；资源/不支持/故障三态分立，不以
panic/timeout 充负例。

**同源接缝（harness 可编译最小缝，T-10 第一件事）**：

```
experimental/ac-av-probe/parallel/823/          # 独立 workspace（空 [workspace] 表脱离父树），自有 lock
├── Cargo.toml    # 唯一依赖 auto-av-core = { path = "../../core" }
└── src/lib.rs    # pub use auto_av_core::{contract, owned_text, ownership, place};
                  # #[path = "../../../core/src/closures.rs"] pub mod closures;
```

- 依赖方向：harness → auto-av-core（path 依赖）+ closures.rs → `crate::{contract,
  owned_text}` **公开项**（lib.rs re-export 解析）；无环，不改 core/lib.rs、不改父
  workspace members、不建 junction。
- closures.rs 可达符号（冻结清单）：`Value/RecordValue/StrHandle/StrView/MutRefV`
  （contract）、`StrArena/AllocEvent/FreeEvent`（owned_text）、`MAX_RECORD_FIELDS/
  MAX_AV_SLOT_BYTES`（contract 常量）、`MirModule::new_owned`、`verify_mir`（仅 harness
  负控测试用，closures.rs 本体不引用）。
- **缺口登记（唯一已知私有缝）**：`av.rs::clone_owned_value` 为模块私有，harness 内
  不可达。裁定：closures.rs 自有 `deep_clone_value` 为**规范实现**（语义镜像 821 CLONE：
  每可达 owned Str 一新 arena 缓冲、显式栈、故障时无半提交）；T-03 核心窗口将 av.rs
  `Clone` 指令实现**改接 `closures::deep_clone_value` 收敛单一实现**（av.rs 一处薄改，
  窗口内允许）。等价 oracle = 821 clone 族重放（probe `av_owned_821_semantics/
  resources`）+ 本模块 V-14 用例双绿。不申请 pub/pub(crate) 导出窗口（不为并行期开公共面）。

## 4. 预算与计量（模块级切片；完整资源表沿 plan §5.2/§5.5 不放宽）

| 项 | 冻结值 | 依据 |
|---|---|---|
| `MAX_ENV_FIELDS` | **1024** | 对齐 `MAX_RECORD_FIELDS`（env 为 owned 聚合，同族表上限） |
| 默认 `max_env_bytes` | **`MAX_AV_SLOT_BYTES` = 8 MiB** | 值数据合计口径（槽+聚合+buffer+环境+Dyn）；executor 侧自行切片 |
| 计量规则 | `deep_size` 按字段计（与 `Value::deep_size` 同律）；clone 按输出字节数 + 访问字段数（与 av.rs CLONE 的 fuel 口径一致） | 821 contract-decision §4/§6 先例 |
| 遍历 | 显式栈，**无语言深度截断**（F821-R2/F06 收口先例：深度由表预算约束，违约→受控拒绝不爆栈） | plan §5.2 "有界显式遍历" |
| cap 两侧输入（T-01 冻结，防边界漏测） | fields=1024 过 / 1025 拒；env bytes 恰在 cap 过 / 超 1 单位拒（两处：prepare 期 staging 总量、commit 期最终总量——防 caller 以同 kind 不同内容源在两相间漂移超限） | 必需正反例 |
| 预算前拒 vs 故障 | 预算拒绝发生在任何分配尝试**之前**（attempts 不增，对齐 V-07"预算前拒 attempt=0"）；arena 故障注入则 attempt 存在而 successful_alloc 不增 | owned_text::try_alloc 既有分态 |

## 5. oracle 映射（case → 子case → 入口 → 阶段/码 → oracle → 必需/适用）

### 5.1 T-10 模块子集用例（本轮真实入口，前缀 `av_closures_823`，selected>0）

| 用例 | 子case/输入 | 入口 | 阶段/码 | 区分性 oracle |
|---|---|---|---|---|
| `t10_prepare_commit_mixed` | move+clone 混合：Str、资源 record、标量 | harness libtest | — | env 字段值/序正确；commit 消费 sources；arena live 反映转移 |
| `t10_env_drop_reverse_once` | drop env；clone 副本与源分别 drop | 同上 | — | live=0；free 事件序=逆创建序；每缓冲恰释放一次（无双重 free） |
| `t10_faulted_clone_sources_readable` | `set_fault_at(n)` 第 n 个 clone 失败 | 同上 | `env.capture.faulted-clone` | **全部**源仍可读（字节完整）；已 staging clone 逆序回滚（live 回落）；attempts 增而 successful_alloc 不增 |
| `t10_move_source_preserved_on_failure` | 先 move 候选后 clone 故障（V-12"先move后clone失败"模块轴） | 同上 | 同上 | 待 move 源未被消费、仍可读（prepare 只借引） |
| `t10_borrow_kinds_rejected` | StrView/MutRef 捕获 | 同上 | `env.capture.unsupported-kind` | 拒绝先于分配（attempts 不变）；无 staging 残留 |
| `t10_commit_drift_rejected` | 错数量 / 错 kind / 同 kind 超预算内容 | 同上 | `env.commit.source-count` / `source-kind` / `bytes-budget` | sources **原样返还**（可读、归 caller）；staged 已逆序清理 |
| `t10_budget_boundaries` | fields 1024/1025；bytes 恰 cap/超 1 | 同上 | `env.capture.too-many-fields` / `bytes-budget` | cap 侧两侧各验；预算拒 attempt=0 |
| `t10_unknown_profile_rejected` | `MirModule` 声明 `av.closures/1`（owned schema）送**当前** `verify_mir` | 同上 | `mir.profile`（既有码） | 真实拒绝——能力未注册不得过验（V-14 版本/窗口负控） |
| `t10_module_subset_identity` | `MODULE_SUBSET` 常量 | 同上 | — | ≠ 任何已注册 profile 字符串 |

### 5.2 集成期用例（T-02+ 真实入口；本轮仅冻结映射，未运行）

| Case | owner | 拟入口（T-02 建） | 模块子集已覆盖轴 |
|---|---|---|---|
| V-01 逃逸正例 | T-02 | 新 profile verify/ABC/AV + first-case 脚本 | 捕获构造/drop 清理 |
| V-02 借用边界 | T-03/04 | verifier 前门 + 运行 | —（借用不在子集） |
| V-03 嵌套/帧复用 | T-03/04 | 同上 | env 容器/释放序 |
| V-04 错签名/generation/跨store | T-03/04 | CallClosure 前门 | —（invoke 不在子集） |
| V-05 构造失败逆序清理 | T-03/04 | admission/resources | **prepare 失败逆序清理、保源**（同因轴已验） |
| V-06 调用期清理 | T-03/04 | semantics | env Drop 单次释放轴 |
| V-07 资源分态 | T-04 | worker budget/allocator hook | **预算前拒 attempt=0 / 故障 attempt 存在**分态轴 |
| V-08 恢复/失效 | T-02/05 | 783 恢复→verify→AV | — |
| V-09 旧 profile 兼容 | T-05 | 旧 recipe/consumer 重放 | — |
| V-10 AutoDown 反控 | T-06 | 正式 recipe + 父侧 oracle | — |
| V-11（r2）旧根/模式/代际/strip | T-03/04/05 | §3 真实 MIR/ABC 授信 | prepare/commit 的保源与逆序清理为"每一字段仅清理一次"轴 |
| V-12（r3）prepare/commit 提交前后 | T-03/04/05 | 真实 profile verifier/AV | **提交前源保留、clone 失败不消费待 move 源、逆序清理**（模块轴已验）；提交后/取消/父子转移属集成 |
| V-13（r4）增量四子case | T-09 | D828 edit 接口 consumer | — |
| V-14（r6）同源/模块 | T-10+T-03/05 | 本轮 harness + 窗口后生产 adapter | **§5.1 全部九用例即 V-14 模块轴**；生产同源/824 拒绝轴待窗口 |

## 6. 写面登记（823 私有；公共编号仅请求）

| 项 | 登记 |
|---|---|
| Plan/revision | PLAN-823 r6（T-01 后 executing） |
| base | master@ff7547f9d（建树时以实际 HEAD 为准并回填） |
| 私有写面 | `experimental/ac-av-probe/core/src/closures.rs`（未注册新文件）、`experimental/ac-av-probe/parallel/823/**`、`docs/plans/823-*.md`+`attachments/823/**`（主检出簿记）、`scripts/verify-av-closures-823.ps1`（T-02 建） |
| 只读依赖 | auto-av-core 公开 API @ §1 指纹 |
| 消费者 | 暂无生产消费者（T-03 薄 adapter 接入；av.rs CLONE 改接为同源收敛点） |
| 对核心 owner 的请求 | av.closures/1 注册、AVAD 族 wire version/profile-id、Closure 指令 opcode+InstructionSpec 行、recipe `av-data-823` 编号（T-02 窗口统一登记，不预占） |
| 开始/释放 | 2026-10-26 起；T-02 窗口交接时释放模块独占面 |

## 7. 未知项处置

无遗留未知项：List 接缝按 D822 记录（不可表达≠拒绝）；借用分析/invoke/env 身份
属 T-03/T-04 集成（§5.2 已映射）；wire 编号待窗口由核心 owner 定（不猜）；commit 期
"无取消观察"由纯代码性质结构保证，运行期取消语义 T-04 用 executor 注入验证。

## 8. r2 集成窗口定稿（2026-10-27，D822 合入后 T-02 落地）

**基面**：master@c6c5ddb25（822 archived 9/9 后），plan-823-dev rebase 至 b095dc460，T-02 交付 @e9bb30606。

| 项 | r2 定稿（原"请求"已由本窗口登记为实际编号） |
|---|---|
| wire/编号 | **AVAD 族 wire version 4、profile id 5、类型 tag 16（TY_CLOSURE）、opcode 0x2F/0x30/0x31**（822 占 v3/id4/tag13-15/0x1D-0x2E 后的下一空位）；820 pin 46→49 刷新 |
| 指令集（首例面） | `CreateClosure{dst,callee,captures:[CaptureOperand{clone,src}]}`、`CallClosure{dst,closure,args}`、`EnvGet{dst,field}`；闭包释放**复用 Drop**（typed 槽派发 env store 释放路径） |
| env 模型 | 执行器 LoopState 持 `closure_envs: HashMap<u64, ClosureEnv>`，handle 单调不复用（gen=handle）；env 字节计入 slot_bytes 单行（值数据 8MiB 口径）；ClosureRefV{func,sig,env,gen} 为 Copy 元数据，**Return 转义时先零槽再帧清扫**（元数据无共享计数——§6"返回owned结果先确认责任转出"的闭包实例）；帧清扫/Drop 经 release_owned_worklist 显式工作清单 |
| 唯一 site 规则 | verify 预扫描：每 callee 恰一个 CreateClosure site，其捕获源槽类型序列 = env 布局（EnvGet 类型依据）；重复/冲突 site、entry 为 body、直接 Call body 均拒绝（`mir.closure.site-*`/`entry-body`/`body-direct-call`） |
| 捕获矩阵（实测） | 标量/Str/List/Variant/Record ✓；StrView/MutRef/**Place**/Closure 值拒绝（`mir.capture.kind`，attempt=0）；嵌套闭包捕获=T-03/T-04 面（诚实拒绝） |
| 深克隆收敛 | **closures.rs 委托 `av::clone_owned_value`**（822 已升 pub 的规范访客：cancel/wall/fuel/部分回滚）——T-10 本地走查退役，单一实现，822 clone 族重放为等价 oracle |
| CaptureOperand 所有权 | clone 读不杀；move 经 ownership 分析杀源（镜像 MOVE：活跃借用冲突拒绝）；CallClosure 参数按 Closure 签名 sig 驱动 move-in 杀（镜像 CALL） |
| 首例 oracle（实测） | 手算结果 **29**（2*10+4+5）、ledger 5 alloc/5 free、live=0、decode→re-encode byte-stable、code-cache 恢复同果+types-fp/entry/snapshot 身份轴 clean miss、四负控真实拒绝 |
| 遗留 T-03+ 面 | borrow 捕获与逃逸前门、V-02..07 反控、环境嵌套/帧复用（V-03）、错签名/generation/跨 store（V-04）、824 交叠拒绝、AutoDown 正式 recipe（V-10 正控全集）、P95/内存——按原 §5.2 映射推进 |
