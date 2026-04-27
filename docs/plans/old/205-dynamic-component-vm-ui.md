# Plan 205: DynamicComponent — AutoVM 驱动的动态 UI 渲染

## Status: ✅ COMPLETE

Verified 2026-04-24:
- ✅ Phase 1: VmBridge — VM ↔ UI bridge with state read/write, handler call
- ✅ Phase 2: AuraViewBuilder — AuraNode → View<DynamicMsg> for all core widgets
- ✅ Phase 3: DynamicComponent — Component trait, message routing, dirty tracking
- ✅ Phase 4: Hot reload — state migration, file change detection, DynamicComponent.reload()
- ✅ Phase 5: iced integration — IntoIcedElement trait, full rendering pipeline (25k+ lines)
- ⚠️ Advanced features deferred: ForLoop iteration, Conditional eval, Component instantiation

## Overview

将 AutoVM 集成到 UI 运行时中，实现 DynamicComponent，使 AURA 脚本无需经过 a2r 转译即可动态构建界面。目标是消除 AOT 编译延迟，实现 .at 文件修改后秒级生效的热重载开发体验。

**核心思路**：VM 持有 widget state + handler bytecode，view tree 直接从 AURA IR 构建（纯结构遍历），事件通过 Msg 枚举路由到 VM handler，不依赖 VM Closure。

## Architecture

```
.at 文件修改
    ↓ (notify 文件监听)
重新解析 → AuraWidget IR
    ↓
┌─────────────────────────────────────────────┐
│           DynamicComponent                   │
│                                              │
│  vm: AutoVM                                  │
│    ├── state object (model 字段)              │
│    └── handler bytecode (on 块逻辑)          │
│                                              │
│  view_template: AuraNode (纯结构模板)         │
│  handler_map: MsgName → VM func_addr         │
│  msg_variants: Vec<AuraMsgVariant>           │
│  dirty: bool                                 │
└──────────┬──────────────────────────────────┘
           │
     ┌─────┴──────────────────┐
     │   UAC Backend Bridge    │
     ├──────────┬──────────────┤
     │  iced    │    gpui      │
     │ Msg路由  │  Closure包装  │
     └──────────┴──────────────┘

view():  遍历 view_template → 读 VM state → 构建 View<DynamicMsg> → into_iced()
update(): DynamicMsg → handler_map → VM.call(addr) → state 变更 → dirty
reload(): 重解析 → 更新 bytecode + template + state 迁移 → dirty
```

## Key Design Decisions

### 1. AURA IR 直接构建 View（非 VM 执行）

AURA 的设计理念是 view 是纯结构（无逻辑）。因此 view() 直接遍历 AuraNode 树构建 View<DynamicMsg>，只有状态绑定表达式（如 `${.count}`）需要从 VM 读取值。这比每次让 VM 执行 view 函数更高效。

### 2. Msg 枚举路由（非 Closure）

AURA widget 定义 `msg Msg { Inc, Dec }`，所有事件在编译期就确定了枚举 variant。事件传递链路：

```
button click → DynamicMsg::Variant{widget:"Counter", variant:"Inc"}
    → update() 查 handler_map["Inc"] → VM.call(handler_addr)
```

**不需要 VM Closure 支持**。对于 gpui 后端，UAC bridge 层在 Rust 侧构造 `impl Fn(Event) -> Msg` 闭包即可。

### 3. VM 持有 State + Handler

- **State**：在 VM 中以对象实例存储，handler 通过 `GET_FIELD`/`SET_FIELD` 直接访问
- **Handler**：每个 `on` 块编译为 VM 函数，`CALL` 调用，`RET` 返回
- **VM 需要的能力**：基本算术、控制流、对象字段读写 —— 当前 VM 已全部支持

### 4. DynamicMsg 定义

```rust
#[derive(Clone, Debug)]
pub enum DynamicMsg {
    /// 简单枚举 variant（如 Msg::Inc）
    Variant { widget: String, variant: String },
    /// 带参数的 variant（如 Msg::SetValue(i32)）
    WithArgs { widget: String, variant: String, args: Vec<Value> },
}
```

## Core Components

### Component 1: VmBridge（替代 InterpreterBridge）

替换现有的 `interpreter/bridge.rs`，用 AutoVM 替代旧的 AST Interpreter。

