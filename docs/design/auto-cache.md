这是一个详细的 **AutoCache** 系统设计文档。该文档旨在指导 Auto 编译器的构建系统（AutoMan）与底层缓存机制的实现。

---

# AutoCache 系统设计规范 (v1.0)

**模块**: Auto Build System / Global Cache
**代号**: AutoCache
**目标**: 实现跨工程、跨版本、基于内容的全局增量编译缓存，最大化缩减编译时间和磁盘占用。
**存储架构**: SQLite (元数据索引) + FileSystem (二进制产物)

---

## 1. 架构总览 (Architecture Overview)

AutoCache 是一个**内容寻址存储 (Content-Addressable Store, CAS)** 系统。它不关心文件属于哪个工程，只关心“这个编译单元的输入特征（指纹）是什么”。

### 1.1 目录结构

缓存默认存储在用户主目录下（如 `~/.auto/cache`），结构如下：

```text
~/.auto/cache/
├── index.db            # SQLite 数据库 (元数据与索引)
├── index.db-shm        # WAL 模式共享内存
├── index.db-wal        # WAL 模式日志
├── blobs/              # 二进制对象存储 (CAS)
│   ├── a1/             # 分片目录 (Hash 前2位)
│   │   ├── a1b2c3d4... # 实际产物文件 (文件名即 Hash)
│   │   └── ...
│   ├── f9/
│   └── ...
└── locks/              # 进程锁目录 (备用，主要依靠 SQLite 锁)
    └── gc.lock         # 垃圾回收时的互斥锁

```

---

## 2. 核心算法：指纹哈希 (The Fingerprint Strategy)

AutoCache 的核心在于如何计算一个编译单元的唯一哈希值（Key）。如果 Key 相同，我们认为编译产物可以 100% 安全复用。

公式：


### 2.1 Content Hash (内容哈希)

针对源代码的哈希，必须排除格式干扰。

* **输入**: 源代码文件的 **AST (抽象语法树)** 序列化数据，或者经过格式化（Canonicalized）后的源码。
* **排除**: 空格、注释、换行符的差异。
* **路径重映射**: 极其重要！
* 所有绝对路径（如 `/User/dev/proj/src/main.auto`）必须替换为相对路径（如 `{ROOT}/src/main.auto`）。
* 这保证了同一份代码在不同电脑、不同目录下编译，Hash 一致。



### 2.2 Context Hash (上下文哈希)

编译环境的影响。

* **Target Triple**: `x86_64-linux-gnu`, `thumbv7m-none-eabi` (MCU)。
* **Compiler Flags**: 优化等级 (`-O2`), 宏定义 (`-dDEBUG`), 符号表设置 (`-g`)。
* **Toolchain Version**: Auto 编译器版本号, 后端 C 编译器版本号 (如 GCC 12.1)。
* **Capabilities**: AutoMan 的 `[capabilities]` 配置 (如 `fs=false`)。

### 2.3 Dependency Hash (依赖哈希)

依赖链的传递。

* 如果模块 A 导入了模块 B，Key 计算必须包含 。
* 这形成了一个 Merkle Tree 结构：底层模块变动，上层模块 Hash 自动雪崩改变。

---

## 3. 存储层设计 (Storage Layer)

### 3.1 SQLite Schema (`index.db`)

使用 SQLite 存储元数据，利用其原子性和 WAL 并发优势。

```sql
-- 开启 WAL 模式以支持并发读写
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

CREATE TABLE artifacts (
    -- 主键：计算出的 SHA256 指纹 (Hex String)
    hash_key      TEXT PRIMARY KEY,
    
    -- 存储路径：相对于 blobs/ 的路径 (例如 "a1/a1b2...")
    blob_path     TEXT NOT NULL,
    
    -- 产物类型：区分不同类型的文件
    -- 0: Object File (.o/.obj)
    -- 1: Static Lib (.a/.lib)
    -- 2: Auto Bytecode (.abc)
    -- 3: C Source (.c) - 可选，用于源码分发
    artifact_type INTEGER,
    
    -- 文件大小：用于空间统计
    file_size     INTEGER,
    
    -- 访问统计：用于 LRU 垃圾回收
    created_at    INTEGER, -- UNIX Timestamp
    last_used_at  INTEGER, -- UNIX Timestamp
    access_count  INTEGER DEFAULT 1
);

-- 索引：加速垃圾回收 (查找最久未使用的记录)
CREATE INDEX idx_lru ON artifacts(last_used_at);

```

### 3.2 文件系统层 (Blob Store)

* **命名规则**: 文件名直接等于 `hash_key`。
* **分片 (Sharding)**: 为避免单目录文件数过万导致文件系统变慢，使用 2 级分片。
* Hash: `a1b2c3d4...`
* Path: `~/.auto/cache/blobs/a1/a1b2c3d4...`


* **原子写入**:
1. 编译生成到临时目录 `tmp/unique_id.o`。
2. 执行 `rename()` 将其移动到 `blobs/a1/...`。确保此时文件完整。



---

## 4. 交互工作流 (Workflow)

AutoMan (构建工具) 与 AutoCache 的交互流程。

