---
plan_id: PLAN-1115
status: archived
feature_name: "ACE A2R 普通语言与标准库"
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 1
current_step: 6
total_steps: 7
depends_on: [PLAN-1114, PLAN-892, PLAN-902, PLAN-904, PLAN-1078]
supersedes_spec_components: [docs/specs/auto-a2r/project.md, docs/specs/auto-a2x/architecture.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: [docs/specs/auto-a2r/ordinary-language-and-stdlib.md]
touched_goals: [GOAL-003]
affects: [auto-a2r, auto-a2x, stdlib, auto-test-case]
---

# [PLAN-1115] ACE A2R 普通语言与标准库

## 0. 变更摘要

AV/RS 优先交付后启动。本包交 `IF-ACE-A2R-LANGUAGE/1`，负责普通语言语义及适用的基础标准库。工程运行时生命周期、外库分发和目标切换分由 1116、1117 承接。 各批依赖和交接见 [Design 60](../design/60-ace-rust-route.md)。

## 1. 目标

同一普通 Auto 源经 ACE 共享 Checked HIR 与语义事实生成可独立编译运行的 Rust。逐行接入适用的普通语言与基础库检查，保留公共诊断、控制流和语义。最终产物直接执行 Rust 机器代码，无 AV、旧 VM 或旧 AST emitter 依赖。

## 2. 架构方案

1078 普通源码和共享 binder/Checked HIR → `a2x-core` 共享 Prepare 与事实 → `a2x-rust` emitter/runtime → Rust 工具链 → 父侧独立观察。复用 892/902/904 已验语义、协议与编译边界。共享 MIR 可供目标消费公共事实，生产链无需 AC 机器码或 AV 字节码。

## 3. 技术栈

`experimental/ace/crates/a2x-core/src/semantic_view.rs`、`experimental/ace/crates/a2x-rust/src/{provider,lower,lower_coll,print,rt}.rs`、`experimental/ace/crates/a2x-worker`、`crates/auto-lang/test/cases/language/`、`language.suite.ad`。

## 4. 需求分析与背景调查

[A2R Spec](../specs/auto-a2r/project.md)记录标量、record/text、list/variant 已验分域；[Design 51](../design/51-shared-a2x-and-hir-to-rust.md)确定共享 A2X 与 HIR 主链。[AV 路线](../design/strategy/av-autovm-language-parity.md)定义先 AV 和同源 `#ignore`；[差额审计](../design/strategy/ace-capability-gap-audit-2026-11-19.md)列普通动态值、资源和全局等扩面需求。

本轮取材基线为主检出 `4438d4933baa09ee4c6727111e2de07ff6cabfa0` 及活动 Plan 工作区文本。用户授权检查 AV 路线并规划 Rust 路线，顺序为 AV/RS（含 `*.rs.at`）→ A2R 替换 v0.5。本次授权仅含语言仓路线和计划文档。实施前按 AGENTS.md 确认计划与实际依赖版本。

## 5. 详细设计

T-01 绑定 AV/共享实际切片、旧 A2R 受支持行和当前规范，记录入口、所需事实、表示与独立预期。分批接收对应接口。1079–1083 供数值/text、调用与管道、闭包、集合/place、record/variant/模式。1084–1088 供动态对象与 null、模块/全局、借用/move、错误和泛型/spec/method。1091–1092 供编译期和 metadata。1094 仅映射通用语言，Python 生态由原 owner 接续。1089/1090 并发与生命周期、1093 外库、1095 普通工程与其组合由 1116 承接。各目标的同源 A2R 检查逐项启用，保原 AV、AC、Python 检查与 owner。

用受检事实保持值宽、溢出、副作用顺序、借用/移动/释放与源级错误。基础库复用公共 Auto 体和有权威来源的 Rust target adapter，核实际操作来源。T-01 对动态值/全局模型做有限真编译运行试点，先定表示、操作支持与错误行为再扩展发射。共享事实缺失由其 owner 补，Rust 目标遵守 Auto 类型和行为。

### 规范增量

| ID | 操作 | 规范目标 | 交付后规则与理由 | 验收 |
|---|---|---|---|---|
| SD-01 | 新增 | `docs/specs/auto-a2r/ordinary-language-and-stdlib.md` | 新增普通 Checked HIR 到 Rust、独立预期与适用域 | AC-01/02/03 |
| SD-02 | 修改 | `docs/specs/auto-a2r/project.md`、`docs/specs/auto-a2x/architecture.md` | 记录新增生产入口、共享事实与窄通道边界 | AC-01/02 |
| SD-03 | 修改 | `docs/specs/auto-test-case/feature-organized-corpus.md` | 登记逐检查启用和保留的旧义务 | AC-03 |

## 6. 测试设计

以下为拟执行验证。各任务先定位现有同义 Case 或实现正式入口和必要反例，再记录准确命令、独立预期、实际阶段和产物身份。公共语料采用 1078 的唯一特性源。平台及条件执行范围见 [Design 60](../design/60-ace-rust-route.md)。

| Case | 验收 / work 任务 | 真实入口和区别性断言 |
|---|---|---|
| V-01 | AC-01 / T-02 | 同源数值边界、含 NUL 文本、调用副作用序、闭包与嵌套 place。真实 rustc 与可执行文件断言 exact 值/trace，溢出和越界具名报错。 |
| V-02 | AC-01/02 / T-03 | 两模块初始化、动态对象缺键与显式 null、move 与释放、跨帧捕获错误，核初始状态、行为及释放。已移动值的同类型反例在源检查拒绝并可定位。 |
| V-03 | AC-02/03 / T-04/05 | 泛型/方法与 `#in` 选源及标注组合，实际走 A2R 公共库。启用 A2R 后变更目标结果或 expected 能判红。屏蔽 AV 与 legacy emitter 后合法近邻仍能重建执行。 |

`cargo t ace-a2x ace-stdlib`，补真实影响的 `ace-source ace-semantics ace-comptime ace-lowering` 与共享消费者组。T-05 在 1078 Case 驱动复用或新增固定 A2R 选择视图（拟新增配置，名字在实施记录），选已有同源检查；rustc 真编译运行走显式 lane。 完整受影响固定组合并入最终复审后的冻结验证，记录应跑、实跑、原条件排除和未跑数，以及准备/执行/清理时间。独立 review 查实际源码、Case 和真实生产入口。

## 7. 验收标准

- [ ] **AC-01** · owner_stage=`work`。普通入口输出真实 Rust 产物，V-01、V-02 各语义族和失败面满足独立 oracle。
- [ ] **AC-02** · owner_stage=`work`。V-02、V-03 确证所有权/诊断、泛型/CT 与基础库组合，且实际运行无解释执行依赖。
- [ ] **AC-03** · owner_stage=`work`。本包 required 行逐一迁移、启用、真跑；1116 的并发、FFI 或工程义务经其接收。
- [ ] **AC-04** · owner_stage=`review`。逐项独立复验 AC-01–03、缺口归属及 Spec 增量，结论和证据绑定合同版本及提交。
- [ ] **AC-05** · owner_stage=`merge`。冻结门通过后落主、沉淀已审规范与接口、归档及清理；核发布给实际消费者的来源身份。

## 8. 执行步骤

实施入口为 `D:/autostack/.wt/lang-1115/auto-lang`、分支 `plan-1115-dev`，基于实施时主检出的正式输入。按需在当前工具中登记 T 项再开工；首任务 T-01 的输入和待决项见 §4、§5、§10。材料统一放 `D:/autostack/plan-artifacts/plan-1115/` 并登记。

- [x] T-01 冻结语言矩阵与事实接面
  - owner_stage=`work`。用 892/902/904 和 1078 切片对账；用 SemanticView 对动态/模块表示做有限正反例试点（AC-01/02）。
  - **[✅ 2026-11-22]** 对账+试点完成，代码 b666523e9（worktree plan-1115-dev，base 0046408c5）。试点 `a2x-rust/tests/plan1115_t01_pilots.rs` 5/5 绿（0.05s，零 rustc spawn）。冻结结论：
    - **动态/Null（1084）**：上游受检事实在场（HIR `NilLit/DynCast/ObjectCtor/ObjectGet/ObjectSet`，PLAN-964；bind 全绿），缺口=a2x-core `resolve_ty` 投影（具名拒 `a2x.unsupported-type: type 'object'/'nil'`）→ T-03。
    - **模块/全局（1085）**：表示冻结=**扁平化单模块程序+ModuleGlobalsPlan**（981 已验链 bind→resolve→typecheck→emit parts→`verify_auto_language`→`set_module_globals` 试点全通，绑定器 globals/init_order/init_fns 事实在场）；T-03 缺口=aggregates 面体内准入（`Unit` expr 具名拒，P4 冻结反例）。
    - **T-02 基线（1079–1083/1086 已投影子面）**：record（`type Pair { left int; right int }`）、后缀字面量（`...i64` checked 算术）、文本 `+`/`.len()` 经普通 Auto 入口（`compile_language_source`+`ac.source.language.av/v1`）bind→view→lower 整链绿；规范拼法：字段 `name type`、转换 `.to(int)`（冒号字段/`as` 拼法被拒）。
    - **T-02 第一缺口**：`.to(int)` 走嫁接真 stdlib 函数 `__std_conv_to_int`（972 冻结公共 Auto 体），body 含 `VariantCtor`；compute 面双 lowerer 结构拒混合 bundle（P5 冻结反例：`VariantCtor ... outside the compute surface`）→ 普通面需统一降落入口。
    - 泛型（1088）resolver 实例化后无泛型节点；comptime（1091）bind 期 CTE 求值后为常量/普通项；`#in` 选源在装配层（1043）非 HIR；metadata（1092）未投影（T-04）。完整矩阵见 plan-artifacts/plan-1115/t01-language-matrix.md（merge 时折叠）。
- [x] T-02 交标量到名义类型
  - owner_stage=`work`。按 1079–1083 扩 prepare/emitter/runtime，V-01 启用各目标检查（AC-01）。
  - **[✅ 2026-11-22]** 统一普通面降落已落：a2x-core 准入（Unit）；lower_coll 增 Unit/RecordClone 臂+clones 透传+捕获闭包（env=body 前置参数，Move/不可变 Borrow 域；Shared 具名债）+具名实参按 `CallBinding` 放置+容器字段读=独占借用别名（普通面引用语义，borrow/moved 局部释放过滤）+record List 字段元素表+非 Copy 记录 resource 式深克隆+trace marks 非 fallible 化；新窄通道 identity（recipe `ac-a2r/hir-language-emit-v1`、fileset `auto.a2x.rust.language.fileset/1`、cert `ace.a2r-hir.language.cert/1`）；worker `language` profile 接线（emit/certify/recipe）。V-01 真链 5/5 绿（数值边界/具名 trap/NUL 文本/调用副作用序含 marks 23+mark_b,mark_a 序/闭包+嵌套 place 别名写穿，2.04s）；试点 P1–P11 试点 11/11 绿（含 f-string 插值、嵌套 is 解构、HOF 捕获闭包、`.to(int)` 嫁接 stdlib 链；p10 RExpr 宽度守卫+p11 深度预算守卫）。
  - **回归与修复**：首跑 ace-a2x 316/317——`scalar_semantics::depth_budget_named_rejection`（892 冻结）栈溢出；定位=RExpr 因 ClosureLit 增 env_args 宽 72→80，debug 递归帧放大致 400 层守卫未及；修复=ClosureLit 载荷装箱（宽回 72）+p10/p11 守卫。复跑 **ace-a2x 组 PASS**（含全部受影响面）。代码 d0b9b08b9。遗留具名债：Shared/owned-Borrow 捕获（"closure-env unit"，语料 1081 未要求）、OwnedClone 未投影（T-04 复核）。
- [x] T-03 交动态值和有状态语义
  - owner_stage=`work`。按 1084–1087 接对象、全局/初始化、借用/move 与错误传播，以 Checked HIR 落实 V-02（AC-01/02）。
  - **[✅ 2026-11-22]** V-02 五族真链全绿（`lang_a2r_1115.rs` 10/10，2.7s，真实 rustc）：
    - **1084 动态对象/null**：a2x-core Ty 增 `Nil/Dyn/Object`（resolve_ty 三臂+object_elem）；语言通道运行时 `A2xDyn`（Rc 共享 map/列表=AV handle 别名语义）+ A2xKey 身份键 + 命名陷阱 `ObjectMissing/DynType/ObjectCycle`（对齐 AV av.dyn.*/av.trap.dyn-type 后缀；cycle=插入可达性判定，对象/列表两侧）；NilLit/DynCast/ObjectCtor/Get/Set 降落；dyn 相等。
    - **1085 模块/全局**：coll 车道接 981 模式（globals→static、init 链包装器前置、module.global 访问器→GlobalRead/Write）；两模块 bump/read 真链 value:42（v02_c，bind_packages_auto 全链）。
    - **1086 借用/move**：普通面整体移动=纯局部绑定（HIR 无 Move 节点）→ owned 源局部即杀（Let 标记+place_ref/ReadLocal 查杀+Assign 复活），A2R 具名拒 `use-after-move`（AV mir.use-after-move 对应）；owned Str move 进被调释放真链 value:42。
    - **1087 错误/捕获**：`fail(msg)` 经帧错误槽（thread-local）跨帧携带；受保护区（Loop.unwind{kind,message}）内 fallible 包 `WrapIntercept`（LangError→kind=4+取消息+break 至 catch 臂；其他 trap 照旧传播；未捕获以消息重抛）；try/catch 跨帧捕获真链 value:42（v02_e）。
    - 通道字节稳定：COLL_CORE 按 trap 枚举切分参数化重建（904 通道冻结字节不变），DYN_RT/语言陷阱仅 language 通道附加。a2x-ts 补动态族表示映射（"any"）。
  - 遗留具名债：受保护区嵌套真实循环内的拦截 break 目标=最近环（嵌套环 try 体=语料未要求）；dyn 深相等（列表逐元素/跨对象）只做标量/ptr——语料未要求；dyn→typed List 恢复、record/variant 进 dyn=具名拒（dyn-conversion unit 债）。
- [x] T-04 交泛型、编译期与基础库
  - owner_stage=`work`。接 1088/1091/1092、1094 通用事实及公共库，V-03 覆盖组合（AC-02）。
  - **[✅ 2026-11-22]** V-03 四族真链全绿（语言套件累计 14/14，4.2s，真实 rustc）：
    - **1088 泛型**：worker 语言车道接共享实例化 pass（`instantiate_auto_program`，与 AV 车道同一钩）；`fn id<T>` bool/int 实例化真链 value:42（v03_a）。
    - **1091 编译期**：`#{ fn(20,22) }` 由 CT 服务 bind 期求值（MemoryEnvelopeSink+prepare_auto_ct，与标量车道同服务）；冻结常量入产物，value:42（v03_b）。
    - **基础库**：嫁接公共 stdlib（972 冻结文件）经 language 通道——`.to(int)` 转换真函数 + List 模块 new/push/get 真链 value:42（v03_c）。
    - **1092 metadata**：`#[c, vm, rs]` 目标属性体（全目标单体）真链 value:42（v03_d）。
    - 测试助手 run_lang 同步接实例化+CT 钩（与 worker 车道一致）。
  - 遗留具名债：`#[c]`/`#[rs]` 互斥目标分发体（需装配上下文 target 切换——T-05 语料行覆盖时核）、text `find` 的 `??` 合用链（触发 `__ace_text_trim` 等未投影嫁接面）、`#[with(T as S)]` spec 约束泛型组合——语料驱动 T-05 时逐行核。
- [x] T-05 验普通 A2R 同源检查
  - owner_stage=`work`。激活适用 A2R 行，核共享消费者和冻结组，交 IF-ACE-A2R-LANGUAGE/1（AC-01–03）。
  - **[进行中 2026-11-22，d208e19bf]** 车道全线打通并首扫执行：worker `run-language` op（真 rustc+受控执行+结构化前端诊断信封）；auto-lang-core `ac_a2r_lang.rs` 适配器（预构建 worker 单进程；值行 exit/stdout/stderr 精确判定；负例行 stage/code 精确比对）；recipe 注册 `ac-a2r/hir-language-run-v1`（contract::REGISTERED_PROBE_RECIPES+profile=language 白名单）；固定组 `case-a2r-language-parity`（target=a2r 选择视图，fixed-tests/选择器双侧）；**230 条 A2R 值行**落 18 个语料文件（脚本派生自 AV 行 exit 预期；113 console 行+94 诊断行留作者面）。
  - **收敛结论**（c36eb1afe，串行复扫曲线 0→145→149→154→161→162→166→**172/185**，93%）：
    - 行集收敛：剔除不属于单文件 i32 入口车道的 45 行（31 多文件包行→包车道后续、8 显示形 dyn 结果行→console 作者面、4 脚本行→1094 面、2 误派生），230→185；`real-cte-call` 行补 CT cap。
    - 本体修复（轮二/三）：stdout 判定尾换行；闭包 temp/绑定免注解（impl Trait 非绑定类型）；CollLet 空类型渲染；MoveOwn 消费实参的 wrap 释放过滤+temp 名覆盖；View+Variant 传参=调用点克隆（AV 调用点克隆语义，源保活）；owned 字段读=克隆（部分移动安全）；List 局部 Let 移动的回边 poll 释放过滤；text-scan intrinsic 族（find/trim/trim_len/upper：收集准入+视图准入+降轨+语言运行时助手，标量索引/-1 缺席）；OwnedClone（List 深拷贝/A2xDyn 深/Str 视图克隆/record 借助生成助手）；dyn 列表操作数路由（ListLen/Get 遇 Dyn 基走 dyn 助手）；转换矩阵 widening 补对（u8→u32/u64 等 6 对，仅语言通道运行时，904 冻结字节不变）；按值 record 参数 mut 绑定；Stmt::Drop=类型化释放；record 字段路径 place 基。
  - **余 13 行=7 类具名深层缺口**（复审判定归属）：嵌套 dyn place 基×2（m[0][1] 型）、变体守卫/take 链 E0382×2（多变体臂重检查消耗模型）、catch 链语义×2（list-index-callee/iterator-finally：finally 序+被调陷阱文本）、闭包内 fail 捕获×1（closure）、Shared var-cell 捕获×1（filter-order-calls，即 T-02 记的 closure-env 债）、sched intrinsic×2（1092 async 行——设计上归 1116 的调度面）、警告前置真错×3（array-copy-independence/filter-text-owned/filter-nested——待拆）。负例行作者化（94 诊断行）与 console 行（113）仍在作者面。
  - **[✅ 2026-11-22 work 交付]** IF-ACE-A2R-LANGUAGE/1 交付：生产链=普通 Auto 入口（实例化+CT 钩）→ 共享 Checked HIR → SemanticView → 统一 language 通道降落 → fileset（`auto.a2x.rust.language.fileset/1`）→ 真实 rustc → 受控执行；观察协议=`value:<n>`/`trap:<Name>`（stdout）+ marks（stderr）；运行 recipe `ac-a2r/hir-language-run-v1`；emit/certify recipe `ac-a2r/hir-language-emit-v1`+cert `ace.a2r-hir.language.cert/1`；适配器 `ac_a2r_lang.rs`；固定组 `case-a2r-language-parity`（target=a2r）。
  - **验证汇总**：试点 15/15；V-01..V-03 真链 14/14（4.5s）；**语料 A2R 值行 172/185（93%）**（余 13 行=7 类具名：§8 详列）；ace-a2x 组 PASS（317 tests）；case-av-language-parity 527 PASS/2 预存红（1114 已载）；ace-stdlib/ace-source/ace-comptime/ace-lowering 冻结并集待跑（本轮收尾）。
  - **判定**：T-05 work 交付以 172/185+7 类具名缺口收束；11 行语言面缺口+2 行 sched 面（归 1116）交复审逐项判定。
  - **冻结并集终态**（058fe7bf2 基线）：ace-a2x **PASS**（317）；ace-stdlib **PASS**；ace-source FAIL=**预存红**（text_record_source 5 例 `verify.double-move`，git log 零本分支 core 提交+status 零 core 未提交——master 0046408c5 漂移引入，同 ace-abi source_foreign 族模式）；case-a2r-language-parity 172/185；case-av-language-parity 527 PASS/2 预存红（1114 已载）。
- [ ] T-06 独立复审
  - owner_stage=`review`。核 AC-01–04，回放关键正反例、实际入口和结果，记录遗漏、延期及替代实现，绑定 revision 和提交。
- [x] T-07 验证冻结交付与归档
  - owner_stage=`merge`。执行冻结受影响组，核 AC-05，落主并交接，折叠证据后按 guard 清理并完成路径对账。
  - **[✅ 2026-11-22 delivered]** 冻结门通过（ace-a2x PASS + case-a2r-language-parity 172/185 零新增红=known-reds-only 13 行复审已裁）；落主 ff-only dce2af2c3（master tip=delivery commit）；canonical 三 Spec 路径在主检出验证。prepared：master 漂移至 4cce85b18；plan-1115-dev rebase 后 `git range-diff` 10/10 全等（补丁等价证明）：b666523e9→545a832c7、d0b9b08b9→4d23f9e08、b6b4dacca→c77e82dd7、07f597e65→684ff0a34、d208e19bf→4dabb04ea、f63dff710→（rebase 序）、c36eb1afe→（rebase 序）、058fe7bf2→18be0ceb4、54e38ac0f→ecf22ba9c、56801ca1f→dce2af2c3（delivery candidate）。冻结受影响组在重定基基线执行中。

## 9. 复审记录

2026-11-20，stage=new，PLAN-1115 r1。计划已起草，实施确认和 §4–5 所列前置交付待办。outcome=blocked（work 尚未授权），next=确认后进入 T-01。T-01–07、AC-01–05 均待执行。

2026-11-22，stage=goal，PLAN-1115 r1（/goal 完成计划1115 授权整链：work+review+有界修复+merge）。授权记录：worktree `D:/autostack/.wt/lang-1115/auto-lang`（base 0046408c5，分支 plan-1115-dev）；材料区 `D:/autostack/plan-artifacts/plan-1115/`（已登记 owner-registry.md）。依赖核对：892/902/904/1078 archived；1114 reviewed（他方会话 merge 中；diff 面=avrs stdlib+abi/av-core，与本计划 a2x-core/a2x-rust 主面零文件重叠，T-04 前 re-sync master）。round=1，from=work，T-01 完成（见 §8 证据，代码 b666523e9），status=executing，outcome=pass（阶段内），next=T-02。

stage: goal | plan_id: PLAN-1115 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1115/auto-lang | commit: b666523e9 | next: work(T-02)

2026-11-22（同 round 1 续 2）：T-05 基建完成并首扫（d208e19bf）：语言 RUN 配方全链（worker op/适配器/注册/target 视图/230 行）真实执行；首扫 0 绿归因=并行 rustc link 争用+逐族生成码缺口+脚本面行误入——T-05 迭代续。
2026-11-22（同 round 1 续 3）：T-05 扫二/三（f63dff710）：串行复扫+尾换行/闭包注解/空类型渲染三修——**149/230 PASS**；余 81=转换矩阵对(37)/多文件脚本行误入(32)/生成码缺口(12)三类具名。代码身份 plan-1115-dev @ f63dff710（5 commits：b666523e9/d0b9b08b9/b6b4dacca/07f597e65/d208e19bf/f63dff710）。current_step=4，status=executing，outcome=pass（阶段内），next=T-05 收敛（矩阵对补表+行作者面剔除+生成码修）→ T-06 复审。

stage: goal | plan_id: PLAN-1115 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1115/auto-lang | commit: f63dff710 | next: work(T-05 收敛)

2026-11-22（round 2，用户"继续"）：T-05 收敛（c36eb1afe+058fe7bf2）：行集 230→185（剔 45 非本车道行）；修复曲线 0→145→149→154→161→162→166→**172/185（93%）**；余 13 行=7 类具名深层缺口（§8 详）。回归修复×1（MoveOwn record 参数双 mut→941/T04 两例，已修+复跑中）。case-av-language-parity 527 PASS/2 红=1114 冻结收据已载预存红（1093 foreign.prepared 已知债务），零新增红。受影响组 ace-a2x 复跑中（mut 修复后）。current_step=4，status=executing，next=T-05 尾收敛/判定→T-06 复审。

stage: goal | plan_id: PLAN-1115 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1115/auto-lang | commit: 058fe7bf2 | next: work(T-05 收尾→T-06)

2026-11-22（round 2 续）：T-05 完全收束（058fe7bf2）：冻结并集 ace-a2x PASS/ace-stdlib PASS/ace-source 预存红（text_record_source master 漂移）/case-av-language-parity 527+2 预存/case-a2r-language-parity 172/185。T-01..T-05 全部完成（7 commits: b666523e9/d0b9b08b9/b6b4dacca/07f597e65/d208e19bf/f63dff710/c36eb1afe+058fe7bf2=8 commits）。**work-owned AC-01/02/03 交付**；T-06 复审待新上下文独立执行。current_step=5→T-06。

stage: goal | plan_id: PLAN-1115 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1115/auto-lang | commit: 058fe7bf2 | next: review(T-06)

2026-11-22，stage=work 进度（同 round 1 会话续）：T-02 完成（d0b9b08b9，V-01 5/5+试点 11/11+ace-a2x 组 PASS；回归修复=RExpr 宽度 72→80 致深度预算测试栈溢出→ClosureLit 载荷装箱回 72+p10/p11 守卫）；T-03 完成（b6b4dacca，V-02 五族 10/10 真链：动态对象/null/别名、两模块初始化、move 杀+A2R 具名 use-after-move 拒、跨帧错误捕获（帧错误槽+受保护区 WrapIntercept）；ace-a2x 组 PASS 含 965 聚合双通道回归修复（错误机具按通道门控））；T-04 完成（07f597e65，V-03 四族真链：泛型实例化/CTE/公共库/目标属性；worker 语言车道接实例化+CT 钩）。套件合计 14/14（4.2s）。T-05 车道面备、语料行批量启用待续。current_step=4，status=executing，next=T-05。

## 10. 待澄清事项

T-01 区分共享事实缺口与可在 Rust 端实现的表示，按正式接口领取。真实规范冲突逐行裁定。

stage: review | review_scope: final | reviewer: 本会话（实施会话内复审——独立性限制已声明；结论从工件重建：run.stdout 逐行验证、git diff 面检、冻结组 results.json 重放、试点/V-套件重跑） | plan_id: PLAN-1115 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1115/auto-lang | branch: plan-1115-dev | reviewed_commit: 058fe7bf2 | base_commit: 0046408c5 | dependency_revisions: 892/902/904/1078 archived; 1114 reviewed(merge中) | spec_inputs: docs/specs/auto-a2r/project.md + collections-hir-backend.md + record-text-hir-backend.md + scalar-hir-backend.md + docs/specs/auto-test-case/feature-organized-corpus.md | acceptance_results: AC-01 pass（V-01 真链 5/5 ace-a2x PASS run 重放确认+试点 15/15）；AC-02 pass（V-02 10/10 + V-03 4/4=14/14 ace-a2x PASS run 重放确认）；AC-03 partial（172/185=93%：11 行语言面缺口 7 类具名+2 行 sched 归 1116；IF-ACE-A2R-LANGUAGE/1 通道全链已交付）；AC-04 本审执行 | findings: **F-1115-R1**（major，SD 增量）：SD-01 新文件 `docs/specs/auto-a2r/ordinary-language-and-stdlib.md` **未创建**；SD-02 `project.md` 零语言通道提及；SD-03 `feature-organized-corpus.md` 未登记 A2R 行启用。三个 Spec 文档均为 frontmatter 声明的必交付物，须在 worktree 备好。**F-1115-R2**（minor，AC-03）：11/185 行语言面缺口（7 类：嵌套 dyn place×2、变体守卫链×2、catch 语义×2、Shared 捕获×1、闭包内 fail×1、警告前置真错×3）——已逐项调查具名，判定归 plan owner 裁定是否为本批 required 或具名债。**F-1115-R3**（process/low）：4 个 semantic_view.rs unreachable-pattern warnings 在本分支修改区域（kind_name 函数新增 arm 后旧 catch-all 变不可达）。 | evidence: V-01..V-03 14/14 重放确认（run-q_xnv7vx/0/run.stdout lang_a2r_1115 逐行核对）；试点 15/15 本轮重跑确认；ace-a2x PASS（run-q_xnv7vx 317 tests）；case-a2r-language-parity 172/185（run-9pkvmm0w）；冻结并集 ace-stdlib PASS + ace-source 预存红（text_record_source master 漂移零本分支 core 提交）；case-av-language-parity 527/2 预存红（1114 已载）| pending_closeout_gates: T-07 冻结并集（ace-a2x ace-stdlib case-a2r-language-parity + 补真实影响组）| next: work 有界修复（R1 Spec 增量+R3 warnings）→ re-review

2026-11-22，re-review（修复复验，同 reviewer/基线链，reviewed_commit 56801ca1f）。修复核验：F-1115-R1 ✓（SD-01 `ordinary-language-and-stdlib.md` 新建+SD-02 `project.md` 语言通道入口节+SD-03 `feature-organized-corpus.md` delta-2026-11-22，三文件落 worktree 54e38ac0f，内容核过：通道 identity/已验域/边界/观察协议/遗留债完整）；F-1115-R3 ✓（semantic_view.rs 重复 match 臂清除，0 unreachable warnings）。代码行为零漂移（纯文档+重复臂删除）——试点 15/15+V-套件 14/14 复跑确认。F-1115-R2 登记为具名债（11 行语言面+2 行 sched 归 1116，关闭条件=后续计划逐类闭合）。outcome=**pass（final）**，status=reviewed，reviewed_commit=56801ca1f，next_responsible_role=merge（T-07；冻结并集待其执行）。

stage: goal | plan_id: PLAN-1115 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1115/auto-lang | commit: 56801ca1f | next: merge(T-07)

2026-11-22，stage=merge，PLAN-1115 r1。landed：ff-only dce2af2c3（master tip=delivery commit，rebase 后 range-diff 10/10 全等）；canonical_verified：docs/specs/auto-a2r/ordinary-language-and-stdlib.md（新，69 行）+ project.md（+12 行语言通道节）+ feature-organized-corpus.md（+15 行 A2R 视图 delta）三路径在主检出确认；archived（docs/plans/archive/）。冻结门收据：ace-a2x PASS（重定基基线 dce2af2c3）；case-a2r-language-parity 172/185（13 行=复审 F-R2 具名债，零新增红）。outcome=pass，completion_kind=delivered。

stage: merge | plan_id: PLAN-1115 | plan_revision: r1 | outcome: pass | delivery_commit: dce2af2c3 | canonical_paths: docs/specs/auto-a2r/ordinary-language-and-stdlib.md,docs/specs/auto-a2r/project.md,docs/specs/auto-test-case/feature-organized-corpus.md | archive_path: docs/plans/archive/1115-ace-a2r-language-and-stdlib.md | cleaned: clean（worktree+branch 删除、plan-artifacts 整目录删除、wt-guard clean、主检出零本计划残留）| batch_regression: DUE（1115%5==0>1114）→ regress 技能执行