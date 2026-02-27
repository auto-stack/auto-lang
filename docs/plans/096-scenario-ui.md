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
Backend Dispatch (React / Compose / GPUI+AutoVM)
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
backend: "vue"      // 声明编译目标
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

## AURA 核心数据结构

### 模块结构

```
auto-lang/src/
├── session.rs          # CompilerSession + Scenario enum
├── aura/               # AURA 核心模块 (原 ui_ir)
│   ├── mod.rs          # 模块入口，导出核心类型
│   ├── types.rs        # AURA 核心类型定义
│   ├── extract.rs      # AST → AURA 抽取 (原 convert.rs)
│   └── atom.rs         # Atom 格式序列化
└── ast/
    └── ui.rs           # UI AST 节点 (WidgetDecl, MsgDecl, etc.)

auto-ui/src/ui_gen/
├── mod.rs              # 后端生成器入口
├── react.rs            # React/TypeScript 后端
├── compose.rs          # Jetpack Compose/Kotlin 后端
├── gpui.rs             # GPUI + AutoVM 后端
└── style.rs            # 样式处理
```

### 核心类型（types.rs）

```rust
/// AURA Widget：核心组件定义
pub struct AuraWidget {
    pub name: String,

    // 状态定义：提取出的纯粹状态签名
    pub state_vars: Vec<AuraStateDef>,

    // 视图树：纯粹的布局与绑定，没有任何逻辑
    pub view_tree: AuraNode,

    // 逻辑载荷：保留为 AST 块或编译为 Bytecode
    pub handlers: HashMap<String, LogicPayload>,
}

/// 状态定义
pub struct AuraStateDef {
    pub name: String,       // e.g., "count"
    pub type_info: Type,    // e.g., Type::Int
    pub initial: Expr,      // 初始值 AST
}

/// 视图节点
pub enum AuraNode {
    Element {
        tag: String,                  // e.g., "col", "button"
        props: HashMap<String, Expr>, // 包含动态绑定的表达式
        events: HashMap<String, String>, // e.g., {"onclick": "Msg::Inc"}
        children: Vec<AuraNode>,
    },
    Text(String),
}

/// 逻辑载荷：支持 AOT 和 动态两种模式
pub enum LogicPayload {
    AstBlock(Vec<Stmt>),    // 供 AOT 转译器使用 (React/Compose)
    Bytecode(Vec<u8>),      // 供 AutoVM 动态执行使用 (GPUI)
}
```

---

## 表层语法：AutoUI Surface

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

**View 语法特殊字符**:
- `button +` - 增量按钮（显示 "+"）
- `button -` - 减量按钮（显示 "-"）
- `h2 > Current Count:` - 文本节点（`>` 后为文本内容）
- `${.count}` - 状态插值（`.` 前缀表示状态引用）
- `.Inc` - 隐式成员推导（等价于 `Counter.Msg.Inc`）

---

## CompilerSession 架构

### session.rs

```rust
/// 编译场景
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    Core,   // 标准 Auto 语言
    UI,     // UI 场景
    Shell,  // Shell 脚本场景
}

/// 编译器会话：向下游传递场景信息
pub struct CompilerSession {
    pub scenario: Scenario,
    pub backend: Option<String>,  // "react", "compose", "gpui", etc.
    // 可扩展：错误收集器、SourceMap 引用等
}
```

### CLI 调度

```bash
# 标准编译（读取 pac.at 中的 scenario）
auto build src/main.at

# 显式指定场景
auto build -s ui src/App.at

# 指定场景和后端
auto build -s ui -b react src/App.at
```

---

## Parser 上下文关键字实现

### parser.rs

```rust
impl<'a> Parser<'a> {
    pub fn new(source: &'a str, session: CompilerSession) -> Self { ... }

    fn parse_statement(&mut self) -> Result<AstNode> {
        let token = self.peek();

        // 仅在 UI 场景下，拦截特定的 Identifier 作为关键字
        if self.session.scenario == Scenario::UI {
            match token.as_str() {
                "widget" => return self.parse_widget_decl(),
                "msg" => return self.parse_msg_decl(),
                // ... 处理其他 UI 独有语法
                _ => {}
            }
        }

        // 如果不是 UI 场景，或者标识符不匹配，走常规 Auto 语句解析
        self.parse_expression_statement()
    }
}
```

