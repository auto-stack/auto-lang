# PLAN-820 r6 R5 — needs_fix (T-09 reopened)

2026-10-24；review_scope: final / affected repair + remaining authorization/session seam。reviewer：本聊天Codex，独立于实施上下文，未创建代理。唯一实施树 D:/autostack/.wt/lang-820/auto-lang；branch plan-820-dev；reviewed_commit affb907bd8657d32a027453268cb209450b6c73b；repair base a789a2adc1afdab97025a0c294bb6121c97aa797；original base 37e51253689631ffa61a9af58041c5f6646a7569。入审及结论前HEAD相同、实现clean、diff-check通过。

主检出入审HEAD369e1c349ea9587907f073aaecd3082e2bb486bd（已晚于用户交接簿记ae733aef9）；Plan入审SHA256 CFB140E4AC311297ABE04A7DD19EB8037B0050D08B2BD94878BB31264C87E979；SD草案SHA256 50B2176B9E9AB83017A26235C0A680B25B771D315C2DC06056FC1B910DF727D8。实际Plan executing7/9，按完成声明再审。

## R4 findings 全部关闭

- **F820-R2-02 CLOSED**：独立TIER_CONTRACT已覆盖13个具名档（core support档有listing/run完整性门），先对合同项与listing，再对listing与真实ok集合。后缀绑定保留root全限定名称兼容。独立运行 `pwsh -File scripts/verify-av-data-820-missing-id-control.ps1`：验收子脚本exit1逐项列出12个缺失合同用例，控制器确认后exit0；R4原listing和execution同缺的形态已拦截。GuardSelfTest 5正7负通过；本轮真实cache Scope与semantics Scope均PASS，分别绑定3+1和13个实际ID，不用模拟输出作为产品正确性证据。
- **F820-R4-01 CLOSED**：独立rustfmt --edition2021 --check --config skip_children=true检查layout/mir_verify/data_session/incremental/resources五文件均exit0，新增资源测试已格式化。本轮修复只格式化资源测试，不使前轮布局行为证据失效。

前轮已闭事务/共享布局预算/真实溢出/真实操作反控/native/父侧峰值证据保留：相关实现、依赖、输入未因本轮脚本与格式变更改变，未重复性能矩阵或完整独立workspace套件。

## F820-R5-01 P1 — 新授权没有更新前端驻留环境，后续合法热编辑被错误拒绝

**既有要求**：§5全链增量专项 / AC-09 / V-13子case3 / T-09 要求授权事实变化的正确失效与拒绝正反控，支持新增能力的真实入口不能仅代码表重装载正确而前端仍按旧授权判断。SD-07亦宣称caps变化重建unit/admit、HIR身份保留。

**绑定输入、命令、期望与实际**：

- [完整重放源码](caps-replay.rs)，[运行器](run-replay.ps1)，[原始输出](caps-result.txt)。运行器用受审probe/core rlib编译外部临时exe，不修改实现。主检出运行 `pwsh -File docs/plans/attachments/820/review-r5/run-replay.ps1`。
- 无授权冷建 `type P { x int }` + main构造P{x:42}并return p.x；调用 `apply_caps(["hir.test.trace.v1"])`；随后唯一函数body改为 `return mark_a()`。
- 期望：grant成功后，同一session的body edit按新授权解析/受检并返回1，保持约定的事务/粒度。
- 实际：`grant=Ok(["d_main"])`；`hot_edit_after_grant=Err("candidate rejected: source.undefined no function named mark_a in this module")`；session仍42。
- 合法冷对照：同一修改后源码、同一caps，`DataIncrementalSession::warmup`成功并返回1，证明该输入在已批准record/trace支持面合法，不是未知语法/不支持调用。
- 合法拒绝对照：从带trace源码/授权的session尝试移除caps，返回mir.capability，原session仍返回1；本轮未把这种失败事务保留旧版行为判成绕权漏洞。

**已建立原因**：data_session.rs:1022..1067的apply_caps只替换self.caps并重新lower/admit、link/run/publish代码快照；没有按新caps更新self.env中的受检解析环境。下一次edit_body:406仍把旧self.env传给prepare_candidate_document。对比apply_source_impl:993..999会调用resolve_env并更新self.env。新增授权的接口目录（mark_a/mark_b）没有进入热前端。现有caps回归只断言重装载数/HIR身份及原程序结果，未组合“授权变化→使用新增能力的body编辑”，故全绿未覆盖此缺口。

**最小关闭范围**：将授权对应的前端环境/接口事实与caps/code/snapshot在同一成功发布点一致更新，失败时全部保留旧值；不要求body不变时重新生成HIR。定向覆盖：无授权→新增trace授权→合法body用mark_a成功且冷/热同值；已有授权无变化零重建；实际使用trace时撤销的受控拒绝及旧会话保留；成功撤销后下一次无权使用被拒。复查env/function槽位/签名目录同步与失败后重试，按实际机制选择代表性兄弟入口；不跑整档、不要只在测试里绕过驻留环境或强制whole apply_source作为热编辑workaround。

## 剩余审面与结论

本轮在R4两项关闭后继续检查了类型消费、宽标量/来源、cache restore路径、InstructionSpec及正式recipe接线。独立semantics13例/cache3例+新worker恢复1例均通过；读取实现证实restore调用envelope→ABC深验，正式data recipe消费record source并以scalar exit父侧判定。完整V-08“好hash坏内层”现场篡改恢复、V-10错bits/工具身份所有子控、完整V-12/SD逐目标对账尚未全部完成，不能由目前测试总数冒领独立pass。SD草案仍有过时统计（如incremental5/5）与caps跨层义务待修后核对，未写canonical或live ledger。

AC-09 fail（新F-R5-01）；AC-06聚合完整性原finding关闭，但其他子项最终全核仍partial；AC-01..05保持前轮各具名关闭/定向证据和未审边界，不标整项pass；AC-07未过；AC-08 merge pending。回executing6/9，**只重开T-09**，T-06保持本轮独立修复完成，不再要求重修已关闭脚本/格式。

先回work补本条具名根因/组合覆盖与实现提交，再审受影响授权机制及未审面；不为已知in-contract缺口运行完整final聚合。merge-owned裸cargo t+tv+独立workspace完整有限合同档仍pending。未改实现、未合入/归档/清树。
