# Plan 101: DevTools 与热重载综合计划

## 背景

UI 开发中，DevTools 和热重载是提升开发效率的关键工具。AutoLang 支持多条 UI 编译路线，每条路线的 DevTools 策略不同：

| 路线 | 目标平台 | DevTools 来源 |
|------|---------|--------------|
| a2vue | Web (Vue) | 浏览器 DevTools |
| a2jet | Android (Jetpack Compose) | Android Studio |
| a2ark | HarmonyOS (ArkTS) | DevEco Studio |
| a2rust | Desktop (gpui/iced) | **需要自建** |

---

## 路线 1: a2vue (Web/Vue)

### DevTools 来源
**浏览器内置 DevTools** - 开箱即用

### 可用功能
| 功能 | 来源 | 状态 |
|-----|------|------|
| 元素检查器 | Chrome/Firefox DevTools | ✅ 成熟 |
| 布局可视化 | 浏览器 | ✅ 成熟 |
| 样式编辑 | 浏览器 | ✅ 成熟 |
| 控制台 | 浏览器 | ✅ 成熟 |
| 网络监控 | 浏览器 | ✅ 成熟 |
| Vue 组件树 | Vue DevTools 扩展 | ✅ 成熟 |
| 性能分析 | 浏览器 Performance | ✅ 成熟 |

### 热重载方案
```
源文件 (.at)  →  编译  →  Vue SFC (.vue)  →  Vite HMR  →  浏览器
     ↓                                              ↓
  文件监听 ←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←← 热更新
```

**实现步骤**：
1. 使用 `notify` crate 监听 `.at` 文件变化
2. 增量编译修改的文件（利用 AIE 增量编译）
3. 生成/更新 `.vue` 文件
4. Vite 自动触发 HMR

**优先级**: P2（Vite HMR 已成熟，只需实现文件监听）

---

## 路线 2: a2jet (Android/Jetpack Compose)

### DevTools 来源
**Android Studio 内置工具**

### 可用功能
| 功能 | 来源 | 状态 |
|-----|------|------|
| Live Edit | Android Studio Giraffe+ | ⚠️ 实验性 |
| Live Previews | Android Studio | ✅ 可用 |
| Layout Inspector | Android Studio | ✅ 成熟 |
| Recomposition Tracer | Android Studio | ✅ 可用 |
| Logcat | Android Studio | ✅ 成熟 |
| Database Inspector | Android Studio | ✅ 成熟 |

### 热重载方案
```
源文件 (.at)  →  编译  →  Jetpack Compose (.kt)  →  Android Studio
                                                               ↓
                                                         Live Edit
                                                               ↓
                                                          模拟器/真机
```

**限制**：
- Live Edit 仍处于实验阶段
- 部分代码修改需要完全重新编译
- 依赖 Android Studio 运行

**工作流程**：
1. AutoLang 编译器生成 Jetpack Compose 项目
2. 用户在 Android Studio 中打开项目
3. 使用 Android Studio 的 Live Edit 和其他工具

**优先级**: P3（依赖 IDE，无法独立实现）

---

## 路线 3: a2ark (HarmonyOS/ArkTS)

### DevTools 来源
**DevEco Studio 内置工具**

### 可用功能
| 功能 | 来源 | 状态 |
|-----|------|------|
| Hot Reload | DevEco Studio 5.1.1+ | ✅ 成熟 |
| 实时预览器 | DevEco Studio | ✅ 亚秒级 |
| Layout Inspector | DevEco Studio | ✅ 可用 |
| 日志控制台 | DevEco Studio | ✅ 成熟 |
| 性能分析 | DevEco Studio | ✅ 可用 |

### 预览器注解
```typescript
@Entry    // 页面预览
@Preview  // 组件预览（每文件最多10个）
@Component
struct MyComponent { ... }
```

### 热重载方案
```
源文件 (.at)  →  编译  →  ArkTS (.ets)  →  DevEco Studio
                                                   ↓
                                               Hot Reload
                                                   ↓
                                             模拟器/真机
```

**效率数据**：
- 万行级代码：比全量构建快 **70%+**
- 十万行级代码：比全量构建快 **50%+**

**工作流程**：
1. AutoLang 编译器生成 ArkTS 项目
2. 用户在 DevEco Studio 中打开项目
3. 使用 DevEco Studio 的 Hot Reload 和预览器

**优先级**: P3（依赖 IDE，无法独立实现）

---

## 路线 4: a2rust (Desktop/gpui/iced)

### 问题：自举困境

DevTools 本身是 UI → 需要用 Abstract UI Components 构建 → 但还没有 DevTools 控件

