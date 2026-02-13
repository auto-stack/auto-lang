# Plan 084: 统一 Type 信息集合体

## 概述

建立统一的 Type 信息管理系统，将分散在 Parser、Codegen 和 InferenceContext 中的类型声明整合到一个集中的、共享的容器中。

## 当前问题

### 类型信息分散在三个位置

```
Parser:
├── infer_ctx.type_env         (变量类型推导)
├── infer_ctx.type_registry  (REPL 支持) ← 未被 infer 模块使用
├── infer_ctx.fn_registry      (函数声明)    ← 未被 infer 模块使用
├── infer_ctx.spec_registry   (spec 声明)   ← 未被 infer 模块使用
└── type_registry (Option<SharedTypeRegistry>) ← 类型包装复杂

Codegen:
├── infer_ctx.type_env         (变量类型推导) ← 实际使用
├── infer_ctx.type_registry  (编译时类型查询) ← 实际使用
├── infer_ctx.fn_registry      (函数声明)    ← 实际使用
├── infer_ctx.spec_registry   (spec 声明)   ← 实际使用
└── generic_registry           (泛型信息)      ← 实际使用
```

**问题分析：**

1. **数据重复**：Parser 和 Codegen 各自维护 InferenceContext 实例
2. **同步困难**：需要通过 `SharedTypeRegistry` 的 RefCell/Rc/Option 包装传递
3. **职责不清**：`infer_ctx.type_env` 和 `infer_ctx.type_registry` 的用途不明确
4. **类型包装复杂**：`SharedTypeRegistry = Rc<RefCell<TypeRegistry>>` 导致访问需要多重解包

## 设计目标

1. 建立统一的 `TypeStore` 作为单一的 Type 信息集合体
2. 提供清晰的类型查找 API
3. 简化类型注册和查询
4. 支持类型推导（保留 type_env）
5. 为 Parser 和 Codegen 提供共享的类型信息访问

## 设计方案

### 方案 1: TypeStore 结构

```rust
// crates/auto-lang/src/types.rs (新建)
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::ast::{Type, TypeDecl, Fn, SpecDecl, GenericParam, GenericTemplate};
use crate::infer::{InferenceContext, TypeRegistry};

/// 统一的 Type 信息存储
#[derive(Clone, Debug)]
pub struct TypeStore {
    /// 类型注册表：类型名 -> TypeDecl
    pub type_decls: HashMap<AutoStr, Rc<TypeDecl>>,

    /// 函数注册表：函数名 -> Fn
    pub fn_decls: HashMap<AutoStr, Rc<Fn>>,

    /// Spec 注册表：spec 名 -> SpecDecl
    pub spec_decls: HashMap<AutoStr, Rc<SpecDecl>>,

    /// 泛型信息表：类型名 -> 泛型模板
    pub generic_templates: HashMap<String, Rc<GenericTemplate>>,
}

impl TypeStore {
    /// 创建新的类型存储
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
        let decl = Rc::new(decl);
        self.type_decls.insert(decl.name.to_string(), decl);

        // 如果是泛型，注册为泛型模板
        if !decl.generic_params.is_empty() {
            let template = GenericTemplate::new(&decl);
            self.generic_templates.insert(template.name().to_string(), template);
        }
    }

    /// 注册函数声明
    pub fn register_fn_decl(&mut self, decl: Fn) {
        let decl = Rc::new(decl);
        self.fn_decls.insert(decl.name.to_string(), decl);
    }

    /// 注册 spec 声明
    pub fn register_spec_decl(&mut self, decl: SpecDecl) {
        let decl = Rc::new(decl);
        self.spec_decls.insert(decl.name.to_string(), decl);
    }

    /// 查找类型声明
    pub fn lookup_type_decl(&self, name: &str) -> Option<&TypeDecl> {
        self.type_decls.get(name)
    }

    /// 查找函数声明
    pub fn lookup_fn_decl(&self, name: &str) -> Option<&Fn> {
        self.fn_decls.get(name)
    }

    /// 查找 spec 声明
    pub fn lookup_spec_decl(&self, name: &str) -> Option<&SpecDecl> {
        self.spec_decls.get(name)
    }

    /// 创建泛型实例的类型
    pub fn create_generic_instance(&self, type_name: &str, type_args: &[Type]) -> Type {
        if let Some(template) = self.generic_templates.get(type_name) {
            // 替换泛型参数
            let field_ty = field_ty.substitute(&template.param_names, type_args);
            Type::GenericInstance(Box::new(field_ty))
        } else {
            Type::Unknown
        }
    }
}
```

