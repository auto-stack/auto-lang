# PLAN-809 独立复审 R4：needs_fix

2026-10-19；review_scope=final，按用户要求重点复核 R3 修复及剩余验收。结论：**尚未全部闭合，不能授予 reviewed**。本轮发现的是既定 AC 的残留及修复回归，不新增迁移范围，也不批准延期。

SD 是 Spec delta（规范增量），不是 SDK。**PLAN-792 已完成、归档，8/8**；其 merge 记录含整合树 driver 25/25。这里检查的是 809 改动生成调用链后，对既有独立交付能力的受影响回归证据，不重开 792。

## 冻结基线与独立性

- plan_revision=1；受审 `6a84a7e28f5579b7c188b54b5ca86a9ae295f3a9`，前轮 `ef6f05cf439389d92ccbd9f1ca5ae4206abac962`；base=`2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 原实施树 `D:/autostack/.wt/lang-809/auto-lang`，分支 `codex/plan-809-dev`；auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。入场和收据前实现/依赖均干净，HEAD 未漂移，diff --check 通过。主检出并行推进及已有其他脏文件不纳入此代码基线。
- reviewer_identity：本轮 Codex 复审上下文，未参与本轮实现，未另开 agent；不虚构独立模型/会话身份。基于提交、实际生产探针与定向断言重新判定，不采信 handoff 的通过总数。
- [输入哈希/测试结果](review-r4-baseline.json.txt)、[冻结 SD](review-r4-spec-delta.md)；SD 包含主 Plan 原表、工作树四份候选正文及主检出相关权威输入哈希。未发布 canonical Specs/ledger。

## 原 finding 闭合表

| 原 ID / 不变量 | R4 结论 | 证据及边界 |
|---|---|---|
| R2-01 Ninja 首遇实例身份 | CLOSED 保持 | 原三态 app_first/lib_first/embedded_only，新 Builder 与冻结旧循环的实际文本一致；非只测排序 helper |
| R3-01 请求侧 sidecar 单源消费 | 原节点反例 CLOSED，完整支持域 OPEN | request_only/disk_only/both 三态现正确；但原支持的对象声明在请求路径漏失，见 R4-02 |
| R3-02 Block 列表 | OPEN | 原 Block 输入重放仍 `actual=[]`，期望一个模块；新增断言没有对应实现，且 test 编译失败 |
| R3-03 源清单及调用上下文 | PARTIAL | custom/front 主文件及 pages/child.at 已进入 input_sources；Generate/Run/output 新投影定向过。gen 本工程仍直接 request_of_in，绕过应用 render_override 的 gen_request_as；不能声称有效后端全入口闭合 |
| R3-04 health | PARTIAL / OPEN | 旧死 helper 已删；新 `from_request_pub` 在普通库构建新增 dead_code 警告，且 core lib-test 编译失败。七文件格式通过不等同 health 全清 |
| R2-04 完整三 Builder 效果矩阵 | PARTIAL / OPEN | 真实 Cargo、Vue 成功和失败及新不可写输出负控过；Ninja 成功分支本轮显式 pending，且新增 C 输入本身非法，见 R4-03 |
| R2-06 SD / 受影响独立交付收据 | 未验证关闭 | 受审修复 diff 仅七个 Rust 文件；可定位 SD 仍含与实现冲突的旧边界，主 Plan 最后 work 行和 handoff 仍写 pending。没有定位到用户所述新收据，不断言从未运行；不能用 792 原交付收据替代未知 809 基线的新证据 |

原 Ark run、Vue 主声明/依赖读取、front/仅中文标题、PATHEXT 工具发现及 786 围栏的已闭原例，本轮原探针/15 项定向保持通过；不重新列为缺陷。

## 可操作 findings

### F809-R4-01（P2，修复回归）：core 必需定向测试无法编译

AC-01/03/05，T-03/08/09。`crates/auto-man-core/src/sidecar.rs:7` 移除 AutoStr 导入，但 tests 的 :113、:115、:143 仍引用该类型。实际执行 `cargo test -p auto-man-core -p auto-project-model --lib` exit101，三处 E0433，故交接“core47/47”不适用于受审提交。本轮不计该组合中的 model 已通过。

修复测试作用域导入并重跑必需定向测试；同时修旧 F809-R3-02：:87 的 str_list 仍只处理 Array/Str，不处理原支持的 Value::Block。独立生产探针已绕过 lib-test 编译错误证明 Block 模块仍从1变0；只补 import 不足以关闭。

### F809-R4-02（P2，R3-01 完整不变量残留）：对象 sidecar 生成成功但静默丢模块

AC-02/03，T-03/06/08。`crates/auto-man/src/rust_ui.rs:3293..3316` 只把 req.raw_nodes 还原为 root；未还原 raw 属性 `rust_sidecar: Value::Obj`（以及 core 声明支持的属性值兜底）。core 本身的 sidecar_decl_from_root :29 支持 Obj，旧 load_sidecar 同样支持。

真实输入：`name:"obj-side"\nrender:"rust"\nrust_sidecar: { modules: ["extra:extra.rs"] }\n`，同一 app.at/extra.rs。实际旧入口和 `_with_request` 均返回 Ok；旧输出 extra.rs=true，新=false。请求中 raw_sidecar=true/raw_nodes=0，定位为消费侧丢失，不是用户缺源。见 [可复用原输入](review-r4-sidecar-obj.rs)、[生产日志](review-r4-sidecar-obj.log.txt)。

按原节点/对象/嵌套支持域恢复求值声明；分别覆盖 modules/deps、请求与磁盘方向反转，继续保持不回读主 pac.at。无须把 sidecar 功能扩成新合同。

### F809-R4-03（P2，R2-04 验收修复回归）：新增 Ninja 成功输入不能被 C 编译器接受

AC-04/05，T-06/08。`crates/auto-man/tests/plan809_management_core.rs:610..612` 的 Rust `\n` 生成 C 字符串内部的实际换行；提交的 C 字节为 `printf("ninja-ok` + LF + `");`，不是 C 的转义 `\\n`。

本轮管理测试输出 V05-PENDING（PATH 未找到 gcc/clang/cl），所以8/8不包含真实 Ninja 成功。独立找到已安装的 VS cl.exe，用合法 MSVC/SDK include 编译完全相同输入：exit2，C2001 常量中有换行符；仅把字符串内换行改为转义的正控 exit0。见 [提取与编译重放](review-r4-ninja-fixture.py)、[原始结果](review-r4-ninja-fixture.log.txt)。这是输入缺陷的证明，**不是一次完整 Ninja build 成功收据**。

修正输入，并在选中工具链确实可用的环境执行原合同所需的真实 Ninja 成功产物检查；保留工具缺失为 pending。不要把内存输出断言、编译器探测或本轮 C 正控替代整个 Builder 验收。

### 继续开放的原 finding

- F809-R3-03：`automan.rs:1517..1529` 的本工程 gen 在 resolve_backend 后构造 self_req，但未带已选 backend；:644..670 的 render_override 修复只在另一个 helper。请求可能仍表达 pac 默认后端。此为来源/覆盖合同检查，未虚构新的运行失败；补真实 gen 覆盖反例确认完整接线。
- F809-R3-04：`rust_ui.rs:3185` 新 `from_request_pub` 普通库构建 unused，见管理日志 :4458。只处理本轮新增警告，不把其余既有警告全归809。
- F809-R2-06：SD-02 仍说 sidecar load 属于“单工程外，R04收编”，与本 Plan T01/AC02 和现修复冲突；全入口/无丢失表述也尚未被验证。需提交准确候选正文及可定位、绑定修订的受影响交付证据。若复用既有结果，应给出代码、依赖、测试配置及输入未变的依据，不要求重开792或无关全档。

## 本轮实际验证

| 检查 | 实际结果 | 证据 |
|---|---|---|
| core/model 定向 lib | exit101，core 三处 E0433；不计47/47 | review-r4-core-model.log.txt |
| 管理/Builder 串行 | 8/8；Cargo/Vue 实际执行；Ninja 成功显式 pending | review-r4-management.log.txt |
| 原生成/786及新 adapter 定向 | 15/15 | review-r4-generation.log.txt |
| 七个变更 Rust 文件格式 | 全过 | review-r4-format.log.txt |
| 原 R2/R3 生产反例及正控重放 | Ninja 三态、Vue/Ark、sidecar 节点反转、custom/front 已修；Block仍空 | review-r4-probe.log.txt |
| 对象 sidecar 真实生成对照 | 旧复制模块，新静默丢失 | review-r4-sidecar-obj.log.txt |
| 新 Ninja C 输入与单变量正控 | 原输入 exit2；escaped-newline 正控 exit0 | review-r4-ninja-fixture.log.txt |

`review-r4-replay.ps1` 编译未改动的 `review-r3-probe.rs`（其中复用原 R2 反例），再运行对象及 C 输入探针；打印当前生产 rlib 哈希。原主探针输出 exe 路径沿用 target/review-809-r2-probe.exe，但每次均重新编译，不借旧可执行文件。探针 exit0 仅表示观测执行完，须按表中不变量解读。未把 raw_args=None 或 generic display_title 与 Vue 自身语义差异误列新缺陷。

未重跑正式792 fresh/重定位locked、777、独立 consumer 全集或未覆盖的工具矩阵；不把前轮结果改记成此提交实跑。只做定向验收，无整档 t/tv/tt/tb/tu/taa/tf，无实现修复、合入、归档。

## AC / SD / 路由

| AC | 结果 / task / 依据 |
|---|---|
| AC-01 | partial；T01/03/04/05/06；核心生产规划原例过，但必需 core 验收编译失败 |
| AC-02 | fail；T02/06/08；请求节点接线已修，Obj 消费和有效后端仍有缺口 |
| AC-03 | partial；T03/04/07；图及786原例保持，旧 Block/Obj 支持域未保全 |
| AC-04 | partial；T05/06/07；Cargo/Vue过，Ninja/受影响独立发行证据不足 |
| AC-05 | fail；T08/09；health、SD及闭合表不能授信全闭 |
| AC-06 | merge-owned pending；T10/11 |

SD-01/04候选基础边界存在，但当前测试/副作用矩阵不足以授信整套 SD；SD-02/03 的全量/无损/调用上下文和 sidecar 边界需先校准。四个 Spec 路径及 GOAL-003 元数据保留，不表示规范增量已审过。

outcome=needs_fix；status=executing；保留 T01/02/04/05，T03/06/07/08 继续开放，T09 未完成，T10/11 待 merge；current_step=4/11。主 Plan 末尾“R3全闭/core47”保留为历史 work 声明，由本记录纠正。

next=work 在同一树修 R4-01..03 及原 R3-02/03/04、R2-04/06；更新漏检根因和支持形态/生产入口/实际工具矩阵的 oracle，提交最终修复和准确 SD/收据再审。记录 debt 不代表批准延期。

pending_closeout_gates：最终 code/acceptance/Spec pass 后，merge 在冻结提交统一跑裸 cargo t 及必要包和真实触面档；最终777/792并集、知识/ledger、归档、已合入接口SHA交接及 wt-guard clean 后清理仍待执行。本轮不授权绕过这些门禁。
