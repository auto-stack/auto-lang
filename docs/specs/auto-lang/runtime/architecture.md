# runtime 架构

## 结构图

```mermaid
graph TD
  subgraph 编译期持久态
    DB[(Database<br/>database/mod.rs)]
    ST[SymbolTable<br/>scope.rs]
    SID[Sid 点分路径]
    CS[CompilerSession / Scenario<br/>session.rs]
  end

  subgraph 运行期易失态
    EE[ExecutionEngine<br/>runtime.rs]
    SF[StackFrame<br/>runtime.rs]
    LIBS[libs/builtin.rs<br/>内建函数表]
  end

  subgraph FFI 桥
    CFFI[CFfiBridge<br/>ffi.rs · id 200+]
    PYFFI[PyFfiBridge<br/>py_ffi.rs · id 400+]
    NI[vm::native::NativeInterface<br/>Rust shim · id 100-199]
  end

  subgraph 服务集成
    SSE[SSEParser / SSEEvent<br/>sse/]
    RT[RouteDiscovery / RouteMerger<br/>route/]
  end

  A2RS[a2r_std.rs<br/>a2r 产物 Rust std]

  EE -->|call_stack / frames| SF
  SF -.->|scope_sid 单向链接| ST
  ST --> SID
  DB --> ST
  EE -->|new() 时装配| LIBS
  CFFI --> NI
  PYFFI --> NI
  NI --> VM[vm/ AutoVM 引擎<br/>CALL_NAT 分发]
  CFFI -.->|libloading| CDLL[外部 C 动态库]
  PYFFI -.->|PyO3 嵌入| CPY[CPython 解释器]
  EE -.->|Value / ValueID| AV[auto_val crate]
  DB -.->|DashMap| DM[dashmap]
  A2RS -.->|被转译产物 use| A2ROUT[a2r 输出 crate]
```

外部依赖：`auto_val`（值体系）、`vm/`（引擎与 NativeInterface）、`PyO3`（feature-gated）、
`libloading`、`dashmap`。本模块不持有 AutoVM 执行循环本身——ABC 指令集与调度器属 vm 模块。

## ADR 日志

### ADR-01: 编译期/运行期状态分离（Universe → Database + ExecutionEngine）
- 日期 / 来源：2025-02-01 完成 / plan-064（`docs/plans/old/064-split-universe-compile-runtime.md`）
- 决策：把原 `Universe` 拆为持久的编译期 `Database`（含 `SymbolTable`）与易失的运行期
  `ExecutionEngine`（含 `StackFrame`），两者以 `StackFrame.scope_sid → SymbolTable.sid` 单向链接。
- 备选：保持单一 Universe（pros：无迁移成本、引用简单；cons：编译期/运行期耦合，
  增量编译与热重载无法实现）；plan 064 §5 记录采用双结构的理由：术语标准化、
  多帧一表支持递归、持久/易失边界清晰。
- 后果：正面——增量编译与递归调用有结构支撑；负面——迁移未收尾，旧 `Scope` 仍以
  DEPRECATED 形态留存，`Scope.get_val` 是恒 `None` 桩；缓解——`SymbolTable::from_scope`
  迁移辅助 + 代码内迁移指南注释。
- 状态：active

### ADR-02: 运行期值引用化（ValueID + 中央 values 表）
- 日期 / 来源：plan 未标注日期 / plan-064 + `crates/auto-lang/src/runtime.rs`（"reference-based system" 注释）
- 决策：作用域/栈帧不直接持有 `Value`，改存 `ValueID`，值本体集中在
  `ExecutionEngine.values: HashMap<ValueID, Rc<RefCell<ValueData>>>`，另配 `weak_refs` 供清理。
- 备选：帧内直接持值（pros：查找快、无间接层；cons：值复制、跨帧共享与所有权
  move 跟踪难以表达）；`moved_vars` 集合配合实现 use-after-move 检查。
- 后果：正面——共享/所有权语义可统一表达；负面——每次取值多一次哈希间接。
- 状态：active

### ADR-03: 混合 FFI——内建 shim + 沙箱动态加载双轨
- 日期 / 来源：plan 未标注日期 / plan-092、plan-094（`docs/plans/old/092-rust-ffi-sandbox.md`、`094-hybrid-ffi-bridge.md`）
- 决策：Rust FFI 走双轨——VM 内建 shim（编译进引擎）+ `use.rust` 沙箱把用户 crate
  编译为 cdylib 动态加载，统一进 `NativeInterface` 混合查找。
- 备选：纯内建（pros：零加载成本、ABI 稳定；cons：用户 crate 无法接入）；
  纯动态（pros：通用；cons：启动开销、ABI 风险、错误面大）。
