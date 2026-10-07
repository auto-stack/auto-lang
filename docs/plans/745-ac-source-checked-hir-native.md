---
plan_id: PLAN-745
status: executing
feature_name: Auto 源码计算子集 → Checked HIR → Windows native
author: [Codex]
created_at: 2026-10-06
updated_at: 2026-10-07
plan_revision: 1
current_step: 3
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-acc/project.md
new_spec_components:
  - docs/specs/auto-ac/source-core-i32.md
touched_goals: []
affects: [auto-hir, auto-ac, auto-acc]
---

# [PLAN-745] Auto 源码计算子集 → Checked HIR → Windows native

## 0. 变更摘要

承接 741 r8、743 r6 已归档基线，认领 743 next-work-packages 候选①及 A1–A3 源码形态。
在独立 experimental/ac-core 新建源码子集前端，经既有 Schema binder 与 verify，复用 native/link 产 PE。
本轮只起草、中央取号、提交计划簿记、建树登记；不实施编译器。

提案 source profile=`auto.source.core-i32.draft` revision 1，CLI 必须显式选择。
子集内 int/i32 精确映射有符号32位；不宣称全语言 int/native 映射冻结。
HIR 三重身份 `auto.hir.core.draft / 1 / core-i32-draft` 保持，Schema/运算面不扩大。

## 1. 目标

单模块函数型 .at → 名字/类型已解析的 HIR → CheckedModule → Windows x64 PE 原生执行。
保持来源、精确类型、词法绑定、实参求值顺序、控制流和运行期 trap；支持集外受控拒绝。
后端只接本进程 verify 返回的 CheckedModule，没有 VM/A2R/旧 AST 回退。

