这是一份经过严格修正、符合 Auto 语言“**赋值一致性**”与“**空格分隔**”律法的最终版设计文档。

---

# 📝 Auto 语言设计文档：大一统枚举 (Unified Enum)

## 1. 核心设计哲学
Auto 语言将传统的 `enum`（状态）与 `tag`（联合体）合并，构建了**物理透明**的数据模型。
* **赋值一致性**：凡是涉及原始标量值（Raw Value）的绑定，统一使用 `=`。
* **空格分隔**：凡是涉及类型（Type）的标注，统一使用空格，严禁使用 `:`。
* **AI 驱动**：消除冗余符号，降低 AST（抽象语法树）的解析熵，提升 AI 生成代码的准确率。

---

## 2. 三大物理形态规约

### 2.1 标量枚举 (Scalar Enum)
用于纯状态表示。支持显式指定底层整数类型，并使用 `=` 绑定原始值。

```auto
// 显式指定 u16 宽度，使用 = 绑定值
enum HttpCode u16 {
    OK = 200
    NotFound = 404
    InternalError = 500
}

// 默认情况：底层为 u8，值从 0 开始自增
enum Color {
    Red
    Green
    Blue
}
```

### 2.2 同构数据枚举 (Homogeneous Enum)
**定义**：枚举名后紧跟类型名。所有分支共享该物理结构。
**特性**：支持**全域成员访问**（无需 `match` 即可访问负载内部属性）。

```auto
type Point {
    x int
    y int
}

// 语法：enum [名称] [共享类型]
enum Vertex Point {
    LeftTop
    LeftBottom
    RightTop
    RightBottom
}

fn reset(v Vertex) {
    v.x = 0  // 物理特权：直接 O(1) 偏移访问
    v.y = 0
}
```

### 2.3 异构数据枚举 (Heterogeneous Enum)
**定义**：各分支可拥有独立的负载类型。分支名与类型间用**空格**分隔。

```auto
enum Msg {
    Quit
    Move Point              // 单一结构负载
    Write string            // 基础类型负载
    Pair (string, string)   // 匿名元组负载（需括号，逗号分隔）
    Update { id int, val float } // 匿名结构体负载（推荐）
}
```



---

## 3. 迁移指南：从 `tag` 演进至 `enum`

Auto 语言正式废弃 `tag` 关键字，现有的 `tag` 定义需按以下逻辑重构为 `enum`。

### 3.1 语法转换对照
| 场景 | 旧版 `tag` 语法 (废弃) | 新版 `enum` 语法 (标准) | 备注 |
| :--- | :--- | :--- | :--- |
| **基础转换** | `tag Msg { Quit, Move Point }` | `enum Msg { Quit, Move Point }` | 关键字替换，去掉分支间的逗号 |
| **元组转换** | `tag M { Pair(int, int) }` | `enum M { Pair (int, int) }` | 分支名与括号间用空格，元组内保留逗号 |
| **同构升级** | `tag V { A Point, B Point }` | `enum V Point { A, B }` | **推荐**：利用同构特性实现直接访问 |

### 3.2 模式匹配（Pattern Matching）转换
由于定义时去除了括号强制要求，提取数据时也同步简化：

```auto
// 旧版
match msg {
    .Move(p) => print(p.x)
}

// 新版 (Auto Standard)
match msg {
    .Move p => print(p.x) // 标签 [空格] 变量名
    .Pair (a, b) => print(a) // 元组解构
}
```

---

## 4. 编译器实现要点 (Implementation Notes)

1.  **内存对齐**：
    * **同构枚举**：编译器计算 `Payload` 的统一偏移量，并在编译期注入 `Getter`。
    * **异构枚举**：Payload 空间按最大分支对齐，采用典型的 `Tag + Union` 结构。
2.  **符号查找顺序**：
    在 `enum Name Type { ... }` 结构中，编译器首先尝试将 `Type` 解析为类型标识符。如果解析失败且存在 `=`，则回退为标量枚举解析逻辑。
3.  **内建方法注入**：
    所有 `enum` 实例自动具备：
    * `.tag()`: 获取底层整数 ID。
    * `.name()`: 获取分支的字符串名称（如 `"LeftTop"`）。

---