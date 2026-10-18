---
plan_id: PLAN-806
status: execution_done
feature_name: UI msg 与 model 声明文法正式外置
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-18
plan_revision: 1
current_step: 6
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/frontend/design/dialect-system.md, docs/specs/auto-lang/ui/design/scenario-dialect.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/auto-ui-dialect/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-ui-dialect, auto-lang/frontend]
depends_on: [PLAN-799, PLAN-801]
---

# [PLAN-806] UI msg 与 model 声明文法正式外置

## 0. 变更摘要

799/801合入后，新建只依赖轻AST/词法/诊断/解析接口的auto-ui-dialect，将现有完整msg/model声明算法及字段注解迁出Parser；顶层、widget/component/store中的所有内部调用真实委托。默认UI生产路径使用新实现，不只追加外部fixture。

## 1. 目标

msg/model完整现有文法有独立实现包、无auto-lang/Parser/UI renderer反向依赖，原AST/诊断/场景及消费行为保持。删除Parser里的原算法，旧公开parse_msg_decl/parse_model_block仅兼容委托。

非目标：完整widget/view/on/actions/style/computed文法、自动向所有import传播任意外部注册、分离UiAst/lowering、关闭既有无UI-feature场景解析能力或搬整个auto-ui仓。默认组合宿主仍可依赖轻UI文法包，不能宣称纯Core Parser/整仓无UI依赖已经完成。

## 2. 架构方案

auto-ui-dialect → 801实际auto-parser-api + 799实际auto-ast + 既有轻词法/诊断；auto-lang组合层提供端口并装配默认UI声明方言。新crate normal/build/dev均不得依赖auto-lang或复制Parser/模型/lexer。核心expr/type沿端口委托，不复制Pratt/类型文法。

顶层经同一builtin派发段，保Override/builtin/Append次序和旧场景/fn/on护栏；结构文法内部通过同一msg/model解析函数，不能保留一条隐蔽旧实现。旧入口保持Rust调用形状和AST类型身份。

## 3. 技术栈

Rust轻方言crate、801安全ParseContext/checkpoint机制、真实Parser/类型/VM/UI生成消费对照及独立方言构建consumer。新API以801最终交付为准，不预写不存在的方法。

## 4. 需求分析与背景调查

用户2026-10-17授权继续建立后续合同。本轮只计划，不执行Rust/建agent或树、不改799/801/802/803状态或验收。取材9932962bc157be13bcaa0520011c7c78082cb2fe及[指纹](attachments/extraction-followup-20261017/inputs.json)。799仍在复审/修复收口、未合入；801 drafting，主检出无auto-ast/auto-parser-api。**两项合入是work硬前置**，不能从worktree复制或假定未来API。

Parser现有parse_msg_decl_inner支持旧msg Name（名字读弃）、quoted variants、多个payload及named/positional payload；named探测当前直用lexer lookahead。parse_model_block_inner支持var、类型/初值和primary/Consume/Provide/NavParam注解、未知注解特定诊断；三个inner方法在widget/component/store等结构路径被直连。只改UiDialect顶层分支会漏掉生产调用。

778/801 model pilot允许显式外部子集的bare field，但默认内建model仍需类型或初值。正式默认切换不能顺手把pilot扩展变为默认文法；未知注解/恢复/字段flags必须保留。低层UI兼容AST仍由799共享，不能以本轮外置删节点。

## 5. 详细设计

T01在799/801交付上完成有界整链可行性：msg命名payload的安全lookahead、完整model注解/expr/type、顶层与结构内调用、错误位置和None/Err恢复。冻结所有callsite/可见性/port差额；可用checkpoint+advance/restore等价实现lookahead，不公开lexer/Parser/TypeStore。如果确需新增通用端口，必须在本计划具名说明/测试其Core含义和兼容性；不能加parse_model/parse_msg回调把真正算法留Parser伪装迁出。无法等价则needs_replan，保留硬目标。

搬parse_msg_decl_inner、parse_model_block_inner、parse_model_field_annotations完整算法至新crate；保留旧msg Name丢弃、quoted/name/payload_names、字段mutable/is_primary/decorators、错误文本/阶段/位置、现有空行/逗号/宽容拼写行为。不要趁抽取收紧/扩展文法或修复历史怪癖。所有结构路径与顶层用同一实现，原公开方法只薄委托。

