---
plan_id: PLAN-547
status: drafting
feature_name: AutoUI high-performance image viewer pipeline
author: [Codex]
created_at: 2026-09-04T18:32:05+08:00
updated_at: 2026-09-04T18:43:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [auto-lang/ui, auto-lang/runtime, auto-lang/trans, auto-man, stdlib, examples/ui]
current_step: 0
total_steps: 42
---

# [PLAN-547] AutoUI 高性能 Image Viewer 与统一后端图片管线

## 变更摘要

新增独立的 `examples/ui/031-image-viewer` 全栈 AutoUI 示例。前端和后端均使用 Auto：
Vue 模式生成 Vue 前端与 Rust HTTP 后端，VM 模式解释执行，Rust 模式通过
`auto run -r rust --server rust --merged` 生成最终进程内原生应用。

图片目录扫描、加载、解码、EXIF orientation、缩略/显示 rendition、请求优先级、
latest-wins、邻图预取和缓存预算全部由后端 Auto 服务定义一次。Vue 只消费生成的 JSON
控制 API 和后端媒体 URI；VM/Rust merged 对同一媒体 URI 使用进程内快速路径。

计划同时新增通用 `ImageSurface` 跨端 widget 和公共媒体资产运行时，消除当前 Iced
普通 `image` 在 view 构建期同步读文件/HTTP，以及进程级无界字节/Handle `HashMap`
不适合图片浏览器的问题。

详细架构以
[`docs/design/autoui/image-viewer-pipeline.md`](../design/autoui/image-viewer-pipeline.md)
为本计划唯一设计输入。

## 目标

1. 新建 `031-image-viewer`，不扩展或改变 `029-photo-gallery` 的产品定位。
2. 一份 front/back Auto 源码在 Vue、VM、Rust 三种运行形态下功能可用。
3. Back Auto 成为图片加载、处理、调度和缓存策略的唯一真源；Vue 不实现 File API、
   图片解码、缩略、预取或应用级缓存。
4. 新增 `/api/__auto/media/{asset_id}/{revision}` 数据面；普通 `api.at` 继续承载 JSON
   控制面，禁止图片经 JSON/base64/`[]int` 传输。
5. VM/Rust merged 使用媒体 URI 的进程内解析，不启动 HTTP、不重复编码/解码。
6. 原生 UI 线程不进行目录扫描、文件读取、解码或高质量重采样。
7. 实现 current-first、prev/next 预取、小线程池、latest-wins、80ms settle 和字节预算
   LRU 等 qimgv 启发的算法，但不复制 qimgv 代码或资源。
8. Rust merged release 达到明确的首帧、邻图切换、空闲 CPU、队列和内存预算门禁。
9. 三端拥有可重复的自动化功能验证和真实截图证据。

## 架构方案

### A. 新示例而非扩展 Gallery

`029-photo-gallery` 保持相册/搜索/排序/收藏/网络缩略图示范；`031-image-viewer` 专注
后端本地媒体管线与桌面查看交互。公共复用边界是 `ImageSurface` 和媒体运行时，两个
应用不共享业务状态。

### B. 双平面传输

- 控制面：`src/back/api.at` 的 JSON API，返回 session、文件列表、generation、元数据、
  asset ticket 和运行时统计。
- 数据面：生成后端自动挂载 `/api/__auto/media/...`，直接返回编码图片。
- Vue：相对媒体 URI 经现有 Vite `/api` proxy 访问 Rust 后端。
- VM/Rust merged：Iced renderer 拦截同一媒体 URI，直接查询进程内
  `MediaAssetRegistry`。

### C. 后端 Auto 策略、公共 Rust 机制

Back Auto 保存 viewer session 和算法状态，决定请求优先级、keep set、generation 和
接受结果的条件。公共 Rust runtime 只提供线程、解码器、编码缓冲、LRU、资产注册表、
HTTP 响应和 renderer handle 等系统机制。

### D. `ImageSurface`

新增后端中立 Display widget，props 为 `src/fit/zoom/offset/rotation/filter/alt`，事件为
`onload/onerror/onwheel/onpan/ondblclick`。Vue generator 输出容器 + `<img>` 与输入归一化；
VM builder 和 Rust generator 输出同一 `View::ImageSurface`；Iced custom widget 完成原生
绘制和指针事件。

### E. 非阻塞 ticket 模型

Auto UI handler 不等待目录或解码任务。Back API 将工作放入有界队列并立即返回 queued
ticket；`ImageSurface` 等待对应媒体资产 ready。当前代由 `(session_id, generation,
view_revision)` 判定，旧结果可以完成解码但不能 publish 为当前资源。

