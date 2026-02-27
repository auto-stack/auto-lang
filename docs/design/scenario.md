# 设计文档：Auto 语言“面向场景编程”与 UI 编译架构

**主题**：基于上下文环境的语法解析与 UI-IR 生成架构
**目标**：在保持 Auto 语言核心编译器轻量、类型安全的前提下，优雅地支持 `UI`、`Shell` 等特定领域语法（方言）。确保全局命名空间不被污染，同时为 LSP（语言服务器）和多端转译器提供精准的上下文。

---

## 1. 架构概述 (Architecture Overview)

“面向场景编程”的核心理念是**环境注入 (Environment Injection) 与 上下文关键字 (Contextual Keywords)**。

编译器不再试图用一个包罗万象的庞大 Parser 来理解所有语法，而是将编译过程视为一个受**全局场景状态 (Scenario State)** 驱动的状态机。UI 独有的概念（如 `widget`, `view`）作为第一类公民（First-Class Citizens）保留在特定场景的 AST 中，拒绝过早降级（Desugaring），从而实现到 UI-IR（Atom 格式）的无损抽取。

---

## 2. 工程配置层：`pac.at` (The Source of Truth)

为了让 IDE (LSP) 能够无歧义地提供代码补全和语法高亮，项目的编译场景必须在工程的根配置文件中明确声明。`pac.at` 使用 Auto 语言原生的语法定义：

```auto
// pac.at (前端 UI 工程示例)
name: "my-counter-app"
version: "1.0.0"
scenario: "ui"      // 核心标识：声明当前工程使用 UI 方言
backend: "vue"    // 声明编译目标转译器

```

```auto
// pac.at (后端或核心库工程示例)
name: "my-core-service"
version: "1.0.0"
scenario: "core"    // 标准 Auto 语言环境
backend: "a2r"      // 编译为 Rust

```

---

## 3. CLI 调度与编译器会话 (CLI & Compiler Session)

统一使用 `auto` 作为入口。CLI 支持通过命令行 Flag 显式覆盖场景配置（适用于单文件脚本或 CI/CD 流水线）。

### 3.1 命令行接口设计

* **标准编译**：`auto build src/main.at` （默认读取 `pac.at` 中的 `scenario`，若无则默认为 `core`）
* **显式指定场景**：`auto build -s ui src/App.at`

### 3.2 核心会话机制 (Compiler Session)

在编译器启动时，构建全局的 `CompilerSession`，向下游传递场景信息，绝对避免使用全局静态变量。

```rust
// auto-core/src/session.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    Core,
    UI,
    Shell,
}

pub struct CompilerSession {
    pub scenario: Scenario,
    // 可扩展：错误收集器、SourceMap 引用等
}

```

---

## 4. 解析器架构：软隔离与上下文关键字 (Parser & Contextual Keywords)

这是保证 Auto 语言普通代码不出错的关键。在 Lexer 层面，`widget`、`view`、`model` 仅仅是普通的 `Identifier`。

Parser 在工作时，必须校验当前的 `Session.scenario`：

```rust
// auto-core/src/parser.rs
impl<'a> Parser<'a> {
    pub fn new(source: &'a str, session: CompilerSession) -> Self { ... }

    fn parse_statement(&mut self) -> Result<AstNode> {
        let token = self.peek();

        // 仅在 UI 场景下，拦截特定的 Identifier 作为关键字
        if self.session.scenario == Scenario::UI {
            match token.as_str() {
                "widget" => return self.parse_widget_decl(),
                // ... 处理其他 UI 独有语法
                _ => {}
            }
        }

        // 如果不是 UI 场景，或者标识符不匹配，走常规 Auto 语句解析
        self.parse_expression_statement()
    }
}

```

**优势**：在 `scenario: "core"` 的后端项目中，开发者完全可以定义 `let widget = create_window();` 而不会触发任何语法冲突。

---

## 5. AST 设计与 UI-IR 抽取 (AST & Extraction Pipeline)

为了保证 UI 声明的意图不丢失，AST 必须原生支持 UI 节点。

### 5.1 原生 UI AST 节点定义

```rust
// auto-core/src/ast.rs
pub enum AstNode {
    FunctionDecl(FuncDecl),
    ClassDecl(ClassDecl),
    // 场景扩展节点
    WidgetDecl(WidgetDecl), 
}

pub struct WidgetDecl {
    pub name: String,
    pub model: Vec<FieldDef>,        // 状态声明
    pub view: ViewBlock,             // 原生视图树结构
    pub on_handlers: Vec<OnHandler>, // 事件逻辑块
}

```

### 5.2 编译流水线 (The Pipeline)

当 `auto build -s ui` 执行时，流水线严格分为三个阶段：

1. **解析 (Parsing)**：生成包含 `WidgetDecl` 的原生 AST。TypeChecker 会对 `.count` 和 `.Inc` 进行自顶向下的精确类型检查。
2. **抽取 (Extraction to UI-IR)**：编译器提取 `model` 和 `view` 节点，**1:1 无损映射**为结构化的 Atom 格式文本/数据结构（UI-IR）。
3. **转译/编译 (Backend Dispatch)**：
* `on_handlers` 内部的逻辑代码被编译为 Auto ByteCode (ABC) 或生成目标语言的 AST 片段。
* 后端转译器（如 `auto-vue-transpiler`）接管 UI-IR 和逻辑载荷，生成最终的 Kotlin/JS 代码。



---

## 6. LSP (语言服务器) 行为规范

LSP 的稳定运行依赖于这套架构：

1. **初始化**：当 VSCode 打开工程时，LSP 扫描项目根目录的 `pac.at`。
2. **环境绑定**：读取到 `scenario: "ui"` 后，LSP 在内存中实例化一个 `CompilerSession { scenario: Scenario::UI }`。
3. **服务提供**：后续的 Diagnostics (报错)、Hover (悬停)、Completion (补全) 全部经由挂载了 UI 插件的 Parser 处理。

---

目前编译器前端（解析和 IR 生成）的基建蓝图已经非常完整了。接下来，是否需要拿之前的 `Counter` 例子，实战演练一下后端的转译过程？例如演示 `auto-vue-transpiler` 是如何读取这颗包含状态的 AST/UI-IR，并最终生成带有 `useState` 和 `dispatch` 的现代 vue 源码的？