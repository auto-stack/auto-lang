---
plan_id: PLAN-793
status: archived
feature_name: 前端诊断基础 crate 抽取与 AST 依赖切断
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lex/project.md]
new_spec_components: [docs/specs/auto-diagnostics/project.md]
touched_goals: [GOAL-003]
affects: [auto-lang/frontend, auto-diagnostics]
depends_on: [PLAN-789]
---

# [PLAN-793] 前端诊断基础 crate 抽取与 AST 依赖切断

## 0. 变更摘要

把 auto-lang/src/error.rs 的通用错误/警告及诊断投影迁入新 auto-diagnostics；原 error 模块成为兼容re-export。通过现有 Name=AutoStr 的真实类型关系切断 error→ast 边，为后续 AST/parser 外置提供底层错误边界。生产管线必须使用新单一定义。

## 1. 目标

独立消费者可使用诊断/attach_source/格式化而不编译Parser/VM/UI；旧error路径和错误类型身份、code/label/help/span/警告保持。非目标：改变诊断文案、语法支持或错误阶段；不搬AST/TypeStore、不扩ParserBridge、不宣称整个前端已无环。

## 2. 架构方案

auto-lang → auto-diagnostics → auto-lex/auto-val/诊断基建；新底层不依赖auto-lang/auto-man/UI。保留789的LexerError与LexError单一定义及From映射，不复制词法错误。Name目前只是AutoStr别名，可在底层使用同一类型，不引入AST。原error.rs保留薄门面；模块专有错误转换在其所有者侧保留合法trait impl，不把route等实现迁进基础包。

## 3. 技术栈

Rust新crate、既有miette/thiserror/auto-lex/auto-val；既有诊断投影测试与独立Cargo consumer；777边界规则。

## 4. 需求分析与背景调查

取材 auto-lang@`9c773601abfdb25b80c08d06922d0ad50edc653a`；[本批输入指纹](attachments/parallel-extraction-20261016/inputs.json) 记录现行 Specs 与实际源码 SHA256。overview仅作导航，以模块Spec和真实代码为准。789/790/791均归档8/8；792为独立构建发行线，当前无交付假设。
用户要求“最好能有几个并行的计划”，本轮按该授权创建可执行合同和编排文档；未授权在本轮实施、创建agent/工作树、发布或修改旧仓refs。确认合同后各自work；本计划不依赖792完成，正式搬仓仍须满足整条路线退出条件。
787/788在途实验线不设为全局前置；实施者必须按实际diff重核类型/manifest共享面。全部实现/测试在本计划唯一组树，主检出只写计划簿记。
已读error.rs：唯一crate内导入为ast::Name；ast.rs声明pub type Name=AutoStr。error聚合包含AutoError、SyntaxError、TypeError、NameError、RuntimeError、Warning、ComptimeError与attach_source/print_error/format_error。route/discovery.rs另有From<RouteDiscoveryError> for AutoError，T01须核对迁出后trait一致性，不能只检查error.rs单文件。
主写面为error.rs和新crate；不改792的rust_ui/ui_gen/交付资源算法、不改794的native/scroll、不改795的lock/target。共享workspace/auto-lang manifest和边界规则按短窗口串行，保持兼容类型后各线源码可并行。

## 5. 详细设计

T01冻结公开类型/重导出/From实现的真实消费清单和诊断投影旧臂，检查语法、类型、名称、运行时、comptime、警告、聚合多错误及source-attached各族，不把format输出文本当唯一oracle。选定new crate依赖不得引入AST/推断。
逐职责搬运，保持AutoError和各子类型同一身份，禁止通过额外包装导致downcast或?传播变化。保留词法Raw→Msg、结构化→Lexer及source attach行为，不能重新包装成失去诊断code的字符串。SourceSpan offset/len、NamedSource及错误排序保持。
保留public旧路径和MietteError/Result等既有重导出；清除旧实现，只留下薄重导出/必要模块专有转换。记录前端剩余parser/AST/type-store/infer/session/module_cache/UI方言边，下一阶段仍需分别解耦。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-diagnostics/project.md | 单一诊断来源、类型/投影和无AST依赖合同 | 底层可消费 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/frontend/overview.md | error改为薄门面、删除error→AST边，明确剩余环 | 不冒称全部外置 | AC-03/04 |
| SD-03 | modify | docs/specs/auto-lex/project.md | 窄同步（R1 F-04 增补）：From<LexError>/attach_source/多错误聚合归属改 auto-diagnostics，删除已切断的 Name 残留表述 | 落地后不留矛盾 current-state 规范 | AC-05 |

