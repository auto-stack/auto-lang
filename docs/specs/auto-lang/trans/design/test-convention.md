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
| `tests/a2c_tests.at` | `Test.run_a2c_dir(path)` | `crates/auto-lang/test/a2c` |
| `tests/a2r_tests.at` | `Test.run_a2r_dir(path)` | `crates/auto-lang/test/a2r` + `test/cookbook` |
| `tests/a2ts_tests.at` | `Test.run_a2ts_dir(path)` | `crates/auto-lang/test/a2ts` |

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

> 来源: docs/plans/old/263-transpiler-tests.md；tests/a2c_tests.at、a2r_tests.at、a2ts_tests.at；docs/plans/archive/240-rust-cookbook-a2r-tests.md；docs/python-transpiler.md §Test Case Template
