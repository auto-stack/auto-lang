# ace-migration: m2-web — Web 服务嵌入与实际后端迁移（PLAN-983）

> Status: **M2-A 切片 + M2-B 双批（AVSERVICE/GENERATED）+ 文本入口切片已交付**
> （A phase fold auto-web `bdc48e8`；B phase fold `fd115ce`，2026-11-17；
> 2026-11-18 文本入口切片 auto-web `3f8de15`，见下文「B-AVSERVICE 文本入口」）。
> CONSUMERS 与 B-GENERATED-R/T-05..07 未交付——见"边界"。来源：
> [PLAN-983](../../plans/983-ace-m2-web.md) §5/§8/§9（各 phase
> work/review/merge 收据）；依赖 967/968/969/974/975/978 均已归档。

## A 切片已验接口

### B-PORT：中立受检 Web 宿主端口（auto-web）

- **公共组合面中立**：`host.rs::listen_borrowed/serve_owned`、
  `http_server_handle::serve_server_handle`、`web_host_port` 模块导出
  （`WebHostedService/WebListenRequest/WebListenKind/WebRouteDecl/WebPortError`）
  不含 `Rc<AutoVM>`/旧 native 签名；legacy AutoVM 退为端口后的**内部**
  适配臂，core 的 `HttpServiceHost` trait 实现是文档化桥接。
- **端口级受检准入**（命名稳定）：`web.port/control-plane-conflict`
  （业务路由占 `/__auto/*`，消息文本与端口前逐字一致）、
  `web.port/empty-address`、`web.port/owner-consumed`（owner 单次消费：
  按值编译期 + 运行期命名拒）；另有纯路由集静态形 `WebHostedService::admit`。
- **行为保持**：owner 线程模型/协作取消/File-Stream 所有权/shutdown 机制
  原样（既有 http_runtime/upload/shutdown 门禁全绿）；`auto-web-contract`
  依赖仅 serde/serde_json（无 Core/UI）。

### B-PROOF（RG-A02）：两腿真实验证（原 TCP fixture 接原 target）

- **AV 腿**：永久 fixture
  `auto-web/tests/fixtures/ace-p983/av-service.at` = member-delivery/b API 面
  （GET /value=200 版本标记、POST /counter/add|double、GET /counter/get 的
  CounterResult JSON、自有 /__auto/shutdown 控制路由）的 AV 程序。
  runner=`ace_av::console_host::compile`+`run_platform`（net974 同款；grants：
  net.listen/read/write/close、mem、file；dev-dep ace-av/ace-av-core）。
  持久化=隔离数据根 counter.txt 经 `file_open_w16`（UTF-16LE staged 路径；
  **Win32 面 `file_open_bytes` 为 Linux-only**）；单 owner 串行 accept（与
  legacy 同步 server-handle 运行时同构）。
  已验 oracle：独立 HTTP bytes/状态（200/404/202 精确、CounterResult 逐
  字节）、隔离 DB 文件由测试直读、i32 全宽大整数 2147483647 精确 JSON、
  断连（半请求+关）不触业务态、CancelFlag→Cancelled+端口重绑、
  202 shutdown+join+端口重绑。
- **A2R 腿**：`auto-web/src/ace_a2r_lane.rs`（显式 `AUTO_WEB_A2R_ENGINE=ace`
  选择，默认 legacy 道不变）——ACE 前门 `compile_text_record_source`+
  `ac.runtime.text.v1`（cert_rt_902 实证配对）→ `SemanticView` →
  `ace_a2x_rust::lower_program` → record-text 文件集；机械预处理（剥
  `#[api(...)]` 属性行/剥顶层 `pub fn` 前缀/无 main 补占位；子集契约=
  显式 `return`）。**A 切片子集**：零参 int/str/bool 返回 handler、无
  db.at/伴生模块（端点门+工程门+发射产物覆盖三重核对；子集外具名拒绝，
  无静默回退）。`api_gen.rs` 早门零写入（坏源保旧 workspace），生成 crate
  内嵌 `a2x_runtime.rs`+`generated_ace.rs`，api.rs 全 handler 委派
  `generated_ace`（`A2xResult` 编组：Int/Bool 直 JSON、StrView 直 JSON、
  StrOwned 转 String 后显式 `a2x_str_free`、trap→具名 500）。
- **依赖身份**：生产=ace-core/ace-a2x-core/ace-a2x-rust（发射侧）；
  ace-av/ace-av-core 仅 dev（AV 腿 runner）。


