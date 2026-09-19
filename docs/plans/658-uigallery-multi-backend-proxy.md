---
plan_id: PLAN-658
status: executing
feature_name: uigallery-multi-backend-proxy
author: [agent]
created_at: 2026-09-19
updated_at: 2026-09-19
plan_revision: 1
current_step: 4
total_steps: 8
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-010]
affects: [auto-lang/vm, auto-lang/ui, auto-man, auto-os/ui-gallery]
---

# [PLAN-658] uigallery-multi-backend-proxy

## 0. 变更摘要

画廊内嵌 fullstack demo 的"后端半身"无宿主进程（PLAN-642 终审遗留
P642-D10/D13）：020-music-player 曲库空（`Http.get_json("/api/media/scan")`
无后端进程应答）、017-chat / 031-image-viewer 被 back 链 native-ns/stream
否决（"暂无内嵌形态"）。本计划建立**单进程多后端宿主（proxy）**：每个
app 的后端以独立 VM session 在同一进程内运行，按子 URL
（`/apps/<id>/api/*`）路由；前端 HTTP 调用经生成器 baseURL 子前缀适配。
**用户裁定范围（2026-09-19）**：不止解决 020，**017/031 的
native-ns/stream 后端为一等公民目标**——stream/WebSocket 转发与 binary
（图片）转发在立项范围内，不是风险排除项。

## 1. 目标

- **G-1 020 曲库内嵌可见**：画廊内嵌 020 经 `/apps/020-music-player/
  api/media/scan` 出真实曲库（E:/Music mp3 列表），P642-D10 核销。
- **G-2 stream 一等公民**：017-chat 的 `pub fn stream() ~Stream<ChatEvent>`
  （api.at:62 实证）经 proxy 转发——内嵌聊天可收发消息，事件流实时到达。
- **G-3 native-ns/binary 一等公民**：031-image-viewer 的 `use auto.image`
  后端（image_service.at 实证）在 session 内可执行，图片字节经 proxy
  转发——内嵌图库缩略图/大图出图。
- **G-4 端口收敛**：全部内嵌 demo 后端经单端口子 URL 前缀服务
  （3049..3050+N → 1）。
- **G-5 崩溃隔离**：单 app 后端 panic 被捕获（自动重启或错误面板），
  不带倒宿主与其他 app session。
- **非目标**：跨机/远程部署形态；子 domain 路由（远期选项）；独立运行
  形态（各 demo 自带 Axum 后端进程）的变更——必须零回归；web 臂
  （Vue）语义变更；proxy 热重载。
- **受影响仓库**：auto-lang（VM session 基建 / auto-man 生成器 baseURL /
  Http stdlib 前缀展开）+ auto-os（gallery 启动编排 + registry 注入）。
- **成功判据**：AC-01..06 全过；独立形态回归零红。

## 2. 架构方案

承 PLAN-642 §8.2 可行性四依据（归档在案）：

1. **会话隔离原型**：`auto serve` daemon（Plan 269）已是单进程多 stateful
   VM session（named pipe）——proxy 把 session 前端从 pipe 换成 HTTP 路由。
2. **back 链进程内执行已通**：PLAN-633 merged 模式（013/015 内嵌）证明
   back .at 可在宿主 VM 进程内装载执行——"每 app 一个 VM session"是同一
   机制的会话化扩展；**路由分发形态采用 session 内 `#[api]` fn 表动态
   分发（merged CALL 语义的 HTTP 化）**，Axum 生成器路由表为辅。
3. **前端适配面小**：生成器 api baseURL `:port` → 子 URL 前缀
   `/apps/<id>`；`Http.get_json` 相对路径按前缀展开——PLAN-617
   `AUTO_HTTP_BASE` 相对展开先例（get/post/put/delete/json 五臂）。
4. **推荐形态**：子 URL 路径前缀（零 DNS/hosts 成本）。

**风险面（用户裁定升级为一等公民目标）**：session 崩溃隔离（per-session
catch + 重启）；SSE/stream 转发（017）；binary 响应转发（031 图片字节）；
可观测性（每 session 日志通道）。

**进程形态**（T-00 决策产物定案，默认倾向）：复用 auto serve daemon 扩展
HTTP 前端 vs auto run 宿主内嵌 proxy 线程——按"gallery 一键体验
（`auto run -r vm` 即含全部后端）+ 启动编排复杂度"权衡。

## 3. 技术栈

