---
plan_id: PLAN-872
status: archived
feature_name: 内建语义双轨裁定与收敛（Str/Math/Time/Reply/nv_to_value）
author: [DeepSeek Harness session]
created_at: 2026-10-28
updated_at: 2026-10-29
plan_revision: 3
current_step: 10
total_steps: 10
supersedes_spec_components: [docs/specs/stdlib/project.md, docs/specs/auto-lang/vm/overview.md]
new_spec_components: []
touched_goals: []              # 有意留空：收敛实现、不改 GOAL 定义；GOAL-003/007 的语义一致性是本计划的动机而非目标变更
affects: [language-sdk, stdlib, auto-ui, auto-man, test-infrastructure]
depends_on: [PLAN-867, PLAN-869]
---

# [PLAN-872] 内建语义双轨裁定与收敛（Str/Math/Time/Reply/nv_to_value）

> **整 Plan 启动门 G872 = U867 且 U869**：867 交付三仓完整档与 ID 闭合（验收基础），869 的减法批次不得先动这 6 族（869 §1 非目标已声明）。门未齐只做只读核查与规划簿记。
> 本计划处理 [PLAN-844](archive/844-cross-repository-dedup-and-net-slimming.md) §2 保留表里明确"超出减法授权"的 6 族；**它们是语义问题，不是重复代码问题**，因此不能沿用"删重复"的判据。

## 0. 变更摘要

844 在去重时冻结了 6 族"看起来重复、实际语义不同"的内建实现，并把它们排除在减法之外：`str.len`（字符 vs 字节）、`math` 20+ 族（0 兜底 vs Err）、`to_int`/`parse_int`、`time_now`（`unwrap_or(0)` vs `unwrap`）、HTTP error-reply 采样族 ×10、`nv_to_value` 双签名。它们的真实差别常常**不在函数体，而在注册/绑定序**：同名 builtin 由"纯 shim（catalog/inventory）"与"native 覆盖块"共同决定最终生效实现（844 D-4 已实证：纯版 `contains/starts_with/ends_with` 恒被覆盖，删纯版后须显式补名字注册）。

因此本计划的第一步不是改代码，而是**冻结一张真实的 builtin 绑定表**（名字 → 注册面 → 直接实现 → 生效实现 → 语义 → 消费者 → 钉住它的测试），再按用户 2026-10-28 的裁定对六族**统一按 DISTINGUISH 处置**（保留两侧、把名字/文档/测试讲清楚，零行为变更）；MERGE／RETIRE 均不在本轮实施面（RETIRE 仅在某侧完全无消费者时另行签核）。

对比例的已见实例：`Str.len` 纯 shim 的文档写 "length in bytes"、实现是 `s.len()`（字节），而同文件 `Str.byte_len` 的注释称 `str.len()` 是 PLAN-055 的**字符数**语义——三者至少有一个是错的，DISTINGUISH 的活就是把"谁在运行时生效"实测钉死，再把名字/文档/测试对齐。

## 1. 目标

- **G1 绑定表可复算**：6 族涉及的全部 builtin 名字，按真实注册/覆盖顺序列出"生效实现"，并可用最小程序实测覆核（不是读码猜测）。
- **G2 每族有裁定（已定）**：用户 2026-10-28 裁定 **六族全部按 DISTINGUISH 落地**（保留两侧，把名字/文档/测试讲清楚，**零行为变更**）；MERGE 未获授权，本计划不得实施任何用户可观察的行为变更；RETIRE 仅在 T-02 证明某侧完全无消费者时提出，并需单独签核。
- **G3 无静默漂移**：处置完成后，名字、文档注释、规范页、测试四者对同一语义一致；`str.len` 这类"名字/文档/实现三方矛盾"不得残留。
- **G4 不破坏既有承诺**：`plan755` 的 HTTP 契约 body、JS/Vue 的 `.length` 语义、a2r 生成代码的调用形态保持；改变任一承诺须显式记录并取得用户同意。
- **G5 门禁联动**：定向验证与账本在work关闭，完整档在final review后由merge执行并继承867限制口径；本轮不实施MERGE，测试改名或另获签核退役均须账本有据。

**非目标**：不做 869 的 crate 清单与减法；不重写 stdlib/VM 架构；不动 `lib-legacy`、aavm、`type_registry` 等 844 保留有证项；不新增 backend 或新内建名字（除 DISINGUISH 需要的新名字）；不把 error-reply 采样族当成性能优化去重写（仅在同 body 字节前提下才允许抽 helper）。

## 2. 架构方案

