---
plan_id: PLAN-801
status: execution_done
feature_name: 方言接口独立与 Parser 依赖倒置
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 6
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/frontend/design/dialect-system.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/auto-parser-api/project.md]
touched_goals: [GOAL-003]
affects: [auto-parser-api, auto-lang/frontend]
depends_on: [PLAN-789, PLAN-793, PLAN-799]
---

# [PLAN-801] 方言接口独立与 Parser 依赖倒置

## 0. 变更摘要

在799的单一AST模型上新增auto-parser-api：独立方言只依赖轻接口/模型，由auto-lang实现解析端口。生产Parser接入新接口，778外部model实现改用该接口；保留旧Dialect/ParserBridge兼容调用。**799现已executing、尚未交付，是本计划work硬前置**；不得以复制AST或依赖完整auto-lang替代。

## 1. 目标

使真实外部文法实现的normal/build闭包不再包含auto-lang及Parser实现，仍能委托核心表达式/类型解析并参加生产派发，保留778优先级、场景、递归、恢复与旧调用行为。

非目标：迁整个Parser/type-store/inference、完整UI文法、自动传播所有import入口注册、分离UiAst或删除UI Token。内建UiDialect仍可使用上层私有Parser能力；剩余边如实登记，不宣称auto-ui已可搬仓。

## 2. 架构方案

外部方言 → auto-parser-api → auto-ast/auto-lex/auto-diagnostics及必要轻值；auto-lang → auto-parser-api并提供端口adapter。接口crate无Parser/VM/UI/Man实现依赖，测试也不以auto-lang作dev依赖；解析集成测试放在上层driver。AST/词法/诊断沿用单一定义，不增加镜像DTO。

采用增量接口：新增ExternalDialect与ParseContext（拟名，T01可等价调整），旧auto_lang::dialect::Dialect及ParserBridge保持。新方言适配进入现有生产有序派发，不维护另一套行为不同的派发表。轻CompilerSession/Scenario和DialectPriority可迁为API单一定义、旧路径re-export，不搬执行会话。旧ParserBridge::new与内建pub(crate) escape hatch留上层，不能公开整个Parser换取迁位。

## 3. 技术栈

Rust workspace crate、对象安全端口/adapter、Arc注册、独立方言crate和上层Parser driver；Cargo metadata闭包检查与778已有/新增生命周期反控。

## 4. 需求分析与背景调查

用户2026-10-17授权建立后续迁移合同，并确认799尚未实施、保留为前置。本轮仅计划/设计/路线簿记，不执行Rust、不创建agent/工作树、不发布canonical Specs、不操作旧仓refs；未授予新预算或模型分工。work按确认后的合同另行启动。

初次取材主检出`143aff0a258fdca8bda9a3b8068dc8a9cced623c`；规划期间复核至`2d246b4e86c1ebf5138f6a316c1896e56a5d35f8`，800已归档8/8、799转executing但主检出仍无auto-ast。789/793及792/794已归档；路径、SHA256及状态见[输入指纹](attachments/extraction-next-20261017/inputs.json)。本计划不消费800 resolver API；work须重新绑定799最终模型路径及交付SHA。

frontend/overview为778现行合同；dialect-system.md仍写mem::take移表，已被778的Arc递归派发取代，是文档陈旧，不能照旧文档恢复问题。ParserBridge直接持Parser，因此“外部crate能实现trait”尚不等于“外部文法闭包无需语言实现”。BridgeCheckpoint含源码生命周期、LexerState及五旗标；syntax_error的泛型便利方法不能原样放入dyn端口。

778仅外置无注解model子集；默认内建model不支持同样的裸字段扩展。widget结构解析和import中新Parser尚未完整注入。本计划不扩大这些语法/入口承诺，不复制model成为第二份真值。

## 5. 详细设计

T01在799合入后完成有界整链可行性验证：新接口方言通过生产Parser注册，递归委托parse_expr/parse_type，执行None回滚与Err恢复，旧trait实现仍编译。记录方法、对象安全/生命周期、checkpoint所有权、轻session迁位和旧API映射。若必须暴露Parser/TypeStore、复制AST或降低恢复承诺才能实现，停止后续任务并needs_replan，不能只交调查报告。

端口覆盖现有桥面的token kind/text/pos、前进/expect/空行、核心expr/type、scenario/fn/on查询和语法错误；跨界只传owned文本/现有AST，不返回Parser/lexer/scope引用。泛型便利方法留具体facade，dyn端口使用对象安全参数。

