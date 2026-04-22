# Plan 212: LSP + VSCode Extension 现代化 — 同步 Auto 语言最新特性

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 VSCode 打开 `.at` 文件不再报红，补全/悬停/跳转覆盖所有 Auto 关键字，TextMate 语法高亮完整支持 f-string、三引号、`is`/`spec`/`ext` 等新语法。

**Architecture:** 三层改进：(1) TextMate grammar 重写为 Auto 专用语法（当前是从 Rust grammar 复制的，很多 Rust 概念不适用）；(2) LSP 补全关键词与 token.rs 同步；(3) LSP 增强 document symbols 和 snippet。TextMate 优先级最高，因为它直接决定用户打开文件是否报红。

**Tech Stack:** TextMate grammar (JSON), Rust (auto-lsp), JavaScript (VSCode extension)

**Depends on:** auto-lang lexer/parser (同仓库，已完成)

---

## Phase 1: TextMate Grammar 重写（P0 — 解决报红）

### Task 1: 更新 keywords 部分

**Files:**
- Modify: `auto-vscode/vscode-extension/syntaxes/auto.tmLanguage.json:662-845`

**当前问题：** keywords 部分包含大量 Rust 关键字（`match`, `trait`, `struct`, `impl`, `unsafe`, `dyn`, `become`, `box`, `final`, `gen`, `override`, `priv`, `typeof`, `unsized`, `virtual`, `where`, `yield`, `try`, `do`, `when`, `abstract`, `ref`），缺少 Auto 专有关键字。

**修改内容：**

1. **control flow keywords** — 替换为 Auto 实际关键字：
```
await|break|continue|else|for|if|in|is|loop|return
```
移除: `do`, `match`, `try`, `when`, `yield`

2. **storage keywords** — 保留 `let`, `mut`, `var`，已正确

3. **declaration keywords** — 合并为一个 pattern：
```json
{
  "comment": "declaration keywords",
  "name": "keyword.declaration.auto",
  "match": "\\b(type|enum|struct|spec|ext|impl|union|tag|alias|static|shared|node|dep|routes|outlet|link|route|nav|widget|model|view|style|on|grid)\\b"
}
```

4. **other keywords** — 替换为 Auto 关键字：
```
as|has|import|pub|use|view|move|copy|take|hold|task|spawn|reply|go|pac|super|to|nil|null
```
移除: `async`, `become`, `box`, `dyn`, `final`, `gen`, `override`, `priv`, `ref`, `typeof`, `unsafe`, `unsized`, `virtual`, `where`

5. **移除 "auto less keywords"** (line 706-708) 和 **ui keywords** (line 710-713) — 已合并到 declaration keywords

6. **移除 Rust 特有 section：**
   - `mod` keyword pattern (line 119-127)
   - `extern crate` import (line 131-160)
   - `macro_rules!` (line 99-115)
   - macro metavariables (line 49-96)
   - `crate` keyword (line 721-723)

7. **保留不变：**
   - `fn` keyword
   - `pac` keyword
   - `mut` modifier
   - `const` keyword

**Verify:** 在 VSCode 中打开包含 `is`, `spec`, `ext`, `view`, `move`, `None`, `Some`, `Ok`, `Err` 的 .at 文件，确认不再报红。

---

### Task 2: 添加 f-string 和三引号字符串语法

**Files:**
- Modify: `auto-vscode/vscode-extension/syntaxes/auto.tmLanguage.json:1087-1165`

**当前问题：** strings 部分只有 `"..."` 和 `'...'`，缺少 f-string (`f"..."`)、三引号 (`"""..."""`)、backtick (`` `...` ``)、c-string (`c"..."`)。

**修改内容：** 在 `strings` repository 的 patterns 开头添加（在现有双引号 string 之前）：

