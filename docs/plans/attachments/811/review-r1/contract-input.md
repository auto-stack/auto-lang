# PLAN-811 T-01 合同基线（contract-baseline）

冻结迁移前实现读写图、旧 oracle、资源预算、native 实际注册事实、设计差额、
owned-runtime 边界决策与真消费者命令矩阵。所有行号/hash 绑定基线
`ab08955c241f6a824091c53d3b02894ca17e193b`（worktree
`D:/autostack/.wt/lang-811/auto-lang`，分支 `codex/plan-811-dev`）。

## 1. 基线身份与源 hash

| 文件 | SHA-256（前16） | 规模 |
|---|---|---|
| `crates/auto-lang/src/ui/image_pipeline.rs` | `485bcacca4acfae9` | 2432 行（含内联测试） |
| `crates/auto-lang/src/ui/media_service.rs` | `145a5b90884a82a1` | 578 行 |
| `crates/auto-lang/src/vm/ffi/stdlib.rs` | `1b9b0ac0b8e85798` | 13900 行（媒体相关见 §4/§5） |
| `crates/auto-lang/src/vm/native_catalog.rs` | `289167c761d0ff33` | 3163 行 |
| `crates/auto-lang/src/ui/iced/renderer.rs` | `3e7b379426ce53ea` | ~33900 行（媒体钩子见 §6.3） |
| `crates/auto-lang/src/back_proxy.rs` | `e02ca697edaa96a7` | — |
| `crates/auto-man/src/api_gen.rs` | `5e6bbf67b1db4771` | — |
| `crates/auto-man/src/rust_ui.rs` | `0439a519f258bdae` | — |
| `stdlib/auto/image.vm.at` | `30129add6bb08c4b` | 17 个 `#[vm]` 声明 |
| `crates/auto-lang/Cargo.toml` | `a2e5f088ac91e17e` | feature 表见 §2.4 |

依赖版本（自 `crates/auto-lang/Cargo.toml` 取）：`image 0.25`（optional）、
`kamadak-exif 0.6`（optional）、`blake3`（workspace，无条件）、`fastrand 2.4.1`、
`thiserror`（workspace）。`serde_json` 仅 image_pipeline 内联测试使用（dev 语义）。

## 2. image_pipeline 完整读写图

### 2.1 公共类型（迁 core 后类型身份唯一）

| 类型 | 当前形态 | 备注 |
|---|---|---|
| `MediaAssetId(pub(crate) u128)` | 32-hex Display | 迁 core 后 `pub(crate)` 保持，core 内测试可构造；auto-lang 侧仅经 ticket 获取 |
| `MediaAssetKey` | fingerprint/orientation/rendition/revision，全 pub 字段 | 缓存去重键 |
| `MediaOrientation` | 8 变体 enum + Default=Normal | EXIF 归一 |
| `MediaAssetState` | 8 状态机 + `can_transition_to` + `is_terminal` | 状态转移表冻结（Queued→Reading→Decoding→Transforming→Ready；任意→Error/Stale/Expired；Ready/Error/Stale→Expired） |
| `RenditionSpec` | w/h/rotation/quality/original_pixels + `viewport()`/`original()` | |
| `MediaMetadata` | w/h/orientation/mime/byte_len | 控制面安全头信息 |
| `MediaAssetError` | thiserror 9 变体 | UnsupportedFormat/FileTooLarge/PixelLimitExceeded/SizeOverflow/Corrupt/Stale/Expired/NotReady/Rejected(String) |
| `MediaPipelineStats` | 9 计数器 | |
| `MediaAssetTicket` | {id, revision} | 唯一公共票据 |
| `MediaLookup` | Missing/Expired/Pending/Failed/Ready(Arc<[u8]>) | |
| `MediaAssetRegistry`（Clone，Arc 内部） | queue/transition/publish_ready/fail/lookup/pending_count/metadata/update_metadata/stats/retain/release/collect_expired/wait_for_terminal/shutdown/is_shutdown | |
| `EncodedByteLru` | 64MiB 默认，字节记账 LRU | |
| `DecodedPixelCache` | 256MiB 默认，MediaPin{None,Soft,Hard}，collect() 硬钉例外 | |
| `MediaPriority` | Thumbnail=5/Neighbor=10/SettledCurrent=90/Current=100 | |
| `MediaWork`/`MediaPriorityQueue` | 容量 8、去重、neighbor≤2、priority+sequence 稳定排序 | |
| `MediaLatestWins` | generation/view_revision 三门（enqueue/decode_complete/publish） | |
| `MediaWorkerPool` | 2 decode + 1 resize 线程；`with_registry`/`submit_path`/`submit_bytes`/`decoded_pixels`/`shutdown`；Drop=shutdown | resize channel `sync_channel(8)` |
| `MediaHttpResponse` | status/headers(BTreeMap)/body(Option<Arc<[u8]>>) | 传输中立 |
| `MediaSurfaceStatus` | Ready/Pending/Failed(String)/NotMedia | PLAN-095 notify 契约 |
| `RenderedImage` | w/h/rgba | `decode_rendition` 输出 |

