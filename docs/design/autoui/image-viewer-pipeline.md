# AutoUI 高性能 Image Viewer 与后端媒体管线设计

> 状态：Draft（PLAN-547 设计输入）
> 日期：2026-09-04
> 范围：`examples/ui/031-image-viewer`、Auto 后端图片服务、AutoUI 媒体资产传输、
> Vue/VM/Rust 三运行形态
> 参考实现：qimgv 的加载优先级、邻图预取、latest-wins 缩放合并和工作集缓存思想；
> 不复制其 Qt/C++ 代码、资源或 UI。

## 1. 背景与决策

现有 `examples/ui/029-photo-gallery` 是图库产品样板：相册导航、过滤、排序、收藏、
缩略图网格和轻量大图预览。它的数据源是固定网络 URL，查看器只是网格应用的一个
视图分支，不承担本地目录扫描、异步解码、资源预算、快速连续翻图和高倍率缩放。

本设计决定新建独立的 `examples/ui/031-image-viewer`，不把性能管线塞入
`029-photo-gallery`。两者未来可以复用同一个 `ImageSurface` 显示契约，但产品状态、
后端职责和验收指标保持独立。

Image Viewer 必须遵守 AutoUI 的基本交付模型：

1. 前端和后端均使用 Auto 编写；
2. Vue 模式生成 Vue 前端与 Rust HTTP 后端，前后端分离；
3. VM 模式解释执行前后端 Auto，用于开发期快速迭代；
4. Rust 模式把前后端转译为 Rust，`--server rust --merged` 下进程内调用，形成最终
   独立原生进程；
5. 图片加载、目录索引、解码/缩略、预取、缓存和过期请求仲裁只在后端 Auto 服务中
   定义一次；Vue 前端不另写 File API、解码器或缓存算法。

## 2. 目标与非目标

### 2.1 目标

- 同一份 front/back Auto 源码可生成 Vue、由 VM 解释、并转译为 Rust 原生应用。
- 原生 UI 线程不执行文件读取、目录扫描、图片解码、缩略或高质量重采样。
- 首次请求立即展示窗口和占位态，当前图片优先于所有预取任务。
- 快速连续翻图和缩放采用 latest-wins；过期结果不得覆盖较新的界面状态。
- 默认只保留当前图和相邻图工作集，并由字节预算约束缓存。
- Vue 通过后端 HTTP 媒体路由读取编码图片；VM/Rust merged 通过相同媒体 URI
  命中进程内资产，不经过 HTTP、JSON 或 base64。
- 支持 JPEG、PNG、WebP 静态图，正确应用 EXIF orientation。
- 支持上一张/下一张、fit-window、fit-width、1:1、自由缩放、锚点缩放、拖拽平移、
  旋转、全屏、缩略图条、文件信息、快捷键和后端原生打开文件/目录。
- 建立可重复的 release 性能和资源占用门禁。

### 2.2 非目标

- 不在首期实现视频、RAW、HEIF/HEIC、AVIF、JPEG XL 或动画逐帧播放。
- 不实现裁剪、调整尺寸、覆盖保存、删除、复制、移动、打印、脚本和壁纸设置。
- 不实现远程上传协议或浏览器 File API；Vue 模式面向本机 Rust 后端或已配置的
  后端可访问目录。
- 不实现任意大图的磁盘级瓦片金字塔；首期保证单幅大图不产生无界重复副本。
- 不追求 Vue 与原生端内部实现一致，只要求可见行为和后端策略一致。
- 不把 Image Viewer 写成 Rust 专用应用，也不在生成产物中手改代码。

## 3. 运行形态

```text
                         同一套 Auto 源码
             ┌────────────────┴────────────────┐
             │                                 │
       src/front/*.at                    src/back/*.at
  UI/viewport/输入/状态展示        目录/加载/处理/队列/缓存策略
             │                                 │
       ┌─────┴─────┐                     ┌─────┴─────┐
       │           │                     │           │
   Vue generator  Rust UI generator    AutoVM       a2r Rust
       │           │                     │           │
       │        View::ImageSurface       │      进程内 API
       │           └──────────┬──────────┘           │
       │                      │                      │
       └─ HTTP JSON 控制面 ───┴─ /api/__auto/media ─┘
            + HTTP 媒体数据面     或进程内媒体解析
```

### 3.1 Vue + Rust split

- Vue 只调用 `api.at` 生成的控制 API，并把 `ImageAsset.src` 绑定给 `ImageSurface`。
- Vite 已代理 `/api` 到 Rust 后端；媒体 URI 继续使用相对 `/api/...`。
- 浏览器对媒体 URI 发普通 GET/HEAD；后端返回正确 MIME、长度、ETag 和缓存策略。
- 浏览器负责最终像素呈现，但没有应用级加载、缩略、预取或缓存实现。

