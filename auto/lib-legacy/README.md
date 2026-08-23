# AAVM v1(legacy)归档

Plan 431 E1:旧自举库(13 个 `.at` 文件)移入本目录归档。

## 保留原因(plan-431 背景/已确认的决策)

1. **108 个 bootstrap 用例的 `.expected.out` 断言语料**——`test/vm/99_bootstrap/`
   的期望输出与这些文件的行为绑定,是 v2 移植一致性判据的可回收语料;
2. **a2r 类型映射规则**——v1 里 Auto↔Rust 类型对应关系的现成实现;
3. **opcode 编号兼容性**——v2 保持编号一致(v0.5 基线 194 条,见
   `docs/specs/aavm/porting-boundary.md`)。

## 现状

- 99_bootstrap 测试的 `AUTO_LIB_FILES` 已指向本目录(`auto/lib-legacy/`),
  归档后测试照常可跑(`#[ignore]`,手动 `cargo test -- --ignored`);
- v2 代码由 Plan 432 逐模块写入新的 `auto/lib/`(纯 Rust 模式,语句级移植,
  参照 `docs/specs/aavm/` 下四份规范);
- v2 全量落成并四向对比通过后(Plan 433),本目录可再评估是否移出仓库。
