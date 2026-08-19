# Plan 413: 跨平台代码编辑器 widget(模拟 cosmic-edit,iced 后端)

> **状态**: 🚧 实施中 —— **Phase 0 ✅ 已通过闸门**(2026-08-19,Windows 11 双后端验证,见 `scratch/code-editor-spike/README.md` 验证矩阵:fill_raw wgpu+tiny-skia 双通、syntect 单实例 5.3.0、fork API 差异表产出、iced_test 无头管线 5/5)。当前推进:Phase 1 widget 本体。
> **仓库**: auto-lang(`crates/auto-lang/src/ui/iced/`、`src/ui/view.rs`、`src/ui/aura_view_builder.rs`、`src/ui_gen/rust.rs` 等,详见 §4 触点清单)
> **背景**: cosmic-edit(System76 COSMIC 桌面的文本编辑器)有一个成熟的自研编辑器 widget(`text_box.rs`,cosmic-text `ViEditor` 引擎),但整个 cosmic 应用栈不落地 Windows。AutoUI 需要一个同能力(语法高亮、行号、软换行、搜索、vi/undo、IME)的**跨平台(Windows/Linux)代码编辑器组件**,以 `code_editor` 内建 widget 的形式接入 AURA DSL → `View` → iced 渲染器 → a2r 代码生成全链路。
> **结论先行**: 上游 iced 0.14(auto-lang 当前锁定版)**已具备** cosmic-edit TextBox 依赖的全部底层原语(`fill_raw`/`Raw`、全局 `font_system()`、`Event::InputMethod` IME、highlighter),cosmic-text 0.15(iced 0.14 的配套版)同时提供 `ViEditor` + `SyntaxEditor`。**方案 = 按 cosmic-edit 架构在 auto-lang 内 MIT 重新实现一个 `code_editor` 自定义 iced Widget,接入 AutoUI 全链路**。注意许可证约束(§5.1):cosmic-edit 是 GPL-3.0,不能直接复制代码。

---

## 0. 背景与问题

### 0.1 cosmic-edit 为什么不支持 Windows

