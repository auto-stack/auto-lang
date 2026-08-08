# Plan 401: AutoUI 示例升级 — 018-027 从单文件玩具到完整 App

> **状态（2026-08-08）**: 🟡 进行中。**018-book-reader 已完成**（playwright 10/10 全绿，干净态可复现）。019-027 待办。
> **分支**: `plan399/018-book-reader`（018；后续每个示例建议独立分支 `plan401/0NN-xxx`）。
> **动机**: 计划 399 §后续第 166 行“继续升级 018-027 为正规 App”。调研结论：016-027 全部是单文件静态玩具（无后端、散装变量、no-op handler），与 015-notes / 017-chat（完整 App + 后端 + playwright）差一个量级。本计划逐个把它们升级为对标生产级应用的完整示例。
> **与 Plan 399 的关系**: 399 是 codegen 基建（SSE 多事件 / a2r 根治 / 混合状态硬检查）；本计划是**纯示例升级**，不引入新 codegen 基建（升级过程中发现的 codegen bug 单独修复并在此记录）。

---

## 升级标准（“完整 App”的硬指标）

对齐 015-notes / 017-chat：
1. **多模块前端**（非单文件）：`app.at` + store + 子组件/pages，散装变量 → 强类型 model/store。
2. **强类型后端**（`api: "rust"`）：`src/back/{api.at, db.at}`，`#[api]` 端点委托 db.rs（命中 Plan 399 路线 B 全覆盖）。
3. **端到端验证**：`auto gen` + `auto run` + curl 后端 + playwright 测试套件（`tests/`，对齐 017-chat 的 package.json/playwright.config.ts/smoke.spec.ts/acceptance.atd 四件套）。
4. **README 更新**：反映新架构的 Concepts + Source + How to Run + Tests。

---

## §018-book-reader ✅ 已完成（2026-08-08）

从单文件静态玩具（散装 `ch1/ch2/ch3`、无后端、no-op handler）升级为首个**「多路由 + 强类型 rust 后端 API」融合**示例，对标生产级阅读器（参考工程 `D:\code\vue\auto-read`，Apple Books 风格）。

### 架构（融合 auto-read 的 routes + 015/017 的 CRUD）

```
src/front/
  app.at              # App 路由壳：routes{4 条} + 侧边栏 nav + <outlet>
  book_store.at       # BooksStore：books 集合 + 进度（跨页共享 store）
  pages/
    bookshelf.at      # / → Library 网格 + 添加/删除
    book_detail.at    # /book/:id → 封面/章节列表/继续阅读
    reading.at        # /book/:id/chapter/:ch → 正文 + 上下章 + 进度持久化
    settings.at       # /settings → 字号 + 主题说明
src/back/
  api.at              # pub type Book/Chapter + 8 个 #[api] 端点（CRUD + 进度）
  db.at               # 强类型内存存储 + 3 本种子书 × 3 章
vue/
  src/components/ThemeToggle.vue   # handmade 主题切换（escape hatch）
  src/assets/index.css             # 覆盖固定 <html class="dark"> → 默认 light
tests/               # playwright T1-T10
```

### 功能（对标 auto-read 的核心 5 页面；笔记/统计/复习后置）
- 书架 Library（网格 + 进度条 + 添加/删除书）
- 书籍详情（封面 + 元信息 + 章节列表 + 继续阅读）
- 阅读视图（章节正文 + 上一章/下一章 + 进度持久化）
- 设置（字号 + 主题）
- 暗色**运行时切换**（handmade ThemeToggle.vue，auto-read 招牌功能）

### 过程中修复 3 个 codegen bug（阻塞本示例，根治通用，已验证 k1/017 无回归）

1. **路由 useRoute 缺失**（`ui_gen/vue.rs` + `ui_gen/ts_adapter.rs`）：
   lifecycle handler（`.Init`）里的 `router.param("id")` 未触发 `needs_route` → 生成的 `.vue` 不 import `useRoute` → `(useRoute().params...)` undefined 崩页。
   修复：(a) `stmts_have_route_access`（ts_adapter.rs）增加 `router.param/query/path` 调用识别；(b) vue.rs 的 needs_route 检测扩展到 `widget.lifecycle`（之前只查 `widget.handlers`，漏了 `.Init`）。

2. **无路径参数 GET URL 多余引号**（`api/targets/typescript.rs:340`）：
   路径无 `:param` 时 url 用单引号 `'...'`，但 GET query 拼接分支按反引号 trim（`trim_start_matches('`')`）→ 单引号漏进字面，URL 成 `` `'/api/chapters'?...` `` → 404。
   修复：无参分支也用反引号 `format!("`{}`", path)`。

