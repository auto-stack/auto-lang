---
plan_id: PLAN-961
status: execution_done
feature_name: 完整数值源码与四端一致语义
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 2
current_step: 5
total_steps: 8
depends_on: [PLAN-959]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-numbers.md]
touched_goals: [GOAL-002]
---

# [PLAN-961] 完整数值源码与四端一致语义

## 0. 变更摘要

将实际i8/u8/byte/u32/i64/u64/float/double字面量、转换、比较、位运算和宽度引入统一源码/HIR/MIR与四端；int保持已裁定宽度，禁止以Rust默认宽度补语义。浮点NaN、负零、舍入/溢出逐操作冻结；保现行checked trap相位。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

依赖：PLAN-959。依赖指正式交付来源及适用接口，不把WIP/reviewed当delivered；不以最终认证尚未交付阻塞独立能力建设。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `experimental/ace/crates/core/src/source`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/av-core/src`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/native/src`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/a2x-c/src`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/a2x-rust/src`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

## 5. 详细设计

将实际i8/u8/byte/u32/i64/u64/float/double字面量、转换、比较、位运算和宽度引入统一源码/HIR/MIR与四端；int保持已裁定宽度，禁止以Rust默认宽度补语义。浮点NaN、负零、舍入/溢出逐操作冻结；保现行checked trap相位。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：完整标量运算反例（r2）

补移位、符号扩展、混合宽度、除余、浮转整、字面量舍入与NaN payload政策；以原规范决定，不从宿主默认推断。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-numbers.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |

## 6. 测试设计

以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | i64 2147483648往返不截断 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | u64高位比较正确 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | MAX+1按合同trap而非C UB | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 短路未执行分支不trap | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | NaN比较和-0由独立bit/值oracle判定 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | MIN/-1、负移位/等于位宽移位、边界浮转整分别按冻结诊断或trap；副作用发生在合法源序，C不得落入UB。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 完整标量运算反例的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |

## 8. 执行步骤

