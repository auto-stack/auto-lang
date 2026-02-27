# AURA 架构迁移计划

> **参考设计文档**:
> - [docs/design/aura.md](../design/aura.md) - AURA (Auto UI Representation Abstract) 核心规范
> - [docs/design/scenario.md](../design/scenario.md) - 面向场景编程架构

## 背景

### 当前问题
当前的 DSL 预处理方式（`widget Counter` → `type Counter is Widget`）存在以下问题：
- UI 声明 → AutoAST → UI-IR 的转换过于复杂
- 文本级别的宏展开丢失语义信息，错误定位困难
- 缺少专门的 UI 场景 AST 结构
- 多后端支持时需要重复解析

### 目标架构
```
pac.at (scenario: "ui", backend: "vue")
    ↓
CLI (auto build -s ui)
    ↓
CompilerSession { scenario: Scenario::UI }
    ↓
Contextual Parser (widget/msg/model/view/on 为上下文关键字)
    ↓
UI AST（WidgetDecl 为一等公民）
    ↓
AURA Extraction (抽取，1:1 无损映射)
    ↓
Backend Dispatch (Vue3/JavaScript / Rust/GPUI)
```

---

## 核心概念

### 1. 场景管道 (Scenario Pipeline)

编译器由 `pac.at` 配置驱动，避免污染 Auto 语言核心语法：

```auto
// pac.at (UI 工程示例)
name: "my-counter-app"
version: "1.0.0"
scenario: "ui"      // 声明 UI 场景
backend: "vue"      // 声明编译目标：vue, rust, gpui
```

```auto
// pac.at (Core 工程示例)
name: "my-core-service"
version: "1.0.0"
scenario: "core"    // 标准 Auto 语言
backend: "a2r"      // 编译为 Rust
```

### 2. 上下文关键字 (Contextual Keywords)

- **Lexer**: `widget`, `view`, `model`, `on`, `msg` 视为普通标识符
- **Parser**: 仅当 `session.scenario == UI` 时，提升为上下文关键字

### 3. AURA (Auto UI Representation Abstract)

AURA 是从 AutoUI 源码中**抽取 (Extract)** 出来的静态、高度结构化的中间表示：
- **剥离**控制流和复杂语法糖
- **保留**: UI 骨架 (View)、响应式状态 (Model)、事件路由 (Msg)
- **序列化**: 遵循 Atom 格式

---

## 后端架构调整

### 后端目标（按优先级）

| 后端 | 优先级 | 状态 | 说明 |
|------|--------|------|------|
| **Vue3/JavaScript** | P0 | 待开发 | 使用 a2js 进行表达式转换 |
| **Rust/GPUI** | P1 | 已有旧实现 | 需从 auto-ui 迁移到 AURA |
| **React/TypeScript** | P2+ | 暂不支持 | 短期内不考虑 |

### 后端生成器位置

```
auto-lang/src/
├── session.rs          # CompilerSession + Scenario enum
├── aura/               # AURA 核心模块
│   ├── mod.rs
│   ├── types.rs
│   ├── extract.rs
│   └── atom.rs
├── ast/
│   └── ui.rs           # UI AST 节点
└── ui_gen/             # UI 后端生成器（新建）
    ├── mod.rs          # 生成器入口
    ├── vue.rs          # Vue3 SFC 生成器
    ├── rust.rs         # Rust/GPUI 生成器（从 auto-ui 迁移）
    └── style.rs        # 样式处理
```

---

## 后端 1: Vue3/JavaScript 生成器

### 设计思路

Vue3 生成器将 AURA 转换为 **Vue3 SFC (Single File Component)** 格式：

1. **利用现有 a2js 转译器**
   - `auto-lang/src/trans/javascript.rs` 已实现 Auto → JS 转换
   - 可复用表达式、语句、函数调用的转换逻辑

2. **生成 Vue3 SFC**
   - `<template>`: 从 `view_tree` 生成
   - `<script setup>`: 从 `model` 和 `handlers` 生成
   - `<style scoped>`: 从 `style` 生成

### Vue3 生成示例

