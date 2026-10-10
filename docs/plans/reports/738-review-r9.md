# PLAN-738 补充复审 R9（2026-10-10，needs_fix）

- stage: review / plan_revision: 3 / outcome: **needs_fix**。
- reviewed_commit: `19973b0899f27267db15a39669c124579f41a505`，入场与结束实现树 clean。
- main 入场：`c212e1c7a71b0bfea6972b0d1c9b86d78eeb794b`；重放基面 `1be1783fea45162fbcae0c6a21859d2fa2588608`；修复父提交 `adfb7a5d1`；前次通过树 `3713337d97f37d68bddaca22ce16fb152cfc8db5`。
- dependency: auto-down `895f8d0f9355c9f5ec3ce8fca268bdb768395846`，只读未改。
- independence: 本上下文曾修订计划并进行 R5 补充复审，未参与 R5–R8 修复实现、R8 复审或 merge 修复；本轮从实际提交/反例重建判断，不冒称全新独立上下文。修复后交独立复审。
- 不修生产代码、不 merge/归档、不发布 canonical/ledger。临时审查测试按原始字节恢复；仅主检出计划簿记。

## 已修复面与实际范围

主链 `execute_autovm_with_deadline` 不再把无契约 stdlib FFI 接口重复 merge 到 production VM。`AutoVM::new` 的 production 三件套、最终契约落位与执行前引用校验仍在；746 的 deadline/join 处理保留。`cb_web_mime` 在本轮 tv 中通过。此次 rebase 引发的假红根因已修复。

`3713337d9..19973b089` 中，stdlib_assembly、Rust collector、api_gen、rust_ui、compile、persistent 与 stdlib 内容不变；VM 主链/ffi 的非格式差异对照来自主线 746/747 等新增能力。range-diff 显示 Phase 3/R5/R6/R7 提交重放补丁相等，旧 SHA 已变但不能因此假定验证有效；本轮重跑受影响门与真实见证。

最新修复提交实际包含 **14 文件、21207 additions/13239 deletions**，并非只有主链删除。逐文件用同版本 rustfmt 标准化父树与最终树：13 文件没有 token/行为差异，lib.rs 唯一行为差异为删除重复注册块（另有说明注释）。renderer/engine 标准化后的剩余差异仅布局；本轮健康检查发现最终树尚未达到格式固定点，见 R9-02。无新增调试插桩。

## 必修发现

### P738-R9-01 / P2：未绑定 lock 的读取失败仍被当成真正缺失

- 位置：`crates/auto-man/src/rust_ui.rs:4025` 的 `read(...).ok().map(...).or_else(absent)`；生成侧 `api_gen.rs:1071` 的 `unwrap_or_else(|_| absent)` 同样丢弃错误类别。
- R5 的永久 `absent` 豁免已修复，首次读取成功后确实绑定并严格比较。剩余角落是**尚未绑定**状态：读取返回 PermissionDenied 等错误，被转成 `absent`，与收据 `absent` 相等，于是新鲜度门返回 true。不能把不可读取作为缺失证明。
- 实际反例：正式 `generate_api` 在隔离工程/隔离 workspace 写收据，确认 `workspace_lock=absent`、真正未建 lock 时 fresh；随后在同路径建 `Cargo.lock` 目录，使真实 `fs::read` 返回非 NotFound 的 PermissionDenied。实际 `backend_generation_is_fresh` 返回 true，收据原文未变。未改业务源、stdlib 或运行实现。
- 复现：在 plan worktree 执行 `python D:/autostack/auto-lang/docs/plans/reports/738-review-r9-probe.py`。脚本复用 R5 harness 的 clean 检查与 finally 精确恢复，只注入临时 cfg(test) 审查测试，命中 1 项；测试退出 101 为反例断言失败，非环境红。

```text
R5_PROBE R9 recorded=absent lock_read_error=PermissionDenied is_fresh=true
unreadable lock must not be treated as genuine absence
test result: FAILED. 0 passed; 1 failed; ... finished in 0.85s
restored_exact_bytes=true
```

- 修复：生成写收据与复用读 lock 两端仅将 `ErrorKind::NotFound` 解释为 absent，其余错误分别传播生成失败/判陈旧；不能先用 `.ok()` 擦掉原因。保留真正缺失与一次成功绑定的既有语义，补 unbound/bound × NotFound/其他读取错误状态反例，恢复可读后证明能收敛。重跑正式生成服务链。
- 映射：AC-06 fail、AC-07 partial、AC-08 fail；T-12/T-13/T-14，SD-05。R5 已明确要求读取失败拒绝；属于原合同遗漏，不需新计划/新 revision，不允许弱化规范。

### P738-R9-02 / P2：格式健康门失败，与交接 clean 声称不符

- `rustfmt --edition 2021 --check --config skip_children=true <14 touched Rust files>` **exit 1**；不加 skip_children 的默认检查同样 exit 1。
- 实际位置：`lib.rs:119`（注释缩进）、`ui/iced/renderer.rs:24761`（launcher match）、`vm/engine.rs:9840/9852`（pop 表达式布局）。rustfmt=`1.9.0-stable (88d9e12ae1 2026-08-18)`。
- 在内存中连续格式化三份文件得到固定点序列 `[false,true,true,true]`，是可收敛的一次剩余格式调整，不是无法解决的 formatter 振荡。
- 修复：只格式化这些位置并提交，重跑实际触面 check；把最终报告“rustfmt 干净”绑定到真正通过的新提交。不要因旧 scope 很大而再扩大格式化范围。
- 映射：AC-08、T-14，AGENTS §3 Health Check。review 不顺手修实现。

## 本轮门禁与证据

