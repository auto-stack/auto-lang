# runtime（运行时与内建库）

> **Status**: partial（核心结构已实现并接入；Plan 064 分层迁移未收尾，`Scope` 遗留）

## 职责

解释器/编译器侧的运行时支撑层，与 `vm/`（AutoVM 字节码虚拟机）是两个层次：

- 运行期状态：`ExecutionEngine`（值存储、调用栈、VM 资源引用、环境变量、内建函数缓存）。
- 编译期符号：`Sid` 作用域标识、`SymbolTable` 符号表（持久，存 Database）。
- 编译会话：`CompilerSession` / `Scenario`（Core/UI/Shell 场景化编译）。
- 内建库绑定：`libs/`（解释器内建函数表）、`a2r_std.rs`（a2r 转译产物的 Rust 运行时）。
- FFI 桥：`ffi.rs`（C FFI）、`py_ffi.rs`（Python FFI，PyO3 嵌入 CPython）。
- AIE 存储：`database/`（FileId/FragId 增量编译存储 + UI 产物缓存）。
- 服务集成：`sse/`（SSE 解析）、`route/`（约定+配置混合路由）。

## 现状

- Plan 064 分层已落地：`SymbolTable`（编译期）+ `StackFrame`（运行期）双结构均在代码中，
  以 `StackFrame.scope_sid → SymbolTable.sid` 单向链接，支持递归（多帧一表）。
- 旧 `Scope` 结构标记 DEPRECATED 但仍存在，`Scope.get_val` 是恒返 `None` 的桩（迁移未完成）。
- 内建函数经 `libs::builtin::builtins()` 注入 `ExecutionEngine`；string/result/may/file/sys 各模块齐备。
- C FFI（`CFfiBridge`，native id 200+）与 Python FFI（`PyFfiBridge`，native id 400+，
  `py_call`/`py_getattr` 固定 450/451）均可用；Rust FFI 见 vm 模块的 `NativeInterface`。
- `database/` 两层（存储 + 缓存）结构在码，UI 产物缓存（`UIArtifact`/`UICache`）已并入。
- 网络栈按 docs/design/13 的层次落地在 `stdlib/auto/`：`http.at`+`http.vm.at`、
  `net.at`、`async.at`+`async.vm.at`、`json/url/log/env/sse` 等，双文件（部分三文件 `.rs.at`）模式真实存在。

## 关键入口

- `crates/auto-lang/src/runtime.rs:ExecutionEngine` — 运行期状态容器（每次执行 ephemeral）
- `crates/auto-lang/src/runtime.rs:StackFrame` — 运行栈帧（vals/moved_vars/父帧链）
- `crates/auto-lang/src/scope.rs:Sid` — 点分路径作用域标识（`kid_of`/`parent`/`name`）
- `crates/auto-lang/src/scope.rs:SymbolTable` — 编译期符号表（持久）
- `crates/auto-lang/src/scope.rs:Scope` — 旧混合结构（DEPRECATED，plan-064）
- `crates/auto-lang/src/scope_manager.rs:ScopeManager`
- `crates/auto-lang/src/session.rs:CompilerSession` / `crates/auto-lang/src/session.rs:Scenario`
- `crates/auto-lang/src/libs/builtin.rs:builtins` — 内建函数表装配
- `crates/auto-lang/src/ffi.rs:CFfiBridge` — C 库注册与 CALL_NAT 桥（plan-081 Phase 5 / plan-216）
- `crates/auto-lang/src/py_ffi.rs:PyFfiBridge` / `crates/auto-lang/src/py_ffi.rs:PyObjectHandle`
- `crates/auto-lang/src/py_ffi_types.rs:PySignature`
- `crates/auto-lang/src/database/mod.rs:Database` / `FileId` / `FragId`
- `crates/auto-lang/src/sse/parser.rs:SSEParser` / `crates/auto-lang/src/sse/types.rs:SSEEvent`
- `crates/auto-lang/src/route/mod.rs:RouteDiscovery` / `RouteMerger` / `RouteDef`
- `crates/auto-lang/src/a2r_std.rs:List` — a2r 转译产物链接的 Rust std

## 使用示例

```rust
// 运行期引擎：内建函数自动装配，帧栈支持词法遮蔽
let mut engine = ExecutionEngine::new();
let fid = engine.push_frame(Sid::from("main"));
engine.current_frame().unwrap().borrow_mut().set("x".into(), engine.alloc_value());

// 编译会话：UI 场景 + 后端
let session = CompilerSession::ui().with_backend("react");
assert_eq!(session.backend_or_default(), "react");

// 路由：约定发现 + 配置覆盖
let discovered = RouteDiscovery::new("routes".into()).discover()?;
let merged = RouteMerger::merge(discovered, config_routes);
```

## 已知坑

- `Scope`（scope.rs）已废弃但未删除；新代码必须用 `SymbolTable` + `StackFrame`，
  `Scope.get_val` 永远返回 `None`（代码内 TODO 明示）。
- `libs/std.rs` 是 0 行空文件，仅占模块位。
- `database/mod.rs` 头注 "Plan 134: UI Artifact" 实为 plan-135（134 是 jet-generator-view-body）。
- `docs/plans/old/114-hybrid-routing.md` 文件内标题写作 "Plan 119"；代码注释按文件名引 plan-114。
- plan 重号：`old/` 下有两个 152（SSE 与 a2ts 各一）；355 在 archive/（session 递归修复）
  与 plans/（a2r async-await 转译）各一，引用须带归档位置区分。
- docs/design/05-vm-runtime.md 的 "Status" 描述的是 `vm/` 子系统，不是本目录这些文件；
  本模块的 VM 字节码细节见 vm 模块 spec。

## 蒸馏来源（Phase 1）

- `docs/design/05-vm-runtime.md`、`docs/design/13-networking.md`
- `docs/plans/old/064-split-universe-compile-runtime.md`、`081`、`092`、`094`、`102`、`114`、`152-streaming-http-sse`、`154`、`195`、`211`、`212`、`214`、`216`、`222`、`224`、`250`、`267`
- `docs/plans/` 300、312-313（archive/）、316-318、321-322、326、328-329、334-335、341、344、349-350、352-353、355（archive/）
- 代码：`crates/auto-lang/src/{runtime,scope,session,ffi,py_ffi,py_ffi_types,scope_manager,a2r_std}.rs`、`libs/`、`database/`、`sse/`、`route/`、`stdlib/auto/`
