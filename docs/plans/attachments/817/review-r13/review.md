# PLAN-817 R13 final review — needs_fix

2026-10-24；Plan r3；review_scope=final；第13次 final review。当前Codex审查上下文独立于实施，未暴露模型/session ID。读取主检出当前Plan，实现树 `D:/autostack/.wt/lang-817/auto-lang`，分支 `plan-817-dev`；冻结提交 `97203795b9167777bd88f5df1882583a50a879d2`，修复基面 `4309f49de`，整体base `d46e03e8a4986f3d5fed5bd09665c2aeebf0340d`。起止HEAD相同、实现树clean。未修改实现。

## 已核实的修复

V-01..V-12表格、AC/task/入口/Scope/证据状态及V-12四子case落点已恢复，修复历史单列；原“只有修复历史”的缺失已解决。四个V-12主入口均在实际测试源码中存在，既有实现/结构证据保留，T-11不用重开。新矩阵的个别证据绑定仍需更正，见R13-02。

脚本cache/freeze/resources改为测试二进制选择确实修复了空过滤/漏族问题。实际release `--test <binary> -- --list`计数为6/7/18，零Rust测试执行。对实际脚本dispatcher和Run-Step作六组路由控制：cache/freeze/resources各有原生命令exit0正控与exit7反控，脚本分别exit0/PASS和exit1/FAILED，错误没有变绿。`replay-routing.ps1`只替代cargo命令边界并记录实际argv，内部真实native进程返回0/7；没有冒称它执行了资源测试。resources的串行参数和acceptance中workspace全量的串行参数均经源码核对。

路由测试的观察边界：PowerShell函数替身会消费`--`参数分隔符，所以argv日志是替身收到的参数，不是操作系统层完整native argv；二进制名、release和串行选项仍可核对，分隔符本身通过实际脚本源码及native cargo列表命令交叉核对。测试替身仅证明这三个scope的选择/退出码传播，不证明Rust测试本体已经运行。

Rust实现、probe依赖树和lock与R12一致，R11-01及此前资源/编码/观测器代码结论复用。没有重跑cache/freeze资源正例族、GC、规模、性能或整档回归。下面的问题是脚本收口和文档正确性，不是新的CTE运行时缺陷。

## F-817-R13-01 / P2 / AC-08,09 / T-08：closeout仍把已认可的783适配判为违规

**位置**：`scripts/verify-ac-cte-817.ps1:82–87`、矩阵V-11。

closeout要求五个消费目录相对master全为零diff。但R8/R9已按收据增加并审查通过783的两个有界读取API，当前冻结树相对master的实际差异仍是 `experimental/ac-artifact-probe/src/store/mod.rs` 的120行追加。脚本假设没有随已认可适配更新，导致合法当前树确定性红。

**实际入口与结果**（不是mock）：

    powershell -NoProfile -File scripts/verify-ac-cte-817.ps1 -Scope closeout
    ==> read-only consumption audit (no edits under consumed crates)
    WRITE VIOLATION under experimental/ac-artifact-probe
    exit = 1

原始输出 `closeout-output.txt`、`closeout-exit.txt`；实际依赖差异保存在 `approved-dependency-current.diff`。此步在任何测试、重族或全局回归前失败。已知收口脚本逻辑错误不能仅以“merge-owned pending”留到合入时再发现；本项是原T-08边界审计/现有适配收据的一致性问题，不新增限制。

**最小关闭oracle**：将已认可的783追加适配绑定到明确基线/修订/差异边界，其他消费面仍保持只读。当前合法树通过；代表性的未授权修改（旧API实现或额外文件）仍必须拒绝。不要简单跳过整个ac-artifact-probe目录或吞掉git错误。复验只需这个只读审计及其正反控，不需要cargo测试、资源重族或全量acceptance。

## F-817-R13-02 / P2 / AC-08,09 / T-08：恢复矩阵的入口与冻结性能数据未准确绑定

**位置**：`docs/reports/plan817/acceptance-matrix.md:30–31`（V-08/V-09）。矩阵已具备结构，以下是对现有交付实物的直接比对：

1. V-08写 `resources_wire_input_bounded`，实际函数是 `cte_resources_wire_input_bounded`。对矩阵入口列与probe/tests函数声明的检查发现这一条无法解析，结果保存在 `matrix-missing-entrypoints.json`。
2. V-09称冻结于R7-fix的 `turnaround.jsonl` 为body274.4/meta14.2/resident234.4ms、树408.8MiB。当前同文件末行实际为：

| 字段 | 当前冻结文件实值 |
|---|---:|
| body-edit P95 ms | 170.668 |
| meta-edit P95 ms | 757.4471 |
| resident-edit P95 ms | 163.6434 |
| tree_private_bytes_peak | 431747072 B |
| target_met | true |

该末行原样冻结在 `turnaround-summary.json`，原报告文件本轮没有改动。不能将另一轮或另一负载的数字写成这个文件的摘要；这不表示实际性能门失败。尤其meta14.2ms是前期缩载阶段出现过的数值，不应被当作规定负载的冻结证据。

**最小关闭oracle**：更正实际入口，给V-09明确绑定被引用的报告/输入/实现版本，按同一原始文件重算或引用样本统计。若要保留历史值，明确历史身份，不能替代现行证据。无需再测性能，更不能为让表格数字相符而覆盖原raw或换小负载。其余映射与历史记录保持。

## 证据和状态

- 实际发现/选择控制：cargo `--list`三次（6/7/18），没有执行Rust测试。
- 脚本实际路由/失败传播：六组控制通过，命令与结果见 `replay-routing.ps1`、`routing-summary.json`及逐项argv/输出文件。
- 真实closeout在已认可783差异处exit1；无需继续无关重族来确认本缺口。
- git diff --check通过。当前Rust/probe/依赖/lock无变更，沿R12已通过证据复用，不重复编译检查或测试。
- 完整HEAD/base/依赖/lock、主Plan、四个canonical Spec、技能、提交矩阵/脚本和数据摘要hash见 `baseline.json`。Spec delta已冻结，SHA256 `DB1B45108E1460861E5F7CC17EF604222148D6A1D4EC99BBC2D5EB9C38AC002A`；本轮只改覆盖索引，原SD-01..06语义与R12代码结论保留。canonical Spec/ledger未写入。

AC-01..07的已闭合行为/性能证据按原适用边界保留；AC-11的V-12映射和实现证据通过。AC-08因以上脚本/矩阵缺口partial，AC-09未通过；AC-10及裸cargo t+必要tv/probe消费面统一门禁仍merge-owned pending。touched_goals空集解释沿R12记录保持。

**next: work，仅修复T-08两项收口问题并冻结。** T-08重新打开，其他已完成任务保留，含T-11；Plan为executing **8/11**，仅T-08/T-09/T-10开放。本轮没有新增运行时finding，没有要求重跑全量测试；后续复审只检查这两个差异及相关脚本控制。未改实现、canonical Spec/ledger，未合入/归档。
