# PLAN-745 r1 最终独立复审（2026-10-06）

**outcome: needs_fix**。当前实现符合主要流水线方向，但有合同内缺陷和验收漏项；不能合入或沉淀为完整 source profile 能力。

stage=review / review_scope=final / plan_revision=1。复审者为原规划 Codex 对话，实现来自独立 GLM 上下文；本上下文未实施产品代码，未委派审查，未修复或合入。本次不改变语义合同或 revision。

## 基线和复现

- 唯一工作树：`D:/autostack/.wt/lang-745/auto-lang`，`plan-745-dev`。
- reviewed_commit：`b3cd8171966281a42f96187780342ce0ff51c5fb`。
- base_commit：`eea7a80618d4d5e75749d666b7609a9a87925bbd`。
- 入口主检出：`v0.6-dev @ 0bc1bea0116140fd0ae29debf3d182bb9f5749ea`。
- 只读组内依赖 auto-down：`fba6563ed2148ce85e68208863159b4ccccac710`。
- Plan 原文 hash、四项 canonical Spec hash、冻结 proposed delta hash：[baseline.json](baseline.json)。冻结增量：[proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md)。二进制、support lib 和 Rust 指纹：[toolchain.txt](toolchain.txt)。
- 反例重放：在主检出运行 `./docs/plans/attachments/745/review-r1/probes.ps1` 和 `./docs/plans/attachments/745/review-r1/supplemental.ps1`，使用上述工作树已构建的 CLI/support lib。脚本只生成复审制品，不修改实现。检查/build-source 均带显式 profile；每个进程有有限等待，输出并发采集。
- 原始结果：[counterexamples.json](counterexamples.json)、[probes.log](probes.log)、[supplemental-results.json](supplemental-results.json)、[supplemental.log](supplemental.log)。各 Cxx 原始源码与结果保存在 `probes/<id>/`。C13/C21 是调查控制，不作为缺陷；不以含糊规则新增验收要求。

## 发现与返工范围

### P745-R1-QA-01 / P2：未括起的表达式仍跨换行

原合同缺口，AC-02，T-02/T-05。`src/source/parser.rs:567–602` 只在看见后续运算符时检查行号，递归解析 RHS 时直接消费下一行；`:698–701` 判断调用不检查 callee 与 `(` 之间的换行。

C01 的 `return 1 +\n 2`、C03 的 `return f\n(3)` 实际 `check-source=0`。§5.2 要求未括起表达式不能跨语句换行。C02 的 `return (1 +\n 2)` 是合法控制，实际通过。

修复：在整个非括号表达式路径维持换行边界，含运算符后 RHS、callee 到开括号；不要限制括号内合法换行。永久测试加入 LF/CRLF、注释、正反成对输入，固定拒绝 code/token/坐标。

### P745-R1-QA-02 / P2：合法源码函数名与 native 内部符号冲突

原合同缺口，AC-02/05/06，T-03/T-04/T-05。`src/native.rs:235–243` 引入 `ExitProcess`，另有 `ac_start` startup；用户函数同名直接进入对象符号表。native 代码来自既有基线，但本计划承诺的源码标识符集合没有排除这两个名字。

C05 定义 `fn ac_start() int { return 3 }`，C06 定义 `fn ExitProcess() int { return 4 }`，都经真实 bind/verify 检查通过，`build-source` 却返回 1 / `backend.lowering` / incompatible signature，无法运行得到 3/4。

修复：用户函数、内部 startup/import 的 native 名字空间必须隔离，同时保持源码 entry 按函数名、前向/递归/调用绑定和 HIR 入口兼容。不得静默新增源码保留名来收缩当前合同。正例至少覆盖这两名作为被调用函数和入口，旧 HIR/trace/native 验收重跑。

### P745-R1-QA-03 / P2：部分原源码诊断没有指到出错 token

原合同缺口，AC-04，T-03/T-05。`src/source/resolve.rs:524–530` 对已离开作用域的 local 故意使用整个 `stmt_span`；`src/source/adapter.rs:215–230` 的块 span 不能精确指出双分支终结后的非法后续语句。

