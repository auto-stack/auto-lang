# 闭包单函数前端生产（PLAN-912）

> 状态：active（实现=`experimental/ace/crates/core/src/source/closures_fn.rs` +
> `core/src/verify.rs::verify_closures_function` + `lowering/src/hir_adapter.rs::lower_function_closures_unit_checked` +
> `av/src/closures_fn_session.rs`，正式入口 recipe `ac-av/closures-fn-run-v1`）。
> 由 PLAN-912 交付（2026-11-02）；承接 865 源码闭包与 909 owner 级缓存，关闭其 R1/R2/R4
> 与单 body 暖编辑中隐藏的整源重解析；R3（新进程恢复前置生产）属 [913 来源绑定恢复]
> (../plans/archive/913-closures-source-bound-recovery.md 的前置)，本规范不宣称已闭。

## 入口与路由

- 冷路径（truth）：整模块 `resolve → typecheck → emit → bind → verify_closures`，同时
  驻留 `ClosuresFunctionFrontend`（保留 `CheckedEnv` + 每顶层函数目录 `UnitEntry`
  （item/body/mint/current span、已验 body digest）+ 每lambda `LambdaSiteFacts` 创建点事实 +
  `ClosuresVerifyContext` 受检 registry + 闭包签名表快照）。
- 显式暖编辑入口 `edit_body(base_source_sha256, UTF-8 range, replacement)`；`apply_source`
  是其差分包装——diff/片段解析/簿记全部在计量窗口内，不藏整源 parse。路由：
  - **lambda 单 body**（快路径）：`parse_closure_fragment` 仅词法/解析目标 lambda range →
    `resolve_lambda_in_context` 在保留创建点事实上重解析该 body（raw capture uses /
    params / move / result 任一变化＝具名 `LambdaNonFast` 转owner路由）→ 单体
    typecheck → 单函数作用域文档（其余单元 body-less 脚手架）→
    `verify_closures_function` 凭证 → `lower_function_closures_unit_checked` →
    至多该一单元提交。
  - **owner body**（含 lambda 创建点事实变化）：`parse_function_fragment` 解析 owner item →
    `resolve_owner_in_env` 经 retained-lambda 复用 oracle 重解析 owner——文本字节相同且每个
    保留捕获按**当前**创建点 locals 经唯一决策表 `decide_capture` 重推全等的 lambda 连同其
    嵌套 lambda（span 按外层位移平移）原样推入，不走其 AST/语义 body；真变化者与 owner
    一起进 batch 作用域文档（复用兄弟保持脚手架）。提交粒度＝wire 字节对比，未改单元
    同对象/同代际。
  - **具名非快路径**：`OwnerSignatureChanged` / `OwnerTopologyChanged{before,after}`（lambda
    数变；cold 重建经 `cold_carrying` 按 (owner, 文本) 唯一匹配保留 lambda 逻辑ID，歧义/新增
    铸新ID）/ `CrossesUnits` / `StaleBase` → 整模块 truth 路径，whole 轴如实非零。
- 会话（`ClosuresFunctionSession`）状态：current spanless identity wires、frontend、
  `WarmGateReport` 结构门。zero-hit（同源重放）＝全阶段零生产但仍 link+execute。

## 单函数凭证与 registry

- `verify_closures_function(bundle, def_index, &ClosuresVerifyContext) -> CheckedFunction`：
  仅验证器铸造；`domain=Closures`，携带 coll_facts（cleanup）与 `ClosureBodyFacts`
  （lambda 自身 env 布局含 cell 位；creator 每创建点的 callee env 与 operands）。
  registry（`ClosuresVerifyContext::from_checked`，仅由整模块 CheckedModule 构建）承担
  跨 body 义务：callee env 声明（作用域文档中 callee 为脚手架时——整模块路径此处原本
  **静默跳过**，现为 `verify.closure-env-unknown`）、创建点唯一/关联
  （`verify.closure-site(-duplicate)`）、禁止直接调用 lambda body
  （`verify.closure-body-direct-call`，原 MIR-only）。owner batch 的 registry 变化经
  `overlay_from_batch` 从真实 body 重推，仅在会话安装成功后随 `commit_candidate` 落位。
- lowering 消费凭证事实（`ClosureFacts` trait：整模块扫描 `ModuleScanFacts` 保持原推导；
  `CheckedClosureFacts` 读凭证），同一映射字节等价（`MIR_LOWERING_VERSION` 不变）；
  异 body 凭证/异 owner 文档＝`lower.closure-facts` 拒。
- 候选事务：`ClosuresFnCandidate`（env_id/base_generation/多单元 states/batch 文档/新源
  与坐标 delta）；`commit_candidate` 唯一安装（`source.stale`/`source.foreign` 拒）；
  M axis 目录坐标平移不触碰任何未改 resolved body。

