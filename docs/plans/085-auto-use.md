# Plan 085: 基于 AIE + AutoCache 的 Use 语句处理

> **状态**: 🔜 待实施
> **优先级**: 高
> **依赖**: Plan 063 (AIE), Plan 064 (Database), Plan 090 (Parser 重构)

## 概述

使用 AIE（Auto Incremental Engine）框架和 AutoCache 替代 Parser 中的 `Universe.import()` 功能，实现高效的增量式模块依赖处理。

## 背景

### 当前问题

Parser 中的 `use` 语句处理效率极低：

```
当前流程：
Source → Parser → 遇到 use → Universe.import() → 加载文件 → 解析 → 合并符号
                          ↑
                          每次都重新做，无缓存，无增量
```

**问题：**
1. 每次 `use` 都重新加载和解析依赖文件
2. 无缓存机制，重复工作
3. 无增量更新，修改一个文件导致全部重编译
4. Parser 职责过重，承担了模块管理的功能

### 已有基础设施

- **AIE 框架** (Plan 063) - 增量编译引擎
- **Database** (Plan 064) - 编译时数据存储
- **TypeStore** (Plan 084/090) - 统一类型存储
- **CompileSession** - 持久化编译会话

## 目标

1. **高效** - 利用缓存避免重复解析
2. **增量** - 只重编译变更的模块
3. **解耦** - Parser 不再负责模块导入
4. **统一** - TypeStore 作为符号查询的唯一入口

## 设计方案

### 新的处理流程

```
理想流程：
┌─────────────────────────────────────────────────────────────┐
│                    CompileSession                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 1. 预处理：扫描 use 语句                               │   │
│  │    use std.io → { module: "std.io", items: ["*"] }   │   │
│  └─────────────────────────────────────────────────────┘   │
│                           ↓                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 2. 检查 AutoCache                                     │   │
│  │    - 命中：直接加载 TypeStore                         │   │
│  │    - 未命中：解析并缓存                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                           ↓                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 3. 合并 TypeStore                                     │   │
│  │    session.type_store.merge(module_type_store)       │   │
│  └─────────────────────────────────────────────────────┘   │
│                           ↓                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 4. Parser 解析                                        │   │
│  │    - 从 session.type_store 查询符号                   │   │
│  │    - 不再调用 Universe.import()                       │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 架构变更

```
Before:
┌──────────┐     use      ┌───────────┐
│  Parser  │ ────────────→ │ Universe  │
└──────────┘   import()    └───────────┘

After:
┌──────────┐              ┌────────────────┐
│  Parser  │ ←─────────── │ CompileSession │
└──────────┘  type_store  └────────────────┘
                               │
                               ↓
                         ┌───────────┐
                         │ AutoCache │
                         └───────────┘
```

## 实施步骤

### Phase 1: Use 语句扫描器

创建独立的 use 语句扫描器，用于预处理阶段：

```rust
// src/use_scanner.rs

/// Use 语句信息
#[derive(Debug, Clone)]
pub struct UseStatement {
    pub module: String,      // "std.io"
    pub items: Vec<String>,  // ["say", "read"]
    pub is_wildcard: bool,   // use std.io.*
    pub alias: Option<String>, // use std.io as io
}

/// 扫描源码中的所有 use 语句
pub fn scan_use_statements(source: &str) -> Vec<UseStatement> {
    // 快速正则匹配，不完整解析
    // 只提取 use 后面的模块路径和导入项
}
```

**文件**: `crates/auto-lang/src/use_scanner.rs`

### Phase 2: TypeStore 合并功能

为 TypeStore 添加合并功能：

```rust
impl TypeStore {
    /// 合并另一个 TypeStore 的内容
    pub fn merge(&mut self, other: &TypeStore) {
        // 合并类型声明
        for (name, decl) in &other.type_decls {
            self.type_decls.insert(name.clone(), decl.clone());
        }

        // 合并函数声明
        for (name, fn_decl) in &other.fn_decls {
            self.fn_decls.insert(name.clone(), fn_decl.clone());
        }

        // 合并 spec 声明
        for (name, spec_decl) in &other.spec_decls {
            self.spec_decls.insert(name.clone(), spec_decl.clone());
        }

        // 合并泛型模板
        for (name, template) in &other.generic_templates {
            self.generic_templates.insert(name.clone(), template.clone());
        }
    }