3. **a2r 返回位置 struct literal**（db.at 源码层规避，未改 a2r）：
   `return Chapter{...}` a2r 报 `undefined variable`（返回位置 struct literal 未注册类型）；改 `let x = Chapter{...}; return x` 规避（a2r `let` 注册类型）。同源另有：reassignment `found = Chapter{...}` 解析失败（改 `let rebuilt = ...; found = rebuilt`）、借用迭代变量字段 move（`ch.title` → let-ctor 内访问触发 a2r clone）。均在 db.at 源码层规避并注释。

### 关键技术约定（019-027 复用）
- **路由参数是字符串**：`router.param("id")` 返回 str，做算术前必须 `.to_int()`（否则 `"1" + 1 = "11"` 字符串拼接）。
- **多语句 computed 不可用**：vue codegen 不为 block-bodied computed 发 `return` → 恒 undefined。需派生值就存 model 字段、在 handler 里算。
- **`auto run` 增量重生成不重扫 routes**（缓存所致）：必须先 `auto gen` 保证 `router/index.ts` 存在，再 `auto run`。这是已知边际，记此备查。
- **handmade vue 件需 `.gitignore` 放行**：仓库根 `examples/**/vue/` + `*.json` 会吞掉 `vue/` 和 `tests/package.json`。需 per-example `.gitignore` 加 `!vue/` + `!tests/package.json`（cf. examples/a3ui-replica/.gitignore）。
- **per-example 专属端口避免并发冲突**（Plan 401 新增）：多个示例同时测试时，默认 3000/8080 会互相抢占（vite 代理指错后端 → 前端收到 HTML 而非 JSON → `Unexpected token '<'`）。每个示例在 pac.at 声明专属端口：`front_port: 30NN` / `back_port: 80NN`（NN = 示例号）。`auto run`/`auto build` 读 pac.at 作为 `-F`/`-B` 的默认值（CLI 优先）。018 用 3018/8018。playwright `baseURL` 需同步改成专属前端端口。实现见 `pac.rs`（front_port/back_port 字段）+ `automan.rs`（`pac_dev_ports()` getter）+ `main.rs`（Run/Build 端口注入块 CLI 优先兜底）。

### 验证流程（每示例通用）
`auto gen`（前端，含 routes）→ `auto run`（后端 axum + vite）→ curl 验证 `#[api]` 端点 → `cd tests && npm test`（playwright）。

### 018 验收
playwright **10/10 全绿**（干净态可复现）：T1 书架渲染 / T2 进度信息 / T3 进详情 / T4 章节列表 / T5 进阅读 / T6 下一章导航 / T7 进度持久化 / T8 添加书 / T9 暗色运行时切换 / T10 控制台无错。

### 不做（明确后置）
- 笔记系统（auto-read 的 notes/review，下批）
- 阅读统计（stats，下批）
- Tauri 文件导入 / SQLite（auto-read 的桌面能力，超出 web 示例定位）
- sepia 第三主题（先 light/dark，sepia 留扩展）
- vm/rust 前端版（Plan 399 §后续已列，全示例统一后置）

---

## §019-027 待办（候选优先级）

| 示例 | 现状 | 升级方向 | 优先级 |
|---|---|---|---|
| 019-video-app | 单文件静态 | 视频列表 + 播放历史后端 | 中 |
| 020-music-player | 单文件静态 | 播放列表 + 喜欢/最近播放后端 | 中 |
| 021-blog-viewer | 单文件静态 | 文章列表 + 评论后端 | 中 |
| 022-kanban | 单文件静态 | 看板 CRUD + 拖拽（Plan 399 §后续点名候选） | 高 |
| 023-realworld | 单文件静态 | RealWorld 精简（文章/评论/关注，Plan 399 §后续点名候选） | 高 |
| 024-widget-gallery | 单文件静态 | 组件目录（展示性，可能无需后端） | 低 |
| 025-notes-extended | 已较完整 | 补 rust 后端 + playwright 对齐 | 低 |
| 026-027 | 能力展示示例 | 保留（非 App 性质，可能不升级） | — |

**推荐下一批**：022-kanban（CRUD + 拖拽，复用 018 的路由/store/后端范式，且 Plan 399 点名）。

---

## 提交历史

| commit/分支 | 内容 | 示例 |
|---|---|---|
| `plan399/018-book-reader` | 018 升级 + 3 codegen 修复 + playwright 10/10 | 018 |
