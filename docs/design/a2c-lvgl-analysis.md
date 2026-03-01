# a2c + LVGL 嵌入式 UI 架构分析

## 概述

本文档分析 AutoLang 在嵌入式系统上的 UI 编译路线：**a2c + LVGL**。

LVGL (Light and Versatile Graphics Library) 是嵌入式系统最流行的轻量级图形库：
- 纯 C 实现，无依赖
- 支持 16/32/64 位 MCU
- 最小 64KB Flash + 16KB RAM
- 类似 CSS 的样式系统
- 丰富的控件（按钮、列表、图表等）

---

## 架构对比

### 现有 UI 路线

```
┌─────────────────────────────────────────────────────────────────┐
│                      现有架构                                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   a2vue    a2rust           a2jet          a2ark                │
│     │         │               │              │                  │
│     ▼         ▼               ▼              ▼                  │
│   Vue     auto-ui      Jetpack Comp    ArkTS                   │
│     │         │               │              │                  │
│     ▼         ▼               ▼              ▼                  │
│  Browser   gpui/iced    Android SDK   HarmonyOS SDK             │
│     │         │               │              │                  │
│     ▼         ▼               ▼              ▼                  │
│  有 GC     Rust GC          JVM GC       ArkTS GC               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### a2c + LVGL 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                      a2c + LVGL 架构                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                      a2lvgl                                     │
│                         │                                       │
│                         ▼                                       │
│                    LVGL API (C)                                 │
│                         │                                       │
│                         ▼                                       │
│                   裸机 / RTOS                                   │
│                         │                                       │
│                         ▼                                       │
│                    无 GC，手动管理                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 困难分析

### 🟢 困难度：低（可直接复用）

#### 1. C 代码生成 (a2c 转译器)

```
✅ 已有完善的 a2c 转译器
✅ 支持函数、结构体、枚举等
✅ 已有测试框架 (test/a2c/)
```

**结论**：直接复用，只需扩展 LVGL 特定的代码生成

#### 2. 组件映射

AURA Widget 与 LVGL Object 映射关系清晰：

| AURA | LVGL |
|------|------|
| Container | `lv_obj` (with flex/grid) |
| Button | `lv_btn` |
| Text | `lv_label` |
| Image | `lv_img` |
| TextInput | `lv_textarea` |
| List | `lv_list` |
| Slider | `lv_slider` |
| Switch | `lv_switch` |
| Checkbox | `lv_checkbox` |

**结论**：组件模型相似，映射直接

---

### 🟡 困难度：中（需要适配）

#### 3. 布局系统

**AURA**: FlexBox 风格
```rust
Row::new()
    .gap(10)
    .child(Button::new("OK"))
    .child(Button::new("Cancel"))
```

**LVGL**: Flex 布局（需启用 `LV_USE_FLEX`）
```c
lv_obj_t * row = lv_obj_create(NULL);
lv_obj_set_layout(row, LV_LAYOUT_FLEX);
lv_obj_set_style_flex_flow(row, LV_FLEX_FLOW_ROW, 0);
lv_obj_set_style_pad_gap(row, 10, 0);

lv_obj_t * btn1 = lv_btn_create(row);
lv_obj_t * btn2 = lv_btn_create(row);
```

**适配方案**：
```rust
// ui_gen/lvgl.rs
fn generate_flex_container(container: &Container) -> String {
    format!(
        r#"
lv_obj_t* {name} = lv_obj_create({parent});
lv_obj_set_layout({name}, LV_LAYOUT_FLEX);
lv_obj_set_style_flex_flow({name}, {flow}, 0);
lv_obj_set_style_pad_gap({name}, {gap}, 0);
"#,
        name = container.name,
        parent = container.parent,
        flow = if container.is_row { "LV_FLEX_FLOW_ROW" } else { "LV_FLEX_FLOW_COLUMN" },
        gap = container.gap
    )
}
```

**结论**：LVGL 支持 Flex，需要适配层

#### 4. 样式系统

**AURA/CSS 风格**:
```rust
Style::new()
    .padding(10)
    .background_color(Color::BLUE)
    .border_radius(5)
```

**LVGL 样式**:
```c
static lv_style_t style;
lv_style_init(&style);
lv_style_set_pad_all(&style, 10);
lv_style_set_bg_color(&style, lv_color_hex(0x0000FF));
lv_style_set_radius(&style, 5);