非目标：旧 parser/typeck/VM/A2X 接线，ACC 主体迁移/自举，str/char/List/record/enum，
全序整数/逻辑运算扩面、跨模块、泛型/所有权、优化/pass框架、生产ABI/桥/DLL/热重载及其它平台。
只写 auto-lang 仓；不改 auto/lib/*.at、生产 Cargo.toml/lock 或外仓。

## 2. 架构方案

```text
.at + explicit source profile
 → S0 spanned tokens/AST（独立子集 parser）
 → 签名预登记 / 词法名字解析 / 精确类型检查
 → S1 core-i32 Atom 文本 + emitted-token→原源码 span 表
 → bind_source（S2 Bundle）→ 全部 HIR span 投影回原 .at
 → verify::verify（S3→S4 CheckedModule）
 → native::capabilities_check/find_entry/lower_object（S5）
 → link::link_object_staged/publish_artifacts（S6）→ PE
```

binder 是类别/引用绑定入口，verify 是 Checked 凭证唯一入口。生成文本是内部确定性编码，
profile/输入hash/HIR身份在 adapter结果与收据随行，不向 descriptor 塞未知字段。
源码/HIR输入模式分开，共用 Checked→native→事务发布；native 不解析源码、不补 typecheck。

## 3. 技术栈

Rust 2021/std、独立 ac-core workspace、原 Cranelift 0.126.2 与 Windows x64 工具链。
不新增旧 Auto crates 依赖。子集parser模块组织可等价调整，合同语义/接口/验收不可静默更改。

唯一专属树：D:/autostack/.wt/lang-745/auto-lang，branch plan-745-dev，基于主检出 v0.6-dev。
[worktree.md](attachments/745/worktree.md) 记录实际创建提交/Git登记/guard/状态。
实现、内部检查、最终独立复审顺序共用该树；new交接前必须实际建好登记，不能留执行时再建。
主检出仅维护 docs/plans/** 簿记；禁止junction/symlink。

## 4. 需求分析与背景调查

### 4.1 授权与来源

用户2026-10-06授权：先读当前 auto-hir/auto-ac/auto-acc Specs 与743后续包，规划下一计划，
明确数值/支持/正反语料，中央取号，new前建树登记；覆盖起草、计划提交、建树，未授权本轮实施/合入。
无指定模型、预算或委派。主线落点按用户指定 v0.6-dev，覆盖通用 master 默认。
AGENTS L1要求先展示合同确认；本r1确认前不开始T-01实施，不自行创建其它chat/agent。

调查HEAD=ae8199416c772e95dc63e6b647ab3b833bd3cc7c，起始工作区clean。
已读三个模块全部当前Specs（含plans/stage-contract）、overview与743 next-work-packages/acceptance-matrix/inventory，
strategy/{auto-ac-subset-and-target,auto-hir-design,auto-acc-bootstrap-contract}，ac-core src/{lib,hir,verify,atom_text,main,native}
与trace测试。来源SHA256见[inputs.json](attachments/745/inputs.json)，work前检查漂移。
旧parser/Value/Rust emitter/string/auto-lib只作事实对照，未接线、未声明v0.5恢复。

### 4.2 约束的实际依据

- 741运算仅add/mul/lt；sub/div/mod/eq/ne/gt/le/ge与&&/||/!属于候选②，不偷渡。
- Constant只接受i32，无bool literal节点；true/false在adapter编码为等价比较，不改Schema。
- 无通用expr statement/unit：本子集只支持有值return、初始化/赋值中的调用。
- Value::Int(i32)与trans/rust.rs::Type::Int→i64并存，AST名不能证明全语言native宽度。
- char路径存在不同宿主表示（旧str_char_at返回单字符字符串，auto-lib使用字符/整数）；
  probe-char-int保留可见路径证据与未知项，源码char统一拒绝，不宣称完整映射。
- MD-408/409需求分类不能因子集变成全面implemented；仅收敛本profile宽度，完整int/char仍open。

### 4.3 中央取号

默认bash为WSL；实际用D:/soft/Git/bin/bash.exe，加/usr/bin:/bin，调用scripts/new-plan.sh。
中央初值744，但744已用于SQLite；脚本ls(active+archive)|grep-q因pipefail+不存在archive glob漏检。
撤回本会话未跟踪744骨架，再调中央脚本获得745，.next-id=746；无手改计数、无覆盖744。
最终核对active+archive中的745前缀唯一；历史13/152等重号不在本计划清理范围。
缺陷登记P745-D1，修脚本另走worktree，不在主检出改代码。

## 5. 详细设计

### 5.1 数值映射（本合同提案）

| 源码 | HIR/ABI | 精确规则 |
|---|---|---|
| int/i32注解、无后缀十进制整数 | I32 | profile内同义，参数/局部/返回统一，无上下文扩宽 |
| 含负号整数字面量 | Constant i32 | [-2147483648,2147483647]；宽临时解析+range检查，越界source.literal-range，禁截断/回绕 |
| -2147483648 或 - 2147483648 | 单个有符号literal | 负号只允许直接数字；-x/-(expr)/二元减法拒绝 |
| +/* | add_i32/mul_i32，overflow:trap | 运行期signed溢出→ExitProcess(70)；常量表达式不得编译期拒绝/提前执行 |
| < | lt_i32→Bool | signed比较；Bool/I32无隐式互转 |
| bool/true/false | Bool；true=0<1，false=0<0 | fresh Expr子图，合成span指literal token，Schema不变 |
| bool参数/局部/返回 | Bool；native i32 0/1 | Bool类型身份与I32不同，既有ABI消费 |
| char/float/i64/u32/u64/uint/usize、带后缀/进制literal | 不支持 | 显式拒绝；char不转整数；全语言映射保持待定 |

退出70只为测试profile约定。普通return70同码，trap证据须配HIR overflow结构+前驱trace；
全宽边界用HIR值和signed比较后小退出码证明，不依赖PowerShell signed退出码展示。
T-01的probe-int-width/probe-char-int先给带输入版本的决策报告；不以旧VM溢出行为作本profile oracle。

### 5.2 支持与拒绝

函数`fn f(a int, b bool) int { ... }`（参数/返回必须注解int/i32/bool）；
局部`let x = expr`/`var x int = expr`（可省注解，只推导I32/Bool），始终初始化。
支持显式有值return、if/else语句、while条件循环、loop、无标签break/continue、局部`x = expr`、
括号与*高于+高于<（各左结合）、直接/互递归/前向函数调用。
函数签名先登记；源码entry是函数名，零参数返回int/i32。全部函数全路径return，以既有verify保守规则为准，
不证明无限loop必执行/终止；终结符后语句拒绝。分支/循环局部不得逃逸。

UTF8源码/注释，标识符限ASCII[A-Za-z_][A-Za-z0-9_]*，关键字不可作名字；空白/换行、//注释、分号分隔。
括号内换行是空白，未括起的表达式不跨语句换行。块注释/Unicode名字/尾垃圾/坏token受控拒绝。
禁止未初始化声明、顶层执行/变量/use、尾表达式/隐式return、未注解函数、方法/默认/变参/泛型。
列出的不支持类型/运算/extern/intrinsic/checked/pass标记均拒绝；不跳过剩余输入，未调用函数也全检。

调用只允许全positional或全`name: expr`，混用拒绝；eval_args按源码左→右（binary亦左→右），
bindings独立按形参名建立双射。重复/未知/缺少实参拒绝，异构形参按映射检查。
同块重复声明、重复param/fn拒绝；内层遮蔽合法，初始化RHS使用声明前环境；local优先函数名，
local遮蔽后的调用报source.not-callable。break/continue只指最近词法loop，循环外拒绝。

while降为loop body内`if cond { source body } else { break }`，每轮/continue后重验条件，不能hoist。
每个语法出现新ExprId，不共用常量/表达式/块；source先查owner/type/place/返回/调用，仍整体verify。
true/false编码不实现一般优化/pass；无折叠、去重、效果重排，保持X1/X2/X4/X7/X8/X9。

### 5.3 来源与可信边界

每个生成Atom token/节点有原源码byte span；合成bool/while指回literal/while条件token。
binder诊断通过生成映射回.at；绑定后types/defs/bodies/locals/exprs/places/blocks全部span投影为.at，
verify/native按同一源文件渲染。pre=生成token覆盖；post=全部span范围/UTF8边界合法且无生成位置残留。
缺映射报受控internal诊断，不伪造source错误、不panic。input hash/profile在result/receipt随行。

现有阶段枚举可复用：text/source.lex、source.syntax、source.unsupported、source.literal-range；
bind/source.undefined、source.duplicate、source.argument、source.not-callable；verify/source.type-mismatch及既有verify.*。
fixture固定code、token与行列，不锁整条message。失败不产生Checked、不调lower/link。
保持741类目/悬空引用/owner/唯一Expr/初始化/循环/双射检查。测试篡改Bundle后必须重新verify，不能复用凭证。

### 5.4 CLI/trace/制品

新增`check-source <file> --source-profile auto.source.core-i32.draft`与
`build-source <file> --source-profile auto.source.core-i32.draft --entry main --output <exe>`。
capability/support-lib选项同旧build；成功/流水线拒绝/用法错=0/1/2。
漏/未知profile拒绝2；不按扩展名猜模式；旧check/build仍只收HIR。
共用Checked后构建管线，保留741受控进程、有限截止、制品发布/回滚/清理合同。
源码元信息写既有.ac-link.txt同事务，不提前发布旁路收据；首次失败无制品，已有SENTINEL三件全保持。

mark_a()/mark_b()仅由显式hir.test.trace.v1激活封闭nullary→int测试intrinsic表，注入HIR带requires，
未提供trace capability时源码mark调用是source.undefined；已注入的HIR若缺能力仍由native门报capability.missing。
native trace显式support-lib；用户函数/参数/local同名优先，不能名字劫持。
用`let observed = mark_a()`表达前驱效果，不用无值statement偷渡unit。任意extern/intrinsic声明拒绝。
来源/凭证检查独立于capability门，不能靠--capability获得checked授权。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-core-i32.md | 无source profile→精确数值/支持/拒绝/CLI/来源合同 | 有限域不可冒充完整Auto | AC01..06 |
| SD-02 | modify | docs/specs/auto-hir/stage-contract.md | S0未实现→仅本profile S0→S2；HIR身份/pass状态保持 | 分清source构造与Checked/pass | AC03,04,07 |
| SD-03 | modify | docs/specs/auto-ac/project.md | HIR→native→增加明确profile源码→native | A1–A3源码证据 | AC05..07 |
| SD-04 | modify | docs/specs/auto-acc/project.md | int/char unknown→仅profile int有证据；完整宽度/char仍open | 盘点与实际能力一致 | AC01,07 |

执行只写proposed delta；review定稿，merge才沉淀Specs/模块plans/index/ledger。
不重开GOAL-017，不以有限adapter算Gen1/2/3。

## 6. 测试设计

[corpus.json](attachments/745/corpus.json)给完整源码及退出值/trace/code/token/结构oracle。
**这些是新接口待实施fixtures设计，不是已运行证据**；T-02/T-05物化到新fixtures/source/{valid,invalid}/。
以下补充矩阵也全部必测，由T-05构造，不可只跑JSON若遗漏这些路径：

| invariant | 区分合理错误实现的正/反例 | verification entrypoint / observable oracle | 覆盖边界 |
|---|---|---|---|
| 精确类型/宽度 | min/max、正负越界、bool-int混合、异构命名反序 | source-HIR类型/值检查；PE signed <后小码；range/code/token | 不验完整语言宽度兼容 |
| runtime trap | MAX+1/MAX*2、未走分支溢出、mark_a后溢出 | build成功；PE=70或0；stderr精确a；HIR trap结构 | 不以VM debug溢出为oracle |
| eval_args双射 | pair(b:mark_b(),a:mark_a())=12/ba；异构映射；重复/缺/未知arg | 实际源码→bind→verify→PE；结构/trace/码 | positional/命名分测，混用拒绝 |
| loop重验/最近目标 | while零次/多次/continue后判条件、nested loop、loop break | PE=1/3/3/5；60s硬deadline；unique block/loop target | 无任意程序终止承诺 |
| scope/identity | 前向/递归/互递归、内层shadow、分支逃逸、同名fn/local/mark | PE值与source.*诊断；用户mark无trace | 不做方法/模块解析 |
| 来源全覆盖 | 未定义/类型错、合成bool/while、中文注释前缀、LF/CRLF | 所有byte span合法；原.at准确line/col/token；缺映射受控失败 | 不收生成文本位置冒充source |
| 全文件全消费 | 坏字节/不闭合/尾垃圾/空文件、未调用函数内unsupported | text/source.*固定code/token；无panic无Checked | 合法运行期trap与unsupported分开 |
| Checked唯一 | 假checked源；adapter Bundle删let/错type/越域ref | 实际bind/verify拒绝，无lower调用；现有HIR负例保留 | 变异只在tests，不开放绕过API |
| CLI/事务 | 新check/build，漏profile，错entry/签名，坏source/坏link | 实际0/1/2；capability门；exe/obj/receipt SENTINEL完整保持 | 同741事务/进程约束，不新造框架 |

角色：T01..06实现；T07内部检查（同实现上下文，只表示ready for final review）；
T08最终独立上下文复审（同树、实施暂停写）；T09收口。用户未指定人员/模型，不创建或联络其它agent/chat。
review入口核对path/branch/revision/HEAD，复审期间HEAD稳定；修复回work同树并使相关证据过期重验。

开发只跑ac-core check/scoped tests。最终在同专属树串行跑：

```powershell
cargo check --locked --all-targets --manifest-path experimental/ac-core/Cargo.toml
cargo fmt --manifest-path experimental/ac-core/Cargo.toml -- --check
cargo test --locked --manifest-path experimental/ac-core/Cargo.toml
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-741.ps1 -SkipMainGates
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-source-745.ps1
cargo check -p auto-lang
cargo t
cargo tv
```

verify-ac-source-745.ps1为T05新建，当前不可执行/不可称已过；串行消费case+补充矩阵，记录输入/binary/hash/HEAD、
真实stdout/stderr/退出值/超时/阶段/三件制品。PE每例硬截止60s，cargo/link/support build均有限截止；
复用既有受控执行器及一键脚本方式，不把无限Command::output等待作为合格执行。
虽不改生产crates，ac-core是编译器实现，work/review按仓库代码门禁跑check+t+tv。
预存红按同源码/环境scoped对照归因，不省门禁、不修不相关旧功能来伪造全绿；
tf非per-plan，taa/tt/tb/tu/docs_gen无触面不跑；merge到期批回归主检出单实例。
本轮纯new文档不跑Cargo。

## 7. 验收标准

| ID | owner_stage | 可观察结果/验证方法 |
|---|---|---|
| AC-01 | work | T01报告+数值cases证明本profile int/i32=32位、range/其它宽度/char拒绝；MD408/409不冒称全面能力 |
| AC-02 | work | 支持集正例/排除集逐族反例全物化，scope/type/call/control/full-consumption固定code/token通过 |
| AC-03 | work | 源经真实bind→verify→Checked；变异/假标记不过门；原descriptor/HIR语义与741拒绝矩阵保留 |
| AC-04 | work | 生成/合成来源全覆盖；bind/verify/native准确原.at位置；UTF8/CRLF/错误token断言通过 |
| AC-05 | work | A1–A3真实源码→PE；算术/循环/递归/异构命名/ba/trap/分支语义匹配，有原始结果和产物指纹 |
| AC-06 | work | 新CLI显式profile/0-1-2/entry/capability/user-shadow/失败不发布；旧CLI事务/进程回归通过 |
| AC-07 | work | proposed SD01..04+MD408/409证据新鲜，inventory strict或待沉淀提案边界准确，char/full-width保留open owner |
| AC-08 | review | 逐AC独立复核、健康/分级门禁/遗漏延后workaround审计绑定r1+HEAD，未批准缺项不得pass |
| AC-09 | merge | v0.6-dev落地、Specs/ledger/index/导航沉淀、真实archive终态9/9，guard clean后同树/分支/空组清理收据 |

AC01..07先内部检查，再最终独立复审全部重新审定；AC09不承接实现正确性。

## 8. 执行步骤

唯一任务表。2026-10-06 第二轮最终复审 needs_fix：T-01 保留完成，T-02..07及相关验证/SD证据重开，T-08/09待完成，current1/total9；下列初版及修复轮work收据保留为历史，不能覆盖最新最终发现。

| ID | 状态 | owner_stage | 依赖 | 文件/符号与可验证产出 | AC / 验证 |
|---|---|---|---|---|---|
| T-01 | [x] | work | r1确认 | 新docs/reports/745-source-core-i32/numeric-boundary.md；核对可见值/parser/trans/char路径，完成两probe与MD408/409提案；若事实推翻profile可行性先修合同 | AC01,07；数值cases+决策来源，先于T02/03类型实施。〔2026-10-06 work：报告落地（worktree commit c535b8fd6）；23项输入hash零漂移；E1–E12证据链（VM栈i32/值存储i32\|i64/旧AST幅值三态/trans i64/char三宿主表示）；probe-int-width=profile内精确有符号32位、probe-char-int=profile拒绝char，MD408/409提案文本入报告§8；无事实推翻r1可行性，§7数值cases待T-05运行回填〕 |
| T-02 | [ ] | work | T01 | 新src/source/{mod,lexer,ast,parser}.rs、lib.rs导出、fixtures/source/、tests/source_frontend.rs；grammar/span/full-consumption可运行 | AC02,04；ac-core check；cargo test --manifest-path experimental/ac-core/Cargo.toml --test source_frontend（新目标）。〔2026-10-06 work：worktree commit 6965c0a04；21正/53反fixture精确物化；12前端测试全绿（30 text级code+token断言、23 bind/verify级前端须通过断言、P01/P02/P04/P07/P11/P15/P20结构span、N03行列2:24、N31 1:24）；ac-core全量测试通过、fmt clean〕 〔r1-fix 2026-10-06：QA-01 换行边界（expr_line 贯穿、callee同行括号）、QA-04 全量关键词集（Reserved 词法类+名字位 unsupported、转义 str/char 单 literal、! 归 unsupported）、QA-03a N09/C10 越域指 ident；新 C 系 fixtures 13 反例+3 控制；frontend 12 测全绿；commit 3e3c1631f〕 〔r2-fix：QA-01 动态行锚（ENTRY 任意行+后续 token 随前一 token 行，括号闭后合法前移）→ D01-D05/D07 native 5/6/1/6/2/7、C01/C03/D06 仍拒；QA-05 NEST_LIMIT=200 深度护栏（E01 过/E02 exit1+syntax，含块）〕 〔r3-fix：护栏改语法级计数（99 层合法括号过/4096 不闭合拒）；parse 迭代 + ast::Expr 迭代 Drop；F01..F10 全过并固化 f04/e01/f11 语料〕 |
| T-03 | [x] | work | T02 | 新src/source/{resolve,typecheck,adapter}.rs，签名/词法栈/精确类型、生成Atom+map→bind_source→全部span投影→verify；tests/source_hir.rs | AC01..04；新source_hir测试、owning/bindings与负变异。〔2026-10-06 work：worktree commit 3fb6c96b6；resolve（签名预登记/块作用域/参数映射/trace intrinsic优先级）+typecheck（source.type-mismatch@verify）+adapter（while→loop/if降低、bool=0<1/0<0合成、return后死代码独立块、stray break/continue phantom loop、条目序span再锚+token碎片回退）；10测试全绿（23负例stage/code/token精确、P03/04/11/15/19结构、4变异拒收）；全ac-core 80测通过零警告零fmt问题〕 〔r1-fix 2026-10-06：QA-02 源码函数符号 sf_ 前缀（identity 走 d_ id，不新增保留名，ac_start/ExitProcess 可用可入口）；QA-03b 死代码分块扩至 return/有效break-continue/双分支终结/returning-body loop（Stray 保持同块），死块 span 钉首个死语句；N15/16/18/N07 锚定回归不变；hir 13 测全绿（含 overflow 结构断言、Bundle 三变异重verify）〕 〔r2-fix：QA-03 Stray 组合不提升为有效终结（D10/D13 loop-target@1:27/1:42 可达；D08/D14 有效 break 锚定保留）〕 〔r3-fix：resolve/typecheck/adapter Binary 脊柱全部显式栈迭代 + ResolvedExpr 迭代 Drop；verify expr_type/walk_value_in/dfs_cycles 迭代化（语义/顺序保持）；16384/65536 合法链全阶段通过〕 |
| T-04 | [x] | work | T03 | src/main.rs新命令/共用Checked构建；source/mod.rs显式profile/trace表；必要link.rs收据追加保持事务；tests/source_cli.rs | AC03,04,06；新source_cli+旧cli/native/trace tests。〔2026-10-06 work：worktree commit c5af427d9；check-source/build-source（漏/未知profile=2、0/1/2分级、entry按函数名查def id）、link.rs新增publish_artifacts_with_receipt（源码profile/hash同事务入.ac-link.txt，旧函数零语义变化委托）；10 CLI测试全绿（真实PE构建运行P01=14/P13=70、失败零制品、entry.not-found/signature、trace无support-lib链接失败、旧check仍仅Atom）；全ac-core 90测通过零警告〕 〔r1-fix 2026-10-06：QA-08 抽取 finish_build 唯一 Checked→staged→事务发布尾部，两模式共用，入口按 d_ id；cli 11 测全绿（含 source SENTINEL 三路：源拒绝/链接失败/publish 回滚三件 byte-identical）〕 〔r2-fix：无新发现；finish_build 共用维持〕 |
| T-05 | [ ] | work | T04 | 新tests/source_native.rs、scripts/verify-ac-source-745.ps1，物化执行JSON全部case和§6补充；docs/reports/745-source-core-i32/verification.md及原始收据 | AC01..06；ac-core全test+新一键脚本，60s/code/trace/token/hash/制品。〔2026-10-06 work：worktree commit addcf4d5a；脚本实测 **all 12 steps PASS (74 cases)**（21正例真实构建+运行退出码/stderr oracle全中含P15=12/stderr ba、P13/P14/P16=70、60s硬截止、三制品+sha256收据；53负例exit 1+stage/code标记全中）；source_native 2测试串行7.5s；§6补充矩阵九行证据映射入verification.md §5；主仓门禁留T-07回填〕 〔r1-fix 2026-10-06：QA-05 tests/common run_deadlined（双管道并发排空+截止 kill+wait 回收）接入 cargo/CLI/PE 全路径；大输出控制（20000×a stderr 全排空 exit 20000）与挂起控制（无状态 loop{continue} 3s 被杀；自增循环会在 2^31 触发 trap 提前退出的现象在案）；native 5 测串行全绿含 C05=3/C06=4/入口 ac_start=3〕 〔r2-fix：QA-02 双执行器重写（Rust 通道化无界 join 消除+字节排空 lossy+单一截止；PS 事件化采集+taskkill /T /F+预算封顶）→ 后代持管道 800ms 有界 timed_out、无效 UTF8 保留、745 脚本 74 例全过含 P15 精确 stderr；QA-04 同类型越域 Bundle 变异（uninitialized-read）+三件真文件 publish 回滚（FileShare.None 持有）入永久测试；D/E 系 21 fixture 全固化〕 〔r3-fix：tests/common 委托 link::run_deadlined_split（741 执行器：受控启动/Job 子树/单截止/全回收）；PS 内嵌 C# AcContainedRunner 同合同；新增后代持管道(0幸存)/迟到写后代(全量LATE)/失控父树杀/无效UTF8/并行收敛控制；f11 300链 native=300 精确和〕 |
| T-06 | [ ] | work | T05 | ac-core/README.md，proposed-spec-delta.md及MD408/409更新提案/新鲜绑定策略，不直接改canonical | AC07；python scripts/acc_inventory.py --check --require-decisions，提案说明待merge绑定，不自动重hash掩漂移。〔2026-10-06 work：worktree commit 8b4526bd7；README 双管线/新测试族/边界更新；proposed-spec-delta.md SD01..04+MD408/409追加note+盘点检查实测（2项失败均为743扫描器工作区态非内容漂移，归merge重扫）+新鲜度绑定策略（r1+提交链+收据指纹，merge重跑替换）〕 〔r1-fix 2026-10-06：QA-07 盘点错误调用与『未入库』误述撤回留档，正确命令实测 CHECK-OK（47 决策新鲜），merge 不预排 --write、MD note 追加后独立复跑+检验消费者；README 边界节补符号隔离/关键词集/换行规则〕 〔r2-fix：报告 §6 历史限度措辞修订（crates 同源表述更正）、§9/§10 r2 执行记录与 oracle 分列〕 〔r3-fix：报告 §11/§12 记录两项处置与边界两侧永久回归；shell 256 退出码截断现象入案〕 |
| T-07 | [ ] | work | T06 | 同树内部检查所有cases/SD/健康，§6全部门禁，提交HEAD与执行收据；work7/7→execution_done（总7/9） | AC01..07；内部pass仅交接review。〔2026-10-06 work：§6全块实测——ac-core check/fmt/all-targets零警告/全test 92绿；verify-ac-741.ps1 -SkipMainGates **13步全过**；verify-ac-source-745.ps1 **12步全过(74例)**；主仓 check -p auto-lang PASS(2m50s)；cargo t 全量 4939/4972=33失败 与主检出同源对照**完全同数**（27共现预存红+6+6对称差flake，745新增失败=0，归因收据 target/ac-verify-receipts/source-745/main-gates/）；cargo tv 158/162=4预存cookbook红两环境逐项一致；环境事件：D:满盘(os112)一次中断+C:重试曾污染ffi_dual oracle marker(已清除全部原生重跑)；组内补建只读依赖树auto-down@fba6563ed。工作树HEAD b3cd81719 clean〕 〔r1-fix 2026-10-06：全门禁重跑——ac-core 99测+零警告零fmt问题；741 13步/745 12步74例全过；正确 inventory strict CHECK-OK；主仓 check PASS、cargo t 4939/4972=33失败（28与前轮共现预存红逐名一致+5名隔离复跑5/5全过=并行flake）、tv 158/162=4预存cookbook红逐名一致，745新增失败=0；修复轮提交链 3e3c1631f→d9f5499b7→36664d9a5→507025957→91b37560c，树 clean @91b37560c〕 〔r2-fix：全门禁重跑——ac-core 104 测/零警告零fmt；741 13步/745 12步74例全过；inventory CHECK-OK；主仓 check PASS、t 4936/4972=36失败（29共现预存+7隔离复跑7/7全过flake轮换）、tv 158/162=4预存cookbook；745新增失败=0；r2 提交链 ad226453b→cd9492f20→a830dac4b→7437eb137，树 clean @7437eb137〕 〔r3-fix：全门禁——ac-core 112 测零警告零 fmt；741 13步/745 12步74例/Inventory CHECK-OK/主仓 check PASS；t 4939/4972=28共现预存+5隔离复跑5/5全过；tv 158/162=4预存cookbook；745新增失败0；r3 提交链 072f345f9→ee4ccbb74→dc5e95be7，树 clean @dc5e95be7〕 |
| T-08 | [ ] | review | T07 | /auto-plan:review独立上下文同树，写入暂停，revision/HEAD稳定，逐AC实际证据/反例/债务/proposed delta；最终报告 | AC08并独立重验AC01..07；pass→reviewed（8/9） |
| T-09 | [ ] | merge | T08+合入授权 | /auto-plan:merge到v0.6-dev；SD/人工盘点/ledger/导航/index/归档；批回归到期判断；wt-guard clean才清理 | AC09；实际链接/终态/9/9/清理收据，不预勾 |

src/tests新路径均相对experimental/ac-core；reports新路径相对docs/reports/745-source-core-i32。
未来实现/最终review不得以计划表的勾选代替证据。

## 9. 复审记录

2026-10-06 stage=work，PLAN-745 r1 交接记录（work 7/7 完成，execution_done）：

- `stage`: work | `plan_id`: PLAN-745 | `plan_revision`: 1 | `outcome`: **pass**（内部检查，非最终复审）
- `worktree_path`: D:/autostack/.wt/lang-745/auto-lang | `branch`: plan-745-dev
- `base_commit`: eea7a80618d4d5e75749d666b7609a9a87925bbd | `code_commit`: b3cd81719（树 clean）
- `dependency_revisions`: auto-down @ fba6563ed2148ce85e68208863159b4ccccac710（组内只读依赖树，detached，零修改）
- `task_ids`: T-01..T-07 全勾（7/9；T-08/T-09 属 review/merge）
- `evidence`: 提交链 c535b8fd6→6965c0a04→3fb6c96b6→c5af427d9→addcf4d5a→8b4526bd7→b3cd81719；
  verify-ac-source-745.ps1 12步全过(74例)；verify-ac-741.ps1 -SkipMainGates 13步全过；
  ac-core 92测全绿零警告零fmt问题；主仓 check/t/tv 带同源归因（745新增失败=0，
  归因收据 experimental/ac-core/target/ac-verify-receipts/source-745/main-gates/）；
  报告 numeric-boundary.md/verification.md/proposed-spec-delta.md 齐
- `blockers`: 无阻塞 review 项。注记：①日常/语料档预存红 33+4 处已同源归因
  （KNOWN-DEBT 396、dep 017 drop_count、musk/chart/gallery 族、cookbook semver 族），
  T-08 可复核归因并考虑登记 KNOWN-DEBT-AND-RISKS；归因时间线注记：主检出对照
  运行（17:1x+）晚于 plan-742 合并（a6e108f60 @16:43，改 trans/vue/auto-man），
  即对照双方 crates 相差 742 变更——27 处共现核心红在 pre/post-742 两种 crates
  状态下逐名一致（既非 742 修复面也非其引入面），且 745 diff 与全部失败域
  零交集；若 T-08 需严格同源对照，可在 eea7a8061 基线另建临时树复跑；
  ②D: 盘曾 100% 满（os112），已恢复 ~30G，C:/tmp/ac745-main-gates 已删除，
  ffi_dual 污染 marker 已清除；③工作树落后主检出（742 合并+簿记），实现分支
  保持 plan-745-dev @ b3cd81719，review 按该 HEAD 执行，merge 时再整合
- `next_responsible_role`: review（T-08 /auto-plan:review 独立上下文，同树执行，
  实现写入暂停；入口核对 path/branch/revision/HEAD=b3cd81719 稳定）


2026-10-06 stage=work（r1 修复轮），PLAN-745 r1 修复交接记录：

- `stage`: work（needs_fix 修复）| `plan_id`: PLAN-745 | `plan_revision`: 1（语义合同未变）
- `outcome`: **pass**（修复轮内部检查；QA-01..08 全部处置，待 T-08 重新最终复审）
- `worktree_path`: D:/autostack/.wt/lang-745/auto-lang | `branch`: plan-745-dev | `code_commit`: **91b37560cffda75cf07cef8620cf524f5d6a3c7c**（树 clean）
- `dependency_revisions`: auto-down @ fba6563ed2148ce85e68208863159b4ccccac710（未变，只读）
- `repair_chain`: 3e3c1631f（QA-01/02/03/04/08 实现）→ d9f5499b7（QA-05/06 测试）→ 36664d9a5（公共 helper 警告清理）→ 507025957（报告/README）→ 91b37560c（修复轮门禁记录）
- `finding_disposition`: QA-01 换行边界=C01/C03/CRLF/注释变体拒绝+C02 括号控制合法；QA-02 sf_ 符号前缀=C05 作被调/入口=3、C06=4；QA-03 越域指 ident（N09 oracle 更正为 x，语料草案 oracle 被 §5.3/AC-04 取代）+死语句锚定（C09 1:57/C10 3:9，UTF8+CRLF 4:2 控制），N15/16/18/N07 锚定回归不变；QA-04 Reserved 关键词集（C12/C23/C24）+转义 literal（C04/C25）+!（C15），未闭合仍 source.lex；QA-05 共享 run_deadlined+大输出（20000 stderr 全排空）/挂起（3s kill）控制；QA-06 overflow 字段断言+Bundle 三变异重verify（逐变异断言输入改变）+source SENTINEL 三路 byte-identical；QA-07 盘点误用撤回+正确命令 CHECK-OK；QA-08 finish_build 两模式共用
- `gates`: ac-core 99 测/零警告/零 fmt；741 13 步、745 12 步 74 例全过；正确 inventory strict CHECK-OK；主仓 check PASS、t=4939/4972（28 预存共现+5 隔离复跑 5/5 全过 flake）、tv=158/162（4 预存 cookbook）——745 新增失败 0
- `evidence`: verification.md §7/§8（修复轮逐项执行记录+门禁）；反例回放固化 fixtures/source/{valid,invalid}/c*；C 系全部进永久测试表
- `blockers`: 无。注记：语料 N09 token oracle 由 "return x" 更正为 "x"（QA-03 裁定精确 token 优先，已在测试注明）
- `next_responsible_role`: review（T-08 按新完整 HEAD 91b37560c 重新最终复审，同树、实现写入暂停；对照 review-r1/REVIEW.md 逐 QA 复核）

2026-10-07 stage=work（r3 修复轮），PLAN-745 r1 修复交接记录（第三轮）：

- `stage`: work（r3 needs_fix 修复）| `plan_revision`: 1（语义合同未变）
- `outcome`: **pass**（r3 修复轮内部检查；R3-QA-01/02 全部处置，待第四轮最终复审）
- `worktree_path`: D:/autostack/.wt/lang-745/auto-lang | `branch`: plan-745-dev | `code_commit`: **dc5e95be757dba647ef758aa283e7e2788015747**（树 clean）
- `dependency_revisions`: auto-down @ fba6563ed2148ce85e68208863159b4ccccac710（未变）
- `repair_chain`: 072f345f9（QA-01 执行器复用+QA-02 迭代化/语法护栏）→ ee4ccbb74（幸存断言并行收敛）→ dc5e95be7（报告）
- `finding_disposition`: R3-QA-01=Rust 测试执行器删自制实现、委托新公开的 link::run_deadlined_split（741 合同执行器）；PS 脚本内嵌 C# AcContainedRunner（CREATE_SUSPENDED→Job→Resume、单截止覆盖 EOF 采集、字节 lossy、TerminateJobObject 树杀、finally 全回收）——后代持管道 800ms 有界 timed_out+0 幸存、迟到写后代全量收集（LATE）、失控父树杀 0 幸存、无效 UTF8 保留，全部永久控制；R3-QA-02=①resolve/typecheck/adapter 的 Binary 脊柱 + AST/ResolvedExpr Drop + verify(expr_type/walk_value_in/dfs_cycles) + native(eval_expr) 全部显式栈迭代（语义/错误序/求值序保持）→ F08（16384 合法链）check exit0、65536 全阶段过、300 链 native=精确和 300；②护栏改语法级计数（括号/实参/块），F03/F04（99/100 层合法）过、F02/F07/F05/F10 受控拒；tests/deep_regression.rs 永久化边界两侧
- `gates`: ac-core 112 测/零警告/零 fmt；741 13 步、745 12 步 74 例、inventory strict 全过；主仓 check PASS、t=4939/4972（28 共现预存+5 隔离复跑 5/5）、tv=158/162（4 预存 cookbook）——745 新增失败 0
- `blockers`: 无。注：shell 观察 PE 退出码的 256 取模现象（bash $? 截断）已在报告/测试注释注明，测试用真实 status.code()
- `next_responsible_role`: review（T-08 按新完整 HEAD dc5e95be7 第四轮最终复审，同树、实现写入暂停；对照 review-r3/REVIEW.md 逐 R3-QA 复核）

2026-10-06 stage=work（r2 修复轮），PLAN-745 r1 修复交接记录（第二轮）：

- `stage`: work（r2 needs_fix 修复）| `plan_revision`: 1（语义合同未变）
- `outcome`: **pass**（r2 修复轮内部检查；R2-QA-01..05 全部处置，待 T-08 第三轮最终复审）
- `worktree_path`: D:/autostack/.wt/lang-745/auto-lang | `branch`: plan-745-dev | `code_commit`: **7437eb137a789259f01446da7a170e74031ff7f2**（树 clean）
- `dependency_revisions`: auto-down @ fba6563ed2148ce85e68208863159b4ccccac710（未变，只读）
- `repair_chain`: ad226453b（QA-01/02/03/05+oracle 缺口）→ cd9492f20（r2 报告记录）→ a830dac4b（门禁结果）→ 7437eb137（脚本行拼接修复重提交；注：一次未提交脚本修改曾被残留清理误删，已自 HEAD 恢复并重应用，门禁结果基于修复版有效）
- `finding_disposition`: R2-QA-01=动态行锚（D01=5/D02=6/D03=1/D04=6/D05=2、D07=7、C01/C03/D06 拒）；R2-QA-02=Rust run_deadlined 通道化+字节排空+单截止+kill回收，PS Invoke-Captured 事件化+树杀+预算封顶+无尾换行行拼接（后代持管道/无效UTF8 控制过，745脚本74例全过含P15精确"ba"）；R2-QA-03=Stray 组合不提升（D10/D13@1:27/1:42，D08/D14保留）；R2-QA-04=同类型越域 Bundle 变异（verify.uninitialized-read，逐变异断言改变）+三件真文件 publish 回滚（FileShare.None 持有 receipt，三件 byte-identical），报告分列两项 oracle；R2-QA-05=NEST_LIMIT=200 护栏（E01 过/E02 CLI exit1+source.syntax 无 panic，块同护栏）
- `gates`: ac-core 104 测/零警告/零 fmt；741 13 步、745 12 步 74 例、inventory strict 全过；主仓 check PASS、t=4936/4972（29 共现预存+7 隔离复跑 7/7 全过）、tv=158/162（4 预存 cookbook）——745 新增失败 0（口径含 §6 历史限度注记）
- `blockers`: 无
- `next_responsible_role`: review（T-08 按新完整 HEAD 7437eb137 重新最终复审，同树、实现写入暂停；对照 review-r2/REVIEW.md 逐 R2-QA 复核）

2026-10-06 stage=review，PLAN-745 r1 最终独立复审：**needs_fix**。

- `review_scope`: final；`reviewer_identity`: 原规划 Codex 对话，独立于 GLM 实现上下文；本轮未实施产品/委派/merge。
- `worktree_path`: D:/autostack/.wt/lang-745/auto-lang；`branch`: plan-745-dev。
- `reviewed_commit`: b3cd8171966281a42f96187780342ce0ff51c5fb；`base_commit`: eea7a80618d4d5e75749d666b7609a9a87925bbd。
- `dependency_revisions`: auto-down @ fba6563ed2148ce85e68208863159b4ccccac710；Spec/Plan/配置/binary hashes 见 [baseline.json](attachments/745/review-r1/baseline.json) 与 toolchain.txt。
- `acceptance_results`: AC01 partial、AC02 fail、AC03 partial、AC04 fail、AC05 fail、AC06 partial、AC07 partial、AC08 fail、AC09 pending。
- `findings`: P745-R1-QA-01..08，详见 [最终报告与修复要求](attachments/745/review-r1/REVIEW.md)：跨行边界、native符号冲突、诊断token、排除边界、测试进程截止、验收oracle漏项、盘点误归因、Checked后管线未共用。
- `evidence`: ac-core check/fmt PASS、92 passed/1 ignored；741 13步/745 12步74例 PASS；独立C01..22与大trace/三件SENTINEL控制。正确inventory strict47 PASS。root check PASS（382旧warning）；daily完整4943/4972通过、29失败；tv完整158/162通过、4失败，exit100，未称绿色。生产源/config零745 diff，历史红集对照有边界，不冒称严格同源逐项零新增。
- `spec_inputs`: 四项canonical Spec hash及冻结SD01..04副本见 [证据包](attachments/745/review-r1/REVIEW.md)；SD01/03整体能力与盘点说明待修后重新审定，不发布canonical/live ledger。
- `pending_closeout_gates`: T09主线整合/SD/人工层/ledger/index/导航/批回归到期判断/真实archive/guard与树清理，全部待最终pass与合入授权。
- `next`: work在**同一树**修复并提交；T02..07重开（包括依赖证据），T01保留完成，status=executing，current_step=1/total_steps=9。T08仍未完成，所有历史pass保留为历史；修后绑定新完整HEAD再最终复审，r1语义合同保持。

2026-10-06 stage=new，PLAN-745 r1：完成合同预检，目标/类型/支持/反例/SD/角色/任务覆盖一致。
未运行Rust/PE，T01..09均未执行，不是最终review pass。建树登记完成后的交接收据见worktree.md。
实际Git工作树已建并登记（创建提交c5e3875b67174d50193f8945fdfb1e3ec9eb1741，reparse点0，状态clean）。
stage=new / outcome=pass（规划交接完成）/ next=work T-01（待用户确认r1）；status=drafting，0/9。
本树不属于Codex应用托管树，登记以Git+本计划为准，不创建第二树；wt-guard.sh缺失由T09恢复，删除前须正式guard clean。
技能源=C:/Users/zhaop/.agents/skills/auto-plan-new/SKILL.md及references/verification-contract.md。

2026-10-06 stage=review，PLAN-745 r1 第二轮最终独立复审：**needs_fix**。

- `review_scope`: final；`review_round`: 2；`reviewer_identity`: 原规划/第一轮复审Codex上下文，独立于GLM实现与修复上下文；本轮只审查/证据/簿记。
- `worktree_path`: D:/autostack/.wt/lang-745/auto-lang；`branch`: plan-745-dev；`reviewed_commit`: 91b37560cffda75cf07cef8620cf524f5d6a3c7c。
- `base_commit`: eea7a80618d4d5e75749d666b7609a9a87925bbd；`repair_diff_base`: b3cd8171966281a42f96187780342ce0ff51c5fb；`dependency_revisions`: auto-down @ fba6563ed2148ce85e68208863159b4ccccac710（未变，只读）。
- `spec_inputs`: 四项canonical Spec、Plan、配置及冻结SD哈希见 [baseline.json](attachments/745/review-r2/baseline.json)；二进制/工具链见toolchain.txt，当前库探针由Cargo JSON唯一绑定。
- `acceptance_results`: AC01 pass、AC02 fail、AC03 partial、AC04 fail、AC05 partial、AC06 partial、AC07 partial、AC08 fail、AC09 pending。
- `findings`: **P745-R2-QA-01..05**，详见 [第二轮最终报告](attachments/745/review-r2/REVIEW.md)：括号内合法换行后续算误拒；新助手/PS采集截止及输出仍有缺口；Stray组合制造死块遮住真实错误；越域和三件publish补充oracle仍不足；4122字节未闭合输入栈溢出退出。
- `prior_disposition`: R1-QA02/04/07/08清偿；R1-QA01/03具体旧反例已修，本轮发现回归；R1-QA05/06部分修复，剩余项按R2追踪。保留具体通过证据，不假称原八项均未修，也不将内部“全部处置”当最终pass。
- `evidence`: ac-core check/fmt零warning、99 passed/1 ignored；741 13步/745 12步74例通过；正确inventory strict47通过。完整日常档4939/4972通过、33失败；tv158/162通过、4失败，均exit100。新hover红隔离1/1通过；未称根门禁绿色/严格同源逐项零新增。原C反例、D组合/E深输入、原样助手/脚本、真实Bundle同type越域控制及三件保留结果已持久化。
- `pending_closeout_gates`: T09主线整合、SD/人工层/ledger/index/导航、到期批回归、真实archive断言、guard与原树清理；本轮未执行、未发布canonical/live ledger。
- `next`: work在原树修复并提交新完整HEAD，再申请最终复审。T02..07及其依赖证据重开，T01保留，status=executing/current_step=1/total_steps=9；T08/T09未完成。r1语义合同保持，历史收据保留。
2026-10-06 stage=review，PLAN-745 r1 第三轮最终独立复审：**needs_fix**。

- `review_scope`: final；`review_round`: 3；`reviewer_identity`: 原规划/前两轮复审 Codex 上下文，独立于 GLM 实现与修复上下文；本轮只审查、验证、证据与簿记。
- `worktree_path`: D:/autostack/.wt/lang-745/auto-lang；`branch`: plan-745-dev；`reviewed_commit`: **7437eb137a789259f01446da7a170e74031ff7f2**。
- `base_commit`: eea7a80618d4d5e75749d666b7609a9a87925bbd；`repair_diff_base`: 91b37560cffda75cf07cef8620cf524f5d6a3c7c；`dependency_revisions`: auto-down @ fba6563ed2148ce85e68208863159b4ccccac710（未变，只读）。
- `spec_inputs`: 四项 canonical Spec、Plan、配置及冻结 SD 哈希见 [baseline.json](attachments/745/review-r3/baseline.json)；最终复核、实际工具链/binary/rlib 与证据清单见同包 final-baseline.json、toolchain.txt、artifact-manifest.json。
- `acceptance_results`: AC01 pass、AC02 fail、AC03 partial、AC04 pass、AC05 partial、AC06 partial、AC07 partial、AC08 fail、AC09 pending。
- `findings`: **P745-R3-QA-01/02（P2）**，见 [第三轮最终报告](attachments/745/review-r3/REVIEW.md)：执行器仍遗留后代/读线程，PS 静默窗口截断成功；深输入护栏误拒合法224字节括号输入，却不能阻止32794字节合法加法链栈溢出。
- `prior_disposition`: R2-QA01/03/04 已清偿；R2-QA02 字节读取/返回有界已修，但生命周期与输出 EOF 未满足；R2-QA05 原 E02 不闭合输入已修，新深度边界问题转 R3-QA02。R1 其它已清偿状态保持，不重新挂原问题。
- `evidence`: ac-core locked check/fmt 零 warning、104 passed/1 ignored；741 13 步、745 原命令 12 步74例通过；独立驱动74/74通过；inventory strict47通过；C/D/E与新F反例、原样两个执行器、真实 Bundle 同类型越域、三件 SENTINEL/真 publish 回滚、大输出均持久化。root check PASS（382旧warning），daily4944/4972通过、27失败+1超时；tv158/162通过、4失败，均exit100。dep018新超时隔离1/1通过；未称根门禁绿色/严格同源main对照。
- `harness_correction`: 初始 .NET 驱动向 PS5.1 传入 PS7 的 PSModulePath，导致哈希命令缺失；环境正反控制确认后校正驱动，按计划原生命令复跑745成功。初始/同环境复跑日志保留为作废环境证据，不记为实施缺陷。
- `spec_delta`: SD01..04 冻结副本保留；SD01/03 支持边界待 R3-QA02 修复后重新绑定。canonical/live ledger 未发布，MD408/409 不重hash掩漂移；完整 int/char 仍 open，不重开 GOAL-017。
- `pending_closeout_gates`: T09 主线整合、SD/人工层/ledger/index/导航、到期批回归、真实 archive 断言、guard 与原树清理；本轮未执行。
- `next`: work 在**同一树**修复两项、提交新完整 HEAD，再申请最终复审。T02/T03/T05/T06/T07 及依赖证据重开；T01 数值报告和 T04 共享 Checked 构建/CLI/事务保留完成。status=executing，current_step=2/total_steps=9；T08/T09仍未完成。plan_revision=1 保持；若确需资源支持边界变更，显式进入 new 合同修订，不自行弱化 r1。

2026-10-07 stage=review，PLAN-745 r1 第四轮最终独立复审：**needs_fix**。

- `review_scope`: final；`review_round`: 4；`reviewer_identity`: 原规划及前三轮复审 Codex 上下文，独立于 GLM 实现/修复上下文；本轮只审查、验证、证据与簿记。
- `worktree_path`: D:/autostack/.wt/lang-745/auto-lang；`branch`: plan-745-dev；`reviewed_commit`: **dc5e95be757dba647ef758aa283e7e2788015747**。
- `base_commit`: eea7a80618d4d5e75749d666b7609a9a87925bbd；`repair_diff_base`: 7437eb137a789259f01446da7a170e74031ff7f2；`dependency_revisions`: auto-down @ fba6563ed2148ce85e68208863159b4ccccac710（未变，只读）。
- `spec_inputs`: canonical Spec/Plan/配置/冻结 SD 哈希见 [baseline.json](attachments/745/review-r4/baseline.json)，终审稳定性与工具链/binary/rlib/证据见 final-baseline.json、toolchain.txt、artifact-manifest.json。
- `acceptance_results`: AC01 pass、AC02 fail、AC03 pass、AC04 pass、AC05 partial、AC06 partial、AC07 partial、AC08 fail、AC09 pending。
- `findings`: **P745-R4-QA-01/02（P2）**，详见 [第四轮最终报告](attachments/745/review-r4/REVIEW.md)：PS 新执行器 Job 分配失败泄漏挂起进程、真实读取错误被当EOF成功；语法护栏计数修正后仍保留未批准的支持上限，426字节200层合法括号被source.syntax拒绝。
- `prior_disposition`: R3-QA01 的 Rust 生命周期原反例及 PS 静默窗口原反例清偿，新增错误出口转R4-QA01；R3-QA02 原16384项崩溃/99-100层括号误拒清偿，未完成边界合同子项转R4-QA02。更早已清偿问题未退化。
- `evidence`: ac-core check/fmt零warning、112 passed/1 ignored；741 13步/745 12步74例通过；独立74/74、strict inventory47通过。Rust/C#持管道、迟到输出、父子超时树杀与UTF8控制通过；真实Win32分配失败的未改cleanup仍留进程，真实ReadFile错误6仍返回Code0/空输出；当前rlib真实Bundle同类型越域拒绝、三件事务、大输出、C/D/E/F及G边界native300/70空stderr/24baab全部持久化。
- `root_gates`: check PASS（382旧warning）；daily4937/4972通过、35失败；tv158/162通过、4失败，exit100。其余34日常红名在历史集合中，唯一新名plan643隔离1/1通过。生产/config/lock零745 diff；未称根门禁全绿或严格同源码main对照。
- `spec_delta`: SD01..04冻结提案与实现一致；SD01/03支持边界须在R4-QA02处置后重新绑定。canonical/live ledger未发布，不重hash人工层掩漂移，完整int/char仍open，不重开GOAL-017。
- `pending_closeout_gates`: T09 主线整合、SD/人工层/ledger/index/导航、到期批回归、真实archive断言、guard与原树清理；均未执行。
- `next`: work在**同一树**修复并提交新完整HEAD，再申请最终复审。T02/T05/T06/T07及依赖证据重开；T01数值、T03真实Checked管线/迭代修复、T04共享构建/CLI/事务保留完成；status=executing，current_step=3/total_steps=9，T08/T09未完成。r1暂保持；如保留资源上限须显式进入new合同修订，不继续宣称语义未变。

## 10. 待澄清事项

r1等待用户按AGENTS L1确认数值profile/支持集后实施；本轮范围没有其它未定义的验收结果。
T01为有界事实核查，不得用unknown扩大语言面；事实冲突须说明并修合同，禁止隐式fallback。
候选②/str/List/完整运算/聚合/ABI/ACC迁移保持后续未编号工作，不在本计划承诺交付。
