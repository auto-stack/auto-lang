---
plan_id: PLAN-782
status: archived
feature_name: AC 文本与 record 组合：拥有型字段、移动与清理
plan_revision: 1
author: [Codex]
created_at: 2026-10-14
updated_at: 2026-10-15
current_step: 9
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-ac/data-resource-contract.md
  - docs/specs/auto-hir/project.md
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-acc/project.md
new_spec_components:
  - docs/specs/auto-ac/source-text-record.md
touched_goals: []
affects: [auto-ac, auto-hir, auto-acc]
---

# [PLAN-782] AC 文本与 record 组合：拥有型字段、移动与清理

## 0. 变更摘要

承接 [Design 36](../design/36-ac-data-resource-expansion.md) §2/§7 和 [后端演进战略](../design/strategy/auto-native-backend-evolution.md) §7.5，在 770 文本与 771 标量 record **修复、复审、实际合入并完成联合回归之后**，实现独立 text-record profile。支持拥有 Str 的非递归 record、聚合 move/显式深 clone、字段借用与替换、参数/返回及全路径正常清理；源码和直接 HIR 均通过真实 verifier→Checked→COFF/PE 执行。

现在可以规划；前置尚未满足，不能开始依赖实施。770/771 的 bug 由原计划修复，不转交本计划。两个独立 profile 通过不代表组合通过。

## 1. 目标

1. 使 i32/bool/char/Str 及非递归嵌套 record 在同一显式 profile 内组合，类型与资源责任闭合。
2. 区分无资源 record 的值复制与资源 record 的显式 move/clone，杜绝拥有 buffer 的位拷贝双释放。
3. 证明字段借用不悬垂、mut 排他、替换/构造/返回不破坏存活输入，所有正常出口唯一清理。
4. 以 Token 形状微程序验证真实数据组合能力，保持四个旧 profile 和发布/执行器边界。

非目标：enum/is、List/Map、use/多模块、用户泛型、部分字段 move、借用 str 字段、递归堆 record、可变实例方法、unit/隐式尾返回扩面、record equality、GC/RC、生产 I/O/ABI/DLL/热重载、ACC 后端桥、完整 lexer/parser、Gen1/2/3、旧 crates 编译器重构。

只修改本仓 experimental/ac-core 与对应脚本/语料/报告。auto-down 兄弟仅供组内依赖解析，不修改。touched_goals 为空：有限 Rust AC 实验 profile 不关闭 GOAL-017，也不登记 ACC 主体迁移完成。

## 2. 架构方案

source-text-record → 类型预登记/名义解析/字段 place/参数模式 → 新 Atom descriptor → bound Bundle → 结构、类型、来源及资源 verify → Checked → 目标聚合布局/有序资源 lowering → 既有 text runtime + Cranelift → 既有 link/executor → PE。

复用 774 的显式拥有/借用、默认 view、ABI A/B/C/D/E 和 770 实际 runtime 合同；公共 HIR 表达类型、访问和资源义务，不包含机器 offset/CLIF 值。资源属性从完整无环字段类型图推导，禁止相信输入自称 copyable/owned/checked。资源 record 仍为 inline 聚合，Str 字段包含内部拥有 buffer 表示；不是把整个 record 改成堆对象。

本计划沿用 Design 36，属于其已分解的 L1 组合扩面；不新建另一套通用类型/所有权架构。新增组合细则先在 T-01 冻结设计与全路径探针，再开展实现。

## 3. 技术栈

独立 auto-ac-prototype workspace；现有锁定 Cranelift 0.126.2、Windows x64/MSVC、COFF/PE、kernel32 text runtime 与 contained executor。工具链及实际 support-lib/合同/hash 纳入收据。构建、链接、运行均沿既有 60s 截止与完整输出收集，不以 VM、A2R 或 Rust 源码转译代替 native。

## 4. 需求分析与背景调查

- 取材起点 master 为 882f74f5573e59144ee20c6919d23f8f710ca3a6；输入指纹采集时主检出已推进至 1dfa6b0b088b60399691304c958a23bd558dfe33。实际文件 SHA256 见 [planning-inputs.tsv](attachments/782/planning-inputs.tsv)，包括共享 Plan 未提交的最新复审记录。并发他线簿记不属于本计划改动。
- 从 docs/specs/overview.md 导航，已读 auto-ac/project/data-resource-contract、auto-hir/project/stage-contract、auto-acc/project 及 auto-lang/types/design/ownership。canonical 当前仅 core/scalar 实现与 774 合同探针；source-text/source-record Spec 尚未沉淀。战略/Design36 的 drafting 快照滞后，不能当当前进度。
- 774 已 archived；770 r2 当前 executing、最新 R4 needs_fix，受审树 HEAD 92c14fe6cbf1761521fb059bf41f667abeacc214；771 r2 当前 executing、最新 R1 needs_fix，受审树 HEAD ecd9530d0086d0bdae35ef09cf229f2fb6f91468。两树只作为待修实现观察，不是可继承的绿基线。
- [770 R4](attachments/770/review-r4/REVIEW.md) 暴露块出口 borrowed backing、短路临时清理支配关系、终止臂状态合流、分支局部责任及跨调用来源别名问题；[771 R1](attachments/771/review-r1/REVIEW.md) 暴露 self-assignment/out-slot 覆盖存活输入、位置构造误拒、CallBinding 越界 panic 及失败分支 oracle 无区分力。本计划需在组合场景重建相应断言，不能只继承原测试总数。
- 真消费者 packages/aavm/lib/lexer.at 的 Token/Tok 有 text str、TokenKind 和整数字段，tokenize 使用 List<Token>、use，cur_char 旧 char/int 混用、f-string 都需后续适配。774 D03 禁止 record 的借用 str 字段，故本计划微程序使用拥有 Str；不改写旧 lexer，不宣称其已可迁移。
- 已核查主检出 source/{lexer,ast,parser,resolve,typecheck,adapter,mod}.rs、hir/descriptor/verify/native/main.rs、link.rs；771 树有 RecordShape/RecordField/FieldId/ParamMode，770 树有 text_resources.rs 的 TextResourcePlan/BodyResourcePlan。最终符号以 T-01 前置合入后的源码核对为准，不复制未修工作树。
- 当前用户授权：使用指定 auto-plan-new 技能起草下一计划文件，范围为上述文本-record组合规划；允许主检出计划簿记和中央取号。未授权实施/合入、创建新 chat/agent，未指定模型、预算或自动续跑。

## 5. 详细设计

### 5.1 前置与身份

硬门：770/771 各自修复完成、独立 final review pass、真实合入并沉淀 Spec；第二合入者完成共同基线两套门禁及 fresh review。T-01 核验原反例闭合、新主基线及 runtime 合同；任何一项缺失停在 drafting，不先建实现树或复制分支绕过。

拟定新 source 身份 auto.source.text-record.draft；HIR 三元组 auto.hir.text-record.draft / 1 / text-record-i32-draft。T-01 核对是否碰撞，改名须先记录等价修订；旧 core/scalar/text/record 身份不得自动扩大。binder/程序化 Bundle/verify/writer/收据同域，改变任一身份维度拒绝。

### 5.2 类型、语法与资源规则

| 场景 | 本计划支持/拒绝合同 |
|---|---|
| 字段类型 | i32/bool/char/Str/非递归已声明 record；str 字段拒 verify.field-borrow；空 record/默认字段/递归包含沿 771 拒绝。char 不隐式转 int。 |
| 资源分类 | 任意传递字段含 Str 即资源 record；仅标量/无资源嵌套仍为 copyable。后者保留 771 值复制，前者不允许隐式 buffer 拷贝/隐式 clone。 |
| 构造 | 位置与命名构造均支持，实参按源码顺序求值再映射字段。Str 字段接收拥有临时或显式 .move；借用文本先 .to(Str)。全部字段构造完成才是可消费的完整值。 |
| 本地赋值/返回 | 资源 record 的现存值须 .move 或 .clone()；新构造/拥有调用结果直接接收。move 后原根不可读、不可重复 move；首批不允许部分字段 move，诊断受控。局部所有权直接 return 亦须显式 .move；临时拥有返回直接转移。 |
| clone | r.clone() 显式递归深复制拥有字段，标量按值；源继续有效，两个拥有 buffer 独立。空 Str 不增加用户分配。无用户自定义 clone hook。 |
| 参数 | 默认 view；显式 mut/move 的定义与调用 .mut/.move 对称。view 不可写/转移字段，move 参数由 callee 清理。只读实例 self 同 view；static new 无 receiver。 |
| 字段读取/借用 | Str 字段可作默认 view 接收者使用 len/char_at/slice/eq/ne、.to(str)；读取不是复制 owner。借用返回可来自存活 view 参数字段或静态字面量；不得来自局部/拥有临时/move-in/mut 参数。 |
| 字段替换 | mutable local 或合法 mut 参数可替换字段；先完成 RHS，再释放旧字段并提交新 owner。RHS 读取旧字段后产出独立拥有值合法；活跃借用下替换/整个根 move 拒 mut-conflict。标量字段写保留 771 规则。 |
| 聚合重叠 | RHS 和被调输入消费结束前不得将目标当作可覆盖 out-slot；合法 clone self-assignment、调用返回赋值需保全旧输入。消费 x.move 的返回赋值只在 RHS 完成后重建 x；未完成时不允许读取旧 owner。 |

