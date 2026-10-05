# PLAN-743 合入后独立质量复核（2026-10-05）

**结论：needs_fix。** 18/18 既有 Python 测试通过，扫描制品可重复生成；
旧 F-01 链接与 F-03 use 边计数修复确认有效。但新增反例发现三个工具缺口，
合同/验收文档有两处不一致，当前输入有待复核的过期决定，另有任务勾选簿记问题。
用户已授权有问题时激活同一计划追加 Phase；本轮不实施产品或工具修复。

## 基线与实测范围

- reviewed_commit：fa3abe47677992d41b1aef948f36ad46c645f7f8；plan_revision=1。
- base_commit：c1ac219e73ee2ed1ef6ba8dfec49c131bfbf1a75（原实施基线）。
- 旧 reviewed d7013b290 的落地映射 9abc87acc，以及 SD 交付 05d0127a4/
  bbfca37b8、ledger 274a9ded4、归档 bab649682 均由 merge-base 确认是当前 HEAD 祖先。
- 743 专用实施 worktree 已清理；本次读已提交主检出、在本会话 scratch/临时副本运行
  工具与单元测试，未对主仓输入或报告执行 --write，未采信执行总结代替重放。
- Python 3.14.2，标准库；无外仓、网络依赖。本轮为调研工具/文档范围，按 Category A 不运行 cargo 或 docs_gen。
- 实际执行：当前 --check；unittest 18/18；真实输入向两个独立输出目录 --write；
  三个人工决定负例；两份扫描输入反例；15 个活动 Markdown 链接检查。
- 固定扫描输入未漂移：8 源模块+注册宿主共 9 文件；use 边独立重数 18。
  两次生成字节相同，去掉仅供审计的 HEAD 后 manifest 与已提交版本相同，summary 字节相同。
- 旧 F-01：15 个受检活动链接无断链（排除提案正文围栏内的未来链接）。
  旧 F-03：18 条 use 边正确；不再次报告它们为未修复。
- SD-01/02 当前 canonical 正文与原冻结提案正文匹配；SD-03 交叉链接在案。
  匹配表示按原合同交付，不能证明原合同的语义无缺口。

### 冻结输入 / Spec 增量指纹

| 来源 | SHA-256 |
|---|---|
| scripts/acc_inventory.py | ea9a39f500f41ed4fa4c7f7b1a3de1d9d8754e38ad2e3c5938f87670833e822f |
| scripts/tests/test_acc_inventory.py | 40ada8524929d95989fe3ff0c01a027766912d360b067bab3334cff42d358b0c |
| docs/plans/archive/743-acc-bootstrap-hir-contract.md | 315dfd3ca8a62a8caba016616057442f5fcdfebc4107c3b3552f11fb3506a01c |
| docs/specs/auto-acc/project.md | 7e4481ae6f9ac220ec69a1c5efb2317c3e9c46a6f6f7aaa377710610aba92251 |
| docs/specs/auto-hir/stage-contract.md | d387ce54701646b5e6f70d42b1949ab6327b75ed7f2d051fcdcc4da306b444ae |
| docs/specs/auto-hir/project.md | ae1a898fe12d5cd9f1d883fb68564049355556b488bc995b894f0976babe9660 |
| docs/specs/auto-ac/project.md | 9679401e686e5de6ed11bc678ea0866d630f26fdfc7890249332a5a44e997f03 |
| docs/design/strategy/auto-acc-bootstrap-contract.md | a83e7a6c83192c8e0b1bc85cf502612cc9e5f6fd3807dc3bedf6700a20613797 |
| docs/reports/743-acc-hir-contract/proposed-spec-delta.md | b6b807110e5beac655e660b00f2639152ca9a041e513843514487c075ed39daa |
| docs/reports/743-acc-hir-contract/manual-decisions.json | a684bf9cb748272088e2599599f99454a5b61aaf535dfbdd2e0fe5ed215cc5aa |

## 发现与处理

### P743-QA-01 / P2：跨行字符串恢复到 code，伪调用进入清单

