# Plan 370: 015-notes 桌面模式验证 — 行为约束 + 测试套件

> **目标**: 为 015-notes 的 VM 模式（`auto run -r vm`）建立和 Web 模式对等的行为验证能力。
>
> **核心挑战**: 桌面模式有大量 Web 特性缺失（语义 token、dark mode、AutoDownEditor、store composable），需要区分"该测的"和"known-gap"。

---

## 1. 调研核心发现

### 1.1 桌面模式的三层交互能力

| 能力 | 接口 | 适用模式 | 自动化测试适用性 |
|------|------|----------|-----------------|
| **A. UI MCP Server** | HTTP `localhost:9247`，10 个 `autoui_*` 工具 | VM 模式 | ✅ 最强：snapshot/click/type/screenshot/state/wait/vtree |
| **B. DynamicComponent 进程内驱动** | `on_with_input()` + `read_state()` | VM 模式（无 GUI） | ✅ 最适合自动化：同步、可重复、无窗口依赖 |
| **C. Headless 渲染器** | `HeadlessRenderer::render()` | 静态 Component | ⚠️ 仅渲染测试，不支持有状态 widget |

### 1.2 Web → 桌面的功能映射

| Web 特性 | VM 模式 | Rust 模式 | 测试策略 |
|----------|---------|-----------|----------|
| 基础布局（col/row/button/text/input） | ✅ iced 渲染 | ✅ a2r 生成 | **可测** |
| 笔记切换（点击 → state 变化 → 重渲染） | ✅ handler 执行 | ✅ match 分支 | **核心测试目标** |
| 输入双向绑定（v-model） | ✅ `input_state_map` + `on_with_input` | ✅ input value 绑定 | **可测** |
| Edit/Save/Cancel 编辑循环 | ✅ handler 执行 | ✅ match 分支 | **可测** |
| New/Delete/Pin/Tag 操作 | ✅ handler 执行 | ⚠️ 部分（store 字段丢失） | **VM 可测，Rust 部分** |
| View tabs（All/Pinned/Recent） | ✅ 条件渲染 | ✅ 条件渲染 | **可测** |
| 语义 CSS token（bg-primary 等） | ❌ 静默丢弃 | ❌ 静默丢弃 | **known-gap** |
| dark mode | ❌ 无实现 | ❌ 无实现 | **known-gap** |
| accent 主题色 | ❌ 无实现 | ❌ 无实现 | **known-gap** |
| AutoDownEditor（Tiptap） | ❌ 渲染为 Empty | ❌ 不生成 | **known-gap** |
| store composable | ⚠️ 用 widget model 替代 | ❌ 不生成 | **VM 部分可用** |
| 搜索过滤（query 参数） | ✅ handler 逻辑 | ⚠️ a2r 可能丢失 | **VM 可测** |

### 1.3 关键限制（VM 模式）

- **payload 字符串编码**：事件参数编码为 `{event}\u{1F}{typechar}\u{1F}{value}` 格式
- **输入类型一致性**：f32 vs f64 + nanbox 算术，输入值必须按字段类型 promote
- **init 时序**：VM 模式 Init 是同步 VM 调用（和 Web 的 async 不同）
- **Rust 模式无 MCP**：只有 VM 模式（`run_dynamic_iced`）启动 UI MCP server

---

## 2. 测试架构设计

### 2.1 三层测试策略