**输入 AURA**:
```rust
AuraWidget {
    name: "Counter",
    state_vars: [AuraStateDef { name: "count", type_info: Type::Int, initial: AuraExpr::Int(0) }],
    messages: [AuraMessage { name: "Msg", variants: [Inc, Dec] }],
    view_tree: AuraNode::Element { tag: "div", children: [...] },
    handlers: { "Msg::Inc" => AstBlock([count += 1]) }
}
```

**输出 Vue3 SFC**:
```vue
<template>
  <div class="counter">
    <button @click="handleInc">+</button>
    <h2>Count: {{ count }}</h2>
    <button @click="handleDec">-</button>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const count = ref(0)

const handleInc = () => {
  count.value += 1
}

const handleDec = () => {
  count.value -= 1
}
</script>

<style scoped>
.counter {
  display: flex;
  flex-direction: column;
}
</style>
```

### 关键实现

```rust
// auto-lang/src/ui_gen/vue.rs
pub struct VueGenerator {
    /// 复用 a2js 转译器
    js_trans: JavaScriptTrans,
}

impl VueGenerator {
    /// 生成 Vue3 SFC
    pub fn generate(&mut self, widget: &AuraWidget) -> Result<String, GenError> {
        let mut output = String::new();

        // 1. 生成 <template>
        output.push_str("<template>\n");
        output.push_str(&self.generate_template(&widget.view_tree)?);
        output.push_str("</template>\n\n");

        // 2. 生成 <script setup>
        output.push_str("<script setup>\n");
        output.push_str(&self.generate_script(widget)?);
        output.push_str("</script>\n\n");

        // 3. 生成 <style scoped>
        if let Some(style) = &widget.style {
            output.push_str("<style scoped>\n");
            output.push_str(&self.generate_style(style)?);
            output.push_str("</style>\n");
        }

        Ok(output)
    }

    /// 生成模板（HTML）
    fn generate_template(&mut self, node: &AuraNode) -> Result<String, GenError> {
        match node {
            AuraNode::Element { tag, props, events, children } => {
                // 映射 tag: col -> div.flex-col, button -> button, etc.
                let html_tag = self.map_tag(tag);
                // ...
            }
            AuraNode::Text(content) => self.generate_text(content),
        }
    }

    /// 生成脚本（复用 a2js）
    fn generate_script(&mut self, widget: &AuraWidget) -> Result<String, GenError> {
        let mut script = String::new();

        // 导入 Vue
        script.push_str("import { ref, computed } from 'vue'\n\n");

        // 状态变量 → ref()
        for state in &widget.state_vars {
            script.push_str(&format!("const {} = ref(", state.name));
            // 使用 a2js 转换初始值
            let init_js = self.js_trans.expr_to_string(&state.initial)?;
            script.push_str(&init_js);
            script.push_str(")\n");
        }

        // 事件处理器
        for (pattern, payload) in &widget.handlers {
            let handler_name = self.pattern_to_handler_name(pattern);
            script.push_str(&format!("\nconst {} = () => {{\n", handler_name));
            // 使用 a2js 转换 handler body
            if let LogicPayload::AstBlock(stmts) = payload {
                for stmt in stmts {
                    let stmt_js = self.js_trans.stmt_to_string(stmt)?;
                    script.push_str(&format!("  {}\n", stmt_js));
                }
            }
            script.push_str("}\n");
        }

        Ok(script)
    }
}
```

---

## 后端 2: Rust/GPUI 生成器迁移

### 当前状态

**位置**: `auto-ui/crates/auto-ui/src/trans/rust_gen.rs`

**当前机制**:
```
widget Counter          # 语法糖
    ↓
type Counter is Widget  # TypeDecl
    ↓
rust_gen.rs             # 旧生成器
    ↓
Rust Component impl     # 输出
```

**问题**:
- 依赖 `TypeDecl` 而不是 `AuraWidget`
- 无法利用 AURA 的语义信息
- 与新架构不兼容

### 迁移计划

**目标**: 将 `rust_gen.rs` 迁移到 `auto-lang/src/ui_gen/rust.rs`，改为从 AURA 生成。

