# PLAN-776 最终独立复审 R3（2026-10-14）

**结论：needs_fix。** 实现侧 F776-R2-01..08 不变量已闭合（代码+反例 oracle 可复现）；
知识增量与决策报告仍停留在 R1 规则，与 R2 实现失配。本轮 3 项 finding：1 P1、2 P2。
退回 `executing`，原合同 revision=1 不变。

## 1. 基线与证据

| 项 | 值 |
|---|---|
| review_scope | final |
| reviewer_identity / limitations | 当前 MiMo 会话独立于修复实施会话；无子 agent/额外模型独立性声明 |
| reviewed_commit | `088e4782318275d75c1ce9d77f782e7ad3476c9a` |
| repair diff base | R2 审查提交 `6494092de6c33875b5e99f81491bb25815852b38` |
| original base | `5e21561d2c303a02319ff7a29e4607933e09a38f` |
| worktree / branch | `D:/autostack/.wt/lang-776/auto-lang` / `plan-776-dev` |
| 主检出 | `0905d3cde59e336fad9faaab8660414d2e4c8c8d` |
| dependency | 组内 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean |
| 实现 cleanliness | 开始/门禁后均 clean，HEAD 未漂移 |
| reviewed SD | `reports/776-return-resource-spec-delta.md`，SHA256 `BB373282…C0940B9`；冻结副本 [reviewed-spec-delta.md](../attachments/776/review-r3/reviewed-spec-delta.md) |
| 输入 hash | [input-hashes.json](../attachments/776/review-r3/input-hashes.json) |

## 2. R2 finding 闭合状态（独立核验）

| R2 ID | 实现/反例 | 判定 |
|---|---|---|
| R2-01 声明授予身份 | `handler_declares_stream_return` 已成 dead code；identity 在返回边界建立（`call_fn_by_name_http` + `WRAP_SSE_RESOURCE`）；`declared_stream_plain_int_is_data`→JSON 0，`real_generator_still_sse`→SSE | **关闭** |
| R2-02 owner 7 位回绕 | `CREDENTIAL_OWNER_BITS=11`，`owner>=2048` 分配前拒绝；`owner_segment_exhaustion_rejected` 顺序 2047 中间 VM + live-A 交叉 | **关闭** |
| R2-03 setter UAF | 五 setter（含 bytes）走 `mutate_response_snapshot` + owned-slot（StakeGuard/rc_push）；`setter_return_survives_grace_window` 9000 步跨窗 201 | **关闭** |
| R2-04 容器滞留 | `release_task_credentials` 扫任务栈全部堆对象槽；`nested_record_error_releases_credential` Box{response}+1/0 回基线 | **关闭** |
| R2-05 client 裸 id | `push_handle_result` 凭据化；`client_response_keeps_status_body` 上游 201+`upstream` 保真 | **关闭** |
| R2-06 bytes 旧表路径 | `response_bytes` 迁移快照；`bytes_setter_roundtrip` body=AB | **关闭** |
| R2-07 alias 未解析 | `ApiType.alias_of` + `from_return_identity` target 展开 + `http.` 限定名；`alias_target_and_qualified_name_identity` | **关闭** |
| R2-08 oracle/健康/SD | oracle 大部补齐（External 时延、harness 结果传回、fmt 干净）；**SD/decision 未按 R2 重写**；future-cache 组合未分诊；生成真 TCP 仍未验证 | **未关闭** |

诊断表（Plan §R2 修复轮）与路径/状态调整属实；未发现新的实现回归。

## 3. 当前 findings

### F776-R3-01 [P1] 规范增量仍描述 R1 已废止规则，不能作为 canonical 沉淀源

- 类型：closeout 缺口 / 知识增量与实现失配。关联 R2-08；AC-07/08/09；T-01/T-08；SD-01/02/05/06。
- `reports/776-return-resource-spec-delta.md` 的 R1 修订节仍是现行拟议规则：
  - SD-02 写「iterator 命中制→`~Iter/~Stream` **声明门**」——R2-01 已裁决声明不得授予身份；
    现行实现是返回边界包装（generator 生产点 + `WRAP_SSE_RESOURCE`）。
  - SD-01 写 owner `7bit<<24` 且「跨 VM 同号不可能」——实现已是 11 位 + 耗尽拒绝。
  - SD-05 写「client int 句柄面保持表语义」——R2-05 已凭据化。
  - SD-06 仍停在「alias 入名集」而非 target 展开。
- R2-08 明文要求「再更新当前 task/AC/SD 表」；R2 修复提交 `882563143` 只改了 Plan 与
  verification，**未触 spec-delta**。
