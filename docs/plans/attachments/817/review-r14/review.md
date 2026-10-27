# PLAN-817 R14 final review — pass

2026-10-24；Plan r3；review_scope=final；第14次 final review。当前Codex审查上下文独立于实施，未暴露模型/session ID。主检出Plan与收据 `040904e91` 已核对。实现树 `D:/autostack/.wt/lang-817/auto-lang`，分支 `plan-817-dev`；冻结提交 `c224a4b0b87ae906dba711c66262c472f3d6de0d`，修复基面 `97203795b`，整体base `d46e03e8a4986f3d5fed5bd09665c2aeebf0340d`。起止HEAD相同、实现树clean。未修改实现，没有触碰其他会话暂存。

## F-817-R13-01 CLOSED

真实执行 `powershell -NoProfile -File scripts/verify-ac-cte-817.ps1 -Scope closeout`：exit0、SCOPE closeout PASS。已认可783追加适配、四目录只读、注册append-only和worktree无reparse point均通过。

独立读取实际Git差异：唯一文件 `experimental/ac-artifact-probe/src/store/mod.rs`，+120/-0；将完整diff行以LF连接、UTF-8编码后计算的SHA256为：

    D8A8B3DF054E11015E1D4D5C33F02DAC84C8B753D5CF0AE5FB7BA8C33924E249

与脚本钉定值一致，授权依据仍为R8/R9已经审查的两个有界读取API追加；没有把整个依赖目录排除出审计。四个其他消费目录仍拒绝差异，所有目录先检查dirty/untracked，Git错误保留失败。

对**实际脚本AST中的审计body**执行6项定向控制：合法输入通过；dirty、只读目录untracked、额外文件集、已认可增量内容改变、git status错误分别命中指定拒绝分支。改变增量时只改实际diff中的函数签名，SHA变为4927F47A…32D42，被原审计拒绝。详细输入与结果见 `gate-input-controls.ps1` 和 `gate-input-controls-output.txt`。

观察边界：完整合法closeout使用真实Git和真实worktree；负控在Git返回值边界注入，不向冻结实现写入脏文件/提交，也没有创建另一个实现checkout。此控制验证判断与错误传播；实施收据所述实际dirty/untracked/提交后改动反控作为补充证据保留，不将本次注入说成真实提交变异。初次控制脚本因环境中有两个git.exe、工具定位返回数组而未进入审计，已修正为选择首个应用路径；原记录保存在 `control-setup-error.txt`，属于审查工具设置问题，不是产品失败。

## F-817-R13-02 CLOSED

V-08入口现为实际存在的 `cte_resources_wire_input_bounded`。独立检查当前矩阵入口列：87次完整具名引用、82个不同入口，全部能在实际测试函数声明中解析；组合缩写另作源码核对。Rust测试与R13没有变化，原 `--list` 清单证据复用，本轮不重新编译/执行测试。见 `matrix-crossrefs.json`。

V-09现绑定同一冻结 `turnaround.jsonl` 末行：body-edit P95约170.668ms、meta-edit 757.4471ms、resident-edit 163.6434ms；树峰431747072B，target_met=true。数值与原raw一致，R4-era数字明确标为历史、不再冒充现行文件摘要。没有重跑性能测试，也没有覆盖raw来匹配表格。

V-11更新了精确适配边界与审计证据；最终merge复跑是未来收尾检查，当前脚本行为已经独立确认通过。V-01..V-12结构、AC/task/Scope/证据状态及V-12四子case映射保持完整。

## 最终验收映射与复用边界

本轮差异仅脚本审计和文档。Rust实现、probe树、依赖树、lock与R12/R13相同；不因文档/脚本新HEAD重跑未受影响行为。具体证据以冻结矩阵及既有复审报告为准：

| AC | 结果与依据 |
|---|---|
| AC-01 | pass；V-01/05/10：T-01合同、首例真实AV/consumer/native、跨进程缓存、正式recipe证据复用；恢复共享机制由R10–R12已验证 |
| AC-02 | pass；V-01/02/04：真实scalar/Meta实参、绑定和HIR回灌；R4 owner/entry缺陷在R5已关闭，后续对应实现未改 |
| AC-03 | pass；V-02/03：调度/环/合法DAG、capability、trap和资源分类；累计deadline在R7关闭，后续定向控制保持通过 |
| AC-04 | pass；V-04..07：依赖/新鲜性、坏块准入、跨进程命中跳AV、租约/root/失败保旧证据复用；R9读取、R10编码、R12解码具体修复均关闭 |
| AC-05 | pass；V-07/08：已通过的规模/GC/树门证据和R10–R12预算反例/控制保持原输入配置适用边界，本轮没有新增测量或声称资源重族重跑 |
| AC-06 | pass；V-09：规定负载的编辑/内存/语义oracle证据沿原匹配记录复用，当前报告摘要绑定准确；旧数值不当作新测量 |
| AC-07 | pass；V-10：正式runner、身份/stale/restart、隔离态/父判证据复用；注册和协议逻辑未变 |
| AC-08 | pass；V-11与本矩阵：当前边界审计真实通过，证据落点及SD索引更正；无开放收口项 |
| AC-09 | pass；本轮独立review，F-817-R13-01/02关闭，未发现新增finding |
| AC-10 | pending，merge-owned；统一门禁、Spec沉淀/ledger、归档合入与guard清理尚未执行 |
| AC-11 | pass；V-12四子case及驻留baseline/身份/增量/失败控制沿R4/R5及其后匹配修订证据复用，代码未改 |

复用不表示旧运行覆盖任意未来修改，也不把原性能数字称为本轮重新测量。merge如变更实现、依赖或测试配置，须检查证据失效范围，补受影响复审/门禁；不能只沿用本次SHA的pass。

## Spec与基线

SD-01..SD-06语义沿R12已经完成的实现/规范审查；覆盖索引沿R13修订，此轮不变。四个现有canonical目标、auto-cte三个新组件路径与Plan元数据一致；touched_goals空集的书面理由沿R12保留（有限AC/CTE实验无对应独立Goal定义，不改既有Goal状态）。规范落地与ledger写回仍归merge，本次未修改canonical内容。

冻结Spec delta SHA256：`DB1B45108E1460861E5F7CC17EF604222148D6A1D4EC99BBC2D5EB9C38AC002A`。完整HEAD/base、依赖树/lock、主Plan、四Spec、技能、矩阵和控制脚本hash见 `baseline.json`。完整合法closeout日志、独立摘要和原矩阵均在本目录，清理worktree后仍可读。

## 判定与下一步

**final pass；status=reviewed，10/11。** T-01..T-09及T-11完成，仅T-10开放。所有既有finding关闭，没有未授权延期、缩小支持面或遗留workaround作为放行条件。

**pending_closeout_gates（merge-owned）**：在最终冻结交付基线上统一执行裸cargo t、必要cargo tv，以及矩阵/脚本规定的probe消费面与resources/acceptance验证、最终closeout；复用同基线有效证据，避免重复运行。cargo tf仍仅按到期条件在主检出单实例批量运行。门禁失败须归因；代码修复后只补受影响复审，再在新冻结基线上执行失效门禁。门禁通过前不得合入或宣称交付完成。

本轮零Rust测试执行，无全局回归。未合入、未归档、未清理实现树；**next: merge**。用户此次仅请求复审，本轮到此交接。
