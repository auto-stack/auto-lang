---
plan_id: PLAN-770
status: executing
feature_name: AC 文本与字符首批原生编译
plan_revision: 2
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-15
current_step: 7
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-hir/project.md
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-acc/project.md
new_spec_components:
  - docs/specs/auto-ac/source-text.md
# touched_goals 为空是有意的（复审 F-04 书面解释）：本计划是实验性
# ac-core 原型的独立计划，不修改任何 GOAL-* 登记项的能力状态，也不关闭
# GOAL-017（ACC Gen1 判据）；spec 沉淀只登记 experimental 支持面。
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
| [x] | T-01 | work | 用户批准r2；772/774已复审合入 | ✅ 已完成：[design.md](../reports/770-text/design.md)@6dbbf2712——47项输入指纹复核39零漂移+8归因、D01..07一致性表（无冲突）、码/节点/资源/Case冻结、774探针65/65绿复跑、771热点合入登记（771仍drafting，770预计先合入）；774 r2/r3/r4/r5修订链全部纳入 | 01..06 |
| [x] | T-02 | work | T-01 | ✅ r9 返工完成 @e60b6eacc（§9.21）：R9-01 闭合——循环入口 move 预杀仅在存在真实回边（!block_terminates）时执行；AwaitLoop after-merge 应用 if-join 同款责任规则（dead 已并入的入口 owner 撤销 caller 清理义务——组合 oracle 抓出的循环后 FREE-INVALID 双释放）。p03 双腿 run3 精确单次转移、p04 重复 move 保持拒、c01..c06 保持。历史交付：** ✅ r8 返工完成 @7468c553e（§9.19）：R8-01 闭合——block_terminates 消费与走查相同的 const_truth 可达性（恒真只看 then、恒假只看 else、未知需全臂+无 else 落空；no-else 臂入 memo；嵌套恒定条件组合正确）。p01/p02/p04 双腿 run0 精确 LIFO trace、p03 真实回边保持拒、c01..c03 正控保持。历史交付：** ✅ r7 返工完成 @314cbbeb6（§9.17）：R7-01 闭合——回边分类显式化：continue 出口恒入重入 fixpoint，body 态仅当 `block_terminates=false`（存在自然落空路径）才入；break/return 终止体零重入贡献（真实 break 态仍入 after-loop、return 仍不合流）。m01/m02 冻结 atom 直接 HIR run0 精确 trace、c05 源码 break-only 合法；r6 重入拒绝（p03/p04/陈旧代际）与全部正控保持。历史：R7 重开前的 r6 返工完成 @643a1b3eb（§9.15）：R6-01/02 闭合——循环体按 RE-ENTRY 状态 fixpoint 重走（base+自然 back-edge+continue 出口 dead/backing 合流；收敛键=dead+id 投影 backing；fresh owned let 清死记+构造代际递增，backing 携 (owner,gen) 陈旧检测）；帧级静态可达性过滤 loop_exits 记录（静态合法性检查不变）。p03/p04 双腿拒、p01/p02 合法化、c01..c04 保持。历史：R6 重开前的 r5 返工完成 @ebd0d05ee（§9.13）：R5-02 闭合——break/continue 状态按目标 LoopId 单独保存（loop_exits），由目标 AwaitLoop 与 body 自然完出口 dead/backing 并集合流；return 仍只指向函数；if join 不回退 R4-03。p04/p05 双腿拒 owned-scope-escape、c02 存活 owner 合法。历史：r4 返工完成 @42b0ba7da（§9.11）：R4-01/03/04/05 验证器侧闭合——块出口寿命入状态（DeadCause::ScopeExit，break/continue 同）、join 只合并真正进入的出口（incoming+implicit，终止臂不贡献 continuation）、join 一致性只覆盖本帧入口 owner（entry_live）、参数对排他约束 fixpoint 传播（verify_text 驱动）。历史：r2+r3 返工 @e81d649ac+@987172981（QA-01..07 + R3-01/02/04/05——实际 WalkState join/alias_reach/escape_reach 分离/派生需求清理义务/adapter move-Str 声明） | 01/02/03/04/05 |
| [x] | T-03 | work | T-01/02 | ✅ r2 返工完成 @e81d649ac：runtime/text/Cargo.lock git add -f 入库（QA-08）；gate check/test --locked 与 20/20 全绿在含锁提交上重跑；gate hashes 步骤现记录其指纹 | 02/03/06 |
| [x] | T-04 | work | T-02 | ✅ r6 返工完成 @643a1b3eb：r6 重放 source 腿 + r5/r4/r3/r2 source 腿在整合树上全保持；R6-03 整合后 text/record 域隔离经 record_hir/record_source/record_native 全族 + text 全族互证（312+9 全绿）。历史：R6 重开前的 r5 返工完成 @ebd0d05ee：词法/解析零改动（两项均为验证器/lowering 层）；r5 重放 source 腿 + r4/r3/r2 source 腿全保持。历史：r4 返工完成 @42b0ba7da：词法/解析零改动（五项均为验证器/lowering 层）；r4 重放 source 腿 + r3/r2 source 腿全保持。历史：r2/r3 source 腿重放 + q01/q02/c01/c02 source 腿 + adapter eq/ne 与 move-Str 参数声明 | 02/04/05 |
| [x] | T-05 | work | T-03/04 | ✅ r6 返工完成 @7e0b9f712+@643a1b3eb：R6-03 整合——master（771）merge 进同一实施树，ParamMode 统一为 View/MutBorrow/MoveOwn、MethodOut 拆分 RecordMethodOut、CallFrame 单一 Box 字段、check_profile_ops 三域统一防线、词法 mut 关键字双面接受、surface_is_scalar_or_wider 覆盖 Scalar|Text|Record、mut-param 门接受 text。历史：R6 重开前的 r5 返工完成 @ebd0d05ee：R5-01 闭合——短路 RHS 临时边界由 LogicalLhs 步骤在 LHS 完成后冻结（LHS 区临时保留真实指针、语句尾无条件释放；嵌套 Logical 组合正确）。p01/p02 唯一 FREE、p03 两枚 LIFO、mark 先于 FREE/4096 保持。历史：r4 返工完成 @42b0ba7da：R4-02 lowering 闭合——短路 RHS 拥有临时经 join 块参数携带（skip 边 null）+ 语句尾逐临时 null 守卫释放；两条入边齐备后才 seal；跳过零分配零 FREE/执行一次配对/mark 先于 FREE/4096 探测帧保持。历史：语句尾寿命合同 + enable_probestack/inline @987172981 | 01..05 |
| [x] | T-06 | work | T-05 | ✅ r9 返工完成：联合门禁 14/14 双收据——代码提交 e60b6eacc（gate-20261015-003622）+ 最终 HEAD 7a73cf0b6（gate-20261015-004319，docs-only delta）；ac-core **350 passed+1 ignored**（r9 净增 text_review_r9 11）；runtime 20/20；零警告 fmt clean。历史交付：** ✅ r8 返工完成：联合门禁 14/14 双收据——代码提交 7468c553e（gate-20261014-230536）+ 最终 HEAD 645054332（gate-20261014-231354，docs-only delta）；ac-core **339 passed+1 ignored**（r8 净增 text_review_r8 8）；runtime 20/20；零警告 fmt clean。历史交付：** ✅ r7 返工完成：联合门禁 14/14 双收据——代码提交 314cbbeb6（gate-20261014-202410）+ 最终 HEAD（gate-20261014-203110，docs-only delta；record gate 实跑 ~201-204s）；ac-core 整合套件 **331 passed+1 ignored**（r7 净增 text_review_r7 10）；runtime 20/20；零警告 fmt clean。历史：R7 重开前的 r6 返工完成：联合门禁 14/14 双收据——代码树 gate-20261014-193430 + 最终 HEAD e01997dc5 gate-20261014-194300（record gate 实跑 ~204s）；record gate 独立 24/24；ac-core 整合套件 **321 passed+1 ignored**（r6 净增 9；771 record 族与 text 全族互证）；runtime 20/20；零警告 fmt clean。历史：R6 重开前的 r5 返工完成：gate 双收据 13/13——代码提交 ebd0d05ee（gate-20261014-175221）+最终 HEAD a14f1470b（gate-20261014-175631，docs-only delta），ac-core **255/255+1 ignored**（净增 text_review_r5 11）+runtime 20/20+三旧 gates+scope 隔离。历史：r4 返工完成：gate 双收据 13/13——代码提交 42b0ba7da（gate-20261014-171125）+最终 HEAD 1f8027500（gate-20261014-171634，docs-only delta），ac-core **244/244+1 ignored**（净增 text_review_r4 17）+runtime 20/20+三旧 gates+scope 隔离；其间一次 gate 失败（170638）为 source_native receipt-missing 瞬态窗口（r2/r3/r4 复审同族，零代码变化串行复跑通过，原文保留） | 06 |
| [x] | T-06 | work | T-05 | ✅ r9 返工完成：联合门禁 14/14 双收据——代码提交 e60b6eacc（gate-20261015-003622）+ 最终 HEAD 7a73cf0b6（gate-20261015-004319，docs-only delta）；ac-core **350 passed+1 ignored**（r9 净增 text_review_r9 11）；runtime 20/20；零警告 fmt clean。历史交付：** ✅ r8 返工完成 @645054332：verification.md 增 r8 交付/重放集/闭合行并重绑收据 7468c553e；SD 冻结提案保持（完成声明待第九轮 fresh review 清零后沉淀）。历史交付：** ✅ r7 返工完成：verification.md 增 r7 交付/重放集/闭合行并重绑收据 314cbbeb6；SD 冻结提案保持（完成声明待第八轮 fresh review 清零后沉淀）。历史：R7 重开前的 r6 返工完成 @e01997dc5：verification.md 增 r6 交付/重放集/闭合行、联合门禁行并更正复审指出的 §2 陈旧收据表述（155448 标注为 r3 历史收据）；SD-02/04 before/after 按合 771 后 canonical 刷新（三域现状/record native 首批/域隔离不承诺组合 profile）。历史：R6 重开前的 r5 返工完成 @a14f1470b：verification.md 增 r5 交付/重放集/闭合行并重绑收据 ebd0d05ee；SD 冻结提案保持（完成声明待第六轮 fresh review 清零后沉淀）。历史：r4 返工完成 @1f8027500：verification.md 增 r4 闭合行/交付行/重放集行并重绑收据 42b0ba7da；SD 冻结提案保持（资源/验证完成声明待第五轮 fresh review 清零后沉淀） | 06 |
| [ ] | T-08 | review | T-01..07完成、writes暂停 | 同树独立AC/SD/反例/生命周期审计，pass才reviewed，失败返work | 07 |
| [ ] | T-09 | merge | T-08pass与合入授权 | 明确两分支实际顺序；只消费新基线经work重跑联合回归且重新review的fresh凭证；SD/ledger/index/归档/计数/批回归到期判定；guard清理 | 08 |

worktree `D:/autostack/.wt/lang-770/auto-lang`、branch `plan-770-dev`，2026-10-14 已建组（基面 master@e849e06f9，含 auto-down 兄弟 detached；前置 772/774 均已复审合入归档，用户指令"实施它"=r2 批准）。r3 返工曾交接 work 完成 7/9（历史证据 @6dbbf2712..@92c14fe6c）；第四轮 final needs_fix 重开为 2/9 后，r4 返工 @42b0ba7da+@1f8027500 恢复 7/9；第五轮重开 2/9 后 r5 返工恢复 7/9；第六轮重开 2/9 后 r6 返工恢复 7/9；第七轮重开 4/9 后 r7 返工恢复 7/9；第八轮重开 4/9 后 r8 返工恢复 7/9；第九轮 final needs_fix（R9-01 单项 P2）重开为 4/9 后，r9 返工 @e60b6eacc（代码+测试）+@7a73cf0b6（收据）恢复 **7/9**（T-01..07 完成，T-08/T09 未动）；下表描述绑定各轮提交。受审代码/基线变化时回同一树work重跑受影响gates、重新独立review；不能把实现/联合正确性推迟到T09。

## 9. 复审记录

以下2026-10-12记录使用当时编号，作为历史保留；现行执行入口以§8及2026-10-13记录为准。

2026-10-12，stage:new，revision1，outcome:pass（有依赖门的草案preflight）。新增T01..09/AC01..08/SD01..05/V01..10。
取号中央脚本串行、ID唯一；未实施、未建树、未跑cargo/PE，表中案例均待物化。支持决定依赖769，不允许跳过该门。
next：批准r1且769/767实际收口后work；可与771并行，组合能力另立合同。

规划期间主检出由`897eb34911a4f496a28ba7cdfd8a82815fa043d1`推进至`540dc7d30839eb7df8e68b4d19472358e170fe5b`，提交仅768计划与取号簿记；实现路径零差异。本计划按最终观察HEAD与文件hash绑定，保留已有768 executing/WIP；它不是本批实施前置。

