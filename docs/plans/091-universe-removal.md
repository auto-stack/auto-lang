# Plan 091: 完全移除 Universe

> **状态**: 🔄 进行中
> **优先级**: 高
> **依赖**: Plan 084 (TypeStore), Plan 085 (Auto-use), Plan 090 (Parser 重构), Plan 066 (Database)

## 概述

完全移除 `Universe` 类型，将其功能迁移到 `TypeStore` + `InferenceContext` + `Database` + `AutoCache` 组合架构。

## 背景

### 当前 Universe 的职责

```
Universe 当前承担的职责:
├── 符号表管理 (scope/sid)
├── 类型存储 (type_decls, fn_decls, spec_decls)
├── 作用域管理 (enter_scope, exit_scope, enter_fn)
├── 模块路径追踪 (cur_spot, enter_mod)
├── 模块导入 (import, register_spec)
└── 运行时值存储 (values)
```

### 替代方案状态

| 职责 | 替代方案 | 状态 |
|------|----------|------|
| 符号表管理 | InferenceContext.type_env | ✅ Plan 090 完成 |
| 类型存储 | TypeStore | ✅ Plan 084/090 完成 |
| 作用域管理 | InferenceContext | ✅ Plan 090 完成 |
| 模块路径追踪 | ModuleTracker | ✅ Plan 090 完成 |
| 模块导入 | CompileSession + AutoCache | ✅ Plan 085 完成 |
| 运行时值存储 | Database + ExecutionEngine | 🟡 Plan 064 进行中 |

## 依赖分析

### Universe 使用统计

| 文件 | 引用数 | 优先级 | 说明 |
|------|--------|--------|------|
| eval.rs | 59 | P0 | 老解释器，考虑删除 |
| parser.rs | 24 | P1 | `scope` 字段待移除 |
| lib.rs | 21 | P1 | 入口点 API |
| trans/rust.rs | 18 | P2 | 已有 Database 支持 |
| interp.rs | 17 | P0 | 老解释器，考虑删除 |
| trans/c.rs | 15 | P2 | 已有 Database 支持 |
| config.rs | 9 | P3 | 配置解析 |
| trans/python.rs | 5 | P3 | 需迁移 |
| trans/javascript.rs | 5 | P3 | 需迁移 |

## 完成状态

| Phase | 状态 | 说明 |
|-------|------|------|
| Phase 1 | ✅ 部分完成 | 移除公开 API，保留内部依赖 |
| Phase 2 | ⏳ 待定 | 转译器迁移到 Database |
| Phase 3 | ⏳ 待定 | 移除 Parser.scope 字段 |
| Phase 4 | ✅ 部分完成 | 入口点已简化 |
| Phase 5 | ⏳ 待定 | config.rs 迁移 |
| Phase 6 | ⏳ 待定 | 删除 universe.rs |

### Phase 1 进度 (commit: pending)

**已完成的清理**:
- ✅ 移除 `run_with_errors()` - 已删除
- ✅ 移除 `interpret()` - 已删除
- ✅ 移除 `interpret_with_scope()` - 已删除
- ✅ 移除 `interpret_file()` - 已删除
- ✅ 移除 `eval_template()` - 已删除
- ✅ 移除 `eval_config()` - 已删除
- ✅ 移除 `eval_config_with_scope()` - 已删除
- ✅ `run_with_scope()` 简化为使用 AutoVM
- ✅ `ExecutionEngine::Evaluator` 重定向到 AutoVM
- ✅ 删除 `config_tests.rs` 和 `template_tests.rs`
- ✅ 注释掉 `vm_tests.rs` 中使用 interpret() 的测试

**保留的内部依赖** (待后续处理):
- `atom.rs` 仍使用 `Interpreter` (AtomReader)
- `vm/*.rs` 仍使用 `Evaler` (VM native functions)
- `universe.rs` 仍有 `evaluator_ptr` (Universe-evaluator binding)

## 实施阶段

### Phase 1: eval.rs 和 interp.rs 决策

**目标**: 确定老解释器的命运

**选项 A: 完全删除**
- 前提: AutoVM 已能满足所有执行需求
- 优点: 最干净，减少维护负担
- 风险: 可能破坏依赖 eval.rs 的代码

**选项 B: 保留为遗留**
- 标记 `#[deprecated]`
- 移到 `legacy/` 目录
- 保持最小维护

