# iced_widget 本地补丁（auto-os）

对 crates.io `iced_widget-0.14.2` 的最小手术，经根 Cargo.toml 的
`[patch.crates-io]` 挂载（升级 iced 版本时需重做/重审本补丁）。

## 补丁内容：container.rs 布局空窗容错（5 处）

iced 0.14 的 `Container::operate/update/mouse_interaction/draw/overlay`
在**布局树与部件树错位窗口期**（视图重建与 operate/draw 竞态，P041-D1
族）会 `layout.children().next().unwrap()` 打穿——一个 app 的布局残缺
直接**杀死整个桌面进程**（宿主 panic 族实证：017-chat 三次复现 +
2026-09-24 017 围栏放行实验 + 2026-09-24 晚用户正常使用再崩）。

补丁语义：children 空窗时**良性降级**——operate/update/draw 跳过本拍、
mouse_interaction 回退默认、overlay 返回 None。不改变任何有子布局时的
行为；至多丢一拍操作/绘制，不崩进程。

## 维护

- 升级 iced 时：diff 新版 container.rs 与本目录，确认上游是否已修
  （上游若改为容错，删本补丁回归 crates.io 版）。
- 其余部件（button/pin/scrollable/text_input/tooltip/responsive）同款
  unwrap ~25 处**未补**——观测到的崩溃全部落在 container.rs:291
  （operate 臂），其余留观；再现同型崩溃时按需扩展。