2026-10-13，stage:new，revision2，outcome:pass（引用/交接核查，非实施完成）。770/771编号不变，前置改为现行774与已归档772；更新活跃链接/取材/拟建路径，保留历史附件与既有脚本路径。Task/AC/Case/SD ID、支持与测试语义不变，status仍drafting、current_step仍0。策略队列见[后端演进§7.5](../design/strategy/auto-native-backend-evolution.md#75-现行acacc队列与编号映射2026-10-13)。[核查收据](attachments/774/reference-update-20261013.md)包含当前路径/链接/指纹。共同774完成并复审合入后才放行770/771的依赖实施；本次未建树/实施/合入/运行Cargo。

### 9.1 工作进行中记录（2026-10-14，T-04 半程状态，供续作精确接手）

- 已完成并提交：T-01 @6dbbf2712（并入同提交的 T-02）、T-03 @e03bb725a、T-04 增量1 @860c33dec
  （SourceProfile::Text + is_text() 门控、tokenize_with 文本词法、compile_text_source 管线入口、
  main.rs 分派；core/scalar 词法逐位保留）。
- T-04 剩余（下一步精确入口）：
  1. ast.rs：TypeWord 增 Str/StrOwned/Char；Param 增 mode（Default/Mut/Move）；Arg 增 moved 形态；
     ExprKind 增 StrLit(String)/CharLit(u32)/Method{recv,name,args}（char.from_codepoint 的接收者
     是 Var(Ident "char")，typecheck 特判）；.to(str|Str) 的参数用 ExprKind::TypeWordAst(TypeWord)。
  2. parser.rs：type_word/param_type/返回类型匹配处接受 str/Str/Char（仅 is_text()，旧 profile
     保持 unsupported）；fn_item 参数循环开头吃 Reserved(mut|move) 作 mode 词（mode 在名字前，
     旧语法 fn f(mut x int)）；expr() 主循环 Tok::StrLit/CharLit 直接入 vals（非 text 不可达）；
     运算符环 P::Dot 后缀：is_text() 且下一 token 是 Ident/Reserved(to/move) → 包 vals 栈顶为
     Method 节点（.move 无括号、其余必须括号）；`.to` 括号内单个类型词。
  3. resolve.rs：VT 增 Str/StrOwned/Char（as_str 对应）；vt_of_word 扩展；局部与 sig 注册带文本
     类型与 param mode（LocalDecl 增 mode）；RExprKind 增 StrLit/CharLit/Method（Method 在 resolve
     仅透传 name/args，类型绑定留 typecheck——或直接在 resolve 由接收者 VT 推导，二选一，倾向
     resolve 推导保持 typecheck 纯规则）；字面量语境定型：str 语境→VT::Str、Str 语境→VT::StrOwned
     （提升）、无注解 let 的字面量默认 str（借用语境）。
  4. typecheck.rs：文本操作数规则（len/len_bytes/char_at/slice/concat/clone/to/codepoint 的
     参数与结果 VT）；eq/ne 同型扩到文本/char；+ 在双文本上→concat（VT::StrOwned）。
  5. adapter.rs：EKind 增文本节点 + 发射对应 atom expr 节点（str_lit 的 text 字段用 atom 转义、
     char_lit 的 scalar u32）；local 的 mode 字段（mut/move）；call binding moved:true（Method
     name=move 且 recv=Var 的 arg 形态）；`.to(str)`→text_to_borrowed、`.to(Str)`→text_to_owned。
  6. tests/text_source.rs + fixtures/text-source/{valid,invalid}：V-02/V-03 source 腿
     （A中😀 len/slice、e+重音、NUL、空串、码点往返、非法字面量码、CRLF 跨行、遮蔽、深链 4096）。
  7. check-source CLI --source-profile 枚举验证（main.rs parse_source_args 的 profile 白名单 +
     usage 文案加 TEXT_SOURCE_PROFILE）。
- T-05 入口备忘（T-04 后更新）：native.rs eval_expr 的文本占位 unreachable 在 T-02 已铺（两处）。
  (ptr,len) 双变量表示；exit frees 从 checked.text_resources() 消费（键 (block, Some(idx))/(block,None)）；
  Str 返回=ABI-C（RAX+out_len 显式出参）、str 返回=双返回值（RAX/RDX）；负状态→runtime 已打 RT-FAIL→
  生成代码 ExitProcess(70)；运行时符号 ac_rt_* 按需 import（镜像 intrinsic 机制）；requires 需在
  adapter 发射（文本 expr>0 时 requires: ["ac.runtime.text.v1"]，仿 TRACE_CAPABILITY 的
  intrinsic_first_use 模式）；finish_build 校验合同（码 design.md §3.5：runtime.contract-missing/
  hash/target/api）+ support-lib 携带 lib+lib.contract.json（T-03 contract.rs 产出）。
  Vt/native 的 value_of_type_use 文本臂现返回 None——text 域入口/调用签名需扩 Vt 或旁路。
  tests：text_native.rs（真实PE+runtime lib链接）、text_resources.rs（fault/budget PE面）。

### 9.2 T-05 进行中（2026-10-14，requires 已发射 @3e4b8620b，lowering 设计已定）

- 已完成：adapter 对含文本 ops 的 text 模块发射 `requires: ["ac.runtime.text.v1"]`
  （FnPlan.has_text_ops，TextOut/StrLit/CharLit 三处置位；check-source 收据已列能力）。
- 剩余（精确设计，直接按此实现）：
  1. **native.rs 分派**：lower_object 开头 `if checked.domain()==Text { return lower_text_object(...) }`
     ——独立文本 lowerer，不动 scalar 路径（零回归）。文件锚点：src/native.rs:204。
  2. **签名展开** `cranelift_sig_text(params,result)`：str/Str 形参 → (I64,I32) 双参；
     Str 结果 → 返回 I64 + **签名尾部追加一个 I32* 显式出参**（ABI-C，774 冻结）；
     str 结果 → 双返回值 (I64,I32)；char → I32；i32/bool → I32。find_entry 的入口规则不变
     （零参 i32）；调用点的实参/返回按同展开。value_of_type_use 文本臂返回 None 的地方，
     文本路径用独立 VT2 枚举（I32/Bool/Char/StrBorrow/StrOwn）。
  3. **表示**：str/Str 局部 = 双 Cranelift 变量（ptr I64 + len I32），char/i32/bool 单 I32。
     vars2: BTreeMap<u32,(Variable,Variable)>。参数映射按展开签名顺序喂 block params。
  4. **eval_text**：迭代 spine 镜像 eval_expr，Val 枚举 {I(Value), T(Value,Value)}：
     - StrLit：rodata（om.declare_data + DataDescription，emit字节+对齐1）→ (addr, len)；
       若 expr.ty 为 Str（提升语境）→ 紧接 own_from_lit 调用（AP-4）。
     - CharLit → I(iconst scalar)。
     - TextLen/LenBytes/CharAt/Slice/Eq/Ne → ac_rt_* import 按需 declare（BTreeMap<&str,FuncRef>
       首用声明，签名同 runtime/text/src/lib.rs SHIM_SIGS）；负状态（Slice/Eq 出参或返回负值）
       → ExitProcess(70)（RT-FAIL 已由 runtime 打）。
     - TextConcat/Clone/ToOwned → ABI-C：栈槽 *mut I32 出参（create_sized_stack_slot 4），
       ptr=RAX 返回；null+负码 → ExitProcess(70)；成功 (ptr,len)。
     - ToBorrowed → 透传接收者 (ptr,len)。
     - CharToCodepoint/FromCodepoint → 直接调用（FromCodepoint 负值→70）。
     - Binary Add（文本 vt）→ 已是 text_concat 节点；Eq/Ne 文本 → ac_rt_text_eq/ne。
     - Logical/Unary/标量 Binary → 沿用原逻辑（复制或共享辅助函数）。
     - Call → 展开签名：文本实参双值入参；Str 结果= ABI-C 出参槽 + ptr 返回；str 结果=双返回。
       moved 实参（binding.moved）→ 转移（不再由本帧清理，计划已排除）。
  5. **清理**：emit_block 语句环，每个语句位置查 plan.exit_frees[(bid,Some(idx))]；
     块自然完（idx==len）查 (bid,None)； frees 为 LIFO 局部 id 序 → 逐个 ac_rt_own_free(ptr)。
     return 的 Str 值：若 owned_returns 含 (bid,idx) → 先取值后按序 frees 再 return_；
     str 返回双值。Drop 语句 → 单 free。
  6. **finish_build**（main.rs:380）：checked.required_capabilities 含 ac.runtime.text.v1 时：
     a) capability.missing 已由 capabilities_check 覆盖（caller 须 --capability 提供）；
     b) 在 support_libs 中恰有一个伴随 `<stem>.contract.json` 的 .lib：读取 JSON，校验
        api=="ac.runtime.text.v1"、api_revision==1、target=="x86_64-pc-windows-msvc"、
        lib_sha256==实际文件 SHA256（实现一个 sha256 或复用 runtime/text/tests/contract.rs
        的实现——移到共享位置或 main 内嵌副本）；失败码：runtime.contract-missing/-hash/-target/-api
        （design.md §3.5）。收据 receipt_extra 记录 lib 路径+合同 sha。全部通过才进入发射。
  7. **tests/text_native.rs**：build-source 真实 PE（--support-lib runtime lib + --capability），
     用 ac-core 的 link/run 执行器（仿 scalar_native.rs 的形态）：V-02 Unicode len/slice/eq、
     V-03 码点往返、V-04 SUMMARY 计数（通过 stderr 捕获 ac_rt_summary? —— 语言面不调 summary；
     计数验证改用 text_resources.rs 的 CLI 面：trap=70+RT-FAIL 行）、V-07 mark_a/mark_b 顺序
     （trace capability 与 runtime capability 并列 requires）、V-09 受控 PE 深链。
     **tests/text_resources.rs**：CLI 面——slice 越界→exit 70+stderr RT-FAIL code=-10；
     预算/故障注入需要 runtime 控制 API…… 语言面无 set_budget 入口：首批 V-05 PE 面以
     "故障=RT-FAIL+70 可观察"为 oracle（注入面归 runtime 测试已覆盖），记录此边界。
  8. **T-06**：scripts/verify-ac-text-770.ps1（镜像 verify-ac-data-resources-774.ps1 结构：
     cargo check/test/fmt 三工作区 + 旧 gates 741/745/772 + acc_inventory 核对）。
  9. **T-07**：README 段落 + reports/770-text/{verification,proposed-spec-delta}.md。

2026-10-14，stage:work，plan_revision 2，outcome:继续中（T-01..04 完成提交，T-05 完成 1/3），
worktree D:/autostack/.wt/lang-770/auto-lang、branch plan-770-dev、base e849e06f9、
code_commits 6dbbf2712/e03bb725a/860c33dec/6cb5ec305/c357efdb4/3e4b8620b、
dependency_revisions runtime/text@e03bb725a（合同 sidecar 自产）。
证据：text_hir 16/16（资源计划精确断言）、text_source 10/10（V-02/03/08/09 source 腿含 4096 深链）、
runtime/text 19 测试 8 二进制全绿（V-04/05 面+trap/hang+contract）、ac-core 全套 18 二进制零回归、
774 探针 65/65 基线复跑。blockers 无；next=work 续 T-05（§9.2 设计已冻结：lower_text_object/
finish_build 合同门/text_native+text_resources 测试）→T-06 gates→T-07 docs→T-08 review。

### 9.3 work 完成交接（2026-10-14，T-01..07 全部完成，ready for review）

2026-10-14，stage:work，plan_revision 2，outcome:pass（work-owned T-01..07 完成提交），
worktree D:/autostack/.wt/lang-770/auto-lang、branch plan-770-dev、base e849e06f9、
code_commits 6dbbf2712/e03bb725a/860c33dec/6cb5ec305/c357efdb4/3e4b8620b/53e8ac7c4/a87de0bd7、
dependency_revisions runtime/text@e03bb725a（合同 sidecar 自产；T-06 复跑 20/20）。

**AC 自检（work-owned AC-01..06，均绑定现行测试入口断言）**：
- AC-01：text_hir（域门/换壳拒/资源计划断言）+ text_binding + v10 隔离腿（无文本 ops→零能力）。
- AC-02：text_native v02_*/v03_*（Unicode/码点 source+直 HIR 真实 PE；invalid scalar trap70+RT-FAIL -8）。
- AC-03：text_native v04_*（LIT/CONCAT/CLONE/FREE 逐 id 配对、LIFO、.move 转移、跨帧 ABI-C 拥有返回、
  视图零释放）+ text_resources（失败类矩阵、单次释放、零长不触分配器）+ runtime 测试族（注入/预算/trap/hang）。
- AC-04：text_hir/text_binding 程序化 Bundle 变异门 + v10 合同变异门（hash/api/revision/target/缺失/多载体
  全受控拒绝，无后端 panic）。
- AC-05：v07 顺序腿（位置/命名映射/短路跳过侧零分配零 trace）、text_source 静态腿（非法字面量码/CRLF/遮蔽）、
  v09 三条 4096 规模腿（链/资源循环/调用链，全 32 位 exit）。
- AC-06：verify-ac-text-770.ps1 13 步全 PASS（收据 gate-20261014-115250.txt @53e8ac7c4；三旧 gates 实跑；
  scope 隔离=crates/test 零改动；SD 冻结于 proposed-spec-delta.md 未发布 canonical）。

**开放 in-contract 发现：无**（合同内缺口清零；§10.4 为记录性边界非缺口）。
**受影响既有例重放**：ac-core 199/199（旧面逐位保持）+ runtime 20/20 + 三旧 gates PASS。
**实现期发现并修复**（详见 verification.md §4）：共享出参槽（4096 链帧溢出 guard page）、
合同数字字段冒号解析、sha256 常量表 K[59] 复制笔误。
blockers 无；next_responsible_role=review（T-08 同树独立审计 AC-01..08/SD/遗漏债务/生命周期，
writes 已暂停，worktree 冻结在 a87de0bd7）。

### 9.4 复审记录（2026-10-14，final，outcome: needs_fix）

