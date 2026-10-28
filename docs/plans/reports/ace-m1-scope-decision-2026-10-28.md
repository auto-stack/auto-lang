# ACE-M1 对等口径决策件：单元格状态词表、`na:m1` 分类与差异裁定（2026-10-28）

> **性质**：**已裁定口径**（2026-10-28 用户问卷 4/4 采纳推荐项）+ 逐行分类。**只动文档**：不建实施树、不改 `binding.tsv` 现状、不改 852 计划（已归档，终态不回开）、不改任何代码。
> 目的：让"M1 全绿"成为一个**可计数、可达**的目标，而不是一个永远收不了口的口号。
> 关联：[852 底座合同](ace-852-baseline-and-capability-validation-2026-10-26.md)、[ACE-M1 语料 Spec](../../specs/auto-test-case/ace-m1-conformance.md)、[Design43 §12](../../design/43-ac-av-typed-mir-and-abc.md)、[Design49](../../design/49-native-single-mir-lowering.md)、[批次规划](ace-next-batch-2026-10-28.md)。

## 0. 摘要

当前 M1 分母（`crates/auto-lang/test/cases/ace-m1/binding.tsv`，103 行 × 4 路径 = 412 单元格）**没有"按合同不适用"这一态**，导致三类互不相同的情况被混记：

1. **真缺口**（该路径应当支持、尚未交付）；
2. **按 profile 决定不支持**（旧引擎没有 typed 入口、没有 Dyn API；native 侧没有闭包 ABI）——被记成 `gap`，于是"全绿"永不可达；
3. **a2r 的两条不同轴**（快照逐字节金样 vs 可编译认证）被塞进同一格——而 852 Spec 明说"行为核验见**独立认证规范**"。

本决策提案：**三轴分离 + 五态词表 + 6 项差异裁定 + 3 类 `na:m1`**，并给出 103 行的逐行重分类（附录 A）。

按此（已裁定）口径测算：**50/103 行已"干净"**（ok / ok+cert-gap / na / accepted diff），**53 行有明确 owner、无待裁定项**——第一次给出了可收敛的账。

## 1. 为什么现在做

- **不占任何人的写面**：868 在实施、其他计划在草案；本件只读写文档，可与任何工作并行。
- **越晚做，账越烂**：864（L/V 源码）、870（mn record/text）落地后，`gap:mn-scalar-only` / `wait:av-*-822` 会成批变成 `ok`，但如果词表不先定，"哪些行必须 ok、哪些永不需要"仍然说不清，下一批计划又要各自猜。
- **它是 852 重绑定的前置**：重绑定本身是证据工作（跑 probe、翻 token），没有口径就只能"按感觉翻"。

## 2. 现状量化（412 单元格）

| 现 token 形态 | 计数 | 问题 |
|---|---|---|
| `ok*` | 136 | 无 |
| `gap:<owner>` | 102 | 混装"真缺口"（`mn-scalar-only`、`av-unsupported-list`…）与"按 profile 不适用"（`ac-native-loader`、`avm/a2r-no-typed-entry`、`avm/a2r-no-dyn-api`） |
| `n/a` | 48 | 全在 scenario 行；虽表达"不适用"，但无规则 ID、无重新进入条件 |
| `snapshot-only-cert-gap` / `emit-ok-run-unverified` / `cert-diff` | 39 | a2r 两轴混记（快照绿 + 认证缺），被当成 M1 阻塞 |
| `diff-*` | 36 | 未区分"已裁定可接受"与"待裁定" |
| `wait:<plan>` | 27 | 指向的 822 未获授权新增 T-11，wait 从未关闭也从未重绑 |
| `probe` | 24 | 未执行，长期滞留 |

合计 136+102+48+39+36+27+24 = **412** 单元格，逐项与 `binding.tsv` 可复核。

## 3. 决策一：单元格状态词表（五态 + 两个限定）

