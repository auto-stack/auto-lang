# PLAN-745 verification 收据（verification.md）

> 记录执行/验证命令、期望、实际结果与代码修订的对应关系（AC-01..AC-07 work 阶段证据）。
> 执行环境：worktree `D:/autostack/.wt/lang-745/auto-lang`，分支 `plan-745-dev`，
> Windows 11 x64（rustc 1.98.1 / cargo 1.98.1 / LLVM 22.1.8）。
> 数值边界决策证据见 [numeric-boundary.md](numeric-boundary.md)（T-01）。

## 1. 提交链（执行序）

| commit | 任务 | 内容 |
|---|---|---|
| c535b8fd6 | T-01 | 数值边界核查报告 + probe-int-width/probe-char-int + MD408/409 提案 |
| 6965c0a04 | T-02 | source 子集前端（lexer/ast/parser）+ 74 fixture + source_frontend 12 测试 |
| 3fb6c96b6 | T-03 | resolve/typecheck/adapter → 真实 bind/verify → CheckedModule + source_hir 10 测试 |
| c5af427d9 | T-04 | check-source/build-source CLI + 收据同事务源码元信息 + source_cli 10 测试 |
| (本提交) | T-05 | source_native 2 测试（21 正例真实 PE）+ verify-ac-source-745.ps1 + 本报告 |

基线：v0.6-dev @ `eea7a80618d4d5e75749d666b7609a9a87925bbd`（专属树登记 HEAD）。
23 项计划输入 hash 复核零漂移（T-01 执行时）。全程未修改 `crates/**`、
根 Cargo.toml/Cargo.lock、`auto/lib/*.at`（只写 auto-lang 仓内 experimental/ac-core + scripts/docs）。

## 2. 一键脚本实测（scripts/verify-ac-source-745.ps1，2026-10-06）

执行 HEAD `c5af427d9c43f1cb4454c1c8f5705f9fc31e51e2`（T-04 提交；T-05 两个新文件
未提交状态在案，context.txt git-status-short-lines=2）。结果：**all 12 steps PASS (74 cases)**。

| 步骤 | 结果 |
|---|---|
| context（HEAD/工具链） | PASS 0.3s（收据 context.txt：rustc -Vv / cargo -V / binary+support-lib sha256） |
| cargo check --locked（ac-core） | PASS 0.1s |
| cargo fmt --check（ac-core） | PASS 0.3s |
| cargo check --all-targets 零 warning | PASS 0.2s |
| cargo test source_frontend | PASS（12/12） |
| cargo test source_hir | PASS（10/10） |
| cargo test source_native（--test-threads=1） | PASS 7.5s（2/2：21 正例逐例真实构建+运行） |
| cargo test source_cli（--test-threads=1） | PASS 1.6s（10/10） |
| 构建工具二进制 + trace 支持库（各一次） | PASS（sha256 入 context.txt） |
| **正例 21：build-source + 运行（60s 硬截止/例）** | PASS 8.4s（退出码+stderr oracle 全中，三制品齐全） |
| **负例 53：check-source 拒绝（stage/code 标记匹配）** | PASS 1.4s（全部 exit 1） |

工具二进制 sha256 `9d0d6dea70dd6911…5836750`；trace 支持库 sha256
`dac1a42b4ff1f98…e5c377e`。收据目录：
`experimental/ac-core/target/ac-verify-receipts/source-745/`（gitignored；
`positive/<id>/{case.txt,build.stdout.txt,run.stdout.txt,run.stderr.txt,app.exe,app.obj,app.ac-link.txt}`、
`negative/<id>/{case.txt,stdout.txt,stderr.txt}`、`cases.txt`、`summary.txt`、`context.txt`）。

## 3. 正例运行结果（74 例之 21，全量见 cases.txt）

| case | exit | stderr | 指纹要点 | case | exit | stderr |
|---|---|---|---|---|---|---|
| P01 | 14 | — | exe-sha256 各例独立入 case.txt | P12 | 0 | — |
| P02 | 1 | — | | P13 | **70** | —（add_i32 trap） |
| P03 | 7 | — | | P14 | **70** | —（mul_i32 trap） |
| P04 | 3 | — | | P15 | 12 | **ba**（named 求值序） |
| P05 | 1 | — | | P16 | **70** | **a**（先观察后 trap） |
| P06 | 3 | — | | P17 | 6 | — |
| P07 | 5 | — | | P18 | 4 | — |
| P08 | 3 | — | | P19 | 6 | —（用户 mark_a 胜出，无 requires） |
| P09 | 8 | — | | P20 | 0 | —（UTF8+CRLF+`- 2147483648`） |
| P10 | 5 | — | | P21 | 4 | — |
| P11 | 0 | — | | | | |

