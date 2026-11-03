请接续跟踪ACE（Auto Compile & Execution）架构实现路线。工作目录`D:\autostack\auto-lang`。先读取仓库AGENTS.md、`docs/specs/overview.md`与`docs/design/strategy/auto-native-backend-evolution.md`，再读：

- `docs/design/strategy/ace-implementation-handoff.md`：上一长会话的架构决定、有限交付基线、未完成计划和工程纪律。
- `docs/plans/reports/ace-wave-handoff-2026-10-28.md`：公共单写/交接纪律；历史波次按当前已交付状态读取。
- `docs/plans/reports/ace-m1-m2-closure-roadmap-2026-10-29.md`：当前M1缺口owner/立项状态、M2已推进能力、865/863后的收口顺序及原始登记数限制。
- `docs/plans/reports/ace-m1-scope-decision-2026-10-28.md`：M1分母、NA/accepted-diff、a2r金样与认证双轴规则。

当前架构：Rust版ACE包括AC/AV/AJ和A2X，共享语义/meta/身份/增量/cache服务；执行走Checked HIR→共享Typed MIR→ABC→AV或Cranelift→native，A2X主要消费结构化HIR。最小Typed MIR已纳入v0.6，不再等v0.7；v0.7再实施Auto版AACE（AAC/AAV），先满足Rust版现有AutoVM/A2X全示例成熟度。M1核心/共同子集、M2语言、M3混合生态、M4产品替换分别验收；完整AJ不是替换旧引擎的前置。

CTE由语义查询调度真实AV，结果冻结并回灌推导/HIR；796/797/817目前只是有限语义/ABI/标量CTE合同，不能宣称完整泛型、反射或Rust/Python FFI。meta/类型/实例/CT结果进入统一cache，命中须包括helper body、类型查询、target/配置/权限与受控外部输入的完整依赖；不能仅看调用点源码。

读取时快照（2026-10-29；先按活动/归档Plan、权威树与merge收据刷新）：

1. 862/864/866/868/870/881均已交付。881 delivery=db0773130，R08/R07 ac/av已闭；方法唯一lowering仍在experimental/auto-mir-lower，不再重复立方法包。入口、集合、方法、CTE/native都只认各自有限profile。
2. 871 executing r4（读取时4/9）：消费者/cache、旧实验HIR→CLIF退役、独立终审/冻结门/合入按原G-COV/G-DELETE收尾；TextResourcePlan的verify分析保留。未交付WIP不交下游，不等于旧AutoVM/A2X产品退役。
3. 865 r2、863 r2仍drafting。公共S队列871收尾→865闭包源码→863 host-import/console；871未就绪让窗，不给865/863新增D871语义前置。865不含map/filter/reduce，863不含所有宿主服务。
4. M1原始binding当前116行=76 corpus+40 scenario，含旧gap/已批准NA或差异未回填/未接线证据。不能用50/103、20/73旧测算或raw gap计数宣称当前clean。typed闭包ok不等原源支持；863完成不自动签M1或M2。
5. 详按M1/M2对账的M1-01..10：native集合/变体与带位宽typed-entry（D864+D871后另立包接N）、T03/T05共享文本、B17–19诊断/infra、场景观测/绑定、资源fault/句柄代际、a2r解析/未定型发射/缺golden、适用规则与全分母验收。各未立项包只是规划owner，不预分号、不擅自扩大871/865/863。
6. 865/863交付后先完整复算并冻结剩余项，再按共享文本→诊断→场景/资源→a2r的默认优先级规划有界包；独立N/R/H可并行，H注册/SDK/binding/比较单写。M1须适用格真执行、阻塞gap/probe/wait/pending清零，NA/accepted-diff有规则，a2r金样/认证分轴，原增量/恢复/资源与P90≤2000ms门成立。
7. M2数值源码/算术、模块/全局、stdlib/HOF、错误传播、用户泛型/meta/反射/复合CTE/必要FFI继续按实际缺口切片，与M1收口交叉。完整AJ/发布器/block级增量不作为统一M1前置。
8. 884 reviewed（读取时8/10）不等完整交付；887整项等U884。887只做其测试治理、B/U复用和三条基础旧语料新增ACE消费，不是全部M1场景/能力/最终验收的owner。885/886及图形线仅按实际触面交接，不自动接管本队列。

共享窗口S=语义/IR/runtime/session，N=mn，H=公共driver/SDK/测试注册/binding。前owner给delivery、修订、Spec/API/profile/lowering/cache版本、兼容证据/限制与释放清单，后owner接受。跨窗同时取，不持窗等上游，不复制WIP或提前合入未过审实现。需要新包时现场按中央分配器取号，本次续接不自动分号/开工。

全线首例即纳入文件化AutoDown/TestCase、独立oracle、fresh/hit/restart/body-edit、恢复重验、未改函数生产工作=0与内存/资源预算。长期亚秒目标保持，现开发期正式硬门为P90≤2000ms；不重置旧任务进度、不改expected/分母凑绿。a2r日常比金样，编译运行认证独立；M1沿已批准NA/accepted-diff规则。拆分867/869/872/873及图形线按实际owner交接SDK/共享测试，不能仅凭crate独立认定无冲突。

主检出已有多会话未提交docs WIP，请保留；实施按auto-plan四技能及唯一worktree，开发/repair只定向检查，冻结后的整档归merge；附件原始证据仅本地，只有Markdown摘要入库。旧归档824/825存在进度计数/收尾收据不齐的记录，若影响依赖先核实，不自动重开旧计划。

本轮先只读刷新上述计划及实际交付/写面owner，给出“已完成、正在实施、可并行开工、仍缺前置、下一步优先规划”的简明清单；保留既有决定与实施进度，再按我后续指定处理具体计划。不要自动新建其他会话、发消息或批量启动实施。