```json
{
  "comment": "triple-quoted f-strings f\"\"\"...\"\"\"",
  "name": "string.quoted.triple.auto",
  "begin": "f\"\"\"",
  "beginCaptures": {
    "0": { "name": "punctuation.definition.string.begin.auto string.quoted.triple.auto" }
  },
  "end": "\"\"\"",
  "endCaptures": {
    "0": { "name": "punctuation.definition.string.end.auto string.quoted.triple.auto" }
  },
  "patterns": [
    { "include": "#fstring-escapes" },
    { "include": "#fstring-interpolation" }
  ]
},
{
  "comment": "triple-quoted strings \"\"\"...\"\"\"",
  "name": "string.quoted.triple.auto",
  "begin": "\"\"\"",
  "beginCaptures": {
    "0": { "name": "punctuation.definition.string.begin.auto" }
  },
  "end": "\"\"\"",
  "endCaptures": {
    "0": { "name": "punctuation.definition.string.end.auto" }
  },
  "patterns": [
    { "include": "#escapes" }
  ]
},
{
  "comment": "f-strings f\"...\"",
  "name": "string.interpolated.auto",
  "begin": "(f)\",
  "beginCaptures": {
    "1": { "name": "keyword.other.interpolation.auto" },
    "2": { "name": "punctuation.definition.string.begin.auto" }
  },
  "end": "\"",
  "endCaptures": {
    "0": { "name": "punctuation.definition.string.end.auto" }
  },
  "patterns": [
    { "include": "#fstring-escapes" },
    { "include": "#fstring-interpolation" }
  ]
},
{
  "comment": "c-strings c\"...\"",
  "name": "string.quoted.other.auto",
  "begin": "(c)\",
  "beginCaptures": {
    "1": { "name": "keyword.other.cstring.auto" },
    "2": { "name": "punctuation.definition.string.begin.auto" }
  },
  "end": "\"",
  "endCaptures": {
    "0": { "name": "punctuation.definition.string.end.auto" }
  }
},
{
  "comment": "backtick strings `...`",
  "name": "string.interpolated.backtick.auto",
  "begin": "`",
  "beginCaptures": {
    "0": { "name": "punctuation.definition.string.begin.auto" }
  },
  "end": "`",
  "endCaptures": {
    "0": { "name": "punctuation.definition.string.end.auto" }
  },
  "patterns": [
    { "include": "#fstring-interpolation" }
  ]
}
```

添加新的 repository entries：

```json
"fstring-escapes": {
  "comment": "escape sequences inside f-strings",
  "patterns": [
    { "name": "constant.character.escape.auto", "match": "\\\\[ntr0\\\\\"']" },
    { "name": "constant.character.escape.auto", "match": "\\{\\{", "comment": "escaped {" },
    { "name": "constant.character.escape.auto", "match": "\\}\\}", "comment": "escaped }" }
  ]
},
"fstring-interpolation": {
  "comment": "f-string interpolation $var and ${expr}",
  "patterns": [
    {
      "name": "meta.interpolation.auto",
      "begin": "\\$\\{",
      "beginCaptures": { "0": { "name": "punctuation.definition.interpolation.begin.auto" } },
      "end": "\\}",
      "endCaptures": { "0": { "name": "punctuation.definition.interpolation.end.auto" } },
      "patterns": [
        { "include": "#keywords" },
        { "include": "#constants" },
        { "include": "#types" },
        { "include": "#variables" }
      ]
    },
    {
      "name": "meta.interpolation.variable.auto",
      "match": "\\$([a-zA-Z_][a-zA-Z0-9_]*)",
      "captures": {
        "1": { "name": "variable.other.auto" }
      }
    }
  ]
}
```

**Verify:** 打开包含 `f"hello $name"`, `f"""multi\nline"""`, `` `raw string` `` 的 .at 文件。

---

### Task 3: 更新类型高亮

**Files:**
- Modify: `auto-vscode/vscode-extension/syntaxes/auto.tmLanguage.json:924-1037`

**修改内容：**

1. **Auto 原始类型** — 替换 primitive types (line 980-982)：
```json
{
  "comment": "Auto primitive types",
  "name": "entity.name.type.primitive.auto",
  "match": "\\b(int|uint|byte|float|double|bool|char|str|void|nil|cstr)\\b"
}
```

2. **Auto Option/Result 字面量** — gtypes section (line 1039-1051) 已经有 `Some|None` 和 `Ok|Err`，保留不变 ✅

3. **移除 Rust 特有类型** — 从 numeric types (line 927-933) 中移除 `i128`, `u128`, `isize`，添加 Auto 风格的 `int`, `uint`, `byte`, `float`, `double`：
```json
{
  "comment": "numeric types",
  "match": "(?<![A-Za-z])(f32|f64|i16|i32|i64|i8|u16|u32|u64|u8|usize|byte)\\b",
  "captures": {
    "1": { "name": "entity.name.type.numeric.auto" }
  }
}
```

4. **Auto 特殊类型标记** — 添加 `~` (async marker), `?` (Option), `!` (Result)：
```json
{
  "comment": "type modifiers",
  "match": "[~!?](?=[A-Za-z])",
  "name": "keyword.operator.type.modifier.auto"
}
```

**Verify:** 打开包含 `fn foo() !int`, `let x ?int`, `fn bar() ~int` 的 .at 文件。

---

### Task 4: 更新运算符

**Files:**
- Modify: `auto-vscode/vscode-extension/syntaxes/auto.tmLanguage.json:735-845`

**修改内容：**

1. **添加 `.?` 错误传播运算符** — 在 dot access 之后：
```json
{
  "comment": "error propagation operator",
  "name": "keyword.operator.propagate.auto",
  "match": "\\.\\?"
}
```

2. **添加 `..=` 包含范围** — 已在 range pattern 中有 `\\.{2}(=|\\.)?`，这已经覆盖 `..=` ✅

3. **移除 `::` namespace operator** — Auto 用 `.` 不用 `::`：
```json
// 移除 line 792-794 的 namespace operator pattern
```

4. **移除 `@` subpattern binding** — Auto 不用这个

5. **添加 `~` async type marker**：
```json
{
  "comment": "async/future type marker",
  "name": "keyword.operator.async.auto",
  "match": "~(?=[A-Za-z])"
}
```

**Verify:** 打开包含 `val.?', `0..=10`, `0..10` 的 .at 文件。