## B 批已验接口（phase fold 2026-11-17，auto-web `fd115ce`）

### B-AVSERVICE：AV 真服务执行臂（文件通道形态）

- **transport 原样 + 引擎枚举**：`http_server::ServiceEngine{Legacy(Rc<AutoVM>),
  Av(Box<AvServiceEngine>)}`——Legacy 段驱动/park/resume 全量保留（legacy
  installed-host 用例零变化）；Av 臂同步完成**永不 park**（parked 机制自然
  旁路）。公共入口 `http_server::serve_av(engine, addr, cfg, shutdown_rx)`；
  路由按引擎来源（legacy=进程全局 codegen 表；Av=服务自身 `#[api]` 声明，
  构造时抽取、编译前机械剥属性行）。
- **AvServiceEngine**（`av_service` 模块；owner 线程驻留，!Send 与
  `Rc<AutoVM>` 同律）：每请求 staged 原始 HTTP 请求文件 →
  `ConsoleProgram::run_platform`（grants `platform.file`+`platform.mem`）→
  响应文件读回 → `ApiReply::Full`。文件通道=现值 AV 接口边界的冻结设计
  （无「具名函数+参数+平台IO」联合公开入口：入口参数仅标量无 Str、每调用
  mint 新 ledger 强制关文件句柄、session/snapshot !Send——侦察核
  av-core/av.rs、resident.rs、session.rs）。
- **具名边界（501/专有面，不静默回退）**：已注册中间件链、上传路由
  （语言侧凭据消费通道缺位）；限流=legacy 专有（VM launch settings 面）；
  认证/权限/stream/SSE/`~T` future 未执行（R 条款记录）。AV 程序 main
  返回非 0 → 500 具名诊断。
- 永久 fixture `tests/fixtures/ace-p983/av-runtime-service.at`（member-
  delivery/b API 面 + counter.txt 持久化）；生产依赖新增 ace-av/ace-av-core
  （自 dev-dep 转[[dependencies]]——引擎为生产执行面）。
- `TransportConfig::from_env` 转 pub（嵌入方配置构造；值面不变）。

### B-GENERATED：ACE A2R 生成道参数扩面

- lane 子集扩至 **int/str 视图/bool 参数（值传）**与同族返回
  （`AceA2rParam`；a2x `l{idx}_` 局部名还原源参数名）；owned/mut str 参数、
  record/list/浮点具名拒；`str+literal concat` 产 owned `Str` 与 `str` 返回
  不匹配=verify 具名拒（恒等体为子集内形态）。
- `generate_ace_api_rs` 参数提取器：GET/DELETE=Query struct、POST/PUT=
  Json body（与 legacy 委派形态同源）；实参编组 int `as i32`/str `as_str`/
  bool。保旧面：route 注册（端点驱动）/store（ACE 道仍具名拒 db.at/伴生，
  默认道不变由 member_delivery+cold_generated 证）/schema（types.rs+
  generation.json 字段不变）/diagnostics+失败保旧（早门零写入+receipt park+
  write_if_changed）。
- V 计时：新生成例 `ace_lane_param_endpoints...` 6.45s（复跑波动至 9.66s；
  冷 cargo 启动+盘压；>10s 离群已记录，B/CONSUMERS 考虑预构建阶段）。

### B-AVSERVICE 文本入口（2026-11-18 用户裁定切片，auto-web `3f8de15`）

- **形态**：与文件通道臂并列的 `ServiceEngine::AvText` +
  `http_server::serve_av_text(engine, addr, cfg, shutdown_rx)`；transport/
  owner 线程纪律原样。文本臂**不持有 `Rc<AutoVM>`**（`ServiceEngine::legacy()`
  为 `None`，无 legacy 派发路径）。
- **编译一次**（`av_text_service::AvTextServiceEngine::new`）：Text profile
  （`ac.runtime.text.v1`）→ `lower_checked_owned` → `verify_mir` → ABC
  encode/decode → `verify_abc`；之后每请求只走
  `ace_av_core::av::execute_entry_text`。收据 `compile_count` 恒 1，ABC
  指纹固定（复用同一产物）。
