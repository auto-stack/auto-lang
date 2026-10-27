# PLAN-843 T-01 合同基线（冻结）

日期：2026-10-28。作者：work（/auto-plan:work）。
基面：auto-lang master `f99facb27b817fade1e5c2f03891c1ff418e28d2`；
权威树：`D:/autostack/.wt/lang-843/auto-lang` @ branch `codex/plan-843-dev`（= master 同提交）；
组内兄弟：`auto-down` detached `c3464d0`（解析依赖，只读）、`auto-ui` detached `bd228b0`
（=842 UI 交付提交；经 `crates/auto-lang/Cargo.toml` 的 `../../../auto-ui/...` path 依赖参与
workspace 解析，只读固定版本）。预检探针：worktree 内 `cargo check -p auto-lang-core`
通过（37.13s dev profile；145 条预存 warnings 为基线，见 §7 健康口径）。

## 1. 启动证据（G843）

- U831 r3 归档 12/12（`docs/plans/archive/831-*.md`）；U832 r3 归档 12/12，lang 交付
  `3934ed4f` 为 master 祖先；U841 r1 归档 12/12，lang 交付 `f16bb468b` 为祖先。
- U842 r3 归档 17/17（2026-10-28 merge 闭环记录：lang 交付 `031c8fd2d` 落 master、
  ledger v105→v106、四树 fresh wt-guard clean、组目录回收），§9 明示"U842 全闭环，放行后继计划"。
- `git worktree list` 无 831/832/841/842 残留树；主检出未提交改动仅 docs/ 簿记与
  `.autoos/specs.json.bak-*` ledger 备份，无 `crates/**`/`test/**` 代码 WIP。

## 2. 七热点冻结（现行路径与规模）

全部位于 `crates/auto-lang-core/src/`（PLAN-814 T-02 big-move `edb78b2ef` 整体迁入；
841/842 未对其内部结构做拆分重组）。生产/注释/空行/测试行分类见
[measurements-baseline.txt](measurements-baseline.txt)（probe 脚本
`D:/autostack/.wt/lang-843/probe-t01/measure.py`，`#[cfg(test)] mod` 花括号配对统计）。

| # | 热点（现行路径） | raw 行 | 生产行 | 关键内部锚（行号） |
|---|---|---|---|---|
| H1 | `parser.rs` | 15204 | 10376 | ParserExt(22)、优先级表(28-155)、CompileDest(156)、FnAnnotations(165)、**Parser struct(197，含 17 pub 字段+方言三段表)**、CodeSection(2110)、api/export ABI 参数解析(5516-5555)、tests(13459)、嵌入 .at 测试源(14972+) |
| H2 | `vm/codegen.rs` | 16753 | 11372 | 辅助/TypeInfo/ParamInfo/ApiCallInfo/FStrPartType(40-176)、**Codegen struct(178)**、bare_native_intrinsics(539-728)、**impl Codegen(730-16106)**、tests(16107)、poisoned_fn_exports(16713) |
| H3 | `vm/engine.rs` | 14148 | 10472 | 栈标签/迭代器族(62-225)、Closure(226)、Timer(252)、**VMError(267)**、StepResult/FrameResult/SegmentOutcome(303-364)、CpuSliceBudget(365)、DriveBudget/ParkedWait/ParkedSegment(412-458)、**AutoVM struct(459-611)**、FutureValue 族(612-650)、nanbox 辅助(651-759)、**impl AutoVM(760-13670)**、test mods(13671+) |
| H4 | `vm/native.rs` | 13119 | 9540 | NativeContract(76)、TerminalOwnerEntry(102)、**NativeInterface(112)+Clone(215)+impl(313-9415 巨型 impl)**、Xorshift64(9416)、StakeGuard(12248)、tests(12892+) |
| H5 | `vm/ffi/stdlib.rs` | 14076 | 10110 | RequestFileRootGuard(28)、ReadTextRangeOut(411)、native_crash_ffi(1165)、exit_trace_ffi(1231)、AsyncResult(2466)、AsyncStreamEvent/Handle(5105)、Http 客户端族 HttpRequestBuilderData/ResponseData/JsonJob/Defaults/MsgDone/PayloadRef(5166-8877)、HttpBaseEnvGuard(12733)、测试族(image_pipeline 78/exit_audit 1554/plan781 2994/e4_default_http 12430/sqlite 12758/tests 13010/plan536 13764/bus 14001) |
| H6 | `vm/ffi/http_server.rs` | 8320 | 6668 | 路由 HttpRoute/RouteMatch/ApiParamSig(19-47)、MultipartPart(395)、请求/应答模型 ApiRequest/ApiBody/FileReplySeed/ApiReply(4062-4103)、DispatchCtx/PendingUpload/ParkStage/ParkedRequest(4676-4735)、MWStep/HandlerEnd/MarshalOutcome(5119-5529)、RequestScope(6060)/ScopeGuard(6321)、SseIteratorCleanup(6937)、GeneratorWait(7060)、ApiArgBindError/ApiTyKind(7251-7260) |
| H7 | `trans/rust.rs` | 31704 | 24894 | RustEdition(59)、**RustTrans struct(90)**、**impl RustTrans(497-17909)**、**嵌入模块 auto_cabi_kit(17910-27937)**、Trans for RustTrans(27938-28783)、ProjectModule(28784+) |
| 根 | `lib.rs` | 4690 | 3508 | mod 声明+script_entry 再导出(1-113)、GLOBAL_RT/ERROR_LIMIT/VM_DEBUG/SCRIPT_ARGS/trace 旗标族(114-273)、run* 执行入口族(283-4690，含 StagedRunFailure 317、deadline/input/path 变体族、join_execution、run_autovm 族) |