**优势**: 在 `scenario: "core"` 的后端项目中，开发者完全可以定义 `let widget = create_window();` 而不会触发任何语法冲突。

---

## 后端转译层

AURA 的高度解耦让代码生成变得极其简单和机械化。

### 目标 A: React (TypeScript)

1. **翻译 Model**: `count` → `const [count, setCount] = useState(0);`
2. **改写 Handlers**:
   - `AssignOp( +=, ".count", 1 )` → `setCount(prev => prev + 1);`
3. **生成 JSX**: `tag: "col"` → `<div className="flex-col">`

### 目标 B: Jetpack Compose (Kotlin)

1. **翻译 Model**: `var count by remember { mutableStateOf(0) }`
2. **改写 Handlers**: `val handleInc = { count += 1 }`
3. **生成 View**: `col` → `Column { ... }`, `button` → `Button(onClick = handleInc) { ... }`

### 目标 C: GPUI + AutoVM (动态)

1. **编译 Handlers**: 将 `on` 块编译为 `LogicPayload::Bytecode`
2. **下发 AURA**: 将 AURA 序列化格式和 Bytecode 发送给 GPUI 客户端
3. **运行时渲染**: GPUI 根据 AURA 构建原生控件，点击时触发 Bytecode 执行

---

## 分阶段实施计划

### Phase 0: 基础设施（2-3天）

**目标**: AURA 核心结构定义，场景机制可用

**关键文件**:
- `auto-lang/src/session.rs` - 新建
- `auto-lang/src/aura/mod.rs` - 新建
- `auto-lang/src/aura/types.rs` - 新建
- `auto-lang/src/aura/extract.rs` - 新建

**任务**:
- [ ] 实现 `Scenario` enum 和 `CompilerSession`
- [ ] 定义 AURA 核心数据结构 (AuraWidget, AuraStateDef, AuraNode, LogicPayload)
- [ ] 实现 AST → AURA 抽取器（手动，暂不用 Plugin）
- [ ] 添加单元测试

**验证点**: `Counter.at` → WidgetDecl AST → AuraWidget 结构正确

### Phase 1: Parser 上下文关键字（3-4天）

**目标**: UI 关键字在 UI 场景下成为一等公民

**关键文件**:
- `auto-lang/src/parser.rs` - 修改（添加场景判断）
- `auto-lang/src/ast/ui.rs` - 新建

**任务**:
- [ ] 修改 Parser 接受 CompilerSession
- [ ] 实现上下文关键字判断逻辑
- [ ] 添加 UI AST 节点定义 (WidgetDecl, MsgDecl, ViewBlock, OnBlock)
- [ ] 解析 view 特殊语法 (`+`, `-`, `>`, `${.}`)
- [ ] 集成测试

**验证点**: Parser 在 `scenario: UI` 时正确识别 `widget`, `msg`, `model`

### Phase 2: 后端生成器（4-5天）

**目标**: React 和 GPUI 后端从 AURA 工作

**关键文件**:
- `auto-ui/src/ui_gen/mod.rs` - 新建
- `auto-ui/src/ui_gen/react.rs` - 新建
- `auto-ui/src/ui_gen/compose.rs` - 新建
- `auto-ui/src/ui_gen/gpui.rs` - 新建

**任务**:
- [ ] 实现 React/TypeScript 生成器
- [ ] 实现 Jetpack Compose/Kotlin 生成器
- [ ] 实现 GPUI + AutoVM 生成器
- [ ] 添加后端测试

**验证点**: 所有后端从 AURA 生成可运行代码

### Phase 3: CLI 与 pac.at 集成（3-4天）

**目标**: CLI 支持 `-s` 和 `-b` 参数，读取 pac.at

**关键文件**:
- `auto-lang/src/cli.rs` - 修改
- `auto-ui/src/trans/api.rs` - 修改

**任务**:
- [ ] 实现 pac.at 解析
- [ ] CLI 支持 `-s <scenario>` 和 `-b <backend>`
- [ ] 更新转译 API 入口
- [ ] 热重载集成

