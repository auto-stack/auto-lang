# Plan 413 Phase 0 spike — 结论报告

**状态:✅ 闸门通过**(2026-08-19,Windows 11 验证)。技术路线 B1(cosmic-text ViEditor + iced 0.14 fill_raw)全部验证项达成,可进入 Phase 1。

## 验证矩阵(Plan 413 §4 Phase 0)

| # | 验证项 | 结果 | 证据 |
|---|---|---|---|
| 1 | `fill_raw` wgpu 渲染 | ✅ | PrintWindow 抓屏:深色背景 + 语法高亮 + 中文 + 行号 + 光标全部可见(`spike-print.png`) |
| 1 | `fill_raw` tiny-skia 渲染 | ✅ | `cargo build --no-default-features --features software` 编译 + 运行抓屏同样完整(`spike-tinyskia.png`)。**双后端都实现了 `graphics::text::Renderer::fill_raw`**(wgpu `lib.rs:781` / tiny_skia `lib.rs:314`) |
| 2 | 全局 `font_system()` 共享无死锁 | ✅ | widget update/draw 全部走 `iced::advanced::graphics::text::font_system()`(RwLock),单 UI 线程短锁策略(editor 锁与 font system 锁顺序一致),5 个无头测试 + 两实例长时间运行无死锁 |
| 3① | pop-os git 0.15.0 vs crates.io 0.15.0 API 一致性 | ✅ | `ViEditor::new(SyntaxEditor)` / `SyntaxEditor::new(buffer, &SyntaxSystem, theme)` / `SyntaxSystem` / `Edit` trait(`action/shape_as_needed/cursor_position/selection_bounds`)/ `Action` 全套按 cosmic-edit 1.0.2 用法可用,无漂移 |
| 3② | fork 专属 API 降级清单 | ✅ | 见下表,三项全部找到上游等价物 |
| 4 | IME 链路 | ✅(管线)/⏳(OS 层) | `Event::InputMethod::{Preedit,Commit,Opened,Closed}` + `shell.request_input_method` + on-the-spot preedit 绘制已实现;`iced_test` 注入 Commit 事件测试通过;微软拼音实机输入需 Phase 5 人工清单确认 |
| 5 | 行号槽 CPU 光栅 + Nearest | ✅ | font-size 1.0 布局缓存 + `physical((x,y), FONT_SIZE)` 放大 + `SwashCache::with_pixels` alpha 混合 + `image::Handle::from_rgba`,仅 `editor.redraw()` 时重画;100% DPI 下清晰(150% 待实机) |
| 6 | two-face 与 cosmic-text syntect 统一 | ✅ | `cargo tree -i syntect` → **单实例 5.3.0**(cosmic-text 0.15.0 与 two-face 0.4.5 共享);cosmic-text 全图单实例 0.15.0 |

## fork(pop-os iced)→ 上游 iced 0.14.0 差异表(Phase 0 产出)

| fork API | 上游 0.14.0 等价物 | 说明 |
|---|---|---|
| `Style::scale_factor`(draw 参数) | **无需替代**(正文)/ `window::Event::Rescaled` 跟踪(行号槽) | 上游 `fill_raw` 路线的 cryoglyph `TextArea.scale = transformation.scale_factor()`,正文按逻辑坐标 shape、光栅自动按 DPI 放大,**比 fork 的整数像素搜索更干净**;行号槽 CPU 光栅需要物理像素,spike 用 `Rescaled` 事件跟踪(初始值默认 100% 的局限记录在案,Phase 1 可在首帧用 `window` 命令查询补齐) |
| `modified_key` | 自维护 `Modifiers`(KeyPressed/KeyReleased/ModifiersChanged) | spike 的 `EditorShared.modifiers`;真实键盘路径下 iced `KeyPressed.modifiers` 字段已够用,自维护值留给事件合成场景 |
| surface-message(弹窗) | 不需要 | widget 层只出 `on_context_menu` 回调消息(Phase 1 契约),弹层由 app 组合 |

## 无头管线测试(iced_test,5/5 通过)

`cargo test` 驱动真实 widget update 路径:

- `typewrite_inserts_text_at_cursor` — 键盘 → `Action::Insert` → ViEditor → buffer
- `enter_splits_line_and_undo_restores` — Enter 分行 + Ctrl+Z 撤销
- `shift_arrow_extends_selection` — Shift+方向键选区锚定(selection_bounds 字节范围断言)
- `ime_commit_inserts_text` — `InputMethod::Commit("你好")` → insert_string,中文入库
- `click_moves_cursor_to_clicked_line` — 鼠标 → 命中测试 → 光标移动

## 其他事实(给 Phase 1)

- **fallback renderer 兼容**:同时开 wgpu+tiny-skia(iced 默认 feature)时 `iced::Renderer` = `iced_renderer::fallback::Renderer`,它实现了 `graphics::text::Renderer`(fill_raw)/`image::Renderer`/`core::Renderer`(fill_quad),自定义 widget 直接写 `impl Widget<Message, Theme, iced::Renderer>` 即可,auto-lang 现有 feature 组合无需改动。
- **Widget trait 形态**:`update` 返回 `()`(用 `shell.capture_event()`,不是返回 Status);`draw` 里 `renderer.fill_text` 的 `Text` 结构含 `wrapping` 字段;`iced::application(boot, update, view)` 第一参数是状态构造函数。
- **Buffer 共享**:`Arc<Buffer>` → `BufferRef::Arc` 进 `SyntaxEditor`,`Arc::downgrade` 给 `Raw{buffer}`;编辑经 `Arc::make_mut` 原地生效,Weak 不会悬空。
- **行号右对齐**:`format!("{number:>digits$}")` + 左侧 pad,monospace attrs。
- **Windows 键盘布局**:`KeyPressed.text` 已含合成字符(dead key 等),Ctrl 组合键时为 None —— 逻辑与上游 text_editor 一致。

## GPL 合规声明

本 spike 为 MIT 原创:文件头标注 "Architecture inspired by cosmic-edit (GPL-3.0, System76); original implementation"。cosmic-edit(epoch-1.0.2 worktree,`D:\github\cosmic-edit-1.0.2`)仅作行为规格与架构阅读,未复制其任何源码。行号光栅的 font-size 1.0 缩放技术、fill_raw 四层 draw 结构等为公开技术思想,代码为独立实现(结构、命名、错误处理均不同)。

## 运行

```
cargo run                     # 默认(wgpu + tiny-skia fallback)
cargo run --no-default-features --features software   # tiny-skia 单后端
cargo test                    # 5 个无头管线测试
cargo tree -i syntect         # syntect 单实例证明
```

F6 切软换行,F7 切 vi 模式(ViEditor passthrough 开关)。状态栏显示 cursor/selection/字符数。

## 残留(交 Phase 1 / Phase 5)

1. OS 级 IME(微软拼音)实机输入 → Phase 5 手动清单(代码路径已被无头测试覆盖)。
2. 150% DPI 下行号槽清晰度 → Phase 5(方案已备:`Rescaled` 跟踪 + 光栅尺寸 ×scale)。
3. Linux 复验(X11/Wayland)→ Phase 5。
4. 滚动条、拖选自动滚动、双击词选 → Phase 1 widget 本体范围。
