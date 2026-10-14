---
plan_id: PLAN-782
status: executing
feature_name: AC 文本与 record 组合：拥有型字段、移动与清理
plan_revision: 1
author: [Codex]
created_at: 2026-10-14
updated_at: 2026-10-15
current_step: 3
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
| [x] | T-02 | work | T-01 | ✅ 已完成 @697e3f811：`schema/text-record.atom`（text∪record 并集 + record_clone/move 双新 case，分支全量 require/forbid）；hir.rs `HirDomain::TextRecord` + `ExprKind::RecordClone{record,base}`/`Move{inner}`（设计 §4.3 修订：moved 旗标改为 **`expr.move` 包装表达式**——77 个 Stmt 构造点零扰动，单一表示，`CallBinding.moved` 在新域禁用）；descriptor binder/writer/domain/TypeUse 内建词（Str/str/char 限 text+text-record 域）；verify：`verify_text_record` 入口（T-02 结构面 + 能力派生门）、check_profile_ops 四域化（旧四域互拒新面）、`record_resource_map` 传递分类（pub 供 T-04）、check_record_module 字段门（str→`verify.field-borrow`、Str/char 放行、域感知映射 4 处）、owning-position/界检/init 三走查 + ExprScan 补臂、`check_text_record_rules`（move 形状 `verify.partial-move`、move 参数类型门 `verify.ownership-required`、裸 let/assign/return 分类拒 resource-assign/ownership-required）；main.rs check/build 分派（build 在 T-05 前受控 `backend.lowering-unsupported`）。text_record_hir **13/13**（V-01 身份五域隔离+writer roundtrip+错维拒；V-02 Str/str/char 字段+传递分类+cycle+copyable 保持 771 值复制+clone 名义拒；V-08 ctor/call binding 越界、clone 跨型、requires 删除受控拒）；前置回归 79 例全绿、零警告、fmt clean | 01/02/03 |
| [x] | T-03 | work | T-02 | ✅ 已完成 @3f0366f6d（核心链 @24171d462 + F-A @a2aa02a5b）：source 七模块/typecheck/adapter 组合面接线——构造（命名/位置双形、AP-4 提升补齐位置构造与静态/实例方法实参位）、字段/方法（Str 字段 view 接收者、771 static/instance/mut 面保持）、mode/move/clone（内建 clone 仅组合域绑定、用户声明 clone → `source.duplicate`、move 参数 `mode: move` 发射+可写根、Move 实参整体转移不被 borrow_view 包裹、缺 `.move` 调用点对称门）；verify 侧修复 Ref-vs-Builtin 字段类型盲点（资源分类/field-borrow/bare assign/return 门对源码 Bundle 生效）、771 调用绑定模式匹配扩展至组合域（含 MoveOwn 三臂）、组合域 Str 局部门白名单。text_record_source **19/19**（V-02/03/05 source 腿、partial-move 含死分支静态检查、旧域保持 N06/770-move）；全套 ac-core **394/0**、零警告、fmt clean | 02/03/05 |
| [ ] | T-04 | work | T-02/03 | text_resources.rs/verify/native：根与字段来源、continuation合流、路径清理/能力推导；runtime/text仅必要组合helper，沿用账本不造第二套；text_record_resources物化V03/04/06/07/09 | 03/04 |
| [ ] | T-05 | work | T-04 | native/main/shared finish_build：资源布局、view/mut/move/out-return、非重叠提交；text_record_native+双腿fixtures物化V02/05/09/10/11/12，真实PE trace/值/计数达oracle | 01..06 |
| [ ] | T-06 | work | T-05 | 新verify-ac-text-record-782.ps1+reports/782-text-record/verification.md：最终基线全新旧门禁和故意失败控制，全部mandatory无skip、locked/fmt/零warning，制品及环境收据hash绑定 | 05/06 |
| [ ] | T-07 | work | T-06 | reports/782-text-record/proposed-spec-delta.md冻结SD01..06；核对支持/拒绝/取证声明、源文件manifest及前置反例组合闭合；canonical不在此步发布；work交execution_done=7/9 | 06 |
| [ ] | T-08 | review | T-01..07完成，writes暂停 | 原树独立审计AC01..06与SD、反例/永久测试oracle/遗漏债务/健康；pass→reviewed=8/9，needs_fix回work，不修改实现 | 07 |
| [ ] | T-09 | merge | T-08 fresh pass与合入授权 | 实际合入/SD/导航/.autoos/specs.json/spec-index；archive状态和计数；批回归到期用regress；bash wt-guard clean后remove/branch/组清理收据，实际完成9/9 | 08 |

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