Checkpoint为与本次上下文关联的不透明凭据；T01选择安全生命周期令牌或有作用域句柄表，记录释放点。禁止unsafe延寿、源码'static要求、全局快照池。None回滚游标+in_on_body/in_fn_body/in_dot_rhs/with_header/body_depth五旗标；Err保失败游标、恢复五旗标；诊断单调追加，不扩展全符号表事务。句柄形式跨上下文/过期使用须受控拒绝且调用退出清理；借用形式误用须编译失败。递归不能覆盖外层快照。

兼容层只做协议转换，不重复语法。旧/新方言混用仍按Override→builtin→Append及同段注册顺序查询，with_session保外部注册，递归可见全部注册。ui-dialect-consumer/src/lib.rs的model实现迁入新独立dialect-api-consumer（只依赖轻包）；原consumer作为含auto-lang的上层driver及旧trait最小兼容用例，不留第二份model算法。

实际删边是外部文法实现→auto-lang；完整内建UI grammar→Parser仍在。与802/803主写面不交叉，共享接线及792闭包责任见[批次编排](reports/extraction-next-parallel-order-20261017.md)。新包进入交付闭包则沿792现有清单/admission/staging接口接线及重验，不绕过其失败保护。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-parser-api/project.md | 无独立接口→轻方言/端口/快照与无实现依赖 | 外部实现脱离语言包 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-lang/frontend/overview.md | 旧桥pilot→兼容双接口、单派发及剩余完整文法边 | 真实生产切口 | AC-02/03/04 |
| SD-03 | modify | docs/specs/auto-lang/frontend/design/dialect-system.md | 陈旧mem::take→已有Arc递归恢复及新adapter | 防止778行为倒退 | AC-02/04 |

## 6. 测试设计

work实现/自检→同树独立final review→merge，未指定额外agent/模型。新fixture由T02/T05创建，本轮不算执行证据。拟唯一树`D:/autostack/.wt/lang-801/auto-lang`，branch `codex/plan-801-dev`；work核对实际路径/base/HEAD/799交付。

| Case / owner | AC / 不变量 | 输入、实际入口 | 区分错误实现的oracle与覆盖边界 |
|---|---|---|---|
| V01 / work | 01 依赖独立 | 新test/fixtures/modularity/dialect-api-consumer/Cargo.toml，fresh测试/metadata | model实现normal/build及本crate测试无auto-lang/Parser/VM/UI；driver另依赖auto-lang，不用dev-deps掩盖环 |
| V02 / work | 02 语法/类型 | 原778 Core/UI字段/裸字段/注解回退、expr/type嵌套，经原consumer driver真Parser | AST投影等于旧臂；accept计数+特异节点证明新方言接管；默认builtin裸字段行为不变 |
| V03 / work | 02 递归/派发 | 新旧方言混排优先级、换session、合法递归、None及Err后下一语句 | 顺序/命中节点明确；None游标+五旗标复原，Err保失败游标且五旗标复原，注册仍可用；不只断言无panic |
| V04 / work | 02 快照寿命 | 同上下文嵌套/递归快照、跨上下文/过期负控 | 有效恢复等于旧臂；误用编译拒绝或明确Err，作用域退出无残留快照；不承诺无限资源/任意深度 |
| V05 / work | 03 旧API | 旧Dialect实现、ParserBridge::new/checkpoint/泛型syntax_error、session字段构造/类型互传 | 旧公开调用编译且真解析，同一session类型身份；内建escape hatch仍不可外部访问 |
| V06 / work | 04 单一性/交付 | 源清单、777审计、792既有交付入口/consumer driver | 无重复model/AST/派发算法；新增闭包可交付，受影响fresh/locked重定位及缺输入/坏hash反控通过；不影响时提供metadata证据 |

