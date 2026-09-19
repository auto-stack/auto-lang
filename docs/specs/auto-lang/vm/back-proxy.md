# back-proxy（单进程多后端宿主）

> **Status**: implemented (PLAN-658, 2026-09-19)
> **母规范**: [vm/overview.md](overview.md) §交互形态；UI 画廊侧契约见
> [ui/overview.md](../ui/overview.md) §ui-gallery。

## 职责

为内嵌 fullstack demo 提供"后端半身"的宿主运行形态：每个 app 的 back 链
以**独立 VM session** 在宿主进程内运行，HTTP 请求按子 URL 前缀
（`/apps/<app_id>/api/*`）路由——merged CALL 语义的 HTTP 化。与
`auto serve` daemon（Plan 269，named-pipe REPL 服务）互不替代：daemon 面向
交互式会话，back-proxy 面向 UI 宿主内嵌后端编排。

## 架构

- **进程形态**（T-00 裁定）：宿主内嵌 proxy 线程（`auto run -r vm` 单进程
  即含全部后端）。启动编排 = `auto_man::vue::start_gallery_back_proxy`
  （rust_ui 画廊钩子，proxy 绑定先于发射）。
- **线程模型**：listener 线程（std TcpListener 手写 HTTP/1.1，每连接一线程）
  + 每 session 一条专线程独占其 AutoVM（线程亲和约定），请求经 mpsc 投递。
- **session 装载**：`collect_module_imports` 扁平化 back 链 → Codegen 有序
  编译（Use → 类型 → Store(var) 全局化 + `__module_init` → Fn）→ Linker →
  VirtualFlash → AutoVM → 显式跑 `__module_init` 激活模块级 var。
- **路由**：每 session 自持 `api_routes`（Plan 312 清单，取自 Codegen），
  **不走**进程级全局 `HTTP_ROUTES` 单表（覆盖式语义多 session 不可共用）。
  路由匹配复用 http_server 的 `match_route`（`:param`/query）。
- **参数绑定**：按名（路径占位符名 → body JSON 字段 → query 参数），与
  Rust 生成器的 serde 字段映射语义一致。

## 服务面

| 面 | 路由 | 语义 |
|---|---|---|
| `#[api]` JSON | `/apps/<id>/<path>` | session 内 `call_fn_by_name` 执行，`nv_to_json` 返回 |
| ~Stream 端点 | 同上（返回类型含 `Stream<`） | **按签名特路**（与两个生成器同语义——函数体不在执行面）：订阅 session 事件总线，SSE `data:` 帧逐事件转发 |
| POST 广播 | 有 Stream 端点的 session | 镜像 api_gen 判别约定：fn 名含 typing → `{"event":"Typing","name":<首参>}`；create 型非 void → 返回实体加 `"event":"New<主类型>"` |
| 原生 media | `/apps/<id>/api/media/scan` `/stream/:id` | 宿主 Rust 直答（media_service 三态 + 单区间 Range 200/206/416）；scan 的 url 字段发**绝对值** |
| 图片字节 | `/apps/<id>/api/__auto/media/{id}/{rev}` | 委托 image_pipeline `media_http_response`（字节/Content-Type/ETag 304 保真）；session 内 auto.image 创建的条目同进程可读 |
| 观测 | `/__backproxy/log?app=<id>` | session 日志环（容量 256）只读 |

## 崩溃隔离（AC-05 契约）

- 隔离边界 = session 线程（构造保证：宿主与其他 session 不受影响）。
- 单请求 panic：`catch_unwind` → 500 可诊断 JSON + 指数退避
  （500ms × 2^n，封顶 8s）→ back 链重装载（**状态归零**——内存态后端语义
  诚实）。VMError（非 unwind）走 500 错误臂不重启。
- 测试面原生 `auto.sys.panic_hard(msg)`（目录 id 3147）触发真 Rust panic
  ——VM 内建 `panic` 有意映射 RuntimeError，走错误臂而非 unwind 边界。

## 相关原生（目录固定 id）

| 原生 | id | 语义 |
|---|---|---|
| `auto.bus.subscribe` | 3144 | ~Stream 端点函数体的**编译 seam**（被实际执行压 -1 响亮失败） |
| `auto.http.sse_open` | 3145 | SSE 客户端句柄形（非迭代器）；单槽 i32 返回 |
| `auto.http.sse_poll` | 3146 | 非阻塞 try_recv：Data→载荷 / 空→"" / 终→"[DONE]" |
| `auto.sys.panic_hard` | 3147 | 测试面：真 Rust panic（隔离路径驱动） |

## 已知边界与债

- `json.encode` 对 VM 对象字面量降格池索引串（占位 shim，PLAN-053 家族）
  ——POST body 组装镜像 emit_api_http_call 用 `json.from_value`，未修根因。
- gallery 内嵌 `timer {}` 块（自定义事件计时器）不触发（017 clock_secs
  实证；`.Tick`+interval 形态正常）——timer 块 demo 内嵌首例。
- 031 缩略图名列表（view for-loop over json-object vmref list）渲染空。
- `~Promise` 签名与 `use auto.*` 同走 proxy 路径（T-05 泛化），流端点判定
  含 `Stream<`；Promise 专路未实测（无语料）。

## 落地

- 实现：`crates/auto-lang/src/back_proxy.rs`（crate 根，un-gated）；
  编排 `crates/auto-man/src/vue.rs::start_gallery_back_proxy`；
  发射 `emit_gallery_vm_demos` stream/native-ns proxy 路径。
- 测试：`crates/auto-lang/src/tests/back_proxy_tests.rs`
  （`cargo th` 档 + `test-http-e2e,ui-iced` 组合，11 例）。
- 计划：[PLAN-658](../../../plans/archive/)（归档后）；设计裁定全文见
  计划 §5。
