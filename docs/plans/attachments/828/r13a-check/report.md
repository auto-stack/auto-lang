# PLAN-828 r13a：指定 C 盘反例及完整分段复审

2026-10-21，**needs_fix**。已按用户要求从 C 盘 r12a-check 原样重放。同值字面量起点原例、9格矩阵及交错会话原例通过；**旧 helper 原例、两侧保留和尾段位移边界仍失败**，该分支没有修改。完整来源映射也未闭合：同值分母扩长仍截断表达式终点。

受审 `3f4bbe8e75ca3c4ef3a577ed551052d2a6a413a5`，基面 `9b747e251c74cc3c981a36682d67064d179dacad`，工作树 `D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`。Plan revision2、executing0/11。review_scope=final中的用户指定受影响 finding/就绪核查，非整份最终验收。reviewer为同一Codex `/root` 独立审查上下文，未实施r13a、未创建agent，使用auto-plan-review。未改实现、未合入。

## 用户点名项的原样重放

直接复制 C 盘 r12a-check 的 `replay.rs`、`historical-map-comparison.rs`、`base-span-map.rs`、`embedding-boundaries.rs`，逐文件比对SHA256保持一致。编译时仅链接当前实施代码，未改变旧输入、断言或期望。原始脚本遗留的打印标签`current r9a`不代表本轮版本：include的是当前工作树的r13a span_map.rs，实际文件哈希见[baseline](baseline.json.txt)。

| 点名检查 | 当前结果 |
|---|---|
| `same_value_leading_zero_literal_maps_to_token_start_not_matching_suffix`，`1→01` | PASS |
| r12a的9格同值十进制矩阵 | 9/9 PASS |
| `same_value_literal_edit_then_trivia_rewrite_reaches_tail_loss`，先同值字面量再改空白 | PASS |
| 保存的旧 helper 原例 | **FAIL：55..55，期望65..70** |
| 同段两侧都存活的边界 | **FAIL：10..10，期望17..22** |
| 仅尾侧存活的边界 | **FAIL：16..21，期望17..22** |
| 仅头侧存活的未改前方span | PASS：2..6 |

因此原会话例通过和完整区间机制修复应明确区分；不能将“两类同根消除”或永久53项通过写成helper/两侧/位移也已闭合。

## 实际执行

所有编译/测试cwd为上述实施树。

