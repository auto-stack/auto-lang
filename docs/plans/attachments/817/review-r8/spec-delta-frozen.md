# PLAN-817 规范增量终稿（T-08 准备，合入时落 canonical Spec）

依据 r3 计划 §5 规范增量表与 contract-decision r1.2。合入由 merge 阶段执行
（`.autoos/specs.json` upsert + 模块文档回写）；本文件是经 work 证据核定的终稿文本。

## SD-01 add docs/specs/auto-cte/project.md + scalar-query-profile.md
- before：无 auto-cte Spec；CT 仅 796 模型接缝（CtRequest/CtResult/model_compute）。
- after：有限真实 AV 查询（i32/bool、纯 callee、查询节点
  Literal/QueryRef/Call/FieldCount/HasField、`cte.meta.fields/v1` 显式授予）、
  consumer 双向绑定（let 锚点→SemaInput 字面量→infer；HIR 原位回填+重verify）。
  **Meta→AV 实参（F-817-R1-05 修复）**：deck `meta << ... >>` 块携带 record
  声明（record-profile 源文本），投影合并进 core 输入的 record 域——
  FieldCount/HasField 结果作为**真实 AV 调用实参**（合同 §5.1 原文路径），
  字段集/缺席字段变化经 manifest meta 事实精确失效；纯 record-profile deck
  （无 meta 块）保持 Meta-only 受限模式。r1 工作收据中"源级组合不可达"的
  表述作废。meta 输入负控：未授 cap → cte.capability.meta（AV 前）；缺
  record → cte.meta.record-not-found；meta 块解析失败 → 语言类诊断；
  含 meta 的 deck 同受 4MiB 深输入界约束。
- 不声称：全语言 comptime/类型生成/泛型/runtime 反射。
- 证据：AC-01/02/03；cte_first_/cte_schedule_。

## SD-02 add docs/specs/auto-cte/frozen-cache.md
- before：A05 在 783 登记为 Future；无真实执行 producer。
- after：`ac817/cte-executed/v1` envelope（producer=plan817/cte-probe 绑定真实 AV
  成功）、自有层 `ac817/a05-cte-exec/v1`（LayoutData 载体）、全字段恢复准入+新鲜性
  （闭包 body/签名、meta 探针/事实负依赖、caps、engine 三元版本）、hit 零 AV 且
  consumer 逐次重准入、坏块受控拒绝（非 miss）、lease/root/GC 按水位。
- **预留顺序（F-817-R6-02/R7-01）**：编解码/恢复路径分配前预留——envelope 编码
  先经计数 writer 得精确字节数、预留 Staging 后再序列化（payload 按值入 store，
  无二次未预留副本）；恢复四段有序（F-817-R7-01）：**零分配大小探测**（经公共 `ManagedStore::db()` 只读 SQL
  `chunks.size`，framed=48B 帧头+载荷）在读取/重组上限处先行拒绝超界载体（加载器零调用）；
  载入预留=loader 真实瞬态足迹（framed Vec+unframed 拷贝）；加载长度须与探测一致（内容
  寻址漂移=异质写窗口受控拒）；decode 以保守 2× 载荷留 Hot（serde 结构/容量开销+与载荷
  同时存活，撤销"JSON 文本≥结构"假设）；worker wire 读入有界（6MiB）且 base64 预检先于解码；预算事件日志
  （Reserve/Alloc/Release 顺序）使"拒绝先于大分配"可观测。
- **请求累计墙钟（F-817-R6-01）**：单 CTE 请求（调度/缓存/编译执行/consumer/
  必要等待）累计 ≤10s；看门狗经取消旗标中断在途 AV 调用，超限为受控资源态
  `cte.resource.deadline`（非语言裁决），请求不成功发布、超限后不发布；预算内
  完成的子调用结果为其自身合法 envelope。逐调用 cap（≤D-798 2s）不变。
- 证据：AC-04/05/06；cte_freeze_/cte_cache_/cte_resources_（预留顺序+deadline
  正反控）。

## SD-03 modify docs/specs/auto-semantics/semantic-contract.md
- before：796 模型合同独立（admit_as_model 拒伪 vm）。
- after：追加"真实执行域"指向：模型合同不变；真实执行走独立 envelope/producer，
  796 `ModelCt`/`admit_as_model` 原文零修改（closeout 审计钉死）。
- 证据：AC-02/04。

## SD-04 modify docs/specs/auto-ac/artifact-contract.md
- before：A05 coverage=Future，producer="expansion pipeline"。
- after：追加 817 有限 A05 实现（producer plan817/cte-probe、Codec、独立层 id）；
  783 原登记不动。
- 证据：AC-04。

## SD-05 modify docs/specs/auto-test-case/architecture.md
- before：probe recipe 表 sema/query-v1、abi/interop-v1、ac-av/core-run-v1。
- after：追加 cte/scalar-query-v1（v2 probe/run、atom-data、query|edit-source|
  restart 以 query 结尾、cte 绑定 entry/caps/session、cte-worker 身份轴族、
  outcome admission：隔离态=ERROR、语言 diag 对照 error completion）。
  旧 recipe 行为零修改。
- 证据：AC-07；官测+cte_adapter_。

## SD-06 modify docs/specs/auto-av/resident-function-install.md（T-11）
- before：828 交付的函数级增量 session/ProgramSnapshot 仅供 828 自己的 recipe。
- after：817 消费面接入——`exec snapshot` 车道（deck 指令）：一次 warm-up 装载全部单元，
  逐调用实参经 wrapper 增量编辑（对齐锚点保证收敛）、ProgramSnapshot 单元视图执行；
  车道为缓存身份轴（closure/snapshot 不共享条目）；增量纯度（未改函数零重编译/代际不变、
  warm 为唯一整模块事件）由 lane 退化计数器与 828 计数器双重观测。
- 不退回整模块热重编/装载；新能力为追加车道，默认 closure 保持。
- R2 补充：冷启动首调用即 bool callee 属正常支持面（warm 按 callee 返回类生成 wrapper 形状）；resident 会话跨请求驻留（单 hunk 增量同步、拒绝则显式重整并计数）；退化反控为正式硬门（resident-edit 场景 rewarms≠0 即红）。
- R3 补充：编辑硬门为暖会话真 IPC 窗口（单驻留 worker 进程，请求构建→序列化→IPC→完整父判）；内存硬门（worker 384MiB/树 512MiB）为正式 gate 组成部分，超标必红；consumer HIR 腿按缩减源编译（R4 修正：OWNER ∪ ENTRY ∪ entry 传递调用闭包，owner≠entry 不丢入口链），verify 失败回退全源——非缩小验证范围。
- R5 补充：树门为真实父子同时存活采样归集（parent+已登记 worker 子进程按 tick 求和取峰）；进程场景 worker 门消费子进程自报峰值，0/缺失=采样不可用即红；warm/spawn 共享零 panic 逐 step 准入（绑定键/类型/前序值/HIR/来源/步数），真实 wire 变异 11 态+合法正控入正式回归；Meta 硬门保持规定负载（256KiB 基线+meta→AV 多查询 DAG）。
- 证据：AC-11；cte_snapshot_ 13/13（含 115 函数基线 30 修订全阶段/IO 观测与强制重建反控）。

## 覆盖
见 [acceptance-matrix.md](acceptance-matrix.md)（V-01..11 → scope/证据/提交）。
