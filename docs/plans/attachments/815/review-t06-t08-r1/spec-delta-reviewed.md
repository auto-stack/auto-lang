# PLAN-815 T-06～T-08 提前定向规范核验

Plan r3；冻结bb4f0a1caeb252ccb82d722659ee827703b82fc1（codec6fdaaaf1）；非T-09终审。用户明确要求T-06～08提前检查、与T-05同轮修复。frozen-handoff.md为当期正文，canonical hashes与来源见baseline.json.txt，未修改canonical/live ledger。

T-08实际源SHA/bin SHA/五操作矩阵及SD-01..07正文已存在；本轮探针使用的冻结bin沿R6核验为c95748c80b512215d89d6c987d3b8b708f697d102a24a62333f7666051c42b61。结案时并行工作已重建二进制，属于未审输入，起止指纹见final-baseline.json.txt。身份正确不能替代支持矩阵、字段与验收事实。

| SD / target | 提前核验结论 |
|---|---|
| SD-01 docs/specs/auto-ui-tools/project.md | 独立工具/资产owner及已闭生成范围保留；Ark run仅准备工程、VM front路径未贯穿，不能沉淀“完整真实运行/全部请求接线已成立”。T-04生成阶段pass不覆盖新增宿主运行行为。 |
| SD-02 docs/specs/tool-protocol/project.md | framing与五操作形式已定义；T-07 consumer五测通过但服务端缺四种正式回执与超深负控。run_ui矩阵将Ark标interactive/cancellable而实际仅生成，结果语义须校对。R6捕获失败回报未闭继续关联，未由新WIP自动覆盖。 |
| SD-03 docs/specs/auto-man/project.md | 旧门面语义按R6关闭边界保留；手工引导式旧入口不能未经区分成为协议“实际执行完成”。T-06须对照旧oracle并真实记录协议服务边界，不能以更改测试/文案自行削减原目标。 |
| SD-04 docs/specs/project-model/project.md | Vue自定义front正控消费指定文件，VM同字段请求验证ok却仍读src/front/app.at。front节点/输入/输出/dep/theme/sidecar等跨profile字段来源表仍需T-06/T-07补齐；本轮只实证front缺口，不把所有待核字段猜成已失败。 |
| SD-05 docs/specs/auto-lang/ui/overview.md | 公开SDK依赖方向大体保持；纯数据消费者闭包脚本漏实际UI文法实现auto-ui-dialect、以及lang-infra auto-bindgen，不能用当前PASS称其机制已钉住纯数据边界。实际未注入的生产依赖并未被指称已经违规。 |
| SD-06 docs/specs/repository-boundaries/generated-ui-distribution.md | 当前指纹可验证；777本分支strict因清单锚过期失败，792/master并集/fresh/locked发行/资源审计明确未完成。DELIVERY26或旧T-04收据不覆盖完整新工具B面。先明确每门输入/版本/正常与失败oracle再执行，不能只补绿色收据。 |
| SD-07 docs/specs/auto-ui-runtime/project.md | R6已关闭Unix具名4项及VM窗口机制保持；不能外推完整Linux/所有profile运行。新检查不回滚这些已闭证据，完整运行与验收仍由当期矩阵限定。 |

SD-01/02目标在A发布后已存在，B应基于当前A正文增补，before/after须区分“原迁移前历史”与当前已发布子集，保留原A来源。T-08定稿必须回填T-07真实完整门禁/适用边界和本轮支持事实；当前只有准备稿核验，不能勾完成或提前交付816-B。

impact保持runtime/Man/UIoverview/model/distribution、tools/protocol、GOAL-003/007，plan_revision=3。已明确未做的T-07范围登记为待办，不伪装此次提前检查已经覆盖全部AC，也不把新发现的交付缺陷降格为可忽略债务。