聚合资源规则是本计划新增的有限合同，不改写 771 无资源 record 的复制承诺。T-01 必须核对实际源语法（type、Name{f: v}/Name(args)、mut/move 参数、.mut/.move、clone、self/.field）并物化案例；若与现行语言承诺有材料冲突，先 revision/交用户决定，不为方便实现更改规则。

首批所有权追踪以完整根为 move 单位；字段访问路径保留实际 owner，借用任何资源字段期间保守阻止整个根 mut/move/替换。不同拥有根的合法操作必须保留；不承诺同根不相交字段的精细借用拆分。字段级部分 move 在源码及程序化 HIR 同样明确拒绝，不能标成“将来修复的已支持”。

### 5.3 清理、失败与 ABI

正常作用域 owned 根按创建顺序逆序清理；根内字段按声明顺序逆序递归清理。局部、临时、move-in 参数责任不得重复。临时保持至完整语句尾；跨分支清理只针对实际构造路径，return/break/continue 不向后继 join 污染状态。返回转移/字段替换提交后唯一责任方更新。

D05 的标量嵌套“赋值=拷贝”仅保留在无资源类型；本计划资源类型采用上述显式语义，需在 SD-02 划清适用域。物理布局仍按声明序/对齐 checked 计算；Str 字段表示和 buffer 释放沿 770 runtime，公共 HIR 不泄漏私有布局。ABI-D/E 的 view/mut/out-return 继续使用；资源 record move 是责任转移，不产生共享 owner。alias/out-slot 规则在 verify/native 两侧各负其责，不能用布局偶然不重叠替代来源验证。

分配失败必须区分 runtime 状态、前门诊断与 fatal trap：若内部构造/clone helper 以状态返回失败，其已成功创建的部分输出资源必须逆序回收，旧目标不被提交覆盖；沿 770 若上层以 fatal trap 终止进程，只承诺约定失败状态/前驱 trace/OS 回收，不声称所有 live owner 执行 Drop 或进程结束前 live=0。不新增语言级 Result/unwind。每个申请点与账本元数据失败都按最终 runtime 支持接线核验，不能用“必 trap”隐藏 helper 自身错误回滚。

T-01 先做有界完整路径探针：含两个 Str 的构造/深 clone、嵌套 out-return、自赋值/重叠输入、短路后的临时字段 view、被借字段替换；冻结布局、资源状态、故障申请点、准确诊断和 trace。探针不等于 feature 已实现。若需要改变 D03/D04/770 已承诺支持面，先修订本计划及受影响合同。

### 5.4 可信门与规模

结构/引用界限先验，再推导字段图及资源状态；签名、FieldId 各段 owner、CallBinding 参数/实参、不可达定义与畸形 partial Bundle 均真实 verify。未知/Error/循环/坏资源来源不得进入 Checked，不靠后端补验证或 panic。派生 runtime capability 包含构造、clone、替换和全部清理义务；只信真实库合同/target/hash，caller 字符串不能授信。

不加固定源码/类型嵌套深度限制。T-01 先探测 0/1/256 嵌套与 4096 次生命周期循环，覆盖 parse/resolve/verify/lower/Drop 和畸形 partial destruction；继承两前置实际深输入控制。256 是必验代表规模而非语法上限；若探针失败先设计修复，不能把合法输入裁成“不支持”过关。新增材料预算需列单位/入口/临界两侧，区别环境耗尽，涉及支持承诺变化先 revision。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-text-record.md | 独立组合身份/支持拒绝/资源语法/诊断/CLI有限合同 | 组合可执行边界 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-ac/data-resource-contract.md | D03/04/05/07 增组合消费规则，标量copy适用域、资源move/clone/字段清理与失败边界；保留774探针身份 | 不将标量复制泛化至buffer | AC-02..04 |
| SD-03 | modify | docs/specs/auto-hir/project.md | 组合名义类型/field place/派生资源/可信门；旧域不扩面 | Checked可信 | AC-01/03 |
| SD-04 | modify | docs/specs/auto-hir/stage-contract.md | 组合verify/资源lowering已实证范围；通用pass/桥仍planned | 阶段隔离 | AC-03..05 |
| SD-05 | modify | docs/specs/auto-ac/project.md | source/HIR组合→native和内部ABI/runtime受限支持 | 正确能力声明 | AC-05/06 |
| SD-06 | modify | docs/specs/auto-acc/project.md | Token拥有文本适配前置已有有限证据；enum/List/use/主体/代际仍未交付 | 不关闭自举 | AC-06 |

## 6. 测试设计

以下 V-01..12 均为 mandatory、owner_stage=work，当前未执行；T-01 冻结 oracle，T-02..06 物化 permanent cases 和脚本。聚合统计绿不能替代子例。新脚本 scripts/verify-ac-text-record-782.ps1，new tests/text_record_{hir,source,native,resources}.rs、fixtures/text-record-{hir,source}/{valid,invalid}/、reports/782-text-record/ 均为拟新建。

| Case | AC/invariant | 代表输入与真实入口 | 区分断言/预期 | 适用边界 |
|---|---|---|---|---|
| V-01 | AC-01 身份 | source CLI/Atom binder/API verify/writer：组合三元组、四旧域、改一维 | 合法组合通过，换头/错域受控拒绝，无Checked；writer roundtrip语义同等 | 独立域，旧域不获新类型 |
| V-02 | AC-02 构造/类型 | 两Str+异质标量+嵌套record；位置/逆声明序命名构造；char字段；同形异名 | trace按源码序；字段哨兵不互换；名义失配/str字段拒准确码 | 非递归完整构造 |
| V-03 | AC-02 移动/clone | 局部→move参数→拥有return；r.clone()后替换其中一份；裸资源赋值/部分字段move反例 | move无新增buffer分配且旧根拒使用；clone两份独立，正常alloc=free/live=0；非法例无native调用 | 完整根移动，不支持partial move |
| V-04 | AC-03 借用来源 | view参数字段返回str；局部/临时/move-in/mut字段escape；两层wrapper同源/异源view-mut | 存活view/static合法；逃逸/同源mut冲突前门拒；异源仍过，不凭形式参数号独立 | 无完整借检查器承诺 |
| V-05 | AC-03 替换/重叠 | r=r.clone()；r=transform(r)（clone返回）；r=consume(r.move)；字段RHS读取旧text再concat；nested ctor | 精确文本/标量值正确；旧输入读完才覆盖，旧buffer恰一次free；fresh destination兄弟对照 | 合法RHS；不能同一表达式先move后再读 |
| V-06 | AC-03 控制流 | 动态单臂/双臂局部、嵌套return move、break/continue、字段view从内层外逃 | 正常出口一次清理；终止臂不污染join；外逃前门拒；0次循环无分配/free | source与直接HIR各一腿 |
| V-07 | AC-04 故障/部分输出 | 两Str构造/深clone各第N申请失败、账本失败、零长、预算临界；内部status与上层trap分别观察 | status helper部分输出已回收/旧目标保全；正常live=0；fatal仅约定码/前驱trace，不冒称Drop | 最终runtime真实注入点；无OOM真耗尽 |
| V-08 | AC-03 可信变异 | 真实Bundle修改字段TypeId/FieldId/owner、binding.arg=u32::MAX、含图自环/断开环、move/view/mut模式与清理来源 | 断言实际变异；确切受控诊断、无panic/Checked；同类型合法兄弟控制通过 | 不以文本binder代替API门 |
| V-09 | AC-04 顺序/临时 | &&/||跳过含拥有字段临时，动态lhs、两边临时；receiver及命名实参markers | 跳过路径alloc/free均0；执行路径一次创建与一次清理；完整语句marker先于free；无SSA支配失败 | 保留770语句尾合同 |
| V-10 | AC-05 规模/静态 | 256层含资源嵌套、4096生命周期循环；前置深链；malformed partial Drop；CRLF/span及未执行分支非法类型 | 全链完成/正常live=0，无栈崩/panic；非法静态仍前门拒并原token定位 | 不承诺无限资源/任意性能 |
| V-11 | AC-05 发布/能力 | 三件旧成功制品；缺库/错target/API/hash及删requires、故意失败fixture；实际build/publish/runner | 拒绝前不覆盖旧obj/exe/receipt；按旧事务合同报告；故意失败必须令gate失败，失败分支exit区别成功 | 复用741设施，非新增通用事务框架 |
| V-12 | AC-06 集成边界 | Token形状微程序 text Str+标量kind/line/at/len；新gate+770/771+741/745/772；scope diff | source/HIR真实PE准确值/trace、全部mandatory无skip；enum/List/use拒；不称真实Token/lexer/Gen1 | 真实内部数据能力而非主体迁移 |

