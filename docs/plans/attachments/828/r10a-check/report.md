# PLAN-828 r10a：旧反例重放、词法空白与尾段核查

2026-10-21，**needs_fix**。上轮三个会话反例均通过，F04-t 的旧文本保存问题已修；但上轮保存的 **F04-u helper 原反例仍然失败**，并可在当前真实会话重新进入同一错误分支。F04-r 的新 token-run 回退仍把非空白字符串片段当词法 token，合法的相邻运算符空白编辑继续丢失来源。

受审提交 `e6e8cdc9ff41cc30a5266a6192f8b06b09553050`，基面 `ee4d1b97b6150516a7db596fdfa03474076b450e`；工作树 `D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`。Plan revision 2，executing 0/11。review_scope=final 中的用户指定受影响 finding/就绪核查，未启动整份计划最终验收。reviewer：同一 Codex `/root` 独立审查上下文，未实施 r10a，未创建 agent。使用 auto-plan-review；未改实现、未合入。

## 实际执行与旧项状态

原始输入直接复制 C 盘 r9a-check，[original-replay.rs](original-replay.rs) 的字节哈希与原文件一致，见 [baseline](baseline.json.txt)。所有构建/测试 cwd 为实施树。

| 检查 | 实际结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 44/44，0 ignored，2.05s，exit0；[raw](permanent-suite-output.txt) |
| 同 manifest `--test function_incremental_828 -- --test-threads=1` | 17/17，13.95s，exit0；[raw](original-suite-output.txt) |
| 同 manifest `-p auto-av-core -- --test-threads=1` | 22/22，0.15s，exit0；doc-tests0 不计覆盖；[raw](core-suite-output.txt) |
| r9a-check 原50例完整重放 | 49过/1失败，11.93s，exit101；唯一失败为已承认的恢复 span=None；[raw](original-replay-output.txt) |
| 原50例+两条真实会话反例+词法空白矩阵 | **49过/4失败**，0 ignored，26.17s，exit101；[源码](replay.rs)、[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 原186格及528条三步组合 | 均全过，内含于 runner 的 test 中，不重复相加 |
| 新词法空白矩阵 | 12组合：3过/9失败，属于扩展 runner 中一个失败 test 的子例 |
| 原样重放 r9a 保存的 helper 反例 | 旧基面65..70，当前55..55，期望65..70，exit101；[源码](historical-map-comparison.rs)、[raw](historical-comparison-output.txt)、[历史模块](base-span-map.rs) |
| 本次三个修改 Rust 文件 rustfmt --check | exit0；[raw](fmt-output.txt) |
| `git diff --check ee4d1b97b6150516a7db596fdfa03474076b450e HEAD` | exit0；[raw](diff-check-output.txt) |

helper 源码与上轮相同，include 路径读取当前工作树的 span_map.rs，已绑定当前提交/文件哈希。其打印文本中的 `current r9a` 是原样保存的旧标签，本轮实际编译的是 r10a；不把标签当版本凭据。基面模块为 r8a 冻结源码，历史控制仍通过。该 helper 结果不与会话 runner 的 test 数合并。

| 上轮发现 | 本轮结论 |
|---|---|
| F04-r 内外 trivia 变化的具体例 | 原例 PASS；同一来源不变量仍有下面词法边界缺口，未完整关闭 |
| F04-t 新代码历史项存上一版文本 | 原例 PASS，代码已从 new_parse/new_src 提取；该生产点缺陷闭合 |
| F04-u 重嵌入漏保留尾段 | 原 remint+literal 会话例 PASS，但 helper 原例 FAIL，真实交错会话也 FAIL，仍 OPEN |
| F04-s 恢复 history 空表 | 原独立 metadata 正控继续 PASS；不等同恢复 payload 有来源 |
| 恢复运行 span=None | 仍 FAIL，work 已明确承认开放；未归因本轮新引入 |

旧 pinned/resume、历史命中、删除/撤销/替换组合、递归、intrinsic/授权正反控及连续编辑正控均保持。原17例中的进程内时延门本轮通过，不代表 F12 正式 worker/IPC/内存门通过。

## F04-r / P1：非空白 run 不是词法 token

位置：`experimental/ac-av-probe/core/src/span_map.rs:60–86,318–333`，以及 from_texts 返回 None 后的 fallback。

`runs()` 只按 ASCII 空白分割，所以 `1 / 0` 被分为三个 run，`1/0` 被视为一个 run。两段源码却具有相同的整数/除法/整数词法序列，完整冷编和增量 edit 均合法，HIR 内容命中。当前回退因 run 数量不一致拒绝重锚，再次退回丢弃表达式来源的几何映射。

反例 `adjacent_operator_rewrite_with_outer_trivia_keeps_actual_lexical_tokens`：原 TRAP_SRC 中 main 的 `[70,75)` 为 `1 / 0`，替换成 ` 1/0 `；断言 edit.ok 及 reuse_hit>0，并以当前源码完整冷编为 oracle。

```text
actual:   func#1 70..75
expected: func#1 71..74
main map: [(29,70,29),(75,77,75)]
```

新12组合矩阵的初始表达式为 `{1 / 0, 1/0, 1  /  0}`，替换为 `{ 1/0 , 1 /0 , 1/ 0 , 1 / 0 }`，逐个新会话真实 edit 和冷编比较。12例全部合法，9例错误；完整输入/actual/expected 在 raw。三条通过控制证明不是所有重锚都失败，也不能用它们推导词法等价全部支持。

修复应依据真实词法/CST/语义来源对应处理相邻运算符和空白增删，不应把非空白片段数量当作 token 数量。该例只使用原有 scalar 除法与空白支持面，是 F04-r 的剩余合同失败，不是新增语法或规模要求。关联 AC05、V02，T02/T07。

## F04-u / P1：尾段错误仍在生产分支，可真实到达

位置：`core/src/span_map.rs:228–232`。本提交没有改 `apply_aligned` 里互斥的 head/tail 分支：同一个存活段同时跨越被重嵌入中段时，保留 head 后就不能保留 tail。上轮的“旧 mint_text 问题”和这个错误是两项独立问题。

r10a 修复 mint_text 后，上轮 remint+等价 literal 例改走全文 re-anchor，绕开错误分支。因此该原例及名称为 keeps_tail 的永久测试虽绿，不能证明分支已修。上轮 helper 输入未变，仍得到55..55。

新增实际会话反例 `disjoint_trivia_edits_reach_unchanged_tail_loss_in_real_session`：

```text
fn helper() int { return 1 }
fn main() int { var y = 1 + 2; return 1 / 0 }
fn tail() int { return 9 }
```

1. 将前方 `1 + 2` 换成 `1+2`。edit 成功，此轮 trap 来源与冷编相同（后方表达式只整体前移2）。
2. 将后方 `1 / 0` 的首个 `1` 换成 ` 1 `，edit 成功并发生内容命中。
3. 全文差异因前方 run 不匹配走 fallback；本次被替换 literal 能重嵌入到 ` 1 `，命中一个跨两侧的 mint 分段，tail 被丢弃。

```text
actual:   func#1 66..67
expected: func#1 66..72
main map: [(29,54,29),(57,67,55),(67,68,66)]
```

这不是仅公共 helper 未被生产使用的理论问题：当前真实会话已经输出错误，除法后半段的位置实际丢失。两个会话都只改空白，未改源码语义或请求新支持面。

必须修复区间手术本身，独立保留 head 和 tail；不能只让一个入口不再走 fallback，也不能因目前另有 token-run 缺口参与到达而将尾段错误判为关闭。公开 helper 反例、实际 fallback 消费和 history 的映射复合均应纳入回归。关联 AC05/V02，T02/T07；沿用上轮 F04-u，不伪称为新的 finding。

## 恢复、覆盖边界与下一轮

恢复实际运行仍在 edit 前得到 DivByZero/span=None。原 test 未抵达后续编辑断言，单独的恢复 metadata test 则已实际验证首次 edit 的 map。F10/F11/恢复来源面保持 OPEN；修复本轮两项也不会自动补出持久化 payload 的来源。

本轮确认 mint_text 生产点修复有效。未发现旧186格/528格或已测 pinned/resume 控制回归，但它们不能覆盖新文本匹配算法的所有分支。反复局部修复后仍存同一 helper 原反例，必须把诊断从“最小会话例变绿”改为“对应生产分支的输入、输出和不变量均通过”：词法等价应按真实 token 识别；区间中段重写应保留两侧；code/text/map 必须同一 mint 版本。下一轮至少重放原50、保存的 helper、两条新会话及12组合矩阵；不要只收编最小例并宣称 F04 全闭。

AC05 来源子项 FAIL、V02 FAIL、V07 恢复来源 FAIL，V05 已测 pinned/resume PASS；其余 AC 保留历史状态，不授整项 PASS；AC09 needs_fix，AC10 pending。Plan 仍 executing0/11；T02/T07 相关来源/历史项未闭。F01 真函数级前端、F02 安装 payload、F05b 取消/关闭、F09 资源、F10 A07/依赖/GC/恢复sidecar、F11 持久化/manifest、F12 正式 worker/IPC/内存、F14 文档/SD 等 OPEN 不变。

[冻结 Plan](plan-input.md)、[冻结 SD-01..06](spec-delta-input.md)、规范/依赖/输入/运行库/二进制哈希见 baseline。未完整最终审查 SD；不发布 canonical/ledger，不把合同内缺陷降格为已批准延期的 debt。

本次修改文件格式和 diff-check 通过；既有 core shift、session new_len/未用关联 history_entry，以及 SpanInfo/context_revision/shim warning 仍保留在 raw，未以通过测试替代清理义务。工作树测试前后 clean，HEAD 固定；auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 未消费。未改实现，未跑整档 t/tv/tt/tb/tu/taa/tf 或 docs_gen；最终独立 pass 后裸t+tv及必要consumer/触面门仍 merge-owned pending；未合入、未归档。

证据完整保存在 C 盘，不干预主检出其他会话工作。待安全回写 Plan §9 的记录见 [pending-bookkeeping](pending-bookkeeping.md)。next：同一实施树 work 修 F04-r/u，并继续已开放的恢复来源工作，提交后受影响面复审。
