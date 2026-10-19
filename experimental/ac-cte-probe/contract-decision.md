# PLAN-817 contract-decision.md — T-01 冻结决策（CTE 查询/冻结缓存合同）

2026-10-20，work stage T-01。本文件冻结 `cte/scalar-query-v1` 探针的逻辑
Schema、请求/查询 IR、consumer 双向绑定映射、权限、预算口径与资源归属、
参考负载与独立 oracle、AutoDown 注册映射。T-02..T-08 在此基础上实现并验收；
任何变更须回到本文件并记 revision。

**证据基础（只读）**：master `08ba8ca2b`（796/783/788 已交付）+ 在途
`plan-798-dev` @ `072cb4e14d`（D-798 未满足：798 尚未 final review pass /
冻结门禁 / 合入 / Spec 收据）。本轮未编译任何在途接口、未创建实现代码、
未修改 canonical Spec 或他方写面。T-02 开工前提 = D-798 满足 + 公共注册
短窗口单写；届时须按实际合入 API 重取本文件 §2 的映射并记录漂移
（若有），不在 798 代码里加第二套推导/执行器。

---

## 1. 身份与 Schema 冻结表

| 项 | 冻结值 |
|---|---|
| 探针请求 IR Schema | `ac817/cte-recipe-ir/v1` |
| 冻结执行 envelope Schema | `ac817/cte-executed/v1` |
| store 层（817 自有 LayerSpec） | `ac817/a05-cte-exec/v1`（coverage=Codec，817 自己实现的有限范围） |
| producer id | `plan817/cte-probe` |
| AutoDown recipe | `cte/scalar-query-v1`（v2 case schema，atom-data 源） |
| worker 协议 / bin / env | `cteworker/1` / `cte_probe_worker`（crate `auto-cte-probe`）/ `AC_CTE_CASE_WORKER` |
| 引擎身份 | 798 `av.core-i32/1` profile；MIR/ABC/AV 语义版本各 `1`；codec=798 冻结 wire |
| 执行能力位 | 默认空集；Meta 参数查询唯一能力 `cte.meta.fields/v1`（显式授予） |
| workspace | `experimental/ac-cte-probe/`：`contract`（`auto-cte-contract`）、`core`（`auto-cte-core`）、`probe`（`auto-cte-probe`）+ 跟踪 Cargo.lock |

`contract/core` 默认依赖闭包不得含 UI/Man/旧VM/Cranelift；`probe` 可消费
已合入的 ac-core 公共 API、798 probe 公开 adapter（HIR→MIR lowering、
ABC 编解码/验证、AV 执行）与 783 `ManagedStore`。

783 侧零修改：不改 registry（A05 在 783 登记保持 `LayerCoverage::Future`，
SD-04 的"817 producer 有限 A05 实现"= 本计划自有 LayerSpec `ac817/a05-cte-exec/v1`，
如 787/798 先例）、不加 ChunkKind（载荷走既有通用载体
`ChunkKind::LayoutData` + 内层 envelope 逐字段校验）、不改旧 producer
Future 覆盖、不复制 796 store 数据库。

## 2. D-798 接口观察（072cb4e14，T-02 复验义务）

817 消费面（只读观察到的实际 API，非合入承诺）：