### 3.2 VM merged

- front/back Auto 均由 VM 执行。
- 控制 API 走既有 merged 宿主桥；图片任务进入公共媒体运行时。
- `ImageSurface` 遇到 `/api/__auto/media/...` 时先查同进程媒体注册表，不开启 HTTP。

### 3.3 Rust + Rust merged

- front/back Auto 均转译为 Rust，控制 API 直接调用后端函数。
- 媒体注册表与 Iced renderer 位于同一进程，`ImageSurface` 直接取得共享资产。
- release 可执行文件不启动 HTTP 服务，也不把像素序列化为 JSON/base64。

## 4. 总体分层

### 4.1 Front Auto：展示与交互

`src/front/app.at` 保存轻量界面状态：

- `session_id`、`generation`、当前索引和文件元信息；
- `asset_src` 与 `asset_revision`，不保存编码字节或 RGBA；
- `fit_mode`、`zoom`、`offset_x/y`、`rotation`；
- 工具栏、缩略图条、信息面板、加载/错误/空状态；
- 80ms settle timer 与最新 viewport revision；
- 键盘、滚轮、拖拽和按钮事件。

Front Auto 不负责目录读取、文件格式判断、解码、缩略、缓存和任务线程。

### 4.2 Back Auto：策略唯一真源

`src/back/image_service.at` 定义：

- viewer session 生命周期；
- 目录图片过滤和确定性排序；
- 当前索引与 generation；
- current/neighbor/settled-rendition 三类请求的优先级；
- 当前图 + 前后邻图的 keep set；
- 快速翻图和缩放的过期结果判定；
- 缓存 retain/release 决策；
- 错误归一化和可观测统计。

底层线程、解码器、编码器、共享字节和 GPU handle 是运行时机制；何时请求、请求谁、
保留谁、接受哪个结果由 Back Auto 决定。

### 4.3 公共媒体运行时：机制层

新增公共 `ui::image_pipeline`，同时被 VM、Rust 生成后端、Rust HTTP server 和 Iced
renderer 使用。它提供：

- 有界优先队列和两个解码 worker；
- 内容指纹与 rendition key 去重；
- encoded、decoded、render-handle 分层缓存；
- 进程内 `MediaAssetRegistry`；
- GET/HEAD 媒体响应；
- 订阅/唤醒和指标快照；
- 丢弃过期结果和安全释放资源。

公共运行时不保存 viewer 的当前索引或业务导航规则。

## 5. 控制面与数据面

### 5.1 为什么不直接扩展普通 JSON API

现有 TypeScript API 客户端默认 `response.json()`，Rust Axum 目标默认 `Json<T>`。
若把图片放进 `[]int` 或 base64 JSON，会产生体积膨胀、大字符串、重复复制和 native
merged 模式无意义的序列化。

本设计保留 `api.at` JSON 控制面，另由生成的后端自动挂载框架媒体数据面。像素不作为
普通 Auto 值跨界。

### 5.2 媒体 URI

Back Auto 返回稳定的相对 URI：

```text
/api/__auto/media/{asset_id}/{revision}
```

- `asset_id` 是进程内生成的不可猜测标识，不包含文件路径；
- `revision` 参与缓存键，内容改变后生成新 URI；
- Vue 直接 GET；
- VM/Rust renderer 识别此前缀并先走进程内注册表；
- split native 模式若未来启用，也可回退 HTTP。

### 5.3 媒体 HTTP 语义

- 支持 GET、HEAD、`If-None-Match`；首期不要求 Range。
- 成功返回 `Content-Type`、`Content-Length`、`ETag`、
  `Cache-Control: private, max-age=31536000, immutable`。
- 未就绪的请求最多等待配置的首帧期限；超时返回 503 + `Retry-After`。
- 不存在返回 404；已过期 ticket 返回 410；解码失败返回 422。
- 响应体直接引用 `Arc<[u8]>` 或文件映射，不构造 base64/JSON。

## 6. Auto 数据契约

后端 API 使用普通可序列化记录：

```auto
type ImageEntry {
    id str
    name str
    path_display str
    width int
    height int
    file_bytes i64
    modified_ms i64
}

type ImageAsset {
    id str
    revision int
    src str
    width int
    height int
    mime str
    encoded_bytes i64
    state str       // queued | ready | error | expired
    error str
}

type ViewerSnapshot {
    session_id str
    generation int
    entries []ImageEntry
    current_index int
    current ImageEntry
    asset ImageAsset
    prev_index int
    next_index int
}

type ViewRequest {
    session_id str
    generation int
    viewport_w int
    viewport_h int
    device_scale float
    fit_mode str
    zoom float
    rotation int
    quality str     // interactive | settled
}
```

