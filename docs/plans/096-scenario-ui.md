# UI-IR 架构迁移计划

## 背景

### 当前问题
当前的 DSL 预处理方式（`widget Counter` → `type Counter is Widget`）存在以下问题：
- UI 声明 → AutoAST → UI-IR 的转换过于复杂
- 文本级别的宏展开丢失语义信息，错误定位困难
- 缺少专门的 UI 场景 AST 结构
- 多后端支持时需要重复解析

### 目标架构
```
DSL (counter_new.at)
    ↓
UI Parser Plugin（scenario=ui）
    ↓
UI AST（widget/msg/model/view/on 为一等公民）
    ↓
UI-IR（结构化中间表示）
    ↓
后端代码生成（GPUI / Vue / Iced）
```

---

## UI-IR 核心数据结构

### 模块结构

**设计决策**: UI-IR 放在 **auto-lang** crate，保持更强的集成和类型一致性。

```
auto-lang/src/ui_ir/
├── mod.rs          # 模块入口，导出核心类型
├── types.rs        # UI-IR 核心类型定义
├── convert.rs      # AST → UI-IR 转换
└── lower.rs        # UI-IR 降级（优化/简化）

auto-ui/src/ui_gen/
├── mod.rs          # 后端生成器入口
├── gpui.rs         # GPUI 后端生成器
├── vue.rs          # Vue 后端生成器
└── style.rs        # 样式处理
```

### 核心类型（types.rs）

```rust
/// UI 模块：包含消息、组件和应用定义
pub struct UIModule {
    pub name: String,
    pub messages: Vec<UIMessage>,
    pub widgets: Vec<UIWidget>,
    pub app: Option<UIApp>,
}

/// 消息类型定义
pub struct UIMessage {
    pub name: String,
    pub variants: Vec<UIMsgVariant>,
}

/// 组件定义
pub struct UIWidget {
    pub name: String,
    pub model: UIModel,       // 状态字段
    pub view: UIView,         // 视图树
    pub on: UIOn,             // 事件处理
    pub style: Option<UIStyle>,
    pub props: Vec<UIProp>,   // 可复用组件的属性
}

/// 视图节点
pub struct UINode {
    pub kind: UINodeKind,
    pub props: HashMap<String, UIExpr>,
    pub children: Vec<UINode>,
}

/// 节点类型
pub enum UINodeKind {
    // 布局
    Col, Row, Center, Container, Scrollable,
    // 基础
    Text, Button, Input, Checkbox, Radio, Select,
    // 高级
    List, Table, Slider, ProgressBar, Tabs,
    // 自定义组件引用
    Widget(String),
}

/// 事件处理器
pub struct UIHandler {
    pub pattern: String,      // "CounterMsg.Inc"
    pub body: Vec<UIStmt>,    // 处理语句
}

/// UI 表达式
pub enum UIExpr {
    Literal(String),
    Int(i64),
    FieldRef(String),         // self.count
    MsgVariant(String, String), // CounterMsg.Inc
    Interpolation(String),    // ${count}
}

/// UI 语句
pub enum UIStmt {
    Assign { target: String, value: UIExpr },
    Update { target: String, op: UpdateOp, value: UIExpr },
}
```

---

## auto-lang 扩展方案

### 1. CompileMode 扩展

**文件**: `auto-lang/src/lib.rs`

```rust
pub enum CompileMode {
    Script,
    Config,
    Template,
    UI,  // 新增：UI 场景
}
```

### 2. Parser Plugin 架构

**文件**: `auto-lang/src/parser/plugin.rs`（新建）

```rust
/// 解析器插件 trait
pub trait ParserPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn keywords(&self) -> &[&str];
    fn can_handle(&self, token: &Token) -> bool;
    fn parse_block(&self, parser: &mut Parser) -> AutoResult<Option<Stmt>>;
}

/// 插件注册表
pub struct PluginRegistry {
    plugins: Vec<Box<dyn ParserPlugin>>,
}
```

### 3. UI AST 节点

**文件**: `auto-lang/src/ast/ui.rs`（新建）

```rust
/// widget 声明
pub struct WidgetDecl {
    pub name: Name,
    pub model: Option<ModelBlock>,
    pub view: Option<ViewBlock>,
    pub on: Option<OnBlock>,
    pub style: Option<StyleBlock>,
}

/// msg 声明
pub struct MsgDecl {
    pub name: Name,
    pub variants: Vec<MsgVariant>,
}

/// view 块中的 UI 节点
pub struct UINodeExpr {
    pub kind: String,
    pub args: Vec<Expr>,
    pub props: Vec<(Name, Expr)>,
    pub children: Vec<UINodeExpr>,
}
```

---

## 分阶段实施计划

### Phase 0: 基础设施（2-3天）✅ MVP

**目标**: UI-IR 核心结构定义，基础解析可用

**关键文件**:
- `auto-lang/src/ui_ir/mod.rs` - 新建
- `auto-lang/src/ui_ir/types.rs` - 新建
- `auto-lang/src/ui_ir/convert.rs` - 新建
- `auto-ui/src/ui_gen/mod.rs` - 新建
- `auto-ui/src/ui_gen/gpui.rs` - 新建

**任务**:
- [ ] 在 auto-lang 中定义 UI-IR 核心数据结构
- [ ] 实现 AST → UI-IR 转换器（手动转换，暂不用 Plugin）
- [ ] 在 auto-ui 中实现基础 GPUI 代码生成
- [ ] 添加单元测试

**验证点**: `counter_new.at` → UI-IR → 有效 GPUI 代码

### Phase 1: Parser 插件集成（3-4天）

**目标**: UI 关键字成为一等公民