命令（只在唯一实施树）：cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets；开发 cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test text_record_hir（及其余三个新测试target）；收口完整 cargo test/fmt、runtime/text 的 locked check/test/fmt、新 gate 与 scripts/verify-ac-text-770.ps1、verify-ac-record-771.ps1、verify-ac-741.ps1 -SkipMainGates、verify-ac-source-745.ps1、verify-ac-scalar-767.ps1（772历史脚本名，以T01核对实际入口）。所有 gate 逐项记录命令/exit/完整输出/失败子例，真实PE hash和support-lib合同也入收据。

本计划不修改 crates/ 或根Schema/docs生成器，不跑根 cargo t/tv/docs_gen；如范围变化先 revision并按仓规补触面门禁。acc_inventory 严格检查仅核验未改人工绑定；SD-06新增有限取证note不偷偷刷新旧决定。无 golden rewrite/update 放宽 oracle。

角色：实施/内检由后续 work 会话承担；final review 独立上下文、同一实施树，未指定额外agent/model；内检只表示可交复审。写入暂停后review，needs_fix回原树；受审代码/依赖改变使受影响凭证stale。任何合同内缺口或mandatory未运行都不能 execution_done。

## 7. 验收标准

| AC | owner_stage | 可观察结果与证据 |
|---|---|---|
| AC-01 | work | 新source/HIR/verify/writer/receipt身份一致；旧域隔离，V01 |
| AC-02 | work | 构造、名义字段、资源/无资源分类、完整move/深clone/调用返回源码与HIR均实际支持；拒绝面明确，V02/03 |
| AC-03 | work | 借用来源/alias/字段替换/存活输入/控制流清理可信，真实API变异受控拒绝，V04/05/06/08 |
| AC-04 | work | 资源计数、申请失败、临时寿命、短路/求值序及fatal边界有区分oracle，V07/09 |
| AC-05 | work | 深输入/partial Drop/span/静态合法性、能力及三件发布行为保持，V10/11 |
| AC-06 | work | 新旧联合门禁在最终基线全部实际运行；Token形状与非目标无混称；SD01..06冻结hash，V12 |
| AC-07 | review | 独立逐AC/SD/原反例和组合兄弟例复核，遗漏/延后/workaround/警告/fmt审计；revision+HEAD+输入hash绑定，零合同缺口才pass |
| AC-08 | merge | 实际合入、Spec/模块导航/ledger/index沉淀、归档计数、批回归到期判定及guard清理收据 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 文件/操作与可验成果 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 用户批准r1实施；770/771满足§5.1 | ✅ 已完成 @1bc7a1665：[design.md](reports/782-text-record/design.md)+[probe-cases.md](reports/782-text-record/probe-cases.md)——§1.1 前置硬门全项收据（770 R11 pass/合入/Spec 沉淀/联合门禁 14/14）、planning-inputs 漂移全归因（data-resource-contract 零漂移）、合同一致性表（无材料冲突）；冻结：新身份 auto.source.text-record.draft→auto.hir.text-record.draft/1/text-record-i32-draft（N07 实证无碰撞）、资源分类传递闭包（派生）、Str 字段布局 {ptr u64@0,len u32@8}=12/8（Token=32，offset 0/12/16/20/24，256 深 closed form 24+8n，u32 上界）、新码仅 verify.partial-move + 扩面 7 码、HIR 增 record_clone+moved 旗标（let/assign/return）、runtime/text **零新符号**、ABI 全复用（move-in 机器形 *mut）、V-01..12 物化落点与预算表。whole-path 探针 P1..P7 **PROBE-OK exit 0 零警告**（files.sha256 绑定）；边界负探针 N01..N09 实跑——**发现 F-A：record 域 member_type 对 Str/str/char 字段类型 panic（exit 101），违反 771 声明的拒绝合同**（770 R6-03 整合引入，无在途计划拥有；裁定 T-03 域内 5 行守卫修复，oracle=source.unsupported exit 1，见§10.5）；前置正控 record 旗舰+四族 68/68 绿 | 01..06 |
| [x] | T-02 | work | T-01 | ✅ **R1 修复后重关闭** @638b10dbe（原 @697e3f811，R1 重开于 F-05）：`schema/text-record.atom`（text∪record 并集 + record_clone/move 双新 case，分支全量 require/forbid）；hir.rs `HirDomain::TextRecord` + `ExprKind::RecordClone{record,base}`/`Move{inner}`（设计 §4.3 修订：moved 旗标改为 **`expr.move` 包装表达式**——77 个 Stmt 构造点零扰动，单一表示，`CallBinding.moved` 在新域禁用）；descriptor binder/writer/domain/TypeUse 内建词（Str/str/char 限 text+text-record 域）；verify：`verify_text_record` 入口（T-02 结构面 + 能力派生门）、check_profile_ops 四域化（旧四域互拒新面）、`record_resource_map` 传递分类（pub 供 T-04）、check_record_module 字段门（str→`verify.field-borrow`、Str/char 放行、域感知映射 4 处）、owning-position/界检/init 三走查 + ExprScan 补臂、`check_text_record_rules`（move 形状 `verify.partial-move`、move 参数类型门 `verify.ownership-required`、裸 let/assign/return 分类拒 resource-assign/ownership-required）；main.rs check/build 分派（build 在 T-05 前受控 `backend.lowering-unsupported`）。text_record_hir **13/13**（V-01 身份五域隔离+writer roundtrip+错维拒；V-02 Str/str/char 字段+传递分类+cycle+copyable 保持 771 值复制+clone 名义拒；V-08 ctor/call binding 越界、clone 跨型、requires 删除受控拒）；前置回归 79 例全绿、零警告、fmt clean | 01/02/03 |
| [x] | T-03 | work | T-02 | ✅ 已完成 @3f0366f6d（核心链 @24171d462 + F-A @a2aa02a5b）：source 七模块/typecheck/adapter 组合面接线——构造（命名/位置双形、AP-4 提升补齐位置构造与静态/实例方法实参位）、字段/方法（Str 字段 view 接收者、771 static/instance/mut 面保持）、mode/move/clone（内建 clone 仅组合域绑定、用户声明 clone → `source.duplicate`、move 参数 `mode: move` 发射+可写根、Move 实参整体转移不被 borrow_view 包裹、缺 `.move` 调用点对称门）；verify 侧修复 Ref-vs-Builtin 字段类型盲点（资源分类/field-borrow/bare assign/return 门对源码 Bundle 生效）、771 调用绑定模式匹配扩展至组合域（含 MoveOwn 三臂）、组合域 Str 局部门白名单。text_record_source **19/19**（V-02/03/05 source 腿、partial-move 含死分支静态检查、旧域保持 N06/770-move）；全套 ac-core **394/0**、零警告、fmt clean | 02/03/05 |
| [ ] | T-04 | work | T-02/03 | **R2 重开，尚未验收** @638b10dbe（原 @3be6031d1，R1 重开于 F-01/02）：text_resources.rs 扩展（770 机器沿用，无第二套账本）——owned 根扩容至资源 record 局部（binding+move-in 参数）；`BodyResourcePlan` 增 `buffer_frees`（逐 Str 字段缓冲统一出口调度，P3 序：根逆创建、字段逆声明、嵌套展开）与 `replace_frees`（P4：RHS 物化后旧缓冲，整根重赋/字段替换/move 后重建三分支）；Move 包装统一进 moves 验证（use-after-move/double-move/循环重入杀根/join 一致性/短路条件 move 拒绝全套沿 770）；字段借用 backing 溯源（按根保守：替换/移根期间活跃借用 → mut-conflict）；字段视图逃逸门（owned/mut/move-in 来源 → borrow-escape）；ctor 字段裸转移门（ownership-required）；verify_text_record 接线 excl 定点走查+计划驱动能力派生；§5.2 字段读取视图化（FieldGet 终型 StrOwned→str + verify 兼容规则 + parser `.to` 组合面让路）与字段替换 AP-4。text_record_resources **25/25**（V-03/04/06/07/09：门与计划形状 oracle、直接 HIR 腿；运行期注入/计数= T-05）；全套 ac-core **419/0**、零警告、fmt clean | 03/04 |
| [ ] | T-05 | work | T-04 | **R2 重开，尚未验收** @638b10dbe（原 @a2fc74d6d，R1 重开于 F-01/03/04/06）：合并 lowering（text lowerer 扩展，旧 text/record 路径零变化）——§4.5 布局（Str 字段 12/8、资源 record 8 对齐、copyable 嵌套保持 4 对齐，checked u64/u32 上界）；record 局部=栈槽聚合、参数/结果=指针形（ABI-D/E、move-in *mut、out-slot 聚合返回）；RecordCtor（提升字段缓冲入临时聚合+转移）、RecordClone（位拷贝+逐 Str 字段 ac_rt_own_clone）、Move/Borrow 地址直传、FieldGet 按偏移装载视图/标量；出口 buffer_frees 与 RHS 后 replace_frees 逐字消费 T-04 计划；聚合临时语句尾 LIFO 释放；方法体定义+CLI build-source 组合域接线；新增 verify 门（view 模式 Str 参数入 ctor 字段=借用逃逸）。text_record_native **11/11**（V-02 值/trace、V-05 重叠精确 trace、V-09 短路零分配、V-10 256 深/4096 循环/0 次零分配、V-11 能力门+拒绝无制品、V-12 Token 微程序+直接 HIR 腿+新旧 profile 共存）；全套 ac-core **430/0**、零警告、fmt clean | 01..06 |
| [ ] | T-06 | work | T-05 | **R2 重开，尚未验收** @56dd15748（原 @8558863ec，R1 重开于 F-06 声明纠正）：`scripts/verify-ac-text-record-782.ps1` + `docs/reports/782-text-record/verification.md`。最终基线 **25 步全绿 0 failed**（收据 gate-20261015-093737 绑定 HEAD 836635a2e，原始输出 raw-20261015-091943/）：两 workspace check/test/fmt 零警告（matcher 正负对照；ac-core **431/0** 四 text_record target 在场断言、runtime/text 20/20）；contained-executor 案例走查 11 例——TP01 旗舰 p01 PE exit 30 + 缓冲 id 多重集配对（泄漏/双释放零容忍）+obj/receipt 在场、TP02 重叠形态 exit 9、HP01 直接 HIR tr01 exit 2 精确 trace、TN01..08 钉码负例、profile 隔离（四旧域拒+未知 exit 2+域行）、capability 故意失败控制（exit 1+capability.missing+零制品）、SENTINEL 事务（失败 build 三件制品 hash 保持）、收据身份五针；旧五门实跑 771/741[-SkipMainGates]/745/767/770；**770 gate 归因裁定**——其 per-plan scope 步 allow-list 仅 770 路径、后继树按定义必败，本 gate 不改 770 已合入脚本，实跑全程断言唯一 [FAIL] 即 scope 步、被标 18 文件恰为本计划域内 artifact（同 git diff 重算对拍），实质步骤全绿（真实越界改动仍红=scope 不变式重基 782 域）；自有 scope 隔离+24 项输入/runtime-lib/合同/三 PE/gate 脚本 hash 入收据。**gate 工作发现并修复 T-05 缺口**：FieldGet 的 `Str` 拼写（verify 兼容规则两拼写等价）在合并 lowering 落标量臂→后续 text 操作 unreachable panic（Checked 不可 lowering，tr01 fixture 首跑抓获）；修复=视图装载臂扩 StrBorrow|StrOwn + 永久回归 `hir_fixture_str_typed_field_get_pe`（exit 2+精确 trace），全套 431/0 | 05/06 |
| [ ] | T-07 | work | T-06 | **R2 重开，尚未验收** @56dd15748（原 @9726a412c，R1 重开：附A 闭合判定重基修复后基线）：`docs/reports/782-text-record/proposed-spec-delta.md` 冻结 SD-01..06——SD-01 新 source-text-record.md 合同（派生资源分类/完整根 move/深 clone/字段视图与替换/FieldGet 两拼写等价/唯一新码 partial-move+扩面七码）、SD-02 D-03/04/05/07 组合消费（标量 copy 适用域划界、backing 按根保守、owned_returns 扩 record 根）、SD-03 HIR 域/record_clone/move 包装形态与派生属性不信自称、SD-04 合并 lowering 已实证阶段范围（通用 pass/ACC 桥仍 planned）、SD-05 ac 工程能力面（runtime 23 符号零新增、770 gate 归因裁定）、SD-06 acc Token 取证与非目标；**附A 前置反例组合闭合核对九类零缺口**——唯一缺项 V-04"两层 wrapper 同源/异源 view-mut"由 @0318bcb93 补齐（mut 链两层：同源 verify.mut-conflict / 异源过，按 SOURCE 不按形式参数号，镜像 770 r4_p07/c02）；附B 源文件 manifest 与支持/拒绝声明对拍无材料冲突。canonical 未发布（归 T-09）。最终基线 gate 重跑 **25/25 全绿**（gate-20261015-100236 绑 9726a412c：ac-core **433/0**、runtime 20/20、旧五门+归因 770） | 06 |
| [x] | T-08 | review | T-01..07完成，writes暂停 | ✅ R3 final pass @17508b703（90/90+37源码重放/组合控制+9 API控制；见R3报告），原树独立审计AC01..06与SD、反例/永久测试oracle/遗漏债务/健康；pass→reviewed=8/9，needs_fix回work，不修改实现 | 07 |
| [x] | T-09 | merge | T-08 fresh pass与合入授权 | ✅ 已完成：merge-owned 联合门禁 25/25 于冻结提交 `17508b703` 实跑（gate-20261015-141557，452/0+runtime 20/20+旧五门+归因 770）；SD-01..06 canonical 落地（新 source-text-record.md + 五文件修改，`99d92513f`）；rebase 17 对 range-diff 全等+1 对仅 plans.md 行合并消解，ff-only 落地（master tip==`99d92513f`）；ledger 守护式离线投影 v22→v23（P782-1/2/T/R3/M 五条目，零扰动证明：仅版本+5 追加）；spec-index 再生（31 projects）；R3 复审簿记+附件 r1..r3 提交（`a1cd75457`）；归档+清理收据见 §9 | 08 |

