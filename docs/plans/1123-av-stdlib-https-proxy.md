---
plan_id: PLAN-1123
status: drafting
feature_name: "AV 标准库 HTTPS 与代理提供者"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-22
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-967, PLAN-974, PLAN-1064, PLAN-1096, PLAN-1114]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-network.md, docs/specs/stdlib/design/av-rust-provider-coverage.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [stdlib, auto-abi, auto-av, ace-m2-build]
---

# [PLAN-1123] AV 标准库 HTTPS 与代理提供者

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](archive/1096-av-parity-full-language-certification.md)。

## 1. 目标

通过 Windows AV 普通工程入口，为已有 HTTP 标准库交付显式 HTTPS/TLS provider 和代理连接。公共行为覆盖直接 HTTPS、经代理转发 HTTP 和经 CONNECT 的 HTTPS，以及验证失败、超时和关闭。

## 2. 架构方案

普通 Auto 请求 → 标准库来源选择 → 已核授权及身份的传输 provider → 真实网络对端。沿 974 的显式选择规则，继续复用公共 HTTP 请求/响应合同。T-01 用普通项目比较 OS 与受检外库接缝，锁定一条可交付的 Windows 来源、部署依赖、错误映射与资源责任。

## 3. 技术栈

`stdlib/ace/std/{net,url}.at`、`stdlib/ace/avrs/net.rs.at`、ACE ABI/M2Build、受控本地 TLS 测试服务和代理。

拟实施树为 `D:/autostack/.wt/lang-1123/auto-lang`，分支 `plan-1123-dev`。

## 4. 需求分析与背景调查

[974 出口](../specs/ace-migration/m2-stdlib-network.md)已验 HTTP 方法族、SSE、基本重定向和 WebSocket 帧层。[1114 B-04](../specs/stdlib/design/av-rust-provider-coverage.md)保留 TLS/proxy 差额；[旧 HTTP 客户端](../../crates/auto-lang-core/src/vm/ffi/stdlib/http_client.rs)有实际 TLS 传输。AutoScape [1119](1119-autoweb-https-readonly-navigation.md)按其宿主网络合同交付，Windows AV 能力完成后可按真实需求消费。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

## 5. 详细设计

T-01 冻结一条显式 Windows provider、直连和代理配置来源、受检信任来源、失败映射和响应/连接所有权。证书与主机名校验必须实际生效。跨请求不得借用前次授权、代理配置或连接；受控服务/代理只用本机回环和局部测试信任。既有 plain HTTP 不要求新 provider。复用已支持的重定向、状态和流消费；代理认证、TLS 客户端证书及更广旧 HTTP API 按旧消费者证据另列，不从测试输出推定已交付。Windows 交付证据交 982/EXEC；Linux 实跑交 1063。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/ace-migration/m2-stdlib-network.md`](../specs/ace-migration/m2-stdlib-network.md) | 增加 Windows AV HTTPS/代理来源、公开错误及适用范围 | AC-01/02 |
| SD-02 | modify | [`docs/specs/stdlib/design/av-rust-provider-coverage.md`](../specs/stdlib/design/av-rust-provider-coverage.md) | 更新 TLS/proxy 边界，并标明真实来源与显式选择条件 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01 · T-02 | 普通 AV 工程向受控服务直接发 HTTPS 请求 | 实际对端见证请求方法和路径，独立检查响应状态及完整字节；换成不受信 CA 或错误主机名，具名失败，无应用响应。 |
| V-02 | AC-01/02 · T-03 | HTTP 代理、HTTPS CONNECT；代理拒绝和关闭 | 代理实际收到转发及 CONNECT，TLS 对端收到真实请求。代理拒绝或超时可区分，资源释放后下一请求仍可执行。 |
| V-03 | AC-02 · T-03 | 去掉 provider 或授权；仓外干净包；plain HTTP 近邻 | 缺失/撤权具名拒绝，预期无对端请求。仓外按发布闭包直连实跑，plain HTTP 按原验收继续通过。 |

冻结验证命令为 `cargo t ace-stdlib ace-av ace-abi ace-m2-build case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [ ] AC-01 · owner_stage=work。直接 HTTPS 与代理 HTTP/HTTPS 的 Windows 公开入口真实执行，V-01/02 的正负对端观察成立。
- [ ] AC-02 · owner_stage=work。锁定 provider、配置和诊断合同，V-02/03 的关闭、失败、可重建与无额外授权等边界可重放。向 982/1063 交明确输入。
- [ ] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
- [ ] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [ ] T-01 核定 Windows 传输来源与公开合同
  - owner_stage=work。以现有 HTTP 普通工程跑本地 TLS 可行性探针，核 974/967 兼容和可重建依赖。固定选择键、配置来源及错误表。
- [ ] T-02 交直接 HTTPS 与验证反例
  - owner_stage=work。依赖 T-01。实现公开请求路径与生命周期控制，执行 V-01。
- [ ] T-03 交代理与工程分发证据
  - owner_stage=work。依赖 T-02。运行 V-02/03，提供 982 和 1063 的源、调用参数、provider 及待验证范围。
- [ ] T-04 独立复审
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
- [ ] T-05 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1123 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

## 10. 待澄清事项

T-01 先核可交付的 Windows 来源和真实 ABI 形。方案需新公共接口时在执行合同中记明实际选择及消费者影响。
