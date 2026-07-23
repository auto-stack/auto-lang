# FFI 桥（C / Rust / Python）

## 范围

`ffi.rs`（C FFI）、`py_ffi.rs` + `py_ffi_types.rs`（Python FFI），以及与 vm 模块
`NativeInterface`（Rust shim）的协作。不含 a2r 直译路径（无 FFI，直接生成 Rust 调用）。

## 原则

- 统一出口：所有外部调用最终编译为 `CALL_NAT <native_id>`，由 shim 完成值转换。
- 按语言分桥，native id 分段避免冲突，稳定 id 允许 codegen 预登记。
- 混合双轨：VM 内建 shim（性能）与沙箱/解释器动态接入（扩展性）并存（plan-094）。

## 细节

### native id 分段（代码注释明示的契约）

| 区段 | 用途 | 出处 |
|------|------|------|
| 100-199 | Rust FFI shim | ffi.rs:71 注释 |
| 200+ | C FFI（`CFfiBridge` 自 200 起分配） | ffi.rs:82 |
| 400+ | Python FFI（`PyFfiBridge` 自 400 起分配） | py_ffi.rs:80 |
| 450/451 | `py_call`/`py_getattr` 固定内建，codegen 在 `BIGVM_NATIVES` 硬编码登记 | py_ffi.rs:63-64 |

### C FFI（`CFfiBridge`，plan-081 Phase 5 / plan-216）

- 工作流四阶段：编译期 `#[c]` 声明 → 注册（library, 函数名, `CSignature`, 动态库路径）
  → codegen 生成 `CALL_NAT <id>` → 执行期 shim 做值↔C 参数互转。
- `functions: HashMap<(String, String), u16>` 记录 (库, 函数)→native_id；
  `libraries: HashMap<String, libloading::Library>` 持有已加载库句柄。
- plan-216 提供 `auto-bindgen` 自动从 C 头文件提取签名，接入构建管线。

### Rust FFI（plan-092/094/212）

- 内建 shim 编译进 VM；用户 crate 经 `use.rust` 沙箱编译为 cdylib 动态加载，
  两路统一进 `NativeInterface` 混合查找。
- plan-212 完成端到端：dep serde_json → cargo build cdylib → AutoVM 加载 .dll → 调用。

### Python FFI（`PyFfiBridge`，plan-214/222/300）

- PyO3 进程内嵌入 CPython（`#[cfg(feature = "python")]` 门控），`import_module` 直接
  导入，不生成 wrapper crate；管线镜像 plan-212 的 Rust FFI。
- marshalling 演进：plan-214 MVP 仅 string→string；plan-222 扩至 int/float/bool/
  string/list（`PySignature`/`PyType`，py_ffi_types.rs）；plan-300 加 NanoValue tag
  检测的 Auto 类型直通参数与返回。
- 不可映射的 Python 对象（如 `datetime.date`、自定义类）包为 `PyObjectHandle`
  （`Py<PyAny>` + 类型名）存入 VM 堆，栈上只压堆 id；后续经 450/451 内建
  回解并分发属性/方法访问。`Py<PyAny>` 在 PyO3 0.29 为 `Send + Sync`，
  解引用仍需经 `Python::attach` 获取 GIL（py_ffi.rs 头注）。

### 不变量

- 同一 (库, 函数) 重复注册不会复用 id——各 Bridge 的 `next_native_id` 单调递增。
- shim 必须保持栈平衡：弹出参数、压回恰好一个返回值（CALL_NAT 语义，见 design/05）。
- Python 路径所有对象访问必须在 GIL 持有内进行。

## 显式非目标

- 不做 polyglot 插件系统的通用 `Plugin` trait（design/05 列为 Open Question）。
- 不做 WebAssembly/JS 宿主桥接。
- Python FFI 不解决 CPython 崩溃隔离（嵌入模型的已知代价，进程隔离是非目标）。

> 来源: crates/auto-lang/src/ffi.rs、py_ffi.rs、py_ffi_types.rs；docs/plans/old/081、092、094、212、214、216、222；docs/plans/300-python-ffi-runtime-maturation.md
