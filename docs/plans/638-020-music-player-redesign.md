---
plan_id: PLAN-638
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: 020-music-player-redesign
author: [Antigravity]
created_at: 2026-09-17
updated_at: 2026-09-17

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 1
total_steps: 4
---

# [PLAN-638] 020-music-player-redesign

## 变更摘要

参考主流音乐播放器（QQ音乐、网易云音乐、汽水音乐、Spotify）重塑 `020-music-player` 的整体界面与视觉动效，并支持播放用户本地音乐仓库（默认 `E:\Music\`）。
扩展 `crates/auto-lang/src/ui/media_service.rs` 支持常见音频格式（flac/mp3/wav/ogg/m4a/aac），在 `pac.at` 中引入 `api: "rust"` 与 `media_root: "E:\\Music\\"`，前端采用现代组件化架构（app, player_store, sidebar, stage, tracklist, controls, queue_drawer, viewport）。

## 目标

1. **界面与交互现代化**：彻底摆脱原本简陋粗糙的两列展示，实现三栏专业级桌面音乐播放器界面，包括黑胶唱盘舞台模式（网易云/汽水音乐风格）与本地歌曲列表模式（QQ音乐/Spotify风格），提供沉浸式底部播控台（拖拽 Seek 进度条、播放模式、音量、待播队列抽屉）。
2. **本地音乐真实扫描与播放**：支持真实扫描 `E:\Music\` 目录下的 393 首歌曲，并经由 HTTP 206 断点流式播放无损音频（FLAC, MP3, WAV）。
3. **完善的曲目元数据与播控能力**：自动解析歌手与歌名、格式徽章、即时搜索与过滤、红心收藏、循环/随机模式、自动切歌。

## 架构方案

1. **媒体文件服务扩展 (`media_service.rs`)**：
   - 将 `mp3`, `flac`, `wav`, `ogg`, `m4a`, `aac` 添加至 `SUPPORTED_EXTENSIONS`。
   - `content_type` 映射对应音频 MIME 类型。
2. **工程配置 (`pac.at`)**：
   - 接入 `api: "rust"`, `media_root: "E:\\Music\\"`, `front_port: 3020`, `back_port: 8320`, `theme: "dark"`, `accent: "emerald"`。
3. **前端组件化设计 (`src/front/`)**：
   - `player_store.at`: 全局播放状态、曲库数据、过滤搜索、Seek、播放控制。
   - `sidebar.at`: 品牌、发现/我的本地/收藏等导航、Mini播放封面。
   - `stage.at`: 旋转黑胶唱盘、唱臂、16段音波律动条、歌曲信息卡片。
   - `tracklist.at`: 本地曲库统计、搜索框、格式过滤标签、高交互歌曲列表。
   - `controls.at`: Spotify/网易云风格底部播控台、进度条与时钟、音量控制。
   - `queue_drawer.at`: 当前待播队列抽屉。
   - `viewport.at`: 受控媒体解码器与原生事件回灌。
   - `app.at`: 根入口。

## 执行步骤

- [ ] **Step 1: 扩展底层媒体服务**：修改 `crates/auto-lang/src/ui/media_service.rs`，添加音频白名单扩展名与 MIME 类型映射。
- [ ] **Step 2: 配置 020-music-player 工程**：修改 `examples/ui/020-music-player/pac.at`。
- [ ] **Step 3: 实现前端组件与状态层**：编写 `player_store.at`, `sidebar.at`, `stage.at`, `tracklist.at`, `controls.at`, `queue_drawer.at`, `viewport.at`, `app.at`。
- [ ] **Step 4: 验证与文档沉淀**：验证类型检查、后端扫描与真实播放，更新 README。
