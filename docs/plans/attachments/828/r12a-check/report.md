# PLAN-828 r12a：注释修复重放与完整来源核查

2026-10-21，**needs_fix**。已确认永久回归50/50，r11a的两条合法注释会话和4格矩阵全过。该批次的具体注释输入修复有效；但“两类失败闭合”的完整结论不成立：**保存的 F04-u helper 原反例仍红，互斥分支没有改动**，并且同值字面量编辑后仍能在真实会话进入该错误分支。F04-r 的字节对齐在同值数字拼写上也仍不等于语义来源对应。

受审 `9b747e251c74cc3c981a36682d67064d179dacad`，基面 `af462a947a0a4c08bbc42d3dd3afbd67fdb5c072`，`D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`；Plan revision2，executing0/11。review_scope=final 中用户指定的受影响 finding/就绪核查，未启动整份最终验收。reviewer：同一 Codex `/root` 独立审查上下文，未实施 r12a、未创建 agent；使用 auto-plan-review。未改实现、未合入。

## 实际重放与验收边界

原 r11a-check 的合法最终56例直接从C盘复制到 [original-replay.rs](original-replay.rs)，哈希一致；未复制上轮已排除的无括号跨行无效探针。本轮所有新增候选均先完整 cold warmup 接受，再 edit 和比较来源。版本、规范、原始输入、运行库及二进制哈希见 [baseline](baseline.json.txt)。所有编译/测试 cwd 为实施树。