---

### Task 5: 添加 compile-time tokens

**Files:**
- Modify: `auto-vscode/vscode-extension/syntaxes/auto.tmLanguage.json`

**修改内容：** 在主 patterns 中添加 compile-time 关键字（在 `#attributes` 之后）：

```json
{
  "comment": "compile-time directives",
  "name": "keyword.control.compiletime.auto",
  "match": "#(if|for|is)(?=\\b)"
},
{
  "comment": "compile-time expression block",
  "name": "keyword.control.compiletime.auto",
  "match": "#\\{"
}
```

**Verify:** 打开包含 `#if`, `#for`, `#is`, `#{` 的 .at 文件。

---

### Task 6: 提交 Phase 1

```
feat(vscode): rewrite TextMate grammar for Auto language (Plan 212 Phase 1)
```

**验证方法：**
1. 在 auto-vscode 目录运行 `npm run compile` 或复制 tmLanguage 到扩展目录
2. 用 VSCode 打开 `crates/auto-lang/test/vm/09_functions/010_closure_hof_map/closure_hof_map.at`
3. 确认 `is`, `fn`, `let`, `f"..."` 等不再报红
4. 确认 `Some`, `None`, `Ok`, `Err` 正确高亮
5. 确认 `#if`, `#for` 等 compile-time 语法不报红

---

## Phase 2: LSP 补全关键词同步（P1）

### Task 7: 更新 LSP keyword_completions