work命令：`cargo check -p auto-parser-api`、`cargo check -p auto-lang`、两个consumer的`cargo test --manifest-path .../Cargo.toml`及新增定向快照/派发测试。final review通过后merge在冻结提交跑裸`cargo t`+`cargo tv`；仅实际修改ui_gen追加单进程`cargo tu`。tf归主检出到期批量单实例；work/review不得整档。本轮Category A不跑cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 新接口及真实外部方言依赖无语言实现，V01通过 |
| AC-02 | work | 生产接入，778语法/混合派发/恢复/递归/快照寿命保持，V02–04通过 |
| AC-03 | work | 旧公开调用/类型身份保持，文法单一实现，V05及源码审计通过 |
| AC-04 | work | 剩余边、删边/迁位量、792闭包及SD素材证据完整，V06通过 |
| AC-05 | review | 独立AC/SD复审及遗漏扫描通过，绑定提交/依赖，merge门pending明确 |
| AC-06 | merge | 冻结t/tv及触面门、Spec/ledger/归档和guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 已核对路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 799已审查合入、合同确认 | parser_bridge.rs/dialect.rs/session.rs/parser.rs及799最终模型，整链可行性/旧API/快照决策与旧臂冻结 | 01/02/03 |
| [✅ 2026-10-17 work] 799 交付重绑：auto-ast 已合入主检出（delivery 9b510f79a，归档回执 48ff19821；闭包=auto-val/auto-lex/serde），`auto_lang::{ast,token,lexer,error}` 均已是 auto-ast/auto-lex/auto-diagnostics 薄门面（类型身份单一实现）。整链核实与 T01 有界决策（D1–D7，全文见 §9 本轮记录）：**D1** 新 crate `crates/auto-parser-api`（lang-core 域）仅依赖 auto-ast/auto-lex/auto-diagnostics，auto-lang 单向依赖之，外部方言闭包零语言实现；**D2** 端口=对象安全 `ParseContext` trait，镜像 778 桥面（游标/expect/空行/checkpoint/parse_type/parse_expr/scenario/on·fn体/syntax_error(String)），泛型便利方法留 ParserBridge 具体门面；**D3** 快照=有作用域句柄表（弃生命周期令牌）：`ParseCheckpoint{parser实例id,slot,全局单调gen}` 不透明凭据，表挂 Parser、派发器每次方言调用后按 watermark truncate（调用退出清理；嵌套水位保外层），跨上下文=实例 id 拒绝、过期=gen 不重复拒绝、restore 单用镜像旧 move 语义，无 unsafe/无 'static/无全局池；**D4** CompilerSession/Scenario+DialectPriority 迁 auto-parser-api 单一定义，旧路径 re-export（类型身份不变）；**D5** 生产接入=ExternalDialectAdapter 实现旧 Dialect trait 进同一三段派发表（pre→builtin→post 与 778 恢复不变量全走既有 try_dialect_stmt/try_token_dialect_stmt，零第二派发表）；**D6** 旧 Dialect/ParserBridge/BridgeCheckpoint/register_dialect/Box 形态与内建 escape hatch 全保持；**D7** fixture 拓扑：新 dialect-api-consumer 仅依赖 auto-parser-api（+verify-deps.sh 闭包 oracle 镜像 791/795/800 模式），旧 ui-dialect-consumer 转 driver（auto-lang+dialect-api-consumer），旧 trait 最小兼容用例=`probe <expr>`→Stmt::Expr（非第二份 model 算法）。结论：无需暴露 Parser/TypeStore、无需复制 AST、无需降低恢复承诺，**可行性成立，进 T-02**；777 增 lang-core 域映射、792 需把 auto-parser-api 纳入 DELIVERY_CRATES 重验（T-05）。实施树改用仓规 `D:/autostack/.wt/lang-801/auto-lang` + branch `plan-801-dev`（AGENTS.md 命名约定优先于计划拟名 codex/plan-801-dev）。 |
| [x] | T-02 | work | T01 | 新crates/auto-parser-api及dialect-api-consumer，轻session/端口/trait，check及V01 | 01/03 |
| [✅ 2026-10-17 work, code 8f3e8e9f8] crates/auto-parser-api（475 行 src：轻 session 迁入/ExternalDialect/DialectPriority/对象安全 ParseContext+句柄凭据 ParseCheckpoint；轻层门面 re-export ast/token/error 与旧 auto_lang 模块名对齐）+ dialect-api-consumer fixture（model 子集单份实现迁入）；workspace members+exclude 接线。V01：fixture 3/3 + verify-deps.sh PASS（normal/build 闭包无 auto-lang/man/vm/ui）+ cargo metadata 闭包=[api,ast,lex,diag,val]。 |
| [x] | T-03 | work | T02 | R1 F-801-R1-01：修正递归复用槽位的快照作用域清理；原生产接入/旧面证据保留 | 02/03 |
| [✅ 2026-10-17 R1 修复, code 49512993b] 不变量修订=快照寿命由**签发作用域**决定（非表长水位）：`PortSnapshotSlot` 增 `scope` 字段（派发器每次方言调用前递增、嵌套保存/恢复），两派发器退出改 `port_release_scope(call_scope)` 按归属置 tombstone+尾部收缩，弃 `truncate(watermark)`；`port_restore` 存活位校验原有。根因（tombstone 复用使内层快照落在外层入口表长之下，长度截断清不到、gen/实例校验全匹配）在 parser_bridge.rs 槽位文档登记。原生产接入/同表派发/门面迁移证据保留（下行历史）。 |
| [✅ 2026-10-17 work, code ace61f7c4] ParserBridge 实现 ParseContext（同名 inherent 委托零重复；checkpoint/restore 走句柄表后端）；ExternalDialectAdapter 进同一三段派发表；register_external_dialect/with_external_dialect；轻 session/DialectPriority 迁移为薄门面（类型身份不变）；派发器每次方言调用后按水位 truncate（调用退出清理，嵌套保外层）。定向：dialect 7/7+session 136/136+plan778 4/4，零新增警告（基线 414→411）。V03/04 测试落 T-04 driver 提交（29ea71232）：v03_dispatch 4/4、v04_snapshot_lifetime 3/3。 |
| [x] | T-04 | work | T03 | R1：保留model迁位/V02/05；补Ident/token×Some/None/Err递归槽位复用永久反例及正控 | 01/02/03 |
| [✅ 2026-10-17 R1 修复, code 49512993b] 复审探针 12 例收编为永久回归 `tests/r1_scope_expiry.rs`（6 无洞正控 + 6 复用槽位反例，Ident/token × Some/None/Err 全组合；oracle 三元组=外层快照存活回位/内层凭据受控拒绝/游标不动），12/12 绿；既有 V04 三例/跨 Parser/旧 BridgeCheckpoint 控制与 778 全套保持绿——driver 全套 48/48（36+12）。model 迁位/V02/V05 原证据不变（29ea71232）。 |
| [✅ 2026-10-17 work, code 29ea71232] ui-dialect-consumer 转上层 driver（auto-lang+dialect-api-consumer+auto-parser-api）；旧 trait 最小兼容用例 LegacyProbeDialect（probe expr→Stmt::Expr，用 by-value checkpoint+泛型 syntax_error，非第二份 model 算法）；778 r1/r2/v01_v05 断言逐字保持仅换 with_external_dialect 注册（V02"AST 投影等于旧臂"oracle）；v05_legacy_api 3/3（session/Priority 跨门面类型身份双向编译断言）。driver 全套 36 测试 8 目标 PASS；期间修 v04 测试自身 if-let MutexGuard 临时量死锁（edition 2021 语义，测试代码缺陷非产品缺陷）。 |
| [x] | T-05 | work | T04 | R1 F-801-R1-02：补本版792 fresh/locked重定位及正式缺输入/坏hash负控，复验受影响V01–06 | 01–04 |
| [✅ 2026-10-17 R1 修复, code 49512993b] 792 正式入口 `generated-ui-consumer/driver.py` 本版完整重跑 **25/25 PASS exit0**：generate crates=13（12 受管含 auto-parser-api+app）；V01 fresh 独立锁（3s，patch 单源+pin 12/12 命中）+真实 build 118s 产物在位；V02 A→B 不带 target/ locked 112s+本地 target+metadata iced_widget 单源@B 内/越界 path=无+资源字节=producer；V03 十四态负控（等长篡改/坏 hash/缺资源/未登记/漏 patch exit101/不兼容 pin exit101/外来目录/换装保旧/恢复后成功/sidecar 外部+根相对/raw TOML/邻位保留/他应用存活）+恢复绿；V04 开发路由+模板同源。收据 attachments/801/delivery-driver-r2-receipt.json+log（F01 修复后代码上执行）。777 strict/单一实现审计原证据不变。 |
| [✅ 2026-10-17 work, code 643d5bdeb+06e559a60+7c2c74ddc] 777：auto-parser-api 域登记 lang-core + 基线重采样（增量恰 4 条合法声明边：auto-lang→auto-parser-api + 端口→ast/lex/diagnostics，零新增禁向，strict verdict=pass 0 违规）；792：DELIVERY_CRATES 11→12 纳入 auto-parser-api，delivery 单测 9/9 PASS；V06 单一实现审计：parse_model_subset/parse_model_block/派发器/ExternalModelDialect 全仓唯一（grep 收据入 t06-receipt）。 |
| [x] | T-06 | work | T05 | R1：更新真实验收/快照收据，补generated-ui-distribution权威闭包SD目标与impact，冻结新提交送审 | 04 |
| [✅ 2026-10-17 R1 修复, code 49512993b] SD-04 新增：generated-ui-distribution.md 交付闭包清单 11→12 成员（+auto-parser-api 及闭包内依赖说明）worktree 备稿；SD-01..03 快照契约同步作用域语义（"水位截断"→"签发作用域失效"）；frontmatter supersedes 增补该 target；t06-receipt 增 R1 修复轮闭合节（finding→不变量→受影响路径→修复与永久用例→结果→剩余缺口=无）。新冻结 code=49512993b 送 T-07 二轮复审。原收据/SHA 见下行历史。 |
| [✅ 2026-10-17 work, code 9f94b32ce] 剩余边四条如实登记（内建 escape hatch 完整文法/生产入口注册注入/widget 结构直连/旧 trait 兼容层，见 attachments/801/t06-receipt.md）；迁位净删清单（27 files +1654/−315，净删边=外部文法实现→auto-lang）；SD-01（新 auto-parser-api/project.md）/SD-02（frontend/overview 增 PLAN-801 节）/SD-03（dialect-system 陈旧 mem::take→Arc 派发+新端口）worktree 备稿待 merge 沉淀；SHA256×10 入收据。work 阶段收口送 T-07 独立复审。 |
| [ ] | T-07 | review | T06 | 同树独立逐项及生命周期/兼容反控复审，merge门pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master，受影响复审/冻结门、Spec/ledger/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-17 / stage: work / PLAN-801 revision: 1 / outcome: pass（R1 两 findings 全修，T-03..T-06 R1 要求完成）/ worktree_path=D:/autostack/.wt/lang-801/auto-lang / branch=plan-801-dev / base_commit=043ca8567 / code_commit=49512993b（fix 轮：9f94b32ce→49512993b；修复在复审冻结基线 9f94b32ce 之上单一提交） / dependency_revisions=auto-down c39ac7f 不变 / task_ids=T-03..T-06（R1 重开项）/ finding_closure=F-801-R1-01：不变量=端口快照寿命由签发作用域决定；受影响路径=Ident/token 两派发×Some/None/Err×tombstone 复用形态；修复=槽位 scope 归属+port_release_scope；永久用例=r1_scope_expiry 12 例（6 正控+6 反例）；结果=12/12+driver 48/48+定向 dialect 7/7+plan778 4/4；剩余缺口=无（跨 Parser/旧 BridgeCheckpoint 控制保持绿）。F-801-R1-02：受影响面=本版 12 成员交付闭包；修复=792 driver 完整重跑 25/25 exit0（V01 fresh 锁+真构建 118s/V02 重定位 locked 112s+metadata 身份/V03 十四态负控+恢复绿/V04 保持）+SD-04 权威闭包 11→12 备稿+supersedes/收据同步；剩余缺口=无。 / evidence=attachments/801/{t06-receipt.md R1 节, delivery-driver-r2-receipt.json, delivery-driver-r2.log.txt, r1_scope_expiry.rs} / blockers=无（merge 门保持 pending：冻结裸 t+tv；tf 主检出批量档） / next_responsible_role=review（T-07 二轮 final review，审查基线 49512993b）