- cosmic-edit v1.6.0 依赖 libcosmic 1.0(git rev `c1897c01`),后者自带 pop-os 的 iced 0.14 fork(submodule `./iced` @ `7918b282`)。
- 历史上 libcosmic 走 `iced_sctk`(smithay-client-toolkit/Wayland)后端,Windows 编译不过(libcosmic issue #505);epoch 时代已合并到 fork 版 `iced_winit` 并把 Wayland 依赖 cfg-gate 到 unix,但 System76 不 CI Windows,默认 feature(`wayland`/`dbus-config`/`gvfs`/cosmic-files/cosmic-config D-Bus)仍带大量 freedesktop 耦合(ashpd、zbus、xdg、cosmic-protocols)。
- **结论:阻塞在 libcosmic 应用壳,不在编辑器 widget 本身。** `text_box.rs` 几乎是纯 iced 代码(对 cosmic 的依赖仅 `Theme` 类型与 `Renderer` 别名)。

### 0.2 AutoUI 现状缺口

- DSL 已有 `input`(→ iced `text_input`)和 `textarea`(→ iced `text_editor`,带 `&'static Content` 全局存储 hack),**没有**可编辑的语法高亮/行号/搜索编辑器。
- 只读方向已有 `codeblock` + `font-mono` 关键字高亮(`renderer.rs:241-287`,iced Rich spans),但不可编辑。
- `autodown_editor`(markdown)在 VM 模式降级为 textarea;vue 模式才有真编辑器。

### 0.3 目标

1. 新内建 widget `code_editor`:AURA DSL 一等公民,VM 模式(解释执行)与 rust 模式(a2r 代码生成)都可用。
2. 能力对齐 cosmic-edit 的 widget 层:语法高亮(syntect)、行号槽、当前行高亮、软换行、正则搜索/替换高亮、undo/redo + vi 模式(cosmic-text `vi` feature)、IME(中文输入)、大文件按需 shape。
3. Windows(winit + wgpu/D3D12,备选 tiny-skia)与 Linux(X11/Wayland)行为一致。
4. 不包含 cosmic-edit 的应用层功能(文件树、git diff、多 tab、菜单栏、项目搜索)—— 用户用 AutoUI 现有 widget 组合;widget 层暴露 cursor 位置等回调供 app 层复刻状态栏。

---

## 1. 调研结论(已验证的事实)

### 1.1 cosmic-edit 架构(D:\github\cosmic-edit @ 0f725de,Epoch 1.6.0)

核心 = `src/text_box.rs`(1,498 行)自定义 iced `Widget`,不包装 iced `text_editor`,直接持有 `Mutex<ViEditor<'static,'static>>`(cosmic-text 0.19,`syntect`+`vi` features,由 tab 层拥有而非 widget):

- **update**(962-1421 行):原始键盘事件 + word-motion 修饰、三击选中、滚动条拖拽(垂直 y→行,水平 x→最长行宽)、拖选 + 自动滚动回调、滚轮、IME(`InputMethodEvent::Opened/Closed/Preedit/Commit`)、`shell.request_redraw/request_input_method`。
- **draw**(384-960 行)三层:
  1. cosmic-text `Buffer::set_metrics_and_size` + `shape_as_needed`(全局 `font_system()` 加锁),scale-factor 整数像素搜索;
  2. **行号槽 = CPU 光栅图像**:`LINE_NUMBER_CACHE`(font-size 1.0 布局缓存)→ `SwashCache::with_pixels` 逐像素 alpha 混合(`draw_rect` 手写 blitter)→ `image::Handle::from_rgba` + `FilterMethod::Nearest`,仅在 `editor.redraw()` 为真时重画;
  3. 正文与装饰走 iced 渲染器:`CustomRenderer`(实现 `cosmic_text::Renderer`,`rectangle()` 转发 `fill_quad`)画选区/光标/当前行高亮,正文 `renderer.fill_raw(Raw { buffer, position, color, clip_bounds })` 直接绘制 cosmic-text Buffer;滚动条为普通 `fill_quad`。
- 高亮:cosmic-text 0.19 内建 `SyntaxEditor`(syntect + two-face + cosmic-syntax-theme 主题),在 tab.rs 构造;**不用 tree-sitter**。
- 搜索:tab.rs:307-467,regex + 环绕 + `Selection` 高亮(改编自 ViEditor search)。
- 可移植核心 ≈ text_box.rs + line_number.rs(53 行)+ tab.rs 编辑器半部 + search.rs;其余(menu/key_bind/config 持久化/icon)是 COSMIC 壳。

### 1.2 上游 iced 0.14.0(auto-lang 锁定版)已具备所需原语 ✅(用 D:\github\iced 的 `0.14.0` tag 逐项核实)

| 原语 | 位置(iced 0.14.0) | 用途 |
|---|---|---|
| 全局 `font_system()` | `graphics/src/text.rs:116` | 与 iced 文本管线共享 FontSystem |
| `Raw { buffer: Weak<Buffer>, … }` | `graphics/src/text.rs:180` | 直接绘制 cosmic-text Buffer |
| `text::Renderer::fill_raw(&mut self, Raw)` | `graphics/src/text.rs:359` | 正文渲染入口 |
| `Event::InputMethod(input_method::Event)` | `core/src/event.rs:29` | IME(Windows TSF/winit) |
| `Shell::input_method` / `request_redraw` | `core/src/shell.rs` | IME 请求 + 重绘 |
| `highlighter` feature(`iced_highlighter`,syntect) | 根 `Cargo.toml:69` | 备用路线的高亮 |
| cosmic-text 依赖 | `Cargo.toml:193` = **"0.15"** | 版本锚点 |

- cosmic-text 0.15(docs.rs 核实)**导出 `ViEditor` 与 `SyntaxEditor`**(features `vi`、`syntect`),与 cosmic-edit 所用 0.19 同源,存在 API 漂移风险(§5.2,Phase 0 spike 消除)。
- Windows:iced 0.14 默认 wgpu(D3D12/Vulkan)+ winit Win32;tiny-skia 软件后备。cosmic-text 用 fontdb 枚举 Windows 字体(不依赖 DirectWrite),Alacritty Windows 版即用它,**含 CJK 字体发现**,中文显示无障碍。
- 上游 iced 0.14 内建 `text_editor` widget 自带:软换行、IME、剪贴板、可覆盖键绑定、通用 `Highlighter` trait + syntect 实现;**不自带**:行号、搜索、undo/redo(`Action` 无 undo)、vi。

### 1.3 AutoUI 架构(auto-lang)

- 三种 render 模式(`pac.at` 的 `render:`):`vue`(Vue SFC + shadcn)/ `vm`(AutoVM 解释,`AuraViewBuilder` → `View<DynamicMessage>` → `IntoIcedElement` 每帧解释)/ `rust`(`RustGenerator` 生成 `Component` impl,`View::xxx()` builder 链,**生成代码不含 iced:: 调用**)。
- iced 只存在于一处:`crates/auto-lang/src/ui/iced/renderer.rs`(10,543 行,iced 0.14 + `advanced` feature 已启用)。
- widget 映射没有单一注册表,iced 侧是**两层硬编码 match**:`aura_view_builder.rs:859`(tracked)/`:1572`(untracked)做 DSL tag → `View` 变体;`renderer.rs:1225 into_iced` 做 `View` 变体 → iced widget。vue 侧才有 `ui_gen/widget/registry.rs` 的 `WidgetSpec`。
- 状态管理先例:`TEXTAREA_CONTENTS: Mutex<HashMap<String, &'static mut text_editor::Content>>`(renderer.rs:55-114,`Box::leak` + "文本变化才重建 + 光标停文档尾" 策略)—— 外部有状态 widget 的集成范式,`code_editor` 直接复用此思路。
- 单行输入的 payload 传递先例:`INPUT_TEXT` thread-local + `last_input_text()`(renderer.rs:118)—— 泛型 `M` 无法构造带参 Msg 的通用解法,`code_editor` 照搬(per-key 版)。

---

## 2. 方案选型

| | A. 扩展上游 iced `text_editor` | **B1. cosmic-edit 架构复刻(推荐)** | B2. 包装 iced-code-editor crate |
|---|---|---|---|
| 思路 | 在 `View::Textarea` 上加 `lang` 属性走 `iced_highlighter` | 自研 `code_editor` 自定义 Widget:cosmic-text `ViEditor` 引擎 + `fill_raw` 渲染 + CPU 行号槽,MIT 重写 | 直接依赖 [iced-code-editor](https://github.com/LuDog71FR/iced-code-editor)(MIT,iced 0.14,canvas 自绘,含折叠/多光标/LSP) |
| 行号槽 | ❌ 需旁列同步滚动,hack | ✅ 内建 | ✅ 内建 |
| 高亮 | ✅ syntect | ✅ syntect(cosmic-text `SyntaxEditor`) | ✅ syntect |
| undo/vi | ❌(`Action` 无 undo) | ✅(cosmic-text `vi`) | ✅(自实现 history) |
| 搜索高亮 | ❌ | ✅(Selection 高亮,tab.rs 移植) | ✅ |
| 大文件 | 中 | ✅ `shape_as_needed` | 中 |
| 与 cosmic-edit 行为一致性 | 低 | **最高** | 低 |
| 工作量 | 小 | 中(~1,200-1,600 行新 widget 代码) | 小-中(适配其 API 到 View 抽象) |
| 风险 | 能力天花板低,后续返工 | cosmic-text 0.15 vs 0.19 API 漂移(spike 消除) | 第三方维护节奏、canvas 路线与本项目文本管线(font_system)分离、demo 化 API |

**决策:B1**。理由:与用户诉求"模拟 cosmic-edit"一致;能力完整;所有底层依赖(iced 0.14 + cosmic-text 0.15 + syntect + two-face)都是 MIT/MPL,自研层可保持 MIT;`fill_raw` 路线与 iced 文本管线共享 FontSystem(字体一致、省一份文本栈)。B2 作为 Phase 0 的**对照组参考**(其 MIT 源码可合法借阅 undo/搜索/IME 的 iced 0.14 写法),不作依赖。A 路线保留为**降级方案**:若 Phase 0 发现 0.15 的 `ViEditor` 缺关键 API 且不可短平快补,则退回 A(vue 模式本就用 CodeMirror,见 §4 Phase 4)。

### 2.1 否决项:Windows 版 libcosmic(决策记录,2026-08)

cosmic-edit 不支持 Windows 的病灶在 **libcosmic 应用壳**,而非编辑器 widget(§1.1)。本计划不移植 libcosmic,只取其编辑器 widget 层的设计,应用壳由 AutoUI 自任:

| libcosmic 职责 | 本计划替代 |
|---|---|
| 应用框架(Application/Core/多窗口/subscription) | AutoUI 现有 iced 0.14 应用壳(renderer.rs `run_app`) |
| 主题(cosmic-theme/palette/spacing) | `CodeEditorTheme`(AutoUI 语义色合成,§3.1) |
| 右键菜单(surface popup) | `on_context_menu` 回调消息 + 现有 overlay 组合 |
| 菜单栏/文件对话框/配置持久化 | 不在组件范围(app 层用现有 widget 组合,对话框后续可评估 rfd) |

否决理由:① 移植 libcosmic 需替换 cosmic-files(freedesktop/gvfs)、cosmic-config D-Bus 后端、ashpd 门户、surface 菜单弹窗等,等于重写其半条命;② libcosmic 绑死 pop-os iced fork,与 auto-lang 锁定的上游 iced 0.14 在同一进程互斥(两套 iced 无法共存);③ 若动机是原样运行 cosmic-edit 应用,则触发 GPL-3.0 传染(§5.1);④ 与 AutoUI 的价值(自有 DSL→View→iced 链路)相反。若未来确需运行 cosmic-edit 整应用或批量复用 cosmic 部件库,应立项独立项目、独立依赖链,不与本计划混合。

---

## 3. 总体架构设计

### 3.1 新 widget 内部结构(三层分层,为 RenderQueue 分离架构预留,详见 §7)

```
ui/code_editor/                    # ① core 层:渲染后端无关,禁止 import iced
├── core/
│   ├── mod.rs      # CodeEditorCore:状态机 = ViEditor + 光标/滚动/拖拽/三击/IME/vi 状态
│   │               #   + core 自有的输入事件类型 EditorInput(key/修饰键/鼠标/滚轮/IME)——
│   │               #   iced 适配层做 iced::Event → EditorInput 映射;分离架构下由宿主事件
│   │               #   通道映射到同一类型,core 不感知事件来源
│   │               #   + 全局存储 CODE_EDITORS: Mutex<HashMap<String, &'static CodeEditorCore>>
│   │               #   (keyed,复用 TEXTAREA_CONTENTS 范式,key = "__code_editor_{widget_event}")
│   ├── render.rs   # 纯函数 core::render(state, viewport, theme) -> EditorDrawList
│   └── highlight.rs# syntect/two-face 语法系统单例(惰性 SYNTAX_SYSTEM),cosmic-text SyntaxEditor 封装,
│                   #   主题从 AutoUI 语义色(DARK_MODE/accent)合成,不引 cosmic-syntax-theme
├── draw.rs         # ② draw list 层:EditorDrawList 数据类型(按行稳定 id 键控的 TextRun + Quad +
│                   #   行号文本 run + Clip/offset),无 iced 依赖、可序列化 —— 未来 RenderCommand
│                   #   lowering 的直接输入
├── theme.rs        # CodeEditorTheme:替换 cosmic::Theme —— 颜色/间距结构体,由语义色 + style class
│                   #   解析(bg-*/text-*/border-*),Dark/Light 双套
└── iced/           # ③ iced 适配层:唯一的 iced 依赖点
    ├── widget.rs   # CodeEditor<'a, M>:impl iced Widget —— update → core,draw → EditorDrawList
    │               #   → fill_raw / fill_quad / image(text_box.rs 架构复刻)
    └── gutter.rs   # 行号槽 CPU 光栅(iced 侧缓存优化:font-size 1.0 布局缓存 + SwashCache + 最近邻
                    #   image;line_number.rs 架构复刻;lowering 路线用 draw.rs 的行号文本 run,不走 image)
```

渲染分工(与 cosmic-edit 相同):正文 `fill_raw`,选区/光标/当前行高亮 `fill_quad`(自实现 `CosmicRenderer` → `fill_quad` 转发),行号槽 CPU 光栅 image,滚动条 `fill_quad`。

### 3.2 `View` 变体与 DSL

`view.rs` 新增(仿 `Textarea` @ view.rs:275):

```rust
CodeEditor {
    key: String,            // 稳定身份,状态存储键
    value: String,          // 外部值;仅当与内部文本不同才回写(单向数据流 + 内部编辑不回灌)
    lang: String,           // "rust" | "python" | "markdown" | "auto"(.at 高亮,复用现有 AUTO_KEYWORDS) | "none"
    line_numbers: bool,     // default true
    wrap: bool,             // default false(代码编辑器默认不软换行,Wrap::None)
    vi: bool,               // default false(vi passthrough)
    highlight_current_line: bool, // default true
    tab_width: usize,       // default 4
    font_size: f32,
    on_change: Option<...>, on_cursor: Option<...>, on_context_menu: Option<...>,
    style: Style,
}
+ View::code_editor(key) 构造器、ViewCodeEditorBuilder、map_msg arm(1207)
```

DSL 用法(对齐现有 `input`/`textarea` 的属性命名):

```auto
code_editor (lang: "rust", line_numbers: true, wrap: false, style: "h-full") {
    content: .source
    oninput: .SourceChanged
    oncursor: .CursorMoved
}
...
on {
    .SourceChanged -> { .source = code_editor_text("editor1") }   // a2r 生成
}
```

payload 读取(per-key thread-local,照搬 `last_input_text()` 范式):`auto_lang::ui::code_editor_text(key)`、`code_editor_cursor(key) -> (line, col, selection_len)`。

### 3.3 rust 模式生成代码形态(不含 iced:: 调用,与其他 widget 一致)

```rust
View::code_editor("editor1")
    .value(self.source.clone())
    .lang("rust")
    .line_numbers(true)
    .on_change(|_| Msg::SourceChanged)
    .build()
```

### 3.4 其他后端降级策略(先例:`autodown_editor` → textarea)

- **gpui**:`View::CodeEditor` → 渲染为等宽 `Textarea`(gpui 后端后续可接 Zed 的编辑器内核,不在本计划)。
- **vue**:`WidgetSpec` 新条目映射到 CodeMirror 6(`npm_package: "vue-codemirror" + "@codemirror/lang-*"`),与 iced 版对齐基础能力(高亮/行号/搜索);Phase 4 只做 spec + 组件壳,vue 端深化另立计划。
- **headless/snapshot**:按 Textarea 快照 + `lang` 元数据。
- **MCP 自动化**(mcp_server.rs):新增 set-text/type-into 支持(键入按行分派 `Action`),使 GUI 测试可自动化编辑器。

---

## 4. 实施阶段与触点清单

### Phase 0 — 技术验证 spike(先行,独立 cargo 项目 `scratch/code-editor-spike`)

依赖:`iced 0.14`(features `advanced`,`canvas` 已有)+ `cosmic-text 0.15`(features `vi`,`syntect`)+ `two-face`。

验证清单(Windows 优先,Linux 复验):
1. `fill_raw` 在 wgpu 与 tiny-skia 两个 renderer 下都正常出字;
2. `font_system()` 与 iced 文本 widget 共享无死锁(renderer 线程模型下加锁策略);
3. cosmic-text 版本核对(§5.2 参考版本决策后残留的两项):① 行为规格基线 epoch-1.0.2 所用 pop-os git 版 cosmic-text 0.15.0(`7051682e`)与 crates.io 0.15.0(iced 0.14.0 所 pin)的 API 一致性;② fork vs 上游 iced 0.14.0 tag 的专属 API 差异清单(`modified_key`、`Style::scale_factor`、surface-message 等),逐项确认降级写法;
4. IME 链路:`Event::InputMethod` Preedit/Commit + `shell` 请求(中文输入);
5. 行号槽 CPU 光栅 + `FilterMethod::Nearest` 在高 DPI 下的清晰度;
6. two-face 语法集与 cosmic-text 0.15 的 syntect 5.x 版本统一。
**退出标准**:200 行 Rust 文件可编辑、中文 IME 可输入、rust 语法高亮、行号显示,Windows 上跑通。spike 代码 MIT 重写,可参照 iced-code-editor(MIT)的等价实现,不复制 cosmic-edit 代码。

### Phase 1 — widget 本体(§3.1 三层结构,~1,200-1,600 行)

- **分层纪律(硬约束)**:`ui/code_editor/core/` 与 `draw.rs` 禁止 import iced;`ui/iced/code_editor/` 是唯一 iced 依赖点。这是 §7 分离架构兼容的前提,Phase 1 就按此组织,避免日后返工。
- 以 cosmic-edit **epoch-1.0.2 tag**(cosmic-text 0.15 纪元,§5.2)的 `text_box.rs`/`line_number.rs` 为**行为规格**(TESTING.md 清单为验收基线),对照实现;研读 1.0.2→HEAD 的 287 行 diff 作为改进 changelog 吸收(重绘效率优化);fork 专属 API 降级:`modified_key` → 自维护 `Modifiers` 状态;renderer `Style::scale_factor` 若 0.14 缺失 → 简化整数像素搜索。
- 对 cosmic 依赖的替换:`cosmic::Theme` → `CodeEditorTheme`(§3.1);滚动条配色取语义色(当前用 `resolve_semantic_rgb` 同源)。
- 超出 cosmic-edit 的必要适配:程序化 `set_text/get_text`(外部 value diff 回写)、cursor 回调、context menu 回调(替代 cosmic surface popup,由 app 层用现有 overlay/popover 组合)。
- Cargo:crates/auto-lang 新 feature `code-editor`(`dep:cosmic-text`,`dep:syntect`,`dep:two-face` + iced 已有 features),挂在 `ui-iced` 下,默认开启。

### Phase 2 — VM 模式接入

| 触点 | 改动 |
|---|---|
| `src/ui/view.rs` | `CodeEditor` 变体 + builder + `map_msg` arm(~1207) |
| `src/ui/aura_view_builder.rs` | tag arms `code_editor`/`codeEditor`(859 tracked、1572 untracked)+ `convert_code_editor`(仿 `convert_textarea` @3278;子标签属性 `content`/`oninput`/`oncursor`/`oncontextmenu`) |
| `src/ui/iced/renderer.rs` | `into_iced` arm(仿 Textarea @1593);`CODE_EDITORS` 全局存储;debug/dynamic builder(~8549)、`patch_input_values`(~8840)、`view_style_of`(~8448)、`view_kind`(~8469)、style probe(~9196) |
| `View` 的全部消费者 | `vnode.rs`(~202)、`vnode_converter.rs`(~255)、`snapshot_builder.rs`(~119)、`node_converter.rs`(~1103)、`mcp_server.rs`(1052, 2034-2058, 2133, 2495, 2604)、`vtree_atom.rs`(~145)、`ui/gpui/{renderer.rs:197, auto_render.rs:381/895, vnode_entity.rs:271}`、headless、DevTools(`render_elements_tab` 6396、`vnode_summary` 6429、KNOWN elements 6546)—— 编译器会逐一枚举,均给 Textarea 级降级实现 |

### Phase 3 — rust 模式代码生成

| 触点 | 改动 |
|---|---|
| `src/ui_gen/rust.rs` | `tag == "code_editor"` 特殊块(仿 textarea @2141)生成 §3.3 形态;`KNOWN_TAGS`(3246)+ `tag_to_view_fn`(3294);登记 `input_fields`(2125)使 handler 侧用 `code_editor_text(key)` 读值 |
| `auto-man/src/rust_ui.rs` | 生成 Cargo.toml 模板透传 `code-editor` feature(1681 一带) |

### Phase 4 — vue registry + gallery + 示例

- `ui_gen/widget/registry.rs`:`register_form_widgets` 增 `CodeEditor` spec(仿 Textarea @806-833,vue → CodeMirror 6 组件,ark/jet 先占位);`ui_gen/vue.rs` tag map(5131)/event map(10783)/`force_native_elements`(3514)。
- `examples/widgets-gallery/src/front/pages/code-editor.at` 新页 + `app.at` 路由(gallery 跑 VM 模式,即 Phase 2 的试金石)。
- 新示例 `examples/ui/041-code-editor/`:一个迷你"Auto Playground"——用 code_editor 编辑 `.at` 代码(lang: "auto" 复用现有 widget-tag 关键字高亮),运行按钮触发 VM 求值;顺带成为编辑器+VM 集成的回归用例。

### Phase 5 — 测试与验收

- 单元:syntax 系统(语言识别/主题合成)、状态存储 keying、外部 value diff 回写不跳光标、codegen 快照(rust.rs 生成物)、`map_msg`。
- 集成:MCP set-text/type-into 脚本化驱动;VM/gallery 回归(widgets-gallery 全量跑通)。
- 手动清单:移植 cosmic-edit `TESTING.md` 行为清单(三击选中、Ctrl+左右 word motion、滚动条拖拽、IME、搜索高亮、vi 模式开关、undo/redo),Windows 11 + Linux(X11 与 Wayland 各一轮)。
- 性能:≥1MB 文件打开与滚动、shape_as_needed 生效、行号槽仅在 `redraw()` 时重画。

---

## 5. 风险与对策

### 5.1 许可证(硬约束)

- cosmic-edit = **GPL-3.0**,auto-lang = **MIT**。**禁止复制/机械改写其源码进 auto-lang**;实现按"行为规格 + 架构参考"MIT 重写。可直接借阅的 MIT 参考:上游 iced 的 `widget/text_editor.rs` 与 examples、iced-code-editor(MIT);cosmic-text/MPL-2.0、syntect/MIT、two-face 作为依赖使用无碍。新文件头注明 "architecture inspired by cosmic-edit (GPL-3.0, System76); original implementation"。

### 5.2 cosmic-text 版本漂移(0.15 vs 0.19)与 iced 升级策略

**漂移的含义**:cosmic-edit(行为规格)的代码写于 cosmic-text **0.19**,而我们的移植目标是 iced 0.14 所锁定的 cosmic-text **0.15**。0.x 语义下不同 minor 版本互不兼容 —— cargo 会编入两份拷贝,而编辑器要把 `Buffer` 传进 iced 的 `fill_raw(Raw { Weak<Buffer> })` 并共用 iced 的全局 `font_system()`,这些类型的边界必须同版本。因此 **app 不能独立选择 cosmic-text 版本,它由 iced 的 pin 决定**。0.15→0.19(pop-os 生态 2026 年密集发版)之间存在 API 增删改,移植不是逐行映射,Phase 0 spike 产出差异表逐项找平。已确认 0.15 存在 `ViEditor`/`SyntaxEditor`(docs.rs);未证实:ViEditor search API(若缺,按 tab.rs 的正则搜索逻辑自实现,本就要移植)、`SyntaxEditor` 构造签名、`BufferLine::layout` 缓存键。不可找平项 → 降级到方案 A(§2)。

**版本血统表**(2026-08 核实):

| 血统 | iced | cosmic-text | 状态 |
|---|---|---|---|
| **AutoUI 现状** | 0.14.0(2025-12-07 发布,最新稳定版,无补丁版) | **0.15** | 本计划目标基线 |
| **cosmic-edit epoch-1.0.2**(2025-12-31 tag) | fork 自报 **0.14.0-dev**(libcosmic@`3b8ad459`,基于 0.14 发布前上游 master + pop 补丁) | **0.15.0**(pop-os git @`7051682e`) | **✅ 本计划的行为规格基线** —— 与 AutoUI 的 cosmic-text 0.15 同 API 纪元,漂移基本归零 |
| cosmic-edit epoch-1.1.0→1.6.0(HEAD) | fork 0.14.0(libcosmic@`c1897c01`,HEAD 锁定) | 0.19.0(crates.io) | epoch 后续版本;1.0.2→HEAD 的 text_box.rs 仅差 287 行(~19%,主要是 1.6.0 重绘效率优化) |
| 上游 master | 0.15.0-dev(活跃,最近提交 2026-08-16,集中于 text editor/combo box/input widget 改进) | 0.16 | 未发布,移动目标 |
| cosmic-text crates.io 最新 | — | 0.19.0(2026-04-22) | — |

**参考版本决策(2026-08)**:行为规格从 cosmic-edit HEAD 改为 **epoch-1.0.2 tag**(`git worktree add ../cosmic-edit-1.0.2 epoch-1.0.2` 即可,与 HEAD 并存)。理由:epoch 架构在 1.0.2 已完整(text_box.rs 1,399 行,行号/上下文菜单/line_number.rs 俱全),且其 cosmic-text 0.15.0 与我们 iced 0.14.0 锁定的 0.15 同纪元 —— §5.2 开头所述"漂移"风险由此基本消解。剩余两件小事由 spike 收尾:① pop-os git 版 0.15.0(`7051682e`,发布后小修)与 crates.io 0.15.0 的 API 一致性核对;② fork vs 上游 0.14.0 tag 的专属 API 差异清单(预期即 Phase 1 已列的 `modified_key`/`Style::scale_factor`/surface-message 三件套)。另将 1.0.2→HEAD 的 diff 作为"改进 changelog"研读吸收(重绘效率优化值得复刻)。

**iced 升级策略(2026-08 决策):留在 0.14,不追 master。** 理由:① Plan 413 所需原语已在 0.14.0 全部核实存在,无功能必要;② master 是移动目标(wgpu 28、renderer/widget API 持续变更),升级 = 移植 10.5k 行 renderer.rs + 全量 gallery/examples 回归 + 同步 rust 模式生成工程的 iced pin,是独立的中型工程;③ 生成工程(rust 模式)依赖 crates.io 稳定版,押 git pin 的未发布分支对用户工程脆弱;④ iced 发布节奏慢(0.12→0.13 七个月,0.13→0.14 十五个月),等 0.15 稳定版再升,成本/风险都更优。**升级触发条件**(满足其一时立项专门的升级计划):a) iced 0.15 稳定版发布(重点评估其 cosmic-text ≥0.16 与 text editor/IME 改进 —— master 近期工作恰好集中在这些);b) Phase 0/1 发现 0.15 的 cosmic-text 缺关键 API 且 widget 层无法绕过;c) RenderQueue Stage 1 启动前选择最新稳定基线。code_editor 的三层设计(§3.1)将编辑器侧升级成本约束在 ~500 行 iced 适配层内。附带改进:生成模板的 `iced = "0.14.0"` 放宽为 `"0.14"`,让未来补丁版自动流入。

