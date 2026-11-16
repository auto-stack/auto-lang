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
- **回边轮询的覆盖（PLAN-1036）**：**每实际回边都消费同一合作 poll**（1 WU + 取消检查），
  包含 `continue` 驱动的回边——`continue` 先执行本循环的推进（counted for-in 的 latch）并在
  本迭代 scoped 集释放之后方才 poll；`break`/`return` 出口不 poll。poll 错误路径=该迭代
  scoped 集 + `loop_survivors`，因此目标端释放必须**真幂等**（见下条）。
- `body_content_digest`：规范 span 无关 body 序列化摘要，凭证层绑定事实与 body 内容
  （V-13）。
- resolve `.len()` 视图融合的延迟索引任务已修正入节点 subtask 序（修复前第二个融合视图
  恒渲染索引 0）；反例 `v02_fused_view_indices_preserved` 常驻。

## C 后端消费边界（PLAN-925 CA-B）

- C lowering 与 scalar 通道一样**只消费**本文件单源事实：param 模式/`FnKind`/
  `BodyResourcePlan` 逐出口清理计划原样入 C；C 端不因"编译器没借用检查"放松——
  同根排他/mut 冲突在 session 入口具名拒（`verify.mut-conflict`）。
- **S 窗守卫（T-05 修复，验证器侧）**：mut 借用根不得进入 exit live 集——三处
  reown 点以 `plan.owned` 为守卫（`text_resources.rs:1822/1951/2028`）；由此
  mut 资源 record 参数由 **callee** 排出口 free（此前 Rust 侧被幂等 take-free
  掩蔽的存量缺口；902 冻结语料无此形状）。replace_frees 经借用照旧。
- 恢复后所需的只读事实经 AXP2 锚+prelude seed（record 表）与逐函数 slice/凭证
  重验重建，不 deserialize Checked/PASS/宿主地址。
- **C 消费（PLAN-926）**：C 集合降低以同一 `CollectionsBodyFacts` 为唯一清理
  权威（owned/exit/replace/err/loop_survivors + body digest 凭证），不重推
  ownership；包装临时（a2x_stro_result/a2x_res_*）被调度引用时释放 `.value`；
  释放幂等（take 置空/memset）使回边 poll 错误路径重列安全。**Rust list 释放的幂等是
  同一规则的一部分（PLAN-1036）**：`a2x_coll_list_free` 先取出缓冲（take）并令陈旧
  handle 的 cap=0，第二次释放是**真 no-op**——不二次 drop、不重发释放事件（此前只清空
  元素、未失效缓冲，围栏错误路径重列会二次释放并 0xC0000374 堆损坏）。S 面 join 修复
  （终止分支不约束 join）对两目标同时生效（golden-L03 最小重冻结）。

## Rust 消费锚（PLAN-931）

- Rust lowering 与 C 一样**只消费**本文件单源事实：`BodyResourcePlan`
  逐出口清理与 `ExternalFn.modes` 原样入发射；owned 临时（view 消费的
  pinned 值）的责任锚与释放作用域见
  [temporary-consumer-lifetime](temporary-consumer-lifetime.md)——pin
  Let/语句根为消费者，`drain_since(mark)` 作用域释放，短路侧块自责。
  目标不重推导、不把 move 强转 view、不默认 View 片段模式。

## PLAN-939 增量：唯一 per-call 凭证计划与严格消费（reviewed）

- core 产出唯一 `ForeignCallCredential`（view borrow 携**派生 owner local**
  （Extent=调用；字面量/临时=无 owner）、exclusive Mut/Out 车道+sig atomic
  承诺、handles_in+provider tag、owned_out 凭证 provider、callback 具名
  目标 def、ctx symbol）与 `ForeignResourcePlan`（受检声明+调用点计算）；
  经 CheckedModule.foreign_resource_plan / CheckedFunction.foreign_rows /
  SemanticView 双路携带，`foreign_call_credential` 统一入口。
- a2x lower_rt **严格消费**：资源 foreign 调用缺凭证行=`a2c.internal` 响亮
  拒（绿色=每个 CI 腿资源调用可证消费计划）；callback 目标计划驱动+逐实参
  覆盖一致性核。native 侧以 MIR callbacks+TextResourcePlan 为所有权权威。
- 后端不复制 ownership 推导；中立 facts 只是投影。

## PLAN-945 增量：公共 std 包四端资源事实（基础数学与文本切片）

HIR→Rust/HIR→C 对冻结 `std` 包（`stdlib/ace/std/*.at`）的真实资源行为：
`std.text.owned`（视图 clone）在两端 exe 的分配故障钩子 `A2X_RT_FAULT_ALLOC`
下为类型化 `trap:Resource`（rust `rt_alloc` 故障序 / C `a2x_rt_cfg_fault`），非
panic；`std.math.square/cube` checked i32 溢出=`trap:Overflow`（stdout 协议
`value:<i32>`/`trap:<Name>`）；`byte_len`/`equal` 无分配。AV arena seam 扫描：每
次注入=typed abort+零存活缓冲。manifest 准入：真实 stdlib 声明（六 Declaration）
经 packages profile SemanticAssembly 在 AV/Rust/Native/C 四 producer
freeze/validate/admit，篡改声明 `assembly.stale-identity` 拒。证据
`experimental/ace/crates/av/tests/cases/stdlib_core_945.rs` 与
`experimental/ace/tests/stdlib/tests/conformance.rs`；详见
[../../stdlib/design/ace-core-and-text.md](../../stdlib/design/ace-core-and-text.md)。


## PLAN-946 附录：跨单元共享实参与集合资源边界（2026-11-14）

链接快照（ProgramSnapshot）按全单元 collections 族标志答 profile 谓词：跨单元
List/Variant 实参不再 move-in 杀调用者槽位，保持调用者所有权（Rc 共享）。资源
边界：AV arena fault 扫（每次分配尝试具名中止、live=0）＋ a2x
`A2X_RT_FAULT_ALLOC` 于 collections 通道=typed `trap:`；失效/恢复（精确段失效、
restore 0/0 重发、stale receipt 拒）合同见
[../../stdlib/design/ace-collections.md](../../stdlib/design/ace-collections.md) §4。

## PLAN-1035 增量：聚合分类修正不转移事实权威（2026-11-16）

- Collections 受检事实的唯一生产者仍是 core verifier（`CollectionsPlan` /
  `CollectionsBodyFacts`：Str/List/Deep 的 owned/pending/exit_frees/replace_frees/
  err_frees/loop_survivors）；record 的 Deep 判定沿用 `TypeClassifier` 的
  Record→List fixpoint（字段可达 List 即 Deep）。本计划**不新增事实字段、不重推
  ownership**。
- 分类修正后 List-only record 的 Deep 释放进入既有计划（实测帧账闭合、`facts.owned`
  含其局部）；A2X 两目标继续只消费原 facts/mode——**不新增聚合 MutBorrow ABI**、
  不改参数模式事实与目标差异。
- Python 目标（PLAN-1042）作为共享事实 consumer 纳入回归（`ace-a2x` 组含
  `ace-a2x-python`），不依赖 GC/`__del__`，不复制分类分析。
- 证据锚：`audit1035_res_01_derived_facts`（facts.owned 含 Bag 局部）、
  `ace-av`/`ace-a2x`/`ace-a2c-abi` 固定组（PLAN-1035 T-03 结果）。
