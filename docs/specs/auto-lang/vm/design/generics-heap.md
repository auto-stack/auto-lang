# 泛型与堆对象管理

## 范围

泛型单态化管线、类型擦除存储、统一堆对象注册表、ListData 存储策略、Option/Result 堆表示。对应代码：`vm/generic.rs`、`vm/monomorphize.rs`、`vm/generic_registry.rs`、`vm/heap_object.rs`、`vm/heap.rs`、`vm/list_data.rs`、`vm/list_storage.rs`。

## 原则

- 单态化：每种类型参数组合生成特化字节码，原始类型零装箱开销（ADR-04）。
- 单一注册表：所有堆对象收敛到一张表，opcode 处理只面向 `HeapObject` trait（ADR-05）。
- 内置集合特化、用户泛型擦除：两条存储路线并存，按构造来源分流。

## 细节

### 单态化管线

- `GenericTable`（generic.rs:84）在编译期记录全部泛型实例化；`extract_generic_instance`（generic.rs:152）解析 `List<int>` 等类型。
- `Codegen::monomorphize()`（codegen.rs:713）把收集到的实例交给 `Monomorphizer`（monomorphize.rs:21），产出 `MonomorphizedModule`；`is_monomorphizable`/`collect_monomorphizable_types`（monomorphize.rs:168/178）筛选可单态化类型。
- 泛型集合 API 的单态派发在编译期按类型解析到具体 shim（plan-194），消除类型后缀 API 名。

### 统一堆对象注册表

- `AutoVM.heap_objects: DashMap<u64, Arc<RwLock<dyn HeapObject>>>`（engine.rs，plan-077 Phase 4）；旧 per-type list 注册表已在 Phase 6 移除（代码注释为准，plan 文件自述 50% 滞后）。
- `HeapObject` trait（heap_object.rs:39）：`type_tag()` 运行期类型检查 + `as_any()/as_any_mut()` 下转；`TypeTag`（heap_object.rs:66）枚举所有堆类型。
- `try_downcast_checked()`（heap_object.rs:297）把 tag 检查与下转合并为单次内联操作，plan-077 记录比分开操作快 17%；下转代价约 15ns/次，被原始 list 6 倍内存节省抵消，净 1.43x。

### ListData 双轨存储

- `List<int>` → `ListData<i32>`（4 字节/元素）；含 struct/str 元素 → `ListData<Value>`（plan-340 §1.1 描述双轨制）。
- plan-322/335/340 逐步补齐 `ListData<Value>` 的方法覆盖：push/len（322）→ read_state_as_vec 解引用（335）→ filter/map/remove/foreach/find/contains/get（340）。
- plan-318 修复 struct 元素 ID 的 nanbox tagging 损坏（VmRef 解引用上游）。

### 用户泛型与 Option/Result

- `GenericRegistry`（generic_registry.rs:617）管理 `ClassTemplate`/`ClassType`；`GenericInstanceData`（generic_registry.rs:506）为类型擦除实例存储，字段 `Vec<Value>`。
- 实例化走对象字面量 `Pair { key: 1, val: "a" }`；字段访问、方法调用可用（plan-087）。
- Option/Result 是堆对象：CREATE_OK/CREATE_ERR 建 GenericInstanceData，IS_OK/UNWRAP_OK/UNWRAP_ERR/ERROR_PROPAGATE/IS_VARIANT 操作（plan-208、plan-201、plan-229a）。

## 显式非目标

- 命名参数函数调用式构造 `Pair(key: 1, val: "a")` 与泛型实例类型标注 `let p Pair<int, string>`（plan-087 限制，report 07 Open Questions）。
- 泛型约束/trait bound 的单态化特化（属类型系统范畴）。

> 来源: docs/plan-reports/07-vm-runtime.md（§Type System and Generics）；crates/auto-lang/src/vm/{generic,monomorphize,generic_registry,heap_object}.rs；plan-076/077/087/194/201/208/318/322/335/340