生产默认装配不冒充用户Override注册；外部Override/Append及with_session/递归保持，Core同名变量/fn/on体对象字段不被抢声明。UI场景在已有无renderer/no-ui feature组合仍可解析，不能偷加UI feature才能解析的新限制。新crate保留纯文法依赖，默认组合位置及仍有的core→轻UI文法边单独登记，后续Parser/组合宿主分层再清理。

pilot不保第二份model算法：801独立model consumer改为复用新model解析实现的显式扩展策略，或等价单源入口；裸字段支持只在显式pilot策略启用，默认false。T01据实际交付确定表达方式，原pilot对照保留，不能既删bare-field正控又宣称兼容。

生产import中新建Parser使用同一默认UI声明实现，并验证一个真实导入文件消费；不承诺本轮让所有自定义外部注册跨import自动继承。完整widget/view/on/actions/style等剩余方法/装配边列清，迁位与净删分列。受影响792包/来源/资源按已交付入口补清单并验证，不能沿用漏新crate的固定闭包。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ui-dialect/project.md | 无独立生产文法包→完整msg/model及端口/依赖边界 | 由pilot进入真实文法迁移 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/frontend/overview.md | 默认内建算法→组合装配/兼容委托及真实结构/import调用 | 不漏生产内部路径 | AC-02/03/04 |
| SD-03 | modify | docs/specs/auto-lang/frontend/design/dialect-system.md | 声明分派与端口权限/恢复规约、默认与pilot策略区别 | 不改变Core和外部优先级 | AC-02/03 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/scenario-dialect.md | msg/model正式外置，其他UI grammar仍上层 | 不夸大全UI文法已拆出 | AC-03/04 |
| SD-05 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 交付闭包12 crate→13（+auto-ui-dialect 及其闭包内依赖边说明） | 权威交付规范不漏新包（F806-R2-02） | AC-04 |

## 6. 测试设计

work→同树独立final review→merge，无额外agent/模型。拟唯一树D:/autostack/.wt/lang-806/auto-lang，branch codex/plan-806-dev。fixture由T02/T05建立；当前仅规划，不是已执行证据。799/801实际SHA/API/Spec先冻结；单计划一棵树，不从别线拼临时实现。

| Case / owner | AC / 不变量 | 输入/真实入口 | oracle与边界 |
|---|---|---|---|
| V01 / work | 01 独立闭包 | 新test/fixtures/modularity/ui-declarations-consumer，轻包编译/单测及metadata；含语言driver分开放上层 | normal/build/dev无auto-lang/Parser/VM/renderer，真算法而非转调Parser函数；AST同一类型 |
| V02 / work | 02 msg全族 | canonical/legacy-name/quoted variant、unit/多payload/named与positional、泛型类型；坏payload/缺闭括号 | 新旧MsgDecl/AST/诊断位置相同，名字/quoted/payload_names不丢；lookahead不吞token，后续语句可按旧规则恢复 |
| V03 / work | 02 model全族 | var/类型/初值、四注解及组合、未知注解、缺闭块；默认bare field反控+显式pilot正控 | 字段flags/类型/init/decorator及具体错误同旧臂；默认不意外接收bare，显式pilot仍接收；两者共用一份字段算法 |
| V04 / work | 03 场景/所有生产调用 | 顶层、widget、component兼容结构、store及真实import文件；Core同名赋值/view参数、fn/on体msg字段、混排注册/递归 | 每个内部调用确实到新算法，默认UI无需显式注册；Core对照与优先级/五旗标/None/Err规则保持；不以顶层fixture代全覆盖 |
| V05 / work | 03 消费保持 | 上述AST真实类型/VM handler、quoted/name payload的Vue/Rust代表生成及model注解消费 | 关键输出/运行结果等于冻结旧臂/既有golden，不更新金样掩盖语义变化；不要求重写其余生成器 |
| V06 / work | 04 单源/交付 | 源码方法/调用清单、旧算法删除、777精确规则、792真实交付driver | 原Parser只兼容委托，pilot无平行实现；新包独立source唯一；受影响fresh/locked/重定位及缺包/坏hash等控制通过 |

