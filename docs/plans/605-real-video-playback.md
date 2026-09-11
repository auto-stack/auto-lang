---
plan_id: PLAN-605
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: real-video-playback
author: [Antigravity]
created_at: 2026-09-11
updated_at: 2026-09-11
plan_revision: 1
current_step: 0
total_steps: 6

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: HTML5 video 原生直通标签与双端播控契约"
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉与行为一致（Vue 与 VM/iced 双端，video 标签直通与优雅降级）"
  - "GOAL-010: 示例应用轨道（030-video-player 与 019-video-app 升级为真正可播放视频的应用）"

affects: [auto-lang/ui, examples/ui/030-video-player, examples/ui/019-video-app]
---

# [PLAN-605] real-video-playback — 原生视频标签支持与 030 / 019 真实视频播放落地

## 0. 变更摘要

在现有的 AutoUI 中，编译器尚未支持原生 HTML5 `<video>` / `<audio>` 标签，`map_tag` 遇到 `video` 时会作为未知标签降级为普通的 `<div>`。这导致 `examples/ui/030-video-player`（系统级媒体播放器）和 `examples/ui/019-video-app`（Bilibili 式 Web 视频门户）仅能通过 CSS 渐变色块与模拟数字来展示播放器界面，无法播放任何真正的视频文件。

本计划遵循用户提议 **“先优化 030 让它能播放真正的视频，再把后端服务与播放模块集成回 019”** 的演进路线，实施端到端闭环：
1. **编译器放行原生多媒体标签**：在 `crates/auto-lang/src/ui_gen/vue.rs` 的 `map_tag` 中放行 `video` 和 `audio` 为原生 HTML5 标签，打通 `src`、`controls`、`autoplay`、`loop`、`muted`、`poster` 等原生属性透传；
2. **030-video-player 具备真视频播放能力**：
   - 配置 `api: "rust"`，编写强类型 Axum 后端（`src/back/api.at` + `src/back/db.at`），引入轻量级开源演示测试视频资产；
   - 升级 `src/front/app.at` 视口为真实 `video` 标签，绑定视频 URL，打通与 OSD 播控（播放/暂停、快进快退、进度跳转、倍速、音量、静音、切集）的联动；
   - 确立双端兼容契约：VM/Iced 模式在底层原生解码器（FFmpeg/mpv）就绪前优雅降级为封面海报与状态反馈，确保 `auto run -r vm` 与 MCP 测试不报错；
3. **将视频流后端与真播放器视口集成回流至 019-video-app**：
   - 在 `019-video-app/src/back/` 增加视频流服务能力，将视频资源 URL 关联至种子数据库；
   - 升级 `019-video-app/src/front/pages/watch.at` 视口为真实的 `video` 播放器，实现从首页推荐列表点击卡片跳转后，观看页能真正播放对应视频；
4. **双端自动化测试全绿**：更新并补充 Playwright E2E（包含真实 `<video>` 渲染断言）与 AutoUI MCP VM 冒烟测试。

---

## 1. 目标

### 1.1 核心目标
1. **编译器原生媒体标签直通**：AutoUI Vue 生成器将 `video` 和 `audio` 原样发射为 HTML5 `<video>` 和 `<audio>`，并透传相关属性与事件。
2. **030 系统播放器真视频播放**：`030-video-player` 支持加载并播放真实的视频文件，OSD 控制条与真实播放状态联动（时间轴、音量、倍速、播放/暂停、切歌）。
3. **019 视频门户真视频播放**：`019-video-app` 观看页接入真实视频播放，支持从首页卡片导航至观看页起播。
4. **双端健全与测试全绿**：VM 模式保持优雅降级，Playwright E2E 和 AutoUI MCP VM 测试 100% 通过。

### 1.2 非目标 (Non-Goals)
- **底层 C/Rust 硬解（FFmpeg / libmpv）接入**：VM/Iced 模式下引入 FFmpeg/mpv 解码并刷新 wgpu 纹理属于高风险的底层 L2 架构任务，本计划聚焦于 Vue 模式下的真实播放与全栈服务落地，VM 端遵循 Design 30 确立的“阶段一：高精度 UI 控件 + 海报与状态降级”原则。
- **重型流媒体协议（HLS / DASH / DRM）**：本阶段使用标准 HTTP/HTTPS 渐进式流（MP4 / WebM），不引入 hls.js 或复杂的流媒体分片协议。

---

## 2. 架构方案