| 状态 | 含义 | 是否阻塞 M1 | 约束 |
|---|---|---|---|
| `ok` | 该路径真实执行并通过冻结 oracle（含 `ok-emit`、`ok-parse-reject`、`ok+...`） | 否 | — |
| `ok+cert-gap` | **a2r 专用**：快照/金样轴绿，认证轴缺（伪 Rust 不可编译等） | 否（M1 的 a2r 判据=金样轴） | 认证缺口另记 `gap:a2r-*`，不得因此把 emit 行标 gap |
| `gap:<owner>` | 该路径**应当**支持且未交付 | **是** | owner 必须是计划 ID 或具名能力包；无 owner 不得使用 |
| `na:m1(<rule>)` | M1 **不要求**该路径判该能力，引用规则 ID（NA-1..NA-3） | 否 | 必须写重新进入条件；只能由本决策或同等决策新增 |
| `diff:accepted(<rule>)` | 已裁定为可接受差异（旧引擎缺陷/profile 差异），保留记录但不阻塞 | 否 | 必须引用裁定 ID（R-A..R-F）；`diff:accepted` **不等于** `ok` |
| `diff:pending` | 执行但语义分歧，**未裁定** | **是** | 到期必须裁定，不得长期滞留 |
| `probe` | 契约已备但未真实执行 | **是** | 列为重绑定任务；不得作为长期状态 |

关键区分：**`na` 不是"难"**，而是"合同决定不需要"；**`diff:accepted` 不是"绿"**，而是"已裁定为旧侧问题"。

## 4. 决策二：a2r 路径拆两轴

852 Spec 原文：a2r 的判决面是"**真实产物逐字节比较冻结金样**；行为核验见**独立认证规范**"，且认证器 `ace-certify` 是独立 bin、任何日常 scope 不隐式调用（G08）。因此：

| 轴 | M1 是否要求 | 现记法 | 新记法 |
|---|---|---|---|
| a2r-emit（金样/快照逐字节） | **是** | `ok-emit` / `snapshot-only-cert-gap` | `ok` |
| a2r-cert（可编译/可运行认证） | **否**（独立认证轴） | 与 emit 混在同一格 | `ok+cert-gap` + `gap:a2r-composite-cert`（独立登记） |

影响：39 个 a2r 单元格从"阻塞 M1"变为"金样已绿 + 认证缺口另账"，这是本次测算里最大的一处账目澄清。**注意**：这不是放宽——认证缺口仍在，只是不被算进 M1 的 a2r 判据；它阻塞的是"a2r 复合类型可编译认证"这条独立线（`docs/plans/242-a2r-feature-gap-tracker.md` 跟踪）。

## 5. 决策三：`na:m1` 三分类与反滥用规则

| 规则 | 适用 | 依据 | 重新进入条件 |
|---|---|---|---|
| **NA-1** | legacy `avm` 与 `a2r` 不判 **typed fixture 入口**（W01–W05） | typed fixture 不是源程序；legacy/a2r 的判据是源程序执行/转译 | 若要求 legacy/a2r 承接 typed 入口，先立能力计划 |
| **NA-2** | legacy `avm` 与 `a2r` 无 **Dyn API**（D01–D09） | 824 的 Dyn 是新 profile 的 typed 能力，legacy 从未提供 | 若保留消费者依赖 legacy Dyn，先裁 SEM-07 |
| **NA-3** | native（mn）不判 **闭包 / Dyn / 在线热重载 / 暂停恢复**（K*、D*、I08、I07 的 ac 列） | 无冻结的 native 闭包/Dyn ABI（823/824 显式排除 native ABI）；Design49 §7 明确排除；I08 需 native loader 能力包 | 相应能力专项计划冻结 ABI/loader 后，逐行改回 `gap:<plan>` |

**反滥用（必须同时生效）**：

1. 每条 `na:m1` 必须带规则 ID；无规则 ID 的 `na` 视为 `diff:pending`。
2. `na` 的**数量只能通过决策减少**：任何计划不得自行新增 `na` 行；新增必须回本决策件或同等决策。
3. 每个 merge 的 852 acceptance 报告必须单列 `na` 行数与规则分布；`na` 增长而无决策记录 = 门失败。
4. `na` 行的 `diff-*`/probe 证据保留原样，不得删除。

## 6. 决策四：差异裁定 R-A..R-F

