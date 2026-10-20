# PLAN-828 r6a：删除、边界与逆映射复合核查

2026-10-21，**needs_fix**。r5a-check的三个失败例现已通过；**此前已通过的撤销正控出现回归**，另有两条插删替换组合路径失败，F04尚未完整修好。

受审 `5bdc4d1f5fb71255b418291190f23a775c296a1f`，修复基面 `0b936f900c223018c3c6edc48ddcdca51fb4d381`，`plan-828-dev`，`D:/autostack/.wt/lang-828/auto-lang`；[baseline及输入/二进制哈希](baseline.json.txt)。reviewer为同一独立上下文Codex `/root`，未实施r6a。按用户本轮明确范围检查原三例及删除、插入边界和复合，不把其他仍in_progress的Plan任务当完整交付。

## 执行与覆盖

全部测试cwd为实施树；原29例输入/断言未降低，新增单独的暂停旧snapshot正控避免被撤销断言提前截断。

| 检查 | 本轮实际结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 永久24/24通过，0 ignored，0.21s |
| 同manifest `--test function_incremental_828 -- --test-threads=1` | 原17/17通过，0 ignored，13.34s；[raw](original-suite-output.txt)，包括本轮进程内时延测试通过；不回溯抹去r5a的历史波动或代替正式F12门 |
| 独立重放r5a-check全部27例+两条组合反例+独立暂停旧快照正控 | **27通过/3失败**，exit101，0.22s；[源码](replay.rs)、[runner](run-replay.ps1)、[raw](replay-output.txt) |
| 三个修改Rust文件rustfmt --check | exit0 |
| `git diff --check 0b936f900 HEAD` | exit2，新提交的两份旧日志EOF空行；[raw](diff-check-output.txt) |

r5a原三条（起点插入、删除后编辑后方、编辑旧插入文本内部）均过；但r5a集合总计为**26过/1旧正控回归**，不能写成整个r5a集合全闭。新增2组合反例失败、独立旧snapshot恢复正控通过，合计30例27过3失败。重复永久/独立执行不累加成唯一覆盖数。

已实际覆盖：纯插入起点/区间内部/终点、连续长短插入、先前后后前顺序、原生空白删除后追加编辑、纯插入产物内继续插入、内容A→B→A、公开snapshot、暂停旧snapshot在后续撤销后恢复、intrinsic身份/内容/effect正反控。剩余失败集中在**删除插入/替换产物及撤销复合**；完整恢复/GC/授权/正式IPC等既有开放任务未授pass。

源码检查和测试前后HEAD固定、实施clean；此后仅新增本核查目录。保留原有3个probe warning及shim既有warning记录；未跑整档、docs_gen、正式IPC/内存门或重复其他workspace兼容全集。主检出计划、KNOWN-DEBT、canonical/ledger及其他合并工作不写。

## F04-l / P1：上轮撤销正控回归，span反向

`undo_and_paused_snapshot_keep_their_distinct_source_maps` **直接复制自已保存的r5a-check**，本轮没有更改它。输入为原helper/main/tail小程序，main中的`1 / 0`会trap。

1. 在main的return前插入32空格，真实edit成功；捕获该新snapshot并暂停一条执行链。
2. 删除刚插入的32空格，断言源码字节已精确回到原TRAP_SRC。
3. 当前session结果与原源码冷编比较：

```text
actual:   func#1 102..75
expected: func#1 70..75
```

这条在r5a通过，r6a发生**新修复回归**；不是新加产品要求。旧用例在当前undo断言处失败，未抵达其resume分支。为避免覆盖误报，本轮另执行 `paused_snapshot_retains_source_map_after_later_undo_control`：同样暂停旧快照、后续撤销，再独立resume，确认旧快照来源仍与其自身冷编等价，**该正控通过**。故问题是当前修订撤销后的映射，不是这条已测旧snapshot保活/映射分支。

## F04-m / P1：部分删除新增空白，把当前长度错当mint长度

