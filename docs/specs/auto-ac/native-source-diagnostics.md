# auto-ac native 源码诊断事实通道（PLAN-911）

> Status: active（实现 = `experimental/ace/crates/native/src/source_diagnostics.rs`、
> `driver.rs::{compile_escalating,check_source}`、`incremental.rs::{edit,close_exit,typed_refusal}`、
> `bin/mn-worker.rs` 的 `DiagMode::SourceV1` 分支；父侧 = `crates/auto-lang-core/src/test_cases/ac_mn.rs::source_diag`；
> decoder = `crates/auto-test-case/src/contract.rs` `mn.diagnostics`）。实验通道，非生产默认入口。

## 目的与边界

- 把**共享前端/HIR 验证已经判定**的真实 `Vec<Diagnostic>`，以**操作级不可变 typed 快照**透传到 native
  worker 与父侧 TestCase 判面，使 M1 B17/B18/B19 的 native 格位按真实 raw stage/code 验证，而不是
  泛化壳 `mn.unsupported.frontend`。
- **不**新增任何语言检查器、**不**重跑编译器取因、**不**迟读 session "last errors"、**不**按 expected
  挑选原因；`primary` 恒为编译器返回顺序的第 0 条。
- 默认（未选模式）路径字节/键集不变：`MnError::Source` 的 `code()`/`message()` 是旧
  `mn.unsupported.frontend` + `"{Stage:?}/{code}: {message}"` 的同事实投影。

## 快照 `mn.source-diagnostic/1`

| 字段 | 含义 / 约束 |
|---|---|
| `schema` / `mode` / `span_basis` | 字面 `mn.source-diagnostic/1` / `source-v1` / `utf8-byte` |
| `producer` | 实际调用点 `ace-core.source` |
| `profile` | 真实**终止**域：`scalar` / `record` / `text-record` |
| `attempted` | 升级链实际尝试序，末项 == `profile` |
| `surface_rejections[]` | 仅用于升级的 `source.unsupported@Text` 拒绝（`{profile, codes[]}`），**不是** primary 候选 |
| `path` | producer 实际接收路径（cold `mn.at` / warm `probe.at`），不是 case 逻辑路径 |
| `source_sha256` / `source_len` | 编译器实际看到的候选源身份 |
| `causes[]` | `{stage, code, message, span{start,end}}`，`stage` = `Stage::as_str()`（text/bind/verify/capability/backend/link/run），顺序 = 编译器返回顺序，**不去重** |
| `primary` | JSON 整数 `0` |
| `operation` | `{kind: cold\|warm-edit\|fallback-cold, step, base_sha256, candidate_sha256, session_revision, edit{start,end,insert_len}}`；`step` = 请求内 **0-based** 步骤索引（一次请求=steps 下标；rounds=父侧回合序），请求级 cold 为 null |

**上限（native 诊断分区，超额为 `mn.resource.source-diagnostic`，不截断授信）**：causes ≤ 64、
单 message UTF-8 ≤ 4096 B、整快照序列化 ≤ 65536 B。空诊断列表不构造快照（保留旧 `frontend failed`）。

## 生产点（全部出口）

| 出口 | 操作 | 说明 |
|---|---|---|
| `MnDriver::request` cold | `cold` | `compile_escalating` 返回终止 profile/attempted/surface；失败 → `MnError::Source` |
| `MnDriver::check_source` | `cold` | 仅前端编译（0 codegen/link/run），严格 fallback 的逐候选语义门 |
| `MnIncrementalSession::warmup` | `cold` | mn 单元编译**先于** 828 session（同编译器同源，不新增第二次编译），warmup 拒绝=typed 事实 |
| `MnIncrementalSession::edit` | `warm-edit` | 从 AV `EditOutcome.rejection` 的 typed ds 构造；绑定 base（accepted prefix）/candidate SHA、session revision、编辑区间 |
| `whole_lower_fallback` | `fallback-cold` | 记录型 whole 回退的编译失败 |

`edit()` 的**每个**出口（拒绝/infra/成功）经 `close_exit`：结构 delta（unaffected/whole）与 core
`body_work` 捕获窗口在失败路径上同样计量；拒绝携带 `EditRefusal{kind, phase, source_error, reason}`。
`RefusalKind`：language-diagnostics（终态，不进 fallback）/ syntax-not-converged（唯一允许显式 fallback）/
signature-or-context-drift / invalid-edit / session-or-candidate / lower-admit / untyped / resource。
kind 由 Rust 枚举直接序列化，**禁止**由 rendered 文本解析。PLAN-932：`session-or-candidate` 同样承接
保留环境守卫的槽位/owner 失配与 env 安装身份族（见
[function-incremental-source.md](function-incremental-source.md) 来源与失效轴）；父侧三态分判
（有效 typed 策略拒绝=verdict、真实语言诊断=raw stage/code 判面、缺/坏 typed 事实=Error）见
[../auto-test-case/native-warm-edit-admission.md](../auto-test-case/native-warm-edit-admission.md)。

