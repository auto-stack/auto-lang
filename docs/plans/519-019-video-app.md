---
plan_id: PLAN-519
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: 019-video-app 升级为完整 App
author: [zhaopuming]
created_at: 2026-09-02
updated_at: 2026-09-02

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 示例升级不预定改 specs；过程性 codegen 修复按实际改动补记
current_step: 0
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
- 前端拆多模块：`app.at` 路由壳 + `video_store.at` 共享 store + `pages/home.at`（首页网格）+ `pages/watch.at`（观看页）。
- 端到端验证：`auto gen` + `auto run` + curl 后端 + playwright 四件套（对齐 018）。
- README 更新为新架构。

## 目标

1. **分类 chips 真过滤**：点击 Gaming/Music/Tech/Food 分类 chip，网格只显示该分类视频（后端过滤，非前端隐藏）。
2. **Tab 切内容源**：Recommend（默认序）/ Trending（按 views 降序）/ Following（仅关注作者的视频）。
3. **搜索真过滤**：顶部搜索框对 title/author 做子串匹配（不区分大小写）。
4. **观看页**：`/watch/:id` 显示视频详情（标题/作者/播放量/点赞/简介）+ 相关推荐（同分类其余视频）。
5. **交互计数**：进入观看页 `POST /api/videos/:id/view` 播放量 +1；点赞按钮 `POST /api/videos/:id/like` 点赞 +1（可再点取消）。
6. **playwright 全绿**：覆盖上述 5 条的验收用例全部通过（干净态可复现）。

## 架构方案

### 目录结构（对齐 018 范式）

```
examples/ui/019-video-app/
  pac.at                    # api:"rust", front_port:3019, back_port:8019
  .gitignore                # !vue/ + !tests/package.json 放行（纲领约定）
  src/front/
    app.at                  # 路由壳：routes{"/" -> home, "/watch/:id" -> watch} + 顶栏 + outlet
    video_store.at          # VideoStore：videos/category/tab/query/current 共享 store（k1 模式）
    pages/home.at           # chips 行 + tabs + 搜索框 + grid（for video in .store.videos）
    pages/watch.at          # 视频详情 + 点赞 + 相关推荐列表
  src/back/
    api.at                  # pub type Video + #[api] 端点，委托 db
    db.at                   # 强类型内存存储 + 种子数据（~12 条，覆盖 5 分类）
  tests/                    # playwright 四件套（package.json / playwright.config.ts / smoke.spec.ts / acceptance.atd）
```

### 后端设计

```auto
pub type Video = {
    id: int
    title: str
    author: str
    category: str        // All 之外的 5 个分类：Gaming/Music/Tech/Food/Travel
    duration: str        // "12:34" 展示格式
    views: int
    likes: int
    desc: str
    followed: bool       // 作者是否被关注（Following tab 数据源）
}
```

端点（全部委托 db，命中 Plan 399 路线 B 全覆盖）：

| 端点 | 语义 |
|---|---|
| `GET /api/videos?category=&tab=&q=` | 列表：category 过滤 + tab 排序/过滤 + q 子串搜索 |
| `GET /api/videos/:id` | 单条详情（`?Video`，未找到返回 null） |
| `POST /api/videos/:id/view` | 播放量 +1，返回新 views |
| `POST /api/videos/:id/like` | 点赞切换（like→unlike），返回新 likes |
| `GET /api/categories` | 分类列表（chips 数据源） |

**query 参数备注**：codegen 的 GET query 拼接路径已在 018 修过引号 bug，预期可用；
若实测发现 query 参数 a2r/codegen 缺口，降级方案是改为语义化路径端点
（如 `/api/videos/trending`），修复记录回写纲领 §提交历史。

### 前端设计

- **VideoStore 字段**：`videos List<Video>`、`category str`、`tab str`、`query str`、
  `current ?Video`、`related List<Video>`。字段名避开 api 函数名（纲领 023 gotcha：
  如不叫 `list_videos`）。