- `auto-av-core`（`experimental/ac-av-probe/core`）：
  - `verify_mir` / `CheckedMir::module() -> &MirModule`；
  - `verify_abc` / `verify_abc_bytes` → `LoadedModule`（checked+av_semantics+
    executable，`LoadedModule::module()`）；
  - `execute_entry(&LoadedModule, &ExecLimits, &CancelFlag, &[Value]) ->
    (ExecOutcome, ExecTrace)`；
  - `ExecLimits { max_steps, max_frames, max_slot_bytes, wall, max_trace_bytes }`，
    Default=D-798 冻结 cap；
  - 常量：`MIR_SCHEMA="auto.mir.av-probe/1"`、`ABC_SCHEMA="auto.abc.av-probe/1"`、
    `AV_PROFILE="av.core-i32/1"`、`PRODUCER_ID="plan798/av-probe"`、语义版本
    1/1/1、`MAX_INPUT_BYTES=4MiB`、`MAX_CODEC_PAGE_BYTES=256KiB`、
    `MAX_TABLE_ENTRIES=65536`、`MAX_FRONTEND_NESTING=128`、
    `MAX_VERIFY_WORK_STEPS=2M`、`MAX_VERIFY_WALL=5s`、`MAX_AV_INSTR_STEPS=1M`、
    `MAX_AV_FRAMES=1024`、`MAX_AV_SLOT_BYTES=8MiB`、`MAX_AV_WALL=2s`、
    `CANCEL_CHECK_INTERVAL=1024`、`MAX_TRACE_BYTES=64KiB`；
  - `ExecOutcome`：Success(value)/Trap(kind+span)/Resource/Cancelled/Infra；
    trap 集 int-overflow/div-by-zero/div-overflow/rem-overflow；未执行分支
    trap 潜伏（§3 合同）。
- `auto-av-probe`（probe）：`hir_adapter::lower_checked`（Checked HIR→MIR）、
  `driver::Driver`（ephemeral/with_store/set_owner/request）、796 语义绑定先例
  `semrefs.rs`（`unit_logical_id`/`unit_decl_ref`/`admit_unit_ref`/
  `content_version_of`）、worker 恢复先例 `worker_restore.rs`。
- 796（master 已交付）：`ac-sema-core` `SemaInput`/`InferSession::infer_function`/
  `reflect::{query_field,query_signature,record_shape,list_fields,list_candidates,
CtRequest,CtResult,PortableValue,ModelCt,admit_as_model}`、
  `query.rs::{fn_sig_canonical,fn_body_canonical,called_names,current_fingerprints,
SemaQueryKey,DependencyFingerprints,manifest_from}`；投影 adapter
  `parse_into_input_typed`/`project`（ac-core AST→SemaInput）。
  **不改 `ModelCt`/`admit_as_model`**（vm_execute 标记独占属于真实执行路径；
  817 冻结 envelope 用自己的 producer 字段，不给 796 模型添信任）。
- 783（master 已交付）：`ManagedStore::open/open_existing`、`PutRequest/
  put_artifact`（staging 8MiB 预算、CAS、同键异容=Conflict）、`get_artifact/
  read_chunk_gated`、`acquire_lease`（OS 文件锁真值）、roots、
  `RefKind::{RequiredData,Recomputable,InvalidationFingerprint,Provenance}`、
  `ExternalRef::validate_external_ref`、GC 水位 `GC_HIGH=64MiB/GC_LOW=48MiB`、
  `run_gc`、`recover`、通用 JSON 载体 `encode_json_chunk/decode_json_chunk`。
- ac-core（master 已交付）：`compile_parts_with(src,module_path,caps,profile)`、
  `verify/verify_scalar`（→`CheckedModule`，唯一获得途径）、
  `CheckedModule::required_capabilities()`、HIR `ExprKind::Constant{value:i32}`
  （类型由 `Expr.ty` 携带）、`DefKind::Function{name,ty,body}`、
  `find_entry`（0参→i32）、native 参考路径（HIR→Cranelift→COFF→PE）。

**观察→实现映射规则**：T-02 只从实际合入 master 的上述 API 调用 AV；
若 798 交付与本观察不同（改名/签名移动/常量调整），在 T-02 收据记录
新旧映射并复验本文件引用的每个 cap 值，不复制分支接口、不长期 mock。

## 3. CTE 请求 IR 与查询节点（`ac817/cte-recipe-ir/v1`）

请求是版本化 recipe IR（测试/查询接口，**不新增 Auto 语言语法**）：

```text
CteRequestV1 {
  schema: "ac817/cte-recipe-ir/v1"
  recipe: "cte/scalar-query-v1"
  callee: { package, module, fn_name, sig_expect: FnSig }   // 796 当前声明内
  args:   [Literal(i32|bool)]                                // 常量实参
  queries: [QueryNode]                                       // ≤128/请求
  consumer: ConsumerBindingTable                             // §5
  caps:    ["cte.meta.fields/v1"?]                           // 默认空
  profile: { engine: "av.core-i32/1", budgets: BudgetSet }   // §7
  snapshot: 请求快照代数（source fingerprints）
}
QueryNode ::= Literal(i32|bool)
            | QueryRef(node_id)                    // 仅引用已完成节点
            | Call { decl: callee_identity, args: [QueryNode] }
            | FieldCount { record }                // 需 cte.meta.fields/v1
            | HasField { record, field }           // 需 cte.meta.fields/v1
```

