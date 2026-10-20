# PLAN-828 T-01 合同冻结（有界全路径首例）

日期：2026-10-20 ｜ 状态：T-01 交付冻结 ｜ 基线：`plan-828-dev` @ master `f79b1b835`
工作树：`D:/autostack/.wt/lang-828/auto-lang` ｜ 验证入口：`cargo test --manifest-path
experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1`

本文冻结 T-01 首例（`main→helper` 双函数、预热后单函数体编辑、全链路真实增量）所作的
全部合同决策。T-02..T-08 在这些接缝上生产化；改动本文件所述语义须先回 new 走有界修订。

## 1. 范围与授权

- 用户 2026-10-20 明确目标：签名与真实依赖事实不变时，只重编并替换被改函数，其他函数
  中间表示与已加载代码不动；完整切片 source→语法/语义→HIR→MIR→ABC→受检安装→真实结果。
- T-01 只交付有界首例与本文；非目标（多文件工程、字符串/堆对象、签名原位兼容安装、
  活帧迁移、guest 结果缓存等）见计划 §1，不因首例存在而视为已交付。

## 2. scalar 语法差异审计（T-01 逐项核对，实证）

对照物：`experimental/ac-syntax/grammars/auto-core.at`（787 冻结，合同=ac-core 旧
core-i32 parser 行为）vs `auto.source.scalar-i32.draft`（798 冻结负载
`probe/src/load256.rs` 实际使用的表面）。两语法共享 `fn name(p int) int { … }` 基本形
（ac-core fixtures `d01-call-multiline-then-add.at` 等为共同子集实证）。

| scalar 构造 | auto-core.at 现状 | 证据 | 处置 |
|---|---|---|---|
| `+ * <` 二元、负字面量 | 已覆盖 | auto-core.at pratt 表（`<`(1) `+`(2) `*`(3)）；fixtures | 首例直接使用 |
| 命名实参、位置调用、bool 字面量 | 已覆盖 | `named-arg` 规则、`true/false` token、投影 `ExprKind::BoolLit` | 首例直接使用 |
| 二元 `- / %`、`== != > <= >=` | **缺** | pratt 表无这些算符（`-` 仅字面量前缀） | **T-02 已补齐**（auto-scalar rev 1） |
| `&& \|\|` 短路、一元 `!` | **缺** | 无对应 token/规则 | **T-02 已补齐**（unary-expr 前缀规则） |
| 条件 `for cond { }` | **缺**（core 用 `while`，scalar 反之拒绝 `while`） | auto-core.at 保留字 `for` 无规则引用；scalar profile 文档 | **T-02 已补齐**（for-stmt→While 形状投影） |
| `while` | 覆盖（core） | scalar **拒绝** while | auto-scalar 不引用 `while` token（同保留字处理） |

**T-02 落地（2026-10-20）**：新增版本化 grammar `grammars/auto-scalar.at`（rev 1，
全 scalar 表面：`||`(1) `&&`(2) `== !=`(3) 关系(4) `+ -`(5) `* / %`(6) 全左结合；
一元 `- !` 右结合经 `unary-expr` primary 规则，`-字面量` 仍走 int-literal 带符号折叠，
`-g()` = Unary(Neg, Call)（callee 包裹层算子链按 parser apply_unaries 施用序重包装，
Unary span = 操作数 span、Call span 起点 = callee ident——与 parser 逐字节一致）。
投影扩展：`CoreKinds` 对 grammar 可缺项改 Option；`project_decl_strict`（单声明投影，
增量热路径原语）；`compile_program_parts`（ac-core 新接缝：已解析 Program 直入
resolve→…→bind，整模块 parse 入口 `compile_parts_with` 原样保留）。

**对拍证据**：`scalar_projection_.rs` 5 组差分（算符/一元与字面量/循环与语句/调用与布局/
拒绝形状）AST+span Debug 全等；冻结负载 256KiB×2 修订（115 函数含二元`-`/`>`/条件for）
`t02_frozen_load_projection_parity` 全等；adapter 全套 44 测试绿；会话 4 测试绿
（V-01/V-04/语法错误反控 t02）。会话热路径现为：增量重扫/嫁接 → 单声明投影 →
synth Program（签名桩）→ compile_program_parts（无整模块 parse/投影，无垫片串）。

**拼接收敛边界（实证）**：增量词法收敛需编辑区后 ≥8 个对齐 token（`CONVERGE_K`）；
被编辑函数若是文件末声明且尾部 token 不足，将得到记录在案的 `NoConvergence` 显式
回退（非快路径，不冒充）。首例冻结 helper 在 main 之前；冻结负载（115 函数、编辑点
fn_037 中部）结构上满足该边界。

## 3. 公共接口与复用决策