1. **绑定表（T-01）**：从三处真源读：① `#[auto_macros::rust_fn("Name")]` 属性注册面（纯 shim 轨道，如 `ffi/stdlib/*.rs`）；② native 覆盖块/名字表（`vm/native.rs` 与 `vm/native/*.rs`）；③ 运行时 `production()` 绑定序（catalog → inventory → nv 覆盖）。产出 `name ⇥ 生效实现 ⇥ 语义 ⇥ 证据`，并用最小 Auto 程序实测对照（VM 侧"字符 vs 字节"等可直接观测）。
2. **裁定（T-02）**：六族已裁定为 **DISTINGUISH**；T-02 仍须逐族出证据（消费者/公开承诺/语义差异）并确认"不存在完全无消费者的单侧"（若存在则提 RETIRE 候选、单独签核）。**MERGE 未授权，不得作为实现路径**（未来若要合并，另开 revision 并取得用户裁定）。
3. **实施（T-04）**：按 DISTINGUISH 改文档/补名字/补测试，**不改运行时行为**；MERGE 未授权，不保留迁移窗口（未来若要合并另开 revision）。
4. **收据（T-05/T-06）**：行为对照收据（同程序两侧输出）、契约测试（plan755）复跑、867 门复跑、规范页更新。

判定原则：

- **以运行时实测为准**，不以文本相似或代码注释为准（844 的教训）。
- MERGE 的成立条件（"两侧语义可被同一实现满足，且没有消费者依赖被放弃的一侧"）本轮未被认定成立；因此六族一律 DISTINGUISH。若消费者分属不同语义（JS既有`.length`语义 vs 文件偏移字节），DISTINGUISH 是**唯一正确**处置。
- 任何改变用户可观察行为的处置都必须有用户裁定记录，不能以"整理代码"名义静默改语义。

## 3. 技术栈

Rust 实现面（`crates/auto-lang-core/src/vm/{native,ffi/stdlib}`）、`auto_macros::rust_fn` 注册机制与 `production()` 绑定序、Auto 最小程序（VM 侧行为对照）、nextest（定向 + 867 三仓完整档）、`plan755` HTTP 契约测试、`docs/specs` 规范页。

## 4. 需求分析与背景调查

本次修订授权（2026-10-28）：用户明确要求按本会话核查建议修订867/869/872/873及总路线，只改主检出计划文档；不执行产品/验证设施改动、不发布canonical/ledger、不建树/agent、不分配874/875或其他新号、不推送。已给的A/B、2,000行批粒度、六族DISTINGUISH裁定继续有效；具体结构性减法批仍沿原签核规则。

### 4.1 授权与轮次

- 用户 2026-10-28 要求把"867 之后的后续计划"写成文档；本计划是其中一份（语义双轨）。
- 本轮只写合同：不建树、不改实现、不跑档、不改 canonical Specs。
- **实施前不需要再裁定 MERGE**：用户 2026-10-28 已裁定六族全部 DISTINGUISH；若实施中发现某侧完全无消费者，才需就 RETIRE 单独签核。

### 4.2 已观察事实（2026-10-28）

| # | 事实 | 证据 |
|---|---|---|
| F-1 | 844 冻结 6 族为"语义不同/超出减法授权"：`str.len` 字符/字节、`math` 双轨、`to_int`/`parse_int`、`time_now`、error-reply ×10、`nv_to_value` | [844 归档 §2 保留表/§6 未知决策 4](archive/844-cross-repository-dedup-and-net-slimming.md) |
| F-2 | 同名 builtin 的生效实现由绑定序决定：catalog → inventory → nv 覆盖；纯版可能恒被覆盖 | 844 D-4 证据（`strings_json.rs` 纯版 3 函数删除 + merge-gate 显式名字注册补回，提交 `5d03c980b`） |
| F-3 | **名字/文档/实现三方矛盾**：`Str.len` 纯 shim 文档写 "length in bytes"、实现 `s.len()`（字节）；同文件 `Str.byte_len`（PLAN-781）注释称 `str.len()` 是 PLAN-055 字符数语义 | `crates/auto-lang-core/src/vm/ffi/stdlib/strings_json.rs:5-8`、`:26-32` |
| F-4 | `Math.*` 28 个注册在 `ffi/stdlib/math.rs`；nv 轨道另有同名覆盖条目，错误语义不同（0 兜底 vs Err） | `ffi/stdlib/math.rs`；844 §2（`native.rs:1713-1747` 对 `math.rs:5-167`，行号为 844 T-01 时点，T-01 需复核） |
| F-5 | `Time.now/now_ms/now_sec/sleep_ms` 注册在 `ffi/stdlib/sys_env.rs:99-123`；nv 侧另有实现，失败处理不同（`unwrap_or(0)` vs `unwrap`） | `sys_env.rs`；844 §2（`native.rs:1834-1863`） |
| F-6 | `nv_to_value` 双版签名不同（`native.rs:2248` vs `interop.rs:128`），跨 FFI 边界 | 844 §2 |
| F-7 | error-reply 采样族 10 处（`server_dispatch.rs` ×5、`server_argbind.rs` ×2 段、`server_scope.rs` ×2），同构非逐字，body 受 `plan755` HTTP 契约钉住 | 844 §2；`plan755_http_contract_tests` |
| F-8 | 867（进行中）将交付三仓完整档 + ID 闭合 + 档位下限守卫；869 已声明这 6 族为其非目标 | [867](867-split-equivalence-test-closure.md)、[869](869-crate-inventory-and-subtraction-batch-2.md) |

