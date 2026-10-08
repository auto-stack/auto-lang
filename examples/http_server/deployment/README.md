# plan736-deployment — HTTP 服务部署基线示例

PLAN-736 的部署验证固定 fixture：`auto service`（server-only，无 UI）+
`proxy_service` profile + 真实 Nginx TLS 终止（单层可信代理）下的双轨验收。

## 契约面（两轨同源 `src/back/api.at`）

| 端点 | 形态 | 说明 |
|---|---|---|
| `GET /api/ping` | 短 JSON record | 双轨 JSON 腿（生成轨为模板桩体——见下"轨差异"） |
| `GET /api/greet/:name` | 路径参 → str | 路径参数 wire |
| `POST /api/uploads` | 730 UploadRequest 契约 | multipart 收储（staging→commit），代理模板关闭请求缓冲 |
| `GET /api/files/:name` | 729 FileResponse | 下载 + Range（`file_response(root, name, opts)` 三参形） |
| `GET /api/ticks` | `~Stream<Tick>` | SSE 订阅（VM=事件总线；生成=events.rs） |
| `POST /api/ticks/publish` | record 返回 | 发布（VM 轨按 698 约定注入 NewTick 广播） |

## 运行

```bash
# VM 轨（进程内）
auto service --server vm  --dir . --http-config service.json
# 生成轨（生成 axum 后端子进程）
auto service --server rust --dir . --http-config service.json
```

`service.json` = proxy_service profile：loopback 监听、Host 允许表、edge 鉴权
责任、单层可信代理（127.0.0.1）、CORS 精确 origin、媒体/照片扫描路由关闭。

环境变量：`DEPLOY_PUBLIC_ROOT`（发布根）、`DEPLOY_STAGING_ROOT`（暂存根）——
只信 env，不信请求数据（730 契约）。

## 轨差异（734 支持面边界，如实记录）

- **生成轨普通 JSON 端点为模板桩体**：本契约含 `pub type`（SSE 契约需要）→
  route-A（api_impl 转译）按 PLAN-681 门不激活 → ping/greet 为默认桩体。
  JSON 腿在生成轨按 wire 级验证（200 + 策略门生效），真实体由 VM 轨与
  api_contract 示例承载。这是 734 route-A 门边界（P670-D1 域），非 736 回归。
- **生成轨 SSE 发布**：route-A 转译 POST 的自动广播在 734 支持面外
  （生成侧 broadcast 注入只挂 CRUD 约定路径）；fixture 的 SSE 帧经真实
  代理的验证在 VM 轨完成。
- 上传/下载为 729/730 宿主胶水（非转译面），**双轨等价真实**。

## 真实代理验收

见 [deploy/http-service/README.md](../../../deploy/http-service/README.md) 与
`run_proxy_fixture.py`（nginx 1.31.6 + OpenSSL 3.2.3 冻结环境，测试 CA 不入库）。