### 2.2 进程级单例（迁 core 后仍是**唯一**实例，不允许第二份）

- `global_media_registry()`：LazyLock，`MediaAssetRegistry::new(Duration::from_secs(30))`——**注意 TTL=30s，与 `DEFAULT_TTL` 常量 2s 不同**（DEFAULT_TTL 仅用于 `Default::impl`，全局实例不用它）。
- `global_media_worker_pool()`：LazyLock，`with_registry(global_media_registry().clone())`。
- `MEDIA_SESSIONS`：LazyLock<Mutex<VecDeque<MediaSession>>>，cap 32 会话 × 256 entry。
- `MEDIA_SESSION_COUNTER`、`MEDIA_CHANGE_GEN`（代次唤醒）、`NEXT_MEDIA_WORK_ID`。
- 自由函数 API：`pending_media_count`/`media_change_generation`/`queue_media_path`/`scan_media_directory`/`open_media_session`/`media_session_snapshot`/`media_session_uri`/`media_session_names`/`navigate_media_session`/`request_media_view`/`close_media_session`/`media_session_stats`/`queue_media_thumbnail`/`media_http_response`/`resolve_media_uri`/`resolve_media_pixels`/`resolve_media_render`/`media_surface_status`/`inspect_image_metadata`/`validate_image_limits`/`decode_rendition`/`checked_rgba_bytes`。

### 2.3 关键行为 oracle（冻结）

- queue 去重命中：references+1、expires_at 清空、返回既有 id/revision；MAX_ENTRIES=512 满时只逐出 references==0 的最旧项。
- publish_ready：revision 不符→Stale；state 非 Transforming→Rejected；成功置 Ready+bytes。
- lookup：revision 不符→Missing；Ready 无 bytes→Pending；Error→Failed(entry.error)；Expired/Stale→Expired。
- release 到 0 引用→expires_at=now+ttl；collect_expired 将到期未引用项置 Expired 且清 bytes（410 可见）。
- retain 对 Expired 返回 false；对存在项 +1 引用并取消到期。
- 双 close/未知会话：`close_media_session` 返回 false；已知会话移除并释放全部 entry 票据→true。
- 会话驱逐：33rd open 驱逐队首会话并释放其票据；entry 上限 256，超限新 entry 直接释放票据不入表。
- request_media_view：index clamp、generation+1、swap ticket 后释放旧票据（不同 id 时）；会话在两段锁之间被关闭→释放新票据并返回 `{"error":"unknown_session"}`。
- populate 与 close 竞争：session 不在表→立即释放票据（无复活/无回灌）。
- media URI：`/api/__auto/media/{id:032x}/{revision}`；parse 拒绝 id≠32hex、revision 含 `/`。
- 队列满：submit 失败→`fail(Rejected("media queue is full"))`。
- 迟到代次：decode/publish 门拒绝→transition Stale。
- 硬钉：Current/SettledCurrent→Hard（可临时超预算）；Neighbor→Soft；Thumbnail→None。
- 限制：`MAX_IMAGE_FILE_BYTES`=1GiB、`MAX_IMAGE_PIXELS`=400_000_000；`(w as u64)*(h as u64)` 溢出→SizeOverflow。
- EXIF orientation 值 2..8 映射 Freeze（FlipH/R180/FlipV/Transpose/R90/Transverse/R270），缺 EXIF/其他值→Normal。
- `inspect_image_metadata` 仅接受 JPEG/PNG/WebP（mime 三值），其他→UnsupportedFormat。
- `queue_media_thumbnail`：扩展名非 4 支持或非文件→""；size clamp[16,1024]；quality 85；Thumbnail 优先级；立即 release（30s 宽限）；返回 URI 字符串。
- HTTP oracle：Ready+GET→200（content-type/length/cache-control immutable/etag `"{id}-{revision}"`）；If-None-Match 命中→304（无 body）；HEAD→无 body；Missing→404；Expired→410；Pending→503；Failed→422；路径畸形→404。**无 HEAD 专用 405/501 分支——非 GET 即无 body。**