work：`cargo check -p auto-ui-dialect`、`cargo check -p auto-lang`，新consumer及801原consumer manifest test，定向场景/结构/import/错误恢复与消费测试（T05记录实际filter/选择数）。典型平链/嵌套expr/type及坏输入沿已有支持域验证，不新增任意固定语法上限或承诺无限资源。最终review后merge冻结裸cargo t+tv；实际修改ui_gen/trans分别追加tu/tt，work/review不跑整档。tf主检出到期单实例，本轮不跑Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 完整msg/model文法在轻包独立实现，无Parser反调/模型复制，V01通过 |
| AC-02 | work | 全语法/注解/错误/lookahead及默认/pilot区别保持，V02/03通过 |
| AC-03 | work | 顶层/结构/import生产真委托，场景/注册/递归/消费保持，V04/05通过 |
| AC-04 | work | 旧算法删除、未迁UI族/净删/792/SD收据齐全，V06通过 |
| AC-05 | review | 同树独立逐项AC/SD及遗漏/恢复审查通过，绑定799/801/实现SHA，merge门pending |
| AC-06 | merge | 冻结日常/语料及实际触面门、Spec/ledger/归档/guard清理完成 |

## 8. 执行步骤

2026-10-18 R2 当前状态：仅 T-03/T-04 验证完成；T-01/T-02/T-05/T-06 因 F806-R2-01～03 重开。各行附属的 2026-10-17 work 完成说明保留为历史声明，不代表本轮验收通过；具体反例和独立验证见 [R2复审](attachments/806/review-r2.md)。

