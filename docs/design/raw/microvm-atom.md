明白，这是一份专注于 **Auto MicroVM 数据结构实现与分级优化策略** 的专项设计文档。

这份文档剥离了语言语法，专注于底层如何在资源受限的 MCU 上高效地存储和操作 Atom（Node/Object/Array 混合体）数据。

---

# Auto MicroVM: Atom 数据结构与分级优化架构设计 (v1.0)

**适用场景**: Auto 语言在嵌入式 MCU 上的运行时环境 (MicroVM)
**核心对象**: **Atom** (Auto Node / JSON Object / Array 的统一超集)
**设计目标**: 在无操作系统 (No-OS)、无动态分配 (No-Malloc) 的环境下实现 DOM 树构建与操作。

---

## 1. 核心数据模型 (The Physical Model)

在 MicroVM 中，我们不区分 Node、Object 和 Array，它们统一由 **"静态池 + 链表"** 构成。

### 1.1 基础内存布局 (Static Pools)

系统启动时，在 `.bss` 段或静态区预分配两大核心池：

1. **`STRING_POOL`**: 全局字符串池。
* 所有 Key (属性名、Type名) 和 String Value 都存储在此。
* **关键机制**: **String Interning (字符串驻留)**。所有字符串在池中唯一，运行时只通过 `uint16_t StrIdx` 操作，禁止 `strcmp`。


2. **`NODE_POOL`**: 全局节点池。
* 一个固定大小的 `struct Node` 数组（例如 `Node nodes[4096]`）。
* 使用空闲链表 (`FREE_LIST`) 管理分配与回收。



### 1.2 统一节点结构 (The Unified Node)

为了极致省内存，使用索引代替指针。

```c
typedef uint16_t NodeIdx; // 节点索引 (0 = NULL)
typedef uint16_t StrIdx;  // 字符串索引

struct Node {
    StrIdx key;       // 键 / 类型名 (Type)
    struct Value val; // 值 / ID名
    NodeIdx next;     // 链表指针 (指向下一个属性或子节点)
};

```

### 1.3 Atom 的物理映射 (Header-Body Pattern)

逻辑结构：`<Type> <ID> (Props...) { Children... }`
物理实现采用 **头节点 + 内容链** 模式：

* **Header Node (头节点)**: 定义 Atom 的元数据。
* `key`: 存储 **Type** (如 "div")。
* `val`: 存储 **ID** (如 "card_1")。
* `next`: 指向 Body 的第一个节点。


* **Body Chain (内容链)**: 混合存储属性和子节点。
* **属性节点**: `key` != 0。`val` 为属性值。
* **子节点**: `key` == 0。`val` 指向子节点的 Header Node。



---

## 2. 三级优化策略 (Three-Tier Optimization Strategy)

根据 MCU 的 RAM 资源（Tiny, Standard, Performance），采用三种不同的内存管理策略。

### Level 1: Tiny (RAM < 64KB) - "纯静态链表"

**目标**: 极致的低内存占用，零碎片。适用于 Cortex-M0/M3 (STM32F103, ESP8266)。

* **实现方案**:
* 完全沿用上述的 **单向链表** 结构。
* **对象属性查找**: 线性遍历 `O(N)`。
* **数组随机访问**: 线性遍历 `O(N)`。


* **特性**:
* **无 Malloc**: 所有内存分配仅为 `free_list_pop()`。
* **内存开销**: 每个属性额外消耗 2 字节 (`next`)。


* **适用场景**: 简单的配置解析、深度不大的 UI 树构建。

### Level 2: Standard (RAM 64KB - 256KB) - "链表 + Hash 加速"

**目标**: 在保持无碎片的前提下，解决大对象的查找性能问题。适用于 Cortex-M4 (STM32F4, ESP32)。

* **实现方案**:
* 引入第三个池：**`HASH_POOL`**。
* **混合策略**:
* 小对象（属性 < 8）：依然是纯链表。
* 大对象（属性 >= 8）：从 `HASH_POOL` 申请一个小块内存，建立 `Hash(Key) -> NodeIdx` 的索引表。


