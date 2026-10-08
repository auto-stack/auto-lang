# http-service — Auto HTTP 服务部署模板与真实代理验收（PLAN-736）

本目录是 **736 交付等级**（本机开发 + 受单层反向代理保护的 HTTP 服务）的
部署模板与验收工具。不改 playground 在用部署文件（`deploy/nginx-auto-playground.conf`
指向远端专项站点，两者无涉）。

## 交付等级与责任矩阵（edge 模式）

| 责任 | 承担方 | 依据 |
|---|---|---|
| TLS 终止（HTTPS→HTTP/1.1 upstream） | Nginx（模板） | Auto 无内置 TLS |
| Host 白名单 | 后端（allowed_hosts 精确匹配，`X-Forwarded-Host` 不放宽） | service.json |
| 访问策略/操作面屏蔽 | Nginx（`/__auto/*` 对外 403） | 模板 location |
| 单层可信代理身份 | 后端消费 `X-Forwarded-For`（仅 trusted peer、**覆盖**语义单段重写） | trusted_proxy_ips |
| 业务鉴权 | 应用自带 middleware（app 自有语义；edge 模式下代理层担访问策略） | auth.responsibility |
| 连接/请求/预算/观测/优雅关闭 | 后端（两轨同构，见 http-service-deployment 契约） | PLAN-736 |

**不在本期**：直接公网、多层代理、后端内置 TLS、账户/JWT/CSRF 系统、分布式限流。

## 文件

- `nginx.conf.example` — 单层代理模板：TLS 终止、`/__auto/*` 屏蔽、普通/SSE/
  上传/下载四分组（SSE 关响应缓冲、上传关请求缓冲、读取间隔≠上传总期限）、
  XFF **覆盖**重写。版本基线 **nginx 1.31.6**（T-01 冻结实测），指令以锁版本
  行为为准，不以官方 latest 资料代替核对。
- `run_proxy_fixture.py` — AC-07 验收 runner：生成测试 CA/证书（仅 localhost
  SAN，**不入库**）→ 物化 nginx 配置 → `nginx -t` → 起后端（vm/rust 双轨）→
  真实 HTTPS wire 验证（测试 CA 验证，无 --insecure）：
  短 JSON / 坏 Host 400 / SSE 帧经代理可见（vm 轨） / 上传→下载 hash 一致 /
  Range 206 / health 身份核对。

## 验收运行

```bash
python deploy/http-service/run_proxy_fixture.py --track vm   --auto target/debug/auto.exe
python deploy/http-service/run_proxy_fixture.py --track rust --auto target/debug/auto.exe
```

先 `cargo build -p auto`。需要 PATH 上有 `nginx`（1.31.6）与 `openssl`（3.2.3）。

## 已实测边界（T-06，如实记录）

- **SSE 帧竞速**：VM 轨总线订阅会话在首播后 ~1-2s 自然收口（698 域时序面）。
  单发发布与收口存在竞速（直连/纯转发/手动 nginx 均帧可达；fixture 形态偶发
  输竞速）——runner 用三连发抢窗口，任一帧经代理可见即 PASS。连续流语义
  （017 聊天形态）不受此影响。
- **上传授权早拒**：guard/早拒语义由 730 契约测试 + uploads 示例承载；
  本 fixture 走无 guard 的公开上传路径。