### 4.3 规范依据

- [stdlib/project.md](../specs/stdlib/project.md)：内建名字与语义的现行登记面（本计划主要 SD 落点）。
- [auto-lang/vm/overview.md](../specs/auto-lang/vm/overview.md)：VM 内建注册与绑定序。
- 844 收据与 [net-slimming.md](../specs/repository-boundaries/net-slimming.md)：保留有证清单（本计划不得越权动）。
- `plan755` HTTP 契约测试：error-reply body 的既有钉定。

### 4.4 假设与不声称

- **A1**：绑定表可从属性注册 + 覆盖块静态导出，并由最小程序实测覆核；若存在动态注册路径（宏/脚本生成），T-01 必须补查并在表内标注。
- **A2**：行号引用（F-4/F-5/F-6）来自 844 T-01 时点，经过 844 的删除后可能漂移，T-01 以文件+名字为准重新定位。
- **不声称**：本轮未读完全部 6 族实现细节、未做实测对照；六族处置已裁定为 DISTINGUISH，因此不需要再判"是否该 MERGE"。F-3 的三方矛盾是**已读到的具体事实**，但"谁在运行时生效"仍需 T-01 实测。

## 5. 详细设计

### 5.1 builtin 绑定表（T-01，关键产物）

- 范围名字集（初值，T-01 以实际绑定结果补全）：`Str.len`、`Str.byte_len`、`Str.to_int`、`Str.parse_int`、`Math.*`（28+）、`Time.now`/`now_ms`/`now_sec`、`nv_to_value` 的两个调用面、error-reply 相关 `ApiReply::Full` 构造点。
- 每行字段：`name | 注册面（catalog/inventory/nv/动态）| 直接实现（file:line）| 生效实现 | 语义（字符/字节/Err/0 兜底/unwrap…）| 消费者（VM 程序/a2r 生成/vue 生成/Rust merged/内部）| 钉住测试 | 备注`。
- 覆核方式：每族 ≥1 个最小 Auto 程序，实际观测（例：CJK 字符串取 `Str.len` 与 `Str.byte_len`；`Math.sqrt(-1)` 的返回/错误；`Time.now` 在不可用环境下的行为——若不可构造则显式记 pending）。
- 产物：`docs/plans/attachments/872/binding-table.md`（md 摘要 + 逐族证据）+ 机器可读 `scripts/builtin-binding-table.json`（含生成命令与注册面指纹）。

### 5.2 逐族裁定（T-02）

| 族 | 现状（844 判定） | 处置（已裁定 2026-10-28） | 落地要点 |
|---|---|---|---|
| `str.len` / `str.byte_len` | 两个**活名字**（字符 vs 字节） | **DISTINGUISH** | JS `.length` 语义与 PLAN-055/781 消费者各自保留；`str.len` 文档改成与实际生效一致 |
| `math` 20+ 族 | 机制性双轨（tag 感知 pop vs Err） | **DISTINGUISH** | 两侧错误契约各自文档化并加区分性测试；合并留待未来 revision |
| `Str.to_int` / `parse_int` | 双版 | **DISTINGUISH** | 解析失败语义分别写明 |
| `Time.now` 等 | 失败处理不同（0 vs unwrap） | **DISTINGUISH** | 不可用环境下的行为写清；不改失败可见性 |
| error-reply ×10 | 同构样板，非逐字；body 被契约钉住 | **DISTINGUISH**（可选抽 helper，仅当 body 字节不变且净删>0） | `plan755` 契约、净收益 |
| `nv_to_value` | 双签名跨 FFI | **DISTINGUISH** | 跨边界签名差异保留并注明 |