**Files:**
- Modify: `crates/auto-lsp/src/completion.rs:130-160`

**当前问题：** 只有 24 个关键字，缺少 Auto 核心关键字。

**新 keyword_completions 函数：**

```rust
fn keyword_completions() -> Vec<CompletionItem> {
    vec![
        // Declarations
        completion_item("fn", "Define a function", CompletionItemKind::FUNCTION, "fn ${1:name}() {\n    \n}"),
        completion_item("let", "Declare immutable variable", CompletionItemKind::KEYWORD, "let ${1:name} = ${2:value}"),
        completion_item("var", "Declare mutable variable", CompletionItemKind::KEYWORD, "var ${1:name} = ${2:value}"),
        completion_item("const", "Declare constant", CompletionItemKind::CONSTANT, "const ${1:name} = ${2:value}"),
        completion_item("mut", "Mutable modifier", CompletionItemKind::KEYWORD, "mut ${1:name} = ${2:value}"),
        completion_item("pub", "Public visibility", CompletionItemKind::KEYWORD, "pub "),
        // Type declarations
        completion_item("type", "Define a type", CompletionItemKind::KEYWORD, "type ${1:Name} {\n    \n}"),
        completion_item("enum", "Define an enum", CompletionItemKind::ENUM, "enum ${1:Name} {\n    ${2:Variant}\n}"),
        completion_item("spec", "Define a spec/trait", CompletionItemKind::INTERFACE, "spec ${1:Name} {\n    fn ${2:method}()\n}"),
        completion_item("ext", "Extend a type or impl a spec", CompletionItemKind::KEYWORD, "ext ${1:Type} {\n    \n}"),
        completion_item("impl", "Implement methods", CompletionItemKind::KEYWORD, "impl ${1:Type} {\n    \n}"),
        completion_item("alias", "Type alias", CompletionItemKind::KEYWORD, "alias ${1:Name} = ${2:Type}"),
        // Control flow
        completion_item("if", "If statement", CompletionItemKind::KEYWORD, "if ${1:condition} {\n    \n}"),
        completion_item("else", "Else statement", CompletionItemKind::KEYWORD, "else {\n    \n}"),
        completion_item("elif", "Else if statement", CompletionItemKind::KEYWORD, "elif ${1:condition} {\n    \n}"),
        completion_item("for", "For loop", CompletionItemKind::KEYWORD, "for ${1:item} in ${2:iterable} {\n    \n}"),
        completion_item("is", "Pattern matching", CompletionItemKind::KEYWORD, "is ${1:value} {\n    ${2:pattern} -> ${3:result}\n}"),
        completion_item("loop", "Infinite loop", CompletionItemKind::KEYWORD, "loop {\n    \n}"),
        completion_item("break", "Break from loop", CompletionItemKind::KEYWORD, "break"),
        completion_item("continue", "Continue loop", CompletionItemKind::KEYWORD, "continue"),
        completion_item("return", "Return from function", CompletionItemKind::KEYWORD, "return ${1:value}"),
        // Imports
        completion_item("use", "Import module", CompletionItemKind::KEYWORD, "use ${1:module}"),
        completion_item("pac", "Package root import", CompletionItemKind::KEYWORD, "pac.${1:module}"),
        completion_item("super", "Parent directory import", CompletionItemKind::KEYWORD, "super.${1:module}"),
        // Ownership
        completion_item("view", "Immutable borrow access", CompletionItemKind::KEYWORD, "view"),
        completion_item("move", "Ownership transfer", CompletionItemKind::KEYWORD, "move"),
        // Literal values
        completion_item("true", "Boolean true", CompletionItemKind::KEYWORD, "true"),
        completion_item("false", "Boolean false", CompletionItemKind::KEYWORD, "false"),
        completion_item("nil", "Nil value", CompletionItemKind::KEYWORD, "nil"),
        completion_item("None", "Option None", CompletionItemKind::ENUM_MEMBER, "None"),
        completion_item("Some", "Option Some", CompletionItemKind::ENUM_MEMBER, "Some(${1:value})"),
        completion_item("Ok", "Result Ok", CompletionItemKind::ENUM_MEMBER, "Ok(${1:value})"),
        completion_item("Err", "Result Err", CompletionItemKind::ENUM_MEMBER, "Err(${1:value})"),
        // Async/Concurrency
        completion_item("task", "Define a task", CompletionItemKind::KEYWORD, "task ${1:Name} {\n    \n}"),
        completion_item("spawn", "Spawn a task", CompletionItemKind::KEYWORD, "spawn ${1:task}"),
        completion_item("await", "Await async result", CompletionItemKind::KEYWORD, ".await"),
        completion_item("go", "Go keyword", CompletionItemKind::KEYWORD, "go "),
        // Storage
        completion_item("static", "Static storage", CompletionItemKind::KEYWORD, "static "),
        completion_item("shared", "Shared storage", CompletionItemKind::KEYWORD, "shared "),
    ]
}
```

