# Plan 090: 移除 Parser 对 Universe 的依赖

> **状态**: 🔜 待实施
> **优先级**: 中
> **依赖**: Plan 084 (TypeStore), Plan 089 (类型声明迁移)

## 概述

将 Parser 从依赖 Universe 迁移到使用 TypeStore + InferenceContext，最终移除 Parser 对 Universe 的依赖。

## 背景

### 当前状态

Parser 目前使用 `scope: Shared<Universe>` 作为符号表，依赖 Universe 提供的功能：

```rust
pub struct Parser<'a> {
    pub scope: Shared<Universe>,  // 依赖 Universe
    pub infer_ctx: InferenceContext,
    pub type_store: Arc<RwLock<TypeStore>>,
    // ...
}
```

### Universe 在 Parser 中的使用

| 功能 | Universe 方法 | 已有替代 | 状态 |
|------|--------------|---------|------|
| 符号定义 | `define()`, `define_rc()` | InferenceContext.bind_var() | ✅ 可替代 |
| 作用域管理 | `enter_scope()`, `exit_scope()` | InferenceContext.push/pop_scope() | ✅ 可替代 |
| 类型/函数/Spec 查找 | `lookup_meta()`, `lookup_type_meta()` | TypeStore + InferenceContext | ✅ 可替代 |
| 变量类型查找 | `lookup_ident_type()` | InferenceContext.lookup_type() | ✅ 可替代 |
| 符号存在检查 | `exists()` | InferenceContext + TypeStore | ✅ 可替代 |
| 模块路径追踪 | `cur_spot`, `enter_mod()` | - | ❌ 需要新实现 |
| Lambda ID 生成 | `gen_lambda_id()` | - | ❌ 需要新实现 |
| 导入处理 | `import()`, `register_spec()` | - | ❌ 需要新实现 |

## 目标

1. **主要目标**: 移除 Parser 对 Universe 的依赖
2. **架构目标**: TypeStore 作为类型信息单一数据源
3. **保持兼容**: 确保现有功能不受影响

## 设计方案

### 方案 1: 新建辅助结构（推荐）

创建小型、职责单一的辅助结构：

```rust
/// 模块路径追踪器
#[derive(Debug, Clone, Default)]
pub struct ModuleTracker {
    /// 当前模块路径
    pub current_module: Vec<String>,
    /// 模块栈（用于嵌套）
    module_stack: Vec<Vec<String>>,
}

impl ModuleTracker {
    pub fn enter_mod(&mut self, module: String) {
        self.current_module.push(module);
    }

    pub fn exit_mod(&mut self) {
        self.current_module.pop();
    }

    pub fn current_path(&self) -> String {
        self.current_module.join("::")
    }
}

/// Lambda ID 生成器
#[derive(Debug, Clone, Default)]
pub struct LambdaIdGenerator {
    counter: u64,
}

impl LambdaIdGenerator {
    pub fn gen_id(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }
}
```

### 方案 2: 扩展 InferenceContext

将模块追踪和 Lambda ID 生成添加到 InferenceContext：

```rust
pub struct InferenceContext {
    // 现有字段...
    pub type_store: Arc<RwLock<TypeStore>>,

    // 新增字段
    pub module_tracker: ModuleTracker,
    pub lambda_id_gen: LambdaIdGenerator,
}
```

### 最终 Parser 结构

```rust
pub struct Parser<'a> {
    // 移除: pub scope: Shared<Universe>,

    // 保留
    pub infer_ctx: InferenceContext,
    pub type_store: Arc<RwLock<TypeStore>>,
    pub type_registry: Option<SharedTypeRegistry>,  // REPL 支持

    // 新增
    pub module_tracker: ModuleTracker,
    pub lambda_id_gen: LambdaIdGenerator,
}
```

## 实施步骤

### Phase 1: 创建辅助结构

1. 创建 `ModuleTracker` 结构
2. 创建 `LambdaIdGenerator` 结构
3. 添加单元测试

**文件**: `crates/auto-lang/src/parser_helpers.rs` (新建)

### Phase 2: 更新 Parser 结构体

1. 添加 `module_tracker` 和 `lambda_id_gen` 字段
2. 更新所有构造函数
3. 保持 `scope` 字段但标记为 deprecated

### Phase 3: 迁移符号定义

1. `define()` → 使用 `infer_ctx.bind_var()` + `type_store.write()`
2. `define_rc()` → 同上
3. `define_alias()` → 添加到 InferenceContext

### Phase 4: 迁移符号查找

1. `lookup_meta()` → 使用 `type_store.read()` + `infer_ctx.lookup_meta()`
2. `lookup_type_meta()` → 使用 `type_store.read()`
3. `exists()` → 使用 `infer_ctx` 和 `type_store`

### Phase 5: 迁移模块追踪

1. `cur_spot` → `module_tracker.current_path()`
2. `enter_mod()` → `module_tracker.enter_mod()`
3. `reset_spot()` → `module_tracker` 方法

### Phase 6: 迁移 Lambda ID

1. `gen_lambda_id()` → `lambda_id_gen.gen_id()`

### Phase 7: 移除 Universe 依赖

1. 删除 `scope` 字段
2. 更新所有使用 `self.scope` 的代码
3. 清理 imports

### Phase 8: 清理和测试

1. 运行所有测试
2. 清理 deprecated 代码
3. 更新文档

## 成功标准

- ✅ Parser 不再依赖 Universe
- ✅ TypeStore 作为类型信息单一数据源
- ✅ 所有现有测试通过
- ✅ 无功能回归

## 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 导入功能损坏 | 中 | 高 | 保留 Universe 作为回退，渐进迁移 |
| REPL 类型持久化失效 | 低 | 中 | 保留 type_registry 字段 |
| 性能下降 | 低 | 低 | RwLock 已经优化 |

## 后续工作

- Plan 064: Database + ExecutionEngine 完全替代 Universe
- 移除 eval.rs 中的 Universe 依赖
- 移除 trans/*.rs 中的 Universe 依赖

## 参考资料

- [Plan 084: Unified TypeStore](./084-unified-type-context.md)
- [Plan 089: 类型声明存储迁移](./089-infer-module-type-declaration-storage.md)
- [Plan 064: Database + ExecutionEngine](./064-database-execution-engine.md)
