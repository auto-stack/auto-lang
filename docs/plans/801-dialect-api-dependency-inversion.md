---
plan_id: PLAN-801
status: drafting
feature_name: 方言接口独立与 Parser 依赖倒置
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/frontend/design/dialect-system.md]
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
| [ ] | T-01 | work | 799已审查合入、合同确认 | parser_bridge.rs/dialect.rs/session.rs/parser.rs及799最终模型，整链可行性/旧API/快照决策与旧臂冻结 | 01/02/03 |
| [ ] | T-02 | work | T01 | 新crates/auto-parser-api及dialect-api-consumer，轻session/端口/trait，check及V01 | 01/03 |
| [ ] | T-03 | work | T02 | 上层adapter与生产注册/同表派发，旧trait/桥兼容及快照作用域，V03/04 | 02/03 |
| [ ] | T-04 | work | T03 | model单份迁入轻consumer、原consumer转driver，V02/05及必要引用适配 | 01/02/03 |
| [ ] | T-05 | work | T04 | V01–06、777精确domain/边，沿792既有入口闭合受影响交付清单与验证 | 01–04 |
| [ ] | T-06 | work | T05 | 剩余完整UI文法/Parser图、单一实现/迁位净删与SD/SHA收据，提交送审 | 04 |
| [ ] | T-07 | review | T06 | 同树独立逐项及生命周期/兼容反控复审，merge门pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master，受影响复审/冻结门、Spec/ledger/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-17 / stage: new / PLAN-801 revision: 1 / outcome: blocked / next: 799实施、final review和合入后，work重核模型并执行T-01。阻塞为已确认的实施前置，本轮T/AC均未完成。见[合同预检](attachments/extraction-next-20261017/preflight.md)。

## 10. 待澄清事项

用户确认时799尚未实施，规划期间已转executing；最终AST交付仍是前置，无待用户决定的状态疑问。T01负责对象安全/快照具体机制与旧调用映射的有界决策，不能以该决策删减AC。完整UI文法/整仓迁移在本接口及真实剩余边上另立计划。
