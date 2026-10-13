---
plan_id: PLAN-770
status: drafting
feature_name: AC 文本与字符首批原生编译
plan_revision: 2
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-13
current_step: 0
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-hir/project.md
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-acc/project.md
new_spec_components:
  - docs/specs/auto-ac/source-text.md
touched_goals: []
affects: [auto-ac, auto-hir, auto-acc]
---

# [PLAN-770] AC 文本与字符首批原生编译

## 0. 变更摘要

在772修复后计算管线上增加显式 text profile，实现 str/Str/char 的源码/HIR→真实 native 与有界运行时资源。
依赖 [774](774-ac-data-resource-contract.md) 冻结共同规则；与 [771](771-ac-record-method-native.md) 标量 record 并行，不依赖其实现。
本计划不编译完整 token/lexer、不支持 List/enum/record 文本字段、不宣称 ACC Gen1。

## 1. 目标

1. 真实支持字符串/字符字面量、拥有/借用转换、len/char_at/slice/concat/eq/ne与字符比较/显式码点转换。
2. str/Str 随语境和资源契约区分，借用合法、拥有返回和清理可验证；不暴露 Rust 私有布局。
3. 新 identity/字段/类型/ownership 经文本与程序化 Bundle 真实 verifier，再入 Cranelift/PE。
4. 旧 profiles、条件 `for`、短路、求值顺序、来源与发布/执行器行为保留。

非目标：全语言 int/i64、完整 borrow checker、字素索引/Unicode normalization、f-string/raw/regex、List/Map、record/enum、use、生产 print/File/process、公开 ABI、ACC桥/迁移/自举、重构旧编译器。

## 2. 架构方案

显式 source-text→S0扩面→版本化 text Atom/来源表→bind/verify→Checked→text runtime calls + Cranelift→COFF/PE。
新建 `experimental/ac-core/runtime/text/` 独立静态库与显式内部 ABI；runtime不解析源码/HIR、不做类型检查、不运行VM。
公共HIR保留名义类型/访问/资源义务；native处理内部物理签名与布局；同一 shared finish_build 负责真实链接/原子发布。
生成runtime调用须声明并显式提供能力`ac.runtime.text.v1`；复用`--support-lib`输入静态库，新增伴随的内部runtime合同描述（target/API版本/实际库hash），T03产出、T05消费。缺库/错版本/错target在发射发布前拒绝；构建收据记录真实库及合同指纹，不能只相信caller给的capability字符串。

## 3. 技术栈

沿用独立ac-core Rust/Cranelift 0.126.2与Windows x64工具链。runtime选择遵守774 D05/06；锁文件/支持库/SDK/二进制指纹必须实际记录。
一切实现和验证在唯一worktree，无产品crates/旧auto-lib改动。

## 4. 需求分析与背景调查

主检出 `540dc7d30839eb7df8e68b4d19472358e170fe5b` 与 [输入表](attachments/770/planning-inputs.tsv)；772已修复复审合入并archived（9/9），scalar Spec已沉淀；共同前置774仍drafting（0/8）。
当前auto-ac/source-core-i32、auto-hir/project/stage-contract、auto-acc/project说明有限支持；774 data-resource-contract 为未来前置，不冒称已存在。
真实需求：lexer.at的char_at/len/slice/拼接、token.at的keyword_kind/kind_name；745报告说明char_at旧实现冲突；Design02/types规定str/Str和容器限制。
现有修改入口：source/{lexer,ast,parser,resolve,typecheck,adapter,mod}.rs、hir.rs、descriptor.rs、verify.rs、native.rs、main.rs。runtime/text、schema/text.atom、tests/text_{hir,source,native,resources}.rs、fixtures/text-{hir,source}为新建。
用户只授权本轮规划；实施待批准r1及前置收口，无模型/agent/预算/续跑授权。

- 2026-10-13引用/交接核查基线：主检出 d777ccbb227c41882ad5f6976c803b1e03b61135。编号映射为标量772、共同契约774、文本770、record771；770/771未改号。历史输入表保留来源，当前指纹见 [引用核查输入表](attachments/774/reference-update-inputs-20261013.tsv)。本次仅修正引用/取材和实施入口，不修改语言支持、验收阈值或执行状态。