## 6. 测试设计

实施者未另指派，独立final review与merge分别交接，不新建或指定额外模型。下表V用例由work实现并执行，不是本轮通过证据。开发及needs_fix只做定向验收与必要cargo check；最终代码提交、final review通过后，由merge在冻结提交跑裸cargo t及实际触面档。tf仅主检出批量单实例，不随各计划并发；无Schema/生成器改动不跑docs_gen。

| Case / owner | AC | 真实入口/输入 | observable oracle / 边界 |
|---|---|---|---|
| V01 / work | 01 | 新test/fixtures/modularity/diagnostics-consumer，直接依赖auto-diagnostics | fresh manifest cargo test通过；normal/build闭包无auto-lang/Man/UI，非借rlib |
| V02 / work | 02 | 冻结旧提交各诊断族代表例：带/不带source、Unicode位置、Lex Raw/结构化、重复attach、多错误、warning | code/severity/help/labels/span/source/Display投影逐项相等；固定颜色/宽度后的format对照，不新造expected |
| V03 / work | 02/03 | 外部consumer同时通过auto_lang::error和新路径互传；真实parse/type/route错误、From及?传播 | 同一类型编译可互传；错误阶段和单调追加保持，坏输入不成功/吞错 |
| V04 / work | 03/04 | 生产Parser/VM/LSP定向诊断案例与777metadata/源码清单 | 单一实现、无AST反向边；remaining DAG收据有owner |

拟树 D:/autostack/.wt/lang-793/auto-lang，branch codex/plan-793-dev。开发定向新crate/consumer及诊断族；merge-owned裸t+tv，新crate测试；若实际触trans才加tt。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | auto-diagnostics独立构建，生产依赖无AST/VM/UI反向边，V01通过 |
| AC-02 | work | 原类型身份、诊断结构及阶段不变，V02/03通过 |
| AC-03 | work | 旧诊断实现删除、兼容出口保持，模块专有From无缺失，V03/04通过 |
| AC-04 | work | 真实消费清单/剩余前端DAG和净搬迁账可回放 |
| AC-05 | review | 独立逐项验收、反控、SD冻结且无合同内未完项 |
| AC-06 | merge | 冻结门禁、规范/ledger/归档、guard后清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 789归档、合同确认 | error/AST Name与全仓From/消费者闭包；冻结V02投影及source/聚合错误旧臂 | 01/02/04 | [✅ 已完成] 消费闭包+38例12族golden冻结收据 attachments/793/t01-consumer-closure.md（SHA256 e97b9450…；Name=AutoStr 身份核对通过，§10 分支不触发） |
| [x] | T-02 | work | T01 | 新crates/auto-diagnostics及精确依赖、单一错误/警告类型；cargo check新crate | 01/02 | [✅ 已完成] crate 建成（deps: auto-val/auto-lex/miette/thiserror，dev: serde_json）；error.rs 逐字迁入（唯三补丁：Name→AutoStr×2 + 头注）；workspace member+boundaries 域 lang-core+auto-lang 依赖登记；cargo check -p auto-diagnostics 通过零警告 |
| [x] | T-03 | work | T02 | 迁source/格式化/建议函数；error.rs薄重导出，保持各转换及旧路径 | 02/03 | [✅ R1 修复后闭合] error.rs 薄门面（**20 名**冻结公开面显式 re-export——R1 F-04 计数修正）；From<String>/&str/io/LexError 随 AutoError 迁入；V02 重放 38/38 逐字段相等且期望件已入库（R1 F-01）；门面 re-export 布局 rustfmt 清洁（R1 F-03，六文件同范围 --check 全清） |
| [x] | T-04 | work | T03 | 生产Parser/VM/工具定向消费、route转换与剩余DAG核对，cargo check -p auto-lang | 02/04 | [✅ R1 修复后闭合] check -p auto-lang 零错误（411 警告=基线持平）；外部消费者 auto-lsp/auto 编译过；lib 测试二进制全量编译+error_tests 4/4+error_spans 8/8；777 observe 域/边合法；strict violations=0（基线重锚 1bb5660b4） |
| [x] | T-05 | work | T04 | 新外部consumer及V01..04；依赖/边界检查，不宽豁免 | 01–04 | [✅ R1 修复后闭合] V01 5/5+verify-deps PASS；V03 **7/7**——R1 F-02 重写：real_type_error_keeps_stage（check_field_type 接线，UI 会话，Type/E0106 span(57,1)，控制组 parse OK）+ real_name_check_error_keeps_stage（check_symbol，Name/E0201）+ compile 用例重标注 parser 恢复（内层 SyntaxWithSource/E0007）；收据 §3/§4 阶段归属已纠正 |
| [x] | T-06 | work | T05 | 净删/搬迁、remaining边/并行合流收据、SD素材，提交冻结代码 | 01–04 | [✅ R1 修复后闭合] 逐字迁移 diff 仅 3 补丁；净搬迁账/剩余 DAG/合流面收据；SD 素材含 SD-03（auto-lex 窄同步，R1 F-04）+ 门面/实现路径修正；冻结提交链 827452a2a→1bb5660b4（基线重锚）→**a8faf1e31（R1 修复）**；golden 入库（.gitignore 否定规则，提交内字节 SHA256=e97b9450… 与冻结件逐字节相等已证） |
| [x] | T-07 | review | T06 | 独立consumer、诊断结构/负控/去重及SD复审 | 05 | [✅ R2 final pass] R1四项全关闭；V02 38例、V01 5/5、V03 7/7、strict/闭包/fmt/diff-check通过；SD01/02/03冻结；[R2报告](attachments/793/review-r2.md)；merge-owned门禁及收尾pending |
| [x] | T-08 | merge | final pass | 裸t+tv及实际触面门，规范/ledger/归档/guard清理；若先于792合流，通知刷新闭包 | 06 | [✅ 已完成] 冻结门禁于 delivery 2a2b15478（二次 rebase range-diff 4/4 全等，证据绑定成立）：裸 t 5326 run 5324 pass（双红 plan484_024/plan707_cancel 均在案 flake，solo 绿归因）+tv 163/163+重放 38/38+consumer 5/5+interop 7/7；SD-01/02/03 沉淀（auto-diagnostics project/plans 新、frontend/overview 三处、auto-lex 窄同步+plans 新、frontend/plans 行、overview/goals、spec-index GROUPS、INDEX 再生 36）；ledger v35 受管离线投影（P793-D1/D2/T1/R2/M1）；两轮 rebase over 795/787（exclude 并集+基线统一重锚 48 边 strict pass；映射 827452a2a→39d7c0a0e/a8faf1e31→403bc7fc1/96bd5cc3e→228e15763/27cada126→2a2b15478）ff-only 线性合入；792 互链通知见 §9 合入收据 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（合同preflight） | next=合同确认后work | 0/8；本次未实现、未测试Rust。