**关键文件**:
- `auto-lang/src/parser/plugin.rs` - 新建
- `auto-lang/src/parser/plugins/ui.rs` - 新建
- `auto-lang/src/ast/ui.rs` - 新建
- `auto-lang/src/lib.rs` - 修改（添加 CompileMode::UI）

**任务**:
- [ ] 设计并实现 PluginRegistry
- [ ] 实现 UIParserPlugin
- [ ] 添加 UI AST 节点定义
- [ ] 集成测试

**验证点**: Parser 正确识别 `widget`, `msg`, `model` 关键字

### Phase 2: 多后端生成器（4-5天）

**目标**: GPUI 和 Vue 后端从 UI-IR 工作

**关键文件**:
- `auto-ui/src/ui_gen/gpui.rs` - 完善
- `auto-ui/src/ui_gen/vue.rs` - 新建
- `auto-ui/src/ui_gen/style.rs` - 新建

**任务**:
- [ ] 完善 GPUI 生成器
- [ ] 实现 Vue 生成器
- [ ] 实现样式处理（Tailwind 类名）
- [ ] 添加后端测试

**验证点**: 所有后端从 UI-IR 生成正确代码

### Phase 3: 集成与迁移（3-4天）

**目标**: CLI 和 API 支持 UI-IR 路径

**关键文件**:
- `auto-ui/src/trans/api.rs` - 修改
- `auto-ui/examples/` - 迁移示例

**任务**:
- [ ] 更新转译 API 入口
- [ ] 添加 `// @ui-ir` 指令支持
- [ ] 迁移现有示例到新路径
- [ ] 热重载集成

**验证点**: `autoc transpile counter.ui.at --backend gpui` 工作

### Phase 4: 清理与废弃（2-3天）

**目标**: 移除遗留代码

**关键文件**:
- `auto-ui/src/trans/dsl_preprocess.rs` - 废弃
- `docs/` - 更新文档

**任务**:
- [ ] 所有示例使用 UI-IR 路径
- [ ] 标记 `dsl_preprocess.rs` 为废弃
- [ ] 更新文档
- [ ] 性能基准测试

**验证点**: 代码库干净，所有测试通过

---

## 里程碑进度表

| Phase | 预计时间 | 状态 | 交付物 |
|-------|---------|------|--------|
| Phase 0 | 2-3天 | ⏳ 待开始 | UI-IR 核心 + GPUI 基础生成 |
| Phase 1 | 3-4天 | ⏳ 待开始 | Parser 插件系统 |
| Phase 2 | 4-5天 | ⏳ 待开始 | 多后端支持 |
| Phase 3 | 3-4天 | ⏳ 待开始 | API 集成 |
| Phase 4 | 2-3天 | ⏳ 待开始 | 遗留代码清理 |

**总计**: 14-19 天

---

## 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| Parser 插件破坏现有代码 | 中 | 高 | 并行实现，特性开关 |
| UI-IR 表达能力不足 | 低 | 中 | 保留 AST 逃逸通道 |
| 性能回退 | 低 | 中 | 基准测试对比 |
| 增量编译中断 | 低 | 高 | 保留遗留路径 |

### 回退策略

每个阶段保持向后兼容：
- Phase 0: 特性开关关闭 → 无变化
- Phase 1: 特性开关关闭 → 使用旧 parser
- Phase 2: 文件扩展名检测 → `.at` 用遗留，`.ui.at` 用新路径
- Phase 3: 指令检测 → `// @legacy` 使用旧路径
- Phase 4: 主版本升级 → 移除废弃代码

---

## 验证标准

### 技术指标
- 编译时间: < 100ms（典型组件）
- 代码质量: 无 clippy 警告
- 测试覆盖: > 80%（新代码）
- 错误信息: 指向原始 DSL 源码

### 功能要求
- [ ] `counter_new.at` → UI-IR 解析正确
- [ ] GPUI 后端生成可运行代码
- [ ] Vue 后端生成可运行代码
- [ ] 热重载与 UI-IR 协作
- [ ] 样式类正确应用
- [ ] 事件处理器正确绑定

---

## 关键文件清单

### 需要新建（auto-lang）
1. `auto-lang/src/ui_ir/mod.rs` - UI-IR 模块入口
2. `auto-lang/src/ui_ir/types.rs` - 核心类型定义
3. `auto-lang/src/ui_ir/convert.rs` - AST → UI-IR 转换
4. `auto-lang/src/parser/plugin.rs` - Parser 插件系统（Phase 1）
5. `auto-lang/src/parser/plugins/ui.rs` - UI Parser 插件（Phase 1）
6. `auto-lang/src/ast/ui.rs` - UI AST 节点（Phase 1）

### 需要新建（auto-ui）
1. `auto-ui/src/ui_gen/mod.rs` - 后端生成器入口
2. `auto-ui/src/ui_gen/gpui.rs` - GPUI 代码生成
3. `auto-ui/src/ui_gen/vue.rs` - Vue 代码生成
4. `auto-ui/src/ui_gen/style.rs` - 样式处理

### 需要修改
1. `auto-lang/src/lib.rs` - 添加 CompileMode::UI 和导出 ui_ir
2. `auto-lang/src/parser.rs` - 集成插件系统（Phase 1）
3. `auto-ui/src/trans/api.rs` - 添加 UI-IR 入口
4. `auto-ui/Cargo.toml` - 添加特性开关

### 最终废弃
1. `auto-ui/src/trans/dsl_preprocess.rs` - 文本级预处理

---

## 下一步行动

1. **启动 Phase 0** - 在 `auto-lang/src/ui_ir/` 创建 UI-IR 模块
2. **实现类型定义** - 完成 `types.rs` 中的核心结构
3. **实现 AST 转换** - 完成 `convert.rs` 中的转换逻辑
4. **建立测试框架** - 使用 `counter_new.at` 作为首个测试用例
