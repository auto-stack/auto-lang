# Plan 405: 023-realworld 升级 — 完整 Conduit spec（分两阶段：vue 原型 → auto 复刻）

> **纲领**: 遵循 [Plan 401](401-autoui-examples-upgrade.md) 硬指标 + 技术约定。本计划是 401 进度总表里 023-realworld 的子计划。
> **技能**: 使用 `auto-ui-creator` 技能（`D:/autostack/skills/auto-ui-creator/`，含 25 条 Gotcha + Vue→AutoUI 映射 + 模板 + Toy→Real 重构模式 R1-R4）。
> **状态（2026-08-09）**: ✅ 阶段 1 完成（vue 原型 + auto 复刻：认证 + feed + 文章详情，playwright 8/8 全绿）。阶段 2 待后续计划。
> **分支/worktree**: `plan401/023-realworld`（`.worktree/plan401-023-realworld`）
> **动机**: 023 现为 227 行单文件玩具（散装 art1_title/art2_author、current_view 字符串切视图、无后端、无交互）。升级为对标 RealWorld (Conduit) 完整 spec 的真实 App。
> **参考**: RealWorld 官方 spec（19 端点 + 数据模型 + 7 页面 + JWT 认证，`Authorization: Token <jwt>` 前缀是 `Token` 非 `Bearer`）。

## 设计决策

- **分两阶段交付**：
  - **阶段 1（本计划范围）**: vue 原型 + auto 复刻 —— 认证（登录/注册/设置）+ 首页 feed（全局/标签过滤）+ 文章详情（只读 + 评论列表）。playwright 全绿即交付。
  - **阶段 2（后续计划）**: 文章 CRUD（编辑器）+ 发评论/删评论 + 关注/取关 + 个人资料页（我的文章/收藏）。
- **先 vue 原型再 auto 复刻**: Vue3 + Vite + Tailwind + shadcn-vue（与 auto codegen 产物同栈），验证交互/样式设计，再逐页面用 auto 复刻。原型放 `examples/ui/023-realworld/vue-ref/`（参考用，不进 codegen 流程）。
- **认证建模**: JWT token 存 localStorage（RealWorld 标准做法）。后端 `var sessions` 内存存 token→user 映射（auto `#[api]` 无认证中间件先例，需验证端点如何拿 token——见"待验证 codegen"）。
- **数据模型降级**: RealWorld spec 的 wrapper 响应（`{article:{...}}`）与 auto codegen 的扁平返回不直接兼容。auto 复刻时**返回扁平结构**（`{slug,title,...}` 而非 `{article:{...}}`），前端 store 直接消费。这是对 RealWorld spec 的合理简化（auto App 是自包含的，不追求跨实现互操作）。

## 目标架构（阶段 1）

```
examples/ui/023-realworld/
  pac.at                      # +api:"rust" +front_port:3023 +back_port:8023
  vue-ref/                    # vue3+shadcn-vue 原型（参考用，.gitignore 放行）
    src/{views,components,stores,api}  # 登录/注册/首页/文章详情 + settings
  src/front/
    app.at                    # 路由壳: routes{/login /register / /article/:slug /settings} + nav + outlet
    auth_store.at             # AuthStore: currentUser/token, login/register/logout
    article_store.at          # ArticleStore: articles/tags/loading, feed/filter
    pages/
      home.at                 # / → 全局 feed + 标签过滤
      login.at                # /login → 登录表单
      register.at             # /register → 注册表单
      article_detail.at       # /article/:slug → 文章 + 评论列表（只读）
      settings.at             # /settings → 改资料 + 登出
  src/back/
    api.at                    # pub type User/Article/Comment + #[api] 端点（阶段1子集）
    db.at                     # 强类型内存存储 + 种子（用户 + 3-5 篇文章 + 评论）
  tests/                      # playwright 四件套（端口 3023）
  .gitignore                  # 放行 vue-ref/ + tests/package.json
```

## 阶段 1 数据模型（src/back/api.at）

```auto
pub type User = { id: int, email: str, username: str, bio: str, image: str, token: str }
pub type Article = { slug: str, title: str, description: str, body: str, tagList: str, author: str, favoritesCount: int }
pub type Comment = { id: int, body: str, author: str }
```
> **简化**: `tagList` 用 `str`（逗号分隔）而非 `[]str`（auto codegen 对字符串数组的支持需验证，先保守）。`createdAt` 等时间戳阶段1先用固定种子字符串。`favorited`/`following` 布尔字段阶段2加（依赖认证态）。

