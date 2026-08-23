# plan-429 B2: 性能摸底报告（VM 解释执行 vs a2r 原生）

- 日期：2026-08-23；环境：Windows/Git Bash，dev profile（auto.exe debug 构建）
- 基准：`bench.at`（use.rust Vec，10 万次 push + len）/ `bench_stdlib.at`（Auto stdlib List 同逻辑）
- 产物：本目录 bench.at / bench_stdlib.at / bench.a2r.rs（转译产物）/ report.md

## 测量结果

| 路径 | 总耗时（3 次中值） | 剥离启动（VM ~100ms / 原生 ~24ms）后净循环 |
|---|---|---|
| VM + use.rust Vec（dispatch 3000） | ~795ms | ~695ms ≈ **7.0µs/次迭代**（push+计数） |
| VM + Auto stdlib List（LIST_* opcode） | ~685ms | ~585ms ≈ **5.9µs/次迭代** |
| a2r 转译 + rustc dev | ~24ms | 循环本身 <1ms（总耗时≈进程启动） |
| a2r 转译 + rustc -O | ~25ms | 同上 |

- dispatch 3000 与内建 opcode 的差距很小（~1µs/次），**瓶颈是 VM 解释执行本身，不是 shim 分发**。
- 原生路径下 10 万次循环低于测量分辨率 → 单操作级比值约 **≥300x**（dev-vs-dev 口径）。

## 意外发现：use.rust Vec 在 VM 模式下是坏的（正确性 bug，非性能问题）

`bench.at` 在 VM 模式输出 `0`（期望 100000）。根因链（vm/ffi/stdlib.rs）：

1. `("Vec","new")` 臂（:7326）把对象类型名写成 **"heapless::Vec"**（复制粘贴自 heapless stub）；
2. `v.push(i)` 按 receiver 类型名命中 `("heapless::Vec","push")` stub 臂（:7942）——**int 被静默截断为 u8**；
3. `v.len()` 的臂（:7522）只匹配 `"Vec"/"Vec<u8>"/"buf"`，不含 `"heapless::Vec"` → 落入兜底**恒压 0**。

即 `use.rust std::vec::Vec` 在 VM 路径完全不可用。这正是手写 shim 层的典型质量病
（类型名三处不一致、截断无声），为 plan-430 生成式 shim 的必要性提供了最直接的实证。
**已列为 430-E 的首个修复项**（生成正确一致的 Vec shim 后该 bug 自然消失）。

## 对 AAVM corpus 预算的建议

1. 单 AutoVM 指令 ~µs 级：AAVM-in-VM = "VM 解释一个编译器"，每个 AAVM 层操作再放大百倍。
   **helloworld/fib 级用例可行**（预估秒-分钟级），稍大 corpus 应只走 AAVM-Rust 路径（plan-433）。
2. VM 模式跑 AAVM 必须用 `--release` 的 auto CLI（本摸底是 dev profile，release 预计好 3-10x，
   430 期间可复测）。
3. corpus 预算建议：单用例 VM 解释执行上限先定 **60s**，超限语料标记 `vm-skip`（431-D2 落地细则）。

## 局限

- 位运算/HashMap 迭代等更重路径未测（shim 缺失，见 B1 报告，待 430-E 后复测）；
- dev profile 比值偏悲观，仅作数量级参考。
