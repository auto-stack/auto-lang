# PLAN-815 T-01 合同基线(工具/字段/资产 owner 图与 I-TOOL 冻结方案)

状态:work T-01 交付物。证据基线:worktree `D:/autostack/.wt/lang-815/auto-lang` @ master `b9f7ab7e1`(分支 `codex/plan-815-dev`)。
本文件冻结 M815-A 的 crate 布局/协议 wire/预算/退出语义,并给出 M815-B 的逐符号 owner 图与旧 oracle 清单;B 阶段执行时以本表为准,不按文件名盲搬。

## 1. 冻结决策(M815-A 适用)

| 决策点 | 冻结值 | 依据 |
|---|---|---|
| 工具包 crate | `crates/auto-ui-tools`(lib + bin,bin 名 `auto-ui-tool`) | §2 架构方案;777 域登记 `ui` |
| 协议 crate | `crates/auto-tool-protocol`(纯数据:版本/能力/预算/错误/framing) | 777 域登记 `data` |
| I-PROJECT payload | 唯一类型 `auto_project_model::GenerationRequest`;wire codec 在数据 owner(auto-project-model 新 `wire` 模块,feature `wire`)同源提供,协议/工具/客户端零复制 | §5.5"T-03 复用 809 单源数据/codec" |
| wire 形态 | JSON;`auto_val::Value` 用带标签编码(`{"t":"str","v":…}` 族),不支持变体(Fn/Widget/… )显式 `UnsupportedValue` 报错,不静默丢字段 | "有界、具名、版本化数据" |
| framing | stdio 上 4 字节大端长度前缀 + JSON 帧;双向共享 `auto_tool_protocol::framing` 单一实现 | §3"实际 codec/framing 在 T-01 probe 后冻结" |
| 控制流/日志分离 | stdout=纯控制流(帧化 JSON:request/response/event);stderr=人类可读日志。交互 stdin:A 阶段不读(interactive=false) | §5.3"日志不得污染 control stream" |
| A 阶段操作 | `describe`(能力/版本/生命周期)、`validate_project`(I-PROJECT 元数据/路径/请求约束校验);`cancel` 已定义,对 A 操作显式 `not_cancellable` | §5.5 |
| 版本策略 | `PROTOCOL_VERSION=1`;请求带版本,未知版本→错误响应、零副作用;`describe` 永远可答 | §5.3 |
| 预算 | `MAX_FRAME_BYTES=8MiB`(帧级,双向强制);`MAX_PROGRESS_BYTES_PER_OP=1MiB`;`MAX_QUEUE_DEPTH=128`(A 操作同步,结构性满足) | §5.3"消息/输出/排队预算与单位" |
| 退出码 | 0=EOF 正常关停;1=内部错误;2=流级致命(坏帧/超限帧,失步后不可恢复) | T-01 冻结 |
| 777 域 | `auto-tool-protocol`→data、`auto-ui-tools`→ui、`auto-project-model`→data(补登记,原 unclassified) | data-stays-bottom 保护协议闭包;ui-no-man-big-lib 即 AC-01 |
| B 阶段发行闭包 | `rust_ui.rs` 的 `DELIVERY_CRATES/PATCHES` 表在 T-04 迁移时增补 `auto-tool-protocol`/`auto-ui-tools`;A 阶段不触 792 管线(rust_ui 零改动) | §5.2 |

## 2. 逐符号 owner 图(M815-B 迁移依据;行号为 b9f7ab7e1 时点)

### 2.1 迁入 auto-ui-tools(UI 工程工具闭包)