```
┌──────────────────────────────────────────────────────┐
│ 第 1 层：Headless 行为测试（进程内，无 GUI）          │
│   DynamicComponent::on_with_input + read_state        │
│   → 快速、同步、可 CI                                 │
│   → 测 handler 逻辑 + state 变化                      │
│   → 用 Rust #[test] 写，直接调 auto-lang API          │
└──────────────────────────────────────────────────────┘
           ↓ 验证逻辑正确性后
┌──────────────────────────────────────────────────────┐
│ 第 2 层：MCP 交互测试（有 GUI，通过 HTTP）             │
│   UI MCP Server localhost:9247                         │
│   → 真实 iced 窗口 + 事件循环                          │
│   → 测渲染 + 点击 + 输入 + 截图                        │
│   → 用 Python/TS/Shell 脚本调 HTTP API                │
└──────────────────────────────────────────────────────┘
           ↓ 验证渲染正确性后
┌──────────────────────────────────────────────────────┐
│ 第 3 层：跨平台契约（acceptance.atd 扩展）             │
│   声明哪些行为是跨平台一致的，哪些是 Web/Desktop 特有  │
│   → 对应 Plan 366 的跨平台测试 DSL 长期方向            │
└──────────────────────────────────────────────────────┘
```

### 2.2 第 1 层：Headless 行为测试（优先实施）

这是 ROI 最高的——不需要 GUI 窗口，直接在 Rust 测试里驱动 DynamicComponent。

**测试原理**：
```rust
#[test]
fn desktop_note_switching() {
    // 1. 从 .at 文件构造 DynamicComponent
    let code = include_str!("../src/front/app.at");
    let mut dc = DynamicComponent::from_code(code);

    // 2. 触发 Init
    dc.fire_init();

    // 3. 验证初始状态
    let notes = dc.read_state_as_vec("notes");
    assert!(notes.len() > 0, "Init should load notes");

    // 4. 切换笔记（触发 SelectNote handler）
    dc.on_with_input("SelectNote", Some("1"));

    // 5. 验证 active_id 变了
    let active_id = dc.read_state("active_id");
    assert_eq!(active_id, Value::Int(1));
}
```

**优势**：
- 纯 Rust `#[test]`，`cargo test` 直接跑，不需要启动 GUI
- 同步执行，无时序问题
- 和 Web 的 Playwright 测试形成互补（一个测逻辑，一个测渲染）

### 2.3 第 2 层：MCP 交互测试

VM 模式启动 iced 窗口后，UI MCP server 自动在 `localhost:9247` 启动。可以通过 HTTP 调用 10 个 `autoui_*` 工具。

**测试原理**：
```python
import requests

# 1. 启动 auto run -r vm（在后台）
# 2. 等 MCP server 就绪（poll localhost:9247）

# 3. 获取 UI 快照
resp = requests.post("http://localhost:9247/mcp", json={
    "jsonrpc": "2.0", "method": "tools/call",
    "params": {"name": "autoui_snapshot", "arguments": {}},
    "id": 1
})
snapshot = resp.json()

# 4. 点击笔记
requests.post("http://localhost:9247/mcp", json={
    "jsonrpc": "2.0", "method": "tools/call",
    "params": {"name": "autoui_action", "arguments": {
        "element_id": "aura_5",  # 从 snapshot 找到目标元素
        "action": "press"
    }},
    "id": 2
})

# 5. 验证 state 变化
resp = requests.post("http://localhost:9247/mcp", json={
    "jsonrpc": "2.0", "method": "tools/call",
    "params": {"name": "autoui_state", "arguments": {
        "fields": ["active_id"]
    }},
    "id": 3
})
assert resp.json()["result"]["active_id"] == "1"
```

---

## 3. 验收契约扩展（acceptance.atd）

### 3.1 跨平台行为（Web + Desktop 都应通过）

这些是核心逻辑行为，不依赖渲染细节：

```markdown
## D1: 笔记切换（跨平台）
- **当** 触发 SelectNote(1)
- **那么** active_id 变为 1
- **且** notes 列表不变
- **平台**: Web ✅ | VM ✅ | Rust ✅

## D2: 创建笔记（跨平台）
- **当** 触发 NewNote
- **那么** notes 列表增加一项
- **且** active_id 指向新笔记
- **平台**: Web ✅ | VM ✅ | Rust ⚠️（store 字段可能丢失）

## D3: 删除笔记（跨平台）
- **当** 触发 DeleteNote(id)
- **那么** notes 列表减少一项
- **平台**: Web ✅ | VM ✅ | Rust ⚠️

## D4: View tabs 切换（跨平台）
- **当** 触发 SelectPinned
- **那么** active_folder 变为 "pinned"
- **平台**: Web ✅ | VM ✅ | Rust ✅

## D5: Edit → Save 循环（跨平台）
- **当** 触发 Edit → edit_title 变更 → Save
- **那么** note 标题更新
- **平台**: Web ✅ | VM ✅ | Rust ✅
```

