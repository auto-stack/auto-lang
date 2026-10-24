# PLAN-845 r1 planning preflight

2026-10-24；stage:new；静态合同检查通过，实施前置 G840 pending。

- Plan：`docs/plans/845-autographics-bound-behavior-and-portable-projects.md`；SHA-256 `d46ab9d271dc7bcf1fd6051f1afa3d9077ca0ca12cb03788c876a1cabddc3377`。
- 结构：0..10 节完整；唯一 T 10 / AC 10 / V 10 / SD 4；任务 owner_stage = work 8 / review 1 / merge 1；所有 AC 均有任务负责。首次尾检发现任务表使用 AC 简写，已展开为明确完整 ID，复检通过；没有删除验收项。
- 取号：主检出持有既有 `.allocation.lock` 排他句柄，重核 active/archive 最大号与 `.next-id` 后以 new-plan.sh 分配845；计数846。841–844属其他路线，未覆盖。
- 来源：21个Git对象按各自ref验证blob/SHA-256/字节数通过；14份文档输入hash仍与观察时一致。840候选 e98e210c769019f153c6b9dc949cdb62e61ddb4e；SDK候选932ad4877e5968f1aa7227d9dcd7fb69b4c69a21；正式集成仍为839交付a239653365a0c3451c85aa9921189fcf011bd8cf。候选不是845正式实施冻结。
- 前置：调查时840 executing r2 3/12，正式API/来源仍在途；831 executing r2 8/12、832 drafting0/12。G840须最终独立审查、完整冻结门、实际落地、SDK来源归属、Spec/ledger/归档/guard/到期收尾。
- 语义核对：T02有界whole-path探针；一个scene/4入口+源码blob+actor ID；hash/字节/角色/API身份贯通，绑定/Open不执行业务callback；Undo真实payload存续、8源码/2MiB预算；GLB+code先准备、同root锁/单entry提交、完整回读再clean；旧schema/840外部临时源语义保留。
- 区分案例：entry前/后故障、坏同hash/未知文件/保留目标、Save As原路径失效、新root冲突、stale epoch/revision/generation、20轮源码/VM/GPU最后owner及合法pin、同binary仓外两例、真实OS/GPU正反与partial，均有case/AC/work owner；所有新fixture由work建立，非本轮已执行。
- 范围与时序：只写拟议driver局部，SDK/AGP/production/root锁只读；完整组件/默认设计体系沿Design47后续研究，不进入840/845。本轮无建845树/branch、prepare、构建/Cargo/原生测试、agent/chat；work/review定向，final pass后merge冻结完整门再落autographics-dev。
- 尾检：相对链接/锚点、附件、忽略规则、空白及scoped diff通过。840与全部14份规划输入未改，canonical未写。

结论：草案可评审。G840完整交付后，后续work指令启动T01重核正式SHA/依赖/Spec，再执行T02。当前drafting0/10，未实施。