lv_obj_add_style(obj, &style, 0);
```

**适配方案**：
```rust
// 生成样式初始化代码
fn generate_style(style: &Style, name: &str) -> String {
    let mut code = format!("static lv_style_t {};\n", name);
    code += &format!("lv_style_init(&{});\n", name);

    if let Some(pad) = style.padding {
        code += &format!("lv_style_set_pad_all(&{}, {});\n", name, pad);
    }
    // ...
    code
}
```

**结论**：样式模型相似，需要转换

#### 5. 事件处理

**AURA**:
```rust
Button::new("Click")
    .on_click(|| { /* handler */ })
```

**LVGL**:
```c
static void btn_click_cb(lv_event_t * e) {
    // handler
}

lv_obj_add_event_cb(btn, btn_click_cb, LV_EVENT_CLICKED, NULL);
```

**适配方案**：
```rust
// 生成事件回调
fn generate_event_handler(widget: &Widget, event: &Event) -> String {
    format!(
        r#"
static void {widget_name}_{event}_cb(lv_event_t* e) {{
    {handler_code}
}}
lv_obj_add_event_cb({widget_name}, {widget_name}_{event}_cb, LV_EVENT_{EVENT_TYPE}, NULL);
"#,
        widget_name = widget.name,
        event = event.name,
        handler_code = generate_handler_body(&event.handler),
        EVENT_TYPE = event.lvgl_type()
    )
}
```

**结论**：事件模型可适配，需要生成回调函数

---

### 🔴 困难度：高（核心挑战）

#### 6. 响应式状态管理

这是**最大的挑战**。

**现有路线的响应式**:

| 路线 | 响应式机制 | 来源 |
|-----|-----------|------|
| a2vue | `ref()` / `computed()` | Vue 运行时 |
| a2rust | `Signal` / `Memo` | gpui/iced 运行时 |
| a2jet | `mutableStateOf()` | Compose 运行时 |
| a2ark | `@State` / `@Prop` | ArkTS 运行时 |
| **a2c+lvgl** | ❓ | **需要自己实现** |

**问题**：C 语言没有：
- 闭包
- 泛型
- 自动内存管理
- 反射

**解决方案 A：最小响应式运行时**
```c
// 简单的脏标记系统
typedef struct {
    void* value;
    bool dirty;
    void (*notify)(void*);
} ReactiveVar;

typedef struct {
    ReactiveVar* deps[16];
    int dep_count;
    void (*compute)(void*);
} ComputedVar;

// 变化时标记脏
void reactive_set(ReactiveVar* var, void* value) {
    var->value = value;
    var->dirty = true;
    if (var->notify) var->notify(var);
}

// 渲染循环检查脏变量
void ui_update() {
    for (int i = 0; i < var_count; i++) {
        if (vars[i]->dirty) {
            update_widget(vars[i]);
            vars[i]->dirty = false;
        }
    }
}
```

**解决方案 B：编译时追踪**
```rust
// 在编译时分析依赖关系，生成更新函数
fn generate_reactive_update(widget: &Widget) -> String {
    // 分析哪些状态会影响哪些 UI 属性
    // 生成直接的更新代码，无需运行时追踪
}
```

**解决方案 C：轮询模式**
```c
// 每帧检查所有状态（简单但效率低）
void ui_update() {
    if (count != last_count) {
        lv_label_set_text_fmt(label, "Count: %d", count);
        last_count = count;
    }
}
```

**结论**：需要实现轻量级响应式运行时，或使用编译时优化

#### 7. 内存管理

**问题**：嵌入式系统内存受限，无 GC

**现有 AURA 设计**:
```rust
// 大量使用动态分配
let widgets = vec

![widget1, widget2, widget3];
let style = Box::new(Style::new());
```

**嵌入式需要**:
```c
// 静态分配或池分配
static Widget widget_pool[32];
static Style style_pool[64];
```

**解决方案**：
```c
// 编译时确定最大数量，静态分配
#define MAX_WIDGETS 64
#define MAX_STYLES 32

typedef struct {
    lv_obj_t* objects[MAX_WIDGETS];
    int count;
} WidgetPool;