2026-10-17 / stage: review / review_scope: final（R1） / reviewer_identity=独立于实施的本轮Codex审查上下文，未另开agent / plan_id=PLAN-801 / plan_revision=1 / outcome=needs_fix / worktree_path=D:/autostack/.wt/lang-801/auto-lang / branch=plan-801-dev / reviewed_commit=9f94b32ce60e2db2dfd5d70c67b8ff540a1e5533 / base_commit=043ca8567a5f6e86c8af9147a706d96e4f2bde3b / dependency_revisions=auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 clean；799 delivery 9b510f79a35310d8b6449d1d94edac4da72f5e06 祖先exit0；根锁e781475c209b7b0cefc77c6f4384d7fd1e4e8925300fe07d35fe79fe9fab56e4 / spec_inputs、命令与输入冻结=[R1报告](attachments/801/review-r1.md)、[baseline](attachments/801/review-r1-baseline.json.txt)；[SD冻结](attachments/801/review-r1-spec-delta.patch) SHA256=d278161487a980b6183f490b3c40cc0c0133895acc21c1700c31628eb187adab，main dry-run exit0、未应用 / acceptance_results=AC01 pass、AC02 fail、AC03已审兼容面pass、AC04 partial、AC05 fail、AC06 merge-owned pending / findings=F-801-R1-01（P1，递归回调复用外层tombstone后，长度水位truncate漏清内层快照，过期凭据可恢复游标）；F-801-R1-02（P2，受影响792交付fresh/locked及缺输入/坏hash正式负控未交，本轮9/9仅生成器单测；SD漏generated-ui-distribution权威闭包11→12更新） / evidence=独立12例6正控通过+6复用反例失败（Ident/token×Some/None/Err）；轻consumer3/3、driver36/36、API4+doctest1、delivery9/9、check exit0（禁用本机sccache端口冲突wrapper）、闭包PASS、777 strict pass/0违规/2既有grandfathered；无新增改动面fmt问题（parser6hunk base已有）、diff-check exit0 / pending_closeout_gates=最终审查通过后的冻结裸cargo t+tv、规范/ledger/索引/归档/guard清理pending，ui_gen无触面不跑tu、tf主检出到期批量档；F02的work-owned V06不能延期为merge门 / next=executing 2/8，保留T01/02、重开T03..06，原树修F01/补永久组合oracle与F02正式交付/SD证据，提交闭合表后再final review；未修实施、未合入。下方work完成记录保留为历史，由本轮实证结论取代。

