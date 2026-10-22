# PLAN-815 最后一项dist验收修复：规范影响核验

2026-10-22，Plan r3，review_scope=phase（仅F-815-T07-R1-01残留，不是完整T-07/T-09）。HEAD c203d9642a58a3bceeb2075a7ee116e79a416e45，测试修复f8c01f76bbd9b3c151388f4f0744701a808d7598，base c050e718e09b66ebb6d7925c1f4e0ce89bb10dc5。

本轮只有lifecycle.rs断言增量，无生产实现、公开类型、wire、预算、依赖、配置、资源或既定支持面变化。工具仍由生产提交e351cb2a构建，SHA256 f21db18136403d1124ecbc1338f61c5603544401c434e7d12fad20a60c37dfb5，与冻结handoff一致；不能要求测试提交改变可执行身份。当前Spec源hash及未变证据输入见baseline.json.txt/evidence-reuse.json.txt。

SD-01..07沿前轮核验，所有权/依赖方向/协议/单源与生命周期规则不变。本次仅使SD-06关联AC-04/V-04的成功产物验收在本生成器Vue打包形态下区分JS入口引用与CSS引用：存在的.js引用须至少一个，且原有引用存在性、JS束和成功报告检查全部保留。正常与嵌套JS接受，缺文件/无引用/CSS-only均拒绝，真实构建正控通过。F-815-T07-R1-01本项关闭，无新增规范合同或Plan修订。

前轮四项CLOSED按源码、依赖、配置、输入未变范围继承；本轮不是重新发布其能力。T-08准备稿及完整SD发布性继续受T-07完整发行/资源/字段/协议矩阵约束。SD-01/02最终基于当前已发布A内容增补并保留来源，原待办不因本项pass消失。Plan的supersedes_spec_components/new_spec_components/touched_goals保持r3，不写canonical/live ledger。T-09完整终审及最终pass后merge-owned冻结裸t/触面档/串行合入与知识沉淀仍pending。
