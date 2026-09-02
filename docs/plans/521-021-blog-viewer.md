---
plan_id: PLAN-521
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 021-blog-viewer 升级为完整 App
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

# [PLAN-521] 021-blog-viewer：Medium 式博客阅读器升级为完整 App

> **纲领引用**：本计划是 [Plan 401](401-autoui-examples-upgrade.md) 的子计划，
> 硬指标（"完整 App"四条）、五步流程与全部技术约定以纲领 §升级标准 / §流程约定 /
> §技术约定 为准，此处不重述。**2026-09-02 裁定**：021 维持 App 轨立项（解除 ⏸ 待裁定）。
> 本计划是 Plan 401 进度总表**最后一个**未升级示例（019→519 / 020→520 / 021→521）。

## 变更摘要

把 `examples/ui/021-blog-viewer` 从 89 行单文件静态玩具（`blog1..blog3` 散装变量、
SelectArticle 只翻 `view_mode` 标志而详情栏恒显 blog1 内容、无后端）升级为对标
018-book-reader 的完整 App：

- 新增强类型 Rust 后端（`src/back/{api.at, db.at}`）：文章库列表/详情/标签过滤/
  发布/删除/点赞（Medium 式 clap）。
- 前端拆多模块：`app.at` 路由壳 + `blog_store.at` 共享 store + 三路由页
  （首页列表 / 文章详情 / 写作页）。
- 端到端验证：`auto gen` + `auto run` + curl 后端 + playwright 四件套（对齐 018）。
- README 更新为新架构。

## 目标

1. **列表真渲染**：左栏文章卡（标题/作者/日期/摘录）来自后端文章表（种子 ≥6 篇），
   顶栏计数动态显示；点卡片进入详情（非视图切换）。
2. **标签过滤**：顶部标签 chips（如 Rust/AutoUI/WASM/All），点击过滤列表（后端过滤）。
3. **详情页**：`/post/:id` 显示完整正文 + 元信息 + clap 点赞（计数 +1，可再点回落）。
4. **发布流**：`/new` 写作页（标题/作者/标签/摘录/正文表单）→ `POST /api/posts` →
   跳回列表且新文章置顶；详情页可删除文章（确认后回列表）。
5. **计数一致性**：clap 与发布/删除在列表与详情两处联动（共享 store 单一数据源）。
6. **playwright 全绿**：覆盖上述 5 条的验收用例全部通过（干净态可复现）。

## 架构方案

### 目录结构（对齐 018 范式）

```
examples/ui/021-blog-viewer/
  pac.at                    # api:"rust", front_port:3021, back_port:8021
  .gitignore                # !vue/ + !tests/package.json 放行（纲领约定）
  src/front/
    app.at                  # 路由壳：routes{"/" -> home, "/post/:id" -> detail, "/new" -> editor} + 顶栏 + outlet
    blog_store.at           # BlogStore：articles/tag/current 共享 store（k1 模式）
    pages/home.at           # 标签 chips + 文章卡列表 + "Write" 入口按钮
    pages/detail.at         # 正文 + 元信息 + clap + 删除确认
    pages/editor.at         # 发布表单（title/author/tag/summary/body）
  src/back/
    api.at                  # pub type Article + #[api] 端点，委托 db
    db.at                   # 强类型内存存储 + 种子数据（≥6 篇，覆盖 3+ 标签）
  tests/                    # playwright 四件套（package.json / playwright.config.ts / smoke.spec.ts / acceptance.atd）
```

### 后端设计

```auto
pub type Article = {
    id: int
    title: str
    author: str
    date: str           // "Apr 15, 2026" 展示格式（发布时由后端生成当日）
    tag: str            // 单标签（All 之外：Rust/AutoUI/WASM…）
    summary: str
    body: str
    claps: int
}
```

端点（全部委托 db，命中 Plan 399 路线 B 全覆盖）：

| 端点 | 语义 |
|---|---|
| `GET /api/posts?tag=` | 列表：tag 过滤（空 = 全量），按 id 降序（新文置顶） |
| `GET /api/posts/:id` | 单篇详情（`?Article`，未找到返回 null） |
| `POST /api/posts` | 发布（body：title/author/tag/summary/body），自增 id + 当日日期 |
| `DELETE /api/posts/:id` | 删除，返回 bool |
| `POST /api/posts/:id/clap` | clap 切换（+1/回落），返回新 claps |

- query 参数复用 Plan 519 同一决策：实测 `?tag=` 缺口则降级语义化路径
  `/api/posts/tag/:tag`，取舍记录回写两份计划。