### 5.3 渲染器差异

- `fill_raw` 需 wgpu 与 tiny-skia 双通道验证(spike 第 1 项);若 tiny-skia 缺陷则文档标注 code_editor 需 wgpu feature(Windows 默认即 wgpu)。
- 高 DPI(scale factor)下行号槽与正文的整数像素对齐:保留 cosmic-edit 的 scale-factor 搜索思路,退化实现也要保证 150% 缩放不糊。

### 5.4 状态存储与重建

- `Box::leak` keyed 存储随窗口生命周期增长:沿用现有 TEXTAREA_CONTENTS 模式(项目已接受),另加 `code_editor_dispose(key)` API 供路由切换时显式释放;外部 value 回写只在 `value != internal_text` 时执行,避免每帧重建导致的光标跳动(textarea 已踩过的坑,直接规避)。

### 5.5 范围控制

- 应用层能力(文件/项目/git/多 tab)明确不做;context menu 只出回调消息,弹层用现有 overlay 组合。vue 模式 CodeMirror 只保证基础对齐,深化另立计划。

---

## 6. 验收标准

1. `examples/widgets-gallery` 新增 code-editor 页,VM 模式下 Windows 与 Linux 展示一致:高亮、行号、当前行高亮、软换行开关、搜索框正则高亮、vi 开关、undo/redo。
2. `examples/ui/041-code-editor` 三模式中 vm/rust 两模式可用,`auto build && auto run` 产物在 Windows 打开即为可编辑。
3. 中文 IME(微软拼音)输入、光标跟随;Ctrl+C/V/X 跨平台剪贴板。
4. 1MB 文件滚动流畅(目标:无肉眼卡顿,行号槽无整帧重画)。
5. `cargo test -p auto-lang` 全绿;codegen 快照更新;MCP 自动化用例通过。
6. 全部新代码为 MIT 原创实现(§5.1 合规)。

