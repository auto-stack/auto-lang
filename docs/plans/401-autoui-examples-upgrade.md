# Plan 401: AutoUI 示例升级 — 018-027 从单文件玩具到完整 App

> **状态（2026-08-20 核查）**: 🟡 进行中。**401 为纲领计划**（定义标准 + 维护进度总表），每个示例的具体实现拆分为独立子计划。018/022（Plan 404）/023（Plan 405）已完成；011 已拆 Plan 403。待办：019/020/021/024/025（024 gallery 已由 [Plan 409](409-widgets-gallery.md) 以 `examples/widgets-gallery/` 形态完成，见总表注）。
> **分支**: 各示例独立分支 `plan401/0NN-xxx`（018 已合并 master）。
> **动机**: 计划 399 §后续第 166 行"继续升级 018-027 为正规 App"。调研结论：016-027 全部是单文件静态玩具（无后端、散装变量、no-op handler），与 015-notes / 017-chat（完整 App + 后端 + playwright）差一个量级。本计划逐个把它们升级为对标生产级应用的完整示例。
> **与 Plan 399 的关系**: 399 是 codegen 基建（SSE 多事件 / a2r 根治 / 混合状态硬检查）；本计划是**纯示例升级**，不引入新 codegen 基建（升级过程中发现的 codegen bug 单独修复并在此记录）。
> **组织约定（2026-08-09 确立）**: 018 是首个示例，建立标准；011 已拆为 Plan 403。**自 022 起，每个示例开独立子计划**（如 `404-022-kanban`），开头引用本纲领的硬指标 + 技术约定。本文件不承载单个示例的实现细节。

---

## 升级标准（"完整 App"的硬指标）

对齐 015-notes / 017-chat / 018-book-reader：
1. **多模块前端**（非单文件）：`app.at` + store + 子组件/pages，散装变量 → 强类型 model/store。
2. **强类型后端**（`api: "rust"`）：`src/back/{api.at, db.at}`，`#[api]` 端点委托 db.rs（命中 Plan 399 路线 B 全覆盖）。
3. **端到端验证**：`auto gen` + `auto run` + curl 后端 + playwright 测试套件（`tests/`，对齐 017-chat 的 package.json/playwright.config.ts/smoke.spec.ts/acceptance.atd 四件套）。
4. **README 更新**：反映新架构的 Concepts + Source + How to Run + Tests。

---

## 流程约定（018 验证可行的五步，子计划套用）

> ⚠️ **不采用"先手写 vue 参考、再翻译成 Auto"**。018 的实际流程是直接写 `.at` → codegen → 遇坑就地补 escape hatch。先 vue 再翻译会掩盖 codegen 真实能力、造成双倍维护。仅在"不确定 Auto 能否表达某交互、需先验证交互设计"时才做一次性 vue 原型（验证完丢弃，不进仓库）。

```
1. 直接写 .at（路由/store/后端，套用 018 范式）
2. auto gen → 生成 vue
3. playwright 跑通核心流程（先 CRUD，后增量功能）
4. 遇到 codegen 不支持的 → 二选一：
   (a) 修 codegen（018 一次修了 3 个，这是示例的红利）
   (b) 短期修不了 → 补 handmade vue 件（escape hatch，注明原因）
5. 全绿后 README + 子计划归档
```

**多版本策略**：vue 版是主验证路径，优先做。vm/rust 前端版统一后置（Plan 399 §后续已列）——待一批示例的 vue 版达到 018 水平、积累了足够多 codegen 边界 case 后，再系统性攻克 vm/rust 前端。

---

## 技术约定（019-027 子计划必须遵守）

以下均来自 018/022 的踩坑沉淀，子计划起草时在开头引用本节即可，无需重述。