    /// 选择性导入符号
    pub fn import_items(&mut self, other: &TypeStore, items: &[String]) {
        for item in items {
            if let Some(decl) = other.type_decls.get(item) {
                self.type_decls.insert(item.into(), decl.clone());
            }
            if let Some(fn_decl) = other.fn_decls.get(&Name::from(item)) {
                self.fn_decls.insert(Name::from(item), fn_decl.clone());
            }
        }
    }
}
```

### Phase 3: CompileSession 扩展

扩展 CompileSession 支持 use 语句处理：

```rust
impl CompileSession {
    /// 预处理 use 语句
    pub fn resolve_uses(&mut self, source: &str) -> Result<()> {
        let use_statements = scan_use_statements(source);

        for use_stmt in use_statements {
            self.load_module(&use_stmt)?;
        }

        Ok(())
    }

    /// 加载模块（优先从缓存）
    fn load_module(&mut self, use_stmt: &UseStatement) -> Result<()> {
        // 1. 检查 AutoCache
        if let Some(cached) = self.auto_cache.get(&use_stmt.module) {
            // 2a. 命中缓存 - 验证有效性
            if cached.is_valid() {
                if use_stmt.is_wildcard {
                    self.type_store.write().unwrap().merge(&cached.type_store);
                } else {
                    self.type_store.write().unwrap().import_items(
                        &cached.type_store,
                        &use_stmt.items
                    );
                }
                return Ok(());
            }
        }

        // 2b. 未命中或失效 - 解析模块
        let module_type_store = self.parse_module(&use_stmt.module)?;

        // 3. 存入缓存
        self.auto_cache.store(&use_stmt.module, &module_type_store);

        // 4. 导入符号
        if use_stmt.is_wildcard {
            self.type_store.write().unwrap().merge(&module_type_store);
        } else {
            self.type_store.write().unwrap().import_items(
                &module_type_store,
                &use_stmt.items
            );
        }

        Ok(())
    }
}
```

### Phase 4: 移除 Parser 中的 import

1. 删除 `parse_use_stmt()` 中的 `Universe.import()` 调用
2. 修改 Parser 只从 TypeStore 查询符号
3. 在调用 Parser 前由 CompileSession 完成 use 预处理

### Phase 5: AutoCache 模块缓存

实现模块级别的缓存：

```rust
/// 模块缓存条目
pub struct ModuleCache {
    pub module_path: String,
    pub type_store: TypeStore,
    pub file_hash: u64,      // 源文件哈希
    pub interface_hash: u64, // 接口哈希（熔断）
    pub dependencies: Vec<String>, // 依赖的其他模块
    pub timestamp: SystemTime,
}

impl ModuleCache {
    /// 检查缓存是否仍然有效
    pub fn is_valid(&self) -> bool {
        // 1. 检查源文件是否修改
        // 2. 检查依赖是否修改
        // 3. 使用接口哈希进行熔断判断
    }
}
```

## API 变更

### Before

```rust
// Parser 中直接处理 use
let parser = Parser::from(source);
let ast = parser.parse()?;  // 内部调用 Universe.import()
```

### After

```rust
// CompileSession 预处理 use
let mut session = CompileSession::new();
session.resolve_uses(source)?;  // 预加载依赖到 TypeStore
let parser = Parser::new_with_type_store(source, session.type_store.clone());
let ast = parser.parse()?;
```

## 成功标准

- [ ] `use` 语句由 CompileSession 预处理
- [ ] 模块解析结果缓存在 AutoCache
- [ ] Parser 不再调用 Universe.import()
- [ ] 增量编译：只重编译变更的模块
- [ ] 所有现有测试通过

## 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 循环依赖 | 中 | 高 | 检测并报错，不支持循环依赖 |
| 缓存失效 | 低 | 中 | 文件哈希 + 接口哈希双重验证 |
| 符号冲突 | 中 | 中 | 后导入覆盖 + 警告提示 |

## 后续工作

- [ ] 支持模块版本管理
- [ ] 支持私有模块
- [ ] 支持远程模块（网络加载）
- [ ] IDE 集成（自动补全导入）

## 参考资料

- [Plan 063: AIE 框架](./063-aie-framework.md)
- [Plan 064: Database + ExecutionEngine](./064-database-execution-engine.md)
- [Plan 090: 移除 Parser 对 Universe 的依赖](./090-remove-universe-from-parser.md)
- [Plan 084: 统一 TypeStore](./084-unified-type-context.md)