C10 应指 line 3 column 9 的 `x`，实际指 line 3 column 2 的 `return`；C09 两分支都 return 后还有 `let x = 2`，实际 `verify.block-structure` 指 1:17 的合法 `if`，非法 `let` 在 1:57。拒绝本身正确，位置错误。N09 在已批准语料中把 token 写成 `return x`，这项 oracle 也与 §5.3/AC-04 的精确 token 要求不一致，现有测试固化了缺陷。

修复：引用诊断保留 Ident span；终结后的语句须以非法语句 token 锚定诊断，覆盖 return、break/continue 和双分支终结。保持真实整体 verify，不以伪来源代替。更新相关 fixture oracle，补 UTF8 前缀及 CRLF 的精准坐标控制。

### P745-R1-QA-04 / P2：排除边界的关键字与诊断分类有缺口

原合同缺口，AC-02/04，T-02/T-05。

- C12：`fn type() int { return 1 } ... return type()` 被接受。`src/source/lexer.rs:172–190` 只保留 profile 使用的少量关键词，`type/enum/use/mut` 等 Auto 关键词可以成为名字；生产 `crates/auto-lang/src/token.rs:356` 有明确关键词表。§5.2 不允许关键词作名字；拒绝关键词不等于支持其语法。
- C15：已能词法识别的 `!true` 返回 `source.syntax`，应按排除运算返回 `source.unsupported`。对照 C16 的 `||` 正确分类。
- C04：合法但排除的字符串 `"a\"b"` 返回 `source.lex` / unterminated，且指到后面的引号；`lexer.rs:302–339` 未处理转义引号。应识别完整排除 literal 并在其位置报 `source.unsupported`；真正未闭合 literal 仍属于 `source.lex`。

修复：保留完整 Auto 关键词集合，区分识别但排除与真正坏 token/坏语法；补声明/参数/局部/命名实参中的关键词及转义 str/char、逻辑 not 的正反分类控制。无需支持排除类型的求值。

### P745-R1-QA-05 / P2：新增测试执行器重新引入无截止等待和管道堵塞

原合同缺口，AC-05/06/08，T-05/T-07。`tests/source_cli.rs:21–25,56–59` 对 CLI 和 PE 直接 `Command::output()`，没有计划要求的 60s 硬截止；`tests/source_native.rs:51–57` 的 support cargo build 同样无有限截止。`:78–90` 先 poll 子进程退出才收集 piped 输出，采集也不受 deadline 约束，超时只 kill 不 wait/回收。§6 明确禁止把无限 `Command::output` 当合格执行，要求继承 741 的进程/输出/回收约束。

实测控制：相同合法 20,000 字符 trace PE，用并发采集助手在 **93ms** 返回 0，stderr 长度 20000；复刻 `run_deadlined` 的 poll-before-drain 顺序，在缩短为 **2s** 的复审截止仍未退出，只读到 4096 字符，必须杀进程。此为同顺序的缩短反例，不冒称运行了原测试的 60s 超时。缓冲区满时 OS 不会自动替调用者排空 pipe。

修复：所有新增测试/script 的 cargo、support、CLI、PE 路径统一有限截止，并发排空输出，截止涵盖采集及后代管道，所有出口等待并回收自有进程。增加大输出、进程迟滞/后代持管道的控制；复用或等价遵循现有受控执行器。不能只改注释或把 deadline 调大。

### P745-R1-QA-06 / P2：补充矩阵有未实际实施的验收 oracle

原合同中的验证缺口，AC-01/03/05/06，T-03/T-05/T-07，尚未发现这些项的产品失败。

- `verification.md:63–64` 声称 source_hir 对 P13/P14 做 `overflow:trap` 结构断言；实际文件只有它们列入通过例，没有检查 add/mul 的 overflow 字段。退出 70 和前驱 `a` 已实测，adapter 字段静态检查也正确，但声明的结构测试不存在。
- §6 要求 adapter **Bundle** 删 let/错 type/越域 ref 后实际重新 verify。现有 `tampered_atom_is_rejected` 变异的是生成 Atom 文本（删 let、错常量 type、悬空 expr、假 checked），不是投影后的 Bundle；越域引用也被换成悬空引用，错误性质不同。错 type 分支还有条件跳过，未强制变异实际发生。
- source_cli 只测试空输出目录的失败，没有 source 模式对既有 exe/obj/receipt 三件 SENTINEL 的回归。旧 CLI 的测试不自动覆盖新增复制出来的源码 build 路径。