| 检查 | 实际结果 / 证据 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 53/53，0 ignored，2.63s，exit0；[raw](permanent-suite-output.txt) |
| 同manifest `--test function_incremental_828 -- --test-threads=1` | 17/17，12.62s，exit0；[raw](original-suite-output.txt) |
| 同manifest `-p auto-av-core -- --test-threads=1` | core22/22，0.09s，exit0；doc-tests0不计覆盖；[raw](core-suite-output.txt) |
| C盘r12a原59例完整重放 | 58过/1失败，15.16s，exit101；唯一失败为既有恢复span=None；[原始副本](original-replay.rs)、[raw](original-replay-output.txt) |
| 原59例+分母端点例+72组合矩阵 | **58过/3失败**，0 ignored，13.50s，exit101；[源码](replay.rs)、[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 原矩阵 | 186/186、528/528、空白12/12、注释4/4、同值9/9；内含于runner，不重复相加 |
| 新双端数字组合矩阵 | 72组合：27过/45失败，属于扩展runner内一个失败test |
| 原helper反例 | exit101；[源码](historical-map-comparison.rs)、[raw](historical-comparison-output.txt)、[历史控制模块](base-span-map.rs) |
| 原embedding边界三例 | 1过/2失败，exit101；[源码](embedding-boundaries.rs)、[raw](embedding-boundaries-output.txt) |
| 修改的两个Rust文件 rustfmt --check | exit0；[raw](fmt-output.txt) |
| `git diff --check 9b747e251c74cc3c981a36682d67064d179dacad HEAD` | exit0；[raw](diff-check-output.txt) |

独立helper检查不并入会话runner总数；旧helper与“两侧”边界都体现同类缺陷，不能伪装为更多独立根因。原17中的进程内时延门本轮通过，不抹去历史波动，也不替代F12正式worker/IPC/内存门。

## F04-u / P1：两侧保留和完整输出长度仍未修

位置：`experimental/ac-av-probe/core/src/span_map.rs:240–244`。本批只改数字token对齐；`apply_aligned`仍使用互斥head/tail分支。

- 同一存活段跨过重嵌入区间时，先满足`else if s0_m < m0`保留head，便不能再进入`else if s1_m > m1`保留tail。原helper输入未变，仍55..55而非65..70。
- 即使没有head、实际进入tail分支，其当前起点仍设为`mid_c0+k+mid_old.len()`，只到嵌回旧片段末尾，忽略替换产物在其后的新增字节。

C盘原边界例的编辑为current `[8,9)` 的`2`换成` 2 `，后方mint `[15,20)`应完整平移+2到17..22。identity `[0,20)`两侧例当前丢tail为10..10；identity `[8,20)`仅尾例当前16..21，少计1字节。前方head-only控制保持正确。

这三项本轮均原样实测，不是只看源码推断。仅使本轮交错会话改走成功重锚路径，不能证明该公开helper或仍保留的fallback机制正确。必须独立修复两侧保留及完整替换长度，单改互斥条件也不足。F04-u维持OPEN，关联AC05/V02，T02/T07。

## F04-r / P1：同值数字的完整终点仍被旧字节边界截断

位置：`span_map.rs:34–112` 的prefix/suffix/token重锚，以及`map_token_end:184–198`的旧分段边界偏置。

新增 `same_value_denominator_extension_updates_whole_expression_end`：原TRAP_SRC的main表达式为`1 / 0`，span70..75。将最后的`0`换成`00`。先完整cold warmup候选成功，再真实edit；断言源码等于候选，`Arc::ptr_eq`确认同一main代码复用。

```text
actual:   func#1 70..75
expected: func#1 70..76
main map: [(29,75,29),(75,77,76)]
```

新digit在旧表达式末端被视为插入blob，`map_token_end`继续选择前段旧末尾，按“token后插空白”的规则将它排除；实际它属于当前同值字面量`00`，完整表达式应延长。r13a修了首token的`1→01`起点，但不能把修复推导为首尾及任意拼写均正确。

新增72组合矩阵使用初始表达式`1 / 0`和`01 / 00`，左侧目标`{1,01,001}`、右侧`{0,00,000}`，分别有/无外侧空白、有/无运算符两侧空白。共2×3×3×2×2=72，全部候选先完整cold warmup，再edit和执行比较；27过/45失败，逐条输入/actual/expected保存在raw。

该矩阵是同一既有字面量与scalar除法支持面上的两端、增删和组合核查，不是新基数/后缀/浮点/范围承诺。原profile已将十进制digits解析为数值并保留整个token span。旧起点例和9格矩阵通过，只能说明这些具体输入闭合。

修复必须给复用指令关联当前完整字面量/表达式的来源范围，不能用旧token字节长度或残留字节边界替代当前端点。prefix/suffix修剪、数字匹配、token起点与终点规则需一起遵守该不变量。沿用F04-r，关联AC05/V02，T02/T07。

## 关闭表与仍开放的工作

| 项 | 当前状态 |
|---|---|
| r12a同值起点/9矩阵/交错会话 | 原例PASS |
| F04-r完整来源不变量 | FAIL：完整数字终点及组合仍错 |
| F04-u旧helper/两侧/尾段位移 | FAIL：原例与原边界均未闭合 |
| F04-t代码mint文本归属、F04-s恢复history metadata | 原例继续PASS |
| 恢复实际执行span=None | 仍FAIL，既有开放恢复来源面，未归因r13a新引入 |

恢复例的失败发生在edit前，不虚构后续编辑分支已执行；独立metadata例继续PASS。pinned/resume、历史内容命中、递归、intrinsic/授权正反控和先前删除/撤销矩阵保持。本轮没有宣称完整快照/GC/恢复/授权验收通过。

原helper从此前批次连续保留且用户本轮明确点名，必须纳入修复门，不能再只收编会话最小例。诊断/覆盖调整应明确：左右存活分段都要保留；tail从完整替换产物末端定位；指令来源使用当前完整语义范围。下一轮需一次重放本包全部原例和边界，再核双端组合；不能把token局部特判变绿当成完整修复。

AC05来源子项FAIL、V02FAIL、V07恢复来源FAIL；V05已测pinned/resume子项PASS。其余AC保留历史状态，未授整项PASS；AC09 needs_fix、AC10 pending。Plan保持executing0/11；T02/T07相关来源/历史项不得关闭。F01真函数级前端、F02安装payload、F05b取消/关闭、F09资源、F10 A07/依赖/GC/恢复sidecar、F11持久化/manifest、F12正式worker/IPC/内存、F14文档/SD等OPEN不变。

[冻结Plan](plan-input.md)、[冻结SD-01..06](spec-delta-input.md)、输入/规范/依赖/运行库/二进制哈希见baseline。SD未做完整最终验收；未写canonical/ledger，不把合同缺口降为获准延期debt。

健康：本次修改Rust格式/diff-check通过；既有core shift、session new_len/未用关联history_entry、SpanInfo/context_revision及shim warning保留于raw。测试前后实施树clean、HEAD固定，auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费。未改实现，未跑整档t/tv/tt/tb/tu/taa/tf或docs_gen；最终独立pass后裸t+tv及必要consumer/触面门仍merge-owned pending；未合入、归档。

证据完整保存在C盘，不干预共享主检出的其他会话工作。[待安全回写Plan §9](pending-bookkeeping.md)。next：同一实施树work完成F04-r/u及已开放恢复来源面，冻结提交后复审。