### 3.2 Desktop known-gap（标记为桌面限制）

```markdown
## D-GAP-1: 语义 CSS token
- **Web**: bg-primary/text-foreground/bg-card 等生效
- **Desktop**: 全部静默丢弃（iced style parser 不解析 semantic token）
- **影响**: 桌面 UI 无颜色主题
- **状态**: known-gap，不计为测试失败

## D-GAP-2: dark mode
- **Web**: :class="{ dark: store.dark_mode }" 生效
- **Desktop**: 无实现
- **状态**: known-gap

## D-GAP-3: AutoDownEditor
- **Web**: Tiptap WYSIWYG 编辑器
- **Desktop VM**: 渲染为 Empty
- **Desktop Rust**: 不生成
- **状态**: known-gap，编辑器测试仅限 Web

## D-GAP-4: accent 主题色
- **Web**: applyAccent 设置 CSS 变量
- **Desktop**: 无实现
- **状态**: known-gap
```

---

## 4. 实施计划

### Phase 1: Headless 行为测试框架（2-3 天）

**目标**: 用 DynamicComponent 建立 VM 模式的进程内测试。

#### 4.1.1 创建测试 harness

新文件：`examples/ui/015-notes/tests/desktop_harness.rs`

```rust
/// 从 .at 源码构造 DynamicComponent 并返回可操作的测试实例。
pub fn load_widget(at_path: &str) -> DynamicComponent {
    let code = std::fs::read_to_string(at_path).unwrap();
    // 构造 DynamicComponent（需要 CompilerSession::ui()）
    DynamicComponent::with_registry_and_imports_from_decls(&code)
}

/// 辅助：触发事件并断言 state
pub fn assert_state_after(dc: &mut DynamicComponent, event: &str, input: Option<&str>,
                          field: &str, expected: &str) {
    dc.on_with_input(event, input);
    let actual = dc.read_state(field);
    assert_eq!(format!("{:?}", actual), expected);
}
```

#### 4.1.2 实现核心测试用例

新文件：`examples/ui/015-notes/tests/desktop_behavior.rs`

```rust
#[test]
fn d1_note_switching() { ... }

#[test]
fn d2_new_note() { ... }

#[test]
fn d3_delete_note() { ... }

#[test]
fn d4_view_tabs() { ... }

#[test]
fn d5_edit_save_cycle() { ... }

#[test]
fn d6_tag_filter() { ... }

#[test]
fn d7_pin_toggle() { ... }
```

#### 4.1.3 验证 store computed

```rust
#[test]
fn d8_pinned_notes_computed() {
    let mut dc = load_widget("src/front/app.at");
    dc.fire_init();
    // 验证 computed 属性
    let pinned = dc.read_state("pinned_notes");
    // ... 断言
}
```

**验证**: `cargo test --features ui-iced -p auto-lang desktop_behavior`

### Phase 2: MCP 交互测试（3-5 天）

**目标**: 通过 UI MCP Server 验证真实 iced 渲染。

#### 4.2.1 测试脚本

新文件：`examples/ui/015-notes/tests/desktop_mcp.py`

