# ACE后继波次：实际写面与交接合同（2026-10-28）

本合同按用户本轮授权修订862–866、870、871的执行边界；保持868原合同，不重置任何计划进度（初订时868在途，现已归档交付），不将规划预检当作实现验收。与[批次依据](ace-next-batch-2026-10-28.md)、[Design49](../../design/49-native-single-mir-lowering.md)配套读取。D编号表示独立复审、冻结门禁、实际合入、Spec及交接收据齐备；开发完成或局部测试通过不等于D。

**当前续接（2026-10-29）**：862/864/866/868/870/881已交付；871 executing r4（读取时4/9），865/863 drafting r2。M1缺口owner/立项状态、116行登记的适用性与接线差额、M2已推进能力及最终收口门见[当前对账](ace-m1-m2-closure-roadmap-2026-10-29.md)及[续接基线](../../design/strategy/ace-implementation-handoff.md)。下表第0/1波及已交付项为历史安排；未改原合同，禁止依据旧波次重复实施。

**当前队列**：S=871收尾→865→863，871未就绪让窗；N=native第二阶段在D864+D871后另立包，可与S并行；H=各计划及887实际触面单写。865/863交付后先做M1全分母证据/适用性复算，再补文本/诊断、场景与资源代际、a2r解析/发射/金样等剩余包，最后revision绑定验收。887整项等U884，只消费其首批范围，不承担全M1收口或新增ACE能力。新能力包均未分号，本轮不立实施计划、不施工。

## 1. 波次与实际交接

波次表示准备及开工优先级，不是等待整波交付的屏障。语义依赖与写面排序分别记录；只读调查、合同草拟及经登记的独立模块可提前开展，不消费上游WIP。

| 波 | 可推进槽 | 真实前置 | 实际写面及交接 |
|---|---|---|---|
| 0（已过） | 868交付；862 T-01/T-02完成 | D868已有归档/merge收据 | 搬移、消费者及SDK身份适配已落地；保留原合同，不追加新能力或重做搬移 |
| 1 | 862 ∥ 870后端主体 | D868；既有D798/820/821/828/852按各自合同 | 862占S、所需H；870 T-01..04只占N。870 T-05公共差分驱动/绑定集成等D862及H释放，再刷新共同基面；默认交付顺序D862→D870 |
| 2 | 864 ∥ 866私有常驻模块 ∥ 871库存/shadow准备 | 864←D862+D868+D822+D828；866←D862+D828；871正式切换←D868+D870+D852 | S默认864→866。871可做独立driver/CLI工作，公共H/N仍串行；ac-core删除另取S窗口，不把871整项当作纯driver任务 |
| 3 | 865 ∥ 863私有console/provider准备 | 865←D864+D862+D868+D823；863←D862+D868 | S默认865→863；863的前端、lowering、MIR/ABC/verifier/AV派发不能与865同时接线。H观察通道/SDK/共享语料注册另交接 |
| 插入窗 | 871最终删除 | 五个旧profile覆盖门G-COV全过，消费者切完，G-DELETE就绪 | 优先在864/866释放后、865接线前插入；未就绪立即让出，不持窗等方法等缺口，改在后续释放点交接。D871不承诺必在第2波完成 |
| 4 | native第二阶段：collections/variants及带位宽typed entry | D864+D871，及实际需用的上游能力 | N主体可独立；H接线串行。另立计划，用中央分配器取号，不预占“874”或其他编号 |
| 5 | M2数值/多模块/stdlib/泛型等功能线 | 各切片自己的真实D门 | 独立研究/私有模块可提前，不等无关整波；公共前端/IR/注册仍交接 |

历史S队列862→864→866后增加881方法包，现已交付；当前871删除窗→865→863按就绪交接。这是写面优先顺序，**不是新增语义D依赖**。调整顺序必须先登记释放/接受和兼容验证，不能只凭“文件冲突可解”并写。

## 2. 写面单写与释放

| 窗口 | 实际路径/合同 | 主要owner |
|---|---|---|
| S：共享语义与会话 | `experimental/ac-core/src/source/**`、`hir.rs`/`descriptor.rs`/`verify.rs`；D868后`experimental/auto-mir-lower/src/**`；`ac-av-probe/core/src/**`的MIR/ABC/verifier/AV及snapshot；probe的共享session/driver/worker接口 | 862、864、866、865、863依上表交接；871删除需单独加入 |
| N：native消费者 | `experimental/ac-mir-native/**`，包括CLI bin、lower/emit/resources/cache/link/worker | 870主体→871消费/CLI→native第二阶段；若871尚未就绪则不占窗 |
| H：公共消费/判定 | `experimental/ac-case-worker/**`、`crates/auto-lang-core/src/test_cases/**`的实际SDK适配器、`crates/auto-test-case/src/contract.rs`的recipe/通道/报告轴，`ace_m1_tests.rs`、`binding.tsv`、共享case注册及脚本 | 862重绑定→870差分集成；此后各计划按登记窗口串行，不因append-only而并写 |
| P：独立私有模块 | 已登记的新console provider、resident模块、独立fixture/脚本与只读调查 | 可并行；不得复制HIR/MIR/VM，不提前注册公共类型/opcode/路由；不改父侧oracle凑绿 |

