# PLAN-769 r2 实施就绪核查

- 2026-10-12；stage:new；outcome:pass（ready for work，非已实施/已测试）。
- observed_main_head：`a45ff543e7f94474f29a0aed76661b8a76f081e0`；plan_revision：2；status：drafting；current_step：0/8。
- 前置证据：PLAN-767 位于 archive，archived 9/9；P767-R1/R2/R3 独立修复重审通过，Spec实际沉淀。auto-ac/source-scalar-i32与HIR阶段合同现存，未知身份/签名类型门已有实现与回归。
- 变更性质：不改变769的目标、支持范围、9类用例或Spec目标；刷新原规划的过期前置说明，细化T-02先冻结oracle、T-03再实现探针的合同，因此revision增为2。r1取材/起草收据保留为历史。
- 8个唯一Task、7个AC、9个mandatory Case、4个SD；work/review/merge责任及一Plan一工作树路径完整；尚无实施勾选或虚构测试证据。
- T-01：实施agent记录用户对r2的执行授权，从启动时主基线创建/确认唯一worktree并刷新输入，调查当前规范与真实消费者差异。
- T-02：新probe-cases.md逐项绑定D/V/AC，先确定候选ABI的值/trace/布局，资源计数口径，非法状态诊断，Unicode预期和预算两侧。该关口属于有界调查工作，允许其具体表示尚待决定；不允许把未定expected交给T-03或770/771。
- T-03/04：Cranelift生成调用者→独立显式shim→链接真实PE→既有contained执行器；宿主直接调用shim不能替代机器调用证据。规则模型负例与生产verifier支持声明分开。正常资源live=0，成功分配和释放配平；fatal trap仅按约定终止，不伪称Drop/unwind。
- T-05/06：D01..07最终全部决定，候选/故障/边界与旧741/745/767门禁实际执行；完整probe/ac-core、fmt/check零warning、严格盘点及真实产品/hash有据，无必需open项才execution_done。
- T-07：独立最终复审同一树的提交、AC、SD和探针oracle；自检pass不能替代最终复审。T-08合入/沉淀/归档/guard清理单独待授权与真实执行。
- 770/771依赖769实际复审合入；文本与record组合、List/enum/use、生产ABI及ACC代际仍为后续范围。
- 就绪核查仅文档：未运行cargo、PE或docs_gen；未创建工作树/agent、未改产品源码、canonical Specs或ledger。保留其他计划WIP。
- 新鲜源/规范/技能/计划指纹见[readiness-inputs-r2.tsv](readiness-inputs-r2.tsv)。旧规划active 767路径映射为实际archive路径；新增scalar规范与767归档收据。存在的link/executor入口包括link_object_staged、publish_artifacts_with_receipt、run_deadlined_split、run_exe，具体调用由T-01/03核对。
- next：用户单独启动实施agent并指示执行当前r2；使用 `/auto-plan:work`，唯一意向树 `D:/autostack/.wt/lang-769/auto-lang`、branch `plan-769-dev`。本收据不授权自动合入、其他计划实施或更换模型。
