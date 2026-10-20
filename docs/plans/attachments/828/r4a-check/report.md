# PLAN-828 r4a 修复核查

2026-10-21，**needs_fix**。受审 `040061b6144654315f4c3372fcbfedabb7c67a3e`，修复基面 `bb2380908ea7873be922f67288cf3aba39f82fc5`，`plan-828-dev`，`D:/autostack/.wt/lang-828/auto-lang`；[基线/依赖/二进制/输入哈希](baseline.json.txt)。reviewer为同一独立上下文Codex `/root`，未实施r4a。

主计划仍executing0/11、work为in_progress并承认其他必需OPEN。本轮仅执行就绪及受影响修复核查，不重启完整最终验收；此前所有最终复审/局部核查历史保留。主检出计划、债务表及合并工作不写，证据留在828原树。

## 本轮结果

| 检查 | 实际结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | 永久18/18过，0 ignored，0.12s |
| 同manifest `--test function_incremental_828 -- --test-threads=1` | 原17/17过，0 ignored，8.96s；[raw](original-suite-output.txt) |
| 上轮独立20例原样重放+新增3个组合控制 | **20过/3失败**，exit101，0.16s；[源码](replay.rs)、[执行器](run-replay.ps1)、[raw](replay-output.txt) |
| 六个改动Rust文件rustfmt --check | exit0 |
| `git diff --check bb2380908 HEAD` | exit2，仅新提交的两份旧日志EOF空行；[raw](diff-check-output.txt) |

上轮5项失败例均已通过，不能将本轮新失败说成其旧输入复活。三个新例验证相同来源映射/实际依赖合同的组合路径，没有扩大语言或规模要求；每例有旧源/第一步或同源冷编正控。测试重叠不累加为独立覆盖。原有shim warning、session.context_revision dead_code保持；r4a另新增unused SpanInfo import和unused old_src变量（其clone全文仍真实执行），归F14健康剩余项。

未跑整档、docs_gen、正式IPC/内存门；未重复scalar差分/798兼容集，不将work交接总数当本轮独立结果。检查期间HEAD及实现clean，随后只新增本目录文档/材料。

## F04-g / P1：第二次编辑混用当前坐标和代码铸造坐标

沿用F04、AC-02/05/06、V-02/05/07的跨修订来源规则。`two_long_whitespace_edits_compose_in_code_source_coordinates` 使用原小型helper/main/tail除零程序，连续两次在main的return前插入32个普通空格（合法、语义不变、代码真实复用）。每一步均与相同新源冷编比较，第一步通过；第二步：

```text
actual:   func#1 102..107
expected: func#1 134..139
```

`ProgramSnapshot::map_pos`用代码原始span位置x，与region的st比较求delta和；但commit将旧region起点按新编辑位移、并添加当前源码中的新编辑位置。第二次编辑的st已越过原始x，导致漏算第二次位移。两个坐标域没有正规化，仅把单次delta放进vector并排序仍不构成映射复合。

## F04-h / P1：先改后方，再改前方，丢掉区间终点变化

`later_then_earlier_edits_preserve_both_range_end_mappings`：先在`1 / 0`内部插入一个空格（第一步与冷编一致）；再在return前插入32个空格。源仍合法、语义不变；第二步得到：

```text
actual:   func#1 102..107
expected: func#1 102..108
```

旧的区间内编辑region被移到当前坐标，新source-map仍拿原始end去比较，早先+1终点变化消失。r3a提出的独立端点映射已接线，但底层编辑链的坐标组合仍错误，因此单点/区间两个API都不可靠。

**修复责任**：明确并实现 `map_new = edit ∘ map_old`，每个编辑在哪一修订坐标中定义，查询点在哪一坐标中进入，都须一致。可保留按修订顺序应用的编辑链，或正规化到共同的代码铸造坐标，定义端点边界策略；不能把当前region位置直接与mint坐标相加比较。继续保留snapshot不可变、内容命中保留真实代码基面和代码payload不变约束。除既有原例外，至少回放连续短/长插入、先前后后前两种顺序、内部端点变化、历史命中及公开/暂停snapshot消费者；本轮未给未执行路径授pass。

## F06/F08-r4 / P1：intrinsic表下标变化导致正常编辑误拒

这是r4a从symbol集合改成表下标后暴露的修复回归，沿用F06实际引用、F08原标量支持面及计划§2“跨修订身份不能用模块数组序号”的既有约束。

独立例 `mixed_intrinsics_keep_binding_across_synth_def_index_shift`：

```text
fn helper() int { return mark_a() }
fn main() int { return helper() + mark_b() + 1 }
fn tail() int { return 9 }
```

trace授权下原源结果4；仅将main末尾`+1`改为`+2`，同源冷编结果5。调用绑定、授权、签名均未变，却得到 `ok:false / revision:0 / body credential minted under a different declaration context`。

原因：CheckedFunction保存`callee.0`，FnEnv按`di as u32`存intrinsic事实。完整源有helper使用的mark_a和main使用的mark_b；合成Program把未改helper清空，只保留main真实body，于是mark_b的表下标前移。这个下标是局部布局，不是稳定引用身份。原symbol交换/effect负控已正确拒绝，但合法布局变化也被误判为语义变化。

**修复责任**：用受检owner/域内稳定声明身份记录每条实际读取的绑定，核对该绑定的symbol、签名、effect及适用授权/版本；不用数组序号跨bundle比较。必须同时保留两类控制：同一稳定绑定仅局部表下标改变应接受，稳定绑定内容交换/变更应拒绝。也需与F01的持久受检接口环境接线，保持未读intrinsic不会过度失效。

## 状态与后续

- 上轮F04-d/e/f和F06交换/effect旧例全部CLOSED；完整F04仍被g/h阻断，F06/F08出现以上合法输入拒绝回归，尚不能完整关闭。
- F03/07/13及其他已验证原例保持；主计划其余明确OPEN不变，继续executing。AC09本轮needs_fix；AC10/SD01..06完整验收与merge-owned门、沉淀/归档/guard仍pending，不改canonical或ledger。
- 先前多轮后的诊断/设计调整要求继续适用：work应先提供坐标域/映射组合定义与稳定引用身份及全部生产消费路径的对账，完成这些表示和接线后再提交；逐例转绿不能代替完整不变式关闭。此处无需新增需求或抬高阈值，均为现有正常编辑规则。
- 未修实现、未提交、未合入。按既有用户约束，报告及`pending-bookkeeping.md`留在原worktree，主检出合并收口后再对最新主计划增量回写，禁止用工作树旧计划整文件覆盖。
