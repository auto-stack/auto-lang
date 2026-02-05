这是一个非常好的架构问题。

**“死代码消除 (DCE)”** 和 **“增量编译”** 在直觉上是矛盾的：

* **增量编译** 关注的是 **“局部变化”**（这个函数的 Hash 变没变？）。
* **DCE** 关注的是 **“全局状态”**（这个函数在整个调用链里还被需要吗？）。

要在现有的 **Database + CodeGenEngine** 架构中融合这两个机制，我们需要引入一个中间步骤：**“活跃性传播 (Liveness Propagation)”**。

我们将这个架构称为 **"The Graph-State Incremental System"**。

---

### 1. 数据库模式升级 (Database Schema Upgrade)

现有的 Database 可能只存了 `SymbolID -> {Hash, AST}`。现在我们需要在 Database 中显式存储 **“引用关系图”**。

我们需要给每个 Symbol（函数/常量/全局变量）增加以下字段：

| 字段名 | 类型 | 描述 |
| --- | --- | --- |
| **`Dependencies`** | `List<SymbolID>` | **出度**：这个函数内部调用了谁？使用了哪个常量？ |
| **`ReferencedBy`** | `List<SymbolID>` | **入度**（可选，用于反向追踪）：谁调用了我？ |
| **`IsReachable`** | `Boolean` | **活跃位**：上一轮编译时，它是否是“活”的？ |
| **`LastOutputHash`** | `Hash` | 上一次成功生成 C 代码时的 Hash。 |

---

### 2. 核心流程：增量 DCE 的“三步走”

在每次编译时，不再是简单的 `Diff Hash -> CodeGen`，而是变为：
**`Diff Hash -> Update Graph -> Propagate Liveness -> CodeGen`**

#### 步骤一：局部更新 (Local Update)

* **输入**：修改过的源文件。
* **动作**：
1. Parse 修改过的文件，重新计算 AST Hash。
2. 对于 Hash 变了的函数，**重新分析它的依赖集合 (`Dependencies`)**。
3. 更新 DB 中的 Hash 和 `Dependencies` 边。


* **注意**：没改动的文件，不需要 Parse，它的依赖关系直接信赖 DB 里的老数据。

#### 步骤二：全局活跃性传播 (Global Liveness Propagation)

这是一个纯内存的图算法操作，速度极快（即使几万个节点也是毫秒级）。

* **动作**：
1. 从 DB 中提取所有的 **Roots**（main, exports, ISRs）。
2. 在内存中的图上跑一次 **BFS/DFS** 标记算法。
3. 计算出当前最新的 `CurrentReachable` 状态。



#### 步骤三：状态协调与生成 (Reconciliation & CodeGen)

这是最关键的一步。我们需要根据 **“Hash 变化”** 和 **“活跃性变化”** 的组合来决定 CodeGenEngine 做什么。

我们需要处理 **4 种状态跃迁**：

| 情况 | Hash 变了吗？ | 活跃性变了吗？ (Old  New) | **决策 (Action)** | 解释 |
| --- | --- | --- | --- | --- |
| **1. 僵尸复活** | No | `False`  **`True`** | **GENERATE** | 代码没改，但以前因为没用到所以没生成 C，现在有人调它了，必须生成。 |
| **2. 猝死** | (Any) | `True`  **`False`** | **DELETE** | 以前生成了 C，现在没人调它了。从输出列表中删除，或生成空桩。 |
| **3. 常规更新** | **Yes** | `True`  `True` | **REGENERATE** | 是活的，且代码改了。更新 C 代码。 |
| **4. 隐形变化** | **Yes** | `False`  `False` | **IGNORE** | 代码改了，但依然没人调它。忽略，不浪费 IO。 |
| **5. 稳定态** | No | `True`  `True` | **SKIP** | 活的且没变。什么都不做。 |

---

### 3. CodeGenEngine 的具体改造