拟定唯一实施树 D:/autostack/.wt/lang-782/auto-lang、branch plan-782-dev；当前未创建。work使用scripts/new-wt-group.sh lang-782 --branch plan-782-dev（master基面、auto-down兄弟detached）或等价仓规路径，不创建任何junction/symlink。主检出仅计划簿记；实施/内检/final review顺序使用同一树。不能将联合回归/实现正确性推迟至merge。

## 9. 复审记录

2026-10-14，stage:new，plan_id:PLAN-782，plan_revision:1，outcome:blocked（实施前置待满足；草案合同自检完成，非实现复审）。

技能来源：D:/autostack/auto-musk/.agents/skills/auto-plan-new/SKILL.md 及 references/verification-contract.md；使用用户指定版本。中央new-plan.sh在master串行取号，调用外包命名mutex并即时核对活跃/归档max=781、next=782、创建唯一；next现783。本轮没有实施、创建worktree/agent、运行cargo/PE或修改canonical Specs。

preflight：T01..07覆盖AC01..06及SD01..06，review/merge独立owner；V01..12均有区分断言和未来物化任务。首批partial move拒绝、字段借用按根保守、资源copy与标量copy隔离、fatal不冒称Drop，未把770/771失败历史当通过凭证。T01承担有限物理布局/helper故障接口/准确新码决策，在dependent实现前固定；不得反读实际输出当expected。

next：可审阅/批准本草案；770/771修复、独立复审并合入且联合门禁后，用户授权实施时进入work T-01。若前置修复改变共同合同，先修订受影响§5–8/SD与revision。changed task_ids=T01..09，acceptance_ids=AC01..08，delta_ids=SD01..06。

2026-10-14，stage:work（入口门未过，未进入实施），plan_id:PLAN-782，plan_revision:1，outcome:blocked（§5.1 硬门未满足）。
worktree_path:无（§5.1 明令前置缺失时停在 drafting、不建实现树；lang-782 未创建）；branch:无；base_commit:master `119c8215a98e8d0bd3cc30fb0714a264cf0dd6ca`。
前置核验（fresh 实测）：771 满足——已合入并归档 `docs/plans/archive/771-ac-record-method-native.md`（delivered）；**770 不满足**——status=executing（plan_revision 2，current_step 2/9），最新 R6 final review outcome=needs_fix 含三项 blocking（P770-R6-01 循环重入资源缺口 P1；P770-R6-02 不可达臂污染 loop_exits P2；P770-R6-03 771 整合基线与联合回归未做 P2，见 attachments/770/review-r6/REVIEW.md），`plan-770-dev` HEAD `a14f1470b1d5e1174653e8d499091147b4257fe6` 经 merge-base 核验未合入 master；`.wt/lang-770` 仍在途。
task_ids:无开启（T-01 依赖"770/771满足§5.1"未满足，T-02..09 依赖 T-01）；code_commit:无；dependency_revisions:前置未冻结，T-01 时按 planning-inputs.tsv 重新比对。
blockers:770 须先在原树闭合 R6-01/02/03 → fresh final review pass → 真实合入并沉淀 Spec；随后第二合入方在新共同基线实跑两套联合门禁并 fresh review（770 T-09 已含该要求）。owner=770 原 work/review/merge，不转交本计划。
next_responsible_role:770 work（同树返工）；本计划保持 drafting/current_step=0，待前置全部满足且用户授权实施后由 T-01 重新入口。本轮零实施、零 worktree/branch/agent 创建、零 cargo/PE 运行、canonical Specs 未动。

