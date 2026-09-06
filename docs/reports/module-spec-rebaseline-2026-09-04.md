# Auto-Lang Module Spec Rebaseline Report

> PLAN-546 execution evidence. Baseline gate passed on resume (2026-09-07).

## Baseline snapshot

- First freeze: 2026-09-04 (Asia/Shanghai) at `170e81ced docs:archive-plan-543` — gate blocked, see below.
- Resume freeze: 2026-09-07 (Asia/Shanghai) at `1c6753a92 merge(plan534)`.
- Execution branch: `plan-546-dev` (worktree `D:/autostack/.wt/lang-546/auto-lang`).
- Default branch: `master`.
- History note: `170e81ced` is no longer an ancestor of `master` (master's line was
  rewritten after the first freeze). The branch was rebased onto current `master`
  on resume; the only branch-unique commit (the original gate record) was
  preserved. All evidence below is gathered against the post-rebase tree.

Reproduction commands:

```powershell
git branch --show-current
git log -1 --oneline
git worktree list
Test-Path 'docs\plans\archive\532-*.md'   # True
Test-Path 'docs\plans\archive\536-*.md'   # True
git branch --merged master                 # 532/536 branches deleted after fold
git log --oneline master --grep=532        # merge(plan532) d4fe4ae48 reachable
git log --oneline master --grep=536        # plan536 review + T12 fix reachable
```

## PLAN-532 / PLAN-536 gate

| Plan | Archived file | Reachable from `master` | Worktree / branch residue | Gate result |
|---|---:|---:|---|---|
| PLAN-532 | `docs/plans/archive/532-aavm-tower-selfhost.md` | yes (`merge(plan532)` = `d4fe4ae48`) | none (cleaned) | pass |
| PLAN-536 | `docs/plans/archive/536-vm-reactive-runtime-fixes.md` | yes (review + `fix(vm)` T12 commits in history) | none (cleaned) | pass |

First-freeze state (2026-09-04, for the record): PLAN-532 worktree existed at
`066d12493` with `plan-532-dev` unmerged and no archive file; PLAN-536 commits
were reachable but the worktree existed and no archive file was present. Both
blocked the gate. On 2026-09-07 both plans are folded, archived, and cleaned —
the hard prerequisite of PLAN-546 §需求分析 is satisfied and execution resumes
from Step 2 on the post-532/post-536 `master`.

## Decision

Baseline accepted at `1c6753a92`. Module evidence gathering (Steps 3-20) proceeds
on this checkout. This plan changes knowledge documents and generated indexes
only — no product code.

## Evidence method

Each module section records the same nine fields, filled only from code/test
evidence gathered at `1c6753a92` (or the worktree HEAD above it):

| Field | Meaning |
|---|---|
| 边界 (boundary) | Files/dirs that constitute the module; what is explicitly outside |
| 生产入口 (production entries) | Public functions/structs reachable from real execution paths (not only tests) |
| 公共 API / 关键数据流 | Key public types and the dataflow between them |
| 当前能力 (current) | Capabilities with production entry and/or a runnable test as evidence |
| 实验能力 (experimental) | Gated/incomplete paths that exist in code but lack full evidence |
| 未实现/计划项 (planned) | Design/plan goals with no code — must never be stated as current |
| 测试证据 (tests) | Concrete test modules/files and how to run them |
| 与旧文档的差异 (drift) | Assertions in current specs found unsupported/contradicted by code |
| 本次修改文件 (files changed) | Spec/design files edited in this rebaseline |

Reproduction commands (run from repo root, in the plan worktree):

```powershell
rg -n "pub mod|pub use|#\[test\]" crates/auto-lang/src/lib.rs
cargo check -p auto-lang        # structural sanity only; this plan changes no code
rg -n "TreeWalker|Evaluator|32-bit|120 opcode|120.*opcode" docs/specs/auto-lang docs/design
```

Status vocabulary: `current` / `experimental` / `planned` / `historical` (PLAN-546 校准规则).

## frontend