2026-10-14，stage: review，review_scope: final，reviewer_identity/limitations:
本会话独立复审（无另开 final-reviewer 会话；自 artifacts + 独立复跑重建裁决，
不采信 executor §9.3 自检摘要为唯一证据）。
plan_id: PLAN-770，plan_revision: 2，outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-770/auto-lang`，branch: `plan-770-dev`，
reviewed_commit: `a87de0bd761df67fd7ad4b23e4a79febdd109dca`，base_commit: `e849e06f976816dda3db0c68a520d485a69904a7`。
dependency_revisions: runtime/text@e03bb725a（合同 sidecar 自产；本复审 SHA 对拍一致）。
实现范围隔离：`git diff e849e06f9..HEAD --name-only` 46 文件全在
`experimental/ac-core/**` + `scripts/verify-ac-text-770.ps1` + `docs/reports/770-text/**`；
零 `crates/` `packages/` `docs/specs/` 触碰（strict acc_inventory 前提成立）。
受审源文件 SHA256 与 work 门禁收据 `gate-20261014-115250.txt` 逐项一致
（native.rs/main.rs/text_resources.rs/adapter.rs/mod.rs/text_native.rs/
text_resources.rs/runtime shim+lib+Cargo.toml+lock/n01.atom/gate-script）。

**spec_inputs**：`docs/reports/770-text/proposed-spec-delta.md`
sha256=`3AA5C11FA4D3322D1634E650686F3549C8445E446ABDF3959C8E7DE889925310`
（提案，未发布 canonical）；design.md sha256=`59A5616EFFAC3770212960CBFB6CBB224C7E63823B500B9DC949140F17043BB1`；
verification.md sha256=`6CD9B3F6375F90243C7C09A7150B5B73552054078AE084AB665B38CA7CFCCB33`。
SD-01..05 目标路径均为现存/拟建 `docs/specs/...` 真实位置；before/after 与现行
canonical（auto-hir/project.md 双域、auto-ac/project.md 774 合同+实现层 open、
auto-acc MD-406/407/408）方向一致。**F-04**：frontmatter `touched_goals: []`
缺书面解释（skill 要求 empty impact 写明理由）——非阻塞，merge 前补一行
「本计划不改 GOAL-*；不关闭 GOAL-017」即可。

**acceptance_results**（AC ID → 证据；均以本复审独立复跑为准）：

| AC | 结果 | 命令/检视 | 证据 |
|---|---|---|---|
| AC-01 身份/隔离 | pass | text_hir 16/16（域门/换壳拒/identity-conflict）+ text_binding 11/11 + v10_scalar_surface_needs_no_runtime | 全绿；writer 换头 `canonical.identity-conflict` 实测 |
| AC-02 Unicode/char | pass | text_native v02_*/v03_* + text_source v02/v03；`A中😀` len=3、组合符 len=2、NUL 不截断、码点 U+10FFFF/代理/-1 | 真实 PE + source 腿全绿 |
| AC-03 资源寿命 | pass | text_native v04_*（LIT/CONCAT/CLONE/FREE 逐 id、LIFO、.move 转移、跨帧 ABI-C）+ text_resources v05_* + runtime 20/20（注入/预算/trap） | 无双 free/UAF；失败类 RT-FAIL 精确行 |
| AC-04 可信变异 | pass | text_hir mutation_* + text_binding 程序化 Bundle + v10_contract_gate_rejections（hash/api/revision/target/缺失/多载体） | 真实 verifier 拒；无后端 panic |
| AC-05 顺序/静态/规模 | **partial** | v07（顺序/短路行为正确）+ text_source v08 + v09 三条 4096 | **V-07 oracle 未按合同**——见 F-01 |
| AC-06 集成/SD/边界 | pass（带 F-02/F-04 注） | 本复审复跑 `verify-ac-text-770.ps1` 13/13 PASS @a87de0bd7（收据 gate-20261014-121834.txt）；旧 gates 741/745/767 实跑；scope 隔离；库/合同/源 hash | 含 check/test/fmt ×2 + 三旧 gates |
| AC-07 review | 本记录 | 独立 AC/SD/遗漏债务审计 | 缺口未清零 → needs_fix |
| AC-08 merge | pending | — | 合入授权后由 T-09 消费 fresh 凭证 |

**pending_closeout_gates**：T-09 merge 时须在**新基线**上由 work 重跑
`scripts/verify-ac-text-770.ps1`（或等价 13 步）并重新独立 review；
批回归到期判定归 `/auto-plan:regress`（非本计划 per-plan 门禁）；
SD 沉淀/ledger/index/归档/计数/guard 清理归 merge。

**findings**：

| ID | 严重度 | 类型 | 影响 AC/Task | 描述与证据 | 纠正 |
|---|---|---|---|---|---|
| **F-01** | **blocking** | original requirement gap / unapproved scope reduction | AC-05 / V-07 / T-05 | 合同 V-07 冻结 oracle=`mark_a/mark_b` 包装的 concat/实参/比较/短路 trace（§6 表；design.md §10 仍写 `tests/text_native.rs（mark_a/mark_b trace）`）。实现 `text_native::v07_arg_order_and_short_circuit_with_text` 改用文本运行时自身 CONCAT/TEXT-SCALARS/FREE 行序。根因：trace 支持库与 runtime/text 皆 Rust no_std staticlib，各带 `__rustc::rust_begin_unwind`，共链 duplicate symbol（rust-lld 硬错）。§10.4(a) 自记「记录性边界（非缺口）」——**未批准的缩围/合同变更**，plan_revision 仍为 2，不得以 debt 登记替代完成 | (A) 任一侧去符号化/弱化 `rust_begin_unwind`（或合库），补 mark_a/b×text PE 面并恢复冻结 oracle；或 (B) 正式 revision r3 批准等价 oracle（需用户确认）后重审。禁止维持现状直接 pass |
| F-02 | non-blocking | closeout defect | AC-06 | work 门禁收据绑定 HEAD `53e8ac7c4`（T-05），非最终 `a87de0bd7`（T-06/07）。缓解：13 个实现文件 SHA 与收据一致；本复审在 `a87de0bd7` 独立复跑 13/13 全绿 | 合入前用最终 HEAD 新收据覆盖 |
| F-03 | non-blocking | contract clarification | SD-02 | SD-02 before「HIR 域仅 core/scalar」对 project.md 大致成立，但 stage-contract.md 已注「text/record 域身份注册归 770/771」，774 已沉淀 data-resource-contract——before 陈述略粗 | merge 写 SD-02 after 时按现行 canonical 精确改写 before/after |
| F-04 | non-blocking | closeout defect | AC-07 metadata | `touched_goals: []` 缺书面解释 | merge 前补「不改 GOAL-*；不关闭 GOAL-017」 |

**遗漏/延后/Workaround 扫描**：除 F-01 外，V-01..V-10 其余 case 均已物化且断言具体
（冻结码 `text.invalid-escape/-scalar/char-multi/char-empty` 有确切断言；
SENTINEL 三件有 v10_rejecting_build_preserves_sentinels；合同四码有 v10 矩阵）。
§10.4(b) 预算/注入无语言入口 = §9.2 预先裁定，合法边界；§10.4(c) char_at=-10 = 冻结 ABI
（design.md §3.4 注册表一致），合法。无其它 workaround/未批准 deferral。

**evidence**：
- 本复审独立复跑：ac-core `cargo test --locked` **199/199**（18 二进制）；
  runtime/text **20/20**；两 workspace `cargo check --locked` 零警告、`cargo fmt --check` clean。
- 完整门禁 `scripts/verify-ac-text-770.ps1` **13/13 PASS** @`a87de0bd7`，
  收据 `experimental/ac-core/target/ac-verify-receipts/770/gate-20261014-121834.txt`
  （含三旧 gates 741/745/767 实跑、scope 隔离、库/合同/源 hash）。
- Spec delta 冻结副本 hash 见上 `spec_inputs`。

**next**：work（T-05 返工）——同树 `plan-770-dev` 解决 F-01（路径 A 或 B），
修正 §10.4(a) 分类与 §9.3「开放 in-contract 发现：无」陈述，补 F-04 解释；
受影响 V-07 门禁重跑；再交 fresh final review。不得 merge。

### 9.5 F-01 返工记录（2026-10-14，路径 A，ready for re-review）

2026-10-14，stage:work，plan_revision 2（未变更合同——路径 A 是合同内技术
修复，无需 r3），outcome:pass（F-01 修复 + F-02/03/04 处置完成提交），
worktree D:/autostack/.wt/lang-770/auto-lang、branch plan-770-dev、
base e849e06f9、code_commits …/c4e169cec（F-01 返工）+5061c20a9（F-02
收据引用，docs-only；前序 6dbbf2712/e03bb725a/860c33dec/6cb5ec305/
c357efdb4/3e4b8620b/53e8ac7c4/a87de0bd7）。

**F-01 修复（路径 A）**：`experimental/ac-core/test-support/marks.rs`——两个
`hir.test.trace.v1` intrinsics 以 `rustc --crate-type=lib --emit=obj -C
panic=abort` 编译为**纯 COFF 对象**：对象形态不携带 `__rustc::
rust_begin_unwind`（测试内按 needle 长度开窗自断言该属性与双 mark 导出），
与文本运行时 staticlib 共链无冲突；标量 suites 继续用原 staticlib（零改动）。
V-07 **冻结 oracle 恢复**：`text_native::v07_mark_wrapped_concat_args_
comparison_and_short_circuit`（mark_a/b 包装 concat/位置实参/命名映射/
比较/短路 + 被跳过侧零 trace 零分配；交错 stderr 精确断言）+
`v07_mark_wrapped_order_direct_hir_leg`（直 HIR：n02-order-marks.atom，
eval_args 顺序与 bindings 映射解耦——param0←arg1 换序绑定，位置重排会得
8 而非 5）。

**重放与门禁**：ac-core 全套 **200/200**（净增 1 个冻结 oracle 测试）、
零警告、fmt clean；runtime/text 未触碰；verify-ac-text-770.ps1 于
c4e169cec 全量重跑 13/13 PASS（收据 gate-20261014-124436.txt）+ 最终
HEAD 5061c20a9（仅 docs 引用更新、gate 对拍输入哈希不变）复跑 13/13 PASS
（收据 gate-20261014-124938.txt）——**F-02 处置**（两收据间 delta 为
docs-only 提交）。

**F-02**：返工后 gate 收据重跑于 c4e169cec（不再绑 53e8ac7c4）；
verification.md 收据引用同步更新。**F-03**：proposed-spec-delta.md SD-02
before 改为现行 canonical 的确切状态（741/767 沉淀的两域 + i32/bool 面 +
资源身份未登记）。**F-04**：frontmatter touched_goals 加书面解释（本计划
不改 GOAL-*、不关闭 GOAL-017，见 YAML 注释）；顺带修复复审编辑造成的
frontmatter 换行损坏（`current_step: 6total_steps: 9`）。

**§9.3 陈述修正**：原「开放 in-contract 发现：无」不成立（F-01 当时在案）；
返工后 F-01 关闭，现无开放 in-contract 发现；AC-05 由 partial 恢复为 pass
（冻结 oracle 双腿物化）。
blockers 无；next_responsible_role=review（fresh final review @c4e169cec，
writes 已暂停）。

### 9.6 第二轮最终独立复审（2026-10-14，needs_fix）

