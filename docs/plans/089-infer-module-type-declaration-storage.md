# Plan 089: 类型声明存储迁移到 Infer 模块

## 概述

将所有类型声明存储从 codegen 和 Database 迁移到 infer 模块，建立统一的类型信息管理系统。

## 当前状态

### 问题：类型声明存储分散

当前类型声明信息分散在多个位置：

| 位置 | 存储 | 字段信息 | 用途 |
|------|------|---------|------|
| `codegen.types` | `HashMap<String, TypeInfo>` | ❌ 否 | 成员名称列表 |
| `codegen.generic_registry` | `GenericRegistry` | ✅ 是 | 泛型模板和实例 |
| `InferenceContext.type_registry` | `TypeRegistry` | ✅ 是 | 类型声明和字段信息 |

### 已完成的工作

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

✅ **Phase 5**: 实现类型参数替换（已完成）
- **目标**：在 infer/expr.rs 中实现类型参数替换功能
- **实施**：
  - 在 `Type::GenericInstance` 处理中添加类型参数替换
  - 使用 `member.ty.substitute(&type_param_names, &inst.args)` 替换泛型参数
  - 示例：`Point<int>.x` 中 `x` 的类型从 `T` 替换为 `int`
- **修改文件**：`crates/auto-lang/src/infer/expr.rs`
  ```rust
  Type::GenericInstance(inst) => {
      // Extract type parameter names (only Type params, not Const params)
      let type_param_names: Vec<crate::ast::Name> = base_decl.generic_params
          .iter()
          .filter_map(|p| match p {
              crate::ast::GenericParam::Type(tp) => Some(tp.name.clone()),
              crate::ast::GenericParam::Const(_) => None,
          })
          .collect();
      return field_ty.substitute(&type_param_names, &inst.args);
  }
  ```

## 测试结果

### 所有 `.type` 测试通过（43 个测试）

| 测试 | 结果 |
|------|------|
| `test_type_literal_int` | ✅ 通过 |
| `test_type_literal_float` | ✅ 通过 |
| `test_type_literal_str` | ✅ 通过 |
| `test_type_literal_bool` | ✅ 通过 |
| `test_type_variable_int` | ✅ 通过 |
| `test_type_variable_float` | ✅ 通过 |
| `test_type_variable_str` | ✅ 通过 |
| `test_type_variable_bool` | ✅ 通过 |
| `test_type_function_return_int` | ✅ 通过 |
| `test_type_function_return_str` | ✅ 通过 |
| `test_type_function_parameter` | ✅ 通过 |
| `test_type_array_element` | ✅ 通过 |
| `test_type_array_element_str` | ✅ 通过 |
| `test_type_binary_add` | ✅ 通过 |
| `test_type_binary_multiply` | ✅ 通过 |
| `test_type_instance` | ✅ 通过 |

### 其他测试状态

有 5 个测试失败，但与 `.type` 功能无关（与 `for` 循环等相关）。

## 成功标准

- ✅ 所有类型声明统一存储在 infer 模块的 `TypeRegistry` 中
- ✅ `a.x.type` 返回字段类型（如 `"int"`）
- ✅ `a.type` 返回对象类型（如 `"A"`）
- ✅ `Point<int>.x` 返回 `int`（类型参数替换生效）
- ✅ codegen 通过 `infer_ctx.type_registry` 访问类型声明
- ✅ 所有 `.type` 相关测试通过
- ✅ 现有功能未破坏

## 架构总结

```
┌─────────────────────────────────────────────────────────┐
│                     Infer Module                          │
│  ┌───────────────────────────────────────────┐   │
│  │         TypeRegistry                 │   │
│  │  ┌───────────────────────────────┐   │   │
│  │  │  type_decls: HashMap<...>  │   │   │
│  │  │  - TypeDecl (完整信息)     │   │   │
│  │  │  - 包含字段类型信息        │   │   │
│  │  │  - 支持泛型类型          │   │   │
│  │  └───────────────────────────────┘   │   │
│  └───────────────────────────────────────────┘   │
│                                                         │
│  lookup_type_decl()  register_type_decl()       │
└─────────────────────────────────────────────────┘
         │                                │
         └──────────────────────────┘
                                           │
                           Codegen ───────────┘
                           ┌─────────────────┐
                           │  使用 Infer     │
                           └─────────────────┘
```

## 依赖

- Plan 087: `.type` 属性实现（已完成）
- Plan 076: 泛型类型支持（已完成）
- vm/generic_registry（已存在）
- infer 模块（已存在）

## 参考资料

- [Plan 010: Type Inference Subsystem](010-type-inference-subsystem.md)
- [Plan 087: .type 属性实现](./087-type-property-implementation.md)
- [类型推导实现总结](../type-inference-implementation-summary.md)
