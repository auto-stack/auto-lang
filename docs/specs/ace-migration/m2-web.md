# ace-migration: m2-web — Web 服务嵌入与实际后端迁移（PLAN-983）

> Status: **M2-A 切片已交付**（phase fold 2026-11-17；auto-web `bdc48e8`，
> 基 `0adebc0`；T-01/B-PORT-I/V/R + B-PROOF-I/V/R 共 7 任务闭合）。
> B 组（AVSERVICE/GENERATED/CONSUMERS/T-05..07 共 12 任务）未交付——
> 见"边界"。来源：[PLAN-983](../../plans/983-ace-m2-web.md) r7 §5/§8/§9
> （phase work/review/merge 收据）；依赖 967/968/969/974/975/978 均已归档。

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

## 边界（具名开放，B 域）

- core trait `listen_blocking` 签名仍由 core legacy trait 固定（桥接已
  文档化）；`http_server.rs` 内部 `Rc<AutoVM>` 执行面未动——均归
  B/AVSERVICE 换执行面。
- A2R 道零参子集；store/db/伴生模块、参数编组、带参 handler 未交付
  （B/GENERATED 扩面）。
- AV 腿 ASCII 数据根约束（非 ASCII 路径 UTF-16 staging 未支持；B/AVSERVICE
  或 973 域）。
- 既有红（非本切片引入）：auto-web
  `api_gen::tests::test_handler_body_r2_matrix_compile_oracle` 基线
  `0adebc0` 原样红。
- V-A-PROOF 新增用例计时：ace_lane 5.89–9.14s（内嵌显式 cargo --offline
  子进程=计划 r7 明文合同）、ace_av_service 0.62–0.89s、ace_av_cancel
  0.62s、端口/lane 单测 ≤0.032s。
