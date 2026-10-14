---
plan_id: PLAN-771
status: archived
feature_name: AC 标量 record、字段与方法首批原生编译
plan_revision: 2
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-14
current_step: 9
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-hir/project.md
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-acc/project.md
new_spec_components:
  - docs/specs/auto-ac/source-record.md
touched_goals: []
affects: [auto-ac, auto-hir, auto-acc]
---

# [PLAN-771] AC 标量 record、字段与方法首批原生编译

## 0. 变更摘要

在772修复后scalar管线上实现显式record profile：名义record、i32/bool/非递归嵌套record字段、位置/命名构造、字段读写、聚合参数/返回、static new、只读实例方法以及显式mut自由函数参数。
依赖[774共同合同](774-ac-data-resource-contract.md)，与[770文本](770-ac-text-char-native.md)无实现依赖，可在独立工作树并行。
方法首批不包含可变实例方法；这与旧P/CG的可变方法写法有差距，明确留给后续扩面，不宣称完整P/parser已可迁移。

## 1. 目标

1. 用名义类型与完整字段place模型支持有限值语义record，完整源码/HIR→native正反闭环。
2. 构造、调用、返回与方法receiver保留源码求值序、可变性与来源；未知引用/环/别名冲突真实verify拒绝。
3. 实现774内部聚合view/mut/out-return规则，以实际PE证明布局/调用，而非输出Rust文本。
4. 各独立profile精确隔离，旧计算能力与发布/执行器行为保留。

非目标：str/Str/char字段、List/enum/Option/Result、泛型/继承/接口/ext/动态分派、递归含指针类型、move-only/堆record、可变实例方法、use/跨模块、全局变量、生产ABI/DLL/热重载、完整ACC词法/解析/自举、旧编译器重构。

## 2. 架构方案

source-record→声明预登记/名义解析/字段和receiver类型检查→record Atom与来源表→真实bind/verify→Checked→native内部聚合布局/调用→COFF/PE。
HIR保留RecordId/TypeId、字段声明序、字段访问路径与模式；native计算目标offset/临时槽/sret，不在公共HIR放CLIF值或寄存器。
所有参与计算的类型必须闭合；结构相同不同名的record不互换。

## 3. 技术栈

沿用独立ac-core Rust/Cranelift0.126.2、Windows x64、既有link/contained executor与trace support。
不引入旧Value/VM，也不要求770 runtime；字段全部无堆资源，支持safe值语义。

## 4. 需求分析与背景调查

规划基线`540dc7d30839eb7df8e68b4d19472358e170fe5b`，证据见[输入表](attachments/771/planning-inputs.tsv)；标量前置772已archived（9/9）；共同前置774仍drafting（0/8），不能当已交付合同。
Specs auto-hir现仅i32/bool/function与local place；源TypeWord仅int/i32/bool，真实实现必须扩名义类型，不能靠将record装进i32或复用Unknown。
743 MD403/404、lexer Token五字段与parser P/CG/static new证明需要；本计划只交付标量子集，不覆盖它们的enum/text/List字段。
语法取自Design10的type/static fn/fn/self/.field及Design04 mut参数与调用模式；T01必须核对既有真实parser和声明语义，不能发明Rust impl写法。
现有修改点：source七模块、hir/descriptor/verify/native/main；新schema/record.atom、tests/record_{hir,source,native}.rs、fixtures/record-{hir,source}、reports/771-record。
仅新Plan/设计授权，尚无实施/合入/agent/model/预算/续跑授权。canonical record Spec尚未存在，774合同尚待产出。

- 2026-10-13引用/交接核查基线：主检出 d777ccbb227c41882ad5f6976c803b1e03b61135。编号映射为标量772、共同契约774、文本770、record771；770/771未改号。历史输入表保留来源，当前指纹见 [引用核查输入表](attachments/774/reference-update-inputs-20261013.tsv)。本次仅修正引用/取材和实施入口，不修改语言支持、验收阈值或执行状态。
- 2026-10-14 实施授权与开工（work 会话）：用户下达「计划771:实施」指令，授权按 r2 全文实施 T-01..07（work-owned）。前置核对：772 archived（9/9）、774 六轮复审 pass 并已合入归档（2026-10-14，master 930d05fae）；770 execution_done（7/9）在 lang-770 树待复审，与本计划无实现依赖。实施基线 master@199d6509e；工作树 `D:/autostack/.wt/lang-771/auto-lang`、分支 `plan-771-dev` 已建（组含 auto-down detached 兄弟）。T-01 以实施时最新基线刷新输入。

## 5. 详细设计

### 5.1 身份与类型

候选 source=`auto.source.record.draft`；HIR=`auto.hir.record.draft` / 1 / `record-i32-draft`（774 r2 同步：唯一身份表见 774 interface-contract §7.2，2026-10-13）。三元组精确注册、binder/Checked/writer/receipt一致；text类型不能从相邻分支漏入。
record名义身份至少包含本模块与声明标识；位置相同或结构相同不构成类型相等。字段i32/bool或已声明record；字段类型包含图必须无环（自环/互环/断开环全部核对）。
支持非递归嵌套，不临时加语法深度排除；声明顺序独立于名字解析依赖顺序，前向类型可预登记后检查闭合。

### 5.2 源码、构造与字段

支持`type Pair { left int; right bool }`等现行声明（示意中的分隔符按真实Auto语法物化）；位置构造`Pair(1,true)`与命名字段构造按T01核对的现行拼写。
字段数/名字/重复/类型/访问归属检查；命名构造按源码序先求值，再映射声明字段，禁止排序字段改变副作用。空record首批显式拒绝；默认字段值/默认构造参数不在范围。
局部值、赋值、函数参数与返回保留record值语义：`var b=a; b.field=...`不能改变a。读字段是rvalue，写字段是typed place，nested路径按实际字段owner逐段验证。
全部字段在构造时初始化；view/let不可写，var与显式mut入口允许写。分支初始化规则不放宽。

