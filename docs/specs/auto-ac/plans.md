# auto-ac — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 741 | ac-hir-native-core | ✅（delivered，r3；r1..r3均已交付） | [archive/741-ac-hir-native-core.md](../../plans/archive/741-ac-hir-native-core.md) | 首个原生 AOT 闭环：Cranelift 0.126.2 → COFF → rust-lld/MSVC 双链路 → PE 真实执行（add=5/1、count=3/0/0、溢出 trap=70、调用顺序 b→a 结果 12）；capability 显式门（hir.test.trace.v1 + no_std 支持库）；ac-probe check/build（退出码 0/1/2、原子制品替换不覆盖、收据）；工具链发现（KitsRoot10 注册表/env 覆盖）收据 toolchain.md；verify-ac-741.ps1 一键复现 |