---

## 7. 与分离渲染架构(RenderQueue / design 20 / Plan 386)的兼容性

**结论:不改变本计划的总体设计与排期。** code_editor 的全部逻辑(状态、事件、DSL、codegen、状态存储)都位于 `View` 抽象之上,而分离架构(Plan 386 Stage 1:VTree → RenderCommand lowering + loopback executor)是在 `View`/VTree 之下**新增一个后端** —— 与 gpui/vue 后端同构,边界天然对齐。前提是 §3.1 的三层分层纪律(core 不 import iced):Stage 1 将来只写一个新适配层(`EditorDrawList` → RenderCommand),core/draw/DSL/codegen 零改动。具体影响与对策:

| # | 影响点 | 对策 |
|---|---|---|
| 7.1 | `fill_raw(Raw { Weak<Buffer> })` 是**进程内指针共享**,无法跨 RenderQueue 传输 | 正文绘制走 `EditorDrawList`(draw.rs):文本表达为"按行稳定 id 键控的 shaped run"。iced 适配层:run → `fill_raw`(同进程,指针语义合法);lowering 层:run → `DrawText { font_id, glyphs }` 序列化(design 20 §4.1 的 glyphs 设计本就隐含 app 侧 shaping,与此一致) |
| 7.2 | 文本 shaping/布局必须留在 app 进程:光标定位、命中测试、软换行、word-motion 都需要**同步**布局,IPC 往返不可接受 | app 侧保留惰性/精选 FontSystem(编辑器只需等宽字体 + 默认 UI 字体);字形光栅化/字体图集归宿主(达成"font 复用"目标的是图集层)。**预期管理:编辑器类 app 的内存地板 = 文档 + undo 历史 + shaping,design 20 的 2–5MB 基线对其不适用**(现实预期 10–30MB,仍远低于 100MB 单体基线) |
| 7.3 | 行号槽 CPU 光栅 image 在分离架构下要走 `UpdateTexture` 传输,浪费 | gutter 双路实现:`EditorDrawList` 中行号本就是文本 run(lowering 直接 DrawText);image 路径只是 iced 适配层的缓存优化,见 §3.1 `gutter.rs` |
| 7.4 | 协议缺口(design 20 需补充的三点):① 事件下行通道缺 **IME**(preedit/commit/cursor rect)—— 分离模式下 winit 窗口在宿主侧,TSF 输入法状态必须转发给 app,否则编辑器中文输入失效(参照 wayland zwp_text_input);② 缺**字体注册**命令(app 自带等宽字体的上传通道,现有 `UpdateTexture` 不覆盖);③ 增量帧缓存需**按行**生效:编辑器滚动 = offset + 少量新暴露行,`CacheControl`/DirtyRect 应能挂在 draw list 的行稳定 id 上 | 记入 Plan 386 设计输入;本计划的行键控 EditorDrawList 设计使宿主端按行缓存自然成立 |
| 7.5 | 排期关系:Plan 386 处于暂缓(启动条件未满足),本计划**不等待、不阻塞** | 现在按 in-process iced 0.14 实施 —— in-process 本就是 Windows dev host(Plan 365 Host ①)的长期运行方式,Phase 0-5 无一次性代码;将来接入 Stage 1 仅新增 ~300–500 行 lowering 适配层 |
| 7.6 | 反向收益:编辑器是 RenderCommand 原语集(quad/text/image/clip/layer)最严苛的消费者(千级 glyph、按行高亮、高频局部重绘),toy widget 验证不了协议完备性 | 建议把 `examples/ui/041-code-editor`(或未来的 auto-edit 应用)纳入 Plan 386 Stage 1 的 golden 对照样例 |
| 7.7 | **时序结论(2026-08 复审,用户已采纳)**:不采用"RenderQueue 先行、auto-edit 后行"。分离架构是纯内存优化而非功能前置(Plan 386 启动条件即此意);editor 先行零丢弃(core 事件类型与 draw 契约已隔离);且 auto-edit 是最佳**协议压力测试**但非最佳**内存受益者**(地板 = 文档 + undo + shaping,轻量 app 才是分离架构的内存主受益方)。量化对比:RenderQueue 全程约 2.5–5 人月/1.5–2.5 万行 vs auto-edit 约 2–3 人周/2–2.5 千行(约 5:1);颠倒顺序最多省 iced 适配层 ~500 行 | 采纳的折中:与 Plan 413 并行做 editor-only 的 draw list → RenderCommand golden lowering **薄切片**(纯函数、无 transport/host,数天级),见 Plan 386 设计输入;全量 Stage 1 仍按其启动条件推进 |

