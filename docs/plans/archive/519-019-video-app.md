---
plan_id: PLAN-519
status: archived                 # drafting → executing → execution_done → reviewed → archived
feature_name: 019-video-app 升级为完整 App
author: [zhaopuming]
created_at: 2026-09-02
updated_at: 2026-09-02

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 示例升级不预定改 specs；过程性 codegen 修复按实际改动补记
current_step: 10
total_steps: 10
---

# [PLAN-519] 019-video-app：Bilibili 式视频浏览器升级为完整 App

> **纲领引用**：本计划是 [Plan 401](401-autoui-examples-upgrade.md) 的子计划，
> 硬指标（"完整 App"四条）、五步流程与全部技术约定以纲领 §升级标准 / §流程约定 /
> §技术约定 为准，此处不重述。**2026-09-02 裁定**：019 维持 App 轨立项（解除 ⏸ 待裁定）。

## 变更摘要

把 `examples/ui/019-video-app` 从 135 行单文件静态玩具（散装 `vid1_title`…`vid6_views`
变量、no-op handler、6 张写死的视频卡）升级为对标 018-book-reader 的完整 App：

- 新增强类型 Rust 后端（`src/back/{api.at, db.at}`）：视频库 CRUD + 分类过滤 + Tab 排序 + 搜索 + 点赞/播放计数。
- 前端拆多模块：`app.at` 路由壳 + `video_store.at` 共享 store + `settings.at` 5色主题设置面板 + `pages/home.at`（首页网格）+ `pages/watch.at`（观看页）。
- 端到端验证：`auto gen` + `auto run` + curl 后端 + playwright 10/10 + AutoUI MCP VM 冒烟（双端一致性）。
- README 更新为新架构。

## 目标

1. **分类 chips 真过滤**：点击 Gaming/Music/Tech/Food 分类 chip，网格只显示该分类视频（后端过滤，非前端隐藏）。
2. **Tab 切内容源**：Recommend（默认序）/ Trending（按 views 降序）/ Following（仅关注作者的视频）。
3. **搜索真过滤**：顶部搜索框对 title/author 做子串匹配（不区分大小写）。
4. **观看页**：`/watch/:id` 显示视频详情（标题/作者/播放量/点赞/简介）+ 相关推荐（同分类其余视频）。
5. **交互计数**：进入观看页 `POST /api/videos/:id/view` 播放量 +1；点赞按钮 `POST /api/videos/:id/like` 点赞 +1（可再点取消）。
6. **双端深浅模式与5色强调色切换**：集成 `017-chat` 同款 SettingsPanel（🌙/☀ 切换 + 5 色调色板）。
7. **playwright & VM MCP 全绿**：覆盖上述用例全部通过（干净态可复现）。

## 验收标准

1. Plan 401 四硬指标全满足：多模块前端 / 强类型后端 / 端到端验证 / README 更新。
2. playwright 全绿（10/10，干净态复跑验证无状态残留）。
3. VM MCP 冒烟全绿（AutoUI MCP 驱动界面与设置面板交互）。
4. curl 冒烟全部返回预期 JSON。
5. `auto check`（或 `auto gen`）无 error。
6. 无临时调试打印、无未说明的 workaround。

## 执行步骤

1. **pac.at 端口与后端声明**：`examples/ui/019-video-app/pac.at` 追加 `api: "rust"`、`front_port: 3019`、`back_port: 8319`、`theme: "dark"`、`accent: "pink"`。[✅ 已完成: pac.at 已配置]
2. **新增 `.gitignore`**：`examples/ui/019-video-app/.gitignore`，放行 `!vue/` + `!tests/package.json` + `!tests/pnpm-lock.yaml`。[✅ 已完成: gitignore 已添加]
3. **后端数据层**：新建 `examples/ui/019-video-app/src/back/db.at`：12 条种子覆盖 5 分类 + 过滤/排序/播放/点赞。[✅ 已完成: 12条种子与过滤/点赞逻辑实现]
4. **后端 API 层**：新建 `examples/ui/019-video-app/src/back/api.at`：5 个 `#[api]` 端点委托 db。[✅ 已完成: 强类型 API 接口定义完成]
5. **前端 store 与设置**：新建 `video_store.at` 与 `settings.at`（VideoStore + SettingsPanel 5色主题与深浅切换）。[✅ 已完成: 支持 5 色与暗色切换]
6. **首页页**：重写 `src/front/pages/home.at`（chips + tabs + 搜索 + responsive grid）。[✅ 已完成: 首页网格与过滤]
7. **观看页**：新建 `src/front/pages/watch.at`（播放器预览 + 点赞计数 + 简介 + 推荐列表）。[✅ 已完成: 观看页与相关推荐]
8. **路由壳**：重写 `src/front/app.at`（routes 2 条 + 侧边栏导航 + 设置弹窗 + outlet）。[✅ 已完成: 双路由壳就绪]
9. **测试套件**：新建 `tests/{package.json, playwright.config.ts, smoke.spec.ts, acceptance.atd, vm-smoke.mjs, test_019_vm.py}`。[✅ 已完成: Playwright 10/10 与 VM smoke 全部通过]
10. **README 与纲领回写**：更新 `README.md` 与 `Plan 401` 总表。[✅ 已完成: 纲领总表与提交历史已更新]

## 复审记录

- **Checklist Audit**:
  - [x] 多模块前端：`app.at`, `video_store.at`, `settings.at`, `pages/home.at`, `pages/watch.at` 架构完整。
  - [x] 强类型 Rust 后端：`api.at`, `db.at` 生成 Axum 服务，端口 `8319` 避开 Windows NAT 保留端口段。
  - [x] 主题与强调色支持：集成 🌙/☀ 暗浅模式及 5 色调色板（Pink, Indigo, Ocean, Sage, Amber）。
  - [x] Playwright 测试：10/10 用例全部通过（首页、分类、Tab、搜索、观看页、点赞、推荐、深浅模式、强调色、控制台无报错）。
  - [x] VM MCP 冒烟测试：`node tests/vm-smoke.mjs` PASS。
- **Workaround Scan**: 零临时 hack，标准 AutoUI 语法与 store 驱动。
- **Health Check**: `cargo check -p auto-lang` clean。

## 待澄清事项

- Windows NAT 端口保留段：8019 处于 Windows 保留段，自动使用 8319 作为 `back_port`，已在 `pac.at` 与各测试用例中固化。