**验证步骤**:
1. 检查 eval.rs/interp.rs 的调用者
2. 确认 AutoVM 功能覆盖度
3. 运行所有测试确保无依赖

### Phase 2: 转译器迁移到 Database

**目标**: 让 a2c, a2r, a2py, a2js 完全使用 Database

**现状**: trans/c.rs 和 trans/rust.rs 已有混合架构
```rust
scope: Option<Shared<Universe>>,      // 旧（已弃用）
db: Option<Arc<RwLock<Database>>>,    // 新（Phase 066）
```

**步骤**:
1. 为 Database 添加类型查找方法
2. 修改入口点使用 `with_database()`
3. 移除 `scope` 字段
4. 更新 python.rs 和 javascript.rs 使用相同模式

### Phase 3: 移除 Parser.scope 字段

**目标**: 完全移除 Parser 对 Universe 的依赖

**步骤**:
1. 将所有 `self.scope` 调用替换为 TypeStore/InferenceContext
2. 更新 Parser 构造函数，移除 scope 参数
3. 更新所有 Parser 创建点
4. 移除 `scope: Shared<Universe>` 字段

**需替换的方法调用**:
- `self.scope.borrow().define()` → `self.type_store.write().register_*()`
- `self.scope.borrow().lookup()` → `self.infer_ctx.lookup_type()`
- `self.scope.borrow().enter_scope()` → `self.infer_ctx.push_scope()`
- `self.scope.borrow().exit_scope()` → `self.infer_ctx.pop_scope()`

### Phase 4: 入口点重构

**目标**: 更新 lib.rs 的公开 API

**当前 API** (使用 Universe):
```rust
pub fn run(code: &str) -> AutoResult<String>
pub fn run_file(path: &Path) -> AutoResult<String>
```

**新 API** (使用 CompileSession):
```rust
pub fn run_with_session(session: &mut CompileSession, code: &str) -> AutoResult<String>
pub fn run(code: &str) -> AutoResult<String>  // 内部创建临时 session
```

**步骤**:
1. 将 `run()` 改为使用 `CompileSession`
2. 移除 Universe 相关的公开类型
3. 更新文档和示例

### Phase 5: config.rs 迁移

**目标**: 配置解析不依赖 Universe

**方案**:
- 使用 TypeStore 存储配置类型
- 或直接使用 AutoVM 执行配置

### Phase 6: 清理 universe.rs

**目标**: 删除 universe.rs 文件

**步骤**:
1. 确认所有引用已移除
2. 删除 `crates/auto-lang/src/universe.rs`
3. 从 `lib.rs` 移除 `mod universe;`
4. 更新 Cargo.toml（如有相关 feature）

## 成功标准

- [ ] eval.rs 和 interp.rs 已删除或标记 deprecated
- [ ] 所有转译器使用 Database（无 Universe 依赖）
- [ ] Parser 无 `scope` 字段
- [ ] lib.rs 入口点不使用 Universe
- [ ] config.rs 不使用 Universe
- [ ] `universe.rs` 文件已删除
- [ ] 所有测试通过

## 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| eval.rs 有未知调用者 | 中 | 高 | 全局搜索验证 |
| 转译器类型查找不完整 | 中 | 中 | 逐步迁移，充分测试 |
| 性能回退 | 低 | 中 | 基准测试对比 |
| API 破坏性变更 | 高 | 中 | 提供迁移指南 |

## 里程碑

| 里程碑 | 目标 | 预计工作量 |
|--------|------|------------|
| M1 | Phase 1 完成（eval/interp 决策）| 分析 + 决策 |
| M2 | Phase 2 完成（转译器迁移）| 代码修改 + 测试 |
| M3 | Phase 3 完成（Parser 清理）| 代码修改 + 测试 |
| M4 | Phase 4-5 完成（入口点 + config）| 代码修改 + 测试 |
| M5 | Phase 6 完成（删除 universe.rs）| 最终验证 |

## 相关计划

- [Plan 064: Database + ExecutionEngine](./064-database-execution-engine.md)
- [Plan 066: Transpiler Database Integration](./066-transpiler-database.md)
- [Plan 084: Unified TypeStore](./084-unified-type-context.md)
- [Plan 085: Auto-use with AIE + AutoCache](./085-auto-use.md)
- [Plan 090: Remove Universe from Parser](./090-remove-universe-from-parser.md)