预定唯一实施树 `D:/autostack/.wt/lang-961/auto-lang`，分支 `plan-961-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

- [x] **T-01｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。
- [x] **T-02｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。
- [x] **T-03｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。
- [x] **T-04｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。
- [ ] **T-05｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。
- [ ] **T-06｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。
- [ ] **T-07｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-961/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。

- [x] **T-08｜owner_stage: work｜AC-06**：补移位、符号扩展、混合宽度、除余、浮转整、字面量舍入与NaN payload政策；以原规范决定，不从宿主默认推断。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。

## 9. 复审记录

2026-11-07 work checkpoint（r2，/goal 授权实施）：T-01 冻结完成——语义/支持/owner/命令决定表已固化于
`D:/autostack/plan-artifacts/plan-961/t01-freeze.md`（§1字面量/§2运算/§3转换/§4 MIR表示/§5四端owner/§6资源/§7策略比较/§8具名差异），策略采用 Num{op,ty} 结构化单变体（B 方案）。依赖 959 archived 已核；worktree `D:/autostack/.wt/lang-961/auto-lang`（base fcc474acd）。T-02 实施推进（MIR/HIR/中端层落地，前端/后端发射待续）：
- `7869ee281` av-core：MirType I8/U8/U32（wire 19/20/21）、MirConst/Value/布局、BinOp::Num（subcode 12+op/ty 字节）、UnOp::NegT（subcode 3）、MirInstr::Conv（0x3A）、abc 编解码数据族门、mir_verify Num/NegT/Conv 规则（EqI32/NeI32 收紧为 i32/bool 对——原宽型准入缝具名关闭）、TrapKind +ShiftOutOfRange/FloatToInt/FloatOverflow、AV eval_num/eval_conv 全矩阵（f32 原生精度、移位量界检查、checked 窄化）。`cargo test -p ace-av-core` 59/59 绿。
- `8db479bde` ace-core：BuiltinType/TypeKind 数值七型 + 判别值 10..16、ValueType 数值型、facts/snapshot/text/descriptor 类型表全走读。`cargo test -p ace-core` 29+359 绿。
- `455657e9b` ace-core/lowering/a2x/native/artifacts：HIR NumOpKind/BinOp::Num/UnaryOp::NegNumeric/ExprKind::NumericConst+Cast、verify 主类型走读（同型纪律/比较→bool/位运算拒浮点/无符号取负拒绝）、lowering→MIR 1:1（Num/NegT/Conv、数值常量位精确）、a2x-core Ty 数值七型（Rust 拼写直映）、a2x-c/a2x-rust 类型渲染全通+新算子具名拒绝（`a2x.unsupported-op`，数值发射属下一单元）、native 窄 slot/Num/NegT/Conv 具名拒绝（数值后端单元待续）、artifacts codec 冻结子集外具名拒绝。`cargo test -p ace-lowering` 5 绿；`-p ace-native` 存在 1 例与本包无关的既有红（mn-collections-worker allocator meter 测试，本包 diff 不触其任何路径，base 验证被在途依赖阻断，具名留证 base-worker-test.log）。
未完（T-02 续）：numeric.atom schema+descriptor 解析臂（constant 后缀集/cast/typed op 名）、源端 lexer/ast/parser/resolve/adapter/CST 语法路径、semantics Ty 与 source-query PortableTy、native CLIF 数值发射、a2x-c/rust checked helpers+发射、T-08 边界 fixture、T-03/T-04。共享 no_std 运行时三仓在裸 `cargo check --workspace` 下 panic-abort profile 特性红为既有状态（AGENTS.md），非本包回归。
status 保持 executing；current_step 0/8（任务级完成以 T-01 冻结+探针实证后勾选，见 §8 注记待补）。

stage: goal | plan_id: PLAN-961 | plan_revision: r2 | round: 1 | from: work | to: work | status: executing | outcome: pass(部分推进,未到handoff) | worktree: D:/autostack/.wt/lang-961/auto-lang | commit: 455657e9b | next: work

2026-11-07 work checkpoint round 2（r2）：descriptor/schema 单元完成并提交——
`ba0a047f4`：`crates/core/schema/numeric.atom`（closures 超集：107 个 `{op}_{ty}` typed BinOp 名、
neg_i8/i64/f32/f64、数值 TypeKind 七型、expr.cast/numeric_const、to: TypeUse 字段、add/sub/mul 型
branch 强制 overflow）；lib.rs `numeric_descriptor()`；descriptor 解析臂（constant 数值后缀集路由
NumericConst 位精确负载、cast 词、typed op 名解析、typed neg 词、数值 builtin type 词——
`Descriptor::is_numeric()` 身份门，数值词不入旧面）；canonical 写读对齐（num_op_atom/atom_word）；
verify eval-position 边补 Cast/NumericConst；探针 fixture numeric-hir/n01 + 4 测试绿
（schema 词表/绑定验证/canonical 不动点/混宽拒绝）；`cargo test -p ace-core` conformance 363/363 绿
（修复一处本单元引入的 i32 后缀越界静默丢弃回归——3000000000i32 负例恢复 bind.literal-range）。
关键架构事实入档：生产前门是 CST 链（parse_program_via_cst），手写 parser 仅差分 oracle——
数值源端须落 grammar+projection，精确七步执行清单见 plan-artifacts/unit-frontend-plan.md。
未完：源端 CST/grammar/投影/ast/resolve/adapter（清单 1-7）、semantics/source-query、native/a2x 发射、
T-08/T-03/T-04。status 保持 executing，current_step 0/8。

stage: goal | plan_id: PLAN-961 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: pass(部分推进,未到handoff) | worktree: D:/autostack/.wt/lang-961/auto-lang | commit: ba0a047f4 | next: work

2026-11-07 work checkpoint round 3（r2）：源端前门前两切片完成并提交（conformance 366/366 绿×2）：
- `95df1bccd` 前门切片：grammar `numeric` profile（amp/vbar/caret 与 10 数值类型关键字 token 均
  `profiles: [numeric]` 门控——旧面 scanner 字节恒等，三处 parity 回归（differential_edge/`&`、
  n22-i64-type、t09 catalog grammar-id）已修复归绿）；numeric-ops pratt 表（& | ^ 同层）；后缀步
  `.as(T)`/`.to(T)` r-as/r-to 门控；SourceProfile::Numeric 全链接线（slug/grammar 槽/parse 臂/
  descriptor 选择 profile_descriptor）；ast TypeWord/VT 数值七型 + 投影 classify_word 分类。
  **冻结增补**：移位/转换均不入 pratt（pratt 算子右操作数仅表达式、`>>` 与嵌套泛型闭合冲突）——
  `.shl(n)`/`.shr(n)`/`.as(T)`/`.to(T)` 方法形承载；中缀 `as` 具名延后（TyWord-atom 技法）。
- `170f45d39` typed 字面量切片：affix-num 解码器按 T-01 §1 冻结表（0x/0b、`_`、后缀全表、
  f32/f64 目标宽度正确舍入 std 解析、溢出→source.literal-range）；ast/resolve/adapter TypedLit
  位精确 NumericConst 发射 + numeric 身份三元组与 7 数值类型行。**全链探针绿**：
  `fn f() i64 { return 1i64 }` 经真实 CST 前门 parse→resolve→typecheck→emit→bind→verify_closures
  落成 HIR `NumericConst(I64,1)`；越界/词表负例绿。
未完（前端余量）：负字面量折叠与数值算子/位算/移位/转换方法在 resolve 的类型规则与 adapter 发射
（EKind::Bin(Num)/Cast 与方法分派 .shl/.shr/.as/.to）、semantics/source-query、native/a2x 发射单元、
T-08/T-03/T-04。status 保持 executing，current_step 0/8。

stage: goal | plan_id: PLAN-961 | plan_revision: r2 | round: 3 | from: work | to: work | status: executing | outcome: pass(部分推进,未到handoff) | worktree: D:/autostack/.wt/lang-961/auto-lang | commit: 170f45d39 | next: work

2026-11-07 work checkpoint round 4（r2）：源端算子面 + AV 执行语义完成并提交（`fd7193d60` + `e65ec7142`）：
- `fd7193d60` 算子面：resolve 同型纪律（混宽→source.type-mismatch；i32/bool 旧路径诊断字节不变——
  typed 分支仅非-I32 数值触发）、位算 & | ^（ast/pratt/投影/typecheck）、`.shl/.shr` 移位与
  `.as/.to` 转换方法分派（resolve_method 数值守者臂→BuiltinMethod::Shl/Shr/NumTo）、负字面量折叠
  （含 -2^63i64 与浮点符号翻转）、adapter Num 词/cast/typed-neg 发射。numeric_source 9 测试 +
  conformance 368/368 绿。
- `e65ec7142` AV 执行：**全链 V 行探针绿**——source→compile_numeric_source→lower_checked_closures
  →mir_verify→AVAD→execute：V-01 i64 2147483649 不截断、V-02 u64 无符号比较、V-03 MAX+1/u8 溢出
  trap、V-04 短路潜伏 trap、V-05 -0.0/NaN 位精确、V-08 移位越界/移位量=位宽/MIN/-1 div+rem/除零/
  浮除零/窄化越界/浮转整 NaN+越界/f64→f32 溢出/符号扩展/截断向零/位算值。配套：f32-kw token 补齐
  （含 ret/ann/type-arg/optional-inner 四位）、minus 感知 i64-MIN 折叠、closures 族入口契约
  （标量参数/结果——SD-01 记录）与数值常量门放行、语义/工件/源查询三通道数值走读（具名拒绝或
  规范字节）。两枚冻结测试按 961 合同更新（SPECS 55 含 CONV；宽算合法化后 AddI32×i64 拒绝码为
  mir.bin-type）。av_numeric 8/8 + core 368/368 绿。
- 已知非本包红（具名留证 av-conf3.log）：av conformance 全量在 av_collections_semantics
  deep/small-stack 系测试 STATUS_STACK_OVERFLOW（16/17 绿后崩，串行亦然；本包 diff 不触
  collections/place 执行路径；PLAN-939 在途面）；closures_recovery perf fresh P90 4905ms>2000ms
  门（本会话重负载下的计时门，命中/重启腿全绿）。二者按环境归因记录，T-04 复核时隔离重放。
未完：native CLIF 数值发射、a2x-c/rust 数值发射+运行时、semantics 通道深度（当前具名拒绝面）、
T-08 fixture 固化进仓、T-03/T-04。status 保持 executing，current_step 0/8。

stage: goal | plan_id: PLAN-961 | plan_revision: r2 | round: 4 | from: work | to: work | status: executing | outcome: pass(部分推进,未到handoff) | worktree: D:/autostack/.wt/lang-961/auto-lang | commit: e65ec7142 | next: work

2026-11-07 work checkpoint round 5（r2）：native CLIF 数值发射单元完成并提交（`168db3fd0`）：
- LowerMode::Numeric（= Legacy 行为 + 数值守者/算子；无数值程序对象与 Legacy 恒等）：i8/u8/u32/f32 骑 I32 守者、
  i64/u64/f64 骑 I64；Num{op,ty} 全矩阵——整数加/减/乘溢出用 room 技术与高半检测（smulhi/umulhi，MAX+1=MIN 型
  回绕不可由结果范围查出——T-04 复核锚）、除/余零与 MIN/-1 预守卫、移位量界检查（<0 或 ≥ 位宽 trap；值位丢弃）、
  位算与符号/无符号比较按型；浮点 IEEE 逐操作（÷±0 trap 保 legacy 相位、% 走 CRT fmod 导入、NaN/-0 位精确经
  bitcast 守者往返）；NegT（signed checked/浮点翻符号）；Conv 冻结矩阵（整↔整 checked 窄化/符号零扩展、整→浮
  最近舍入、浮→整 NaN/±Inf/范围检查后向零截断、f64→f32 有限溢出 trap）。
- 前门修复伴随：numeric 算子层进 CST 走读 prec_of（& | ^ 层位 5）、ctor-expr 从 numeric 原子面移除（962 聚合面
  归属；消除 ident-后-{ 的 ctor 提交吃掉 if 块——closures 面预存缝具名记录 CI-closures-int）、affix 原始拒绝对
  numeric 全关、i32 MIN 量级负折叠泛化到全部有符号宽度。
- native lane：driver 末位升级 Numeric profile（& | ^ 字符的窄面 lex 错按面事实升级）、closures/collections 族
  进 emit 准入（823 冻结排除由 SD-01 取代——closures_compat/data_owned 两个冻结负例更新为 961 合同）、slot_ty
  硬化不 panic。
- 证据：numeric_codegen_tier 4/4 绿（i64 幅值/溢出/除余、u8 宽度算术、移位界、i32 位算、浮点与转换边界——经
  真实 COFF+PE 执行独立 exit 断言）；core numeric 9/9、av numeric 8/8 复绿；两枚 961 合同负例绿。native 全量
  conformance 的 resources 两性能腿在并行满载下红、隔离绿（环境归因，同 round 4）。
未完：a2x-c/a2x-rust 数值发射（仍具名拒绝占位）、semantics 通道、T-08 fixture 入库、T-03/T-04 六职责组并集。
status 保持 executing，current_step 0/8。

stage: goal | plan_id: PLAN-961 | plan_revision: r2 | round: 5 | from: work | to: work | status: executing | outcome: pass(部分推进,未到handoff) | worktree: D:/autostack/.wt/lang-961/auto-lang | commit: 168db3fd0 | next: work

2026-11-08 work checkpoint round 6（r2）：a2x 双 provider 数值发射单元完成并提交（`97fc27306`）——第四端闭环：
- a2x-rust：RT_NUMERIC 运行时（checked 各宽度整算术/div/rem 守卫/移位界/fmod/转换矩阵 + 位算 + 字面量构造器，
  InvalidScalar trap 类），lower/lower_coll 的 NumericConst/Cast/Num{op,ty}/NegNumeric 真发射，typed literal 通道。
- a2x-c：runtime.rs 生成器扩为按宽度/符号生成全套 helper（_mul128/_umul128 高半检测、i64/u64 room 检查、
  窄宽度域上范围检查、unsigned 源无下界检查防 C 符号比较翻转、移位量 A2X_TRAP_INVALID、float→int goto bad
  域检查、f32↔f64）；typed result 通道结构 + 按函数结果类型的陷阱回传通道 + W4/WX 清洁的显式窄化 cast；
  lower/coll/rt 三路径全发射。
- 通道接线：a2x-core（Closures 域算子/常量/转换准入 + prepare 规划/序列化走读 + profile_of numeric）、
  a2x-store（Numeric profile 冷装配）、sha256 双 pin（admitted_text + abi CERTIFICATION_C_RUNTIME_SHA256）
  按新运行时字节重钉；921 金样三套按新真实发射重冻结。
- 证据：worker true chains —— Rust(rustc) 12/12 绿（i64 幅值/越界、u8 宽度、移位界/值、MIN/-1、浮点÷±0、
  窄化 trap、符号扩展、截断向零、f64→i64 域、位算）；C(MSVC cl /W4 /WX) 12/12 绿（含修 float.h 包含、
  复合字面量类型、int64→int32 显式窄化）；921 金样 PASS；a2x-c/rust/core/abi 套件绿（abi 并行文件竞争
  串行绿——环境归因）；worker 全量 152+7+26 绿。
未完：T-08 边界 fixture 固化入库（V 行已在 av/native/a2x 三端探针中但未作为仓内 fixture）、T-03 消费者接线
梳理、T-04 六职责组并集验证。status 保持 executing，current_step 0/8。

stage: goal | plan_id: PLAN-961 | plan_revision: r2 | round: 6 | from: work | to: work | status: executing | outcome: pass(部分推进,未到handoff) | worktree: D:/autostack/.wt/lang-961/auto-lang | commit: 97fc27306 | next: work

2026-11-08 work handoff（r2，T-04/T-08 完成，全部 work 任务收口）：
- **T-08 完成**（`56559edfb`）：字面量舍入 fixture（0.1f32/0.1d 最近舍入经全链位精确观测）；至此 T-08 全家族
  （移位界/值、符号扩展、混合宽度、除余边界、浮转整 NaN/±Inf/域、窄化、截断向零、fmod、字面量舍入、NaN payload）
  均有仓内永久 fixture + 真实命令（av_numeric_semantics 9 测试、numeric_codegen_tier 4 测试、
  numeric_semantics 双 provider 24 断言、numeric_source 10 测试、numeric_hir 4 测试）。
- **T-03 完成**：四端真实消费者已接线且无 legacy fallback——AV（AVAD wire+execute）、native（CLIF→COFF→PE 执行）、
  a2x-rust（文件集→rustc 编译→运行）、a2x-c（文件集→MSVC cl →运行），全部经 numeric 源 profile 真实 dispatch，
  driver 末位升级与 store profile 门已在 round 5/6 落地。
- **T-04 六职责组并集已执行**（t04-six-groups.log）：`cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime`
  单次调度——**PASS：ace-source、ace-lowering、ace-a2x、ace-runtime**；FAIL 两组均为具名既有环境红，非本包触面：
  ace-av 三 ABORT 全部是 av_collections small-stack/partial_clone 回滚族栈溢出（round 4 起具名、PLAN-939 在途面，
  conformance 后续 av_core 1044/1044 续跑绿）；ace-native 单 FAIL 为 turnaround concurrent_gc_30 GC racer
  并行负载抖动——隔离重放绿（62.45s，本轮实测）。数值面全部测试在各自组内绿（round 4-7 多轮独立证据，输入未变）。
- **共享通道现状**（work 责任内边界，均已具名）：semantics 查询通道保持有限推断 profile（数值注解=具名拒绝
  numeric-annotation——其 infer 面扩容属后继计划）；CTE 值面维持 I32/Bool（冻结记录）；comptime av_driver 数值
  常量执行面经 round 1 av.rs eval_num 全矩阵已可执行（comptime admission 未扩——具名）。
- 交付物清单：12 个实现提交（7869ee281..56559edfb），~10.4k 行净增/90 文件；四端语义一致（同一 T-01 冻结矩阵）；
  金样/SHA pin 按合同变化重冻结并记录。
- 未完（review/merge 职责）：T-05 独立复审（AC-04，含混宽/浮点/转换抽查与本收据全部声明核对）、T-06/07 merge
  与归档清理（AC-05）。规范增量 SD-01（m2-numbers spec 待落）/SD-02 待 merge 阶段沉淀。

stage: work | plan_id: PLAN-961 | plan_revision: r2 | outcome: pass | worktree: D:/autostack/.wt/lang-961/auto-lang | branch: plan-961-dev | base_commit: fcc474acd | code_commit: 56559edfb | task_ids: T-01,T-02,T-03,T-04,T-08 (T-05/06/07 review/merge-owned) | evidence: t01-freeze.md + t04-six-groups.log + 各轮测试记录 | blockers: 无 | next_responsible_role: review (T-05)


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