```rust
pub struct VmBridge {
    /// AutoVM 实例
    vm: AutoVM,
    /// Widget state 在 VM 中的对象 ID
    state_obj_id: u64,
    /// Handler 函数地址映射：event_name → func_addr
    handler_map: HashMap<String, u32>,
}
```

**职责**：
- 初始化 VM、创建 state 对象、加载 handler bytecode
- 提供 `read_state(field_name) -> Value` 接口供 view 构建使用
- 提供 `call_handler(event_name, args)` 接口执行 handler
- 支持热重载时重新加载 bytecode + state 迁移

### Component 2: AuraViewBuilder（AuraNode → View<DynamicMsg>）

新增模块，从 AuraNode 模板树直接构建 View<DynamicMsg>。

```rust
pub struct AuraViewBuilder<'a> {
    /// VM bridge 用于读取状态值
    bridge: &'a VmBridge,
    /// 当前 widget 名称
    widget_name: String,
}

impl AuraViewBuilder {
    /// 从 AuraNode 模板构建 View<DynamicMsg>
    pub fn build(&self, node: &AuraNode) -> View<DynamicMsg> { ... }
}
```

**状态绑定解析**：AuraNode 中的 `AuraExpr::StateRef("count")` → `bridge.read_state("count")`

### Component 3: DynamicComponent（实现 Component trait + iced 接口）

```rust
pub struct DynamicComponent {
    /// VM 桥梁
    bridge: VmBridge,
    /// AURA view 模板
    view_template: AuraNode,
    /// Widget 元数据
    widget_meta: AuraWidgetMeta,
    /// dirty 标记
    dirty: bool,
    /// 文件监听器
    watcher: Option<UIWatcher>,
}

impl Component for DynamicComponent {
    type Msg = DynamicMsg;

    fn on(&mut self, msg: DynamicMsg) {
        match msg {
            DynamicMsg::Variant { variant, .. } => {
                self.bridge.call_handler(&variant, &[]);
            }
            DynamicMsg::WithArgs { variant, args, .. } => {
                self.bridge.call_handler(&variant, &args);
            }
        }
        self.dirty = true;
    }

    fn view(&self) -> View<DynamicMsg> {
        let builder = AuraViewBuilder::new(&self.bridge, &self.widget_meta.name);
        builder.build(&self.view_template)
    }
}
```

### Component 4: State Migration（热重载状态迁移）

热重载时保留兼容的 state 字段：

```rust
fn migrate_state(old_state: &HashMap<String, Value>, new_fields: &[AuraStateDef]) -> HashMap<String, Value> {
    let mut new_state = HashMap::new();
    for field in new_fields {
        if let Some(old_val) = old_state.get(&field.name) {
            // 类型兼容 → 保留旧值
            if type_compatible(old_val, &field.ty) {
                new_state.insert(field.name.clone(), old_val.clone());
            } else {
                new_state.insert(field.name.clone(), field.default_value.clone());
            }
        } else {
            // 新增字段 → 用默认值
            new_state.insert(field.name.clone(), field.default_value.clone());
        }
    }
    new_state
}
```

## File Structure

```
crates/auto-lang/src/ui/
├── component.rs              # 现有 Component trait（不变）
├── dynamic.rs                # [NEW] DynamicComponent 主结构
├── vm_bridge.rs              # [NEW] VmBridge — VM ↔ UI 桥梁
├── aura_view_builder.rs      # [NEW] AuraNode → View<DynamicMsg> 转换
├── state_migration.rs        # [NEW] 热重载状态迁移
├── interpreter/
│   ├── mod.rs                # 保留，添加 VmBridge re-export
│   └── bridge.rs             # 现有（降级为 legacy，保留向后兼容）
├── node_converter.rs         # 现有（保留，AuraViewBuilder 可复用部分 helper）
├── hot_reload.rs             # 升级：添加 DynamicComponent 热重载逻辑
├── event_router.rs           # 现有（保留，DynamicComponent 内部简化路由）
├── iced/
│   └── renderer.rs           # 添加 IntoIcedElement<DynamicMsg> 实现
└── gpui/
    └── renderer.rs           # 后续：gpui bridge Closure 包装
```

## Implementation Phases

### Phase 1: VmBridge 核心能力（~3 天）

**目标**：让 VM 能加载 AURA widget 的 state 和 handler