### 2.4 feature 门

- `ui/mod.rs`：`#[cfg(feature = "image-pipeline")] pub mod image_pipeline;`（`image-pipeline = ["dep:image","dep:kamadak-exif"]`；`ui-iced` 包含 `image-pipeline`；default 含 ui-iced）。
- `pub mod media_service;` **无门**（blake3 无条件）。
- `photo_service` 同门 image-pipeline，**不在本 Plan 迁移面**（029-photo-gallery 专用索引+缩略图，镜像而非复用 media_service；留 R04 收口，见 §9）。
- `mpv/channel.rs` 仅注释引用（镜像 MediaLatestWins 语义），非消费方。

## 3. media_service 完整读写图

- `SUPPORTED_EXTENSIONS`：12 扩展名（mp4/m4v/webm/mkv/mov/avi/mp3/flac/wav/ogg/m4a/aac）——**与 image_pipeline 的 4 图像扩展（jpg/jpeg/png/webp）是两套白名单，不合并**。
- `MAX_DEPTH`=32；`collect_files` 用 `symlink_metadata` 不跟随 symlink/junction（reparse 防环）。
- `MediaEntry`{id=blake3(rel_path) hex64, name, rel_dir, relative_path(/分隔), extension(小写), bytes}；`MediaIndex`{root, entries}。
- `resolve_root`：explicit trim 非空→用之；否则 env `AUTO_MEDIA_ROOT` trim 非空；否则 None。
- 自然排序：数字段 u128 数值序、文本段小写、Number<Text；并列回退相对路径字典序。
- `find`/`entry_path`（token 反查，绝不接受调用方路径）；`content_type` 12 映射+octet-stream 默认。
- `StreamPlan`：Full(200)/Partial{start,end}(206)/Unsatisfiable(416)；`parse_range` 单 range only（多 range/未知 unit/解析失败→Full）；`bytes=N-`/`-N` 后缀/`N-M` clamp 到 file_len-1；file_len==0 一律 Unsatisfiable（除无 header→Full）；`-0`→Unsatisfiable；`("","")`→Full。
- `content_range`/`human_size`(KB/MB/GB/TB,1位小数)/`display_title`/`parse_artist_and_title`（"A - B"→(A,B)；单连字符且左非纯数字；前导音轨号→("精选音乐",题)；否则 ("本地音乐",题)）。

## 4. 17 native 实际注册与 ID 事实