**已关闭热点核销记录：无。** M2 报告观察的七热点在 831/832/841/842 后全部仍存
（同一组文件经 `edb78b2ef` 整体搬入 auto-lang-core，`git log --follow` 各文件历史连续；
841 的 AST/Token/lowering 终态与 842 的旧入口退役未触及上述文件内部算法布局）。
前置已做的相关结构工作（不重复、不抵扣本 Plan 任务）：http FFI 旁路文件
（`http_server_file/stream/transfer/transport/upload/return_resource.rs`）在 big-move 前
已存在；`infer/`、`ownership/`、`stdlib_assembly/`、`sse/`、`vm/abt/`、`trans/escape/`
已是目录模块。本 Plan 七热点全部须交付主题模块拆分，无一延后或以"已整理"核销。

## 3. 内部 DAG 与状态 owner（冻结）

依赖方向（→ = 依赖；只列主数据流，禁止反向边）：

```
lexer/token → parser(+dialect 表,parser_bridge) → ast
ast+token → types(TypeStore) → infer(context/constraints/unification) → typeck/trait_checker
parser+types+infer → compile(CompileSession: 模块图/缓存) → vm/codegen(Codegen)
vm/codegen(ABC) → vm/engine(AutoVM: tasks/RAM/frames/iterators/futures)
vm/engine → vm/native(NativeInterface 注册) + vm/ffi(编码/宿主服务桥)
ast+types → trans/*(rust/c/python/ts/gdscript/tscn/r2a) → Sink 产物
lib.rs = 入口组合（run*/test*/trans_* 经 script_entry）+ 全局旗标/RT
session/database/scope_manager 横跨前端（唯一 owner 见下）
```

| 状态 | 唯一 owner | 不变量（本 Plan 不得破坏） |
|---|---|---|
| 解析游标/方言表/恢复深度 | `Parser` | 17 pub 字段合同、方言派发序 pre→builtin→post（R1 Arc 共享句柄递归可见）、ERROR_LIMIT 全局 |
| 类型/符号/模块表 | `TypeStore`（types.rs 133） | 单一 Store；LoadedModule/SymbolConflict 语义 |
| 推断约束/泛型 | `InferenceContext`（infer/context.rs） | infer/ 已分模块，仅整理边界不加第二套 |
| 编译会话/模块缓存 | `CompileSession`+`module_cache` | 增量/指纹失效规则不变 |
| 字节码发射 | `Codegen`（flash/符号表构建） | Opcode/ABC/ABI 不改；执行次序不变 |
| 执行状态 | `AutoVM`（tasks/RAM/frames/预算/挂起） | Rc/线程亲和（!Send 不跨 OS 线程）、锁外回调、CpuSliceBudget/DriveBudget、挂起唤醒 |
| native 注册 | `NativeInterface`（单一 registry） | 注册/替换优先级、TerminalOwner、StakeGuard RAII |
| FFI 编码/宿主服务 | vm/ffi 族 + host_capabilities provider 端口 | RequestFileRootGuard/HttpBaseEnvGuard 生命周期、TCP ready/端口策略、退出清理 |
| 转译 | 各 `*Trans`（无跨目标共享大杂包） | 同算法复用；Sink 产物字节可比 |