(auto-edit 应用层 —— 文件树/多 tab/搜索栏/状态栏 —— 由现有 AutoUI widget 组合,同样位于 `View` 之上,不受分离架构影响。)

---

## 8. 实施启动指引(给实施 agent)

1. **入口与顺序**:Phase 0(spike)→ 1 → 2 → 3 → 4 → 5,严格按序。Phase 0 是闸门:独立 cargo 项目(建议 `scratch/code-editor-spike/`,**不进** auto-lang workspace),达成退出标准后才进入 Phase 1;若 cosmic-text 0.15 关键 API 缺失且 widget 层无法绕过,停下按 §2 方案 A 重新评估,勿硬闯。
2. **参考代码挂载**(在 cosmic-edit 仓库执行):`git worktree add ../cosmic-edit-1.0.2 epoch-1.0.2`。行为规格 = epoch-1.0.2(cosmic-text 0.15 纪元);改进 changelog = 1.0.2→HEAD 的 diff(重绘优化值得吸收)。**GPL 红线(§5.1):cosmic-edit 仅供阅读理解,禁止复制/机械改写其任何源码进 auto-lang**;可直接借阅的 MIT 参考上游 iced 0.14.0 tag(`D:\github\iced`)与 `D:\github\iced-code-editor`。
3. **硬约束**:§3.1 三层分层(core 与 draw.rs 禁止 import iced;`EditorInput`/`EditorDrawList` 契约);cosmic-text 锁 **0.15**(与 iced 0.14.0 统一,禁止引入第二个 minor —— 会产生双拷贝类型冲突);two-face 选与 cosmic-text 0.15 的 syntect 兼容版本;新 cargo feature `code-editor` 挂在 `ui-iced` 下;不做 iced 版本升级(§5.2 升级策略)。
4. **工程惯例**:每次编辑会话后 `cargo build -p auto`、改动后 `cargo test -p auto-lang`(仓库 CLAUDE.md 约定);Phase 4 生成 `.at` 示例前先调用 `/auto-lang-creator` skill;每阶段完成后更新本计划状态标记,全部完成后走 finish-plan 流程。
5. **验收**:以 §6 为最终验收 + cosmic-edit `TESTING.md` 行为清单(Phase 5);Windows 优先,Linux 复验。