### 方案 2: 集成到 Parser 和 Codegen

```rust
// crates/auto-lang/src/parser.rs
use crate::types::TypeStore;
use std::sync::Arc;

impl<'a> Parser<'a> {
    pub type_store: Arc<TypeStore>,

    pub fn from(code: &'a str) -> Self {
        // 创建新的类型存储
        let type_store = Arc::new(TypeStore::new());

        // 创建 InferenceContext，使用共享的类型存储
        let mut infer_ctx = InferenceContext::new();
        // 注入类型存储引用到 infer_ctx
        // TODO: 实现注入机制

        Self {
            type_store,
            infer_ctx,
            ...现有字段...
        }
    }

    pub fn new_with_type_store(code: &'a str, type_store: Arc<TypeStore>) -> Self {
        Self {
            type_store,
            ...现有字段...
        }
    }
}

// crates/auto-lang/src/vm/codegen.rs
use crate::types::TypeStore;
use std::sync::Arc;

pub struct Codegen {
    pub type_store: Arc<TypeStore>,

    // ... 其他字段保持不变 ...
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            type_store: Arc::new(TypeStore::new()),
            ...其他字段...
        }
    }

    pub fn with_type_store(type_store: Arc<TypeStore>) -> Self {
        Self {
            type_store,
            ...其他字段...
        }
    }
}
```

### 方案 3: InferenceContext 集成 TypeStore

```rust
// crates/auto-lang/src/infer/context.rs
use crate::types::TypeStore;

pub struct InferenceContext {
    // 类型推导相关的类型环境
    pub type_env: HashMap<crate::ast::Name, crate::ast::Type>,

    // 类型存储引用
    pub type_store: Arc<TypeStore>,
}

impl InferenceContext {
    pub fn new() -> Self {
        Self {
            type_env: HashMap::new(),
            ...现有字段...
            // 使用空类型的类型存储
            type_store: Arc::new(TypeStore::new()),
        }
    }

    pub fn with_type_store(type_store: Arc<TypeStore>) -> Self {
        Self {
            type_env: HashMap::new(),
            ...现有字段...
            type_store: type_store.clone(),
        }
    }
}
```

### 方案 4: 迁移路径

1. **Parser 路径**：
   - 保持 `InferenceContext.type_env` 用于类型推导
   - 通过 `type_store` 访问类型声明和函数签名
   - 移除 `infer_ctx.type_registry`, `infer_ctx.fn_registry`, `infer_ctx.spec_registry`

2. **Codegen 路径**：
   - 保持 `InferenceContext.type_env` 用于变量类型推导
   - 通过 `type_store` 访问类型声明和函数签名
   - 移除 `infer_ctx.type_registry`, `infer_ctx.fn_registry`, `infer_ctx.spec_registry`
   - 移除 `generic_registry`（泛型实例可以在 `type_store` 中创建）

3. **SharedTypeRegistry 路径**：
   - 移除 REPL 专用类型存储
   - 类型声明通过 `TypeStore` 统一管理
   - 如果需要 REPL 特定功能，添加回退机制

## 实施步骤

### Phase 1: 创建 TypeStore 结构和模块 ✅ 完成

**文件：** `crates/auto-lang/src/types.rs`

**任务：**
1. ✅ 定义 `TypeStore` 结构体
2. ✅ 实现基本方法（new, register, lookup）
3. ✅ 添加泛型实例创建方法

**实施总结：**
- 创建了完整的 `types.rs` 模块文件
- 实现了 `TypeStore` 结构体，包含：
  - `type_decls: HashMap<AutoStr, Rc<TypeDecl>>` - 类型声明注册表
  - `fn_decls: HashMap<Name, Rc<Fn>>` - 函数声明注册表
  - `spec_decls: HashMap<AutoStr, Rc<SpecDecl>>` - Spec 声明注册表
  - `generic_templates: HashMap<String, Rc<GenericTemplate>>` - 泛型模板注册表