| Field | Evidence |
|---|---|
| 边界 | `crates/auto-lang/src/{lexer,token,parser,parser_helpers,ast,ast/,dialect,dialect/,use_scanner,resolver}.rs`；`macro_/`（UI 宏）与 `mode.rs` 脚本方言判定相邻。执行/类型检查/转译在边界外 |
| 生产入口 | `lib.rs` `pub fn parse`（快照 2490 行）；`Parser`（`Parser::parse/parse_stmt/parse_expr/build_dialects/try_dialect_stmt`）；`use_scanner::scan_use_statements`；`resolver::{ModuleResolver, FilesystemResolver}`；`dialect::Dialect` trait + `UiDialect`；`mode.rs::resolve_script_mode`（plan-555 八格矩阵） |
| 公共 API / 关键数据流 | 源码 → `Lexer`（内部模块，f-string `$var`/`${expr}` 插值，`fstr_note '$'`）→ `TokenKind/Token/Pos` → `Parser`（持 `TypeStore`/`InferenceContext`/`ModuleTracker`/方言表）→ `Code/Stmt/Expr`（`ast.rs` + `ast/` 33 文件）→ `ToNode/ToAtom/AtomWriter` 序列化（166 impls，`rg -c` ast.rs+ast/*.rs） |
| 当前能力 | 递归下降全语法解析；方言派发（`Dialect` trait，UiDialect 接管 widget/msg/model/view/on，mem::take 模式 611-634 行）；`.as/.to` Cast/To（2714/3027/3037 行）；`super/pac` ModulePath 解析；use 两层扫描（字符串级 + ModuleResolver）；W2 语法糖批（plan-560：with/Power `**`/`@`/`is` 中缀）；with-as 绑定（plan-567，`with_header` pratt 截断）；ScriptMode 八格矩阵（plan-555） |
| 实验能力 | s2s lower 管线经 `auto_s2s` 消费 parser 产物（`.as` 执行翻转 lower→compile，plan-560——成熟度归 trans 节裁定） |
| 未实现/计划项 | 符号属性 `.?`/`.*`/`.@`、位操作、Auto Flow `Iter<T>`（ADR-08 设计层 active、实现 planned）——overview Status 行已如实标注 |
| 测试证据 | 内联 `mod tests`：lexer.rs/parser.rs/token.rs/resolver.rs（`rg -l "mod tests"`）；`src/tests/{test_generic_parse,widget_macro_tests,conformance_tests}.rs` 等；plan550/555/560/567 验收测试在对应归档 plan 有账 |
| 与旧文档的差异 | ①overview/architecture "求值器…四类后端消费/后端：evaluator"——Evaluator 仅重定向 AutoVM（execution_engine.rs:3 头注 Plan 091），已改述；②行号断言漂移（parser.rs 1979/2257→2714+/3027+、lib.rs 2114→2490、mem::take 434→611），已附 rg 复现+快照日期；③"parser.rs 超 13k 行"→实测 20562 行；④plans.md "活跃 plan" 325/332/367/448 实均已归档（status 以归档文件为准），`old/` 归档引用已统一为 `archive/`；⑤ADR-06 引 parser.rs:188 注释已失效（现为 Plan 306 GDScript 注解），改引 dialect.rs |
| 本次修改文件 | `docs/specs/auto-lang/frontend/{overview,architecture,plans}.md`（design/ 四文件无证据冲突，未动） |

## types

| Field | Evidence |
|---|---|
| 边界 | `crates/auto-lang/src/{types.rs, type_registry.rs, typeck.rs, typeck/, infer/（10 文件）, ownership/（borrow/cfa/lifetime）, trait_checker.rs, symbols.rs}`；类型表示本体在 `ast/types.rs`（frontend 目录，跨引用） |
| 生产入口 | `types.rs:TypeStore`（单一数据源）；`infer/context.rs:InferenceContext`（`type_store: Arc<RwLock<TypeStore>>` :79）；`infer/{expr:infer_expr, stmt:check_stmt, functions:check_fn, unification:unify}`；`trait_checker.rs:TraitChecker`（spec 符合性，parser 快照 10279/10327 调用）；`ownership/{BorrowChecker, LifetimeContext, LastUseAnalyzer}`；`symbols.rs:{SymbolLocation, CodePak}` |
| 公共 API / 关键数据流 | parser → `infer_expr`（快照首中 8329 行）→ TypeStore 注册/查询；`impl X as Spec` → `TraitChecker::check_conformance`；`type_registry.rs:TypeRegistry`（100 行，REPL 持久化侧，`SharedTypeRegistry` 为 lib.rs 再导出） |
| 当前能力 | 39 变体 Type 枚举（awk 实测）；HM/Robinson 统一（occurs check）；spec/vtable 符合性检查；泛型（模板+单态分发）；所有权三阶段（mod.rs 头注：move ✅/owned str ✅/borrow 🔄）；Plan 514 W1 fn 体作用域索引栈（`fn_scope_idxs`）；错误码 E0101-E0106+E0201-E0204 |
| 实验能力 | `ParamChecker`（typeck/param_check.rs）已实现但零外部调用点（typeck/ 之外无引用——与 overview 原文一致，复核维持）；借检查 Phase 3 in-progress（头注 🔄） |
| 未实现/计划项 | storage-injection（plan-055 ⏳ 未落地，plans.md 已如实标注）；`infer/registry.rs` DEPRECATED 孤儿模块待删除（新发现的清理项，非能力缺口） |
| 测试证据 | 内联 `mod tests` 14 文件（infer/ 9 + ownership/ 3 + trait_checker/types）；`src/tests/{infer_tests, ownership_tests, may_tests, generic_spec_tests, const_generic_tests, const_generic_integration_tests}.rs` |
| 与旧文档的差异 | ①"约 35 变体"→实测 39；②parser 行号 6598-6654/8246 漂移→8329/10279（附 rg 复现+快照日期）；③registry 消费方声称（type_registry/parser/vm::codegen/autovm_persistent 引用）已不成立——实测 `rg -rn "infer::registry" crates/` 零生产命中，模块孤儿化；④infer/context.rs:73→:79；⑤plans.md `old/` 归档引用→`archive/` |
| 本次修改文件 | `docs/specs/auto-lang/types/{overview, architecture, plans}.md` + `design/{type-representation, type-inference, typestore}.md`（error-types/ownership/type-inference 其余部分无冲突） |

## comptime

| Field | Evidence |
|---|---|
| 边界 | `crates/auto-lang/src/comptime/`（mod.rs 30 行 + transformer.rs 383 行）+ `ast/comptime.rs`（401 行 AST 节点）+ `compile.rs`（3151 行 CompileSession 编排，跨模块共享） |
| 生产入口 | `comptime/transformer.rs:CTEE`（内嵌 `VmInterpreter` + builtins OS/ARCH/DEBUG/VERSION）；`CTEE::transform`（:81，Stage-1 逐语句重写 `Code.stmts`）；七处管线集成点实测：lib.rs ×5（1188/1616/4198/4420/5027）+ trans/c.rs:4525 + trans/rust.rs:21433 |
| 公共 API / 关键数据流 | parse 后 → `CTEE::transform`（#if 裁剪/#for 展开/#is 匹配/#{} 求值）→ type-check/codegen；AST 节点 `HashIf/HashFor/HashIs/HashBrace`（ast/comptime.rs）；`Expr::Comptime`（parser.rs:2337 快照）/`Stmt::HashBrace`（parser.rs:7908 快照） |
| 当前能力 | 语句级 `#if/#for/#is/#{}` 全链路（token→AST→parser→变换→七处集成）；`compile_error()` 拦截；ComptimeError E0401-E0404（error.rs:1164 快照） |
| 实验能力 | 无——边界清晰 |
| 未实现/计划项 | 表达式级 `#{expr}` 编译期替换（vm/codegen.rs:10254 快照 TODO 自述运行时求值）；`comptime_mode` 标志（仅 mod.rs 文档注释提及，无字段）；确定性沙箱与 `CTEELimits` 资源限额（零命中确认未实现）；`#for` 仅整数上界（transformer.rs:299 "needs proper array iteration"） |
| 测试证据 | transformer.rs 内联 mod tests（5 处 #[test]/tests 标记）；ast/comptime.rs 内联测试；`test/comptime/{01_basic,02_intermediate,03_advanced}` 三级语料（未接自动化运行器——plan-137 原状维持） |
| 与旧文档的差异 | 仅行号漂移 4 处：parser.rs 1807→2337、6285→7908、error.rs 1148→1164、vm/codegen.rs 8115→10254（均附 2026-09-07 快照）；plans.md `old/`→`archive/`。限制性断言（表达式级不替换/comptime_mode 不存在/#for 限整数/builtins 裸标识符才命中/value_to_expr 五类型/I64 截断）全部复核成立，维持原文 |
| 本次修改文件 | `docs/specs/auto-lang/comptime/{overview, plans}.md` + `design/{hash-syntax, ctee-pipeline, comptime-eval}.md`（determinism-sandbox 无冲突） |

## interpreter

| Field | Evidence |
|---|---|
| 边界 | `interpreter/`（mod.rs 379 行 + vm_interpreter.rs 247 行）+ `execution_engine.rs`（102 行）+ `vm.rs`（844 行，VM 外观/任务定义）。旧 `eval.rs`/`interp.rs` 已物理删除（plan-091，commit `6862bb45f` 实测存在） |
| 生产入口 | `AutoInterpreter::{eval, eval_template, merge_atom}`；`VmInterpreter::{run, call, set_global, get_global}`；`ExecutionEngine::{default_engine, from_env, get}`；`execute_with_engine(_capture)`；lib.rs 高层 `run`(:336)/`run_with_capture`(:346)/`run_autovm`(:446)/`get_global_runtime`(:23)（快照行号） |
| 公共 API / 关键数据流 | 字符串 → Parser → Codegen（ABC 字节码）→ 重定位 → VirtualFlash → AutoVM 任务执行 → 栈顶提取 `Value`（nanbox 标记 + 对象 ID 区间解码）；引擎选择层全部落到 `run_autovm(_capture)` |
| 当前能力 | AutoVM 薄封装编程式求值（auto-gen 模板/auto-man 资产/UI 桥/编译期求值消费）；stdout 捕获（plan-177）；`AUTO_EXECUTION_ENGINE` env 覆盖（evaluator 等值仅告警重定向） |
| 实验能力 | 结果提取部分覆盖（int/f32/f64/string/object/array；test_simple_eval/test_string_eval 仍 `#[ignore="Result extraction not yet implemented"]`） |
| 未实现/计划项 | `VmInterpreter::call()`（:211 TODO 恒返 Nil；test_function_call `#[ignore]`）；globals 侧表不注入 VM 执行环境；持久 session/增量编译归 CompileSession（compile.rs）不在此 |
| 测试证据 | execution_engine.rs 内联 2 测试；interpreter/mod.rs 测试（3 ignore + test_eval_succeeds/test_merge_atom_obj/test_global_scalar_visible_in_eval/test_for_over_injected_global_array/test_mold_template_for_over_node_array 等活跃） |
| 与旧文档的差异 | ①"use-evaluator feature 路径在代码中已不存在"表述不精确——Cargo.toml:37 仍有空声明 `use-evaluator = []`（零 cfg 引用），已改述（overview + design/engine-selection.md 两处）；②call() 行号 170→211；③"3 个测试因此 #[ignore]"细化为 2 结果提取 + 1 函数调用；④plans.md `old/`→`archive/`。TreeWalker/Evaluator 历史化叙事（ADR-01/02、design/vm-backed-interpreter.md "不做 TreeWalker"）复核成立，维持 |
| 本次修改文件 | `docs/specs/auto-lang/interpreter/{overview, plans}.md` + `design/engine-selection.md`（architecture/template-evaluation/vm-backed-interpreter 无冲突） |

## vm

| Field | Evidence |
|---|---|
| 边界 | `vm/` 62 文件（engine/codegen/opcode/task/virt_memory/heap*/generic*/scheduler/task_system/native*/ffi/abt/debugger/disasm + tests_* 15 文件）+ `vm.rs`（844 行外观）+ `autovm_{persistent,repl,daemon,client}.rs` + `bigvm_repl.rs`；合计 ~62k 行（wc 实测） |
| 生产入口 | `lib.rs:run_autovm/run_with_capture`；`vm/codegen.rs:Codegen`（AST→ABC）；`vm/opcode.rs:OpCode`（**194 个**，awk 快照复现）；`vm/engine.rs:AutoVM::run_task_loop/run_one_instruction`（10139 行快照）；`vm/task.rs:AutoTask`；`virt_memory.rs:VirtualFlash/VirtualRAM`；`heap_object.rs:HeapObject`；`monomorphize.rs + generic_registry.rs`；`scheduler.rs/task_system.rs`；`native_registry.rs`；`ffi/c_ffi.rs`；`debugger.rs`；`abt/` |
| 公共 API / 关键数据流 | AST → Codegen（ABC 字节码）→ 重定位 → VirtualFlash → AutoTask 派发循环；栈槽=`NanoValue`（**u64 NaN-boxing**，auto-val/nano_value.rs:8 `pub type NanoValue = u64`；plan-221 引入/298 单路径）；堆对象统一 heap_objects 注册表（plan-077） |
| 当前能力 | CALL_SPEC nanbox 对齐（plan-474）；字符串池记账自持（plan-510，PoolHealth+soak 双档）；null 家族守卫（plan-550，13 例单测）；互操作分发层 obj_get/set/call/len/iter/type_name（plan-555）；W2 糖批 CALL_NAT_COUNTED（plan-560）；Err 值通道拦截+py 桥 480（plan-567）；py 返回值方法分派动态化（plan-569）；退出审计三挂点（plan-575）；文件测试框架 tests/vm_file_tests.rs + test/vm/（plan-177） |
| 实验能力 | AAVM 自举线（auto/lib/*.at）——独立档位 `cargo taa`，触发条件/作用域见 AGENTS §AAVM/AA2R Test Tier（plan-568 分层） |
| 未实现/计划项 | AutoLive 热重载、MicroVM C 实现、Tier-2 JIT、多语言 FFI 插件（design/05 Open Questions——overview:47 已如实标注）；双重解释器路径 12 测试 Windows cfg_attr 跳过（plan-574 裁定，非能力缺口） |
| 测试证据 | `vm/tests_*.rs` 15 个内联测试模块（bigvm/channel/chart_geometry/clipboard/closures/closures_borrow_check/collections/concurrency/known_limits/loader/parser_stack/rc_lifecycle/string_pool/tag/types）；`tests/vm_file_tests.rs`（test/vm/ 语料 golden）；`cargo tv` 档 3578 测（AGENTS 资源表 2026-09-06 实测） |
| 与旧文档的差异 | ①design/05 三处旧断言修正：opcode.rs "311 行/~120 opcodes"→880 行/194 个、engine 3515→10139、codegen 7079→14456（附复现）；②design/05 "32-bit stack-based execution/Each stack slot is 32 bits wide"→NaN-boxed 64-bit NanoValue（原 32 位设计降为 historical note）；③design/05 指令表 RET_D (2-slot return)→已删除（plan-377 §3.3 单槽化）；④vm overview "178 个 opcode/6882/11437 行"→194/10139/14456；⑤design/bytecode-engine.md 同步 194；⑥plans.md `old/`→`archive/`（含 212b/229a 行内引用） |
| 本次修改文件 | `docs/design/05-vm-runtime.md` + `docs/specs/auto-lang/vm/{overview, plans}.md` + `design/bytecode-engine.md` |

## trans

| Field | Evidence |
|---|---|
| 边界 | `trans.rs`（244 行 trait 层）+ `trans/` 17 文件：{rust, c, python, javascript, gdscript, typescript+ts_×4, tscn, r2a, auto_s2s, s2s_rules, py_known, emit, escape/}；CLI 枚举 `crates/auto/src/main.rs:TransTarget` |
| 生产入口 | `Trans` trait + `Sink`/`MultiSink`（trans.rs:158/32/200）；各后端 `*Trans` + `transpile_*`；lib.rs 库级 `trans_c(:4839)/trans_rust(:4847)/trans_c_legacy/trans_rust_legacy/trans_*_with_session/trans_rust_merged(:5236)/trans_python(:5304)` 等（快照行号）；CLI `auto trans -i x.at {rust,c,ts,python,js,gd,tscn,godot}` |
| 公共 API / 关键数据流 | AST → `Trans::trans(ast, sink)` → 目标源码 + source map；a2r 特有：逃逸分析（escape/analyzer.rs）→ 借用/clone/Rc 分层 + post_process 正则清理；s2s：`.as` 脚本糖 token/AST 双帧形改写（plan-555/560/567） |
| 当前能力 | 八后端家族（a2c/a2r/a2p/a2j/a2ts/a2gd/tscn/r2a）+ s2s 改写器；W1/W2/收官波三批已落地（auto_s2s 骨架→A/B/C/E1 规则→W3 注解预言机）；A1 `.len()` 双通道（plan-569）；a2r 自举线扩容（plan-532） |
| 实验能力 | s2s AST 发射器链式语义（P555-D2 登记债）；a2j 维护态（a2ts 迁移后） |
| 未实现/计划项 | a2r async/await 转译（plan-355 已归档未实施）、`#[api]` Axum server（plan-328 已归档待实施）、COSMIC 复制缺口（plan-364 已归档）——plans.md 已标"已归档"，不称 current |
| 测试证据 | 约定式发现（plan-263）：`src/tests/{a2c,a2r,a2ts}_tests.rs` + FFI `Test.run_*_dir` 扫 `test/a2*/`；`.at` 用例快照：a2c 123/a2p 97/a2r 262/a2ts 85/a2j 10/a2gd 69/cookbook 163（find 复现）；`cargo tt` 档 3786 测（AGENTS 资源表） |
| 与旧文档的差异 | ①行数表全列漂移：rust.rs 13842→23792（plan-532 扩容）、c 4533→4534、python 2702→3031、gdscript 2072→2091、ts 2274→2320、tscn 675→688（附 wc 复现+快照）；②用例计数漂移：a2c 144→123、a2p 23→97、a2r 23→262、a2ts 16→85（附 find 复现）；③plans.md 328/355/364/400/442 标 plans/ 活跃实已归档→改 archive/；④plans.md/design 36 处 `old/`→`archive/` |
| 本次修改文件 | `docs/specs/auto-lang/trans/{overview, plans}.md` + `design/sink-and-source-map.md`（其余 design 无冲突） |

## runtime

| Field | Evidence |
|---|---|
| 边界 | `runtime.rs`（ExecutionEngine/StackFrame）+ `scope.rs`（Sid/SymbolTable/DEPRECATED Scope）+ `scope_manager.rs` + `session.rs`（CompilerSession/Scenario）+ `host.rs`（ShellHost）+ `ffi.rs`（CFfiBridge）+ `py_ffi{,_types}.rs`（PyFfiBridge/PySignature）+ `libs/` + `a2r_std.rs` + `database/` + `sse/` + `route/` + `stdlib/auto/`（Auto 源码标准库） |
| 生产入口 | `runtime.rs:ExecutionEngine`(:126)/`StackFrame`(:51)；`scope.rs:Sid/SymbolTable`；`session.rs:CompilerSession`(:84)/`Scenario`(:28)；`libs/builtin.rs:builtins()`；`ffi.rs:CFfiBridge`；`py_ffi.rs:PyFfiBridge`；`database/mod.rs:Database/FileId/FragId`；`sse/parser.rs:SSEParser`；`route/`：RouteDiscovery/Merger/Def；`a2r_std.rs:List`（快照行号） |
| 公共 API / 关键数据流 | Universe 拆分（plan-064）：Database（编译期持久）+ ExecutionEngine（运行期 ephemeral）；`StackFrame.scope_sid → SymbolTable.sid` 单向链接；内建函数经 builtins() 装配；网络栈在 stdlib/auto/（http.at+http.vm.at+http_stream.at/net.at+net.vm.at/async.at+async.vm.at/json/url/log/env/sse 双文件模式实测在码） |
| 当前能力 | C FFI（CFfiBridge，CALL_NAT 桥）+ Python FFI（PyO3 嵌入，py_call/py_getattr 450/451，plan-560/567 扩至 480）+ Rust FFI（vm/NativeInterface）；AIE 增量存储+UI 产物缓存（UIArtifact/UICache）；SSE 解析；约定+配置混合路由；ShellHost（system/exit/export） |
| 实验能力 | Plan 064 分层迁移未收尾：`Scope` DEPRECATED 仍在码，`get_val` 恒返 None 桩（scope.rs:300 TODO 自述——复核成立） |
| 未实现/计划项 | WebSocket（plan-350 归档设计）、中间件/session/SSR/OpenAPI（plan-352 归档设计）——plans.md 现均标 archive/ 不称 current |
| 测试证据 | `libs/` 内联测试；`src/tests/{ffi_tests, ffi_dual_tests, mode_tests, stdlib_tests, storage_tests, storage_integration_tests, default_storage_tests}.rs`；HTTP 真栈测试 `cargo th` 档 20 测 |
| 与旧文档的差异 | ①plans.md 严重漂移：标 `plans/` 活跃的 300/317/318/322/328/329/334/335/341/344/349/350/352/353/355/442/458 共 18 行实已全部归档→归档列整体校正+头注复核记录；②`old/` 引用→`archive/`（plans.md+overview.md 重号注记）；③其余现状断言（Scope 桩/libs/std.rs 0 行/Plan 134 注记/stdlib 双文件）全部复核成立，维持 |
| 本次修改文件 | `docs/specs/auto-lang/runtime/{overview, plans}.md`（architecture/design 四文件无冲突未动） |

## ui

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |

## mcp

| Field | Evidence |
|---|---|
| 边界 | 待取证 |
| 生产入口 | 待取证 |
| 公共 API / 关键数据流 | 待取证 |
| 当前能力 | 待取证 |
| 实验能力 | 待取证 |
| 未实现/计划项 | 待取证 |
| 测试证据 | 待取证 |
| 与旧文档的差异 | 待取证 |
| 本次修改文件 | 待取证 |
