---
plan_id: PLAN-606
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: photo-gallery-thumbnails
author: [zhaopuming]
created_at: 2026-09-11
updated_at: 2026-09-11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals:
  - "GOAL-007: AutoUI 跨端一致——image dynamic prop/bindings 解析与 fit 双端对齐"
  - "GOAL-010: 示例应用轨道——examples/ui/029-photo-gallery 缩略图功能完善"

affects: [auto-lang/ui, auto-lang/examples]
current_step: 1
total_steps: 5
---

# [PLAN-606] photo-gallery-thumbnails — 029 图库缩略图功能完善与 Image 绑定修复

## 变更摘要

修复 `examples/ui/029-photo-gallery` 在打开后的浏览界面中图片无法显示为缩略图、均呈现统一蓝色替代图的问题。
根因为 `aura_view_builder.rs` 中 `image` 标签转换分发未透传当前作用域的 `bindings`，导致循环变量 `item.thumb` 在 view 构建时被求值为空字符串 `""`，进而触发 Iced 渲染器的占位蓝块降级；同时 `fit` prop 亦未被解析为 `object-fit` 样式。
本计划在核心层修复 `image` 标签的上下文绑定与 `fit` 解析支持、在 Iced 渲染器中支持 `data:image/` 协议，并在 `029-photo-gallery` 中完善兼具确定性高保真与动态网络特性的缩略图功能。

## 目标

1. **G1 核心绑定修复**：`aura_view_builder.rs` 转换 `image`/`img`/`icon` 时透传 `bindings`，使 `for item in .list` 循环内的 `image (src: item.prop)` 能正确获取动态属性值，不再产生空 URL。
2. **G2 样式与协议增强**：
   - 支持 `image` 的 `fit: "cover" | "contain" | "fill" | "none" | "scale-down"` 属性，映射为对应 `object-fit` 样式（VM 注入 `StyleClass::ObjectFit`，Vue 输出 `object-*` 类）。
   - Iced 渲染器 `load_image_bytes` 增强支持 `data:image/`（SVG / PNG / JPEG 等 Data URL），使内嵌/离线缩略图具备零网络开销、秒级呈现能力。
3. **G3 029 示例缩略图完善**：`029-photo-gallery` 打开后浏览界面 24 张照片网格均能立即呈现主题明确的高质量缩略图（自然/城市/天空/抽象），彻底消除无图蓝块，且大图查看器联动完好。
4. **G4 双端机验覆盖**：补齐 `029-photo-gallery` 的单元测试与冒烟测试，保障 Vue（Playwright）与 VM（AutoUI MCP）双端一致可用。

## 需求分析与背景调查

- **Plan 537 存留债务**：Plan 537 引入了 `029-photo-gallery`，但由于当时 `convert_image_or_icon` 为 Plan 408 静态图标时期的旧代码，未接收 `bindings`，且未发现测试中 VM 呈现的蓝块实际是 `src: ""` 导致的降级错误而非“网络延迟”；
- **Iced 渲染降级行为**：`crates/auto-lang/src/ui/iced/renderer.rs:5010-5028`，当 `load_image_bytes` 返回 `None` 时，降级渲染一个背景为 `iced::Color::from_rgb(0.24, 0.47, 0.85)`（浅深海蓝）的矩形容器；
- **Data URL 兼容性**：Vue 浏览器原生完全支持 `data:image/svg+xml,...` 与 `data:image/png;base64,...`；Iced VM 若补全 `data:` 前缀解码，将使同一份数据在双端均可实现 100% 离线、瞬时就绪的缩略图体验。

## 详细设计

1. **`aura_view_builder.rs`**：
   - 修改 `convert_image_or_icon(&self, props: &HashMap<String, AuraPropValue>, bindings: &Bindings) -> View<DynamicMessage>`。
   - `src` 从 `self.extract_string_with(props, "src", bindings)` 获取。
   - 读取 `self.extract_string_with(props, "fit", bindings)`，若匹配 `cover|contain|fill|none|scale-down`，构建 `StyleClass::ObjectFit` 追加进 `style`。
   - `convert_element` 与 `convert_element_untracked` 中 `"img" | "image" | "icon"` 以及 `"avatar-image"` 分支传入 `bindings`。
2. **`iced/renderer.rs`**：
   - `load_image_bytes(url: &str)` 拦截 `data:` 前缀：
     - 分离 `;base64,` 或 `,`。
     - 若为 base64 则调用 `base64::engine::general_purpose::STANDARD.decode`。
     - 若为 utf8/svg 则直接取字节，使 SVG 及位图 data URL 无缝进入 Handle 缓存并渲染。
3. **`ui_gen/vue.rs`**：
   - `"image" | "img"` 标签发射时，提取 `fit` prop 并追加 `object-{fit}` 至 class 列表。
4. **`029-photo-gallery` 缩略图完善**：
   - 为 24 张照片内置确定性主题缩略图（SVG Data URL，根据 4 种相册主题：晨雾山谷/林间小径为翠绿山峦渐变，夜色天桥/老城街角为深蓝城市剪影，落日余晖/破晓霞光为橙红晚霞渐变，光影折纸/色彩流体为多边形抽象色相），同时保留大图查看器网络加载能力。

## 执行步骤

1. **T1 骨架准备与 Worktree 创建**：
   - 提交 master 上的 `docs/plans/.next-id` 与 `docs/plans/606-photo-gallery-thumbnails.md`。
   - 创建专用 worktree：`git worktree add D:/autostack/.wt/lang-606/auto-lang -b plan-606-dev`。
2. **T2 核心视图构建器与渲染器增强**：
   - 在 worktree 中修改 `crates/auto-lang/src/ui/aura_view_builder.rs`，修复 `image` 的 bindings 透传与 `fit` prop 解析。
   - 在 `crates/auto-lang/src/ui/iced/renderer.rs` 中为 `load_image_bytes` 增加 `data:` 协议支持。
   - 在 `crates/auto-lang/src/ui_gen/vue.rs` 中增加 `fit` prop 类名发射。
   - 编写 `aura_view_builder` 与 `load_image_bytes` 单测，运行 `cargo t aura_view_builder` 与 `cargo t iced` 验证。
3. **T3 029 示例缩略图功能完善**：
   - 在 `examples/ui/029-photo-gallery/src/front/app.at` 中完善 24 张照片的缩略图与大图配置，更新 `SPEC.md`。
   - 执行 `auto build` 验证编译。
4. **T4 双端运行与自动化冒烟验证**：
   - 在 `examples/ui/029-photo-gallery` 下添加自动化冒烟测试脚本 `tests/smoke.spec.ts` 与 `tests/vm-smoke.mjs`。
   - 运行 Vue 与 VM 冒烟测试，断言卡片缩略图全部有效渲染，无全蓝替代块。
5. **T5 复审、沉淀与收尾归档**：
   - 执行 `/auto-plan:review` 独立复审，检查 checklist。
   - 回写 specs 与归档 plan，按规范安全移除 worktree 并合并。

## 复审记录

（待复审时填写）

## 待澄清事项

- 无。