## 5. 详细设计

### 5.1 版本与支持面

候选 source=`auto.source.text.draft`；HIR=`auto.hir.text.draft` / 1 / `text-i32-draft`（774 r2 同步：唯一身份表见 774 interface-contract §7.2，2026-10-13）。
精确三元组注册，写另一descriptor或程序化替换任一维必须拒绝；旧core/scalar不自动获得text能力。
新增Builtin/TypeDef StrSlice/StrOwned/Char语义身份，字段/签名/返回/调用/place与资源注解不得残留Unknown，所有引用重新核验。
D01..06 经774沉淀且与本节一致才实施；有材料变化先revision本合同，禁止实现者自改预期。

| 语义面 | 当前r1要求与可观察oracle |
|---|---|
| 文本存储/字符 | UTF-8；char独立Unicode标量类型，字面量1标量；显式`c.to_codepoint()`得码点、`char.from_codepoint(n)`反转（n 须有效标量）（774 r2 同步拼写，见 774 interface-contract §7.3）；无隐式char/int互转。具体合法拼写由774核查，若变更先修订。 |
| 索引 | len/char_at/slice按Unicode标量，不按byte或字素；`A中😀`len=3，0/1/2对应三字符；组合e+重音len=2。 |
| slice | `[start,end)`，空范围合法；负数、反向、end>len按774的runtime范围失败（本测试profile trap70）；不偷偷裁剪。 |
| equality | 完整UTF-8文本值相等；NUL不截断；不做normalization；只支持eq/ne，文本排序不在本计划。 |
| 字面量 | 普通/转义/跨行形态按774 D02确定；非法escape/UTF-8/多字符char在source诊断，未执行分支仍检查静态合法性。 |
| 资源/语境 | 采用774逐语境str/Str表；拥有构造/concat/slice提升与返回依赖显式/规范化转换；借用不外逃，禁move后使用与alias mutation；不把str全改拥有型。 |
| 方法解析 | 内建len/char_at/slice及`.to`按接收者类型绑定，局部/用户声明遮蔽先解析；不凭字符串方法名猜宿主。 |
| 控制流 | 继承scalar条件for/短路；分支、调用与命名实参左→右；Drop位置受effect/order约束。 |

### 5.2 runtime与可信门

runtime ABI/allocator/layout只由774合同定义；明确caller/callee所有权、长度单位、error/outbuffer、不panic穿ABI。
正常return/break/continue/分支转移清理唯一；故障注入沿合同回收已拥有资源；fatal进程终止不要求析构但不能伪称unwind。
literal/intrinsic/source/程序化Bundle均走真实类型+借用/移动Verifier；未知类型/TypeId越界/错误field/tag/profile受控拒绝，后端不能补验证。
新增诊断码清单T01冻结，mandatory case以确切码/原token定位验证；范围trap和分配失败类型分别取证，不把普通return70计为trap。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-text.md | text支持/拒绝、索引、字面量、资源、诊断与CLI有限合同 | 可执行语言边界 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-hir/project.md | 增加text身份/类型/资源verify，旧profiles规则保留 | 信任凭证 | AC-01/04 |
| SD-03 | modify | docs/specs/auto-hir/stage-contract.md | text前端/清理lowering实施状态，未实现通用pass仍planned | 阶段隔离 | AC-04/05 |
| SD-04 | modify | docs/specs/auto-ac/project.md | text runtime、source/HIR native能力及边界 | 不冒称生产ABI | AC-02/03/06 |
| SD-05 | modify | docs/specs/auto-acc/project.md | MD406/407/408有限profile新增取证及真实lexer剩余前置；不自动改旧人工绑定 | 双清单分离 | AC-06 |

## 6. 测试设计

Case均为未来T02..06物化，owner=work；现有测试绿不代表这些通过。