- `Call` 实参在查询求值期解析为 i32/bool 字面量后作为真实 AV 调用实参；
  `FieldCount/HasField` 经 796 当前签名/字段投影读取（`record_shape/
  list_fields`），分别返回 i32/bool——它们计入 `meta_query` 计数，
  **不冒称 AV 执行**（`ct_execute` 不动）。
- 不存在的 record → 语义诊断 `cte.meta.record-not-found`（解析/语义错误，
  非运行值）；record 存在但缺字段 → `HasField=false` 并记录集合负依赖
  （候选集指纹取 796 `fn_candidates/record_candidates` 排序指纹）。
- `QueryRef` 只能引用**已完成**节点（DAG 前驱），引用未完成/失败节点是
  请求级 admission 错误。
- 值域：参数/返回仅 i32/bool；pure 直接函数调用；有限控制流与 D-798
  已支持精确算术/trap；普通函数递归由 AV fuel 管，不等同查询依赖环。
- CTE 默认无 import；无文件/环境 broker；无任意宿主函数扩面。

## 4. 权限与能力审查（执行前，av/ct_execute=0 处完成）

1. callee 可执行闭包 = callee + 其调用闭包（796 `called_names` 传递闭包）
   内全部函数，各自经 ac-core `compile_parts_with` + `verify*` 检查
   （**callee 检查先于执行；不要求 consumer 所在全程序先 final Checked**）。
2. 能力审查：闭包的 `CheckedModule::required_capabilities()` 必须为空集；
   任何能力（含 `hir.test.trace.v1`、runtime text）→ `cte.capability.denied`，
   在任何 AV 调用前拒绝（`ct_execute=0`）。CTE 连测试 trace 标记都不是
   普通 CT 权限。
3. `cte.meta.fields/v1` 只授予 FieldCount/HasField 查询节点（参数来源），
   不进入 AV 执行闭包；未授予而使用 → admission 错误。
4. 未知指令/profile/能力，或请求声称的签名与 796 当前声明不符 → 拒绝；
   只有模型 producer（796 ModelCt）存在的 callee 不得执行。
5. 调用者传入的 deps 字符串只作输入提示；生产者依赖以 envelope manifest
   为准（§6）。

## 5. consumer 双向绑定映射（CtRef → 796 推导 + HIR 回填）

**绑定锚点方案（冻结）**：consumer 是普通 Auto 源码（Core/Scalar profile），
每个待求值位置写作一个普通局部绑定 `let <binding> = <placeholder字面量>;`。
recipe 的 `ConsumerBindingTable` 每项：

```text
CteBinding { ct_ref, owner_fn, binding_name, placeholder: i32|bool,
             expected: I32|Bool, source_query: node_id }
```

- **796 语义腿**：`project`(ac-core AST→SemaInput) 后，在 `owner_fn` 内
  将 `Let{binding_name}` 的 `value` 替换为 `InputExpr::int(v)` /
  `InputExpr::boolean(v)`（CT 结果投影成 796 InputExpr 字面量后才做局部
  推导），`InferSession::infer_function(owner_fn)` 推导类型必须等于
  `expected`。consumer 推导面严格限 796 已交付 core i32/bool 操作
  （+ - * / % < <= > >= == != && ||、neg/not、局部 let/var、直接调用、
  条件约束）——callee 侧 798 支持更多 scalar 操作也**不**因此扩 796 求解器。