| 状态 | ID | owner_stage | 依赖 | 路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 799/801审查合入、合同确认 | parser.rs三算法/all callsites、801最终端口及pilot，整链可行性/差额/旧臂/默认策略冻结；R2：前瞻等价性未成立 | 01/02/03 |
| [✅ 2026-10-18 R2 修复, code e2fc6ebf3] 不变量修订=前瞻必须零副作用（不跳注释/不收 docs/不记词法错误/游标不动/不扩文法）；受影响路径=named payload 探测×{注释分隔,doc 相邻,词法错误窗口,位置形态}；修复=端口具名通用能力 `ParseContext::peek_next_kind`（auto-lang `port_peek_next_kind` 原语=旧 lexer.next+push_token 逐字同构）替换 checkpoint+advance/restore；永久用例=plan806 r2_* 7 例（oracle=R2 冻结新旧探针日志）；结果=14/14 绿；剩余缺口=无。 |
| [✅ 2026-10-17 work] 801 已合入归档（delivery 786c49964，auto-parser-api 在主检出）；决策 D1–D7 冻结（crate/闭包、checkpoint+advance/restore lookahead 等价+微残差登记、expect_ident 逐字复刻、ModelGrammar 单份策略、薄委托+守卫不变、syntax_error 同构、777/792 登记面），§9 work T-01 记录+master 侧只读预检 attachments/806/t01-readonly-prep.md。 |
| [x] | T-02 | work | T01 | 新crates/auto-ui-dialect及完整msg/model/注解算法，安全lookahead，轻consumer与check/V01；R2：修复F806-R2-01 | 01/02 |
| [✅ 2026-10-18 R2 修复, code e2fc6ebf3] declarations.rs 前瞻改 peek_next_kind 形态；轻 consumer 增 peek 语义测试 10/10+verify-deps PASS；cargo check×2 零新警告。 |
| [✅ 2026-10-17 work, code 970960b4a] 新 crate 三文件（declarations.rs 逐字迁出 master 5d8834bcc parser.rs 14917-15132 基线）；ui-declarations-consumer 轻消费方 9/9 绿 + verify-deps PASS（闭包={lex,parser-api,ui-dialect} 无 auto-lang）；cargo check -p auto-ui-dialect 零警告。 |
| [x] | T-03 | work | T02 | Parser公开/inner薄委托与UiDialect默认装配、全部widget/component/store及默认import路径接线，V04 | 02/03 |
| [✅ 2026-10-17 work, code 557e0a176] inner 两方法一行建桥委托、parse_model_field_annotations 删除（parser.rs −213 行）；pub 签名/Stmt 包装、UiDialect 分支与结构守卫、with_session 默认装配不变；check -p auto-lang 0 error 零新增警告（411=801 基线）；冻结旧臂内联 12/12 + 滤面 45/45；V04 plan806 7/7（含真实被导入文件 treeview.at）。 |
| [x] | T-04 | work | T03 | 801 pilot复用单份model、默认bare反控及显式bare正控；删除原算法，V02/03/05 | 02/03 |
| [✅ 2026-10-17 work, code 8093301b8] dialect-api-consumer 删第二份字段算法（−65 行）改调 parse_model_block_subset（NotClaimed→Ok(None) 协议保持）；轻测 3/3+闭包门 PASS；上层 driver 48/48（r2_bare_field_boundary 默认反控+pilot 正控 8/8 等）；默认文法零漂移。 |
| [x] | T-05 | work | T04 | V01–06全矩阵、777精确边/792source闭包与受影响交付控制，实际命令收据；R2：补前瞻副作用反例/三文件format | 01–04 |
| [✅ 2026-10-18 R2 修复, code e2fc6ebf3] 三反例+相邻对照收编 plan806（14/14）；三文件+触及文件 rustfmt --check 全过（parser.rs 无关重排已回退，仅+16 行原语）；重放 light 10/10+driver 48/48+内联 12/12+滤面 45/45。 |
| [✅ 2026-10-17 work, code 8a86fe323+504ca0e0d] 777 增 lang-core 域+基线重采样（增量恰 2 合法边，strict pass 0 违规）；792 DELIVERY_CRATES 12→13+delivery 单测 10/10+交付 driver 25/25 exit0（fresh 111s/locked 135s/十四态负控/恢复绿，收据入附件）；V05 tu 867/867 零金样更新；命令矩阵全收据 attachments/806/t05-receipt.md。 |
| [x] | T-06 | work | T05 | 未迁widget/view/on等清单、净删/组合宿主剩余边及SD/SHA收据，提交送审；R2：纠正等价/微残差说明，补交付权威Spec/impact | 04 |
| [✅ 2026-10-18 R2 修复, code e2fc6ebf3] SD-01 lookahead/怪癖适用边界校准（缺块闭括号+EOF=无界怪癖≠已覆盖错误出口；载荷缺 `)`=expect 确定性报错）；generated-ui-distribution.md 12→13+依赖边说明（SD-05 具名+frontmatter supersedes 增补）；t05-receipt 增 R2 更正节（微残差叙事作废+tu 整档时序越权注记）；新冻结 code=e2fc6ebf3 送 T-07 二轮复审。 |
| [✅ 2026-10-17 work, code af3748db7+f84f405f4+a20fcf6fe] SD-01 新 auto-ui-dialect/project.md+SD-02..04 三处备稿（worktree 待 merge 沉淀）；未迁清单/净删/微残差登记入 t05-receipt；代码链 970960b4a→a20fcf6fe 共 8 提交，送 T-07 同树独立复审，merge 门 pending。 |
| [ ] | T-07 | review | T06 | 同树独立完整语法/所有调用/恢复/消费/SD审查，merge门pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、沉淀/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-18 / stage: work / PLAN-806 revision: 1 / outcome: pass（R2 三 findings 全修，T-01/T-02/T-05/T-06 R2 要求重勾 6/8）/ worktree_path=D:/autostack/.wt/lang-806/auto-lang / branch=codex/plan-806-dev / base_commit=5d8834bcc / code_commit=e2fc6ebf3（修复在复审冻结基线 a20fcf6fe 之上单一提交；工作树 clean）/ dependency_revisions=auto-down c39ac7f9 不变 / finding_closure=F-806-R2-01：不变量=前瞻零副作用且不扩文法；受影响路径=named payload 探测×{注释分隔伪命名,doc 相邻重复收集,词法错误窗口翻倍,位置形态,comment-before-rparen,FStrStart}；修复=端口具名通用能力 ParseContext::peek_next_kind（Core 含义/幂等/零副作用已文档化+Core 语义永久测试）+auto-lang port_peek_next_kind 原语+declarations.rs 弃 checkpoint+advance/restore；永久用例=plan806 r2_* 7 例（三反例 oracle 逐字=R2 冻结新旧探针日志：Err@25 UnexpectedToken RParen/docs 单份/UnknownCharacter@14 单份）+真实 use-import 链收编（review-r2-import.rs，含 Bad806 负控）；结果=plan806 14/14+light 10/10+driver 48/48+内联 12/12+滤面 45/45；剩余缺口=无（相邻 named/positional/comment/恢复对照保持）。F-806-R2-02：受影响面=权威交付规范+plan impact；修复=generated-ui-distribution.md 12→13+auto-ui-dialect 闭包内依赖边说明+SD-05 具名+frontmatter supersedes 增补；剩余缺口=无。F-806-R2-03：修复=三文件+全部触及文件 rustfmt --check 过（parser.rs 无关重排回退，仅+16 行原语）；剩余缺口=无。 / evidence=attachments/806/t05-receipt.md R2 修复轮更正节（含 tu 整档时序越权注记与作废的原微残差叙事）/ blockers=无（merge 门保持 pending：冻结裸 t+tv；ui_gen/trans 源未改不追加 tu/tt；tf 主检出到期判定）/ next_responsible_role=review（T-07 二轮 final review，审查基线 e2fc6ebf3）