### 解决方案：分层实现

```
┌─────────────────────────────────────────────────────────────┐
│                    Phase 1: 最小调试层                        │
│  - 边框高亮 (debug_highlight)                                │
│  - 类型名显示                                                │
│  - 快捷键切换 (F12)                                          │
│  - 不需要额外 UI 控件                                         │
├─────────────────────────────────────────────────────────────┤
│                    Phase 2: Web DevTools                     │
│  - 独立浏览器窗口                                             │
│  - WebSocket 通信                                            │
│  - 复用浏览器成熟 UI                                          │
├─────────────────────────────────────────────────────────────┤
│                    Phase 3: 原生 DevTools (可选)              │
│  - 使用 auto-ui 组件自举                                      │
│  - 完全统一的技术栈                                           │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1: 最小调试层

### 目标
在不构建完整 DevTools UI 的情况下，提供基本的调试能力。

### 实现内容

#### 1.1 Debug 模式标记
```rust
// auto-ui crate
pub struct Widget {
    // 现有字段...
    debug_mode: bool,
    debug_name: Option<String>,
}

impl Widget {
    pub fn debug(mut self, name: &str) -> Self {
        self.debug_name = Some(name.to_string());
        self
    }
}
```

#### 1.2 边框高亮
```rust
impl Widget {
    pub fn paint_debug(&self, ctx: &mut PaintCtx) {
        if !self.debug_mode {
            return;
        }

        // 绘制边框
        let rect = self.bounds();
        ctx.stroke_rect(rect, DEBUG_COLOR, 1.0);

        // 绘制类型名
        if let Some(name) = &self.debug_name {
            ctx.draw_text(name, rect.origin(), DEBUG_TEXT_COLOR);
        }

        // 显示 padding/margin（可选）
        self.paint_spacing_debug(ctx);
    }
}
```

#### 1.3 全局快捷键
```rust
// 在应用事件循环中
fn handle_event(&mut self, event: &Event) {
    if let Event::KeyDown(key) = event {
        if key == Key::F12 {
            self.toggle_debug_mode();
        }
    }
}
```

#### 1.4 布局可视化
```rust
impl Widget {
    fn paint_spacing_debug(&self, ctx: &mut PaintCtx) {
        // Padding - 绿色
        ctx.fill_rect(self.padding_rect(), Color::GREEN.with_alpha(0.1));

        // Margin - 橙色
        ctx.fill_rect(self.margin_rect(), Color::ORANGE.with_alpha(0.1));

        // Border - 蓝色
        ctx.stroke_rect(self.border_rect(), Color::BLUE, 1.0);
    }
}
```

### 文件修改
- `crates/auto-ui/src/widget.rs` - 添加 debug 模式支持
- `crates/auto-ui/src/paint.rs` - 添加调试绘制方法
- `crates/auto-ui/src/event.rs` - 添加快捷键处理

### 验证方法
```bash
# 运行应用，按 F12 切换调试模式
cargo run --example todo_app

# 应看到：
# - 所有 widget 显示边框
# - 显示 widget 类型名
# - padding/margin 以不同颜色显示
```

---

## Phase 2: Web DevTools

### 架构
```
┌──────────────────────────────────────────────────────────────┐
│                     Desktop Application                       │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              Abstract UI Components                     │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │  │
│  │  │   Widget     │  │   Widget     │  │   Widget     │  │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  │  │
│  └────────────────────────────────────────────────────────┘  │
│                           │                                   │
│                           ▼                                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              DevTools Server (WebSocket)                │  │
│  │  - 暴露 widget 树结构                                    │  │
│  │  - 接收选择/修改命令                                     │  │
│  │  - 发送状态更新                                          │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                            │
                            │ WebSocket
                            ▼
┌──────────────────────────────────────────────────────────────┐
│                    Browser DevTools (Vue)                     │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐               │
│  │ Widget 树  │ │ 属性面板   │ │ 日志控制台 │               │
│  └────────────┘ └────────────┘ └────────────┘               │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐               │
│  │ 布局可视化 │ │ 样式编辑器 │ │ 性能面板   │               │
│  └────────────┘ └────────────┘ └────────────┘               │
└──────────────────────────────────────────────────────────────┘
```

### 协议设计
```rust
// DevTools 协议消息
#[derive(Serialize, Deserialize)]
pub enum DevToolsMessage {
    // 应用 → DevTools
    WidgetTree { root: WidgetNode },
    WidgetUpdate { id: WidgetId, changes: Vec<Change> },
    Log { level: LogLevel, message: String },
    Performance { fps: f64, frame_time: f64 },