- **HIR 腿**：consumer 源码经 `compile_parts_with` 得 `Bundle`；按
  (def=owner_fn, Stmt::Let 的局部名) 定位初始化表达式 ExprId 锚点，替换为
  回填子树：i32 → `ExprKind::Constant{value}`（`Expr.ty`=I32）；bool →
  `Binary{Lt, Constant{0}, Constant{1|0}}` 子树（镜像 ac-core 源适配器对
  true=`0<1`/false=`0<0` 的确定性编码，保证 verify 形状兼容）。回填后
  **重新走 `verify*`**（不是只改节点），并记录双向绑定/source origin
  （Let 的 span）。
- **绑定校验（全拒绝项）**：`ct_ref` 重复；`owner_fn` 不在 SemaInput；
  `binding_name` 在 owner_fn 内解析到 ≠1 个 Let（含遮蔽）→ 
  `cte.consumer.binding-ambiguous`；源码内 placeholder 字面量与声明不符
  （源/绑定漂移）；推导类型 ≠ expected → `cte.consumer.type-mismatch`；
  绑定跨越其声明的 owner（一处求值、他处冒用第二绑定）→ 拒绝。
  好帧错内层/页 key/嵌入 ref 拒绝见 §6。
- **可观察性证明义务（V-01）**：不能只算值输出 JSON。必须证明 (a) 796
  推导事实改变（如未回灌前该绑定类型/值不可得或分支走另一臂），(b) 回填后
  HIR 含预期常量子树且 verify 通过，(c) 既有 native 自检出口与独立 expected
  一致（旧 HIR-native 路径，不等待 818）。不声称完整 Auto 源码 `#`/
  comptime 语法已接入。

**首例（T-02 冻结）**：
- callee `fn cte_sum(a: i32, b: i32) -> i32 { return a + b; }`，
  query `Call(cte_sum, [20, 22])` → AV 真实返回 I32(42)，`ct_execute=1`；
- consumer `fn main() -> i32 { let ct = 0; return ct; }`，绑定
  `{ct_ref:"q1", owner:"main", binding:"ct", placeholder:0, expect:I32}`；
- 回灌后：796 推导 i32；HIR `Constant{42}`；native 运行出口 = 独立手算
  期望 42（闭式镜像，不经 AV/新 freeze 生成）。
- bool 分支正例：callee `fn cte_flag(x: i32) -> bool { return x < 10; }`，
  `Call(cte_flag, [9])` → true；consumer
  `fn main() -> i32 { let gate = false; if gate { return 1; } return 2; }`，
  绑定 placeholder=false/expect=Bool——回灌前后 native 出口 2→1，
  证明 CT 结果真实改变 consumer 分支。
- Meta 参数例（设计例，T-03 实测）：`FieldCount(Pair)`=2 作为
  `Call(scale, [FieldCount(Pair)])` 的 AV 实参（`meta_query=1`）。

## 6. 冻结 envelope、依赖 manifest 与 cache 语义

### envelope（`ac817/cte-executed/v1`，经 783 通用载体落库）

```text
CteExecutedV1 {
  schema, recipe, request_identity（完整字段，非仅 hash）,
  value: i32|bool, declared_type,
  executed_by: producer + engine 身份 + ExecTrace 摘要（steps/wall）,
  deps_manifest, origin_sidecar（源快照 id/span）, created
}
```

- producer 创建路径**绑定真实 AV 成功结果**（Success 才 mint；Trap/
  Resource/Cancelled/Infra 不落成功 envelope）。值只支持 i32/bool，携带
  声明/预期类型。
- 可引用 796 `PortableValue` 表达值，但保留其原有模型准入规则：
  `marking=vm_execute`、输入自带 trusted、仅有内容 hash，都不能证明适用
  于当前请求——恢复是**数据校验+新鲜性检查**，不反序列化进程信任。
- 错 producer/schema/request 身份/语义版本 = 受控拒绝（非 miss、非就地
  重建覆盖）；同名异包、好 frame 错地址/owner/key、旧嵌入类型同样拒绝。

### 依赖 manifest（至少记录）

实际 helper body/签名/调用闭包（796 `fn_body_canonical/fn_sig_canonical/
called_names` 传递闭包）、CT 实参、读取 Meta 集合与候选负依赖指纹、
算法/Schema/AV 版本、配置/权限/target 相关语义。
**声明粒度（冻结）**：实现按"可执行闭包全部 body"保守粒度（当前快照
指纹比对）；manifest 头部显式声明该粒度，不声称动态读集最小化。