WidgetPool* widget_pool_get() {
    static WidgetPool pool = {0};
    return &pool;
}
```

**结论**：需要限制动态特性，或提供内存池机制

#### 8. DevTools

**问题**：嵌入式系统没有浏览器/IDE

**解决方案**：通过串口连接 PC DevTools

```
┌─────────────────┐      UART/USB      ┌─────────────────┐
│  Embedded Device │ ←──────────────→ │  PC DevTools    │
│  (LVGL App)      │    Serial Port    │  (Vue App)      │
│                  │                    │                 │
│  - Serial RPC    │                    │  - Widget Tree  │
│  - State Dump    │                    │  - Property Ed  │
│  - Event Log     │                    │  - Console      │
└─────────────────┘                    └─────────────────┘
```

**协议**:
```c
// 通过串口发送调试信息
void devtools_send_widget_tree();
void devtools_send_state(const char* name, void* value);
void devtools_log(const char* msg);

// 接收命令
void devtools_on_command(const char* cmd);
```

**结论**：可以实现，但需要串口通信层

---

## 架构适配建议

### 扩展 auto-ui + 添加 lvgl 后端

```
┌─────────────────────────────────────────────────────────────┐
│                    AURA (现有)                               │
│  - Widget 抽象                                              │
│  - Style 抽象                                               │
│  - Event 抽象                                               │
│  - State 抽象 ← 需要为嵌入式简化                             │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
          ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  gpui/iced      │  │    Vue          │  │  LVGL (新增)    │
│  (现有)         │  │    (现有)       │  │                 │
│                 │  │                 │  │  - lvgl.rs      │
│  - 完整响应式   │  │  - 完整响应式    │  │  - 简化响应式   │
│  - 丰富组件     │  │  - 丰富组件      │  │  - 核心组件     │
│  - DevTools     │  │  - DevTools      │  │  - 串口 DevTools│
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

### 需要新增的模块

```
crates/auto-lang/src/ui_gen/
├── vue.rs          # 现有
├── rust.rs         # 现有
└── lvgl.rs         # 新增：LVGL 代码生成

crates/auto-ui/src/backends/
├── gpui.rs         # 现有
├── iced.rs         # 现有
└── lvgl.rs         # 新增：LVGL 后端适配

crates/auto-reactive/  # 新增：响应式运行时
├── mod.rs
├── full.rs         # 完整版
└── embedded.rs     # 嵌入式简化版
```

---

## 总结

| 方面 | 困难度 | 说明 |
|-----|--------|------|
| C 代码生成 | 🟢 低 | a2c 已完善 |
| 组件映射 | 🟢 低 | LVGL 组件模型相似 |
| 布局系统 | 🟡 中 | LVGL 支持 Flex，需适配 |
| 样式系统 | 🟡 中 | 需要转换层 |
| 事件处理 | 🟡 中 | 需生成回调函数 |
| **响应式状态** | 🔴 高 | **需要实现轻量级运行时** |
| **内存管理** | 🔴 高 | **需要静态分配/池化** |
| DevTools | 🟡 中 | 可通过串口实现 |

### 结论

**a2c + LVGL 是可行的**，但需要解决两个核心问题：

1. **响应式状态管理**：需要实现一个极简的响应式运行时（脏标记 + 轮询）
2. **内存管理**：需要限制动态特性，使用静态分配或内存池

### 建议

1. 先完成 a2vue/a2rust 的核心功能
2. 在设计 auto-ui 时考虑嵌入式约束（可选的简化模式）
3. a2c+LVGL 作为 Phase 2 目标，优先级低于 a2jet/a2ark

---

## 实施路线图

### Phase 1: 基础代码生成 (2-3 天)
- [ ] 创建 `ui_gen/lvgl.rs`
- [ ] 实现基础组件映射
- [ ] 实现样式生成

### Phase 2: 布局和事件 (2-3 天)
- [ ] 实现 Flex 布局生成
- [ ] 实现事件回调生成
- [ ] 添加 LVGL 测试用例

### Phase 3: 响应式运行时 (3-5 天)
- [ ] 设计轻量级响应式系统
- [ ] 实现脏标记机制
- [ ] 生成状态更新代码

### Phase 4: DevTools (3-5 天)
- [ ] 实现串口通信协议
- [ ] PC 端 DevTools 适配
- [ ] 状态查看和修改

### Phase 5: 内存优化 (2-3 天)
- [ ] 静态分配方案
- [ ] 内存池实现
- [ ] 内存使用分析工具

---

## 相关资源

- [LVGL 官方文档](https://docs.lvgl.io/)
- [LVGL GitHub](https://github.com/lvgl/lvgl)
- [Plan 100: a2js → a2ts 移植计划](../plans/100-a2js-to-a2ts.md)
- [Plan 101: DevTools 与热重载综合计划](../plans/101-devtools-hotreload.md)