stage: review；review_scope: final；reviewer_identity/limitations: 用户本轮请求后的
独立审查会话，从 Plan/Specs/受审代码与复跑证据重建裁决，未参与该次实施、未创建额外 agent。
plan_id: PLAN-770；plan_revision: 2（合同未改）；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`；
reviewed_commit: `5061c20a9e63b79da0d9bb505f35c40cfb549484`；
base_commit: `e849e06f976816dda3db0c68a520d485a69904a7`。
实施树入场/结束 clean、HEAD 无漂移；当前 master 相对基面无本计划 ac-core/相关 Specs 新差异。

dependency_revisions: 774 canonical 沉淀 `930d05fae798917ce2ab3026bcfa8d0ab81713d5`；
772 修复已归档、基面已包含。Spec/依赖/配置/源 hash 见
[baseline.json](attachments/770/review-r2/baseline.json)。
spec_inputs: [Spec delta 冻结副本](attachments/770/review-r2/proposed-spec-delta.frozen.md)，
SHA256=`A34D70C4D22F20279B276D6DF14D9374B8584770B94AF7F77AC0A60114EEEF68`；
Plan 输入 SHA256=`E013E9A31C224F97D9F247AF3BD1EC6B1AEDF265EBC44E8E13CD74877A543B70`。

完整报告、AC→Task→code→evidence、原始反例/合法控制、重放命令与纠正要求：
[第二轮审查报告](attachments/770/review-r2/REVIEW.md)。

| Finding | 级别 | 影响 | 实际证据/要求 |
|---|---|---|---|
| P770-R2-QA-01 | P1 blocking | AC-01/04、T-02 | Bundle Text→Core/Scalar 确认变异后 verify_text 仍给 Checked，native panic；恢复前门 verify.domain |
| P770-R2-QA-02 | P1 blocking | AC-01/04/06、T-02/05 | 删除 runtime requires 后，未提供 capability、库无 sidecar，真实 HIR build0/PE3；需求必须来自实际调用与清理义务 |
| P770-R2-QA-03 | P1 blocking | AC-03/05、T-02/05 | return-move/if-move 双 FREE，false&&move 漏 FREE；source/HIR 同现，修完整路径清理模型 |
| P770-R2-QA-04 | P1 blocking | AC-03/04、T-02/04/05 | 默认 view Str 参数直接作为拥有返回，caller 双 FREE；分离名义类型与持有权 |
| P770-R2-QA-05 | P1 blocking | AC-03/04/05、T-02/04/05 | owned Call/literal 借用存 local 漏 FREE；合法语句内借用返回先 FREE 后消费，PE70/-7；补 provenance/完整语句寿命 |
| P770-R2-QA-06 | P2 blocking | AC-03/04、T-02/04 | 同源 mut+view 与 mut 参数视图返回均误放行；必须 verify.mut-conflict/borrow-escape 前门拒 |
| P770-R2-QA-07 | P2 blocking | AC-03/05、T-02/04 | 循环每轮新建 owned 再 move 首次也 use-after-move 误拒；区分循环内 fresh 与外部来源 |
| P770-R2-QA-08 | P2 blocking | AC-06、T-03/06 | runtime/text Cargo.lock 未提交，--locked 绿依赖本地 ignored 输入；补 lock 纳入新受审提交 |
| P770-R2-CLOSE-01 | P2 docs correction | SD/T-07 | len_bytes 的单位、trace staticlib/marks.obj 边界、SD-02 来源修正文案，不批准缩围 |

acceptance_results: AC-01 fail；AC-02 值语义受测面 pass；AC-03 fail；AC-04 fail；
AC-05 partial；AC-06 partial；AC-07 fail；AC-08 merge-owned pending。

evidence: 在受审树独立 `verify-ac-text-770.ps1` **13/13 PASS**（gate-20261014-132113.txt，
[持久收据](attachments/770/review-r2/gate-pass.txt) + gate-raw），ac-core **200 passed/1 ignored**，
runtime **20 passed**；两 workspace check 零警告、fmt clean；741/745/767 实跑通过。
17 source+19 HIR 冻结输入（8 正控制双腿均过；9 source 反例+2额外 HIR变异有具体 JSON）；
6 Bundle 变异项真实 API 重放，包含换域、无 runtime requires、owned literal view 与 bad ref 反控。
首轮 gate 的 exe 拒绝访问与 CLI 探针运行重叠；停止探针后串行完整复跑通过，未列实现回归。
没有运行根 t/tv/tf/docs_gen。

上轮 F-01/F-02/F-04 本轮关闭；F-03 来源文字并入 CLOSE-01。
F-01 的 source/HIR marks oracle 全 suite 实跑，再补 c07 文本 concat 操作数与 c08 文本比较
操作数的 marks 包装双腿，结果/trace 精确符合；没有把本轮新资源缺口伪装成旧 F-01 未修。
遗漏/延后/Workaround 扫描：无产品扩围/ACC Gen1 冒称；预算注入与 -10 边界保持原裁定。
本轮 8 项为合同内缺口，已登记 KNOWN-DEBT-AND-RISKS 仅追踪，不批准豁免/延后到 merge。

pending_closeout_gates: T-09 merge/archive/SD沉淀/ledger/index/实际计数/批回归到期判定/guard清理，
仅在修复后新 fresh final pass 再消费，不替代 implementation/review 义务。

next: **work 同树返工**；T-02/03(lock)/04/05/06/07 重新打开，T-01 保留完成，
status=executing、current_step=1/9。plan_revision 不因进度/本审查记录改变。
本轮是第二次 final needs_fix：下一次 ready-for-review 前先完成诊断/覆盖设计调整与 finding
closure 表（为什么前两轮漏测、五类来源/控制流完整模型、永久 oracle、剩余缺口），
重放受影响历史与本轮共享反例/控制，提交后完整 gate，再交第三轮独立 review。不得 merge。

### 9.7 r2 返工记录（2026-10-14，QA-01..08 全修复，含两连败诊断）

2026-10-14，stage:work，plan_revision 2（合同未变更——全部修复为合同内实现），
outcome:pass（QA-01..08 + CLOSE-01 处置完成提交 @e81d649ac），worktree
D:/autostack/.wt/lang-770/auto-lang、branch plan-770-dev、base e849e06f9、
前序 …/5061c20a9（r1 返工终态，r2 受审基线）。

**诊断表（两连败根因——为什么前两轮漏测；work 自查，第三轮 review 核验）**：

| 发现 | 为什么前两轮没测到 | 受影响 paths/states | 修复与永久 oracle |
|---|---|---|---|
| QA-01 伪造域 | 测试只走「正确域进入正确入口」，从未构造错域 bundle 直接调 verify_text | verify_text 入口对任意 bundle | verify_text 前置域比较（verify.domain）；qa01 测试双伪造域+诚实反控 |
| QA-02 requires 可删 | 能力门语义实现为「文档声明⊆提供」，没问「内容真实需要什么」；adapter 总发声明，source 通道掩盖了程序化通道 | verify_text/capabilities_check/finish_build 三门的触发源 | 派生需求（bundle_needs_text_runtime）三处接线：verify 前门拒缺声明、capabilities 后备、finish_build 合同门；adapter 补 eq/ne 声明（否则合法 eq 程序被新门误拒）；qa02 测试（删声明 source+CLI+mut02 形态） |
| QA-03 move 责任 | T-02 资源走查按「全路径状态合流」建模（dead 取并集），把「不可再读」当成了「已释放」；测试只覆盖直线 move（v04 面），从未测分支/return/短路中的 move | text_resources 走查的 Return/If/Join/Logical/Loop 状态更新 + exit_frees 生成 | 常量条件折叠（const_truth，迭代+短路早退）剪枝可达臂；return 先应用 move 再算 frees；join 一致性（所有可达入边都移交→撤 caller 责任，否则 ownership-required 拒）；短路未知极性 move 拒；永久 oracle=p01/p02/p08/c06/c05/both-arms/dyn-branch/dyn-sc/break 矩阵（source+HIR PE 精确 trace，唯一 FREE、无 FREE-INVALID） |
| QA-04 view 参数持有权 | owned 判定按名义类型+角色，view Str 参数不在 owned 集却可经 return/let 流出 | text_resources 的 owned 集消费点（let/return） | owned let/return 对任何 ReadLocal 初始/返回拒（ownership-required）；qa04 测试（steal source+mutation 腿+move 参数反控） |
| QA-05 provenance | scanner 只认 concat/clone/to_owned 三个语法节点为 owned-producing（漏 owned Call/提升字面量）；lowering 把借用 Call 结果当无主视图、实参临时在 call 后立即释放 | scan_expr 的 owned-producing 分类 + native.rs TVal/Call/消费者释放点 | scanner 补 Call-StrOwned/owned-StrLit；lowering 改 backing 向量随值传播（视图/借用返回携带至末消费者后释放，帧恒小——语句尾全量存活模型在 4096 链实测 32KB 帧跨 guard page，已弃）；p03/p07/c03/mut01 测试（source+mutation+PE trace：free 在消费之后） |
| QA-06 mut 排他 | mut 只存了模式词，没有活跃区/别名语义；774 D-03 排他承诺未落实现 | provenance（owned_reach）与 validate_moves 的调用点检查 | mut 参数进 provenance 集；调用点 mut 实参与其它实参 reach 交集拒（mut-conflict）；返回 str 的 prov 含 mut/owned 来源拒（borrow-escape）；p06/p05/c04/derived 测试 |
| QA-07 循环 fresh | 循环预杀按「体内任意 move」无差别标记，未区分「循环外初始化的所有者」（重入危险）与「体内 fresh 所有者」（每轮新建） | Loop 语句的 moves_inside 处理 | 预杀集 retain(入口存活)；p09（每轮 LIT/FREE 配对）+ outer-move 仍拒（use-after-move）测试 |
| QA-08 锁文件 | T-03 记录「锁文件入库」但根 .gitignore:8 静默忽略；--locked 本地通过依赖未入库输入 | runtime/text/Cargo.lock 交付 | git add -f 入库（@e81d649ac 含）；gate hashes 步骤随后可见其指纹 |

**重放与门禁**：r2 冻结重放集（p01..p09+c01..c08+mut01/02）永久化为
`tests/text_review_r2.rs` 18 测试（source 腿真实 PE 精确 trace + HIR 腿=
合法 bundle 定向变异再 verify，证明两通道共享同一 verify_text 门）；
ac-core 全套 **218/218**（净增 18）、零警告、fmt clean、runtime/text 20/20；
fixture 补 requires 声明（新前门的一致性要求）。CLOSE-01 三处文档修正
（len_bytes=显式字节长度；staticlib 共链措辞+marks.obj 可用；SD-02 before
对账 PLAN-772 后 canonical）。

**收据**：代码提交 e81d649ac gate 13/13 PASS（gate-20261014-143045.txt）；
最终 HEAD 41291aa10（docs 收据引用更新，gate 对拍输入不变）gate 13/13 PASS
（gate-20261014-144407.txt）——F-02 同流程。注：其间两次 gate 失败
（143125/143510，均为 source_native::positive_corpus 的 receipt 缺失窗口）
经串行复跑全量 218/218 绿后干净重跑通过——与 r2 复审 §2 记录的同类环境
窗口（exe 文件占用/AV 扫描），非实现回归（两失败间零代码变化）。

blockers 无；next_responsible_role=review（第三轮 fresh final @41291aa10，
writes 已暂停，worktree clean）。

### 9.8 第三轮最终独立复审（2026-10-14，needs_fix）

stage: review；review_scope: final；reviewer_identity/limitations: 用户请求后的独立审查上下文，未参与本次返工、未创建额外 agent，从 Plan/Specs/代码及独立运行重建裁决。
plan_id: PLAN-770；plan_revision: 2；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`；
reviewed_commit: `41291aa10d22eb2e9a263e7aa574361fb55eafd1`；
base_commit: `e849e06f976816dda3db0c68a520d485a69904a7`。入场/结束实施树 clean、HEAD未漂移。
dependency_revisions: 774 canonical=`930d05fae798917ce2ab3026bcfa8d0ab81713d5`，772已归档且包含于基面。
spec_inputs/code/config/hash: [baseline.json](attachments/770/review-r3/baseline.json)；
[SD冻结](attachments/770/review-r3/proposed-spec-delta.frozen.md)，SHA256=`DCC0717A873EEB4D8054716FEF77858DA80E1CDD09E4B1441B94D400AA8F2307`。

完整 AC→Task→代码→证据、冻结 source/HIR、程序化变异、重放命令与纠正要求：
[第三轮审查报告](attachments/770/review-r3/REVIEW.md)。

| Finding | 级别 | AC/Task | 当前缺口 |
|---|---|---|---|
| P770-R3-01 | P1 blocking | AC03/04/05，T02/04/05/06 | 嵌套不可达move被算必移交而漏FREE；HIR条件Drop后FREE-INVALID；不可达move污染dead误拒合法读取 |
| P770-R3-02 | P2 blocking | AC03/04，T02/04/06 | 普通view参数同源mut+view reach为空，前门漏拒；独立双参数正控制仍过 |
| P770-R3-03 | P2 blocking | AC03/05、SD，T05/06/07 | 临时缓冲在完整语句内mark_a之前FREE，仍违背774 D04/design4.4；4096帧风险不能自行降低寿命合同 |
| P770-R3-04 | P1 blocking | AC01/04/06，T02/05/06 | 仅清理move参数的函数未派生runtime需求；删除requires、无capability且库无sidecar仍HIR build0/PE0 |
| P770-R3-05 | P1 blocking | AC04，T02/06 | 新runtime扫描先于引用验证，真实Bundle Eq.lhs=u32::MAX导致text_resources.rs:77索引panic |

acceptance_results: AC01 partial；AC02值语义受测面pass；AC03 fail；AC04 fail；AC05 partial；AC06 partial；AC07 fail；AC08 merge-owned pending。
evidence: 独立fresh gate **13/13 PASS**（gate-20261014-145312.txt，持久gate-pass+gate-raw）；ac-core **218 passed/1 ignored**、runtime **20 passed**，两workspace check零warning/fmt clean，741/745/767实跑。历史r2 **17source+19HIR**原字节重放符合修复后的预期；QA01域控制与QA08提交锁关闭。F01 marks保持通过。旧例closure有效，但QA02/03/05/06 full-invariant closure仍未完成，且R3-05为返工新增回归。

§9.7两连败诊断材料已核验，允许本轮入场；诊断模型尚未覆盖以上组合。遗漏/workaround扫描：当前五项为原合同内阻断，已登记KNOWN-DEBT仅追踪，不授权豁免；语句尾模型放弃并未获合同修改批准。SD路径/before/len_bytes/marks.obj/772来源/touched_goals解释有效，资源/验证完成声明待修复再沉淀；未发布canonical/live ledger。

pending_closeout_gates: T09实际合入、SD/导航/ledger/index、归档/计数、批回归到期判定、guard清理。
next: **work同树返工**；T01/T03保留，T02/04/05/06/07重新打开，status=executing、current_step=2/9，T08/T09未完成。提交后重放新旧输入/正控制、完整fresh gate与更新闭合表，再交独立final review；不得merge。plan_revision不因进度/复审记录改变。

### 9.9 r3 返工记录（2026-10-14，R3-01..05 全修复，第三轮闭合表）

2026-10-14，stage:work，plan_revision 2（合同未变更——语句尾寿命合同按原样
实现，探测帧为其工程解），outcome:pass（R3-01..05 处置完成提交 @987172981 +
docs 收据 @92c14fe6c），worktree D:/autostack/.wt/lang-770/auto-lang、
branch plan-770-dev、base e849e06f9、前序 …/41291aa10（r3 受审基线）。