## 4. 具体拆分目标（主题模块与量化指标）

通则（每热点一致，review 逐项核）：
- 入口文件（`mod.rs`）仅保类型定义/构造/明确派发与组合；**目标 ≤3000 生产行**（H7 rust
  放宽 ≤3500，因 Trans trait 组合面大）。
- 主题模块按语义职责组织（下表），**禁止** include! 机械分片、空 re-export 模块、
  文件切两半式平移、新增 >4000 生产行的 common。
- 可复核改善指标：①入口生产行 vs 基线（§2 表）；②主题模块数 ≥3/热点；
  ③`pub(crate)`/`pub(super)` 计数不高于基线（measurements §公开面计数列），新内部接口
  只扩最小 `pub(super)`/`pub(crate)`，不一律 pub；④测试行随 owner 家族迁移（`#[cfg(test)]`
  mod 保持文件级，不合并进共享巨测试模块）。
- 同输入产物保持：ABC 字节码、trans 文本、诊断文本、求值序（V-03 oracle 见 §5）。

| 热点 | 拆分主题（目标模块） |
|---|---|
| H1 parser | `parser/mod.rs`(Parser struct/构造/方言装配/parse() 入口/错误恢复核心/CodeSection) + `expressions`(pratt/优先级表/atom/运算符) + `statements`(语句族 let/if/for/on/task/store…) + `type_syntax`(parse_type/generic) + `fn_decl`(fn/注解/CompileDest/api-export ABI 参数) |
| H2 codegen | `vm/codegen/mod.rs`(Codegen/TypeInfo 族/构造/发射入口) + `expr_emit` + `stmt_emit` + `fn_emit`(闭包/task/frame) + `ty_conv`(ObjectType/ConvSrcType/native 后缀/默认值) + `fstr`(FStrPartType) + `intrinsics`(bare_native_intrinsics) |
| H3 engine | `vm/engine/mod.rs`(AutoVM struct/构造/公开 drive-step API) + `dispatch`(段执行循环) + `iterators`(7 迭代器+Iterator enum) + `futures_timers`(FutureValue 族/Timer) + `frame_task`(FrameResult/挂起/ParkedSegment/预算) + `vmerr`(VMError/StackTag/错误码) |
| H4 native | `vm/native/mod.rs`(NativeInterface/NativeContract/注册 API) + 按族：`strings`/`collections`/`math`(含 Xorshift64)/`system_process`/`terminal_owner`/`task_coroutine` + `stake`(StakeGuard)（族边界按 impl 内实际聚簇在 T-04 细化，属常规实现选择） |
| H5 stdlib | `vm/ffi/stdlib.rs` 收敛为注册/派发入口 + `ffi/file_io`(ReadTextRangeOut/RequestFileRootGuard) + `ffi/crash_trace`(native_crash_ffi+exit_trace_ffi) + `ffi/async_jobs`(AsyncResult/AsyncStream 族) + `ffi/http_client`(Http 客户端族 5166-8877) + `ffi/env_guard`(HttpBaseEnvGuard) |
| H6 http_server | `vm/ffi/http_server.rs` 收敛为服务装配/ready 入口 + `server/route`(路由匹配/ApiParamSig) + `server/model`(ApiRequest/ApiBody/ApiReply/MultipartPart) + `server/dispatch`(DispatchCtx/Park 族/MWStep/HandlerEnd/MarshalOutcome) + `server/scope`(RequestScope/ScopeGuard/SseIteratorCleanup/GeneratorWait) + `server/argbind`(ApiArgBindError/ApiTyKind) |
| H7 trans/rust | `trans/rust/mod.rs`(RustTrans struct/构造/Trans trait 组合 27938 段) + `rust/expr` + `rust/stmt` + `rust/decl_types`(声明/类型/RustEdition) + `rust/cabi_kit`(auto_cabi_kit 整段迁出 17910-27937，天然缝) + `rust/project`(ProjectModule/工程输出) |
| 根 lib | `lib.rs` 保 mod 声明/再导出 + `entry.rs`(run* 执行入口族+StagedRunFailure+join_execution，仅组合) + `flags.rs`(GLOBAL_RT/ERROR_LIMIT/VM_DEBUG/SCRIPT_ARGS/trace/audit 旗标族) |