### 5.3 方法与调用

`static fn new(...) Pair`无receiver；普通实例方法绑定只读self（可用`self.field`与`.field`），接收者只求值一次，随后显式实参按源码序求值。
本轮显式`mut`自由函数参数按774核对的合法声明/调用拼写，禁止把同名字段或局部变量猜成self；readonly实例方法写字段受控拒绝。
所有方法首批有显式非unit返回类型及return；unit/隐式尾返回扩面不偷偷混入。
实例/静态成员、字段/方法同名冲突和类型名被局部遮蔽须明确解析；重名诊断有原token；不依靠拼接字符串制造唯一语义身份。

### 5.4 verifier/native

全部FieldId/TypeUse/PlaceId/DefRef/参数返回/field path归属与界限覆盖；真实Bundle变异走同门，包括不在body中的签名/record定义。
聚合布局/调用/返回采用774 D05：size/alignment/offset checked arithmetic、view/mut pointer和返回out-slot归属明确；无Rust结构表示跨界，不冒称生产ABI。
构造临时、赋值copy、返回sret不得重叠破坏参数；独占mut别名/只读view冲突在前门拒绝，不能只靠runtime偶然正确。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-record.md | 显式标量record支持/拒绝、语法/调用/字段/诊断 | 有限语言合同 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-hir/project.md | record名义类型、字段place、完整引用与图可信门；旧profiles保留 | Checked完整性 | AC-01/03 |
| SD-03 | modify | docs/specs/auto-hir/stage-contract.md | 聚合来源/参数/返回与lowering状态；通用pass仍planned | 阶段边界 | AC-03..05 |
| SD-04 | modify | docs/specs/auto-ac/project.md | record source/HIR native与内部ABI限定 | 不冒称公开ABI | AC-02/04/06 |
| SD-05 | modify | docs/specs/auto-acc/project.md | MD403/404有限子集取证；P/Token剩余text/enum/List/mut方法前置 | 不关Gen1 | AC-06 |

## 6. 测试设计

全部Case owner=work，待T02..06物化，不是现成通过证据。

| Case | AC/invariant | 实际入口/输入 | 可观察预期与区分断言 | 边界 |
|---|---|---|---|---|
| V-01 | AC-01 身份 | 源/HIR API、writer与三维未知/错域；text字段输入 | 正常record通过，未知/改头/错域受控拒绝；不产Checked或native | 与core/scalar/text隔离 |
| V-02 | AC-02 值语义 | source与HIR PE：异质字段/嵌套record/赋值copy；改副本字段 | 原对象不变，副本改变；精确返回/trace | 非资源record |
| V-03 | AC-03 名义/图 | 两同结构不同名record、字段类型前向引用、自环/互环/断开环 | 不同名赋值拒绝；合法前向nested通过；全部含环拒绝且无layout递归崩溃 | 单模块闭合类型 |
| V-04 | AC-03 引用变异 | 实际Bundle改字段TypeId/FieldId/pathowner、签名params/result、方法receiver | 证明确改输入；真实verify拒绝，无Checked/native panic；合法兄弟例通过 | 文本与程序化两腿 |
| V-05 | AC-04 调用 | PE：混合scalar/record参数、命名实参重排、嵌套调用、record返回、递归标量+record | 与独立值oracle一致，receiver一次、源码序trace；sret不覆盖存活实参 | Windows内部ABI |
| V-06 | AC-04 可变性 | mutable局部field、mut自由函数、readonly view/let/方法写字段、同对象mut/view两参 | 合法mut只改目标；非法readonly/别名在source或verify拒绝，诊断定位 | 支持模式由774冻结 |
| V-07 | AC-05 构造顺序 | 声明left/right，源码按right then left命名且mark_a/mark_b各一次；直接HIR同义 | trace按源码`ab`，字段值按名字落位；缺/重/错字段明确拒绝 | 不允许按声明序先求值 |
| V-08 | AC-05 方法/遮蔽 | static new、self/.field readonly方法；同名局部/类型/字段/方法和错receiver | 正确声明绑定；合法用户函数保留，冲突/错类有确切source/span/code | 不含可变实例方法 |
| V-09 | AC-05 深度/释放 | 类型含图1024层、字段访问混合4096、malformed partial、原scalar深输入 | 类型排序/layout/verify/source/emit/Drop全路径无栈崩溃；若环境耗尽受控，不加固定语法深度排除 | 首批受限类型；PE scale T01探针冻结 |
| V-10 | AC-06 集成 | record新gate、741/745/772；若770已合入text gate；失败build三件SENTINEL | 实际gates通过、旧制品字节保持，最终读取新基线/二进制hash | 独立profile，不宣称text-record |

T01先验证整个类型含图/字段路径/构造/Drop与内部聚合调用的可行性，决定支持预算/环境耗尽处理，不能scope缩掉合法nested语法。
开发locked all-targets check+scoped record tests；复审完整ac-core test/fmt、新`scripts/verify-ac-record-771.ps1`+旧gates。strict inventory只check不改绑定；无crates/root定义改动，不跑根t/tv/docs_gen。
真实source/HIR PE走既有contained executor、60s截止/完整输出/hash；旧VM仅语义参考，整数oracle精确独立。768不是前置，golden不自动更新。
默认同一树独立final review，无额外agent/model角色授权。

## 7. 验收标准

