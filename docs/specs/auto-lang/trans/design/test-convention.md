# 转译测试约定（tests/a2*_tests.at）

> 范围：转译器测试的目录约定、发现机制与比对方式（plan-263 定型）。

## 约定

- 每个后端一个用例目录：`crates/auto-lang/test/a2{c,r,ts,p,j,gd}/`，
  每个用例一个子目录，内含 `<name>.at` 输入与 `<name>.expected.<ext>` 期望输出。
- 比对是**逐字节**的：转译输出必须与 `.expected.*` 完全一致。
  推论：任何不写入输出的信息（如逃逸分析 warning）都不能泄漏进字节流。
- 测试声明在 `tests/` 下的 `.at` 文件里，用 `#[test]` 函数调 FFI 目录 runner：

| 声明文件 | FFI | 扫描目录 |
|----------|-----|----------|
| `test/runners/a2c_tests.at` | `Test.run_a2c_dir(path)` | `crates/auto-lang/test/a2c` |
| `test/runners/a2r_tests.at` | `Test.run_a2r_dir(path)` | `crates/auto-lang/test/a2r` + `test/cookbook` |
| `test/runners/a2ts_tests.at` | `Test.run_a2ts_dir(path)` | `crates/auto-lang/test/a2ts` |

由 `auto test` 统一执行（与 VM 测试同一机制）。

## 新增用例流程

1. 在对应 `test/a2*/` 下建目录，写 `<name>.at`。
2. 跑测试，runner 生成 `.wrong.*` 输出。
3. 人工确认后把 `.wrong.*` 改名为 `.expected.*`。

## 不变量

- 加用例不改任何 Rust 代码（plan-263 废掉了 ~420 个逐用例 `#[test]` 样板）。
- a2r 的 cookbook 集（163 个 .at，plan-240）是 assert 驱动的行为测试，与 .expected.rs 字节比对并存。

## 显式非目标

- 不在 runner 层做模糊比对/格式化归一——字节差异即失败。
- a2p/a2j/a2gd 目前主要靠后端文件内 inline `#[test]`（cargo test -p auto-lang），
  尚未全部迁入 `tests/*.at` 声明式。

> 来源: docs/plans/archive/263-transpiler-tests.md；test/runners/a2c_tests.at、a2r_tests.at、a2ts_tests.at；docs/plans/archive/240-rust-cookbook-a2r-tests.md；docs/python-transpiler.md §Test Case Template

## 双轨只读 pilot（TestCase 协议，Plan 773 / Design 35 A）

旧目录机制保持不动；新增**引用式**第二轨：`crates/auto-lang/test/cases/pilot/`
的 `.case.ad/.suite.ad` 合同原位引用旧语料（fixture_root 指向 `test/`，零搬动、
零 wrapper 修改），由 `auto-case` CLI（`--features test-cases`，无默认 feature
可无窗口跑）经 `src/test_cases/` adapter 执行 a2r/emit 检查。

- **emit ≠ compile**：pilot 的 a2r 检查只判发射产物 bytes（逐字节对拍金样，
  与本文件"字节差异即失败"同律）；Rust 真编译/运行是 Design 35 B，能力表显式
  缺口，不得宣称编译覆盖。
- 只读与产物纪律：金样/源/旧目录永不被写（新失败产物与候选基准只写隔离
  out 目录）；`.wrong.*` 流程在 pilot 轨不适用——候选基准走 proposal-only
  （旧新 sha256 + diff + 共享消费者清单，accept 是独立人工动作）。
- 对账：`cases/pilot/legacy-map.tsv`（16 列）逐行登记旧 source/oracle 内容哈希、
  入口/工程上下文与新 case/check 的对应，测试锁定哈希与报告集合一致。
- 档位：`cargo tt` 经 `a2r_tests::pilot_aggregate_a2r` 聚合入口触达 pilot 的
  a2r 半部（21 case/24 check 全集见 test_cases_tests 家族）。
- 新 oracle（pilot 新增 emit 金样，`cases/pilot/oracle/`）与旧 `.expected.rs`
  分域存放；旧金样被引用时原位不动。

> 来源: docs/plans/773-autodown-testcase-protocol-pilot.md（Design 35 A 阶段，
> R1 修复轮后）；协议合同见 docs/specs/auto-test-case/。