`path_display` 仅用于 UI，不能反向用作媒体读取路径；真实 canonical path 保留在后端
session 中。

## 7. 后端 Auto 状态机

### 7.1 打开

1. `open_file/open_directory` 创建新 session，并递增 generation。
2. 后端 canonicalize 用户选择，确定根目录和当前文件。
3. 扫描同目录的 JPEG/PNG/WebP，按自然文件名排序并生成 `ImageEntry`。
4. 向运行时提交 current 请求，优先级 100。
5. 提交 prev/next 请求，优先级 10。
6. keep set 更新为 current + prev + next；旧 session 资源解除 pin。
7. API 立即返回 queued asset；`ImageSurface` 自行等待媒体就绪。

### 7.2 翻图

1. generation 加一，计算循环 prev/current/next。
2. 清除尚未开始的旧 current 请求；正在解码的任务允许结束但不得 publish 为当前代。
3. 若目标已在缓存，立即发布；否则提交优先级 100。
4. 更新 keep set，邻图优先级 10。
5. 任一结果提交前比较 `(session_id, generation, source_fingerprint)`。

### 7.3 缩放与平移

- pointer/wheel 事件先在 Front Auto 更新 viewport transform，原生端只变换现有纹理，
  Vue 只变换已有 `<img>`。
- 每次 zoom 改变递增 `view_revision` 并打开 80ms settle timer。
- settle 时 Front Auto 发送 `quality = settled` 的 `ViewRequest`。
- Back Auto 只保留同 session 最新 settled revision；中间请求被覆盖。
- 新 rendition 就绪后发布新 asset revision；前端保留视觉中心并无闪烁替换。

### 7.4 关闭

- `close_session` 取消未开始任务、标记在途结果不可发布、解除所有 pin。
- 媒体条目在无引用且超过 TTL 后回收。
- 窗口关闭和后端进程退出均触发 worker shutdown，不遗留线程。

## 8. 调度与缓存

### 8.1 队列

- decode worker 默认 2 个；高质量 resize lane 默认 1 个。
- 优先级：current 100、settled current 90、neighbor 10、thumbnail 5。
- 去重键：`source_fingerprint + orientation + rendition_spec`。
- 队列最多保留 8 项；同 session 的 neighbor 最多 2 项。
- latest-wins 通过 generation/revision 在入队、解码完成和 publish 三处校验。

### 8.2 缓存

- Metadata：路径指纹和尺寸，默认最多 4096 项。
- Encoded rendition：默认 64 MiB。
- Decoded host pixels：默认 256 MiB。
- Render handles：按 asset key 弱引用；不再使用进程生命周期无界 `HashMap`。
- 当前图可突破 decoded budget 一次；发布完成后必须驱逐其他非 pinned 条目，使常态回到
  `budget + current_asset_bytes` 以内。
- 邻图是软 pin：内存压力下先于当前图淘汰。

所有预算可由环境变量覆盖，但测试和示例使用默认值。

## 9. 图片处理策略

- 首期格式：JPEG、PNG、WebP 静态首帧。
- 先读 header 和 EXIF orientation，再决定 rendition。
- fit 模式优先生成接近 `viewport × device_scale` 的显示版本，避免小窗口长期保留
  多份全尺寸 RGBA。
- 1:1 或放大到原始像素需求时才请求原尺寸版本。
- interactive 阶段使用现有纹理线性过滤；settled 阶段用高质量缩小算法。
- 旋转 90/270 度交换逻辑尺寸，缓存键包含 orientation/rotation。
- Alpha 图片保持透明；无 Alpha 输出优先 JPEG/WebP，避免无意义 RGBA 网络体积。
- 后端永不覆盖源文件。

## 10. `ImageSurface` 跨端契约

新增 Display widget `ImageSurface`：

```auto
image-surface (
    src: .asset_src,
    fit: .fit_mode,
    zoom: .zoom,
    offset_x: .offset_x,
    offset_y: .offset_y,
    rotation: .rotation,
    filter: if .settling { "linear" } else { "high" },
    alt: .title,
    onload: .ImageLoaded,
    onerror: .ImageFailed,
    onwheel: .ZoomAt,
    onpan: .PanBy,
    ondblclick: .ToggleOneToOne
) {}
```

事件语义：

