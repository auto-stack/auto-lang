# Native 与 FFI

## 范围

native 函数注册体系、Rust/C FFI 动态加载、标准库 shim。对应代码：`vm/native.rs`、`vm/native_registry.rs`、`vm/native_catalog.rs`、`vm/qualified_name.rs`、`vm/ffi/`（mod.rs、c_ffi.rs、rust_stdlib.rs、stdlib.rs、convert.rs、http_server.rs、websocket.rs、c_bindings/）。

## 原则

- 单一注册：编译期元数据与运行时 shim 同源，消除双注册表（ADR-09）。
- 限定名解析：函数以 `QualifiedName` + import scope 解析，不留短名别名（plan-203）。
- FFI 错误经 `VMError::FFI`（engine.rs:182）统一上报。

## 细节

### Native 注册体系

- `AutoVMNativeRegistry`（native_registry.rs:42）：编译期全局注册表，存函数 ID 与返回类型（`NativeRetType`，native_registry.rs:31）；`register_builtin_natives()`（native_registry.rs:455）集中注册内置函数。
- `NativeInterface`（native.rs:33）：运行时 shim 表，CALL_NAT 按 ID 派发。
- plan-249 落地单一注册架构：惰性注册 + catalog 宏（native_catalog.rs），消除历史双注册表（BIGVM_NATIVES vs shim registry）。
- plan-198 使 native 元数据从 `#[vm]` 源声明派生，消除硬编码。
- plan-203 引入 `QualifiedName`/`resolve_qualified`/import scope，消除约 137 个短名别名，单态派发随之重构（Phase 5f 仍 deferred）。

### Rust FFI

- 混合桥：`#[rust_fn]` 宏声明 shim，plan-094 完成 43 个 shim；plan-092 建立沙箱约定。
- 动态加载端到端：依赖（如 serde_json）→ `cargo build` cdylib → AutoVM `load` .dll → 调用（plan-212b，MVP 为 string→string）。
- `ffi/rust_stdlib.rs` 为 Rust 实现的标准库函数提供 shim 注册。

### C FFI

- `CFfiRuntime`（ffi/c_ffi.rs:19）基于 libloading 装载动态库；`load_builtin_manifest`（c_ffi.rs:449）从 C 头文件生成绑定清单。
- plan-216 把 auto-bindgen 接入 CLI 构建管线（4 个阶段完成）：头文件 → 绑定 → 编译 → VM 调用。
- `ffi/convert.rs` 负责 Value ↔ C ABI 类型转换；`ffi/error.rs` 定义 FFI 错误面。
- **FnPtr（回调）注册期拒绝（plan-595，承 Plan 267 "Impossible" 定性）**：
  manifest 含 `CTypeDesc::FnPtr` 签名的函数（如 windows.h 的
  SetConsoleCtrlHandler）在 `load_header` 注册期即返回
  `VMError::FFI`（清晰报错替代 panic/静默误调）；封送循环另有同语义
  防御臂。回调消费走 a2c 后端（闭包→函数指针，Plan 060）。

### dep 方法 shim 包（三方 crate 动态加载）

- **装载与校验**：`dep_methods::register_pack` 解析 cdylib 内嵌
  `auto__shim_manifest`；**manifest format 校验拒载旧版**（format bump 一键
  失效，PLAN-591 v2）。布局探针 `auto__shim_layouts` 装载期合并（缺失容忍）。
- **DepOpaqueObject**：cdylib 堆对象句柄（ptr + 析构符号 + lib 保活）+
  `layout: Arc<TypeLayout>`（探针实测偏移）。
- **字段直读直写（PLAN-591 T1）**：GET_FIELD offset 直读优先——标量字段按
  探针偏移 `read_unaligned` 读 cdylib 堆；String/嵌套句柄等非标量落合成
  getter 面（592 桥，clone 语义）。SET_FIELD dep 臂标量 `write_unaligned`
  直写（写穿透 cdylib 堆，Rust 侧读回可见）。写失效/别名语义=KNOWN-DEBT
  P591-D1（单线程纪律）。
- **Option/Result 语义（PLAN-591 T2）**：`Option<T>` Some 压值/None 压 null
  （s/p 槽）；`Result` Err 经 `auto__last_error` 通道转 VMError。
  `dispatch` 的 `unwrap` 恒等桥：a2r `.unwrap()` 透传（null unwrap 报错）。
- **Display 路由（DIV-DEP-8 print 半边）**：print/TYPE_TO_STR/write 对 dep
  对象路由 shim 包合成 to_string（rustdoc 对 impl Display 类型合成）；
  无 Display 面维持占位。native opaque 面（RustStdlibObject）走
  `format_rust_stdlib_obj` 手写 Display 臂（semver::Version/url::Url 等，
  PLAN-596 T-07 补 url::Url——print 与 TYPE_TO_STR 共用此表）。
- **EQ 数值谓词（DIV-DEP-16 修复）**：`nv_is_numeric`/`nv_as_f64` 补
  TAG_I64——dep 宽整型返回与字面量相等比较此前恒 false。
- **trait 白名单转发（PLAN-596 T3）**：classify 对 trait impl 不再全排除——
  opt-in 白名单（`Display`/`ToString`/`Clone`/`base64::Engine`，编译期常量
  表 + manifest 记版本）产出 `TraitPlan`，emit 转发 wrapper
  `auto_<Type>__trait_<Trait>_<method>_<sig>`，内体
  `<Type as Trait>::method(recv, args)` 编译期定死分发。白名单外 trait 维持
  排除（调用报 Unknown，含 type.method 名与行号，可诊断）。
- **泛型实例化（PLAN-596 T4）**：rustdoc 泛型项不再无条件 skip——
  codegen 词法推导调用点实参（全整型→i64/全字符串→String）作 mono 提示，
  经 dep_scanner 写入 FunctionShim；emit 按替换引擎产
  `fn::<ConcreteType>` 实例 shim（条目名带 `__<label>` 后缀，dispatch 兜底
  剥后缀重试）。边界：mono 实参限基元/Str/白名单句柄类型；嵌套泛型维持 skip。
  manifest 增 mono 段入指纹与快路径比对（BTreeMap 稳定序列化）。
- **反向回调 adapter 原型（PLAN-596 T5，experimental）**：rust 形参
  `Box<dyn Fn(i64)->i64>` 时 wrapper 产 adapter（持回调令牌 u64），经注入式
  宿主跳板 `auto__register_host_trampoline` 重入 VM（`vm.call_closure`
  同步重入、线程局部回调帧、深度 1 守卫、跳板内 catch_unwind→CB_PANIC→
  VMError）。边界：单线程/同步/重入深度 1；回调内再调三方方法不支持
  （运行时报错）；Send/Sync/多线程泵归后续（KNOWN-DEBT 指针在案）。

### 内置服务 shim

- `ffi/http_server.rs`、`ffi/websocket.rs` 把 HTTP/WebSocket 服务暴露为 native 函数（plan-312/313/349/350 系列）；任务挂起配合 `waiting_http_request_id`/`waiting_sse_stream_id`（见 concurrency.md）。

## 显式非目标

- 多语言 FFI 插件系统（`Plugin` trait、handle 表生命周期）：design/05 Open Questions，未实现。
- Python FFI 运行时（CALL_PY 等）：属 python 集成线（plan-300/369），不在本主题展开。

> 来源: docs/plan-reports/07-vm-runtime.md（Plan Index 198/203/212b/216/249 行）；crates/auto-lang/src/vm/{native,native_registry,native_catalog,qualified_name}.rs、ffi/ 目录；plan-092/094/198/203/212b/216/249