- 端点函数名用 `posts` 域（`list_posts/create_post/...`），与 store 字段 `articles`
  无同名冲突（纲领 023 gotcha）。

### 前端设计

- **BlogStore 字段**：`articles List<Article>`、`tag str`（当前过滤，空 = All）、
  `current ?Article`。
- **路由**：`/post/:id` 的 `router.param("id")` 是 str，算术前 `.to_int()`（纲领约定）；
  同一 widget 不映射多条路由（023 gotcha）——editor 独立 `/new`。
- **删除确认**：详情页删除按钮 → 二次确认（条件渲染确认条，对齐 022 删除确认范式），
  确认后 `delete_post` + `router.push("/")`。
- **发布校验**：editor handler 内校验 title/body 非空（空则置 model 错误文本，
  条件渲染显示），通过后 `create_post` → 跳回列表。
- **clap 联动**：详情 clap 后同步更新 `.store.current.claps` 与列表内对应项
  （handler 内遍历替换，非 computed）。
- **正文多段**：body 用 `\n\n` 分段，展示侧按段拆分循环渲染（for over 派生
  `List<str>`——在 handler 中拆好存 model 字段，规避多语句 computed 限制）。

### 技术栈

`.at` 前端（vue codegen 主验证路径）+ a2r Rust 后端 + playwright（chromium）。
vm/rust 前端版后置（纲领多版本策略；本例单 store，VM 多 store bug 已修亦不受影响）。

## 需求分析与背景调查

（取材 [docs/specs/overview.md](../specs/overview.md) 与 Plan 401 纲领）

- **现状**：021 是 Plan 401 进度总表**最后一个**未升级示例（019/020 已于 2026-09-02
  拆出 Plan 519/520）。89 行单文件；现行"详情栏"把 blog2/blog3 的 body 串在 blog1
  正文下面，SelectArticle/BackToList 只翻一个永不参与渲染的 `view_mode` 标志。
- **范式成熟度**：018（10/10）/022（6/6）/023（14/14）已验证五步流程与四硬指标；
  本例与 018 同构度最高（列表 + 详情 + 路由 + CRUD），风险最低。
- **表单先例**：发布表单对齐 010-contact-form（提交反馈）/005-login 的 input 受控
  模式；条件渲染确认条对齐 022 删除确认。
- **API 基建**：`api: "rust"` 路线有 018/022 生产级先例；`auto run` 已注入 pac.at
  端口（commit e865566e）。

## 详细设计

### 种子数据（db.at）

6 篇文章：沿用现例 3 篇（Rust 入门/AutoUI 声明式 widget/WASM 2026）+ 补 3 篇
（标签覆盖 Rust×2 / AutoUI×2 / WASM×2）；每篇 body 3-5 段（验证分段渲染）；
2 篇 claps 基数 >0；日期交错（列表按 id 降序与日期序一致，便于断言）。

### 关键 handler 流

- `home.at` `.Init` → `list_posts(.store.tag)` 灌 `.store.articles`。
- chip 点击 `TagChanged(tag str)` → 置 `.store.tag` → 重拉列表；chips 高亮态由
  `tag == .store.tag` 决定。
- 卡片点击 → `router.push("/post/" + id.to_str())`。
- `detail.at` `.Init` → `get_post(id)` 灌 `current` + 拆段；clap → `clap_post(id)` →
  更新 `current.claps`；删除两步 → `delete_post(id)` → `router.push("/")`。
- `editor.at` 提交 `PublishClicked` → 校验 → `create_post(...)` → `router.push("/")`
  （列表 Init 重拉后新文置顶）。

### a2r 已知规避（写源码时直接套用）

- `return T{...}` 改 `let x = T{...}; return x`；reassignment 同理。
- 函数体内 `List<T>` 声明放在 `var result T{...}` 之前。
- 不用 `tag` 作**参数名**（保留字）→ 端点参数用 `tag` 会撞吗：`tag` 是保留软关键字，
  **参数名禁用**——端点签名用 `category str` 语义不变，或前端传参字段名对齐调整
  （执行时以 parser 实测为准，首选 `category`）。
- store 字段名避开 api 函数名。

## 测试设计

**curl 冒烟**（`auto run` 起服务后）：

