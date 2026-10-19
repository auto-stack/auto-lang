# PLAN-809 R6：needs_fix（sidecar已闭，验收收据仍缺）

2026-10-19，review_scope=final；按用户要求重点确认R5剩余问题。**R4-02/R5 sidecar完整不变量本轮已关闭；不能判定全部闭合，尚有两项work-owned验收证据和一条新增health问题。** 未合入或归档。

## 基线与独立性

- plan_revision=1；reviewed_commit=`23e94eb74666479789345c5c1b81f8ec6545824b`；prior=`0fba0b604bd81076b8404ffbb9161432acfc0fe3`；base=`2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 原工作树`D:/autostack/.wt/lang-809/auto-lang`，branch=`codex/plan-809-dev`；auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场/结论前实现与兄弟依赖clean，HEAD未漂移、diff --check过。
- reviewer_identity：此前独立复审的Codex上下文，未参与本轮实现，未另开agent，不虚构模型/会话ID。以本轮实际生产调用、原反例及正负控判定，不采信修复总数。
- [输入/源版本/结果基线](review-r6-baseline.json.txt)、[送审Plan](review-r6-plan-input.md)、[SD冻结](review-r6-spec-delta.md)、[复用输入核对](review-r6-reuse-inputs.json.txt)。主检出并行推进，仅在主检出修改Plan复审簿记与附件/debt；未改实现、live Specs/ledger。

## 闭合矩阵

| 原finding | 结论 | 本轮实际证据及边界 |
|---|---|---|
| F809-R4-02 / R5对象与嵌套属性漏失 | **CLOSED** | 原R4对象源码未改，旧/新真实生成extra.rs都存在；R5属性Node同名与异名内部head均保留1模块+1依赖 |
| 同一不变量的优先级/失败控制 | pass | 节点与对象并存时仍以节点为准，避免对象的缺源错误；仅有对象且缺源时旧/新均报同一module source not found，不再静默忽略 |
| R3节点来源反转、Block、自定义front；R2 Ninja身份/Ark/Vue/786原例 | CLOSED保持 | 本轮旧生产重放及16项定向通过，request_only/disk_only/both、Block、三态Ninja内存产物一致 |
| R4-01编译/Block、R4-03 C输入、R3-03指定gen入口覆盖、R3-04旧死helper | CLOSED保持 | 代码/锁/配置变化核对；受影响调用重放，本轮无这些旧错误。R5的core46/model38与C输入独立编译证据按未变范围复用，不改记为本轮重跑 |
| F809-R2-04真实Ninja成功产物 | **OPEN** | 本轮管理8/8仍打印V05-PENDING；实际成功分支未执行。没有找到可定位的新真实成功/可消费产物收据 |
| F809-R2-06 809受影响独立交付回归 | **OPEN** | 当前Plan最新work行仍列pending；809附件未新增正式fresh/重定位locked收据或充分复用论证，默认scratch-consumer目录不存在 |
| 新增health | **F809-R6-01 P3 OPEN** | rust_ui.rs:3289移除extend后保留mut，普通库构建新增unused_mut；见下文 |

不把未定位收据等同于断言从未运行；本次用户也未提供其他位置。792本身已归档8/8、原正式driver25/25，保持完成；这里核查809改变生成链之后的受影响验收，不重开792。

## 已闭合的sidecar证据

当前实现把req.raw中的rust_sidecar直接还原为core所读root的属性，把raw_nodes仍作为具名子节点；保留core原“节点优先、Obj/Node属性兜底”语义，不再把属性包成另一层子节点。

[重放脚本](review-r6-replay.ps1)重新编译并运行未改动的R2/R3探针、[R4对象原输入](review-r4-sidecar-obj.rs)、[R5嵌套属性输入](review-r5-shapes.rs)，再运行[本轮优先级/缺源控制](review-r6-extra.rs)。均链接本轮刚构建的生产rlib，哈希在日志；probe exit0只表示观测完成，判定依据是如下值：

- OBJECT：old_extra=true / new_extra=true，旧新均Ok；
- NODE_PROP：head=rust_sidecar、head=sidecar_value，两态actual_extra=true / actual_dep=true，core oracle均1模块+1依赖；
- node_priority：对象指向缺源，但节点声明有效extra，两臂均Ok并复制extra；
- object_missing_source：两臂均Err，诊断同为missing模块源文件缺失。

[完整原始输出](review-r6-probe.log.txt)。这些是原支持域及原反例的组合控制，不引入新语法/规模承诺；未知节点参数与通用display_title的既有差异未误列新缺陷。

## F809-R6-01（P3）：修复后新增unused_mut

AC-05，T-08/09。`crates/auto-man/src/rust_ui.rs:3289`的`let mut sidecar_nodes`在本轮移除`.extend(declared)`后不再修改；当前普通库编译明确报告“variable does not need to be mutable”，生成日志:4407起。此前R5代码确有extend，不是把既有告警归本轮。

去掉不必要的mut，在下一冻结提交做受影响定向构建/复验即可；无需为此跑整档。其余既有warning不计为本轮回归。格式两文件通过也不能替代Plan的零新增未处理warning要求。

## 缺失的验收证据及边界

R2-04：`plan809_management_core`本轮确实运行真Cargo/Vue成功与失败、不可写输出等控制，8项测试绿；但Ninja成功分支因PATH未找到gcc/clang/cl显式pending。已安装cl能编译C输入的R5正控，仅证明输入合法，不证明选中工具链的真实Ninja Builder成功、产物可消费。需要实际命令/退出/产物及当前代码绑定证据。

R2-06：809主附件只有旧777收据和历史review日志，最新Plan末行仍承认独立交付回归未补；driver默认输出`D:/autostack/.wt/lang-809/scratch-consumer/driver-receipt.json`所在目录不存在。792目录最新driver-receipt.jsonl仍为原10月16日交付。若结果在别处，可提供路径及输入身份；若复用，须证明受影响代码、依赖、配置、输入未变。本轮sidecar消费改变，不能仅凭792原交付25/25关闭809的新受影响链。

这两项在809 AC-04/V05/V06中明确owner_stage=work；不是把冻结日常回归提前，也不能转记为merge-owned以授予reviewed。当前已知未完，不再执行一轮完整终验或擅自降低合同。记录debt不代表批准延期。

## 本轮验证与复用

| 检查 | 实际结果 | 持久证据 |
|---|---|---|
| cargo test -p auto-man --lib -- plan809_v02 plan809_r2_parity generation_request_adapter project_display_reader --test-threads=1 | exit0，16/16，含新增对象回归 | review-r6-generation.log.txt |
| cargo test -p auto-man --test plan809_management_core -- --test-threads=1 --nocapture | exit0，8/8；Ninja成功未运行 | review-r6-management.log.txt |
| 两变更Rust文件rustfmt --check --config skip_children=true | exit0 | review-r6-format.log.txt |
| 原生产反例及补充控制 | sidecar、旧Ninja身份/生成读取例及正负控通过 | review-r6-probe.log.txt |
| R5 core46/model38、C输入compile exit0 | 限定复用，未重跑 | review-r5-core-model.log.txt、review-r5-c-fixture.log.txt；reuse-inputs哈希与git差异核对 |

本轮相对R5仅rust_ui.rs及generation_request_adapter.rs变更；core/model源码、依赖manifest/锁、测试配置及C输入未变，复用范围限定为对应纯核心/数据与C输入检查。实际改变的生成消费路径已全部重新链接重放。未跑整档t/tv/tt/tb/tu/taa/tf、777、正式独立交付全集；无实现修复/合入/归档。

## AC / SD / 状态

| AC | 当前结论 / task / 依据 |
|---|---|
| AC-01 | pass（既有规则/核心委托与公开门面已验范围）；T01/03/04/05、core基线及实际生产重放；Builder效果完备性另由AC04约束 |
| AC-02 | pass（声明/请求读取矩阵）；T02/06，原Obj/Node/节点来源缺口本轮真实产物关闭 |
| AC-03 | pass（已声明支持域）；T03/04/07，图/786原例保持，Block/Obj/Node属性读取现一致 |
| AC-04 | partial；T06/07，两项work-owned验收证据仍缺 |
| AC-05 | fail；T08/09，新增health及闭合收据尚未齐全 |
| AC-06 | merge-owned pending；T10/11 |

SD-02三形态消费陈述现在与本轮行为相符，该具体规范冲突已关闭；SD-01/03/04保持现有核心/静态边界/效果adapter划分。SD候选及四路径/GOAL-003元数据已冻结核对；不得将未完成的AC04验收写为已通过。最终整体review仍不pass，未发布canonical知识或ledger。

outcome=needs_fix；status=executing；T01..05保留完成，T06因真实Ninja验收仍开放、T07因受影响独立交付证据开放、T08因health/收口开放、T09未完成、T10/11待merge；current_step=5/11。

next=work在同一树处理R6-01，补齐R2-04/06的实际绑定收据、校准当前任务行和handoff，提交后再审；已有sidecar关闭结论仅适用于该提交和已验范围，未来修改须复验受影响项。

pending_closeout_gates：最终code/acceptance/Spec pass后，merge在冻结提交执行裸cargo t及必要包/按最终diff需要的触面档；最终777/792并集、规范/ledger、归档、已合入接口SHA交接、wt-guard clean后清理仍pending。本轮未授予合入资格。
