# PLAN-811 handoff — 媒体宿主域、资源生命周期与跨端消费整体解耦

stage: work | plan_id: PLAN-811 | plan_revision: 1 | worktree:
`D:/autostack/.wt/lang-811/auto-lang` | branch: `codex/plan-811-dev` | base:
`ab08955c241f6a824091c53d3b02894ca17e193b` | code HEAD（送审冻结）:
`0f2ab56a9`（10 commits：4f1ec0a00→0f2ab56a9）| 依赖树：组内兄弟
`D:/autostack/.wt/lang-811/auto-down`（detached @c39ac7f9，a2r-actor-tests 的
`../auto-down` 解析需要；零源改动）。

## 1. 交付概览

新 `crates/auto-media-core`（workspace 成员，域=lang-core，闭包=
image 0.25/kamadak-exif 0.6/blake3/fastrand/thiserror + std；零
auto-lang/auto-man/auto-ui/auto-vm/iced）承接：

- **image_pipeline**（数据面+运行时，逐字迁移自
  `crates/auto-lang/src/ui/image_pipeline.rs` 2432 行）：公共类型
  （MediaAssetId/Key/Ticket/State/Lookup/Error/Metadata/RenditionSpec/
  Stats）、registry（queue/transition/publish_ready/fail/lookup/retain/
  release/collect_expired/wait_for_terminal/shutdown，512 上限+去重+TTL）、
  EncodedByteLru(64MiB)/DecodedPixelCache(256MiB, Hard/Soft/None pin)、
  MediaPriorityQueue(容量8/去重/neighbor≤2)、MediaLatestWins 三门、
  2 decode+1 resize worker（sync_channel(8)）、目录 scan、host-owned
  会话表（32×256）、media_http_response（200/304/404/410/422/503+ETag/
  cache-control/HEAD 无 body）、resolve_media_uri/pixels/render、
  media_surface_status（notify 契约）、EXIF/metadata/limits
  （1GiB/400MP/SizeOverflow）、decode_rendition/checked_rgba_bytes。
- **新增聚合 `MediaRuntime`**（计划 §2 owned runtime）：registry+worker+
  会话表单 owner；`global_media_runtime()` 为进程唯一共享实例（旧
  `global_media_registry()/global_media_worker_pool()` 派生自它——不存
  在第二组 LazyLock）；owned 实例隔离票据/会话空间；
  `shutdown()/drop` 停新任务+join 线程+registry 全终态化
  （pending_count→0，迟到 publish 被状态机拒绝——关闭无复活）+清会话表。
  修入的缺口：旧 `MediaWorkerPool::shutdown` 单独调用不终态化 registry
  （合同基线 §10 D2 记录）；resize channel 满时 `send(Stop)` 排队尾有界
  收敛（D5，测试固化）。
- **media_service**（逐字迁移自 `ui/media_service.rs` 578 行）：12 扩展
  白名单、depth-32 递归、symlink_metadata 不跟随 reparse、blake3 token、
  自然排序、resolve_root 序（explicit→env）、content_type 表、单 range
  StreamPlan（后缀/clamp/416/多 range 回退 Full）、display/artist 解析。

**旧路径薄门面（类型身份不变）**：`auto_lang::ui::image_pipeline`（26 行
pub use）与 `auto_lang::ui::media_service`（14 行 pub use）——每个重导出
即 core 同名项（TypeId 断言钉住）；`ui/mod.rs` 的 feature 门
（image-pipeline/无门）原样保留。门面新增重导出仅
MediaRuntime/global_media_runtime（加法面）。

**真实消费者切换（直调 core 公开入口）**：VM stdlib 17 shim+票据表+
HTTP 快路径（25 处引用）、back_proxy 媒体字节路由+媒体库面、Iced
renderer 19 处钩子（纹理键改用 opaque MediaAssetId——u128 字段保持
core-private）、Man api_gen（MEDIA_SERVICE_HANDLERS+会话 handler）与
rust_ui（viewer 桥 8 函数）模板、生成 manifest 五处（workspace 表/
交付 app/交付闭包 DELIVERY_CRATES 14→15/runtime_deps/三个 e2e 档）。

## 2. 冻结面核对（对合同基线 §2-§5）

- 17 native：16 固定 ID 2960-2975 精确核对；**thumb 无固定 ID**
  （NATIVE_ID_MAP 无此键，按名注册动态分配——测试钉住）；注册门
  `ui-iced` 不变；`image.vm.at` 17 声明与 IMAGE_NATIVE_NAMES 一致。
  request 旧 stub（pop 宽高、原样返回 handle）以真 shim 调用断言钉住。
- feature 语义：no-ui-iced 构建（--no-default-features --features
  with-file-history）编译通过=名字不存在（不给原 VM 新增图像能力）；
  image-pipeline 门继续控制 codec lane（auto-media-core/image-codec）。