| AC | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | 三重身份/旧域隔离与receipt准确；V01 |
| AC-02 | work | 标量/nested record构造/字段/复制语义源码与直接HIR真实PE；V02 |
| AC-03 | work | 名义类型/全类型图/字段/签名/receiver程序化可信门，无后端panic；V03/04 |
| AC-04 | work | 聚合参数返回/static/实例只读/mut自由函数与别名控制；V05/06 |
| AC-05 | work | 构造/receiver/命名实参序、真实名字/span、深输入/Drop；V07..09 |
| AC-06 | work | 最终受审基线的实际gates/边界/SD/hash，旧发布与第二合入基线回归；V10 |
| AC-07 | review | 独立AC/SD/遗漏债务与变异审计，绑定r1/HEAD，无缺口才pass |
| AC-08 | merge | 实际合入、Spec/ledger/index、归档/计数、guard清理收据 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 路径/操作、成果与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 用户批准r2；772/774已复审合入 | inputs/语法/共同合同核对；新reports/771-record/design.md字段/调用/码/Case/预算与whole-path probe；与770登记共享热点顺序。[✅ 已完成] worktree f909781b4：design.md 冻结（真实 parser 语法核对——参数模式前缀 `mut name Type`、构造 `Name{f: v}`/`Name(args)`、前导 `.field`=self；774 D04/D05 消费核对无冲突，§10.1 拼写裁定记录于 design.md §1）；whole-path probe P1(迭代环检测:1024链+自环/互环/断开环)/P2(布局 size=4100/align=4/checked-u64)/P3(4096段路径+owner规则+求值序+拷贝) exit 0，hash 钉于 design.md §6；ABI 半边引证 774 PE 探针 | 01..06 |
| [x] | T-02 | work | T-01 | hir/descriptor/verify、schema/record.atom、record_hir.rs；名义类型/字段/含图/签名/owner可信门与V01/03/04。[✅ 已完成] worktree fb8bd7e14：HirDomain::Record 三元组 + TypeKind::Record + 4 新 ExprKind + PlaceKind::FieldPath + DefKind::Method + param_mode；record.atom descriptor + binder/canonical writer/registry；verify_record（迭代环检测[探针P1四形态]、名义 ValueType::Record(tid)、ctor 完整性、逐段 owner、borrow 位置/模式、per-call mut 排他、core/scalar 域防御、字段类型 bounds）；record_hir.rs 19 测试（V01 身份/隔离/dispatch、V03 名义/前向/三环/1024链/深路径、V04 8 组程序化变异）全绿；完整 ac-core 164/164（旧域零回归）+ fmt clean；R1修复：record_hir.rs 21 测试（R1 增：binding.arg 边界值/u32::MAX/缺失绑定→verify.binding-invalid 受控拒绝——F-771-R1-03；verify.rs:901 所有权走查安全 .get()）；变异 harness 双模式 controlled reject | 01/03 |
| [x] | T-03 | work | T-02 | source七模块名义声明/构造/field/static/readonly self/mut参数，record_source与fixtures；V02/06..09原span/拒绝。[✅ 已完成] worktree f46cd128b：lexer/mod（type/static/mut 档位关键字 + SourceProfile::Record）、ast（TypeRef/TypeItem/Receiver/ParamMode/构造/字段/方法调用/.mut/字段路径赋值目标 + 迭代 Drop）、parser（type item、mut 前缀参数模式、Dot 后缀链、命名构造 Frame[逗号/换行分隔]、前导 .name=self、mut fn 拒绝）、resolve（record 预登记/前向引用、方法表、隐式 self param0、VT::Rec 名义、逐段路径）、typecheck（路径末端赋值类型、借位模式契约）、adapter（records/methods/限定 id/mode/field_path place/ctor 源码序求值+按名绑定、receiver borrow_view 首参一次求值、BorrowMut 透传、view 包装、compile_record_source CLI 面）；record_source.rs 14 测试 + fixtures 全绿；全 suite 178/178（一次并行 link flake 重跑清）；旧 core/scalar 面零改动；R1修复：record_source.rs 20 测试（R1 增：位置构造五例 F-771-R1-02——resolve_call 路由 ctor_positional、元数/命名参/名义类型诊断、fn 同名优先；malformed 2000 深局部拒绝+迭代 Drop——resolve 嵌套 ctor 递归迭代化 Task::CtorOut、ResolvedExpr 迭代 Drop 扩全 record 子形态）；fixtures 增 i08/i09/i10 | 02/04/05 |
| [x] | T-04 | work | T-01/02 | native.rs聚合size/align/offset/view/mut/out-return、nested/copy/临时，内部ABI控制V02/05/06。[✅ 已完成] worktree b0a7c31f9：record_layouts（迭代 memoized 尺寸、checked u64、探针P2形态）；ABI-E 签名变换（declare+define 双点：record 结果→尾 out 指针参+void 机器返回；方法走体循环独立导出符号）；FuncEmitter record 局部=I64 存储指针（参数直绑入参指针、let/var 新栈槽+值拷贝）；内联字拷贝（i64 字+i32 尾，无 libc）；字段路径赋值（标量 store/record 拷贝经根局部存储）；eval 机 CtorOut（临时槽+源码序求值+按字段绑定落位）/FieldOut（平坦偏移标量载入或 record 字段新值拷贝=rvalue 语义）/BorrowOut 指针透传/BorrowMut 叶=槽指针；调用点附加 callee out-slot 临时；t_i32/t_bool typedef 引用正确归类标量；旗舰 fixture 真 PE 运行绿（exit 0）；178/178 + fmt；R1修复：R1 增 mention 触集预计算与 dest 转发不相交门控（Assign 整体/字段路径/ctor 子 dest；Let 新槽与 out-param 恒安全）——F-771-R1-01 别名自赋值读-后-写正确；深构造零临时槽效率保留；R2修复：R2：record 值操作数（绑定 record 字段的 ReadLocal）在其自身求值位即时物化（eager_reads 集 + dest 直写/新槽快照，produces 判定纳入 ReadLocal 保单拷贝；借位实参保持指针透传）——F-771-R2-01 | 02/04 |
| [x] | T-05 | work | T-03/04 | main/shared finish_build/record_native tests、source/HIR PE；V02/05..09，一次receiver与构造eval序。[✅ 已完成] worktree 67ee3cc01：record_native.rs 9 PE 测试（源码旗舰 exit0、HIR fixture、命名构造源码序 trace `ba`、receiver 一次+实参序 trace `aab`、命名实参重排+嵌套 record 实参、out-slot 链式 swap(mk)/swap(swap(mk))、.mut 递归、mut 只写目标、V-09 深输入 1024 层类型图+4096 混合字段路径段运行绿）；copy elision（dest 穿透：生产型表达式直写消费存储——let/assign/return/out-slot 下传目的地，仅生产型子表达式内联），嵌套 ctor 链 O(n²) 临时槽清零（800 层 main 324817→3217 指令、46s→1.1s 构建；1024 层 1.6s 构建 0.4s 运行）；adapter CallOut 迭代路径补 view 包装；全 suite 187/187 + fmt；R1修复：R1 后 record_native 13 PE（增：别名赋值值语义六例、位置构造 PE、HIR r02 强制失败控制 exit 11、HIR r03 直连腿嵌套拷贝/聚合返回/别名；r01 成功码 0 失败码 11/12 互异）；全 suite 199/199；R2修复：R2 后 record_native 15 PE（增 ctor 操作数快照源码序七例 + r04 直连腿 exit0/失败码 31..34）；全 suite 202/202 | 01..05 |
| [x] | T-06 | work | T-05 | 新verify-ac-record-771.ps1/case manifest；mandatory补充与新旧gates，若770已合入联跑其gate；严格盘点/scope/SENTINEL。[✅ 已完成] worktree ecd9530d0：18 步门禁（全 ac-core check/fmt/零警告 all-targets/test、record_native 单线程、正向源码旗舰+求值序 trace `ba`、正向 HIR、7 例钉码负向、profile 隔离、SENTINEL 事务、收据身份行）+ 旧 gates 741(-SkipMainGates)/745/767 串行——18/18 PASS 0 failed 10 cases；770 未合入故未联跑其 gate（interface-contract §6 顺序，第二合入者责任）；实施期 V-08 遮蔽规则落地（局部/参数不得与 record 类型同名）与 SN04/SN07 fixture oracle 修正；R1修复：R1 重跑 22 步全绿 0 failed 19 cases（增 HP02 失败控制/HP03 r03/SP03 别名三例/SP04 位置构造/SN08..10；HP01 成功码 1→0）；770 仍未合入；R2修复：R2 重跑门禁 24 步全绿 23 cases（增 SP05 快照三例/HP04 r04）；770 仍未合入 | 06 |
| [x] | T-07 | work | T-06 | README、verification/proposed-spec-delta、inputs/依赖/实际产品hash；明确下一组合域owner，不发布canonical。[✅ 已完成] worktree ecd9530d0：README record profile 节；reports/771-record/verification.md（Case 全表、18 步门禁收据、输入 SHA256 表、后续 owner：text-record 组合/可变实例方法/unit/资源字段/List/use、通用 pass 保持 planned）；proposed-spec-delta.md（SD-01..05 提案，未发布 canonical）；依赖零 diff（无 auto-down/crates 改动）；R1修复：R1 修订：verification.md 增 §4b 修复轮记录（四发现闭合行+V-09 补证）；README copy-elision 补不相交门控说明；删除一次性辅助 patch_a2.py/patch_r1.py（复审卫生项）；R2修复：R2：verification.md §4c 修复轮记录 + 两连败诊断；卫生项修复（删旧 15 步草稿清单/187 旧总数/exit1 旧行，R1 链接改 ../../plans）；README 补 R2 快照语义 | 06 |
| [x] | T-08 | review | T-01..07完成、writes暂停 | 同树独立AC/SD/真实Bundle变异/ABI/order/深度复审，pass才reviewed | 07 |
| [x] | T-09 | merge | T-08pass与合入授权 | 只消费新基线经work联合gate且重新review的fresh凭证；SD/导航/ledger/index/实际archive/计数/批回归到期判定；guard clean清理树 | 08 |