2026-10-17 / stage: new / PLAN-801 revision: 1 / outcome: blocked / next: 799实施、final review和合入后，work重核模型并执行T-01。阻塞为已确认的实施前置，本轮T/AC均未完成。见[合同预检](attachments/extraction-next-20261017/preflight.md)。

2026-10-17 / stage: work / PLAN-801 revision: 1 / outcome: pass（T-01 完成，前置解除）/ worktree=D:/autostack/.wt/lang-801/auto-lang / branch=plan-801-dev / base=master 4755b486a / 前置核验=799 已合入归档（delivery 9b510f79a，终回执 48ff19821），auto-ast 在主检出，789/793/800 均归档 / evidence=T-01 决策 D1–D7（§8 T-01 行），主检出自查无代码路径 WIP（仅他线 docs/plans 簿记） / blockers=无 / next=T-02 实施（crates/auto-parser-api + dialect-api-consumer）。

2026-10-17 / stage: work / PLAN-801 revision: 1 / outcome: pass（T-01..T-06 全部完成，work-owned AC 达成）/ worktree_path=D:/autostack/.wt/lang-801/auto-lang / branch=plan-801-dev / base_commit=043ca8567（master 分叉点，含 T-01 簿记）/ code_commit=9f94b32ce（链：8f3e8e9f8→ace61f7c4→29ea71232→643d5bdeb→06e559a60→7c2c74ddc→9f94b32ce）/ dependency_revisions=auto-down c39ac7f（组内兄弟 detached，只读） / task_ids=T-01..T-06 / evidence=AC-01（V01：fixture 3/3+verify-deps+metadata 闭包无语言实现）；AC-02（V02 778 断言逐字保持 26 测试+V03 4/4+V04 3/3，driver 全套 36/8 目标）；AC-03（V05 3/3 类型身份+旧 API 面；单一实现 grep 审计四项全唯一）；AC-04（777 strict pass+基线 4 边重采样；792 DELIVERY_CRATES 12+delivery 9/9；SD-01..03 备稿+SHA 收据 attachments/801/t06-receipt.md） / blockers=无（review/merge-owned 门禁显式 pending：冻结裸 cargo t+cargo tv——parser.rs 触 VM/编译器面；ui_gen 零触面不跑 tu；tf 到期主检出批量档；预存 bench E0601 非本线引入已在收据登记） / next_responsible_role=review（T-07 同树独立复审）

## 10. 待澄清事项

用户确认时799尚未实施，规划期间已转executing；最终AST交付仍是前置，无待用户决定的状态疑问。T01负责对象安全/快照具体机制与旧调用映射的有界决策，不能以该决策删减AC。完整UI文法/整仓迁移在本接口及真实剩余边上另立计划。
