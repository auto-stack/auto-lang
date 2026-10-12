# PLAN-767 验证取证（T-06）

- worktree `D:/autostack/.wt/lang-767/auto-lang`，branch `plan-767-dev`。
- 代码 HEAD（T-06 执行时）：`7abeb26548c9a2188b3652b3ff5cc98d7500ea20`。
- 工具链：rustc 1.98.1（48a229cea 2026-09-01，x86_64-pc-windows-msvc，LLVM
  22.1.8）/ cargo 1.98.1；Cranelift 0.126 lock 未动（Cargo.lock 无 diff）。
- 被测二进制 `auto-ac-prototype.exe`
  sha256 `16a10c151496cda90c67faf4cbb325b70e285f011f3ebb73a0d7798ac084416f`；
  trace 支持库 `ac_trace_support.lib`
  sha256 `70aeb8c8737956f2d23a3e094e5a0a8e7246cba8a9cc7ec7beb19a4998a4e124`。
- 原始收据目录：`experimental/ac-core/target/ac-verify-receipts/scalar-767/`
  （gitignored；每 case 记录 input 路径+sha256、exit、exe sha256 与完整
  stdout/stderr）。

## 一键门禁实际执行（2026-10-12）

| 门禁 | 结果 | 明细 |
|---|---|---|
| `scripts/verify-ac-scalar-767.ps1`（新增） | **14/14 步 PASS，45 例，exit 0** | check/fmt/all-targets 零 warning + scalar_hir/source/native 三族 + 11 源码正例 + 15 HIR 正例 + 19 负例 + CLI 隔离 + 收据身份 |
| `scripts/verify-ac-741.ps1 -SkipMainGates` | 全步 PASS，exit 0 | 原型 case 集不缩水；仅跳过主仓 root 档（本计划 Category A/B 不触 crates/，见下） |
| `scripts/verify-ac-source-745.ps1` | 13/13 步 PASS，74 例，exit 0 | 旧 profile 全语料零回归（含执行器故障注入控制） |
| 完整 `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked` | **143 通过 / 0 失败** | 全测试族（741/745 旧族 + 767 三新族） |
| `python scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check --require-decisions` | CHECK-OK | 47 条人工结论绑定全部新鲜；严格门闭环 |

root 档隔离说明：741 脚本**已有** `-SkipMainGates` 显式模式（跳
`cargo check -p auto-lang` / `cargo t` / `cargo tv`，原型 case 集保持不变），
本计划未改任何旧脚本（`git diff scripts/` 仅新增 verify-ac-scalar-767.ps1），
故无脚本 diff 需要记录；case 身份以各脚本内置表为准并在收据中逐例落盘。

## V-01..V-12 证据索引

| Case | 证据（全部真实执行） |
|---|---|
| V-01 | scalar_hir `core_descriptor_rejects_scalar_ops_in_text`/`bind_document_dispatches_by_declared_identity`/`verify_entries_are_domain_gated`/`write_canonical_refuses_identity_laundering`；scalar_native `cli_profile_isolation`（同一 p-cmp-matrix.at：core→source.unsupported operator `-`；scalar→成功；未知 profile exit2）；767 脚本 profile-isolation 步 |
| V-02 | scalar_native `hir_pe_divrem_table_exit0`（HIR 六行自断言 exit0）+ 源码 p-cmp-matrix（PE exit0）；HP01/SP04 |
| V-03 | scalar_native `hir_pe_trap_matrix_exit70_stderr_a`（七行 exit70+stderr 精确 `a`）与 `hir_pe_unexecuted_trap_exit3`（对照）；HP02..HP09 |
| V-04 | scalar_native `hir_pe_shortcircuit_polarity`（六极性：跳过=无 b/无 trap，执行=精确一次 b 或 trap70）；源码 p-short-circuit-trace（stderr `aaaba`）/p-short-circuit-trap（exit70）；HP10..HP15、SP09/SP10 |
| V-05 | scalar_native `source_pe_scalar_corpus` 之 p-short-circuit-contexts：let/assign/if/**条件for**/call/命名实参全上下文、循环 ≥2 轮每轮重算、trace 精确 `abababaaaabab`（exit0）；SP11 |
| V-06 | scalar_source `scalar_negative_matrix`（e01..e06：混合 bool/int、!1、-true、true\|\|unknown()、false&&(1+2) 全部静态拒绝）+ `scalar_positive_corpus_compiles`（p-cmp-matrix 含 MIN/MAX/负数全序与优先级组合） |
| V-07 | scalar_hir `mutation_*`（错类型 unary/logical/eq、共享 Expr、自环、二节点环、跨 body → verify.bad-reference；canonical 文本不改性质）+ `scalar_type_and_graph_negatives`（i02..i08） |
| V-08 | scalar_source 负例 e12..e19（缺操作数/缺右括号/未闭合调用/字面量域/一元+/跨行）与正例 p-a-minus-1/p-binop-args/p-crlf（正例真实 PE 于 scalar_native；SP05/SP07/SP08） |
| V-09 | scalar_source 四个深输入测试（16384 项逻辑链、20000 层 `!`、4096 嵌套调用+逻辑、深未闭合受控拒绝）+ scalar_native `source_pe_deep_mixed_1000`（1000 层混合真实 PE exit0） |
| V-10 | scalar_native `receipt_identity_and_transaction_preservation`（成功收据含 source-profile/hir 三重身份/input-hash；SENTINEL 三件在 source/HIR 两类拒绝构建后 hash 不变）+ 767 脚本 identity 步 |
| V-11 | scalar_native 源码 PE：p-gcd-euclid（gcd(48,18)=6 等 5 组）、p-count-digits（12345=5）、p-is-digit（48/57/47/58 正反边界，明确非原 lexer 编译）；SP01..SP03 |
| V-12 | scalar_source e07..e11（非 bool 条件/for-in/初始化子句/未闭合/while 均原 token 定位）+ p-for-loops（零次/嵌套 break-continue）+ p-gcd（`for y != 0`）真实 PE + CLI 双向隔离（SP01/SP06、767 脚本 profile-isolation 步） |