* Header Node 增加一个标志位，指向 Hash 索引。


* **特性**:
* **查找性能**: 接近 `O(1)`。
* **内存开销**: 大对象会额外消耗 16~64 字节的 Hash 空间。
* **碎片风险**: 低。Hash 块通常固定大小（如 16 字节），易于管理。



### Level 3: Performance (RAM > 256KB) - "Slab 分配 + 紧凑数组"

**目标**: 追平甚至超越 QuickJS 的性能，支持复杂业务逻辑。适用于 Cortex-M7 (STM32H7, ESP32-S3)。

* **实现方案**:
* **放弃链表**：不再使用 `next` 指针串联属性。
* **Slab Allocator (对象片分配器)**:
* 将内存划分为固定大小的 **Slot** (如 16B, 32B, 64B, 128B)。
* 当创建一个有 4 个属性的对象时，直接申请一个 `64B Slot`。


* **数组化存储**:
* 属性数据 `[Key|Val]` 连续紧凑存储在 Slot 中。
* CPU 缓存命中率大幅提升。




* **特性**:
* **查找性能**: `O(1)` (数组下标访问) 或 二分查找。
* **内存密度**: 极高。省去了每个节点的 `next` 指针 (节省 ~25%)。
* **竞争优势**: 这种**“连续内存布局”**比 QuickJS 的通用堆分配更快，且通过 Slab 机制完全避免了外部内存碎片。



---

## 3. 性能对比预估

| 指标 | Level 1 (链表) | Level 2 (链表+Hash) | Level 3 (Slab数组) | 对标 MicroQuickJS |
| --- | --- | --- | --- | --- |
| **启动内存开销** | < 2KB | ~4KB | ~10KB | > 30KB |
| **属性查找 (N=10)** | 慢 (遍历10次) | 快 (Hash计算) | 极快 (数组偏移) | 快 (Hash/Array) |
| **属性查找 (N=100)** | 极慢 | 快 | 极快 | 快 |
| **内存碎片率** | **0%** | < 5% | **0% (内部碎片除外)** | 高 (依赖 Malloc) |
| **实现复杂度** | 低 | 中 | 高 | 极高 |

---

## 4. 关键算法逻辑

### 4.1 属性查找 (Lookup)

```c
// Level 1 & 2 的查找逻辑
Value get_prop(NodeIdx obj_idx, StrIdx key) {
    // 1. Level 2 优化: 检查是否有 Hash 索引
    if (has_hash_index(obj_idx)) {
        return lookup_hash(obj_idx, key);
    }

    // 2. Level 1 回退: 链表遍历
    NodeIdx curr = NODE_POOL[obj_idx].next; // 获取 Body
    while (curr != 0) {
        if (NODE_POOL[curr].key == key) {
            return NODE_POOL[curr].val;
        }
        curr = NODE_POOL[curr].next;
    }
    return VAL_UNDEFINED;
}

```

### 4.2 垃圾回收 (GC)

由于采用了静态池，我们采用 **引用计数 (Reference Counting)** 作为主 GC 策略。

* **机制**: `Node` 结构体中通过 padding 或 bit-field 增加 `ref_count`。
* **回收**: 当 `ref_count` 归零时，将 `NodeIdx` 归还给 `FREE_LIST`。
* **优势**: 实时性高，无 STW (Stop-The-World) 卡顿，非常适合实时控制系统。
* **循环引用**: 对于 Level 3 系统，可由用户显式调用 `gc_collect()` 进行一次 Mark-Sweep 来解决循环引用，Level 1/2 通常忽略此问题（依靠编程规范避免）。

---

## 5. 结论

该设计方案通过**统一的数据结构**和**分级的存储策略**，完美解决了不同 MCU 资源下的痛点：

1. **低端 MCU**: 用“时间换空间”，保证能跑起来，绝不 OOM。
2. **高端 MCU**: 用“空间换时间”，利用 Slab 数组化技术，在性能上正面硬刚 MicroQuickJS，同时保持了嵌入式系统必须的**内存确定性**。