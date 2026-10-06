# auto-hir — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 741 | ac-hir-native-core | ✅（delivered，r5；r1..r5均已交付） | [741-ac-hir-native-core.md](../../plans/archive/741-ac-hir-native-core.md) | 首个闭环：Atom 文本 reader（双槽/命名字段互换/重复拒绝/span）+ core-i32-draft 版本化 descriptor（分支 require/forbid、类别失配 vs 悬空引用分诊）+ 语义校验（初始化交集/循环零次保守/块唯一入口角色/求值位置唯一/表达式无环/返回路径）→ CheckedModule 私有构造；四份设计样例 + 15 bind 反例 + 21 verify 反例物化为 fixtures，37 测试全绿 |
| 743 | acc-bootstrap-hir-contract | executing（r5 Phase5；r4合入后needs_fix） | [active Plan](../../plans/743-acc-bootstrap-hir-contract.md) | 阶段契约沉淀：S0–S6 阶段表 + R1–R6 阶段规则（R3=效果分析/语义两分）+ pass 合同模板 + X1–X9 非法变换负面清单；canonical=[stage-contract.md](stage-contract.md)，全文=auto-acc-bootstrap-contract.md |