## 技术栈

- Auto：front widget、back service、`api.at`、stdlib `auto.image` 接口。
- Rust：公共 `ui::image_pipeline`、Iced custom widget、VM FFI、a2r 适配、Axum media route。
- `image 0.25`：JPEG/PNG/WebP 解码、静态图变换和编码；保持在 `ui-iced`/媒体 feature 下。
- EXIF reader：只读取 orientation，依赖作为可选媒体 feature 接入。
- Iced 0.14：VM/Rust 原生显示、纹理 transform、输入与重绘。
- Vue 3/Vite：生成前端；现有 `/api` proxy 承载媒体 GET。
- Playwright：Vue + Rust backend E2E。
- AutoUI MCP：VM/Rust 原生状态、交互和截图验证。
- PowerShell/系统进程采样：Windows release 启动时间、CPU、RSS 和退出资源验证。

## 需求分析与背景调查

### 仓库现状

`docs/specs/overview.md` 将 Auto 定义为“脚本开发 → 转译发布”的多目标语言；`auto-lang`
同时包含 VM、trans、runtime、ui 和 ui_gen，`auto-man` 负责构建调度与前端生态集成，
`examples/` 是应用验证轨道。本计划同时触及：

- `auto-lang/ui`：AURA、View、Iced renderer、Vue/Rust generator；
- `auto-lang/runtime` 与 `vm`：后台任务、媒体注册表、VM native；
- `auto-lang/trans`：`auto.image` 的 VM/a2r 双实现；
- `auto-man`：Rust HTTP 后端路由和生成 workspace 依赖；
- `stdlib`：公开的后端 Auto 图片接口和 `ImageSurface` widget 声明；
- `examples/ui`：三运行形态的真实应用和验收语料。

### 已确认事实

1. 当前 `View::Image` 只有 `src + style`；Rust UI generator 仅生成 `View::image(...)`。
2. Iced renderer 当前在构建 image element 时同步调用 `load_image_bytes`。
3. 网络路径使用 blocking HTTP；字节缓存与 Handle 缓存是进程级无容量限制 `HashMap`。
4. 当前 TypeScript API 客户端默认 `response.json()`，Axum 普通 endpoint 默认 `Json<T>`，
   不适合作为图片数据面。
5. Rust UI 生成物引用公共 `auto_lang::ui::{Component, View}` 并通过公共 Iced runner 启动，
   因此 VM 与转译 Rust 可以共享同一原生媒体 runtime。
6. Vite 已把 `/api` 代理到生成的 Rust backend，可直接承载框架媒体路由。
7. `image 0.25` 已作为 `auto-lang` 的可选依赖存在；`rfd` 已用于原生文件对话框能力。

### qimgv 参考结论

只吸收以下算法思想：两个加载 worker、当前图高优先级、提交当前请求前清理未开始旧任务、
只保留当前与相邻工作集、单 resize lane、连续缩放只执行最新请求、80ms settle、最小化
重绘。Qt object、signal/slot、QImage/QPixmap、源码和素材均不移植。

### 范围裁定

- 首期格式固定为 JPEG、PNG、WebP 静态图；EXIF orientation 必须正确。
- 首期不做动画、视频、RAW/HEIF/AVIF/JXL、编辑、文件删除/移动、远程上传和瓦片金字塔。
- Vue 通过本机/已授权 Rust backend 访问文件；不引入浏览器 File API。
- 通用 `BinaryResponse` 留待媒体路由验证后另立计划；本计划只实现框架媒体数据面。

## 详细设计

### 1. 公共媒体资产

`MediaAssetKey` 由 source fingerprint、orientation、rendition spec 和 revision 构成。
注册表条目保存 opaque asset id、状态、MIME、逻辑尺寸、encoded `Arc<[u8]>`、可选 decoded
pixels、错误、ETag、引用/pin 和最后访问时间。路径永不进入 URI。

```text
Queued → Reading → Decoding → Transforming → Ready
   └──────────────→ Error
任意非终态 ──────→ Stale（generation/revision 失效，不 publish）
Ready/Error ──────→ Expired（无引用 + TTL/LRU）
```

### 2. 后端队列

- decode worker：2；settled resize lane：1；总排队上限：8。
- priority：current=100、settled-current=90、neighbor=10、thumbnail=5。
- current 是硬 pin；prev/next 是软 pin；其他条目可立即淘汰。
- 入队、decode 完成、publish 三处检查 generation/revision。
- 相同 rendition key 共用任务和结果；同 session 只保留最后 settled request。

