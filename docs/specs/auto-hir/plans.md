# auto-hir — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 741 | ac-hir-native-core | 🚧（executing，r3；r1/r2已交付） | —（docs/plans/） | 首个闭环：Atom 文本 reader（双槽/命名字段互换/重复拒绝/span）+ core-i32-draft 版本化 descriptor（分支 require/forbid、类别失配 vs 悬空引用分诊）+ 语义校验（初始化交集/循环零次保守/块唯一入口角色/求值位置唯一/表达式无环/返回路径）→ CheckedModule 私有构造；四份设计样例 + 15 bind 反例 + 21 verify 反例物化为 fixtures，37 测试全绿 |
| 743 | acc-bootstrap-hir-contract | 🚧（executing，r4；r1/r2/r3已交付） | —（docs/plans/） | r3盘点/规范修复已落地；合入后复审仍有consumer类型、证据适用、导入/参数遮蔽与生命周期缺口，Phase4 T-22..27 / AC-21..24跟踪；原63测试与真实清单严格门通过，不能覆盖新负例。 |