| Case | AC/invariant | 实际入口/输入 | 可观察预期与区分断言 | 边界 |
|---|---|---|---|---|
| V-01 | AC-01 身份 | check/build-source与HIR API；text/core/scalar头及三维未知身份 | 合法两条路径通过，未知/错域/writer换头拒绝；无Checked/native调用 | 新旧profiles |
| V-02 | AC-02 Unicode | 源码和直接HIR真实PE：A中😀、e+重音、NUL、空串、边界slice | len3/2；slice(1,3)=中😀；NUL后文本仍相等检查；stdout/stderr/exit精确 | 标量索引，不是字素 |
| V-03 | AC-02 char | 字面量/比较/显式转换97、0、0x10ffff；surrogate、-1、超上限 | 合法char名义类型通过；invalid scalar按阶段拒绝；int/char隐式混用verify拒绝 | Unicode标量 |
| V-04 | AC-03 正常寿命 | 真实PE拥有返回/参数view/move/clone/slice提升；循环/分支/return/break/continue | live=0、alloc=free，复制/释放次数遵守774；借用仍活着，move不双free | runtime测试计数能力 |
| V-05 | AC-03 失败寿命 | 每个申请点失败、concat长度溢出、slice/char_at越界；合法反控 | 约定失败分类/前驱trace；无panic/UAF/成功假象；既有资源按合同收口 | runtime fault注入，不变更生产调用行为 |
| V-06 | AC-04 可信变异 | 实际Bundle改params/result/类型ID、资源来源、访问模式、borrow escape、重复move | 确认变异；真实verifier拒绝，无Checked；合法view/转移兄弟例仍过 | 不用文本binder代替程序化门 |
| V-07 | AC-05 顺序 | mark_a/mark_b 包装的concat/实参/比较/短路及被跳过分配/trap | 精确trace，参数映射不重排、不执行跳过RHS；source与HIR各一腿 | 继承772短路 |
| V-08 | AC-05 静态/来源 | 未执行分支非法char/type、CRLF跨行文本、未知方法/名字遮蔽 | source原token与冻结stage/code，非法静态输入仍拒绝；合法用户函数不被内建夺名 | 不是解析全文所有语法 |
| V-09 | AC-05 生命周期规模 | scalar深链/Drop controls保持；文本链与调用混合各4096、资源循环4096 | check/构造/释放与受控PE（运行深度T01预flight）无栈崩溃；无固定语法深度修补 | 不承诺任意规模性能 |
| V-10 | AC-06 集成 | 新text gate +741/745/772 +若771已合入则record gate；build失败SENTINEL三件 | 全实际执行；三旧制品字节保持；缺runtime能力/库、错ABI版本/target受控拒绝；无scope越界或未来功能宣称 | 独立profile，非text-record组合 |

T01先做整个文本深链/临时资源/return路径可行性探针，冻结支持预算与拒绝类别；不能把合法语法规模截断当crash修复。
开发`cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets`+scoped text tests；最终完整ac-core test/fmt、`cargo test --manifest-path experimental/ac-core/runtime/text/Cargo.toml --locked`与runtime fmt/check/锁文件、`scripts/verify-ac-text-770.ps1`及旧gates。无crates/root定义改动，不跑根cargo t/tv/docs_gen；strict acc_inventory只核对未修改绑定。
真实exe用既有contained runner/build/run各60s与完整输出；测试support只观测，不解释Auto/HIR。768不是前置，不更新golden。独立final review在同一树，未指定额外agent/model。

## 7. 验收标准