- catalog 固定 ID（`native_catalog.rs:2963-2978`）：queue=2960、open=2961、scan=2962、request=2963、retain=2964、release=2965、close=2966、stats=2967、open_session=2968、snapshot=2969、current_uri=2970、names=2971、navigate=2972、request_view=2973、close_session=2974、session_stats=2975。**16 项。**
- `thumb`：**不在 NATIVE_ID_MAP**——经 `native_registry.register(name)` 动态分配（`allocate_id()` 自 100 起、跳过全部固定 ID）；**不得臆造固定 ID 或写入 catalog**（ABI 冻结）。
- 注册路径（`stdlib.rs:10208-10238`）：全部 17 项在 `#[cfg(feature = "ui-iced")]` 块内先 `BIGVM_NATIVES.register(name)`（17 名，含 thumb）再 `register_shim_by_name`。**无 ui-iced 时这些名字不存在**（不是 stub-返回错误，而是未注册）——core 独立编译不改变这一点（不给原 VM 新增图像能力）。
- `IMAGE_NATIVE_NAMES`（stdlib.rs:5053-5071）=17 项；测试断言 17 且全部 `auto.image.` 前缀。
- `stdlib/auto/image.vm.at` 声明签名（17 个 `#[vm]`）：`queue(path str) ImageTicket`、`open(ticket) ImageTicket`、`scan(path str) []ImageTicket`、`request(ticket,width int,height int) ImageTicket`、`retain(ticket) ImageTicket`、`release(ticket) void`、`close(ticket) void`、`stats() ImageStats`、`open_session(root str) str`、`snapshot(session str) str`、`current_uri(session str) str`、`names(session str) []str`、`navigate(session str,delta int) str`、`request_view(session str,index int,viewport_width int,viewport_height int,quality int) str`、`close_session(session str) bool`、`session_stats(session str) str`、`thumb(path str,size int) str`。
- shim 行为冻结（stdlib.rs:5097-5293）：
  - `queue`：pop path（VMConvertible）→ `queue_media_path(path, Current)` → handle 表插入选代 → push handle。
  - `open`：pop handle→在表→原样 push；不在→push 0。
  - `request`：**stub——pop height、width 丢弃，pop handle 原样 push**（不得升级语义）。
  - `scan`：pop path→`scan_media_directory`→push 0（int，非列表）。
  - `retain`：在表→registry.retain→push handle；否则 push 0。
  - `release`/`close`：close 移除表项；均无返回值。release 不移除表项（仅减引用）。
  - `stats`：push `registry.stats().completed as i64`。
  - `session_stats`：忽略 session 参数，返回全局 registry stats JSON 串（现行语义）。
  - `thumb`：pop size(i32) 先、pop path 后→URI 串。
  - handle 表：全局 `IMAGE_TICKET_HANDLES` VecDeque cap 256，驱逐队首并 release 其票据；counter 自 1。
  - 会话族直接转发 image_pipeline 自由函数（§2.2）。

## 5. HTTP 数据面

- VM 快路径 `media_response_for_vm_request`（stdlib.rs:5355-5371，cfg ui-iced）：prefix `/api/__auto/media/` 才截获；`split_once('?')` 去查询串；转发 `media_http_response(global_media_registry(),…)`。
- `write_media_response`（同文件 5373-5425）：非 media prefix→false（落入用户路由）；wire 格式 `HTTP/1.1 {status} {reason}\r\nContent-Length: {headers.content-length|0}\r\nConnection: close\r\n`+逐 header+空行+GET body。reason 表：200 OK/304 Not Modified/404/410 Gone/422/503，其余 "Error"。**无 ui-iced 时 write_media_response 恒 false**（用户路由不受影响）。
- `run_http_server_blocking`（5429 起）：解析 request line+headers（content-length/if-none-match）→ body → `write_media_response` 先于 routes 派发。
- back_proxy（back_proxy.rs:585-624，cfg ui-iced）：`/apps/<id>/api/__auto/media/…` 子路径命中时复用 `media_response_for_vm_request`，以 ProxyReply::Stream 直答（Arc bytes → Cursor reader），content-type 提取、其余 headers 透传。序位：native media（`ui` feature）→ native photo（`image-pipeline`）→ 字节路由（`ui-iced`）→ session 分发。
- media_service 流出口：Man api_gen `MEDIA_SERVICE_HANDLERS` 模板（api_gen.rs:3159 起 raw string）+ back_proxy `/api/media/*`（index/stream/artist，back_proxy.rs:824-890）。

## 6. 消费者清单（迁移接线面）

### 6.1 VM stdlib（§4 全部）
### 6.2 back_proxy
- `media_service::{resolve_root,index_directory,parse_artist_and_title,display_title,human_size,MediaIndex,StreamPlan…}`（151/243/391/824-890）。
- 字节路由经 stdlib 快路径函数（§5）。

