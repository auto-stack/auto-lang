这份文档将我们之前所有的架构推演、语法设计和工程哲学融为一体，正式确立 **AURA (Auto UI Representation Abstract)** 作为 Auto 语言跨端 UI 编译器的核心规范。

---

# 核心架构设计文档：AURA (Auto UI Representation Abstract)

**版本**: v1.0
**定位**: AutoUI 的官方中间表达层 (UI-IR)
**设计哲学**: 结构与逻辑绝对解耦、状态精确制导、跨端编译零开销。

## 1. 什么是 AURA？

AURA 是 Auto 编译器前端从 AutoUI 源码中**抽取 (Extract)** 出来的静态、高度结构化的中间表示。
它不是普通的语法树（AST），它是剥离了控制流和复杂语法糖后，只保留了 **“UI 骨架 (View)”**、**“响应式状态 (Model)”** 和 **“事件路由 (Msg)”** 的纯粹数据描述。

在内存中，它是强类型的 Rust 结构体；序列化后，它是遵循 Atom 格式的结构化文本。

---

## 2. 场景管道 (The Scenario Pipeline)

AURA 的生成完全由工程上下文（`pac.at`）驱动，避免污染 Auto 语言的核心全局语法。

### 2.1 触发机制

当且仅当 CLI 读取到 `scenario: "ui"`（通过 `pac.at` 或命令行 `-s ui`）时，编译器启动 UI 扩展管线。

### 2.2 词法与语法隔离 (Contextual Parsing)

* Lexer 将 `widget`, `view`, `model`, `on`, `msg` 视为普通标识符。
* Parser 检查 `Session.scenario == UI`，在此条件下，顶级作用域遇到 `widget` 标识符时，将其提升为**上下文关键字 (Contextual Keyword)**，并生成一等公民的 AST 节点 `WidgetDecl`。

---

## 3. 表层语法层 (The AutoUI Surface)

开发者编写的代码。强调高内聚、局部推导和状态的安全隔离。

```auto
// 文件: Counter.at
widget Counter {
    // 1. 局部事件枚举 (编译后展开为 Counter_Msg)
    msg Msg { Inc, Dec }

    // 2. 状态/属性声明 (外部可通过 Counter(initial_count: 0) 传入)
    model {
        count int = 0 // 默认值
    }

    // 3. 静态视图树 (基于 Atom 格式的扩展语法)
    view {
        col {
            // .Inc 隐式成员推导
            button + { onclick: .Inc } 
            
            // ${.count} 状态显式追踪 (前缀 . 防止变量遮蔽)
            h2 > Current Count: ${.count} 
            
            button - { onclick: .Dec }
        }
    }

    // 4. 事件处理器 (MVU 架构的 Reducer)
    on {
        .Inc => { .count += 1 }
        .Dec => { .count -= 1 }
    }
}

```

---

## 4. 抽取层：AURA 规范 (The Extraction & Specification)

编译器前端不对 `widget` 进行降级，而是直接从 `WidgetDecl` AST 中提取并转换出 AURA 结构。

### 4.1 内存形态 (Rust 侧数据结构)

```rust
// 核心 AURA 数据结构定义
pub struct AuraWidget {
    pub name: String,
    
    // 状态定义：提取出的纯粹状态签名
    pub state_vars: Vec<AuraStateDef>, 
    
    // 视图树：纯粹的布局与绑定，没有任何逻辑
    pub view_tree: AuraNode,
    
    // 逻辑载荷：保留为 AST 块或编译为 Bytecode
    pub handlers: HashMap<String, LogicPayload>, 
}

pub struct AuraStateDef {
    pub name: String,       // e.g., "count"
    pub type_info: Type,    // e.g., Type::Int
    pub initial: Expr,      // 初始值 AST
}

pub enum AuraNode {
    Element {
        tag: String,                 // e.g., "col", "button"
        props: HashMap<String, Expr>,// 包含动态绑定的表达式
        events: HashMap<String, String>, // e.g., {"onclick": "Msg::Inc"}
        children: Vec<AuraNode>,
    },
    Text(String),
}

pub enum LogicPayload {
    AstBlock(Vec<Stmt>),    // 供 AOT 转译器使用 (如 React/Compose)
    Bytecode(Vec<u8>),      // 供 AutoVM 动态执行使用
}

```