- HTTP oracle：503/200/304/HEAD/404/410/422 逐项（core+auto-lang+
  Man consumer 三处测试）；VM 快路径 prefix 截获+query 剥离+非媒体路由
  不受影响（stdlib 测试重放）。
- 预算/上限：64MiB/256MiB/8/512/256/32×256/1GiB/400MP 逐字迁移并有
  定向测试（含 hard pin 临时越限、soft 先逐、budget 例外）。

## 3. 验证证据（owner_stage=work，全部实际执行）

| 项 | 命令/入口 | 结果 |
|---|---|---|
| crate 编译 | `cargo check -p auto-media-core -p auto-lang -p auto-man` | 0 error；auto-media-core 0 warning |
| 无 ui-iced 编译 | `cargo check -p auto-lang --no-default-features --features with-file-history` | 通过 |
| core 单测 | `cargo test -p auto-media-core --lib` | **30/30**（数据面/缓存/调度/门/codec/EXIF/限界/HTTP/运行时/会话/生命周期反例×4/junction reparse） |
| VM/门面/native | `cargo test -p auto-lang --lib plan811` | **7/7**（剪断锚/ID 冻结/TypeId 同一 service/17 shim 真调用/会话族/thumb） |
| 媒体族定向 | `cargo nextest -p auto-lang --lib image` / `-E 'test(media)'` | 20/20、22/22 |
| parity | `cargo nextest -p auto-lang --test image_pipeline_parity` | PASS |
| back_proxy 真TCP | `cargo nextest -p auto-lang --lib --features test-http-e2e http_e2e_back_proxy` | 19 run：媒体面 **4/4**（native_media_no_root/runtime_native_media_add_remove/native_media_routes/media_uri_byte_fidelity）+031 native-ns（真 VM `use auto.image` 会话）；**1 预存红**=`real_routes_corpora_data_face` 基线 ab08955c2 同失败（018-book-reader 路由参数，非媒体，已基线重放归因） |
| Man 消费方 | `cargo test -p auto-man --test plan811_media_consumers` | **3/3**（真编译实跑 core：索引/Range/registry/HTTP/owned runtime/真解码/关闭终态化+生成面锚+DELIVERY_CRATES） |
| api_gen 生成面 | `cargo nextest -p auto-man --lib api_gen` / `delivery` / `rust_ui::tests` | 50/50、9/9、40 PASS+2 预存红（crud_fallback/shell_pack freshness——基线同款同模式，非本计划回归） |
| 777 | `python scripts/modularity-audit.py --mode strict --no-cargo-check` | **verdict=pass**（基线刷新 @fa89814bb561：declared_edges 65、core_to_ui 文本候选 911→882） |
| 792 | `python test/fixtures/modularity/generated-ui-consumer/driver.py` | **25/25，exit 0**（V01 fresh-lock 真构建/V02 A→B 重定位 locked/V03 负控×7+恢复/V04 dev 模式与产物同源；consumer-a/b 交付根含 crates/auto-media-core） |
| 独立 consumer | `cargo test --manifest-path test/fixtures/modularity/media-core-consumer/Cargo.toml` + `verify-deps.sh` | 3/3 + closure PASS（闭包仅 auto-media-core 族） |

**V-05 pending（如实登记，不标 pass）**：浏览器实机走查（autoui-verifier
`test_vue_playwright.mjs` 对 031 Vue split + `test_vm_mcp.py`）本会话未
执行——需要可交互显示/浏览器环境与较长 app 生命周期；解锁动作：在具备
环境的会话跑两脚本对照 Loaded/error/换源/卸载事件。数据面等价证据已由
真 TCP（back_proxy th 4/4）+ 792 真构建 + api_gen 编译 oracle 覆盖；
Iced 渲染后事件由 renderer notify 单测面（media_surface_status/代次/泵
门）覆盖。**Man test-http-e2e wire 档在 master 基线即编译红**
（generate_api_rs 签名漂移+plan734 tauri fixture，基线 25 err / 本树
8 err 全预存）——非本计划引入，未代修（超范围）。

## 4. 净迁出/删除/剩余账目

- 迁出：`ui/image_pipeline.rs` 2432 行 → core `image_pipeline.rs`
  2762 行（+330：MediaRuntime 聚合 ~250 行 + 生命周期反例测试×4）；
  `ui/media_service.rs` 578 行 → core 620 行（+42：junction 反例+pid+seq
  scratch 修正）。原路径残 40 行纯 pub use 门面（类型身份同一）。
- UI 目录剩余媒体相关：`photo_service.rs` 760 行（029 图片库镜像实现，
  **R04 owner**，合同基线 §9）、`mpv/*`（播放器，R04）、Iced renderer
  纹理/事件投影（按 SD-04 留 UI）。
- VM 侧：stdlib 媒体段仅剩 shim 参数 pop/栈语义（SD-02 口径的 VM 职责），
  无 UI 类型/业务规则重复。
