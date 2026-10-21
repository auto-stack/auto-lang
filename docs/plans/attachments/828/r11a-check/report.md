# PLAN-828 r11a：词法对齐与未修尾段分支复审

2026-10-21，**needs_fix**。上轮两条真实空白编辑反例及12格矩阵已通过；但保存过的 F04-u helper 原例仍然失败，源码中相应互斥分支未改。新扫描器尚不等于已有源码 lexer：合法的括号内行注释既能让来源锚落入注释，也能使真实会话再次触发尾段丢失。F04-r/u 仍未完整关闭。

受审 `af462a947a0a4c08bbc42d3dd3afbd67fdb5c072`，修复基面 `e6e8cdc9ff41cc30a5266a6192f8b06b09553050`；`D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`。Plan revision2、executing0/11。review_scope=final 中用户指定的受影响 finding/就绪核查，不是整份最终验收。reviewer 为同一 Codex `/root` 独立审查上下文，未实施 r11a，未创建 agent。使用 auto-plan-review；未改实现、未合入。

## 输入有效性与实际执行

原 r10a-check 的53例直接从 C 盘复制为 [original-replay.rs](original-replay.rs)，字节哈希保持一致；[baseline](baseline.json.txt) 绑定版本、源码、规范、依赖、输入、运行库和二进制。

本轮新增的最终行注释例**全部先完整 cold warmup 成功，再调用 edit，且在相同候选源码上比较 trap**。源码 parser 明确规定未加括号表达式不跨行、括号内可跨行；最初一组未加括号探针不合法，已排除其产品结论和验收计数。该次探索原始 [源码](invalid-newline-probes.rs)/[日志](invalid-newline-probes-output.txt) 单独保留，不能拿它们的拒绝证明源码支持面失败，也不能把未抵达的分支算作已测。最终合法版本是 [replay.rs](replay.rs)。

所有编译/测试 cwd 为上述实施树。