独立 source SENTINEL 控制已经通过：坏源码、缺 support lib 导致坏链接、错误入口签名均 exit1，三件 SHA256 前后完全相同（supplemental-results.json）。因此这里不报告事务破坏，而要求把计划已规定的回归加入永久 gate。

修复：增加真 HIR overflow 字段断言和实际 Bundle 变异/重新 verify，断言每次变异确实改变输入、诊断对应不变量；加入 source SENTINEL 三件保留路径，至少覆盖 source拒绝/link失败/publish回滚。修订报告，以执行记录说明每个补充项，不能拿旧 CLI 相邻测试当已测源码路径。

### P745-R1-QA-07 / P3：盘点失败归因和待沉淀策略不准确

证据/收口缺口，AC-07，T-06/T-07。`proposed-spec-delta.md:72` 将 `--output` 的目录参数写成 `.../inventory.md`，因而在不存在的子目录找 manifest/decisions；`:79` 声称 source-manifest 本不入库，`git ls-files` 证明 source-manifest.json 与 manual-decisions.json 都已跟踪。

使用正确命令 `python scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check --require-decisions`，本次同树 **exit0 / CHECK-OK / 47 decisions / 9 inputs / 146 unknown**（inventory-correct.log）。因此不能把错误命令的失败归为 743 扫描器状态，也没有证据支持为消除这两项失败而在 merge 自动 `--write` 重扫/重绑。

修复：修正命令、事实与 T-06/报告的失败归因，保留旧误调用历史但明确作废；追加 MD-408/409 note 时按实际语义变更独立检验新鲜度和消费者，避免重 hash 掩漂移。完整宽度与 char 仍保持 open。SD01/03 的完整能力叙述须待其它发现修复后重新审定。

### P745-R1-QA-08 / P3：Checked 后构建流程未共用

原设计合同缺口，AC-06，T-04/T-05。`src/main.rs:186–339` 新 source build 和 `:389–495` 旧 HIR build 分别复制 capability/entry/lower/staging/tool discovery/link/publish/cleanup 流程；只共用了底层函数。§2/§5.4/T-04 明确要求模式独立解析、共用 Checked 后构建管线。

本次普通失败 SENTINEL 和旧事务控制均通过，未据此捏造行为失败。问题是设计要求未落实，旧 CLI 的回归不足以代替新副本的控制，后续修复可能只落其中一路。

修复：模式前端只负责构造本进程 Checked、解析入口和源码 receipt 附加元信息；统一调用一个 Checked→native→staged link→事务发布/清理入口，保留两模式的诊断来源和0/1/2语义，双模式实测回归。

## 验收映射

| AC | 结果 | 任务/主要证据 |
|---|---|---|
| AC-01 | partial | T01数值报告、边界PE/HIR值、C22负溢出70通过；QA06缺结构oracle。完整int/char unknown 未被关闭 |
| AC-02 | fail | 74原语料通过；QA01/02/04独立边界反例失败，合法标识符native不完整 |
| AC-03 | partial | 真实 parse→resolve→Atom→bind_source→投影→verify→Checked 链及旧HIR拒绝通过；QA06规定的Bundle变异矩阵缺失，未发现绕过Checked入口 |
| AC-04 | fail | 基本span合法、UTF8/CRLF及既有负例通过；QA03/04精确token/code反例失败 |
| AC-05 | fail | 21正例全部实际PE含12/ba、70/a；C07=8、C08=21/ba、C22=70；QA02合法函数名无法native，QA05/06执行器/结构门缺口 |
| AC-06 | partial | 显式profile/entry/capability/trace shadow和三项source SENTINEL通过；QA02/05/06/08阻止全合同pass |
| AC-07 | partial | 四个SD和MD note保留有限域/full-width/char open；正确strict inventory通过；QA07报告和沉淀策略须纠正，SD01/03待修后复审 |
| AC-08 | fail | 本独立最终复审完成，发现未清偿；根门禁确实运行但非绿色，无新增健康警告证据 |
| AC-09 | pending | merge所有权：主线整合、SD/人工层/ledger/index/导航、到期批回归、真实archive断言、guard/清理；本轮未执行 |

