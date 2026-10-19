# 814–816调度合同：阶段发布与并行审查修复（r2）

2026-10-20。按用户明确要求修订既有三份Plan，不增加编号。详细scope/接口发布门、状态/Spec增量、验证修复并行与失效规则以[共用执行合同](staged-delivery-review-814-816.md)为准；本报告为调度入口。原r1整项串行/不允许phase merge的限制已被本轮授权替代，原完整交付目标不变。

## 当前状态

809/I-PROJECT=b819a602e、810文法=f0ec8a91f、811媒体=d06cdc6f6均归档，实际commit为输入HEAD祖先。814/815/816主检出合同仍drafting/current_step 0，本轮未宣布实施完成。814已有登记实现树，本轮不检查/修改其内容，owner开始时读取r2并核实base。当前证据见[输入](../attachments/sdk-tools-r2-20261020/inputs.json)。

| Plan/阶段 | 可立即开展 | 必须等实际交付的门 | 最终仍要完成 |
|---|---|---|---|
| [814](../814-language-sdk-ui-package-separation.md) A | 真实纯SDK、UI compiler及旧默认兼容/消费者闭包 | 已合入807/808/810/811均满足；阶段审查与冻结合入才能放行下游 | B runtime/renderer/全部公开兼容/测试与发行 |
| [815](../815-ui-project-tools-and-assets-extraction.md) A | 轻协议/同源I-PROJECT codec、真实describe/metadata校验端点；资源清单 | 809已满足；不等814 | B实际UI生成/资源迁移等814-A；完整runtime集成等814-B |
| [816](../816-tool-execution-protocol-and-thin-cli.md) A | 现有执行设施与独立定向寿命控制 | 真实客户端等815-A；不等814或815-B | 管理adapter等814-A+自身A；最终CLI/UI切换等814-B+815-B |

每个A包含work候选、独立阶段review、merge冻结门/实际master合入/Spec子集与受管ledger增量/版本来源收据；不是只合接口草图。各Plan仅追加内部任务，保留原任务/最终AC。A不归档、不标整个Plan reviewed、不清理唯一树，B继续原最终流程。不能调用会归档整Plan的组合merge动作处理A；现有Runner未增加阶段调度功能，人工按Markdown合同组织。

## 审查和修复如何并行

814/815-A可同时实施/审查/修复；815-A发布后816-A可以真实消费。814-B修renderer时，815生成器/资产与816执行器/管理adapter的独立scope可同时审查、定向验证与修复；尚缺上游的真实集成记pending，独立审查不等于整体final pass。

同Plan同树单写者，审查冻结SHA后停止该树改动，修复提交后复审受影响范围；各树fixture/临时目录/端口分开。共享SDK/协议修复归owner，新发布后影响方同步并重放，不复制上游。共享manifest/777/792/Spec ledger/master变更、裸t及触面整档走串行合入队列；重型/GUI/OS全局资源预约独占，tf仍主检出到期单实例。

每轮先修齐已确定findings，回放共同反例与适用相邻状态组合再送审，不一条修复跑一次整档。发布证据绑定代码/依赖/配置/输入；相同签名不能证明语义未变，未受影响且适用证据可复用。详见共用合同§4/5。

## 共享写面与后续

工作树各一份，跨仓解析遵守既有顺序，不创建junction/symlink。主Cargo/锁、public导出、宿主安装、777规则/792来源及Spec索引按精确scope短窗口合流；后合入者同步master取并集、重审/重放影响部分，不能整文件覆盖或宽化forbidden。

三份B完成后才满足本批完整SDK/UI工具/薄CLI交接；R07发行与R08/R09旧仓承接、R10–R12退出目标仍保留。A能力发布不代表完整UI支持或三仓搬迁完成。历史r1来源见[旧输入](../attachments/sdk-tools-batch-20261019/inputs.json)与提交6a23c2e29；本轮[预检](../attachments/sdk-tools-r2-20261020/preflight.md)仅文档，无Cargo/docs_gen、无实现/agent/新工作树。
