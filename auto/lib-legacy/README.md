# AAVM v1(legacy)归档

Plan 431 E1:旧自举库(13 个 `.at` 文件)移入本目录归档。

## 保留原因(plan-431 背景/已确认的决策)

1. **108 个 bootstrap 用例的 `.expected.out` 断言语料**——`test/vm/99_bootstrap/`
   的期望输出与这些文件的行为绑定,是 v2 移植一致性判据的可回收语料;
2. **a2r 类型映射规则**——v1 里 Auto↔Rust 类型对应关系的现成实现;
3. **opcode 编号兼容性**——v2 保持编号一致(v0.5 基线 194 条,见
   `docs/specs/aavm/design/porting-boundary.md`)。

## 封存状态(plan 432 收尾,2026-08-24)

**本目录已正式封存,不再演进。v2 主体在 `auto/lib/`(唯一事实源
`AUTO_LIB_FILES_V2`,lib.rs)。**

- v2 六层(token/lexer/parser/typeinfo/codegen/engine)已全部落地并在
  AutoVM 内自举:主里程碑 M3 达成(helloworld + fib(10)=55 全管线),
  M1-M5 六道闸门全绿(30 语料字节码结构 + 行为双一致),M4 扩语料已
  回收 99_bootstrap 038-052 语料(见 `test/vm/aavm2/corpus_m4/`
  b11-b26 与 divergences.md);
- 99_bootstrap 测试的 `AUTO_LIB_FILES` 仍指向本目录(`#[ignore]`,
  手动 `cargo test -- --ignored`)——这是**有意保留的 v1 归档消费路径**,
  其 `.expected.out` 金样与 v1 行为绑定,是可回收语料的事实来源;
- 除 99_bootstrap runner(vm_file_tests.rs)与 lib.rs 的 v1 清单外,
  无其他测试引用本目录;**新工作一律进 `auto/lib/` v2,不得修改本目录**;
- **快照(2026-08-24,master a04ec045 + 432 M4-corpus 切片)**:99_bootstrap
  `--ignored` 全跑 137 例 = 101 过 / 36 败——36 个既有失败为 master 存量
  (两分支实测失败集逐名一致),与本目录封存无关,留档供后续变更对比;
- v2 全量落成并四向对比通过后(Plan 433),本目录可再评估是否移出仓库。
