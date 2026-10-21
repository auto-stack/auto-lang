# PLAN-828 r9a：文本重锚与恢复初始化核查

2026-10-21，**needs_fix**。r8a-check 的四个修复目标均通过，恢复历史空表 F04-s 的已测 metadata 反例闭合；但 F04-r 的完整语义来源关联仍未闭合，并发现两项新实现缺陷。上轮已承认的恢复 payload `span=None` 仍失败，本轮不把它包装为新增回归。

受审 `ee4d1b97b6150516a7db596fdfa03474076b450e`，修复基面 `e2fc8edc3226cdd57698f716fa342c4c1367ed34`，分支 `plan-828-dev`，工作树 `D:/autostack/.wt/lang-828/auto-lang`。Plan revision 2、executing、0/11。review_scope=final 中用户指定的受影响 finding/就绪核查，不是整份计划最终验收。reviewer：同一 Codex `/root` 独立审查上下文，未实施 r9a、未创建 agent。遵循 auto-plan-review，未改实现、未合入。

## 实际执行

原输入从 C 盘 r8a-check 直接复制到 [original-replay.rs](original-replay.rs)，字节哈希完全一致，见 [baseline](baseline.json.txt)。所有编译/测试 cwd 为上述实施树。

| 检查 | 实际结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 40/40，0 ignored，1.97s，exit0；[raw](permanent-suite-output.txt) |
| 同 manifest `--test function_incremental_828 -- --test-threads=1` | 17/17，14.11s，exit0；[raw](original-suite-output.txt) |
| 同 manifest `-p auto-av-core -- --test-threads=1` | core22/22，0.09s，exit0；doc-tests0 不计覆盖；[raw](core-suite-output.txt) |
| 原样重放 r8a-check 全部45例 | 44过/1失败，10.79s，exit101；唯一失败为已知恢复 span=None；[raw](original-replay-output.txt) |
| 保留原45例，再加5个相邻组合 | **46过/4失败**，0 ignored，8.74s，exit101；3条新反例、1条既有恢复失败；[源码](replay.rs)、[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 原186格和528条三步矩阵 | 本轮均全部通过，已包含于上面 runner 的 test 内，不重复相加 |
| 历史映射 helper 对照 | r8a `65..70`，r9a `55..55`；当前期望断言失败，exit101；[源码](historical-map-comparison.rs)、[raw](historical-comparison-output.txt)、[基面模块](base-span-map.rs) |
| 本次三个修改 Rust 文件 rustfmt --check | exit0；[raw](fmt-output.txt) |
| `git diff --check e2fc8edc3226cdd57698f716fa342c4c1367ed34 HEAD` | exit0；[raw](diff-check-output.txt) |

原三条 token 反例（周围加 trivia、同字节重写后插入、A-B-A 返回）和恢复 history seed 的公开映射断言均 PASS；旧 pinned/resume、历史命中、删除/撤销/替换组合、递归、intrinsic/授权正反控也保持。新加的三次连续长插入与三次等价整表达式重写两条正控均 PASS。不能把原45例写成45/45，或把含已知失败的 aggregate 标记全绿。

原17例的进程内时延门本轮通过，不抹去历史波动，也不替代 F12 正式 worker/IPC/内存门。

## F04-r / P1：内外 trivia 同时变化时，旧子串不存在仍丢锚

位置：`core/src/span_map.rs:34–70` 的 `from_texts` 与 `191–218` fallback，消费点 `probe/src/session.rs:1101–1116`。

`equivalent_rewrite_with_internal_and_external_trivia_keeps_semantic_anchors`：原 TRAP_SRC 的 main trap 为 `70..75`；将完整 `[70,75)` 的 `1 / 0` 换成 `  1  /  0  `。edit 成功，断言 reuse_hit>0，当前源码冷编接受，语义 token 和 HIR 内容相同。

```text
actual:   func#1 70..81
expected: func#1 72..79
map main: [(29,70,29),(75,77,81)]
```

新实现能处理旧表达式整体作为连续字节子串重新出现的例子；此例内部也改了空格，`1 / 0` 不再是新文本的连续子串。`from_texts` 返回 None，`apply_aligned` 也找不到子串，回退几何删除，所有表达式来源再次丢弃。所谓同内容命中并不保证原文本中段连续重嵌入；这是 r8a F04-r 同一不变量的剩余缺口，不是新增语言需求。仅原三例闭合，完整 F04-r 仍 OPEN。

修复需要可信的语义 token/表达式来源对应，包含多个分离 trivia 变化；不能以全文 prefix/suffix 加一个原始子串匹配代替完整对应。关联 AC05/V02，T02/T07。

## F04-t / P1：新 code 的历史项保存了旧版本文本

位置：`probe/src/session.rs:1282–1295`。

`reminted_longer_literal_then_identical_edit_keeps_its_own_mint_source` 使用：

```text
fn helper() int { return 1 }
fn main() int { var y = 1; return 1 / 0 }
fn tail() int { return 9 }
```

1. 把 `var y = 1` 中的 `1` 换成 `21`。真实重铸 B code，该轮与冷编一致，trap `64..69`。
2. 再将同一个 `21` 原字节替换成 `21`。断言当前源码逐字节未变，且 `Arc::ptr_eq` 确认同一 B code 被复用。
3. 当前 trap 却变成 `65..70`，冷编仍为 `64..69`。

commit 为新 content_version 插入历史时，从 `self.parse/self.src` 提取文本；这仍是安装前 A 的声明，稍后才推进到 prepared.new_parse/new_src。于是 B code、B 的 mint origin/map 与 A 文本被装成一个元组。下一次内容命中的 `from_texts` 又把 A→B 的位移应用于已经按 B 降低的指令，额外偏移一字节。

`history_insert` 对已有 key 使用 `or_insert`，所以原 content 的连续空白修改正控仍可通过，不能靠这些正控排除此缺陷。问题在新 content 首次入表时的归属：重铸 code 必须和实际铸造源文本同属一个版本；命中则保留该 code 自身的 mint 文本，不能拿上一版或当前显示文本替代。关联 AC05/06、V02/07，T04/T07。本轮新增的文本身份字段和生产点存在实质迁移遗漏。

## F04-u / P1：重嵌入中段只保留 head，丢弃同段 tail

位置：`core/src/span_map.rs:204–208`。

`reminted_code_then_equivalent_earlier_literal_rewrite_keeps_tail_segment` 使用同一 var y 程序：先把 `1` 改为 `2`，新 code 与冷编通过；再把这个 `2` 换成 ` 2 `，内容命中复用。后方 `return 1 / 0` 的文本完全未变，仅整体后移2。

```text
actual:   func#1 55..55
expected: func#1 65..70
map main: [(29,53,29),(53,54,54)]
```

在 `apply_aligned` 的连续 mint 区间重嵌入分支，同一个原分段常常同时跨过替换区间左右两侧。代码使用 `else if s0_m < m0` / `else if s1_m > m1`，保留 head 后不会再保留该段 tail，导致后方 trap 的全部来源消失；map_span 最后的 clamp 只把错误表现为零宽，并没有修复来源。

历史 helper 对照直接编译上轮冻结 `SpanSegments`：同一 identity `[29,70)`、编辑 current `[53,54)`→3字节，旧 `apply` 映射后方 `[63,68)` 得到正确 `65..70`；新 `apply_aligned(...,b"2",b" 2 ")` 得到 `55..55`。这是 helper 层确认的修复回归，加上当前公开会话的真实失败；未声称历史完整会话已重编重放。

区间手术必须独立保留左右两侧全部存活分段；需同时验证 head-only、tail-only、两侧都有以及多段/间隙的组合。修好旧文本归属可能让此具体会话路径避开 fallback，但不会修复已证实错误的公共 helper；两项不能互相遮蔽。关联 AC05/V02，T02/T07。

## 既有恢复来源失败保持开放

`restored_same_content_edit_initializes_nonempty_source_alignment` 在真实 restore 后、尚未编辑时仍得到 `Trap { kind: DivByZero, span: None }`。它未到达后续编辑断言，本报告不虚构该分支已运行。独立 metadata test 则实际继续执行首次插入，证明 F04-s 空表问题的原例现已通过。二者范围不同。

本轮 work 在主 Plan 明确承认 spanless 持久化 payload/sidecar 尚未完成。保留 F10/F11 和 V07 恢复来源面 OPEN，不当作 r9a 新引入，也不将 metadata 修复等同恢复运行来源正确。

## 诊断、验收和后续

本轮验证了新增机制的三条独立约束：语义同内容不等于原文本为子串；code/text/map 必须同一 mint 修订；重嵌入要保留所有未覆盖字节。此前依靠四个最小输入和总数，未覆盖这些新分支；已多轮失败，下一轮应先给出这三条机制约束的修正说明，再重放原45、新三个反例和两条正控及矩阵，不能仅增加特判或只收编本轮三个最小例。

AC05 来源子项 FAIL、V02 FAIL、V07 恢复来源仍 FAIL；V05 已测 pinned/resume PASS。其余 AC 保留历史状态，不以本轮定向测试授整项 pass；AC09 needs_fix、AC10 pending。Plan 仍 executing 0/11，T02/T07 及相关历史/来源验收不得关闭。F01 真函数级前端、F02 安装 payload、F05b 取消/关闭、F09 资源、F10 A07/依赖/GC/恢复 sidecar、F11 持久化与 manifest、F12 正式 worker/IPC/内存、F14 文档/SD 等 OPEN 不变。

[冻结 Plan](plan-input.md)、[冻结 SD-01..06](spec-delta-input.md)、规范/依赖/二进制哈希见 baseline。本轮未完整最终验收 SD，未写 canonical 或 ledger。所有已知合同内缺口保持工作义务，不降格为已获准延期的 debt。

健康：本次修改文件格式与 diff-check 均通过。新增未处理 warning 为 core 的 `shift`、session 的 `new_len` 与未使用的关联 `history_entry`；原 SpanInfo/context_revision 及 shim warning 仍在，原 old_src warning 已消除。F14 健康面仍未清。

测试前后工作树 clean、HEAD 固定；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 未消费。未改实现，未跑整档 t/tv/tt/tb/tu/taa/tf 或 docs_gen；最终独立 pass 后的裸 t+tv 与必要 consumer/触面门仍为 merge-owned pending。未合入、归档。

本包完整保存在 C 盘；不干预共享主检出的其他会话工作。待安全回写 Plan §9 的记录见 [pending-bookkeeping](pending-bookkeeping.md)。next：原树 work 修 F04-r/t/u 及仍开放恢复来源面，冻结提交后复审。