- Rust：`crates/auto-lang/src/vm/`（session 基建——`auto serve` daemon
  Plan 269 形态参考）、session 内 `#[api]` 路由注册表（Plan 312
  `register_http_routes` 既有钩子）、axum（宿主路由层）。
- auto-man：`crates/auto-man/src/`（back→Axum 生成器路由清单复用 + api
  baseURL 子前缀适配）。
- Http stdlib：`crates/auto-lang/src/vm/ffi/stdlib.rs`（AUTO_HTTP_BASE
  相对展开，PLAN-617 五臂）。
- 语料/产物：examples/ui/{017-chat,020-music-player,031-image-viewer}
  back 链（语料不动）；auto-os ui-gallery 启动编排 + registry.at。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户 2026-09-19（PLAN-642 收口裁定）："这个我同意独立立项，且不止解决
  020 一个问题，也要解决 017/031 的问题。先把它写进 DEBT 里，等 642 收尾
  后再单独立项。" → P642-D13 登记（KNOWN-DEBT-AND-RISKS.md 2026-09-19
  增补二），本计划即该裁定承接。
- 可行性调研授权承 PLAN-642 T-19 前置调研（§8.2，归档
  `docs/plans/archive/642-uigallery-vm-embed-repair.md`）。

### 4.2 关键实证（起草期核实）

| demo | back 形态 | 证据 |
|---|---|---|
| 020-music-player | `#[api]` 路由族（player/status 等；媒体索引"自动经 /api/media/scan 与 /api/media/stream/:id 服务"） | back/api.at:3/11 |
| 017-chat | `pub fn stream() ~Stream<ChatEvent>`（流签名后端） | back/api.at:62 |
| 031-image-viewer | `use auto.image`（原生命名空间图像管线） | back/api.at:7 / image_service.at:5 |

既有发射器否决点（auto-man vue.rs fullstack 级联的 unsupported 判定）：
`use auto.*` / `~Stream` / `~Promise` → back_ok=false → 静态面板——本计划
打通后按 demo 逐族解除。

### 4.3 背景

- PLAN-633：merged 模式 back 内嵌（013/015）——进程内 CALL reloc 语义。
- PLAN-617：AUTO_HTTP_BASE 相对 URL 展开（get/post/put/delete/json）。
- Plan 269：auto serve daemon named-pipe session 管理。
- Plan 312：`#[api]` 路由收集（register_http_routes）。

## 5. 详细设计

（T-00 决策产物 2026-09-19 回填定案；调研证据：四路并行探索 + 主线核实，
关键行号见各节。）

### 5.1 进程形态裁定：auto run 宿主内嵌 proxy 线程（方案 B）

| 维度 | A：auto serve daemon 扩展 HTTP 前端 | **B：宿主内嵌 proxy 线程（选定）** |
|---|---|---|
| 一键体验 | 需额外拉起/发现 daemon 进程 | `auto run -r vm` 单进程即含全部后端 |
| 启动编排 | 双进程生命周期（spawn/健康/清理） | run_vm_ui 内一个钩子位（rust_ui.rs:3075-3088 画廊钩子旁） |
| 授权面 | 改 daemon 公共协议 → 越本计划授权（§10 呈报项） | 零新安装面、零协议变更 |
| 先例 | named-pipe REPL 服务 | start_vm_server（rust_ui.rs:2680，VM HTTP 线程）+ Plan 061 外部后端，同为进程内后端先例 |
| session 线程模型 | 复用 daemon 两线程模式 | 同样复用：IO 线程 + 每 session 专线程（AutoVM 线程亲和，autovm_daemon.rs:129-131 先例） |

### 5.2 session 模型与路由分发

- 模块：`crates/auto-lang/src/ui/back_proxy.rs`（新）。宿主 = 手写
  HTTP/1.1 前端（std TcpListener + 每连接一线程，仿 http_server.rs
  serve_blocking_stdnet 先例；无新依赖——axum 为可选 dep 且需 tokio
  桥接，收益不足）。端口 `AUTO_GALLERY_PROXY_PORT`（默认 3358，冲突
  回退 +1..+10，仿 MCP 9247 先例）。
- `HashMap<app_id, SessionHandle>`；SessionHandle = 专线程 + mpsc 请求
  通道（daemon 模式）。每 session 装载该 demo back 链编译产物
  （CompiledPackage 自带 `api_routes: Vec<(method,path,fn_name)>`，
  loader.rs:45-47）→ AutoVM。**绕过进程级全局 HTTP_ROUTES 表**
  （stdlib.rs:3481 覆盖式单表，多 session 不能共用）。
