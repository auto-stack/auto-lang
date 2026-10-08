# PLAN-736 验证总报告：门禁与 AC/SD 证据索引

> 绑定：worktree 分支 `plan-736-dev`（基线 master de3a64353）· 本报告=复审入口；
> 分报告：[decision](736-http-decision.md) · [config](736-http-config.md) ·
> [policy](736-http-policy.md) · [lifecycle](736-http-lifecycle.md) ·
> [proxy](736-http-proxy.md) · [load](736-http-load.md)。

## 门禁复现（§6.3，worktree `D:/autostack/.wt/lang-736/auto-lang`）

| 门 | 命令 | 结果 |
|---|---|---|
| 开发三 crate | `cargo check -p auto-lang -p auto-man -p auto`（逐个） | 0 error |
| 计划族 | `cargo t plan736` | 11/11（10 scoped + P729-D1 drain e2e） |
| 触面族 | `cargo t plan734 plan705 plan730` | 46/46 |
| 语料档 | `cargo tv` | 162/162 |
| 裸日常档 | `cargo t --no-fail-fast` | 失败集与 master 基线**完全一致**（16 预存族：musk p053/p054×6、plan707×2、plan606、schema_fence、desktop_protocol 等）——**零新增确定性红** |
| 真 TCP | `cargo th` | 2 FAIL = master 同（corpora_data_face 734 在案 + native_ns 同族）——零新增 |
| fmt | `cargo fmt -p auto-lang -p auto-man -p auto`（仅 736 触面文件保留格式化；预存未格式化 examples 等已回退原状） | 736 文件 clean |

## AC 证据索引

| AC | 结论 | 证据入口 |
|---|---|---|
| AC-01 双轨独立服务/配置 | ✅ | [config](736-http-config.md)（实机启动矩阵+负向非零全表） |
| AC-02 ready 身份贯通 | ✅ | [lifecycle](736-http-lifecycle.md) §状态机/身份 + config §AC-02 |
| AC-03 预算实际有界 | ✅ | [policy](736-http-policy.md) §AC-03（低预算 wire 双轨）+ load（超载门 64/64 cap 关闭） |
| AC-04 CORS/Host/代理/鉴权 | ✅ | [policy](736-http-policy.md) §AC-04（proxy_service 双轨实机矩阵）+ proxy（经真实 nginx） |
| AC-05 有界观测 | ✅ | [lifecycle](736-http-lifecycle.md) §观测（恰一次/脱敏/计数同点） |
| AC-06 关闭/进程管理 | ✅ | [lifecycle](736-http-lifecycle.md) §关闭（双轨 shutdown 全链）+ **P729-D1 e2e**（drain 窗在途文件体） |
| AC-07 真实 HTTPS 单代理 | ✅ | [proxy](736-http-proxy.md)（双轨 nginx 1.31.6 实测；vm 全矩阵含 SSE 帧；轨差按 734 支持面如实记录） |
| AC-08 固定负载/资源门 | ✅ | [load](736-http-load.md)（双轨四门全 PASS，raw 全录；阈值未放宽） |
| AC-09 canonical/兼容沉淀 | → merge | SD-01..06 草案随 plan §5"规范增量"；merge 时沉淀（本报告 §边界汇总即披露面） |

## 明示未验证/支持边界（AC-09 披露面）

- 跨 OS 生产实机（CI Linux 面 e2e 除外）、多层代理、直接公网、后端内置 TLS——不在 736 等级。
- 生成轨：route-A 门内普通 JSON 端点桩体（P670-D1 域）；route-A 转译 POST 的
  自动广播不在 734 支持面。
- 698 域 SSE 订阅会话首播后 ~1-2s 自然收口（帧竞速观察项，连续流不受影响）。
- VM resources 曲线 10 周期中 6 个有效 RSS 样本（4 次 OpenProcess 瞬断）；
  漂移/峰值判定余量充足（59.6/512 MiB）。
- 734 遗留债（P734-D4/D5/D7）不通过 736 宣布清偿。

## 遗漏/延期扫描（复审输入）

- 无未勾选任务；无绕道实现；三处预存根因顺手修复（a2r_std upload 壳/E0601 空
  main.rs/OnceLock 重入死锁）+ 一处生成胶水修复（upload Path 解包）均登记在案。
- `cargo tf` 批量档：**未到本轮执行窗**（merge 到期判定 736%5≠0 → 由
  /auto-plan:regress 按收据到期规则处理），不在 per-plan 门禁内。
