# PLAN-736 T-06 报告：同源部署示例与真实代理验收（AC-04/07/09, SD-01/02/06）

> 绑定：`04992cc25` · 环境（T-01 冻结）：nginx 1.31.6（scoop）+ OpenSSL 3.2.3
> （Git for Windows）+ Windows 11 26200 x64 · 证据 = `run_proxy_fixture.py`
> 双轨实跑原始输出（本文件引用结论，raw 在各 fixture stdout）。

## AC-07 证据：真实 HTTPS 单层代理（测试 CA 验证，无 --insecure）

runner 链路：openssl 现场生成测试 CA+证书（localhost SAN，temp 目录不入库）→
物化 nginx.conf（模板 `<DEPLOY_DIR>`/端口替换）→ `nginx -t` rc=0 → 起 nginx →
后端（`auto service`，proxy_service profile）→ 全部请求经
`https://localhost:18443`（证书按测试 CA 验证）。

### VM 轨（全矩阵）

| 检查 | 实测 |
|---|---|
| `nginx -t` | rc=0 |
| 短 JSON 经代理 | `GET /api/ping` → 200 `{"ok":true,"service":"plan736-deployment"}` |
| 坏 Host 经代理 | `Host: evil.test` → **400**（后端 allowed_hosts 策略，代理只转发） |
| SSE 帧经代理（proxy_buffering off） | `data: "{\"msg\":\"frame-0-0\",\"event\":\"NewTick\"}"` 客户端可见 |
| 上传经代理（request_buffering off） | 201 `{"ok":true,"path":"fixture.bin","size":"3700"...}` |
| 上传→下载 hash | src=e142850460d3576c dl=e142850460d3576c（一致，3700B） |
| Range 经代理 | **206**，`bytes 0-15/3700`，16B |
| 身份核对 | health/ready 体含 instance_id/bound/profile/config_hash（与 READY 行一致） |

### 生成轨（支持面内全真实体）

| 检查 | 实测 |
|---|---|
| 短 JSON 经代理 | 200（模板桩体——见下边界） |
| 坏 Host → 400 / 操作面屏蔽 | ✓（`/__auto/*` nginx 403 + 后端策略双保险） |
| 上传→下载 hash | e142850460d3576c 一致（730/729 胶水=真实体） |
| Range | 206 `bytes 0-15/3700` |

## 支持面边界（如实记录，不缩小 AC-07：SSE 帧经真实代理已在 VM 轨达成）

1. **生成轨普通 JSON 端点为模板桩体**（本契约含 `pub type` → 681 route-A 门
   不激活；upload/download 为 729/730 宿主胶水=真实体）。P670-D1 域边界。
2. **生成轨 SSE 发布**：route-A 转译 POST 的自动广播在 734 支持面外（生成侧
   broadcast 注入只挂 CRUD 约定）；SSE 订阅线（events.rs）存在。SSE 帧经真实
   代理的验证在 VM 轨完成。
3. **SSE 帧竞速（698 域时序面，登记 736 观察项）**：VM 总线订阅会话在首播后
   ~1-2s 自然收口；单发发布与收口竞速（直连/纯 TCP 转发/手动 nginx 三形态帧
   均可达；fixture 形态偶发输竞速）。runner 用三连发抢窗口。连续流（017 聊天
   形态）不受影响。
4. 上传授权早拒：guard 语义由 730 契约测试承载；fixture 走无 guard 公开上传。

## playground 隔离

`deploy/nginx-auto-playground.conf`（远端 112.74.45.241 专项站）零触碰。
