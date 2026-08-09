# Plan 401: AutoUI 示例升级 — 018-027 从单文件玩具到完整 App

> **状态（2026-08-09）**: 🟡 进行中。**401 为纲领计划**（定义标准 + 维护进度总表），每个示例的具体实现拆分为独立子计划。018 已完成；022 阶段 1 完成（Plan 404）。
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
  - ⚠️ **端口缺口（2026-08-09 Plan 404 实测发现）**：`pac.at` 的 `front_port`/`back_port` 已在 `pac.rs` 解析、`automan.rs` 有 `pac_dev_ports()` getter，但 **`auto run`（`run_vue`/`run_vue_project`）当前未调用它注入环境变量**，`main.rs` 也无 `-F`/`-B` CLI 参数定义。结果 `auto run` 仍默认 3000/8080，pac.at 端口声明不生效（018 README 描述的 `-F`/`-B` 与自动注入当前未实现）。**临时方案（022 已验证）**：环境变量分离启动——后端 `AUTO_HTTP_PORT=80NN ./app-NNNN-back.exe`；前端 `cd gen/front/vue && AUTO_FRONT_PORT=30NN AUTO_HTTP_PORT=80NN pnpm dev`。playwright `baseURL` 用 `http://localhost:30NN`（vite 默认监听 IPv6 `[::1]`，`127.0.0.1` 连不上）。**根治**：在 `run_vue_project` 调 `pac_dev_ports()` 并 `env::set_var("AUTO_FRONT_PORT"/"AUTO_HTTP_PORT")`，属独立 codegen 改进。
- **拖拽 codegen 事件已支持，但属性/数据未验证**：`vue.rs` 有完整 HTML5 事件映射（`ondragstart/drag/dragend/dragover/drop`）；`onmousemove.window` 全局修饰符也已支持（026 自定义滚动条即此模式）。但 `draggable` 属性 + `dataTransfer` 在 codegen 无专门处理，属"需验证"灰区（022 阶段 2 待评估）。
- **for + if 模式的 :key**（R006 warning，已验证无害）：`for x in xs { if cond { el{...} } }` 会触发 R006 warning，但 codegen 实际会为内层元素生成 `:key`（022 实测 `for card in .store.cards { if card.column=="todo" { row{key:card.id,...} } }` 生成 `:key="card.id"`）。warning 判断滞后于 key 生成，可忽略；若要消 warning 就改用 store 预派生分组（让 for 体直接是 Element）。
- **row/col 布局元素现在支持任意 HTML 属性**（022 阶段2 修复）：之前 row/col 的 shadcn 分支只输出 class，丢弃 draggable 等普通 prop。现已通过 `push_passthrough_attrs` 透传（vue.rs）。后续示例在 row/col 上写 `draggable`/`data-*` 等任意属性均可。
- **a2r 返回位置 struct literal 的已知规避**：`return T{...}` a2r 报 `undefined variable`；改 `let x = T{...}; return x` 规避。同类：reassignment `found = T{...}` 解析失败（改 `let rebuilt = ...; found = rebuilt`）、借用迭代变量字段 move（在 let-ctor 内访问触发 a2r clone）。均在源码层规避并注释。

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
| 022-kanban | 已升级 | ✅ 完成 | [Plan 404](404-022-kanban.md) | CRUD + 列移动 + HTML5 拖拽，6/6 全绿；修 row/col 属性穿透 bug |
| 023-realworld | 已升级 | ✅ 阶段1完成 | [Plan 405](405-023-realworld.md) | Conduit spec 阶段1(认证+feed+详情)，8/8 全绿；阶段2待后续 |
| 019-video-app | 135 行单文件 | ⬜ 待办 | — | 中 |
| 020-music-player | 115 行单文件 | ⬜ 待办 | — | 中 |
| 021-blog-viewer | 89 行单文件 | ⬜ 待办 | — | 中 |
| 024-widget-gallery | 283 行展示型 | ⬜ 待办 | — | 低（可能无需后端） |
| 025-notes-extended | 6 文件无后端 | ⬜ 待办 | — | 低 |
| 026-keyboard-mouse-events | 121 行能力展示 | ⏸ 不升级 | — | 非 App 性质（能力 demo） |
| 027-native-css | 79 行能力展示 | ⏸ 不升级 | — | 非 App 性质（能力 demo） |

**推荐批次顺序**：022-kanban（✅）→ 023-realworld（✅ 阶段1）→ 019/020/021（中等）→ 024/025（低）。

---

## 提交历史

| commit/分支 | 内容 | 示例 |
|---|---|---|
| `plan401/018-vm-routing` → master | 018 完整升级 + per-example 端口 + VM/iced 路由支持 + storage 内置（合并 master `bc5e1041`） | 018 |
| `plan399/018-book-reader` | 018 升级 + 3 codegen 修复 + playwright 10/10 | 018 |
| `plan401/022-kanban` | 022 阶段1：CRUD + 列移动 + playwright 5/5 | 022 |
| `plan401/022-drag` | 022 阶段2：HTML5 拖拽 + 修 row/col 属性穿透 bug + playwright 6/6 | 022 |
| `plan401/023-realworld` | 023 阶段1：Conduit 认证+feed+详情 + vue 原型 + 修 3 个 a2r codegen bug + playwright 8/8 | 023 |