2026-10-18 / stage: review / review_scope: final / reviewer_identity: 当前独立 Codex 检查会话（未参与实施，无额外 agent；未暴露身份不臆造） / plan_id: PLAN-806 / plan_revision: 1 / outcome: needs_fix / worktree_path: D:/autostack/.wt/lang-806/auto-lang / branch: codex/plan-806-dev / reviewed_commit: a20fcf6fe9885102d0059c04c9ac1e0551a3e3dc / base_commit: 5d8834bcc9cb4bde66ca9b610259036b82642227 / dependency_revisions: 799/801已合入，兄弟auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / spec_inputs: [baseline](attachments/806/review-r2-baseline.json.txt)五源blob与实现/配置SHA256，[SD冻结patch](attachments/806/review-r2-spec-delta.patch)sha256=dcd77cffcd1d568c5c4b5b4471ff9018bea6955e3a775ba6c6bcf4d43164f820 / acceptance_results: AC01独立包/闭包pass、AC02 fail、AC03/04 partial、AC05 fail、AC06 merge-owned pending / findings: R1送审基线缺失已CLOSED；F806-R2-01(P2，原始前瞻改advance后扩大注释夹名/冒号文法并重复pending_docs/lexer错误)、F806-R2-02(P2，generated-ui-distribution权威闭包仍12且漏auto-ui-dialect，需补SD/impact)、F806-R2-03(P2，三新增测试文件scoped format未过) / evidence: [R2完整报告](attachments/806/review-r2.md)，轻测9/9、旧driver48/48、plan806 7/7、msg/model/dialect 45/45、定向生成4/4；独立生产use+VM handler状态1→2及导入注解错误注册；无UI库features=[]构建+解析通过；777 strict0违规；正式792 driver25/25 exit0（fresh123s/重定位locked113s） / pending_closeout_gates: 最终pass后冻结裸t+tv及实际触面档、规范/ledger/归档/guard；本轮无整档/发布/merge/清实施树 / next: 原树work修复三finding并提交，受影响复审。保留T03/T04，重开T01/T02/T05/T06，executing 2/8。R1仅就绪核查，本轮首次实际完整实施审查；不以历史绿总数替代原合同。