**步骤**:
1. **分析现有实现**
   - 理解 `generate_widget()`, `generate_struct()`, `generate_component_impl()`
   - 理解消息处理和视图生成逻辑

2. **适配 AURA**
   - 输入从 `TypeDecl` 改为 `AuraWidget`
   - 使用 `state_vars` 生成 struct 字段
   - 使用 `view_tree` 生成 `view()` 方法
   - 使用 `handlers` 生成 `on()` 方法

3. **保持输出兼容**
   - 生成的 Rust 代码应与现有 `Component` trait 兼容
   - 确保 auto-ui 框架无需修改

### 迁移后的架构

```rust
// auto-lang/src/ui_gen/rust.rs
pub struct RustGenerator;

impl RustGenerator {
    /// 从 AURA 生成 Rust 代码
    pub fn generate(widget: &AuraWidget) -> Result<String, GenError> {
        let mut code = String::new();

        // 生成 Msg enum
        code.push_str(&Self::generate_msg_enum(&widget.messages));

        // 生成 struct
        code.push_str(&Self::generate_struct(widget));

        // 生成 Component impl
        code.push_str(&Self::generate_component_impl(widget));

        Ok(code)
    }

    fn generate_msg_enum(messages: &[AuraMessage]) -> String {
        // 合并所有消息变体
        let mut variants = Vec::new();
        for msg in messages {
            for variant in &msg.variants {
                variants.push(variant);
            }
        }
        // ...
    }

    fn generate_struct(widget: &AuraWidget) -> String {
        // 从 state_vars 生成字段
        // ...
    }

    fn generate_component_impl(widget: &AuraWidget) -> String {
        // 从 view_tree 生成 view()
        // 从 handlers 生成 on()
        // ...
    }
}
```

---

## 分阶段实施计划

### Phase 0: 基础设施（已完成 ✅）

**目标**: AURA 核心结构定义，场景机制可用

**已完成**:
- [x] 实现 `Scenario` enum 和 `CompilerSession`
- [x] 定义 AURA 核心数据结构
- [x] 实现 AST → AURA 抽取器
- [x] 添加单元测试

### Phase 1: Parser 上下文关键字（已完成 ✅）

**已完成**:
- [x] 修改 Parser 接受 CompilerSession
- [x] 实现上下文关键字判断逻辑
- [x] 添加 UI AST 节点定义
- [x] 解析 view 特殊语法

### Phase 2: Vue3 生成器（4-5天）

**目标**: Vue3/JavaScript 后端从 AURA 工作

**关键文件**:
- `auto-lang/src/ui_gen/mod.rs` - 新建
- `auto-lang/src/ui_gen/vue.rs` - 新建
- `auto-lang/src/ui_gen/style.rs` - 新建

**任务**:
- [ ] 设计 Vue3 SFC 输出格式
- [ ] 实现 `VueGenerator` 结构
- [ ] 复用 a2js 转译器进行表达式转换
- [ ] 实现 template 生成（view_tree → HTML）
- [ ] 实现 script 生成（state → ref, handlers → functions）
- [ ] 添加后端测试

**验证点**: `Counter.at` → AURA → 有效的 Vue3 SFC

### Phase 3: Rust 生成器迁移（3-4天）

**目标**: 从 auto-ui 迁移 Rust 生成器到 AURA

**关键文件**:
- `auto-ui/trans/rust_gen.rs` - 分析
- `auto-lang/src/ui_gen/rust.rs` - 新建

**任务**:
- [ ] 分析现有 `rust_gen.rs` 实现
- [ ] 设计 AURA → Rust 映射规则
- [ ] 实现 `RustGenerator` 从 AURA 生成
- [ ] 保持与 auto-ui 框架的兼容性
- [ ] 添加后端测试

**验证点**: `Counter.at` → AURA → 有效的 Rust Component

### Phase 4: CLI 与 pac.at 集成（2-3天）

**目标**: CLI 支持 `-s` 和 `-b` 参数，读取 pac.at

**关键文件**:
- `auto-lang/src/cli.rs` - 修改

**任务**:
- [ ] 实现 pac.at 解析
- [ ] CLI 支持 `-s <scenario>` 和 `-b <backend>`
- [ ] 更新转译 API 入口
- [ ] 热重载集成