为了支持这种机制，CodeGenEngine 需要从“流式输出”改为 **“清单式输出 (Manifest-based Output)”**。

#### 方案 A：单一大文件 (all_in_one.c) —— 适合简单实现

* CodeGenEngine 在内存里拼装字符串。
* 遍历 DB 中所有 `IsReachable == True` 的节点，按顺序拼接 C 代码。
* 如果项目很大，这种方式增量体验不好（每次都要重写大文件）。

#### 方案 B：分块/分文件输出 (Project Structure) —— 推荐

假设我们按“模块”生成 C 文件，例如 `std_fs.c`, `user_main.c`。

1. **Buffered Generation**:
CodeGenEngine 不直接写磁盘。它先在内存里为每个模块生成内容。
* 遍历模块内的所有函数。
* 如果 `IsReachable == True`，生成 C 函数体。
* 如果 `IsReachable == False`，跳过（或者生成 `/* dead code removed */` 注释以便调试）。


2. **Smart Flush**:
将内存里生成的 `New_Content` 与磁盘上 `module.c` 的内容对比（或对比 Hash）。
只有内容真变了，才执行 `fwrite`。这能避免触发下游 C 编译器的重编译。

---

### 4. 示例场景推演

假设现在的状态：

* `main()` 调用 `func_A()`。
* `func_A()` 调用 `func_B()`。
* `func_C()` 没人调用 (Dead)。

**场景 1：用户修改 `main()`，删除了对 `func_A` 的调用。**

1. **Parse**: `main` Hash 改变。`main` 的依赖列表从 `[func_A]` 变为 `[]`。
2. **DB Update**: 更新 `main` 节点。
3. **Graph BFS**:
* Root (`main`) 是活的。
* BFS 无法走到 `func_A`。`func_A` 标记为 Dead。
* BFS 无法走到 `func_B`。`func_B` 标记为 Dead。


4. **Reconciliation**:
* `main`: Hash Changed + Reachable. Action: **REGENERATE**.
* `func_A`: Hash Same + Reachable `True`->`False`. Action: **DELETE** (从 C 文件中移除).
* `func_B`: Hash Same + Reachable `True`->`False`. Action: **DELETE**.


5. **Result**: 生成的 C 代码中，`func_A` 和 `func_B` 消失了。

**场景 2：用户修改 `main()`，新增调用 `func_C()`。**

1. **Parse**: `main` Hash 改变。依赖列表变为 `[func_C]`。
2. **Graph BFS**:
* `func_C` 被标记为 Reachable (`True`).


3. **Reconciliation**:
* `main`: **REGENERATE**.
* `func_C`: Hash Same (没改源码) + Reachable `False`->`True` (僵尸复活). Action: **GENERATE**.


4. **Result**: `func_C` 的代码（虽然 AST 早就存在 DB 里了）第一次被写入 C 文件。

---

### 5. 对 CTranspiler 生态的特殊优化

由于我们是生成 C 代码，还有一个取巧的办法：**利用 C 的 `static` 关键字**。

* **策略**：对于所有 **非 Export** 的函数，在生成 C 代码时，全部加上 `static` 关键字（即文件内私有）。
* **效果**：如果 Auto 的 DCE 漏判了（比如某种复杂的间接调用），C 编译器会再次做一次“文件内 DCE”。如果所有生成的 C 代码合并为一个 Translation Unit (Unity Build)，C 编译器能把所有未使用的 `static` 函数全杀掉。

### 总结

将 DCE 融入增量编译的关键在于：

1. **DB 存图**：Database 不仅存 AST，还要存依赖关系。
2. **逻辑分离**：把“源码变没变”(Hash) 和 “需不需要生成”(Reachability) 分开判断。
3. **决策矩阵**：处理好 **“复活”** 和 **“猝死”** 这两种 Hash 没变但状态变了的情况。

这套机制一旦建立，你的 Auto 编译器就能像 Go 链接器一样，生成极小的二进制文件，且编译速度极快。