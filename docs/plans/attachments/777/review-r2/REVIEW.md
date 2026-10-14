# PLAN-777 第二轮独立 final review

结论：**needs_fix**。旧例 7/7、自测 12/12 均通过，但补充矩阵 5 项中有 2 项失败。输入只读和新增禁向边检测仍未完整成立。Plan 返回 executing / 0/6；本轮不修实现、不合入、不发布 Spec/ledger、不清理树。

## 审面与独立性

- Plan revision 1；实现树 `D:/autostack/.wt/lang-777/auto-lang`，branch `plan-777-dev`。
- reviewed_commit `aaf70201110ed8afcfbbc6e1ccd99c4e82019982`；修复 code `3b50ea63ec30d68bc820448cf0fcbf100375bfe8`；base `17bc8431457848b040bffa5c261e13b057771559`。
- auto-down 组内兄弟 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。主检出 HEAD `dab2f1897865cde21cc0b78abad7b07412832c12`。
- 本评审会话参与规划与 R1，未参与实现；直接核查提交并选择独立组合输入，未创建子 agent/额外 worktree。
- Spec `docs/specs/design/repository-layout.md` SHA256 `3800856129eb9b5bd742fef9ea2aa0c94f0db780ab2a8ba01fc7ae8965dde49b`；冻结增量见 `spec-delta-reviewed.md`。
- 整个受管输入集合前后 hash 一致，包括 Cargo.lock；HEAD/branch/status 无漂移，实施树干净。详见 `fingerprints.jsonl`。

## 阻断 findings

### F777-R2-01 / P1：外置 auto-ui 没有域映射，新增禁向边假绿

原 AC-02、T-02/T-03/T-04；F777-R1-06 修复路径中的遗漏。

生产配置第 101–105 行将外部禁向边限定为 `to_domains=["ui","man","man-ecosystem"]`，但 `[domains.members]` 没有 `auto-ui="ui"`。`domain_of`（脚本第 913–914 行）按包名查询该表，查不到就给 `unclassified`。因此外置 `auto-ui` 无法命中新规则。这不是未约定的扫描扩面：配置自己的注释明确声明语言域禁止依赖外置 auto-ui/auto-man，Design37 §2 也规定此方向。

独立 S01-auto-ui 用**生产域映射和生产跨根规则**，只把 fixture-lang 定义为当前语言巨 crate 同等的 lang-monolith；干净冻结后真实新增 `auto-ui={path="../../../auto-ui"}`。Cargo metadata oracle 确认依赖有效，工具也报告 to=auto-ui / scope=external / optional=false，但 strict **exit 0 / pass / new_allowed_edges**。同样输入的 auto-man 对照 exit 2；合法外置 SDK 对照 exit 0。原始 manifest、规则、domain、oracle、结果见 `supplemental.jsonl`。

修复：让当前配置中声明要限制的外置实现包得到明确域映射，或用有依据的包/目标映射约束；对未知外置目标保留真实 unknown，不能假称所有外部依赖都违反规则。补生产配置级 UI/Man 禁向与 SDK 合法控制，包含 normal/build、target/继承组合。不要新增豁免或把合法跨仓 SDK 一律禁掉。

### F777-R2-02 / P1：报告可覆盖已读取的外部依赖 manifest

原 AC-01、T-01/T-03/T-04；F777-R1-03 的具体配置旧例关闭，但完整只读不变量仍未闭合。

`extract_declared_edges` 在第 382–383 行读取外部 path 依赖目标 `Cargo.toml`，它是本次调查的输入；`guard_output_target` 第 877–878 行却将扫描根之外的目标直接认定为“安全另存”。S02-external-manifest-output 在有效外置 SDK 对照上执行 `--mode observe --output <external-sdk>/Cargo.toml`：**exit 0，输入 SHA256 改变，Cargo.toml 被 JSON 报告覆盖**。所有变更只发生在评审自有临时副本，真实兄弟仓没有被改写。

