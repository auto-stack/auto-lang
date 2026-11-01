# auto-a2x 受检资源/模式事实出口

> 状态：active（AX-B 形态；PLAN-902 T-03 S 窗）。承接
> [architecture](architecture.md) ADR-1（不复制前端/分析器）。

## 单源权威

- 资源事实的**唯一生产者**是 ac-core 验证器：文本/文本记录域的
  `TextResourcePlan`（`BodyResourcePlan`：owned 集、逐出口 LIFO
  `exit_frees`/统一 `buffer_frees`（含资源 record 字段路径）、
  `owned_returns`、`replace_frees`）与 pairwise 参数排他约束
  （r4 R4-05 fixpoint，`param_exclusions_fixpoint`）。
- **A2X 层禁止重做 ownership/infer**：`SemanticView`/provider 只原样消费
  验证器计划（`resource_plan()/body_plan()`），从不从 MIR/AV 结果或目标
  猜测恢复语义。

## 单体凭证（text/text-record）

- `verify_text_function(bundle, def_index, excl_seed)`：单体文本凭证=同一
  `check_body` 结构走查 + 本体 770/782 资源走查；**排他种子来自保留环境**
  （整模块 fixpoint）——缺 body 脚手架不猜模式（881 retained 环境规则）。
- 凭证携带自身 `BodyResourcePlan` 与本签名发现的 `excl_adds`
  （`CheckedFunction::resource_plan()/excl_adds()`）；能力前门同整模块
  （无 `ac.runtime.text.v1` 且需要时= `capability.missing`）。
- Record 域单体凭证沿 `verify_function_ref`（结构校验、无跨体 fixpoint），
  方法体按 receiver=params[0] 同源校验。

## 凭证绑定（V-01）

- `SemanticView::from_parts` 以**声明上下文修订指纹**绑定凭证
  （`a2x.credential` 命名拒绝过期/外来凭证）：def id 位置性、单独可被同形
  异 owner 包冒配；上下文指纹按名义混入 record 类型（名+有序字段形状），
  同形异 owner 即失配（实测钉住）。标量模块无 record、指纹字节不变。

## 参数模式事实

- `ParamFacts.mode`（view/mut/move）与 `FnKind`（自由函数/owner 限定方法）
  随视图导出；**ExternalFn 携带验证 param modes**（从被调方自身 param
  locals 取，非缺 body 猜测）——被调方模式变化是依赖变化（入指纹）。

## Collections 受检事实（PLAN-904 AX-C）

- collections/closures 族新增验证器权威出口 `CollectionsPlan`/`CollectionsBodyFacts`
  （`core/src/collections_facts.rs`）：owned 局部集、逐出口 LIFO `exit_frees`（return/
  break/continue/块贯穿/回边）、`replace_frees`（owned 局部赋值提交时释放旧值）、
  逐可失败节点 `err_frees`（含 pending 临时与存活局部——trap/cancel 出口同样清帧一次）、
  `loop_survivors`（回边轮询错误路径的存活集）。目标 lowering 只消费该计划，不再推导
  ownership；Rust 编译器交叉核对调度（错调度=生成代码编译错）。
- `body_content_digest`：规范 span 无关 body 序列化摘要，凭证层绑定事实与 body 内容
  （V-13）。
- resolve `.len()` 视图融合的延迟索引任务已修正入节点 subtask 序（修复前第二个融合视图
  恒渲染索引 0）；反例 `v02_fused_view_indices_preserved` 常驻。