2026-10-15，stage:work（重新入口，进入实施），plan_id:PLAN-782，plan_revision:1，outcome:entered。
前置复核（fresh，§5.1 全项满足）：771 合入归档；770 R11 final review **pass**（§9.24，reviewed_commit `afe45e2d65332b0cd448f3a2b5edf562903296b6`）、T-08 勾、status=reviewed 8/9、真实合入（master HEAD `4568e4ea7` = plan-770-dev tip，merge-base 实测）、Spec 沉淀 `f4b1d2e82`（source-text 契约 SD-01 + hir/stage SD-02/03 + ac/acc SD-04/05）、第二合入者联合门禁 14/14（text 361+1/runtime 20/20/record 24/24+23 cases，gate-20261015-014555）+ R11 fresh pass。770 T-09 归档/清理簿记仍开放，不属 §5.1 构成项，不阻塞。
worktree_path:`D:/autostack/.wt/lang-782/auto-lang`；branch:`plan-782-dev`；base_commit:`4568e4ea7`（new-wt-group.sh 建组，auto-down 兄弟 detached c39ac7f，组内零链接）。
task_ids:T-01 开启；授权=用户 2026-10-14"实施它"+2026-10-15"计划770已经merge了。请继续"。T-01 前置比对基线改为合入后 master（planning-inputs.tsv 采集于合入前，T-01 记录漂移归因，不回滚取材）。

2026-10-15，stage:work，plan_id:PLAN-782，plan_revision:1，outcome:pass（T-01 完成=1/9，继续 T-02）。
worktree_path:`D:/autostack/.wt/lang-782/auto-lang`；branch:`plan-782-dev`；base_commit:`4568e4ea7`；code_commit:`1bc7a1665`（design.md+probe-cases.md+probe/ 17 文件）。dependency_revisions:master==plan-770-dev tip（770 实现+Spec 沉淀全含）；771 archived。
task_ids:T-01。evidence：design.md §1/§8（前置收据+探针收据）、whole_path_probe PROBE-OK exit 0 零警告、N01..N09 边界探针原文、正控 68/68；材料冲突=无；F-A 发现与裁定见 §10.5。
blockers:无。next_responsible_role:work（T-02：schema/text-record.atom+descriptor/verify 域扩展+text_record_hir 物化 V-01/02/08）。

2026-10-15，stage:work，plan_id:PLAN-782，plan_revision:1，outcome:pass（T-02 完成=2/9，继续 T-03）。
code_commit:`697e3f811`（11 文件：schema/text-record.atom 新、tests/text_record_hir.rs 新、fixture tr01 新、hir/descriptor/lib/verify/main/native/adapter/text_resources 改）。task_ids:T-02。
evidence：text_record_hir 13/13（V-01/02/08）；前置回归 record_hir 21+record_source 21+text_hir 16+text_source 10+text_binding 11=79 全绿；check --all-targets 零警告；fmt clean。设计 §4.3 修订（moved 旗标→`expr.move` 包装）在提交内注明理由（77 个 Stmt 构造点零扰动；`CallBinding.moved` 新域禁用，canonical 单一表示）。
blockers:无。next_responsible_role:work（T-03：source 七模块/typecheck/adapter 组合面 + member_type F-A 修复 + text_record_source）。

2026-10-15，stage:work，plan_id:PLAN-782，plan_revision:1，outcome:pass（T-03 完成=3/9，继续 T-04）。
worktree_path:`D:/autostack/.wt/lang-782/auto-lang`；branch:`plan-782-dev`；base_commit:`4568e4ea7`；code_commit:`3f0366f6d`（T-03 收口：resolve/verify/adapter/main 4 改 + fixtures/text-record-source 10 新 + tests/text_record_source.rs 新；核心链前置提交 `24171d462`、F-A 修复 `a2aa02a5b`）。dependency_revisions:同 T-01 收据。
task_ids:T-03。evidence：text_record_source 19/19（V-02 顺序/提升/名义 oracle、V-03/V-05 合法形、i01..i08 全部命中冻结核查码：resource-assign/partial-move/ownership-required/field-borrow/source.duplicate/type-mismatch、死分支静态检查、旧域 N06+770-move 保持）；全套 ac-core cargo test 394 通过 0 失败；check --all-targets 零警告；fmt clean。
本轮修复的实现缺口（探针逐一定位后修）：① 内建 clone 臂无 profile 门→旧 record 域 `.clone()` 从 N06 的 source.undefined 漂移为 adapter internal error（已加 TextRecord 门恢复）；② RecordClone atom 发射用索引而非名义 `t_rec_<name>`（dangling TypeRef）；③ `record_resource_map`/`check_record_module` 字段门只识别内联 Builtin TypeUse、不解析 Ref 目标类型种类→源码 Bundle 的资源分类/bare assign/return/str 字段门全部失效（不变式修复：按解析后类型种类决定语义）；④ 位置构造与方法调用实参位缺 AP-4 字面量提升；⑤ move 参数 mode 发射塌缩为 view + verify 侧不可写（违反 §6.3 替换根合同）；⑥ Move 实参被 borrow_view 包裹（违反整体转移表示，§4.3 修订案）；⑦ 缺 `.move` 调用点对称门（771 模式匹配走查未扩展到组合域）；⑧ 组合域 Str 局部被 body 级 local-type 门误拒。
发现的基线预存缺陷（非本计划引入，§10 登记）：771 命名构造字段值位不支持二元运算表达式；770 assign 位 `.move` 产生 source.internal 而非受控拒。
blockers:无。next_responsible_role:work（T-04：text_resources.rs 资源走查扩展——owned 根扩容至资源 record、逐字段缓冲释放、moved 状态/use-after-move/double-move、字段借用 backing 溯源与 mut-conflict、V-03/04/06/07/09 物化）。

## 10. 待澄清事项

1. 实施前置尚未满足，owner=770/771原work/review/merge；不影响本草案审阅。
2. 资源record完整move/深clone/按根保守借用及拒绝partial move属于本r1新增支持合同，随草案批准一并确认；目前无实施授权。
3. 组合物理布局、helper故障返回接口、准确新诊断码/对应span及复用runtime的申请点，owner=T-01；须有决定 artifact+区分oracle+探针后才放行T-02，不留给native猜测。
4. 若前置最终Spec/API有材料变化，owner=T-01，先比对planning-inputs.tsv并记录revision与证据；旧计划状态和战略快照不得覆盖fresh收据。
5. **F-A（T-01 发现，owner=T-03，已修复 @a2aa02a5b）**：合入基线上 record 域字段声明类型位遇 `Str`/`str`/`char` 标识符 panic（`parser.rs::type_word` unreachable，exit 101），违反 771 Spec"str/Str/char 字段显式拒绝"的一等合同；scalar/core 域同位均受控拒（N08/N09 对照），`member_type()` 独缺守卫，系 770 R6-03 整合引入且无在途计划拥有。修复：`member_type()` 补与 `param_type()/decl_ann()` 对齐守卫；冻结 oracle 命中（N01..N03 复验 `error[text/source.unsupported]: type 'X' is not part of this source profile` exit 1，`negative-run-post-fix.txt`）；record_source +1 回归测试（22/22）。属恢复 771 已声明合同，非旧域扩面。
6. **基线预存缺陷（T-03 探针发现，owner 待定——均非 782 引入，不转交本计划修复）**：
   - **D-771-A**：record 域（含组合域继承面）命名构造字段值位不支持二元运算表达式——`Pair { left: 1 + 2, ... }` 在纯 record 域即报 `source.syntax: expected operator`（771 交付面即如此）。782 组合程序以 let 提升规避（fixtures 已按此写）；若后续需在 ctor 字段值位直接写复合表达式，需独立修复并回归 771 面。
   - **D-770-B**：text 域 assign 位 `.move`（如 `o = x.move`）产生 `source.internal: generated Atom text failed to bind`（adapter 发射 text descriptor 不接受的 `expr.move`），而非受控拒绝；770 语义上该位本就不支持 move 标记，缺的是前门拒绝。782 组合域 assign 位 move 走 `expr.move` 包装（descriptor 接受）不受影响。
   - **D-771-C**（T-05 发现）：布尔 let（RHS 为逻辑式或 bool 返回调用）后跟 `if <变量>` 在纯 record 域即报 `source.syntax: expected expression, found return`（text/scalar 域同形通过，782 组合面继承 record 行为）。782 测试以内联条件规避；独立修复需回归 771 面。