## 5. API/行为基线与可执行 oracle

- **API 基线**：各热点 pub fn/类型/字段计数冻结于 measurements §公开面计数；`auto_lang_core::*`
  公开路径经 `crates/auto-lang/src/lib.rs` 兼容门面（`pub use auto_lang_core::{...}`）与
  auto-lsp/auto-cosmic/examples 等真实消费者编译面保持。Parser 17 pub 字段、
  `TypeStore`/`InferenceContext`/`CompileSession`/`AutoVM`/`NativeInterface` 类型身份不变。
- **行为 oracle（V-03 冻结输入）**：同源 .at 程序的 ①解析/类型/诊断（结构化错误文本）
  ②字节码+VM 求值（stdout/求值序 trace）③各现行 trans 文本（rust/c/python/ts/gdscript/tscn）
  ——以既有耐久测试入口为准：语料三族（corpus_m1..m5/corpus_use，`cargo tv` 选择面）、
  `test/vm/**` golden、trans 族（tt 档）、`run_with_capture*`/`run_autovm` 集成测试。
  拆分前后同一冻结输入的产物必须逐字节可比（路径/身份注记类差异单独解释）。
- **结构 oracle（V-01）**：§4 量化指标表 + 前后 measurements 对拍；禁止以压缩格式/挪测试行降数。
- **消费 oracle（V-04）**：`test/fixtures/modularity/language-sdk-consumer`（Core
  parse/compile/VM 真执行+trans 产物+UI 文法显式拒绝负控）、`three-repository-consumer`、
  auto-ui（bd228b0）/auto-man（4cc912e=842 Man 交付）固定版本只读消费；fresh+locked。
- **负控（V-05）**：源指纹缺失红→恢复绿；测试 ID/alias 迁移前后选择面不减。

## 6. 支持/预算/错误/退出规则

继承现行合同，本 Plan 不新增/放宽任何限额：错误面（VMError 码、诊断文本）不变；
CpuSliceBudget/DriveBudget 语义不变；FFI guard 退出清理与 TCP ready/端口策略（OS 临时
分配先例）不变；`vm_thread_stack_size` 缺省/下限不变。线程/退出/回调所有权（AutoVM !Send、
spawn_local 局部执行器、锁外回调）不变。

## 7. 正反例

- **正例（必须成立）**：`auto_lang_core::run("1 + 2")` 行为不变；`trans_rust_merged` 同输入
  产物不变；`cargo tv` 语料 golden 全绿；language-sdk-consumer 5/5；`pub(crate)` 计数不升。
- **反例（必须拒绝）**：include!/宏拼接还原大文件；入口 re-export 一切但实现原样；两份
  TypeStore/registry；为降行数删测试或注释；把 UI/Man 依赖引入 SDK 闭包；改 Opcode/ABI/
  缓存语义并归因"结构整理"；未测环境声称通过。
- **健康口径**：145 条预存 warnings 为基线（842 交付时在案）；本 Plan 不得新增 warning，
  review 健康检查按"零新增"核。

## 8. 未知决策/owner/关闭条件

| 项 | 状态 | 关闭条件 |
|---|---|---|
| native.rs 巨型 impl 的族聚簇精确边界 | 未冻结（常规实现选择） | T-04 实施时按实际聚簇定名，登记入 T-09 handoff |
| parser 内联测试（1399 行）随家族 or 集中 parser/tests.rs | 未冻结 | T-02 实施时定，保持文件级 cfg(test) mod，选择面不减 |
| trans/rust.rs 内 .at 嵌入源与 cabi_kit 的模块归属细节 | 未冻结 | T-06 实施时定，产物逐字节可比 |
| auto-man 消费复验入口（源码树 vs 固定版本 clone） | 待 T-08 | 按固定版本只读 clone 实测，不借他计划 WIP |
| 844 去重候选清单 | 由本 Plan 记录不执行 | T-09 汇总具名候选（如重复的 HttpDefaults/迭代器构造）交 844 |

关闭条件（T-01 本身）：本文件 + measurements 冻结入库、G843 证据入 §9 —— 已满足；
T-02 可启动。
