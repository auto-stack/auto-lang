# PLAN-796 规范增量（spec-delta）

2026-10-16；stage:work（T-07 T-06 无关部分交付）；锚定代码 `5a8cd61ba`+本轮；
冻结决策出处：[contract-decision.md](contract-decision.md)（D1–D12）。
**本文是评审用 delta 文档，不是已发布的规范**——canonical Spec 落盘由 merge 阶段执行
（`/auto-plan:merge` + specs.json upsert + spec-index.py）。SD-07/08 显式 pending D-788。

## 状态总览

| delta | 状态 | 说明 |
|---|---|---|
| SD-01..04 | 内容终稿（随 T-01..T-05 实现） | 新 auto-semantics 规范四件 |
| SD-05/06 | 内容终稿 | 对既有 auto-ac/auto-hir 规范的追加段（不改原producer范围） |
| SD-07/08 | **pending D-788** | 788 合入后在 T-06 单写者窗口定稿（probe adapter/recipe/decoder 能力版本） |

## SD-01 add `docs/specs/auto-semantics/project.md`

- before：无 auto-semantics 模块导航。
- after：新增模块总览——三类数据分离（Semantic Facts / Inference Work State /
  Runtime Reflection Projection，Design 42 §2）；新独立 workspace
  `experimental/ac-semantics`（contract/core/probe 三包，core-ast feature 隔离后端依赖，
  默认构建 `cargo tree` 零 auto-lang/UI/VM/Cranelift/auto-man，sema_boundary_ 断言）；
  支持矩阵引用 SD-02 的 category matrix（15 类：6 Implemented / 7 SchemaOnly / 2 Future，
  `sema_identity_matrix_invariants_hold` 锁定）；验证入口
  `scripts/verify-ac-semantics-796.ps1 -Scope acceptance|closeout`。
- 验收：AC-01/06；测试 sema_identity_、sema_boundary_。

## SD-02 add `docs/specs/auto-semantics/semantic-contract.md`

- before：名义身份/版本/递归组无统一合同，各后端自持类型副本。
- after：`LogicalId = blake3(producer‖"ac796/logical/v1"‖len₈·(package,module,kind,name))`
  ——名义相等、重启 intern 序无关、同名跨包永不合并；`DeclRef`（logical+组件+ContentVersion）
  重导校验防篡改；ref envelope `ac796/semaref/v1` owner/schema admission；描述 codec
  `ac796/sema-page/v1`（TypeDesc 13 变体全集含 view/mut/owned，递归 record 一律间接
  DeclRef 组 RecGroup，组版本成员序无关，>8192 嵌套受控 TooDeep，≤512KiB 页）；
  category matrix 全表（owner/schema/消费者/粒度/ref/驻留/现状/future owner）。
- 验收：AC-01；测试 sema_identity_ 14 例。

## SD-03 add `docs/specs/auto-semantics/inference-profile.md`

- before：无独立推导 profile（旧 InferenceContext 混合 Database/TypeStore/工作态）。
- after：有限 i32/bool 双向推导+显式 worklist（`ac796-bidir-worklist/1`）；未注解绑定
  铸类型变量、union-find 代换、循环代换 occurs 受控诊断（不挂死）；unsolved/歧义候选/
  unknown-name 为确定性诊断，**无 Any 兜底**；资源/取消为基础设施态永不缓存；步数定义
  （worklist 解一/AST 节点一/作用域查一，≤1M 步 ≤5s）；差分对照合同——与现有 core
  resolve+typecheck 独立运行，接受/拒绝与绑定类型（程序序）逐对一致（240+ 生成例）；
  推导工作态（变量/代换/worklist）永不持久化。
- 验收：AC-02；测试 infer_core_ 13+4 例。

## SD-04 add `docs/specs/auto-semantics/query-cache.md`

