# PLAN-828：只补 C05 的公开 recheck 发布失效

基线 `plan-828-dev@0b612fda1ce004f0565a4f2e1057768f841a0f31`，同树 `D:/autostack/.wt/lang-828/auto-lang`。先读 [主计划](D:/autostack/auto-lang/docs/plans/828-av-function-incremental-session.md) 最新§8/9与[本轮报告](review.md)。已有候选所有权设计、旧八行控制及 C01/C02/C03/C04/F05b 保持已验证成果，不重做、不另开Plan/树。

1. **仅修一个发布入口遗漏。** 公开 `recheck_function` 成功会把当前body替换，却未推进供候选校验的代际；只有 FnCandidate::commit 推进，导致旧候选可覆盖经旧入口发布的新当前。盘点已经明确：candidate commit和公开原地recheck是真发布，prepare内部临时swap/恢复、失败recheck、取消/drop不发布，close为终态。使真实发布统一参与基面失效，保留旧公开入口语义，别仅靠文档禁止混用。
2. 收编 [publisher-controls.rs](publisher-controls.rs) 三项同输入/同状态oracle：候选2准备后，旧公开入口成功发布3，旧候选提交必须拒绝且保3；新候选提交3也使旧候选过期；旧recheck类型错未发布时，原候选仍合法提交2。前者本轮FAIL，后两合法对照PASS。不要在prepare共用的临时recheck_swap上无条件推进代际，导致合法同基面候选失效；保留原[八行适配控制](transaction-controls.rs)8/8。
3. 更新 interface-decision 的generation含义/真实发布点及当前关闭表，代码修完后跑三项联动、八行适配、contract16/t01r4、原boundary11/admission2/关闭9和实际触及source/session模块定向验证、格式。不得整档t/tv/tt/tb/tu/taa/tf或docs_gen。提交完整SHA、clean/依赖与精确raw后再受影响面复审；这一遗漏关闭前不开始T02全面换接。

Plan仍executing0/11，C05不另拆新ID；其他OPEN、完整session接线/状态矩阵、最终验收及merge-owned门/沉淀/归档/guard均保持。只补失效规则，不撤销旧能力、不降低基面保护，不再用测试总数替代正反断言。