**验证点**: `auto build -s ui -b react Counter.at` 工作

### Phase 4: 清理与废弃（2-3天）

**目标**: 移除遗留代码

**关键文件**:
- `auto-ui/src/trans/dsl_preprocess.rs` - 废弃
- `docs/` - 更新文档

**任务**:
- [ ] 所有示例使用 AURA 路径
- [ ] 标记 `dsl_preprocess.rs` 为废弃
- [ ] 更新文档
- [ ] 性能基准测试

**验证点**: 代码库干净，所有测试通过

---

## 里程碑进度表

| Phase | 预计时间 | 状态 | 交付物 |
|-------|---------|------|--------|
| Phase 0 | 2-3天 | ⏳ 待开始 | AURA 核心 + CompilerSession |
| Phase 1 | 3-4天 | ⏳ 待开始 | 上下文关键字解析 |
| Phase 2 | 4-5天 | ⏳ 待开始 | 多后端支持 |
| Phase 3 | 3-4天 | ⏳ 待开始 | CLI 集成 |
| Phase 4 | 2-3天 | ⏳ 待开始 | 遗留代码清理 |

**总计**: 14-19 天

---

## 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 上下文关键字破坏现有代码 | 低 | 高 | 仅在 scenario=UI 时激活 |
| AURA 表达能力不足 | 低 | 中 | 保留 AST 逃逸通道 |
| 性能回退 | 低 | 中 | 基准测试对比 |
| pac.at 解析失败 | 中 | 中 | 提供命令行覆盖 |

### 回退策略

- Phase 0: 默认 scenario=Core → 无变化
- Phase 1: scenario=Core 时 parser 行为不变
- Phase 2: 保留旧转译路径
- Phase 3: CLI `-s core` 强制使用核心编译
- Phase 4: 主版本升级 → 移除废弃代码

---

## 验证标准

### 技术指标
- 编译时间: < 100ms（典型组件）
- 代码质量: 无 clippy 警告
- 测试覆盖: > 80%（新代码）
- 错误信息: 指向原始 DSL 源码

### 功能要求
- [ ] `Counter.at` → WidgetDecl AST 解析正确
- [ ] WidgetDecl → AuraWidget 抽取正确
- [ ] React 后端生成可运行代码
- [ ] Compose 后端生成可运行代码
- [ ] GPUI + AutoVM 后端工作
- [ ] 事件处理器正确绑定
- [ ] `${.state}` 插值正确处理

---

## 关键文件清单

### 需要新建（auto-lang）
1. `src/session.rs` - CompilerSession + Scenario
2. `src/aura/mod.rs` - AURA 模块入口
3. `src/aura/types.rs` - 核心类型定义
4. `src/aura/extract.rs` - AST → AURA 抽取
5. `src/aura/atom.rs` - Atom 格式序列化
6. `src/ast/ui.rs` - UI AST 节点

### 需要新建（auto-ui）
1. `src/ui_gen/mod.rs` - 后端生成器入口
2. `src/ui_gen/react.rs` - React/TypeScript 生成
3. `src/ui_gen/compose.rs` - Jetpack Compose/Kotlin 生成
4. `src/ui_gen/gpui.rs` - GPUI + AutoVM 生成
5. `src/ui_gen/style.rs` - 样式处理

### 需要修改
1. `auto-lang/src/lib.rs` - 导出 session 和 aura
2. `auto-lang/src/parser.rs` - 集成上下文关键字
3. `auto-lang/src/cli.rs` - 添加 `-s` 和 `-b` 参数
4. `auto-ui/src/trans/api.rs` - 添加 AURA 入口

### 最终废弃
1. `auto-ui/src/trans/dsl_preprocess.rs` - 文本级预处理

---

## 架构收益

1. **无状态视图的纯粹性**: AURA 的 `view_tree` 绝对干净，不包含任何业务逻辑，为可视化编辑器双向同步铺平道路
2. **极低的转译器开发成本**: 新增目标平台只需编写几百行的 AURA 遍历生成器
3. **多端性能极致**: AOT 生成目标平台最原生代码，无运行时损耗
4. **语法隔离**: `widget` 等关键字仅在 UI 场景生效，Core 项目不受影响