- **派生值不写 computed**（纲领：多语句 computed 不可用）：格式化计数（如 views 显示）
  直接用后端返回的 str 字段或存 model 字段在 handler 里算。
- **grid 过滤即重新拉取**：chips/tabs/搜索任一变化 → handler 调 `list_videos(...)`
  重灌 `.store.videos`，让 for 体直接是 Element（规避 R006 for+if :key warning）。
- **路由参数**：`/watch/:id` 的 `router.param("id")` 是 str，算术前 `.to_int()`（纲领约定）。
- **播放计数触发**：watch 页 `.Init` lifecycle 里 `view_video(id)`（对齐 018 的 Init 拉取模式）。

### 技术栈

`.at` 前端（vue codegen 主验证路径）+ a2r Rust 后端 + playwright（chromium）。
vm/rust 前端版后置（纲领多版本策略；多 store VM 链接 bug 未修，本例单 store 不受影响）。

## 需求分析与背景调查

（取材 [docs/specs/overview.md](../specs/overview.md) 与 Plan 401 纲领）

- **现状**：019 是 Plan 401 进度总表仅剩的三个未升级项之一（019/020/021，2026-09-02
  裁定 019/020 维持 App 轨立项）。135 行单文件，无后端、无路由、无真实交互。
- **范式成熟度**：018（10/10）/022（6/6）/023（14/14）已验证"直接写 .at → codegen →
  playwright"五步流程与完整 App 四硬指标可行；`ui` 模块（ui_gen/vue.rs）经 018/022/023
  三轮示例反哺已修复 useRoute 缺失、无参 GET 引号、row/col 属性穿透等通用 bug。
- **API 基建**：`api: "rust"` 路线（auto-cli `auto gen`/`auto run` + api_gen.rs +
  a2r 转译）已有 018/022 两个生产级先例；`auto run` 已注入 pac.at 端口
  （commit e865566e），无需再走环境变量分离启动。
- **widget 能力**：019 现用的 `tabs`/`grid`/`input` 均为一等 widget；搜索双向绑定
  与 chips 点击态切换在 022（列过滤）/017（聊天输入）有先例。

## 详细设计

### 种子数据（db.at）

12 条视频，5 分类各 2-3 条；字段覆盖：不同 views 量级（构造 Trending 排序可断言）、
2 个 `followed: true` 作者（构造 Following 非空且可断言）、title 含可搜索子串
（如两条含 "Rust"）。

### 关键 handler 流

- `home.at` `.Init` → 并行拉 `categories()` + `list_videos("", "recommend", "")`。
- chip 点击 `CategoryChanged(category str)` → 置 `.store.category` → 重拉列表 →
  chip 高亮态由 `category == .store.category` 决定（点击态存 model，不用 computed）。
- tab 切换 `TabChanged(tab str)` → 置 `.store.tab` → 重拉列表。
- 搜索 `SearchSubmitted` → 置 `.store.query` → 重拉列表。
- 卡片点击 → `router.push("/watch/" + id.to_str())`。
- `watch.at` `.Init` → `router.param("id").to_int()` → `get_video(id)` + `view_video(id)` +
  `related = list_videos(category, "recommend", "")` 过滤掉自身。
- 点赞 `LikeToggled` → `like_video(id)` → 更新 `.store.current.likes`。

### a2r 已知规避（写源码时直接套用）

- `return T{...}` 改 `let x = T{...}; return x`；reassignment 同理。
- 函数体内 `List<T>` 声明放在 `var result T{...}` 之前。
- 不用 `tag` 作参数名（保留字）→ 用 `category`。
- db 过滤列表实现参照 018 `db.at` 的 filter+push 范式。

## 测试设计

**curl 冒烟**（`auto run` 起服务后）：

```bash
curl -s http://localhost:8019/api/videos | head -c 400          # 全量列表
curl -s "http://localhost:8019/api/videos?category=Music"       # 分类过滤
curl -s "http://localhost:8019/api/videos?tab=trending"         # views 降序
curl -s "http://localhost:8019/api/videos?q=rust"               # 搜索（不区分大小写）
curl -s -X POST http://localhost:8019/api/videos/1/like         # 点赞 +1
```