- 路由分发序列：请求 → 剥 `/apps/<id>` 前缀 → session 路由表命中
  （复用 http_server.rs match_route 的 :param/query 语义）→
  `call_fn_by_name`（参数 marshalling 仿 VmBridge::call_vm_fn，
  vm_bridge.rs:1295-1382；位置参数约定仿 build_handler_args）→
  nv_to_json 返回。路由未命中 .at 表 → 宿主原生存（§5.4/5.5）。
- **020 不建 session**：其 .at back 仅 `status()->[]`（gallery 内已
  merged CALL，返回值无人消费）；真实曲库来自宿主原生 media 路由。
  013/015/020(status)/022 全族维持 merged CALL 零回归。

### 5.3 前端适配（T-02；实施裁定收敛）

- 实施裁定（2026-09-19 T-02）：**不经 AUTO_HTTP_BASE**——画廊 runner
  在 proxy 绑定后将 base 以线程局部传给发射器，发射器对拷入 demos/ 的
  per-demo 语料做 `/api/` 字面量前缀化（相对 → 绝对
  `http://127.0.0.1:<port>/apps/<id>/api/...`）；proxy 原生 media scan
  响应里的 `url` 字段直接发绝对 URL。理由：AUTO_HTTP_BASE 是进程单值
  env，多 app 各需各的前缀无法表达；字面量前缀化 per-app 精确、独立
  形态零触及（仅画廊发射路径），且 `resolve_http_base_url` 语义零改动
  （绝对 URL 透传）。
- 020（语料不动，画廊拷贝件被前缀化）：`Http.get_json("/api/media/
  scan")` → 绝对子前缀 URL；曲库条目 url 由 proxy 发绝对值。
- 017/031（发射期改写）：发射器为每 demo 生成 client 模块
  `demos/<ns>_client.at`（把 `use back.api:` 的 fn 逐个实现为
  Http.*_json 绝对 URL；类型定义自 back api.at 的 pub type/tag 摘出
  随行）——merged 编译单元内纯 .at 可编译，api 调用不经 codegen 三态
  决策（非 api import）。发射顺序约束：proxy 先绑端口 → 发射器拿
  端口 → run_file。

### 5.4 stream 转发时序（017，T-04）

- wire 形态查证：独立 Axum 形态 = SSE（api_gen.rs:1817-1833，events.rs
  broadcast channel）；VM HTTP server 对 iterator 返回值同样 SSE 化
  （http_server.rs:2666-2700）。**本计划 proxy 亦用 SSE**——前端消费
  语义与独立形态一致。
- 关键实证：`bus` 无 VM 原生实现（全仓 grep 零命中）——`stream()`
  函数体 `bus.subscribe()` 在 VM 侧是死码；两个生成器均按**签名**
  （返回类型含 `Stream<`）特路处理、忽略函数体，SSE 广播由宿主在
  POST 处理器完成（api_gen.rs:2092-2101 broadcast + discriminator 推断
  broadcast_event_name:1226）。proxy 复刻同语义：~Stream 端点按签名
  特路 → 每 session 宿主侧事件总线 → SSE 连接逐事件转发；POST 成功
  后按 create/typing 惯例广播。**语料零改动**。
- 前端 VM 臂接线（新增能力）：`http.sse_get_stream(url)` 原生已存在
  （Plan 341，stdlib.rs:6255，reqwest async + mpsc + AsyncHttpStream
  迭代器）但无 codegen 接线；`store.Msg(args)` 发送语法存在
  （app.at:87 `store.Init()`）。T-04 落地形态（三选一按实证定）：
  发射器注入消费循环 / client 模块提供 stream 接线 / store 级注入。
  Vue 臂 EventSource 接线（ts_adapter.rs:1273）不动。

### 5.5 native-ns/binary（031，T-05）

- `auto.image` 17 natives 全部进程内可用（stdlib.rs:4055-4226，cfg
  ui-iced——VM 臂 auto 二进制满足；注册进进程级 BIGVM_NATIVES，session
  VM 同样可调）。宿主 image pipeline 实为 **PLAN-547**（非计划原文
  的 PLAN-646——646 是 select-anything，已核实修正）。
- 字节服务：快照返回不透明 URI `/api/__auto/media/{id}/{rev}`；proxy
  增宿主短路路由委托 `media_http_response`（image_pipeline.rs:1232，
  原始编码字节 + Content-Type/ETag/304 保真）——AC-03 字节级校验经
  此路由取证。
