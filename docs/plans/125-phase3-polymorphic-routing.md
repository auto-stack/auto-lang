# Plan 125: Phase 3 - 多态路由、隐式联合体与显式消息上下文

## 概述

本文档描述 Auto 语言 Task/Msg 系统的 Phase 3 实现计划。Phase 3 的核心目标是：

1. **隐式联合体 (Implicit Union)**：前端 AST 自动提取 `on` 块中的所有数据类型，在底层隐式合成消息信封
2. **显式消息上下文 (Explicit Context)**：将 `on` 块升级为带可选参数的闭包，将 `reply` 方法挂载到第一公民对象
3. **全能匹配器 (Omnipotent Matcher)**：支持字面量精确匹配、类型捕获绑定以及守卫表达式

## 设计决策

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 实现范围 | 全面替换显式 enum | 符合设计文档原始意图 |
| MessageContext 类型 | 内置类型 | 编译器自动注入，更符合语言特性 |
| 上下文参数名 | 任意有效标识符 | 灵活性更高，支持 ctx/req/origin 等 |

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    前端语法 (Auto Language)                  │
│                                                              │
│  on(ctx) {                                                   │
│      "ping" => { ctx.reply("pong") }                        │
│      msg string => { write_to_disk(msg) }                   │
│      amount int if amount > 10000 => { ... }                │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   AST 层 (crates/.../ast/)                  │
│                                                              │
│  TaskOnBlock {                                               │
│      context_param: Option<Name>,  // ctx/req/origin        │
│      handlers: Vec<(Pattern, Guard, Body)>,                 │
│  }                                                           │
│                                                              │
│  Pattern = Literal | TypeBinding | EnumVariant              │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              隐式联合体生成器 (Implicit Union)               │
│                                                              │
│  on { "ping", msg string } → enum TaskEnvelope {            │
│      LiteralPing,                                            │
│      StringValue(String),                                    │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                 运行时层 (crates/.../vm/)                   │
│                                                              │
│  MessageContext {                                            │
│      sender_id: Option<u64>,                                 │
│      trace_id: String,                                       │
│      is_ask: bool,                                           │
│      reply_tx: Option<oneshot::Sender<Value>>,               │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
```

## 实现步骤

### Phase 3.1: AST 层扩展 (预计 2 天)

**目标文件**：
- `crates/auto-lang/src/ast/task.rs`

**任务清单**：

1. **扩展 `TaskMsgPattern` 枚举**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskMsgPattern {
    // === 现有变体 (Phase 1/2) ===
    /// 简单变体: Reset, Print
    Simple(Name),
    /// 带绑定的变体: Add(val), Log(msg)
    WithBindings {
        variant: Name,
        bindings: Vec<Name>,
    },

    // === 新增变体 (Phase 3) ===
    /// 字面量精确匹配: "start", 404, true
    Literal(LiteralValue),
    /// 类型捕获绑定: msg string, u User, data []byte
    TypeBinding {
        name: Name,      // 绑定变量名
        type_expr: Type, // 类型表达式
    },
}

/// 字面量值
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralValue {
    String(AutoStr),
    Int(i64),
    Uint(u64),
    Float(i64, i64), // 整数部分, 小数部分
    Bool(bool),
    Char(char),
}
```

2. **扩展 `TaskOnBlock` 结构**

```rust
#[derive(Debug, Clone)]
pub struct TaskOnBlock {
    /// 上下文参数名 (如 ctx, req, origin)
    /// None 表示无上下文参数
    pub context_param: Option<Name>,

    /// 消息处理器: (模式, 守卫表达式, 函数体)
    pub handlers: Vec<(TaskMsgPattern, Option<Expr>, Body)>,

    /// else 处理器
    pub else_handler: Option<Body>,

    /// 源码位置
    pub pos: Pos,
}
```

3. **新增 `MessageContext` AST 节点**（用于类型检查）

```rust
/// 消息上下文类型（编译器内置）
#[derive(Debug, Clone)]
pub struct MessageContextType {
    pub sender_id: Type,  // ?u64
    pub trace_id: Type,   // string
    pub is_ask: Type,     // bool
}
```

4. **更新 Display/ToAtom/ToNode 实现**

为新增的枚举变体实现格式化输出。

**验收标准**：
- [ ] `TaskMsgPattern::Literal` 正确解析和显示
- [ ] `TaskMsgPattern::TypeBinding` 正确解析和显示
- [ ] `TaskOnBlock.context_param` 正确存储和访问
- [ ] 所有新增类型有完整的单元测试

---

### Phase 3.2: Parser 层扩展 (预计 3 天)

**目标文件**：
- `crates/auto-lang/src/parser.rs`

**任务清单**：

1. **解析 `on(ctx)` 语法**

```rust
// 扩展 parse_task_on_block 方法
fn parse_task_on_block(&mut self, pos: Pos) -> AutoResult<TaskOnBlock> {
    self.expect(TokenKind::On)?;

    // 检查是否有上下文参数: on(ctx) 或 on { ... }
    let context_param = if self.is_kind(TokenKind::LParen) {
        self.next();
        let name = self.expect_ident()?;
        self.expect(TokenKind::RParen)?;
        Some(name)
    } else {
        None
    };

    self.expect(TokenKind::LBrace)?;

    // 解析处理器...
}
```

2. **解析字面量模式**

```rust
fn parse_literal_pattern(&mut self) -> AutoResult<TaskMsgPattern> {
    match &self.cur.kind {
        TokenKind::String(s) => {
            let pattern = TaskMsgPattern::Literal(LiteralValue::String(s.clone()));
            self.next();
            Ok(pattern)
        }
        TokenKind::Number(n) => {
            // 解析整数/浮点数
        }
        TokenKind::True | TokenKind::False => {
            // 解析布尔值
        }
        _ => Err(...) // 不是字面量模式
    }
}
```

3. **解析类型绑定模式**

```rust
fn parse_type_binding_pattern(&mut self) -> AutoResult<TaskMsgPattern> {
    // 格式: name Type 或 name Type< generics >
    // 例如: msg string, u User, data []byte

    let name = self.expect_ident()?;  // 绑定变量名
    let type_expr = self.parse_type()?;  // 类型表达式

    Ok(TaskMsgPattern::TypeBinding { name, type_expr })
}
```

4. **解析守卫表达式**

```rust
fn parse_guard_expression(&mut self) -> AutoResult<Option<Expr>> {
    // 格式: pattern if guard_expr => { body }
    // 例如: amount int if amount > 10000 => { ... }

    if self.is_kind(TokenKind::If) {
        self.next();
        let guard = self.parse_expr()?;
        Ok(Some(guard))
    } else {
        Ok(None)
    }
}
```

5. **实现双遍扫描机制**

```rust
/// 第一遍扫描：收集所有模式类型
fn collect_pattern_types(&self, on_block: &TaskOnBlock) -> ImplicitUnionInfo {
    let mut info = ImplicitUnionInfo::new();

    for (pattern, _, _) in &on_block.handlers {
        match pattern {
            TaskMsgPattern::Literal(lit) => info.add_literal(lit.clone()),
            TaskMsgPattern::TypeBinding { type_expr, .. } => info.add_type(type_expr.clone()),
            _ => {} // 处理其他模式类型
        }
    }

    info
}
```

**验收标准**：
- [ ] `on(ctx)` 语法正确解析
- [ ] `on` (无参数) 语法仍然有效
- [ ] 字面量模式 `"ping"`, `404`, `true` 正确解析
- [ ] 类型绑定模式 `msg string`, `u User` 正确解析
- [ ] 守卫表达式 `if amount > 10000` 正确解析
- [ ] 双遍扫描正确收集模式类型

---

### Phase 3.3: 隐式联合体生成器 (预计 2 天)

**目标文件**：
- `crates/auto-lang/src/implicit_union.rs` (新建)

**任务清单**：

1. **定义隐式联合体信息结构**

```rust
/// 隐式联合体生成信息
#[derive(Debug, Clone)]
pub struct ImplicitUnionInfo {
    /// 字面量模式列表
    pub literals: Vec<LiteralValue>,
    /// 类型绑定模式列表
    pub type_bindings: Vec<(Name, Type)>,
    /// 生成的枚举名称
    pub envelope_name: String,
}

impl ImplicitUnionInfo {
    pub fn new(task_name: &str) -> Self {
        Self {
            literals: Vec::new(),
            type_bindings: Vec::new(),
            envelope_name: format!("{}Envelope", task_name),
        }
    }

    pub fn add_literal(&mut self, lit: LiteralValue) {
        if !self.literals.contains(&lit) {
            self.literals.push(lit);
        }
    }

    pub fn add_type(&mut self, name: Name, type_expr: Type) {
        if !self.type_bindings.iter().any(|(n, _)| n == &name) {
            self.type_bindings.push((name, type_expr));
        }
    }
}
```

2. **实现 Rust 代码生成**

```rust
impl ImplicitUnionInfo {
    /// 生成 Rust 枚举定义
    pub fn generate_rust_enum(&self) -> String {
        let mut code = format!("pub enum {} {{\n", self.envelope_name);

        // 生成字面量变体
        for lit in &self.literals {
            let variant_name = self.literal_to_variant_name(lit);
            code.push_str(&format!("    {},\n", variant_name));
        }

        // 生成类型绑定变体
        for (name, type_expr) in &self.type_bindings {
            let variant_name = name.to_pascal_case();
            let rust_type = type_expr.to_rust_type();
            code.push_str(&format!("    {}({}),\n", variant_name, rust_type));
        }

        code.push_str("}\n");
        code
    }

    fn literal_to_variant_name(&self, lit: &LiteralValue) -> String {
        match lit {
            LiteralValue::String(s) => format!("Literal{}", s.to_pascal_case()),
            LiteralValue::Int(n) => format!("LiteralInt{}", n),
            LiteralValue::Bool(b) => format!("Literal{}", if *b { "True" } else { "False" }),
            // ... 其他类型
        }
    }
}
```

3. **生成消息信封结构**

```rust
/// 生成完整的消息信封结构
pub fn generate_envelope_struct(&self) -> String {
    format!(
        r#"
pub struct {task_name}Message {{
    pub context: MessageContext,
    pub payload: {task_name}Envelope,
}}
"#,
        task_name = self.envelope_name.strip_suffix("Envelope").unwrap()
    )
}
```

**验收标准**：
- [ ] 从简单模式正确生成枚举
- [ ] 从混合模式正确生成联合体
- [ ] 生成的 Rust 代码语法正确
- [ ] 信封结构包含 MessageContext

---

### Phase 3.4: MessageContext 运行时实现 (预计 2 天)

**目标文件**：
- `crates/auto-lang/src/vm/message_context.rs` (新建)

**任务清单**：

1. **定义 MessageContext 结构**

```rust
use tokio::sync::oneshot;
use auto_val::Value;

/// 消息上下文 - 运行时表示
#[derive(Debug)]
pub struct MessageContext {
    /// 发送方 ID (可能为空)
    pub sender_id: Option<u64>,
    /// 链路追踪 ID
    pub trace_id: String,
    /// 是否需要回执 (ask 模式)
    pub is_ask: bool,
    /// 回复通道 (ask 模式专用)
    reply_tx: Option<oneshot::Sender<Value>>,
}

impl MessageContext {
    /// 创建普通上下文 (send 模式)
    pub fn new(sender_id: Option<u64>, trace_id: String) -> Self {
        Self {
            sender_id,
            trace_id,
            is_ask: false,
            reply_tx: None,
        }
    }

    /// 创建 ask 上下文 (带回复通道)
    pub fn for_ask(
        sender_id: Option<u64>,
        trace_id: String,
        reply_tx: oneshot::Sender<Value>,
    ) -> Self {
        Self {
            sender_id,
            trace_id,
            is_ask: true,
            reply_tx: Some(reply_tx),
        }
    }

    /// 回复方法 - 暴露给 Auto 语言
    pub fn reply(&self, payload: Value) -> Result<(), String> {
        if let Some(tx) = &self.reply_tx {
            tx.send(payload).map_err(|_| "Reply channel closed".to_string())
        } else {
            Err("No reply channel available (not in ask mode)".to_string())
        }
    }

    /// 检查是否可以回复
    pub fn can_reply(&self) -> bool {
        self.reply_tx.is_some()
    }
}
```

2. **实现 VM 绑定**

```rust
// 在 crates/auto-lang/src/vm/ffi/ 中添加 MessageContext FFI

#[auto_macros::rust_fn("MessageContext.reply")]
pub fn message_context_reply(ctx: &MessageContext, payload: Value) -> Result<(), String> {
    ctx.reply(payload)
}

#[auto_macros::rust_fn("MessageContext.can_reply")]
pub fn message_context_can_reply(ctx: &MessageContext) -> bool {
    ctx.can_reply()
}
```

3. **注册到 VM**

```rust
// 在 vm 注册表中添加 MessageContext 类型和方法
impl AutoVM {
    pub fn register_message_context(&mut self) {
        // 注册类型
        self.register_type("MessageContext");

        // 注册方法
        self.register_method("MessageContext", "reply", message_context_reply);
        self.register_method("MessageContext", "can_reply", message_context_can_reply);
    }
}
```

**验收标准**：
- [ ] `MessageContext::new()` 正确创建上下文
- [ ] `MessageContext::for_ask()` 正确创建带回复通道的上下文
- [ ] `ctx.reply(payload)` 在 ask 模式下成功发送回复
- [ ] `ctx.reply()` 在非 ask 模式下返回错误
- [ ] `ctx.can_reply()` 正确返回可回复状态

---

### Phase 3.5: VM 层集成 (预计 3 天)

**目标文件**：
- `crates/auto-lang/src/vm/task_system.rs`
- `crates/auto-lang/src/vm/mod.rs`

**任务清单**：

1. **扩展 TaskSystem 消息分发**

```rust
impl TaskRegistry {
    /// 分发消息（带上下文）
    pub async fn dispatch_with_context(
        &self,
        task_type: &str,
        message: Value,
        context: MessageContext,
    ) -> Result<(), String> {
        // 获取任务句柄
        let handle = self.get_singleton(task_type)
            .ok_or_else(|| format!("Task '{}' not found", task_type))?;

        // 构造消息信封
        let envelope = Value::Object(indexmap::indexmap! {
            "context".into() => context_to_value(&context),
            "payload".into() => message,
        });

        // 发送消息
        handle.send(envelope).await
    }
}
```

2. **实现模式匹配路由**

```rust
/// 模式匹配器
pub struct PatternMatcher;

impl PatternMatcher {
    /// 匹配消息与模式
    pub fn match_pattern(
        pattern: &TaskMsgPattern,
        message: &Value,
    ) -> Option<Vec<(Name, Value)>> {
        match pattern {
            TaskMsgPattern::Literal(lit) => {
                if Self::match_literal(lit, message) {
                    Some(vec![])  // 字面量匹配，无绑定
                } else {
                    None
                }
            }
            TaskMsgPattern::TypeBinding { name, type_expr } => {
                if Self::match_type(type_expr, message) {
                    Some(vec![(name.clone(), message.clone())])  // 绑定变量
                } else {
                    None
                }
            }
            // ... 其他模式类型
        }
    }

    fn match_literal(lit: &LiteralValue, message: &Value) -> bool {
        match (lit, message) {
            (LiteralValue::String(s), Value::Str(v)) => s.as_str() == v.as_str(),
            (LiteralValue::Int(n), Value::Int(v)) => *n == *v as i64,
            (LiteralValue::Bool(b), Value::Bool(v)) => *b == *v,
            // ... 其他类型
            _ => false,
        }
    }
}
```

3. **实现守卫表达式求值**

```rust
impl PatternMatcher {
    /// 求值守卫表达式
    pub fn evaluate_guard(
        guard: &Expr,
        bindings: &[(Name, Value)],
    ) -> bool {
        // 在绑定环境中求值守卫表达式
        // 如果守卫为 None，默认返回 true
        // 如果守卫求值失败，返回 false
    }
}
```

4. **集成 ctx.reply() 支持**

```rust
/// 在任务执行环境中注入 ctx 变量
impl AutoTask {
    pub fn execute_handler_with_context(
        &mut self,
        handler: &Body,
        context: &MessageContext,
        bindings: &[(Name, Value)],
    ) -> Result<Value, String> {
        // 注入 ctx 变量到作用域
        self.universe.bind("ctx".into(), Value::Context(context.clone()));

        // 注入绑定变量
        for (name, value) in bindings {
            self.universe.bind(name.clone(), value.clone());
        }

        // 执行处理器
        self.eval_body(handler)
    }
}
```

**验收标准**：
- [ ] `dispatch_with_context` 正确分发带上下文的消息
- [ ] 模式匹配正确匹配字面量和类型绑定
- [ ] 守卫表达式正确求值
- [ ] `ctx` 变量正确注入到处理器作用域
- [ ] `ctx.reply()` 在处理器中正确工作

---

### Phase 3.6: 静态类型检查 (预计 2 天)

**目标文件**：
- `crates/auto-lang/src/typeck.rs`
- `crates/auto-lang/src/infer/` (扩展)

**任务清单**：

1. **隐式联合体类型推导**

```rust
impl TypeChecker {
    /// 从 on 块推导消息信封类型
    pub fn infer_envelope_type(&mut self, on_block: &TaskOnBlock) -> Type {
        let mut variants = Vec::new();

        for (pattern, _, _) in &on_block.handlers {
            match pattern {
                TaskMsgPattern::Literal(lit) => {
                    variants.push(Type::literal(lit.clone()));
                }
                TaskMsgPattern::TypeBinding { type_expr, .. } => {
                    variants.push(type_expr.clone());
                }
                _ => {}
            }
        }

        Type::Union(variants)
    }
}
```

2. **send/ask 类型校验**

```rust
impl TypeChecker {
    /// 校验 send 消息类型
    pub fn check_send_type(
        &mut self,
        task_type: &str,
        message_type: &Type,
    ) -> Result<(), TypeError> {
        // 获取任务的信封类型
        let envelope_type = self.get_task_envelope_type(task_type)?;

        // 检查消息类型是否匹配信封中的某个变体
        if !envelope_type.accepts(message_type) {
            return Err(TypeError::MessageNotAccepted {
                task: task_type.to_string(),
                expected: envelope_type,
                found: message_type.clone(),
            });
        }

        Ok(())
    }
}
```

3. **reply 类型推导**

```rust
impl TypeChecker {
    /// 从 ctx.reply(T) 推导 ask 返回类型
    pub fn infer_reply_type(&mut self, handler: &Body) -> Option<Type> {
        // 遍历处理器 AST，查找 ctx.reply(expr) 调用
        // 提取 expr 的类型作为 ask 返回类型

        for stmt in &handler.stmts {
            if let Some(reply_type) = self.extract_reply_type(stmt) {
                return Some(reply_type);
            }
        }

        None
    }
}
```

**验收标准**：
- [ ] 从 `on` 块正确推导信封类型
- [ ] `Task.send(wrong_type)` 编译期报错
- [ ] `ask` 返回类型正确推导
- [ ] 跨 Task 类型一致性检查

---

### Phase 3.7: 测试与文档 (预计 2 天)

**目标文件**：
- `crates/auto-lang/src/tests/phase3_tests.rs` (新建)

**测试用例**：

1. **AST 测试**

```rust
#[test]
fn test_parse_on_with_context() {
    let code = r#"
        task TestTask {
            on(ctx) {
                "ping" => { ctx.reply("pong") }
            }
        }
    "#;
    // 验证 context_param == Some("ctx")
}

#[test]
fn test_parse_literal_pattern() {
    let code = r#"
        on {
            "start" => { engine.ignite() }
            404 => { print("error") }
        }
    "#;
    // 验证 Literal(String) 和 Literal(Int)
}

#[test]
fn test_parse_type_binding() {
    let code = r#"
        on {
            msg string => { write(msg) }
            u User => { save(u) }
        }
    "#;
    // 验证 TypeBinding { name: "msg", type: string }
}

#[test]
fn test_parse_guard_expression() {
    let code = r#"
        on {
            amount int if amount > 10000 => { approve(amount) }
        }
    "#;
    // 验证 guard == Some(Expr::Binary(amount > 10000))
}
```

2. **隐式联合体测试**

```rust
#[test]
fn test_generate_implicit_union() {
    let patterns = vec![
        TaskMsgPattern::Literal(LiteralValue::String("ping".into())),
        TaskMsgPattern::TypeBinding {
            name: "msg".into(),
            type_expr: Type::Str,
        },
    ];

    let info = ImplicitUnionInfo::from_patterns("TestTask", &patterns);
    let rust_code = info.generate_rust_enum();

    assert!(rust_code.contains("LiteralPing"));
    assert!(rust_code.contains("Msg(String)"));
}
```

3. **运行时测试**

```rust
#[tokio::test]
async fn test_message_context_reply() {
    let (tx, rx) = oneshot::channel();
    let ctx = MessageContext::for_ask(Some(1), "trace-123".into(), tx);

    assert!(ctx.can_reply());
    assert!(ctx.reply(Value::Str("response".into())).is_ok());

    let response = rx.await.unwrap();
    assert_eq!(response, Value::Str("response".into()));
}

#[tokio::test]
async fn test_pattern_matching() {
    let matcher = PatternMatcher;

    // 字面量匹配
    let result = matcher.match_pattern(
        &TaskMsgPattern::Literal(LiteralValue::String("ping".into())),
        &Value::Str("ping".into()),
    );
    assert!(result.is_some());

    // 类型绑定匹配
    let result = matcher.match_pattern(
        &TaskMsgPattern::TypeBinding {
            name: "msg".into(),
            type_expr: Type::Str,
        },
        &Value::Str("hello".into()),
    );
    assert!(result.is_some());
    assert_eq!(result.unwrap()[0].0, "msg");
}
```

4. **类型检查测试**

```rust
#[test]
fn test_send_type_check() {
    // 定义任务信封类型: string | int
    // 发送 string → 通过
    // 发送 bool → 编译错误
}

#[test]
fn test_ask_return_type_inference() {
    // ctx.reply("response") → ask 返回 ~string
    // ctx.reply(42) → ask 返回 ~int
}
```

5. **集成测试**

```rust
#[tokio::test]
async fn test_phase3_full_example() {
    let code = r#"
        task NodeWorker {
            on(ctx) {
                "ping" => { ctx.reply("pong") }
                msg string => { print(msg) }
                amount int if amount > 10000 => { ctx.reply("need_approval") }
                amount int => { ctx.reply("approved") }
            }
        }

        fn main() ! {
            // 字面量匹配
            NodeWorker.send("ping")

            // 类型绑定匹配
            NodeWorker.send("hello")

            // ask 模式
            let result = NodeWorker.ask(5000).await.?
            print(result)  // "approved"
        }
    "#;

    // 执行并验证输出
}
```

**验收标准**：
- [ ] 所有 AST 测试通过
- [ ] 所有隐式联合体测试通过
- [ ] 所有运行时测试通过
- [ ] 所有类型检查测试通过
- [ ] 集成测试完整运行

---

## 验收标准 (Acceptance Criteria)

### 功能验收

1. **废除 reply 关键字**
   - AST 解析器从保留字列表中移除 `reply`
   - 所有回复必须通过 `ctx.reply()` 方法调用

2. **上下文作用域隔离**
   - 声明 `on(ctx)` 后，`ctx` 变量仅在当前匹配分支内有效
   - 分支执行完毕后 `ctx` 自动失效

3. **双遍扫描正确性**
   - 编译器准确推导隐式联合体
   - `Task.ask("ping").await` 自动推导返回类型为 `~string`

### 性能验收

1. **零运行时开销**
   - 隐式联合体编译后与手写 enum 性能相同
   - 模式匹配编译为高效跳转表

2. **类型检查延迟**
   - 编译期完成所有类型检查
   - 运行时无类型验证开销

### 兼容性验收

1. **Phase 1/2 兼容**
   - 显式 enum 协议仍然可用
   - 现有代码无需修改即可编译

2. **渐进式迁移**
   - 可以在同一项目中混合使用 Phase 1/2 和 Phase 3 语法

---

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 模式匹配复杂度爆炸 | 编译时间增加 | 限制单个 on 块的模式数量上限 |
| 隐式联合体命名冲突 | 代码生成错误 | 使用唯一前缀 (TaskName_Envelope) |
| ctx 变量逃逸 | 作用域混乱 | 编译期检查 ctx 使用范围 |
| 类型推导失败 | 编译错误不清晰 | 提供详细的类型推导错误信息 |

---

## 时间线

| 阶段 | 预计时间 | 依赖 |
|------|----------|------|
| Phase 3.1: AST 层扩展 | 2 天 | 无 |
| Phase 3.2: Parser 层扩展 | 3 天 | 3.1 |
| Phase 3.3: 隐式联合体生成器 | 2 天 | 3.2 |
| Phase 3.4: MessageContext 运行时 | 2 天 | 3.1 |
| Phase 3.5: VM 层集成 | 3 天 | 3.3, 3.4 |
| Phase 3.6: 静态类型检查 | 2 天 | 3.5 |
| Phase 3.7: 测试与文档 | 2 天 | 全部 |
| **总计** | **16 天** | - |

---

## 参考文档

- [设计文档: docs/design/task-msg.md](../design/task-msg.md) - Phase 3 规范
- [Plan 121: Task/Msg 系统](./121-async-task-msg-system.md) - Phase 1 实现
- [Plan 124: Async/Future/Await](./124-async-future-await.md) - Phase 2 实现
