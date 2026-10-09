# PLAN-738 revision 2 修复记录

日期：2026-10-09。用户已授权在原计划、原 worktree 内修订和修复；AC-01..08、SD-01..07 保留。本文是实施进度，不是通过收据，未合入、未归档。

## HTTP 帧时序的确定性归因

原断言（两帧间隔至少 150ms）保留。原基线 `e4425b673eec46fabc25b4534084c19f2738a38f` 在独立 detached worktree 使用相同 `test-http-e2e` 配置运行原测试通过；R1 修复末提交 `4089b84183c6044ee68f1b6962752d4cb54da821` 曾稳定失败，两帧约在 1 秒后同时到达。单凭这些运行不能认定为环境红。

实际等待对象存在错位：`AsyncHttpStream` 的消费者读桥接后的 `ASYNC_STREAMS` 队列，generator 等待路径却检查原始 `STREAMS` 队列。bridge 提前取走 raw 数据后，legacy 消费队列虽已有帧，等待者仍可能睡到下一次 heartbeat；这解释了两帧同到达的观测。

修复提交 `53010f1452169adbf5edab25abba574df9ea8bbd` 改为等待真实消费队列，订阅通知后检查队列与终态，并在桥接的数据/终态写入后通知。新增确定性测试先排空 raw 队列、保留 legacy 数据，再证明旧 wait 超时而新 wait 立即返回；还验证订阅后到达的数据能唤醒等待者。原时序测试的修复后定向运行约 2.63–2.71s 通过；最终整档和重复运行仍待补。

证据在 `D:/autostack/.wt/verify-738-base/auto-lang/.verify738-baseline-{timing,root-timing}.log` 及实施 worktree `.repair738-r2-proof-tests{3,4}.log`。后续构建可能覆盖共享测试二进制，不把现存二进制误称旧基线产物。

## 已实现、正在验证的范围

- 共同 `NativeInterface::production` 保留实际注册/覆盖次序；register/merge 撤销旧 contract 与证明。编译与运行查询同一生产工厂。新增最终绑定的执行前检查，不能到后面的错误调用才发现前面的业务已执行。
- 不把旧 JSON string-wire helper 的 ID 别名当作 `JsonValue` receiver 方法证明；JSON parse 保留真实堆值 materialization。TCP read 的已知缓冲/双输出差异保持 Unverified 拒绝，不改公开 ABI。
- resolver 捕获源内容与层身份；clone epoch 的 TypeStore 分离，named import 保留来源。双 body/签名冲突保留公共与实现源位置。未引用的 unsupported 模块导入不直接拒绝。
- C 发射消费完整 AST，项目 Rust 模块发射消费实际选定 `.rs.at` 层。真实 MSVC/rustc 同源样例分别执行 C=43、Rust=42；VM=41 的最终同轮重验、现有 C IO 能力样例仍待补。
- manifest schema 3 冻结 selected/public 源、实际引用证明、target/environment、编译 features 和本地 producer/compiler 源及 Cargo 版本输入。CLI、生成记录和新鲜度门已接线；VM/持久会话/Rust/C 产物共同消费正在补验。
- 六核心 fn/method/type/field 公共分母扩展到三目标×两环境。未证明项保留明确原因，文件存在不升为执行证据。

## 真实生成服务验收

测试走正式生成器、生成 workspace 的 cargo 构建、实际服务 binary、ready 和业务请求，再只改标准库而保持 API 源不变，验证旧 receipt 陈旧及重新生成。没有手改生成 Rust 或替换业务实现。

该链发现并修复两项实际问题：仓库外项目的框架依赖 fallback 路径不存在，跨 Windows 磁盘的 canonical 路径还带不适合 Cargo URL 的 verbatim 前缀；source_hashes 写成数组而读取端按对象取值，曾跳过源内容检查。生成开始前撤销旧 receipt，最后重新核对冻结输入，不为中途变化/失败产物写成功收据。

`.repair738-r2-generated-service6.log` 第一轮真实构建、ready 与业务请求通过，stdlib-only 改动使旧 receipt 失效并触发再生。第二次构建与正在进行的源修改重叠，读到了随后修正的变量名错误；该轮证据作废。此前尝试分别发现既有的 auto-man integration fixture 编译错误（限定正式 lib 目标继续）、错误依赖路径、source_hashes 漏检以及验证 harness 的 Windows verbatim target 路径问题；不把这些失败当通过或批准延期。

## 稳定代码验证与从头重跑（2026-10-09 稳定树收口，worktree `b838f5f16`）