scripts/acc_inventory.py:130–145 在换行时将 string/char 状态恢复 code；
scan_module 又把 unterminated-string 升为跨行能力证据。
multiline-string.at 中 pretend() 完全在字符串内，却生成 unknown-bare-call（line 3），
不是“字符串语义未知”的正确占位。auto/lib/engine.at:314–315 本身就有合法跨行字符串；
MD-506 却将换行恢复称为保守正确处理。当前保护测试只覆盖单行字符串。

影响 AC-01/02、T-02/03。修复：合法跨行字面量保持屏蔽直到实际结束，正确跟踪换行/转义；
不支持形态整体标记词法不确定，不能把内部伪代码当真实声明/调用/use。
补多行伪代码/转义换行/关闭后真实调用的反例，并重审 MD-506、生成清单与受影响结论。

### P743-QA-02 / P2：按方法名认 native，越过接收者身份

scripts/acc_inventory.py:312–318 的 NATIVE_METHODS 判定早于本地类型判定；
变量接收者无需任何类型证据，仅叫 len/new 等就归 native-runtime。
same-name-method.at 明确声明 Meter.len/new，x.len() 和 Meter.new() 仍被标 native-runtime。
并非只有合成反例：已交付 manifest 把真实 CG.new（codegen.at:4168 ×5）与
Ar.new（a2r.at:4654 ×1）标为 native-runtime。

影响 AC-01/02/03，违背“未解析/同名方法保留 unknown”规则。
修复：本地类型/owner 证据优先；宿主 namespace 及能证明的内建接收者才可标 native；
未知变量接收者、裸/管道同名方法没有作用域证据时保持 unknown。
不要求实现完整名字解析器；补调用分类断言，不能只测试声明按 owner 计数。

### P743-QA-03 / P2：没有绑定或证据也能“全部新鲜”

scripts/acc_inventory.py:553–593 仅检查 decisions 是 list；
bound_input_hashes 缺失/空值走空循环，证据不存在只 WARN，空 decisions 报 0 条全部新鲜。
独立 fixture 副本实测：
- unbound-decision：非空决定、空绑定 → --check exit0 / 人工层 ok。
- missing-evidence：不存在文件证据、空绑定 → WARN 后仍 CHECK-OK。
- empty-decisions：空决定集 → 0 条全部新鲜 / 人工层 ok。

影响 AC-02/03/08。这是已提供人工层校验缺口，不把有意支持的 T-03 前
“人工层 absent、仅检查扫描”模式误判为 bug。
修复：校验版本/记录结构/唯一 ID/合法状态、非空绑定、证据文件及其 hash 覆盖；
完成态增加显式严格门，拒绝空/缺失人工层及未闭环项，早期 scan-only 模式清楚区分。
反例应非零且定位决定 ID，不能抛无上下文 traceback或只警告后宣称 ok。

### P743-QA-04 / P2：常量折叠的编译期拒绝边界未定义

docs/design/strategy/auto-acc-bootstrap-contract.md:137–147：
eval.const-fold 在必经路径常量溢出时允许诊断 fold.unsafe-context；
但 §2 要求保持 overflow:trap 的运行期行为，preconditions 的“const 语境”没有定义为语言显式要求的编译期求值域。
语义反例（合同分析，非本轮新跑 native）：mark_a(); return MAX_I32 + 1（overflow:trap）。
如果该 pass 按当前笼统规则覆盖普通运行期常量表达式，原 profile 先有可观察 a、再 ExitProcess(70)；若以“必经路径”为理由编译期拒绝，
程序不再运行，a 消失，错误阶段也变化。可达性不能证明编译期拒绝与运行期 trap 等价。若作者仅意指显式 comptime/const 语境，应明确定义排除普通 runtime 表达式；本项是合同边界缺口，不是已有优化器的运行缺陷。
此外 R3 的“effect 只能收窄”与例示 pure→observable 方向术语混用，容易误导后续 pass。