---

## 附:调研证据索引

- cosmic-edit:`src/text_box.rs`(Widget impl 290、draw 384-960、update 962-1421、State 1446-1481)、`src/line_number.rs`、`src/tab.rs`(search 307-467)、`Cargo.toml`(libcosmic rev c1897c01、cosmic-text 0.19、syntect/two-face、default features)。
- iced 0.14.0 tag(D:\github\iced):`graphics/src/text.rs:116/180/359`、`core/src/event.rs:29`、`core/src/shell.rs`、根 `Cargo.toml:69/193`(highlighter feature、cosmic-text "0.15")。
- cosmic-text 0.15:docs.rs 确认 `ViEditor`/`SyntaxEditor` 导出;crates.io 声明 Linux/macOS/Windows 支持(fontdb 字体发现,Alacritty Windows 实证)。
- auto-lang:`crates/auto-lang/Cargo.toml:116`(iced 0.14 + advanced)、`ui/iced/renderer.rs`(TEXTAREA_CONTENTS 55-114、INPUT_TEXT 118、into_iced 1225、Textarea 1593)、`ui/aura_view_builder.rs`(859/1572/3278)、`ui_gen/rust.rs`(2141/3246/3294)、`ui_gen/widget/registry.rs`(806-833)。
- libcosmic Windows 史:pop-os/libcosmic#505 及 PR #507;epoch fork 结构(iced_winit 内 Wayland cfg-gate)。