### cache 行为

- **key**（至少）：callee 身份（包/模块/函数+签名指纹）、闭包 body 指纹、
  常量实参、查询算法/profile、引擎与 MIR/ABC/AV 语义版本、codec 版本、
  权限/能力配置、target 语义、Meta 读集+负依赖指纹。不同包同名及方法
  owner 必须区分（796 DeclRef 域）。
- **hit**：`ct_execute=0`、模型计数不增加；**consumer 仍每次重新准入/
  比较**（新鲜性验证 + 796 推导 + HIR verify 逐次执行）；通过禁调 AV 的
  反控证明。hit 不缓存旧运行 PASS（当轮输入 hash 绑定）。
- **miss 正控**：`ct_execute>0` 且是 AV 真实调用（禁调 AV 反控下 miss
  路径必须红/受控失败，不得命中旧运行）。
- **新鲜性**：上次 manifest 必须先验证完整新鲜——caller 源不变但 helper
  body/签名、字段集（含负依赖候选集）、schema/profile/caps/target 语义
  变化 → 相关值失效且结果 = 无 cache 基线；未读面按声明粒度（闭包级）
  复用。
- **失败发布**：瞬态 resource/cancel 不持久成程序错；失败发布保留旧提交、
  释放 lease/暂存。
- **持久化/GC 归属**：独立 namespace + `ac817/a05-cte-exec/v1` 层；
  类型/常量等消费必须存活的数据走 `RequiredData` 边；helper body 及来源/
  失效记录不永久 pin 全部 AST（residency=cold）；字面常量 native 物化后
  无需保留 AV 活堆。复用 783 水位/lease/root/GC，不造第二数据库。

## 7. 预算、资源归属与硬门

### 请求级预算（`BudgetSet`，分配前预留——先于 clone/decode/编码）

| 预算 | 冻结值 | 分类码 |
|---|---|---|
| recipe+source/HIR 总输入 | ≤4 MiB | `cte.resource.input-size` |
| 查询节点/请求 | ≤128 | `cte.resource.query-nodes` |
| 查询依赖深度 | ≤128（两侧边界必测） | `cte.resource.query-depth` |
| AV 执行（fuel/帧/槽/wall） | **≤ D-798 cap**：1M 步 / 1024 帧 / 活槽 8 MiB / wall 2s | `cte.resource.av-*` |
| 验证（步/wall/表族/codec页/前门嵌套） | ≤ D-798：2M / 5s / 65536 每表族 / 256 KiB / 128 | `cte.resource.verify-*` |
| 单请求墙钟 | ≤10 s（含调度/等待） | `cte.resource.request-wall` |
| 结果页 / 元数据 IPC | ≤512 KiB / ≤4 MiB | `cte.resource.result-page` / `ipc` |
| stdout/stderr | 各 ≤1 MiB（kill-on-overflow） | `cte.resource.stream` |

AV 与验证 cap 直接取 `auto-av-core` 冻结常量（不大于、权限更严）；
817 不复定义第二套数值，运行时以引擎实际 ExecLimits 传递。

### 受管内存（分区台账，单报不重复计数）

- 总额硬门 ≤32 MiB = **hot 8**（热值/store 热页）+ **请求/编码/索引
  暂存 16**（含 798 r3 的 verify 位图/解码堆估算核算，折入同一台账）+
  **AV 槽 8**。跨库同时持有的 796 对象计入相应分区，不重复领预算。
- worker 自身 private bytes 峰值硬门 ≤384 MiB（采样含瞬态峰值；已知
  锚点：798 实测 267 KiB 前端输入瞬态峰 ~211 MiB——817 worker 同规模
  前端在预算内但贴线，T-05 复测记录）。
- 整个测试进程树峰值 ≤512 MiB，采样单报；native 参考进程另报，不从
  worker 峰值中任意扣掉前端。
- 超标不填 `target_met=true`；瞬态 spike 反控必须真红。

### 时间硬门（AC-06，六场景 × ≥30 样本）

