# PLAN-828 r2 — r23a / F05b-C01 受影响面独立复审

2026-10-21。**affected_outcome: PASS，F05b-C01/P2 CLOSED**。F-05b 已测取消/关闭态及关闭拥有引用释放不变量闭合，无新增 finding。整体仍 executing0/11；其他既有必需项未完成，本次不授整份 reviewed 或合入许可。

## 基线与范围

- 用户指定冻结提交 `47894c2e6747c8c06032be7a3d40bf00b90a204d`；原实施树 `D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`，修复基面 `f1cd60778f5438beecc3e35bb4b2afb56a9030fc`，原合同基面 `f79b1b83591310b45b3828dd300e6d368987d6c0`。
- 读取主检出当前 Plan，revision2，work 为 in_progress。reviewer 为本聊天 Codex `/root`，未实施该修复，独立从代码、上轮冻结输入和本轮控制核验；未创建 agent/聊天或声明未知模型身份。
- 实际 diff 三文件：session 关闭拥有引用、core 空终态根构造、永久回归。入口与冻结实现 clean；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 未消费。提交/依赖/Spec/锁/配置/技能/实际测试库及输入哈希见 [baseline.json.txt](baseline.json.txt)，原共享 Plan 见 [plan-input.md](plan-input.md)，结束状态见 [final-state.json.txt](final-state.json.txt)。实施全程只读。

## 不变量与闭合证据

| 拥有者 / 状态 | 本轮独立验证 | 结果 |
|---|---|---|
| 无外部拥有者的当前根 | r22a 原 `close_review_releases_unpinned_current_snapshot_root` 原样重放，close 后 Weak 立即死亡，closed session 仍存活 | PASS；原失败转绿 |
| 无外部拥有者的当前 HIR/code | r22a 原 `close_review_releases_unpinned_current_hir_and_code` 原样重放，close 后两个 Weak 立即死亡 | PASS；原失败转绿 |
| 历史代码 / 外部 pin / 原暂停链 | 原其余四关闭控制原样通过：退役历史 Weak 死亡、外部 pin 正确运行、真实暂停链经 close/drop 后按自身修订与来源完成、取消和全部终态守卫不变 | PASS |
| 关闭后 getter / 空终态根 | 独立新例核非零 revision、entry 保留，code/span_maps 空、find_code/generation_of 缺席、resident/function_names 空；重复 close 保持同一空根；普通及分步公开执行在取消flag两态、步数0/1/10下均受控 Infra，不产生暂停执行或 panic | PASS |
| 外部尚未处置的 PreparedEdit | 独立新例用真实 history-hit 候选，确认复用计数=1。close 后旧根死亡，但候选合法持有的 HIR/code 仍活；分别经 closed commit / closed cancel 消费并拒绝后两个 Weak 死亡，计数不增 | PASS；不误把外部拥有者当泄漏 |
| 最后一个 PausedRun 拥有者 | 独立新例先改前方函数长度并实际暂停调用。close 后旧根/code 由暂停链保活；正常完成的 trap 完整 SpanInfo 与新源冷编一致；真实执行取消也受控结束。两条退出后旧根/code Weak 均死亡，closed session 全程保留 | PASS |

机制已核实：session.rs 的 close 幂等守卫后释放 store、clear 全部 FnRecord，并以 `ProgramSnapshot::vacant` 替换当前根，撤销 session 对实际执行载荷和 HIR/code 的拥有引用；外部 Arc/PausedRun/PreparedEdit 按各自生存期保活。vacant 只保存轻量 entry/revision，空代码、来源图及 merged payload；既有 dispatch 的缺入口守卫在读帧/类型前受控返回 Infra。没有通过空对象重新执行旧程序或破坏私有代码准入。

修复不仅关闭两个旧 Weak 样例，也验证了新空根和访问接口、最后拥有者退出、外部候选共享代码等相邻路径。`src`/语法/接口元数据仍保留，受管预算和完整 store GC 仍属既有 F-09/F-10；本轮不声称关闭后的全部内存为零。