### 3. 缓存预算

- metadata 最多 4096 项；encoded 64 MiB；decoded host pixels 256 MiB。
- 当前图允许临时突破 decoded budget；publish 后驱逐非 pinned 项，常态不得超过
  `256 MiB + current decoded bytes`。
- Handle cache 改为受 registry 生命周期约束的弱/有界映射，禁止新的永久静态 HashMap。
- 关闭 session 后解除所有 pin，2 秒内 in-flight/pinned/asset 引用归零。

### 4. 媒体路由

路径固定 `/api/__auto/media/{asset_id}/{revision}`，支持 GET/HEAD/ETag/304；404 表示不存在，
410 表示 ticket 已过期，422 表示解码失败，未就绪等待超时后返回 503 + Retry-After。
成功响应具有准确 MIME/Content-Length 和 private immutable cache header。

### 5. 后端 Auto API

`src/back/api.at` 暴露：

- `open_file()` / `open_directory()`：后端原生选择器，创建 queued session；
- `open_path(path)`：测试和显式配置入口，只允许 canonical 授权根；
- `snapshot(session_id)`：当前文件、索引、generation、asset 和邻图；
- `navigate(session_id, delta)`：更新 current、keep set 和预取；
- `request_view(ViewRequest)`：提交 latest settled rendition；
- `close_session(session_id)`：取消/解除 pin；
- `image_stats(session_id)`：测试和 DevTools 只读统计。

所有 endpoint 返回 JSON 元数据，图片内容只通过 media route。

### 6. Front Auto 交互

- 初始为空态，打开按钮调用后端选择器；测试可通过 `open_path`。
- Left/Right 或滚轮非 Ctrl：上一张/下一张；Home/End：首/末；Esc：退出全屏或关闭面板。
- Ctrl+wheel：以 pointer 为锚点指数缩放，范围 0.05×–64×。
- 拖拽更新 offset；双击在 fit-window 与 1:1 间切换。
- `1/2/3/Space` 对应 fit-window/fit-width/1:1/循环 fit。
- `R/Shift+R` 只改变显示旋转，不写源文件。
- zoom 更新后开启 80ms 条件 timer，只提交最新 `view_revision`。
- front model 只保存字符串、数字和轻量 record，不保存图片字节。

### 7. `ImageSurface`

Schema 和生成器统一支持：

- props：`src str`、`fit str`、`zoom float`、`offset_x/y float`、`rotation int`、
  `filter str`、`alt str`；
- events：`onload(width,height,revision)`、`onerror(code,message,revision)`、
  `onwheel(delta_y,x,y)`、`onpan(dx,dy,phase)`、`ondblclick(x,y)`；
- Vue 输出只含媒体 URI 绑定、transform 和输入归一化；
- VM/Rust 共享 Iced custom widget；
- GPUI 提供普通 image 降级以保持 feature 编译，不纳入性能验收。

### 8. 图片处理

- header/EXIF 先于完整 decode；像素上限默认 400 MP，文件上限默认 1 GiB。
- fit rendition 以 viewport×device-scale 为目标，保持长宽比。
- 1:1/高倍 zoom 才请求原始像素级 rendition。
- interactive 阶段只变换已上传纹理；settled 阶段后台高质量缩小并无闪烁替换。
- checked arithmetic 防止宽高与缓冲长度溢出；源文件始终只读。

### 9. 可观测与性能

统计包括队列、运行、完成、丢弃、cache bytes、hit/miss/eviction、各阶段耗时、session、
asset、worker、最大 UI thread stall。release harness 输出 JSON，失败时保留采样和最后状态。

性能门禁：空窗口首次可见 P95≤250ms；24MP JPEG 冷 fit 首帧 P95≤500ms；预取邻图
切换 P95≤100ms；100 次导航只有最终 generation publish 且队列≤8；静置 10 秒 CPU≤1%；
decoded 常态≤`256 MiB + current`；关闭 2 秒内资源归零。

## 测试设计

### 单元与属性测试

- registry 状态迁移、opaque id、TTL、引用与并发读取；
- LRU 字节记账、硬/软 pin、超预算 current；
- priority、去重、队列上限、generation/revision latest-wins；
- EXIF orientation、尺寸交换、缩小、透明度、损坏输入、像素/文件上限；
- GET/HEAD/ETag/304/404/410/422/503 与安全路径；
- `ImageSurface` schema、VM builder、Rust/Vue generator、Iced geometry/events；
- `auto.image` VM 与 a2r 同输入同输出状态。

### Fixture

