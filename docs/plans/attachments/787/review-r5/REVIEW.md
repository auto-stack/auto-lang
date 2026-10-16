# PLAN-787 final review R5 — needs_fix

2026-10-16；stage: review；review_scope: final；plan_revision: 1。
reviewer_identity: 当前 Codex 审查会话，延续 R1–R4 审查，未参与实施、未创建代理。实施源码写入暂停；本轮只新增审查探针/证据及主检出计划簿记。

## 基线

- worktree `D:/autostack/.wt/lang-787/auto-lang`；branch `plan-787-dev`。
- reviewed_commit `d2b0ec99ba579547a1506a2babc40b232c3dcda3`；previous_commit `15cfeac977a3a87b35b18aa92e56bfe9097b7174`；base_commit `bf773ca9bfc5a2d6ed672f904011419b03eff1ef`。
- 主检出入口 HEAD `fcd3cc930c546d7c6014ba1e7fe1cc73e4452f10`；共享计划 execution_done 7/9，本轮明确送审。已核四连败诊断/闭环表；此次确实改为规范形存储、完整子树准入、窗口坐标重定位、阶段内 OS 采样及容量计数，非仅改旧例。
- auto-atom/auto-ac-prototype/ac-artifact-probe 为同实施树 path 依赖；清单/锁/验收配置 hash 见 `baseline-hashes.tsv`。当前主检出 Spec/Design 的实际工作副本 hash 见 `spec-inputs.tsv`；不覆盖并行任务的工作。
- 冻结拟议规范 `spec-delta.md` SHA-256 `b313da6b908ab235c76c0c80ea9b292a66ecc9479e8d6a6e9ced0a8bd47dd364`。canonical/ledger未修改。
- 入口工作树 clean；结束基线核验见 `final-baseline.txt`。

## 验证与 R4 闭环

- R1 **7/7**、R2 **18/18**、R3 **8/8**、R4 **3/3** 全部独立重放通过，见各 `rN-replay.log`。集合重叠，不相加为独立总数。所有旧植入数据仍真实命中稳定键形；无通过增加版本使反例空过。
- 新独立行为探针 **3项失败**，见 `counterexamples.log`；三个函数覆盖两个行为finding（矩阵和最小例属于同一缺陷），均有有效正控；另一个容量实测清单正常执行，见 `allocation-inventory.log`。
- `cargo check --manifest-path experimental/ac-syntax/Cargo.toml --workspace`通过（只见既有shim-metadata警告）；全语法Rust文件 rustfmt检查通过；`git diff --check 15cfeac97 HEAD`通过；对base的生产 `crates/` 与 `experimental/ac-core/src/` diff为空。见 `check.log` / `fmt.log` / `diff-check.log`。
- 本轮定向 acceptance（含64MiB）**16族全部通过，raw passed合计106次**（过滤族有重叠，非独立测试去重总数），见 `acceptance.log`。预算断言绿不消除已识别的漏计。未跑根整档/docs_gen或 merge closeout。

| 旧 finding | 本轮结论 |
|---|---|
| R4-01 fresh/增量入库冲突 | CLOSED：规范形重排由根孩子文档序/先序决定；旧反例与正反顺序、二次编辑、重开永久例通过。 |
| R4-02 child越源未拒 | 原反例 CLOSED；span/归属/字段/重复边在decl_subtree真实入口校验。但合法epsilon节点的新误拒见R5-01。 |
| R4-03 跨页trivia下溢 | 原反例 CLOSED；跨页空白/CRLF/Unicode注释/多页/非首声明永久例通过。更大合法前导窗口成功空名见R5-02。 |
| R4-04 峰值计量缺口 | PARTIAL：OS后台期间采样与build/warm/cold/rebuild分列成立，目录/JSON/put零克隆和大多数parse容量已接线；返回树/源码仍有漏计见R5-03。不再重复旧“只在返回点采样”指控。 |
| R4-05 旧SD | 旧revision/4096规则已删除，更新为会话仲裁/2048/规范形/准入/采样/缺失项；原陈旧文字 CLOSED。SD-05/06最终正确性受本轮行为/计量问题影响，整体暂不通过。 |
| R4-06 格式 | CLOSED：直接全语法rustfmt检查通过。 |

## F787-R5-01 / P1 — 子树准入误拒合法空参数调用

AC-05（V-09/10），T-06/T-07；R4-02修复引入的消费者回归，未扩语言支持面。

实际入口 `adapters/artifacts/src/lib.rs:1411` 强制每个非trivia孩子跨度位于父span。现有parser对合法零宽 `args` 规则产生 start=0/len=0，decl=1；它表示空参数，不是越源数据。`fn main() int { return f() }` 被既有core源码parser接受，真实 parse→put_tokens→put_cst 成功，open命中；decl_subtree(0)却返回 `Admissions("child 22 span [0,0) escapes parent 17 [23,26)")`。单参数 `f(1)` 全链通过，区分空产生式的表示与真实坏span。

独立枚举现有core fixtures/source/{valid,invalid}与10个mini源：**136个文件、144次声明查询，12次拒绝**，其中 **7个valid、5个invalid**；valid包括c05、c06、d07、m18、p15、p16、p19。invalid语料中的可恢复CST也应能查询；严格语义拒绝不等于缓存准入可以误拒结构。