- **路由参数是字符串**：`router.param("id")` 返回 str，做算术前必须 `.to_int()`（否则 `"1" + 1 = "11"` 字符串拼接）。
- **多语句 computed 不可用**：vue codegen 不为 block-bodied computed 发 `return` → 恒 undefined。需派生值就存 model 字段、在 handler 里算。
- **`auto run` 增量重生成不重扫 routes**（缓存所致）：必须先 `auto gen` 保证 `router/index.ts` 存在，再 `auto run`。这是已知边际，记此备查。
- **handmade vue 件需 `.gitignore` 放行**：仓库根 `examples/**/vue/` + `*.json` 会吞掉 `vue/` 和 `tests/package.json`。需 per-example `.gitignore` 加 `!vue/` + `!tests/package.json`（cf. examples/a3ui-replica/.gitignore）。
- **per-example 专属端口避免并发冲突**：多个示例同时测试时，默认 3000/8080 会互相抢占（vite 代理指错后端 → 前端收到 HTML 而非 JSON → `Unexpected token '<'`）。每个示例在 pac.at 声明专属端口：`front_port: 30NN` / `back_port: 80NN`（NN = 示例号）。playwright `baseURL` 需同步改成专属前端端口。
  - ✅ **端口缺口已修复（2026-08-20 核查更正）**：下述 08-09 记录的缺口实际已于 08-08 commit e865566e 修复——`main.rs:824` 的 `Commands::Run` 已调 `pac_dev_ports()` 注入环境变量，且 `-B`/`-F` CLI 参数已定义（main.rs:332-387）。原文留档：`pac.at` 的 `front_port`/`back_port` 已在 `pac.rs` 解析、`automan.rs` 有 `pac_dev_ports()` getter，但 **`auto run`（`run_vue`/`run_vue_project`）当前未调用它注入环境变量**，`main.rs` 也无 `-F`/`-B` CLI 参数定义。结果 `auto run` 仍默认 3000/8080，pac.at 端口声明不生效（018 README 描述的 `-F`/`-B` 与自动注入当前未实现）。**临时方案（022 已验证）**：环境变量分离启动——后端 `AUTO_HTTP_PORT=80NN ./app-NNNN-back.exe`；前端 `cd gen/front/vue && AUTO_FRONT_PORT=30NN AUTO_HTTP_PORT=80NN pnpm dev`。playwright `baseURL` 用 `http://localhost:30NN`（vite 默认监听 IPv6 `[::1]`，`127.0.0.1` 连不上）。**根治**：在 `run_vue_project` 调 `pac_dev_ports()` 并 `env::set_var("AUTO_FRONT_PORT"/"AUTO_HTTP_PORT")`，属独立 codegen 改进。
- **拖拽 codegen 事件已支持，但属性/数据未验证**：`vue.rs` 有完整 HTML5 事件映射（`ondragstart/drag/dragend/dragover/drop`）；`onmousemove.window` 全局修饰符也已支持（026 自定义滚动条即此模式）。但 `draggable` 属性 + `dataTransfer` 在 codegen 无专门处理，属"需验证"灰区（022 阶段 2 待评估）。
- **for + if 模式的 :key**（R006 warning，已验证无害）：`for x in xs { if cond { el{...} } }` 会触发 R006 warning，但 codegen 实际会为内层元素生成 `:key`（022 实测 `for card in .store.cards { if card.column=="todo" { row{key:card.id,...} } }` 生成 `:key="card.id"`）。warning 判断滞后于 key 生成，可忽略；若要消 warning 就改用 store 预派生分组（让 for 体直接是 Element）。
- **row/col 布局元素现在支持任意 HTML 属性**（022 阶段2 修复）：之前 row/col 的 shadcn 分支只输出 class，丢弃 draggable 等普通 prop。现已通过 `push_passthrough_attrs` 透传（vue.rs）。后续示例在 row/col 上写 `draggable`/`data-*` 等任意属性均可。
- **a2r 返回位置 struct literal 的已知规避**：`return T{...}` a2r 报 `undefined variable`；改 `let x = T{...}; return x` 规避。同类：reassignment `found = T{...}` 解析失败（改 `let rebuilt = ...; found = rebuilt`）、借用迭代变量字段 move（在 let-ctor 内访问触发 a2r clone）。均在源码层规避并注释。