现有工作树 `D:/autostack/.wt/lang-771/auto-lang`、branch `plan-771-dev`。2026-10-14 R1 final review needs_fix：T-02..07 曾重新打开（T-01 保持完成）；同日 work 修复轮闭合 F-771-R1-01..04 与 V-09 malformed partial 缺口（详 §9 R1 修复记录与 reports/771-record/verification.md §4b），T-02..07 全部重新完成，execution_done 7/9。review8/9、实际closeout9/9。受审代码/基线变化时回同一树work重跑受影响gates、重新独立review；不把实现/联合正确性延后给T09。

## 9. 复审记录

以下2026-10-12记录使用当时编号，作为历史保留；现行执行入口以§8及2026-10-13记录为准。

2026-10-12，stage:new，revision1，outcome:pass（前置受控的草案preflight）；T01..09/AC01..08/SD01..05/V01..10新增。
中央脚本串行取号、active/archive唯一；只书写设计/计划，不改实现、不建树、不测试。
next：批准r1且767/769收口后work，可与770并行；如果共同规则或真实语法核查改变材料行为，先revision对应合同。

规划期间主检出由`897eb34911a4f496a28ba7cdfd8a82815fa043d1`推进至`540dc7d30839eb7df8e68b4d19472358e170fe5b`，提交仅768计划与取号簿记；实现路径零差异。本计划按最终观察HEAD与文件hash绑定，保留已有768 executing/WIP；它不是本批实施前置。

