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

## 6. 主仓最终门禁（T-07 内部检查，2026-10-06 实测回填）

按 plan §6 最终门禁串行块执行。执行环境注记：D: 盘在门禁期间 100% 满
（os error 112 中断过一次测试构建；工作树 root target 曾用
`CARGO_TARGET_DIR=C:/tmp` 重试——该次重试把 ffi_dual oracle 构建产物写到 C:
却留下 `.parity_cache_ok` marker，后续原生运行被 marker 骗过跳过构建；已删除
4 个污染 marker 并删除 C: target，全部原生重跑）。释放工作树 root target 后
D: 恢复 ~30G。最终数字均为**原生同树执行**（无 env 覆盖）。

| 门禁 | 命令 | 结果 |
|---|---|---|
| check | `cargo check -p auto-lang` | **PASS**（2m50s；382 warning 为生产 crate 既有，非 745 引入——745 零改 crates/**） |
| 日常档 | `cargo t`（nextest 全量 --no-fail-fast） | 4939/4972 通过、33 失败，与主检出对照**完全同数同源**（27 处双方共现预存红 + 6+6 对称差 flake，见下归因） |
| 语料档 | `cargo tv`（--no-fail-fast） | 158/162 通过、4 失败 = 预存红（cookbook_vm_tests::cb_versioning_semver_{parse,latest,increment,prerelease}，两环境失败集合逐项一致） |
| 741 回归 | `verify-ac-741.ps1 -SkipMainGates` | **all 13 steps PASS**（§2） |
| 745 全量 | `verify-ac-source-745.ps1` | **all 12 steps PASS (74 cases)**（§2） |

### 日常档预存红归因（同源对照法；历史证据限度注记）

745 的 diff 完全不含 `crates/**`（git diff 逐文件核对：仅 experimental/ac-core、
scripts、docs）。为满足"预存红按同源码/环境 scoped 对照归因"，在**主检出**
跑同一全量命令并做失败集合 diff。**历史限度（r2 复审更正）**：该对照运行时主检出
已合入 plan-742（crates 相差 742 变更），并非严格同源码对照；27 处共现核心红在
pre/post-742 两种 crates 状态下逐名一致、745 diff 与全部失败域零交集支持归因，
但"严格同源逐项零新增已证明"的表述不成立——正确表述为：**在两种相近基线上
失败集合一致，且无任何失败落在 745 触面上**。后续轮次的对照同此口径。

- **双方共现 27 处**（预存红核心，与 745 无关）：a2r_std_signature_parity
  （KNOWN-DEBT 396 签名漂移，v0.6-dev ↔ auto-down master）、ffi_dep_parity/
  ffi_dual 的 017_dep_lifecycle（VM 腿 `Unknown Rust stdlib call: .drop_count`）、
  musk_vm_track p053/p054 七例、plan484/492/498/499/502/503/606/707_wait、
  desktop_protocol client_runtime、ash_leak_probe、docs_gen kitchen_sink、
  schema_drift 双 fence。
- **对称差 6+6 处**（plan437/498 area/499 crosshair·sector/707_spike/daemon vs
  plan484 smoke/492 fstr/498 legend·line/499 single-pointer/spike699）：图表
  交互/指针时序/守护进程/http 类**并行负载偶发 flake**（AGENTS 在案类：
  plan707 flake=4f123a50e、th 环境相关红归因），两环境各自随机出现，
  非确定性、非 745 触发。
- 结论：**745 引入的新增失败 = 0**（工作树 33 失败 = 27 预存核心 + 6 flake；
  主检出同为 33 失败 = 27 预存核心 + 6 flake；失败总数与预存核心完全一致，
  仅 flake 名单两环境互换）。

不修不相关旧功能来伪造全绿（plan §6）；预存红清单移交 T-08 复盘与
KNOWN-DEBT 登记参考。

### 依赖树注记

主仓构建经 `crates/auto-lang` 的 optional path 依赖
`../../../auto-down/autodown/...`（workspace 装载需 manifest 可读）。按 Plan 529
组布局在 `.wt/lang-745/` 补建只读依赖树 auto-down @ fba6563ed（detached，=
master HEAD，与 lang-742 组同基线），745 未修改 auto-down 任何文件；登记见
[worktree.md](../../plans/attachments/745/worktree.md)。

## 7. r1 needs_fix 修复轮执行记录（2026-10-06，reviewer 报告 QA-01..08 逐项）

复审报告：[review-r1/REVIEW.md](../../plans/attachments/745/review-r1/REVIEW.md)。
修复提交链（本轮）：3e3c1631f（QA-01/02/03/04/08 实现）→ d9f5499b7（QA-05/06
测试）→ 36664d9a5（共享 helper 警告清理）→ 本提交（报告修订）。全部反例在
修复轮逐条回放通过（命令与原始输出见下），并已固化为永久 fixtures/tests。

| 发现 | 修复 | 执行证据 |
|---|---|---|
| QA-01 未括起表达式跨行 | parser 以表达式首行贯穿 binary/unary：越行 operand 报 `source.syntax`@越行 token；callee 与 `(` 必须同行；括号/实参内不限制 | C01（LF）/CRLF/行尾注释三变体拒绝 @"2"（3:2）；C03 拒绝 @"("（3:1）；C02 括号控制合法；tests/source_frontend `text_stage_rejections` 三变体+c03 固化 |
| QA-02 合法函数名撞 native 符号 | adapter 将源码函数符号定为 `sf_<name>`（identity 仍在 `d_<name>` def id）；不新增源码保留名 | C05 `fn ac_start` 作被调=3、作 entry=3；C06 `fn ExitProcess`=4；source_native POSITIVE 表 + `ac_start_as_entry_runs` |
| QA-03 诊断未指到 token | 越域 local → ident span（N09 oracle 更正为 `x`，语料草案 oracle 被 §5.3/AC-04 精确性取代）；死代码块 span 钉在首个死语句；终结判定扩至 return/有效 break-continue/双分支终结/returning-body loop（Stray break 保持同块以保 loop-target-scope） | C09 锚 1:57 `let x = 2`（"let x = 2" 切片）；C10 锚 3:9 `x`；N15/N16 仍锚 "break"、N18 仍锚 "return 1"（回归确认）；UTF8+CRLF 坐标控制 `review_counterexample_coordinates`（4:2） |
| QA-04 关键词/分类缺口 | lexer 镜像生产 `all_keywords()`+`with` 为 `Reserved`（我的 15 个子集词之外），名字位一律 `source.unsupported`；`!` → unsupported@op；str/char 转义对并入同一排除 literal；真正未闭合仍 source.lex | C12 `fn type()` 拒 @"type"（1:4）；C15 `!true` 拒 @"!"（1:20）；C04 `"a\"b"` 拒 @完整 literal（1:25）；C23/C24（param/局部名关键词）、C25（`'\''`）入 text 表；N40/N46 等既有分类不回归 |
| QA-05 执行器无截止/管道堵塞 | tests/common 落 `run_deadlined`：piped+双管道并发排空+try_wait 轮询+到期 kill+wait 回收；source_cli/source_native 全路径（cargo/CLI/PE）接入；新增大输出与挂起控制 | `big_stderr_output_is_fully_drained`：20,000×'a' stderr 全量排空 exit 20000（对照复审：poll-before-drain 在 4096 字符卡死）；`hanging_exe_is_killed_at_deadline`：无状态 `loop{continue}` PE 3s 截止被杀（注：自增循环会在 2^31 触发 add_i32 trap=70 提前退出，故用无算术循环） |
| QA-06 验收 oracle 缺口 | `overflow_trap_structure_in_hir`：P13/P14 真实 Checked bundle 断言 add/mul_i32 + `overflow: Trap`（无折叠）；`tampered_bundle_is_rejected_by_real_verify`：投影后 **Bundle** 三变异（删 let→verify.eval-position / 错 expr type→verify.type-mismatch / 错 local ref→verify.type-mismatch）逐个断言输入确实改变；`source_build_never_overwrites_existing_artifacts`：source 三路 SENTINEL（源拒绝/链接失败/收据位被目录占用的 publish 回滚）三件 byte-identical | 全部为永久 gate（ac-core 99 测）；本表即 QA-06 要求的执行记录 |
| QA-07 盘点归因错误 | proposed-spec-delta.md §盘点：错误调用与"未入库"说法撤回留档；正确命令实测 CHECK-OK（47 决策新鲜） | 本轮实测输出见该节；merge 不预排 --write 重扫，MD note 追加后独立复跑+检验消费者 |
| QA-08 Checked 后流程未共用 | main.rs 抽 `finish_build`（lower→staged obj→工具链→staged link→事务发布+staged 清理合同）唯一实现，build/build_source 前端只构造 Checked/入口/收据元信息 | 双模式回归：cli.rs 11 测 + source_cli 11 测全绿（含各自事务/入口/能力路径）；§2 一键脚本重跑见 §8 |

## 8. 修复轮门禁（同树串行重跑，2026-10-06，HEAD 507025957）

| 门禁 | 结果 |
|---|---|
| ac-core `cargo check --locked --all-targets` | PASS，零 warning |
| ac-core `cargo fmt -- --check` | PASS |
| ac-core `cargo test --locked` | **99 passed / 1 ignored / 0 failed** |
| `verify-ac-741.ps1 -SkipMainGates` | **all 13 steps PASS** |
| `verify-ac-source-745.ps1` | **all 12 steps PASS (74 cases)** |
| 复审反例回放 | C01/C03/C04/C05/C06/C09/C10/C12/C15 逐条按预期（拒绝码/token/坐标 或 native 3/4/入口 3），已固化为 fixtures+永久测试 |
| `acc_inventory.py --check --require-decisions`（正确调用） | **CHECK-OK**（47 决策新鲜；QA-07 更正入 proposed-spec-delta.md） |
| `cargo check -p auto-lang` | PASS（增量 1.5s；382 生产既有 warning 不变） |
| `cargo t`（全量 --no-fail-fast） | 4939/4972 通过、33 失败：28 处与前轮双方共现预存红逐名一致；其余 5 名（state_file lock / plan437 / plan498_area / plan499_crosshair / plan643_t01）**隔离复跑 5/5 PASS** = 并行负载 flake 轮换（7.8s）；745 新增失败 = 0 |
| `cargo tv`（全量） | 158/162 通过、4 失败 = 预存 cookbook semver 族（与前轮逐名一致） |

预存红口径沿用前轮（同源对照 + 名称集合），不修不相关旧功能；已知 27–29
核心红名单在历史收据与复审 daily-results.txt 中，供 KNOWN-DEBT 登记参考。


## 9. r2 needs_fix 修复轮执行记录（2026-10-06，reviewer 报告 R2-QA-01..05 逐项）

复审报告：[review-r2/REVIEW.md](../../plans/attachments/745/review-r2/REVIEW.md)。
修复提交：ad226453b（实现+测试）→ 本提交（报告）。全部 D01..D14/E01..E02 反例
回放通过并固化为永久 fixtures/tests；上轮 QA-02/04/07/08 经复审确认清偿。

| 发现 | 修复 | 执行证据 |
|---|---|---|
| R2-QA-01 括号内换行后续算误拒 | 行纪律改为动态锚：非括号表达式的 ENTRY token 可起于任意行，其后每个 token 须与**前一已消费 token** 同行；括号/实参子式闭括号后 `last_line` 合法前移 | D01=5/D02=6/D03=1/D04=6/D05=2 native；C01/C03/D06 仍拒；D07=7（ac_start+sf_ac_start 并存） |
| R2-QA-02 助手超时返回成功+吞输出 | Rust `run_deadlined` 重写：字节级排空（无效 UTF8 → lossy 保留而非清空）+通道化收件（无任何无界 join）+单一截止覆盖执行与采集+kill 后 wait 回收；PS `Invoke-Captured` 同步重写：事件化增量采集（MessageData 传引用）+到期 taskkill /T /F 树杀+正常路径有界静止窗口排空（预算内封顶） | 控制实测：后代持管道 5s/截止 800ms → 有界返回 timed_out=true（<3s）；无效 UTF8 → PREFIX/SUFFIX 保留+FFFD 替换；PS 探针：后代持管 449ms 有界、失控进程 1240ms 树杀 TimedOut |
| R2-QA-03 两 Stray 分支被当有效终结 | `stmt_term` 组合规则修正：任一 arm 含 Stray → 整体按 Stray 处理（不分裂尾块），结构门不再先败、数据流走查恢复；纯 (Div,Div) 仍分裂 | D10 双 Stray → loop-target-scope@1:27(break)+1:42(continue)；D13 同；D08/D14 有效 break/continue 的死语句锚定保留（let x = 2） |
| R2-QA-04 两项 oracle 缺口 | ① 同类型越域 Bundle 变异入永久测试：outer/inner 同为 i32，仅改 return 的 ReadLocal 指向 inner → 真实 verify 报 `verify.uninitialized-read`（逐变异断言输入改变）；② 三件**真文件** publish 回滚：FileShare.None 持有 receipt → 备份 rename 失败 → 回滚后 exe/obj/receipt 三件 byte-identical；旧"目录占位"路径保留并明确标注为用户占位保护控制，两者分列 | tampered_bundle 第4变异；`source_publish_failure_rolls_back_three_real_files`；报告本节分列 |
| R2-QA-05 深层不闭合栈溢出 | parser 递归下降汇点（表达式 unary/expr 与 block）加 `NEST_LIMIT=200` 深度护栏：越限 → `source.syntax` "nesting deeper than 200 levels"；资源护栏而非支持集收缩（语料实测最深 ~4） | E01（16 层合法）通过；E02（4096 层不闭合，4122B）经 CLI 实测 exit 1 + source.syntax（原 -1073741571 溢栈）；深层不闭合 BLOCKS 同护栏（in-process 测试） |

D 系反例额外确认：D11 转义 char、D12 关键字命名实参分类正确（r1 轮已修，固化
为 fixtures）；D09（return/break 混合臂）按 verify 保守规则接受为调查控制，
不新增验收要求。

## 10. r2 修复轮门禁（同树串行重跑，2026-10-06）

| 门禁 | 结果 |
|---|---|
| ac-core `cargo check --locked --all-targets` | PASS，零 warning |
| ac-core `cargo fmt -- --check` | PASS |
| ac-core `cargo test --locked` | **104 passed / 1 ignored / 0 failed**（含 r2 新增 5 项：深度护栏×2、执行器控制×2、同类型越域、三件回滚等） |
| `verify-ac-741.ps1 -SkipMainGates` | **all 13 steps PASS** |
| `verify-ac-source-745.ps1`（重写后 Invoke-Captured） | **all 12 steps PASS (74 cases)**（P15 stderr 精确 oracle "ba" 保持） |
| `acc_inventory.py --check --require-decisions`（正确调用） | **CHECK-OK** |
| `cargo check -p auto-lang` | PASS（增量 1.4s；382 生产既有 warning 不变） |
| `cargo t`（全量 --no-fail-fast） | 4936/4972 通过、36 失败：29 处与 r1 双方共现预存红逐名一致；其余 7 名（plan437/498_area/499_crosshair·donut/707_sse/716_supply/rqhost daemon）**隔离复跑 7/7 PASS** = 并行负载 flake 轮换（同族名单跨轮轮转，r1 轮即有先例）；745 新增失败 = 0（口径见 §6 历史限度注记） |
| `cargo tv`（全量） | 158/162 通过、4 失败 = 预存 cookbook semver 族（逐名一致） |

r2 修复轮提交链：ad226453b（QA-01/02/03/05 + oracle 缺口实现与测试）→
本提交（报告）。修复差异对 `crates/`、生产 Cargo/lock、`.cargo/.config` 保持
为零。


## 11. r3 needs_fix 修复轮执行记录（2026-10-07，R3-QA-01/02 逐项）

复审报告：[review-r3/REVIEW.md](../../plans/attachments/745/review-r3/REVIEW.md)。
修复提交：072f345f9（执行器复用+迭代化+语法级护栏）→ 879..（收敛断言）→ 本提交（报告）。

| 发现 | 修复 | 执行证据 |
|---|---|---|
| R3-QA-01 执行器生命周期 | Rust：删除 tests/common 自制助手，**直接委托 link.rs 741 执行器**（新公开 `run_deadlined_split`，旧 `run_with_deadline` 委托合并语义零变化）——受控挂起启动、Job 子树约束先于首指令、单截止覆盖执行+EOF 采集、全出口回收树与读线程；PS：脚本内嵌 C# `AcContainedRunner`（同合同：CreateProcess+CREATE_SUSPENDED→Job→Resume、字节排空 lossy、超时 TerminateJobObject 树杀、finally 全回收），`Invoke-Captured` 成薄封装 | Rust 控制全过：后代持管道 800ms 有界 timed_out 且 **0 幸存**；迟到写后代（预算内）全量输出含 LATE；失控父+后代树杀 0 幸存；无效 UTF8 前后缀保留；PS 探针 A/B/C/D 全过（B=913ms timedOut/0 幸存、C=含 LATE、D=树杀）；745 脚本 74 例全过（P15 stderr 字节精确）；幸存断言改有界收敛以容并行同族 |
| R3-QA-02 深输入护栏两缺陷 | ① 16384 合法加法链崩溃：根因=resolve/typecheck/adapter 的 Binary 脊柱递归 + AST/ResolvedExpr 递归 Drop；全部改**显式栈迭代**（resolve/typecheck/plan_expr）+ 迭代 Drop；verify 的 `expr_type`/`walk_value_in`/`dfs_cycles` 与 native 的 `eval_expr` 同改迭代（语义/错误顺序/求值顺序保持）；② 护栏误拒 99 层合法括号：计数改**语法级**（括号组/实参表/块各 1 层，实现帧不再计入），200 层护栏=语法层 | F01/F03/F04/F06/F08 check 全 exit 0（含 32794B/16384 项链与 99/100 层括号）；F02/F07 缺 RHS=受控 syntax；F09=bool 链 type-mismatch；F05/F10 深块=nesting 受控；**native**：300 项链 build+run 退出码=精确和 300（shell 256 截断现象在案）；65536 项链 check 全阶段通过（tests/deep_regression.rs 永久化，含边界两侧：99 合法过/4096 不闭合拒） |

注：F08 系列此前"4096 通过/16384 崩溃"的对比在 slot-bug 修正后重测：前端各阶段真迭代后
16384 与 65536 均过；`verify`/native 的迭代化使崩溃类在合同内输入上关闭。

## 12. r3 修复轮门禁（同树串行，2026-10-07）

| 门禁 | 结果 |
|---|---|
| ac-core `cargo check --locked --all-targets` | PASS，零 warning |
| ac-core `cargo fmt -- --check` | PASS |
| ac-core `cargo test --locked` | **112 passed / 1 ignored / 0 failed**（新增 deep_regression 5、执行器控制 3、CLI 深链 1、f04/f11 语料） |
| `verify-ac-741.ps1 -SkipMainGates` | **all 13 steps PASS** |
| `verify-ac-source-745.ps1`（C# 受控执行器版） | **all 12 steps PASS (74 cases)** |
| `acc_inventory.py --check --require-decisions` | **CHECK-OK** |
| `cargo check -p auto-lang` | PASS |
| `cargo t`（全量 --no-fail-fast） | 4939/4972 通过、33 失败：28 与历史共现预存红逐名一致；其余 5 名隔离复跑 **5/5 PASS**（并行 flake 轮换）；745 新增失败 = 0 |
| `cargo tv`（全量） | 158/162、4 预存 cookbook semver（逐名一致） |


## 13. r4 needs_fix 修复轮执行记录（2026-10-07，R4-QA-01/02 逐项）

复审报告：[review-r4/REVIEW.md](../../plans/attachments/745/review-r4/REVIEW.md)。

| 发现 | 修复 | 执行证据 |
|---|---|---|
| R4-QA-01 PS 执行器错误出口 | ① 约束失败（Job 创建/分配失败）：挂起子进程**未入 Job**——新增 TerminateProcess+WaitForSingleObject 直接终止回收（resume 失败同样兜底）；② ReadFile false 不再一律当 EOF：broken-pipe/handle-EOF=正常结束，其余记 ReadError 并在成功出口传播为 Error（不伪造截断成功）；③ RunResult 增 Pid/读错误字段；RunControl 注入口（741 可注入工厂先例）驱动真实 Win32 失败 | 探针五连：valid=READ_CONTROL 全文；assignFail（真 AssignProcessToJobObject 失败）→ Error + **pid 存活=False**（无泄漏）；readFail（真 ReadFile 失败，Win32 错误码 6）→ Error 非 code0 成功；holder/late 回归不退化；脚本新增永久步骤 "executor failure-exit controls" |
| R4-QA-02 200 层合法括号被拒 | 括号**零上限**：连续 `(` 塌缩循环消费 + 核表达式单次解析 + 部分 `)` 闭合后以既有 lhs **续算**（`((1+2)*3)` 混合嵌套正确）——纯分组零 AST 节点零递归，不闭合输入深度无关地在缺 `)` 处受控失败；调用/实参表与块保持资源护栏但**按实测重设为 1000**（每层真实递归：parser+resolve+emit+verify+native；999 层全阶段含 native 通过，1001/1200 层与不闭合调用链受控拒） | G01=199/G02=200 过；5000/50000/**100000** 层括号 check 全过；mix1..4 混合嵌套 check+native 全对（9/6）；call 300/999 过、1001 拒；if 300/999 过、1200 拒；999 层调用/if **build+run native 正确**；deep_regression 永久化两侧 + g02-parens-200 fixture |

## 14. r4 修复轮门禁（同树串行，2026-10-07）

| 门禁 | 结果 |
|---|---|
| ac-core check --all-targets / fmt | PASS，零 warning |
| ac-core `cargo test --locked` | **113 passed / 1 ignored / 0 failed** |
| `verify-ac-741.ps1 -SkipMainGates` | **all 13 steps PASS** |
| `verify-ac-source-745.ps1`（+失败出口步骤） | **all 13 steps PASS (74 cases)** |
| `acc_inventory.py --check --require-decisions` | **CHECK-OK** |
| `cargo check -p auto-lang` | PASS |
| `cargo t`（全量 --no-fail-fast） | 4942/4972 通过、30 失败：27 历史共现预存 + 3 名隔离复跑 **3/3 PASS**（plan498_area/plan499 crosshair·donut 轮换 flake）；745 新增失败 = 0 |
| `cargo tv`（全量） | 158/162、4 预存 cookbook semver（逐名一致） |


## 15. r5 needs_fix 修复轮执行记录（2026-10-07，R5-QA-01/02）

复审报告：[review-r5/REVIEW.md](../../plans/attachments/745/review-r5/REVIEW.md)。

| 发现 | 修复 | 执行证据 |
|---|---|---|
| R5-① 非连续括号栈溢出 | 表达式解析**整体重写为迭代 shunting-yard**（值栈+算符栈+括号/调用帧；`-lit`/bool/ident/命名实参/行纪律语义保持：非括号上下文算符同行、RHS 随算符行、括号内换行空白、`f
(` 非调用）；resolve 拆 `resolve_call` 并入 Enter/CallOut 迭代走查；adapter plan 加 CallOut；`ast::Expr` Drop 扩展 Call args 迭代泄流 | H01（4096 层 `1+(` 交错完整）check exit0；H02（未闭合）受控 `expected )`；100000 层纯括号、20000 层调用嵌套全编译；999 层调用 native build+run=1；调试期三个回归（空参 fall-through、`)` 前归约、命名实参 span）被语料套件当场捕获修复 |
| R5-② 支持边界未批准 | **表达式与调用嵌套：零上限达成**（上表）；剩余=语句块嵌套（if/while/loop 体）的 1000 层资源护栏 | 块路径剩余递归=verify `walk_block`（数据流 Flow/Init 合并）与 native `emit_block`（cranelift 块切换）——二者为 741 共享组件的最深逻辑，本轮上下文不足以安全完成其迭代化改造+全量重验；每层真实递归（parser+resolve+emit+verify+native），999 层全阶段含 native 通过、1200 层受控拒 |

## 16. r5 修复轮门禁（同树串行，2026-10-07）

| 门禁 | 结果 |
|---|---|
| ac-core check/fmt | PASS，零 warning |
| ac-core `cargo test --locked` | **113 passed / 1 ignored / 0 failed** |
| `verify-ac-741.ps1 -SkipMainGates` | **all 13 steps PASS** |
| `verify-ac-source-745.ps1` | **all 13 steps PASS (74 cases)** |
| `acc_inventory.py --check --require-decisions` | **CHECK-OK** |
| `cargo check -p auto-lang` | PASS |
| `cargo t`（全量） | 4944/4972、28 失败：25 历史共现 + 3 名隔离复跑 **3/3 PASS**（plan499_donut/plan502_hover/rqhost 轮换）；745 新增失败 = 0 |
| `cargo tv`（全量） | 158/162、4 预存 cookbook semver |

r5 提交链：d1943418b（迭代解析器+调用迭代化）→ 本提交（报告）。
