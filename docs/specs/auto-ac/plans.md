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
| 782 | ac-text-record-owned | ✅（delivered，r3；R1 六发现/R2 两发现修复清偿后 pass） | [782-ac-text-record-owned.md](../../plans/archive/782-ac-text-record-owned.md) | text-record 组合 profile：派生资源分类、完整根 move/深 clone、拥有形态 allowlist 四位、字段视图与替换（字段写永不复活移出根）、770 清理不变式扩资源根、跨类型统一 LIFO、短路聚合临时跨嵌套存在标记、故障双通道（CLI 注入+部分输出回滚）；25 步联合门禁+90 测试 |
| 783 | ac-artifact-contract-probe | ✅（delivered，r6；R1–R5 五轮反例修复清偿后 pass） | [783-ac-artifact-contract-probe.md](../../plans/archive/783-ac-artifact-contract-probe.md) | A00–A12 统一产物契约+缓存可行性探针：域分离身份/CAS 读取准入/二进制 HIR 恢复重验/两种冻结布局(128MiB×2)/受管 store(单临界区发布/lease OS 真值/完整状态矩阵 GC/64-48MiB 真实总量收敛/8MiB 临时空间含残留 tmp 原子配额/namespace 根边界);512 churn 冻结水位绿+30样本×6场景;[artifact-contract.md](artifact-contract.md)+[artifact-probe.md](artifact-probe.md) |