### 4.1 查找 (Query / Get)

**输入**: 编译单元信息 (AST, Config, Deps)。

1. AutoMan 计算 `TargetHash`。
2. 查询 SQLite: `SELECT blob_path FROM artifacts WHERE hash_key = TargetHash`。
3. **Case A: Miss (未命中)**
* 返回 `null`。AutoMan 触发编译任务。


4. **Case B: Hit (命中)**
* 检查文件系统中 `blob_path` 是否真实存在（防止用户误删文件）。
* 如果存在：
* **异步** 更新 SQLite: `UPDATE artifacts SET last_used_at = NOW(), access_count += 1 ...`。
* 在当前工程的构建目录建立 **硬链接 (Hard Link)** 指向 Blob。
* 返回 `Success`。





### 4.2 存入 (Store / Put)

**输入**: 编译好的临时文件 `temp.o`, `TargetHash`.

1. 计算分片目录，确保目录存在 (`mkdir -p`).
2. 移动文件: `mv temp.o ~/.auto/cache/blobs/xx/xxxx...`。
* *优化*: 如果目标文件已存在（Hash碰撞极低概率，或并发写入），对比大小/内容。如果一致则直接覆盖或忽略。


3. 写入 SQLite:
* `INSERT OR REPLACE INTO artifacts ...`



---

## 5. 与编译器组件的集成

### 5.1 与 Transpiler (a2c) 配合

a2c 将 Auto 源码转译为 C 源码，然后调用 GCC/Clang 生成 `.o`。**这是最耗时的步骤，也是 AutoCache 的主要优化点。**

* **粒度**: 以 **Module (模块)** 或 **Translation Unit (编译单元)** 为单位。
* 例如 `std.io` 模块对应一个 `std_io.o`。


* **流程**:
1. a2c 生成 C 代码之前，先算 Hash。
2. 查 AutoCache。
3. 命中 -> 跳过 C 生成，跳过 GCC 编译，直接 Link。
4. 未命中 -> 生成 C -> 调 GCC -> 存入 Cache。



### 5.2 与 AutoVM (a2b) 配合

AutoVM 执行的是字节码 (`.abc`)。

* **粒度**: Bytecode Chunk。
* **流程**:
* a2b 编译器将模块编译为字节码。
* 将字节码 Blob 存入 AutoCache (Type = 2)。
* 运行时加载器 (Loader) 可以直接从 Cache `mmap` 字节码文件，减少加载时间。



### 5.3 跨版本共享 (Cross-Version Sharing)

假设 `MyLib v1.0` 和 `MyLib v1.1` 只有 `README` 变了，代码没变。

* 由于 Hash 计算的是 **AST**，两个版本的代码 AST 一模一样。
* AutoCache 会自动识别出 Hash 一致。
* **结果**: v1.1 编译时直接复用 v1.0 的二进制产物。

---

## 6. 维护与垃圾回收 (GC Strategy)

磁盘空间不是无限的，必须有淘汰机制。

### 6.1 触发机制

* **被动触发**: 每次编译结束后，检查 `blobs` 目录总大小。
* **主动触发**: 用户运行 `auto cache prune`。

### 6.2 清理算法 (LRU)

设定阈值：例如 `MAX_SIZE = 10GB`。

1. 检查当前 DB 中所有文件的 `SUM(file_size)`。
2. 如果 > 10GB:
* 计算需要释放的大小 `TargetFree = Current - 8GB` (回落到安全水位)。
* 查询: `SELECT hash_key, blob_path, file_size FROM artifacts ORDER BY last_used_at ASC`。
* 遍历结果，删除文件，累加释放大小，直到满足 `TargetFree`。
* 批量删除 DB 记录: `DELETE FROM artifacts WHERE hash_key IN (...)`。
* (可选) 执行 `PRAGMA incremental_vacuum;` 整理数据库碎片。



---

## 7. 调试与排错 (Debuggability)

当缓存命中但程序运行异常时（极罕见的哈希冲突或环境污染），需要手段排查。

* **环境变量**: `AUTO_NO_CACHE=1`。强制绕过 AutoCache，执行全新编译。
* **调试命令**: `auto cache inspect <module_name>`。打印计算出的 Hash 因子，帮助开发者看看到底是哪个 Context 导致了 Hash 变化。
* **日志**: AutoMan 在 Verbose 模式下应输出 `[Cache Hit] std.io (Hash: a1b2...)`。

---

## 8. 未来扩展：集群模式 (AutoHub)

由于采用了 Content-Addressable 设计，该架构天然支持远程化。

* **Client**: 在查询本地 SQLite 失败后，发送 HTTP GET `/artifact/<hash>` 到服务器。
* **Server**: 一个简单的 KV 存储服务 + 对象存储。
* **CI/CD**: CI 服务器编译完后，执行 `PUT` 将产物上传。团队成员拉取代码后，编译时间接近于零。

---

**总结**:
AutoCache 利用 **AST 级哈希** 屏蔽了格式差异，利用 **SQLite** 解决了并发和索引难题，利用 **Context Hash** 解决了跨平台兼容性。这套设计能让 Auto 语言的构建速度随着使用时长的增加而越来越快。