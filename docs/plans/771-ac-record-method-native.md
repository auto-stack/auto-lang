---
plan_id: PLAN-771
status: drafting
feature_name: AC 标量 record、字段与方法首批原生编译
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
| [ ] | T-01 | work | 用户批准r2；772/774已复审合入 | inputs/语法/共同合同核对；新reports/771-record/design.md字段/调用/码/Case/预算与whole-path probe；与770登记共享热点顺序 | 01..06 |
| [ ] | T-02 | work | T-01 | hir/descriptor/verify、schema/record.atom、record_hir.rs；名义类型/字段/含图/签名/owner可信门与V01/03/04 | 01/03 |
| [ ] | T-03 | work | T-02 | source七模块名义声明/构造/field/static/readonly self/mut参数，record_source与fixtures；V02/06..09原span/拒绝 | 02/04/05 |
| [ ] | T-04 | work | T-01/02 | native.rs聚合size/align/offset/view/mut/out-return、nested/copy/临时，内部ABI控制V02/05/06 | 02/04 |
| [ ] | T-05 | work | T-03/04 | main/shared finish_build/record_native tests、source/HIR PE；V02/05..09，一次receiver与构造eval序 | 01..05 |
| [ ] | T-06 | work | T-05 | 新verify-ac-record-771.ps1/case manifest；mandatory补充与新旧gates，若770已合入联跑其gate；严格盘点/scope/SENTINEL | 06 |
| [ ] | T-07 | work | T-06 | README、verification/proposed-spec-delta、inputs/依赖/实际产品hash；明确下一组合域owner，不发布canonical | 06 |
| [ ] | T-08 | review | T-01..07完成、writes暂停 | 同树独立AC/SD/真实Bundle变异/ABI/order/深度复审，pass才reviewed | 07 |
| [ ] | T-09 | merge | T-08pass与合入授权 | 只消费新基线经work联合gate且重新review的fresh凭证；SD/导航/ledger/index/实际archive/计数/批回归到期判定；guard clean清理树 | 08 |

意向 `D:/autostack/.wt/lang-771/auto-lang`、branch `plan-771-dev`，尚未创建。work7/9 execution_done、review8/9、实际closeout9/9。受审代码/基线变化时回同一树work重跑受影响gates、重新独立review；不把实现/联合正确性延后给T09。

## 9. 复审记录

以下2026-10-12记录使用当时编号，作为历史保留；现行执行入口以§8及2026-10-13记录为准。

2026-10-12，stage:new，revision1，outcome:pass（前置受控的草案preflight）；T01..09/AC01..08/SD01..05/V01..10新增。
中央脚本串行取号、active/archive唯一；只书写设计/计划，不改实现、不建树、不测试。
next：批准r1且767/769收口后work，可与770并行；如果共同规则或真实语法核查改变材料行为，先revision对应合同。

规划期间主检出由`897eb34911a4f496a28ba7cdfd8a82815fa043d1`推进至`540dc7d30839eb7df8e68b4d19472358e170fe5b`，提交仅768计划与取号簿记；实现路径零差异。本计划按最终观察HEAD与文件hash绑定，保留已有768 executing/WIP；它不是本批实施前置。

2026-10-13，stage:new，revision2，outcome:pass（引用/交接核查，非实施完成）。770/771编号不变，前置改为现行774与已归档772；更新活跃链接/取材/拟建路径，保留历史附件与既有脚本路径。Task/AC/Case/SD ID、支持与测试语义不变，status仍drafting、current_step仍0。策略队列见[后端演进§7.5](../design/strategy/auto-native-backend-evolution.md#75-现行acacc队列与编号映射2026-10-13)。[核查收据](attachments/774/reference-update-20261013.md)包含当前路径/链接/指纹。共同774完成并复审合入后才放行770/771的依赖实施；本次未建树/实施/合入/运行Cargo。

## 10. 待澄清事项

1. 774 D04/D05及具体命名构造/参数修饰拼写需T01事实核对，支持结果须与r2一致后才能实现；有冲突先修订，不能以编译方便更换语法。
2. 可变实例方法、unit、资源字段、enum/List/use有后续owner；本次不宣称完整ACC P/Token需求满足。
3. text-record组合尚未立项，770/771各自pass不能作为组合验收；第二合入者必须重新绑定受审基线。