862引入Unit等公开枚举若需要mn修改穷尽匹配，T-02登记S/N短窗：870暂停N、释放并由862接受，完成最小受控拒绝/旧域兼容适配后再交回；不把并行消费编译失败视为他方债务。方法覆盖能力包也要另按实际S窗口接线，不能在871删除中夹带实现。

申请时列出具体路径/符号、写入与只读集合、所有受影响消费者及需同时占用的窗口。跨S/N/H改动同时取窗；未满足前置不得持有一个窗口等待另一个。review/repair期间同一公共合同仍由原owner负责，直到最终复审、merge门、合入和兼容收据完成；计划不得把未合入定义交下游。并行读已合入基面不受此限制。

交接摘要至少记录：前owner与释放范围、计划修订号、合入revision、Spec/API/profile/lowering版本、下游及兼容结果/限制、cache键失效方式、未完成项；后owner在T-02或接线前刷新基面、复核证据并记录接受。后续修改影响上游消费者时重新验证受影响面。共享写面不得用复制WIP、临时第二份lowering或合入未过审实现来绕开。

旧AutoVM/a2r及拆分路线继续独立；任何实验Plan都不接管产品默认或拆分owner路径。H若涉及正在迁移的SDK/测试文件，先确认拆分owner实际位置及释放，必要时私有准备先行。不能仅以crate独立推断全无冲突。

## 3. 871覆盖及删除门

**G-COV**：T-01/T-02列全旧`core / scalar / text / record / text-record`五个profile的源码支持、诊断、CLI/协议/脚本、产物身份与cache消费者；每行有current Spec/旧测试依据、共享lowering+mn对等用例、独立预期、实测及owner。text必须保留自己的profile身份和拒绝边界，不能只经text-record通道冒领。至少含静态/只读实例方法、mut/move/clone、别名安全字段赋值、Unicode、借用/资源/全出口清理。

D870仅证明AV已有的data/owned子集，不自动满足旧方法能力。方法producer现为已交付的PLAN-881（db0773130，R08/R07 ac/av已闭）；871必须接受其方法/参数模式/资源/增量的逐域证据，不以包名代替G-COV。非方法gap依[当前责任表](ace-m1-m2-closure-roadmap-2026-10-29.md)保留owner；871不在迁移中暗增语言语义。既有合法行为不得用NA、unsupported、删用例或改expected消掉。已覆盖profile可独立推进，全量D871仍须全G-COV/G-DELETE。

**G-DELETE**：五profile各自G-COV、shadow、默认切换及全部消费者迁移通过，cache迁移/恢复重验有效，保留代码白名单与删除符号/依赖证据齐备，才允许删旧lowering。注入缺失方法/消费者行必须挡住切换或删除，旧可用路径保持可用。`text_resources.rs`是verify层仍使用的`TextResourcePlan`分析，不是整文件删除对象；保留该分析及verify调用，仅删除可证明独占于旧native的部分。

最终检查无旧lowering消费者与重复实现，保留所有旧语言行为，才可声明单源完成。局部profile切换不等于D871；也不由此宣布M1/M2或G7/G8完成。

## 4. 864源码范围与首例指标

864必须交付标量List、`List<Str>`及`List<非递归拥有型type>`（例如含`Str`字段的Item），包括852原L04/L05源码。复用822现有合法元素/Place/资源合同；record中容器字段、嵌套List、借用str元素、循环拥有图、任意用户泛型仍排除。合法拥有型正例不能以“诚实拒绝且有owner”代替验收。

构造、push/insert/get/set/remove/len/clear逐项冻结源码形态；get遵循Copy或受检只读view/Index Place，不隐式复制拥有权；remove转移拥有权；显式clone才复制资源。失败注入验证提交前原容器/输入所有权不变、全出口once-release。拥有型for-in首版使用不逃逸的受检只读元素视图，迭代期间结构修改拒绝，不隐式move或深clone。

各Plan首例即走文件化AutoDown/TestCase父侧判定，fresh/hit/restart/body-edit分别计量，cache恢复decode+verify、不恢复信任；不改既有expected或分母。沿现行开发期P90≤2000ms硬门与未改函数生产工作=0，P95/P99及亚秒长期目标另列；内存/资源峰值、live/释放和cache容量沿已交付预算交集，T-02冻结数字及测量口径，预算不能自行放宽。871增加切换前后及旧键失效比较，864增加拥有元素clone/失败清理和反复编辑后的内存回落。a2r金样轴与cert轴继续分开，既有NA/accepted-diff规则保持，不扩大豁免。
