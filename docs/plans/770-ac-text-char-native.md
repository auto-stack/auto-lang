---
plan_id: PLAN-770
status: executing
feature_name: AC 文本与字符首批原生编译
plan_revision: 2
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-14
current_step: 4
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
| [x] | T-01 | work | 用户批准r2；772/774已复审合入 | ✅ 已完成：[design.md](../reports/770-text/design.md)@6dbbf2712——47项输入指纹复核39零漂移+8归因、D01..07一致性表（无冲突）、码/节点/资源/Case冻结、774探针65/65绿复跑、771热点合入登记（771仍drafting，770预计先合入）；774 r2/r3/r4/r5修订链全部纳入 | 01..06 |
| [x] | T-02 | work | T-01 | ✅ 已完成：@6dbbf2712——HirDomain::Text+schema/text.atom超集、12文本ExprKind、ParamMode(view/mut/move)、moved绑定、drop语句、verify_text+text_resources前门（TextResourcePlan：LIFO出口frees/owned_returns/borrow-escape/use-after-move/double-move/mut-conflict/owned-scope-escape/resource-assign）、旧域隔离门（profile-op）；text_hir 16/16绿、ac-core全套绿（旧面零回归，eq/ne旧域文案保留） | 01/02/04 |
| [x] | T-03 | work | T-01/02 | ✅ 已完成：@e03bb725a——experimental/ac-core/runtime/text 独立 workspace（shim=774同构改名+text_eq/ne+RT-FAIL trace，剪除record/R2/探针族；23符号，锁文件入库）；19 测试 8 二进制全绿（V-04面：借用不释放/clone内容控制/LIFO/零长/跨帧拥有返回；V-05面：AP-3/4注入/溢出先于分配/预算0+清除/账本回滚/失败类RT-FAIL oracle；V-07子集；trap 0xC0000005无SUMMARY；hang=deadline+trace幸存）；.contract.json sidecar（api/target/sha256）产出 | 02/03 |
| [x] | T-04 | work | T-02 | ✅ 已完成：@860c33dec+@6cb5ec305+@c357efdb4——lexer（跨行/转义白名单/四码）、parser（类型词/mode词前置/迭代方法链复用CallFrame+method_recv/.move/.to/char类型级接收者）、resolve（VT文本族+内建方法表+语境retag+moved绑定+迭代MethodOut）、typecheck（concat/eq文本规则）、adapter（t_str/t_Str/t_char+mode/moved发射+文本节点+迭代TextOut，4096链无递归）；text_source 10/10（V-02/03/08/09 source腿，fixture内联——深链无法静态物化）；ac-core 18二进制全绿零回归（旧profile词法/拒绝面逐位保持）；CLI usage列text profile | 02/04/05 |
| [ ] | T-05 | work | T-03/04 | native/main/共享finish_build接runtime与source/HIR PE，text_native/resources tests；V02..07/09，旧事务不分叉 | 01..05 |
| [ ] | T-06 | work | T-05 | 新verify-ac-text-770.ps1/case manifest，mandatory补充全量；新旧gates、scope/严格盘点；若771已合入联跑其gate V10 | 06 |
| [ ] | T-07 | work | T-06 | README与reports/770-text/{verification,proposed-spec-delta}.md；证据hash/SD/边界冻结，不发布canonical | 06 |
| [ ] | T-08 | review | T-01..07完成、writes暂停 | 同树独立AC/SD/反例/生命周期审计，pass才reviewed，失败返work | 07 |
| [ ] | T-09 | merge | T-08pass与合入授权 | 明确两分支实际顺序；只消费新基线经work重跑联合回归且重新review的fresh凭证；SD/ledger/index/归档/计数/批回归到期判定；guard清理 | 08 |

worktree `D:/autostack/.wt/lang-770/auto-lang`、branch `plan-770-dev`，2026-10-14 已建组（基面 master@e849e06f9，含 auto-down 兄弟 detached；前置 772/774 均已复审合入归档，用户指令"实施它"=r2 批准）。work完成7/9、review8/9、实际closeout9/9。受审代码/基线变化时回同一树work重跑受影响gates、重新独立review；不能把实现/联合正确性推迟到T09。

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

## 10. 待澄清事项

1. 774 D02/03/04/06的语义/资源支持先决结果：当前r2提出确切候选oracle，前置结果不一致先revision，不默认授权改承诺。
2. 771共享热点顺序在T01登记，非共用worktree；第二合入方重新验证，不能只解文本冲突。
3. 完整lexer/容器/enum/文本record/IO/ACC由Design36后续owner认领，不是本计划待补验收。