- 实现了 `GenericTemplate` 和 `GenericParamType` 结构
- 提供了完整的公共 API：
  - `new()` - 创建新的 TypeStore
  - `register_type_decl()` - 注册类型声明
  - `register_fn_decl()` - 注册函数声明
  - `register_spec_decl()` - 注册 spec 声明
  - `register_generic_template()` - 注册泛型模板
  - `lookup_type_decl()` - 查找类型声明（两个版本：AutoStr 和 String 参数）
  - `get_template()` - 获取泛型模板
  - `create_generic_instance()` - 创建泛型实例
  - `list_types()` - 列出所有类型声明
  - `list_functions()` - 列出所有函数声明
  - `list_specs()` - 列出所有 spec 声明
  - `list_generic_templates()` - 列出所有泛型模板
- 添加了 4 个单元测试验证基本功能
- 在 `lib.rs` 中添加了 `pub mod types;` 导出

### Phase 2: 集成到 Parser ✅ 完成

**文件：** `crates/auto-lang/src/parser.rs`

**修改：**
1. ✅ 添加 `type_store: Arc<types::TypeStore>` 字段到 Parser
2. ✅ 添加 imports: `use crate::types;` 和 `use std::sync::Arc;`
3. ✅ 修改 `from()` 创建 TypeStore 并使用
4. ✅ 添加 `new_with_type_store()` 构造函数，允许 Parser 和 Codegen 共享同一个 TypeStore 实例
5. ✅ 修改 `define()` 函数，为 Meta::Fn, Meta::Spec, Meta::Type/Enum 添加 `type_store` 注册调用

**实施总结：**
- Parser 现在具有统一的 TypeStore 用于类型管理
- 类型、函数、Spec 声明都会被注册到 type_store
- InferenceContext 继续用于类型推导，但类型查询可通过 type_store 进行
- 为 Parser 和 Codegen 提供共享类型存储的基础架构已建立

**文件：** `crates/auto-lang/src/parser.rs`

**修改：**
1. 添加 `type_store: Arc<TypeStore>` 字段
2. 修改 `from()` 创建 `TypeStore`
3. 添加 `new_with_type_store()` 构造函数

**任务：**
1. 在解析类型声明时注册到 `type_store`
2. 在 `infer_type_expr()` 中使用 `type_store.lookup_type_decl()`

### Phase 3: 集成到 Codegen ✅ 完成

**文件：** `crates/auto-lang/src/vm/codegen.rs`

**修改：**
1. ✅ 添加 `type_store: Arc<types::TypeStore>` 字段
2. ✅ 添加 imports: `use crate::types;` 和 `use std::sync::Arc;`
3. ✅ 修改 `new()` 构造函数初始化 type_store
4. ✅ 添加 `new_with_type_store()` 构造函数，允许 Parser 和 Codegen 共享同一个 TypeStore 实例

**实施总结：**
- Codegen 现在具有统一的 TypeStore 用于类型管理
- 可以通过 `new_with_type_store()` 与 Parser 共享同一个 TypeStore 实例
- 为后续统一类型查询 API 奠定基础

### Phase 4: InferenceContext 集成 TypeStore ✅ 完成

**文件：** `crates/auto-lang/src/infer/context.rs`

**修改：**
1. ✅ 添加 `type_store: Option<Arc<types::TypeStore>>` 字段
2. ✅ 添加 imports: `use crate::types;` 和 `use std::sync::Arc;`
3. ✅ 修改 `new()` 和 `with_database()` 初始化 type_store 为 None
4. ✅ 添加 `with_type_store()` 构造函数
5. ✅ 添加 `set_type_store()` 方法

**实施总结：**
- InferenceContext 现在可以可选地持有 TypeStore 引用
- 通过 `with_type_store()` 或 `set_type_store()` 可以设置共享的 TypeStore
- 保持了与现有代码的向后兼容性（type_store 为 Option）
- 为后续统一类型查询 API 奠定基础

