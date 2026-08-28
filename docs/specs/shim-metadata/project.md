# shim-metadata

> **Status**: active
> 路径：`crates/shim-metadata`  | 技术栈：Rust（syn / serde / clap）

rustdoc 元信息工具（Plan 430 管线）：从三方 Rust crate 的 rustdoc JSON / 源码 syn AST
提取签名，按规律分类器生成 **FFI method pack**（`.autoos` shim 包），供 AutoVM 经
`use.rust` 直接调用原生 Rust 库。lib 供进程内调用（被 auto-cache 与 auto-lang 依赖），
bin 为离线 CLI。（注释中"不入 auto-lang 依赖树"的说法已过期——`auto-lang/Cargo.toml` 已 path 依赖。）

## 目标与范围

- 签名提取（方法/自由函数/泛型过滤）× 规律分类 → 生成 shim 包（注册/路由/链接三关 + 实体）。
- 与 auto-cache 的方法包缓存（指纹/版本/manifest）配合，实现 `use uuid` 类直连。
- 不做：运行时桥接（那是 auto-lang vm/ffi 的职责）。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| lib | 提取 + 分类 + shim 生成（进程内 API） | active |
| bin | 离线 CLI 入口 | active |

## plans

- **plan-429** aavm B1 shim inventory ✅ archived——shim 存量盘点（reports/429-b1）
- **plan-430** dep methods 管线 ✅ archived——rustdoc→method pack 全管线（含 430-fixes 四项复审修复）
