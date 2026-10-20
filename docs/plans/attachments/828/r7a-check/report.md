# PLAN-828 r7a：r6a 原例重放与区间复合核查

2026-10-21，**needs_fix**。C 盘 r6a-check 的三个失败例全部闭合，原包 30/30 通过；但区间复合尚未完整修好。本轮独立补查的 8 个具体组合全部失败，另一个有界矩阵含 186 个合法组合，其中 26 个失败。最终扩展 runner 为 **30 过 / 9 失败**（矩阵计为一个 test，不能再与其子例相加）。

受审提交 `099766b83d998b3fae8d41f01fb5415645288f5e`，修复基面 `5bdc4d1f5fb71255b418291190f23a775c296a1f`；分支 `plan-828-dev`，工作树 `D:/autostack/.wt/lang-828/auto-lang`。Plan revision 2，当前 executing、0/11。本轮是用户指定的受影响 finding 核查，不是整份计划最终验收。reviewer 为同一 Codex `/root` 审查上下文，未实施 r7a；未创建其他 agent。遵循 `/auto-plan:review`，未改实施、未合入。

本包在 C 盘为完整权威副本；原输入直接取自同一父目录下 `r6a-check/replay.rs`，其 SHA256 与本包 `original-replay.rs` 完全一致：`7823bc972f066154bf2e8c6b551122cec5a3c22064b539849dee7397a0436bcd`。未使用可能不完整的 D 盘副本。

## 实际执行

所有编译/测试 cwd 都为上述 Plan 工作树。哈希、版本、依赖和命令结果见 [baseline](baseline.json.txt)。

