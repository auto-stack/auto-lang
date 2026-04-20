这是一个基于昨天深度讨论的 **Auto Incremental Engine (AIE)** 架构设计文档。这份文档将“流水线数据库工厂”的概念具体化，作为未来开发工作的蓝图。

---

# Auto Incremental Engine (AIE) Architecture Design Document

**Project:** Auto Language Compiler
**Version:** 1.0
**Date:** 2026-01-31
**Status:** Draft / Blueprint

## 1. Executive Summary (执行摘要)

为了实现 Auto 语言的 **AutoLive（亚秒级热重载）** 愿景，传统的“全量流水线”编译器架构（Source -> AST -> Binary）已无法满足需求。

**AIE (Auto Incremental Engine)** 是一种全新的**基于查询 (Query-Based)** 的编译器架构。它将编译器视为一个**增量数据库**，通过细粒度的依赖追踪和多级哈希缓存，确保**“只编译修改的部分及其直接受影响的部分”**。

---

## 2. Core Philosophy (核心设计哲学)

### 2.1 From "Process" to "Database"

* **传统编译器 (Current)**: `compile(source) -> binary`。这是一个过程，每次从头开始，状态（Universe）是副作用的产物。
* **AIE 编译器 (Target)**: `db.query(artifact_id)`。编译器是一个**常驻内存的数据库**。编译过程是对数据库的查询。

### 2.2 The "Pull" Model

* **Push Model (Legacy)**: Parser 解析完代码，主动去修改 `Universe`。
* **Pull Model (AIE)**: `TypeChecker` 需要某个符号的类型时，向 `Database` 发起查询。如果数据过期，触发重算；如果数据有效，直接返回缓存。

---

## 3. System Architecture (系统架构)

### 3.1 The Database (全局单一事实来源)

`Database` 是全局唯一的结构体，替代原本的 `Rc<RefCell<Universe>>`。它包含两部分：

1. **Storage (存储层)**: 存放源码、AST、符号表。
2. **Cache (缓存层)**: 存放中间计算结果（类型、字节码、依赖关系）。

```rust
struct Database {
    // Input Data (由 Indexer 写入)
    sources: HashMap<FileId, String>,
    asts:    HashMap<FragId, Arc<FnDecl>>,
    symbols: HashMap<Sid, SymbolMeta>, // meta: name, offset, file_id

    // Derived Data (由 Query Engine 惰性计算并缓存)
    types:      DashMap<Sid, Type>,
    bytecodes:  DashMap<FragId, Blob>,
    dep_graph:  DependencyGraph, 
}

```

### 3.2 The Indexer (录入器)

这是系统中**唯一**拥有 `&mut Database` 写权限的组件。

* **职责**:
1. **Resilient Parsing**: 快速扫描源码，识别出函数/结构体的边界。
2. **Fragmenting**: 将源码切分为独立的 **Frag (颗粒)**。
3. **Registration**: 为每个 Frag 分配稳定的 ID，存入 Database。



### 3.3 The Query Engine (查询引擎)

所有的编译逻辑（类型检查、代码生成）都重构为 **纯函数查询**。

* **输入**: `&Database` (只读) + `QueryID`
* **输出**: `Result`
* **逻辑**: Check Cache -> (Miss) -> Compute -> Update Cache -> Return。

---

## 4. Granularity & Dependency (粒度与依赖)

### 4.1 Granularity: Declaration Level (声明级)

既不是粗糙的文件级，也不是琐碎的语句级，而是**声明级**。

* **Compilation Unit**: `Function`, `Struct`, `Global Const`.
* 每个 Unit 被封装为一个 **Micro-Object**。

### 4.2 Hashing Strategy (多级哈希熔断)

为了防止无效的级联编译，采用三级哈希检查：

1. **L1 Text Hash**: 源码文本变了吗？
* *No* -> 停止。
* *Yes* -> 解析 AST。


2. **L2 AST Hash**: 剔除注释和格式后，结构变了吗？
* *No* -> 停止。
* *Yes* -> 重新进行语义分析。


3. **L3 Interface Hash (关键)**: 函数的签名（参数/返回值）变了吗？
* *No* -> **熔断依赖传播**。虽然函数体变了（需要重编自己），但依赖它的其他函数不需要重编。
* *Yes* -> 触发依赖它的模块重编。



### 4.3 Dependency Graph (依赖图)

维护一张**反向依赖表**：`Map<ProviderID, List<ConsumerID>>`。

* 当 `ProviderID` 发生变更（且 Interface Hash 改变）时，将列表中的 `ConsumerID` 标记为 Dirty。

---

## 5. Workflow: The Lifecycle of a Change (变更生命周期)

假设用户修改了 `fn calculate()` 的一行代码：

1. **Capture**: IDE/FileWatcher 捕获文本变更。
2. **Index**: Indexer 重新解析该文件，定位到 `fn calculate` 的 Frag，更新 AST。
3. **Invalidate**: 清除 `fn calculate` 在 Database 中的 `bytecode` 缓存。
4. **Propagate**:
* 计算 `calculate` 的 **Interface Hash**。
* 如果签名没变 -> 依赖传播结束。
* 如果签名变了 -> 查找依赖图，将调用过 `calculate` 的函数缓存也清除。


5. **Re-Query (Lazy)**:
* 当 AutoLive 请求更新时，触发 `query_bytecode(calculate)`。
* 编译器重新生成该函数的 Micro-Object。



---

## 6. Integration with AutoLive (与运行时对接)

AIE 的最终产物不是一个巨大的 ELF 文件，而是一个 **Patch Stream (补丁流)**。

### 6.1 Patch Structure

```c
struct Patch {
    uint32_t frag_id;    // 哪个函数？
    uint32_t code_size;
    uint8_t  code[];     // 新的机器码/字节码
    Reloc    relocs[];   // 内部引用的其他符号
};

```

### 6.2 Implementation (RAM Overlay)

1. **Build**: AIE 生成上述 Patch。
2. **Inject**: Debugger 将 Patch 写入 MCU 的 RAM Hot Zone。
3. **Link**: AIE 计算新地址，更新 MCU 端的 **GOT (Global Offset Table)**，将旧函数指针指向新地址。

---

## 7. Implementation Roadmap (实施路线图)

### Phase 1: Structural Refactoring (架构重构) - *Current Priority*

* **Goal**: 消除 `RefCell<Universe>`，建立 `Database` 骨架。
* **Action**:
* 定义全局 `Database` 结构。
* 剥离 `Symbol` 中的运行时 `value`。
* 重构 Parser，使其变为纯函数，只输出 AST，不副作用修改环境。



### Phase 2: File-Level Incremental (文件级增量)

* **Goal**: `import` 变动时只重编相关文件。
* **Action**:
* 在 Database 中实现 `FileHash`。
* 实现粗粒度的 `FileDependencyGraph`。



### Phase 3: Fine-Grained & AutoLive (细粒度与热重载)

* **Goal**: 函数级增量，对接 MicroVM/Trampoline。
* **Action**:
* 实现 **Interface Hash** 逻辑。
* 实现 Codegen 的 `Patch` 生成器。
* 实现 MCU 端的 `Loader`。



---

## 8. Conclusion

AIE 架构将编译器从一个“批处理工具”转变为一个“实时响应服务”。它是实现 AutoLive 的核心引擎。通过**声明级粒度**和**接口哈希熔断**，我们将实现理论上最小的编译开销，从而达成亚秒级的开发体验。