须统一生产者/持久化/准入对epsilon与missing等零宽节点的坐标合同，保留真实越源/越父/错归属拒绝；不能简单放开任意零宽坏跨度或删掉父子校验。补双profile、空/非空args、错误恢复、非零origin与fresh/增量入库恢复的永久正反控制。最小例与矩阵源位于 `probes/counterexamples.rs`。

## F787-R5-02 / P1 — 大前导trivia被成功查询静默截成空名称

AC-05（V-09/11），T-06/T-07；R4-03修复的窗口边界回归。

实际query bin `ac787-query.rs:73–97` 先把 `[origin,span_hi)` 相交页全部追加进window_pages，随后以2MiB guard决定是否查询；超限分支返回空String并打印正常QUERY/退出0。

独立真实StreamingCstBuilder与原query程序：`" ".repeat(2MiB+10) + "api f() -> i32;\n"` 构建/子树恢复均成功，进程内outline准确name=f；实际子进程输出 `span=2097162,2097177 name=`、exit0。prefix=0正控输出name=f。该输入仅一个小声明+一个合法空白token，不是承诺任意规模性能；2MiB隐藏guard不属于已批准语言/资源拒绝合同。它还在判界之前物化整个前导窗口，不能当成有界内存的保护。

须按真正需要的token/源范围保留坐标、控制驻留，或在配置资源不足时明确受控ResourceExhausted；绝不能静默空outline冒充精确恢复成功。不得只删guard后无限物化trivia，也不能靠加预算或饱和坐标掩盖。永久oracle应覆盖阈值两侧、精确名称与受控资源出口。

## F787-R5-03 / P2 — 返回树的实际容量及源码页仍未进入受管峰值

AC-05/06（V-11），T-06/T-07；R4-04剩余计量缺口。**没有测得OS或受管真实峰值超限，不据此声称超限。**

`lib.rs:1792–1813` 的transient_base枚举nodes/edges/fields、bases、block_decl、decls、tokens与lex_diag外层，但遗漏仍在out.tree内存活的 `blocks` Vec、`root_edges` Vec、tree.diags及其String容量；push_chunk内部新建的snap/store源码页也一直存活到返回（源码文档明确这些页应计入持有者工作集）。这些不是已显式排除的parse Machine内部临时栈。

对与生成器同型的首个65556B块，独立实测 `blocks.capacity` **128B**、`root_edges.capacity` **22600B**，仅返回树这两项即漏 **22728B**；snap源码页payload另至少 **65556B**（尚不含页/引用/哈希目录容器）。见 `allocation-inventory.log`。本轮计数较旧轮大幅改善，但还不能称“parse产物实际容量全部计入/只有Machine缺失”。当前8027KiB读数靠近8MiB合同线，不能未经对账认全口径通过。

请把实际驻留项及put/finish瞬态逐项对账、纳入同驻留高水位；明确受管范围及每个排除项而不改写原预算合同，重新测量并更新报告/SD-06。不要求靠源码检查看似接近来推定超限，也不要求把OS采样值替代受管计数。OS采样改善已认可。

## AC / task / Spec 判定

| AC | 结果 | 证据与边界 |
|---|---|---|
| AC-01 | pass | snapshot/position/handle/代际未受本轮更改，旧反例及acceptance维持。 |
| AC-02 | pass | 配置双profile/生成期拒绝保持。 |
| AC-03 | pass（runtime范围） | parser/strict/无损/深度仍可用；本轮错误在缓存消费者准入，未把f()重新判成语法不支持。 |
| AC-04 | pass（runtime范围） | 局部复用/200编辑/迟到控制保持；fresh/增量真入库冲突原例已修。 |
| AC-05 | fail | 合法零宽树误拒、真实query成功空名及受管计量残留。 |
| AC-06 | partial | 生产零切换/依赖兼容/性能证据存在；SD-05/06与峰值断言需随修复复核。 |
| AC-07 | fail | 三项finding未闭合，T-08不能通过。 |
| AC-08 | pending merge-owned | final pass后冻结提交完整独立语法工作区+783触面套件、SD/索引/ledger、合流归档与guard清理。 |

规范：supersedes/new路径准确；touched_goals=[]有仅实验/生产零切换解释，无需虚构目标。SD-01/02/04有限语法/版本/导航方向成立；SD-03运行时成果保留；SD-05/06须修正消费者兼容与实际计量的事实。冻结稿整体不批准，不发布canonical。

**outcome: needs_fix；status: executing；current_step: 5/9。** 保留T-01..05，仅重开T-06/T-07；T-08/T-09未完成。旧例闭合与新回归分开记录，不重开已证实的运行时/投影成果。原树auto-plan-work修复三个finding、永久组合oracle和SD/矩阵后提交再审；不批准转延期债。未修实现、未合入。

复现：在原worktree设RUSTC_WRAPPER为空、CARGO_TARGET_DIR=experimental/ac-syntax/target，执行 `cargo test --manifest-path D:/autostack/auto-lang/docs/plans/attachments/787/review-r5/probes/Cargo.toml --test counterexamples -- --nocapture --test-threads=1`。行为日志取前3项；容量清单单独按inventory_returned筛选执行。探针含锁文件，path依赖仅原树，range-query直接编译受审原bin；子进程复用probe run_deadlined，30s截止。
