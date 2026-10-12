# AutoLang 测试静态盘点（2026-10-12）

- 主仓基线：`897eb34911a4f496a28ba7cdfd8a82815fa043d1`，分支 `v0.6-dev`。
- 范围：全部 13462 个 Git 跟踪文件；未跟踪文件、构建缓存和 node_modules 不在统计集合。
- 方法：扫描 Rust/Auto/Python/JS 测试标记与测试目录资产，逐项记录内容 SHA-256；对主要发现器、执行器、比较器与门禁配置作人工核查。
- 这是全仓静态资产与入口盘点，**不是测试执行结果，也不是每条用例都已逐条语义复审**。条件编译、宏展开、参数化循环、注释和未挂接文件会使标记数与实际可运行数不同。

- Rust：703 个带测试标记的文件，9562 个词法标记。
- 各语言合计：813 个标记入口文件；测试路径资产 4489 个（含 harness、支持文件和文档）。
- 文件型 Auto/AutoScript 源码：2012 个；完全相同字节的重复组 72 个，额外副本 84 个。相同字节不代表相同执行环境或可直接合并。

## 完整清单

- [entrypoints.tsv](entrypoints.tsv)：所有扫描命中的测试入口，标记种类、数量与 hash。
- [assets.tsv](assets.tsv)：所有目录段为 test/tests/fixtures/snapshots/corpus_* 的跟踪资产，类别与 hash。
- [identical-sources.tsv](identical-sources.tsv)：.at/.as 字节完全相同的源码组。
- [external-autodown.tsv](external-autodown.tsv)：只读核查的外仓 AutoDown 解析/块模型实现，记录内容 hash；不计入主仓总数。外仓 Git revision 未读取，不能据此宣称两仓已按版本锁定。
- 资产归类只是路径分析；其中 source 并不保证已被现有 runner 发现，expected 也不保证被正确比较。
- 内联 Rust 测试由 entrypoints 全仓扫描覆盖；没有 marker 的 fuzz/bench/外部脚本和 CI 另在设计文档说明。

## 发现规则与设计结论

[Design 35](../../design/35-autodown-test-cases.md) 记录各族的现状入口、比较语义、保留边界、AutoDown 能力与迁移建议。

按 `test_runner.rs` 当前两层目录/stem/expected 规则对跟踪文件作静态模拟（非实际 runner 执行）：

| 发现器 | 匹配 source | 满足 expected 条件 | 缺 expected |
|---|---:|---:|---:|
| VM | 504 | 494 | 10 |
| a2r | 244 | 241 | 3 |
| a2c | 125 | 118 | 7 |
| a2ts | 85 | 85 | 0 |

上述规则无法代表所有 Rust helper 的发现或所有门禁；嵌套项目、辅助 source、conformance 等另有入口。缺 expected 不直接等于应补金样：迁移时必须判定该资产是 helper、未挂接语料、smoke 还是遗漏，并逐项对账。

目录统计与清单可复核：以基线 `git ls-files -z` 为集合，按目录段 `test/tests/fixtures/snapshots/corpus_*` 选择 assets；source 后缀为 `.at/.as/.ad/.atom`，expected 按路径中的 expected/golden 或 `.snap` 分类，其他为 harness/support。Rust 标记扫描含普通 `#[test]`、tokio/async_std 测试属性；Auto 为 `#[test]`，Python 为 `def test_*`，JS/TS 为词法 `it/test(`。这些粗分类可能包含文档例子和宏注释，不替代真实 build/test discovery。

## 按路径族统计资产

| 路径族 | Source | Expected | Harness/support | 总文件 |
|---|---:|---:|---:|---:|
| `crates/a2r-actor-tests` | 0 | 0 | 1 | 1 |
| `crates/a2r-std` | 0 | 0 | 1 | 1 |
| `crates/auto` | 0 | 0 | 2 | 2 |
| `crates/auto-cache` | 0 | 0 | 1 | 1 |
| `crates/auto-gen` | 0 | 0 | 1 | 1 |
| `crates/auto-lang` | 10 | 6 | 23 | 39 |
| `crates/auto-lang-macros` | 0 | 0 | 4 | 4 |
| `crates/auto-lang/src/tests` | 0 | 1 | 192 | 193 |
| `crates/auto-lang/test/a2ark` | 19 | 19 | 1 | 39 |
| `crates/auto-lang/test/a2c` | 126 | 230 | 1 | 357 |
| `crates/auto-lang/test/a2gd` | 69 | 73 | 0 | 142 |
| `crates/auto-lang/test/a2j` | 10 | 9 | 4 | 23 |
| `crates/auto-lang/test/a2p` | 99 | 99 | 3 | 201 |
| `crates/auto-lang/test/a2r` | 286 | 286 | 10 | 582 |
| `crates/auto-lang/test/a2ts` | 85 | 85 | 0 | 170 |
| `crates/auto-lang/test/a2vue` | 16 | 18 | 0 | 34 |
| `crates/auto-lang/test/ast` | 0 | 0 | 7 | 7 |
| `crates/auto-lang/test/atom` | 0 | 0 | 1 | 1 |
| `crates/auto-lang/test/autodown` | 4 | 6 | 0 | 10 |
| `crates/auto-lang/test/composition` | 2 | 0 | 0 | 2 |
| `crates/auto-lang/test/cookbook` | 163 | 202 | 125 | 490 |
| `crates/auto-lang/test/copy_wrong.py` | 0 | 0 | 1 | 1 |
| `crates/auto-lang/test/eval` | 1 | 0 | 0 | 1 |
| `crates/auto-lang/test/ffi_dual` | 19 | 20 | 25 | 64 |
| `crates/auto-lang/test/nested_mutation` | 2 | 0 | 0 | 2 |
| `crates/auto-lang/test/parity` | 1 | 7 | 1 | 9 |
| `crates/auto-lang/test/r2a` | 0 | 37 | 37 | 74 |
| `crates/auto-lang/test/router` | 2 | 0 | 0 | 2 |
| `crates/auto-lang/test/term_mux_pumps` | 1 | 0 | 0 | 1 |
| `crates/auto-lang/test/ui` | 44 | 0 | 10 | 54 |
| `crates/auto-lang/test/vm` | 684 | 592 | 0 | 1276 |
| `crates/auto-lang/test/vm_engine_face` | 1 | 0 | 1 | 2 |
| `crates/auto-lsp` | 0 | 0 | 2 | 2 |
| `crates/auto-macros` | 0 | 0 | 4 | 4 |
| `crates/auto-man` | 4 | 1 | 7 | 12 |
| `crates/auto-playground` | 0 | 0 | 6 | 6 |
| `docs` | 8 | 0 | 0 | 8 |
| `examples` | 0 | 2 | 125 | 127 |
| `experimental/ac-core` | 184 | 0 | 11 | 195 |
| `packages` | 0 | 1 | 3 | 4 |
| `parity/libs/consumer` | 9 | 0 | 18 | 27 |
| `parity/libs/dep` | 6 | 0 | 12 | 18 |
| `parity/libs/framework` | 1 | 0 | 2 | 3 |
| `parity/libs/lang` | 8 | 0 | 8 | 16 |
| `parity/libs/python` | 21 | 0 | 22 | 43 |
| `parity/libs/rust` | 13 | 0 | 16 | 29 |
| `scripts` | 20 | 0 | 2 | 22 |
| `test` | 149 | 0 | 10 | 159 |
| `tests` | 15 | 0 | 2 | 17 |
| `website` | 0 | 0 | 11 | 11 |