### 4.2 序列化形态 (Atom 格式化输出)

用于调试、跨语言工具链调用，或 AI 智能生成。

```atom
Widget {
    name: "Counter",
    states: [
        { name: "count", type: "int", default: 0 }
    ],
    view: Node {
        tag: "col",
        children: [
            Node { tag: "button", events: { onclick: Dispatch("Msg::Inc") }, children: ["+"] },
            Node { tag: "h2", props: { text: Expr("Concat('Current Count: ', .count)") } }
        ]
    },
    // handlers 作为附属物附加
}

```

---

## 5. 后端转译层 (The Transpilation Backends)

AURA 生成后，根据 `pac.at` 中的 `backend` 配置，分发给不同的转译器。AURA 的高度解耦让代码生成变得极其简单和机械化。

### 5.1 转译目标 A：React (TypeScript)

`auto-react-transpiler` 的工作流：

1. **翻译 Model**：遍历 `state_vars`，将 `count` 翻译为 `const [count, setCount] = useState(0);`。
2. **改写 Handlers**：接管 `LogicPayload::AstBlock`。
* 遇到 AST 节点 `AssignOp( +=, ".count", 1 )`。
* 重写为 React 模式：`setCount(prev => prev + 1);`。


3. **生成 JSX**：递归遍历 `view_tree`，遇到 `tag: "col"` 生成 `<div className="flex-col">`。遇到事件绑定，直接连线到生成的 Handler 函数。

### 5.2 转译目标 B：Jetpack Compose (Kotlin)

`auto-compose-transpiler` 的工作流：

1. **翻译 Model**：生成 `var count by remember { mutableStateOf(0) }`。
2. **改写 Handlers**：生成 Kotlin 的 Lambda `val handleInc = { count += 1 }`（由于 Compose 支持直接赋值，无需改写为 `setCount` 风格）。
3. **生成 View**：递归遍历，`col` 生成 `Column { ... }`，`button` 生成 `Button(onClick = handleInc) { ... }`。

### 5.3 转译目标 C：动态宿主 (GPUI + AutoVM)

`backend: dynamic` 的工作流：

1. **编译 Handlers**：调用核心编译器，将 `on` 块编译为 AutoVM 的 `LogicPayload::Bytecode`。
2. **下发 AURA**：将 AURA 的序列化格式和 Bytecode 发送给 GPUI 客户端。
3. **运行时渲染**：GPUI 根据 AURA 构建原生控件。当按钮被点击时，触发对应的 Bytecode 交由内嵌的 AutoVM 执行，AutoVM 修改状态后通知 GPUI 重绘。

---

## 6. 简化语法层 (Simplified Syntax Layer)

为了提升开发体验，AURA 在表层语法层提供了一系列语法糖，让开发者可以用更简洁的方式表达常见的 UI 模式。这些简化在解析阶段被转换为标准的 AURA 结构。

### 6.1 核心简化原则

**内容优先于样式**：子元素和内容应该出现在样式属性之前，提高代码可读性。

```auto
// ✅ 推荐：内容在前，样式在后
col {
    text "Hello" {
        style: "text-xl"
    }
    style: "p-4 bg-white"
}

// ❌ 不推荐：样式混杂在参数中
col (style: "p-4 bg-white") {
    text (style: "text-xl", text: "Hello") {}
}
```

### 6.2 center 组件 (居中容器语法糖)

`center` 是 `col` 的语法糖，自动添加居中相关的 Tailwind 样式。

**语法**:
```auto
center {
    // children
    style: "optional-extra-styles"  // 可选，会追加到默认样式
}
```

**转换规则**:
```auto
// 输入
center {
    text "Hello"
    style: "bg-white"
}

// 转换为
col {
    text "Hello"
    style: "w-full h-full justify-center items-center bg-white"
}
```

**默认样式**:
- `w-full` - 宽度100%
- `h-full` - 高度100%
- `justify-center` - 垂直居中
- `items-center` - 水平居中