| 检查 | 实际结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 47/47，0 ignored，3.21s，exit0；[raw](permanent-suite-output.txt) |
| 同 manifest `--test function_incremental_828 -- --test-threads=1` | 17/17，14.63s，exit0；[raw](original-suite-output.txt) |
| 同 manifest `-p auto-av-core -- --test-threads=1` | 22/22，0.09s，exit0；doc-tests0 不计覆盖；[raw](core-suite-output.txt) |
| 原53例完整重放 | 52过/1失败，12.07s，exit101；唯一失败为已承认的恢复 span=None；[raw](original-replay-output.txt) |
| 原53例+两条合法注释会话反例+4组合矩阵 | **52过/4失败**，0 ignored，7.79s，exit101；[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 内含的原矩阵 | 186/186、528/528、词法空白12/12，均全过；不与 test 数相加 |
| 新合法行注释矩阵 | 4组合：0过/4失败，内含于一个失败 test，不重复计数 |
| 保存的 helper 反例原样重放 | 当前55..55，期望65..70，exit101；[源码](historical-map-comparison.rs)、[raw](historical-comparison-output.txt)、[历史模块](base-span-map.rs) |
| 两个修改 Rust 文件 rustfmt --check | exit0；[raw](fmt-output.txt) |
| `git diff --check e6e8cdc9ff41cc30a5266a6192f8b06b09553050 HEAD` | exit0；[raw](diff-check-output.txt) |

helper 固定源码的打印标签 `current r9a` 沿用原文，本轮 include 的是当前 r11a 工作树 span_map.rs，受审哈希绑定该实际文件；历史 r8a 模块为控制，不是新受审基面。旧 helper 结果不与真实会话 runner 的 test 数混加。它仍是上轮已保存失败输入，不能因新增永久测试总数绿而忽略。

原17例的进程内时延门本轮通过，不抹去历史失败，也不代替 F12 正式 worker/IPC/内存门。旧 pinned/resume、递归、intrinsic/授权正反控、连续编辑/撤销/历史命中等正控均保持。

## F04-r / P1：可把注释文本当作可执行 token 的来源

位置：`experimental/ac-av-probe/core/src/span_map.rs:54–86,320–350`；现有 `from_texts` 首先进行裸字节子串匹配，新的 `runs()` 则只识别数字、标识符、单字符标点和空白，没有采用现行源码 lexer 的行注释规则。

`valid_line_comment_text_cannot_mint_expression_anchors` 的初始程序为原 TRAP_SRC 中 main 改为 `return (1 / 0)`，其余不变。把括号内的 `1 / 0` 替换为：

```text
 1 // 1 / 0
 / 0 
```

先 cold warmup 完整候选源码并执行，确认括号内的跨行表达式合法；再真实 edit，断言成功、reuse_hit>0、源码与候选逐字节相同。

```text
actual:   func#1 77..82
expected: func#1 72..87
main map: [(29,71,29),(71,76,77),(76,79,88)]
```

原始 `1 / 0` 的第一个连续字节匹配位于 `//` 注释里。`from_texts` 在识别词法 token 之前就用它重锚，实际 trap 被指向注释中的1/0，真实表达式的源范围跨过换行直到后方分母，冷编为72..87。

新增4组合矩阵统一使用括号内候选，依次为：

```text
 1 // note\n / 0 
 1 / // note\n 0 
 1 // 1 / 0\n / 0 
 1 // 来源\n / 0 
```

这里 `\n` 表示源码中的实际换行。四例都 cold warmup 成功、edit 成功；四个位置比较都失败，完整 actual/expected 在 raw。普通注释和 UTF-8 注释走 token 不匹配的几何回退，含旧表达式的注释则走错误子串重嵌入。

现行 `ac-core/src/source/lexer.rs` 明确跳过 `//` 任意 UTF-8 注释，scalar grammar 也注册了 line-comment trivia。最终输入没有用被排除的块注释、字符串或新语言能力。因此这是既有 F04-r 来源不变量的剩余缺口，不是扩展支持面的新要求。

修复需让所有锚点匹配遵守真实词法/语义来源边界：仅在 `runs()` 中补跳过注释仍不够，位于它之前的原始子串快路同样会误锚。关联 AC05/V02，T02/T07。

## F04-u / P1：同一个区间手术错误仍可从合法会话到达

位置：`core/src/span_map.rs:228–232`。本提交再次只扩大 from_texts 的成功范围，没有修改 `apply_aligned` 的互斥 head/tail 分支。先保留 head 后，无法再保留同一段的 tail。保存的 helper 仍得到55..55而非65..70；不是本轮新发明的辅助 API 要求。

`valid_comment_then_literal_rewrite_reaches_tail_loss_in_real_session`：

```text
fn helper() int { return 1 }
fn main() int { var y = (1 + 2); return 1 / 0 }
fn tail() int { return 9 }
```

1. 将括号内 `1 + 2` 替换成 `1 // note\n+ 2`（实际换行）。完整候选先 cold warmup，edit 成功，该轮后方 trap 与冷编相同。
2. 将后方 `1 / 0` 的第一个 `1` 换成 ` 1 `。候选也先 cold warmup，再 edit 成功，发生内容复用。
3. 由于全文含已插入注释，简化 token 匹配不能建立整体对应，走 apply_aligned fallback；本次 literal 的重嵌入覆盖一个跨两侧的存活段，tail 又被丢掉。

```text
actual:   func#1 78..79
expected: func#1 78..84
main map: [(29,56,29),(56,69,64),(69,70,78)]
```

该例的冷编、edit、第一轮正确性和最终错误均实际执行；不同于已排除的未加括号探针。r11a 的12格空白对齐修复使上轮会话输入避开 fallback，但没有消除该分支的错误或可达性。把两类问题说成“同根消除”与当前 helper/运行证据不符。

必须修复区间手术本身对两侧存活分段的保留，并同时守住公开 helper、history map 和当前会话消费路径；不能只继续拓宽不经过它的路径。关联 AC05/V02，T02/T07。沿用 F04-u，保持 OPEN。

## 关闭表、边界和状态

| 既有项 | 本轮 |
|---|---|
| F04-r 原运算符空白例及12矩阵 | PASS；完整词法来源不变量仍 FAIL |
| F04-u 原交错空白会话例 | PASS；helper 原例 FAIL，合法注释交错会话 FAIL |
| F04-t mint文本生产点 | 本轮原例继续 PASS，保留闭合 |
| F04-s 恢复 history 初始化 | 独立 metadata 原例继续 PASS |
| 恢复实际执行 span=None | 仍 FAIL，既有 F10/F11/来源恢复面 OPEN，不归因本轮新增 |

恢复原例在 edit 前便无span，后续分支未被该 test 执行；独立 metadata test 则已验证首次 edit 的map。不能混称恢复来源已完整通过。语法无效的首次探索输入已单列排除，未计为新增产品 finding。

诊断应从“有限词法近似让旧例走成功分支”转为“来源锚点与完整已支持词法一致、几何手术独立正确”。已连续多轮指出 F04-u，不能仅将新会话最小例加入回归却不重放原 helper。下一轮至少重放原53、保存 helper、最终两个合法注释会话和4组合；不得只报告47项永久套件或旧12矩阵来关闭不变量。

AC05 来源子项 FAIL、V02 FAIL、V07 恢复来源 FAIL；V05 已测 pinned/resume子项 PASS。其余 AC保留历史状态，未授整项PASS；AC09 needs_fix、AC10 pending。Plan保持executing0/11；T02/T07及来源/历史项不得关闭。F01真函数级前端、F02安装payload、F05b取消/关闭、F09资源、F10 A07/依赖/GC/恢复sidecar、F11持久化/manifest、F12正式worker/IPC/内存、F14文档/SD等OPEN不变。

[冻结Plan](plan-input.md)、[冻结SD-01..06](spec-delta-input.md)、规范/源码/输入/依赖/二进制哈希见baseline。SD未做完整最终验收；未写canonical/ledger，未将合同缺口降为获准延期的debt。

本次修改文件格式/diff-check通过；既有 core shift、session new_len/未用关联history_entry、SpanInfo/context_revision及shim warning保留于raw，F14仍开放。测试前后实施树clean、HEAD固定，auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费。未改实现，未跑整档t/tv/tt/tb/tu/taa/tf或docs_gen；最终独立pass后裸t+tv及必要consumer/触面门仍merge-owned pending；未合入、归档。

证据完整保存在C盘；不干预共享主检出其他会话工作。[待安全回写Plan §9](pending-bookkeeping.md)。next：原树work修F04-r/u并完成仍开放恢复来源面，提交后复审。