在 `examples/ui/031-image-viewer/tests/fixtures/` 保存小型确定性 JPEG/PNG/WebP、带 orientation
JPEG、透明 PNG 和损坏文件。24MP 性能图由 Rust harness 在临时目录确定性生成，避免向 Git
提交大文件。

### E2E

- Vue + Rust backend：标准 Playwright runner 执行动作 JSON，检查媒体响应不是 JSON/base64、
  翻图/缩放/平移/fit/旋转/错误恢复并截图。
- VM：标准 AutoUI MCP runner 启动，场景脚本检查状态与截图。
- Rust merged：生成并运行原生 Rust app，以相同 fixture 和状态断言验证。
- 三端比较语义状态、asset generation 和关键几何；像素截图只做人工/阈值视觉检查。

### 门禁等级

本计划修改 compiler/UI/runtime/API 协议，属于 Category B + 核心协议重构：开发期间使用
`cargo check -p auto-lang` 和 scoped tests；复审前只运行一次 `cargo tf`。只有实际修改
Schema/AURA 参考后才运行 `docs_gen`，本计划因新增 schema widget 必须运行一次。

## 验收标准

- [ ] `examples/ui/031-image-viewer` 是独立 full-stack AutoUI app，`029-photo-gallery` 行为未变。
- [ ] front/back 均为 Auto 源码，生成产物没有人工修改。
- [ ] Vue、VM、Rust merged 三种模式启动并完成同一核心交互矩阵。
- [ ] Vue 前端没有 File API、目录扫描、图片处理、预取队列或应用级图片 cache 实现。
- [ ] 图片不经过 JSON/base64/`[]int`；Vue 使用 media GET，VM/Rust merged 命中进程内资产。
- [ ] `/api/__auto/media/...` 通过 GET/HEAD/ETag、MIME、状态码和路径安全测试。
- [ ] UI thread 上无文件 I/O、目录扫描、decode、encode 或 settled resize。
- [ ] current-first、邻图预取、两个 decode worker、一个 resize lane、queue≤8、latest-wins
  和 80ms settle 均有自动化证据。
- [ ] encoded/decoded/handle cache 有容量与生命周期，不新增进程级无界图片 HashMap。
- [ ] JPEG/PNG/WebP 与 EXIF orientation 正确；损坏/超限/权限错误显示稳定错误态。
- [ ] ImageSurface props/events 在 Vue generator、VM builder、Rust generator 和 Iced renderer
  语义一致；GPUI feature 可编译降级。
- [ ] Rust merged release 达到本计划性能预算并生成机器可读报告。
- [ ] Vue/VM/Rust 各至少保存初始、缩放和平移/旋转后的真实截图。
- [ ] `cargo check -p auto-lang`、scoped tests、`docs_gen` 和最终一次 `cargo tf` 通过；无新增
  warning、debug print、生成物或后台进程残留。
- [ ] 设计索引、示例轨道、示例 SPEC/README 和媒体契约说明与实现一致。

## 执行步骤

### Task 1：建立 feature、依赖和模块骨架

- 文件：`crates/auto-lang/Cargo.toml`、`crates/auto-lang/src/ui/mod.rs`、
  `crates/auto-lang/src/ui/image_pipeline.rs`、`crates/auto-lang/src/ui/iced/mod.rs`、
  `crates/auto-lang/src/ui/iced/image_surface.rs`。
- 操作：新增 `image-pipeline` feature、EXIF 可选依赖、空模块和公开导出；`ui-iced` 启用该
  feature，默认无 UI 构建不引入图片依赖。
- 验证：`cargo check -p auto-lang --no-default-features; cargo check -p auto-lang --features ui-iced`。

### Task 2：定义媒体资产类型与状态机

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`。
- 操作：实现 `MediaAssetId/Key/State/Metadata/RenditionSpec/Error/Stats`，并加入状态迁移和
  checked-size 单测。
- 验证：`cargo test -p auto-lang image_pipeline::tests::asset_state --lib --features ui-iced`。

### Task 3：实现进程内 MediaAssetRegistry

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`。
- 操作：实现不可猜测 id、revision、Arc 内容、wait/notify、引用、TTL、404/410 区分和 shutdown。
- 验证：`cargo test -p auto-lang image_pipeline::tests::registry --lib --features ui-iced`。

### Task 4：实现 encoded 字节预算 LRU

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`。
- 操作：实现 64 MiB 默认 encoded LRU、访问更新、去重记账、eviction 和指标。
- 验证：`cargo test -p auto-lang image_pipeline::tests::encoded_lru --lib --features ui-iced`。

### Task 5：实现 decoded cache 与硬/软 pin

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`。
- 操作：实现 256 MiB decoded budget、current 硬 pin、neighbor 软 pin、current 临时超预算及
  publish 后回收。
