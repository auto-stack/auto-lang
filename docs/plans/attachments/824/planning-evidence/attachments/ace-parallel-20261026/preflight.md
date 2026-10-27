# 823–826依赖审计与模块并行预检

2026-10-26；stage:new；outcome:pass（规划合同静态预检，非独立实现review或执行证明）。

用户授权审计实际依赖并修订计划/战略，尽量并行独立模块，保留共享核心单写/交接。使用auto-plan-new，仅读取current Spec、具名Plan与主检出实现，不新建实施Plan、树、代理，不联系其他会话，不改Rust/Schema/现行Spec/ledger，不实施822修复或新模块。

## 输入与结论

输入提交/文件指纹见[inputs.json](inputs.json)。调查821已archived r6、owned Spec已交付，特别核实MutRefV无解引用入口且不承诺runtime失效句柄/旧帧/跨store/generation回绕校验。823/824的新访问能力仍须各自实现实证，不继承不存在的保证。

主检出MirType/Value和公开layout/place/StrArena、verify_mir/verify_abc、FunctionCode/ProgramSnapshot提供已交付数据及信任边界；ModuleView为pub(crate)、Frame私有，无已交付通用插件executor API。823完整首例含List<record>，仍等D822；824递归白名单拒Fn/Closure，不消费823环境/调用，正式集成改为D822。825派生PC表可独立，真实派发需核心窗；826codec算法消费语义MIR而非825私有帧，独立先做，正式路由/准入/LoadedProgram/AV仍等D825。

822在本轮期间由其他审查者记录R1 needs_fix并将进度6/9→4/9；本轮未动822，保留初始hash与concurrent_input_observations。该变化不解除D822，反而确认不能消费在途List/增量实现；当前Spec与主检出实现输入hash保持。它不是本轮检查失败，也不是本轮宣告822修复完成。

## 修订范围与预检

- 823/824 r5→r6，825/826 r4→r5；各新增work T-10，total9→10，drafting/0和原勾选保留；T-01→T-10→原T-02集成及后续验收，不增平行的第二任务表。
- 新AC-10与823/824 V-14、825/826 V-13核模块/生产同源、阶段D门、权限/窗口和实际集成；现有SD-01追加模块/准入边界，未移除原任务、AC/V/SD、目标或预算。AC-07/T-07显式核新义务。
- 824 V-12明确类型可达性：Fn/borrow及未知tag现在可拒；Closure已交付时必须用真实类型。最后集成者负责真实Closure与Dyn合法值并集，不以future skip或unknown-tag冒充已存在类型控制。
- 同一新增生产源由parallel/<id>独立harness编译，最终核心直接消费，不建第二值体系/IR/store/VM，不改公共装配，不产生core↔扩展循环；私有API不足在T-10最小接缝阶段给具体缺口，不靠WIP/unsafe通过。路径/构建机制为拟议实现，当前未创建或编译，T-01/T-10负责可行性。
- 模块绿不替代原T-02首个真实全路径的store/new-worker/AutoDown/trace/资源，原结构/P95/内存/cleanup反控保持。公共类型/指令/verifier/loader/dispatch/snapshot、lib/manifest及recipe注册单写；review期间其他私有模块可继续，不交未审公共定义，不持窗等上游。

机器可读检查见[checks.json](checks.json)：各Plan原ID/标记/状态与预算承诺保持、revision与total按预期增量、阶段依赖图、review门、新链接、输入变化归因、差异空白检查。原规范行除AC-07/SD-01补边界及824 V-12类型可达性外保持；新义务不是实现已通过。

本轮未执行cargo、docs_gen、AutoDown或性能/内存战役。工作区原有他方修改保留；最终review和merge-owned冻结门禁仍需各Plan独立完成。当前可交work的是T-01及满足已交付基面后的T-10，不代为启动。
