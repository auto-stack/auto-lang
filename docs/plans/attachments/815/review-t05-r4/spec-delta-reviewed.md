# PLAN-815 T-05 R4 / T-06 / T-08 当期规范核验

Plan r3；review_scope=phase/提前定向；冻结02401f77cc31c0c4196f588b6b1ccfe2ae10d530。完整B规范尚未准备完成，本文件是审查核验，不是可发布的规范稿。canonical来源hash见baseline.json.txt；交接稿原件冻结为frozen-handoff.md（SHA256 9ee3ebfaf08637387fbbb28a5160563483c2594a77cc22a713acb82c8d5c2f03）。

| ID / target | 本轮核验 |
|---|---|
| SD-01 / docs/specs/auto-ui-tools/project.md | 实际已加build_ui及多profile分派；单profile生成输出回报、多目标生成、构建失败/取消仍有反例。不得沉淀为完整真实支持面已验收。VM原外部模块/初始化q原例修复成立，串行上下文及后端装载失败语义仍需修复。 |
| SD-02 / docs/specs/tool-protocol/project.md | 版本1新增BuildUi/BuildOk/BuildReport，五能力。取消回执与工作终态必须分开；build终态为BuildOk、run为RunOk。现有19协议测试不等于新增执行矩阵通过。原非Windows有界清理缺口仍在，不能把Windows结果外推为全平台保证。 |
| SD-03 / docs/specs/auto-man/project.md | 六个兼容/媒体/请求缝定向测试通过。旧CLI正常行为不能作为协议侧静默吞构建失败的理由；完整旧入口/媒体/sidecar/backend矩阵仍T-06/07，不以调用了同名旧函数代替保持语义。 |
| SD-04 / docs/specs/project-model/project.md | 请求模型无改动；VSCode generate和Vue build等新分派却走legacy主pac读取，违背AC-02。合法Multi请求不能只生成第一项却回报全部frontends。无需修改数据合同或放宽输入以迁就这些缺陷。 |
| SD-05 / docs/specs/auto-lang/ui/overview.md | 依赖方向保持；子进程所有权必须覆盖新增Jet/Ark/VSCode/Tauri实际外部命令。直接Command::status绕过owned设施，不能声明控制能力已统一。 |
| SD-06 / docs/specs/repository-boundaries/generated-ui-distribution.md | DELIVERY26及既有清单输入未变；T-07并集/fresh/locked/792/资源完整验收仍待审。新交接稿以.rs路径代二进制身份，缺源SHA/二进制SHA/支持指纹，无法提供冻结消费身份。 |
| SD-07 / docs/specs/auto-ui-runtime/project.md | 主Plan已登记公开退出窗口API候选；初始化q两例及已呈现q正控成立。交接稿漏SD-07，新增窗口实现产生两个编译诊断；完整规范正文须纳入实际生命周期及失败边界后送T-09。 |

T-08当前只有SD-01..06路径索引，没有逐项before/after正文，工作记录也明确正文留T-09前。按任务原依赖T-07尚未完成，本轮只确认准备稿partial，不将已承认的待办重复编号为新产品finding。须补SD-01..07正文、真实冻结身份、来源/能力矩阵，纠正Cancel终态和全profile保证，与修复后的实现重新对账；不能提前发布规范或交付M816-B。

impact元数据保持主Plan：supersedes含runtime/Man/UIoverview/model/distribution；new含tools/protocol；GOAL-003/007。A/L与T-04历史发布不回滚，未修改canonical或live ledger，plan_revision保持3。