| 检查 | 实际结果 / 证据 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 27/27，0 ignored，0.21s，exit 0；[raw](permanent-suite-output.txt) |
| 同 manifest `--test function_incremental_828 -- --test-threads=1` | 17/17，0 ignored，8.05s，exit 0；[raw](original-suite-output.txt) |
| 原封不动重放 C 盘 r6a-check 的 30 例 | 30/30，0 ignored，0.22s，exit 0；[原始输入副本](original-replay.rs)、[raw](original-replay-output.txt) |
| 保留原 30 例并补充组合检查 | 30 过 / 9 失败，0 ignored，1.77s，exit 101；[源码](replay.rs)、[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 新矩阵内部计数 | 186 个组合：160 过 / 26 失败；上行一个 test 内的子例，逐个失败输入/actual/expected 在 raw 中 |
| 三个修改 Rust 文件 `rustfmt --check --edition 2021 --config skip_children=true` | exit 0；[raw](fmt-output.txt) |
| `git diff --check 5bdc4d1f5fb71255b418291190f23a775c296a1f HEAD` | exit 2：提交的两份 r6a 旧日志存在 EOF 空行；[raw](diff-check-output.txt) |

runner 以当前 Cargo 产物链接 Rust 公共 API，真实执行 `warmup → edit → run`；不是只测映射 helper。每个组合的第一步与冷编先对照通过，再执行第二/第三步并与该轮完整当前源码独立冷编对照。所有输入仅改空白，源码仍合法，语义 token 不变；不能以词法或新语言面解释失败。原套件本轮进程内时延门通过，不抹去历史波动，也不代替 F-12 正式 worker/IPC/内存门。

## 原三例闭合与覆盖边界

| r6a 失败例 | 当前结果 | 仍需区别的相邻状态 |
|---|---|---|
| F04-l `undo_and_paused_snapshot_keep_their_distinct_source_maps` | PASS：插 32 再删全部 32，源码和当前 span 恢复；暂停旧 snapshot 的 resume 也通过 | 部分删除后再撤销仍失败（F04-q） |
| F04-m `deleting_part_of_inserted_whitespace_does_not_delete_mint_tokens` | PASS：插 32 后删中间 16，当前 span 对齐冷编 | 修剪后继续插入、继续删除仍失败（F04-q） |
| F04-n `expanded_whitespace_replacement_cannot_move_an_unmodified_earlier_trap` | PASS：后置 main 内的扩长替换产物删除，不再污染前置 helper | 替换产物全段/尾段被覆盖，或修剪后继续编辑仍失败（F04-p/q） |

旧包其余控制也全部通过，包括原生删除后编辑后方、token 起点/区间内部/终点插入、同点连续插入、先后/后前顺序、历史内容命中、公开 snapshot、独立暂停旧 snapshot 恢复、递归及 intrinsic 身份/effect/授权正反控。不能据三条原例或永久 27 例通过，宣布完整 F04 关闭。

## F04-o / P1：整段重写新增文本，忽略本次非空产物

位置：`experimental/ac-av-probe/core/src/program_snapshot.rs:286–292`。

`replacing_entire_insertion_with_nonempty_output_keeps_new_output`：

1. 对原 TRAP_SRC，main 的 `return 1 / 0` 前（当前 63）插入 32 空格；这轮与冷编相同。
2. 用 16 空格替换这段新增文本的完整 `[63,95)`。
3. edit 成功，但 snapshot 三个函数的 map 全变成 `[]`。

```text
actual:   func#1 70..75
expected: func#1 86..91
```

`erased_only_rewritten_output` 只检查旧范围来自被移除区域，未区分本次 `new_len=0` 与非空产物。它把“删除新增文本”直接扩成了“替换新增文本也完全不留痕迹”。新文本仍存在，却不再具有位移记录。矩阵中的“在原四空格之后插 8，再将这 8 个字节替换为 1/8”也失败。

修复应让重写后的实际输出进入新映射；单纯加 `new_len==0` 守卫还不能解决下面两类问题。关联 AC-05/V-02，T-02/T-07，沿用 F-828-R1-04。

## F04-p / P1：吸收旧区域后仍按 current 长度差记账

位置：`program_snapshot.rs:270–292`。被完整重写的旧区域先被移除，随后仍无条件使用 `new_len - cur_len` 作为相对 mint 的新 delta，旧 delta 的贡献丢失/重复扣减。

三个独立可复现输入：

| 测试 | 精确流程（每个首步与冷编通过） | actual → expected |
|---|---|---|
| `replacement_spanning_inserted_and_mint_whitespace_restores_original_source` | 在 `1 / 0` 前原有一个空格的起点插 32；把“新增 32 + 原有 1”共 33 空格替换成 1；断言源码字节精确恢复原文 | `38..43 → 70..75` |
| `replacing_expansion_output_with_original_separator_recovers_identity` | 把同一个原生空格扩成 64；再将完整 64 替换回 1；源码字节精确恢复原文 | `7..12 → 70..75` |
| `disjoint_insertions_then_covering_replacement_preserves_current_coordinates` | 初始 `return` 后 4 空格，前后各插 8；完整 20 空格换回 4；源码精确恢复该四空格基线 | `57..62 → 73..78` |

第一例中丢掉旧 +32 后又记 -32；第二例丢掉旧 +63 后又记 -63。源码复原本应等价恒等映射，错误 span 可落到别的语法/函数位置。

边界并非只在整个替换产物被覆盖时出错。矩阵包含“4 空格扩成 12，再仅删除最后 1 个空格”：第二次范围 current `[gap+11,gap+12)` 的右端正好在产物结尾，两个端点的 collapse 策略将整个旧 mint 区间纳入包含判定，旧 +8 被完整移除，actual `72..77`，expected `80..85`。覆盖尾端的半开区间不能据压缩后的两个点推定整块原像已被重写。

需保留未覆盖产物及原始文本的归属，复合时明确吸收旧区域后新映射的净变化；不能把逐步 current 差值当成剩余 mint 表的完整变化。关联 AC-05/V-02，T-02/T-07。

## F04-q / P1：部分修剪记录在后续编辑中被误移除

位置：`program_snapshot.rs:270–279`，以及 `unmap_pos:230–248` 使用同一事件表推导当前像。负 trim 以 `(p,0,-16)` 记录；后续零宽编辑双端塌缩到同一 mint 锚点后，非正插入分支把此 trim 当成“完全被重写区域”移除。实际当前没有删除那 16 字节的反操作，位移却增加 16。

| 测试 | 流程 | actual → expected |
|---|---|---|
| `partial_insert_delete_then_delete_rest_recovers_identity` | return 前插 32，删中间 `[+8,+24)` 16；再删剩下 16，源码精确复原 | `86..91 → 70..75` |
| `insertion_prefix_deletion_then_insertion_keeps_token_boundaries` | trap token 起点插 32，删新增文本前 16，前两轮冷编等价；在当前 token 起点再插 8 | `110..115 → 94..99` |
| `insertion_suffix_deletion_then_insertion_keeps_token_boundaries` | 同上，改删新增文本后 16；独立执行，未被前一例失败遮蔽 | `110..115 → 94..99` |
| `replacement_internal_delete_then_reinsert_compares_each_revision` | 原生一个空格扩成 64，删除产物内部 `[+32,+48)` 16（该轮对齐冷编）；在剩余产物 `+12` 再插 7 | `140..145 → 124..129` |

首尾删除后的真实最终表均变成 `[(70,0,8),(70,0,32)]`，缺少应继续起效的 -16；替换内部删除再插入的表变成 `[(69,0,7),(69,1,63)]`，同样丢掉 -16。raw 已保存这些 runtime snapshot map。

同点多个事件的排序与累加并没有形成可反复编辑的实际输出分段。需要正规化部分保留的插入/替换产物和其来源关系，再做区间复合；不能把“mint 长度 0”统一理解成可由零宽编辑消除。`session.rs:1143/1151` 对驻留记录和所有 history map 都调用这个 helper，修复必须覆盖两条消费路径，同时保留 pinned snapshot 自身坐标。关联 AC-05/06、V-02/05，T-02/T-07。

## 有界矩阵、诊断与下一轮要求

矩阵基线为 main 的 `return` 后 4 空格。三种首步：在空白前插 8、在空白后插 8、4 空格换成 12。均得到长 12 的当前空白；第二步遍历边界 `{0,1,4,6,11,12}` 的所有 `a<=b` 范围，替换长度 `{0,1,8}`，排除删除全部分隔符。每个首步控制通过，随后 186 个合法二步组合中 26 个位置错误。包含空范围、完全覆盖、部分覆盖、插入/替换像内部、两端及混合 mint/新增文本；这不是任意规模压力承诺。

诊断/覆盖调整：此前测试停在“插后删除一次”或“完全删除新增文本”，未验证删除后继续编辑，也未检查吸收旧区域时非空产物及 current/mint 长度转换。当前实现虽名为 full interval surgery，仍只是 collapse 两个点、删若干事件、追加一个 current delta。反例表明这种新路径尚未保持输出归属、守恒与区间边界；本轮不接受全不变量闭合声明。下一次 work 应先给出分段/归属规则及三个根因如何统一消除，再完整重放原 30、全部 8 个具体新组合及矩阵；不能再只收编三个最小反例。

这些是既有“当前来源与冷编一致、编辑撤销为恒等、旧快照自洽”的合同失败，不是新增语言/性能需求；新发现不自动标为本提交独有回归。原 30 正控本轮无回归。

## 状态、规范和未执行范围

- 本轮 AC-05 的来源子项 FAIL；V-02 FAIL，V-05 已测 pinned/resume 子项 PASS；T-02/T-07 与 F04 保持未闭合。其余 AC 只保留既有状态，不以本轮定向测试授整项 PASS；AC-09 本次 needs_fix，AC-10 pending。
- [冻结 Plan 输入](plan-input.md)、[冻结规范增量](spec-delta-input.md)及 main canonical Spec 哈希见 baseline。SD-01..06 未做完整最终审查，沿用既有未闭合状态；新增/替代 component 元数据保持原 Plan，不发布 canonical/ledger。
- F01 真函数级前端、F02 安装 payload、F05b 取消/关闭、F09 剩余资源、F10 A07/依赖/GC、F11 changed-only 持久化与 manifest 恢复、F12 正式 worker/IPC/内存、F14 文档/SD 等既有 OPEN 未因本轮变更而关闭。
- 健康项：既有 probe 三项 warning（SpanInfo、old_src、context_revision）和 shim unused doc comment 仍在 raw；格式通过；旧日志 EOF 空行使 diff-check exit 2。未清理或改写历史证据。
- 测试前后实施树 clean，HEAD 固定，auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 未消费。主检出在另一会话活动中前进；本轮不修改其计划簿记/KNOWN-DEBT/规范/ledger，受审工作树未同步，所读 Plan/Spec 内容哈希复核保持一致。
- 未跑整档 t/tv/tt/tb/tu/taa/tf、docs_gen 或正式 IPC/资源门。最终独立验收通过后的裸 t+tv 与必要触面/consumer 门仍为 merge-owned pending；未合入、未归档。

next：同一实施树 work，修复 F04-o/p/q 并提交，再做受影响面复审。Plan 保持 executing 0/11；本轮记录待安全回写主计划 §9，见 [簿记回写记录](pending-bookkeeping.md)。
