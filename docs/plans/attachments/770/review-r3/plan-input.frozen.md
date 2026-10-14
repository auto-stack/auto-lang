---
plan_id: PLAN-770
status: execution_done
feature_name: AC 文本与字符首批原生编译
plan_revision: 2
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-14
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
| [x] | T-02 | work | T-01 | ✅ r2 返工完成 @e81d649ac：verify_text 域前置拒（QA-01）+派生 runtime 需求前门（QA-02，声明强制）；资源走查重建——常量条件折叠/join 一致清理责任/return 先应用 move/短路条件 move 拒/循环只杀入口存活者（QA-03/07）；owned let/return 对 ReadLocal 全拒（QA-04）；scanner 补 owned Call/提升字面量（QA-05 前门面）；mut 进 provenance+调用点别名+返回外逃（QA-06）。诊断表见 §9.7；旧测试保持 + 新 18 重放测试 | 01/02/03/04/05 |
| [x] | T-03 | work | T-01/02 | ✅ r2 返工完成 @e81d649ac：runtime/text/Cargo.lock git add -f 入库（QA-08）；gate check/test --locked 与 20/20 全绿在含锁提交上重跑；gate hashes 步骤现记录其指纹 | 02/03/06 |
| [x] | T-04 | work | T-02 | ✅ r2 返工完成 @e81d649ac：词法/解析零改动；新增 source 腿负例/合法控制（qa03/qa04/qa05/qa06/qa07 系列——默认 view 参数、mut、临时返回、每轮 fresh move 的 source→verify 真实路径）；adapter 补文本 eq/ne 的 requires 声明（QA-02 配套） | 02/04/05 |
| [x] | T-05 | work | T-03/04 | ✅ r2 返工完成 @e81d649ac：finish_build/capabilities 按派生需求触发（QA-02）；backing 向量传播至末消费者（视图+借用返回，语句内合法借用安全、唯一 FREE、帧恒小——语句尾全量模型 4096 链实测跨 guard page 已弃并记录）（QA-05）；p01..p09 PE 行为修正由 verify 层责任模型保证（QA-03）；F-01 marks.obj 面保留；无 case 特判（模型级修复，诊断表 §9.7） | 01..05 |
| [x] | T-06 | work | T-05 | ✅ r2 返工完成：gate 含 text_review_r2 重放族全量重跑于返工提交（收据见 §9.7 补充与 verification.md；13/13 PASS 含 218 测试面+runtime 20/20+三旧 gates+scope 隔离） | 06 |
| [x] | T-07 | work | T-06 | ✅ r2 返工完成 @e81d649ac：CLOSE-01 三处修正（len_bytes=显式字节长度、staticlib 共链措辞+marks.obj 可用、SD-02 before 对账 PLAN-772 后 canonical）；verification.md 增 r2 发现修复行；SD 仍标注未实现缺口已清才可沉淀 | 06 |
| [ ] | T-08 | review | T-01..07完成、writes暂停 | 同树独立AC/SD/反例/生命周期审计，pass才reviewed，失败返work | 07 |
| [ ] | T-09 | merge | T-08pass与合入授权 | 明确两分支实际顺序；只消费新基线经work重跑联合回归且重新review的fresh凭证；SD/ledger/index/归档/计数/批回归到期判定；guard清理 | 08 |

worktree `D:/autostack/.wt/lang-770/auto-lang`、branch `plan-770-dev`，2026-10-14 已建组（基面 master@e849e06f9，含 auto-down 兄弟 detached；前置 772/774 均已复审合入归档，用户指令"实施它"=r2 批准）。第二轮复审后当前完成1/9（T-01）；其余受影响work任务返工，review与实际closeout均待完成。受审代码/基线变化时回同一树work重跑受影响gates、重新独立review；不能把实现/联合正确性推迟到T09。

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