```bash
curl -s http://localhost:8021/api/posts | head -c 400          # 全量列表（id 降序）
curl -s "http://localhost:8021/api/posts?category=Rust"        # 标签过滤
curl -s http://localhost:8021/api/posts/1                      # 单篇详情
curl -s -X POST http://localhost:8021/api/posts/1/clap         # clap +1
curl -s -X POST http://localhost:8021/api/posts -d '{"title":"T","author":"A","category":"Rust","summary":"S","body":"B"}'  # 发布
curl -s -X DELETE http://localhost:8021/api/posts/2            # 删除
```

**playwright 用例**（`tests/smoke.spec.ts`，baseURL `http://localhost:3021`）：

1. 首页加载：顶栏 "My Blog" + 计数 "6 articles" + 左栏 6 张卡。
2. 标签过滤：点 Rust chip → 只剩 2 张 Rust 卡；点 All 恢复 6 张。
3. 进详情：点首卡 → URL `/post/:id`、标题/作者/正文与卡片一致、正文分段 ≥3 段。
4. clap：点赞 → 计数 +1；再点 → 回落。
5. 发布：顶栏 Write → `/new` 表单填写 → Publish → 回列表且新卡置顶、计数变 7。
6. 发布校验：空标题提交 → 错误提示出现、不跳转。
7. 删除：详情页删除 → 确认条 → 确认 → 回列表、卡片消失、计数回落。

`tests/acceptance.atd` 记录同等断言的自然语言验收脚本（对齐 018 四件套）。

## 验收标准

1. Plan 401 四硬指标全满足：多模块前端 / 强类型后端 / 端到端验证 / README 更新。
2. playwright 全绿（目标 7/7，干净态复跑两次验证无状态残留）。
3. curl 冒烟六条全部返回预期 JSON。
4. `auto check`（或 `auto gen`）无 error；codegen 修复（若有）单独成 commit 并回写纲领。
5. 无临时调试打印、无未说明的 workaround；所有 escape hatch 在 README 与纲领注明原因。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **pac.at 端口与后端声明**：`examples/ui/021-blog-viewer/pac.at` 追加 `api: "rust"`、
   `front_port: 3021`、`back_port: 8021`。验证：`cat pac.at` 三行齐。
2. **新增 `.gitignore`**：`examples/ui/021-blog-viewer/.gitignore`，内容对齐 018
   （`!vue/` + `!tests/package.json` + 生成物忽略）。验证：`git status` 确认
   tests/package.json 不被吞。
3. **后端数据层**：新建 `examples/ui/021-blog-viewer/src/back/db.at`：Article 内存表 +
   6 篇种子 + `filter_posts/find_post/create_post/delete_post/toggle_clap`。
   验证：`cargo check -p auto-lang`。
4. **后端 API 层**：新建 `examples/ui/021-blog-viewer/src/back/api.at`：5 个 `#[api]`
   端点委托 db（query 参数名避开保留字）。验证：`auto gen` 生成 rust workspace 无报错。
5. **前端 store**：新建 `examples/ui/021-blog-viewer/src/front/blog_store.at`
   （articles/category/current）。验证：`cargo check -p auto-lang`。
6. **首页页**：新建 `examples/ui/021-blog-viewer/src/front/pages/home.at`（chips +
   卡片列表 + Write 按钮 + TagChanged handler）。验证：`auto gen` 产物含 home 组件。
7. **详情页 + 写作页**：新建 `src/front/pages/{detail,editor}.at`（详情/clap/删除确认；
   表单/校验/发布）。验证：`auto gen` 产物含两页组件。
8. **路由壳**：重写 `examples/ui/021-blog-viewer/src/front/app.at`（routes 3 条 + 顶栏 +
   outlet + 计数），删除原散装 model。验证：`auto gen` 后 `router/index.ts` 含三条
   路由；`auto run` 双服务起、三页可 navigATE。
9. **测试四件套**：新建 `examples/ui/021-blog-viewer/tests/{package.json,
   playwright.config.ts,smoke.spec.ts,acceptance.atd}`（config 抄 018 改 baseURL 3021）。
   验证：`cd tests && pnpm install && pnpm exec playwright test` 全绿。
10. **README 与纲领回写**：重写 `examples/ui/021-blog-viewer/README.md`（Concepts/
    Source/How to Run/Tests）；Plan 401 总表 021 行翻 ✅ + 提交历史补一行 + 状态段
    收尾注记（018-027 全示例升级完成，纲领可议归档）。验证：
    `grep -n "021" docs/plans/401-autoui-examples-upgrade.md` 命中刷新行。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- GET query 参数（`?category=`）与 Plan 519 共用同一实测决策；降级路径已备。
- `tag` 保留字对参数名的实际禁用范围以 parser 实测为准（设计已首选 `category` 规避）。