### 023-realworld 阶段2 新增 gotcha（2026-08-10）
- **`tag` 是保留软关键字**（TokenKind::Tag，enum 别名）→ 不能作函数参数名，否则 a2r 报 `expected '{' or type after 'str'`。用 `filter_tag`/`needle` 等替代（023 db.at）。
- **store model 字段名不能与 `use back.api` 导入的函数名同名**：codegen 对 store 字段生成 `import { fn } from api` + 模块级 `const fn = ref(...)`，后者覆盖前者 → handler 调 `fn()` 报 `not a function`。023 的 `current_user` 字段改名 `me` 规避。规则：store 字段名避开 api 函数名（如 `current_user`/`list_articles` 等）。
- **同一 widget 不能映射多条路由**：codegen 给同 widget 的多条路由生成同 `name`，vue-router 4 重名丢弃先注册的 → 路径不匹配。023 编辑器合并成 `/editor/:slug`（新建用 `/editor/new`）规避。
- **store 的 struct 字面量初始值退化为 null**（codegen 已知边际，未根治）：`var x T = T { ... }` 生成的 store ts 是 `const x = ref<any>(null)`（struct literal 走 Expr::Node，store_init_to_js 未处理）。规避：模板访问字段前加 `!= nil` 判断（023 app/settings/article_detail）。根治需在 store_init_to_js 处理 Expr::Node → JS 对象（Node 是 widget-node 结构，映射较深，留后续）。
- **a2r List<T> 声明顺序敏感**（parser 已知边际，未根治）：`var result T = T { 多字段 }` 之后的 `var xs List<T> = List<T>.new([])` 解析失败（`Expected Gt, found ]`）；但顺序反过来（List 声明在前）正常。规避：函数体内 `List<T>` 声明放在 `var result T = T{...}` 之前（对齐 018）。根因是 parser 解析 struct literal 后状态泄漏到下一行泛型解析，留后续。
- **a2r 双路径参数只提取第一个**：`/api/:a/:b` 的 handler 只提取 `:a`，`:b` 丢失（023 delete_comment）。规避：改单路径参数（`/api/comments/:id`，评论 id 全局唯一）。
- **a2r POST/PUT 无 body 参数时多余 Json body**（已修复，api_gen.rs endpoint_has_body）：POST/PUT 只有路径参数时原生成 `Json<User>` body 提取 → 拒绝请求。已改：body 存在 iff 有 body 参数。
- **a2r_std import 路径**（已修复，trans/rust.rs）：生成的 `use a2r_std` 改为 `use auto_lang::a2r_std`（a2r_std 是 auto_lang 模块非独立 crate）+ back Cargo.toml 加 `auto-lang.workspace = true`（api_gen.rs generate_cargo_toml，has_db 时）。
- **VM 多 store 链接 bug**（未修复，留独立计划）：`auto run --render vm` 在多 store 项目报 `Undefined symbol: handler_X_Y`。根因：`handler_codegen.rs:1246` 把 store 映射硬编码 key `"store"`（多 store 覆盖）+ `lib.rs:2684` route-page loading 不递归加载页面的 store。单 store 项目（018/022）不受影响。详见 Plan 405 §VM 模式评估。

---

## 018 范式锚点（子计划参照）

018 是首个完整示例，其结构作为后续示例的模板。**实现详情见 master 上的源码**，本纲领只保留索引：

```
examples/ui/018-book-reader/
  pac.at                    # front_port:3018 / back_port:8018 / api:"rust"
  src/front/
    app.at                  # App 路由壳：routes{4 条} + 侧边栏 + outlet
    book_store.at           # BooksStore：跨页共享 store（k1 模式）
    pages/*.at              # 4 个路由页
  src/back/
    api.at                  # pub type + #[api] 端点，委托 db
    db.at                   # 强类型内存存储 + 种子数据
  vue/src/components/*.vue  # handmade escape hatch（暗色运行时切换）
  tests/                    # playwright 四件套
```

**018 过程中修复的 3 个 codegen bug（已根治通用）**：
1. 路由 useRoute 缺失（`ui_gen/vue.rs` + `ui_gen/ts_adapter.rs`）：lifecycle `.Init` 里的 `router.param()` 未触发 `needs_route` → 扩展 `stmts_have_route_access` 识别 `router.param/query/path`，needs_route 检测扩展到 `widget.lifecycle`。
2. 无路径参数 GET URL 多余引号（`api/targets/typescript.rs:340`）：无 `:param` 时 url 用单引号，GET query 拼接按反引号 trim → 单引号漏进字面 → 404。改无参分支也用反引号。
3. a2r 返回位置 struct literal：源码层规避（见上节技术约定），未改 a2r。