- before：A06 无消费者，无语义缓存合同。
- after：查询键组件（query/package/module/algo/schema/producer/target-sem/caps + 按类
  输入指纹，InferBody 含 **called-sigs-fp**——被读签名变化=键变化，一键不容两容，
  783 Conflict 不得绕过）；依赖 manifest（正：config/fn-sig/fn-body/field-set；负：候选
  集域内名字缺席+集合指纹；source-map fail-closed）命中时整体重验；783 受管隔离
  namespace（layer `S796`、SnapshotRecord 载体、非 Hir 非 A06，读入三重 admission）；
  `lookup_only` 无 compute 路径（禁调反控结构性）；预算（热页 LRU≤8MiB、scratch≤8MiB、
  受管≤16MiB、页流式≥32MiB、private Δ≤64MiB、深链≥4096 全迭代、512 换轮 GC 收敛+
  哨兵保活）；六场景 trace（请求/缓存读验/推导/发布不重叠计时，30 原始样本不删）。
- 验收：AC-03/04；测试 sema_cache_ 12、sema_budget_ 5、sema_turnaround_ 5；报告
  `docs/reports/plan796/turnaround.{md,json}`。

## SD-05 modify `docs/specs/auto-ac/artifact-contract.md`

- before：A06 semantic-interface-meta 为 Future 登记（producer "resolve/typecheck"），
  无语义 producer 消费先例。
- after：**追加**一节"796 语义 producer 的 A06 接缝"：S796 namespace 使用 783 通用
  块/图 API 作容器（SnapshotRecord 载体、producer/schema admission、lease/root 复用）；
  原有 Future 登记与原 producer 范围**一字不改**（796 不冒充旧 producer 已支持 A06）；
  A05/A12 接缝仅 schema 冻结（CT seam `ac796/ct-seam/v1`、RuntimeProjectionPolicy 四级，
  model_compute 标记，vm_execute 由真 AVM 路径独占）。
- 验收：AC-01/03；测试 sema_identity_、sema_cache_、meta_query_（CT 接缝）。

## SD-06 modify `docs/specs/auto-hir/stage-contract.md`

- before：语义事实与 checked 准入无信任边界描述。
- after：**追加**一节：语义事实（InferFacts，无工作态）是 checked 准入的输入之一；
  有限 profile 的推导结果不残留未解变量即可准入，资源/取消状态不得作为准入裁决；
  旧实现（infer 嵌 Codegen 等）不切换，仅作为差分对照 oracle 被只读引用。
- 验收：AC-01/02；测试 infer_core_ parity。

## SD-07 modify `docs/specs/auto-test-case/architecture.md` — **pending D-788**

- before：TestCase 架构为 v1 支持边界（无 probe adapter/recipe 注册面）。
- after（定稿于 T-06 单写者窗口，788 合入后）：受限 probe/run adapter 公共接缝——
  Atom 数据 source kind、声明式 registered recipe（语义 recipe 首个实现，797 按同合同
  追加 ABI recipe）、结构化结果/诊断/trace；admission 只接受已登记 recipe 与可信工具
  身份，case 不携 shell/任意 exe；复用 788 worker/产物身份/预算/containment。
- 验收：AC-05；测试 probe_adapter_、AutoDown 统一 runner 真执行（V-08）。
- **本轮状态**：未实现、未定稿——.case.ad 三份仍标 not-executed（诚实 pending）。

## SD-08 modify `docs/specs/auto-test-case/profile-v2-native.md` — **pending D-788**

- before：v2 native/syntax 能力版本，无 probe/Atom 数据 kind。
- after（定稿于 T-06）：显式增加 probe/Atom 数据 kind/观察与能力版本；decoder 与报告
  同步；已有 ac/syntax/v1 行为保持不变（兼容正例在 T-06 验证）。
- 验收：AC-05/06。
- **本轮状态**：未实现、未定稿。

## 依赖与并行审计（V-09 边界，随 sema_boundary_ 落地）

- **796 依赖**（只读消费）：ac-core 公开 AST/parse（feature 门控，默认零后端依赖）；
  783 ManagedStore/codec/identity（零修改，merge-base..HEAD diff 于
  experimental/ac-artifact-probe、experimental/ac-core、crates/、根 Cargo.toml 均为空）。
- **依赖 796**：797（ABI probe）只读消费 ref/description 快照（D6 全集 +
  ac796/ct-seam/v1 已冻结可读）；共享接口变更须通知式收据（本轮无变更）。
- **冻结版本出处**：全部接口常量（producer/三个 schema 域/algo id/layer/载体）在
  contract-decision.md 有文可查（sema_boundary_frozen_versions_have_provenance 锁定）。