修复：扫描时收集本次实际读取的输入身份（包括根外 path 目标 manifest），在写报告前统一校验，不能仅以 root 内外判断安全；保留根外独立报告另存正控。输出/刷新保护需要一张完整输入与目标矩阵，包含配置 override、基线、成员/外部 manifest、源码、锁及路径别名；现有根内拒绝和合法刷新控制继续保留。

## R1 闭合对账与验证边界

| R1 finding | 旧例 | 完整不变量结论 |
|---|---|---|
| 01 多 target 表 | pass | 本轮检查关闭；额外 target + inherited alias + build/dev 组合仅 Windows build 边判红，Cargo oracle 一致 |
| 02 workspace path 基点 | pass | 本轮检查关闭；上述组合也正确解析 |
| 03 输出覆盖输入 | pass | partial；根内配置旧例关闭，根外已读取 manifest 仍会覆盖（R2-02） |
| 04 optional 激活 | pass | 本轮检查关闭；完整边键含 optional/scope/dep_key，未新增豁免 |
| 05 ownership 遗漏 | pass | 本轮检查关闭；27 行、7 组、6 补登记路径均存在，owner/exit_phase 与调研一致 |
| 06 跨根 optional 误红 | pass | 原误红关闭；新生产规则的 auto-ui 目标缺映射（R2-01） |

原 C06 replay 构造的是旧 scope-only 规则文本，并不实际把完整的新生产方向规则注入夹具；现在的 pass 来自 scope-only 被处理成惰性。因此该旧例的绿色不能证明新生产跨仓规则的正向拦截覆盖。本轮 S01 使用实际生产规则及域表完成对照，发现遗漏。工作摘要“六控”实际是 7 条控制（1 阳性 + 6 缺口），本次按原始 case 计数。

配置之外的文本 scanner 明确为候选，未假称语义解析；UI→语言扫描仅 auto_lang 前缀的限制已写清。scope-only 惰性行为是本次代码变化，应在修复诊断中说明与冻结规则合同的关系；不以惰性旧 C06 作为新方向规则的验收证明。

## AC → task → 证据

| AC | 结果 | task / evidence |
|---|---|---|
| AC-01 | partial | T01/T04；真实双扫稳定、规模/hash/revision 可复现，S02 外部输入覆盖失败 |
| AC-02 | fail | T02/T03/T04；窄豁免/撤销/过期/宽豁免控制通过，S01 auto-ui 新增禁向边漏判 |
| AC-03 | pass | T02/T03；完整 ownership 登记、路径存在与候选分类/盲区核查通过 |
| AC-04 | 审计完成，未获 pass | 本报告及独立可重放输入 |
| AC-05 | pending | merge 阶段 Spec/投影/合流/归档/guard；review 不执行 |

独立 `--help`、`--self-test` 12/12、双 observe、真实 strict exit 0；36 manifest 边与 Cargo metadata --locked 完全一致、2052 个 Rust 文件 / 945894 物理行、真实存量 grandfathered=2。双扫证据字节 SHA256 `edd74300df2e259b281269d99d8650ba9dd99c3b95593bced05997feffbb9b00`（评审保存为 Windows 文本；解析后的报告与 work t04r2-observe-a.json 相同）。`git diff --check` 通过，无 Rust 源码改动、编译或 Cargo 测试；Category A 不触发 Rust 整档门禁。真实 strict 正控不抵消变异输入失败。

## 重放与下一步

在任意位置运行（只读实施树，变异均在自有临时副本）：

```text
python docs/plans/attachments/777/review-r2/replay.py --implementation D:/autostack/.wt/lang-777/auto-lang
python docs/plans/attachments/777/review-r2/supplemental.py
```

第二条将每个 case 的 contract_pass 与 actual_exit 写到 supplemental.jsonl；本审面两项失败，脚本整体 exit 1。修复后应核查 case 断言与整体失败传播。

两次 final review 均失败，下一次送审前 work 必须先提供这两类完整路径矩阵、诊断与覆盖调整，并校正闭合表，避免只补具体旧例。同一 lang-777 树修复、补永久控制、提交，再独立复审；T01–T04 因相关实现/配置和依赖验收证据重开，历史成功证据保留。r1 合同不变。