**Verify:** `cargo build -p auto-lsp`

---

### Task 8: 更新 LSP type_completions

**Files:**
- Modify: `crates/auto-lsp/src/completion.rs:162-183`

**修改内容：** 替换 `array`, `list`, `dict`, `object` 为 Auto 实际类型：

```rust
fn type_completions(content: &str) -> Vec<CompletionItem> {
    let mut items = vec![
        // Primitive types
        completion_item("int", "Signed 32-bit integer", CompletionItemKind::TYPE_PARAMETER, "int"),
        completion_item("uint", "Unsigned 32-bit integer", CompletionItemKind::TYPE_PARAMETER, "uint"),
        completion_item("float", "Floating point number", CompletionItemKind::TYPE_PARAMETER, "float"),
        completion_item("double", "Double precision float", CompletionItemKind::TYPE_PARAMETER, "double"),
        completion_item("bool", "Boolean value", CompletionItemKind::TYPE_PARAMETER, "bool"),
        completion_item("str", "String", CompletionItemKind::TYPE_PARAMETER, "str"),
        completion_item("char", "Character", CompletionItemKind::TYPE_PARAMETER, "char"),
        completion_item("void", "Void/unit type", CompletionItemKind::TYPE_PARAMETER, "void"),
        completion_item("byte", "8-bit unsigned integer", CompletionItemKind::TYPE_PARAMETER, "byte"),
        // Sized integer types
        completion_item("i8", "8-bit signed integer", CompletionItemKind::TYPE_PARAMETER, "i8"),
        completion_item("i16", "16-bit signed integer", CompletionItemKind::TYPE_PARAMETER, "i16"),
        completion_item("i64", "64-bit signed integer", CompletionItemKind::TYPE_PARAMETER, "i64"),
        completion_item("u8", "8-bit unsigned integer", CompletionItemKind::TYPE_PARAMETER, "u8"),
        completion_item("u16", "16-bit unsigned integer", CompletionItemKind::TYPE_PARAMETER, "u16"),
        completion_item("u32", "32-bit unsigned integer", CompletionItemKind::TYPE_PARAMETER, "u32"),
        completion_item("u64", "64-bit unsigned integer", CompletionItemKind::TYPE_PARAMETER, "u64"),
        completion_item("usize", "Pointer-sized unsigned integer", CompletionItemKind::TYPE_PARAMETER, "usize"),
        // Collection types
        completion_item("List", "Growable list", CompletionItemKind::CLASS, "List<${1:T}>"),
        completion_item("Map", "Key-value dictionary", CompletionItemKind::CLASS, "Map<${1:K}, ${2:V}>"),
        completion_item("Option", "Optional value", CompletionItemKind::CLASS, "Option<${1:T}>"),
        completion_item("Result", "Result type", CompletionItemKind::CLASS, "Result<${1:T}, ${2:E}>"),
    ];
    items.extend(user_defined_types(content));
    items
}
```