### 6.3 Iced renderer（renderer.rs）
- 4370/4442：texture handle 缓存键=MediaAssetId；`resolve_media_render(&src)`。
- 8141：`resolve_media_uri(url)` 回退前查注册表。
- 10502/29837：`media_change_generation()` 代次唤醒（50ms 泵对照）。
- 25453/29839：`pending_media_count()==0` 停泵门。
- 29886-29913：`media_surface_status` → Ready（resolve+单发 loaded）/Failed（单发 error）/Pending。
- 33832-33862：测试直接构造 MediaAssetKey+registry 状态机。

### 6.4 Man 生成器（模板字符串，产物为**生成应用代码**）
- `api_gen.rs:3159` `MEDIA_SERVICE_HANDLERS`（r#"…"#）：`auto_lang::ui::media_service::{resolve_root,index_directory,find,entry_path,content_type,parse_range,content_range,MediaIndex,StreamPlan}` + `auto_lang::ui::image_pipeline::{media_http_response,global_media_registry}`（3512-3526：/api/__auto/media/ 直查 + miss 回退 404）。
- `api_gen.rs:4406-4434`：image 会话 API handler 串（8 个 `auto_lang::ui::image_pipeline::*` 调用）。
- `api_gen.rs:1326/7400/7654/7990`：生成 app Cargo.toml `auto-lang = { path=…, features=["ui","image-pipeline"] }`（workspace=true 变体 + 相对 path 变体）。
- `rust_ui.rs:1834-1862`：merged Rust UI 的 8 个媒体桥函数串（`auto_lang::ui::image_pipeline::*`）。
- `rust_ui.rs:3521/3924`：生成 Cargo `auto-lang = { path="../auto-lang", features=["ui-iced"] }` / 相对 path 变体。
- auto-man 自身编译期依赖 `auto-lang = { path="../auto-lang", features=["ui-iced"] }`（Cargo.toml:12，用于生成 handler 的实编 wire e2e 测试）。

### 6.5 测试
- `crates/auto-lang/tests/image_pipeline_parity.rs`（ui-iced 门；VM/a2r 双 registry 可观察状态一致）。
- `src/tests/back_proxy_tests.rs:1021` 引 image_pipeline 类型。
- image_pipeline/media_service 内联测试（随文件迁移，见 §7 fixture 路径注记）。

## 7. 测试夹具路径事实

image_pipeline 内联测试以 `CARGO_MANIFEST_DIR/../../examples/ui/031-image-viewer/tests/fixtures` 定位夹具。**crates/auto-media-core 与 crates/auto-lang 同深**（`crates/<name>/../..`=仓根），迁移后相对路径逐字不变即可继续命中；夹具资产（7 项 manifest）归 examples-ui（777 资产表），core dev 依赖 serde_json。

## 8. 设计差额（docs/design/autoui/image-viewer-pipeline.md "Implemented" 标签 vs 代码）

| 设计目标 | 代码现状 | 处置 |
|---|---|---|
| Metadata 缓存 4096 项 | 无独立 metadata 缓存；registry MAX_ENTRIES=512 统一记账 | 差额登记，不补实现 |
| Render handles 按 asset key 弱引用 | renderer 侧 texture Handle 缓存（UI 职责，留 renderer） | 不在本 Plan |
| 预算可环境变量覆盖 | 全部常量，无 env 覆盖 | 差额登记 |
| 无 Alpha 输出优先 JPEG/WebP | publish 存原始 encoded bytes，不转码 | 差额登记 |
| viewport×device_scale 逼近 | request_media_view 仅收 w/h，无 scale 因子 | 差额登记 |
| interactive 线性/settled 高质量过滤 | render_decoded 恒 Lanczos3 | 差额登记 |
| 优先级 100/90/10/5、队列 8、neighbor≤2、latest-wins 三门 | 一致 | 已实现，冻结 |
| encoded 64MiB/decoded 256MiB/hard-pin 例外/软 pin 先逐 | 一致 | 冻结 |
| 1GiB/400MP 拒绝边界 | 一致（validate_image_limits） | 冻结 |
| （设计未提）全局 registry TTL=30s、DEFAULT_TTL=2s、256 handle 表、32×256 会话表 | 代码事实 | 以代码为准冻结 |

## 9. 剩余 owner（本 Plan 不迁、不虚报完成）

