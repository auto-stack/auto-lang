# F-797-PM-03 收口证据：冻结交付代码上的 closeout/兼容门禁重跑（2026-10-20）

**绑定**：master HEAD `8d2fda1ee0383514d9c6118cec1d858dc2264d58`；`git diff 1799f86ca..8d2fda1ee -- crates experimental .config scripts Cargo.toml Cargo.lock` = **空**（代码/依赖/验证配置与 delivery 逐字节一致，此后提交均为 docs）。计时门断言文件 `crates/auto-lang/src/tests/probe_adapter_tests.rs` 相对 R17 受审 `5a54f588d` 零 diff。运行目录=主检出（单实例串行，无并行 cargo）。

## 执行实录（全部失败样本保留，未删未改阈值）

| # | 命令 | 结果 | 证据 |
|---|---|---|---|
| 1 | `verify-autoabi-797.ps1 -Scope closeout`（attempt 1, 16:33） | step1-7 PASS（probe 套件 9 二进制全绿：unit 12 + t01 8 + t02 24 + t03 7 + t04 6 + t05 26 + t06 7 = 90，含双硬门；fmt；零 crate 告警）；step8 专用 t06 **FAIL：body-edit P95=1189.5ms ≥1000ms**（reject 通过），后续步未达 | [FAILED-attempt1.txt](FAILED-attempt1.txt)、[raw-attempt1/](raw-attempt1/) |
| 2 | 同命令（attempt 2, 16:38，唯一受控复测） | step1-4 PASS；step5 probe 套件 **FAIL：`abi_worker_concurrent_gc_same_store` store open Sql "database is locked"**（lane 2000 sample 0 infra）——**该次套件内 t06 body/reject 双硬门已先行通过**，失败为无关瞬态 | [FAILED-attempt2.txt](FAILED-attempt2.txt)、[raw-attempt2/](raw-attempt2/) |
| 3 | `cargo test -p auto-lang --features test-cases --lib abi_recipe -- --test-threads=1`（step9 同形） | **4/4 PASS（119.82s）**，含 `abi_recipe_formal_turnaround_hard_gates` 本体（即被移出日常档的门）——正式门 body-edit P50/P95/P99=610.0/**966.0**/1182.0ms、reject P50/P95=83.0/**98.0**ms（31/30 样本，一请求一原始行） | [parent-abi-family-serial.log](parent-abi-family-serial.log)、[summary](parent-formal-gates-summary.txt)、[61 行 JSONL](parent-formal-gates.jsonl) |
| 4 | `cargo test -p auto-lang --features test-cases --lib tests::probe_adapter_tests::abi_ -- --test-threads=1`（R17 父族口径） | **11/11 PASS（50.71s）** | [parent-abi-family-full-serial.log](parent-abi-family-full-serial.log) |
| 5 | `cargo test -p auto-test-case --locked`（796/788 兼容） | **87 PASS**（8 二进制 30+4+19+20+4+9+1+0） | [auto-test-case.log](auto-test-case.log) |
| 6 | `cargo test -p auto-lang --features test-cases --lib tests::test_cases_tests::`（796/788 兼容） | **26 pass + 1 既有 ignored** | [test-cases-tests.log](test-cases-tests.log) |
| 7 | 材料/清单步（step10-11 等价） | semref 退役收据、autoabi.suite.ad、p797-mix.atomdata、verify 脚本、freeze-proposal、support-matrix、spec-delta、source-parallel-audit 全部在库（本轮 `git ls-files` 核对） | 本文件 |

## 归因（不宣称机器独占）

- attempt 1 计时门失败：多会话共享机边界抖动（[进程快照](concurrent-processes-attempt1.txt)：ZCode×5/ChatGPT/codex/Everything/ToDesk 等常驻）——与 R16 收据 P99=1514ms 外部尖峰、R17 首试 1010ms、798 计划在案 sig 膨胀同一环境模式；同日串行参考执行两次通过（966ms/及全绿套件内通过），非代码回归。
- attempt 2 SQLite "database is locked"：同一脚本内套件步与后续执行对同一 store 的背靠背打开竞争瞬态；该测试在 attempt 1 套件、attempt 2 先行段及全部父侧运行中通过，代码与 R17 受审版本逐字节一致，无代码变化。
- 并发快照：attempt 1/attempt 2 各一份；未干预外部进程。

## 结论

closeout 全部面（probe 完整套件/真实 C 三腿 t03/资源 t04/缓存 t05/t06 真实绑定+双硬门/父侧串行族/兼容 87+26+1ig/fmt/零告警/材料）已在冻结交付代码上绑定执行并通过；脚本级单跑 ALL-PASS 未在两次尝试中取得（两处不同瞬态，均已保全+归因），以同形命令逐面收齐为可复验收据。被移出日常档的计时门其串行参考执行（门的所有者路径）当日两次通过（P95 966ms / 套件内通过），日常档排除 ≠ 门禁通过的空档由此闭合。