| 模块 | 迁入符号 | 证据/调用闭包 |
|---|---|---|
| vue.rs(14654 行,~3700 行测试) | `VueProject` 全方法族、`build/gen/run_vue_project(_with_request)`、`I18nConfig`、gallery 系列(`start_gallery_back_proxy`/`refresh_gallery_registry`/`emit_gallery_vm_demos`/`write_registry_at`/`GalleryDemoRow`/`gallery_mode`/`desktop_mode`/`set_gallery_proxy_root`) | 调用点:automan.rs:1125..1965、rust_ui.rs:6583-6588、tauri.rs:54/91/376、vscode.rs:354、gallery_cache.rs:28;仓外无直接引用 |
| rust_ui.rs(8712 行) | `generate/build/run_rust_ui(_with_request)`、`run_vm_ui`、`generate_rust_ui_delivery(_with_request)`、`ensure_shared_workspace`、`get/resolve_rust_workspace_dir`、`write_shared_cargo_config`、`back_member_name`、`start/stop_api_server`、`start_vm_server`、`generate_shell_pack_lib(_sources)`、`collect_component_semantics`、`PacSource`(读源缝)、792 发行全套(§2.4) | automan.rs:1198..1829;**仓外符号级消费:crates/auto-man/tests/plan811_media_consumers.rs:189-194 用 `DELIVERY_CRATES/PATCHES`(迁移时同步改或保投影)** |
| jet.rs(1156 行) | `JetProject`、`generate/build/run_jet_project(_with_request)` | 仅 automan.rs 调用 |
| ark.rs(813 行) | `ArkProject`、`generate/build/run_ark_project(_with_request)` | 仅 automan.rs 调用 |
| tauri.rs(467 行) | `run_tauri_project(_with_request)` | automan.rs:1177/1857/1973 |
| vue_shadcn.rs(281 行) | `ShadcnUiAssets`/`bundled_components`/`is_bundled`/`MaterializeReport`/`materialize` | vue.rs 5 处;**仓外:crates/auto/src/cmd_vue.rs:25/635(`auto vue` 离线组件源)** |
| gallery_assets.rs(72 行)/wm_assets.rs(67 行) | `GalleryAssets`/`WmAssets` 及 `materialize` | 仅 vue.rs;wm 含跨目录 include `packages/drawlist-renderer/src/*.ts`×5(迁移须连同外部构建输入) |
| gallery_cache.rs(214 行) | `DiskEntries`/`load`/`store`/`demo_content_hash`/`pac_dep_roots` | 仅 vue.rs:8366-8454 |
| image_viewer.rs(395 行) | 全部(`ImageViewerService` 等;零本仓依赖) | 仓内仓外无符号级调用;auto-media-core/media_service.rs:14 注释镜像其形状——迁移后同步注释指向 |
| tauri_backend.rs(233 行) | `generate_tauri_backend` | 仅 tauri.rs:116 |
| vscode.rs(1087 行) | `VscodeConfig`/`VscodeProject`/`generate/build/run_vscode_project` | 仅 automan.rs |
| api_gen.rs 之 UI 服务桥部分 | `PHOTO_SERVICE_HANDLERS`、media/viewer HTTP 路由模板(photo_service/media_service 消费桥)——按生成目标(vue/tauri backend)随生成器走 | 嵌在模板字符串;与 §2.3 通用提取共享 `generate_api` 分派,B 阶段拆分时单份算法 |

### 2.2 留 Man / 归上层 adapter(非 UI)

| 模块/符号 | 归属 | 依据 |
|---|---|---|
| api_gen.rs 通用 API 提取/转译:`try_full_parse`、`extract_api_lenient`、`transpile_db_to_rs`、`transpile_back_module_to_rs`、TS client、`generate_rust_server`、SSE bus、upload/file glue、assembly 指纹 | 语言/服务构建 adapter(816 管理服务闭包;不随 UI 搬) | 777 ownership `split-man-service-gen`(owner=lang-service-adapter) |
| http_service.rs(`resolve_config`/`serve_project`) | 语言/服务 adapter;**仓外消费 crates/auto/src/cmd_service.rs:10,A 阶段零改动** | `split-man-http-service` |
| sidecar.rs(`SidecarSpec`/`load_sidecar`/`apply_sidecar_to_crate`) | 留 Man(声明单源 auto-man-core;效果 adapter 服务 rust_ui+api_gen 两处)——B 阶段 rust_ui 迁走后由 UI 工具消费 Man 公开的声明读取或迁纯规则 | §5.1"sidecar/front/theme/backend/locale/env 覆盖按已合入规则保持" |
| generation_request_adapter.rs | 留 Man(I-PROJECT 动态臂唯一构造点,gen/run 分派在 automan.rs) | 809 交付;815 只消费不复制 |
| pkg(进程执行)/builder/target/Git/FS/通用锁/resolver | 留 Man/共享包;**不搬** | §0"不搬通用锁/resolver/Builder" |
| Man 模板/构建器资源(auto-man-assets) | 不动 | 803 已交付;§5.2"803 既有通用 templates/GHS/IAR 资源不重新迁" |

### 2.3 旧 oracle(迁移不得静默改的验收面)

