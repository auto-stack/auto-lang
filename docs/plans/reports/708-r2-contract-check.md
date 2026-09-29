# PLAN-708 r2 修订检查与交接

日期：2026-09-29。对象：[Plan 708 r2](../708-vm-render-responsiveness.md)。

## 1. 本轮完成

本轮按用户“OK，继续下一步”完成 replan：plan_revision 从 1 到 2，保留全部历史审查，13 个任务、12 个验收项、4 项拟议规范增量。原 stable IDs 保留，新加 T-00/11/12、AC-09..12、SD-04。

创建 [专题架构设计](../../design/autoui/vm-loading-responsiveness.md)，在 design 总索引和 autoui 索引登记。设计明确 proposed，长期 worker 尚未实施；canonical Specs 与派生 ledger 未改。

**本检查由修订者自己完成，属于契约自检；没有独立复审通过或运行验收通过。** 计划仍 drafting/current_step=0。r1 的 pass/needs_replan 只覆盖历史版本。

## 2. 审查发现的处置

| r1 finding | r2 修订位置与任务 | 本轮结论 / 待验证 |
|---|---|---|
| R708-01 CPU 仍无界 | §5 M-02；T-11；AC-10/11；设计§3 | 已列入本阶段首发/resume可续跑、总预算/公平/安全累计；native单指令成本与共享堆写序待T-00/T-11证明 |
| R708-02 基线陈旧/归因缺失 | §4/5 S-02；T-00/02/06；AC-03/09 | 已承认gallery原有memo及无Init页；新增冷/暖分段归因，不把开启旗标当收益；尚无性能采样 |
| R708-03 帧后时序不成立 | §5 M-03；T-00/04；AC-05/06 | 真实run_session入口、主动唤醒、多App、dirty传播已明确；frames不是成功present回执，实际屏障作为前置探针，失败返回replan |
| R708-04 Init 生命周期 | §5 M-01；T-03/05；AC-04/07/11 | queued/running/runnable/waiting/completed/failed/cancelled与generation完整；重复view/key/迟到/错误/卸载测试列明；未实施 |
| R708-05 宿主局部失效 | §5 S-01/M-04；T-01/12；AC-01/12 | 局部epoch、props/params/theme/probe及pending失效门禁齐；风险未被静态文字销号 |
| R708-06 env=0公式错误 | §5 开关矩阵；T-01；AC-02 | 0强制关、1强开、未设/其他值按Option<bool>与缺省；12组合回归，046旧并集变更显式记录 |
| R708-07 只有首帧截图 | §6/7；T-06/09；AC-06/09..12 | 首帧/连续占用/交互/完整就绪/资源/多App指标齐；阈值为提议，实施前确认，不伪装实测 |
| R708-08 worker放入现状ADR | design专题§7；T-07；AC-08/SD-03 | 正式proposed设计已登记，后续独立Plan；Spec只拟沉淀已验证S/M，RefCell与线程所有权按实际类型判定 |

以上是“契约已补齐”，不是“8 个运行问题已修好”。KNOWN-DEBT 的 P708-R1/R2 保持待核实，只有实现证据才能销号。

## 3. 参数、授权与未决项

CPU 段预算拟议初值为 4ms/4096 指令、64 步查时钟、宿主总 pump 8ms；队列建议128。首帧 p95≤100ms/max≤250ms，加载期连续UI占用max≤50ms，交互反馈p95≤100ms/max≤250ms。60Hz 16.7ms 保留为优化目标；完整就绪p95相对基线退化不超过25%。这些数值未实测，也不从r1自动继承批准。

auto-plan-new 的“Revisions and authorization”要求：改变验收阈值、兼容/执行契约时，须给出具体修订稿供决定，再进入依赖的行为实施。本轮已提供具体 r2；因此 stage:new 交接记录 blocked，保留 drafting，原因是此契约确认与独立方案检查尚未发生。本轮修订任务本身已完成。

技术未知项有明确 owner：

1. 帧屏障：T-00 最多一轮完整调查和一轮纠正复测；不成立则 needs_replan，不能 tick/sleep fallback。
2. CPU 写序/RC/异常/native 成本：T-00 冻结决策，T-11实现和回归；不借CPU片悄改store所有权或纯CPU事件原写序。
3. cold computed/build/layout：T-00归因，T-02/12覆盖；关键超限项不得移到L后宣称完成。
4. 707 readiness/cleanup：按实际落地commit联合检查；在途707分支观察为2da544537994d0f1dcff53e00b7dac9f60b6e007，不认作已经合入。
5. 708 WSL worktree：只读发现gitdir环境差异，T-00先在创建环境验证并复用；未prune/remove或建立链接。

## 4. 结构与证据检查

- 基线：auto-lang `66c9cac193ad1a674be521e61440fcf5c65145e4`；gallery `93050a6a19231238cd5d4478909e9d6efd30be95`。Spec/design/707/r1正文哈希已在计划§4记录。
- 检查结果：编号节0..10齐；T-00..12 checklist唯一/齐；total_steps=13；AC-01..12唯一/齐；plan_revision=2；status=drafting；r1 needs_replan历史存在。
- 所有AC均映射任务，所有SD均映射AC，新增路径注明新产物。相对链接检查覆盖本轮计划/设计；未来work证据只列路径，不伪造文件内容。
- `git diff --check` 无错误。新增文档的结构、内容与本轮链接检查通过。
- 未修改Rust源码或执行Cargo/实机性能测试，遵循Category A；不把静态检查写为compiler/runtime健康通过。

## 5. 下一步

确认r2新增执行契约与指标并完成独立方案检查后，进入 work T-00 的基线/探针；通过decision再实施S/M。最终实现必须由独立review逐项复核AC和SD，绑定实现commit，按改VM范围一次cargo tf收口。当前没有merge/归档凭据。

## 6. 自检内容绑定

以下绑定本次修订稿与拟议delta，后续语义变更会使本检查过期：

- `plan_sha256`: `782CDA09CC0EB90EDD807666E434810B0BE78B18BBA3F45DA74ABB566FDFEAE7`
- `design_sha256`: `EB78C29E8B2236110DF63B70855178F8FC3E410EE85E5F00713FCD2BE503EA11`
- `spec_delta_sha256`: `70310003BC92919A78D185AF2B27241D306ACE47481D903780A08A21C016B395`

Spec delta hash口径：计划中从 `### 规范增量` 到 `## 6. 测试设计` 之前的UTF-8文字（LF换行）。这些是文稿哈希，不是实现验收凭据。