- 验证：`cargo test -p auto-lang image_pipeline::tests::decoded_budget --lib --features ui-iced`。

### Task 6：实现有界优先队列

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`。
- 操作：实现 priority 100/90/10/5、稳定序号、总上限 8、同 key 去重和 neighbor 上限 2。
- 验证：`cargo test -p auto-lang image_pipeline::tests::priority_queue --lib --features ui-iced`。

### Task 7：实现 generation/revision latest-wins

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`。
- 操作：在 enqueue、decode-complete、publish 三处应用 session generation/view revision gate，
  记录 dropped 统计。
- 验证：`cargo test -p auto-lang image_pipeline::tests::latest_wins --lib --features ui-iced`。

### Task 8：实现 worker 生命周期

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`。
- 操作：建立两个 decode worker 和一个 resize lane，支持空闲阻塞、唤醒、关闭与 join，禁止
  固定高频轮询。
- 验证：`cargo test -p auto-lang image_pipeline::tests::worker_shutdown --lib --features ui-iced`。

### Task 9：实现 header、格式和 EXIF 读取

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`、`crates/auto-lang/tests/fixtures/images/*`。
- 操作：识别 JPEG/PNG/WebP，读取尺寸/orientation，拒绝 >1 GiB 或 >400 MP，并归一错误码。
- 验证：`cargo test -p auto-lang image_pipeline::tests::metadata_and_limits --lib --features ui-iced`。

### Task 10：实现 decode、orientation 与 rendition

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`。
- 操作：解码静态图、应用 orientation、保持 alpha、生成 viewport rendition 和原尺寸 rendition，
  缓存键包含 orientation/rotation/尺寸/quality。
- 验证：`cargo test -p auto-lang image_pipeline::tests::decode_and_rendition --lib --features ui-iced`。

### Task 11：实现媒体 HTTP 响应器

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`。
- 操作：实现路径解析与 GET/HEAD/ETag/304/404/410/422/503 响应，输出 MIME、长度、immutable
  cache header，且 URL/日志不泄漏路径。
- 验证：`cargo test -p auto-lang image_pipeline::tests::http_response --lib --features ui-iced`。

### Task 12：把媒体路由挂到生成的 Rust backend

- 文件：`crates/auto-man/src/api_gen.rs`。
- 操作：在 full-cover 和 stateful 两种 Axum main 模板中挂载 `/api/__auto/media/{id}/{revision}`
  GET/HEAD，并为生成 backend 添加公共 runtime feature 依赖。
- 验证：`cargo test -p auto-man media_route --lib`。

### Task 13：把媒体路由挂到 VM HTTP server

- 文件：`crates/auto-lang/src/vm/ffi/stdlib.rs`、
  `crates/auto-lang/src/vm/ffi/axum_adapter.rs`。
- 操作：在普通 VM HTTP server 与 axum adapter serve 路径中优先分派框架媒体 route，保持
  既有用户 routes 不变。
- 验证：`cargo test -p auto-lang image_pipeline_vm_http --lib --features ui-iced`。

### Task 14：实现 native media URI 解析

- 文件：`crates/auto-lang/src/ui/image_pipeline.rs`、
  `crates/auto-lang/src/ui/iced/renderer.rs`。
- 操作：让 `/api/__auto/media/...` 在 VM/Rust merged 中先查 registry，只有 split/native 无命中
  时才允许 HTTP fallback；普通相对文件和网络 URL 保持兼容。
- 验证：`cargo test -p auto-lang image_pipeline::tests::native_uri --lib --features ui-iced`。

### Task 15：声明 `auto.image` 标准库接口

- 文件：`stdlib/auto/image.at`、`stdlib/auto/image.vm.at`、`stdlib/auto/image.rs.at`、
  `stdlib/auto/prelude.at`。
- 操作：声明 queue/open/scan/request/retain/release/close/stats 的轻量 ticket API；接口只返回
  scalar/record/URI，不暴露 pixels。
- 验证：`cargo test -p auto-lang native_registry --lib`。

### Task 16：实现 VM `auto.image` natives

- 文件：`crates/auto-lang/src/vm/ffi/stdlib.rs`、
  `crates/auto-lang/src/vm/native_registry.rs`。
- 操作：把 `.vm.at` 声明映射到公共 image pipeline，完成参数检查、record 编码和稳定错误。
- 验证：`cargo test -p auto-lang image_natives --lib --features ui-iced`。

