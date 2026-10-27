# PLAN-828 r2 — r20a 修复独立审核

2026-10-21，`outcome: needs_fix`。本次是最终审核角色对当前修复及就绪状态的核查，**不是整份计划的最终验收通过**。受审 `e083f2a28ec4d8df05caf5013d502b0c16c0ce0d`。

## 基线与范围

- 主计划读自主检出，revision 2，`executing 0/11`；最新 work 明确 `in_progress`，F-01/02/05b/09/10其余/11其余/12/14 仍 OPEN。因此先核就绪、重放 r19a 原例和相邻来源绑定控制，不再花一轮完整验收确认已承认的未完成工作。
- 原实施树 `D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`，入口与收据冻结时实现 clean；修复基面 `763dbf941c4ddaf767d5942a9436cd03f6bdc999`，原合同基面 `f79b1b83591310b45b3828dd300e6d368987d6c0`。实际修改 cache.rs、session.rs、永久回归文件，未修改其他实施文件。
- reviewer 为本聊天 Codex `/root`，本上下文未实施 r20a；未创建其他 agent/聊天，不虚构独立模型或会话身份。判断来自实际代码与本轮重放，不采用 work 的“全部闭合”作为证据。
- auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，本路径未消费；当前锁/配置/Spec/技能/测试库和输入 SHA-256 见 [baseline.json.txt](baseline.json.txt)，原始共享 Plan 见 [plan-input.md](plan-input.md)。独立 harness 直接链接本轮 cargo 生成的库，选择的精确库路径/哈希已冻结。无实施写入、合入或清树。
- 普通沙箱对原工作树 status 遇权限限制；按已授权审核范围扩权重读后确认 clean，未将受限命令的空输出当作 clean 证据。

## 当前结论与 findings

### F11-S01 / function-id（P2，既有来源不变量仍未闭）

`probe/src/cache.rs:315–318` 只逐行比较映射后的 `start/end`，忽略 `SpanInfo.func`。函数编号是执行器选择 snapshot 来源映射的索引，并非无关诊断字段。

复现 `binding_review_v2_wrong_function_must_not_bypass_source_map`：合法单函数 `fn main() int { return 1 / 0 }` 的真实 unit、cv、owner、mint 和 start/end 原样，只将 v2 sidecar 的 `func` 从 0 改为 `u32::MAX`，确认 mutation 非空。新谓词返回 true。当前源码在 return 前插 16 空格后，restore 准入 1 单元，真实执行返回 **`func=4294967295, 23..28`**；当前源冷编结果为 **`func=0, 39..44`**。执行器 `core/src/av.rs:150` 找不到此函数的映射，直接返回旧坐标。相同坏 primary 后放入完整正确 repair 仍得到错误结果。

旧格式同样 func 变异会被完整 span 表比对拒绝，合法 v2 同源/位移均准入并正确，两个控制通过。因此不是无支持面的新需求，而是 r20a 新增 v2 来源证明漏了实际消费的身份轴。

**要求**：来源证明绑定真实函数身份、当前坐标与所有实际消费字段；不能只证明端点。完整适用性失败后继续尝试 repair。保留错编号无 repair、有 repair、legacy 拒绝及合法 v2 同源/位移控制，并收编真实执行断言。

### F10-S02 / instruction-id（P2，既有选择/写回不变量仍未闭）

同一比较还以 `_` 丢弃两侧指令编号，`sidecar_applicable` 仅检查 trap 覆盖与非倒置区间。把真实表第一行指令 0 改为 1，得到 `[1,1,2]`，覆盖 trap 指令 2、span 区间/mint 全部保持正确，谓词仍返回 true。

- `binding_review_duplicate_instruction_must_not_hide_good_repair`：已存正确 repair，restore 先选了坏 primary，后面的 MIR 验证才发现非严格排序并回冷；实际 `restore_units_admitted=0`，应使用已可用 repair 得 1。
- `binding_review_duplicate_instruction_must_heal_future_restore`：无 repair 时，首轮正确回冷，但 persist 复核 survivor 仍用同一弱谓词，把坏 primary 当有效，未修复；第二次 restore 仍 admitted 0，应为 1。运行值/诊断虽经回冷正确，恢复修复机制未完成。

**要求**：完整准入证明包含指令身份及表形状，restore 的 primary/repair 选择与 persist 的 survivor/repair 判定采用一致且充分的标准。保留两条真实 store 恢复反例；后置验证拒绝不能替代候选选择及修复写回。

两个问题沿用既有 F11-S01 和 F10-S02，不引入新产品能力或规模要求。原 r19a 的两个具体失败已闭合，不能据此宣布完整不变量 CLOSED。

## 本轮实测

所有运行 cwd 为原实施树，`--test-threads=1`。独立脚本在本目录，不写实施代码。所有 harness 的合法控制/真实 mutation 都实际执行；未以 ALL-PASS 摘要替代断言。

