---
plan_id: PLAN-794
status: drafting
feature_name: scroll controller 与子锚点宿主能力迁移
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/widgets/scroll-pane.md, docs/specs/auto-lang/ui/design/scroll-child-anchor.md]
new_spec_components: []
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-lang/vm, auto-lang/ui, widgets]
depends_on: [PLAN-781, PLAN-790]
---

# [PLAN-794] scroll controller 与子锚点宿主能力迁移

## 0. 变更摘要

将9900–9905 controller/offset族及781新增9922/9923子锚点族的VM→UI直引迁为owned宿主provider；保留renderer队列、测量和锚点状态机单一来源。八个native全部纳入，不能只迁旧offset六函数。

## 1. 目标

无UI宿主能注入真实scroll能力并经源码/VM观察可区分状态；UI生产通过adapter保持现有滚动与子锚点效果，core shim无ui::scroll实现依赖。非目标：code_editor滚动9911..9913、term engine、managed content新锚点支持、方法语法糖或滚动算法重写。

## 2. 架构方案

core保留参数/返回编解码；provider只暴露owned句柄、轴/intent、测量/锚点状态DTO，不含UI Axis/ScrollIntent类型、VM heap id或renderer引用。UI adapter委托现有controller registry与状态机。沿790优先级：显式per-VM/默认快照保持，兼容installer仅补空槽；不在core暗建UI实例。

## 3. 技术栈

既有host_capabilities/NativeInterface/VM session、StakeGuard、ui/scroll/controller.rs、scroll-pane与781锚点collector；autoui-verifier双端定向验证。

## 4. 需求分析与背景调查

取材 auto-lang@`9c773601abfdb25b80c08d06922d0ad50edc653a`；[本批输入指纹](attachments/parallel-extraction-20261016/inputs.json) 记录现行 Specs 与实际源码 SHA256。overview仅作导航，以模块Spec和真实代码为准。789/790/791均归档8/8；792为独立构建发行线，当前无交付假设。
用户要求“最好能有几个并行的计划”，本轮按该授权创建可执行合同和编排文档；未授权在本轮实施、创建agent/工作树、发布或修改旧仓refs。确认合同后各自work；本计划不依赖792完成，正式搬仓仍须满足整条路线退出条件。
787/788在途实验线不设为全局前置；实施者必须按实际diff重核类型/manifest共享面。全部实现/测试在本计划唯一组树，主检出只写计划簿记。
native_catalog明确9900–9905和9922/9923；native.rs现仍直接调用ui::scroll、按pending_native_arg_count解码可选轴。controller.rs含全局队列、最近测量和绑定generation；781合同规定queued不是完成、managed=unsupported、换绑/失锚等invalid reason与一次性消费。必须完整继承，不因provider有返回值就认为已滚动。
主写面native scroll族、host_capabilities/engine/相关宿主安装、UI adapter与定向scroll测试；不改793 error实现、不改795 lock/target、不改792 rust_ui及依赖升级。lib.rs/manifest等共享片段按写入窗口串行；不要求等792整项结束。

## 5. 详细设计

T01冻结八函数ID/arity/栈消费/返回形态及各feature缺席分支。保持controller为任意非空opaque string，不能新强制@scrollctl前缀。轴默认/未知轴、相对/绝对参数解码与state record类型保持；Core DTO在shim转换，UI adapter转换为原UI类型。
保留renderer的binding/measured state唯一真源；不复制第二个core registry。to_child入队与anchor_state查询仍分离；pending/applied/invalid/unknown、unbound首绑与换绑、key查找、managed unsupported和重复请求覆盖均不变。列表/字符串/记录所有权在成功/失败/缺席/提前返回配平，锁外调用provider。
per-VM隔离针对provider选择及快照，不声称同一UI adapter的既有全局registry自动变成多租户隔离；UI句柄分配和双pane寻址保持。不要顺手修改全局registry算法。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md | 八native provider/DTO/缺席与ownership | core去反向边 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | UI adapter与生产装配、未迁族清单 | 接回真宿主 | AC-02/04 |
| SD-03 | modify | docs/specs/widgets/scroll-pane.md | 增宿主装配说明，原测量/intent合同保持 | 责任清楚 | AC-04 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/scroll-child-anchor.md | 调用边界迁provider，不改状态机/完成oracle | 781完整继承 | AC-02/04 |