### Task 17：实现 a2r `auto.image` 适配

- 文件：`stdlib/auto/image.rs.at`、`crates/auto-man/src/rust_ui.rs`、
  `crates/auto-man/src/api_gen.rs`。
- 操作：让生成的 Rust UI/backend 直接调用公共 image pipeline，并在 shared workspace 模板中
  启用一致 feature；不生成第二套缓存。
- 验证：`cargo test -p auto-man image_stdlib_codegen --lib`。

### Task 18：增加 VM/a2r 后端算法 parity fixture

- 文件：`crates/auto-lang/tests/image_pipeline_parity.rs`、
  `crates/auto-lang/tests/fixtures/image_pipeline_service.at`。
- 操作：以相同目录/请求序列对比 VM 与转译 Rust 的排序、generation、keep set、ticket 状态
  和统计，不比较 opaque id 字面值。
- 验证：`cargo test -p auto-lang --test image_pipeline_parity --features ui-iced`。

### Task 19：声明 ImageSurface schema/widget

- 文件：`schema/aura.at`、`stdlib/aura/widgets/display/ImageSurface.at`、
  `stdlib/aura/widgets/display/mod.at`、`crates/auto-lang/src/ui_gen/widget/registry.rs`。
- 操作：登记 props/events/primary src/backend 映射，并保持普通 `Image` 合约不变。
- 验证：`cargo test -p auto-lang image_surface_schema --lib`。

### Task 20：增加后端中立 View 节点

- 文件：`crates/auto-lang/src/ui/view.rs`。
- 操作：新增 `View::ImageSurface`、props、typed callbacks、constructor、map/debug/clone 等所有
  穷举臂。
- 验证：`cargo test -p auto-lang image_surface_view --lib --features ui-iced`。

### Task 21：接通 VM Aura builder

- 文件：`crates/auto-lang/src/ui/aura_view_builder.rs`、
  `crates/auto-lang/src/ui/dynamic.rs`。
- 操作：解析 ImageSurface 动态 props 与五类事件，生成与 View callback 一致的 VM handler
  参数和稳定 vnode id。
- 验证：`cargo test -p auto-lang image_surface_vm_builder --lib --features ui-iced`。

### Task 22：接通 Rust UI generator

- 文件：`crates/auto-lang/src/ui_gen/rust.rs`。
- 操作：生成 `View::image_surface(...)`、动态 props 和 typed messages，覆盖 literal/state/
  conditional 表达式。
- 验证：`cargo test -p auto-lang image_surface_rust_codegen --lib --features ui-iced`。

### Task 23：接通 Vue generator

- 文件：`crates/auto-lang/src/ui_gen/vue.rs`。
- 操作：生成裁剪容器、`<img :src>`、transform、load/error/wheel/pan/dblclick 事件归一化；
  生成结果不得出现 File API、decode、prefetch 或 cache 实现。
- 验证：`cargo test -p auto-lang image_surface_vue_codegen --lib`。

### Task 24：实现 Iced ImageSurface 布局与绘制

- 文件：`crates/auto-lang/src/ui/iced/image_surface.rs`。
- 操作：实现 contain/width/1:1/free geometry、rotation、clip、背景、linear/high filter 和资源
  ready/error/placeholder 绘制。
- 验证：`cargo test -p auto-lang image_surface_geometry --lib --features iced-layout-tests`。

### Task 25：实现 Iced ImageSurface 输入

- 文件：`crates/auto-lang/src/ui/iced/image_surface.rs`。
- 操作：输出 surface-local wheel 坐标、pan start/move/end、double-click、load/error 一次性事件，
  并按 src revision 丢弃旧事件。
- 验证：`cargo test -p auto-lang image_surface_events --lib --features iced-layout-tests`。

### Task 26：集成 Iced renderer 并收敛图片 cache

- 文件：`crates/auto-lang/src/ui/iced/renderer.rs`。
- 操作：渲染 `AbstractView::ImageSurface`；媒体 URI 使用 registry handle；删除该路径上的同步
  I/O；把普通 Image 的媒体资源接入有界生命周期，避免新增或保留 viewer 使用的无界 cache。
- 验证：`cargo test -p auto-lang image_surface_renderer --lib --features ui-iced`。

### Task 27：补齐其他 View 消费者

- 文件：`crates/auto-lang/src/ui/gpui/renderer.rs`、
  `crates/auto-lang/src/ui/gpui/auto_render.rs`、`crates/auto-lang/src/ui/snapshot_builder.rs`、
  `crates/auto-lang/src/ui/vnode_converter.rs`、`crates/auto-lang/src/ui/vm_bridge.rs`。