| 裁定 | 涉及行 | 现状 | 推荐裁定 | 若不采纳的后果 |
|---|---|---|---|---|
| **R-A** 旧引擎遮蔽/命名实参绑定差异 | B08、B11 | avm 算出 10/-7，冻结 oracle 7 | **可接受（旧引擎缺陷，SEM-08 类）** | 这 2 行永久阻塞 M1 |
| **R-B** 旧引擎整数语义（wrapping vs checked trap） | B13、B15 | avm `diff-old-int-i64` | **可接受（SEM-01 profile 差异）**，新 profile 保持 trap | 同上；或被迫统一语义（需改已批准 profile） |
| **R-C** 旧引擎接受非法程序（无类型/无检查） | B17、B18、B19、T06、R09、R10、L06 | `diff-legacy-accepts` | **可接受（旧引擎限制）**，新 profile 的拒绝是规范 | 9 行永久阻塞 |
| **R-D** 旧引擎运行时/值缺陷 | T01–T04、R01、R03、R04、V03、K02、K05 | `diff-legacy-*` | **可接受（旧引擎缺陷，SEM-08）** | 13 行永久阻塞 |
| **R-E** a2r 不做名解析 | B16、E03 | `diff-no-name-check` | **不接受**：记为 `gap:a2r-resolve`（a2r 真缺口） | 若接受，等于放弃"未执行分支也查名"的语义门 |
| **R-F** a2r 认证差异 | B13/B14/B15/B20 等 | `cert-diff` | **认证轴差异**：`ok+cert-gap`，随 a2r 复合认证线处理 | 阻塞 a2r 认证线（不阻塞 M1） |

裁定原则：**旧引擎的行为不自动成为新规范**（能力矩阵 §3 SEM-08 已定），但旧侧的差异必须**具名、可追溯**，不能"标绿了事"。`diff:accepted` 保留原 token 作为证据，不允许从表里消失。

**裁定结果（2026-10-28 用户问卷，4/4 采纳推荐项）**：R-A / R-B / R-C / R-D 按推荐通过（旧侧已知差异，`diff:accepted`，不再阻塞 M1）；**R-E 按"不接受"通过**（a2r 名解析记为真缺口 `gap:a2r-resolve`）；R-F 按认证轴差异通过（`ok+cert-gap`）。适用范围补充：R-D 同时覆盖 L01–L07 与 R07 的旧引擎值差异；R07 的 a2r "拒绝发射未定型值"记为 `gap:a2r-untyped-emit`（a2r 能力缺口，非 pending）。

## 7. M1 收口的可计数定义

**干净行（clean）** = 四路径单元格全部属于 {`ok`, `ok+cert-gap`, `na:m1`, `diff:accepted`}。
**M1 全绿** = 103 行全部 clean（`probe`/`gap`/`diff:pending` 归零）。

当前测算：**50 / 103 clean**；53 行含至少一个阻塞单元格，分布：

| owner | 行数 | 关闭者 |
|---|---|---|
| `gap:P870`（native record/text） | 24 单元格 / 涉及 T*、R*、E02、I04、R01–R04 | [PLAN-870](../870-ac-mn-record-text-native-expansion.md) |
| `gap:mn-collections`（native 集合/变体） | 14 | native 第二阶段（拟 872+） |
| `gap:P864`（AV 集合源码） | 7 | [PLAN-864](../864-av-source-collections-and-variants.md) |
| `gap:native-typed-entry` | 5 | native typed entry（待立项，可与 R2-N 合并） |
| `probe`（需重跑） | 53 | 852 重绑定任务（见 §8） |
| `gap:a2r-resolve` / `a2r-cert` | 2 / 39 | a2r 跟踪（242）/ 独立认证线 |
| `gap:av-*`（method / alloc-fault / generation） | 3 | AV 侧三个小能力（待立项） |
| `gap:P866` | 1 | [PLAN-866](../866-av-resident-online-hot-reload.md) |
| `gap:a2r-untyped-emit` | 1 | a2r 跟踪（242） |
| `diff:pending` | **0** | 原 8 个已按 R-A..R-D 裁定为 `diff:accepted` |

**注意**：53 个阻塞行中有 **35 行**至少含一个 `probe` 单元格，其中 **7 行"仅因未真实跑过"而阻塞**（全部 `probe` 单元格共 53 个）。它们多半**不是缺口**——这正是"重绑定"能一次性清掉最多行的地方。

## 8. 对 852 与脚本的影响（Spec 增量提案，非本次实施）