- 修复：按 R2 实现重写 SD-01..07 after 规则（返回边界身份、owner 11 位耗尽、
  setter owned-slot、容器级联、client egress 凭据、alias target/限定名、bytes 快照）；
  同步重写 decision §2/§3 中已废止的「无显式 owner id / client 保持 int / 声明门」表述。
  修订后冻结 hash 再进最终复审。

### F776-R3-02 [P2] 验证报告 AC 表与 R2 实现矛盾，AC-08 过判

- 类型：closeout 证据失配。关联 R2-08；AC-03/08/09；T-07/T-08。
- `776-return-resource-verification.md` 原始 AC 表仍写：
  - AC-01「iterator 族对照由 legacy 查表臂保留」——查表臂已退役；
  - AC-08 标「过」——同时文内又记「生成真 TCP=预存编译漂移未验证」。
- R2 曾裁定：「需可运行现行支持 fixture 的实编/wire 证据，或如实记其阻塞/未验证面」；
  诚实记录可接受为不接管 742，但**不能把 AC-08 标成 pass**。
- 修复：AC 表按 R2 终形重写；AC-08 改 partial（api_gen/back_proxy/merged 面已证 +
  生成真 TCP 未验证面单列）；AC-03 若继续以代表例覆盖，需在表内写明单入口编组
  （`resource_claim_reply`）是同形依据，并列出未逐格展开的组合边界。

### F776-R3-03 [P2] future-cache 组合调查未在 async 矩阵中分诊

- 类型：R2 指定未完成项。关联 R2-08；AC-03/04/06；T-07。
- R2 保留了 `future-cache` 探针（缓存 Future 再 await / replay），结论「初次请求即 500、
  重放 0；没有成功首回控制」，并要求「须在完整传播/async 矩阵中分诊」。
- R2 修复测试族无对应成功首回控制或分诊记录；不能默认为非缺陷。
- 修复：补成功首回 + 缓存 Future 重放的 wire 控制（或证明该形态属 705 状态机非本计划
  边界并给出最小复现与归属计划）；把结果写进 AC-03/04 对账。

## 4. AC 与门禁（本轮 actual）

| AC | verdict | 证据/限制 |
|---|---|---|
| AC-01 | pass | 碰撞族 + declared-stream-plain-int + real-generator；plan776 48/48 |
| AC-02 | pass | 传播探针 + 5 producer + client/bytes/setter + legacy plan326 内含 |
| AC-03 | pass | 单入口 `resource_claim_reply`；三 Future 形态 + i64/record/资源终值；closure 面 plan442 |
| AC-04 | pass | Failed→500；nested 清理；plan705 迟到/取消族 |
| AC-05 | pass | owner 11 位耗尽、stale/wrong-owner、Future 0xF0 守卫 |
| AC-06 | pass | 1000 闭环、常量重放、SSE 断连、setter 跨窗 |
| AC-07 | pass | alias target + 限定名 + 755/738/751 |
| AC-08 | **partial** | api_gen 46+1、back_proxy 守卫恢复、不泄漏裸 id；**生成真 TCP 未验证**（master 同形预存） |
| AC-09 | **fail** | 门禁无新增未解释红；但 SD/decision 未按实证更新（F776-R3-01/02） |

| 本轮命令 | 结果 |
|---|---|
| `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 -j1` | **48/48** |
| `cargo check -p auto-lang` | 0 error（预存 warnings） |
| `cargo tv` | 162/162 |
| scoped plan755/738/734/705 | 109/109 |
| `cargo test -p auto-man --lib api_gen` | 46 passed, 1 ignored |
| 裸 `cargo t --no-fail-fast` | 5301/5304；3 红单跑全绿（plan492/plan707×2 并行 flake，非本 diff 触面） |
| `cargo th -j 1 --no-fail-fast` | **123/124**；唯一红=back_proxy corpora（master 在册预存） |
| rustfmt --check 触面（含 mod.rs） | 干净 |

不跑 tf/taa/tt/tu：无相应触面；needs_fix 不启动合入/批量回归。

## 5. 诊断与 R4 入场要求

| 剩余不变量 | R2→R3 为何仍缺 | R4 前必须交付 |
|---|---|---|
| 知识增量=现行行为 | R2 只改代码与 Plan 交接，SD/decision 停在 R1 | SD-01..07 after 规则与 decision 表示节按 `088e47823` 重写并冻结 |
| AC 对账=实证 | 验证报告 AC 表未随 R2 改写；AC-08 过判 | AC 表终形 + AC-08 partial + 生成真 TCP 未验证面单列 |
| async 缓存 Future | R2 指定分诊未做 | 成功首回控制 + 缓存/重放 wire 或归属裁定 |

实现提交链 `a142756c9..088e47823` 保留；T-02..T-06 实现子项维持通过。
本轮不 merge/archive/cleanup、不发布 canonical/live ledger。
