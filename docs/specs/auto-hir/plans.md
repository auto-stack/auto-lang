# auto-hir — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 741 | ac-hir-native-core | ✅（delivered，r8；r1..r8均已交付） | [741-ac-hir-native-core.md](../../plans/archive/741-ac-hir-native-core.md) | 首个闭环：Atom 文本 reader（双槽/命名字段互换/重复拒绝/span）+ core-i32-draft 版本化 descriptor（分支 require/forbid、类别失配 vs 悬空引用分诊）+ 语义校验（初始化交集/循环零次保守/块唯一入口角色/求值位置唯一/表达式无环/返回路径）→ CheckedModule 私有构造；四份设计样例 + 15 bind 反例 + 21 verify 反例物化为 fixtures，37 测试全绿 |
| 743 | acc-bootstrap-hir-contract | ✅（r6 reviewed→archived；r1..r5 已交付） | archive/ | 阶段契约沉淀：S0–S6 阶段表 + R1–R6 阶段规则（R3=效果分析/语义两分）+ pass 合同模板 + X1–X9 非法变换负面清单；canonical=[stage-contract.md](stage-contract.md)，全文=auto-acc-bootstrap-contract.md |
| 745 | ac-source-checked-hir-native | ✅（delivered，r1；第七轮最终复审pass） | [745-ac-source-checked-hir-native.md](../../plans/archive/745-ac-source-checked-hir-native.md) | 仅source profile S0接入；生成Atom+来源投影经真bind/verify到Checked，HIR身份/Schema/pass边界保持 |
| 772 | ac-scalar-ops-short-circuit | ✅（delivered，r2；独立复审needs_fix三项修复后final pass） | [772-ac-scalar-ops-short-circuit.md](../../plans/archive/772-ac-scalar-ops-short-circuit.md) | 双身份域（core/scalar）+11 op/Unary/Logical lazy节点；Bundle携HirDomain、verify分域入口、程序化Bundle身份与签名类型边界门（P767-R1/R2）、canonical writer未注册/错域全拒 |
| 771 | ac-record-method-native | ✅（delivered，r3；R1 四发现/R2 一发现修复清偿后 pass） | [771-ac-record-method-native.md](../../plans/archive/771-ac-record-method-native.md) | 第三身份域 record-i32-draft：TypeKind::Record/FieldPath place/record_ctor/borrow_view/mut/Method+ReceiverKind 形态与可信门（含图环/名义/构造完整/mut 排他/含 binding 界检）；stage-contract record 域责任与存储不重叠义务 |
| 770 | ac-text-char-native | ✅（delivered，r11；R2..R10 十轮反例修复清偿后 pass） | [770-ac-text-char-native.md](../../plans/archive/770-ac-text-char-native.md) | 第四身份域 text-i32-draft：StrSlice/StrOwned/Char 类型+12 文本节点+Drop+moved 绑定+ParamMode 三态（与 771 统一枚举）；verify_text 分域入口+TextResourcePlan 资源前门；stage-contract text 域 S0-S6 责任（experimental） |