2026-10-13，stage:new，revision2，outcome:pass（引用/交接核查，非实施完成）。770/771编号不变，前置改为现行774与已归档772；更新活跃链接/取材/拟建路径，保留历史附件与既有脚本路径。Task/AC/Case/SD ID、支持与测试语义不变，status仍drafting、current_step仍0。策略队列见[后端演进§7.5](../design/strategy/auto-native-backend-evolution.md#75-现行acacc队列与编号映射2026-10-13)。[核查收据](attachments/774/reference-update-20261013.md)包含当前路径/链接/指纹。共同774完成并复审合入后才放行770/771的依赖实施；本次未建树/实施/合入/运行Cargo。

2026-10-14，stage:work，plan_id:PLAN-771，plan_revision:2，outcome:pass（work-owned T-01..T-07 全部完成并提交；execution_done 7/9，T-08 独立复审未开始）。
code_commit: 工作树 D:/autostack/.wt/lang-771/auto-lang @ plan-771-dev，基面 199d6509e（=master），收尾 HEAD ecd9530d0（T-01 f909781b4 → T-02 fb8bd7e14 → T-03 f46cd128b → T-04 b0a7c31f9 → T-05 67ee3cc01 → T-06/07 ecd9530d0，六实现提交+收尾；依赖无改动）。
task_ids: T-01..T-07 全勾。
evidence: design.md（语法/码/Case/预算冻结+真实 parser 核对——参数模式前缀 `mut name Type`、构造 `Name{f: v}`/`Name(args)`、前导 `.field`=self）；whole-path probe P1/P2/P3 exit 0（hash 钉于 design.md §6）；HIR 域（record.atom/record_hir 19 测试：身份/名义/含图四形态环/1024 链/8 组 Bundle 变异）；source 面（七模块+record_source 14 测试）；native（ABI-D/E、out-slot、copy-elision：800 层嵌套构造 324817→3217 指令 46s→1.1s；record_native 9 PE：旗舰 exit0、HIR exit1、求值序 trace `ba`、receiver 一次 trace `aab`、命名实参重排、out-slot 链式、.mut 递归、mut 只写目标、V-09 深输入 1024 层+4096 段运行绿）；门禁 verify-ac-record-771.ps1 18/18（含旧 gates 741/745/767 全过）；全 ac-core 187/187 + fmt 零警告。
blockers: 无已知合同内缺口。实施期三处 oracle/规则侧修正均已记录（SN04/SN07 fixture 初稿为合法程序→改真实违规；V-08 遮蔽规则按计划§5.3 落地为 source.duplicate；测试期望值笔误×3 修正）。两次全量 suite 中的单次 positive_corpus 失败为已知并行 link flake（单跑+三连跑全绿，plan707 同族）。
next: T-08 同树独立复审（AC-07：AC01..06/SD/遗漏债务/真实 Bundle 变异/ABI/order/深度，绑定 HEAD ecd9530d0），pass 后 T-09 merge（需用户合入授权）。770 尚 execution_done 未合入——若 770 先合，771 为第二合入者须在新整合基线重放全门禁+770 gate（interface-contract §6）。

2026-10-14，stage:review，review_scope:final，plan_id:PLAN-771，plan_revision:2，outcome:needs_fix。
reviewer_identity/limitations: 当前 Codex 复审上下文独立于实施会话；未创建额外 agent/model，不虚构会话身份。
worktree_path: D:/autostack/.wt/lang-771/auto-lang；branch: plan-771-dev；reviewed_commit: ecd9530d0086d0bdae35ef09cf229f2fb6f91468；base_commit: 199d6509e60821c3c0a6414ecd0bedf484105e4d；复审始终 clean，未改实现。主检出观察 HEAD 882f74f5573e59144ee20c6919d23f8f710ca3a6。
dependency_revisions/spec_inputs: 772/774 已 archived，770 未合入；跨仓/依赖源码零 diff；当前 canonical Spec/774 interface/decisions 指纹见 [spec-inputs.sha256](attachments/771/review-r1/spec-inputs.sha256)，规范提案冻结于 [SD 快照](attachments/771/review-r1/proposed-spec-delta.frozen.md)，附件 hash 见 [files.sha256](attachments/771/review-r1/files.sha256)。touched_goals 空因有限实验 profile 不关闭 GOAL-017 等正式目标。
acceptance_results: AC-01 pass；AC-02 fail；AC-03 fail；AC-04 fail；AC-05 partial；AC-06 fail；AC-07 fail；AC-08 pending（merge-owned）。
evidence: 独立重跑 verify-ac-record-771.ps1 18/18、10 cases；全 ac-core 187/187，record_native 9/9，fmt/all-targets 零 warning，741/745/767 全绿。完整收据及反例见 [R1 复审报告](attachments/771/review-r1/REVIEW.md)。无 crates 改动，不跑 root t/tv/docs_gen。
findings: F-771-R1-01 P1 聚合 copy-elision/out-slot 覆盖存活输入：直接 nested ctor 自赋值与 p=swap(p) 两真实 PE 均 exit12（期望0），fresh destination 控制 exit0；F-771-R1-02 P1 位置构造 Q(7) 被 source.undefined 拒绝，违反 §5.2/SD-01；F-771-R1-03 P1 实际 Bundle Call binding.arg=u32::MAX 令 verify.rs:901 panic，合法控制通过/邻接 ref 变异受控拒绝；F-771-R1-04 P2 HIR fixture 首个 copy 失败分支与成功均 exit1，强制进入失败分支仍满足 native test/gate oracle。均为合同内阻断，登记 KNOWN-DEBT-AND-RISKS 不代表可延期。
knowledge_delta: metadata 路径正确，SD-01..04 的完整支持/可信门/不重叠/all-green 断言未成立；已冻结但未批准发布。SD-05 有限 ACC 取材不抵消支持缺口。未写 canonical/ledger。
pending_closeout_gates: 修复后新基线全门禁/受影响直接 HIR 与 source PE 反例重放/独立复审；如770先合，第二合入者新整合基线联跑770 gate并重新review；AC-08实际merge/Spec/index/archive/guard清理仍pending。
next: 同一树 auto-plan:work 修复 F-771-R1-01..04，补永久区分断言与 malformed partial/deep Drop 证据，修订虚假完成声明并提交，重新独立review。T-02..07 checkbox重开，T-01保留，current_step=1；本次未改语义合同/plan_revision，未实施修复、合入、归档或清理。

2026-10-14，stage:work，plan_id:PLAN-771，plan_revision:2，outcome:pass（R1 needs_fix 修复轮：F-771-R1-01..04 全闭合 + V-09 malformed partial 缺口修复；T-01..T-07 重新全部完成，execution_done 7/9，T-08 独立复审待开始）。
worktree_path: D:/autostack/.wt/lang-771/auto-lang；branch: plan-771-dev；基面 199d6509e（=master，未动）；修复提交 `86f3a91a0`（单提交，落在 R1 受审 HEAD ecd9530d0 之上）。依赖零改动。
task_ids: T-02..T-07 重勾（R1修复证据已附于各任务行）。
evidence: 四发现先在基线 ecd9530d0 复现（alias-ctor/alias-sret exit12、positional source.undefined、bundle-mutation binding-arg PANIC、hir-broken exit1=成功码），修复后同反例全清零（0/0/0、check exit0、双模式 controlled reject、exit1≠成功码0）。修复要点：F-01 native.rs 每表达式 record 局部触集预计算 + dest 转发不相交门控（Let 新槽/out-param 恒安全，后者由调用点规则归纳）；F-02 resolve_call 位置构造路由 ctor_positional（fn 同名优先，删死 AST CtorPos）；F-03 verify.rs:901 安全 .get()（诊断码保持 verify.binding-invalid 与 core 域先例一致）；F-04 r01 成功0/失败11/12 互异 + controls/r02 强制失败控制 + valid/r03 直连腿（失败码21..29，oracle 灵敏度实证翻转→21）。V-09 补证顺带修复真缺陷：resolve 嵌套 ctor 递归（2MB 栈 300 深溢出）迭代化 + ResolvedExpr 迭代 Drop 扩全 record 子形态；20000 深幸存。永久回归：record_hir 21 / record_source 20 / record_native 13 = 54 新测试；门禁 22 步全绿 0 failed 19 cases（HP01 码0、HP02 失败控制11、HP03 r03、SP03 别名三例、SP04 位置构造、SN08..10）；全 ac-core 199/199 + fmt/all-targets 零警告；旧 gates 741/745/767 全绿。卫生：patch_a2.py/patch_r1.py 已删（复审 nonblocking 项）。此前已清行为全数保留（旧面 145 测试零回归）。
blockers: 无已知合同内缺口。实施期一次全量 suite 中 source_native deadline 测试因并行构建竞争单次红（单跑+全量复跑全绿，负载 flake）。
next: T-08 同树独立复审（AC-07，绑定修复后 HEAD；建议重点：mention 门控不变式证明审查、r03 oracle、22 步门禁收据、SD 提案与实际支持面重对齐），pass 后 T-09 merge（需用户合入授权）。770 仍 execution_done 未合入——第二合入者按 interface-contract §6 在新整合基线重放全门禁+770 gate。
2026-10-14，stage:review，review_scope:final，plan_id:PLAN-771，plan_revision:2，outcome:needs_fix（R2）。
reviewer_identity/limitations: 当前 Codex 独立复审上下文未实施本轮修复；不声明额外 agent/model 身份。
worktree_path: D:/autostack/.wt/lang-771/auto-lang；branch: plan-771-dev；reviewed_commit: 86f3a91a0cabfafede9b9b69579d368bdf3df522；base_commit: 199d6509e60821c3c0a6414ecd0bedf484105e4d；prior_review_commit: ecd9530d0086d0bdae35ef09cf229f2fb6f91468；复审前后实现树 clean/HEAD不变。main观察37ad632900f7467b7af975a422c725f46f996061。
dependency_revisions/spec_inputs: 772/774 archived、770尚未合入；依赖源码无改动。当前Spec/774合同hash、冻结SD、final产品hash与输入hash见 [R2附件](attachments/771/review-r2/files.sha256)。touched_goals空仍因有限实验profile不关闭正式goal；canonical/ledger未发布。
acceptance_results: AC01 pass；AC02 fail；AC03 pass；AC04 fail；AC05 fail；AC06 fail；AC07 fail；AC08 pending。
evidence: 独立重跑verify-ac-record-771.ps1 22/22 PASS、19 cases；ac-core199/199、record_native13/13、fmt/all-targets零warning、旧741/745/767全过。R1原alias-ctor/alias-sret/alias-control/positional四输入PE exit0；实际Bundle原harness双模式controlled reject；r01成功0/失败11/12+r02强制失败11+r03直连oracle已核查。V09 malformed2000迭代resolve/Drop证据复现。完整 [R2报告](attachments/771/review-r2/REVIEW.md)。
findings/closure: F-771-R1-02/03/04闭合；F-771-R1-01原例闭合但全值语义不变量未闭合。新增F-771-R2-01 P1 blocking：`let p=P{a:q,b:bump(q.mut)}` 与位置形式 `P(q,bump(q.mut))` 应先捕获q值1再bump到2，实际均PE exit11（p.a变成2）；显式保存副本control exit0。native.rs:1256记录record指针，:1495-1498延后到所有字段求值后才复制，fresh destination门控不能守住先前值。影响AC02/04/05/06，T04..07。非新增scope，不可作为债务延期。
knowledge_delta: SD01位置支持/SD02可信门修复已实证；SD03/04完整值语义/order/all-green断言仍不成立，冻结但不批准。T07另有当前收据混旧187/9/exit1与R1相对链接错路径的非阻断卫生项。
pending_closeout_gates: 修复后新提交/源与直接HIR效应组合永久断言/门禁/独立review；770若先合需新组合基线联跑再review；实际merge/Spec/index/archive/guard仍pending。
next: 同树auto-plan:work；T04..07重开、T01..03保留，executing3/9，T08仍未通过。已有两次failed final review，下一交接必须先给record值捕获与view/mut参数语境的诊断/覆盖调整，不能只报旧例关闭。未改plan_revision或实施修复/合入归档清理。
2026-10-14，stage:work，plan_id:PLAN-771，plan_revision:2，outcome:pass（R2 needs_fix 修复轮：F-771-R2-01 闭合；T-04..07 重新完成，execution_done 7/9，T-08 独立复审待开始）。
worktree_path: D:/autostack/.wt/lang-771/auto-lang；branch: plan-771-dev；基面 199d6509e（=master，未动）；修复提交 `e0ad328d8`（落在 R2 受审 HEAD 86f3a91a0 之上）。依赖零改动。
两连败诊断（auto-plan-work 检查点；R2 复审明确要求下一交接必须携带）：
- 剩余不变式：record 值操作数须在其自身源码序求值位捕获，仅借位上下文（view/mut 调用实参）传活指针——值物化与借位指针传递必须分离。
- 前两轮检查为何漏：R1 的 mention/不相交分析只保护「目的地不被 RHS 提前写入」，未保护「操作数源存储不被更晚兄弟实参在 CtorOut 延迟拷贝前改写」；R1 测试面仅目的地别名与新目的地形态，无「操作数源被更晚实参变异」用例。
- 受影响路径/状态：native.rs ReadLocal 以裸指针进槽 + CtorOut 于全部实参求值后才拷贝字段（命名/位置构造同构；Let 新目的地不触发 R1 门控故 R1 修复无效）。
- 设计/oracle 调整：值物化与借位分离（eager_reads + Enter ReadLocal 值路径 + produces 纳入 ReadLocal）；oracle 增加源码序快照族（含 mut 先行重排读后值=后变异值的反向断言）与借位语义不放松钉子（同调用合并实参树 view+mut 仍互斥）。
task_ids: T-04..T-07 重勾（R2 修复证据已附）。
evidence: 三反例先在 86f3a91a0 复现（ctor-snapshot/positional-snapshot exit11、snapshot-control exit0），修复后 0/0/0；R1 全反例复跑保持 0。审计变体：mut 先行重排（读后值=后变异值，方向性断言）、嵌套 ctor 操作数、部分字段变异、函数返回操作数全绿；view+mut 同调用合并实参树（含经 ctor 操作数的读）仍被 verify.mut-conflict 拒绝（a5/a6 探针，钉子入 record_source.rs）。永久回归：record_native 15（+2：ctor_operand_snapshot_source_order_pe 七例、hir_ctor_snapshot_order_pe）/ record_source 21（+1：mut_exclusivity_spans_ctor_arg_trees）= 57 新测试；fixtures/record-hir/valid/r04（失败码 31..34，灵敏度实证翻转→31）；门禁 24 步全绿 23 cases（SP05 三例/HP04）；全 ac-core 202/202 + fmt/all-targets 零警告；旧 gates 741/745/767 全绿。文档卫生项（R2 nonblocking）：verification.md 删除与 22 步表重复的旧 15 步草稿清单（旧 187/9 总数与「exit 1=快乐路径」旧行随之移除）、R1 链接修正 ../../plans。
blockers: 无已知合同内缺口。
next: T-08 同树独立复审（AC-07，绑定修复后 HEAD；R1/R2 反例族与 24 步门禁收据重核；SD-03/04 值语义/order 断言对齐实际），pass 后 T-09 merge（需用户合入授权）。770 仍 execution_done 未合入——第二合入者按 interface-contract §6 新整合基线重放。

2026-10-14，stage:review，review_scope:final，plan_id:PLAN-771，plan_revision:2，outcome:pass（R3）。
reviewer_identity/limitations: 当前Codex复审上下文未实施修复；无额外agent/model角色声明。两轮失败后的诊断/覆盖调整已审查，按值物化与借位分离重构验证，不仅检查旧例。
worktree_path: D:/autostack/.wt/lang-771/auto-lang；branch: plan-771-dev；reviewed_commit: e0ad328d80bfe00ca7563174b3352edeb8e6347b；base_commit: 199d6509e60821c3c0a6414ecd0bedf484105e4d；prior_review_commit:86f3a91a0cabfafede9b9b69579d368bdf3df522；复审前后实现树clean、HEAD稳定。main观察047c2696306afbe65ddbf0a712d50cd3d3b16681。
dependency_revisions/spec_inputs: 772/774 archived，770未合入，依赖源码零改动；当前canonical输入hash与R2一致，见 [spec inputs](attachments/771/review-r3/spec-inputs.sha256)。SD冻结 [proposal](attachments/771/review-r3/proposed-spec-delta.frozen.md)，最终产品/输入/证据hash见 [manifest](attachments/771/review-r3/files.sha256)。touched_goals空因不关闭GOAL017等正式goal。
acceptance_results: AC01..07 pass，AC08 pending（merge-owned）；T08勾选，status reviewed/current_step8/9，T09未执行。
evidence: 独立重跑verify-ac-record-771.ps1 24/24 PASS、23 cases；ac-core202/202、record_native15/15、fmt/all-targets零warning、旧741/745/767全过。R1四源码输入PE均0、实际Bundle变异双模式controlled reject；R2三快照输入均0；新组合探针含mut参数返回构造/异质bool字段/record字段值/三次循环均PE0；直接HIR r04成功0，故意修改前捕获期望→32、后捕获期望→34（distinct failure），验证断言灵敏度。负向同调用view+mut（含经ctor操作数）仍verify.mut-conflict。完整 [R3报告](attachments/771/review-r3/REVIEW.md) 与durable gate receipts。
findings/closure: F-771-R1-01..04/F-771-R2-01全部闭合，无新阻断。新增F-771-R3-01 low/nonblocking：verification.md §4当前表仍残留187/9旧数量/耗时，与202/15当前概述及§4c删旧声明矛盾；复审实际收据明确，closeout修正文档即可，无实施语义缺口。
knowledge_delta: SD01..05与现有bounded profile/可信门/值捕获与借位/内部ABI/ACC有限取证相符；批准冻结提案供merge沉淀，未发布canonical或ledger。保持unit/mut实例方法/text资源字段/List/use/通用pass/公开ABI的后续边界。
pending_closeout_gates: 实际merge/Spec导航ledger/index/archive计数/批回归到期判定/guard清理pending；如770先合，771在新组合基线重跑全门禁+770 gate且重新review（不得用本凭证覆盖新代码）。
next: auto-plan:merge，仅消费本revision/commit且fresh的证据；本次未改实现、未合入归档清理。非阻断F-R3-01登记债务，closeout校正实际验证表。
2026-10-14，stage:merge，plan_id:PLAN-771，plan_revision:2，outcome:pass（consolidation receipt，键 PLAN-771:r2）。
prepared: R3 final pass 绑定 e0ad328d8（rebased e279ee018，range-diff 全 `=`）；canonical diff=SD-01..05（新 docs/specs/auto-ac/source-record.md + auto-hir/project、auto-hir/stage-contract、auto-ac/project、auto-acc/project 追加，lasting-rule 表述）+ F-R3-01 收据表校正（187/9→202/15、时序更新）；delivery_commit=`8b7485c9b`（reviewed 提交之上两文档/投影后代：bd1dabae1 规范沉淀 + 8b7485c9b ledger 投影；实现/依赖零改动核验）。
landed: rebase plan-771-dev→master 后 `git merge --ff-only` 成功，master tip == 8b7485c9b == 分支 tip，无合并提交；旧→新映射 f909781b4→3feae83e6 / fb8bd7e14→520103021 / f46cd128b→2bc34e6fc / b0a7c31f9→c16de9da7 / 67ee3cc01→a95f5f164 / ecd9530d0→d640fa199 / 86f3a91a0→f28a71f70 / e0ad328d8→e279ee018 / d6ee10d49→bd1dabae1（git range-diff 全等）。
merge_gates: merge-owned 裸 `cargo t` 于主检出全绿（5295/5295 passed、1545 skipped、123.3s）；不变输入论证：`git diff master..HEAD -- crates test benches examples Cargo.toml Cargo.lock .cargo` 为空（delivery 与 master 被测面零差异）；触面档（tv/tt/tb/tu/taa）零触发（无 crates/ 改动）；ac-core 面由 24 步门禁（工作树同内容）覆盖；主检出冒烟：ac-core build + r01 check（record 域行）+ 旗舰源码 PE exit0。
ledger_refreshed: Regime B 受管离线投影（无 store 服务可达；.autoos/specs.json 受版本管理，经工作树提交随 ff 落地）：v13→14，新增 P771-1/P771-2（designs）、P771-T（tests）、P771-R3（reviews）、P771-M（reports），全部带 file+tags；守卫=字节级备份+程序化读改写+diff 证明外科范围（仅 version 与新条目）+六节模式校验+回读核验；spec-index 重生成（模块集无变化）。首版 indent=1 整文件重排被守卫拦截回滚，重写为 indent=2 匹配序列化。
archived: docs/plans/archive/771-ac-record-method-native.md（git mv）；status: archived、current_step 9/9、T-09 勾结；plans.md 三行回写（auto-ac/auto-hir/auto-acc 指归档路径）；R1/R2/R3 复审附件随本提交纳入版本管理（K13）；观察项：experimental/ac-core 为自含实验 workspace，无生产 release 二进制/Web bundle 消费其产物——无部署制品需重建（auto-musk K13 教训核对）。
batch_regression_due: 否——docs/plans/.last-batch-regression.json last_covered_plan_id=774（2026-10-14T11:30Z，<48h）；771%5=1 非 divisible-by-5 触发；本次合入后回执仍在 48h 窗口内。
cleaned: 完成——wt-guard 对 lang-771/auto-lang 与兄弟 auto-down 均 clean（零 reparse point、零未提交改动、两树 tip 均已落地/零改动）；`git worktree remove` 移除两树，分支 plan-771-dev（=8b7485c9b 已合并）与临时 plan-771-dev-pre-rebase（d6ee10d49，range-diff 证等价后废弃）删除，组目录 .wt/lang-771 移除；最终核验：master tip=本收据提交、归档 9/9、无 771 相关未跟踪/未提交文件残留。

## 10. 待澄清事项

1. 774 D04/D05及具体命名构造/参数修饰拼写需T01事实核对，支持结果须与r2一致后才能实现；有冲突先修订，不能以编译方便更换语法。
2. 可变实例方法、unit、资源字段、enum/List/use有后续owner；本次不宣称完整ACC P/Token需求满足。
3. text-record组合尚未立项，770/771各自pass不能作为组合验收；第二合入者必须重新绑定受审基线。