**后端映射**:
| 后端 | 生成代码 |
|------|----------|
| ArkTS | `Column().width('100%').height('100%').alignItems(HorizontalAlign.Center).justifyContent(FlexAlign.Center)` |
| Jetpack Compose | `Column(modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center))` |
| React | `<div className="w-full h-full flex justify-center items-center">` |

### 6.3 text 主属性简写 (Primary Prop Shorthand)

`text` 组件的 `text` 属性是主属性，可以直接使用字符串字面量。

**语法**:
```auto
text "content" { style: "..." }
```

**转换规则**:
```auto
// 输入
text "Hello, World!" {
    style: "text-2xl font-bold"
}

// 等价于
text (text: "Hello, World!") {
    style: "text-2xl font-bold"
}
```

**适用组件**:
以下组件的主属性都是 `text`，支持相同的简写语法：

| 组件 | 主属性 | 示例 |
|------|--------|------|
| `text` | text | `text "Hello"` |
| `h1`-`h6` | text | `h1 "Title"` |
| `p` | text | `p "Paragraph"` |
| `button` | text | `button "Click me"` |
| `label` | text | `label "Name:"` |
| `a` / `link` | text | `a "Learn more"` |

**注意事项**:
- 主属性简写仅适用于**字符串字面量**
- 动态绑定仍需使用完整语法：`text (text: .message)`
- 主属性简写后仍可添加 `{ style: ... }` 块

### 6.4 style 属性后置 (Trailing Style Syntax)

样式属性（尤其是 `style`）可以放在元素体中的子元素之后。

**语法**:
```auto
element {
    child1
    child2
    style: "tailwind-styles"
    onclick: .Handler  // 事件也可以后置
}
```

**解析规则**:
1. 解析元素标签
2. 解析参数列表 `(props)` 如果存在
3. 解析 `{ }` 体：
   - 如果看起来像 `key: value`（标识符后跟冒号），解析为属性/事件
   - 否则解析为子元素
4. 属性和子元素可以混合，但建议子元素在前

**示例**:
```auto
// 完整示例
col {
    text "Welcome" {
        style: "text-2xl font-bold"
    }
    text "Subtitle" {
        style: "text-gray-600"
    }
    style: "p-4 bg-white rounded-lg"
}
```

### 6.5 常用组件简化模式

#### Hello World 模式
```auto
widget App {
    view {
        center {
            text "Hello, World!" {
                style: "text-2xl font-bold"
            }
            style: "bg-white"
        }
    }
}
```

#### 列表项模式
```auto
row {
    image (src: .avatar, style: "w-12 h-12 rounded-full") {}
    col {
        text .name {
            style: "font-medium"
        }
        text .email {
            style: "text-sm text-gray-500"
        }
    }
    style: "p-3 items-center"
}
```

#### 卡片模式
```auto
col {
    image (src: .cover, style: "w-full h-32 object-cover") {}
    col {
        h3 .title {}
        p .description {
            style: "text-gray-600"
        }
    }
    style: "bg-white rounded-xl overflow-hidden shadow-md"
}
```

### 6.6 语法糖对照表

| 简化语法 | 标准语法 | 说明 |
|----------|----------|------|
| `center { }` | `col (style: "w-full h-full justify-center items-center") { }` | 居中容器 |
| `text "content"` | `text (text: "content")` | 主属性简写 |
| `button "Label"` | `button (text: "Label")` | 主属性简写 |
| `h1 "Title"` | `h1 (text: "Title")` | 主属性简写 |
| `{ child style: "..." }` | `(style: "...") { child }` | 样式后置 |

---

## 7. 架构收益总结

1. **无状态视图的纯粹性**：AURA 的 `view_tree` 绝对干净，不包含任何业务逻辑，这为未来的可视化编辑器（Visual Editor）双向同步铺平了道路。
2. **极低的转译器开发成本**：由于复杂的类型推导、作用域解析和宏展开已经在前端生成 AURA 时完成，新增一个目标平台（例如 Flutter/Dart 后端）只需编写一个几百行的 AURA 遍历生成器即可。
3. **多端性能极致**：AOT（提前编译）路线生成的是目标平台最原生、最符合 Best Practice 的代码，完全没有类似跨端框架的巨大运行时损耗。
4. **简化语法的渐进式采用**：开发者可以根据需要选择简化语法或完整语法，不会影响最终生成的代码质量。
