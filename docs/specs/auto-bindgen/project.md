# auto-bindgen

> **Status**: active
> 路径：`crates/auto-bindgen`  | 技术栈：Rust（clap / serde）

C 头文件/manifest 生成器：为 Auto 的 C FFI 生成 C 头文件与绑定 manifest（Plan 216）。

## 目标与范围

- 从 Auto 源码中提取 extern/FFI 声明（extractor）。
- 做 Auto 类型 → C 类型映射（type_map），产出 C 头文件与 JSON manifest。
- 不做：不生成 Rust 侧绑定；不做 C → Auto 方向的 bindgen。

## 注解导入器（PLAN-967，M2-A 已验）

`rust_import.rs` 与 `c_import.rs` 是注解源码导入器（元数据=源文件自身，无手写 per-fn manifest）：

- **Rust**（`auto.bindgen.rust-provider/1`）：`//! auto-ffi provider <id>/<ver> target= profile=`
  头 + `// auto-ffi fn <name> symbol= form= ret=|errors=` 条目 + `// auto-ffi dep <name> <spec>`
  依赖注解（cargo 单元消费）+ `frame=p967.frame/1` 或 `p967.frame/2` tag 帧轴（恰一
  `frame: &F0Frame` 参数；`/2` 是带类型浮点值的超集，PLAN-1129，其他版本仍具名拒绝；
  其中 `f` 标签的 payload 须高 32 位为零，超出具名拒绝、不截断）。解码器源文本是同目录的
  `frame_shim_prelude.rs`：生成器 `include_str!` 原样拼入 provider，单元测试 `include!` 直接编译
  同一份文本。参数域 view+i32/bool 混合（≤4）；域外形状（泛型/owned String/Vec/Result/async/trait对象/
  &mut）逐项具名拒绝 `rust-import.*`。产出中性 manifest + 生成 C 兼容 shim（含两版 tag 帧的
  `F0Val/F0Kwarg/F0Frame` 类型与同一游标解码器）+ 共享 `ace.foreign.manual/2` 投影。冻结 target 校验。
- **C**（`auto-ffi-c` 注解头）：真 cl /EP 预处理闭包解析（record/enum/callback/view/out/
  opaque/static-inline/常量；varargs/union/bitfield/packing 具名拒绝）；inline shim 重发与
  enum 常量提取。
- 消费与验证见 [ace-migration/m2-host.md](../ace-migration/m2-host.md)（F0/T-09..15 各节）。

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
  auto-lang/vm/design/ffi.md）；a2c 原生消费（闭包→函数指针）；
  **a2r 双形态消费（Plan 610 ⑥，004 §3.5"一套语义三个 lowering"
  收口）**——manifest 即共享 IR：`"link": "static"`（serde 默认；
  新字段，旧 JSON 不受影响）生成 S 形态（`#[link(name)]` extern 块 +
  安全包装），`"link": "dynamic"` 生成 D 形态（libloading，env
  `<LIBRARY>_DLL` → exe 同目录 → 裸名解析序）；FnPtr 在 a2r 侧暂显式
  报错（trampoline 选型 A' 待语料驱动落地，PLAN-610 §5）。
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

## PLAN-1093 增量（普通 AV 消费端）

`rust_import::import` 派生的 manifest 与 `c_import::to_manual_manifest` 产物现由普通
AV worker 的预构建 provider 闭包消费：`ace-av` 的 `foreign_source` 模块纯派生
manifest/shim、按 stamp+SHA 校验 prepared cdylib 并映射 967 冻结 callable。
`use.c <string.h>` 头角括号拼写由共享 `auto-source.at`（`provider-choice` +
`c-header-item` 规则）识别。provider 的物理签名（ViewText (data,len) 等）经
resource sig 进入 AV stub 表；非冻结形状具名拒绝。manifest 不入内置字典，
随 provider 闭包（`ACE_AV_PROVIDER_ROOT/<provider>.descriptor.json`）分发。