| AC | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | source/HIR/Checked/writer/receipt精确text身份，新旧隔离；V01 |
| AC-02 | work | 全表文本/char规则源码与直接HIR真实PE或明确拒绝；V02/03 |
| AC-03 | work | str/Str语境、合法寿命与故障路径实际计数，无资源安全缺口；V04/05 |
| AC-04 | work | 程序化Bundle类型/资源变异真实门拒绝、无后端panic；V06 |
| AC-05 | work | 顺序/短路、原span/静态合法性、深输入与Drop保持；V07..09 |
| AC-06 | work | 最终受审基线的新旧实际gates、未实现边界、SD冻结与输入/产品hash闭环；V10 |
| AC-07 | review | 独立AC01..06/SD/遗漏债务审计，revision+HEAD绑定；缺口清零才pass |
| AC-08 | merge | 实际合入、SD/导航/ledger/index、归档/计数、守卫清理收据 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 路径/操作、成果与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 用户批准r2；772/774已复审合入 | 复核inputs/D01..07与r2一致；新reports/770-text/design.md含新增码/节点/资源/Case清单、全路径probe；与771登记热点合入顺序 | 01..06 |
| [ ] | T-02 | work | T-01 | hir/descriptor/verify、schema/text.atom、tests/text_hir.rs；名义类型/资源可信门、三维身份与Bundle变异V01/03/06 | 01/02/04 |
| [ ] | T-03 | work | T-01/02 | 新runtime/text静态库与锁文件/显式ABI/fault计数，独立runtime测试V04/05；按774内存策略实现 | 02/03 |
| [ ] | T-04 | work | T-02 | source七模块扩面与fixtures/text-source、text_source.rs，真实bind/verify、literal/span/方法解析V02/03/08/09 | 02/04/05 |
| [ ] | T-05 | work | T-03/04 | native/main/共享finish_build接runtime与source/HIR PE，text_native/resources tests；V02..07/09，旧事务不分叉 | 01..05 |
| [ ] | T-06 | work | T-05 | 新verify-ac-text-770.ps1/case manifest，mandatory补充全量；新旧gates、scope/严格盘点；若771已合入联跑其gate V10 | 06 |
| [ ] | T-07 | work | T-06 | README与reports/770-text/{verification,proposed-spec-delta}.md；证据hash/SD/边界冻结，不发布canonical | 06 |
| [ ] | T-08 | review | T-01..07完成、writes暂停 | 同树独立AC/SD/反例/生命周期审计，pass才reviewed，失败返work | 07 |
| [ ] | T-09 | merge | T-08pass与合入授权 | 明确两分支实际顺序；只消费新基线经work重跑联合回归且重新review的fresh凭证；SD/ledger/index/归档/计数/批回归到期判定；guard清理 | 08 |

意向 `D:/autostack/.wt/lang-770/auto-lang`、branch `plan-770-dev`，尚未创建。work完成7/9、review8/9、实际closeout9/9。受审代码/基线变化时回同一树work重跑受影响gates、重新独立review；不能把实现/联合正确性推迟到T09。

## 9. 复审记录

以下2026-10-12记录使用当时编号，作为历史保留；现行执行入口以§8及2026-10-13记录为准。

2026-10-12，stage:new，revision1，outcome:pass（有依赖门的草案preflight）。新增T01..09/AC01..08/SD01..05/V01..10。
取号中央脚本串行、ID唯一；未实施、未建树、未跑cargo/PE，表中案例均待物化。支持决定依赖769，不允许跳过该门。
next：批准r1且769/767实际收口后work；可与771并行，组合能力另立合同。

规划期间主检出由`897eb34911a4f496a28ba7cdfd8a82815fa043d1`推进至`540dc7d30839eb7df8e68b4d19472358e170fe5b`，提交仅768计划与取号簿记；实现路径零差异。本计划按最终观察HEAD与文件hash绑定，保留已有768 executing/WIP；它不是本批实施前置。

2026-10-13，stage:new，revision2，outcome:pass（引用/交接核查，非实施完成）。770/771编号不变，前置改为现行774与已归档772；更新活跃链接/取材/拟建路径，保留历史附件与既有脚本路径。Task/AC/Case/SD ID、支持与测试语义不变，status仍drafting、current_step仍0。策略队列见[后端演进§7.5](../design/strategy/auto-native-backend-evolution.md#75-现行acacc队列与编号映射2026-10-13)。[核查收据](attachments/774/reference-update-20261013.md)包含当前路径/链接/指纹。共同774完成并复审合入后才放行770/771的依赖实施；本次未建树/实施/合入/运行Cargo。

## 10. 待澄清事项

1. 774 D02/03/04/06的语义/资源支持先决结果：当前r2提出确切候选oracle，前置结果不一致先revision，不默认授权改承诺。
2. 771共享热点顺序在T01登记，非共用worktree；第二合入方重新验证，不能只解文本冲突。
3. 完整lexer/容器/enum/文本record/IO/ACC由Design36后续owner认领，不是本计划待补验收。