## 阶段 1 API 端点（src/back/api.at 子集）

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/users/login` | 登录，返 User(含 token) |
| POST | `/api/users` | 注册，返 User(含 token) |
| GET | `/api/user` | 当前用户（需 token） |
| GET | `/api/articles` | 文章列表（query: tag/author） |
| GET | `/api/articles/:slug` | 单篇文章 |
| GET | `/api/articles/:slug/comments` | 评论列表 |
| GET | `/api/tags` | 标签列表 |

> 阶段2补：PUT /user、文章 CRUD、评论 POST/DELETE、关注、收藏。

## 待验证的 codegen 能力（阶段 1 复刻时解决，遇坑就修）

1. **认证：端点如何拿 token**：auto `#[api]` codegen 当前无认证中间件先例（018/022 都是无认证 CRUD）。需探索生成的 axum handler 能否读请求头。降级方案：阶段1后端不强制鉴权，前端用 token 判登录态，后端宽松。
2. **wrapper 响应**：已决定降级为扁平。
3. **tagList 字符串数组**：先用 str，验证 `[]str` 后再升。
4. **slug 生成**：后端 db.at 用简单 title→slug 转换（空格转连字符 + 小写），种子文章预置 slug。

## 执行步骤

### 阶段 1.1: vue 原型（vue-ref/）
1. 用 Vite 创建 `vue-ref/`（Vue3 + TS + Tailwind + shadcn-vue）
2. 实现阶段1范围页面：login/register/home(feed+tag)/article_detail(含评论) + settings
3. 原型用 mock 数据（前端硬编码用户/文章），验证交互/样式
4. 浏览器确认原型符合 RealWorld 视觉 + 交互预期

### 阶段 1.2: auto 复刻（src/）
1. **后端先行**：写 `src/back/{api.at, db.at}`（种子 3-5 篇文章 + 2 用户 + 评论）→ `auto gen` → curl 验证 7 端点
2. **前端**：写 `src/front/{app.at, auth_store.at, article_store.at, pages/*.at}`，套用 SKILL.md 范式（U1 store 访问、U5 无 v-model、U10 无 vue 指令、R1 散装→typed list、R4 设计 token）→ `auto gen`
3. **遇 codegen 坑就地修**
4. **测试**：tests/ 四件套（端口 3023），playwright T1-T8

### 阶段 1.3: 验证
- `auto gen` + 启动 + curl 7 端点全绿
- playwright 阶段1用例全绿
- README 更新（含 vue-ref 说明 + 端口缺口启动）

## 阶段 1 playwright 用例（初拟）

- T1 首页渲染（全局 feed 显示种子文章）
- T2 标签过滤（点标签 → feed 按标签过滤）
- T3 进文章详情（点标题 → 详情页 + 正文 + 评论列表）
- T4 注册（填表 → 成功 → 回首页，nav 显示用户名）
- T5 登录（已有用户 → 成功）
- T6 设置页（改资料 + 登出）
- T7 未登录访问受保护页跳转登录
- T8 控制台无错

## 不做（明确后置 = 阶段 2）

- 文章 CRUD（编辑器新建/编辑/删除）
- 发评论/删评论
- 关注/取关用户
- 收藏/取消收藏文章
- 个人 feed（/articles/feed）
- 个人资料页的 Favorited Articles tab
- 分页（limit/offset）
- markdown 渲染（正文纯文本显示）

## 风险与回退
- **风险1**: 认证 codegen 缺口大。回退：阶段1后端不强制鉴权，前端用 token 判登录态。记遗留，阶段2补。
- **风险2**: 工程量超预期。回退：阶段1再拆，先做"只读 feed + 详情"（无认证），认证后置。
- **vue 原型价值兜底**: 即使 auto 复刻受阻，vue 原型本身也是有价值的交互参考。

## 验证清单（阶段 1）
- [x] vue-ref/ 原型可运行，5 视图(home/login/register/article/settings)渲染验证
- [x] 后端 `cargo build` + curl 6 端点正确（list_tags 移到客户端派生）
- [x] `auto gen` + 启动 + 前端正常加载（hash 路由 5 条）
- [x] **playwright 8/8 全绿**（2026-08-09：T1 feed / T2 标签过滤 / T3 详情+评论 / T4 注册 / T5 登录 / T6 设置+登出 / T7 未登录提示 / T8 无错）
- [x] README 更新
- [x] 401 进度总表 023 状态更新

### 阶段 1 过程中修复的 codegen 问题（4 个，影响后续示例）
1. **`tag` 是保留软关键字**（TokenKind::Tag，enum 别名）→ 不能作参数名（a2r "expected '{' or type after 'str'"）。用 `filter_tag` 规避。
2. **a2r_std import 路径错**（trans/rust.rs:17746）→ `use a2r_std` 改 `use auto_lang::a2r_std`（a2r_std 是 auto_lang 模块，非独立 crate）。
3. **back Cargo.toml 缺 auto-lang 依赖**（api_gen.rs generate_cargo_toml）→ has_db 时加 `auto-lang.workspace = true`。
4. **store model 的 struct 字面量初始值退化 null**（未修 codegen，.at 侧用 `!= nil` 判断规避）—— 记入 401 技术约定，留后续根治。

### 待补（阶段 2 或独立计划）
- store struct 字面量初始值 codegen 修复（根治 #4）
- 真正的 token 认证（current_user 读 token，端点鉴权）
- 文章 CRUD / 评论 POST-DELETE / 关注 / 收藏 / 资料 / 分页
