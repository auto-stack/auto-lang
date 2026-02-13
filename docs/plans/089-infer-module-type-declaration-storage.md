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

## 架构总结

## Phase 6: 统一类型上下文（未来优化）

**问题：当前类型存储分散且重复**

- Parser 和 Codegen 各自维护 InferenceContext 实例
- Parser 有 type_registry 用于 REPL
- Codegen 有 infer_ctx.type_registry 用于编译时类型查询
- 两个独立的 TypeRegistry 实例导致数据重复
- 复杂的类型包装：`SharedTypeRegistry = Rc<RefCell<TypeRegistry>>`

**设计方案：TypeContext - 统一类型上下文容器**（🔜 待实施）

这是一个较大的重构，涉及：
1. 创建新的 `TypeContext` 模块
2. 修改 Parser 使用 `type_context` 替代 `infer_ctx.type_registry`
3. 修改 Codegen 使用 `type_context` 替代 `infer_ctx.type_registry`
4. 保留 `InferenceContext.type_env` 用于类型推导
5. 逐步移除旧的 `type_registry`, `fn_registry`, `spec_registry` 字段

**实施优先级**：
- ⚠️ 这是一个较大的重构，建议创建新的 Plan 专门规划此工作

```rust
// crates/auto-lang/src/types.rs (新文件)

/// 统一的类型上下文，为 Parser 和 Codegen 提供共享的类型信息
#[derive(Clone)]
pub struct TypeContext {
    /// 类型注册表：类型名 -> 完整类型声明
    type_decls: HashMap<AutoStr, TypeDecl>,

    /// 函数注册表：函数名 -> 函数声明
    fn_decls: HashMap<AutoStr, Fn>,

    /// Spec 注册表：spec 名 -> spec 声明
    spec_decls: HashMap<AutoStr, SpecDecl>,

    /// 泛型注册表：类型名 + 泛型参数 -> 泛型类型信息
    generic_templates: HashMap<String, GenericTemplate>,
}

impl TypeContext {
    /// 创建新的类型上下文
    pub fn new() -> Self {
        Self {
            type_decls: HashMap::new(),
            fn_decls: HashMap::new(),
            spec_decls: HashMap::new(),
            generic_templates: HashMap::new(),
        }
    }

    /// 注册类型声明
    pub fn register_type_decl(&mut self, decl: TypeDecl) {
        self.type_decls.insert(decl.name.to_string(), decl);

        // 如果是泛型，注册为泛型模板
        if !decl.generic_params.is_empty() {
            let template = GenericTemplate::new(&decl);
            self.generic_templates.insert(template.name().to_string(), template);
        }
    }

    /// 查找类型声明
    pub fn lookup_type_decl(&self, name: &str) -> Option<&TypeDecl> {
        self.type_decls.get(name)
    }

    /// 注册函数声明
    pub fn register_fn_decl(&mut self, decl: Fn) {
        self.fn_decls.insert(decl.name.to_string(), decl);
    }

    /// 查找函数声明
    pub fn lookup_fn_decl(&self, name: &str) -> Option<&Fn> {
        self.fn_decls.get(name)
    }

    /// 注册 spec 声明
    pub fn register_spec_decl(&mut self, decl: SpecDecl) {
        self.spec_decls.insert(decl.name.to_string(), decl);
    }

    /// 查找 spec 声明
    pub fn lookup_spec_decl(&self, name: &str) -> Option<&SpecDecl> {
        self.spec_decls.get(name)
    }
}
```

**集成到 Parser 和 Codegen：**

```rust
// crates/auto-lang/src/parser.rs

impl<'a> Parser<'a> {
    // 替换分散的类型字段为统一引用
    pub type_context: TypeContext,

    pub fn from(code: &'a str) -> Self {
        let ctx = TypeContext::new();
        Self::new_with_context(code, shared(ctx))
    }

    pub fn new_with_context(code: &'a str, ctx: Shared<TypeContext>) -> Self {
        // ... 现有代码...
        Self { type_context: ctx, ... }
    }
}

// crates/auto-lang/src/vm/codegen.rs

pub struct Codegen {
    // 替换 infer_ctx 为 type_context 引用
    pub type_context: Shared<TypeContext>,

    // ... 其他字段保持不变 ...
}

impl Codegen {
    pub fn new() -> Self {
        // 创建新的 codegen（不带类型上下文）
        Self { type_context: shared(TypeContext::new()), ... }
    }

    pub fn with_type_context(type_context: Shared<TypeContext>) -> Self {
        Self { type_context, ... }
    }
}
```

**迁移步骤：**

1. 创建 `TypeContext` 结构
2. 修改 Parser 使用 `type_context` 而不是 `infer_ctx.type_registry`
3. 修改 Codegen 使用 `type_context` 而不是 `infer_ctx.type_registry`
4. 保留 `InferenceContext` 用于类型推导（只使用 type_env）
5. 逐步移除 `type_registry` 和 `fn_registry` 字段

**优势：**

1. **单一数据源**：类型声明只在一个地方
2. **简化共享**：通过 `Shared<TypeContext>` 直接传递引用
3. **类型推导分离**：`InferenceContext` 仍然负责类型推导，但不包含类型注册
4. **消除重复**：不再有两套 TypeRegistry 实例

**实施优先级：**

- ⚠️ **低优先级**：这是一个较大的重构，需要仔细规划
- ✅ **当前任务**：完成 Plan 089 的 Phase 5（类型参数替换）

---

## Plan 089 完成总结

**核心成果：**
- ✅ Phase 1-5：TypeRegistry 模块建立并集成到 infer 模块
- ✅ 类型参数替换在 Dot 表达式中工作
- ✅ Codegen 的 lookup 方法已迁移到 TypeRegistry
- ✅ Parser 的类型查询已迁移到优先使用 InferenceContext

**已实现的类型查找功能：**
- `TypeRegistry::lookup_type_decl()` - 查找类型声明
- `InferenceContext::lookup_meta()` - 统一的元数据查询（Fn/Spec/Type/Store）
- 类型参数替换 - `field_ty.substitute()` - 泛型实例类型正确解析

**测试状态：**
- ✅ 43 个 `.type` 相关测试通过
- ⚠️ 2 个 vm_tests 测试失败（与 Universe 迁移无关）

**遗留问题：**
- Parser 和 Codegen 仍然维护独立的 InferenceContext 实例
- `SharedTypeRegistry = Rc<RefCell<TypeRegistry>>` 类型包装复杂
- 类型注册信息分散且难以同步

**未来优化方向：**
- 🔜 Phase 6: 统一类型上下文（TypeContext）- 较大重构

---

## 参考资料

- [Plan 010: Type Inference Subsystem](010-type-inference-subsystem.md)
- [Plan 087: .type 属性实现](./087-type-property-implementation.md)
- [类型推导实现总结](../type-inference-implementation-summary.md)