- 操作：补齐穷举臂、snapshot 属性和 GPUI 普通 image 降级，禁止静默丢节点。
- 验证：`cargo check -p auto-lang --features ui-gpui`。

### Task 28：增加 ImageSurface 跨生成器契约测试

- 文件：`crates/auto-lang/tests/image_surface_contract.rs`。
- 操作：同一 `.at` fixture 分别走 VM builder、Rust generator 和 Vue generator，逐项断言 props、
  callback 参数、fit/rotation 和 media src 保真。
- 验证：`cargo test -p auto-lang --test image_surface_contract --features ui-iced`。

### Task 29：创建 031 应用骨架

- 文件：`examples/ui/031-image-viewer/pac.at`、
  `examples/ui/031-image-viewer/src/front/app.at`、
  `examples/ui/031-image-viewer/src/back/api.at`、
  `examples/ui/031-image-viewer/src/back/image_service.at`。
- 操作：登记 media 类 full-stack app、端口和 front/back 模块，建立可生成的空态。
- 验证：`Push-Location examples/ui/031-image-viewer; auto gen; Pop-Location`。

### Task 30：实现后端数据类型与目录索引

- 文件：`examples/ui/031-image-viewer/src/back/types.at`、
  `examples/ui/031-image-viewer/src/back/directory_index.at`、
  `examples/ui/031-image-viewer/src/back/image_service.at`。
- 操作：实现 ImageEntry/ImageAsset/ViewerSnapshot/ViewRequest、canonical root、格式过滤、自然排序、
  循环索引和路径展示脱敏。
- 验证：`cargo test -p auto-man image_viewer_directory --lib`。

### Task 31：实现后端 session、调度和缓存策略

- 文件：`examples/ui/031-image-viewer/src/back/image_service.at`。
- 操作：实现 open/navigate/request_view/close/stats、generation、current-first、prev/next keep set、
  80ms settled revision 接受规则，并只调用 `auto.image` 机制 API。
- 验证：`cargo test -p auto-man image_viewer_service --lib`。

### Task 32：实现控制 API

- 文件：`examples/ui/031-image-viewer/src/back/api.at`。
- 操作：导出 open_file/open_directory/open_path/snapshot/navigate/request_view/close_session/
  image_stats，确保返回值全为 JSON metadata/ticket，媒体字节不进入 API 值。
- 验证：`cargo test -p auto-man image_viewer_api --lib`。

### Task 33：实现完整 Front Auto UI

- 文件：`examples/ui/031-image-viewer/src/front/app.at`、
  `examples/ui/031-image-viewer/src/front/image_viewport.at`。
- 操作：实现极简桌面布局、空/载入/错误态、ImageSurface、工具栏、缩略图条、信息栏、键盘、
  锚点 zoom、pan、fit、1:1、rotation、全屏和 settle timer；model 不含图片字节。
- 验证：`Push-Location examples/ui/031-image-viewer; auto gen; auto check; Pop-Location`。

### Task 34：增加确定性 fixture 与应用级测试配置

- 文件：`examples/ui/031-image-viewer/tests/fixtures/*`、
  `examples/ui/031-image-viewer/tests/vue-actions.json`、
  `examples/ui/031-image-viewer/tests/README.md`。
- 操作：加入小型 JPEG/PNG/WebP/orientation/alpha/corrupt fixtures 和三端同一测试路径说明；
  fixture 许可和生成参数写入 README。
- 验证：`cargo test -p auto-lang image_pipeline::tests::fixture_manifest --lib --features ui-iced`。

### Task 35：验证 Vue + Rust backend

- 文件：`examples/ui/031-image-viewer/tests/vue-actions.json`、
  `examples/ui/031-image-viewer/tests/screenshots/vue-*.png`。
- 操作：启动 `auto run`，使用标准 Playwright runner 验证打开、翻图、zoom、pan、fit、rotation、
  错误恢复；检查 media response MIME 且前端生成物无处理/cache 逻辑。
- 验证：`node .agents/skills/autoui-verifier/scripts/test_vue_playwright.mjs http://127.0.0.1:3031 --actions-file examples/ui/031-image-viewer/tests/vue-actions.json`。

### Task 36：验证 VM merged

- 文件：`examples/ui/031-image-viewer/tests/screenshots/vm-*.png`、
  `examples/ui/031-image-viewer/tests/README.md`。
- 操作：使用标准 AutoUI MCP runner 启动 VM，随后以 MCP keyboard/action 驱动相同核心场景，
  记录 generation/cache stats 和截图。