| 接缝 | 决策 |
|---|---|
| 函数级前端 | **不改 ac-core 前端入口**。会话以"合成源码"驱动既有真实入口 `compile_parts_with`：未改函数只贡献签名原文 + O(1) 空体 `{ }` 桩，被改函数的真实声明文本以空格垫片对齐到新源绝对偏移（其诊断/span 即新源坐标；桩 span 属合成坐标，永不消费）。桩行走计入 `synth_stub_walks`，不计入未改函数任何 stage |
| HIR 函数凭证 | ac-core 新增 `verify::verify_function(bundle, def_index) -> CheckedFunction` + `CheckedFunction`（仅验证器构造；字段私有）。复用 `check_module` 的上下文构造（类型界检查、text 种类隔离、type_map、全 def 签名表、目标 def↔body 所有权）+ `check_profile_ops` + `check_body` 只查目标 body。`context_revision` = FNV-1a(全类型+全 def 签名) —— 上下文变化使凭证按指纹失效，不从旧 CheckedModule 切片盖章 |
| 函数级 lower | probe 新增 `hir_adapter::lower_function_unit(bundle, def_id, caps, snapshot) -> MirModule(unit)`：复用私有 `lower_function`；跨函数 callee 预先写成 `fn:<name>` 单元 import（与 `split_units` 产物同形），自递归 = `Func(0)`；不调用 whole lower |
| 单元准入 | 复用 798 既有 `verify_mir` / `encode_abc` / `decode_abc` / `verify_abc`；新 `core::function_code::FunctionCode::admit_unit(LoadedModule)`：仅接受**不可执行单元凭证**（executable/is_unit/单函数三重校验），无法从该缝塞入伪造闭包 |
| 链接与安装 | 新 `core::program_snapshot::ProgramSnapshot::link(code_set, entry, revision)`：对已准入单元做签名核对链接（interned-type 等值 + `fn:<name>` 延迟 patch + 重名/入口检查），**无整闭包 codec 往返、无整模块重验证**——这是设计而非捷径：单元已各自 ABC 准入，链接环境只建立调用绑定（计划 §5.3） |
| 执行 | av.rs 抽出共享派发循环；新 `execute_program(&ProgramSnapshot, …)`：merged 视图**由构造保证可派发**（仅经 link 产生），顶层调用捕获 snapshot `Arc`，在途链固定旧版（V-05 种子已在首例断言） |
| 798 路径 | `lower_checked`/`split_units`/`assemble_closure`/整闭包往返/Driver 全部原样保留（本次回归：core 21、mir 8、abc 15、binding 4、vertical 3、cache 20 全绿） |

### T-03 增补（function_queries / 凭证失效）

- `probe/src/function_queries.rs`：`FnEnv`（owner 域=模块 id + 全函数接口事实 + 内容
  修订）与查询：`interface_of/def_id_of`（owner 作用域；未知名与他 owner 同名=缺席）、
  `interface_unchanged`（快路径接口门：被改函数接口与整个声明上下文修订都不变）、
  `credential_current`（body 凭证仅在铸造时的声明上下文修订下可复用）。
- **内容规范化指纹（关键修正）**：签名/上下文指纹一律按类型**形状**（i32/bool/递归
  fn 形）解析，不用类型表下标——真实体程序与签名桩程序的类型表布局不同，下标指纹会
  误伤失效（实测复现后修正：ac-core `context_fingerprint` 与 probe `sig` 渲染同步改）。
- 会话接线：warm-up 铸造上下文修订（对 entry 跑一次 verify_function）；每次编辑后
  `interface_unchanged` 门 + `credential_current` 防御性复检；`sf_<name>` 内部符号
  在查询层剥离为源名（方法符号不剥离、不在 scalar 会话面）。
- 证据：t03_signature_change_is_non_fast_path（签名编辑被接口门拒绝、旧版可运行）、
  t03_body_credentials_are_context_bound（不同声明上下文→不同修订→凭证不可跨上下文
  复用；owner 作用域查询）。
- **重排记录（r3）**：函数 HIR A07 codec/producer/恢复原列 T-03，但恢复的唯一消费方
  是 T-07 的 store/重启路径——无 store 的 codec 即"只定义 key 未接消费"（计划明令禁
  止）。A07 持久化整体随 T-07 落地（codec+producer+恢复+重验一次交付），T-03 交付
  上述查询/凭证/失效层。此为任务内重排，不减验收面。

## 4. 身份、引用与常量

- 跨修订函数身份 = **def id 文本**（`d_<name>`，bundle def 序）；不使用模块内数组序号、
  源绝对 offset 或代码地址。`FunctionCode`/HIR body 以 `Arc` 复用，identity 断言用
  `Arc::ptr_eq`（V-01 已断言 main 三重 identity：HIR body、FunctionCode、snapshot code 槽位）。