| 验证 | 实际结果 |
|---|---|
| `cargo t --no-fail-fast` | 5224 run / 5210 passed / 14 failed，102.539s |
| `cargo tv` | 162/162，3.960s，含 cb_web_mime |
| `cargo tt --no-fail-fast` | 5595 run / 5585 passed / 10 failed，108.418s |
| plan738 / plan724（完整日常档实际命中） | 73/73、5/5；含 12 方法×双拼写矩阵、真实三目标身份与失效测试 |
| CLI stdlib | 10/10，0.64s |
| auto-man api_gen | 44 passed / 1 ignored，1.64s，含真实生成消费者腿 |
| R5 lock 一次绑定状态机 / 两张 freshness 真值表 | 1/1，0.98s / 2/2 |
| 三项 ignored 真编译执行见证 | 3/3，7.08s；Rust host provider、VM=41/Rust=42/C=43、C stdio=65 |
| 正式生成服务完整链 | **1/1，48.95s**：生成→收据→实编→serve→ready→业务→stdlib-only 失效→再生→重建 |
| 三 crate check / 实际 auto.exe build | exit 0，10.90s / exit 0，25.57s；存量警告保留 |
| CLI 实际反例复验 | 基线 exit0 + parse/JsonValue.keys/JsonValue.has_key 三项 signature_checked；has_key int→str 漂移 exit1 DRIFT；同名 json 变量 exit0 |
| HTTP 全档（串行） | 102 selected，**45 完成：43 pass/2 fail**；后续停止，57 项未完成，**不记全档通过** |
| 失败隔离复跑 | plan484_024 2/2，0.344s；plan502 1/1，0.299s；plan707_client 与 plan606_029 本轮隔离仍红，保留预存失败归属 |
| diff check | exit 0 |
| 格式健康门 | exit 1（R9-02） |
| R9 不可读 lock 负例 | 门误放行，负面审查断言失败（R9-01） |

日常 14 红逐名：p053_4、plan484_024、plan502、plan707_client、748 handler dynamic/static ×2、plan606_029、desktop_protocol projector、iced renderer ×2、ash_leak、schema_drift ×2、docs_gen。tt 10 红为上述子集，未出现新名称。748 两项在归档计划 748 §T-00 定音/§9 已明确是 in-proc harness 不泵 HTTP 续体，marker=unset，真实 E2E 绿；不是本树新引入。其余沿 R5–R8/批量回执的预存与负载 flake 名册分诊；本轮 plan484/502 隔离绿，plan707_client/606_029 隔离仍红，不能把后两项写成“本轮复跑绿”。完整失败名字与日志 hash 入 baseline JSON。

HTTP 已完成的两红为 back_proxy corpora data face（400/200 的在册基线）与 plan730_client730_server_interop（109.655s，固定端口 18980 upload 传输失败，与 Phase 3 verification §2/3 的历史反例同形）；后续 legacy_bind_failure_cleans 又超过 60s。鉴于本轮已由 R9-01/02 确定 needs_fix，停止本轮 controller 及经父子链核验的本 worktree 测试进程，保存 stop 记录；没有把未跑部分计绿或用缩减 filter 清账。下一次最终收口仍需完成全档/逐名基线分诊。本轮已完成的帧时序 `http_e2e_plan707_relay_frames_timed_single_termination` PASS@2.624s、断连取消 PASS@2.606s，仅证明本轮该用例通过，不宣称时序 flake 已被长期排除。

实际 CLI 文件 SHA256=`f53528cdc5953e53d59866779e04b483dbe6c27f52cf095f0272e721d1b6d966`。三份 CLI 探针使用此新编译二进制与隔离 AUTO_STDLIB_ROOT；不是以提交号替代二进制身份。

## AC / SD 与下一步

| AC | 本轮判断 | 任务/证据 |
|---|---|---|
| AC-01 | pass | T-01/02/09：inventory 全分母与六模块矩阵测试，最新 14 文件分类 |
| AC-02 | pass | T-03/10/11/12：共同入口、三目标真实执行/身份、persistent 在完整档验证；R8 未变来源事实 |
| AC-03 | pass | T-04/10/11：production 契约顺序、核心严格门/方法分母漂移测试，主链假红修复 |
| AC-04 | pass | T-03/05/11：源映射/use/用户同名隔离测试与原实现一致 |
| AC-05 | pass | T-04/07/10/11：六核心 target/env 分母，Unsupported/真实漂移诚实拒绝 |
| AC-06 | **fail** | T-05/06/12：已有内容失效与成功绑定序列绿；不可读 unbound lock 被 R9-01 证伪 |
| AC-07 | **partial** | T-06/07/12/14：CLI/服务真实链绿，但复用门不可读状态不能核验 |
| AC-08 | **fail** | T-08/09/13/14：R9-01 与格式健康门阻塞最终收口 |

审查 WT 内 `adfb7a5d1` 的待合入 SD 沉淀：SD-01/02/03/04/06/07 没有本轮新语义矛盾；SD-05 的读取失败保守拒绝规则应保留，当前实现未完全兑现，**不批准其“已实现”验收**。冻结 SD 草稿与七 canonical + 新 assembly-manifest 文档 hash 见 baseline JSON；未发布主检出 canonical/ledger。R8 两项非阻塞观察继续保留，不重复升级。

计划保持 executing，重开 T-12 的 lock/读取失败验收及对应报告、T-13 对账、T-14 最终门禁；current_step 从 7 重算为 **4**（T-01/T-09/T-10/T-11 完成）。历史 R8 pass 与此次主链修复证据保留，但不能覆盖 R9 反例。执行 agent 在同一 Phase 3 修复两项，提交并补反例/更新最终证据后，再交独立复审；本轮不进入 merge。
