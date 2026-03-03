这是一份为您梳理并系统化的 **AutoDown 架构与语法设计文档**。它将我们推演出的所有核心理念、符号法则以及底层编译机制进行了彻底的提炼，可直接作为 Auto 语言生态的官方技术白皮书。

---

# AutoDown 设计文档：面向多端编译的现代结构化文档方言

**版本**: v1.0
**隶属**: Auto 语言生态系统
**定位**: 文本主导（Text-Dominant）的排版与文档生成 DSL
**核心基建**: 基于 AURA (抽象 UI/文档表示层) 和 Flip（模式翻转）机制

## 1. 设计哲学与愿景 (Philosophy & Vision)

AutoDown 的诞生是为了解决现代技术写作与排版中的“心智割裂”与“格式孤岛”问题。
传统的 Markdown 表达力极其有限（无法胜任严肃排版），而 LaTeX/Typst 的语法又与现代高级编程语言脱节。

AutoDown 的核心哲学是：

1. **文本第一公民 (Text-First)**：提供零心智负担的沉浸式写作体验，兼容 Markdown 的核心直觉。
2. **逻辑大一统 (Logic Singularity)**：一旦需要复杂的逻辑控制、组件调用或变量插值，无缝切换为 100% 纯正的 Auto 语言，消除 UI 编程与文档排版的割裂感。
3. **AST 降维打压 (AST-Driven)**：一切文档最终在内存中“翻转 (Flip)”为严谨的 Auto 抽象语法树（AURA），从而实现一次编写，多端转译（PDF、Word、Web）。

---

## 2. 核心语法法则：符号三权分立 (The Ultimate Symbol Law)

为了保证解析器的高效与用户视觉的绝对干净，AutoDown 仅引入三个核心逃逸符号。其余普通文本及轻量排版（如 `*粗体*`，`- 列表`）完全沿用 Markdown 标准。

### 2.1 `#` —— 标题域 (Header)

专职负责文档层级，完美兼容 Markdown 肌肉记忆。

```markdown
# 一级标题
## 二级标题

```

### 2.2 `$` —— 逻辑域 (Auto Logic Singularity)

只要看到 `$`，即代表 Auto 编译器接管了当前上下文，进入严谨的代码解析模式。

* **内联变量/表达式插值**：使用 `${ ... }`。
```markdown
当前的系统熵值为 ${system.entropy}，迭代次数为 ${count + 1}。

```


* **控制流指令**：直接跟随保留字。
```markdown
$if data.is_valid {
    数据校验通过，可以进行演进。
} $else {
    警告：数据异常。
}

$for user in .users {
    - 参与者：${user.name}
}

```


* **组件调用与尾随闭包 (Trailing Closures)**：
调用 Auto 语言中的函数或 UI 甚至排版组件。如果组件最后一个参数是视图/内容块，直接使用 `{ ... }` 包裹纯文本。
```markdown
$callout(type: "warning") {
    请注意，这是一个**高风险**的操作。
}

```



### 2.3 `%{ ... }%` —— 数学排版域 (AutoMath)

彻底摒弃 LaTeX 反人类的冗余语法（`\`、`{}` 嵌套）和 Markdown 中极易与文本冲突的 `%...%` 或 `$公式$`。

* **统一的内外形态**：无论行内还是块级公式，均使用 `%{ ... }%`。
* **函数式降维**：公式内部完全遵循 Auto 语言的函数与操作符语义。

```markdown
行内公式如 %{ E = 1/2 * m * v^2 }% 所示。

微观粒子配分函数的块级表达为：
%{
    Z = sum(i=0 .. infinity, exp(-E_i / (k * T)))
}

```

---

## 3. 底层机制：Flip (模式翻转) 预处理管线

AutoDown 并不是一个孤立的渲染引擎，而是一个**分块状态机前置预处理器**。

### 3.1 词法状态机工作流

1. **Text Mode (默认)**：解析器以段落、列表、普通文本的方式读取字符。
2. **Flip to Code**：当遇到 `$` 或 `%{`，解析器立即将后续 Token 的控制权移交给 Auto 语言核心 Parser。
3. **Flip to Text**：在尾随闭包 `{ ... }` 的内部，解析器再次翻转回 Text Mode，将里面的内容包装为 Auto 语言的 `view { ... }` 节点。

### 3.2 AST 映射等价性

任何 AutoDown 源码，经过 Flip 预处理后，都会变成一棵等价的纯 Auto 语言代码树。

**AutoDown 源码:**

```markdown
$set page(paper: "a5")

# 标题分析
这是一个测试：%{ a + b }%。

```

**翻转后的等价 Auto AST 伪代码:**

```auto
document {
    style {
        page.set(paper: "a5")
    }
    h1 > 标题分析
    p {
        "这是一个测试："
        math { a + b }
        "。"
    }
}

```

---

## 4. 后端转译架构 (Multi-Backend Transpilation)

基于前置的 Flip 机制，Auto 编译器核心（TypeChecker 等）拿到的全是标准的结构化数据（AURA 树）。通过挂载不同的转译器（Transpilers），实现降维打击：

### 4.1 转译目标 A：学术与出版级排版 (Typst PDF)

* **引擎**：`auto-typst-transpiler`
* **机制**：将 AURA 树映射为 Typst 原生脚本语言（`.typ`），并可直接通过 Rust 嵌入的 Typst 核心库，在毫秒级时间内生成精美的 PDF 文件。
* **AutoMath 映射**：`math { sum(...) }` 自动翻译为 Typst 的 `$ sum_(...) $`。

### 4.2 转译目标 B：企业级办公文档 (DOCX Word)

* **引擎**：`auto-docx-transpiler`
* **机制**：借助 Rust 的 `docx-rs` 库，直接将 AURA 的 `h1`, `p`, 表格等节点，精确映射并封装为 OOXML 格式，注入到预设的 Word 模板中。
* **AutoMath 映射**：将函数式的数学 AST 直接翻译为 Word 原生支持的 MathML，实现 Word 内的公式可编辑。

### 4.3 转译目标 C：现代 Web 框架 (React / Vue)

* **引擎**：`auto-react-transpiler`
* **机制**：将文档树降级为包含 HTML 标签的 JSX/TSX 组件。配合 Tailwind CSS，直接生成静态博客、企业知识库或交互式技术文档。

---

## 5. 商业场景扩展：基于 AI 的逆向模板生成

得益于 AutoDown 的高度结构化和文本可读性，它天然适合作为**大型语言模型（LLM）与企业复杂排版格式之间的中间件**。

1. **解析 Word**：读取客户提供的复杂 Word 模板（包含特殊颜色批注）。
2. **生成 AutoDown 模板**：将批注转化为 AutoDown 中的变量占位符 `${.data}` 和大模型 Prompt。
3. **AI 填空与生成**：LLM 输出结构化的 AutoDown 文本。
4. **无损还原**：`auto-docx-transpiler` 将包含真实数据的 AURA 树重新塞回原 Word 模板，实现 100% 格式还原的自动化文档生成系统。

---

**总结**
AutoDown 是 Auto 语言生态中补全“图文排版”版图的关键拼图。通过极简的符号法则和强大的底层模式翻转机制，它不仅重塑了开发者的书写体验，更为跨格式的数字内容生成建立了一套统一的底层标准。