**闭合表（每项发现：为何 r2 修复未闭合 / 修复 / 永久 oracle）**：

| 发现 | r2 修复为何未闭合 | 修复 | 永久 oracle |
|---|---|---|---|
| R3-01 join 责任 | r2 用 block_definite_moves 语法预扫描（子树并集）近似责任流：不可达嵌套 move 被算必移交（q01 漏释放）、可达臂 Drop 不在 move 扫描内（q04 FREE-INVALID）、不可达臂 dead 无条件合流（c01 误拒） | join 改为臂的**实际 WalkState**：常量条件剪枝可达性、终止臂不入 join、无 else 时隐式 fall-through 为携带快照的虚拟臂；每个 owned 局部在所有入边 live（保留责任）/全 dead（撤销责任）/混合（拒）三态；不可达臂状态整体丢弃（静态合法性诊断仍发射）；block_definite_moves（循环预杀）常量感知+收集 Drop | q01（LIT+FREE 一次）/q04（程序化 Drop 变异腿：恰一个 free 位点=Drop 处）/c01（读取合法 exit3）/动态单边仍拒 |
| R3-02 view 参数别名 | r2 的 reach 只给 owned/mut 参数身份，普通 view 参数 reach 为空——同源 view 实参对 mut 参数不可见 | alias_reach（任何文本参数+owned+backing 全源）驱动调用点排他与 provenance 播种；escape_reach（owned+mut 过滤；本函数 view 参数活过调用）驱动借用返回检查 | q02（relay(s,s)→mut-conflict）/c02（异源双参数 exit4 LIFO） |
| R3-03 语句尾寿命 | r2 以 4096 链帧溢出为由改用「消费者即释放」——未获合同修订批准；根因实为 cranelift 0.126 **enable_probestack 默认 false**，>4KB 帧不探测 guard page | 恢复完整语句尾模型：pending_temps 按语句尾 LIFO 释放、owning let/owned return 根转移；enable_probestack=true + probestack_strategy=inline（x64 内联探测=unroll+loop，无外部符号） | q03（mark_a 的 `a` 先于 FREE）+ 4096 链语句尾模型（4094 临时存活、探测帧、唯一 FREE 集） |
| R3-04 清理义务 | r2 派生需求只查表达式——纯 `move Str` 参数函数无文本表达式但出口发射 ac_rt_own_free | plan_needs_text_runtime：owned 局部/exit_frees 非空即需求；verify 在结构检查+plan 之后统一派生；adapter 对 move-Str 参数函数声明 requires | q05（unused(move s) 删声明→capability.missing；含 adapter 声明反控） |
| R3-05 扫描 panic | r2 把派生扫描放在结构校验前且直接索引——bad-reference 未拒先 panic | 扫描后置于结构检查/plan 之后；body_needs_text_runtime 全程 .get() 安全访问 | q06（Eq.lhs=u32::MAX 与 TextLen.operand=u32::MAX 均 verify.bad-reference，不 panic） |

**重放与门禁**：r3 冻结重放集（q01..q05+c01/c02）永久化为
`tests/text_review_r3.rs` 9 测试；r2 重放集 18 测试全部保持通过（历史 closure
不被本轮破坏）。ac-core 全套 **227/227**（净增 9）、零警告、fmt clean、
runtime/text 20/20。gate 双收据：代码提交 987172981（gate-20261014-155448.txt）
+ 最终 HEAD 92c14fe6c（gate-20261014-155918.txt，docs-only delta）均 13/13
PASS。其间一次 gate 失败（154917：trace_execution 临时 obj 文件被并发窗口
移除，单独复跑两次绿后串行重跑通过——与 r2/r3 复审记录同类环境窗口，零代码
变化间隔，非实现回归）。

blockers 无；next_responsible_role=review（第四轮 fresh final @92c14fe6c，
writes 已暂停，worktree clean）。

### 9.10 第四轮最终独立复审（2026-10-14，needs_fix）

stage: review；review_scope: final；reviewer_identity/limitations: 用户请求后的独立审查上下文，未参与本轮返工、未创建额外 agent，由合同/Specs/代码与本次运行重建裁决。
plan_id: PLAN-770；plan_revision: 2；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`；
reviewed_commit: `92c14fe6cbf1761521fb059bf41f667abeacc214`；
base_commit: `e849e06f976816dda3db0c68a520d485a69904a7`。
dependency_revisions: 772/774 已归档且在基面，774 canonical 沉淀 `930d05fae798917ce2ab3026bcfa8d0ab81713d5`；771 尚未合入。
spec_inputs/Plan/hash: [baseline.txt](attachments/770/review-r4/baseline.txt)；
[SD 冻结](attachments/770/review-r4/proposed-spec-delta.fresh.frozen.md)，SHA256=`DCC0717A873EEB4D8054716FEF77858DA80E1CDD09E4B1441B94D400AA8F2307`。
入场/结束实施树 clean，HEAD、受审代码、Spec/SD 未漂移。

完整 AC→Task→实现→证据、原例闭合、具体纠正要求见 [R4 审查报告](attachments/770/review-r4/REVIEW.md)。

| Finding | 级别 | AC/Task | 当前缺口 |
|---|---|---|---|
| P770-R4-01 | P1 blocking | AC03/04/05，T02/04/05/06 | 内层 owner 出口释放、外层 view 仍 Checked；两腿 PE 读取已释放内存后 RT-FAIL -7/70 |
| P770-R4-02 | P2 blocking | AC02/03/05，T05/06/07 | 短路 RHS 拥有临时加入全局语句尾清理，指针不支配 join；合法 check0/Bundle Checked，source/HIR build1 backend.lowering |
| P770-R4-03 | P2 blocking | AC03/04/05，T02/04/06 | return 终止臂的 dead/backing 仍合入 continuation，合法后续读取误拒 use-after-move（含嵌套） |
| P770-R4-04 | P2 blocking | AC02/03/04/05，T02/04/06 | join 比较全部函数 owner，将动态分支自有局部误算跨臂责任，合法单/双臂 owned 构造误拒 ownership-required |
| P770-R4-05 | P2 blocking | AC03/04，T02/04/06 | wrapper 两个 view 形参由 caller 传入同源时，内部 mut/view 排他约束未传播；source/HIR/Bundle 仍放行 |

acceptance_results: AC01受测面pass；AC02partial；AC03fail；AC04fail；AC05fail；AC06partial；AC07fail；AC08merge-owned pending。
evidence: 独立 fresh gate **13/13 PASS**（gate-20261014-162529.txt，持久 gate-pass/gate-raw）；ac-core **227 passed/1 ignored**、runtime **20 passed**，两 workspace check 零 warning/fmt clean，741/745/767 实跑。首次 gate 在 source_native 的 receipt missing 瞬态失败，原文保留，零代码变化完整复跑绿；未证明 AV/竞争根因，不拿此归因解释新反例。未跑根 t/tv/tf/docs_gen。

历史 r2 **17 source+19 HIR**、r3 **5 source+7 HIR** 原字节本次重放恢复修复后的预期，另 API 真实 Drop/requires/bad-reference 变异确认；R3-01..05 **旧例 closure 有效**，完整控制流/词法寿命/跨调用排他仍未闭合。新组合 **15 source+15 HIR** 和真实 Bundle API 证据持久化，含四个合法正控制；SD 路径/before/边界与空 touched_goals 解释有效，完成声明待修复再沉淀。五项均原合同内阻断，已登记 KNOWN-DEBT 仅追踪，不授权延后/缩围。

pending_closeout_gates: T09 实际合入、SD/导航/ledger/index、归档/计数、批回归到期判定、guard清理。
next: **work 同树返工**；T01/T03保留，T02/04/05/06/07重新打开，status=executing、current_step=2/9，T08/T09未完成。先交漏测原因、完整状态/来源模型调整与永久组合 oracle，再提交/重放新旧正反控/完整 fresh gate，重新独立 final review；不得 merge。plan_revision 不因簿记改变。

### 9.11 r4 返工记录（2026-10-14，P770-R4-01..05 全修复，第四轮闭合表）

2026-10-14，stage:work，plan_revision 2（合同未变更——五项均为合同内实现），
outcome:pass（P770-R4-01..05 处置完成提交 @42b0ba7da + 收据 @1f8027500），
worktree D:/autostack/.wt/lang-770/auto-lang、branch plan-770-dev、
base e849e06f9、前序 …/92c14fe6c（r4 受审基线）。

**诊断表（为何 r3 永久 oracle 漏掉这五类组合路径）**：

| 发现 | r3 永久 oracle 为何没测到 | 修复 | 永久 oracle |
|---|---|---|---|
| R4-01 作用域出口借用 | 块自然出口只安排 FREE 不进走查状态：q01/c01 测的是 live owner 的词法链与 move/drop dead，从未构造"owner 出作用域后经 backing 读取" | `DeadCause::ScopeExit`：块自然出口与 break/continue 把自身 frees 标记入出向状态；check_reads/validate_moves/Drop 的 dead 分支按 cause 区分诊断（ScopeExit→owned-scope-escape）；"非 live 非 dead"不再是合法 backing | p10（源+冻结 atom 双腿拒 owned-scope-escape）+ c04（存活 owner 合法保持）+ 循环出口组合（for 体内 owner 的 backing 出循环拒读） |
| R4-03 终止臂污染 | r3 引入 then_incoming/else_incoming 但只用于清理责任；dead/backing 状态合并仍按 then_reachable——测试只有不可达臂（q01/c01），没有"可达但终止"臂 | merge_children 只合并真正进入 join 的出口（then/else_incoming + implicit）；终止臂保留其终出口释放/转移但不贡献 continuation；嵌套时内层 join incoming 空集自然组合；join 一致性仅覆盖本帧入口 owner（entry_live 快照，同时闭合 R4-04） | p05/p06（终止 return 臂后合法读取，exit3 唯一 FREE，双腿；嵌套同判） |
| R4-04 臂内局部误责 | join 遍历函数全部 self.owned：c03 是静态 true（单 incoming，臂局部恒 live_cnt=1/1 通过）；动态双臂从未构造 | 同上 entry_live 收窄 + 臂内局部在自身作用域出口已 ScopeExit 收口（live_cnt=0 → 撤责不误拒） | p08/p09（动态单/双臂 owned 构造合法，exit0 唯一 FREE，双腿）+ r3 单边 move 拒绝保持（入口 owner 混合路径仍拒） |
| R4-05 包装排他绕过 | 调用点排他只看直接 callee 的 mut 模式：q02 是同一参数重复（形参身份即来源），两个形参由 caller 喂同源是组合形态 | 参数对排他约束 **fixpoint 传播**：种子=mut×其他参数对（恰为 r2 QA-06 规则的成对化）；包装体把交叉侧约束再出口到自己的签名（仅 view/mut 形参为载体，move 形参转移后唯一）；verify_text 驱动迭代至不动点，最终 pass 的诊断/plan 对最终约束计算 | p07（源+冻结 atom 双腿拒 mut-conflict）+ 两层 wrapper 传播组合 + c02（异源合法保持 exit4） |
| R4-02 短路 RHS 临时 | r3 恢复语句尾寿命后 RHS 临时指针入全局 pending 列表；r3 的短路测试（c06/p08）RHS 均为借用字面量，无拥有临时，SSA 支配缺口不可见 | RHS 拥有临时经 join 块参数携带（skip 边传 null），语句尾逐临时 null 守卫释放（每临时一枚菱形）；两条入边齐备后才 seal（首版 seal 过早触发 cranelift debug 断言，返工内修正）；帧尺寸不变（共享 ABI-C 槽） | p01/p03/p11（跳过侧零分配零 FREE）+ p02/p04（执行侧一次配对/双临时 LIFO 语句尾，双腿）+ 组合 oracle：mark 后于短路求值仍先于 FREE（r3 q03 语义在短路形态下保持）+ c01（借用字面量 RHS 隔离正控）+ 4096 探测帧不退化（既有 v09/q03-chain 保持） |

**闭合表（finding → 不变量 → 修复 → 当前结果）**：五项均闭合——p01..p11
反例全部达到复审要求（p10/p07 前门拒、p01..p06/p08/p09/p11 合法 build/run
真实 PE），c01..c04 正控逐字节保持预期 trace/exit；**剩余缺口：无**。
历史重放：r2 17 source + r3 5 source 冻结字节 CLI 重放符合修复后预期
（c07/c08/q03 marks 例需专用旗标，由测试套覆盖）；r2 18 + r3 9 测试全绿
（历史 closure 不被本轮破坏）。

**AC 自检（r4 返工增量）**：
- AC-02/03/05：p01..p04/p11/p08/p09 合法输入恢复（check/build/run=0 + 真实
  PE 精确 trace）；p10 UAF 前门拒；短路语句尾寿命（含 mark 先于 FREE）恢复。
- AC-03/04：p07 同源 mut/view 经包装闭包拒（双通道）；p05/p06/08/09 合法
  误拒清零；冻结 atom 直接 HIR 腿与源码腿同一 verify_text 判决。
- AC-06：gate 双收据 13/13（42b0ba7da→gate-20261014-171125；
  1f8027500→gate-20261014-171634），ac-core **244 passed+1 ignored**
  （净增 text_review_r4 17）、runtime 20/20、三旧 gates 实跑、scope 隔离、
  零警告、fmt clean。其间一次 gate 失败（170638）为 source_native 的
  `receipt missing` 瞬态窗口（r2/r3/r4 复审记录同族；零代码变化串行复跑
  通过，原文保留）。

**实现文件**：`src/text_resources.rs`（状态模型/排他传播）、`src/verify.rs`
（fixpoint 驱动）、`src/native.rs`（短路临时路径合流）、
`tests/text_review_r4.rs`（新增 17 测试）；词法/解析/runtime 零改动。

blockers 无；next_responsible_role=review（第五轮 fresh final @1f8027500，
writes 已暂停，worktree clean）。

2026-10-14，stage:work，plan_revision 2，outcome:pass（r4 返工完成，
T-01..07 全部完成=7/9），worktree_path D:/autostack/.wt/lang-770/auto-lang、
branch plan-770-dev、base_commit e849e06f9、code_commit 42b0ba7da（+docs
1f8027500）、dependency_revisions runtime/text@e03bb725a（未触碰）。
task_ids T-02/T-04/T-05/T-06/T-07（r4 重开后完成）。evidence：见上。
blockers 无。next=review（T-08 同树独立第五轮 final review）。

### 9.12 第五轮最终独立复审（2026-10-14，needs_fix）

stage: review；review_scope: final；reviewer_identity/limitations: 独立审查上下文，未参与实现，未创建额外 agent；从合同/Specs/代码及本轮实际运行重建结论。
plan_id: PLAN-770；plan_revision: 2；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`；
reviewed_commit: `1f8027500d52cc69bfe279fe652c4f454c919523`；
base_commit: `e849e06f976816dda3db0c68a520d485a69904a7`。

