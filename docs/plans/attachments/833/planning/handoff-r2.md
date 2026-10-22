# PLAN-833 r2：来源冻结与实施交接

2026-10-22，stage=new，**outcome=pass，next=work / T-01**。本次只核源、冻结和修订计划，没有运行SDK/图形测试或实施代码，没有创建树/分支/agent。任务保持drafting 0/10；实施agent收到执行833任务后可直接进入work，无须重新索取来源放行。

## 1. 当前合同与唯一树

- 当前合同为主检出的 [833 r2](../../../833-autographics-auto-source-minimal-loop.md)。历史代码基面不含这份当前合同，必须从主检出读取，并在T-01记录其hash，不能用旧树计划覆盖主检出。
- 创建 `D:/autostack/.wt/lang-833/auto-lang` / `plan-833-dev`，**明确从** `764e87559cff016d9e0d6b2141e5f1f9903398c5` 创建；不使用缺省master基面的建组命令、不复用已删除的830树。
- 同一树依次承担work、内部检查和独立最终review；按原流程审毕、冻结门通过后合回autographics-dev。生产代码不合master，生产AutoUI/RQHost/Forge和跨仓迁移门不变。

## 2. G830：已满足

| 核验 | 已核事实 |
|---|---|
| 原计划终态 | [830归档](../../../archive/830-autographics-surface-portal-composition-and-input.md)：archived、completion_kind=delivered、10/10 |
| 最终验收 | [R5](../../830/review-r5/review.md) pass，冻结提交764e87559cff；原R4两项CLOSED，无开放阻塞finding |
| 冻结合入门 | [test/build](../../830/work-r6/merge-gates.txt)：AGP locked全套162/162、release build通过；[probe门](../../830/work-r6/merge/gates.txt)：六surface、真实OS输入/IME、829兼容、independent-consumer重定位29字段0差异 |
| 首轮环境红 | iced-shared-device首轮posted_queue注入未生效，presses=0；相同冻结提交串行重跑presses=1、2/2门通过，原红保留并已归因；不能只删首轮红或把当前已成功重试写成阻塞 |
| 分支交付 | `git rev-parse autographics-dev` = `764e87559cff016d9e0d6b2141e5f1f9903398c5`；tip等于受审/交付提交；祖先检查和AGP子树diff均exit0 |
| 图形身份 | AGP subtree `fdfdf351e91f52ff1848bff51134eb85a5f8928f`；锁SHA256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf` |
| 知识沉淀 | 当前surface-composition SHA256 `e68bba42d91e76feea78fe0c57bd6bc01468c09658c268f9c2b428f118a95d5b`、nested-input `68b3b38fda584254923be56476299f3709c2733eb3e45365afd2074503d6c5c1`，均与R5冻结SD一致；profile/交接/模块索引已有830增量。标题保留draft字样属历史表述，以冻结字节/正式merge收据为当前实验来源 |
| 清理与批量门 | 原830树/分支已退役；当前worktree list无830。归档记录主检出单实例tf/tt/tb完成，批量收据覆盖830；本轮只核收据，不重跑 |

本次还逐一解析10份成功的既有merge report，exit均pass，fingerprint均绑定764e87559cff与上述锁；hash与路径记录于[基线清单](../baseline-r2/baseline.json.txt)。这属于证据核对，不是本轮新跑探针。

已核公开图形消费点：`host/composition_input.rs::CompositionInput` 的pointer、keyboard_target、take_focus_transfer、cancel/revalidate等；`host/surface_runtime.rs::{SurfaceRuntime,SurfaceContent,Game2dContent,Scene3dContent}`；`core/composition/`的身份/图/输入。通过公开lib模块可见，不把将来需要的新语言bridge误称为当前已有。

## 3. GSDK：冻结来源与输入

- SDK来源：本仓已交付master提交 **`9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`**。它是包含814实际交付的后继提交，只用于提取固定SDK源码，**不整体merge至图形分支**。SDK最后变更`9c7e44855c88b936835d57111c06e629e30de0bf`；当前闭包/source/stdlib/root manifest对固定源无差异。
- 对normal/build/dev及target表、optional路径和workspace继承作TOML语义递归，闭包 **17包/29边**，包含16个crates成员和vendor/autodown-test-parser；无旧auto-lang/UI dialect/compiler/runtime/tools/Man。这是本地源码依赖图检查，不是本轮fresh build或最终runtime能力验证。
- 每包固定Git tree、manifest SHA256、29条依赖边、关键公开入口/build/include输入及stdlib tree记录于[baseline.json.txt](../baseline-r2/baseline.json.txt)，并绑定本r2 Plan hash。manifest分别记录Git对象与工作副本字节hash（CRLF/LF可能不同），提取以固定Git源为准，镜像记录自己实际输入hash；不得把合法换行差异误认成未授权源码漂移。禁止用“当前主检出路径”代替该固定来源，禁止复制上游WIP。
- 原根Cargo.lock为Git未跟踪输入，已保存 **274244字节** 的[精确快照](../baseline-r2/sdk-Cargo.lock.txt)，SHA256 **`5192d08d502a8f17e277ad8912f5884175659bef642b614e25b394243d1ff091`**；原[Cargo.toml](../baseline-r2/sdk-Cargo.toml.txt)也保存。源提交不能替代这一份实际锁字节。
- 提取时保留实际workspace继承值/build/include输入，受管投影另记转换与hash。原根iced_widget patch不被纯SDK消费，投影须登记为不适用并排除，不能为保存该patch引入生产UI源码；AGP仍用stock 0.14.2。新组合driver的独立锁与可搬动载体由T-03生成/验证，当前上游锁不等于其最终消费锁。
- 根build输入扫描中的AUTO_DOWN_ROOT是可选显式输入：本切片不设置它，也不借外部auto-down补目录；若确需外部输入，须具名记录并按原写面处理，不静默扩大闭包。

已再核底层 `create_vm_from_source`、分片调用/恢复、VMConvertible实现存在；高层 `VmInterpreter::call` 依旧TODO恒返Nil。本r2不修SDK，不把这些符号的存在认作有状态/f32/RC图形调用已经通过。

## 4. 立即可执行的入口与停止条件

1. T-01读取主检出r2及基线附件，核树/分支/固定SHA/锁，准备显式只读SDK源；仅重核固定来源，不能追新分支tip。
2. T-02建立最小独立fixture/manifest/锁，最多比较复合数据与标量查询两种**公开**接法，真实验证参数顺序、状态、f32与所用引用寿命、返回值、预算/退出与输出消费。
3. 两方案都不能闭合时，提交确切SDK缺口并回new；不得删验收、改精度/玩法golden或把玩法移回Rust。后续T-03..08按既有依赖推进，T-09独立review，T-10冻结门/合入/清理。

任务10项、AC9项、V8项、SD3项及支持平台/写面不扩张。I1仍只做首个Auto游戏切片，Squash真实Auto、生产I2/I3、Shader/CAP与完整ACE不自动完成。

## 5. 修订与历史

r2改变来源/交接合同，故revision从1升2，保持drafting。T-01改为重核已冻结源，T-03明确SDK投影/未消费UI patch；功能AC/V/SD不删减。r1全文保存在[历史副本](plan-r1-before-freeze.md)，其中链接按原`docs/plans/`位置解释，历史pending/未授权描述已被本r2交接取代。

**结论：可以交给另一个agent实施。** 本会话只完成准备和可执行交接，未推进任何实施任务。
