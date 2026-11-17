# ace-migration: m2-man-gen — AutoMan 配置/任务模板与生成器迁移（PLAN-982）

> Status: **PLAN-982 的 M2-A 切片（route C，用户 2026-11-17 裁定）已交付**。
> 完整 CONFIG/EXEC/TEMPLATE/RUST/C/WEBTARGET/PY/REVERSE/EXPORT/DEVSDK 产品迁移
> **未交付**（M2-B）；配置**引擎**迁移到 ACE 是具名、owner 未指派的缺口
> `GAP-982-CONFIG-ACE`（[PLAN-982](../../plans/982-ace-m2-man-gen.md) §5/§10）。
> 来源：PLAN-982 §5（T-01 冻结记录、A 切片定义）、§9（work/review/merge 收据）。
> 实现：auto-man `crates/auto/src/ace_candidate.rs`、auto-lang
> `crates/auto-lang-core/src/sdk_identity.rs`。
> 本文件只写**已验现值行为与边界**；未列入的能力不视为已支持。

## 1. A 切片的范围（route C）

RG-A01 的 A 切片只证明产品侧"**一次运行里，配置读取腿与最小 CLI 执行腿属于同一个
SDK 构建，且执行腿不走旧引擎**"。配置**求值引擎**按 PLAN-969 已验门的冻结面
（[m2-facade](./m2-facade.md) §边界）仍是 SDK 的**指定 legacy owner**：
`eval_config_with_vm` 对 `pac.at`/`am.at` 文档求值，ACE 门当时明确没有复制该面。
因此本切片证明的是**身份 + 纪律**，不是配置引擎的迁移。

| 面 | A 切片交付 | 不属 A（具名差额 → M2-B） |
|---|---|---|
| 配置消费 | 产品配置读取仍由 SDK 指定 owner 完成，并与执行腿同属一构建；候选腿对真实文档求值并记录逐值证据（含产品自己的 `Pac` 目标图） | 配置引擎迁移到 ACE（`GAP-982-CONFIG-ACE`） |
| 执行腿 | `auto <script.at>` 的**显式 ACE 候选入口**（opt-in），经 `run_ace` console 车道真实执行 | run/service/req/debug/test、模板、各 target、watch、安装 |
| 身份 | 两腿身份 + **实际服务引擎**一致性断言，落盘 receipt | SDK 候选安装/分发身份（B-DEVSDK） |
| 纪律 | argv/dump/诊断/拒绝/资源/取消一律具名拒绝且零 legacy dispatch；坏配置执行前拒绝且工程树零写；两工程隔离 | 全产品失败面（B） |

## 2. 已验接口（可依赖）

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 显式选择 | 环境变量 `AUTO_ACE_CANDIDATE=1\|true\|yes`；默认（未设置）完全走既有路径 | `ace_candidate::selected()` |
| 构建闸门 | Cargo 特性 `ace-candidate`（**不进** `default`/`core`）；桥依赖 `auto-lang-ace-host` 只在 `cfg(windows)` 目标声明 | `crates/auto/Cargo.toml` |
| 平台 | 候选腿只在 Windows 可用；其他目标以 `ace.host/unsupported-platform` 具名拒绝（绝不回落旧引擎） | `ace_candidate::run_script` |
| 请求形状 | `AceRequest{console:true, grants:["console.stdout","console.stderr"], wall?, cancel?}`；`grants` 每 run 显式授权 | `run_script_on_ace` |
| 成功输出 | 先重发 ACE 捕获的 stdout/stderr 字节，再按旧腿位置打印返回值；human 模式补同样的尾随空行 | 同上 |
| 退出 | 成功 0；一切拒绝非 0（human=miette 诊断，`--ai`=JSON `{"status":"error","code","message"}`） | `refuse()` |
| 具名拒绝码 | `ace.args/unsupported`、`ace.dump-lowered/unsupported`、`ace.host/unsupported-platform`、`ace.host/install-failed`、`ace.source/read-failed`、`ace.config/invalid`、`ace.diagnostic/<stage>/<code>`、`ace.resource/<Kind>`（如 `ExecutionWall`）、`ace.run/cancelled`、`ace.sdk/identity-mismatch`、以及 ACE 自身拒绝码（如 `ace.resource/InstrSteps`） | 同上 |
| 回执（测试钩子） | `AUTO_ACE_PROOF_RECEIPT=<path>` 写 JSON：`schema`、`sdk`、`same_sdk`、`config{owner,path,present,values,deps,targets}`、`execution{owner,engine,engine_is_identity_engine,outcome,detail,steps,stdout_bytes,stderr_bytes,dispatches}`、`script`、`script_bytes`、`started_unix_ms`、失败时 `refusal{code,message}` | `write_receipt()` |
| 预算/取消钩子（测试） | `AUTO_ACE_WALL_MS`、`AUTO_ACE_CANCEL_AFTER_MS` | 同上 |
| 身份合同 | `auto.sdk/1`；`config_owner_identity()`/`execution_owner_identity()`/`sdk_identity()`/`same_sdk(a,b)`；纯数据、零求值、零 ACE 依赖 | `auto_lang_core::sdk_identity` |