**Verify:** `cargo build -p auto-lsp`

---

### Task 9: 清理 goto_def.rs 调试输出

**Files:**
- Modify: `crates/auto-lsp/src/goto_def.rs`

**修改内容：** 移除所有 `eprintln!` 调用（约 10 处，lines 20-66）。

**Verify:** `cargo build -p auto-lsp`

---

### Task 10: 提交 Phase 2

```
feat(lsp): sync completion keywords with Auto language (Plan 212 Phase 2)
```

---

## Phase 3: LSP 增强（P2）

### Task 11: 实现 Document Symbols

**Files:**
- Modify: `crates/auto-lsp/src/backend.rs:337-350`

**当前：** `text_document_document_symbol` 返回 `None`。

**实现：** 解析 AST，提取以下符号：
- `fn` → Function (SK_FUNCTION)
- `type` → Class (SK_CLASS)
- `enum` → Enum (SK_ENUM)
- `spec` → Interface (SK_INTERFACE)
- `ext` → Namespace (SK_NAMESPACE)
- `const` → Constant (SK_CONSTANT)
- `let`/`var` → Variable (SK_VARIABLE)

```rust
fn text_document_document_symbol(&self, params: DocumentSymbolParams) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
    let uri = params.text_document.uri.as_str();
    let content = match self.documents.get(uri) {
        Some(c) => c.clone(),
        None => return Ok(None),
    };

    let mut symbols = Vec::new();
    // Parse and extract top-level symbols using regex (simple approach)
    // For each fn/type/enum/spec/ext declaration, create a DocumentSymbol
    // with name, kind, range

    // fn declarations
    for cap in Regex::new(r"(?m)^\s*(pub\s+)?(mut\s+)?fn\s+([a-zA-Z_]\w*)").unwrap().captures_iter(&content) {
        symbols.push(DocumentSymbol {
            name: cap[3].to_string(),
            kind: SymbolKind::FUNCTION,
            range: /* from line start to end */,
            selection_range: /* just the name */,
            detail: Some("function".to_string()),
            ..Default::default()
        });
    }
    // Similar for type, enum, spec, ext, const, let/var

    if symbols.is_empty() { Ok(None) } else { Ok(Some(DocumentSymbolResponse::Nested(symbols))) }
}
```

**注意：** 这是一个基于 regex 的简单实现。后续可以用 AST 遍历替换。先让它工作起来。

**Verify:** `cargo build -p auto-lsp`

---

### Task 12: 提交 Phase 3

```
feat(lsp): implement document symbols outline (Plan 212 Phase 3)
```

---

## Phase 4: VSCode 插件增强（P3）

### Task 13: 添加 Auto 代码片段

**Files:**
- Create: `auto-vscode/vscode-extension/snippets/auto.code-snippets.json`

**内容：** 常用 Auto 代码片段：