| delta_id | 目标 | 变更 |
|---|---|---|
| SD-01 | `docs/specs/auto-test-case/ace-m1-conformance.md` | 追加五态词表、`ok+cert-gap` 语义、`na:m1(rule)` 与 `diff:accepted(rule)` 的判定规则；**不改 12 列 schema**（状态列 token 自由形态，已在用） |
| SD-02 | 同上（验收节） | acceptance scope 的计数改为**分轴**：clean / blocking-gap / na / accepted-diff / probe；`na` 单列且需规则分布 |
| SD-03 | `docs/specs/auto-test-case/architecture.md` | 明确"a2r 认证轴独立于 M1 判据"，与 G08 对齐 |

重绑定执行（不在本件范围）：一次真实重跑 → 把 27 个 `wait:*` 与 24 个 `probe`（换算后共 **53 个 `probe` 单元格**）逐格定级。**2026-10-28 用户裁定：不另立计划，并入 [PLAN-862](../862-av-source-entry-unit-and-tail-semantics.md) 的 T-06（§5.5）**，由该 Plan 的 merge 门一并收口。**在重绑定完成前，"clean 行数"只能当估算，不能当验收。**

## 9. 与在途/待立计划的映射

```text
862（入口）→ 解锁 T*/R*/L*/V* 的源码前门
864（集合源码）→ 关 7 个 gap:P864
870（mn record/text）→ 关 24 个 gap:P870
871（切换退役）→ 不直接关行，但让 ac 列身份稳定（否则重绑定会反复失效）
866（在线重载）→ 关 1 个 gap:P866（I08 的 AV 列）
native 第二阶段 → 关 14 + 5（collections/typed entry）
a2r 线（242）→ 关 39 个认证缺口 + 2 个名解析
```

## 10. 已裁定（2026-10-28，用户问卷）

| # | 问题 | 裁定 | 直接后果 |
|---|---|---|---|
| 1 | 三类"本阶段不要求"是否接受 | **接受**（NA-1 旧引擎/a2r 不测带位宽入口；NA-2 不测动态类型；NA-3 native 不测闭包/动态类型/在线热重载/暂停恢复） | 94 个格子转为 `na:m1`，并写明重新进入条件 |
| 2 | 旧引擎算错/接受非法程序是否记为已知差异 | **接受**（R-A 遮蔽与命名实参、R-B 整数语义、R-C 接受非法程序、R-D 运行时与值缺陷，含 L01–L07、R07） | 33 个格子转为 `diff:accepted`，保留证据但不阻塞 |
| 3 | a2r 不做名字检查 | **算真缺口**（`gap:a2r-resolve`，R-E 不接受） | 2 个格子保持阻塞，由 a2r 补 |
| 4 | a2r 的"对金样"与"能编译"是否分开 | **分开记** | 39 个格子转为 `ok+cert-gap`；认证缺口另账，不阻塞 M1 |

裁定只作用于**旧侧**（旧引擎 / a2r 的既有差异）；新引擎（AV / native）与冻结 oracle 的任何差异仍是 `diff:pending`，不得套用上述裁定。

## 11. 风险

- **`na` 变成垃圾桶**：靠 §5 的四条反滥用规则；merge 报告单列分布。
- **`diff:accepted` 掩盖新实现缺陷**：裁定只针对**旧侧**行为；新侧（AV/mn）与冻结 oracle 的任何差异仍是 `diff:pending`，不得套用 R-A..R-D。
- **重绑定把 probe 直接翻成 ok**：必须有真实执行证据与父侧判定，不得据"契约已备"推断。
- **本件与 852 归档状态冲突**：852 已归档，本件只写**口径**（不改 `binding.tsv`）；实际改状态由后续重绑定计划执行，并在其 plan 内留证。

## 附录 A：逐行重分类（103 行）

按 2026-10-28 已裁定口径生成。列说明：`现状(ac / av)` 取自当前 `binding.tsv` 的 mn/av 两列原文；`新 ac/av/avm/a2r` 为定稿值（`na:m1(NA-x)`、`diff:accepted(R-x)`、`ok+cert-gap` 为新 token）。avm/a2r 两列对同一族高度一致，逐行仍全列以保证可核。