2026-10-15，stage:work，plan_id:PLAN-782，plan_revision:1，outcome:pass（T-04 完成=4/9，继续 T-05）。
worktree_path:`D:/autostack/.wt/lang-782/auto-lang`；branch:`plan-782-dev`；base_commit:`4568e4ea7`；code_commit:`3be6031d1`（text_resources/verify/parser/resolve 改 + tests/text_record_resources.rs 新 + p01/i02/i07 fixture 随走查合同修正）。dependency_revisions:同 T-01 收据。
task_ids:T-04。evidence：text_record_resources 25/25——V-03（use-after-move/double-move/循环重入杀根/缺 .move 对称/ctor 裸转移拒、move 参数被调方清理计划、clone 不扰借用）；V-04（view 参数字段返回合法；owned/mut/move-in 字段逃逸拒；替换/移根期间活跃借用 mut-conflict；悬挂视图使用点拒）；V-05 资源腿（replace_frees 计划形状：单字段/整根克隆重赋/move 后重建零调度；链式替换合法）；V-06（臂局部自臂出口闭、嵌套 return move 转移不排空、break/体末释放、直接 HIR 腿）；V-07 计划形状（P3 双根双字段序、嵌套展开 [1]/[0,0]、owned_return 不排空）；V-09（未知极性短路 move 拒、常真 lhs 死支不转移）。
本轮顺带修复：①§5.2 字段读取视图化三件套（resolve FieldGet 终型、verify FieldGet 兼容+expr_type 返 declared、parser 组合面 `.to` 让路 text 臂——`to` 是 Reserved 词 record 臂会拒）；②字段替换位 AP-4 字面量提升；③p01 旗舰 fixture 的 use-after-move 违约（走查上线后抓获，n4 oracle 生效证据）、i02/i07 改推断形式以命中 partial-move 冻结码。
T-05 交付面（下会话）：native/main/shared finish_build 合并 lowering——buffer_frees/replace_frees 消费、RecordClone/Move/字段 view 的 CLIF 发射、ABI-D/E/*mut move-in、trace oracle（每缓冲一行、alloc==free/live=0）、故障双通道注入、V-02/05/09/10/11/12 native 腿与双腿 fixtures。
blockers:无。next_responsible_role:work（T-05）。

2026-10-15，stage:work，plan_id:PLAN-782，plan_revision:1，outcome:pass（T-05 完成=5/9，继续 T-06）。
worktree_path:`D:/autostack/.wt/lang-782/auto-lang`；branch:`plan-782-dev`；base_commit:`4568e4ea7`；code_commit:`a2fc74d6d`（native.rs 合并 lowering + text_resources.rs 逃逸门 + main.rs build 接线 + tests/text_record_native.rs 新 + p01 fixture 按合同修 move 参数）。dependency_revisions:同 T-01 收据；runtime/text 零改动（23 符号原样，§5 合同保持）。
task_ids:T-05。evidence：首例组合域 PE 即过（m1：LIT/CLONE/TEXT-SCALARS/FREE 精确配对、P3 逆序、exit=5）；全功能冒烟 exit=7、5 分配==5 释放、P3/P4 语义精确（替换释放/被调方清理/出口逆序）；text_record_native 11/11——V-02（Token 四字段值+char、嵌套 record 深克隆独立性）、V-05（r=r.clone()/transform/字段 concat/move 重建的逐步旧缓冲释放 trace）、V-09（短路跳过臂零分配零释放）、V-10（256 深嵌套构造+8 段路径读取、4096 迭代新鲜根、0 次循环零分配）、V-11（缺 capability 拒绝且无制品）、V-12（Token 微程序 static/instance 方法+clone+move exit=57、text/record 旧 profile 共存构建运行、直接 HIR 腿 bind+verify+lowering）；全套 ac-core 430/0（含 text_native 19/19 零回归）；零警告、fmt clean。
本轮修复的实现缺口：①RClone 的 own_clone 结果滞留 pending_temps（应归聚合）——首 PE 冒烟即暴露；②record 参数同时入 rslots/rvars 致读未初始化自有槽——隔离复现后修（参数仅存指针变量）；③方法体未在 text lowerer 定义循环中（Method kind 被跳过）；④buffer_frees 对纯 text 域 Str 根的重复释放路径（exit_frees 已覆盖，需跳过）。
本轮新增 verify 门：view 模式 Str 参数的 ReadLocal 进 record ctor 字段=借用逃逸（774 借用返回规则的ctor 字段对偶）；move 参数入 ctor 字段的合法拼法为 `text.move`（§6.1 拥有表达式清单）。
发现的基线预存缺陷（771 域，§10.7 登记）：D-771-C——布尔 let（含逻辑/调用 RHS）后跟 `if <var>` 在纯 record 域即解析失败（text/scalar 域通过）。
T-06 交付面（下会话）：scripts/verify-ac-text-record-782.ps1 新门禁 + reports/782-text-record/verification.md——最终基线全新旧门禁（770/771/741/745/767/772 + 新 gate）与故意失败控制，全部 mandatory 无 skip、locked/fmt/零 warning、制品及环境收据 hash 绑定。
blockers:无。next_responsible_role:work（T-06）。

2026-10-15，stage:work，plan_id:PLAN-782，plan_revision:1，outcome:pass（T-06 完成=6/9，继续 T-07）。
worktree_path:`D:/autostack/.wt/lang-782/auto-lang`；branch:`plan-782-dev`；base_commit:`4568e4ea7`；code_commit:`8558863ec`（gate 脚本+verification.md；前置修复提交 `836635a2e`）。dependency_revisions:同 T-01 收据；runtime/text 零改动（lib/合同 sidecar hash 入收据）。
task_ids:T-06。evidence：新 gate 25 步全绿 0 failed（verification.md §4 逐步表），绑定冻结基线 `836635a2e`（gate-20261015-093737）；ac-core 431/0（四 text_record target 在场）、runtime/text 20/20、双 workspace 零警告+fmt clean；11 个 gate 案例全过（TP01 exit30+缓冲配对、TP02 exit9、HP01 tr01 exit2 精确 trace、TN01..08 钉码、profile 隔离、capability 控制、SENTINEL、身份五针）；旧五门 771/741[-SkipMainGates]/745/767/770 实跑全过。
裁定记录（review 需知）：①**770 gate scope 归因**——770 脚本 scope 步 allow-list 仅 770 路径，后继树按定义必败（本计划 18 个域内 artifact 被标）；本 gate 不修改已合入的 770 脚本（旧面字节不变），实跑全程并断言唯一 [FAIL]=scope 步、被标集与同 git diff 重算一致且 ⊆782 允许清单——真实越界改动仍令本步红，scope 不变式重基 782 域；770 尾部 hashes 步（纯记录）中止由本 gate 步骤 25 覆盖。②**T-05 缺口修复 @836635a2e**——FieldGet `Str` 拼写（verify 兼容两拼写）在 TStep::Field 落标量臂→text 操作 unreachable panic（违反 Checked 可 lowering 合同；tr01 fixture 拼写恰为 Str，HP01 首跑抓获）；修复=视图装载臂扩 StrBorrow|StrOwn，永久回归 hir_fixture_str_typed_field_get_pe（tr01 端到端 exit 2+精确 trace）。③gate 脚本 PS 5.1 陷阱（[string] 参数默认空串致 $null -ne 守卫失效）第 1 轮 13 步后暴露，IsNullOrEmpty 修复后第 2/3 轮全绿；三轮原始输出在 receipts。
blockers:无。next_responsible_role:work（T-07：proposed-spec-delta.md 冻结 SD-01..06 + 支持/拒绝/取证声明核对 + 源文件 manifest + 前置反例组合闭合核对；完成后交 execution_done=7/9）。

2026-10-15，stage:work，plan_id:PLAN-782，plan_revision:1，outcome:pass（T-06+T-07 完成=7/9，work 交付，交 final review）。
worktree_path:`D:/autostack/.wt/lang-782/auto-lang`；branch:`plan-782-dev`；base_commit:`4568e4ea7`；code_commit:`b025a85aa`（gate 绑定基线 `9726a412c`，其后仅 verification.md rebind 文档差）。dependency_revisions:同 T-01 收据；runtime/text 零改动。
task_ids:T-06、T-07。evidence：T-06——新 gate `scripts/verify-ac-text-record-782.ps1` 25 步 + `verification.md`（绑定链：836635a2e→gate-20261015-093737 431/0，9726a412c→gate-20261015-100236 **433/0** 最终）；T-07——`proposed-spec-delta.md` SD-01..06 冻结 + 附A 反例闭合零缺口 + 附B manifest。
就绪记录（work-owned）：AC-01..06 全部有测试/gate 证据（verification.md §2 逐 V 落点、§4 逐步表）；合同内开口=零（V-04 两层 wrapper 缺口已补 0318bcb93，Str-拼写 lowering 缺口已修 836635a2e 带永久回归）；前置反例九类组合闭合（delta 附A）；未决合同决策=无。review/merge-owned 待办（不阻塞本交接）：T-08 独立复审（冻结于 b025a85aa）；T-09 合入+canonical 沉淀+归档；主仓裸 cargo t 等 merge-owned 门禁按 2026-10-14 裁定由 merge 在冻结提交执行（本计划不触 crates/，根档不适用）；批回归到期判定归 regress。
裁定携带（review 需复核）：①770 gate scope 归因消费（verification.md §4b——真实越界仍红）；②Str/str 两拼写 FieldGet=视图读取（836635a2e）；③770 r4_p07 形状在组合域需 mut 链转发（view 参数不可 mut 再借——0318bcb93 测试形状与 770 text 域形状的差异属域规则，非合同放宽）。
blockers:无。next_responsible_role:review（T-08：独立上下文、同一实施树，冻结 b025a85aa 审计 AC01..08/SD01..06/反例 oracle/遗漏债务/健康；pass→reviewed=8/9）。


## 9. 复审记录（续：final R1，2026-10-15）

stage: review；review_scope: final；reviewer_identity: 本次 Codex 独立检查会话 `/root`（未参与实施，未创建额外 agent，模型标识未暴露）；plan_id: PLAN-782；plan_revision: 1；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-782/auto-lang`；branch: `plan-782-dev`；reviewed_commit: `b025a85aa38197700763b944d1b7fec2ecbc4ad0`；base_commit: `4568e4ea795996a759b4747fc56b30a30c4308db`。入口/出口 HEAD 相同且实施树 clean。dependency_revisions/spec_inputs/Spec delta 冻结 SHA256 见 [R1 完整报告](attachments/782/review-r1/REVIEW.md) 与同目录 spec-inputs.tsv / proposed-spec-delta.frozen.md。

本轮独立复验四个 text_record target **71/71**，locked all-target check / fmt / diff 检查通过；另跑 14 个源码反例/合法兄弟及真实 Bundle 9 个控制，发现六项 blocking：F-782-R1-01 嵌套资源字段通过 let/return/ctor 隐式复制并双 free；02 同语句 move-return 重建先释放返回缓冲、PE trap70；03 动态短路循环跳过侧释放上轮聚合临时；04 文本/聚合分开清理破坏统一 LIFO；05 Move.inner 越界在结构界检前 panic；06 mandatory V-07 无实际组合故障注入，模拟 P6/清理计划形状不能证明 status 回滚与 fatal 语义。每项关联 AC/Task、原文/预期/实测/修复方向及控制证据见报告。

acceptance_results: AC01 已检面 pass；AC02/03/04 fail；AC05 partial/fail；AC06 partial；AC07 本轮审查完成但验收 non-pass；AC08 pending merge。SD01..06 目标和有限范围理由可追溯，但支持与清理实证声明受上述 findings 反证，不批准发布。
pending_closeout_gates: fresh final pass 后由 merge 在冻结提交执行适用联合收口门禁；本计划不触 crates/，根 t/tv 等不适用；tf 仅主检出批量档到期判断。本轮未跑根整档、未合入/归档/改实现/canonical/ledger。
next: 同树 work 修复全部 findings，补逐项与兄弟组合永久测试、纠正 verification/delta 取证声明、提交后再送审；T02/04/05/06/07 重开，保留 T01/03，status=executing，current_step=2/9，T08/09 pending。六项同时登记 KNOWN-DEBT，均为合同内阻断，不以登记债务视作完成。

2026-10-15，stage:work（R1 needs_fix 修复轮），plan_id:PLAN-782，plan_revision:1，outcome:pass（F-01..06 全修复=7/9，交 final review 复审）。
worktree_path:`D:/autostack/.wt/lang-782/auto-lang`；branch:`plan-782-dev`；base_commit:`4568e4ea7`；code_commit:`a52a54472`（修复链：`638b10dbe` F-01..06 修复+15 永久回归 → `56dd15748` verification.md §5b 修复轮收据+delta 更正 → `a52a54472` gate 重绑收据）；R1 修复后基线 gate **25/25 全绿**（gate-20261015-124603 绑 `56dd15748`，ac-core 448/0、runtime 20/20、旧五门+归因 770；`a52a54472` 相对绑定基线仅 verification.md 文档差）。dependency_revisions:同 T-01 收据；runtime/text 零源码改动（23 符号原样，注入走既有控制符号）。
task_ids:T-02/T-04/T-05/T-06/T-07（R1 重开面）。evidence：复审反例原文重放 14 探针全闭合（review-r1/replay.py 于修复后基线）——F-01 三例前门拒（resource-assign/ownership-required×2）+clone 对照过；F-02 move_return exit 3 单一 FREE（fresh 对照同形）；F-03 short_loop 零 FREE-INVALID；F-04 双向创建序 LIFO 正确；F-05 Move.inner=u32::MAX 受控 verify.bad-reference（5 个真实程序化变异测试，含取证纠正——T-02 收据声称的变异测试当时未物化，现已存在）；F-06 V-07 真实交付（--rt-fail-nth-alloc/--rt-set-budget/--rt-fail-ledger-nth wrapper 前奏注入既有控制符号 + 组合域 status 失败部分输出 trap 前逆序回滚；5 oracle 测试：clone/ctor 部分回滚精确 trace、旧目标保全、预算恰好 vs 差 1（-4/-5 区分）、账本@1、零长控制）。全套 ac-core **448/0**（新测试 86：hir 18/source 21/resources 27/native 20）；check --all-targets 零警告；fmt clean。
不变式修复范围（非逐例）：①拥有门按值形态+类型（allowlist=RecordCtor/RecordClone/Call/Move）覆盖四入口；②replace_frees 统一扣除 RHS-move 目标（Str 字段位+整根两臂）；③统一 pending 创建序表+聚合存在标记（语句尾/回滚共用守护释放，嵌套逻辑组合）；④出口逐字消费计划侧交错 buffer_frees（计划本已统一，消费侧两阶段 drain 修复）；⑤结构界检先行（check_body 先于语义规则+纯函数门）；⑥回滚发射门控组合域（纯 text 域 trap 合同字节不变——旧 770 测试全绿）。
合同关系澄清（F-06 裁定）：冻结设计 §5.3/§7 的"编译器发射的逐字段回滚"以**语句部分输出**粒度实现（ctor 字段实参缓冲/clone 逐字段克隆即语句 pending 临时——与设计语义一致）；status 通道=runtime 负 len 返回值，回滚后仍 fatal trap（无语言级 Result，沿 §5.3 上层 fatal 承诺）；注入=CLI 测试面非语言入口（沿 770 §9.2 无语言入口裁定）。
blockers:无。next_responsible_role:review（T-08 R2：独立上下文、同一实施树，冻结于 gate 重跑绑定基线；重点复核 §5b 六项闭合与 V-07 oracle）。

## 9. 复审记录（续：final R2，2026-10-15）

stage: review；review_scope: final；reviewer_identity: 本次 Codex `/root` 独立检查上下文（未参与实施，未创建额外 agent，模型/会话标识未暴露）；plan_id: PLAN-782；plan_revision: 1；outcome: **needs_fix**。
worktree_path: `D:/autostack/.wt/lang-782/auto-lang`；branch: `plan-782-dev`；reviewed_commit: `a52a54472728755027078fe712a1de6d2002f905`；base_commit: `4568e4ea795996a759b4747fc56b30a30c4308db`；入口/出口 HEAD 相同、树 clean。dependency_revisions/spec_inputs/冻结 delta 与 hash 见 [R2 完整报告](attachments/782/review-r2/REVIEW.md) 及同目录 TSV/冻结副本。

定向四 target **86/86**，locked build/check all-target/fmt/diff check 通过；R1 原文14项重放与真实 Bundle 9 控制已复验。另选8个嵌套短路/部分重建反例及合法兄弟，发现两项 P1 blocking：F-782-R2-01 外层 LogicalRhs 将内层 Agg.present 覆盖常量1，嵌套 &&/|| take→skip loop 仍 FREE-INVALID；F-782-R2-02 整根 move 后字段赋值错误 reown 整根，产生悬空字段读取 trap70 或重复 FREE-INVALID。旧原例闭合不等于完整不变式闭合。

acceptance_results: AC01/02 已检面 pass；AC03/04 fail；AC05 深输入与变异已检面 pass、发布历史面未全重跑；AC06 partial；AC07 final non-pass；AC08 pending。SD01..06 元数据/有限范围保持，支持与零缺口实证声明仍受反证，不批准 canonical 发布。
pending_closeout_gates: fresh final pass 后 merge 在冻结提交执行适用联合收口门禁；无 crates 改动，根整档不适用；tf 仅主检出批量到期判定。本轮未合入/归档/修改实现或 ledger。
next: 同树 work 修复两项，补诊断与永久组合 oracle、更正 verification/delta 后提交再送审；T04/05/06/07 重开，保留 T01/02/03，status=executing、current_step=3/9，T08/09 pending。连续两次 final needs_fix，下一交接必须带“剩余不变式/先前为何漏检/适用路径与状态/设计或oracle调整/当前结果”诊断表并实际验证调整，详见 R2 报告。

2026-10-15，stage:work（R2 needs_fix 修复轮，含两连败诊断），plan_id:PLAN-782，plan_revision:1，outcome:pass（F-R2-01/02 修复=7/9，交 final review R3）。
worktree_path:`D:/autostack/.wt/lang-782/auto-lang`；branch:`plan-782-dev`；base_commit:`4568e4ea7`；code_commit:`17508b703`（修复链：`7e59e06f3` 两项修复+4 永久回归+预算控制纠正 → `17508b703` verification §5c+delta+gate 重绑）。dependency_revisions:同 T-01 收据；runtime/text 零源码改动。
**两连败诊断（技能检查点，全文见 verification.md §5c）**：R2-01 剩余不变式=聚合存在标记须跨嵌套合流组合（外层 join 转发内层标记值、仅无条件创建传 1、外层 skip 恒 0）；早前漏因=R1 嵌套回归两 clone 全在全真路径且首 if 提前 return，"skipped if"整语句未执行、无跨迭代 take→skip 重入，标记组合从未被执行。R2-02 剩余不变式=字段赋值永不复活已整体移出的根；早前漏因=R1 F-02 只修 replace_frees 调度、原样保留 reown 状态翻转——推理"释放什么"而非"根是否还可写"。
task_ids:T-04/T-05/T-06/T-07（R2 重开面）。evidence：复审 extra/more 八控制原文重放全过——nested_and/nested_or_take_skip exit 3 零 FREE-INVALID（一次 CLONE/FREE 对）、outer_skip/nested_or 对照保持、partial_reown/nested_reown 前门拒 verify.use-after-move、field_live_control/whole_rebuild_control exit 3 正确；新增标量位与跨语句两形同样前门拒。永久回归 +4：`f782_r2_01_nested_marker_composition`（嵌套 &&/||-take-skip/三层，精确 trace）+ `f782_r2_01_nested_marker_rides_the_rollback_releaser`（fail@3 回滚只释放已执行 clone——共享释放器消费组合标记）+ `f782_r2_02_field_write_on_moved_root_rejects`（四形钉码）+ `f782_r2_02_live_replacement_and_whole_rebuild_stay_legal`（两对照精确 trace）。V-07 预算"差 1"控制 6→7（峰值 8 恰差一字节，措辞纠正按复审意见）。全套 ac-core **452/0**（native 24）；零警告；fmt clean；R2 修复基线 gate **25/25**（gate-20261015-134523 绑 `7e59e06f3`；`17508b703` 相对绑定基线仅文档差）。
blockers:无。next_responsible_role:review（T-08 R3：独立上下文、同一实施树，冻结 `17508b703`；重点复核 §5c 两项闭合、嵌套标记组合/回滚消费、字段位×语句期全矩阵）。

## 9. 复审记录（续：final R3，2026-10-15）

stage: review；review_scope: final；reviewer_identity: 本次 Codex `/root` 独立检查上下文（未参与实施、未创建额外agent，模型/会话标识未暴露）；plan_id: PLAN-782；plan_revision: 1；outcome: **pass**。
worktree_path: `D:/autostack/.wt/lang-782/auto-lang`；branch: `plan-782-dev`；reviewed_commit: `17508b703c84e6f24496ede7b908621a1c05f365`；base_commit: `4568e4ea795996a759b4747fc56b30a30c4308db`。入口/出口HEAD相同、树clean；auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 clean。dependency_revisions/spec_inputs见 [R3完整报告](attachments/782/review-r3/REVIEW.md) 与baseline-hashes.tsv/spec-inputs.tsv。

已核实§5c两连败诊断与实际设计/oracle调整：外层join转发已有内层Agg.present、字段写统一拒绝复活moved根。四target **90/90**，locked build/all-target check/fmt/diff检查通过、零警告；R1/R2共22原文控制重放、9真实API控制复验；另增15个组合控制——ctor/clone/拥有call-return×正常/fail@3/fail@4=9条真实PE覆盖join后标记0/1的回滚消费，三字段终型×同/跨语句move=6前门拒。全部断言通过，八项历史blocking关闭，无新增阻断。

acceptance_results: AC01..07 pass，AC08 pending merge。SD01..06目标与有限支持/拒绝/non-goal审核通过；冻结delta SHA256 `6c5ea9dd9dd3b16310160ffe8a8f4b935983a1570479103aaac052e0698b5daf`。canonical七输入与R2相同。work联合gate收据7e59e06f3→17508b703仅两文档差、24项gate输入hash重验一致，明确复用其代码相同基线证据；非本轮重新跑全档，不替代merge门禁。

pending_closeout_gates: **merge-owned pending**——冻结提交及实际共同基线的experimental/ac-core/runtime/text locked check/test/fmt、782新旧联合收口gate/能力/事务控制；代码或依赖变化须受影响复审+新冻结门禁。无crates/根Schema改动，根t/tv/tt/tb/tu/taa/docs_gen不适用；tf仅主检出批量到期判定。此pass不表示交付完成或允许跳过门禁合入。
next: merge（T09：门禁后合入、canonical/导航/ledger/index、归档计数与guard清理）；status=reviewed、current_step=8/9，T08完成，T09仍pending。本轮未修改实现/canonical/ledger、未合入/归档/清理工作树。归档取证使用R3冻结实际hash与案例，不使用verification.md历史节选的旧工具/PE指纹。

## 11. spec-sync 回写记录（merge，v1 惯例）

2026-10-15，stage:merge，plan_id:PLAN-782，plan_revision:1，outcome:pass，completion_kind:delivered。
prepared：R3 pass（reviewed_commit `17508b703`）+ 冻结 delta（SHA256 6c5ea9dd…）→ canonical 落地 `99d92513f`（SD-01 新 source-text-record.md；SD-02..06 五文件；三模块 plans.md 行）。landed：merge-owned 联合门禁 25/25 于冻结提交实跑绑定 gate-20261015-141557（25 步：双 workspace 零警告/452 测试/11 案例含缓冲配对与故意失败控制/旧五门/归因 770 scope/24 输入 hash）；rebase master 后 17 对 range-diff 全等 + 1 对（spec deposit）仅 plans.md 行合并消解（无丢弃）；主检出 `git merge --ff-only` 落地，master tip==plan-782-dev tip==`99d92513f`。ledger_refreshed：`.autoos/specs.json` v22→v23 守护式离线投影（channel=guarded offline projection, no store writer reachable；备份+零扰动 diff 证明：仅 version + P782-M/P782-1/P782-2/P782-T/P782-R3 五追加，无删改）；`python scripts/spec-index.py` 再生（31 projects）。archived：本文件 git mv 至 docs/plans/archive/，status: archived，9/9；R3 复审簿记与附件 r1..r3 随 `a1cd75457` 入库。cleaned：见下方 cleaned 行。生产制品观察：本仓为语言/实验库仓，无后端二进制/守护进程/web bundle 部署面，Landing≠Deployment 项不适用（无可重启生产进程）。

2026-10-15，PLAN-782:r1 cleaned checkpoint：wt-guard clean（无 reparse point）→ `git worktree remove` + `git branch -d plan-782-dev`（was 99d92513f=master tip）→ auto-down 兄弟（detached c39ac7f，clean，零修改）经其主检出移除 → 组目录 `D:/autostack/.wt/lang-782` 删除。批回归到期判定：782%5≠0；`.last-batch-regression.json` 见下。merge 9/9 全checkpoint 完成（prepared/landed/ledger_refreshed/archived/cleaned），completion_kind: delivered。
批回归到期判定（2026-10-15）：收据 last_covered_plan_id=781 @ 2026-10-15T03:55Z（~2.5h 前）——782%5=2≠0 落地不触发；<48h 不触发。**未到期，不运行**；下一次到期由后续 %5 计划或 48h+合并规则触发（届时 PLAN-782 的 experimental/ac-core 面进入批量窗口）。