- 净效果：语言仓 UI 目录不再拥有通用媒体机制；非"生态净删"——算法
  迁移单源化（777 core_to_ui 文本候选 911→882）。

## 5. SD 草案（merge 时沉淀 canonical；本轮不私改）

- **SD-01 add `docs/specs/auto-media-core/project.md`**：crate 使命/公开面
  （image_pipeline 全 API+MediaRuntime+media_service）/资源合同（预算表、
  1GiB/400MP、TTL 双值 30s 全局/2s 常量）/生命周期合同（shutdown 终态
  化、无复活、满队列有界收敛、锁序：无用户回调/无 registry 锁内等
  worker）/发行合同（image-codec feature、闭包断言）。
- **SD-02 modify `vm/overview.md`**：17 native+HTTP 快路径消费
  auto-media-core（直调，非 UI 门面）；ID/按名注册/feature 门/stub 语义
  冻结表；控制面不搬像素/路径。
- **SD-03 modify `ui/overview.md`**：image_pipeline/media_service 迁 core、
  门面身份同一；renderer/photo_service/mpv/i18n/frame 等 UI 剩余 owner
  真实列明（不虚报全 UI 独立）。
- **SD-04 modify `ui/design/image-surface-events.md`**：规则不变
  （Ready 后单发/形参投影/换源退订/A→B→A/50ms 代次）；registry/代次/
  pending 门 owner 改 core，事件与渲染拍留 UI。
- **SD-05 modify `auto-man/project.md`**：api_gen/rust_ui 媒体生成直调
  core；生成 manifest 携带 auto-media-core（workspace/path/交付闭包）；
  项目模型/UI 工具搬迁仍未发生（809/R04 边界）。
- **SD-06 modify `repository-boundaries/generated-ui-distribution.md`**：
  DELIVERY_CRATES +auto-media-core（image/exif/blake3/fastrand 走
  registry）；受影响 profile 的 fresh/locked 合同由 792 钉住。

## 6. 环境与工具

Git Bash on Windows 10.0.26200；cargo 1.9x（stable-x86_64-pc-windows-msvc）
+ cargo-nextest（别名按 `.cargo/config.toml`）；真 TCP 测试用
test-http-e2e feature（OS 临时端口）；792 driver 产物在
`D:/autostack/.wt/lang-811/scratch-consumer/`（收据 driver-receipt.json）。
测试内**未**创建任何 worktree 内 symlink/junction（junction 反例在
`std::env::temp_dir()` 下创建并清理）。

## 7. 遗留与边界

- R04 交接（不丢弃）：photo_service、mpv、i18n、a2r 直接 frame、播放器/
  纹理 renderer、浏览器实机走查、Man wire 档基线编译红修复。
- merge 门（pending，属 merge 阶段）：冻结提交上裸 `cargo t` + `cargo tv`
  （VM 触面）；tf 归主检出批量档。
- 已知非阻塞：global registry TTL=30s 与 DEFAULT_TTL=2s 并存（历史双值，
  合同基线 §2.2 冻结为代码事实）；设计差额 6 项登记未补实现（§8）。

## R1 修复轮增补（2026-10-19，review-r1 needs_fix 后）

修复细节与逐 finding 闭合表见 [fix-r1.md](fix-r1.md)；要点：

- 代码基线推进：`0f2ab56a9` → **`33fc23208`**（`c66ca5a6a` 生命周期/准入/
  格式 + `33fc23208` 生成面两修复）。
- §3 表修正：core 单测 30/30 → **34/34**（新增 5 个关闭 oracle，重写 1 个
  为真 PNG+外截止）；「V-05 pending」**撤销**——四形态真机收据全绿
  （VM/Iced 9/9、Vue split 7/7、Rust merged 6/6、生成媒体 HTTP 13/13，
  见 [v05-receipts/](v05-receipts/)）；777/792 收据绑定 `0f2ab56a9`，
  修复轮未触其判定面（模板两行级修复经 api_gen 50/50 + delivery 9/9 +
  rust_ui 40 PASS 复验；merge 前在冻结提交复核 strict 一次）。
- 「shutdown 无复活/满队列有界收敛」等 §5 SD 草案措辞在修复后为真实
  （dead-ticket 准入语义补入 SD-01/02 草案：关闭后 queue 返回死票据、
  open_session 返回空串——公开签名不变）。
- 新增 R04 具名交接：merged 模式 ImageSurface loaded/error 事件分发
  （sweep 未接 devtools_update，PLAN-095 扩展）；sweep 静止期绑定边界；
  031 生成后端 types.at companion/`crate::image` 发射破损（master 基线
  同款，归因收据在 fix-r1）；Man wire 档基线编译红。

## 8. 下一步

T-09 review（同树独立上下文审 AC-01..06/SD/生命周期/真消费者/旧路径）；
review pass 后 T-10 merge（统一门+冻结+沉淀）→ T-11 归档清理（guard
clean 后移除 worktree/分支/组目录，含 auto-down 兄弟）。