- 渲染快路径注记：VM 臂 renderer 对 media URI 有进程内解析
  （resolve_media_uri renderer.rs:6066）——session 与画廊宿主同进程
  共享 pipeline，窗口内出图可不经 HTTP；proxy 路由保证字节面可独立
  取证 + 未来消费方（Vue 臂）可用。

### 5.6 崩溃隔离与可观测性（T-06）

- 每 session 专线程 `catch_unwind` 包请求执行：panic → 500 + JSON
  error + session 标记 degraded；重启策略 = 退避重建（back 链重编译
  装载，状态归零——后端本就内存态）。VMError（非 panic）同样 500 +
  日志，不重启。宿主线程与其余 session 不受影响。
- 可观测性：每 session 日志通道 `[proxy:<app_id>]` 前缀 + 环形缓冲；
  `AUTO_BACK_PROXY_TRACE=1` 打开请求级打点（仿 AUTO_LANG_HTTP_TRACE）。

### 5.7 调研修正记录（对计划原文的三处校准）

1. "PLAN-646 形态"提法与仓库事实不符——image pipeline 实为 PLAN-547
   （archive/547-image-viewer-pipeline.md + design/autoui/
   image-viewer-pipeline.md，标注 Implemented）。
2. "3049..3050+N"是本计划对假设现状的概括记法，两仓无该区间分配
   代码；实际端口带：画廊前端口 3049（auto-os/ui-gallery/pac.at:15）、
   demo 独立 back 8xxx。
3. 020 的 /api/media/scan 与 /api/media/stream/:id **不在 .at 语料**——
   是生成器对全部后端无条件发射的宿主原生路由（api_gen.rs:2604-2636；
   media_service.rs 扫描/Range 语义）。proxy 侧这两路由由宿主 Rust
   直接实现（media_root 自 demo pac.at 读出，resolve_root 显式传参），
   不经 VM session。

### 5.8 任务集校准

原 T-01..T-07 任务集保持不变（无裂变）；T-03 含 proxy 宿主原生 media
路由实现，T-04 含 VM 臂 store 流接线新能力，T-05 含发射器解除 017/031
否决 + client 模块发射（T-04/T-05 各自验收内完成）。

### 规范增量

| delta_id | op | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/vm/back-proxy.md（新建） | 无 → 多后端 proxy 契约：子 URL 路由、per-app VM session 模型、#[api] 动态分发语义、stream/binary 转发、崩溃隔离边界 | 单进程多后端宿主为 enduring 架构面 | AC-01..05 |
| SD-02 | add | docs/specs/auto-lang/ui/overview.md §ui-gallery | 无 → 内嵌 fullstack demo 后端经 proxy 子 URL 服务的画廊契约（registry/启动编排/回退页解除条件） | 画廊内嵌形态扩展 | AC-01..03 |
| SD-03 | modify | docs/specs/auto-lang/vm/plans.md | 沉淀行追加 PLAN-658 | 账目 | — |

## 6. 测试设计

- **单元**：session 注册/路由命中/前缀解析；stream 转发 chunk 序列；
  binary 转发字节保真；panic 注入隔离（单测可注入假 session）。
- **E2E（主验收）**：画廊 VM 实例 MCP 驱动——020 曲库列表出现 + 播放器
  状态路由；017 双向消息时序；031 缩略图网格 + 大图加载；panic 注入后
  其他 demo 存活。
- **回归**：独立形态 017/020/031 `auto run`（各自 Axum 后端）行为不变；
  既有内嵌矩阵（013/015/022..）零回归；`cargo t` 日常档 + 触发面档。

## 7. 验收标准

- **AC-01**：画廊内嵌 020 曲库非空（E:/Music mp3 列表可见，请求经
  `/apps/020-music-player/api/*` 前缀）；P642-D10 核销。（G-1）
- **AC-02**：内嵌 017 可发送消息并收到流式回复（~Stream 事件经 proxy
  到达前端，时序截图/快照证据）。（G-2）
- **AC-03**：内嵌 031-image-viewer 图库出图（缩略图网格 + 点开大图，
  二进制经 proxy 保真——像素级或字节级校验）。（G-3）
- **AC-04**：全部内嵌后端流量经单端口 `/apps/<id>/` 子前缀（端口数=1；
  抓包/日志证明无 3049+ 直连）。（G-4）
- **AC-05**：单 app 后端注入 panic → 该 app 降级错误态（自动重启或错误
  面板），宿主与其他 app session 存活。（G-5）
