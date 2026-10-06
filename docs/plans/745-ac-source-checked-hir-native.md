---
plan_id: PLAN-745
status: executing
feature_name: Auto 源码计算子集 → Checked HIR → Windows native
author: [Codex]
created_at: 2026-10-06
updated_at: 2026-10-06
plan_revision: 1
current_step: 5
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

唯一任务表，全部[ ]，total9/current0；new起草/建树不算实施完成。

| ID | 状态 | owner_stage | 依赖 | 文件/符号与可验证产出 | AC / 验证 |
|---|---|---|---|---|---|
| T-01 | [x] | work | r1确认 | 新docs/reports/745-source-core-i32/numeric-boundary.md；核对可见值/parser/trans/char路径，完成两probe与MD408/409提案；若事实推翻profile可行性先修合同 | AC01,07；数值cases+决策来源，先于T02/03类型实施。〔2026-10-06 work：报告落地（worktree commit c535b8fd6）；23项输入hash零漂移；E1–E12证据链（VM栈i32/值存储i32\|i64/旧AST幅值三态/trans i64/char三宿主表示）；probe-int-width=profile内精确有符号32位、probe-char-int=profile拒绝char，MD408/409提案文本入报告§8；无事实推翻r1可行性，§7数值cases待T-05运行回填〕 |
| T-02 | [x] | work | T01 | 新src/source/{mod,lexer,ast,parser}.rs、lib.rs导出、fixtures/source/、tests/source_frontend.rs；grammar/span/full-consumption可运行 | AC02,04；ac-core check；cargo test --manifest-path experimental/ac-core/Cargo.toml --test source_frontend（新目标）。〔2026-10-06 work：worktree commit 6965c0a04；21正/53反fixture精确物化；12前端测试全绿（30 text级code+token断言、23 bind/verify级前端须通过断言、P01/P02/P04/P07/P11/P15/P20结构span、N03行列2:24、N31 1:24）；ac-core全量测试通过、fmt clean〕 |
| T-03 | [x] | work | T02 | 新src/source/{resolve,typecheck,adapter}.rs，签名/词法栈/精确类型、生成Atom+map→bind_source→全部span投影→verify；tests/source_hir.rs | AC01..04；新source_hir测试、owning/bindings与负变异。〔2026-10-06 work：worktree commit 3fb6c96b6；resolve（签名预登记/块作用域/参数映射/trace intrinsic优先级）+typecheck（source.type-mismatch@verify）+adapter（while→loop/if降低、bool=0<1/0<0合成、return后死代码独立块、stray break/continue phantom loop、条目序span再锚+token碎片回退）；10测试全绿（23负例stage/code/token精确、P03/04/11/15/19结构、4变异拒收）；全ac-core 80测通过零警告零fmt问题〕 |
| T-04 | [x] | work | T03 | src/main.rs新命令/共用Checked构建；source/mod.rs显式profile/trace表；必要link.rs收据追加保持事务；tests/source_cli.rs | AC03,04,06；新source_cli+旧cli/native/trace tests。〔2026-10-06 work：worktree commit c5af427d9；check-source/build-source（漏/未知profile=2、0/1/2分级、entry按函数名查def id）、link.rs新增publish_artifacts_with_receipt（源码profile/hash同事务入.ac-link.txt，旧函数零语义变化委托）；10 CLI测试全绿（真实PE构建运行P01=14/P13=70、失败零制品、entry.not-found/signature、trace无support-lib链接失败、旧check仍仅Atom）；全ac-core 90测通过零警告〕 |
| T-05 | [x] | work | T04 | 新tests/source_native.rs、scripts/verify-ac-source-745.ps1，物化执行JSON全部case和§6补充；docs/reports/745-source-core-i32/verification.md及原始收据 | AC01..06；ac-core全test+新一键脚本，60s/code/trace/token/hash/制品。〔2026-10-06 work：worktree commit addcf4d5a；脚本实测 **all 12 steps PASS (74 cases)**（21正例真实构建+运行退出码/stderr oracle全中含P15=12/stderr ba、P13/P14/P16=70、60s硬截止、三制品+sha256收据；53负例exit 1+stage/code标记全中）；source_native 2测试串行7.5s；§6补充矩阵九行证据映射入verification.md §5；主仓门禁留T-07回填〕 |
| T-06 | [ ] | work | T05 | ac-core/README.md，proposed-spec-delta.md及MD408/409更新提案/新鲜绑定策略，不直接改canonical | AC07；python scripts/acc_inventory.py --check --require-decisions，提案说明待merge绑定，不自动重hash掩漂移 |
| T-07 | [ ] | work | T06 | 同树内部检查所有cases/SD/健康，§6全部门禁，提交HEAD与执行收据；work7/7→execution_done（总7/9） | AC01..07；内部pass仅交接review |
| T-08 | [ ] | review | T07 | /auto-plan:review独立上下文同树，写入暂停，revision/HEAD稳定，逐AC实际证据/反例/债务/proposed delta；最终报告 | AC08并独立重验AC01..07；pass→reviewed（8/9） |
| T-09 | [ ] | merge | T08+合入授权 | /auto-plan:merge到v0.6-dev；SD/人工盘点/ledger/导航/index/归档；批回归到期判断；wt-guard clean才清理 | AC09；实际链接/终态/9/9/清理收据，不预勾 |

src/tests新路径均相对experimental/ac-core；reports新路径相对docs/reports/745-source-core-i32。
未来实现/最终review不得以计划表的勾选代替证据。

## 9. 复审记录

2026-10-06 stage=new，PLAN-745 r1：完成合同预检，目标/类型/支持/反例/SD/角色/任务覆盖一致。
未运行Rust/PE，T01..09均未执行，不是最终review pass。建树登记完成后的交接收据见worktree.md。
实际Git工作树已建并登记（创建提交c5e3875b67174d50193f8945fdfb1e3ec9eb1741，reparse点0，状态clean）。
stage=new / outcome=pass（规划交接完成）/ next=work T-01（待用户确认r1）；status=drafting，0/9。
本树不属于Codex应用托管树，登记以Git+本计划为准，不创建第二树；wt-guard.sh缺失由T09恢复，删除前须正式guard clean。
技能源=C:/Users/zhaop/.agents/skills/auto-plan-new/SKILL.md及references/verification-contract.md。

## 10. 待澄清事项

r1等待用户按AGENTS L1确认数值profile/支持集后实施；本轮范围没有其它未定义的验收结果。
T01为有界事实核查，不得用unknown扩大语言面；事实冲突须说明并修合同，禁止隐式fallback。
候选②/str/List/完整运算/聚合/ABI/ACC迁移保持后续未编号工作，不在本计划承诺交付。
