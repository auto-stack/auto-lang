# AC 扩面计划编号与战略引用核查（2026-10-13）

- stage:new；scope：用户要求核对后续770/771是否改号并更新相关文档，尤其战略步骤。outcome：pass（引用维护，不是实施或最终验收通过）。
- 主检出：`D:/autostack/auto-lang`，master，基线 `d777ccbb227c41882ad5f6976c803b1e03b61135`。
- 映射：开发线原767标量→772；原768 AutoDown→773；原769共同契约→774；**文本770与record771未改号**。发布线767–769保持各自原义。
- 三份活动Plan：774 revision3 / drafting 0/8，770 revision2 / drafting 0/9，771 revision2 / drafting 0/9。修订仅涉及活跃引用、取材/实施入口和当前事实，支持语义、Task/AC/Case/SD IDs与验收门不变。

## 更新范围

- 774：前置链接指向已归档772；实施指令/拟建工作树/branch/未来脚本和报告改用774；消费者路径用packages/aavm/lib；保留attachments/769历史来源和P767-R1..R3历史发现ID。
- 770/771：共同前置链接及D01..07规则引用指向774；标量前置指向772并说明已交付；两者本身编号、拟建工作树及互相引用不变。实际依赖仍是774完成并复审合入后才能实施。
- [Design 36](../../../design/36-ac-data-resource-expansion.md)与设计索引：更新依赖图、当前阶段和编号说明。
- [后端演进战略§7.5](../../../design/strategy/auto-native-backend-evolution.md#75-现行acacc队列与编号映射2026-10-13)：新增现行队列/映射/依赖图，保留2026-10-04历史快照；路线为772已交付→774→770/771并行→后续组合与真实ACC/自举。
- [v0.6 roadmap §3.3](../../../roadmap-v0.6.md#33-rust-ac--auto-aac--自举闭环)：关联现行计划与战略队列。
- auto-ac/auto-hir/auto-acc模块Specs与plans导航：只修正标量Plan引用和真实归档链接；支持/拒绝、profile、阶段、脚本及实现报告语义不改，没有发布data-resource-contract或宣称新语言能力。

## 保护与验证

- 旧planning-inputs/preflight/readiness、冻结反例/输出/报告、archived Plan均未改写；旧脚本`verify-ac-scalar-767.ps1`与`reports/767-scalar-i32/`实际仍使用，不凭改号重命名实现资产。
- 本次新增/更新的Markdown相对文件引用以及三份Plan、战略/设计/模块Spec当前链接验证存在（锚点按当前标题核对）；三份Plan头部ID与文件名对应，任务数量/未勾状态保持；活跃部分不再指向旧共同769或旧标量归档路径。KNOWN-DEBT旧段另有12处预存链接失效（指向已归档的741/743/762旧active路径），与HEAD原文比对保持同一集合，本次没有新增此类错误，也未修改那些历史段落。
- 当前输入指纹及历史到现行路径映射：[reference-update-inputs-20261013.tsv](reference-update-inputs-20261013.tsv)。T-01仍需在真正启动实施时刷新工作树基线与工具依赖。
- P774-D1交接引用漂移销号：工作树/指令/前置/消费者路径及下游链接已同步；不表示774实施完成。
- 保留775计划/合流收据/refs JSON的原未提交字节与未跟踪auto/；不修改其他会话工作，不建树、不合入、不归档、不改ledger或.next-id。
- 文档/计划引用维护：不运行Cargo、PE、docs_gen；仅链接、指纹、编号/任务计数及whitespace核查。

next：在实施agent授权执行774当前revision3并完成其独立复审/合入；之后才能并行实施770/771当前revision2。