- 后果：正面——stdlib 性能与用户扩展性兼得；负面——两套注册路径，排错需分辨来源；
  plan-212 完成 cdylib 端到端管线（dep → cargo build → AutoVM 加载 → 调用）。
- 状态：active

### ADR-04: FFI 桥按语言拆分，native id 分段约定
- 日期 / 来源：plan 未标注日期 / plan-081 Phase 5、plan-214；代码 `ffi.rs`、`py_ffi.rs` 头注
- 决策：每种宿主语言一个 Bridge（`CFfiBridge`、`PyFfiBridge`，Rust 侧为 `NativeInterface`），
  CALL_NAT 的 native id 分段：100-199 Rust FFI、200+ C FFI、400+ Python FFI；
  `py_call`/`py_getattr` 固定 450/451 以便 codegen 在 `BIGVM_NATIVES` 预登记。
- 备选：全桥统一动态分配 id（pros：实现简单；cons：codegen 无法在不查询 bridge 的
  情况下登记稳定 id，跨模块编译产物不可重定位）。
- 后果：正面——id 空间无冲突、稳定常量可硬编码；负面——分段上限是隐式契约，
  新语言桥需自行避开既有区段。
- 状态：active

### ADR-05: Python FFI 嵌入 CPython（PyO3），镜像 Rust FFI 管线
- 日期 / 来源：plan 未标注日期（plan-222 完成，commits `9e119bb9`/`8cc9cc50`）/ plan-214、plan-222、plan-300
- 决策：`use.py` 通过 PyO3 在进程内嵌入 CPython，直接 import 模块注册 shim，
  不生成 wrapper crate；整体管线镜像 plan-212 的 Rust FFI，仅把 DLL 加载换成解释器导入。
- 备选：子进程 IPC（pros：解释器隔离、崩溃不波及 VM；cons：序列化开销、部署需
  伴随进程）；嵌入（pros：调用开销低、管线与 Rust FFI 同构；cons：GIL 约束、
  需 `python` feature gate、CPython 崩溃即进程崩溃）。
- 后果：plan-222 扩到 int/float/bool/string/list 多类型 marshalling；plan-300 加
  NanoValue tag 检测的 Auto 类型直通；无法映射的 Python 对象包成 `PyObjectHandle`
  存 VM 堆（plan-369 Task 12），经 450/451 内建做属性/方法分发。
- 状态：active

### ADR-06: stdlib 双/三文件模式（.at + .vm.at + .rs.at）
- 日期 / 来源：文档未标注日期 / `docs/design/13-networking.md` §"Dual-Mode Execution"；实证 `stdlib/auto/`（28 个 `.vm.at`/`.rs.at` 文件）
- 决策：公共 API 写在 `.at`；VM 专用 FFI 绑定写 `.vm.at`（`#[vm]` 声明映射 Rust 实现）；
  a2r 转译提示写 `.rs.at`（`#[rust_fn]`）。两种执行模式暴露同一 API。
- 备选：单文件内条件编译（pros：文件少；cons：公共 API 与平台实现混杂，
  两种后端互相污染）。
- 后果：正面——13 章规划的 async/net/http/json/url/log/env 模块全部按此落地；
  负面——同一语义维护至多三份声明，漂移风险由测试覆盖（plan-211）。
- 状态：active

### ADR-07: 混合路由——约定发现 + 配置覆盖
- 日期 / 来源：plan 未标注日期 / plan-114（`docs/plans/old/114-hybrid-routing.md`，文件内标题作 "Plan 119"）
- 决策：路由两来源——扫描 `routes/` 目录按文件命名约定（`index.at`→`/`、
  `user/[id].at`→`/user/:id`）自动发现，与 `routes {}` 配置块合并，配置覆盖约定。
- 备选：纯约定（pros：零配置；cons：layout/auth 等元数据无处表达）；
  纯配置（pros：显式可控；cons：每个页面都要写样板）。
- 后果：`route/` 下 `RouteDiscovery` + `RouteMerger` 两组件，`RouteDef.source`
  记录来源以便诊断；meta（layout、auth）只能走配置通道。
- 状态：active

### ADR-08: 场景化编译会话（Scenario: Core / UI / Shell）
- 日期 / 来源：代码现状 / `crates/auto-lang/src/session.rs` 头注（"Scenario Programming"）
- 决策：编译行为由 `CompilerSession` 携带的场景驱动——UI 场景才激活 `widget`/`view` 等
  上下文关键字；默认后端按场景分派（Core→a2r、UI→gpui、Shell→vm）。
- 备选：全局关键字全集（pros：解析器简单；cons：UI 关键字污染脚本/核心场景命名空间）。
- 后果：正面——同一语言服务三种场景而互不干扰；负面——关键字可用性依赖会话上下文，
  报错与 IDE 支持都需感知场景。
- 状态：active