**已裁定：六族全部 DISTINGUISH**（2026-10-28）。原判据（"只有两侧能被同一实现满足且无消费者依赖被放弃一侧才 MERGE"）本轮未被认定成立，因此 MERGE 不在实施面；每族的处置是"补名字（若缺）+ 改文档 + 补区分性测试"，不改变运行时行为。

### 5.3 实施与兼容（T-03/T-04）

- **冻结既有行为**：现有名字、大小写/别名、catalog→inventory→native覆盖顺序及最终生效实现、参数/返回/错误、生成调用目标保持不变。记录覆盖关系并使其可审，不把“不得静默覆盖”实现为禁用既有覆盖或改优先级。
- 新名字只可作为不碰撞的显式新增出口（确有必要时T-02冻结）；既有消费者不得强制改名、旧名字不得消失、已存在未知名字的失败行为影响须具名评估。若无法满足零既有行为变更则needs_replan，不能以DISTINGUISH自动授权替换。新增出口新增行为单列验收，不伪称它在旧版本已有。
- 运行时间/非确定性数据不得直接按两次当前字节相等判等；T-02冻结可重放输入或同一时钟/错误分支等价观测，保持既有返回类型/精度/错误语义。无法构造的分支列具名限制，不伪造不可用时钟实测通过。


- MERGE：**本轮未授权，不实施**（保留此条仅作未来 revision 的接口说明）。
- DISTINGUISH：补名字（若缺）、补语义文档、补区分性测试（两侧各 ≥1 例，含边界：CJK、空串、负数、Unicode 数字、不可用时钟）。
- error-reply：若抽 helper，必须证明 10 处 body 字节级不变（`plan755`定向契约绿，原必需正控），否则不做。

### 5.4 执行合同（work/review/merge 共用）

| 字段 | 内容 |
|---|---|
| 规则与边界 | G1–G5；六族 DISTINGUISH、**零行为变更**；不改 844 保留项；MERGE 未授权 |
| 实际路径 | 属性注册 + 覆盖块 + `production()` → 绑定表 → 最小程序实测 → 逐族裁定 → 实现/文档/测试 → plan755 + 867 门 |
| 所有权与状态 | 绑定表为验收输入（改需原因+哈希）；名字 owner=语言仓；a2r/vue 生成器调用面随名字变更同步 |
| 区分性用例 | V-01..V-06：绑定表复算、实测覆核、**行为零变更对照**、DISTINGUISH 文档一致、契约 body 不变、867 门复跑 |
| Oracle 与关闭 | 每 AC 绑命令与期望输出（含两侧程序输出的字节对照）；关闭需原始日志 + 哈希 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/stdlib/project.md | 内建名字语义从"分散注释"→ 单一 `name → 语义 → 生效实现` 矩阵，含 `str.len`/`str.byte_len`、`math` 错误契约、`time` 失败行为 | 现有三方矛盾（F-3） | AC-02, AC-03 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | 显式登记catalog→inventory→native的既有覆盖顺序、最终生效实现及消费者；禁止未记载漂移，不禁止既有覆盖、不改注册优先级 | 消除静默差异而非改变机制 | AC-01,AC-05 |

## 6. 测试设计

`owner_stage` 均为 `work`；本轮未执行。