- 单元自包含：类型/常量/import 各单元独立 intern；跨函数引用 = `fn:<name>` import +
  被调方签名（链接期等值核对）。槽/PC/块号为单元内位置量，不构成跨修订身份。
- 入口约定沿用 798：def id（`d_main`），非源名（`DefKind::Function::name` 是发射器内部
  符号 `sf_<name>`，不得当源名匹配——首例实证）。

## 5. 安装可见性与寿命（首例语义）

- prepare/commit 最小形态：候选 FunctionCode 构造（prepare，不动当前根）→ `link`
  成功才整体换 `Arc<ProgramSnapshot>` 根（commit）。前端诊断、HIR 验证失败、MIR/ABC
  拒绝、link 失败一律**保留旧根且旧根可继续运行**（V-04 断言 23 不变 + 恢复编辑成功）。
- 拒绝编辑后会话停在**最后接受修订**：resident 语法状态不前进（后续编辑按该修订坐标），
  源文件发散的重建路径归 T-06/T-07，不在首例降级为整模块重算后冒充快路径。
- snapshot 不可变；顶层 `execute_program(&pinned)` 与 `session.run()`（取当前）并存，
  旧 snapshot 由持有方 `Arc` 保活（T-06/T-07 补 lease/退役/GC）。

## 6. stage 计数合同（V-09 反控将绑定同名入口）

- 每 stage × 函数计数，键 `stage:<fn-name>`（源名）；`STAGES` 冻结 11 轴：
  `project/resolve/typecheck/hir_build/mir_lower/abc_emit/decode/verify_hir/verify_mir/
  verify_abc/load`。计划 §5.5 的单 `verify` 轴拆为三层**等价名**，零语义不变。
- 结构零门：未改函数 11 轴全 0（`stages_of`）；`whole_module_{frontend,lower,
  closure_roundtrip}` 计数器结构性为 0（会话编辑路径不存在调用点）。语法证据计数：
  `syntax_rescan_bytes/reparsed_decls/reused_decls` 来自 ac-syntax 真实路径。
- 每次计数发生在对应真实入口调用点；合成桩行走单独计 `synth_stub_walks`，不计为任何
  函数的 stage 工作。V-01 已断言：helper 11 轴各 1、main 全 0、install 集恰为 helper。

## 7. 资源归因（首例口径）

- 首例只证结构（计数/identity），性能与内存归 T-08 双门（P95<1000ms + 两线内存）。
- 会话常驻表示（查询图、HIR/MIR/单元、候选/旧 snapshot、合成串）全部计入未来峰值，
  不得移出基线；warm-up 整模块编译是冷路径，不计入任何编辑热路径指标（单独列报）。
- 继承 798 r5 双线预算与 32MiB 受管分区，不新增会话预算副本。

## 8. 正式会话入口（T-08 冻结位）

- 首例入口：`session::{warmup, FunctionIncrementalSession::{edit, run, current,
  counters, resident}}`（进程内、单线程、Rc 语法态）。T-08 在其上挂 AutoDown 正式
  recipe/worker/父侧 judge 与 ready 分界（首启动/初次装载单列，编辑入队→结果发布计
  时含当次必要 IPC/编译/安装/执行）；T-07 补 store 持久化/恢复/lease。

## 9. 首例界限（诚实清单，不得冒充已交付）

1. 单编辑单函数体；多声明编辑、声明增删/改名、签名变化、词法回退=显式拒绝并记录原因。
2. 未改函数的 O(1) 桩行走与整表签名重注册是首例实现细节（115 函数规模可忽略，但仍是
   O(N) 记账）；T-03 的 `function_queries`/接口环境将其消去。
3. 垫片串 ∝ 被改声明偏移、commit 时 O(N) merged 视图重建：首例可接受，T-06 收敛为
   增量绑定表。
4. 无 store 持久化/重启恢复/GC/引用计数退役（T-07）；无 worker/IPC/正式计时（T-08）。
5. span 仅对被改函数保证新源坐标；未改函数诊断 span 不适用（其桩体无诊断面），相对
   span/文件映射层归 T-02/T-07。
6. 语法面仅共同子集（§2 表）；scalar 全表面 grammar 与投影对照归 T-02。

## 10. 验证证据（本修订实测）

- `function_incremental_828`：V-01 首例（23→28、main 三重 identity、11 轴计数、
  install={helper}、新旧 snapshot 双运行）+ V-04（类型错误候选拒绝、旧版可运行、
  恢复编辑成功）全绿，`--test-threads=1`。
- 798 回归面：auto-av-core 21、probe mir/abc/binding/vertical/cache = 8+15+4+3+20 全绿
  （av.rs 派发循环重构无回归）。
- 失败即证据：helper 位于文件尾时编辑 `NoConvergence` 显式回退（§2 边界，已实证并
  记录，非静默降级）。