```python
"""MCP 交互测试：启动 VM 模式 → 调 autoui_* 工具 → 断言"""

import subprocess, requests, json, time

class DesktopMCP:
    def __init__(self, port=9247):
        self.url = f"http://localhost:{port}/mcp"
        self.id = 0

    def call(self, tool, **args):
        self.id += 1
        resp = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool, "arguments": args},
            "id": self.id
        })
        return resp.json()

    def snapshot(self): return self.call("autoui_snapshot")
    def click(self, eid): return self.call("autoui_action", element_id=eid, action="press")
    def state(self, *fields): return self.call("autoui_state", fields=list(fields))
    def screenshot(self): return self.call("autoui_screenshot")
    def wait(self, field, timeout=5000): return self.call("autoui_wait", field=field, timeout_ms=timeout)

def test_note_switching():
    mcp = DesktopMCP()
    snap = mcp.snapshot()
    # 找到笔记按钮，点击，验证 state
    # ...
```

#### 4.2.2 运行方式

```bash
# 终端 1：启动 VM 模式
cd examples/ui/015-notes
auto run -r vm

# 终端 2：跑 MCP 测试
cd tests
python desktop_mcp.py
```

### Phase 3: 跨平台契约文档（1 天）

#### 4.3.1 扩展 acceptance.atd

在现有的 `tests/acceptance.atd` 里增加桌面特定条目：
- D1-D7 跨平台行为契约
- D-GAP-1 到 D-GAP-4 桌面 known-gap

#### 4.3.2 更新 Plan 366（跨平台测试 DSL）

Plan 366 的 §2.1 提到的 `ui.click(.note_titled("Shopping List"))` 抽象选择器——
在桌面模式下对应 MCP 的 `autoui_action(element_id=...)` 或 `DynamicComponent::on_with_input`。
补充映射关系。

---

## 5. 验收标准

### 可量化

| 指标 | 目标 |
|------|------|
| Headless 行为测试数 | ≥ 7（覆盖 D1-D7） |
| MCP 交互测试数 | ≥ 5（核心路径） |
| known-gap 文档化 | D-GAP-1 到 D-GAP-4 全部标注 |
| `cargo test` 通过 | Headless 全绿 |
| MCP 测试通过 | VM 模式全绿 |

### 质量标准

- [ ] Headless 测试在 `cargo test` 里 10 秒内跑完
- [ ] MCP 测试能在 `auto run -r vm` 后 30 秒内跑完
- [ ] 每个 known-gap 有明确的技术原因和未来修复方向
- [ ] 跨平台契约（D1-D7）同时被 Web Playwright 和 Desktop Headless 验证

---

## 6. 与其他 Plan 的关系

- **Plan 366（跨平台测试 DSL）**: 本计划的 D1-D7 契约是 Plan 366 "目标无关测试意图" 的桌面实例
- **Plan 367（代码质量）**: 桌面测试可能暴露新的 a2r/VM bug（如 store 字段丢失）
- **Plan 363（Skill）**: known-gap 应录入 Skill 的 generator-contracts（D-GAP-1 到 D-GAP-4）
- **Plan 361（校验）**: 桌面 known-gap 可考虑加校验规则（如"用了语义 token 时提示桌面不生效"）

---

## 7. 风险与注意事项

1. **DynamicComponent 构造可能复杂**：它需要 CompilerSession、VmBridge、registry 等。如果 API 不友好，可能需要加一个测试专用的 `DynamicComponent::from_at_file(path)` 构造方法。

2. **store composable 在 VM 模式的行为**：调研显示 VM 模式用 widget model 而非 store composable。如果 015-notes 的 store 逻辑（NotesStore）在 VM 模式下不被执行，D2/D3 测试可能需要针对 App widget 的 model 而非 store。

3. **Rust 模式（a2r）的 MCP 缺失**：`run_app_devtools`（静态组件）不启动 UI MCP server。如果需要 MCP 测 Rust 模式，需要给它也加 MCP 支持——这是额外工作。

4. **CI 环境**：Headless 测试不需要显示服务器（纯 Rust 进程），适合 CI。MCP 测试需要 iced 窗口（需要 display server 或 virtual framebuffer）。
