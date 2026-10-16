# PLAN-798与796/797、787/788及拆分路线的并行合同

2026-10-16，PLAN-798 r1附件。它是798的约束，不修订796/797的任务、状态、revision或验收，不授予修改其他计划工作树的权限。

## 写面与依赖

| owner | 独占主写面 | 798消费/禁止写面 |
|---|---|---|
| 798 | 新experimental/ac-av-probe、test/cases/ac-av、798脚本/报告/附件；D-796后仅追加自己的recipe/worker | 只读已合入ac-core/783/796公共API；不改其推导/底层store/生产native/旧VM |
| 796 | 共同语义ref/Schema、有限推导/查询/CT模型、probe/run公共IPC/注册接缝 | 798不另造权威TypeStore，不抢改decoder/mod/CLI；最终用真实合入库并退出临时语义ref快照 |
| 797 | Windows x64有限AutoABI/C互调/target布局/资源 | 798不接原生FFI，不定义物理AutoABI；内部AV调用/ABC版本与之分离，无797前置 |
| 787/788 | syntax/分页及其store热点；TestCase v2、native/syntax worker、预算/观测和共享协议 | 不搬/cherry-pick在途代码；798正式AutoDown等D-788及D-796 |
| 拆分计划 | auto-lang/auto-ui/auto-man及diagnostics/scroll/package-lock等生产写面 | 不改根members、产品TypeStore/Codegen/VM、诊断/宿主/包锁；新workspace不等于可重复抽取公共库 |

实施前重核当前diff、依赖闭包/合入receipt，源码路径和版本相同才可复用证据。共享注册/manifest和Spec导航安排单写者短窗口；两个分支都改同一公共协议不算并行隔离。

1. T-01–05只需已交付ac-core/783，可以与796/797并行；只读其计划快照不能解锁正式接口。
2. D-796须独立review、冻结门禁、实际合入及Spec/merge收据齐全。T-06后真实消费其ref/codec/语义描述，重跑全部受影响验收。
3. D-788同理。796的公共probe/run交付本身已等待788，798 T-07只追加已合入接缝的独立recipe/worker，不能修改在途worker/decoder或自己做.ad parser。
4. 797不是798前置，798也不是796/797前置；只有将来AV↔native adapter才消费其原生ABI。最终兼容只对实际合入recipe取证，没有797合入时不宣称其兼容已通过。
5. 若正式公共接缝不支持所需输入/资源/会话，凭真实字段/用例回规划，由公共owner显式修订；不保留第二长期接口绕过等待。正式796绑定/AutoDown仍pending时不能execution_done。

## 共同事实与未来能力

796拥有语义身份/类型/Meta，798拥有有限MIR/ABC/AV表示、验证和执行；target布局/原生ABI归797。i32/bool以外元信息可由796表示，不代表798能够执行。CTRequest/CTResult由796冻结，本计划不真实调用CTE，不修改其模型计数或伪报AV结果缓存能力。

逻辑单元/ref、版本/profile/依赖、内部调用与代际、可重定位原生产物准入从798第一例设计；AJ、画像、在线跨层替换、OSR/deopt和发布器仍future。native reference从已合入HIR直达Cranelift，与新MIR/ABC独立；双方都错时独立expected必须抓住。

store继承783的受管root/lease/GC与图边语义；A08/A09新producer有自己的内层Schema/carrier准入，不把旧producer的Future改成Codec，不修改787正在使用的底层。正常执行cache命中仍须av_execute>0且重新比较；禁调MIR/ABC producer证明实际跳过。CTE冻结值跳过执行属于后续计划。

AutoDown源/oracle、Rust定向探针及真实span从T-01出现；正式统一runner状态在依赖前只能是pending。最终P95<1000ms含adapter/IPC/排队/比较和必要检查，不删失败样本、不以阶段P95相加替代、不换空查询或提高阈值。

只有798的唯一组树用于实施/自检/独立review；不得创建链接。最终门禁和批量回归按AGENTS.md后置/主检出单实例纪律，不因AV路线触发旧AAVM整档。
