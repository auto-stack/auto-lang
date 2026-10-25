# PLAN-848 r1 planning preflight

2026-10-25；stage:new；草案静态检查通过，G846完整交付pending。

- Plan SHA-256：`ca8642cf9a08f9f2bdea23612edf2d7361f2ecea260cd4bff6404830ba00b951`。
- 结构：0..10节、10任务/10AC/12case/4Spec delta唯一；owner_stage = work8/review1/merge1；所有AC有任务负责，状态drafting0/10。
- 取号：主检出既有allocation.lock排他句柄下重核active/archive max和counter，用new-plan.sh分配848，next-id849。847属于probe-test边界路线；本轮只创建一份图形草案，扫掠/运动后继未取号。
- 来源：26个固定Git对象按其各自ref核blob/SHA-256/字节数通过；16份文档输入hash与尾检一致。候选846=655c366a3a4c53160c259a0b91a50c02d1cb020a、SDK=7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467，正式集成仍为845=c1c7cfea5c9cc84987091ddeaf6cde59cefd5d4f；候选不作848执行冻结。
- 上游：调查从846 execution_done9/11进入review:R1 needs_fix/executing6/11，F-846-R1-01为CLI --behavior-api缺值未拒绝；R1披露同实施上下文，非最终独立pass。新Plan明确继承修复后的选项presence/value检查；有效最终审查、完整根/driver门、SDK/图形分离落地、Spec/ledger/archive/guard/到期均须正式收据，当前G846未闭合。
- 语义：有限world segment/AABB closed slab、nearest按f32 t再ordinal、inside/零长/终点/parallel/退化与正常miss均已定义；过滤与unavailable/数值错误不混淆。结果是查询瞬间单槽快照，正token仅同callback、后续cast失效；miss token仍可读，错误占位不代表成功。
- 兼容：v3/profile9显式，v1/v2 API/managed hash/reserved集合原样，同raw按API分别准入；scene4字段不改。默认/绑定/临时覆盖、Undo/Verify/来源与旧CPU/Stop隔离保持。
- 资源：v3 weighted32credits（cast8/getter1/AABB1），128节点宽/深、2cast+10get+6AABB、最多2组写后查为合法预算控制；slot原值<=16标量，warm不扫模型/上传/读源。T02最多两种局部实现验证实际Auto/GPU/项目全链，无法闭合回new，不抬预算或改SDK。
- 证据分工：V01..12/AC/work owner覆盖独立数学参考、旧world/旧token、NaN/Inf producer、same-raw API误用、真实OS/GPU/partial、仓外同binary两例及20轮owner/扫描/上传反控；所有新增fixture由work建立，非本轮已跑。
- 时序与边界：本轮无848树/branch、prepare/构建/Cargo/docs_gen/原生/GPU、agent/chat，无846修复/merge或canonical改写。未来work/review定向，final独立pass后merge冻结完整门，source只落autographics-dev，原生产/迁移门保持。
- 尾检：相对链接/锚点/附件存在、ignore与空白/scoped diff检查通过；16份规划输入未改。840/845/846/841与Schema/ACE/HMR合同未修改。

结论：草案可评审；G846完整交付后，后续work指令启动T01重核正式基面、T02验证有界可行性。未宣称任何实现或运行验收完成。