按用户指示先验证稳定代码、再从头重跑服务验收；全部证据绑定提交 `b838f5f16`（R2 主体实现 + 本轮 host 门修复一并入库的稳定树快照）。

**稳定代码门禁**（全部在 `D:/autostack/.wt/lang-738/auto-lang`）：

- 三 crate `cargo check`（auto-lang/auto-man/auto）零错误；仅预存警告（shim-metadata doc comment、`cli_passthrough_tests::parse` 在非 test cfg 下的 dead-code、auto-down 兄弟仓），均不在本轮修改面。
- `cargo t plan738` **55/55**（52 既有 + 本轮 host 新增 3：再导出解析/await 模式/漂移反例）；此前的 `legacy_wire_json_helpers_do_not_inherit_receiver_proofs` 变量名修正已生效。
- auto-man `api_gen` 全族 **43/43**（本轮修复前 41 passed + 2 failed，见下）；module_cache 16/16、native_registry 13/13、use_semantics 7/7、autovm_persistent 20/20、CLI `auto stdlib` 10/10（--test-threads=1）。

**本轮新勘两真实红与门修复**（R2 严格门自身形状缺陷，非 729/730 回归；`PROVIDER_CLAIM_NO_CALLEE` 此前无任何日志在案）：

1. `test_plan729_file_endpoint_generation` / `test_plan730_upload_endpoint_generation` 失败——`verify_rust_reference`（Embedded 运行时）在内嵌 `a2r_std.rs` 只认 `Item::Fn`，而 `file_response`/upload 族是文档化的 `pub use` 同形转发壳（`a2r_std.rs` → 独立 crate `http.rs` → `server_file.rs`/`server_upload.rs`），被误报 NO_CALLEE；730 的 async producer 又被整拒。
2. 修复（`stdlib_assembly/host.rs`）：①`pub use` 再导出链跟随到定义文件（闭世界路径→文件映射，glob 再导出不跟随、如实 NO_CALLEE；producer 身份=定义文件，如 `crates/a2r-std/src/http/server_file.rs::file_response`）；②async producer 与发射 `.await` 双向核对为 §5.7 mode 证据，错配=SIGNATURE_DRIFT（新增 sync+stray-await 反例）；③`impl AsRef<str>` 视图适配与不透明资源类型（FileResponse/UploadRequest/UploadSession/UploadReceipt）按名身份，适配变体写入 contract error_shape 单独记录。公共声明与真实实现签名核对全通过（file_response 3×str→FileResponse；upload_receive 4 参→UploadSession 等）。

**服务验收从头重跑（最终绑定 `b838f5f16`）**：`cargo test -p auto-man --lib --features test-http-e2e http_e2e_plan738 -- --ignored --test-threads=1` —— **1 passed（110.9s）**，`.repair738-r2-generated-service9.log`。链全程：正式 `generate_api` → 生成 workspace cargo 实编 → 服务进程 spawn → `/__auto/health/ready` 返回 assembly_fingerprint 与 receipt 一致 → 业务 `/api/answer` 返回 42 → 仅改 stdlib `json.at`（API 源字节不变断言）→ `backend_generation_is_fresh`=false → 再生成指纹不等且恢复新鲜 → 重建 → 复跑 ready+业务。注：首次重跑曾因漏 `--features test-http-e2e` 得到 0 测试假绿（`.repair738-r2-generated-service7.log`），已识别并废弃该轮，带 feature 重跑为准。

**rustfmt**：HEAD 干净而本轮变脏的 5 文件（build.rs/plan/providers/validate/cmd_stdlib）+ 新文件 host/reference 已格式化归零；预存脏遗留大文件（trans/rust.rs 1018 处、codegen 377 处等，HEAD 同脏）保持基线不做整文件重排（避免无关巨量 churn）；lib.rs 的大计数系 CRLF 噪声（LF 归一化后零差异）。

## 收口条件

本轮已完成：稳定代码验证、服务验收从头重跑（绑定 `b838f5f16`）、host 门两处形状修复、scoped 门禁与 fmt。生成服务验收证据链就此闭合（不再以 service6 中途重叠轮为准）。

仍待完成：实际 callee/来源闭包与方法/别名/依赖的遗漏审计，manifest 全消费者一致性及生成 consumer 的实际构建依赖/feature 绑定，C IO 正证与 VM=41 同轮重验，最终 `t/tv/tt/th` 与既有红双树分诊、完整 SD 正文和新提交上的独立复审。T-02..T-08 仍打开，不以本记录清偿所有 R1..R9。