**验证点**: `auto build -s ui -b vue Counter.at` 工作

### Phase 5: 清理与迁移（2-3天）

**目标**: 移除遗留代码，更新 auto-ui 使用新生成器

**关键文件**:
- `auto-ui/trans/rust_gen.rs` - 废弃
- `auto-ui/trans/api.rs` - 修改为调用新生成器

**任务**:
- [ ] auto-ui 调用 `auto-lang::ui_gen::RustGenerator`
- [ ] 标记旧 `rust_gen.rs` 为废弃
- [ ] 更新文档
- [ ] 性能基准测试

**验证点**: auto-ui 使用 AURA 路径，所有测试通过

---

## 里程碑进度表

| Phase | 预计时间 | 状态 | 交付物 |
|-------|---------|------|--------|
| Phase 0 | 2-3天 | ✅ 已完成 | AURA 核心 + CompilerSession |
| Phase 1 | 3-4天 | ✅ 已完成 | 上下文关键字解析 |
| Phase 2 | 4-5天 | ✅ 已完成 | Vue3 生成器 |
| Phase 3 | 3-4天 | ✅ 已完成 | Rust 生成器迁移 |
| Phase 4 | 2-3天 | ✅ 已完成 | CLI 集成 |
| Phase 5 | 2-3天 | ✅ 已完成 | 清理与迁移 |
| Phase 6 | 2-3天 | ⏳ 待开始 | 修复已知问题 |

**总计**: 18-24 天

---

## 关键文件清单

### 已完成（auto-lang）
- [x] `src/session.rs` - CompilerSession + Scenario
- [x] `src/aura/mod.rs` - AURA 模块入口
- [x] `src/aura/types.rs` - 核心类型定义
- [x] `src/aura/extract.rs` - AST → AURA 抽取
- [x] `src/aura/atom.rs` - Atom 格式序列化
- [x] `src/ast/ui.rs` - UI AST 节点

### 需要新建（auto-lang）
- [ ] `src/ui_gen/mod.rs` - 后端生成器入口
- [ ] `src/ui_gen/vue.rs` - Vue3 SFC 生成
- [ ] `src/ui_gen/rust.rs` - Rust/GPUI 生成（迁移自 auto-ui）
- [ ] `src/ui_gen/style.rs` - 样式处理

### 需要修改
- [ ] `auto-lang/src/cli.rs` - 添加 `-s` 和 `-b` 参数
- [ ] `auto-ui/trans/api.rs` - 调用新 AURA 生成器

### 最终废弃
- [ ] `auto-ui/trans/rust_gen.rs` - 旧 Rust 生成器

---

## 验证标准

### 技术指标
- 编译时间: < 100ms（典型组件）
- 代码质量: 无 clippy 警告
- 测试覆盖: > 80%（新代码）
- 错误信息: 指向原始 DSL 源码

### 功能要求
- [x] `Counter.at` → WidgetDecl AST 解析正确
- [x] WidgetDecl → AuraWidget 抽取正确
- [ ] Vue3 后端生成可运行 SFC
- [ ] Rust 后端生成可编译代码
- [ ] auto-ui 框架与新后端兼容
- [ ] 事件处理器正确绑定
- [ ] `${.state}` 插值正确处理

---

## 架构收益

1. **无状态视图的纯粹性**: AURA 的 `view_tree` 绝对干净，不包含任何业务逻辑
2. **复用现有转译器**: Vue3 生成器复用 a2js，减少重复工作
3. **统一后端位置**: 所有 UI 后端生成器集中在 `auto-lang/src/ui_gen/`
4. **auto-ui 简化**: auto-ui 框架层只需调用新生成器，无需了解 AURA 细节

---

## Phase 6: 已知问题与待修复（Todo）

### 问题 1: Handler Body 提取不完整 ✅ 已修复

**状态**: ✅ 已修复 (commit 724b85e)

**现象**:
```auto
on {
    Inc => {
        count = count + 1
    }
}
```