| ID | 能力 | 现状(ac / av) | 新 ac | 新 av | avm | a2r | owner / 依据 |
|---|---|---|---|---|---|---|---|
| B01 | 整数/优先级 | `ok` / `ok` | ok | ok | ok | ok | — |
| B02 | 有符号除余 | `ok` / `ok` | ok | ok | ok | ok | — |
| B03 | 比较与布尔 | `ok` / `ok` | ok | ok | ok | ok | — |
| B04 | 短路AND | `ok` / `ok` | ok | ok | ok | ok | — |
| B05 | 短路OR | `ok` / `ok` | ok | ok | ok | ok | — |
| B06 | 零次条件循环 | `ok` / `ok` | ok | ok | ok | ok | — |
| B07 | 嵌套循环/break/continue | `ok` / `ok` | ok | ok | ok | ok | — |
| B08 | 词法作用域/遮蔽 | `ok` / `ok` | ok | ok | diff:accepted(R-A/R-B) | ok+cert-gap | accepted diff |
| B09 | 递归/显式返回 | `ok` / `ok` | ok | ok | ok | ok | — |
| B10 | 前向引用/相互递归 | `ok` / `ok` | ok | ok | ok | ok | — |
| B11 | 命名实参绑定 | `ok` / `ok` | ok | ok | diff:accepted(R-A/R-B) | ok+cert-gap | accepted diff |
| B12 | 一元运算 | `ok` / `ok` | ok | ok | ok | ok | — |
| B13 | 精确i32溢出 | `ok` / `ok` | ok | ok | diff:accepted(R-A/R-B) | ok+cert-gap | accepted diff |
| B14 | 除零 | `ok` / `ok` | ok | ok | ok | ok+cert-gap | — |
| B15 | MIN余负一 | `ok` / `ok` | ok | ok | diff:accepted(R-A/R-B) | ok+cert-gap | accepted diff |
| B16 | 未执行分支也查名 | `ok` / `ok` | ok | ok | ok | gap:a2r-resolve | a2r 名解析（242 跟踪） |
| B17 | bool与int分离 | `probe` / `probe` | probe | probe | diff:accepted(R-C) | probe | 需真实重跑后定级 |
| B18 | let不可写 | `probe` / `probe` | probe | probe | diff:accepted(R-C) | probe | 需真实重跑后定级 |
| B19 | 全路径返回 | `probe` / `probe` | probe | probe | diff:accepted(R-C) | probe | 需真实重跑后定级 |
| B20 | 实际执行短路RHS陷阱 | `ok` / `ok` | ok | ok | ok | ok+cert-gap | — |
| T01 | Unicode标量/UTF8字节 | `gap:mn-scalar-only` / `wait:av-text-822` | gap:P870 | probe | diff:accepted(R-D) | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| T02 | 标量索引切片 | `gap:mn-scalar-only` / `wait:av-text-822` | gap:P870 | probe | diff:accepted(R-D) | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| T03 | Str深clone/move/拼接 | `gap:mn-scalar-only` / `wait:av-text-822` | gap:P870 | probe | diff:accepted(R-D) | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| T04 | Unicode字符码点 | `gap:mn-scalar-only` / `wait:av-text-822` | gap:P870 | probe | diff:accepted(R-D) | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| T05 | 文本NUL比较 | `gap:mn-scalar-only` / `wait:av-text-822` | gap:P870 | probe | ok | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| T06 | 借用不外逃 | `gap:mn-scalar-only` / `wait:av-text-822` | gap:P870 | probe | diff:accepted(R-C) | probe | PLAN-870；需真实重跑后定级 |
| T07 | 非法Unicode标量 | `gap:mn-scalar-only` / `wait:av-text-822` | gap:P870 | probe | ok | ok | PLAN-870；需真实重跑后定级 |
| T08 | 切片越界 | `gap:mn-scalar-only` / `wait:av-text-822` | gap:P870 | probe | ok | probe | PLAN-870；需真实重跑后定级 |
| R01 | type值复制 | `gap:mn-scalar-only` / `wait:av-record-822` | gap:P870 | probe | diff:accepted(R-D) | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| R02 | 嵌套type字段 | `gap:mn-scalar-only` / `wait:av-record-822` | gap:P870 | probe | ok | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| R03 | 命名type构造映射 | `gap:mn-scalar-only` / `wait:av-record-822` | gap:P870 | probe | diff:accepted(R-D) | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| R04 | 拥有文本字段深clone | `gap:mn-scalar-only` / `wait:av-record-822` | gap:P870 | probe | diff:accepted(R-D) | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| R05 | move参数/返回 | `gap:mn-scalar-only` / `wait:av-record-822` | gap:P870 | probe | ok | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| R06 | 重叠赋值先物化RHS | `gap:mn-scalar-only` / `wait:av-record-822` | gap:P870 | probe | ok | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| R07 | mut参数写回 | `gap:mn-scalar-only` / `wait:av-record-822` | gap:P870 | probe | diff:accepted(R-D) | gap:a2r-untyped-emit | PLAN-870；a2r 未定型发射（242 跟踪）；需真实重跑后定级 |
| R08 | static与只读方法 | `gap:mn-scalar-only` / `gap:av-lower-method` | gap:P870 | gap:av-method | ok | ok+cert-gap | AV 实例方法（待立项）；PLAN-870 |
| R09 | 同布局不同名不兼容 | `gap:mn-scalar-only` / `wait:av-record-822` | gap:P870 | probe | diff:accepted(R-C) | probe | PLAN-870；需真实重跑后定级 |
| R10 | move后读取 | `gap:mn-scalar-only` / `wait:av-record-822` | gap:P870 | probe | diff:accepted(R-C) | probe | PLAN-870；需真实重跑后定级 |
| L01 | 类型化List基本操作 | `gap:mn-scalar-only` / `gap:av-unsupported-list` | gap:mn-collections | gap:P864 | diff:accepted(R-D) | ok+cert-gap | PLAN-864；native 第二阶段（待立项） |
| L02 | List插入/移除 | `gap:mn-scalar-only` / `gap:av-unsupported-list` | gap:mn-collections | gap:P864 | diff:accepted(R-D) | ok+cert-gap | PLAN-864；native 第二阶段（待立项） |
| L03 | 空List/clear | `gap:mn-scalar-only` / `gap:av-unsupported-list` | gap:mn-collections | gap:P864 | diff:accepted(R-D) | ok+cert-gap | PLAN-864；native 第二阶段（待立项） |
| L04 | List拥有type/借用读 | `gap:mn-scalar-only` / `gap:av-unsupported-list` | gap:mn-collections | gap:P864 | diff:accepted(R-D) | ok+cert-gap | PLAN-864；native 第二阶段（待立项） |
| L05 | List移出owned值 | `gap:mn-scalar-only` / `gap:av-unsupported-list` | gap:mn-collections | gap:P864 | diff:accepted(R-D) | ok+cert-gap | PLAN-864；native 第二阶段（待立项） |
| L06 | List越界 | `gap:mn-scalar-only` / `gap:av-unsupported-list` | gap:mn-collections | gap:P864 | diff:accepted(R-C) | ok | PLAN-864；native 第二阶段（待立项） |
| L07 | List元素替换 | `gap:mn-scalar-only` / `gap:av-unsupported-list` | gap:mn-collections | gap:P864 | diff:accepted(R-D) | ok+cert-gap | PLAN-864；native 第二阶段（待立项） |
| V01 | Option Some/None | `gap:mn-scalar-only` / `wait:av-variants-822` | gap:mn-collections | probe | ok | ok+cert-gap | native 第二阶段（待立项）；需真实重跑后定级 |
| V02 | Result Ok | `gap:mn-scalar-only` / `wait:av-variants-822` | gap:mn-collections | probe | ok | ok+cert-gap | native 第二阶段（待立项）；需真实重跑后定级 |
| V03 | Result拥有错误载荷 | `gap:mn-scalar-only` / `wait:av-variants-822` | gap:mn-collections | probe | diff:accepted(R-D) | ok | native 第二阶段（待立项）；需真实重跑后定级 |
| V04 | 有限enum带载荷 | `gap:mn-scalar-only` / `wait:av-variants-822` | gap:mn-collections | probe | ok | ok+cert-gap | native 第二阶段（待立项）；需真实重跑后定级 |
| V05 | enum拥有type载荷 | `gap:mn-scalar-only` / `wait:av-variants-822` | gap:mn-collections | probe | ok | ok+cert-gap | native 第二阶段（待立项）；需真实重跑后定级 |
| V06 | 切换活分支 | `gap:mn-scalar-only` / `wait:av-variants-822` | gap:mn-collections | probe | ok | ok+cert-gap | native 第二阶段（待立项）；需真实重跑后定级 |
| V07 | Option None分支 | `gap:mn-scalar-only` / `wait:av-variants-822` | gap:mn-collections | probe | ok | ok+cert-gap | native 第二阶段（待立项）；需真实重跑后定级 |
| K01 | move闭包捕获 | `gap:mn-scalar-only` / `ok:av-closures-823` | na:m1(NA-3) | ok | ok | ok+cert-gap | — |
| K02 | 创建者退出后owned闭包 | `gap:mn-scalar-only` / `ok:av-closures-823` | na:m1(NA-3) | ok | diff:accepted(R-D) | diff:accepted(R-C) | accepted diff |
| K03 | 同步非逃逸借用捕获 | `gap:mn-scalar-only` / `ok:av-closures-823` | na:m1(NA-3) | ok | ok | ok+cert-gap | — |
| K04 | 借用闭包逃逸拒绝 | `gap:mn-scalar-only` / `ok:av-closures-823` | na:m1(NA-3) | ok | ok | ok | — |
| K05 | 闭包环境显式深clone | `gap:mn-scalar-only` / `ok:av-closures-823` | na:m1(NA-3) | ok | diff:accepted(R-D) | diff:accepted(R-C) | accepted diff |
| K06 | owned闭包可变环境 | `gap:mn-scalar-only` / `ok:av-closures-823` | na:m1(NA-3) | ok | ok | ok+cert-gap | — |
| W01 | i64位型typed入口 | `gap:mn-no-typed-entry` / `ok:av-data-typed` | gap:native-typed-entry | ok | na:m1(NA-1) | na:m1(NA-1) | native typed entry（待立项） |
| W02 | u64位型typed入口 | `gap:mn-no-typed-entry` / `ok:av-data-typed` | gap:native-typed-entry | ok | na:m1(NA-1) | na:m1(NA-1) | native typed entry（待立项） |
| W03 | f64负零typed入口 | `gap:mn-no-typed-entry` / `ok:av-data-typed` | gap:native-typed-entry | ok | na:m1(NA-1) | na:m1(NA-1) | native typed entry（待立项） |
| W04 | f64 NaN payload | `gap:mn-no-typed-entry` / `ok:av-data-typed` | gap:native-typed-entry | ok | na:m1(NA-1) | na:m1(NA-1) | native typed entry（待立项） |
| W05 | f32 NaN payload | `gap:mn-no-typed-entry` / `ok:av-data-typed` | gap:native-typed-entry | ok | na:m1(NA-1) | na:m1(NA-1) | native typed entry（待立项） |
| D01 | Dyn正确take | `gap:ac-composite-native` / `ok:av-dyn-824` | na:m1(NA-3) | ok | na:m1(NA-2) | na:m1(NA-2) | — |
| D02 | Dyn错型take保源 | `gap:ac-composite-native` / `ok:av-dyn-824` | na:m1(NA-3) | ok | na:m1(NA-2) | na:m1(NA-2) | — |
| D03 | 名义类型identity | `gap:ac-composite-native` / `ok:av-dyn-824` | na:m1(NA-3) | ok | na:m1(NA-2) | na:m1(NA-2) | — |
| D04 | 四独立值pack | `gap:ac-composite-native` / `ok:av-dyn-824` | na:m1(NA-3) | ok | na:m1(NA-2) | na:m1(NA-2) | — |
| D05 | 递归白名单门 | `gap:ac-composite-native` / `ok:av-dyn-824` | na:m1(NA-3) | ok | na:m1(NA-2) | na:m1(NA-2) | — |
| D06 | owned树freeze/恢复 | `gap:ac-composite-native` / `ok:av-dyn-824` | na:m1(NA-3) | ok | na:m1(NA-2) | na:m1(NA-2) | — |
| D07 | portable载荷变异 | `gap:ac-composite-native` / `ok:av-dyn-824` | na:m1(NA-3) | ok | na:m1(NA-2) | na:m1(NA-2) | — |
| D08 | revision/权限失效 | `gap:ac-composite-native` / `ok:av-dyn-824` | na:m1(NA-3) | ok | na:m1(NA-2) | na:m1(NA-2) | — |
| D09 | 共享DAG/循环拒绝 | `gap:ac-composite-native` / `ok:av-dyn-824` | na:m1(NA-3) | ok | na:m1(NA-2) | na:m1(NA-2) | — |
| E01 | 命名实参求值序 | `probe` / `probe` | probe | probe | probe | probe | 需真实重跑后定级 |
| E02 | type构造源码序 | `gap:ac-composite-native` / `wait:av-record-822` | gap:P870 | probe | probe | probe | PLAN-870；需真实重跑后定级 |
| E03 | 静态合法性与可达性分离 | `ok` / `ok` | ok | ok | ok | gap:a2r-resolve | a2r 名解析（242 跟踪） |
| C01 | 标量CTE执行/回灌 | `ok` / `ok` | ok | ok | na:m1 | na:m1 | — |
| C02 | Meta负依赖/权限 | `ok` / `ok` | ok | ok | na:m1 | na:m1 | — |
| R01 | owned作用域LIFO | `gap:ac-composite-native` / `probe` | gap:P870 | probe | diff:accepted(R-D) | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| R02 | 构造/clone失败注入 | `gap:ac-composite-native` / `gap:av-alloc-fault` | gap:P870 | gap:av-alloc-fault | ok | ok+cert-gap | AV 失败注入（待立项）；PLAN-870 |
| R03 | 活借用/旧buffer | `gap:ac-composite-native` / `gap:av-821-gen` | gap:P870 | gap:av-generation | diff:accepted(R-D) | ok+cert-gap | AV 句柄代际（待立项）；PLAN-870 |
| R04 | variant切换释放 | `gap:ac-composite-native` / `wait:av-variants-822` | gap:P870 | probe | diff:accepted(R-D) | ok+cert-gap | PLAN-870；需真实重跑后定级 |
| R05 | closure创建者退出 | `gap:ac-composite-native` / `ok:av-closures-823` | na:m1(NA-3) | ok | ok | ok+cert-gap | — |
| R06 | budget/cancel/trap退出 | `probe` / `probe` | probe | probe | ok | ok+cert-gap | 需真实重跑后定级 |
| I01 | 稳定body编辑30修订 | `ok` / `ok` | ok | ok | na:m1 | na:m1 | — |
| I02 | callee-only-body编辑 | `ok` / `ok` | ok | ok | na:m1 | na:m1 | — |
| I03 | callee签名改变 | `ok` / `ok` | ok | ok | na:m1 | na:m1 | — |
| I04 | type/layout改变 | `gap:ac-composite-native` / `wait:av-record-822` | gap:P870 | probe | na:m1 | na:m1 | PLAN-870；需真实重跑后定级 |
| I05 | 改回历史内容 | `ok` / `ok` | ok | ok | na:m1 | na:m1 | — |
| I06 | fresh/hot/重启/跨工程 | `ok` / `ok` | ok | ok | na:m1 | na:m1 | — |
| I07 | AV旧调用暂停/恢复 | `gap:ac-composite-native` / `ok` | na:m1(NA-3) | ok | na:m1 | na:m1 | — |
| I08 | native在线热重载 | `gap:ac-native-loader` / `gap:av-online` | na:m1(NA-3) | gap:P866 | na:m1 | na:m1 | PLAN-866 |
| I09 | 布局不兼容/活状态 | `probe` / `probe` | probe | probe | na:m1 | na:m1 | 需真实重跑后定级 |
| I10 | 512修订churn+GC | `ok` / `ok` | ok | ok | na:m1 | na:m1 | — |
| I11 | 单函数编辑P95 | `ok` / `ok` | ok | ok | na:m1 | na:m1 | — |
| I12 | 测试灵敏度负控 | `ok` / `ok` | ok | ok | ok | ok | — |
| G01 | 真实a2r认证 | `gap:ac-composite-native` / `gap:av-822` | probe | probe | na:m1 | ok | 需真实重跑后定级 |
| G02 | 日常零编译运行 | `n/a` / `n/a` | na:m1 | na:m1 | na:m1 | ok | — |
| G03 | 输出不变复用证书 | `n/a` / `n/a` | na:m1 | na:m1 | na:m1 | ok | — |
| G04 | 内容漂移FAIL | `n/a` / `n/a` | na:m1 | na:m1 | na:m1 | ok | — |
| G05 | 依赖漂移STALE | `n/a` / `n/a` | na:m1 | na:m1 | na:m1 | ok | — |
| G06 | 无证书不称通过 | `n/a` / `n/a` | na:m1 | na:m1 | na:m1 | ok | — |
| G07 | 篡改/漏产物非绿 | `n/a` / `n/a` | na:m1 | na:m1 | na:m1 | ok | — |
| G08 | 重复认证复用收据 | `n/a` / `n/a` | na:m1 | na:m1 | na:m1 | ok | — |