2026-10-16 stage=work | plan_revision=1 | 入场 | worktree=D:/autostack/.wt/lang-793/auto-lang | branch=codex/plan-793-dev | base=579f6dec0（master；inputs.json 五文件 SHA256 零漂移核对：error.rs/ast.rs/route/discovery.rs/Cargo.toml/module-boundaries.toml）；789 已归档（依赖满足）；主检出预检零代码 WIP；组含 auto-down 兄弟 detached@c39ac7f。

2026-10-16 stage=work | plan_revision=1 | worktree=D:/autostack/.wt/lang-793/auto-lang | branch=codex/plan-793-dev | base=579f6dec0 | code_commit=827452a2a（冻结）+1bb5660b4（基线重锚） | dependency_revisions=auto-down 兄弟 detached@c39ac7f（只读）；auto-lex/auto-val=master 基面内 path 依赖 | task_ids=T-01..T-06 | outcome=pass（work-owned 全闭合） | evidence=AC-01: cargo check -p auto-diagnostics 零警告+V01 fixture 5/5+verify-deps 闭包零 auto-lang/man/vm/ui；AC-02: V02 golden 38/38 逐字段相等（SHA256 e97b9450）+V03 同一类型互传 5/5；AC-03: error.rs 1231→26 行门面+18 名冻结面+owner 侧 From×2 留存+auto/auto-lsp 编译零改动；AC-04: t01 消费闭包+t05-t06 净搬迁账/剩余 DAG 收据可回放；777 strict violations=0（重锚后）；check -p auto-lang 411 警告=基线零新增 | blockers=无 | open_findings=无（合同内未完项零） | merge_owned_pending=T-08 裸 cargo t+tv（冻结提交上）+实际触面档+SD-01/02 沉淀+ledger+归档+guard 清理；若先于 792 合流按 T-08 通知刷新闭包 | next_responsible_role=review

