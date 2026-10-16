# 2026-10-16 AC/ACC战略要求对787与后续计划的影响

本记录是设计/排程影响说明，不是PLAN-787 r2，不修改正在执行的r1任务、AC、预算、进度或证据。用户本轮讨论了版本分期、ABI/FFI时序、首次编译即增量cache及新测试尽早接入；据此同步目标态文档，不实施代码。

调查时master为4677d7361c0de7a7ad1d863c78f2d329c6ebb48e，787为executing r1/0步，工作区已有其status更改及其他计划WIP。本轮不覆盖这些改动、不联系实施者或创建代理。783已在canonical auto-ac plans中登记交付；787进入缓存集成时仍应按原D-783核实际API/收据，而不是沿用旧复审状态。

## 决定与影响

| 要求 | 本轮同步的战略 | 787/下一计划 |
|---|---|---|
| v0.6正确性，MIR/优化v0.7 | 独立Typed MIR与性能pass非v0.6门槛；资源/ABI正确降级、效果合同保留 | 787无需加MIR；不能以延期省增量/预算 |
| LLVM约80%、少量高收益优化 | 作为待冻结基准目标；先测Cranelift档位，不以pass数量换性能 | 后续性能计划先定基线/语义/样本/阈值，非787任务 |
| ABI/FFI早做 | 有限AutoABI合同+Auto↔Auto/C双向独立编译探针，公开前冻结；Rust C-shim随后扩面 | 另立ABI计划，不等完整AC，不在787写loader |
| 首条完整管线带cache | snapshot/HIR/object/link各实际阶段从首例有key/依赖/store及失效验证 | 787仍只交语法层；紧随的native集成按G-01–05交付，不靠783探针替代集成 |
| AutoDown尽早 | 优先Design35 B：AC source/HIR/native；syntax/增量观测显式扩adapter | 下一个优先新计划是测试接线；787保留既有内省/差分测试，外部adapter引用其公开结果 |

## 后续规划交接

同日进一步要求：亚秒周转指标从系统及每阶段首例进入测试。依据Design39§5.6、Design35§10和后端G-06，后续AutoDown接线及语义/CT/ABI/原生集成计划须绑定端到端trace、阶段耗时/等待/命中计数/内存I/O、负载/冷热/变更与代表性能门禁；不能以每阶段<1s证明总周转，也不因v0.7优化延期后置测量。本记录不直接改787执行合同；其已有perf/增量用例可作数据来源，若新增测试/阈值改变AC或预算，按实际证据修订合同并保留历史。

同日新增要求详见[Design42](../../../design/42-semantic-metadata-inference-and-comptime.md)：语义meta、类型推导、泛型实例化、受控HIR→ABC→AVM编译期查询及runtime反射保留必须共同设计。A05/A06查询与783/787的全局store合同一致；“源不变直接复用CT结果”还需验证被调用实现、反射类型/候选集合、target/配置/VM版本与受控外部输入不变。下一轮语义语言计划须先消费该合同，787的partial CST不授语义/执行信任。此处仍不改787任务/预算/AC，新增语义工作包单独取号。

测试接线计划应先读auto-test-case/profile-v1、Design35与ac-core现有executor；新增原生adapter属于现行v1不支持面，必须显式版本/支持表、阶段负例/产物链/deadline与缓存策略，不能仅给文件换后缀。首批引用式导入现有core/HIR正反及资源样例，required缺例/skip/tool-error保持红。语法节点/诊断/增量读数需要单独声明观测，不能忽略未知字段。

ABI计划先形成有限profile与真实互调证据，再称定稿；原型内部ABI与Cranelift工具桥不冒充公开AutoABI。FFI与公开模块代码继续按原Plan/worktree范式。

首次新前端到native的集成计划须明确clean-vs-incremental、进程重启/跨工程共享、body/interface/config/target失效、读者保活/GC、坏块重算与端到端资源/时间；输入可以很小，但不能以hello-world无优化价值省掉架构接线。

本轮仅核设计/代码事实，无编译、优化基准或运行测试。来源：后端演进战略、ABI RFC、Design35/39/40/41、当前auto-test-case Spec、ac-core/native.rs两处none；锁定cranelift-codegen0.126.2本机源码Context::optimize含egraph入口，官方当前settings提供none/speed/speed_and_size。上游或其它语言的性能不作Auto性能达标证据。