## 门禁、健康与归因限度

全部命令工作目录均为原745树，串行运行；ac-core独立workspace，不链接生产旧前端/VM，不变生产Cargo/config，无调试打印遗留发现。命令/状态/raw output：ac-core-gates.log、ac-core-gates-status.txt、verify-741.log、verify-745.log、scripts-status.txt、main-check.log、main-check-status.txt、main-t-complete.log、main-tv-complete.log。

| 命令 | 本轮实际结果 |
|---|---|
| ac-core `cargo check --locked --all-targets --manifest-path ...` | PASS，零warning |
| ac-core `cargo fmt --manifest-path ... -- --check` | PASS |
| ac-core `cargo test --locked --manifest-path ...` | PASS，92 passed / 1 ignored |
| `verify-ac-741.ps1 -SkipMainGates` | PASS，13步骤 |
| `verify-ac-source-745.ps1` | PASS，12步骤 / 74例；green不覆盖本报告补充反例 |
| 正确 `acc_inventory.py ... --check --require-decisions` | PASS，47人工决定 |
| `cargo check -p auto-lang` | PASS；382旧warning，生产源/config未被745修改 |
| 裸 `cargo t` | exit100，fail-fast：1051 run / 1050 pass / 1 fail / 3921未运行 |
| `cargo t --no-fail-fast` | exit100，4972 run / 4943 pass / 29 fail / 1534 skipped，80.820s |
| 裸 `cargo tv` | exit100，fail-fast：124 run / 120 pass / 4 fail / 38未运行 |
| `cargo tv --no-fail-fast` | exit100，162 run / 158 pass / 4 fail / 6330 skipped，4.196s |

补跑的理由是 fail-fast 未覆盖剩余门禁，没有反复跑全量求绿。未另跑 tf/taa/tt/tb/tu/docs_gen 专项（裸 t 内自带的 docs_gen 测试仍如实记录）。完整失败列表见 daily-results.txt / corpus-results.txt。

745 diff 对 `crates/`、根Cargo.toml/lock、`.cargo/.config` 为零。生产工作树与合并742后的主检出有差异，不能称严格同源码对照；本轮没有另外创建复审/基线树。历史执行对照见 historical-attribution.txt，仅证明名字共现，不将“对称差”直接证明为flake。本轮29个daily失败均在历史双方红集的并集中，4个tv失败同为cookbook semver parse/latest/increment/prerelease。以生产代码未变、历史重复失败和明确红清单区分生产基线问题，**不宣称根门禁绿色、不冒称逐项严格同源零新增已证实**。修复后的T07须重新出具适当范围门禁及归因。

## 规范增量与交接

冻结SD01..04为本次审查输入，不发布canonical Specs或live ledger。SD02实际有限S0路径有证据；SD04有限宽度/open边界可保留；SD01/03的完整source合同描述等待修复，QA07盘点策略纠正后重新冻结。impact元数据三项modify、一项add与Plan目标一致；touched_goals为空，因为本轮不重开既有GOAL-017。

未发现未经批准扩大支持运算/类型、VM/A2R fallback、外仓实现或假Checked入口。明确拒绝缺项不能靠登记债务降为完成；QA01..08已入KNOWN-DEBT-AND-RISKS。不存在已批准延后这些合同项的证据。

Plan返回 `executing`，重开 T-02..T-07 及其依赖证据，T-01 保留完成；current_step=1 / total_steps=9。T-08/T-09仍未完成。保留GLM历史内部pass，但它不能覆盖本次needs_fix。实现者继续原树，修复提交后按新完整HEAD申请最终复审；不换树、不 merge/archive、不清理原树。