| Case ID | AC / invariant | 输入或 setup | 真实入口 | 可观察期望 | 断言/证据 | 覆盖边界 |
|---|---|---|---|---|---|---|
| V-01 | AC-01 绑定表可复算 | 冻结提交 + 同工具链 | 表生成脚本两次运行 | 逐字节一致；与注册面指纹一致 | 表 JSON + sha256 | 6 族名字集 |
| V-02 | AC-01 实测覆核 | 最小 Auto 程序（CJK 取 len/byte_len、`Math.sqrt(-1)`、不可用时钟若有构造） | `auto`/VM 实际执行 | 生效实现与表一致；不一致即红 | 程序输出 + 表 | 可构造的名字；不可构造显式 pending |
| V-03 | AC-02 行为零变更 | 冻结同程序/输入与旧名绑定，含新旧名字碰撞负控；时钟可重放输入 | VM+a2r/Vue既有消费及plan755 | 旧名、生效实现、输出/错误/调用目标不变；改优先级/强制改名必须红；新增无碰撞出口单列 | 绑定指纹/前后对照 | 六族；非确定分支按冻结观测，不声称随机字节相同 |
| V-04 | AC-03 DISTINGUISH 一致性 | 名字/文档/测试 | 规范页 + 代码注释 + 测试 | 三者对同一语义一致；`str.len` 文档必须与实际生效语义相同 | 文本对照 + 定点测试 | DISTINGUISH 族 |
| V-05 | AC-04 契约不破 | error-reply 处置 | `plan755_http_contract_tests` | body字节不变，plan755定向契约绿，不以整档限制放宽正控 | 测试输出 | HTTP 应答面 |
| V-06 | AC-05 门禁联动 | 冻结三仓与867已认证总账 | work定向账本/消费者；merge实际完整档 | 无LOST/未解释差额，旧红有证且不扩大、新红拒绝；未执行不计pass | 定向记录/merge收据 | 867约定支持面，不追加豁免 |

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | builtin 绑定表覆盖 6 族全部名字，可复算 + 实测覆核；不可构造项显式 pending |
| AC-02 | work | 六族DISTINGUISH，旧名/绑定顺序/生效实现/错误和生成调用不变；必要新增无碰撞出口单列；V-03反控可区分优先级或旧名漂移；MERGE不实施、RETIRE另签核 |
| AC-03 | work | 处置落地后名字/文档/规范/测试一致；`str.len` 三方矛盾消除 |
| AC-04 | work | plan755定向HTTP契约须绿（原必需正控，不借867既存红口径放宽）；JS既有.length语义/生成器调用目标保持，不以“字符数”泛称替换后端既有计数单位 |
| AC-05 | work | 867义务总账及定向消费者验证通过，测试处置有据；完整档配置与红集比较就绪，merge pending，不在work跑整档 |
| AC-06 | review | 独立终审绑定冻结 commit/依赖/输入，核 V-01..V-06、SD-01/SD-02、遗漏与健康面；merge 门 pending |
| AC-07 | merge | final review后冻结实际触面档+三仓完整档，同输入同选面不重复；按867认证既存红/零新增红/必需正控未跑阻塞口径，SD/ledger/归档/guard闭环才U872 |

## 8. 执行步骤

10 项 = 8 work + 1 review + 1 merge；G872=U867∧U869。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G872 | builtin 绑定表（静态导出 + 最小程序实测覆核）；确认动态注册路径无遗漏 | AC-01 [✅ 2026-10-29] 绑定表冻结：attachments/872/binding-table.md（六族全行+实测）+ scripts/builtin-binding-table.json（879 条指纹、唯一同名双 ID str.len:[170,1500]）+ 复算守卫测试（fresh-registry peek 视角，并发/串行均逐字节稳定）；动态注册路径核查=generated_std.rs（类型/方法派发，零族名）+动态段(≥10000)零族名；探针=面自省+行为+字节码三面（plan872_builtin_binding_probes.rs，13 绿） |
| [x] | T-02 | work | G872；T-01 | 六族 DISTINGUISH 落地材料（消费者/承诺/语义差异/文档口径/是否需补名字）；确认无"完全无消费者的单侧"（否则提 RETIRE 并单独签核） | AC-02 [✅ 2026-10-29] 六族 DISTINGUISH 材料并入绑定表 §1-§7：双侧活消费者=族一（.length 字符侧 engine/JS vs len 字节侧语料/偏移）、族五（各 server 路径）、族六（collections vs interop）无 RETIRE；单侧遮蔽=math nv 宏 20 项/纯 parse_int/nv time（RETIRE 候选 3 项列 §7 待签核，未实施）；无需补名字（Str.to_int 的 CALL_SPEC 缺名是既有失败行为，具名登记不补） |
| [x] | T-03 | work | G872；T-02 | 按 DISTINGUISH 实施：补名字/改文档/保持两侧实现，**零行为变更** | AC-02, AC-03 [✅ 2026-10-29] 零行为变更实施：10 处注释/7 文件（Str.len 字节语义+区分注、byte_len 与 stdlib.rs PLAN-055 误指订正、to_int_nv Result/CREATE_OK 误述订正、math/time 生效轨+遮蔽轨注明、nv_to_value 双签名互指、native.rs 过时手工 math 注册注释订正）；diff 证据=08ea8b35d..65943e083+dc10ea723 中 auto-lang-core 零非注释实现行 |
| [x] | T-04 | work | G872；T-03 | 测试：区分性/边界例 + **行为零变更对照** + `plan755` 契约；更新受影响生成器调用面（仅文档/名字，不改行为） | AC-03, AC-04 [✅ 2026-10-29] plan872_builtin_semantics_tests.rs 5 测：字符/字节区分（中文/a中b/空串）、绑定槽位 pin（170/1500/1526，改名/改优先级即红=V-03 反控）、Math 生效语义（NaN-on-int vs 0.841…、half-away、域外值）、to_int 0 兜底（含全角数字）、Time 三件套；plan755 定向 2 绿（V-05，server 面零触 diff） |
| [x] | T-05 | work | G872；T-04 | 行为对照收据（两侧程序输出字节）+ 绑定表更新 | AC-01, AC-03 [✅ 2026-10-29] 行为对照收据 attachments/872/behavior-receipts.md（两侧输出字节转录+零行为变更对照论证：探针先采、断言后冻同值）；绑定表随 String.len 实测（resolve 171=字节/peek 170 分歧）与 math 遮蔽面订正更新 |
| [x] | T-06 | work | G872；T-05 | 867账本/红集接口与相关定向消费者核对，完整档标merge pending；T-10统一执行 | AC-05 [✅ 2026-10-29] 867 接口核对：红集基线零新增红（新测试 ID plan872_* 全绿）、金样台账输入集（test/vm、a2c、a2r、packages、book）零触面、迁移账本零改名；定向消费者 plan781 10 绿（byte_len+a2r 镜像）/plan755 2 绿；完整档=merge pending（裸 t+tv 触面；tb/tu/taa/docs_gen 不适用面记录在案） |
| [x] | T-07 | work | G872；T-06 | SD-01/02 proposal及handoff冻结送审，work不发布canonical | AC-02,AC-03 [✅ 2026-10-29] SD-01/SD-02 提案冻结 attachments/872/sd-proposals.md（stdlib/project.md 语义矩阵 + vm/overview.md 绑定序登记），work 未发布 canonical |
| [x] | T-08 | work | G872；T-07 | 全 AC/V 自检与遗漏扫描；未闭项列 finding/pending | AC-01..AC-05 [✅ 2026-10-29] AC-01..05 自检全过（收据见 §9）；遗漏扫描产出 2 处订正（native.rs 过时注释、绑定表 math 遮蔽面）已闭合；无未闭 finding |
| [x] | T-09 | review | G872；T-08 冻结提交 | [✅ 2026-10-29] 独立终审（子代理独立上下文，同树只读）：AC-01..05 全 pass；零行为变更结构性证明（auto-lang-core diff 非注释行=空）；V-01 双跑复算+V-02 源码抽验+V-03 槽位 pin 复核；finding F-872-R1-01（探针注释自矛盾，LOW）当轮修复 dfe3a5b93 并重绑 PASS；F-872-R1-02（附件行号漂移，INFO）no-action；merge 门（裸 t+tv）pending | AC-06 |
| [x] | T-10 | merge | G872；T-09 pass | [✅ 2026-10-29] 门禁=裸 t 3197/3198+ffi_dual_018 环境flake单跑绿（867 基线在案族；NEXTEST_JOBS=3 偏差=外部 93% CPU 负载，选择面不变）+ tv 163/163；tb/tu/tt/taa/docs_gen 不适用（零触面）；SD-01/02 落盘 af6b3f2bf（ff-only，rebase 全等 5/5）；ledger v124（5 条，受控离线投影+回读校验）；归档+清理收口 | AC-07 |