dependency_revisions: 772/774 已归档且在基面，771 未合入；两锁文件/runtime 输入 hash 见 [baseline-fresh.txt](attachments/770/review-r5/baseline-fresh.txt)。主检出期间有其他门禁修复合入，相关六份 canonical 与实施树逐项 hash 相同；实施 HEAD/clean 未漂移。SD 冻结 SHA256=`DCC0717A873EEB4D8054716FEF77858DA80E1CDD09E4B1441B94D400AA8F2307`，未沉淀 canonical/ledger。

完整 AC→Task→实现→证据、反例/正控、原例与完整不变量闭合区别、纠正要求见 [R5 审查报告](attachments/770/review-r5/REVIEW.md)。

| Finding | 级别 | AC/Task | 当前缺口 |
|---|---|---|---|
| P770-R5-01 | P1 blocking | AC03/05，T05/06/07 | RHS 临时边界在求值 LHS 前冻结，把已执行 LHS 的临时也改成 skip 边 null；&&/||/嵌套双腿正常返回漏 FREE |
| P770-R5-02 | P1 blocking | AC03/04/05，T02/04/05/06/07 | if continuation 丢弃终止臂的 backing/dead，loop 未另存 break/continue 目标出口；两腿与 Bundle 放行悬空借用，FREE 后读内存，exit70/-7 或偶然 exit3 |

acceptance_results: AC01受测面pass；AC02受测操作面pass；AC03fail；AC04fail；AC05fail；AC06partial；AC07fail；AC08merge-owned pending。
evidence: fresh gate **13/13 PASS**（gate-20261014-172651，持久 gate-fresh-pass/gate-fresh-raw），ac-core **244 passed/1 ignored**、runtime **20 passed**，两 workspace check 无 warning/fmt clean，741/745/767 实跑。未跑根 t/tv/tf/docs_gen。新增 **9 source+9 HIR** 与真实 Bundle API，历史 R4 **15+15**、R2 **17+19**、R3 **5+7** 原字节 fresh 重放；marks/Drop/requires/bad-reference/4096 的既有断言经本次全套重跑并核对。R4-01..05 旧例闭合有效，但 R4-02 的新 skip-side 回归与 R4-01/03 的目标出口完整不变量仍有缺口。命名实参 wrapper 同源拒、异源合法控制保持；早期附件两例错误命名拼写已改成实际 `:` 语法重跑，旧输出不充当有效证据。

SD 路径/before/边界、空 touched_goals 解释有效；完成声明须随上述实现闭合后再定稿。两项均原合同内阻断，登记 KNOWN-DEBT 仅追踪，不授权延后或缩围。
pending_closeout_gates: T09 实际合入、SD/导航/ledger/index、归档/计数、批回归到期判定、guard清理。
next: **work 同树返工**；T01/T03保留，T02/04/05/06/07重开，status=executing、current_step=2/9，T08/T09未完成。§8 各行既有“完成”描述是本轮入场的历史交付证据，现行状态以未完成 checkbox 与本节为准；上轮“剩余缺口：无”已被本轮反例证伪。先修模型/永久组合 oracle，提交后重放新旧正反控+完整 gate，再交 fresh final review；不得 merge。plan_revision 不因复审簿记改变。

### 9.13 r5 返工记录（2026-10-14，P770-R5-01/02 全修复，第五轮闭合表）

2026-10-14，stage:work，plan_revision 2（合同未变更——两项均为合同内实现），
outcome:pass（P770-R5-01/02 处置完成提交 @ebd0d05ee + 收据 @a14f1470b），
worktree D:/autostack/.wt/lang-770/auto-lang、branch plan-770-dev、
base e849e06f9、前序 …/1f8027500（r5 受审基线）。

**诊断表（为何 r4 修复引入/遗留这两条）**：

| 发现 | 根因 | 修复 | 永久 oracle |
|---|---|---|---|
| R5-01 跳过侧漏 LHS 释放 | r4-02 的 temps_mark 在进入整个 Logical 时记录（求值 LHS 之前），把 LHS 已分配的拥有缓冲也归入"未执行 RHS"，skip 边 null 守卫吞掉其 FREE——r4 测试的 RHS 临时都在 RHS 区内，LHS 拥有临时+跳过的组合从未构造 | LogicalLhs 步骤在 LHS 完成（含嵌套 Logical）后冻结 temps_mark 并在此处才压入 Enter(rhs)/LogicalRhs：LHS 区临时保留真实指针、语句尾无条件释放；RHS 区仍走 join 传 null；栈纪律保证嵌套组合正确 | p01/p02（&&/|| 跳过侧 LHS 唯一 FREE，双腿）/p03（嵌套外层跳过，id1/id2 全 LIFO 释放）/c01（双侧执行控制）+ 组合 oracle：mark 后于整个逻辑仍先于 LHS 临时 FREE |
| R5-02 目标出口状态丢弃 | r4-03 的 join 只保留 incoming（正确），但 break/continue 的终止臂状态被整体丢弃；r4-01 只在**循环体自然出口**标 ScopeExit——臂内 break/continue 的出口状态没有任何通道到达 loop 之后 | break/continue 在标记自身 frees 后把整份状态按目标 LoopId 存入 loop_exits；目标 loop 的 AwaitLoop resume 把 body 自然完出口与全部目标出口一并 dead/backing 并集合流（live 仍以父帧为权威）；return 出口仍只指向函数不合流；if join 行为零回退 | p04/p05（if 臂内 break/continue 后借用已释放 loop 作用域 owner，源+冻结 atom 双腿拒 owned-scope-escape）/c02（存活外层 owner 经 break 合法 exit3）+ 组合 oracle：双臂 break/continue 各自 arm-local owner 出循环拒读；r4 returning-move（p05/p06-r4）与 r2 break 矩阵保持 |

**闭合表**：两项闭合——p01..p06 反例全部达到复审要求（p04/p05 前门拒、
p01..p03 合法唯一/LIFO FREE、p06 命名实参同源拒保持），c01..c03 正控逐字节
保持预期 trace/exit；**剩余缺口：无**。历史重放：r4 冻结 15 source CLI 重放
零回归；r2 18 + r3 9 + r4 17 测试全绿（历史 closure 不被本轮破坏）。

**AC 自检（r5 返工增量）**：AC-03/05 正常路径漏释放清零（LHS/RHS 临时分区、
每分配唯一清理、mark 先于 FREE）；AC-03/04 悬空借用前门拒（目标出口状态
合流、双通道判决一致）；AC-06 gate 双收据 13/13（ebd0d05ee→
gate-20261014-175221；a14f1470b→gate-20261014-175631），ac-core
**255 passed+1 ignored**、runtime 20/20、零警告、fmt clean。

**实现文件**：`src/native.rs`（LogicalLhs 冻结边界并接管 RHS 步骤压栈）、
`src/text_resources.rs`（loop_exits 目标出口状态）、`tests/text_review_r5.rs`
（新增 11 测试，含 p04/p05/p06 冻结 atom 嵌入）；词法/解析/runtime 零改动。

blockers 无；next_responsible_role=review（第六轮 fresh final @a14f1470b，
writes 已暂停，worktree clean）。

2026-10-14，stage:work，plan_revision 2，outcome:pass（r5 返工完成，
T-01..07 全部完成=7/9），worktree_path D:/autostack/.wt/lang-770/auto-lang、
branch plan-770-dev、base_commit e849e06f9、code_commit ebd0d05ee（+docs
a14f1470b）、dependency_revisions runtime/text@e03bb725a（未触碰）。
task_ids T-02/T-04/T-05/T-06/T-07（r5 重开后完成）。evidence：见上。
blockers 无。next=review（T-08 同树独立第六轮 final review）。

### 9.14 第六轮最终独立复审（2026-10-14，needs_fix）