2026-10-16 stage=review | review_scope=final | reviewer_identity=当前独立 Codex 审查上下文（未参与本计划实施；未另派 agent） | plan_id=PLAN-793 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-793/auto-lang | branch=codex/plan-793-dev | reviewed_commit=1bb5660b44d7835617fd88aa0dfbf7ee433b280e | base_commit=579f6dec0941fb711664679a9cb8c29fa71bad20 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；auto-val/auto-lex=同树path依赖 | spec_inputs/SD_sha256=见[R1报告](attachments/793/review-r1.md) | acceptance_results=AC01 pass，AC02/03/04 partial，AC05 fail，AC06 pending | findings=F-793-R1-01(P1 golden ignored未入提交)；02(P2真实type用例实际Syntax)；03(P3本批门面fmt未净)；04(P2 SD漏同步auto-lex现行归属且门面路径错误) | evidence=新crate本机38例绿、两个consumer各5绿、777 strict零违规、定向error_tests4/error_spans8绿；不代表干净检出V02或真实type路径已通过 | pending_closeout_gates=最终复审通过后merge-owned裸cargo t+tv及新crate/consumer、SD/ledger/归档/guard清理；本轮未跑整档 | next=原树work修复四项并提交再审；保留T01/02，重开T03..06，executing 2/8；未改实现/合入。

2026-10-16 stage=work | plan_revision=1 | 修复轮（R1 F-793-R1-01..04 全闭合） | worktree=D:/autostack/.wt/lang-793/auto-lang | branch=codex/plan-793-dev | code_commit=a8faf1e31（修复冻结；前置链 827452a2a→1bb5660b4） | finding closure：F-01→invariant=永久重放期望件随库走→fix=.gitignore 否定规则+入提交→results=git show a8faf1e31:…json SHA256=e97b9450…与旧臂冻结件逐字节相等已证+重放 38/38 绿→gap 无；F-02→invariant=阶段归属与 oracle 真实性→fix=real_type_error_keeps_stage（check_field_type 生产接线，UI 会话，Type/E0106 span(57,1)，有效控制 parse OK）+real_name_check_error_keeps_stage（check_symbol→Name/E0201）+compile 用例重标注 parser 恢复（内层 SyntaxWithSource/E0007）+ParamChecker 无调用方/infer ctx.errors 不冲刷两事实记录→results=V03 7/7 绿→gap 无（Type 族生产面=check_field_type 已接线即为可达上界）；F-03→invariant=本批改动文件 fmt 净→fix=rustfmt 门面 re-export 布局→results=六文件同范围（复审口径）rustfmt --check 全清→gap 无；F-04→invariant=SD 不留矛盾 current-state→fix=sd-materials 增 SD-03（auto-lex 窄同步：From/attach_source/多错误聚合归属改 auto-diagnostics、上限仍归 auto-lang lib.rs、删 Name 残留）+SD-02 门面/实现路径修正+计数 18→20 名→results=sd-materials.md 重写；frontmatter supersedes 增 auto-lex/project.md、增量表增 SD-03 行→gap 无 | evidence（绑定 a8faf1e31）=V02 重放 38/38；V01 5/5+verify-deps PASS；V03 7/7；六文件 fmt 清；777 strict violations=0 | replay=旧例全保持（parse E0099/route Msg/? 传播/同类型互传/CJK 投影） | blockers=无 | open_findings=无 | merge_owned_pending=T-08 裸 cargo t+tv+实际触面档（冻结提交 a8faf1e31 上）、SD-01/02/03 沉淀、ledger、归档、guard 清理、792 互链通知 | next_responsible_role=review（R2）

