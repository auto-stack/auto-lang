# PLAN-828 r5a 交接修复核验

2026-10-21，**needs_fix（F04完整不变式未闭）**。受审 `0b936f900c223018c3c6edc48ddcdca51fb4d381`，基面 `040061b6144654315f4c3372fcbfedabb7c67a3e`，`plan-828-dev`，`D:/autostack/.wt/lang-828/auto-lang`；[baseline](baseline.json.txt)。reviewer为原独立核查上下文Codex `/root`，未实施r5a。

本轮响应用户r5a完成交接，只核查修复及实际相邻路径；不改标此前最终review历史，不把此次局部交接当完整最终验收。主计划仍executing0/11、work=in_progress，F01/02/05b/09/10/11/12/14明确开放；完整AC/SD及merge-owned门不在本轮授pass。依既有用户约束，证据留原树，不写主检出计划/债务表/合并或canonical/ledger。

## 验证结果

所有测试cwd为上述实施树，源码与HEAD检查前后固定、实现clean。本目录随后只新增核查材料。

| 检查 | 本轮实际结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 永久21/21过，0 ignored，0.21s |
| 上轮独立23例原样重放+3新反例+1撤销/旧快照正控 | **24过/3失败**，exit101，0.24s；[源码](replay.rs)、[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 原suite `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1` | **16过/1失败**，44.33s；t08_v08的P95=3531.541ms≥1000，P50=1301.2/P99=4624.7；[原始失败保留](original-suite-output.txt) |
| 仅复跑同suite `t08_v08_formal_p95_and_structural_dual_gate`，本助手未并行其他Cargo测试 | **1/1过**，10.74s；[raw](perf-replay-output.txt)。波动未归因，不宣称代码回归已证明，也不删原失败/拼成一次17/17 |
| 五个修改Rust文件rustfmt --check | exit0 |
| `git diff --check 040061b61 HEAD` | exit2，两份新增旧日志EOF空行；[raw](diff-check-output.txt)，归F14收据健康面 |

上轮3个失败例全部通过，其他旧23例保持通过；新增撤销并恢复旧source字节、暂停旧snapshot后继续运行仍使用旧来源的正控也通过。未把相同输入在永久/独立套件的重复执行相加为独立覆盖。

未重复scalar差分/798兼容集，不把work报告的总数冒充本轮结果。无整档t/tv/tt/tb/tu/taa/tf、docs_gen或正式IPC/内存验收。旧shim warning、session的unused SpanInfo、unused old_src（全文clone仍执行）、context_revision dead_code保持。

## F06/F08：本轮稳定身份修正的正反控通过

从callee表序号改为owner域下的def id_text后，混合mark_a/mark_b时caller的正常体编辑通过；原symbol在两个def间交换、effect翻转、单个symbol改变、跨域/错签名及同环境控制均通过。没有证据复活这些旧例；当前intrinsic绑定身份修正比r4a正确。

这是对本轮实际改动及已执行控制的结论，不能替代尚未交付的F01受检环境、F09授权/预算、F10依赖/恢复及完整AC05验收。

## F04仍有三个失败路径（P1）

沿用AC-02/05/06、V-02/05/07的source映射/内容复用合同，不增加语言、输入规模或新的性能阈值。三例都是现有tiny scalar程序的合法空白编辑，均真实执行edit、AV和同源冷编oracle。

### F04-i：恰在trap起点插入空格

`whitespace_inserted_exactly_at_span_start_tracks_token_start`：在main的`1 / 0`第一个`1`前插入一个空格，IR内容相同，代码可复用；新源冷编的token起点右移。

```text
actual:   70..76
expected: 71..76
```

`map_pos`对start/end统一采用`region.start < x`，恰在插入点的start不动；而该start对应的真实token已经移到右侧。需要明确位置/区间端点的边界关联。已有“区间内部插入只延长end”控制应同时保持。

### F04-j：删除之后，编辑后方反而移动前方trap

`deletion_then_edit_after_trap_does_not_move_earlier_span`：

1. 初始源在main的`var x=helper()`前预留32空格，完整编译。
2. 删除这32空格，edit安装成功，当前trap与冷编均70..75（删除正控）。
3. 再在`1 / 0`表达式之后插入一空格，冷编仍70..75，该编辑不应移动之前的token范围。

```text
actual:   71..76
expected: 70..75
```

`unmap_pos`的`hi=c`依赖注释所述`map_pos(x) >= x`；删除后映射可小于原位置，正确mint前像可能大于当前c，二分范围排除了它。此外当前前向map忽略old_len、删除区域产生向下跳变，不具备二分要求的单调性。例如一个start=10、delta=-5的区域使f(10)=10、f(11)=6。不能仅扩大hi就声称完整修复。

### F04-k：在先前插入的文本内部继续编辑

`editing_inside_prior_inserted_whitespace_keeps_nearby_anchor`：

1. 在原trap起点前1字节处插入32空格，第一轮与冷编一致（102..107）。
2. 在这段新插入空白的中间再插入1空格。

```text
actual:   102..108
expected: 103..108
```

逆映射当前采用前向map的lower-bound；前向严格`start < x`存在插入跳跃，落在插入文本内部的c被反算到原插入点后1字节，而不是原插入锚点，恰好漏掉相邻token起点的第二次位移。用户所述“插入内部位置塌缩到插入点”尚未实际成立。

## 根因与后续责任

统一mint坐标是正确方向，但当前仍是忽略old_len的delta事件求和，未形成支持删除塌缩、插入区间和端点关联的完整单调分段映射；逆映射的两个前提均不成立。应先给出正向/逆向映射的域、边界关联、插入/删除/替换规则和复合定义，再实现并验证。不是只修`hi`或把`<`统一改成`<=`：后者会把正确的“表达式之后插入不改变end”控制改坏。

下一次至少同时覆盖原23例、上述3路径、当前撤销/暂停旧快照正控，以及删除/替换/插入内部的组合；保持公开执行/steps/resume使用同一个不可变snapshot映射，内容命中绑定真正铸造来源。建议用有明确边界规则的分段映射或逐修订编辑复合，不依赖未证明的二分单调性；不得通过禁用这些合法编辑或强制全量重编降级验收。

F04的旧g/h等原例关闭保持，i/j/k仍OPEN；F03/07/13和本轮intrinsic控制保持。计划继续executing，其他已承认OPEN不变；时延首跑红/单项重跑绿如实保存，需在后续正式性能证据中归因。r5a不是整Plan交付，未做合入/归档/清理。
