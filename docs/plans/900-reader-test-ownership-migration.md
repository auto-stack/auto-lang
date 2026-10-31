---
plan_id: PLAN-900
status: drafting
feature_name: Reader验收驱动与夹具归属迁移
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
plan_revision: 1
current_step: 0
total_steps: 6
supersedes_spec_components: [docs/specs/auto-lang/vm/design/handler-execution-budget.md, docs/specs/auto-lang/ui/design/native-acceptance-channel.md, docs/specs/auto-lang/ui/overview.md]
new_spec_components: []
touched_goals: []
affects: [auto-reader, test-infrastructure]
---

# [PLAN-900] Reader验收驱动与夹具归属迁移

## 0. 变更摘要

按用户要求将auto-lang中Reader验收夹具与对应两份驱动迁至auto-reader，解决应用测试寄居语言仓及外部技能目录隐式依赖。独立于PLAN-899 ACE清理；不修改Reader产品行为、语言VM预算语义或AutoUI框架能力。本轮仅立项，依仓规L1先展示计划再实施。

## 1. 目标

- G1：两个fixture目录、生成器与应用验收驱动由auto-reader拥有，auto-lang不留重复实现。
- G2：auto-reader可从任意cwd使用显式auto二进制运行驱动，不导入auto-lang源码树/技能目录。
- G3：现有用例、输入hash、oracle、拒写/冷重启/进程收割与退出码保持；当前文档/调用入口正确承接。

非目标：改产品、削减测试/样本、迁全部Reader历史脚本、迁通用MCP框架实现、清缓存、更新其他应用/组合仓pin、push或发布。框架通用预算/滚动/MCP单测仍归原owner，本包不搬其生产实现。

## 2. 架构方案

| 当前auto-lang路径 | 新auto-reader路径（新建） |
|---|---|
| test/fixtures/native-reader-acceptance/ | tests/fixtures/native-reader-acceptance/ |
| test/fixtures/reader-import-budget/ | tests/fixtures/reader-import-budget/ |
| scripts/tools/test_native_reader_acceptance.py | tests/native/test_native_reader_acceptance.py |
| scripts/tools/test_reader_import_budget.py | tests/spec/test_reader_import_budget.py |

原生fixture是Reader场景的框架正对照，非真实Reader源码副本；按用户指定归Reader集成验收，框架Spec引用外部入口。应用驱动在Reader侧自带必要的轻量MCP客户端适配，保现有请求/截图/错误语义；不搬整个autoui-verifier技能，也不改其通用客户端。若现有已交付公共包可直接消费，T-01核证后采用等价依赖，不新增未经核实的发布前置。

CLI明确接收auto-bin（现有原生驱动已required；导入驱动统一显式参数或AUTO_BIN），Reader根与fixture位置按脚本自身位置解析；app-dir/reader-dir现有显式覆盖保留。不得硬编码D:/autostack路径或回退到语言仓旧副本。auto二进制属于必要运行依赖，不等同于依赖语言仓源码。

## 3. 技术栈

现有Python标准库、Auto split-VM HTTP/MCP、.at fixture与确定性生成器。不引入新测试框架。代码改动跨两个专用worktree；通过来源hash与逐项收据保迁移可追溯性，跨仓无单事务，先目标可用后源退役。

## 4. 需求分析与背景调查

### 4.1 用户授权与边界

用户明确提出这两个Reader测试目录应迁到auto-reader。本轮已授权调查及准备迁移，依auto-lang AGENTS.md L1“present for confirmation before executing”展示具体合同后实施。不创建代理/聊天，不联系其他会话，不改其WIP。此前ACE方案未获确认，本包不顺带执行899。

### 4.2 事实来源

- auto-lang已提交基面cec583174（调查后实施前重核）；规范入口docs/specs/overview.md。
- auto-reader main当前154b1ca0e07e2be6f56a75bede1485634c423e13，git status干净；根无AGENTS.md，实施前继续核父级及目标子树指令。
- auto-reader/README.md测试表仍指向auto-lang/scripts/tools/test_reader_import_budget.py，已有tests/{fixtures,review,spec}，fixtures当前有library目录。
- test_reader_import_budget.py实际启动Reader，执行10MiB TXT/MD导入、hash/章逐字、冷重启、低预算拒写、坏编码与双实例；fixture缺席时生成。
- test_native_reader_acceptance.py为Reader/最小对照app提供真实窗口几何、锚点、保存/恢复与故障控制；当前从auto-lang/.agents/skills/autoui-verifier/scripts/test_vm_mcp.py导入AutoUiMcpClient/pick_free_port。
- scroll-geometry/src/front/app.at含120行与scroll-anchor；save-restore是ASCII120段框架正对照，真实HTTP落盘/恢复，不能误称等同于Reader完整多字节恢复验收。
- 当前Spec引用：docs/specs/auto-lang/vm/design/handler-execution-budget.md、ui/design/native-acceptance-channel.md及ui/overview.md；历史781文档保原证据，只补当前迁移指向。

唯一实施组D:/autostack/.wt/lang-900/{auto-lang,auto-reader}；语言分支plan-900-dev，Reader分支codex/plan-900-reader-tests。每仓一树，实现与复审按同树顺序进行，无junction/symlink。主检出只维护本计划/规范；目标Reader源码、README与测试改动均在目标树。

## 5. 详细设计