- `photo_service.rs`（029 图片库索引/缩略图，镜像 media_service/image_pipeline 算法）→ R04 运行桥收口后随 UI 侧定夺。
- i18n、a2r 直接 frame、播放器/纹理 renderer、mpv → R04（808 handoff 残边表一致）。
- Man 项目/构建规则搬迁 → 809/R04/R06。

## 10. Service owner / owned runtime 边界决策（T-01 冻结）

**D1 默认共享 service**：`global_media_registry()/global_media_worker_pool()/MEDIA_SESSIONS` 进程单例语义保持——VM、HTTP 快路径、renderer、生成代码全部命中同一实例与同一票据命名空间。VM drop **不**触碰共享池（现状保持）。

**D2 owned runtime**：新增聚合句柄 `MediaRuntime`（core 内），持有 registry+worker+session 表：
- `MediaRuntime::new(ttl)`：局部实例，用于可控寿命与测试（隔离票据空间——两 runtime 的 ticket id 不串：id 为 128bit 随机，会话 id 为全局计数器命名空间 `image-session-N` 撞库概率≈0，但**会话表是 per-runtime**，不跨实例查询）。
- `MediaRuntime::shutdown()`：worker.shutdown()（停新任务、清队列、Stop resize lane、join 3 线程）+ registry.shutdown()（全部 entries→references=0/expires_at=now/非终态→Stale）。**当前缺口**：单独 `MediaWorkerPool::shutdown` 不终态化 registry 条目（pending_count 永不归零）——owned runtime 修此缺口；默认共享单例不提供 shutdown（进程退出回收，与现状一致）。
- 旧自由函数（open_media_session 等 13 个 §2.2 列出项）改为默认 runtime 的薄转发——签名/语义不变。

**D3 迟到发布/关闭竞争不变式**（迁移后必须有反例测试，V-02/03）：registry 状态机 + revision 比对已防复活；shutdown 后 publish_fail/publish_ready 只会得到 Expired/Rejected——迁移不得引入绕过路径（不得在 worker 内持有 entry 引用跨 shutdown 复活）。

**D4 锁序**：worker 锁序现状=state→(latest|registry|caches 各自独立短暂持锁)；`submit` 先 latest 后 state；无嵌套用户回调；迁移保持，不引入 registry 锁内等 worker（死锁红线 §5.2）。

**D5 满 channel 有界性**：resize lane `sync_channel(8)` 满时 decode worker `send` 阻塞至 lane 消费；shutdown 的 `send(Stop)` 排在队尾——join 有界于在途工作量，非死锁（lane 持续消费）。T-03 以受控测试固化（外层截止时间）。

## 11. 真消费者命令矩阵（T-06/7 验收入口）

| 形态 | 入口 | 备注 |
|---|---|---|
| VM 模式 | `auto run -r vm`（examples/ui/031-image-viewer） | ImageSurface/会话/HTTP |
| Vue split | `auto run`（031；生成后端 api_gen） | /api/__auto/media + /api/media |
| Rust merged | Man rust_ui 生成工程编译运行 | api 桥函数 |
| 生成 HTTP 真TCP | auto-man wire e2e 测试（Cargo.toml:12 注记）+ plan811_media_consumers | OS 临时端口 |
| 777 | `python scripts/modularity-audit.py --mode strict --no-cargo-check` | 基线刷新需干净树 |
| 792 | `python test/fixtures/modularity/generated-ui-consumer/driver.py` | fresh/locked/重定位/负控 |
| 独立 consumer | `test/fixtures/modularity/media-core-consumer/`（新建） | verify-deps.sh 闭包断言 |

工具环境：Git Bash on Windows（win32 10.0.26200）；cargo/nextest 按仓 `.cargo/config.toml` 别名。跨端实机走查用 `.agents/skills/autoui-verifier/scripts/`（`test_vm_mcp.py`/`test_vue_playwright.mjs`）；环境不足时记 pending，不标 pass。

## 12. 阻塞点裁定

无未决阻塞。thumb 无固定 ID（§4）、TTL 双值（§2.2）、stub 语义（§4）均按代码事实冻结；§8 差额全部登记不补实现；`MediaAssetId` 跨 crate 构造限制以「core 内测试构造 + 外部仅经票据」解决，不升 pub。
