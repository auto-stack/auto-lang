---
plan_id: PLAN-814
status: drafting
feature_name: 语言 SDK、UI 编译与运行包组整体分层
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-19
plan_revision: 1
current_step: 0
total_steps: 12
supersedes_spec_components: [docs/specs/auto-lang/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/auto-ui-dialect/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/language-sdk/project.md, docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [language-sdk, auto-ui-compiler, auto-ui-runtime, auto-lang/compat, auto-vm]
depends_on: [PLAN-807, PLAN-808, PLAN-810, PLAN-811]
---

# [PLAN-814] 语言 SDK、UI 编译与运行包组整体分层

## 0. 变更摘要

完成R04的语言/编译/运行边界：提供不依赖UI实现或Man的语言SDK，把UI编译生成与UI运行/renderer形成独立可消费包组；默认组合和旧`auto_lang::*`兼容明确归上层。迁移实现、公开路径、资源、测试选择、真正消费者及发行闭包一起交付，不止增加几个空crate。

不搬Man的UI工程工具/资源（815），不接完整CLI工具协议（816），不搬Git仓/切旧默认分支，不在本期删除UI兼容AST/TokenKind（R10）。纯SDK不等于原名auto-lang同一包既重导出完整UI又保持纯Core。

## 1. 目标

- 新纯SDK可独立完成Core parse→compile→VM、现有trans与基础服务消费；normal/build/dev闭包无UI文法实现、UI compiler/runtime/renderer、Man实现。
- UI分析/模型适配、AURA/a2ui、四生成器、设计token/registry等及UI runtime/interpreter/renderer/宿主adapter有单一owner和无环依赖；相关轻数据按真实消费者归属，不新建common巨包。
- 默认UI入口及旧public API/feature/类型身份、动态装配/全局与实例行为兼容；旧兼容实现处于上层，外部消费者可选择真正SDK而无需UI checkout。
- i18n/直接frame/VM到UI残边及隐式默认装配具名收口；原有测试族不会因跨包迁移从别名门中消失。

## 2. 架构方案

候选布局：新`crates/auto-lang-core`为纯SDK；新`auto-ui-compiler`与`auto-ui-runtime`分别拥有UI编译/生成和运行/renderer；旧`auto-lang`包保上层兼容/默认组合，依赖这些包。包名可在T-01按真实碰撞/身份固定，不能以SDK反依赖上层兼容包实现旧入口。允许共享既有轻ast/lex/parser-api/text/terminal/media等，不复制语言或VM。

UI compiler可使用SDK，runtime可使用SDK与必要UI编译公开面；compiler生成的runtime路径是代码输出合同，不可再用normal/dev反向依赖runtime形成环。跨层集成测试放上层/独立fixture。默认Parser/AutoVM等旧构造面、pub字段/方法/trait和回调类型的兼容必须有整链编译probe；如需薄包装，只有一份状态，不能靠Deref名义下藏双Parser/VM或破坏旧签名。具体布局T-01冻结，不假设改几行re-export就能满足所有inherent方法。

## 3. 技术栈

现有Rust包、同源AST/Value、CompilerSession、Core语义/VM、轻宿主capability与显式factory/装配；UI后端features按现行支持面迁移，不额外退役后端。现有Schema/stdlib/registry/资产与build输入随owner迁移。

## 4. 需求分析与背景调查

授权：用户要求建立下一批较大计划并明确合入前置；本轮仅auto-lang仓内规划，不实施/创建agent或工作树、不写相邻仓/远端。来源：[总路线](reports/v06-extraction-execution-roadmap.md)、[批次调度](reports/sdk-tools-parallel-order-814-816.md)、[输入hash](attachments/sdk-tools-batch-20261019/inputs.json)。

权威输入：[语言现状](../specs/auto-lang/project.md)、[frontend](../specs/auto-lang/frontend/overview.md)、[VM](../specs/auto-lang/vm/overview.md)、[UI](../specs/auto-lang/ui/overview.md)、[方言](../specs/auto-ui-dialect/project.md)、[交付](../specs/repository-boundaries/generated-ui-distribution.md)。808已实际合入5833e155e并归档10/10；810/811本轮起读为execution_done、随后回到executing修复，均未合入，handoff仅作预期接口观察，不能当规范或被生产依赖。811还登记实机环境pending，原owner收口，本Plan不代审；当前header以本批输入快照为准。

实际写面：`crates/auto-lang/{Cargo.toml,build.rs,src/lib.rs}`（build输入以T-01存在性/引用核实）、`src/{parser,parser_bridge,compile,config,a2r_std,back_proxy}.rs`、`src/{ui,ui_gen,aura,a2ui,design_tokens,vm,trans}/`、`crates/auto-vm`、根Cargo与`.cargo/config.toml`/`.config/nextest*.toml`。当前默认带ui-iced，`Parser`有pub scope/cur/infer_ctx等，lib全局STORE_EXTRA_FILES/USE_DIAGS及UI辅助也混在旧根；无环拆分必须处理身份/装配/生命周期，不能仅关default-features伪装。

### 开工门

- **现在可做T-01只读盘点/兼容方案/测试归属设计**，不以未合入接口编译新生产模块。
- **T-02起主搬包必须等810与811实际review+merge**；807/808已满足，809不是本Plan前置。T-01若需要代码probe，也等该门后在唯一树运行。
- 不cherry-pick或叠加810/811送审分支，不创建phase merge；等待期间可以完成本合同只读交付物，不记为整个实施已可完成。

## 5. 详细设计

1. **SDK与UI owner图**：逐模块列出源码、public项、normal/build/dev、feature、生成字符串/宏/include/build input依赖；解析/推断/编译/VM/非UI trans留同份Core，UI生成/运行移包；依赖语法名不等于UI实现依赖。现有UI兼容AST可在轻auto-ast暂留，明确R10退出，不把UI私有TypeStore/registry推回Core。
2. **默认组合与旧API**：旧auto-lang可为上层兼容包；SDK新名字公开且实测独立。对`Parser::with_session`、AutoVM/NativeInterface构造、外部Dialect、native callback、pub字段/trait/inherent与模块路径构造编译pin；旧UI场景仍默认可用，SDK的Core场景不加载UI实现，SDK UI解析须显式获得UI方言组合。不能靠linker inventory偶然拉入初始化或Core硬编码UI factory。若无法兼容，T-01具体提出重规划，不自行砍API。
3. **UI compiler**：现有ui_gen四生成器/registry、AURA/a2ui与UI相关分析按闭合图迁移；通用trans/shared helper经SDK公开最小能力，不暴露Core全部私有结构。生成canonical runtime/SDK import由一份路径策略产生；旧路径门面保兼容。允许必要import/header基线变更须显式映射新旧路径，不能以更新golden掩盖行为差异。
4. **UI runtime**：ui目录runtime、renderer、editor/terminal/widget、desktop/MCP/宿主adapters及其状态/资源迁有归属的包；VM/SDK只保轻能力与非UI机制。i18n lookup/装载、a2r frame直连、config/UI依赖和其他反向边逐项验清；未消除边必须有本Plan内任务，不能在最终声明“纯Core”时留豁免实现。
5. **状态/并发**：registry/OnceLock/thread_local/global runtime/VM Rc、provider默认/实例覆盖/owner history、模板/语法session/cache不可跨新包产生第二真源；线程亲和、锁外回调、store额外文件drain、use诊断、终端/媒体生命周期/快照共享保持。移动字节不等于状态单源证据。
6. **消费者与测试**：旧路径消费者在兼容层继续可编译；至少一个真实非UI消费者（首选auto-vm，若其实际依赖需调整则T-01具名替代）直接切SDK。UI compiler/runtime及后续工具必需的语言adapter（如auto-gen可执行模板层）也要沿实际闭包切SDK，不能经旧auto-lang兼容包把UI实现重新拉回。auto-lsp等UI补全可暂用上层具名adapter，不谎称其整体纯Core。迁出的lib/test/integration/doc/schema测试保单份来源；cargo t/tv/tu/tt/tb实际选择面映射覆盖新包，UI tu保持合适单进程registry复用，不因包名变化丢测试。
7. **发行**：777精确域边/forbidden、792受管source与manifest/资源/producer/lock/重定位更新至真实包图；新SDK独立fresh/locked无UI checkout，UI实例工程可用新包，不靠父仓隐式path。SDK不含UI实现的证明覆盖normal/build/dev，UI部署profile仍保原功能而非删除默认重后端。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/language-sdk/project.md | 原巨型包→纯SDK职责/API/feature/显式组合与正常闭包 | 基础可独立消费 | AC-01, AC-02 |
| SD-02 | add | docs/specs/auto-ui-compiler/project.md | 原UI分析/生成→独立compiler包、公开接口/资源 | UI编译owner | AC-03 |
| SD-03 | add | docs/specs/auto-ui-runtime/project.md | 原UI运行/renderer→独立runtime与宿主/状态归属 | 运行边界 | AC-03, AC-04 |
| SD-04 | modify | docs/specs/auto-lang/project.md | old包标为上层兼容/组合，实际SDK位置明确 | 避免纯Core虚报 | AC-01, AC-02 |
| SD-05 | modify | docs/specs/auto-lang/frontend/overview.md | Parser默认/显式UI组合与旧API承载 | 解析身份/装配 | AC-02 |
| SD-06 | modify | docs/specs/auto-lang/vm/overview.md | 无UI实现的VM能力/默认组合与余边清零 | 真正解耦 | AC-01, AC-04 |
| SD-07 | modify | docs/specs/auto-lang/ui/overview.md | UI包组/旧路径/资源/消费者事实 | 完整交接 | AC-03..05 |
| SD-08 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 真实新包图/测试与资源输入进入发行profile | 迁包可分发 | AC-05 |

## 6. 测试设计

T-01冻结旧public/API与消费者/支持面、测试族选择清单；T-08建立新增`test/fixtures/modularity/language-sdk-consumer/`、`ui-package-consumer/`及上层`plan814_package_boundaries`定向测试。以下均owner_stage=work，当前未执行。

| Case | AC / invariant | 真实入口/设置 | 区分性oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 纯SDK/单源 | 独立manifest→Core parse/compile/VM/trans；无UI checkout、真实auto-vm消费 | normal/build/dev无UI compiler/runtime/grammar实现/Man；单份AST/Value/VM/源码，不用空包或重导出旧巨包骗绿 |
| V-02 | AC-02 兼容/装配 | 旧Parser pub字段/方法/ExternalDialect/native callback、UI默认/import、Core及显式UI组合，features开关/两VM覆盖 | 旧签名consumer编译与AST/执行结果保持；SDK无UI能构建，旧UI默认可用且不靠inventory偶然性；无双Parser/VM可变状态 |
| V-03 | AC-03 真UI闭环 | 同源示例经四生成器与VM/Iced；新UI compiler/runtime及兼容路径 | 产物/行为相同，import变化单独解释；生成工程真实编译运行。不能只字符串检查或修改golden消掉漂移 |
| V-04 | AC-04 余边与寿命 | i18n/帧值/a2r/registry/provider替换，UI装载/退出、终端/媒体/编辑器/跨thread_local调用 | SDK不引用UI实现；原缺席/覆盖/owner、线程亲和和退出保持，诊断/额外文件不因分包丢失或双份 |
| V-05 | AC-05 测试覆盖/发行 | 迁前后nextest list/家族选择映射、tu单进程、独立fresh/locked/792重定位、缺source/反向边负控 | 原测试族按owner仍被同档选中，无以总数碰巧相等代替映射；实际source/资源齐全、负控红 |

work只定向check/新包测试/相关反例及新consumer；T-01在合入门后冻结实际包名和命令，例如`cargo check -p auto-lang-core -p auto-ui-compiler -p auto-ui-runtime -p auto-lang`，未来包创建前不把命令当执行证据。777用`python scripts/modularity-audit.py --mode strict --no-cargo-check`；792沿现有generated-ui-consumer driver。真实跨端按autoui-verifier现有设施选有代表性的实际触面，环境缺失pending。

独立final review同树独立上下文验AC/SD与原始反例；未指定agent/model不创建会话。merge最终冻结提交按迁移后的真实门入口统一裸cargo t+tv+tu+tt，若文档/Schema生成实际迁移触面则补docs_gen及tb必要面；开发/复审不跑整档。tf主检出到期单实例。复用绑定源码/依赖/配置/输入，合流变化复审/重放影响面。本轮仅文档Category A，无Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 真SDK可独立消费且normal/build/dev无UI实现/Man，真实非UI消费者落地、语言/VM单源；V-01 |
| AC-02 | work | 旧公开API/字段/trait与默认UI保持，纯SDK显式组合可用，无隐式链接装配/双状态；V-02 |
| AC-03 | work | UI编译/生成/运行职责完整迁移、四生成器与真VM/Iced/生成工程消费保持；V-03 |
| AC-04 | work | i18n/frame等余边清偿，provider/全局/TLS/资源与退出保持；V-04 |
| AC-05 | work | 测试族迁移选择面、777/792与fresh/locked/资源负控通过，SD/来源/剩余兼容与净迁移清单完整；V-05 |
| AC-06 | review | 同树独立审查绑定修订，遗漏/规范/警告/格式/debug检查通过，merge门pending明确 |
| AC-07 | merge | 冻结门、canonical Specs/ledger/归档/接口与资源交接、guard clean后清理有实际收据 |

## 8. 执行步骤

12项；T-01可立即只读开展，其代码probe及T-02..09必须在810/811合入后。

| 状态 | ID | owner_stage | 依赖/启动门 | 文件/交付、验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 现在只读；probe等810/811 | 新attachments/814/contract-baseline.md：所有者DAG/旧API构造/状态/测试族/资源/真实消费者清单与候选布局；前置合入后整链编译probe冻结SDK/默认UI兼容方案，不可行先重规划 | AC-01..05 |
| [ ] | T-02 | work | T-01决策+810/811已合入 | 建真实SDK包/上层兼容骨架，单份Core实现与最小公开出口迁移，V-01/02编译pin | AC-01,02 |
| [ ] | T-03 | work | T-02 | UI compiler及aura/a2ui/design_tokens/registry按闭合图迁移、无环公开面/生成路径策略；定向V-03 | AC-03 |
| [ ] | T-04 | work | T-03 | UI runtime/renderer/宿主adapter与资源迁移，VM依轻能力；保持真实编辑器/终端/媒体与跨端 | AC-03,04 |
| [ ] | T-05 | work | T-04 | 默认Parser/VM/新SDK显式UI装配、旧auto-lang字段/方法/feature/旧路径兼容；真实import及外部consumer V-02 | AC-02 |
| [ ] | T-06 | work | T-05 | i18n/a2r frame/config等残边、全局/TLS/registry单源和退出验证；必要非UI消费者切SDK，V-01/04 | AC-01,04 |
| [ ] | T-07 | work | T-06 | 测试源码/别名/nextest选择面按owner迁移，全部旧族逐项映射，移除旧实现；V-05选择清单 | AC-05 |
| [ ] | T-08 | work | T-07 | 新consumer/上层定向矩阵V-01..05、777/792新闭包、fresh/locked/重定位与负控 | AC-01..05 |
| [ ] | T-09 | work | T-08 | SD-01..08备稿、真实API/包源/资源/测试归属及compat退出清单，新增attachments/814/handoff.md绑定SHA/结果，提交送审 | AC-05 |
| [ ] | T-10 | review | T-09 | 同树独立完整DAG/公开兼容/生产/寿命/测试选择与SD审查，merge门pending | AC-06 |
| [ ] | T-11 | merge | T-10 pass | master并集/变化复审，最终冻结提交统一§6门，失败修复后重审重冻 | AC-07 |
| [ ] | T-12 | merge | T-11 | 规范/ledger沉淀、归档、已合入SDK/UI出口交815，guard clean后清理、终态计数 | AC-07 |

唯一树`D:/autostack/.wt/lang-814/auto-lang`，分支`codex/plan-814-dev`；work/review/merge同树顺序使用，记录实际base/兄弟依赖，不在master改代码。无phase merge，不提前借810/811分支；与815/816写面及合入见本批报告。

## 9. 复审记录

2026-10-19 / stage: new / revision: 1 / outcome: pass（合同预检，非实现/独立验收pass）/ next: work T-01只读；代码probe和T-02起等待810/811实际合入 / changed: T-01..12、AC-01..07、SD-01..08。未执行未来Cargo/fixture、未创建工作树/agent或搬仓。

## 10. 待澄清事项

没有需用户重复授权的当前问题；前置合入是客观门，不能超时推定放行。T-01有界证明旧公开类型/默认装配及SDK布局可行；若必须删旧API/改变支持/复制状态或新增兼容退役范围，具体needs_replan，不自行降低AC。包组仍留语言仓，Git承接后续R08。
