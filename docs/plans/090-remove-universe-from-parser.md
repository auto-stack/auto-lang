# Plan 090: 移除 Parser 对 Universe 的依赖

> **状态**: 🔄 进行中 (Phase 1-5 完成，Phase 7 部分完成)
> **优先级**: 中
> **依赖**: Plan 084 (TypeStore), Plan 089 (类型声明迁移)

## 概述

将 Parser 从依赖 Universe 迁移到使用 TypeStore + InferenceContext，最终移除 Parser 对 Universe 的依赖。

## 完成状态

| Phase | 状态 | 说明 |
|-------|------|------|
| Phase 1 | ✅ 完成 | 创建 `parser_helpers.rs`，实现 `ModuleTracker` 和 `LambdaIdGenerator` |
| Phase 2 | ✅ 完成 | 添加 `module_tracker` 和 `lambda_id_gen` 字段到 Parser |
| Phase 3 | ✅ 完成 | 迁移符号定义到 TypeStore（含 type_aliases 支持）|
| Phase 4 | ✅ 完成 | 迁移符号查找到 TypeStore + InferenceContext |
| Phase 5 | ✅ 完成 | 迁移模块追踪到 `ModuleTracker` |
| Phase 6 | ⏭️ 跳过 | Lambda 已被 closure 替代，`gen_lambda_id()` 仅在 deprecated 代码中使用 |
| Phase 7 | 🔄 进行中 | 移除 Universe 依赖 - 需要处理 44 处 `self.scope` 使用 |
| Phase 8 | ⏳ 待定 | 清理和测试 |

## 已完成的工作

### Phase 1: 创建辅助结构 (commit: 6e075db)

**文件**: `crates/auto-lang/src/parser_helpers.rs`

```rust
/// 模块路径追踪器 - 替代 Universe 的 cur_spot, enter_mod()
#[derive(Debug, Clone, Default)]
pub struct ModuleTracker {
    path_stack: Vec<String>,
}

/// Lambda ID 生成器 - 替代 Universe 的 gen_lambda_id()
#[derive(Debug, Clone, Default)]
pub struct LambdaIdGenerator {
    counter: u64,
}
```

- 4 个单元测试全部通过

### Phase 2: 更新 Parser 结构体 (commit: 4203184)

- 添加 `module_tracker: ModuleTracker` 字段
- 添加 `lambda_id_gen: LambdaIdGenerator` 字段
- 更新 3 个构造函数初始化新字段

### Phase 3: 迁移符号定义 (commit: bdb9e98)

- TypeStore 添加 `type_aliases` 字段
- 添加 `register_type_alias()`, `lookup_type_alias()`, `resolve_type_alias()` 方法
- `define_alias()` 同时注册到 TypeStore
- `define_rc()` 处理所有 Meta 类型

### Phase 4: 迁移符号查找 (commit: 4916c2e)

- `exists()` 优先使用 TypeStore + InferenceContext
- 检查 fn_decls, spec_decls, type_decls, type_aliases
- Universe 作为回退

### Phase 5: 迁移模块追踪 (commit: 557db22)

- 两处 import 代码使用 `module_tracker` 追踪模块路径
- 保存 `universe_spot` 和 `module_spot` 两种位置
- 保持 Universe 同步以向后兼容

## Phase 7 进度分析

### self.scope 使用统计 (44 处)

| 类别 | 数量 | 已有替代 | 状态 |
|------|------|---------|------|
| 符号定义 (`define`, `define_alias`) | 4 | ✅ TypeStore + infer_ctx | 可移除 |
| 符号查找 (`lookup_meta`, `exists`) | 6 | ✅ TypeStore + infer_ctx | 可移除 |
| 作用域管理 (`enter_scope`, `exit_scope`) | 4 | ✅ infer_ctx | 可移除 |
| 模块追踪 (`cur_spot`, `enter_mod`) | 8 | ✅ module_tracker | 可移除 |
| 函数作用域 (`enter_fn`) | 3 | ❌ | 需实现 |
| 导入 (`import`, `register_spec`) | 6 | ❌ | 需保留或重构 |
| 类型查找 (`find_type_for_name`) | 2 | ❌ | 需实现 |
| 名称列表 (`get_defined_names`) | 2 | ❌ | 需实现 |
| Lambda ID | 1 | ⚠️ deprecated | 可忽略 |
| Parser 创建 (传递 scope) | 2 | ❌ | 需重构 |

### 阻塞项

以下功能需要在 TypeStore/InferenceContext 中实现后才能完全移除 Universe：

1. **`enter_fn()`** - 进入函数作用域
   - 解决方案：在 InferenceContext 中添加 `enter_fn()` 方法

2. **`import()`** - 模块导入
   - 解决方案：重构 import 逻辑，直接操作 TypeStore

3. **`find_type_for_name()`** - 查找类型的父类型
   - 解决方案：在 TypeStore 中添加类型继承查询

4. **`get_defined_names()`** - 获取所有定义的名称（用于 LSP）
   - 解决方案：在 TypeStore 中添加 `list_all_names()` 方法

## 当前 Parser 结构

```rust
pub struct Parser<'a> {
    pub scope: Shared<Universe>,  // 保留，待移除
    pub infer_ctx: InferenceContext,
    pub type_store: Arc<RwLock<TypeStore>>,
    pub type_registry: Option<SharedTypeRegistry>,
    pub module_tracker: ModuleTracker,    // Plan 090 新增
    pub lambda_id_gen: LambdaIdGenerator, // Plan 090 新增
    // ...
}
```

## 下一步

1. 为阻塞项实现替代方案
2. 或者采用渐进式移除策略：
   - Phase 7a: 移除已有替代的简单用法
   - Phase 7b: 实现复杂功能的替代
   - Phase 7c: 完全移除 `scope` 字段

## 成功标准

- [ ] Parser 不再依赖 Universe
- [x] TypeStore 作为类型信息单一数据源
- [ ] 所有现有测试通过
- [ ] 无功能回归

## 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 导入功能损坏 | 中 | 高 | 保留 Universe 作为回退，渐进迁移 |
| REPL 类型持久化失效 | 低 | 中 | 保留 type_registry 字段 |
| LSP 功能受损 | 中 | 中 | 实现 `get_defined_names()` 替代 |
| 性能下降 | 低 | 低 | RwLock 已经优化 |

## 提交历史

- `6e075db` Plan 090 Phase 1: Create parser helper structures
- `4203184` Plan 090 Phase 2: Add module_tracker and lambda_id_gen to Parser
- `bdb9e98` Plan 090 Phase 3: Migrate symbol definition to TypeStore
- `4916c2e` Plan 090 Phase 4: Migrate symbol lookup to TypeStore
- `557db22` Plan 090 Phase 5: Migrate module tracking to ModuleTracker

## 参考资料

- [Plan 084: Unified TypeStore](./084-unified-type-context.md)
- [Plan 089: 类型声明存储迁移](./089-infer-module-type-declaration-storage.md)
- [Plan 064: Database + ExecutionEngine](./064-database-execution-engine.md)