- **请求文本契约**：handler = `fn main(query str, body str) int`（982 受检
  文本入口）。`query` = 原始请求目标 `?` 之后的部分（`%XX` 与 `+` 在宿主侧
  解码后才交给 handler；传输层给的是百分号编码的 ASCII）；`body` = UTF‑8
  请求体（可为空）。返回 `int` ⇒ 200 + 十进制文本；`Cancelled`/`Resource`
  ⇒ 504；`Trap`/`Error`/`Infra`/`Exit` ⇒ 500 具名诊断。非法 UTF‑8 在 handler
  执行前具名拒绝（`entry text argument is not valid UTF-8`）。
- **路由**：引擎构造时显式声明 `HttpRoute`；未命中 404 与文件通道 AV 臂
  逐字同形且**不派发**；中间件/上传 501 边界同形。
- **已验**（真实 TCP，`--test http_runtime` 7/7 绿，7.48s；新例 0.60s）：
  等字节长度 `中文`/`日文`（各 6 UTF‑8 字节）按内容分流——GET 查询 3/4、
  POST 体 1/2、查询+体组合体优先 2；`/text?%FF` ⇒ 500 `not valid UTF-8`；
  `/missing` ⇒ 404；收据
  `av_dispatches=6 legacy_dispatches=0 compile_count=1 artifact_fingerprint=9db3cce95d63f38e entry=d_main`。
  `--lib` 188 例 187 绿（唯一红=基线既有 `test_handler_body_r2_matrix_compile_oracle`，
  非本切片引入）；`--test http_generation` 3/3 绿（176s，默认道不变）。
- **未接（本切片边界）**：平台 I/O 与文本参数**同一次调用**仍缺共享入口
  （983 首消费者本轮只用文本，未触发；需时交 982 验证补齐）；完整 JSON
  （嵌套/浮点/字符串/请求头）按首个真实消费者字段推进，尚未实现；本切片
  无中间件/上传/stream/SSE 面。

### C0 公开面（供 982 `--stack web`，2026-11-18，auto-web `5c484d9`）

- `av_service::AvServiceEngine::new_with_grants(source, scratch_root, grants)`
  ——显式 grants（能力键，如 `platform.file`/`platform.mem`/`platform.env`）；
  默认 `new` 仍是 file+mem。
- `av_service::AvServiceEngine::receipt() -> Arc<AvServiceReceipt>`——
  `av_dispatches()`（真实派发数；路由 miss 在派发前拒绝、不计）/
  `legacy_dispatches()`（文件通道臂结构常量 0）。收据句柄须在引擎被
  `serve_av` 消费前克隆。
- `http_transport::TransportConfig::from_service_config(&HttpServiceConfig)`
  ——预算/超时/排水/连接上限的公开单一映射；
  `from_service_config_with_policy(Arc<HttpServiceConfig>)` 额外装配
  `ServiceRuntime`（CORS/可观察/限速/关停）。
- **集成注意**：用 `*_with_policy` 时必须
  `ServiceRuntime::bind_shutdown(tx)`，否则 `POST /__auto/shutdown` 的
  `request_shutdown()` 返回 false（服务不会停）；无 policy 时用
  `http_server::bind_control_plane_shutdown(tx)`。
- **ready 地址**：`http_server::bound_addr_snapshot() -> Option<String>`
  在真实 bind 后由 serve 路径写入（进程内单服务假设）；CLI 就绪行以此
  回报（auto-man `--server av` 已消费）。

## 边界（具名开放，CONSUMERS/后续域）

- core trait `listen_blocking` 签名仍由 core legacy trait 固定（桥接已
  文档化）；legacy `Rc<AutoVM>` 段驱动面保留为默认引擎（Av 臂为并行选项，
  默认链不切换）。
- A2R 道：store/db/伴生模块（sqlite 无 a2x 道）、owned/mut str 参数、
  record/list/浮点、str concat owned 促升——后续扩面具名拒。
- AV 执行臂：认证/权限/中间件/上传/下载/stream/SSE/`~T`（语言侧通道缺位，
  501 具名）；ASCII scratch/数据根约束。
- CONSUMERS 批、B-GENERATED-R 与 T-05..07 未交付（B-AVSERVICE-R 已随文本
  入口切片复核，见计划 §9）。
- 既有红（非本切片引入）：auto-web
  `api_gen::tests::test_handler_body_r2_matrix_compile_oracle` 基线
  `0adebc0` 原样红。
- V-A-PROOF 新增用例计时：ace_lane 5.89–9.14s（内嵌显式 cargo --offline
  子进程=计划 r7 明文合同）、ace_av_service 0.62–0.89s、ace_av_cancel
  0.62s、端口/lane 单测 ≤0.032s。