## 语法来源接缝（PLAN-928）

- 本前端的**语法生产**（整源冷入口与片段解析）已接统一共享 CST 链
  （`parse_program_with` 对 Closures/Collections profile 路由
  `core::source::cst`；规则/裁剪见 auto-syntax 的
  [grammar-contract](../auto-syntax/grammar-contract.md) 与
  [source-frontdoor](../auto-syntax/source-frontdoor.md)）。捕获/凭证/
  资源语义**不变**：`decide_capture`/`ClosuresVerifyContext`/verify 链仍是
  语义事实唯一裁决，语法链不重做资源分析、不从 target 猜类型。
- 语法单元目录（[composite-source-projection](../auto-syntax/composite-source-projection.md)）
  的同文 lambda 语法复用**不免除**当前 capture 重验：目录对 owner 变化
  （locals 顺序/类型/写性/mode/layout）显式携带 `semantic_recheck` 义务
  信号，由本前端/resolve 的既有 oracle 消费；值绑定变化但 lambda 布局
  可复用时，creator 创建点操作数仍须按当前事实更新，不得以"重编所有
  lambda"替代精确依赖、也不得以"语法同文"跳过重验。
- 逐轴不藏工作：语法目录的 produced/reused/discarded 计量与本规范的
  P/whole/M/R 轴分账并存，互不吞并。

## 计量、预算与边界

- P 轴（每逻辑函数真实 attempt，失败早退也计）：parse_project / resolve / typecheck /
  plan / bind / hir_verify / mir_lower / mir_verify / abc_encode / decode_admit /
  index_build（closures profile 被 `ResidentProgram::prepare` 门控为 0，接线于 link 缝、
  非缺席）/ artifact_write；whole 轴（whole_parse/plan/bind/verify/lower）在暖支持路径
  必须 0；M 轴（fragment_diff/source_hash/catalog/coord_remap）单列；R 轴 restart 的
  decode/deep-verify/relower 如实非零（R3 未闭）。`WarmGateReport` 拒绝 Default-0、隐藏
  兄弟 walk、whole 轴、越单元生产；同 wire 重提交（同 hash 新对象）由 823 结构门拒。
- 预算：驻留前端池（source+AST+resolved+目录+registry）≤16MiB（parent 判决同门）；
  4MiB 源前门（865 路径原无）；code current+history 8MiB/32、活值 8MiB、verify/decode
  16MiB 原 cap 不变；父 working-set 相对冷基线增量≤128MiB。
- A→B 交接：`AcceptedSourceState`（schema `ace.closures.accepted-source/1`；仅由已提交
  状态形成；source SHA256/逻辑 owner（module path）/entry/profile/caps/producer 版本
  （MIR_LOWERING_VERSION/MIR_CLOSURES_SCHEMA/av-closures-unit/v1）/accepted_generation/
  commit_point/context_revision/每单元 logical/body_digest/依赖与 env 身份/spanless wire
  fp/store artifact ref/mint+current span）。数据视图≠凭证：913 落盘后仍须深重验；不含
  guest 值/PASS/活 env/cell/宿主地址。
- 正式面：固定组 `case-ace-m1-closures-fn-912`（F01/F02 四态 + 原语料 K01/K06/S02 同字节
  重跑）；worker recipe `ac-av/closures-fn-run-v1`；父侧 `judge_closures_fn` 独立判值与
  zero 轴（body-edit＝fast 路由+gate pass+whole 0+submitted 严格小于全量；hit＝全零；
  restart＝全命中+mir_lower 如实==N）。V-13：64 顶层函数/70 单元、每被测 owner≥3 兄弟
  lambda、四状态各30样本 P90≤2000ms（323/2/85/25ms 实测）、512 修订轮转 churn、池/父峰值门。
- 边界（如实）：owner 路由的 parse_project 按 owner item 计 1（lambda 文本属 owner body，
  无任何 lambda 语义阶段）；owner 级 `scan_shared_capture_cells` 为语法扫描；嵌套 lambda
  仅经 owner 路由编辑；`__closure_<owner>_<k>` 显示名仍按 owner 内序号（拓扑变化时重排、
  逻辑 ID 保持）；runtime cell/env fault sweep 沿用 909 既有（同一 823 会话）；
  native/A2R 闭包、共享 owned/cyclic/跨挂起、持久恢复（913）不在域内。

详见[函数级增量源前端](function-incremental-source.md)、[源码闭包域](source-closures.md)、
[共享可变捕获](shared-mutable-captures.md)、[闭包捕获 MIR](../auto-mir/closure-capture-profile.md)、
[HIR lowering 边界](../auto-mir/hir-lowering-boundary.md)、
[常驻函数安装](../auto-av/resident-function-install.md)与
[ACE M1 一致性语料](../auto-test-case/ace-m1-conformance.md)。
