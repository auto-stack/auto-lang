# auto-bindgen

> **Status**: active
> 路径：`crates/auto-bindgen`  | 技术栈：Rust（clap / serde）

C 头文件/manifest 生成器：为 Auto 的 C FFI 生成 C 头文件与绑定 manifest（Plan 216）。

## 目标与范围

- 从 Auto 源码中提取 extern/FFI 声明（extractor）。
- 做 Auto 类型 → C 类型映射（type_map），产出 C 头文件与 JSON manifest。
- 不做：不生成 Rust 侧绑定；不做 C → Auto 方向的 bindgen。

## manifest 模型（Plan 595）

- `CTypeDesc` 覆盖原语 + 指针 + **`FnPtr{ret, params}` 回调变体**（首个
  回调用例 = win32 `SetConsoleCtrlHandler` 的 handler 参数）；FnPtr 的
  C/Rust 拼写由 type_map 组合生成（返回 `String`，非 `&'static str`）。
- `CHeaderManifest.abi` 注记：serde default `"c"`，win32 头用
  `"system"`（x64 上与 C 同约定；x86 stdcall 差异为已知边界）。旧 JSON
  无 abi 字段可继续反序列化（默认 c）。
- 内置数据集：5 标准头（string/math/stdio/stdlib/time，Plan 216）+
  **windows.h kernel32 console 子集 6 函数**（GetCommandLineA/
  FreeConsole/AttachConsole/SetConsoleCtrlHandler/GenerateConsoleCtrlEvent/
  Sleep，library=kernel32，abi=system）——`use.c <windows.h>` 与
  `kernel32` 均可命中；CLI 可导出 JSON 至 `c_bindings/`。
- 消费面：VM 运行期对 FnPtr 签名在注册期即拒绝（见
  auto-lang/vm/design/ffi.md）；a2c 原生消费（闭包→函数指针）。
- JSON 文件加载面（Plan 597）：VM 侧 `load_manifest_file(path)`
  （vm/ffi/c_ffi.rs）装载任意 JSON manifest 文件（非内置数据集；
  codegen 先查内置字典，header 为 `.json` 后缀时回退文件装载，
  `use.c "x.json"` / `use.c <x.json>` 双拼写命中）。engine 专属
  manifest **不入内置字典**，归语料侧随用例分发（先例
  `crates/auto-lang/test/vm_engine_face/engine_face.json`，autoterm
  引擎 8 标量符号）。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| main | CLI 入口 | active |
| extractor | 内置 manifest 数据集（标准头 + windows.h console 子集） | active |
| type_map | Auto ↔ C 类型映射（FnPtr 组合签名，String 返回） | active |
| manifest | manifest/头文件输出模型（serde；FnPtr + abi） | active |