    // DevTools → 应用
    SelectWidget { id: WidgetId },
    UpdateProperty { id: WidgetId, path: String, value: Value },
    HighlightWidget { id: WidgetId },
    RequestWidgetTree,
}
```

### DevTools Server (Rust)
```rust
// crates/auto-ui/src/devtools/server.rs
pub struct DevToolsServer {
    ws_server: WebSocketServer,
    widget_tree: WidgetTree,
}

impl DevToolsServer {
    pub fn start(port: u16) -> Result<Self> {
        // 启动 WebSocket 服务器
        // 等待浏览器连接
    }

    pub fn send_widget_tree(&self, root: &Widget) {
        // 序列化 widget 树
        // 发送到浏览器
    }

    pub fn on_message(&mut self, msg: DevToolsMessage) {
        match msg {
            DevToolsMessage::SelectWidget { id } => {
                // 高亮选中的 widget
            }
            DevToolsMessage::UpdateProperty { id, path, value } => {
                // 更新 widget 属性（支持热修改）
            }
            // ...
        }
    }
}
```

### DevTools Client (Vue)
```typescript
// devtools/src/App.vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'

const ws = ref<WebSocket>()
const widgetTree = ref<WidgetNode>()
const selectedWidget = ref<WidgetId>()

onMounted(() => {
  ws.value = new WebSocket('ws://localhost:9527')
  ws.value.onmessage = (e) => {
    const msg = JSON.parse(e.data)
    if (msg.type === 'WidgetTree') {
      widgetTree.value = msg.root
    }
  }
})

function selectWidget(id: WidgetId) {
  selectedWidget.value = id
  ws.value?.send(JSON.stringify({ type: 'SelectWidget', id }))
}
</script>
```

### 文件结构
```
crates/auto-ui/src/devtools/
├── mod.rs              # 模块导出
├── server.rs           # WebSocket 服务器
├── protocol.rs         # 消息协议定义
├── tree.rs             # Widget 树序列化
└── inspect.rs          # 属性检查/修改

devtools/               # 独立的 Vue 项目
├── src/
│   ├── App.vue         # 主界面
│   ├── components/
│   │   ├── WidgetTree.vue    # Widget 树视图
│   │   ├── PropertyPanel.vue # 属性面板
│   │   ├── Console.vue       # 日志控制台
│   │   └── LayoutView.vue    # 布局可视化
│   └── protocol.ts     # 协议类型定义
└── package.json
```

### 验证方法
```bash
# 1. 启动应用（自动启动 DevTools server）
cargo run --example todo_app --features devtools

# 2. 打开浏览器
# http://localhost:9528

# 3. 测试功能
# - 查看 widget 树
# - 点击选择 widget（应用中高亮）
# - 修改属性（应用实时更新）
```

---

## Phase 3: 热重载 (Hot Reload)

### 架构
```
┌─────────────────────────────────────────────────────────────┐
│                     Source Files (.at)                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 File Watcher (notify crate)                  │
│  - 监听文件变化                                              │
│  - 过滤 .at 文件                                             │
│  - 防抖处理 (debounce)                                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Incremental Compiler (AIE)                      │
│  - 只重新编译修改的文件                                       │
│  - 利用 Interface Hash 熔断机制                              │
│  - 生成更新的 Widget 定义                                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Hot Reloader                                 │
│  - 动态加载新代码 (libloading / hot-lib-reloader)            │
│  - 保留应用状态                                               │
│  - 触发 widget 重建                                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   Running Application                        │
│  - Widget 树更新                                             │
│  - 状态保留                                                   │
│  - 重新渲染                                                   │
└─────────────────────────────────────────────────────────────┘
```

### 实现方案

#### 方案 A: 动态库热加载
```rust
use hot_lib_reloader::HotLibReloader;

pub struct HotReloadManager {
    lib_reloader: HotLibReloader,
}

impl HotReloadManager {
    pub fn new() -> Result<Self> {
        let lib_reloader = HotLibReloader::new(
            "target/debug/libmyapp_widgets.so",
            Duration::from_millis(500),
        )?;

        Ok(Self { lib_reloader })
    }

    pub fn check_reload(&mut self) -> bool {
        self.lib_reloader.update().is_ok()
    }
}
```

**限制**：需要编译为动态库，Windows/macOS 有额外限制

#### 方案 B: 解释执行（推荐初期）
```rust
// Widget 定义部分用解释器执行
pub struct WidgetInterpreter {
    session: CompileSession,
}

