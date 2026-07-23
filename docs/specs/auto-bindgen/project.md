# auto-bindgen

> **Status**: active
> 路径：`crates/auto-bindgen`  | 技术栈：Rust（clap / serde）

C 头文件/manifest 生成器：为 Auto 的 C FFI 生成 C 头文件与绑定 manifest（Plan 216）。

## 目标与范围

- 从 Auto 源码中提取 extern/FFI 声明（extractor）。
- 做 Auto 类型 → C 类型映射（type_map），产出 C 头文件与 JSON manifest。
- 不做：不生成 Rust 侧绑定；不做 C → Auto 方向的 bindgen。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| main | CLI 入口 | active |
| extractor | 从 Auto AST 提取 FFI 声明 | active |
| type_map | Auto ↔ C 类型映射 | active |
| manifest | manifest/头文件输出模型（serde） | active |