## 实际执行

cwd 均为原实施树；定向单线程。脚本 [run-controls.ps1](run-controls.ps1)链接本轮 cargo 产生的库，路径与 SHA256 已冻结。

| 命令 / 入口 | 本轮实际结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | **86/86，exit0，6.08s**；[raw](permanent-output.txt)。新永久 `f05b_close_releases_unpinned_owned_resources` 实际进入门；此前r22a 24格收编仍执行通过 |
| `run-controls.ps1 -Source close-controls.rs -Filter close_review_` | **6/6，exit0**，两个旧失败原样全转绿；[raw](close-output.txt) |
| `run-controls.ps1 -Source close-adjacent.rs -Filter close_adjacent_` | **3/3，exit0**；[源码](close-adjacent.rs)、[raw](close-adjacent-output.txt)；各组合断言见上表 |
| `run-controls.ps1 -Source original-replay.rs` | **62/62，exit0，8.09s**；186/528/12/4/9/72各矩阵全过；[raw](original-replay-output.txt) |
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1` | **17/17，exit0，8.12s**；[raw](original-suite-output.txt) |
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core --lib -- --test-threads=1` | **22/22，exit0，0.09s**；[raw](core-output.txt) |
| `rustfmt --check --edition 2021 --config skip_children=true` 三个修改文件；`git diff --check f1cd60778 HEAD` | 均exit0；[fmt](fmt-output.txt)、[diff](diff-check-output.txt) |

新增取消组合控制首次输入过短，在既有每1024条指令轮询之前先执行到 trap，因审核输入误假定即时取消而失败；保留 [首次输入](close-adjacent-initial.rs)/[首次输出](close-adjacent-initial-output.txt)。最终输入使用能到达真实轮询点的有界循环，仍断言 Cancelled 和结束后 Weak 死亡。未修改产品代码、降低取消断言或将首次测试错误登记为实现缺陷。

各集合有重叠，不合计宣称独立case总数。core 有新增空根构造，已用当前库独立覆盖其执行入口并重跑 core22；798旧缓存/ABC/绑定消费代码、身份谓词与配置未变，本轮未重跑全部旧兼容集，只保留历史证据，不冒称重新执行。

健康：probe原四warning（SpanInfo、new_len、context_revision、history_entry）仍归 F-14，依赖 shim-metadata 的旧 doc warning 单列；无本轮新增 warning。current 的文档注释有非阻塞重复句，随已有 F-14 文档整理处理，不影响此生命周期结论。

## 规范、状态与路由

- [冻结 SD-01..06](spec-delta-input.md) SHA256 `ed0077a2a9bdf622db527d59bf4e12bbfaefb1df4f51df212768cb041d8a206e`，与 r22a 相同；相关 canonical Spec/锁/配置哈希变化0。frontmatter精确目标路径和GOAL-002保留，revision仍2；没有批准缩面或改canonical/ledger。
- SD-04 的取消/关闭拥有引用规则本轮子集通过，空根/关闭后getter的已实现行为应在 F-14 具体正文中明确。其他未交付内容维持既有状态，不授整份delta pass。
- **F05b-C01 CLOSED**。F-05b本次已测取消、终态守卫、关闭拥有引用释放范围CLOSED；AC-04/06、V-05/06受影响子集PASS。T-06/T-07及AC-09整份验收未整体完成，不能把本次局部通过标记为整份 reviewed。
- 计划保持 **executing0/11**；其余F-01/02/09/10其余(A07/依赖/GC)/11其余(changed-only/manifest)/12/14 OPEN，既有来源闭合保持，无新增阻断。下一责任角色为同树 work，按既有指示继续F-01/F-02等必需项。
- 最终代码/验收/规范审查通过后，merge-owned冻结裸t+tv及必要consumer/触面、Spec/ledger/归档/guard仍必需，tf为主检出批量档。本轮未整档、未实施修复、未合入、归档或清树。
