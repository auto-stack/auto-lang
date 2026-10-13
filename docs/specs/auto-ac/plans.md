# auto-ac — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 741 | ac-hir-native-core | ✅（delivered，r8；r1..r8均已交付） | [741-ac-hir-native-core.md](../../plans/archive/741-ac-hir-native-core.md) | 首个原生 AOT 闭环：Cranelift 0.126.2 → COFF → rust-lld/MSVC 双链路 → PE 真实执行（add=5/1、count=3/0/0、溢出 trap=70、调用顺序 b→a 结果 12）；capability 显式门（hir.test.trace.v1 + no_std 支持库）；ac-probe check/build（退出码 0/1/2、原子制品替换不覆盖、收据）；工具链发现（KitsRoot10 注册表/env 覆盖）收据 toolchain.md；verify-ac-741.ps1 一键复现 |
| 745 | ac-source-checked-hir-native | ✅（delivered，r1；第七轮最终复审pass） | [745-ac-source-checked-hir-native.md](../../plans/archive/745-ac-source-checked-hir-native.md) | 单模块Auto计算源码→Checked→Windows PE；精确I32/Bool、求值序/runtime trap、事务发布与迭代深块正反验收 |
| 772 | ac-scalar-ops-short-circuit | ✅（delivered，r2；独立复审needs_fix三项修复后final pass） | [772-ac-scalar-ops-short-circuit.md](../../plans/archive/772-ac-scalar-ops-short-circuit.md) | scalar-i32 profile：减/除/余（0、MIN/-1、MIN%-1 trap70）、全序比较、同类eq/ne、一元、布尔短路与条件for；身份精确可失败映射（未注册descriptor双拒bind/writer）、verify分域入口、签名/引用边界门；14步45例门禁+独立复审反例修复 |
