# Native 暖编辑准入与拒绝分类（native-warm-edit-admission）

> **Status**: active（实现/证据 = PLAN-932，2026-11-04；关闭 D-927-2 的分类债。判据继承
> PLAN-914 归档复审 N3——typed 拒绝是 declared diagnostic verdict、覆盖全部增量 mn
> recipe——与 PLAN-911 的事实准入边界；本文件是暖编辑「准入比较 / typed 拒绝保真 /
> 父侧三态分判」的规范正文）。语料：`crates/auto-lang/test/cases/ace-m1/native-warm-admission-932/`，
> 固定组 `case-ace-native-warm-admission-932`；原 115 函数 t11 负载腿在
> `experimental/ace/tests/conformance/tests/host/mir_native_protocol.rs::performance::mir_native_incremental_formal_runner_gate`
> （ace-host 与 ace-performance 两入口）；父侧 typed 事实负控 = `auto-lang-core` `ac_mn::refusal_tests`（8）。

## 1. 准入比较（真接口事实，非文本/位置代理）

- 暖编辑快路径按**受检语义接口**比较：变更 decl 唯一、声明名序不变、签名重建
  （param 名/类型/结果，冷端同规则）与保留环境一致、候选接口/声明上下文
  `interface_unchanged`。body-only 编辑（如字面量/语句变化）必须真增量成功：
  改函恰一新 unit、其余 resident hit、whole/fallback=0、独立新退出值。
- `recheck_swap` 的守卫码分列（不共用一个"签名变化"）：
  `source.incremental.slot`（槽位/名字失配——会话完整性事实）、
  `source.incremental.owner`（method owner 记录缺失——保留环境完整性事实）、
  `source.incremental`（干净重建与保留签名真实差异——真签名漂移）。
  签名重建**内部**的真实 bind 级诊断（重复参数名、未知 record 注解等）原样返回
  （语言事实），不折叠成签名漂移。session 侧据此映射 typed kind：
  slot/owner → `session-or-candidate`；真签名漂移 → `signature-or-context-drift`；
  真实诊断 → `language-diagnostics`（带 typed ds 与阶段）。
- env 侧候选安装（commit）的守卫码族（`source.incremental*`/`source.foreign`/
  `source.stale`/`source.closed`）是身份事实 → `session-or-candidate`，绝不 typed 成
  语言拒绝（那会为会话完整性拒绝伪造 source 快照）。

## 2. typed 拒绝保真（发生处 → wire → 父侧）

- typed kind 由 Rust 枚举（`RefusalKind`）在拒绝发生处直接产生并序列化
  （kebab-case），**禁止**从 rendered 文本/last-error 解析或猜测。
- `diagnostics: "source-v1"`（仅 `ac/mir-native-scalar-v1`）打开 typed 事实通道：
  拒绝步记录携带 `refusal{kind, phase, source_error?, reason}`，首失败即停，
  `accepted_prefix` 语义见 [native-source-diagnostics.md](../auto-ac/native-source-diagnostics.md)。
  default 模式 wire 键集不变（兼容对照），仅有 legacy `refused` 字符串。
- 拒绝后：session 保留 accepted prefix（源/revision/单元 wire/object/manifest 与
  诊断归属不被错误提交），旧合法程序可继续，下一合法 body 编辑成功且与 fresh 等价；
  历史 A→B→A 编辑内容命中复用历史 unit（同指纹不重发）。

## 3. 父侧三态分判（oracle 比较之前先过事实准入）

| 事实 | 父侧判面 |
|---|---|
| 完整有效 typed 策略拒绝（kind ∈ signature-or-context-drift / invalid-edit / session-or-candidate / lower-admit） | diagnostic completion verdict（stage `verify`、code `mn.incremental.refused.<kind>`），按声明 expect 判 PASS/**FAIL**；故意签名控制保持 success/exit 期望 → 该 case FAIL（控制测试断言 Fail 即通过），声明精确 diagnostic 期望（I09 形）→ PASS |
| 真实语言诊断（source_error 快照准入通过） | raw stage/code 判面（text→parse、bind→resolve、verify→typecheck），字段/顺序/span/候选源绑定保真 |
| 缺/坏 typed 事实：default 模式 untyped `refused`、缺/未知 `refusal.kind`、`untyped`/`resource` kind、语言/非收敛 kind 出现在 refusal 面（应走 source_error/fallback rounds） | **Error**（infra），不降级为 FAIL 掩盖缺事实 |
| 找不到替换 literal 等父请求错误、协议破损、非零 worker 退出、超时、缺产物 | Error（沿 911/914 既有判面） |

- 反控锚点：宿主门 (c) typed→Fail、(c2) 精确码/阶段 Pass、(c3) untyped→Error、
  (d) 坏 literal→Error；套件 7 case（body 成功/语言拒绝/修复/签名拒绝typed/状态存活/
  多decl invalid/历史回退）；`refusal_tests` 8 单控（无拒绝/合法kind/缺对象/缺kind/
  未知kind/untyped+resource/矛盾kind/严格形末步）。

## 4. 边界与余账

- 新进程会话恢复→首编辑（V-09 腿）pending：worker rounds 每请求冷建会话，
  `session::restore` 未接协议；通用来源状态/恢复统一归 PLAN-929（在其稳定接缝上联验）。
- D-911-3（syntax-not-converged 无自然样本）按原 owner 保留；本文件不开放
  signature/语言/resource 的 fallback（唯一允许 fallback 的种类仍是 syntax-not-converged）。
- 正式 P90≤2000ms/原样本/115fn 负载与平台条件不变（`::performance::` 档，单实例）；
  t11 宿主门在 ace-host 与 ace-performance 双入口同一测试函数。