- **AC-06**：门禁零新增红；独立形态三 demo 回归通过；既有内嵌矩阵零回归。

## 8. 执行步骤

（原子任务；每步完成后追加 [✅] 证据行）

- [x] **T-00 有界调研定案：session 模型与进程形态**
      决策产物回填 §5：①auto serve daemon 扩展 HTTP 前端 vs auto run 宿主
      内嵌 proxy 线程（一键体验/启动编排/生命周期对比表+裁定）；②017
      stream 的 wire 形态查证（~Stream 在既有 Axum 生成器的落线方式——
      SSE/长连接）；③031 auto.image 的进程内可用面（宿主 image pipeline
      PLAN-646 形态）。产出：设计决策记录 + 任务集校准（如需裂变在册）。
      [✅ 已完成 2026-09-19] §5.1-5.8 回填定案：形态=宿主内嵌 proxy 线程
      （§5.1 五维对比）；017=SSE wire + bus 无 VM 实现的死码实证 +
      sse_get_stream(Plan 341)/store.Msg() 两个先例（§5.4）；031=
      auto.image 17 natives 进程内可用 + media_http_response 字节短路
      （§5.5）；三处调研修正（§5.7：PLAN-646→实为 547、3049..3050+N
      为假设记法、020 media 路由是宿主原生存）。任务集无裂变（§5.8）。
      证据锚：autovm_daemon.rs:129（session 线程模型）、loader.rs:45
      （CompiledPackage.api_routes）、stdlib.rs:3481（全局路由表约束）、
      vm_bridge.rs:1295（marshalling 范本）、api_gen.rs:1817/2092（SSE
      发射+broadcast 语义）、stdlib.rs:6255（sse_get_stream）。
- [x] **T-01 proxy 骨架 + 纯 JSON API 面**
      axum 单宿主 + `HashMap<app_id, VmSession>` 注册表 + 子 URL 前缀解析
      + session 内 `#[api]` fn 表动态分发（Plan 312 路由清单）+ JSON
      响应。冒烟：020 `/api/player/status` 经子前缀返回真实状态。
      [✅ 已完成 2026-09-19] commit 6270bda92：`crates/auto-lang/src/
      back_proxy.rs`（crate 根，镜像 autovm_daemon 先例；手写 HTTP/1.1
      前端 std TcpListener，**非 axum**——设计定案 §5.2 无新依赖；per-
      session 专线程 + mpsc；路由表取自 Codegen api_routes 绕过全局单表；
      按名参数绑定路径占位符/body 字段/query；__module_init 会话态）。
      测试 `tests/back_proxy_tests.rs`（挂 test-http-e2e 门 + http_e2e_
      前缀 → `cargo th` 收录）4/4 PASS：①种子列表+跨请求态存续
      （POST 后 GET 见第二记录）②404 三面 ③缺参 400 ④**020 真实语料
      冒烟 `/apps/020-music-player/api/player/status` → 200 `[]`**（T-01
      验收句达成）。auto-down 依赖 worktree 补建于组目录（detached @
      7c0b774，跨仓 path 依赖 `../../../auto-down` 解析需要）。
- [x] **T-02 生成器 baseURL 子前缀适配**
      auto-man api baseURL `:port` → `/apps/<id>`（PLAN-617 AUTO_HTTP_BASE
      相对展开先例，五臂覆盖）；独立形态零变化（形态开关判定）。
      [✅ 已完成 2026-09-19] commit e40b3d496：实施裁定收敛（§5.3 回填）
      ——**不经 AUTO_HTTP_BASE**（进程单值 env 无法表达 per-app 前缀），
      改为发射器字面量前缀化：`GALLERY_PROXY_ROOT` 线程局部（rust_ui 在
      proxy 绑定后、refresh_gallery_registry 前注入；None=独立形态零
      改写）+ `prefix_api_url_literals`（仅含 `Http.` 的行改 `"/api/` →
      绝对 `<root>/apps/<id>/api/`；五臂天然覆盖——按行不按方法）。
      语料实证：三 demo 前端全部相对调用点仅 020 player_store.at:92 一处。
      测试 `test_emit_gallery_vm_demos_proxy_url_prefixing` 双臂（前缀化
      生效 + #[api] 属性路径保持相对 + 独立形态零改写）；gallery 发射
      15/15 全过零回归。resolve_http_base_url 语义零触碰（绝对 URL 透传）。
