# runtime 相关 plan 索引

> 状态以 plan 文件自身为准；归档列为当前位置（`plans/`、`archive/`、`old/`，均相对 `docs/plans/`）。
> 重编号提示：318 文件内自述"原编号 336，2026-07-23 因冲突改为 318"（同批 327/336/337/338/342/351/355/359
> 的后创建者已改号为 317/318/320/322/330/346/347/348）；`old/` 下另有两个 152（SSE 与 a2ts）并存；
> 355 在 archive/（session 修复）与 plans/（a2r async-await）各一，引用时须带归档位置。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 020 | stdlib-io-expansion | 🔄 ACTIVE（plan 自述） | old/ | File I/O 用新 ext 机制落地，stdlib 扩张起点 |
| 027 | stdlib-c-foundation | 🔄 ACTIVE（plan 自述） | old/ | 迁移到 AutoLang-first 架构，自托管前置 |
| 064 | split-universe-compile-runtime | ✅ COMPLETE（2025-02-01） | old/ | Universe 拆为 Database（编译期）+ ExecutionEngine（运行期），SymbolTable/StackFrame 双结构确立 |
| 081 | autovm-default-mode | ✅ Implemented and tested | old/ | AutoVM 成默认执行模式；Phase 5 落地 C FFI 桥（CALL_NAT→动态库） |
| 090 | remove-universe-from-parser | ✅ 完成 | old/ | Parser 不再依赖 Universe |
| 091 | universe-removal | ✅ 基本完成 | old/ | Universe 全面移除，职责分流到 Database/Engine |
| 092 | rust-ffi-sandbox | ✅ Phases 1-6 | old/ | 沙箱把用户 crate 编成 cdylib 供 VM 动态加载 |
| 094 | hybrid-ffi-bridge | ✅ Phase 1-5（43 个 shim） | old/ | 内建 shim + 动态加载双轨统一进 NativeInterface |
| 102 | http-server-stdlib | 🔄 Phase 1-4 完成，Phase 5 未开始 | old/ | Task/Msg 异步框架上建 HTTP server 与 net/json/url 模块（plan-indices 记 ✅，以 plan 自述为准） |
| 105 | auto-router | 未标注 | old/ | Router 初版实现规划 |
| 106 | router-use-syntax | 未标注 | old/ | Router `use` 语法改进 |
| 114 | hybrid-routing | 未标注（代码已落地 route/） | old/ | 约定发现 + 配置覆盖的混合路由；文件内标题作 "Plan 119"（编号漂移） |
| 119 | a2rs-backend-stdlib | 🚧 IN PROGRESS（Phase 1 started） | old/ | a2r 后端 stdlib 支撑（a2r_std.rs 方向） |
| 135 | ui-incremental-compilation | ✅ COMPLETED（2025-03-19） | old/ | UIArtifact/UICache 入 Database，UI 生成复用 AIE 增量（database 头注误作 plan-134） |
| 152 | streaming-http-sse | ✅ Completed | old/ | SSE 解析模块（sse/）与流式 HTTP 落地；与 old/152-a2ts 重号 |
| 154 | real-http-streaming | ✅ Completed | old/ | HttpStreamData 真实流式响应（reqwest blocking） |
| 195 | http-client-async-unification | ✅ COMPLETE | old/ | HTTP client 升级 reqwest，统一 auto.http 与异步支持 |
| 211 | stdlib-test-coverage | ✅ COMPLETE（2026-04-23 验证） | old/ | stdlib 测试 80%+：51 VM FFI + 17 a2r 测试 |
| 212 | rust-ffi-e2e | ✅ Phase 1-3 COMPLETE | old/ | dep→cdylib→AutoVM 加载→调用端到端管线 |
| 214 | python-ffi-use-py | ✅ COMPLETE | old/ | PyO3 嵌入 CPython，`use.py` MVP（string→string） |
| 216 | cffi-bindgen | ✅ COMPLETE | old/ | auto-bindgen 自动签名提取 + libloading C FFI 运行时 |
| 222 | python-ffi-multi-type-marshalling | ✅ DONE | old/ | Python FFI 扩至 int/float/bool/string/list |
| 224 | vm-async-runtime | ✅ COMPLETE | old/ | TaskSystem.run 桥接 + AWAIT_FUTURE 重入 + async FFI shim |
| 250 | auto-stdlib-enhancement | ✅ COMPLETE | old/ | 新增 11 个 stdlib 模块（cmp/clone/fmt/sort/datetime/fs/hash 等） |
| 267 | ffi-complex-patterns | ✅ COMPLETE | old/ | 外部迭代器等复杂 FFI 模式（Rust 状态机↔VM for-in 协议） |
| 300 | python-ffi-runtime-maturation | 未标注（分批实施，含验收记录） | plans/ | Auto 类型经 NanoValue tag 直通 Python FFI 参数与返回 |
| 312 | autovm-api-routing-http-server | ✅ Phase 1-4 Delivered（2026-06-16） | archive/ | AutoVM 一等 #[api] 路由 + 可用 HTTP server |
| 313 | autovm-tcp-flush-sse-server | ✅ Phase 1-2 Delivered；Phase 3 留待后续 | archive/ | TCP flush + 服务端 SSE 推送（sse_server.at） |
| 316 | fix-312-server-panic | ✅ 已修复（2026-06-16 合并） | archive/ | #[api] server 自动启动 panic 修复 |
| 317 | vm-async-scheduling-investigation | 调研完成；Phase 1 已合并，Phase 2-4 待实施 | plans/ | 敲定 yield/~Iter、await、Task/Msg 三套异步机制在 AutoVM 的真实状态（原号 327 改来） |
| 318 | list-struct-id-corruption | 未标注（诊断已确认，含验收标准） | plans/ | List\<Struct\> 元素 ID nanbox tagging 损坏修复（文件自述原号 336 改 318） |
| 321 | generator-runtime-yield-iter-stream | ✅ Phase 1-6 Delivered（2026-06-17） | archive/ | Generator 运行时：yield + 统一 ~Iter/~Stream |
| 322 | list-struct-runtime-diagnosis | 排查总结（1 测试仍 ❌，待 generic constructor 修复） | plans/ | CALL_NAT/CALL_SPEC 多路径分发排查经验（原号 338 改来） |
| 326 | vm-runtime-struct-list-serialization | ✅ Phase 1-5 完成 | archive/ | VM 运行时补全：struct/List/序列化/类型转换 |
| 328 | a2r-http-server-architecture | 设计完成，待实施 | plans/ | #[api] 直译为 Axum 原生 Rust server |
| 329 | ipc-sse-channel-support | 设计完成，待实施 | plans/ | Tauri Channel 流式推送复用 SSE |
| 334 | vm-vm-merge-skip-http-backend | 未标注（含验收标准） | plans/ | vm+vm 同进程合并，默认跳过冗余后端 HTTP 进程 |
| 335 | list-struct-runtime-fix | 未标注（含验收标准） | plans/ | List\<T\> 运行时 + 渲染层 VmRef 解引用完整修复 |
| 341 | vm-debugging-methodology | 方法论总结 | plans/ | VM 脚本化测试把 UI bug 排查提速 30-60 倍 |
| 344 | unified-http-comm-architecture | 设计文档 / TODO（未实现） | archive/ | 同步/异步 × 流式/非流式 × VM/a2r 统一 HTTP 架构 |
| 349 | http-roadmap | 规划文档（持续更新） | plans/ | HTTP 库能力矩阵与扩展路线 |
| 350 | websocket | 设计文档 / TODO | plans/ | WebSocket 双向通讯设计 |
| 352 | middleware-session-ssr-openapi | 设计文档 | plans/ | Web 框架四项缺失能力（中间件/session/SSR/OpenAPI） |
| 353 | io-fs-module-design | 调研与设计文档 | archive/ | auto.file/File.* 现状盘点与 IO/FS 模块设计 |
| 355 | fix-persistent-session-fn-body-recursion | ✅ 已修复（2026-06-27 合并） | archive/ | 持久 session 解析 fn 内复合语句无限递归修复；与 plans/355-a2r-async-await 重号 |
