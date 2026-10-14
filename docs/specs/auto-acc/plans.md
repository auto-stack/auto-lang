# auto-acc — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 743 | acc-bootstrap-hir-contract | ✅（r6 reviewed→archived；r1..r5 已交付） | archive/ | ACC 自举盘点：八源模块审定 + 六新建主体 + 能力四态取证 + 可重跑盘点工具（r6 后含全序绑定优先/整组作废/缺失绑定淘汰）；r1 盘点、r2 质量、r3 完整性、r4 消费者完整性、r5 收口、r6 边界兑现六轮独立复审闭环 |
| 745 | ac-source-checked-hir-native | ✅（delivered，r1；第七轮最终复审pass） | [745-ac-source-checked-hir-native.md](../../plans/archive/745-ac-source-checked-hir-native.md) | MD408/409追加有限profile数值取证；完整int/char与ACC主体/Gen1–3仍open，不升级required清单 |
| 772 | ac-scalar-ops-short-circuit | ✅（delivered，r2；独立复审needs_fix三项修复后final pass） | [772-ac-scalar-ops-short-circuit.md](../../plans/archive/772-ac-scalar-ops-short-circuit.md) | MD-409/410标量子合同有限域取证（全序/eq-ne/除余截断/短路/条件for，真实PE）；required不因有限域证据改项，全语言宽度/char/str/List仍open |
| 771 | ac-record-method-native | ✅（delivered，r3；R1 四发现/R2 一发现修复清偿后 pass） | [771-ac-record-method-native.md](../../plans/archive/771-ac-record-method-native.md) | MD-403/404 有限子集 Rust 原型取证（type 块/static fn/构造拼写/名义语义/字段路径/mut 参数）；required 不改项、Gen1/GOAL-017 不关闭 |
| 770 | ac-text-char-native | ✅（delivered，r11；R2..R10 十轮反例修复清偿后 pass） | [770-ac-text-char-native.md](../../plans/archive/770-ac-text-char-native.md) | MD406/407/408 text profile 有限取证：lexer.at char_at/len/slice/拼接与 token.at keyword 需求有首批真实 native 语言面可对拍；完整 token/lexer 编译仍后续 owner，required 不改项 |