| 检查 | 实际结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | **50/50**，0 ignored，3.24s，exit0；[raw](permanent-suite-output.txt) |
| 同manifest `--test function_incremental_828 -- --test-threads=1` | 17/17，12.94s，exit0；[raw](original-suite-output.txt) |
| 同manifest `-p auto-av-core -- --test-threads=1` | core22/22，0.09s，exit0；doc-tests0不计覆盖；[raw](core-suite-output.txt) |
| r11a-check 原56例完整重放 | 55过/1失败，12.87s，exit101；唯一失败为已承认的恢复span=None；[raw](original-replay-output.txt) |
| 原56例+同值数字两条会话+9组合矩阵 | **55过/4失败**，0 ignored，12.60s，exit101；[源码](replay.rs)、[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 原矩阵 | 186/186、528/528、词法空白12/12、合法注释4/4；内含于runner，不重复加到test数 |
| 新同值数字拼写矩阵 | 9组合：7过/2失败，内含于一个失败test |
| 保存的尾段helper原反例 | 仍55..55，期望65..70，exit101；[源码](historical-map-comparison.rs)、[raw](historical-comparison-output.txt)、[历史控制模块](base-span-map.rs) |
| 重嵌入两侧边界补查 | 1过/2失败，exit101；[源码](embedding-boundaries.rs)、[raw](embedding-boundaries-output.txt) |
| 本次两个修改Rust文件 rustfmt --check | exit0；[raw](fmt-output.txt) |
| `git diff --check af462a947a0a4c08bbc42d3dd3afbd67fdb5c072 HEAD` | exit2：本提交复制的旧r11a报告输入示例带尾空格；[raw](diff-check-output.txt) |

helper 源码沿用原始打印标签 `current r9a`，但include的是当前r12a工作树span_map.rs，实际文件由baseline哈希绑定；历史模块为r8a控制，不是本轮受审基面。helper/边界独立结果不并入会话runner总数。

旧pinned/resume、内容历史、恢复metadata、删除/撤销、递归、intrinsic/授权正反控保持。原17例的进程内时延门本轮通过，不抹除历史波动，也不替代F12正式worker/IPC/内存门。

## F04-r / P1：同值token的新起点仍被字节后缀替代

位置：`experimental/ac-av-probe/core/src/span_map.rs:34–86` 的prefix/suffix及匹配重锚逻辑。

`same_value_leading_zero_literal_maps_to_token_start_not_matching_suffix`：原TRAP_SRC的main表达式`1 / 0`位于`70..75`。将首个`1`换成`01`，完整候选cold warmup成功，edit成功，源码与候选逐字节一致；`Arc::ptr_eq`确认仍复用同一main代码。

```text
actual:   func#1 71..76
expected: func#1 70..76
main map: [(29,70,29),(70,77,71)]
```

当前算法把新数字中的后缀`1`当成旧token的首字节，并将start向右移动1；但当前数字token是完整`01`，其来源起点仍为70。cold语义数值与旧代码相同，指令可复用，但该指令的来源范围必须覆盖当前完整字面量。

新增9组合矩阵遍历初始/目标拼写`{1,01,001}`，每次独立warmup、候选cold warmup和真实edit。7过/2失败：`1→01`错误`71..76`（期望`70..76`），`1→001`错误`72..77`（期望`70..77`）。包括同拼写no-op及反向收缩正控，不是所有数字编辑都红。

这属于已有source scalar profile：`ac-core/src/source/parser.rs:1420`将十进制digits解析为数值并保留整个token span；未新增基数、后缀、浮点或数值范围。所有候选在本轮真实cold warmup接受。不是因新测例而增加语言需求。

修复不能把几何字节存活或相同字节后缀视为完整语义来源。复用代码所引用的字面量/表达式应获得当前受检投影的完整来源事实；文本匹配即便用作优化，也不能成为唯一来源真值。沿用F04-r，关联AC05/V02，T02/T07。

## F04-u / P1：旧helper仍红；两侧保留和尾段位移均有错误

位置：`core/src/span_map.rs:228–232`。r12a没有修改head/tail的互斥`else if`，所以同一段跨过编辑中段时仍只能保留head。该原反例已从r9a连续保存，不是本轮新增边角要求；通过更多全文重锚路径只能绕开它。

### 真实会话可达

`same_value_literal_edit_then_trivia_rewrite_reaches_tail_loss`：

```text
fn helper() int { return 1 }
fn main() int { var y = 1; return 1 / 0 }
fn tail() int { return 9 }
```

1. 先将`var y = 1`中的`1`换成`01`。候选先cold warmup，edit成功，后方trap的该轮位置与冷编相同。
2. 将后方`1 / 0`的首个`1`换成` 1 `。候选同样先cold warmup，edit成功且当前源等于候选。
3. 全文token拼写不能匹配同值`1/01`，走apply_aligned fallback；本次literal可重嵌入，进入未修分支。

```text
actual:   func#1 65..66
expected: func#1 65..71
main map: [(29,53,29),(53,63,54),(63,64,65)]
```

两轮都合法、语义数值不变，第一轮后方来源控制通过。第二轮是真实执行输出被截成首个token，不能把F04-u描述成不再被生产路径调用的helper缺陷。

### 独立边界诊断

新增`embedding-boundaries.rs`仅测试helper合同，与真实会话证据区分：current `[8,9)` 的旧`2`替换成` 2 `，后方mint span`[15,20)`应整体移2，输出`17..22`。

| 存活分段布局 | actual | expected |
|---|---|---|
| 同一段有head和tail：identity `[0,20)` | `10..10` | `17..22` |
| 仅有tail：identity `[8,20)` | `16..21` | `17..22` |
| 仅有head：identity `[0,9)`，比较未改前方`[2,6)` | `2..6` | `2..6` |

第二行排除了head互斥遮蔽：即使tail分支执行了，它用`mid_c0+k+mid_old.len()`作为tail当前起点，只到“嵌回旧token的末尾”，未计替换产物后部新增空白；正确位移需依据完整替换后的输出边界。故仅将`else if`改成两个独立判断也不足以修复整个分支。

区间手术应同时守住左右存活字节、完整输出长度及current/mint关系；需同时重放旧helper、实际fallback消费和这组边界，不再只报告被全文重锚绕开的会话最小例。沿用F04-u，关联AC05/V02，T02/T07。

## 原例关闭、恢复与诊断要求

r11a两条注释会话及4组合实际闭合，comment-aware修复对此有效；原50条永久回归的绿色已核实。F04-t旧mint文本生产点和F04-s恢复history初始化原例继续PASS。但F04-r完整来源对应仍FAIL，F04-u原helper及实际消费仍FAIL。

恢复实际执行仍为DivByZero/span=None，失败发生在edit前；独立metadata例继续PASS。保留F10/F11/恢复来源工作面OPEN，不归因r12a新增，也不以metadata修复冒充runtime来源完整。

同一helper反例连续未闭，说明“新最小例通过”与“机制闭合”仍混淆。下一次应先给出来源事实的语义身份、完整区间左右保留及输出长度守恒的修正说明，再一次性重放原56、保存helper、同值字面量/交错会话/9矩阵及三种helper边界。不得把同值字面量编辑记录为范围外需求，也不得仅继续拓宽绕开错误分支的fast path。

AC05来源子项FAIL、V02FAIL、V07恢复来源FAIL；V05已测pinned/resume子项PASS。其余AC保留历史状态，未授整项PASS；AC09 needs_fix、AC10 pending。Plan保持executing0/11；T02/T07相关来源/历史项不得关闭。F01真函数级前端、F02安装payload、F05b取消/关闭、F09资源、F10 A07/依赖/GC/恢复sidecar、F11持久化/manifest、F12正式worker/IPC/内存、F14文档/SD等OPEN不变。

[冻结Plan](plan-input.md)、[冻结SD-01..06](spec-delta-input.md)、规范/输入/依赖/二进制哈希见baseline。SD未做完整最终验收；未写canonical/ledger，不把合同缺口降为已批准延期debt。

健康：修改Rust文件格式通过。diff-check红来自旧r11a报告中保留的输入尾空格，非Rust格式错误；可在报告中改用引号/转义呈现，原重放输入应保持原样。既有core shift、session new_len/关联history_entry、SpanInfo/context_revision及shim warning仍在raw，F14未清。测试前后工作树clean、HEAD固定，auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费。未改实现，未跑整档t/tv/tt/tb/tu/taa/tf或docs_gen；最终独立pass后裸t+tv及必要consumer/触面门仍merge-owned pending；未合入、归档。

证据完整保存在C盘，不干预主检出其他会话工作。[待安全回写Plan §9](pending-bookkeeping.md)。next：原树work修F04-r/u并完成仍开放恢复来源面，冻结提交后复审。