2026-10-16 stage=review | review_scope=final | reviewer_identity=当前Codex审查上下文（未参与实施，独立重建证据，未另派agent） | plan_id=PLAN-793 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-793/auto-lang | branch=codex/plan-793-dev | reviewed_commit=a8faf1e311debb96df0c3ed660b7d7ccbb0e23a4 | base_commit=579f6dec0941fb711664679a9cb8c29fa71bad20 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；auto-lex/auto-val同树path依赖 | spec_inputs=见[R2报告](attachments/793/review-r2.md)；SD冻结sd-materials.md SHA256=cfa16b87d5f5d4fbc8e2671244cbf50546bf51aeb7f647ac62e221a575e994fc | acceptance_results=AC01..05 pass，AC06 pending | findings=F-793-R1-01..04 CLOSED，无新增 | evidence=提交内golden与旧臂冻结件三方字节一致(e97b9450…2a09d)+38例重放通过；V01 5/5；V03 7/7(真实Type/Name及有效控制通过)；strict=pass/0 violations/2 grandfathered；闭包/fmt/diff-check通过；生产逻辑/依赖/测试配置未变，明确复用R1定向error_tests4/error_spans8证据 | pending_closeout_gates=merge在最终冻结提交跑裸cargo t+tv及新crate/consumer、SD01/02/03沉淀、ledger/索引、归档、guard清理与792互链 | next=merge；reviewed 7/8；未跑整档、未合入或清树；R1历史不再表示当前开放项。

2026-10-16 stage=merge | PLAN-793:r1 | outcome=pass | delivery_commit=2a2b15478（master tip，ff-only 线性） | 交接链=reviewed a8faf1e31→rebase over 795(3e5aa91f6)→spec 沉淀 27cada126→再 rebase over 787 簿记(efa968660)→2a2b15478；映射与等价=首轮 range-diff 修复提交 = 全等/feat ! 仅 exclude 并集，终轮 4/4 全等（827452a2a→39d7c0a0e、a8faf1e31→403bc7fc1、96bd5cc3e→228e15763、27cada126→2a2b15478）；基线重锚=union 树 48 边 @228e15763（strict pass/0 violations，替代被略过的 1bb5660b4 锚） | checkpoints：prepared=SD01/02/03 按冻结正文（sd-materials SHA cfa16b87…）落 docs/specs（auto-diagnostics/project+plans 新、frontend/overview 三处、auto-lex 窄同步+plans 新、frontend/plans 793 行、overview 语言核心表行、goals GOAL-003+793、spec-index GROUPS+INDEX 再生 36 projects）｜landed=master tip==2a2b15478（git merge --ff-only，无 merge commit）｜ledger_refreshed=.autoos/specs.json v34→v35 guarded offline projection（通道=受管离线投影，无 store writer 可达——HTTP 探测空响应；备份 specs.json.bak-v34-793；六 section 前缀逐字节保持+仅追加 P793-D1/D2/T1/R2/M1+version+1；回读校验过）｜frozen_gates=裸 cargo t 5326 run 5324 pass（plan484_024/plan707_cancel 双红均在案 flake，solo 各自绿归因）+cargo tv 163/163+auto-diagnostics 重放 38/38+diagnostics-consumer 5/5+interop-consumer 7/7（绑定 27cada126，终轮 rebase 仅 docs/plans 祖先差，range-diff 全等证据复用）｜archived=本文件 git mv 至 docs/plans/archive/ + status: archived + 8/8；attachments/793 全量入库（golden 冻结件/receipts/review-r1/r2 及审计 JSON，-f 越过 *.json 忽略，K13） | production_artifacts=auto-lang 为库仓：无部署面（release 二进制/web bundle 不适用，与 789/790/791/795 同例） | batch_regression_due=否（793%5≠0；收据 3e5aa91f6 为今日 fresh，last_covered 795>793） | 792 互链通知（T-08 尾款）：793 已先于 792 合流——workspace members/exclude/边界基线均已推进（union 树 48 边），792 的 review/merge 在其冻结闭包清单上须按 793 后 master 重核（尤其 rust_workspace 独立解析面若触及根 manifest 快照） | cleaned=见下一行收据 | next=无（终态 archived）

2026-10-16 stage=merge | PLAN-793:r1 | cleaned=双树 wt-guard clean（auto-lang+auto-down 均零 reparse point）→ worktree 移除+分支 codex/plan-793-dev 删除（was 2a2b15478，已含于 master）+组目录 lang-793 移除零残留；master 终态 f59de89d0（archive 收据）←2a2b15478（delivery）线性；批量回归未到期（793%5≠0；收据 2026-10-16T16:10Z fresh，last_covered 795）| completion_kind: delivered | merge outcome=pass

## 10. 待澄清事项

若T01发现真实TypeError载荷并非Name别名可表达，或转换出现无法保持兼容的trait约束，给具体最小环和方案后修订；不得改公开错误为String或把AST整包倒灌新crate。