`deleting_part_of_inserted_whitespace_does_not_delete_mint_tokens`：在main的return前插32空格，第一轮与冷编一致；再删除这段新空白中间16字节（第8..24字节），源合法、IR内容未变。

```text
actual:   func#1 102..91
expected: func#1 86..91
```

`compose_region`只逆映射起点，却把current-coordinate的old_len=16原样保存成mint-coordinate删除长度。实际删除的16字节全是之前新增文本，在mint中不应删掉任何原始字节。新的`map_span`用`p+len<=start`决定是否移动起点，因虚构的mint删除区间跨过原token起点而漏减16；end却减了16，产生start>end。

完全撤销同理：mint插入事件+32和误建的32字节mint删除事件对start/end作出不同判断，因此撤销不能成为恒等映射。

## F04-n / P1：在扩长替换产物内删除，破坏未改前置函数

`expanded_whitespace_replacement_cannot_move_an_unmodified_earlier_trap`：

```text
fn helper() int { return 1 / 0 }
fn main() int { return helper() }
fn tail() int { return 9 }
```

1. 将main的return前**已有1空格替换成64空格**。helper在它之前，helper代码和来源不应变化；实际第一轮与冷编均25..30（合法替换正控）。
2. 删除这64空格内部第32..48字节，仍只改变后方main的空白。

```text
actual:   func#0 25..14
expected: func#0 25..30
```

`unmap_pos`只对`d>0 && len==0`的纯插入识别新增区间；扩长替换`len>0`的产物没有对应规则。逆映射把后方main内部的编辑位置折回到了无关前置helper的区间，所有函数复用的来源事件随之被污染，出现反向span。这是现有同profile合法编辑的来源隔离失败，沿用AC-02/05/06、V-02/05/07及原F04；不新增规模或语言要求。

## 根因与修复要求

目前仍是“只逆映射一个起点，然后携带当前old_len”的delta事件模型；它无法区分删除的是mint原文还是之前插入/替换产生的字节。给start/end增加边界判断修复了r5a原三例，却暴露了旧撤销控制的回归。

需要先明确并实现**当前片段到mint/新增片段的来源关系**，对整个编辑区间作分解/投影，而非只映射起点：

- 删除纯新增文本应缩短/取消相应新增片段，不能伪造mint删除区间；撤销插入后映射应回到恒等。
- 扩长/缩短/等长替换产物都要有插入区间与边界关联规则，逆映射不能仅处理len==0。
- 保持snapshot不可变、start/end与实际token关联、公开execute/steps/resume一致，以及未改代码payload/对象不动。映射必须与冷编oracle一致；交换或夹紧反向端点不构成修复。
- 回放**整个**已保存r5a集合（含旧正控），并保留本轮部分删除、替换内部删除与独立旧快照控制。已修的intrinsic正反控继续保留；不能通过拒绝合法编辑或强制整模块重编规避。

## 路由

F04-i/j/k原失败例关闭保持，l为旧正控新回归，m/n为同一删除/替换复合不变式的未闭路径；F04整体仍OPEN。intrinsic稳定身份及其他既有通过控制无新增失败证据。主计划继续executing，其他已承认OPEN和完整AC/SD/merge-owned门不变。

本轮不修实现、不提交或合入。报告、raw和待回写收据留在原828工作树，后续对最新主计划增量对账，不用工作树旧Plan覆盖主检出。多轮后的根因/表示/消费路径诊断要求仍有效：下次就绪应先对账来源片段与完整区间复合规则，再证明全套正反控，而非仅收编新增失败例。

> 存储收尾：复制到D盘工作树时，replay.rs因磁盘空间不足未成功，D盘目录暂不完整。完整权威证据包保存在 C:/Users/zhaop/.codex/visualizations/2026/10/20/01a159ce-3d48-79f1-a573-f0b203b80273/r6a-check/；未删除用户文件或修改实现。