**实施树**：`D:/autostack/.wt/lang-872/{auto-lang,auto-ui,auto-man}`（仅需写被裁定的仓），分支 `plan-872-dev`；禁止 junction/symlink；清理前 `wt-guard.sh` clean。
**门禁**：work/review仅必要check、定向反例/模块/真实消费者；final review后merge冻结串行实际完整档与适用触面，覆盖相同选面可复用。语言空tu退役后由UI仓tu承接；tf批量主检出单实例。

## 9. 复审记录



本轮合同预检（非实现复审/测试）：章节0–10、单一任务表、任务DAG无环、编号唯一、AC/V/SD引用及frontmatter依赖检查通过；本Plan 10任务/7 AC/6 V/2 SD。来源内容hash：`docs/specs/stdlib/project.md` sha256 `4feb124789ee25e23b57c97a5769c1401895de8233268cdde3716204668b517b`；`docs/specs/auto-lang/vm/overview.md` sha256 `e38aed5464f73f2401234d68a01f481245eafe49acc2280beb84265ac6108ed4`。仅文档校验，未运行Cargo/docs_gen。

2026-10-28 / stage: new / plan_id: PLAN-872 / plan_revision: 3 / outcome: blocked（合同就绪，等U867且U869） / authorization: 用户本会话修订授权 / input_sha256: `b96ba3ba8cbf570d14edf5cb5492d924b150f8ffd7f70a220c32fcab2eb70588` / changed: depends_on补869、§5.3、SD-02、V-03/06、AC-02/05/07、T-06/07及整档后置 / progress: drafting 0/10保持 / next: 完整前置齐后原T-01；六族DISTINGUISH裁定不再询问。后续未编号语言结构包须等U872，避免同时改native/stdlib/HTTP。本轮无实现/测试/建树/发布canonical/ledger/推送；原始证据留本地、md摘要记录路径+hash。旧“不得覆盖”不得解释为取消现有覆盖机制。

