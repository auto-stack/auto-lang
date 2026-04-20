## 1. Atom-Text 物理规范 (Node-Centric Spec)

Atom 的核心优势在于它能够以极简的语法同时描述 **“平铺的数据 (Object)”** 与 **“嵌套的结构 (Node)”**。

### 1.1 Node 语法的文本表现
`node` 遵循：`parent id (attr: val) { field: val; child_node; ... }`
* **parent**: 节点的类型名称（如 `div`, `button` 或自定义 `widget`）。
* **id**: 可选的唯一标识符。
* **(...)**: 属性区，用于存放轻量级、不参与深度逻辑的元数据。
* **{...}**: 内容区，包含键值对字段（Fields）和嵌套的子节点（Kids）。

### 1.2 Tag (Enum) 的序列化策略
对于 `tag Color { RED, GREEN }`，在文本 Atom 中，我们**坚持生成名称标签**。
* **表示法**：`color: RED` (不带引号，作为标识符) 或 `color: "RED"`。
* **理由**：在 `node` 的属性区或字段区，`button (color: RED)` 的可读性远超 `button (color: 0)`。编译器会自动维护 `String <-> ID` 的映射表。

---

## 2. 编译器自动化合成方案 (Compiler-Driven)

开发者定义的 `type` 或 `widget` 都会被编译器通过 AST 分析，自动合成序列化逻辑。

### 2.1 序列化：AST 遍历与合成 (To Atom)
编译器在 AOT 阶段会生成两个版本的 `to_atom`：`to_compact()` 和 `to_pretty()`。

#### 逻辑合成伪码：
```rust
// 编译器生成的伪逻辑
fn __auto_gen_to_atom(data: MyType, writer: &mut AtomWriter) {
    // 1. 处理 Node 头部
    writer.write_node_head("MyType", data.id); 
    
    // 2. 处理属性区 (Attributes)
    writer.begin_attrs();
    writer.write_attr("version", data.version);
    writer.end_attrs();

    // 3. 处理内容区 (Fields & Children)
    writer.begin_content();
    writer.write_field("count", data.count);
    
    // 如果是 Enum/Tag
    writer.write_tag("status", data.status); // 自动查表输出 "ACTIVE"
    
    // 递归处理子节点
    for child in data.children {
        child.to_atom(writer);
    }
    writer.end_content();
}
```

### 2.2 反序列化：有限状态机 (From Atom)
反序列化是最复杂的环节。由于 `node` 语法比 JSON 复杂（多了 `[]`, `()`, `{}` 的分层），编译器会合成一个**带有上下文栈的递归下降解析器**。

#### 核心步骤：
1.  **词法标记 (Tokenization)**：识别 `[`, `(`, `{` 等分界符。
2.  **类型对齐**：当解析器读到 `parent` 名称时，立即从符号表查找对应的 `type` 定义。
3.  **内存分配**：根据定义的 `size` 预留空间。
4.  **Enum 还原**：读到 `RED` 时，通过编译器合成的 `switch` 分支，将其还原为物理值 `0`。

---

## 3. 示例：Auto 源码与 Atom 输出

### Auto 源码 (type 定义)
```auto
tag Status { Active, Suspended }

type Profile {
    id    string
    name  string
    age   int
    state Status
}
```

### 自动生成的 Atom-Text (美丽格式)
```auto
Profile [user_001] (version: 1.0) {
    name: "Gemini"
    age: 18
    state: Active  // Tag 自动转化为名称
    
    // 假设有嵌套的 Node
    Bio {
        text: "Authentic & Adaptive"
    }
}
```

---

## 4. 远程引用的重新设计：`link` 机制

既然 `node` 被正名为树状结构，那么原先的 `@uuid` 远程句柄，我们将其命名为 **`link`**。

* **语法**：`link @uuid(protocol://address)`
* **物理本质**：它是一个跨进程的**句柄锚点**。
* **序列化**：当一个 `link` 被写入 Atom 时，它表现为：`@ref: "uuid_string"`。
* **反序列化**：解析器读到 `@ref` 时，不会创建新对象，而是去系统的 `LinkRegistry` 中查找并绑定现有的物理通道。

---

## 5. 总结：Atom 体系的三个支柱

| 概念 | 语法表现 | 物理用途 |
| :--- | :--- | :--- |
| **Object** | `{ k: v }` | 纯粹的数据载体（类似 JSON）。 |
| **Node** | `p [id] (a) { c }` | **树状拓扑描述**（替代 XML/HTML）。 |
| **Link** | `@ref(uuid)` | **远程/跨进程句柄**（原先的远程引用）。 |

---
