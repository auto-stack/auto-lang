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
| ~~trans/python.rs~~ | ~~5~~ | ~~P3~~ | ✅ 已移除 Universe 依赖 |
| ~~trans/javascript.rs~~ | ~~5~~ | ~~P3~~ | ✅ 已移除 Universe 依赖 |

## 完成状态

| Phase | 状态 | 说明 |
|-------|------|------|
| Phase 1 | ✅ 完成 | 移除公开 API，Evaluator 重定向到 AutoVM |
| Phase 2 | ✅ 完成 | 所有转译器已迁移（trans_c, trans_rust, trans_python, trans_javascript）|
| Phase 3 | ✅ **完成** | Parser scope 用法从 21 减少到 1（仅注释）|
| Phase 4 | ✅ 部分完成 | 入口点已简化 |
| Phase 5 | ✅ 完成 | config.rs 已迁移到 AutoVM（无 Universe 依赖）|
| Phase 6 | 🔄 **尝试中** | 尝试删除 universe.rs |

### Phase 3 完成状态 (提交: 490ec67)

**Parser scope 用法：21 → 1（仅注释代码）**

**已完成**:
- ✅ 添加 `db` 字段和 `set_database()` 方法
- ✅ 作用域管理迁移到 InferenceContext
- ✅ 移除所有 `define()` 系列的 Universe 写入
- ✅ 移除所有查询方法的 Universe 回退
- ✅ 添加基础类型直接识别（int, float, bool, str 等）
- ✅ `define_type_alias()` 迁移到 Database
- ✅ 动态类型（slice/array）直接返回，不存储

**关键突破**：
- 基础类型（int, float, bool 等）原本存储在 Universe.define_sys_types()
- 现在在 parse_ident_or_generic_type() 中直接识别
- 彻底消除了 Parser 对 Universe 的运行时依赖

### Phase 5 完成详情 (提交: e026fbd)

- ✅ 移除 `eval_config_with_vm()` 的 Universe 参数
- ✅ 移除 `AutoConfigReader.univ` 字段
- ✅ 移除 `AutoConfig` 的 Universe 相关方法
- ✅ config.rs 不再依赖 Universe

### Phase 6 无法完成的原因

**Universe 全局引用统计**: 212 处（24 个文件）

**主要阻碍**:
- eval.rs (59 处) - 老解释器
- interp.rs (17 处) - 老解释器
- parser.rs (11 处) - 查询方法回退
- lib.rs (19 处) - 入口点 API
- 其他文件 (100+ 处)

**完全删除 Universe 需要**:
1. 删除或重构 eval.rs 和 interp.rs
2. 增强 TypeStore 覆盖所有查询场景
3. 全面重构入口点 API

**结论**: 删除 universe.rs 需要更大规模的重构，超出 Plan 091 的范围。
1. **回退逻辑** - 保持向后兼容，可在未来删除
2. **作用域管理** - 需要保留或迁移到 InferenceContext 的 scope 栈
3. **专用注册** - 特殊用途，可按需迁移

**提交记录**:
- `82ccde5` - Remove deprecated lambda() method
- `c7d8b46` - Add optional db field to Parser
- `9b510f6` - Add define_symbol_location wrapper method
- `20ce1ff` - Add get_defined_names wrapper method
- `202d989` - Add find_type_for_name wrapper method

### Phase 1 进度 (commit: 15a8f3c)

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

### Phase 2: 转译器迁移到 Database (commit: b597599)

**已完成的迁移**:
- ✅ `trans_c()` 使用 `trans_c_with_session()` 内部
- ✅ `trans_rust()` 使用 `trans_rust_with_session()` 内部
- ✅ 添加 `trans_c_legacy()` 和 `trans_rust_legacy()` 作为回退
- ✅ `trans_python()` 移除 Universe 依赖 (PythonTrans 不需要 scope)
- ✅ `trans_javascript()` 移除 Universe 依赖 (JavaScriptTrans 不需要 scope)

### Phase 2 Original: 转译器迁移到 Database

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

**分析结果** (2025-02-14):