warm 会话局部 CT helper body 编辑及 Meta 集合编辑的
"请求准备/序列化→IPC/排队→新鲜性/CT执行或命中→冻结/consumer推导/
HIRverify→父侧准入/比较"单墙钟 **P95<1000ms**（含真实必要等待，不扣
序列化/哈希/比较；数值/trace/计数逐样本绑定当轮新 hash，不删失败）。
完整 native 参考另报；此门是 CTE 查询端到端，不冒称完整应用编译/链接
或热重载亚秒。无 cache 结果/消费观察 = cache 路径语义基线。

### 调度与失败分态

- 显式查询栈/worklist；节点 InProgress/Completed/Failed + 当前 snapshot
  代数。A→B→A 查询环 → `cte.schedule.query-cycle`，诊断携带调用链定位
  （镜像 796 `CyclicRequest` 链式诊断语义，作用于查询 DAG 层）；不能靠
  "全程序先 Checked" 绕过。合法共享 DAG/重复子查询不误拒（single-flight
  纯查询去重；失败/取消不缓存成永久语言错）。
- 旧 snapshot 迟到完成不发布当前结果（代数守卫）。
- compiletime 已执行溢出/trap → 带 CT 调用链的 `cte.trap.*` 语义诊断；
  未执行分支 trap 保持潜伏；fuel/内存/取消/宿主崩溃 → resource/cancel/
  infra 分态，不当预期语言负例。
- 诊断投影只用已支持 stage/type + 精确 code 和来源（ac-core Stage 集
  Text/Bind/Verify/Capability/Backend/Link/Run + runner 既有 admission/
  frontend 映射），不因 CTE 另造未知 stage 被 runner 忽略。

## 8. 参考负载与独立 oracle（T-01 冻结）

- **参考机/工具**：本机（PLAN-817 会话用 Windows x64 开发机；与 798
  2026-10-16 实测锚点同机序）；rustc/cargo 1.98.0（88d9e12ae 2026-08-18 /
  797e8a9bc 2026-08-05）；release 构建；性能/峰值/大规模 GC 测量单实例
  串行并记录其他计划负载状态。
- **256 KiB 负载**：`load256cte::generate()`——100 个有意义函数规模
  （混合 callee/consumer，含 meta 查询节点与多查询 DAG），≈256 KiB 真实
  源码，无空白填充；期望值由生成器**闭式镜像**推导（不经 AV/新 freeze/
  任何后端）。
- **局部编辑配方**：helper body 编辑 = 指定 callee 首个乘子 +1（期望随
  之变）；Meta 编辑 = 指定 record 增/删字段（FieldCount/HasField 期望与
  负依赖失效面随之变）；签名冲突编辑 = 指定 leaf 增参（正确前端诊断，
  非崩溃）。
- **六场景**（各 ≥30 原始样本，路径与名字一致）：fresh（每次新 store）/
  same-process-hot / restart（每次真新进程）/ new-version body+helper
  编辑 / 字段/负依赖编辑 / 真实并发 GC。
- **规模/GC**：独立脚本流式生成 ≥32 MiB 真实冻结页/manifest + 512 次
  **不同输入**修订 + 真实 query/重键/结果断言；GC 场景冷集以真实不同
  产物推动同一 store 超过 783 高水位（64 MiB，冷集目标 ≥80 MiB 保证
  超越余量），至少一次物理删除、活集低于低水位（48 MiB）时实测收敛；
  活 lease 强引用保活、释放后按水位收敛、用户哨兵不变；不以未触发 GC
  的低占用算通过。128 单请求查询上限不阻止跨请求造数。
- 依赖链 128 边界两侧、坏嵌入引用、深层输入/解码/释放全路径有控制。

## 9. AutoDown 注册映射（公共写面，短窗口单写）

T-02 最小注册 + T-06 完整 suite，映射五点（镜像 798 `ac-av/core-run-v1`
先例；在 797/798 合入、814–816 边界交接后，同步新 master 单写追加）：