T-01固定已跟踪来源及生成资产边界，记录文件hash和case清单。小型确定性种子/生成器按现有规则入库，大型生成文件及执行证据留本地，遵守>5MB门禁；迁移前后用生成器重建核hash，不把现存未跟踪数据误认为可随意提交或删除。

T-02在Reader创建目标目录、搬迁两驱动和fixture并改根解析/CLI；MCP适配仅承接驱动实际使用方法与错误语义。自适应截图、进程终止/端口释放和隔离数据要求保持。源脚本待目标验证成功后由T-04退役，无长期双份copy或静默回退。

当前README/规范指向新入口并说明应用外部验收与框架单测各自职责；历史归档/证据不批量重写。全仓活跃调用者搜索核源退役，源可执行入口不留空壳伪PASS。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/design/handler-execution-budget.md | Reader deep oracle路径由本仓迁至auto-reader；语言预算单测不搬 | 应用归属 | AC-01/03 |
| SD-02 | modify | docs/specs/auto-lang/ui/design/native-acceptance-channel.md | Reader原生验收/fixture改外部owner及可运行入口 | 框架合同保留，应用验收独立 | AC-01/03 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | 当前驱动路径/owner承接 | 消除错误导航 | AC-03 |

## 6. 测试设计

以下未执行。仅迁移触面，不跑ACE、全语言/全UI或Reader全部产品套件。若真实窗口验证适用，执行前读取autoui-verifier技能并用现有通道；单实例有界验证，不并发窗口抢占。

| ID | 不变量与入口 | 可观察结果/边界 | owner_stage |
|---|---|---|---|
| V-01 | 迁移来源文件/生成器输出hash、case及断言清单对账 | 仅路径/导入/CLI必要变化；数据、样本、oracle、退出码守恒 | work |
| V-02 | 两新驱动--help；非repo cwd导入/加载；Reader本地fixture生成/manifest hash校验 | 不访问auto-lang源码或技能目录；无旧路径回退 | work |
| V-03 | Reader新导入驱动原完整用例；显式auto-bin/reader-dir与隔离目录 | TXT/MD10MiB+冷重启/坏编码/低预算拒写/双实例原case与原assert全通过；退出码/结果同时核 | work |
| V-04 | Reader新原生驱动原默认required集合；显式v12_injected_fail反控单独运行 | 默认required全通过；故意失败非零；真实几何/保存恢复/锚点/故障判定与reap保持 | work |
| V-05 | 源退役活跃调用者/文档检查与独立删除审查 | 原目录/驱动无重复实现；目标唯一可运行；历史路径仅历史说明保留 | review |

列实际发现/执行/条件排除，原首次红、环境缺口及重放不抹去；准备/运行/清理/总时长分列。环境不可用不算通过；确有框架产品缺陷保复现并回相应owner，不顺带重写产品。没有Rust生产改动，不要求cargo t全组。

## 7. 验收标准

- [ ] AC-01（work）：两fixture、生成器及两驱动在Reader唯一归属，来源/输入/oracle守恒；V-01/03/04。
- [ ] AC-02（work）：Reader入口可从任意cwd运行，无语言仓源码/技能目录隐式依赖；V-02/03/04。
- [ ] AC-03（work）：Reader README及语言当前Spec/活跃调用者迁移正确；历史记录保真；V-05。
- [ ] AC-04（review）：独立逐AC/删除与遗漏复审pass，无静默减覆盖、workaround或未处理本包格式/告警。
- [ ] AC-05（merge）：先Reader交付再语言退役，记录两仓提交与Spec回写，归档并guard清理两树；不自动更新组合pin或push。

## 8. 执行步骤

| ID | owner_stage/依赖 | 操作、实际路径及结果 | AC/验证 |
|---|---|---|---|
| T-01 | work/方案确认 | 核两仓指令/基面，建唯一兄弟树；来源hash/case/调用者/生成资产审计，冻结MCP实际方法集合与命令 | AC-01/02；V-01 |
| T-02 | work/T-01 | Reader新增tests/native与目标fixtures，迁两驱动并去外部技能导入，更新CLI/README | AC-01/02/03；V-01/02 |
| T-03 | work/T-02 | Reader树完整迁移触面真实验收及反控；记录首次红/退出状态/数据hash/端口reap | AC-01/02；V-03/04 |
| T-04 | work/T-03 | 语言树删除源两目录及两驱动，准备当前Spec迁移delta，核所有活跃调用者 | AC-03；V-05 |
| T-05 | review/T-04 | 冻结两树，同树独立上下文审查逐AC/删除与SD，需修回原树；不以实现自检代复审 | AC-04；V-01..05 |
| T-06 | merge/T-05 | 先Reader合入其main，再语言master退役+Spec沉淀；归档900、按最新5计划留存、逐树wt-guard clean后清理分支/组 | AC-05；记录两仓delivery及清理收据 |

下一执行入口：r1方案确认后T-01；不依赖899执行。创建源码树/构建/测试前核主仓同期变更，不能复制别人的WIP。无新增审查模型/代理授权。

## 9. 复审记录

2026-10-31 / stage:new / PLAN-900 r1 / outcome:pass（草案自查，非实现验收）/ next:展示确认后work。来源、目标、AC/任务/SD及去外部技能依赖已明确；未迁文件、未跑UI/HTTP或修改Reader。仅提交计划与.next-id。

## 10. 待澄清事项

用户已指定目标owner为auto-reader；此处不再询问归属选择。待确认上述具体跨仓迁移方案后实施，尤其包含两份驱动及其路径/MCP辅助依赖，而非只移动数据目录。