| 入口 / 证据 | 实测 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1`；[现场摘录](permanent-suite-observation.txt) | 76/76，exit0，4.91s；四个 r20a 新例实际进入永久门 |
| `run-controls.ps1 -Source applicability-controls.rs -Filter app_review_`；[raw](old-applicability-output.txt) | 原 5/5，exit0；16格选择矩阵全过，r19a 两个失败原例已转绿 |
| `run-controls.ps1 -Source sidecar-generations.rs -Filter gen_review_`；[raw](old-generation-output.txt) | 8/8，exit0；合法重定位、反复 restore/edit/pin、旧格式及损坏 mint 控制保持 |
| `run-controls.ps1 -Source restore-sidecar.rs -Filter sidecar_review_`；[raw](old-sidecar-output.txt) | 6/6，exit0；F10-S01 零函数受控拒绝、缺失/失效写回旧例保持绿 |
| `run-controls.ps1 -Source original-replay.rs`；[raw](old-replay-output.txt) | 62/62，exit0；186/528/12/4/9/72 各矩阵全过 |
| `run-controls.ps1 -Source binding-controls.rs -Filter binding_review_`；[源码](binding-controls.rs)、[raw](binding-output.txt) | **2 pass / 4 fail，exit101**；四失败归上述两个剩余机制，另 81 测试被过滤，未计入本次新覆盖 |
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1`；[raw](original-suite-output.txt) | 17/17，exit0，12.68s；进程内 formal 测试绿不等于未交付的正式 worker/IPC/内存门 |
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core --lib -- --test-threads=1`；[raw](core-output.txt) | 22/22，exit0 |
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test cache_tier --test mir_tier --test abc_semantics_tier --test binding_tier --test vertical --test case_binding -- --test-threads=1`；[raw](compatibility-output.txt) | cache20 / mir8 / abc15 / binding4 / vertical3 / case5 全过，exit0 |
| `rustfmt --check --edition 2021 --config skip_children=true` 本轮三个修改文件；`git diff --check 763dbf941 HEAD` | 均 exit0；[fmt](fmt-output.txt)、[diff](diff-check-output.txt) |

以上集合有重叠，不相加声称独立 case 总数。此前纯 span-map 元数据 harness 未再次全部运行：本轮未修改 span_map.rs，已用其实际 session/restore 消费路径及永久矩阵重放覆盖受影响面，不扩大成完整来源证明。

健康项：probe 仍有四条 work-owned warning：session.rs:46 `SpanInfo` 未用、1399 `new_len` 未用、265 `context_revision` 不读、725 `history_entry` 未用，保留在 F-14。依赖 shim-metadata:213 的旧 doc comment warning 单列，不误报为 r20a 新增。格式和 diff 通过不表示 warning 全清。

## AC / V / SD 与就绪核查

| 项 | 当前证据 / 未完成 |
|---|---|
| AC-01/02；T-01/02/03 | 既有 F-01 仍 OPEN。session.rs:1196–1216 仍遍历全部声明构造签名桩 Program 并 `compile_program_parts`，没有以本轮来源修复替代真正函数级前端 |
| AC-03 | 单函数 MIR/ABC 定向测试保持通过；未据此授予整份增量链路通过 |
| AC-04；T-05/06 | F-02 仍 OPEN。ProgramSnapshot 仍 clone 全部 blocks/slots/spans 构造 merged 执行视图；F-05b 取消/关闭态仍未完成 |
| AC-05；V-02/04/07；T-02/07 | r19a 两个原例闭合；F11-S01/function-id 和 F10-S02/instruction-id 四个新重放失败，完整来源与修复准入仍 PARTIAL/FAIL |
| AC-06；V-06/07/09；T-07 | 已有旧 snapshot/512 修订正控保持；F-09 其余预算、F-10 A07/实际依赖/GC、失败/关闭释放仍 OPEN，历史长度上限不能代替真实资源和释放证据 |
| AC-07/08；V-08/09；T-08/09 | F-12 正式持续 worker/IPC/父侧判定、双线内存及真实反伪造 oracle 仍未交付；17项内测通过不覆盖这些要求 |
| AC-09 / T-10 | 本次定向审核 needs_fix；整份独立最终验收尚不就绪 |
| AC-10 / T-11 | merge-owned pending，仅最终代码/验收/规范 pass 后运行冻结根裸 t + tv 及必要触面/独立workspace/consumer门；Spec/ledger/归档/guard 实际收尾也未完成 |

知识增量：主计划 SD-01..06 与 frontmatter 的精确目标路径/GOAL-002 保持，已冻结 [当前 delta](spec-delta-input.md)；canonical 输入哈希见 baseline，当前新增目标尚不存在的事实逐项记录，未发布 canonical/ledger。无语义修订或批准缩面，plan_revision 仍 2。SD-04/05 的来源/恢复正确性受上述失败阻断；SD-01/02/03 的前端/产物、SD-06 的正式会话仍随 F-01/02/10/11/12/14 保留。worktree 的 contract-decision、coverage-manifest、successor-interface-contract 仍有历史完整交付/延期表述，F-14 的重整与具体规范文本仍待做，不能当作经审核的 Spec delta。

## 诊断、状态与下一步

先前只改变 start/end 验来源，未改变实际驱动执行和选择的 func/instr 字段；r20a 在 zip 中显式忽略两侧身份，故“逐指令完整比对”的叙述超过断言。下一轮 work 应按 **sidecar 解码字段 → 适用性谓词 → 选择/写回 → verify → 链接 → map_span** 对账身份与坐标，而不是再仅增加一种端点例。两个原失败、四个新失败及两个合法/legacy 控制均为共享可重放输入，要求原样收编并补受影响组合。

计划保持 **executing 0/11**，T-02/T-07 仍未完成；其他任务/既往关闭项保留原状态，不因局部回归绿勾整任务。F10-S01 保持 CLOSED；F10-S02、F11-S01 整体仍 OPEN；F-01/02/05b/09/10其余/11其余/12/14 仍 OPEN。下一责任角色为同树 work，修复并提交后复核受影响面；全部 work-owned 必需项完成前不得提出整份 execution_done/final pass。未运行整档 t/tv/tt/tb/tu/taa/tf 或 docs_gen，未修实施、合入、归档、清树。