| 点 | 位置（master 观测） | 817 追加 |
|---|---|---|
| recipe 表 | `crates/auto-test-case/src/contract.rs` `REGISTERED_PROBE_RECIPES` | 追加 `"cte/scalar-query-v1"` |
| 绑定选项 | `Options.probe_av` 先例 | `Options.probe_cte: Option<ProbeCteSpec>`（`{callee, bindings[], caps[], session?}`，session id 同 798 约束） |
| 步词汇 | recipe 专属 gate | `query|edit-source|restart`，序列以 `query` 结尾；atom-data 源 admission（现有 recipe 均 .at-only，817 recipe 专属接受 `.atomdata`） |
| 分派 | `mod.rs::dispatch_check` probe 臂 | 按 recipe 路由到新独立 `cte.rs` adapter（复用 `sema.rs::containment`，不复制） |
| stale 轴 | `stale.rs` per-recipe family | `["cte-worker-exe-sha256","cte-worker-version","cte-protocol","recipe"]` |

- worker：`AC_CTE_CASE_WORKER` env 否则限定 release 构建
  `experimental/ac-cte-probe -p auto-cte-probe --bin cte_probe_worker`
  （BUILD_WALL 同 798）；store root：`cte.session` 设定 →
  `%TEMP%/cte817-session-{id}`（跨调用共享 warm store），否则
  `artifact_dir/store`。
- 期望面：既有 cache 词汇 `hit/miss/stale/published/reprojected/diags` +
  CTE 计数（`ct_execute`、`meta_query`、`query_nodes`）+ `exit`（native
  参考 observable，probe exit 观察者已由 798 接通）+ `stages_include`。
- 父侧判定：`ok` 零诊断；resource/cancelled/infra 与生命周期失败 =
  `Status::Error` 永非语言裁决；坏 actual 好 expected 不能绿。
- 语料：`crates/auto-lang/test/cases/ac-cte/`（suite+case+.atomdata/附件
  sidecar）。不得另写 .ad parser、case 携 shell/任意工具路径、自造 PASS。
- 若正式接口无法表达已定源/步骤，带具体反例显式修订合同，不在 T-02 后
  保留长期私有 runner。

## 10. V 映射（设计元素 ↔ 验证义务）

| V | 本文件依据节 |
|---|---|
| V-01 真实闭环 | §5 首例/bool 分支例 + §4 能力审查 |
| V-02 顺序/阶段 | §7 调度分态 + §4（callee 先查、trap 语义） |
| V-03 环/取消 | §7 调度（query-cycle/DAG/递归区分/取消） |
| V-04 信任 | §4 + §6（marking/trusted/hash 不授信、ct_execute 计数） |
| V-05 真缓存 | §6 cache 行为（hit 禁 AV 反控/miss 正控/跨进程） |
| V-06 依赖 | §6 manifest/新鲜性（helper/字段/负依赖/版本/权限/target） |
| V-07 坏块/生命周期 | §6（错身份/错内层/嵌入 ref 拒绝）+ §8（lease/GC/哨兵） |
| V-08 规模内存 | §7 分区/硬门 + §8 规模造数/GC |
| V-09 周转 | §7 时间硬门 + §8 六场景/编辑配方/阈值反控 |
| V-10 runner | §9 注册映射/stale/父判/旧通道兼容 |
| V-11 并行/依赖 | §2 复验义务 + 本文件边界（零他方写面修改） |

## 11. 边界（不因实现进度改口）

- 非目标重申：Auto 完整元编程语法、类型/声明生成、type-as-value 执行、
  用户泛型实例化、runtime 反射调用、text/record 运行值、任意 IO/环境/
  时间/随机/网络/FFI、JIT、发布器、全面前端/缓存生产化、自举、MIR→native
  新后端（818 域）。只读 Meta 描述可作参数来源，不等于 AV 已能操作
  record/类型对象。
- 本文件 T-01 范围=设计冻结；T-02 尚未正式运行 AutoDown 不得勾选；接口
  不足、性能/硬内存不达或需改在途 owner 文件时，带证据回规划：不抬预算、
  不降用例、不复制 AV。
- 与 818 互不依赖；native 观察用旧 HIR-native。797 原生 ABI 不在本任务
  调用链。