生成的 Rust 代码中 handler body 为空：
```rust
fn on(&mut self, msg: Self::Msg) {
    match msg {
        Msg::Inc => { }  // 空！
        _ => {}
    }
}
```

**原因**:
`aura/extract.rs` 中的 `extract_body_stmts()` 只处理 `Stmt::Store`，但 `count = count + 1` 不是 Store 语句。

**修复方案**:
扩展 `extract_body_stmts()` 处理 `Stmt::Expr` 中的赋值表达式：
- `Op::Asn` → `AuraStmt::Assign`
- `Op::AddEq` → `AuraStmt::Update { AddAssign }`
- `Op::SubEq` → `AuraStmt::Update { SubAssign }`
- 等等

---

### 问题 2: View Tree 子节点未生成

**现象**:
```auto
view {
    col {} {
        text {} { text: "Hello" }
    }
}
```

生成的 Rust 代码：
```rust
fn view(&self) -> View<Self::Msg> {
    View::col().build()  // 子节点丢失！
}
```

**原因**:
`extract_view_node()` 正确提取了 children，但 `RustGenerator::generate_view_tree()` 可能没有正确处理子节点。

**位置**:
- `auto-lang/src/aura/extract.rs` - 提取逻辑
- `auto-lang/src/ui_gen/rust.rs` - 生成逻辑

**修复方案**:
1. 检查 `AuraNode::Element { children }` 是否正确填充
2. 检查 `RustGenerator::generate_view_tree()` 是否递归处理 children
3. 确保 `.child()` 调用被正确生成

**优先级**: 高

---

### 问题 3: 状态引用 (`.count`) 未正确转换 ✅ 已修复

**状态**: ✅ 已修复 (commit 724b85e)

**现象**:
```auto
model {
    count int = 0
}

on {
    Inc => {
        count = count + 1  // .count 前缀可选
    }
}
```

在 handler body 中，`count` 应该转换为 `self.count`。

**修复说明**:
修复问题 1 时，`extract_expr()` 正确地将 `count` 标识符转换为 `AuraExpr::StateRef("count")`，
然后 `RustGenerator::expr_to_rust()` 将其转换为 `self.count`。

**验证**:
生成的代码 `self.count = self.count + 1` 是正确的。

---

### 问题 4: 事件绑定 (`onclick: .Inc`) 未正确提取

**现象**:
```auto
view {
    button {} { onclick: .Inc }
}
```

`onclick` 事件应该生成 `.on_click(|_| Msg::Inc)` 调用。

**位置**:
- `auto-lang/src/aura/extract.rs` - 事件提取
- `auto-lang/src/ui_gen/rust.rs` - 事件生成

**修复方案**:
1. 确保 `ViewEvent { name, handler }` 正确提取
2. 在 `RustGenerator` 中处理事件绑定生成

**优先级**: 中

---

### 测试用例

创建完整的端到端测试：

```auto
// examples/counter_full.at
widget Counter {
    msg Msg { Inc, Dec }

    model {
        count int = 0
    }

    view {
        col {} {
            text {} { text: "Count: " }
            text {} { text: .count }
            row {} {
                button {} { text: "-", onclick: .Dec }
                button {} { text: "+", onclick: .Inc }
            }
        }
    }

    on {
        Inc => {
            count = count + 1
        }
        Dec => {
            count = count - 1
        }
    }
}
```

预期生成的 Rust 代码：
```rust
impl Component for Counter {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Msg::Inc => { self.count += 1; }
            Msg::Dec => { self.count -= 1; }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .child(View::text("Count: "))
            .child(View::text(&self.count.to_string()))
            .child(View::row()
                .child(View::button("-").on_click(|_| Msg::Dec))
                .child(View::button("+").on_click(|_| Msg::Inc))
            )
            .build()
    }
}
```

---

### 修复进度

| 问题 | 优先级 | 状态 | 负责人 |
|------|--------|------|--------|
| Handler Body 提取 | 高 | ✅ 已修复 | commit 724b85e |
| 状态引用转换 | 中 | ✅ 已修复 | commit 724b85e |
| View Tree 子节点 | 高 | ⏳ 待修复 | - |
| 事件绑定生成 | 中 | ⏳ 待修复 | - |