2026-10-28 / stage: new / plan_id: PLAN-872 / revision: 1 / outcome: **pass（合同可执行；实施等 U867∧U869 + 用户语义裁定）** / next: 门成立后 work T-01，裁定在 T-02 交用户点选 / changed: 新建 T-01..T-10、AC-01..AC-07、V-01..V-06、SD-01..SD-02。

- 取号：`scripts/new-plan.sh builtin-semantics-dual-track-ruling`（872 → 873）。
- 本轮只写合同：未做实测对照、未定任何族的 MERGE、未建树、未改实现/规范/ledger。
- 已发现并在计划内登记 F-3（`Str.len` 名字/文档/实现三方矛盾），但不预判谁生效。

2026-10-28 / stage: new（r2 语义修订） / plan_id: PLAN-872 / plan_revision: 2 / outcome: **pass（六族处置已裁定为 DISTINGUISH，仍等 U867∧U869）** / ruling: 用户裁定 **六族全部 DISTINGUISH、零行为变更**；MERGE 未授权（D-1 关闭）、兼容口径问题随之不适用（D-2 关闭）/ changed: §1 G2、§2 裁定项、§5.2 默认、§5.3 MERGE 分支、AC-02、T-02/T-03、V-03（改为"行为零变更"对照）、§10 D-1/D-2 / next: 门成立后 work T-01；若未来要合并 `math` 等，另开 revision 并取得裁定。


2026-10-29 / stage: work / plan_id: PLAN-872 / plan_revision: 3 / outcome: **pass（execution_done 8/10；T-09 review / T-10 merge 待续）** / worktree_path: D:/autostack/.wt/lang-872/auto-lang / branch: plan-872-dev / base_commit: 08ea8b35d / code_commit: 65943e083 + d57eaf23e + dc10ea723 / dependency_revisions: auto-ui@154a396（detached 只读）、auto-man@e6cc06f（detached 只读）、auto-down@c3464d0（组内 detached） / task_ids: T-01..T-08 / evidence: [绑定表](attachments/872/binding-table.md)、[行为收据](attachments/872/behavior-receipts.md)、[SD 提案](attachments/872/sd-proposals.md)、scripts/builtin-binding-table.json（复算守卫）；定向绿=plan872 13/plan755 2/plan781 10；cargo check -p auto-lang 0 错（警告预存非本面） / findings: ①生效实现与 844 表述反转——Math/Time 生效=纯 shim（inventory 同 ID 后写覆盖 catalog），nv tag 感知/unwrap_or(0) 版被遮蔽；②str.len 语义按派发路径分裂（.length 属性 nat#170=字符 / .len() nat#1500=字节 / CALL_SPEC 内联臂=字符）；③Str.to_int 双名同挂 1516 生效=nv 0 兜底裸 i32（纯 Result 版遮蔽）；④RETIRE 候选 3 项（math nv 宏 20/纯 parse_int/nv time）待单独签核未实施；⑤native.rs 手工 math 注册注释过时已订正 / blockers: 无 / pending_closeout_gates: merge-owned 裸 cargo t + cargo tv（VM/编译器触面档）于冻结提交统一执行；tb/tu/taa/docs_gen 不适用（book/ui_gen/aavm/文档生成面零触 diff）；tf 批量回归按到期规则另判 / next_responsible_role: review T-09（冻结提交 dc10ea723 上独立复审：绑定表重算、行为零变更对照重放、plan755 契约、文档一致性）

2026-10-29 / stage: goal | plan_id: PLAN-872 | plan_revision: 3 | round: 1 | from: new(r3 blocked) | to: work | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-872/auto-lang | commit: dc10ea723 | next: review