**身份谓词的含义**：`same_sdk` 是**构建一致性**检查（两腿 package+version 相同），并且
候选腿要求 ACE 门报告的**实际服务引擎**等于身份里写的引擎
（`report.engine == EXECUTION_ENGINE_ID`）；两者同时成立才写 `same_sdk: true`。它**不是**
引擎语义等价证明。

## 3. 已验证行为（永久验证）

`crates/auto/tests/plan982_ace_candidate_config.rs`（需 `--features ace-candidate`）：

1. `p982_a_config_and_execution_share_one_sdk` — 真实 `pac.at`（name/version/scene + 一个
   `dep("kernel")`）经 SDK owner 求值，产品 `Pac` 投影得 `deps=1`/`targets=1`；脚本经 ACE
   console 车道执行，`engine=av`、`detail=7`、stdout 含 `42`；receipt 两腿同 package+version。
2. `p982_a_dual_projects_are_isolated` — 同脚本、两工程不同 `pac.at` 各自解析，不串值。
3. `p982_a_bad_config_refuses_before_any_write` — **解析失败**（`Expected term`）与
   **求值失败**（`Undefined variable`）两种文档都 `ace.config/invalid`、`execution=not-started`，
   且工程树字节级不变（内容哈希快照）。
4. `p982_a_candidate_never_falls_back_to_legacy` — 两臂 oracle：旧腿对同一命令
   （argv 透传）成功且出 `42`；候选腿 `ace.args/unsupported` 且 `42` 零出现；
   `--dump-lowered` 在候选下`ace.dump-lowered/unsupported`、不打印 lowering，旧脚仍正常。
5. `p982_a_bounded_run_refuses_instead_of_legacy` — 同一脚本无预算可成功（非空证），
   wall=1ms 得 `ace.resource/ExecutionWall`、取消钩子得 cancelled/resource-denied。

`crates/auto/tests/plan982_ace_candidate_feature_gate.rs` — 特性关闭时打印显式跳过说明，
避免"0 test 绿"。

语言侧：`cargo test -p auto-lang-core --lib sdk_identity` — 身份 schema/两腿同源/异版本或
畸形身份不通过。

**实测（交付时，Windows）**：core 形态与 full 形态各 `5/5 PASS`（0.4–1.3s）；默认/`core`
关特性 `cargo check` 绿；新增代码零告警；`rustfmt --check` 干净（`cmd_run.rs` 的既有
rustfmt 状态与本次无关）。

## 4. 边界与开放项

- **配置引擎**：`GAP-982-CONFIG-ACE`——ACE 无 config/document `SourceProfile`、无文档值
  种类、`canonical_owned` 拒 `Object`，真探针 `av-worker request` 对真实 `pac.at` 直接
  `frontend:source.syntax` 拒；owner 未指派，归 M2-B。
- **argv/stdin**：AV 车道无输入面（PLAN-969 T-08）→ 候选腿具名拒绝，不静默丢弃。
- **`--dump-lowered`**：旧审查面，候选腿拒绝；B 迁移时另行处理。
- **非 Windows**：候选腿具名拒绝；不承诺跨平台（与 `auto-lang-ace-host` 一致）。
- **身份强度**：receipt 的 `version` 是 crate 版本，不含 git 修订；`identity-mismatch`
  负臂在当前单一桥下构造不出（按构造不可达，未伪造负例）。
- **输出节奏**：内容与返回值位置已对齐；个别空行的"字节级完全一致"未逐字节固定。
- 完整产品迁移（EXEC/TEMPLATE/RUST/C/WEBTARGET/PY/REVERSE/EXPORT/DEVSDK、安装/升级/退役）
  与 959 行的产品级闭合仍在 M2-B；本文件不声明这些能力。