stage: review；review_scope: final；reviewer_identity/limitations: 本次独立审查上下文未参与返工，无额外 agent/model；由主检出合同/现行canonical、同树提交与fresh运行重建结论。
plan_id: PLAN-770；plan_revision: 2；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`；
reviewed_commit: `a14f1470b1d5e1174653e8d499091147b4257fe6`；
base_commit: `e849e06f976816dda3db0c68a520d485a69904a7`。
main observed HEAD: `6787bbd4a0257067af43845c4450f3644125af6e`。

dependency_revisions: 772/774在基面；**771已合入归档**，实现`e279ee018f58cc4e9c89f8125e37155f66fb3aad`非受审HEAD祖先。四份canonical已随771变化，774与锁未漂移；树/main分别指纹见 [baseline-hashes.txt](attachments/770/review-r6/baseline-hashes.txt)。结束实施HEAD/clean与产品未漂移见final-stability.txt。
spec_inputs: 现行主线Specs优先；SD冻结SHA256=`DCC0717A873EEB4D8054716FEF77858DA80E1CDD09E4B1441B94D400AA8F2307`，旧before不能直接沉淀。完整AC→Task→代码→证据/纠正要求见 [R6报告](attachments/770/review-r6/REVIEW.md)。

| Finding | 级别 | AC/Task | 当前缺口 |
|---|---|---|---|
| P770-R6-01 | P1 blocking | AC03/04/05，T02/04/05/06/07 | continue/自然back-edge只合流到after-loop，未重新核查重入读；上一轮owner释放后下一轮读view，Bundle Checked、source/HIR check0/build0/run70/-7 |
| P770-R6-02 | P2 blocking | AC03/04/05，T02/04/06/07 | 不可达臂也写全局loop_exits，false分支的break/continue污染循环后状态，合法静态空串读取双腿误拒owned-scope-escape；r5修复回归 |
| P770-R6-03 | P2 blocking，依赖漂移 | AC06，T02/04/05/06/07 | 771先合入后仍使用旧独立树，gate仅741/745/767无record；SD02/04现行before过时；774要求新整合基线实跑两profile回归并fresh review |

acceptance_results: AC01独立受测面pass、整合保持性未核；AC02操作面pass/合法控制受R6-02阻断；AC03fail；AC04fail；AC05fail；AC06fail；AC07fail；AC08merge-owned pending。
evidence: 本树fresh gate **13/13 PASS**（gate-20261014-181950，持久gate-pass/gate-raw），ac-core **255 passed/1 ignored**、runtime **20 passed**、两workspace check无warning/fmt clean、741/745/767实跑；未跑根t/tv/tf/docs_gen。不将旧树的绿当作771整合验收。新增 **8 source+8 HIR** 与真实Bundle API；历史R5 **9+9**、R4 **15+15**、R2 **17+19**、R3 **5+7** 原字节全重放（合计46 source+50 HIR）恢复返工后预期。R5-01原例与新增嵌套短路控制闭合；R5-02 post-loop原例拒绝闭合，完整可达性与re-entry仍有缺口。现有marks/变异/来源/Unicode/fault/4096断言均实跑并核对。
knowledge_delta: 路径/有限实验边界与touched_goals空解释有效；须对照现行record canonical更新before/after与最终整合证据，verification.md当前gate表的混旧HEAD/收据也随T07纠正。未发布canonical/ledger，未修实现或合入。
pending_closeout_gates: T09实际合入、SD/导航/ledger/index、归档/计数、批回归到期、guard清理。**实现安全与771联合正确性不是T09延后项**。
next: **work同树返工**；T01/T03历史设计/runtime完成保留，T02/04/05/06/07重开，status=executing、current_step=2/9，T08/T09未完成。先交重入/可达目标出口诊断与覆盖模型、永久新旧组合oracle，整合已落771并提交；新整合HEAD实跑text完整gate+record回归+共同旧面，更新SD/收据后fresh final review。登记KNOWN-DEBT只追踪，不授权延后/缩围；不得merge。plan_revision不因复审簿记改变。

### 9.15 r6 返工记录（2026-10-14，P770-R6-01/02/03 全处置，第六轮闭合表）

2026-10-14，stage:work，plan_revision 2（合同未变更——三项均为合同内实现；
R6-03 为既定条件触发），outcome:pass（P770-R6-01/02/03 处置完成提交
@7e0b9f712+@643a1b3eb + 收据 @e01997dc5），worktree
D:/autostack/.wt/lang-770/auto-lang、branch plan-770-dev、原基面 e849e06f9、
merge 基 6787bbd4a（master，含 771）。

**诊断表（为何 r5 修复引入/遗留这三条）**：

| 发现 | 根因 | 修复 | 永久 oracle |
|---|---|---|---|
| R6-01 重入读取 | 循环体只从入口快照走查一次：continue/自然 back-edge 状态只并入 after-loop（r5 目标出口修复的正确半边），没有反馈 loop header 再验证——重入后循环内的读取从未对照上一轮已死借用校验 | AwaitLoop 改 fixpoint 重走：re-entry 状态=入口 base ∪（自然完出口+continue 出口）的 dead/backing 合流，收敛键=dead+id 投影 backing（单调有界必收敛）；fresh owned `let` 清死记并递增构造代际（每轮重定义资源身份），backing 携 (owner,generation)——fresh let 后、视图重赋值前的读取按陈旧代际拒 | p03/p04（continue/自然 back-edge 重入读取，源+冻结 atom 双腿拒 owned-scope-escape）+ c01/c02（存活外层 owner/静态 view 重入合法）+ 组合 oracle：重入后 fresh `let` 之后的陈旧代际借用拒读（超出复审最低要求的 gen 机制验证）+ 4096 资源循环保持 |
| R6-02 不可达出口污染 | r5 的 loop_exits 在 break/continue 处无条件记录——不可达臂（常量条件剪枝）的出口状态无法被 if join 撤销（join 正确丢弃臂状态，但副作用已写入全局表） | 帧级静态可达性：RFrame.reachable 随 if 臂传播（cond_t 剪枝），不可达帧的 break/continue 不记录 loop_exits（静态类型/引用/字面量合法性检查不变，exit_frees 照旧不回退） | p01/p02（`if false` 臂内 break/continue，双腿 exit0 零分配零 FREE）+ r4/r5 可达悬空反例保持 |
| R6-03 771 先合入 | 修复收据的时间基线在 771 合入前；受审树不含 record 实现、text gate 不跑 record gate、SD before 过期 | 同一实施树 `git merge master`（6787bbd4a）：共享热点按 774 语义调和——hir::ParamMode 统一 View/MutBorrow/MoveOwn（771 Mut≡MutBorrow，AST 层保留 Default/Mut/Move 且词法 mut 关键字与保留字双形接受）；check_profile_ops 三域统一（text 面/record 面/core 算子互拒，move 仅 text、mut 仅 text+record）；resolve 的 text Method 与 record Field/MethodCall 分派链接、Task::MethodOut 拆出 RecordMethodOut；adapter 的 record 调用 moved 标志改为与 bindings 平行（record flagship 测试抓出 arg 平行误构）；surface_is_scalar_or_wider 覆盖 Scalar\|Text\|Record（r5/r6 短路与比较测试抓出 text 被排除）；mut-param 类型门接受 text `mut str/Str`；descriptor 参数模式解析 schema 可移植（record schema 无 ParamModeKind 枚举）。verify-ac-text-770.ps1 接入 verify-ac-record-771 为第四旧 gate（V-10 条件触发） | 联合门禁 14/14 双收据（gate-20261014-193430/194300，record gate 实跑 ~204s）+ record gate 独立 24/24 + 整合套件 321+1（771 record 族 60+ 测试与 text 全族在同一树互证零回归）+ SD-02/04 before/after 按现行 canonical 刷新 |

**闭合表**：三项闭合——r6 冻结集 8 source+8 HIR 在整合树上逐项符合（p01/p02
合法 exit0 零分配、p03/p04 双通道拒、c01..c04 正控 trace/exit 保持）；
**剩余缺口：无**。历史重放：r2 18/r3 9/r4 17/r5 11 测试在整合树全绿；
record 族（771）全绿。

**AC 自检（r6 增量）**：AC-03/04/05 重入 UAF 前门拒+不可达误拒清零；AC-06
联合基线 14/14（text gate 含 record gate）+record 独立 24/24+SD 刷新+收据
卫生（§2 陈旧表述更正）；AC-01/02 整合后受测面全绿。

blockers 无；next_responsible_role=review（第七轮 fresh final @e01997dc5，
writes 已暂停，worktree clean）。

2026-10-14，stage:work，plan_revision 2，outcome:pass（r6 返工完成，
T-01..07 全部完成=7/9），worktree_path D:/autostack/.wt/lang-770/auto-lang、
branch plan-770-dev、base_commit e849e06f9（merge 基 6787bbd4a）、
code_commit 7e0b9f712+643a1b3eb（+docs e01997dc5）、dependency_revisions
runtime/text@e03bb725a（未触碰）；771 整合=e279ee018 在 merge 基内。
task_ids T-02/T-04/T-05/T-06/T-07（r6 重开后完成）。evidence：见上。
blockers 无。next=review（T-08 同树独立第七轮 final review）。

### 9.16 第七轮最终独立复审（2026-10-14，needs_fix）

stage: review；review_scope: final；reviewer_identity/limitations: 当前独立审查上下文未实施r6返工，未创建额外agent；由合同/现行Specs/同树提交与fresh运行重建结论。
plan_id: PLAN-770；plan_revision: 2；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`；
reviewed_commit: `e01997dc593c1d20b3ccab045b33b167eedf4a23`；
base_commit: `6787bbd4a0257067af43845c4450f3644125af6e`（整合基线；原e849e06f9保留历史）。
dependency_revisions: 772/774在基面；771实现e279ee018为HEAD祖先，真实命令返回0；相关六份canonical/774输入与实施树相同。main起始1c7a88159、结束观察fd0835ca3f7680da6a6cfb1563e76435b906d49a，期间其他计划变化未改受审实现/相关输入。hash/结束clean见 [R7附件](attachments/770/review-r7/final-stability.txt)。
spec_inputs: baseline-hashes.txt；SD冻结SHA256=`7E29C7CCF4CC7C11125FC4F6B57A7ADEA2285B10C464B4F643D51210C2B4B00C`。
完整AC→Task→实现→证据、真实Bundle变异、纠正要求见 [R7报告](attachments/770/review-r7/REVIEW.md)。

**上轮闭合**：R6-01原continue/自然UAF两腿拒，新增跨调用/嵌套旧代际反例拒，fresh-view/嵌套合法PE保持；R6-02false臂原例+true/else控制合法；R6-03同树771整合、真实record门禁、SD02/04与收据卫生已闭合。以下为新修复回归，不抹去既有原例closure。

| Finding | 级别 | AC/Task | 当前缺口 |
|---|---|---|---|
| P770-R7-01 | P2 blocking | AC02/03/04/05，T02/06/07 | AwaitLoop无条件将body_st当自然back-edge；直接HIR只有break的循环被重走，第一条合法view读取误报owned-scope-escape；真实Bundle/CLI拒，source对应控制及同图活owner反控合法 |

acceptance_results: AC01受测面pass；AC02partial；AC03partial；AC04fail；AC05fail；AC06partial（联合门禁/SD通过，最终修复证据待重绑）；AC07fail；AC08merge-owned pending。
evidence: fresh联合gate **14/14 PASS**（gate-20261014-195755，持久gate-pass/gate-raw），ac-core **321 passed/1 ignored**、runtime **20 passed**，两workspace check无warning/fmt clean；771 gate实跑 **24/24、23 cases**、741/745/767实跑。未跑根t/tv/tf/docs_gen。新增 **7 source+9 HIR**（包括m01/m02真实变异）与BundleAPI；历史R6 **8+8**、R5 **9+9**、R4 **15+15**、R2 **17+19**、R3 **5+7** 原字节全重放（54 source+58 HIR）符合返工后预期。核对身份/变异/Unicode/fault/marks/来源/4096与record隔离/值语义断言，未跳过声明风险面。
knowledge_delta: SD路径/before/after/有限实验边界与空touched_goals解释有效；完成声明待R7-01闭合才沉淀，canonical/ledger未改。新缺口属于现有正常break/直接HIR验收，不引入新规模或full borrow checker。
pending_closeout_gates: T09实际合入、SD/导航/ledger/index、归档/计数、批回归到期、guard清理。
next: **work同树返工**；仅T02/T06/T07重开，T01/T03/T04/T05保持完成，status=executing、current_step=4/9，T08/T09未完成。显式区分自然完成/continue回边/break目标/return函数出口，合法直接HIR终止路径不得创造重入；永久化m01/m02与终止/真实回边控制，提交后重放受影响新旧正反控+联合门禁，fresh final review。登记KNOWN-DEBT仅追踪，不授权延后/缩围；不得merge。plan_revision不因簿记改变。

### 9.17 r7 返工记录（2026-10-14，P770-R7-01 修复，第七轮闭合表）

2026-10-14，stage:work，plan_revision 2（合同未变更——r6 fixpoint 的终止态
分类回归修复），outcome:pass（P770-R7-01 处置完成提交 @314cbbeb6 + 收据
docs 提交），worktree D:/autostack/.wt/lang-770/auto-lang、branch
plan-770-dev、受审基线 e01997dc5（merge 基 6787bbd4a 保留）。

**诊断与修复**：r6 的重入 fixpoint 把 body 存态无条件当回边——results 只存
WalkState 不区分终止方式，break-only/return-only 体的终止态被送回 header
重走，伪造从未发生的第二次迭代，首读被误判陈旧（m01 直接 HIR 合法程序
Err owned-scope-escape）。修复一行分类：continue 出口恒入 fixpoint；body 态
仅当 `block_terminates(body)=false`（存在自然落空路径）才入。break/return
终止体零重入贡献；真实 break 态仍入 after-loop 合流（r5 语义）、return 态
仍不合流（r4 语义）。不取消真实 back-edge 验证（R6-01 全部拒绝例保持），
不限制直接 HIR 形态。

**永久 oracle（tests/text_review_r7.rs 10 测试）**：m01/m02（复审冻结
atom 字节一致，直接 HIR break-only run0 精确 trace）、c05（源码 break-only
双腿）、return-only（终止矩阵补充：return-only 体不重走 exit3）、
p01/p02（陈旧重入经调用/嵌套循环，源+冻结 atom 双腿拒）、c01..c04
（自然/continue 刷新视图重入 exit6、true-else 不可达 continue exit0、
嵌套 fresh owner exit12）。

**闭合表**：R7-01 闭合——复审新集 7 source+9 HIR 逐项符合；**剩余缺口：
无**。历史重放：r2 18/r3 9/r4 17/r5 11/r6 9 测试全绿（历史 closure 不被
本轮破坏）。

**AC 自检（r7 增量）**：AC-02..05 直接 HIR 合法 break-only 恢复（check/
build/run=0）；AC-04 双通道判决一致；AC-06 联合门禁 14/14 双收据
（314cbbeb6→gate-20261014-202410；最终 HEAD→gate-20261014-203110），整合
套件 331+1、runtime 20/20、零警告 fmt clean。

blockers 无；next_responsible_role=review（第八轮 fresh final @最终 docs
HEAD，writes 已暂停，worktree clean）。

2026-10-14，stage:work，plan_revision 2，outcome:pass（r7 返工完成，
T-01..07 全部完成=7/9），worktree_path D:/autostack/.wt/lang-770/auto-lang、
branch plan-770-dev、base_commit e849e06f9（merge 基 6787bbd4a）、
code_commit 314cbbeb6（+docs 收据提交）、dependency_revisions
runtime/text@e03bb725a（未触碰）。task_ids T-02/T-06/T-07（r7 重开后完成）。
evidence：见上。blockers 无。next=review（T-08 同树独立第八轮 final
review）。

### 9.18 第八轮最终独立复审（2026-10-14，needs_fix）

