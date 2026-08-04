# Plan 089: 类型声明存储迁移到 Infer 模块

> **状态**: ✅ **全部完成** (2026-02-13)
> **完成度**: 100%

## 概述

将所有类型声明存储从 codegen 和 Database 迁移到 infer 模块，建立统一的类型信息管理系统。

## 完成状态

| Phase | 状态 | 说明 |
|-------|------|------|
| Phase 1 | ✅ | 创建 TypeRegistry 模块 |
| Phase 2 | ✅ | 集成 TypeRegistry 到 InferenceContext |
| Phase 3 | ✅ | 更新 infer/expr.rs 的 Dot 表达式处理 |
| Phase 4 | ✅ | 更新 codegen.rs 使用 TypeRegistry |
| Phase 5 | ✅ | 实现类型参数替换 |
| Phase 6 | ✅ | 统一类型上下文 TypeStore（由 Plan 084 完成）|

## 已完成的工作

### Phase 1-5: TypeRegistry 模块（本 Plan 完成）

✅ **Phase 1**: 创建 TypeRegistry 模块
- 新建文件：`crates/auto-lang/src/infer/registry.rs`
- `TypeRegistry` 结构包含 `type_decls` 和 `generic_templates`
- 提供方法：`register_type_decl()`, `lookup_type_decl()`, `get_template()`

✅ **Phase 2**: 集成 TypeRegistry 到 InferenceContext
- 添加 `type_registry: TypeRegistry` 字段
- 在 `new()` 和 `with_database()` 中初始化
- 添加便捷方法 `register_type_decl()` 和 `lookup_type_decl()`

✅ **Phase 3**: 更新 infer/expr.rs 的 Dot 表达式处理
- 添加 `.type` 属性的特殊处理（返回对象类型）
- 实现 `Type::User(type_decl)` 和 `Type::GenericInstance(inst)` 的字段类型查找
- 数组元素访问返回元素类型

✅ **Phase 4**: 更新 codegen.rs 使用 TypeRegistry
- 在 `register_type()` 方法中添加同步调用
- 类型声明注册时同时更新 `infer_ctx.type_registry`
- 确保 infer 模块可以访问所有类型声明

✅ **Phase 5**: 实现类型参数替换
- 在 `Type::GenericInstance` 处理中添加类型参数替换
- 使用 `member.ty.substitute(&type_param_names, &inst.args)` 替换泛型参数
- 示例：`Point<int>.x` 中 `x` 的类型从 `T` 替换为 `int`

### Phase 6: 统一类型上下文 TypeStore（由 Plan 084 完成）

**原问题**：类型存储分散且重复
- Parser 和 Codegen 各自维护 InferenceContext 实例
- 两个独立的 TypeRegistry 实例导致数据重复
- 复杂的类型包装：`SharedTypeRegistry = Rc<RefCell<TypeRegistry>>`

**解决方案**：Plan 084 实现了 `TypeStore`

```rust
// crates/auto-lang/src/types.rs

/// 统一的类型存储（Plan 084 实现）
#[derive(Debug, Clone)]
pub struct TypeStore {
    type_decls: HashMap<AutoStr, TypeDecl>,
    fn_decls: HashMap<Name, Fn>,
    spec_decls: HashMap<AutoStr, SpecDecl>,
    generic_templates: HashMap<String, GenericTemplate>,
}
```

**Plan 084 完成的工作**：
- ✅ 创建 TypeStore 模块
- ✅ 集成 TypeStore 到 Parser、Codegen、InferenceContext
- ✅ 统一类型查询 API
- ✅ 实现类型同步机制（使用 `Arc<RwLock<TypeStore>>`）
- ✅ 移除 InferenceContext 中的冗余注册表（type_registry, fn_registry, spec_registry）

**最终架构**：

```
┌─────────────────────────────────────────────────────────────┐
│                    TypeStore (单一数据源)                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  type_decls: HashMap<AutoStr, TypeDecl>            │   │
│  │  fn_decls: HashMap<Name, Fn>                       │   │
│  │  spec_decls: HashMap<AutoStr, SpecDecl>            │   │
│  │  generic_templates: HashMap<String, GenericTemplate>│   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         ↑ shared via Arc<RwLock<TypeStore>>
         │
    ┌────┴────┬────────────┐
    │         │            │
 Parser   Codegen   InferenceContext
```

## 测试结果

### 所有 `.type` 测试通过

| 测试类型 | 数量 | 状态 |
|----------|------|------|
| `.type` 属性测试 | 43 | ✅ 全部通过 |
| infer 模块测试 | 116 | ✅ 全部通过 |
| types 模块测试 | 4 | ✅ 全部通过 |

## 成功标准

- ✅ 所有类型声明统一存储在 TypeStore 中
- ✅ `a.x.type` 返回字段类型（如 `"int"`）
- ✅ `a.type` 返回对象类型（如 `"A"`）
- ✅ `Point<int>.x` 返回 `int`（类型参数替换生效）
- ✅ Parser、Codegen、InferenceContext 共享同一个 TypeStore
- ✅ 移除冗余的 type_registry, fn_registry, spec_registry 字段
- ✅ 所有相关测试通过

## 相关 Plan

- **Plan 084**: 统一 TypeStore - 完成了 Phase 6 的工作
- **Plan 087**: AutoVM 泛型系统 - 90% 完成
- **Plan 088**: 函数参数传递模式 - 100% 完成

## 参考资料

- [Plan 084: Unified TypeStore](./084-unified-type-context.md)
- [Plan 010: Type Inference Subsystem](010-type-inference-subsystem.md)
- [类型推导实现总结](../type-inference-implementation-summary.md)