## 6. 测试设计

实施者未另指派，独立final review与merge分别交接，不新建或指定额外模型。下表V用例由work实现并执行，不是本轮通过证据。开发及needs_fix只做定向验收与必要cargo check；最终代码提交、final review通过后，由merge在冻结提交跑裸cargo t及实际触面档。tf仅主检出批量单实例，不随各计划并发；无Schema/生成器改动不跑docs_gen。

| Case / owner | AC | 入口/输入 | observable oracle / 边界 |
|---|---|---|---|
| V01 / work | 01/02 | 新test/fixtures/modularity/scroll-provider-consumer，no-ui源码→编译→八native | fake记录axis/offset/key并返回特异快照/状态，真实VM输出匹配；独立manifest无UI类型provider |
| V02 / work | 02/03 | 各arity/轴/Unicode句柄与key、缺席降级、两VM/Clone/显式默认优先、重复调用清栈reap | 参数顺序/返回形态与旧臂一致；不串provider、不泄漏、不覆写；无假默认值掩盖没接线 |
| V03 / work | 02/04 | 真实UI普通pane/双pane/锚点；首绑、换绑、失锚、未测量、managed、重复请求 | 原pending/applied/invalid与one-shot成立；真实offset变化+目标正面积交集，queued/progress不是完成证明 |
| V04 / work | 04 | autoui-verifier对现有scroll/781 fixture Vue+VM定向交互 | 双端原行为一致，不修改golden或移除失败case；不要求全画廊 |

拟树D:/autostack/.wt/lang-794/auto-lang，branch codex/plan-794-dev。定向验证必须在final review前完成；merge-owned裸t+tv，若改ui_gen加tu；其他重档仍主检出单实例。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 八native无UI实现直引，no-ui独立源码VM注入V01通过 |
| AC-02 | work | ID/arity/缺席/状态与781语义保持，V02/03通过 |
| AC-03 | work | 成功/失败/缺席ownership、实例选择/优先级负控通过 |
| AC-04 | work | UI真实adapter/renderer测量链不漂移，V03/04通过，剩余宿主族清单明确 |
| AC-05 | review | 独立源码VM+真实UI完成oracle及SD复审通过 |
| AC-06 | merge | 冻结门禁、规范/ledger/归档与guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 781/790归档、合同确认 | 冻结native/feature/arity/record与781状态旧臂，列宿主构造清单 | 02/03/04 |
| [ ] | T-02 | work | T01 | owned provider/DTO、VM及默认快照/优先关系，注册定向控制 | 01/03 |
| [ ] | T-03 | work | T02 | 八shim去UI引用、栈解码和返回编组，保留降级并配平引用 | 01/02/03 |
| [ ] | T-04 | work | T03 | UI新adapter及统一宿主安装，委托原registry/collector，不复制状态机 | 02/04 |
| [ ] | T-05 | work | T04 | 外部consumer、V01..04及真实UI证据，777检查八族删边 | 01–04 |
| [ ] | T-06 | work | T05 | 触面/未迁族、双端/ownership/并行收据与SD素材；提交冻结代码 | 01–04 |
| [ ] | T-07 | review | T06 | 独立八native及UI实际完成条件、负控、SD冻结 | 05 |
| [ ] | T-08 | merge | final pass | 裸t+tv及实际tu；沉淀/归档/guard清理，后合入者同步重验 | 06 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（合同preflight） | next=合同确认且native热点空闲后work | 0/8；无需等792，不以fake代替UI完成证明。

## 10. 待澄清事项

DTO具体字段在T01按现有state/anchor投影冻结；若必须改变UI状态机、feature缺席值或加入新跨线程调度，先出具体冲突修订，不能顺手发明新的scroll架构。