```
+───────────────────────────────────────────────────────────────────────────────+
| AutoUI 架构与媒体流拓扑                                                          |
+───────────────────────────────────────────────────────────────────────────────+
|                                                                               |
|  [AutoLang 编译器 (ui_gen/vue.rs)]                                             |
|      │                                                                        |
|      ├── map_tag: 放行 "video" / "audio" 原生 HTML5 标签直通                     |
|      └── props:   透传 src, poster, controls, autoplay, loop, muted 等属性    |
|                                                                               |
|  [030-video-player (系统播放器)]                  [019-video-app (门户网站)]   |
|      │                                                │                       |
|      ├── src/front/app.at                             ├── src/front/pages/    |
|      │    └── <video :src="current_url" ...>          │    └── watch.at       |
|      │                                                │         └── <video>   |
|      └── src/back/api.at & db.at                      └── src/back/api.at     |
|           └── Axum 静态/流服务 (HTTP 200/206)              └── 视频 URL 与流服务 |
|                                                                               |
|  [运行模式分流]                                                                |
|      ├── Vue 模式 (auto run): 浏览器原生硬件解码播放真视频                        |
|      └── VM 模式  (auto run -r vm): 视口优雅降级为封面与状态，OSD 交互不挂        |
+───────────────────────────────────────────────────────────────────────────────+
```

---

## 3. 技术栈

- 语言与运行时：AutoLang (`.at`)、Rust (Axum 后端、编译器 `auto-lang`)
- 前端代码生成：Vue 3、HTML5 `<video>` API、Tailwind CSS
- 测试框架：Playwright E2E (Web)、AutoUI MCP (VM)
- 媒体格式：WebM / MP4 (H.264 / VP9 / AV1)

---

## 4. 需求分析与背景调查

1. **需求来源**：
   - 用户审查发现 `019-video-app` 与 `030-video-player` 两个视频相关示例均无法播放真实视频。
   - 用户明确指示：“建议先优化 030 让它能播放真正的视频，再把它的后台代码集成回 019，让两边的 app 都能播放视频。”
2. **代码现状**：
   - `crates/auto-lang/src/ui_gen/vue.rs` 中的 `map_tag` 仅包含 `img` / `image`，缺少 `video` / `audio`，落入通配通告 `_ => "div".to_string()`；
   - `examples/ui/030-video-player` 目前无后端（纯前端单文件），视口仅是 CSS 渐变背景与图标；
   - `examples/ui/019-video-app` 拥有 Axum 后端（`src/back/api.at` + `db.at`），但只管理元数据，视口也是 CSS 渐变色块。
3. **授权范围**：
   - 允许修改 `crates/auto-lang/src/ui_gen/vue.rs` 编译器标签映射；
   - 允许为 `030-video-player` 配置后端、添加测试媒体资产及升级前端；
   - 允许改造 `019-video-app` 的后端数据与前端观看页视口。

---

## 5. 详细设计

### 5.1 规范增量 (Spec Deltas)

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-lang/ui/overview.md` | before: `video` / `audio` 未知标签降级为 `div`<br>after: `video` / `audio` 放行为原生 HTML5 标签直通 | 使得 AutoUI 能够声明原生的多媒体视频与音频播放元素 | AC-01 |
| SD-02 | add | `docs/specs/auto-lang/ui/overview.md` | before: 示例无真实视频播放<br>after: 030 与 019 具备强类型后端视频流与真播放能力 | 满足系统原生播放器与 Web 流媒体门户的应用完整性标准 | AC-02, AC-03 |

### 5.2 编译器标签直通 (`crates/auto-lang/src/ui_gen/vue.rs`)
在 `map_tag` 的 `// Media` 节中追加：
```rust
// Media
"video" | "Video" => "video".to_string(),
"audio" | "Audio" => "audio".to_string(),
"source" | "Source" => "source".to_string(),
"track" | "Track" => "track".to_string(),
"image" | "Image" => "img".to_string(),
"img" | "Img" => "img".to_string(),
"icon" | "Icon" => "span".to_string(),
```
确保在生成 Vue SFC 时，DSL 中的 `video { src: .url, controls: true }` 正确输出为 `<video :src="url" :controls="true">`。

### 5.3 030-video-player 后端与媒体资产
1. `pac.at` 开启 `api: "rust"`，配置 `back_port: 8330`；
2. 引入轻量测试短视频（体积控制在 1MB 左右，或使用成熟、免版权的高可用测试 URL / 后端内置的合成分片）；
3. 编写 `src/back/api.at` 与 `src/back/db.at`，提供播放列表查询与视频流信息接口。

### 5.4 030-video-player 前端改造 (`src/front/app.at`)
1. 视口中央替换为真实的 `video` 标签，绑定 `src: .current_video_url`；
2. 增加或联动播放/暂停、快进快退、进度条寻道、音量调节；
3. 保留 OSD 悬浮条与控制逻辑，在 VM 模式下展示封面图与状态徽章。

### 5.5 019-video-app 后端与前端集成
1. `src/back/db.at` 中的 `Video` 结构体增加 `video_url: str` 字段；
2. `src/front/pages/watch.at` 中的播放器容器替换为真实 `video` 元素，接入 `.video.video_url`，实现点击播放；
3. 保持现有点赞、浏览量、分类过滤与深浅主题功能 100% 不受影响。