```json
{
  "Function": {
    "prefix": "fn",
    "body": ["fn ${1:name}(${2:params}) ${3:ret_type} {", "\t$0", "}"],
    "description": "Function definition"
  },
  "Let binding": {
    "prefix": "let",
    "body": ["let ${1:name} = ${2:value}"],
    "description": "Immutable binding"
  },
  "Var binding": {
    "prefix": "var",
    "body": ["var ${1:name} = ${2:value}"],
    "description": "Mutable binding"
  },
  "Type definition": {
    "prefix": "type",
    "body": ["type ${1:Name} {", "\t${2:field} ${3:Type}", "}"],
    "description": "Type definition"
  },
  "Enum definition": {
    "prefix": "enum",
    "body": ["enum ${1:Name} {", "\t${2:Variant}", "}"],
    "description": "Enum definition"
  },
  "Spec definition": {
    "prefix": "spec",
    "body": ["spec ${1:Name} {", "\tfn ${2:method}() ${3:ret_type}", "}"],
    "description": "Spec/trait definition"
  },
  "Ext block": {
    "prefix": "ext",
    "body": ["ext ${1:Type} {", "\tfn ${2:method}() ${3:ret_type} {", "\t\t$0", "\t}", "}"],
    "description": "Extension block"
  },
  "Is match": {
    "prefix": "is",
    "body": ["is ${1:value} {", "\t${2:pattern} -> $0", "}"],
    "description": "Pattern matching"
  },
  "For loop": {
    "prefix": "for",
    "body": ["for ${1:item} in ${2:iterable} {", "\t$0", "}"],
    "description": "For loop"
  },
  "F-string": {
    "prefix": "fstr",
    "body": ["f\"${1:text}$${2:var}\""],
    "description": "F-string with interpolation"
  },
  "Print": {
    "prefix": "print",
    "body": ["print(${1:value})"],
    "description": "Print to stdout"
  },
  "Ok result": {
    "prefix": "ok",
    "body": ["Ok(${1:value})"],
    "description": "Ok constructor"
  },
  "Err result": {
    "prefix": "err",
    "body": ["Err(${1:value})"],
    "description": "Err constructor"
  }
}
```

**注册：** 在 `package.json` 的 `contributes.snippets` 中添加引用。

---

### Task 14: 更新 language-configuration.json

**Files:**
- Modify: `auto-vscode/vscode-extension/language-configuration.json`

**添加内容：**

```json
{
  "comments": {
    "lineComment": "//",
    "blockComment": ["/*", "*/"]
  },
  "brackets": [
    ["(", ")"],
    ["[", "]"],
    ["{", "}"]
  ],
  "autoClosingPairs": [
    { "open": "(", "close": ")" },
    { "open": "[", "close": "]" },
    { "open": "{", "close": "}" },
    { "open": "\"", "close": "\"", "notIn": ["string"] },
    { "open": "'", "close": "'", "notIn": ["string", "comment"] },
    { "open": "`", "close": "`", "notIn": ["string"] },
    { "open": "/*", "close": " */", "notIn": ["string", "comment"] }
  ],
  "surroundingPairs": [
    ["(", ")"],
    ["[", "]"],
    ["{", "}"],
    ["\"", "\""],
    ["'", "'"],
    ["`", "`"]
  ],
  "folding": {
    "markers": {
      "start": "^\\s*//\\s*#region",
      "end": "^\\s*//\\s*#endregion"
    }
  },
  "wordPattern": "[a-zA-Z_][a-zA-Z0-9_]*",
  "indentationRules": {
    "increaseIndentPattern": "\\{\\s*$",
    "decreaseIndentPattern": "^\\s*\\}"
  }
}
```

**Verify:** 在 VSCode 中测试括号自动关闭、注释快捷键。

---

### Task 15: 提交 Phase 4

```
feat(vscode): add code snippets and update language config (Plan 212 Phase 4)
```

---

## Dependency Graph

```
Phase 1 (TextMate) ──> independent, P0, do first
Phase 2 (LSP completion) ──> independent, P1
Phase 3 (Document Symbols) ──> independent, P2
Phase 4 (Snippets + Config) ──> independent, P3
```

所有 Phase 独立，可以并行或串行执行。建议串行按优先级。

## Risks

1. **TextMate grammar 复杂度**: `f"""` 与 `"""` 的优先级可能冲突 — 确保 `f"""` pattern 在 `"""` 之前
2. **Regex 性能**: Document symbols 用 regex 遍历可能在大文件上慢 — 后续改 AST
3. **LSP 补全顺序**: 新增关键字可能与变量名冲突 — 补全按上下文分发已处理
4. **向后兼容**: 移除 Rust 关键字（`match`, `trait`, `struct`）可能影响旧代码 — Auto 不用这些，安全移除