2026-10-17 / stage: work / PLAN-806 revision: 1 / outcome: pass（T-01..T-06 全部完成，work-owned AC-01..04 达成）/ worktree_path=D:/autostack/.wt/lang-806/auto-lang / branch=codex/plan-806-dev / base_commit=5d8834bcc（master 分叉点，含 T-01 簿记）/ code_commit=a20fcf6fe（链：970960b4a→557e0a176→8093301b8→8a86fe323→504ca0e0d→af3748db7→f84f405f4→a20fcf6fe，工作树 clean）/ dependency_revisions=组内兄弟 auto-down detached c39ac7f9（只读，a2r-actor-tests path 解析用）；driver scratch 9.8GB 已清（收据先归档） / task_ids=T-01..T-06 / evidence=[t05-receipt.md](attachments/806/t05-receipt.md) 命令矩阵：AC-01（V01 轻测 9/9+verify-deps PASS+check 零警告）；AC-02（冻结旧臂重放 12/12+48/48+45/45，默认/pilot 差异 r2 8/8，错误文本字面量钉住）；AC-03（V04 plan806 7/7 含真实被导入文件+V05 tu 867/867 零金样更新）；AC-04（单源审计三标记仅存新 crate、parser.rs −213/pilot −65、777 strict pass 增量恰 2 边、792 driver 25/25 exit0+delivery 单测 10/10、SD-01..04 备稿） / blockers=无（merge 门显式 pending：冻结裸 cargo t+cargo tv——parser.rs 触 VM/编译器面；ui_gen/trans 零源触面不追加 tu/tt；tf 主检出到期判定；docs_gen 不触发） / 已登记微残差=T-01 D2 lookahead 窗口 lexer Err 提前记录（时序差异，计划 §5 认可形态）+历史怪癖 msg{+EOF 无限形态保持不修 / next_responsible_role=review（T-07 同树独立完整复审，审查基线 a20fcf6fe）

2026-10-17 / stage: work / PLAN-806 revision: 1 / outcome: pass（T-01 完成，前置解除）/ worktree=待建 D:/autostack/.wt/lang-806/auto-lang（branch codex/plan-806-dev，§6 冻结名）/ 前置核验=799 已合入（auto-ast 在位）；**801 已合入归档**（archive/801 status: archived 终态 8/8；delivery 786c49964 冻结门裸 t 5264/5265[唯一红 plan730 时序 flake solo 绿]+tv 163/163；worktree 已清）——主检出 crates/auto-parser-api 在位 / 决策 D1–D7= D1 新 crates/auto-ui-dialect 唯一依赖 auto-parser-api（闭包 ast/lex/diagnostics+val 轻层无 auto-lang；fixture ui-declarations-consumer 镜像 dialect-api-consumer 独立 workspace 模式+verify-deps）。D2 msg named-payload 探测以 checkpoint+advance/restore 等价替换 lexer.next/push_token 直读（探测点 pushback 缓冲必空；advance=Parser::next→lexer 同一前沿；restore 恢复 lexer 全态+cur/prev+五旗标；登记微残差=lexer Err 时旧码静默吞 vs advance 提前记录同内容错误，仅时序/error_limit 提前，§5 认可形态）。D3 关键字消费不用 ctx.expect_keyword（错误文本 "expected 'x'"+Ident kind 检查 ≠ expect_ident "Expected 'x'"+纯文本比较）——新 crate 逐字复刻 expect_ident。D4 单份 model=ModelGrammar::{Builtin,ExternalSubset} 一份块循环+字段算法按策略开关三差异（注解 Builtin 解析/Subset 字段首遇 Hash→NotClaimed；名字 Subset 需 Ident 否则 Err "model field name expected identifier, got '{}'"；裸字段 Subset 终止符 Asn/Newline/RBrace/EOF→Unknown，Builtin 仅 Asn→Unknown 否则必 parse_type；EOF 循环守卫仅 Subset）；dialect-api-consumer 删第二份 parse_model_subset 改调子集入口。D5 Parser pub 两方法保签名+Stmt 包装，inner 三方法一行建桥委托，注解方法删除；UiDialect/widget(13259,13262)/store(14532,14535)/component(16002,16006) 守卫不变（cur.text 匹配，master 当前行号）；import 经 with_session/build_dialects 默认装配不变。D6 未知注解错误经 ctx.syntax_error（桥面=SyntaxError::Generic 同文消息+pos_to_span(cur.pos) 逐字同构）；expect 族经桥面→Parser::expect 同实现。D7 777 module-boundaries 增 auto-ui-dialect=lang-core+基线重采样；792 DELIVERY_CRATES（auto-man/src/rust_ui.rs:3988 现 [&str;12]）12→13 / 冻结策略=V02/V03 期望字面量测试先旧臂跑绿（错字面量以旧臂实跑修正）再换新臂重放 / blockers=无 / next=T-02 建树实施

2026-10-17 / stage: review / review_scope: final / reviewer_identity: 当前独立 Codex 检查会话（未参与实施，无额外 agent；身份未暴露部分不臆造） / plan_id: PLAN-806 / plan_revision: 1 / outcome: blocked（送审基线缺失） / worktree_path: 指定 D:/autostack/.wt/lang-806/auto-lang 不存在，worktree/ref 无806 / branch: 未找到 / reviewed_commit: 无 / base_commit: 无实施基线；仅核查 master 2b57be3a6838a63ba7d175d7693b7aaa65c2edc7 / dependency_revisions: 799/801 均已合入归档，801历史阻塞已解除 / spec_inputs: [R1报告](attachments/806/review-r1.md)所列四源blob / acceptance_results: AC01/04可见现状fail、AC02/03 partial未运行新实现验收、AC05送审就绪fail、AC06 merge-owned pending / findings: F806-R1-01(P1)，无实施树/提交/轻方言crate/consumer/完成收据，Parser三算法及pilot仍为旧实现 / pending_closeout_gates: final pass后冻结裸t+tv及实际触面档、规范/ledger/归档/guard；本轮均未执行 / evidence: [R1报告](attachments/806/review-r1.md)、[SD冻结副本](attachments/806/review-r1-spec-delta.md) / next: 定位原实施树及已提交代码并补work交接；若未实施则work执行原合同。executing 0/8，所有任务保持未勾选，无Rust修改/测试/merge。此为就绪核查，不能当完整语义审查或交付通过。

2026-10-17 / stage: work / PLAN-806 revision: 1 / outcome: blocked（前置半满足） / worktree_path=未建树（前置未满足；§6 拟 D:/autostack/.wt/lang-806/auto-lang，branch codex/plan-806-dev） / branch=无 / base_commit=master 80f40af641d087823ba40910d390b1224abc9632 / code_commit=无（零代码改动，仅本簿记+附件） / dependency_revisions=799 已合入归档（auto-ast 在主检出）；801 execution_done、code 49512993b 送 T-07 二轮复审、merge 门 pending、主检出无 auto-parser-api；本轮未读取/复制 801 worktree 未交付 API / task_ids=T-01（仅 master 侧只读预检，任务未完成） / evidence=[t01-readonly-prep.md](attachments/806/t01-readonly-prep.md)：三算法定位与行为冻结点（msg 旧名读弃/quoted/named-payload lexer 回推；model 注解/未知注解诊断原文与 span/默认无裸字段）、生产调用点全图（顶层 dialect/ui.rs:49-50 + widget 13098/13101 + store 14371/14374 + component 15841/15845 直连 inner + import 走 with_session 默认装配 build_dialects）、pilot 状态（master 无 parse_model_subset/ExternalModelDialect，778 fixture r2_bare_field_boundary 在位）、旧臂锚点起点 / blockers=801 T-07 二轮复审→T-08 合入（冻结裸 t+tv 等 merge 门）是唯一解阻动作；解阻后以主检出真实 auto-parser-api 完成 T-01 剩余（lookahead 等价/pilot 单源策略/callsite 差额冻结）并建树 / next_responsible_role=801 review（T-07）→ 801 merge（T-08）→ 806 work（T-01 执行）

2026-10-17 / stage: new / PLAN-806 revision: 1 / outcome: blocked / next: 799与801审查合入后，work以真实API执行T-01。此前仅只读准备，不复制未交付类型/接口；不等待802/803/804/805整体完成。[预检](attachments/extraction-followup-20261017/preflight.md)。

## 10. 待澄清事项

无待用户决定的问题；阻塞是799/801交付前置。T01有界确认lookahead和pilot单源策略，不能改默认文法/放宽AC。后续完整widget/view/on迁移须据本批实际端口与剩余图另立中等计划。