- 验证：`python .agents/skills/autoui-verifier/scripts/test_vm_mcp.py --app-dir examples/ui/031-image-viewer --initial-screenshot vm-initial --save-dir examples/ui/031-image-viewer/tests/screenshots`。

### Task 37：验证 Rust merged 生成与原生行为

- 文件：`examples/ui/031-image-viewer/tests/screenshots/rust-*.png`、
  `examples/ui/031-image-viewer/tests/README.md`。
- 操作：运行 `auto run -r rust --server rust --merged`，以同一 fixture 验证进程内 media 命中、
  无 HTTP backend、交互状态和资源关闭；不得编辑生成的 `examples/rust-workspace/031-image-viewer`。
- 验证：`Push-Location examples/ui/031-image-viewer; auto run -r rust --server rust --merged; Pop-Location`。

### Task 38：实现 release 性能 harness

- 文件：`examples/ui/031-image-viewer/tests/perf_release.ps1`、
  `examples/ui/031-image-viewer/tests/perf_expectations.json`。
- 操作：确定性生成 24MP JPEG，构建/启动生成的 release exe，执行冷开、邻图、100 次导航、
  10 秒 idle、close，采集时间/CPU/RSS/queue/cache/shutdown JSON 并按预算失败退出。
- 验证：`powershell -ExecutionPolicy Bypass -File examples/ui/031-image-viewer/tests/perf_release.ps1`。

### Task 39：补齐示例与设计文档

- 文件：`examples/ui/031-image-viewer/SPEC.md`、`examples/ui/031-image-viewer/README.md`、
  `docs/design/autoui/image-viewer-pipeline.md`、
  `docs/design/autoui/examples-app-track.md`、`docs/design/autoui/README.md`、
  `docs/design/00-intro.md`。
- 操作：以实际实现回填运行命令、三形态差异、API/媒体契约、性能报告和应用轨道；删除与实现
  不符的设计声明。
- 验证：`rg -n "031-image-viewer|image-viewer-pipeline|/api/__auto/media" examples/ui/031-image-viewer docs/design/autoui docs/design/00-intro.md`。

### Task 40：执行 scoped 健康检查

- 文件：本计划全部 Rust/Auto/TS/测试改动。
- 操作：运行 rustfmt、Auto 检查、auto-lang/auto-man scoped tests，扫描 warning、debug print、
  base64/RGBA 入 Auto state、生成物和遗留后台进程。
- 验证：`cargo fmt --all -- --check; cargo check -p auto-lang; cargo test -p auto-lang image_pipeline --lib --features ui-iced; cargo test -p auto-man image_viewer --lib`。

### Task 41：执行 schema/docs 与最终全量门禁

- 文件：`schema/aura.at`、文档生成输出和全仓测试。
- 操作：因本计划修改 schema，运行一次 docs_gen；随后按核心协议变更门禁只在收尾运行一次
  `cargo tf`，记录任何 master 基线红的独立复现证据。
- 验证：`cargo test -p auto-lang --test docs_gen; cargo tf`。

### Task 42：完成执行态审计并交接独立复审

- 文件：`docs/plans/547-image-viewer-pipeline.md`、实现 worktree 全部 diff。
- 操作：逐项核对验收标准，扫描遗漏/延后/workaround、未完成标记、debug、未跟踪生成物和残留
  进程；记录证据，把状态置为 `execution_done`，不自行置为 reviewed 或合并。
- 验证：`git diff --check; git status --short; rg -n "TO.DO|FIX.ME|HA.CK|workaround|dbg!|println!" crates/auto-lang/src/ui/image_pipeline.rs crates/auto-lang/src/ui/iced/image_surface.rs examples/ui/031-image-viewer`。

## 复审记录

待 `/auto-plan:review` 在执行完成后独立填写。复审必须重新运行验收，不采信执行步骤中的
勾选状态，并检查遗漏、未经批准的延期、生成产物手改和旁路实现。

## 待澄清事项

无。下列决策已在本轮确认并固定：

1. 新建 `031-image-viewer`，不扩展 `029-photo-gallery`。
2. Vue 是必须可运行的 AutoUI 目标；本次性能重点为 VM/Rust，最终指标以 Rust merged release。
3. 图片加载和处理统一写在 Back Auto，Vue 不实现浏览器图片处理管线。
4. 控制面使用生成 JSON API，像素使用框架媒体 route；不在本计划扩展通用 BinaryResponse。
5. 首期只做静态 JPEG/PNG/WebP 与 EXIF orientation，其余格式和编辑功能不进入范围。
