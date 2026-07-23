# types 相关 plan 索引

> 状态列：plan 文件有显式 Status 的以文件为准；无显式 Status 的旧格式 plan 以
> docs/plan-indices/ 对应章节为准（已逐一比对）。归档列 = 当前位置。
> 重编号提示：327/336/337/338/342/351/355/359 已重编号为 317/318/320/322/330/346/347/348，
> 其中 318（list-struct-id-corruption）与 322（list-struct-runtime-diagnosis）涉 List 运行时
> 数据结构，但属 VM 模块范畴，未收入本表；引用时注意与同名旧号区分。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 008 | error-message-system | ✅（plan-indices/03） | old/ | miette 诊断 + 错误码体系奠基 |
| 009 | runtime-error-integration | 🔧 | old/ | evaluator panic! 替换为 RuntimeError，未收尾 |
| 010 | type-inference-subsystem | ✅（plan-indices/03） | old/ | infer/ 七阶段 HM 推断子系统落成 |
| 018 | type-composition-improvements | 🔧 | old/ | has 组合方法解析；evaluator/转译器仍 no-op |
| 019 | spec-trait-system | ✅ | old/ | spec 关键字、vtable、成员级委托、默认方法 |
| 021 | single-inheritance | ✅（plan-indices/02） | old/ | is 单继承 + C/Rust 转译支持 |
| 024 | ownership-first-implementation | ✅ | old/ | 三阶段所有权：move、拥有型 str、借检查 |
| 025 | string-type-redesign | ✅（plan-indices/02） | old/ | 字符串操作库 + C FFI |
| 026 | property-keywords | ✅ | old/ | view/mut/take 前缀关键字改后缀点记法 |
| 034 | borrow-checker-redesign | ✅（plan-indices/04） | old/ | Target 归一化 + 生命周期区域借检查 |
| 038 | fix-vm-borrowing | ✅ | old/ | VM 对象 RefCell 内部可变性修复 |
| 048 | generic-types | ✅ | old/ | 泛型类型替换；stdlib May<T>/List<T> 泛型化 |
| 049 | may-operators-generic-types | ✅（plan-indices/02） | old/ | ?T 操作符从硬编码 May 迁到泛型系统 |
| 052 | storage-based-list | ✅ | old/ | 定长/运行时数组与指针类型（Type::Array/Ptr 等） |
| 055 | storage-injection | ⏳ | old/ | 平台感知存储策略注入（MCU=Fixed/PC=Dynamic），未落地 |
| 056 | dot-expression-field-access | ✅（plan-indices/02） | old/ | 点表达式字段读写，区分字段与方法 |
| 057 | generic-specs | ✅ | old/ | 带类型参数的 spec（Storage<T>）+ 单态化 vtable |
| 058 | type-alias-syntax | ✅（plan-indices/02） | old/ | `type X = Y` 别名语法 |
| 059 | generic-type-fields | ✅（Phase 2-3 deferred） | old/ | 结构体泛型字段、const/mut 指针限定 |
| 060 | closure-syntax | ✅ | old/ | `x => x*2` 闭包、捕获、Type::Fn |
| 061 | generic-constraints | ✅（plan-indices/02） | old/ | 内联 `<T: Spec>` 约束（实际选择，取代 #[with] 设想） |
| 084 | unified-type-context | ✅（plan-indices 无，代码注释佐证） | old/ | TypeStore 单一数据源落地 |
| 085 | auto-use | ✅（plan-indices 无，types.rs 注释佐证） | old/ | TypeStore merge/import_items 模块导入 |
| 088 | param-passing-modes | ✅（plan-indices/04） | old/ | ABO-01：语义 view、小类型实现 copy；ParamChecker 未接管线 |
| 089 | infer-module-type-declaration-storage | ✅（infer/registry.rs 头注释） | old/ | infer 侧 TypeRegistry（现 DEPRECATED，让位 TypeStore） |
| 090 | remove-universe-from-parser | ✅（types.rs 注释佐证） | old/ | parser 去 Universe，TypeStore 提供 find_type_for_name/别名 |
| 120 | error-types-option-result | ✅（Phase 5 deferred） | old/ | Option/Result 入 AST，May 移除；传播操作符推迟 |
| 121 | task-msg-system | ✅（plan-indices/02 无，Type::Handle 佐证） | old/ | task 句柄类型 Type::Handle |
| 122 | value-access-refactor | ✅ | old/ | 调用点 .view/.mut/.move 值访问重构 |
| 125 | phase3-polymorphic-routing | ✅（infer/mod.rs 注释佐证） | old/ | task 类型检查 TaskTypeChecker（Phase 3.6） |
| 190 | rust-stdlib-use-extension | ✅（types.rs 注释佐证） | old/ | use.rust 类型入 TypeStore（Type::Rust） |
| 191 | assert-and-precise-linker-errors | ✅（plan-indices/02） | old/ | assert 内建 + 链接错误精确定位 |
| 193 | conv-type-conversion | ✅（plan-indices/02；遗留注记称仍 DRAFT） | old/ | 统一 .to() 与 Conv<From,To> spec |
| 194 | monomorphic-dispatch | ✅（plan-indices/02） | old/ | HashMap/HashSet 泛型 API 编译期单态分发 |
| 198 | native-metadata-from-source | ✅ | old/ | TypeStore.all_fn_decls 支撑返回类型推导元数据 |
| 208 | result-heap-object | ✅ | old/ | Result 堆对象化：IS_OK/UNWRAP/ERROR_PROPAGATE |
| 310 | auto-ownership-escape-analysis | ✅（任务 4.3 推迟） | archive/ | 逃逸分析 + Rc/Arc 回退，用户免写生命周期 |