## 测试策略

### 单元测试
- 所有 43 个 `.type` 相关测试必须保持通过
- 测试新的泛型实例创建功能
- 测试类型查找 API（lookup_type_decl, lookup_fn_decl）

### 向后兼容
- 如果 `type_store` 不满足某些需求，可以添加回退机制
- 保留 `SharedTypeRegistry` 用于 REPL 场景

## 风险与注意事项

**风险：**
- 大规模重构，涉及多个核心组件
- 可能引入新的 bug
- 需要仔细测试所有场景

**注意事项：**
- 保持 `InferenceContext.type_env` 清晰职责：仅用于类型推导
- `type_store` 使用 `Arc` 保证线程安全
- 泛型实例创建逻辑需要完整和正确

## 进度跟踪

| Phase | 状态 | 说明 |
|-------|------|------|
| Phase 1 | ✅ 完成 | 创建 types.rs 模块并实现 TypeStore |
| Phase 2 | ✅ 完成 | 集成 TypeStore 到 Parser |
| Phase 3 | ✅ 完成 | 集成 TypeStore 到 Codegen |
| Phase 4 | ✅ 完成 | InferenceContext 集成 TypeStore |
## 完成状态

**Phase 1 实施完成** (2026-02-13)

- ✅ 创建了 `crates/auto-lang/src/types.rs` 模块文件
- ✅ 实现了 `TypeStore` 结构体
- ✅ 实现了 `GenericTemplate` 和 `GenericParamType` 类型
- ✅ 实现了完整的公共 API
- ✅ 添加了单元测试
- ✅ 在 `lib.rs` 中导出了 types 模块
- ✅ 代码编译成功，无 types.rs 相关错误

**Phase 2 实施完成** (2026-02-13)

- ✅ 在 Parser 中添加了 `type_store: Arc<types::TypeStore>` 字段
- ✅ 添加了必要的 imports
- ✅ 修改了 `from()` 函数创建 TypeStore
- ✅ 添加了 `new_with_type_store()` 构造函数
- ✅ 修改了 `define()` 函数注册声明到 type_store
- ✅ 代码编译成功

**Phase 3 实施完成** (2026-02-13)

- ✅ 在 Codegen 中添加了 `type_store: Arc<types::TypeStore>` 字段
- ✅ 添加了必要的 imports
- ✅ 修改了 `new()` 构造函数初始化 type_store
- ✅ 添加了 `new_with_type_store()` 构造函数
- ✅ 代码编译成功

**Phase 4 实施完成** (2026-02-13)

- ✅ 在 InferenceContext 中添加了 `type_store: Option<Arc<types::TypeStore>>` 字段
- ✅ 添加了必要的 imports
- ✅ 修改了 `new()` 和 `with_database()` 初始化 type_store 为 None
- ✅ 添加了 `with_type_store()` 构造函数
- ✅ 添加了 `set_type_store()` 方法
- ✅ 代码编译成功

## Plan 084 完成总结

**所有 4 个 Phase 已完成** ✅

Plan 084 成功创建了统一的 TypeStore 系统，为 AutoLang 提供了集中化的类型信息管理：

1. **TypeStore 模块** - 集中存储类型、函数、Spec 声明和泛型模板
2. **Parser 集成** - 解析时自动注册声明到 TypeStore
3. **Codegen 集成** - 编译时可共享 TypeStore 实例
4. **InferenceContext 集成** - 类型推导时可访问 TypeStore

**架构优势：**
- 单一数据源：所有类型信息存储在 TypeStore 中
- 共享访问：通过 `Arc<TypeStore>` 实现跨组件共享
- 向后兼容：保留现有注册表作为回退

**后续工作：**
- 将类型查询统一到 TypeStore API
- 移除冗余的类型注册表（type_registry, fn_registry, spec_registry）
- 实现完整的类型同步机制

## 参考资料

- [Plan 089: 类型声明存储迁移到 Infer 模块](./089-infer-module-type-declaration-storage.md)
- [类型推导实现总结](../type-inference-implementation-summary.md)
- [Plan 085: 泛型类型支持](./085-generic-types-support.md)