Parser 中 `self.scope` 的使用统计（24+ 处）:
```
define()           - 定义符号（函数/变量）
define_alias()     - 定义类型别名
exists()           - 检查符号是否存在
enter_scope()      - 进入新作用域
exit_scope()       - 退出作用域
lookup_meta()      - 查找符号元数据
lookup_type_meta() - 查找类型元数据
lookup_ident_type()- 查找标识符类型
gen_lambda_id()    - 生成 lambda ID
enter_fn()         - 进入函数作用域
find_type_for_name()- 查找名称对应的类型
get_defined_names()- 获取所有已定义名称
```

**现有替代设施**:

Database 已有:
- `get_scope()` / `insert_scope()` - Legacy Scope
- `get_symbol_table()` / `insert_symbol_table()` - 新 SymbolTable
- `type_info_store()` - 类型信息存储
- `get_type_alias()` / `insert_type_alias()` - 类型别名
- `get_spec()` / `insert_spec()` - Spec/trait
- `get_lambda_counter()` / `increment_lambda_counter()` - Lambda 计数器
- `get_cur_spot()` / `set_cur_spot()` - 当前作用域位置
- `get_code_pak()` / `insert_code_pak()` - 代码包

**迁移方案**:

**方案 A: 渐进式迁移（推荐）**
1. 为 Parser 添加 `db: Option<Arc<RwLock<Database>>>` 字段
2. 保留 `scope: Shared<Universe>` 作为回退（deprecated）
3. 逐个方法迁移到 Database API
4. 迁移完成后移除 `scope` 字段

**方案 B: 直接替换**
1. 一次性将 `scope: Shared<Universe>` 替换为 `db: Arc<RwLock<Database>>`
2. 更新所有 Parser 方法
3. 风险较高，但更彻底

**建议采用方案 A**，因为：
- Parser 是核心组件，影响面广
- 渐进式迁移可以分阶段测试
- 保持向后兼容性

**步骤**:
1. 为 Parser 添加 `db` 字段（可选）
2. 添加 `with_database()` 构造函数
3. 逐个方法迁移：
   - `gen_lambda_id()` → `db.increment_lambda_counter()`
   - `define()` → `db.insert_symbol_table()` / `db.get_scope_mut()`
   - `lookup_*()` → `db.get_symbol_table()` / `db.get_scope()`
4. 更新所有 Parser 创建点
5. 移除 `scope` 字段

**需替换的方法调用**:
- `self.scope.borrow().define()` → `self.db.write().insert_scope()` 或 `insert_symbol_table()`
- `self.scope.borrow().lookup()` → `self.db.read().get_scope()` 或 `get_symbol_table()`
- `self.scope.borrow().enter_scope()` → `self.db.write().set_cur_spot()`
- `self.scope.borrow().exit_scope()` → `self.db.write().set_cur_spot()`
- `self.scope.borrow().gen_lambda_id()` → `self.db.write().increment_lambda_counter()`

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

## 最终结论

### Plan 091 成果

**Parser scope 用法**: 21 → 11 (减少 48%)

**已完成**:
- ✅ Phase 1: 移除公开 Universe API
- ✅ Phase 2: 所有转译器不再依赖 Universe
- ✅ Phase 3: Parser 核心功能迁移到 TypeStore + InferenceContext
- ✅ Phase 5: config.rs 完全移除 Universe

**无法完成**:
- ❌ Phase 6: 删除 universe.rs（需要更大规模重构）

### 剩余工作

如需完全删除 Universe，需要：

1. **重构 eval.rs/interp.rs** (76 处引用)
   - 考虑完全删除或标记为 legacy

2. **增强 TypeStore**
   - 支持泛型参数
   - 覆盖所有 Universe 查询场景

3. **重构入口点 API** (19 处引用)
   - 统一使用 CompileSession

### 建议

接受当前状态作为 Plan 091 的最终成果：
- Universe 保留但标记为 "deprecated but required"
- 新代码避免使用 Universe
- 未来在单独的计划中处理 eval.rs/interp.rs

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