---

## 6. 测试设计

1. **编译器单测**：
   - 验证 `video { src: "test.mp4" }` 能够被 `auto gen` 正确转译为 `<video src="test.mp4"></video>`（非 `<div>`）。
2. **030 Playwright E2E 测试**：
   - 验证页面包含 `<video>` 标签；
   - 验证视频 `src` 属性正确绑定且非空；
   - 验证点击播放/暂停、切集能够正确改变当前视频源与状态机。
3. **019 Playwright E2E 测试**：
   - 验证进入 `/watch/:id` 后，页面成功挂载 `<video>` 标签且 `src` 匹配该视频的数据源；
   - 验证原有的 T1~T10 冒烟用例全部持续通过。
4. **VM MCP 冒烟测试**：
   - 运行 `030` 和 `019` 的 `vm-smoke.mjs`，确认 VM 模式下控件渲染正常，无运行时崩溃。

---

## 7. 验收标准

- **AC-01**：编译器成功放行 `video` / `audio` 标签，Vue SFC 模板中出现 `<video>` 而非 `<div>`。
- **AC-02**：`030-video-player` 拥有独立 Rust 后端，提供视频媒体源；前端能加载并渲染真实的 `<video>` 元素。
- **AC-03**：`030-video-player` 的 OSD 播控（播放/暂停、切集、进度提示）与状态联动正确。
- **AC-04**：`019-video-app` 观看页成功升级为真实 `<video>` 视口，原有的分类、推荐、点赞功能均正常。
- **AC-05**：`030` 与 `019` 在 VM 模式下运行平稳，AutoUI MCP 冒烟测试 PASS。
- **AC-06**：`030` 与 `019` 的 Playwright E2E 测试全部全绿通过。

---

## 8. 执行步骤

### T-01: 编译器支持原生 `video` / `audio` 标签
- **文件**：`crates/auto-lang/src/ui_gen/vue.rs`
- **操作**：在 `map_tag` 函数中添加 `video`, `audio`, `source`, `track` 的原生直通映射。
- **验证**：`cargo check -p auto-lang`，并针对包含 `video` 的测试用例执行 `cargo test`。

### T-02: 030-video-player 搭建后端与引入媒体源
- **文件**：`examples/ui/030-video-player/pac.at`, `examples/ui/030-video-player/src/back/api.at`, `examples/ui/030-video-player/src/back/db.at`
- **操作**：配置 `api: "rust"`, `back_port: 8330`；创建后端接口与视频种子数据（包含真实的有效视频测试流 URL / 文件路径）。
- **验证**：`auto run` 后访问 `http://127.0.0.1:8330/api/videos` 返回有效 JSON。

### T-03: 030-video-player 前端接入真视频标签与播控联动
- **文件**：`examples/ui/030-video-player/src/front/app.at`
- **操作**：将渐变色占位区域改造为 `<video>` 播放视口，绑定 `src`，并与 OSD 控制条、播放列表联动；针对 VM 模式保留海报与降级保护。
- **验证**：`auto gen` 检查生成的 SFC 包含 `<video>` 标签。

### T-04: 030-video-player 双端测试验证
- **文件**：`examples/ui/030-video-player/tests/smoke.spec.ts`, `examples/ui/030-video-player/tests/vm-smoke.mjs`
- **操作**：更新 Playwright 测试增加 `<video>` 元素和真实属性断言；运行 VM MCP 冒烟测试。
- **验证**：Playwright 测试全绿，VM 冒烟全绿。

### T-05: 019-video-app 后端扩充视频源与观看页升级
- **文件**：`examples/ui/019-video-app/src/back/db.at`, `examples/ui/019-video-app/src/front/pages/watch.at`
- **操作**：在后端 `Video` 增加 `video_url`；在观看页将占位色块替换为真实 `<video>` 播放器。
- **验证**：`auto gen` 成功输出，Vue 前端正常载入真实视频标签。

### T-06: 019-video-app 测试验证与全局回归
- **文件**：`examples/ui/019-video-app/tests/smoke.spec.ts`
- **操作**：运行 019 的 Playwright 10/10 冒烟套件及 VM 冒烟测试。
- **验证**：Playwright 10/10 PASS，VM 冒烟 PASS，`cargo check -p auto-lang` clean。

---

## 9. 复审记录

- **stage**: new
- **plan_id**: PLAN-605
- **revision**: 1
- **outcome**: pass
- **next**: work (T-01)

---

## 10. 待澄清事项

1. **测试视频资产体积**：为防止 git 仓库膨胀，本计划采用轻量开源的公共测试视频片段（如 Big Buck Bunny 示例 WebM/MP4）或本地微小（<500KB）测试视频，避免引入大体积多媒体文件。