**playwright 用例**（`tests/smoke.spec.ts`，baseURL `http://localhost:3019`）：

1. 首页加载：标题 VideoApp + 网格 ≥ 6 张卡。
2. 分类过滤：点 Gaming chip → 网格只剩 Gaming 卡。
3. Tab 切换：Trending → 首卡为种子数据 views 最高的视频。
4. 搜索：输入关键字 → 网格只剩匹配项；清空恢复。
5. 观看页：点首卡 → URL `/watch/:id`、标题/作者匹配、播放量较卡片 +1。
6. 点赞：点赞按钮点击 → 数字 +1；再点 → 回落。
7. 相关推荐：观看页列表不含自身、均属同分类。

`tests/acceptance.atd` 记录同等断言的自然语言验收脚本（对齐 018 四件套）。

## 验收标准

1. Plan 401 四硬指标全满足：多模块前端 / 强类型后端 / 端到端验证 / README 更新。
2. playwright 全绿（目标 7/7，干净态复跑两次验证无状态残留）。
3. curl 冒烟五条全部返回预期 JSON。
4. `auto check`（或 `auto gen`）无 error；codegen 修复（若有）单独成 commit 并回写纲领。
5. 无临时调试打印、无未说明的 workaround；所有 escape hatch 在 README 与纲领注明原因。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **pac.at 端口与后端声明**：`examples/ui/019-video-app/pac.at` 追加 `api: "rust"`、
   `front_port: 3019`、`back_port: 8019`。验证：`cat pac.at` 三行齐。
2. **新增 `.gitignore`**：`examples/ui/019-video-app/.gitignore`，内容对齐 018
   （`!vue/` + `!tests/package.json` + 生成物忽略）。验证：`git status` 确认后续
   handmade/测试文件不被吞。
3. **后端数据层**：新建 `examples/ui/019-video-app/src/back/db.at`：`Video` 类型内存表 +
   12 条种子 + `all_videos/filter_videos/find_video/add_view/toggle_like/categories`。
   验证：`cargo check -p auto-lang`（语法门禁）。
4. **后端 API 层**：新建 `examples/ui/019-video-app/src/back/api.at`：5 个 `#[api]` 端点
   委托 db。验证：`auto gen` 生成 rust workspace 无报错。
5. **前端 store**：新建 `examples/ui/019-video-app/src/front/video_store.at`（VideoStore：
   videos/category/tab/query/current/related）。验证：`cargo check -p auto-lang`。
6. **首页页**：重写 `examples/ui/019-video-app/src/front/pages/home.at`（chips + tabs +
   搜索 + grid + 三类 handler）。验证：`auto gen` 后 `gen/front/vue` 存在 home 组件。
7. **观看页**：新建 `examples/ui/019-video-app/src/front/pages/watch.at`（详情 + 点赞 +
   相关推荐）。验证：同上。
8. **路由壳**：重写 `examples/ui/019-video-app/src/front/app.at`（routes 2 条 + 顶栏 +
   outlet），删除原散装 model。验证：`auto gen` 后 `router/index.ts` 含两条路由；
   `auto run` 双服务起、页面可开。
9. **测试四件套**：新建 `examples/ui/019-video-app/tests/{package.json,
   playwright.config.ts,smoke.spec.ts,acceptance.atd}`（config 抄 018 改 baseURL 3019）。
   验证：`cd tests && pnpm install && pnpm exec playwright test` 全绿。
10. **README 与纲领回写**：重写 `examples/ui/019-video-app/README.md`（Concepts/Source/
    How to Run/Tests 四段反映新架构）；Plan 401 总表 019 行翻 ✅ + 提交历史补一行。
    验证：`grep -c "019" docs/plans/401-autoui-examples-upgrade.md` 命中刷新行。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- GET query 参数（`?category=&tab=&q=`）在 api client codegen 的支持度未实测；
  备选语义化路径端点方案已备（见架构方案备注），执行时按实测结果二选一并记录。