- [x] **T-03 gallery 集成 + 020 曲库端到端**
      auto-os 启动编排（proxy 随画廊拉起）+ registry/适配器注入子 URL +
      内嵌 020 出曲库（AC-01 全链）；P642-D10 核销。
      [✅ 已完成 2026-09-19] commit 571dcf4c6（auto-lang worktree；auto-os
      侧零代码改动——编排钩子全在 lang 侧 rust_ui 画廊位，启动编排=
      `auto run -r vm` 既有单命令不变）。实现：①proxy 原生 media 路由
      （cfg ui；scan 三态语义镜像 api_gen + url 字段发绝对值；stream 单区
      间 Range 200/206/416 + 流式写出不整读）；②`start_gallery_back_proxy`
      编排（注册判定镜像 registry loadable 三档并集——020 为 loadable 档
      fullstack=false，按 fullstack 过滤漏注册的实测修正；行缓存消两次
      分钟级扫描；失败降级不阻断画廊）；③rust_ui 画廊钩子 proxy 先于发射
      启动。**AC-01 全链 MCP 实证**（evidence/p658/：t03_020_library.png
      + t03_020_scan.json + range headers）：画廊 VM 臂窗口侧栏点选 020 →
      `共 393 首歌曲 · 默认路径 E:\Music` + 真实曲目渲染（粉雪/Adele/
      BEYOND），press 的 state_changes 显示 `current_url` 直指
      `http://127.0.0.1:3358/apps/020-music-player/api/media/stream/...`；
      发射产物 demos/player_store.at:92 前缀化实证；Range 206 窗口与真文件
      逐字节一致（bytes 0-511/7759377）。e2e 双配置：
      `test-http-e2e,ui-iced` 6/6 + `test-http-e2e` 4/4。
      **P642-D10 核销**（曲库非空达成）。CI 挂靠（media e2e 需 ui-iced
      feature 组合）记 T-07 收口。
- [ ] **T-04 stream 转发一等公民（017）**
      session 内 ~Stream 执行 + HTTP chunk/SSE 逐事件转发 + 前端事件流
      消费；内嵌 017 双向收发（AC-02）；017 否决解除（发射器 unsupported
      判定按 demo 通路放宽）。
- [ ] **T-05 native-ns/binary 转发（031）**
      `use auto.image` in-session 执行（宿主 image pipeline）+ 图片字节
      转发（Content-Type 保真）；内嵌 031 图库出图（AC-03）；031 否决解除。
- [ ] **T-06 崩溃隔离与可观测性**
      per-session panic catch + 重启策略（退避）+ 每 session 日志通道；
      panic 注入验收（AC-05）。
- [ ] **T-07 E2E 全矩阵 + 债核销 + 门禁收口**
      AC-01..06 全量复验；KNOWN-DEBT 核销/部分核销（D10 全销、D13 主体
      核销留远期项、031 家族债核）；spec 增量落稿；`cargo t` 日常档 +
      触发面档；独立形态回归三件套。

## 9. 复审记录

```yaml
stage: new
plan_id: PLAN-658
plan_revision: 1
outcome: pass        # 契约就绪,待用户确认后进入 work
code_commit: N/A
task_ids: []         # T-00..T-07 待执行
evidence: 起草期核实（§4.2 三 demo back 形态证据表）+ P642-D13 承接链
next: work（用户确认契约后：master commit 本计划 → 建 worktree 组
  D:/autostack/.wt/lang-658/{auto-lang,auto-os} → T-00 有界调研定案）
```

```yaml
stage: work
plan_id: PLAN-658
plan_revision: 1
outcome: pass        # 进入执行（2026-09-19 用户指令"实施它"=契约确认）
code_commit: N/A     # worktree 未建前主检出仅簿记
task_ids: []         # T-00 起
evidence: 用户会话指令确认契约；auto-lang 主检出 clean、auto-os 主检出
  WIP 定性为 widgets-gallery 生成漂移+会话产物（非 ui-gallery 面，已呈报不入本计划）
next: 建 worktree 组 D:/autostack/.wt/lang-658/{auto-lang,auto-os} → T-00
```

## 10. 待澄清事项

- T-00 决策产物"进程形态"若越出本计划授权面（如需改 auto serve 公共
  协议或引入新守护进程安装面），呈报用户裁定后再继续对应任务。
- 017 stream 若查证为非 HTTP wire 形态（纯 pipe/进程内），转发层设计
  相应调整并回填 §5——语义目标（内嵌可收发）不变。