- vue: crates/auto-man 内 plan075_bp_tests.rs(bp 链)、vue.rs 内联 ~3700 行测试(gallery/desktop/registry/i18n/shadcn 物化);`auto vue` 命令走 cmd_vue.rs(vue_shadcn 离线源)。
- rust_ui: plan811_media_consumers.rs:189-194(DELIVERY 表);792 driver 24/24(809 收据:V01 fresh/V02 重定位 locked/V03 负控/V04 dev 同源)。
- 777 strict 基线(ui/man/data 域规则)与 module-boundaries.toml ownership 表(`split-man-ui-orchestration`/`split-man-rust-ui` owner=auto-ui-tool-plugin,R5)。
- 画廊围栏/autoui-verifier 双端(auto run / -r vm)——B 阶段真生成工程验收用。

### 2.4 资产 owner/source/许可证清单(T-02 输入,生产 embed 源 T-04 前不动)

| 资产组 | 现位置 | 来源/许可证 | 迁移目标(A 阶段仅登记) |
|---|---|---|---|
| gallery | crates/auto-man/assets/gallery/AppViewport.vue | 本仓自著 | auto-ui-tools assets/gallery |
| wm | crates/auto-man/assets/wm/(5 .vue+4 .ts)+ packages/drawlist-renderer/src/*.ts×5(wm_assets.rs:52-57 include) | 本仓自著;drawlist 为仓库根外部构建输入,须连同迁 | auto-ui-tools assets/wm + drawlist 输入 |
| shadcn-ui | crates/auto-man/assets/shadcn-ui/(345 文件:285 .vue+59 .ts+SNAPSHOT.md) | 上游 shadcn-vue(MIT)registry 快照 2026-08-27,SNAPSHOT.md 记 57/61 组件+2 处 baked patch(PLAN-571) | auto-ui-tools assets/shadcn-ui;**迁移时补随附许可证文本** |
| am.at | crates/auto-man/assets/am.at | 工具链配置(805 域),非 UI,不迁 | — |

## 3. I-TOOL 协议合同(A 发布面)

- **消息**:`ClientEnvelope{protocol_version,id,body}`;body=`describe`/`validate_project{request}`/`cancel{target_id}`。`ServerEnvelope{id,body}`;body=DescribeOk(ToolDescriptor)/ValidateOk(report)/Event(Progress)/Error(ToolError)。
- **ToolDescriptor**:tool_name=`auto-ui-tool`、tool_version(crate 版本)、protocol_version、capabilities(describe/validate_project,均 cancellable=false、interactive=false)、lifecycle(shutdown_on_eof=true、exit 码表)。
- **ToolError 谱系**:`UnknownProtocolVersion{client,server}`、`UnknownOperation`、`MissingField{field}`、`InvalidField{field,reason}`、`BudgetExceeded{budget,limit,observed}`、`NotCancellable{op}`、`Internal`——全部可序列化、有 Display;版本/能力拒绝先于任何副作用。
- **validate_project 诚实边界**:只检 I-PROJECT 元数据/路径存在性/请求约束;report.summary 显式列 `checked`(metadata/paths/request-constraints)与 `not_checked`(language-parse/evaluation/generation/build);StaticAdmitted 含 unsupported 构造→warning finding(不算已求值)。
- **消费合同**:816 客户端在 M815-A 合入后只依赖 `auto-tool-protocol` + `auto-project-model`;不得静态链接 auto-ui-tools 实现进 CLI(Design 37 `cli -.子进程协议.-> ut`)。

## 4. 预算与单位(T-03 落地为常量,工具/客户端共享)

| 预算 | 值 | 单位/语义 | 违反分类 |
|---|---|---|---|
| max_frame_bytes | 8 MiB | 单帧字节数(含前缀后载荷),双向 | 输入帧超限→流致命(exit 2);输出帧超限→写失败(工具侧 Internal) |
| max_progress_bytes_per_op | 1 MiB | 单操作累计 progress 载荷字节 | BudgetExceeded 错误响应 |
| max_queue_depth | 128 | 客户端待消费通知条数(816 执行器侧强制) | 客户端侧 backpressure;A 工具同步无队列 |

## 5. B 阶段执行序(依赖门复核)

T-04(等 M814-A+M815-A 发布):按 §2.1 表迁 vue/rust_ui/jet/ark/tauri 主体+资产 embed 一次性切换(单次迁移,T-02 只备映射);旧 Man 公开门面投影;DELIVERY 表增补。T-05(等 M814-B):真实 build/run/交互/取消全路径。T-06..08:兼容全路径+consumer+SD 沉淀。api_gen 通用/UI 分拆在 T-04 按"单份算法"执行,不保两套 server generator。