退出码 70 与普通 return 70 同码的区分以 HIR overflow 结构（P13/P14 build 成功 +
add/mul_i32 overflow:trap 字段，source_hir 结构断言）+ P16 前驱 trace（stderr 精确 `a`
先于 trap）作证，不依赖 PowerShell signed 退出码展示（plan §5.1）。

## 4. 负例拒绝矩阵（53 例，stage/code 标记全中）

- text 级 30 例：source.literal-range（N01/N02）、source.unsupported 23 例、
  source.syntax（N30/N52/N53）、source.lex（N31）；
- bind 级 13 例：source.undefined（N03/N09/N50）、source.duplicate（N08/N32/N33）、
  source.argument（N10..N13）、source.not-callable（N27/N28/N29，含 capability 提供）；
- verify 级 10 例：source.type-mismatch（N04/N05/N06/N14/N51）、verify.immutable-write（N07）、
  verify.loop-target-scope（N15/N16）、verify.no-return-path（N17）、verify.block-structure（N18）。

token/行列精确断言（fixture 固定 code、token 与行列，不锁整条 message）由
tests/source_frontend.rs（30 text 级 token+span）与 tests/source_hir.rs（23 bind/verify 级
stage+code+token 切片、N03 2:24）承担；脚本层断言 exit 1 + `error[stage/code]` 标记。

## 5. §6 补充矩阵 → 证据映射

| invariant | 证据 |
|---|---|
| 精确类型/宽度 | source_hir `p11_literal_domain_in_hir`（±2^31 字面量入 HIR）+ `negative_corpus…`（N01/N02/N22/N37/N38/N47）+ 本表 P11/P13/P14 退出码；不验完整语言宽度兼容 |
| runtime trap | source_native 正例表 P13/P14（70）、P12（未走分支不 hoist，build 成功 exit 0）、P16（mark_a 后 trap，stderr `a`）；不以 VM 行为为 oracle |
| eval_args 双射 | P15（stderr `ba`=12，bindings param0←arg1/param1←arg0 由 source_hir `p15_eval_order_and_binding_map` 结构断言）+ P17（异构命名反序=6）+ N10..N13（缺/重/未知/混用） |
| loop 重验/最近目标 | P04=3（每轮重验）、P05=1（零次）、P06=3（continue 后重验）、P07=5（break 最近嵌套）；source_native 60s 硬截止；无任意程序终止承诺 |
| scope/identity | P08 递归=3、P09 前向=8、P21 互递归=4、P10 内层遮蔽 RHS 外层=5、N09 分支逃逸拒绝、N27/N28/N29 local/param 遮蔽调用拒绝、P19 用户 mark_a 无 trace |
| 来源全覆盖 | source_frontend（P20 CRLF/中文注释 span/token、N03 2:24、N31 1:24）+ source_hir（p03 合成 bool span 指 literal token）+ adapter 条目序再锚 + §5.3 post 检查（边界/UTF8/无残留，internal 受控诊断） |
| 全文件全消费 | text 级负例含 N34（未被调用函数内的 `%` 仍拒绝）、N48/N49/N53（坏字节/块注释/空文件）；不跳过剩余输入 |
| Checked 唯一 | source_hir `tampered_atom_is_rejected`（删 let 死表达式/错类型/悬空引用/伪 checked 标记 4 变异均被真实 bind/verify 拒绝）+ 既有 hir_verify 拒绝矩阵保留通过 |
| CLI/事务 | source_cli 10 例（0/1/2 分级、漏/未知 profile=2、entry 错误、失败零制品、trace 无库链接失败、旧 check 仍仅 Atom）+ 741 SENTINEL 事务合同未动（link.rs 仅追加委托函数） |

## 6. 主仓最终门禁

按 plan §6 最终门禁串行块（cargo check -p auto-lang / cargo t / cargo tv +
verify-ac-741.ps1 -SkipMainGates）在 T-07 内部检查统一执行并回填本节。