## worker 协议（外层 `mnworker/1`、`mnworker/1.0` 不变）

- 请求 `diagnostics: "source-v1"`（仅 `ac/mir-native-scalar-v1`；未知值 → `mn.worker.request`）。
- 严格模式下**每个** `ok` 容器 echo `diagnostics_mode` / `diagnostics_schema`（合法程序亦然）；
  旧 worker 静默忽略 → 父侧 ERROR。
- 互斥结果：① 成功 = echo + 原执行结果；② 源码拒绝 = 顶层 `source_error` 快照 + legacy
  `diagnostic{mn.unsupported.frontend}`、`ok` 仅元数据、**无**执行结果；③ 其他 typed 拒绝 = 原
  `error`/`diagnostic`（code ≠ `mn.unsupported.frontend`），无 `source_error`。
- 增量（一次请求多 step）：typed 拒绝**首失败即停**，`ok.accepted_prefix{steps_applied, source_sha256}`，
  后续 step 不执行；`syntax-not-converged` 走严格逐候选 fallback（非末编辑候选各一次
  `check_source` 门，首语义失败停，全过才一次整请求）。
- rounds：warmup/拒绝/run 各出口同 echo；语言拒绝答复本回合（快照入 `ok`/顶层），session 保
  accepted prefix，**父侧**停止后续 step。
- 默认模式保留旧行为（含"拒绝后继续后续 step、响应层不报 exit"的覆盖中间态行为，作兼容对照）。

## 父侧严格准入（oracle 比较之前；失败 = `Status::Error`，非语言 FAIL）

1. 原 schema/nonce/request_hash/recipe/worker_version/mn_abi 门。
2. `diagnostics_mode`/`diagnostics_schema` echo 必在。
3. **原始字节**重复键扫描（`source_error` 对象及其嵌套，任意顺序）。
4. 精确字段集（快照/cause/span/operation/surface）、字面值、profile/升级链真值（末项=profile）、
   producer path、hex sha、`source_len`==父侧候选字节数、causes 1..=64/message ≤4096/payload ≤65536、
   span `start≤end≤len` 且两端落在候选 UTF-8 char boundary、`primary` 为 JSON 整数 0
   （`"0"`/`0.0`/`false` 拒）、stage 仅 text/bind/verify 可承族（其余 ERROR，不默认成族）。
5. operation 按**父侧自身视图**绑定：cold = candidate SHA == 父发送源、base/session/edit 为 null
   （rounds 内 cold 整请求则 step == 父侧回合索引）；warm-edit = base == accepted prefix、candidate == 父自算
   候选、**step == 父侧自身 0-based 回合索引**；fallback-cold 同 base/candidate/step。`source_sha256 == candidate_sha256`。
6. 与执行结果值级互斥（非 null exit / 非空 stream / manifest_fp 与 `source_error` 共存 → ERROR）。
7. legacy `diagnostic` 必为 `mn.unsupported.frontend` 且 message == causes[0] 投影。
8. 通过后 raw stage 映射判面：`text→parse`、`bind→resolve`、`verify→typecheck`；code 原样比较。

## 正式父报告落点

父侧校验通过后写 `artifact_dir/mn-source-diagnostic.json`（schema `mn-source-diagnostic-fact/1`：
`execution_key`（suite:sha:case:check）、admitted、mode/schema、recipe、worker_version、
source/candidate SHA、raw_stage、observed_stage/code、primary_message、primary_index 0、快照原样），
写后重读校验，经 `ArtifactRef` 发布；写失败 = ERROR。**不**改 `NativeRecord`/`test-report` 键集。
M1 格位回填只据该 artifact + 父报告。

## 入口支持矩阵（不借 capability echo 冒支持）

| 入口 | 严格观察 |
|---|---|
| whole source / worker 普通 run（scalar→record→text-record 升级前门） | 是（cold） |
| `serve_incremental` / `serve_rounds` → `MnIncrementalSession`（scalar） | 是（warm-edit / fallback-cold） |
| `OwnedSourceMnSession`（record/text 暖 API） | 否——原 `MnError` 转换/默认计量保持，非本通道 |
| typed / collections recipe、default/v1 | 否——`mn.diagnostics` 在这些 recipe 上 decoder 拒绝 |

## 已知边界（具名）

- `syntax-not-converged` 严格 fallback 门：代码路径与单元逻辑完备，当前树内**无端到端可触发的
  非收敛合法编辑形状**（828 引擎限制形状未知），无真实 fallback 样本（PLAN-911 §9 登记）。
- 正式四状态计量（`::performance::mn_sd911_source_v1_four_state_p90_30`）：restart-warm / body-edit /
  signature-reject 门 P90 ≤ 2000 ms；`fresh`（debug tier 冷编译 115 函数 + 115 COFF + link + run）
  按 native tier 既有惯例**报告不门**，并附 default 模式对照（同成本级）。