**018 验收**：playwright **10/10 全绿**（干净态可复现）。

---

## §进度总表

| 示例 | 现状 | 升级状态 | 子计划 | 备注 |
|---|---|---|---|---|
| 018-book-reader | 已升级 | ✅ 完成 | (本纲领 §018) | 10/10 全绿，合并 master `bc5e1041` |
| 011-calculator | 整数四则 | 🔀 已拆出 | Plan 403 | grid 重构 + MCP + 多模式 |
| 022-kanban | 已升级 | ✅ 完成 | 本纲领 §022（提交历史） | CRUD + 列移动 + HTML5 拖拽，6/6 全绿；修 row/col 属性穿透 bug |
| 023-realworld | 已升级 | ✅ 完成(阶段1+2) | [Plan 405](405-023-realworld.md) | 完整 Conduit(认证+CRUD+评论+关注+收藏+资料)，14/14 全绿；vue 原型 |
| 019-video-app | 135 行单文件 | ⬜ 待办 | — | 中 |
| 020-music-player | 115 行单文件 | ⬜ 待办 | — | 中 |
| 021-blog-viewer | 89 行单文件 | ⬜ 待办 | — | 中 |
| 024-widget-gallery | 283 行展示型 | 🗑 已清理 | [Plan 409](409-widgets-gallery.md) | gallery 已落地为 `examples/widgets-gallery/`（62 页三模式一致性）；旧 `examples/ui/024-*` 目录（含单数拼写的空壳）已于 2026-08-23 删除，唯一入口为 `examples/widgets-gallery/` |
| 025-notes-extended | 6 文件无后端 | 🗑 已清理 | — | 015 的前端丰富度临时 fork，store+路由概念已被 015-notes 吸收（Plan 354 §7）；目录于 2026-08-23 删除（SPEC 留存于 git 历史） |
| 026-keyboard-mouse-events | 121 行能力展示 | 📦 已迁出 | — | 非 App 性质（能力 demo）；2026-08-23 迁至 `examples/capability-tests/` |
| 027-native-css | 79 行能力展示 | 📦 已迁出 | — | 非 App 性质（能力 demo）；同批迁出 |

> **2026-08-23 目录分轨**：`examples/ui/` 现在只放 App 性质示例（含 038-minesweeper、041-auto-edit）。021-block-static + 026–040 的 16 个能力样板（撞号的 038-vshow 也在其中）整体迁至 `examples/capability-tests/`（见其 README 的 Feature fixtures 区段）；特性获得可覆盖的内联测试后按 README 约定退役。

**推荐批次顺序**：022-kanban（✅）→ 023-realworld（✅ 阶段1+2）→ 024（✅ 经 409，旧目录已清理）→ 019/020/021（中等）。（025 已删除，不再排队）

---

## 提交历史

| commit/分支 | 内容 | 示例 |
|---|---|---|
| `plan401/018-vm-routing` → master | 018 完整升级 + per-example 端口 + VM/iced 路由支持 + storage 内置（合并 master `bc5e1041`） | 018 |
| `plan399/018-book-reader` | 018 升级 + 3 codegen 修复 + playwright 10/10 | 018 |
| `plan401/022-kanban` | 022 阶段1：CRUD + 列移动 + playwright 5/5 | 022 |
| `plan401/022-drag` | 022 阶段2：HTML5 拖拽 + 修 row/col 属性穿透 bug + playwright 6/6 | 022 |
| `plan401/023-realworld` | 023 阶段1：Conduit 认证+feed+详情 + vue 原型 + 修 3 个 a2r codegen bug + playwright 8/8 | 023 |
| `plan401/023-stage2` | 023 阶段2：Conduit 写操作(CRUD+评论+关注+收藏+资料) + 2 codegen 修复 + playwright 12/14 | 023 |
| `plan401/023-editor-fix` | 023 编辑器修复(store 字段同名 + 重名路由) → playwright 14/14 全绿 | 023 |