2026-10-29 / stage: review / review_scope: final / reviewer: 独立子代理上下文（同 plan-872-dev 树只读复审+定向重跑；非第二检出；终审与修复重绑两轮）/ plan_id: PLAN-872 / plan_revision: 3 / outcome: **pass（reviewed；重绑至 dfe3a5b93）** / worktree_path: D:/autostack/.wt/lang-872/auto-lang / branch: plan-872-dev / reviewed_commit: dfe3a5b9369eba7b0d95e0b36e84dd9e46ce1a27（初审 dc10ea723 + F-872-R1-01 注释修复 dfe3a5b93）/ base_commit: 08ea8b35d / dependency_revisions: auto-ui@154a396、auto-man@e6cc06f、auto-down@c3464d0（detached 只读） / spec_inputs: stdlib/project.md sha256 4feb1247…、vm/overview.md sha256 e38aed54…（r3 合同预检指纹，未变） / acceptance_results: AC-01 pass（绑定表六族全覆盖+JSON 879 指纹+复算双跑）；AC-02 pass（diff 非注释实现行=空、零删除、零新注册、RETIRE 未实施）；AC-03 pass（F-3 三方矛盾消除，源/表/测试一致）；AC-04 pass（plan755 2 绿、server 零触面）；AC-05 pass（work 口径：867 红集零新增、金样输入集零触面、整档标 pending）/ pending_closeout_gates: merge-owned 裸 cargo t + cargo tv（冻结 dfe3a5b93 上）；tb/tu/taa/docs_gen 不适用面在案；tf 批量按到期另判 / findings: F-872-R1-01（LOW，探针注释自矛盾）已修复重绑；F-872-R1-02（INFO，附件行号漂移）no-action / evidence: 复审者独立重跑 plan872 13 绿/plan755 2 绿/plan781 10 绿/JSON 守卫双跑；零行为变更 grep 证明；五处源码抽验（shim_str_len/shim_str_to_int_nv/production() 序/strings_json 字节语义/过时注释订正）均与绑定表一致 / next: merge T-10（冻结 dfe3a5b93：先跑裸 cargo t + cargo tv 门禁，再 SD/ledger 落盘、归档、guard+清理）

2026-10-29 / stage: goal | plan_id: PLAN-872 | plan_revision: 3 | round: 2 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-872/auto-lang | commit: dfe3a5b93 | next: merge


2026-10-29 / stage: merge / plan_id: PLAN-872 / plan_revision: 3 / outcome: **pass（delivered；completion_kind: delivered）** / delivery_commit: af6b3f2bf690d31bb046fa749373dfdeafbcd24f（master 尖，ff-only） / reviewed_commit: 368b886f2（=dfe3a5b93 重绑，rebase 全等 5/5 range-diff `=`；映射 65943e083→f976fe414 / d57eaf23e→5836a2083 / dc10ea723→8559dd9fa / dfe3a5b93→368b886f2 / efcce9880→af6b3f2bf） / checkpoints: prepared=SD-01/02 落盘 worktree efcce9880（docs-only 后裔合规升级为 delivery）；landed=master==af6b3f2bf ff-only 无合并提交；ledger_refreshed=.autoos/specs.json v124（regime B 受控离线投影：字节备份 bak-p872-merge→脚本改写仅 5 条插入→schema/回读/范围守卫全过；designs P872-D1/D2 + tests P872-T1 + reviews P872-R1 + reports P872-M1，全带 file+tags）；archived=docs/plans/archive/872-…md status:archived + 本收据；cleaned=待清理段回填 / gates: 裸 cargo t=3197/3198 PASS+1 TIMEOUT（ffi_dual_018_dep_fields，867 红基线在案负载 flake 族（red-baseline.md:71），单跑 2.07s 绿；执行偏差 NEXTEST_JOBS=3=外部 93% CPU 交互负载，选择面不变）；cargo tv=163/163；tb/tu/tt/taa/docs_gen=不适用（book/ui_gen/trans/aavm/文档生成面零触 diff）；首次两跑全并行负载超时（ac_av_probe/ace_m1 族）经定向复跑全绿归因为环境，未据此 waived / landing_artifacts: 零行为变更（注释级+测试+工件），release binary/daemon/vue bundle 语义无涉——显式观察项记录，不重建 / batch_regression_due: 872%5≠0；按 48h 规则另查 .last-batch-regression.json / next: done（U872 成立）

2026-10-29 / stage: goal | plan_id: PLAN-872 | plan_revision: 3 | round: 3 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-872/auto-lang | commit: af6b3f2bf | next: done

## 10. 待澄清事项

| ID | 事项 | 影响 | 处置 |
|---|---|---|---|
| D-1 | ~~六族里哪些允许 MERGE~~ **已裁定（2026-10-28）**：全部 DISTINGUISH，MERGE 未授权 | T-03 实施面 | 关闭：按 §5.3 DISTINGUISH 路径实施；未来要合并另开 revision |
| D-2 | ~~MERGE 的兼容口径~~ **本轮不适用**（无 MERGE） | 用户程序兼容 | 关闭；若未来授权 MERGE 再定 |
| D-3 | error-reply 是否值得抽 helper（收益/风险） | 净删与契约风险 | T-03 有界判定：仅当 body 字节不变且净删>0 |
| D-4 | 不可构造的实测项（如不可用时钟）如何记 pending | AC-01 完备性 | 显式 pending + 环境说明，不计 pass |