## 修复轮取证（needs_fix → 重提交，2026-10-12）

Codex 独立最终复审（[review-codex-r2.md](../../../docs/plans/attachments/767/review-codex-r2.md)）
裁定 needs_fix，三项发现按不变量修复（非仅修样例）：

| 发现 | 修复（作用域） | 永久回归 |
|---|---|---|
| P767-R1（P1）未知三重身份归为 Core、writer 可换壳 | `Descriptor::domain()` 改**精确可失败映射**（`Option<HirDomain>`，仅两个已注册三元组）；`descriptor::bind` 对未注册 descriptor 整体拒绝（新码 `bind.descriptor-unregistered`）；`write_canonical` 对未注册/错域一律 `canonical.identity-conflict`（消息标注 unregistered） | scalar_hir `unregistered_descriptor_identities_are_rejected_everywhere`（schema/revision/profile 三维 × bind+writer+改头文档+注册表控制） |
| P767-R2（P1）签名 TypeId 越界得 Checked、native panic（native.rs:91） | verify 新增**模块级签名类型边界门**（函数类型 params/result 的 `TypeUse::Ref` 先于签名求值，`verify.bad-reference`，失败跳过该模块走查）；native `value_of_type_use`/`signature_of` 全化（越界 → 受控 backend 错误）。作用域审计：TypeUse::Ref 全部入口 = types 表（新门）、defs ty（既有 verify.callee-type .get 控制）、locals/places/exprs（既有 body 边界门）——无遗漏路径 | scalar_hir `signature_type_refs_are_bounds_checked_before_checked`（params[0]/result 各 TypeId(9999) + 有效/内建控制） |
| P767-R3（P2）设计收据减法护栏公式错误 | design.md 公式改为与实现一致的 `(b>0 && a<min+b) \|\| (b<=0 && a>max+b)`（含可表示性说明）；未动正确后端 | 文档修正 + 复审探针重放 |

**复审者反例重放**（同一探针程序 review-codex-probes.rs，对修复版 rlib 重编译）：
16/16 全 PASS、`CONTRACT_FAILURES=0`、exit 0——WRITER-schema/revision/profile 三项
`canonical.identity-conflict`、UNKNOWN-schema/revision/profile 三项
`bind.descriptor-unregistered`、SIGNATURE-parameter/result 两项
`verify.bad-reference`（不再有 native panic）、8 个控制项保持 PASS。

**门禁重跑**（修复提交）：完整 ac-core 套件 **145/0**（143+2 新回归）；
verify-ac-scalar-767 **14 步 0 失败 45 例**；verify-ac-741(-SkipMainGates) 13 步
PASS；verify-ac-source-745 13 步 74 例 PASS；acc_inventory CHECK-OK；fmt clean、
all-targets 零 warning。**复审者原生探针重放**（review-codex-native.ps1，修复版
二进制）：ARITHMETIC exit=0；SHORTCIRCUIT exit=0、stderr 精确
`abaabaabababaabaaba`——与其冻结 oracle 一致。

## 已知边界与不宣称

- MIN%-1 trap70 是本 profile 固定测试语义，不推导全语言/生产异常 ABI。
- char/str/List/record/enum/完整 int 宽度/ACC 主体/代际均 open（§4.3 工作包）。
- 深输入验证到 Checked（含 Drop）与指定规模真实 PE（1000 层）；不承诺任意
  规模性能或无限内存。