| Step | Task | Files |
|------|------|-------|
| 1.1 | 实现 `VmBridge::new()` — 创建 VM、注册必要 native 函数 | `ui/vm_bridge.rs` |
| 1.2 | 实现 `VmBridge::load_widget()` — 从 AuraWidget 初始化 state 对象 | `ui/vm_bridge.rs` |
| 1.3 | 实现 handler bytecode 编译 — AuraStmt → VM bytecode | `ui/vm_bridge.rs` 借用 `vm/codegen.rs` |
| 1.4 | 实现 `VmBridge::read_state()` 和 `call_handler()` | `ui/vm_bridge.rs` |
| 1.5 | 单元测试：加载 Counter widget，读写 state，调用 handler | `ui/vm_bridge.rs` tests |

### Phase 2: AuraViewBuilder（~2 天）

**目标**：从 AuraNode 模板构建 View<DynamicMsg>

| Step | Task | Files |
|------|------|-------|
| 2.1 | 定义 DynamicMsg 类型（在 `ui/interpreter/mod.rs` 中更新） | `ui/interpreter/mod.rs` |
| 2.2 | 实现 AuraViewBuilder — 遍历 AuraNode → View<DynamicMsg> | `ui/aura_view_builder.rs` |
| 2.3 | 实现状态绑定解析 — `${.field}` 从 VmBridge 读取值 | `ui/aura_view_builder.rs` |
| 2.4 | 复用 node_converter 的 helper 函数 | `ui/node_converter.rs` 小幅调整 |
| 2.5 | 单元测试：Counter widget 的 view 构建和状态绑定 | `ui/aura_view_builder.rs` tests |

### Phase 3: DynamicComponent + iced 集成（~3 天）

**目标**：DynamicComponent 可在 iced 中运行

| Step | Task | Files |
|------|------|-------|
| 3.1 | 实现 DynamicComponent 结构体 | `ui/dynamic.rs` |
| 3.2 | 实现 iced 的 Sandbox/Program trait | `ui/dynamic.rs` + `ui/iced/renderer.rs` |
| 3.3 | 实现 IntoIcedElement for View<DynamicMsg> | `ui/iced/renderer.rs` |
| 3.4 | 端到端测试：Counter widget 在 iced 窗口中运行 | `ui/dynamic.rs` tests |
| 3.5 | 添加 `run_dynamic(path: &Path)` 入口函数 | `ui/app.rs` |

### Phase 4: 热重载集成（~2 天）

**目标**：修改 .at 文件后自动更新界面

| Step | Task | Files |
|------|------|-------|
| 4.1 | 实现 state_migration — 保留兼容字段 | `ui/state_migration.rs` |
| 4.2 | 升级 hot_reload — 文件变化 → 重解析 → 更新 VmBridge + template | `ui/hot_reload.rs` |
| 4.3 | DynamicComponent 集成 UIWatcher | `ui/dynamic.rs` |
| 4.4 | 端到端测试：修改 Counter → 热重载生效 | `ui/dynamic.rs` tests |

### Phase 5: 完善 + 示例（~2 天）

**目标**：打磨体验，添加更多 widget 支持

| Step | Task | Files |
|------|------|-------|
| 5.1 | 支持更多 AuraNode 类型（Input, Checkbox, Select, List 等） | `ui/aura_view_builder.rs` |
| 5.2 | 错误处理和用户友好提示（解析失败、handler 执行错误） | 全局 |
| 5.3 | 添加示例：Counter, TodoList, Form | `examples/dynamic-ui/` |
| 5.4 | CLI 集成：`auto dev` 命令启动动态 UI 开发模式 | `crates/auto/src/main.rs` |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| VM handler 执行耗时阻塞 iced 渲染 | UI 卡顿 | 限制 handler 执行指令数上限，超时 yield |
| AURA IR 遍历 + 状态读取性能 | 每帧开销 | 缓存 View，仅在 dirty 时重建 |
| State 迁移类型不兼容 | 状态丢失 | 提供可视化迁移提示，保留旧值直到用户确认 |
| AuraNode 不覆盖所有 View variant | 部分组件不支持 | Phase 5 逐步扩展，先支持核心组件 |

## Dependencies

- Plan 201（VM enum/closure/result/spec）：handler 中的 enum match 需要良好的 VM 支持
- Plan 197（VM ADT/generic/list）：复杂 widget state（List<T>, Map<K,V>）需要这些能力
- 现有 AURA extraction pipeline（`aura/extract.rs`）：必须能正确提取 AuraWidget