stage: review；review_scope: final；reviewer_identity/limitations: 当前独立审查上下文未实施r7返工，未创建额外agent；由合同/现行Specs/同树提交和fresh运行重建结论。
plan_id: PLAN-770；plan_revision: 2；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`；
reviewed_commit: `c717555679451b4c109d308d740777e19d7ce2af`；
base_commit: `6787bbd4a0257067af43845c4450f3644125af6e`；main观察 `50c3526b3f66666ba8623ab5b569805aaafc5555`。
dependency_revisions: 772/774在基面，771 e279ee018为HEAD祖先（实际返回0）；相关canonical/774输入与整合树一致。SD冻结SHA256=`7E29C7CCF4CC7C11125FC4F6B57A7ADEA2285B10C464B4F643D51210C2B4B00C`，未发布canonical/ledger。
spec_inputs/源配置指纹与结束HEAD/clean/product hash见 [R8附件](attachments/770/review-r8/final-stability.txt)。完整AC→Task→代码→证据/纠正要求见 [R8报告](attachments/770/review-r8/REVIEW.md)。

R7 m01/m02直接HIR原例本次check/build/run0，c05/return/fresh-view等控制保持；**旧例闭合有效，完整真实退出不变量仍未闭合**。

| Finding | 级别 | AC/Task | 当前缺口 |
|---|---|---|---|
| P770-R8-01 | P2 blocking | AC02/03/04/05，T02/06/07 | block_terminates只认两臂语法同时终止、不消费const_truth；恒真then break/return与恒假else break仍被认成自然回边，首条合法view读两腿/Bundle误拒owned-scope-escape |

acceptance_results: AC01受测面pass；AC02partial；AC03partial；AC04fail；AC05fail；AC06partial（联合门禁/SD通过，修复后最终证据待重绑）；AC07fail；AC08merge-owned pending。
evidence: 串行fresh联合gate **14/14 PASS**（gate-20261014-222710，持久gate-pass/gate-raw），ac-core **331 passed/1 ignored**、runtime **20 passed**，两workspace check零warning/fmt clean；771内嵌gate **24/24、23 cases** 与741/745/767实跑。未跑根t/tv/tf/docs_gen。首次gate-20261014-221637在741 cargo run重建CLI时因审查者并行重放占用exe报os error5；属审查调度失误，保留gate-first-failed/gate-first-raw，重放结束后无代码变化完整串行重跑通过，非实现发现。
新增 **7 source+7 HIR** 与真实BundleAPI：p01/p02/p04合法恒定终止误拒，c01双臂break/c03直接return合法exit0精确trace；c02真实刷新continue合法exit6、p03陈旧真实回边拒绝保持。历史R7 **7+9**、R6 **8+8**、R5 **9+9**、R4 **15+15**、R2 **17+19**、R3 **5+7** 原字节全重放（61 source+67 HIR）达到返工后预期；核对完整身份/变异/Unicode/fault/marks/来源/4096及record隔离/值语义断言，未因首个发现跳过既定风险面。
knowledge_delta: SD路径/before/after/有限实验边界与空touched_goals解释保持有效；完成声明待恒定可达终止路径闭合后沉淀。不是新增scope/规模承诺，plan_revision不变；未修实现/合入归档/发布canonical。
pending_closeout_gates: T09实际合入、SD/导航/ledger/index、归档/计数、批回归到期、guard清理。
next: **work同树返工**；仅T02/T06/T07重开，T01/T03/T04/T05保持完成，executing4/9，T08/T09未完成。使终止/自然出口判定与走查常量可达性一致，永久化恒真/恒假/无else/嵌套/return/break/continue与真实回边正反控，提交后串行重放受影响新旧集+联合gate，再fresh final review；不得缩围/延后为merge债务，不得merge。

### 9.19 r8 返工记录（2026-10-14，P770-R8-01 修复，第八轮闭合表）

2026-10-14，stage:work，plan_revision 2（合同未变更——r7 门的常量感知补全），
outcome:pass（P770-R8-01 处置完成提交 @7468c553e + 收据 @645054332），
worktree D:/autostack/.wt/lang-770/auto-lang、branch plan-770-dev、受审基线
c71755567（merge 基 6787bbd4a 保留）。

**诊断与修复**：r7 的回边门用 block_terminates，但该 helper 是常量盲的
语法分析——只认直接终止语句或 then+else **语法上同时**终止（且只为有
else 的 if 递归计算子臂）。恒真守卫的 break/return、恒假守卫的 else-break
都被当作可自然落空，伪造重入误拒合法首读（p01/p02/p04 双腿 check1
owned-scope-escape）。修复：block_terminates 消费与走查相同的
const_truth 可达性——恒真只看 then、恒假只看 else（无 else 则落空）、
未知需全部可能臂终止；no-else 臂也入 memo（恒真用）；嵌套恒定条件组合
正确。不可达分支静态合法性检查不变；真实 continue/自然回边验证不回退。

**永久 oracle（tests/text_review_r8.rs 8 测试）**：p01（`if true { break }`）、
p02（`if false {..} else { break }`）、p04（`if true { return 0 }`）双腿
run0 精确 LIFO trace；嵌套恒定条件组合矩阵（outer true + inner
false-else-break）；p03 真实 continue 回边陈旧借用保持拒（与 r6 p03 字节
相同，冻结 atom 腿已在 text_review_r6）；c01（动态双臂 break 合法）、
c02（真实 continue 逐轮刷新 exit6）、c03（直接 return）。

**闭合表**：R8-01 闭合——复审新集 7 source+7 HIR 逐项符合；**剩余缺口：
无**。历史重放：r2 18/r3 9/r4 17/r5 11/r6 9/r7 10 测试全绿。

**AC 自检（r8 增量）**：AC-02..05 恒定条件终止控制流合法恢复；AC-04 双
通道一致；AC-06 联合门禁 14/14 双收据（7468c553e→gate-20261014-230536；
最终 HEAD→gate-20261014-231354），整合套件 339+1、runtime 20/20。

blockers 无；next_responsible_role=review（第九轮 fresh final @645054332，
writes 已暂停，worktree clean）。

2026-10-14，stage:work，plan_revision 2，outcome:pass（r8 返工完成，
T-01..07 全部完成=7/9），worktree_path D:/autostack/.wt/lang-770/auto-lang、
branch plan-770-dev、base_commit e849e06f9（merge 基 6787bbd4a）、
code_commit 7468c553e（+docs 645054332）、dependency_revisions
runtime/text@e03bb725a（未触碰）。task_ids T-02/T-06/T-07（r8 重开后完成）。
evidence：见上。blockers 无。next=review（T-08 同树独立第九轮 final
review）。

### 9.20 第九轮最终独立复审（2026-10-15，跨日，needs_fix）

stage: review；review_scope: final；reviewer_identity/limitations: 当前独立审查上下文未实施r8返工，未创建额外agent；由合同/现行Specs/同树提交与fresh运行重建结论。
plan_id: PLAN-770；plan_revision: 2；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`；
reviewed_commit: `6450543321bdff0e595c0eb61007fd5bfaadf40b`；base_commit: `6787bbd4a0257067af43845c4450f3644125af6e`。
dependency_revisions: 772/774在基面、771 e279ee018为HEAD祖先（实际返回0）；main入场b41a66814、后续观察ebd5ca4c0883c0c81c3308e49e7265a01233412e，相关canonical/774与整合树一致。源/配置hash、结束HEAD/clean与制品hash见 [R9附件](attachments/770/review-r9/final-stability.txt)。SD冻结SHA256=`7E29C7CCF4CC7C11125FC4F6B57A7ADEA2285B10C464B4F643D51210C2B4B00C`。
完整AC→Task→代码→证据、已有scope依据与纠正要求见 [R9报告](attachments/770/review-r9/REVIEW.md)。

**R8-01已闭合**：原恒真then/恒假else/no-else组合source/HIR恢复PE0精确trace，新增嵌套break/return通过；恒真continue与恒假无else真实回边陈旧view拒保持。另有遗留entry-move预扫描仍在虚构重复迭代，非本轮const-aware helper回归。

| Finding | 级别 | AC/Task | 当前缺口 |
|---|---|---|---|
| P770-R9-01 | P2 blocking | AC02/03/04/05，T02/06/07 | for true中直接return consume(x.move)，x为loop外存活owner，函数第一次调用后即终止；block_definite_moves却在进入body前先标x为Moved，source/HIR/Bundle首move误拒use-after-move |

acceptance_results: AC01受测面pass；AC02partial；AC03partial；AC04fail；AC05fail；AC06partial（联合gate/SD通过，修复后最终证据待重绑）；AC07fail；AC08merge-owned pending。
evidence: 串行fresh联合gate **14/14 PASS**（gate-20261014-235101，持久gate-pass/gate-raw），ac-core **339 passed/1 ignored**、runtime **20 passed**，两workspace check无warning/fmt clean；771内嵌gate **24/24、23 cases** 与741/745/767实跑。未跑根t/tv/tf/docs_gen。本次gate无失败/重跑，结束后才运行CLI重放。
新增 **10 source+10 HIR** 与真实BundleAPI：6合法控制均精确PE trace/exit，3真实回边unsafe反控拒保持，p03确定单次移交合法但误拒。c05移owner构造至loop内/c06去掉loop，两腿PE3、LIT/len/FREE恰一次，隔离了预杀而非callee/拼写/return lowering；p04实际重复move拒保持。历史R8 **7+7**、R7 **7+9**、R6 **8+8**、R5 **9+9**、R4 **15+15**、R2 **17+19**、R3 **5+7** 原字节全重放（68 source+74 HIR），均达到返工后预期；完整身份/变异/Unicode/fault/marks/来源/4096与record隔离/值语义断言已实跑/核对。
knowledge_delta: SD路径/before/after/有限实验边界与空touched_goals解释仍有效；完成声明待原合同内的首次move路径闭合后沉淀。不是新增语法/规模/full borrow checker要求，不能以保守预杀缩围；未修实现/合入归档/发布canonical。
pending_closeout_gates: T09实际合入、SD/导航/ledger/index、归档/计数、批回归到期、guard清理。
next: **work同树返工**；仅T02/T06/T07重开，T01/T03/T04/T05保持完成，executing4/9，T08/T09未完成。由真实可达入口/回边/目标出口处理move责任，确定零重入的首次move不得预判死，真实重复移交继续拒，caller/callee/退出后live/dead与清理唯一一致；永久化本次正反控，提交后串行重放受影响新旧集+联合gate再fresh review。登记债务仅追踪，不授权缩围/延后；不得merge。plan_revision不因簿记改变。

### 9.21 r9 返工记录（2026-10-15，P770-R9-01 修复，第九轮闭合表）

2026-10-15，stage:work，plan_revision 2（合同未变更——遗留预扫描规则的回边
感知补全），outcome:pass（P770-R9-01 处置完成提交 @e60b6eacc + 收据
@7a73cf0b6），worktree D:/autostack/.wt/lang-770/auto-lang、branch
plan-770-dev、受审基线 645054332（merge 基 6787bbd4a 保留）。

**诊断与修复**：r2 的循环入口预杀（block_definite_moves 对全部 entry-live
owner 在首次进入 body 前预判死）以"第二次迭代会重复 move"为由，未消费
r7/r8 已有的真实回边区分——return-only/零重入体执行至多一次，其对外层
owner 的首次 `.move` 被误判重复移动（p03 双腿 use-after-move 误拒）。
修复一：预杀仅在 `!block_terminates(body)`（存在真实回边，常量感知）时
执行；真实可重复循环（p04）与循环内 fresh owner（c05）语义不变。修复二
（恒定 break 终止 move 的组合 oracle 抓出）：AwaitLoop after-merge 补上
if-join 同款责任规则——dead 已并入的入口 owner 从 frame.st.live 撤销
caller 清理义务，否则循环后 return 会再次 free 已转移值（运行期
FREE-INVALID 双释放）；maybe-transfer 按并集保守读作死（与既有 join 语义
一致）。

**永久 oracle（tests/text_review_r9.rs 11 测试）**：p03（return-only 循环
外层 owner 首次 move，双腿 run3 精确单次转移 trace）/p04（真实自然循环
重复 move 保持 use-after-move 拒）/c05/c06（fresh owner 与无循环隔离
控制）/恒定 break 终止 move 组合矩阵；c01..c04（嵌套恒定 break、
false-then-true return、恒真 continue 刷新、未知自然刷新）与 p01/p02
（真实回边陈旧 view 保持拒）。

**闭合表**：R9-01 闭合——复审新集 10 source+10 HIR 逐项符合；**剩余缺口：
无**。历史重放：r2 18/r3 9/r4 17/r5 11/r6 9/r7 10/r8 8 测试全绿。

**AC 自检（r9 增量）**：AC-02..05 确定单次转移路径合法恢复；AC-03 运行期
无双释放（组合 oracle 断言无 FREE-INVALID）；AC-04 双通道一致；AC-06 联合
门禁 14/14 双收据（e60b6eacc→gate-20261015-003622；最终 HEAD→
gate-20261015-004319），整合套件 350+1、runtime 20/20。

blockers 无；next_responsible_role=review（第十轮 fresh final @7a73cf0b6，
writes 已暂停，worktree clean）。

2026-10-15，stage:work，plan_revision 2，outcome:pass（r9 返工完成，
T-01..07 全部完成=7/9），worktree_path D:/autostack/.wt/lang-770/auto-lang、
branch plan-770-dev、base_commit e849e06f9（merge 基 6787bbd4a）、
code_commit e60b6eacc（+docs 7a73cf0b6）、dependency_revisions
runtime/text@e03bb725a（未触碰）。task_ids T-02/T-06/T-07（r9 重开后完成）。
evidence：见上。blockers 无。next=review（T-08 同树独立第十轮 final
review）。

## 10. 待澄清事项

1. 774 D02/03/04/06的语义/资源支持先决结果：当前r2提出确切候选oracle，前置结果不一致先revision，不默认授权改承诺。
2. 771共享热点顺序在T01登记，非共用worktree；第二合入方重新验证，不能只解文本冲突。
3. 完整lexer/容器/enum/文本record/IO/ACC由Design36后续owner认领，不是本计划待补验收。
4. **记录性边界**（复审 2026-10-14 重新分类；2026-10-14 返工后更新）：
   (a) **【F-01 缺口，已按路径 A 修复 @c4e169cec】** trace 支持库与文本运行时
   均为 Rust no_std staticlib、各携带 `__rustc::rust_begin_unwind`，共链一个
   PE 为 duplicate symbol——首批曾以此为由把 V-07 oracle 换成文本运行时自身
   trace（复审判为未批准缩围）。修复：`test-support/marks.rs` 以
   `rustc --emit=obj -C panic=abort` 产**纯 COFF 对象**（无 panic 入口符号，
   测试内自断言）承载 mark_a/b，与运行时 staticlib 共链无冲突；V-07 冻结
   oracle（mark 包装 + source/HIR 双腿）已恢复。两 staticlib 之间的共链限制
   本身仍在（对后续需要两个 staticlib 的组合面适用；解除需任一侧去符号化）。
   (b) 预算/故障注入无语言入口（§9.2 预先裁定），
   归 runtime 测试族——合法边界。(c) char_at 越界按冻结 ABI 为 -10（-11 声明未用）
   ——与 design.md §3.4 一致，合法边界。
