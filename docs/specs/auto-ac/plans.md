# auto-ac — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 741 | ac-hir-native-core | ✅（delivered，r8；r1..r8均已交付） | [741-ac-hir-native-core.md](../../plans/archive/741-ac-hir-native-core.md) | 首个原生 AOT 闭环：Cranelift 0.126.2 → COFF → rust-lld/MSVC 双链路 → PE 真实执行（add=5/1、count=3/0/0、溢出 trap=70、调用顺序 b→a 结果 12）；capability 显式门（hir.test.trace.v1 + no_std 支持库）；ac-probe check/build（退出码 0/1/2、原子制品替换不覆盖、收据）；工具链发现（KitsRoot10 注册表/env 覆盖）收据 toolchain.md；verify-ac-741.ps1 一键复现 |
| 745 | ac-source-checked-hir-native | ✅（delivered，r1；第七轮最终复审pass） | [745-ac-source-checked-hir-native.md](../../plans/archive/745-ac-source-checked-hir-native.md) | 单模块Auto计算源码→Checked→Windows PE；精确I32/Bool、求值序/runtime trap、事务发布与迭代深块正反验收 |
| 772 | ac-scalar-ops-short-circuit | ✅（delivered，r2；独立复审needs_fix三项修复后final pass） | [772-ac-scalar-ops-short-circuit.md](../../plans/archive/772-ac-scalar-ops-short-circuit.md) | scalar-i32 profile：减/除/余（0、MIN/-1、MIN%-1 trap70）、全序比较、同类eq/ne、一元、布尔短路与条件for；身份精确可失败映射（未注册descriptor双拒bind/writer）、verify分域入口、签名/引用边界门；14步45例门禁+独立复审反例修复 |
| 774 | ac-data-resource-contract | ✅（delivered，r6；六轮独立复审收敛） | [774-ac-data-resource-contract.md](../../plans/archive/774-ac-data-resource-contract.md) | AC 数据扩面共同契约：类型/文本/str-Str/借拥有前门、内部 ABI 五类、资源合同（完整账本+u64 计数+预算）；probes/data-resources 64 探针+状态探针全绿、五轮反例重放清偿；770/771 唯一接口来源 |
| 771 | ac-record-method-native | ✅（delivered，r3；R1 四发现/R2 一发现修复清偿后 pass） | [771-ac-record-method-native.md](../../plans/archive/771-ac-record-method-native.md) | record profile：名义 record（前向引用/无环）、位置/命名构造（源码序求值+值捕获不变式）、字段路径、static/只读方法、mut 参数；目的地不相交 copy elision 与操作数求值位捕获两不变式；ABI-D/E 聚合参数返回；24 步门禁 23 例+57 测试 |
| 770 | ac-text-char-native | ✅（delivered，r11；R2..R10 十轮反例修复清偿后 pass） | [770-ac-text-char-native.md](../../plans/archive/770-ac-text-char-native.md) | text profile：str/Str/char 首批有限值语义与 774 资源合同前门（TextResourcePlan：owned 词法作用域/provenance 构造代际/清理责任 incoming 三态/参数对排他跨函数传播）；runtime/text 静态库+合同 sidecar；清理责任不变式 r2..r10 迭代冻结 |
