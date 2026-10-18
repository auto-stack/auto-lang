# PLAN-809 R1：终审就绪核查（needs_fix）

日期：2026-10-18。stage=review，review_scope=final，plan_revision=1。
reviewer：当前独立于实施的 Codex 审查上下文；未创建其他 agent，未虚构模型/session 身份。

## 基线与审查边界

- 主检出 Plan：`docs/plans/809-man-project-management-build-core.md`，revision 1；读取的是主检出当前文件。
- 唯一实施树：`D:/autostack/.wt/lang-809/auto-lang`，branch=`codex/plan-809-dev`。
- reviewed_commit：`aa1d40e1d90b313b7dd6ba57b79123f5278ed90f`。
- base_commit / 当前 master 共同祖先：`2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 主检出 HEAD：`3d61507a87a9f98e3695f74d153331cbfe3d60c9`；存在其他计划簿记脏改，本轮仅追加 809 报告/计划及债务记录。
- 兄弟 auto-down：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，detached；两树 `git status --porcelain=v1` 均为空。
- 最终复核 HEAD/branch/cleanliness 未漂移；`git diff --check 2046a025b..HEAD` exit 0。
- Plan/交接/Spec/关键实现与测试文件 SHA256 见 `review-r1-baseline.json.txt`；原始 SD 冻结副本为 `review-r1-spec-delta.md`。

入口虽称实施完成，但 Plan 仍为 executing，T-06 自认带 pending，T-07 部分，T-08 未勾选。
交接明确承认缺真实工具 driver、run 再生成未切请求、777/792 未收口。
按 auto-plan-review 的 readiness 规则，本轮核实这些已知 work-owned 缺口即退回 work，
**不进行另一轮完整最终验收，不将既有测试总数授信为本轮 pass**。
下列为已证实的合同缺口，未列出的实现、错误路径、公开 API 兼容、全后端/生成输出对照均待完整复审。

## Findings（均 OPEN，不批准延期或缩面）

### F809-R1-01 / P2：真实三 Builder 与失败路径验收未实施，现有 Ninja 断言允许空产物

- 类型：原验收缺口；AC-04，T-06，V-05。
- 计划 §5.4 / V-05 要求 Pac→resolve/scan→三类真实 Builder 的受控工具执行与产物、失败阶段/exit 验证。
  环境缺失必须记 pending；这不表示允许完成声明，更不把“时间预算”视为环境不足。
- `handoff.md` pending #1 明确 driver 未落，理由为时间预算；没有受控真实 Ninja/Cargo/Vue 执行收据。
- `crates/auto-man/tests/plan809_management_core.rs:267` 仅工具存在性探针和 Ninja 内存模式，未提供 Cargo/Vue 真实执行及工具失败/不可写输出/缺依赖控制。
- 同文件 `:312` 的断言为 `contains("build.ninja") || !ninja_file.is_empty() || outputs.is_empty()`。
  空 map 由第三项通过；非空 map 的任意非空首值由第二项通过。它不能证明生成了期望构建规则、源/依赖或可消费产物。
- 修复：落实已冻结 V-05 fixture/driver，对真实工具可用分支执行并断言产物和失败传播；缺失工具逐族记录实际探测证据并保持未完成。内存断言必须对非空、正确目标/源/链接规则有区分力，加入空/坏输出负控。不得改成用探测或 mock 总计替代真实工具验收。

### F809-R1-02 / P2：I-PROJECT 仅切 gen 部分入口，实际生产再生成仍绕回旧读取器

- 类型：原需求缺口 / 未批准延期；AC-02，T-06，V-02。
- 计划 §5.1 明确实际 UI 生成器的项目读取先经请求/访问合同，R04 承接生成算法/资源迁移；没有授权将现有生产 run 路径读取切换延期。
- `automan.rs:1752` 的 run_vue 委托 `run_vue_project`；`vue.rs:7305` 实际调用 `VueProject::from_workspace`，后者 `:3614` 读 pac.at 并 `VueReads::scan`。
- `jet.rs:850` / `ark.rs:756` 的 run 调用旧 `generate_*_project`；后者分别 `jet.rs:798` / `ark.rs:646` 调用 legacy from_workspace；`rust_ui.rs:6109,6174` 也仍调用旧 generate_rust_ui。
- `automan.rs:644,773` 的生产生成调用也没有接 `_with_request`。因此“legacy 仅 oracle/测试”及“生产分派全量切请求”的声明不成立。
- 交接 pending #2 将 run 再生成归 R04，无对应批准修订；不能以此判 AC-02 完成。
- 修复：穷举并完成已约定的生成项目读取入口接线，保留效果 adapter 和必要兼容门面，证明生产调用不绕回第二份读取规则；补齐动态/覆盖/跨 cwd 以及真实生成输出对照。跨文件 sidecar/dep/backend 读面须逐项映射到合同，不能统称“单工程外”自动豁免。

### F809-R1-03 / P2：完整管理/构建规划的纯规则仍留在旧效果层

- 类型：原需求缺口 / 未批准范围收缩；AC-01，T-05（并影响 T-06），V-01。
- 计划 §5.2 要求 Builder 目标、编译/链接输入、依赖约束及动作规划进入 core，上层执行具名动作。
  T-01 contract 的 R-07 还明确依赖下载的去重/合并/版本选择决策进入 core，而 Git/FS/求值执行留 adapter。
- `builder/ninja/builder.rs:172-195` 在 adapter 内按 deps→target、rename 去重后即时执行；注释称为 recorded adjustment。
  `auto-man-core/src/planning.rs` 当前只有端口绑定与 Rust 包选择，并未承接这段依赖目标规划。
- `pac.rs:657-752` 仍在下载/求值编排中直接维护 target_names/new_deps、决定 lib/传递 dep 合并，未形成 R-07 声明的纯决策入口。
- “只有一个消费者”不满足将已约定完整规划留在旧层的理由；本项不要求新增无用抽象，也不要求搬效果实现。
- 修复：从真实消费路径拆出纯决策，typed 输入/结果供原 adapter 消费并保旧序/错误/效果边；补静态委托审计和可区分 oracle。若认为合同不成立，须回 new 提出明确修订并获批准，不能靠实施注释降低 AC。

### F809-R1-04 / P2：work-owned 模块/发行证据及最终规范增量交接未收口

- 类型：验收/收口缺口；AC-04/05，T-07/08，V-06，SD-01..04。
- T-07 明确只完成模块边登记，将 strict 基线重跑与 792 source/resource 清单并集后移；主检出附件仅 contract-baseline.md 和 handoff.md，没有对应验证收据。
- T-08 未完成接口示例/字段读取映射/版本及最终 SD 交接；现有 SD 表仍是原始 before/after 提案，没有核实后的最终规则文本及与当前 canonical 的冲突处理。
- 原始 SD 主张完整单份管理/规划和请求读取，与 F02/F03 的未迁面尚不一致。当前 metadata 的四个 docs/specs 路径和 GOAL-003 保留，**不表示 SD 已通过**。
- 修复：work 完成自身 777 strict、受影响 792 源/资源声明和独立 fresh/locked 消费证据，提供可复跑收据及最终 SD/接口映射；merge 仍负责最终并集后的复验。不得把 work 的接口完整性与发行清单补齐统统归为冻结合入门。

## AC → 任务 → 证据

| AC | 本轮结果 | 任务 / 实现与证据 |
|---|---|---|
| AC-01 | partial（不具备 pass） | T-01..05；core/门面/闭包已有实现，F03 证实完整规则尚缺；其余未做完整最终验收 |
| AC-02 | fail | T-02/06；新请求及 gen 接线存在，F02 的实际生产路径仍旧读取；V-02 全部支持矩阵/产物待验 |
| AC-03 | pending | T-03/04/07；本轮未复跑静态/动态、图 oracle 与可变状态，不据实施自报通过授信 |
| AC-04 | fail | T-06/07/08；F01/F04，真实三 Builder、失败面与发行证据未齐 |
| AC-05 | fail（readiness） | T-09；同树基线与缺口已独立核实，但不满足完整 AC/SD/health 最终审查入口 |
| AC-06 | pending（merge-owned） | T-10/11；本轮不合入、不沉淀 live Specs/ledger、不归档/清树 |

## 路由与后续门禁

outcome=needs_fix；Plan 保持 executing，T-05/06/07 重开，T-08/09/10/11 待办；
T-01..04 保留原实施完成标记（不是本轮独立验收通过），current_step=4/11。
next=在原 809 树按 work 补齐 F01..04、完成所需定向验收、提交并给完整交接后再终审。
本轮未修改实现、未运行 Rust 测试或整档回归；不是环境阻塞结论。

pending_closeout_gates：最终 code/acceptance/Spec review pass 后，merge 在冻结提交上执行裸 cargo t，
core/project-model/auto-man 必要包档及按最终 diff 确定的 UI/trans/VM 触面档（不得在本次 needs_fix 中提前整档）；
最终 777/792 并集复验、规范/ledger/接口已合入版本交接、归档、guard clean 和清理仍 pending。
未授予 merge 权限，未把 reviewed 等同交付。无其他模块实现改动或回退。