impl WidgetInterpreter {
    pub fn reload(&mut self, source: &str) -> Result<WidgetDef> {
        // 重新解析和执行
        let result = self.session.run(source)?;
        // 返回 widget 定义
        result.into_widget_def()
    }
}
```

**优点**：
- 不需要动态库
- 跨平台一致
- 利用现有 AIE 增量编译

**缺点**：
- 性能略低（但 widget 定义通常不是性能瓶颈）

### 文件结构
```
crates/auto-ui/src/hotreload/
├── mod.rs              # 模块导出
├── watcher.rs          # 文件监听
├── reloader.rs         # 热重载逻辑
└── state.rs            # 状态保留
```

### 验证方法
```bash
# 1. 启动应用
cargo run --example todo_app --features hotreload

# 2. 修改源文件
# 编辑 examples/todo_app.at

# 3. 保存后应用自动更新
# - Widget 结构更新
# - 样式更新
# - 状态保留（如输入框内容）
```

---

## Phase 4: 原生 DevTools (可选)

### 目标
使用 auto-ui 组件自举构建 DevTools，实现完全统一的技术栈。

### 前提条件
- auto-ui 已实现足够的基础控件
- 包括：Tree、Table、Panel、Input、Button 等

### 实现方式
```rust
// DevTools 作为 overlay widget
pub struct DevToolsOverlay {
    widget_tree: WidgetTreeView,
    property_panel: PropertyPanel,
    console: Console,
}

impl Widget for DevToolsOverlay {
    fn build(&self) -> WidgetTree {
        Row::new()
            .child(ResizablePanel::new(
                WidgetTreeView::new(),
                300.0,
            ))
            .child(PropertyPanel::new())
            .child(Console::new().minimized())
    }
}
```

**优先级**: P4（长期目标，可延后）

---

## 实施计划

### 前置条件
- 完成 todoMVC 示例（验证 a2rust 基本功能）

### Phase 时间线

| Phase | 内容 | 时间 | 优先级 |
|-------|------|------|--------|
| Phase 1 | 最小调试层 | 3-5 天 | P1 |
| Phase 2 | Web DevTools | 7-10 天 | P2 |
| Phase 3 | 热重载 | 5-7 天 | P2 |
| Phase 4 | 原生 DevTools | 14-21 天 | P4 (可选) |

### 里程碑

```
M1: 最小调试层可用
    - F12 切换调试模式
    - Widget 边框高亮
    - 类型名显示

M2: Web DevTools 可用
    - 浏览器查看 widget 树
    - 选择 widget 高亮
    - 查看属性

M3: 属性编辑可用
    - 在 DevTools 中修改属性
    - 应用实时反映修改

M4: 热重载可用
    - 修改源文件自动更新
    - 状态保留

M5: 原生 DevTools (可选)
    - 所有 DevTools 用 auto-ui 实现
```

---

## 文件清单

### 需要创建的文件

#### Phase 1 (最小调试层)
- `crates/auto-ui/src/debug/mod.rs`
- `crates/auto-ui/src/debug/highlight.rs`
- `crates/auto-ui/src/debug/spacing.rs`

#### Phase 2 (Web DevTools)
- `crates/auto-ui/src/devtools/mod.rs`
- `crates/auto-ui/src/devtools/server.rs`
- `crates/auto-ui/src/devtools/protocol.rs`
- `crates/auto-ui/src/devtools/tree.rs`
- `crates/auto-ui/src/devtools/inspect.rs`
- `devtools/` (独立 Vue 项目)

#### Phase 3 (热重载)
- `crates/auto-ui/src/hotreload/mod.rs`
- `crates/auto-ui/src/hotreload/watcher.rs`
- `crates/auto-ui/src/hotreload/reloader.rs`
- `crates/auto-ui/src/hotreload/state.rs`

### 需要修改的文件
- `crates/auto-ui/src/widget.rs` - 添加 debug/devtools 支持
- `crates/auto-ui/src/lib.rs` - 导出新模块
- `crates/auto-ui/Cargo.toml` - 添加依赖

---

## 各路线总结

| 路线 | DevTools | 热重载 | 实现难度 | 优先级 |
|-----|----------|--------|---------|--------|
| a2vue | 浏览器内置 | Vite HMR | 低 | P2 |
| a2jet | Android Studio | Live Edit (实验) | 中 | P3 |
| a2ark | DevEco Studio | Hot Reload | 中 | P3 |
| a2rust | **需自建** | **需自建** | 高 | P1 |

---

## 状态

- [ ] Phase 1: 最小调试层
- [ ] Phase 2: Web DevTools
- [ ] Phase 3: 热重载
- [ ] Phase 4: 原生 DevTools (可选)

**前置条件**: 完成 todoMVC 示例