影响 AC-04/05、T-04。修复：普通运行期表达式保留 trap（或等价 trap 节点），
只有语言独立规定必须编译期求值的上下文才可按其规则诊断；
明确 effect 保守分析与实际语义保持的不同义务，更新两例/反例及 proposed delta，
不实现优化器、不改 741 运行语义、不新增 effect lattice 冻结。

### P743-QA-05 / P2：锚点状态和工作包前置不一致

acceptance-matrix.md:13 将 A2 的“同语义源码版”列 existing，
但已有证据只是 Atom HIR；源码 adapter 未实现。同表 A3 则正确分开 HIR existing / 源码 gated。
next-work-packages.md:13 要计算 int/bool adapter 的正例包含 A8 str/List，
候选①范围不含它们；候选②才扩 str/List，且明确须候选①落地才能验源码。
候选②:34 又把 A9 enum/is-match 当其语料，范围未交付 enum/is-match 前置。
“每个 gated 都能在两候选/ABI 找到 owner”的声明不足以解决这些能力缺口。

影响 AC-06/07、T-05。修复：区分 HIR 已有证据与待建源码输入；
两候选的最小交付/前置/延期锚点形成无环依赖，A8 必须由提供 str/List 的工作包认领，
A9 的 enum/is 必须显式列前置与后续负责工作线。保留 10 个锚点，不伪造源码证据，
不在本轮抢占新的计划号或实施额外运行时。

### P743-QA-06 / P2（输入新鲜度，非漂移门实现 bug）：8 条决定、9 处绑定过期

当前 --check exit1 / CHECK-FAIL 9 项。MD-203/205/301/302/304/406/409/421 共 8 条，
引用后续 741 r2 更新的 auto-ac/project.md 与 auto-hir/project.md。
扫描源码未漂移；陈旧决定门按设计生效，不能凭 r1 合入收据宣称当前可直接复用。
修复：逐条读 Spec diff 与相关证据，记录结论不变/调整/待定及依据后更新绑定；
不要统一替换 hash 或免除 delta 目标文件漂移。741 Phase 3 未修问题由 741 负责，
743 标明其事实/验收可用性，不在本计划偷偷修 native 实现。

### P743-QA-07 / P3：归档任务勾选与完成收据不一致

归档 frontmatter=current_step 6/6、merge pass；T-05/T-06 仍 [ ]，
而原修复交接明确写“复位 [x]”。这次重新激活时校正历史任务勾选，
将新 Phase 任务另列待执行，保留旧 needs_fix/pass/merge 收据。

## AC 对账与路由

| AC | 本轮结果 | 证据/缺口 |
|---|---|---|
| AC-01 | partial | 9 输入/8 模块/18 use 有实证；QA-01/02 污染分类 |
| AC-02 | fail | 18 测试和确定性绿；QA-01/02/03 新反例、当前人工层 stale |
| AC-03 | partial | 模块去留/新主体/非主体分层齐全；QA-02/03/06 决定可信度需重核 |
| AC-04 | partial | S0–S6/API/目标边界存在；QA-04 effect/当前事实与未来约束需消歧 |
| AC-05 | partial | pass 模板和两例/X1–X8 齐全，但 QA-04 编译期域/trap 保真边界未定义 |
| AC-06 | partial | 10 锚点/Gen1–3/桥矩阵齐全；QA-05 状态不准确 |
| AC-07 | partial | 两候选/估算/SD 交付齐全；QA-05 依赖不可直接执行 |
| AC-08 | fail | 新失败已实测，旧 pass 不能覆盖；需独立 r2 复审 |

将全部发现与修复方案追加到同一 PLAN-743 Phase 2，status=executing/revision 2；
不销账、不更新 canonical 行为 Spec，也不修改 Python 实现。
next=work 修复后再独立 review/merge。741 Phase 3 保持独立范围。
报告 results.json/audit.json/tests.txt 固定为本次基线；复现脚本用新输出目录，
避免重写历史资料：python -B reproduce.py <repo-root> <fresh-output-dir>。