- `onwheel(delta_y, x, y)`：坐标为 surface 本地逻辑像素；正值缩小、负值放大。
- `onpan(dx, dy, phase)`：phase 为 start/move/end，增量为逻辑像素。
- `onload(width, height, revision)`：当前 src 成功可见后仅发一次。
- `onerror(code, message, revision)`：失败可重试，旧 revision 事件必须丢弃。
- `ondblclick(x, y)`：由 Auto 决定 fit 与 1:1 切换。

Vue 映射为裁剪容器 + `<img>`，只负责展示、输入归一化和 transform；图片内容仍来自后端
媒体 URI。VM builder 与 Rust generator 都构造 `View::ImageSurface`，Iced 共享实现负责
原生绘制。

GPUI 暂不做性能实现，但新增 View variant 时必须提供明确的普通 Image 降级，确保 feature
组合可编译。

## 11. 安全与健壮性

- 只有后端文件选择器返回或配置根目录内的 canonical path 可进入 session。
- HTTP 媒体 URI 不接受文件路径、`..`、盘符或 UNC 路径。
- asset id 至少 128 bit 随机性，并绑定当前进程/session。
- 解码前检查文件大小、声明尺寸和像素总量；默认拒绝超过 1 GiB 文件或 400 MP 图像，
  返回可见错误而不是分配失控。
- 所有尺寸乘法使用 checked arithmetic。
- 损坏图、截断图、不支持格式和权限错误均归一为稳定错误码。
- 不在日志中输出完整用户路径；调试日志使用 basename + asset id 前缀。

## 12. 可观测性与性能预算

媒体运行时暴露只读统计：

- queued/running/completed/dropped 任务数；
- encoded/decoded cache bytes、entries、hit/miss/eviction；
- current request 的 queue/decode/publish 耗时；
- 活跃 session、asset、worker 数；
- UI thread 最大阻塞采样。

Rust merged release 在仓库参考机器上的首期门禁：

- 空窗口首次可见 P95 ≤ 250ms；
- 24MP JPEG 冷打开到 fit 首帧 P95 ≤ 500ms；
- 已预取邻图切换到可见 P95 ≤ 100ms；
- 连续 100 次导航后只允许最后 generation 成为当前图，队列峰值 ≤ 8；
- 静置 10 秒后进程 CPU 平均 ≤ 1%，无固定高频轮询；
- 非当前缓存回落到配置预算内，decoded 常态 ≤ `256 MiB + 当前图 decoded bytes`；
- 关闭 viewer 后 2 秒内 session pin、worker in-flight 和媒体引用归零。

Vue 和 VM 不套用上述绝对启动时间，但必须通过同一状态机、工作集和过期结果断言。

## 13. 测试矩阵

### 13.1 纯逻辑

- Back Auto：自然排序、循环索引、generation、keep set、latest-wins、错误归一化。
- Runtime：优先级、去重、软/硬 pin、字节预算、TTL、shutdown。
- Media route：GET/HEAD/ETag/304/404/410/422/503、MIME、无路径泄漏。

### 13.2 生成与语义 parity

- Aura schema/registry 识别 `ImageSurface`。
- VM builder 和 Rust generator 对所有 props/events 生成同一 View 语义。
- Vue generator 只生成媒体 URI 绑定与展示/输入代码，不生成目录扫描、解码或缓存逻辑。
- front/back API 在 Vue split、VM merged、Rust merged 返回同形记录。

### 13.3 应用 E2E

- Vue + Rust backend：打开目录、首图、翻图、缩放、平移、fit、旋转、错误恢复。
- VM：同场景 AutoUI MCP 驱动与截图。
- Rust merged：同场景 MCP/原生输入驱动与截图。
- 三端比较状态快照和关键几何，不要求逐像素完全相同。

## 14. 实施切片

PLAN-547 作为一个 L2 实施计划依次完成：

1. 公共媒体注册表、调度器、缓存和统计；
2. Back Auto `auto.image` 机制桥与媒体 HTTP 路由；
3. `ImageSurface` 的 schema、View、Vue、VM、Rust 和 Iced 链路；
4. `031-image-viewer` 全栈 Auto 示例；
5. 三端功能验收与 Rust merged release 性能门禁。

若实施中发现“非阻塞 Back API 调用”无法通过现有 queue/ticket 模式表达，必须回写计划的
待澄清事项并停止扩大范围；不得临时在 Rust 生成产物或 Vue 源中手写旁路。

## 15. 后续演进

- `029-photo-gallery` 可在后续计划中复用 `ImageSurface`，但不迁入本计划。
- 动画、RAW/HEIF/AVIF/JXL、色彩管理、瓦片金字塔、编辑和文件操作单独立项。
- 通用 `BinaryResponse`/Range API 可从媒体路由经验中抽象，首期不为此扩大公共 API 类型系统。
