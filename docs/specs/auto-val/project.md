# auto-val

> **Status**: active
> 路径：`crates/auto-val`  | 技术栈：Rust（ecow / indexmap）

底层值/AST 节点数据结构（Value/Node/Obj/Pair），整个 workspace 最底层的基础库，不依赖其他 auto crate。

## 目标与范围

- 定义 Auto 运行时表示：Value、AST Node、Obj/Pair 键值结构。
- 提供紧凑字符串与集合类型（AutoStr/StrSlice/OwnedStr/CStr、Array/Linear/Kids）。
- 提供路径（AutoPath）、元信息、emit/to_value 等基础工具。
- 不做：不实现解析/求值；不依赖上层 crate，保持零 auto 内部依赖。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| value / nano_value | 值表示（Value 枚举及紧凑变体）。plan-576 契约钉：encode→decode 位型恒等对照集（f64/f32 含整值 240.0/0.0/-0.0——tag 保真 + 0.0/-0.0 按位可分），encode/decode 纯往返不自证消费端正确性（043 期实锚在 engine 消费臂，见 auto-lang vm/bytecode-engine.md plan-576 注记）。Plan 566 装箱瘦身终态：13 胖变体（Node/Obj/Instance/Widget/Meta/Model/View/Grid/Method/Closure/FutureData/Fn/Type 等）Box 直通，Value 296B→40B（Node 296→112→72B 三段），≤48B 由 size_report 防回退断言锁死；Str 40B 为地板（热路径字符串不装箱，32B 否决） | active |
| size_report | 尺寸审计（Plan 566 新增）：size_table 全变体载荷排名 + 防回退断言（`VALUE_SIZE_LIMIT`=48 const + 运行期 `assert!(size_of::<Value>() <= ...)`，超标即测试红） | active |
| node | AST 节点结构 | active |
| obj / pair | 对象与键值对结构 | active |
| string / str_slice / owned_str / cstr | 字符串类型族（AutoStr 等） | active |
| array / linear / kids | 集合/子节点容器 | active |
| meta / path / shared / types / to_value / emit | 元信息、路径、共享类型与转换输出工具 | active |
