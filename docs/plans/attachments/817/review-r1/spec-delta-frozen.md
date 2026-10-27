# PLAN-817 规范增量终稿（T-08 准备，合入时落 canonical Spec）

依据 r3 计划 §5 规范增量表与 contract-decision r1.2。合入由 merge 阶段执行
（`.autoos/specs.json` upsert + 模块文档回写）；本文件是经 work 证据核定的终稿文本。

## SD-01 add docs/specs/auto-cte/project.md + scalar-query-profile.md
- before：无 auto-cte Spec；CT 仅 796 模型接缝（CtRequest/CtResult/model_compute）。
- after：有限真实 AV 查询（i32/bool、纯 callee、查询节点
  Literal/QueryRef/Call/FieldCount/HasField、`cte.meta.fields/v1` 显式授予）、
  consumer 双向绑定（let 锚点→SemaInput 字面量→infer；HIR 原位回填+重verify）、
  profile core|record 源级分界（record 面 AV 调用准入拒绝）。
- 不声称：全语言 comptime/类型生成/泛型/runtime 反射。
- 证据：AC-01/02/03；cte_first_/cte_schedule_。

## SD-02 add docs/specs/auto-cte/frozen-cache.md
- before：A05 在 783 登记为 Future；无真实执行 producer。
- after：`ac817/cte-executed/v1` envelope（producer=plan817/cte-probe 绑定真实 AV
  成功）、自有层 `ac817/a05-cte-exec/v1`（LayoutData 载体）、全字段恢复准入+新鲜性
  （闭包 body/签名、meta 探针/事实负依赖、caps、engine 三元版本）、hit 零 AV 且
  consumer 逐次重准入、坏块受控拒绝（非 miss）、lease/root/GC 按水位。
- 证据：AC-04/05/06；cte_freeze_/cte_cache_/cte_resources_。

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
- 证据：AC-11；cte_snapshot_ 4/4。

## 覆盖
见 [acceptance-matrix.md](acceptance-matrix.md)（V-01..11 → scope/证据/提交）。
