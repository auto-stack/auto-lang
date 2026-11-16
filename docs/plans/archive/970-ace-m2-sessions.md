---
plan_id: PLAN-970
status: archived
feature_name: REPL持久会话daemon与在线更新
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-15
plan_revision: 6
current_step: 8
total_steps: 8
depends_on: [PLAN-969, PLAN-968]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-sessions.md]
touched_goals: [GOAL-002]
---

# [PLAN-970] REPL持久会话daemon与在线更新

## 0. 变更摘要

2026-11-12 r5：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r4当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

迁REPL/daemon/client/MCP及持久变量、错误后继续执行、会话隔离；复用ACE snapshot/lease/store实现同PID更新，旧在途调用持旧代；不兼容布局明确拒绝或用户可见重启，不假装任意状态迁移。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `crates/auto-lang-core/src/autovm_persistent.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/autovm_daemon.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/mcp`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/av/src`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：REPL/daemon/同PID编辑与恢复。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/auto-lang-core/src/autovm_persistent.rs](../../crates/auto-lang-core/src/autovm_persistent.rs)：AutovmReplSession/Stats旧入口语义；读取SHA256前16 `c33ebf86a154e2f9`

- [crates/auto-lang-core/src/autovm_daemon.rs](../../crates/auto-lang-core/src/autovm_daemon.rs)：daemon协议/退出参考；读取SHA256前16 `6380a8b0ef767342`

- [experimental/ace/crates/av/src/resident.rs](../../experimental/ace/crates/av/src/resident.rs)：已有resident owner/代际，扩统一通路；读取SHA256前16 `2a1c1712d245f116`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


迁REPL/daemon/client/MCP及持久变量、错误后继续执行、会话隔离；复用ACE snapshot/lease/store实现同PID更新，旧在途调用持旧代；不兼容布局明确拒绝或用户可见重启，不假装任意状态迁移。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：协议版本与持久状态迁移（r2）

明确daemon/client/MCP版本、断连重连、请求幂等/取消及旧持久会话格式迁移/拒绝。源工件恢复与用户状态恢复分开，不用删用户状态完成升级。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r5当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at)（下列节选L1–L1；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn main() int { return 2 + 3 * 4 }
```

本包所需独立结果：B01初值14；同PID将最后常量4改5后应17，坏编辑保14/17上一已接受代，修复后新值；两个session不得串状态。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** REPL/daemon/edit请求→969会话→增量query/候选→safepoint/原子发布→后续调用与持久恢复。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct SessionRevision { source: RevisionId, accepted: RootId, candidate: Option<CandidateId>, leases: OwnerSet }
struct UpdateReceipt { old: RootId, new: RootId, changed: OwnerSet, diagnostics: DiagnosticSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/auto-lang-core/src/autovm_persistent.rs](../../crates/auto-lang-core/src/autovm_persistent.rs) | AutovmReplSession/Stats旧入口语义 |
| [crates/auto-lang-core/src/autovm_daemon.rs](../../crates/auto-lang-core/src/autovm_daemon.rs) | daemon协议/退出参考 |
| [experimental/ace/crates/av/src/resident.rs](../../experimental/ace/crates/av/src/resident.rs) | 已有resident owner/代际，扩统一通路 |

**状态、错误与资源。** 坏UTF-8编辑/旧revision/错base/cancel/陈旧provider不得切当前根；重启不能冒同PID热更。 在途旧代保活、晚结果不写新代，最后pin释放；恢复重验而非反序列化Checked，body改不重编未变宿主。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at) | 改源值14→17，独立算术oracle | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `b7e5f25e46902be2` | 拟ace.m2.p970.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [experimental/ace/crates/av/src/query_entry.rs](../../experimental/ace/crates/av/src/query_entry.rs) | 已有query公开消费/恢复接缝 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `eab8602115ef3666` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |
| [docs/specs/auto-semantics/source-query-session.md](../specs/auto-semantics/source-query-session.md) | 已有增量/恢复合同，不造第二store | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `1a5375476e35f390` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 真实edit API/返回和REPL变量持久策略、safepoint/不兼容更新、daemon close/输入EOF先冻结；不是仅打印PID。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-sessions.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-sessions.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-sessions.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |

### 1024发现承接合同（2026-11-13，r6，仅规划）

承接1024的F05：会话职责与生命周期收敛，优先级P2（F12仅保留约束）。用户仅授权复审与分配，尚未授权实现；原支持/AC/任务/阶段/预算不减少，原任务进度0保持。

T-01先冻结入口→状态→安装层→worker/外部consumer职责表；T-02/03只收敛被证明重复的候选/接受态术语与机制，T-04保独立邻例。1030只修历史864外层，1032/1033各修自己的事务，不由本Plan接管这些修复；消费其正式合入SHA。

1002关系：核心职责/原入口反例/固定组治理均不以1002为硬前置；新作者格式迁移或document/IR/adapter重构仅等待G1002（1002 T-03四例七检查及负控通过，T-04独立复审、接口正式合入并绑定SHA）。不等待全语料搬迁；1002不反向依赖本Plan整包。现有M2能力前置不因承接登记被豁免，也不把这些非紧急结构改造伪装成可立即施工。

来源：[1024](1024-ace-m2-code-quality-and-refactoring-audit.md) r2与Git 81d8fc03f10f4a7d179b0bc3a917602406ed7b91。F01–35高风险修复另立1027–1037；本Plan消费正式合入结果并在T-01重核漂移。与在途文件同写须串行，1017成本身份单独协调，不能靠删例减成本。规范影响沿本Plan原SD目标，review补当前实际实现，不改canonical Specs。

### T-01 冻结记录（2026-11-15，r6 work，实施授权＝goal 会话）

**前探证据**：scratch probe `D:/autostack/plan-artifacts/plan-970/probe/`（`src/main.rs` sha256前16 `2223c67b5f771790`，P1..P10共38项检查全过，单跑0.18s，log `aeabc3b1e54b76f8`）。覆盖：B01初值14／同PID改4→5=17（revision+1）／坏编辑具名拒＋旧接受代保持17／修复=20／双会话隔离＋跨会话候选拒／bool通道if-encode／变量prelude注入＋纯捕获探针／陈旧base拒／warmup_persisted→restore零冷回退＋恢复后首改fast-path／resident混代值。基面 master `95de79c11`；依赖969（archived，merge收据dd88a41d8）与968（archived，8f4e6a5a8）正式来源已核；§4五个hash重核一致。

**实现策略裁定**（比较过的备选）：(a) B01原形单函数会话——拒（最后区域编辑不收敛，实测SyntaxNotConverged；F形锚后可编辑）；(b) 全源累积每eval重建（旧MCP式）——不作主路径（重执行≠持久、O(n) churn），保留为声明集变化时的显式reshape路径；(c) **冻结声明集+通道模型（选定）**；(d) resident暂停链作REPL主执行——不选（每eval需新执行），保留为长eval异步链（V-02/V-04）。实测：顶层声明增删被"declaration set changed (add/remove/rename)"具名拒绝——函数增量域为封闭声明集、仅body快速路径，此为产品约束非本包缺陷。

**规范程序形状**：`fn main() int { return __repl_eval_int() }`（可编辑dispatch，非最后区域）＋`fn __repl_eval_int() int`＋`fn __repl_eval_bool() bool`（可编辑eval体）＋`fn __tail_anchor() int { return 0 }`（对齐锚，恒最后）。保留名（`__repl_eval_int/__repl_eval_bool/__tail_anchor`）冲突→具名拒绝。输入源（B01等）规范化装载：entry body进int通道、用户函数保序保留、锚追加；观测值不变（P1=14）。bool eval经if-encode走int入口（1/0），报告层解码；dispatch仅在通道切换时编辑（常态每eval 1次commit）。

**REPL变量持久策略**：会话变量表（名→标量值，宿主持有，cap 64具名界）＋prelude注入（字面量）＋纯捕获探针（对改动的绑定以其RHS纯再求值；scalar profile无副作用操作支撑纯性）；非int/bool标量、record/closure值不可携带→具名边界（`session.var/non-carryable`），会话继续并在报告登记。错误后继续：坏eval零效果（表不动、revision不动、旧代可运行；P3实测）。eval尾表达式→`return (E)`重写；纯语句eval→sink return＋报告value=none（对齐旧REPL无显示行为）。支持域边界（scalar profile实测）：f64/i64/str/char/unit类型与表达式语句不在此profile——涉及→具名拒绝；text/console/collections域不并入本包（985/B或后继消费）。

**同PID更新/在途**：`FunctionIncrementalSession::prepare_edit/commit`（原子单发布点、revision+1、跨会话/陈旧base/关闭具名拒；P2/P5/P8实测）＋`ResidentRunner`安全点提交（§2.3 deferral guard、在途帧钉旧代；P10混代值）＋close释放pin（V-04）。编辑API：字节区间(start,end,insert)；坏UTF-8/旧revision/错base→具名拒绝不切当前根。

**协议冻结（T-08实施）**：daemon JSONL over named pipe(Windows)/unix socket；请求/响应带`version`（现值1=`ac.session.daemon/1`，缺省视为1）；方法名保旧7个（new-session/eval/inspect/reset/delete/snapshot/list/ping）＋新增`edit/eval-async/result/cancel/close`；幂等＝请求id＋末响应缓存重放（重连重试不重做有副作用提交）；半条消息/畸形行→具名错误后继续（旧stdio行为）；版本不匹配→具名拒绝列支持版本。close/EOF：stdio EOF→取消在途＋脏会话持久化＋关闭；pipe逐连接EOF守护常驻（旧行为）；session/close即释放pin。MCP：initialize按规范响应己方支持版本（2024-11-05），不再忽略客户端版本；旧7工具名保留＋新增`auto_edit/auto_vars`。

**持久状态迁移**：会话用户态记录`ac.session.state/1`（source_history/变量表/revision/时间戳，JSON）与工件恢复（warmup_persisted/restore，重验而非反序列化Checked；P9零冷回退）分离；旧持久格式不存在（旧daemon仅source_history导出）→legacy源快照按源重放显式重建（具名`legacy-source-replay`）；未知schema/坏升级→具名拒绝且原文件不动（V-08）。

**职责表（AUDIT-05，1024 F05承接）**：

| 入口 | 状态 | 安装层 | worker/外部consumer |
|---|---|---|---|
| ACE session facade（新neutral `auto-lang-core::ace_session`，无ACE依赖） | 会话id/变量表/revision/通道dispatch | `FunctionIncrementalSession`（函数域prepare/commit） | ace-host会话桥（Windows）→AV执行 |
| `ace_daemon`/`ace_client`（auto-lang-core新文件） | 协议id/幂等缓存/连接生命周期 | 经facade | 管道/stdio客户端；auto-man CLI接线归985/B |
| MCP ACE变体（`mcp/ace_server.rs`新文件） | 经facade | 经facade | MCP stdio客户端 |
| 旧autovm_persistent/daemon/client/mcp | 保留不动（退役归987） | 旧VM帧 | playground/plan082/plan738既有测试（conformance锚扫描保持绿） |

重复收敛（T-02/03范围）：候选/接受态术语与观察统一到facade报告；不动data/closures/collections/864各session的domain策略（1024二轮裁定保留）。

**命令/组冻结**：完整职责组＝固定组`ace-av ace-artifacts ace-semantics auto-lang-core integration`＋`ace-facade`（ace-host改动触发）；新语料`crates/auto-lang/test/cases/ace-m2/p970/`＋新recipe`ac-av/session-repl-v1`（worker serve模块＋ac_av.rs check＋append-only dispatch＋组`case-ace-m2-sessions-970`）；host桥fixtures `auto-lang-ace-host/tests/t970_*.rs`；neutral单测`p970_*`入auto-lang-core（旧auto-lang-core组的ACE家族排除规则自动归属显式组）。

**资源合同**：max_sessions默认20、idle 1800s（旧值保持）；变量表cap 64；ExecLimits默认＋请求wall；新增测试用例各≤10s（probe单跑0.18s基准）。平台：实机门与host桥在Windows（named pipe；沿969 cfg(windows)）；stdio/neutral逻辑跨平台可编译。

**依赖漂移重核**：1030/1032/1033仍drafting未合入——本包不接管其修复、不等待（各修各自事务；function-session线不受864外层影响）；G1002已满足（本包用旧作者格式+Rust入口）；1024 F12沿用（lease/存储用ace-artifacts既有，不建第二存储）。

**T-02实施调整（2026-11-15，work）**：原冻结的"新recipe`ac-av/session-repl-v1`＋worker serve＋语料组注册"改为沿PLAN-969 facade先例——永久fixtures驻`auto-lang-ace-host/tests/`（t970_sessions.rs 11例全绿，真实产品路径＝neutral contract＋bridge，B01/14→17→20、隔离、变量携带、混代、取消/关闭、cap、快照），不再在av-worker侧复制会话语义（避免F05第二语义模型，也避免worker响应无法参与父侧source-hash绑定）。拟`ace.m2.p970.source01`映射＝t970_b01_same_pid_edit_oracle（B01原件输入、独立算术oracle）。范围/AC不变；语料目录`ace-m2/p970/`不建。

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 会话A写x不影响B | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 同PID旧长调用返旧值、新调用返新值 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 坏更新保旧 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 断连/关闭释放最后pin | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | fresh恢复后第一次编辑不重建全源，下一次恢复读到已提交状态 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-av ace-artifacts ace-semantics auto-lang-core integration`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 重复请求不重做有副作用提交；半条消息/旧协议具名错误；坏升级保原状态文件；同PID更新后重连读正确新代；降级行为交986。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | B01初值14；同PID将最后常量4改5后应17，坏编辑保14/17上一已接受代，修复后新值；两个session不得串状态。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 坏UTF-8编辑/旧revision/错base/cancel/陈旧provider不得切当前根；重启不能冒同PID热更。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 在途旧代保活、晚结果不写新代，最后pin释放；恢复重验而非反序列化Checked，body改不重编未变宿主。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。


### 1024承接的具名验证

| Case ID | 发现 / 原任务 / AC | 输入/设置 | 实际入口 | 独立关闭oracle | 状态 |
|---|---|---|---|---|---|
| AUDIT-05 | F05 / T-01..04 / AC-AUDIT | source/typed/owned/closure/历史864入口；body/签名/坏编辑/恢复后首改/close | 各AV session→真实worker→M2 session facade | 职责/接受态/真实生产与恢复消费者全表；共用机制保domain，不把所有session当重复删除 | 拟实施，未运行 |

原组/平台/ignore/规模与正式性能分轴不变；新格式需G1002，原格式和Rust入口可先验证。每条oracle必须在本Plan原work验证完成前取得，不能移到merge或借登记冒关闭。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 协议版本与持久状态迁移的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

- [x] AC-AUDIT / work：1024承接F05的上述每条AUDIT用例及保留边界完成；原任务T-01/T-02/T-03/T-04负责，原独立review逐项核，未执行不PASS。［✅ 见T-04证据行：职责表全列落地、接受态统一到facade报告、兄弟session域保留、真实生产/恢复consumer接线；source/typed/owned/closure/历史864入口未见删除或静默合并］

## 8. 执行步骤

当前共8任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。［✅ 2026-11-15：冻结记录见§5"T-01冻结记录"；probe P1..P10全38项过（plan-artifacts/plan-970/probe，main.rs `2223c67b5f771790`/log `aeabc3b1e54b76f8`，0.18s）；依赖969/968 archived已核；§4五hash重核一致；AUDIT-05职责表已入冻结；1030/1032/1033未合入不等待；G1002满足］

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。［✅ 2026-11-15：neutral contract `auto-lang-core::ace_session`（5单测）＋参考host桥 `auto-lang-ace-host::session_bridge`（冻结声明集+通道模型；Engine Direct/Resident）＋永久fixtures `tests/t970_sessions.rs` 11/11绿（B01 14→17→20/隔离/变量携带/bool通道/reshape+保留名/错误后继续/混代V-02/取消关闭V-04/重置/cap/快照）；语料承载调整见§5"T-02实施调整"；commit 6e62352fc] 

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。［✅ 2026-11-15：`ace_daemon`（ac.session.daemon/1版本纪律/幂等缓存64/坏行继续/stdio EOF优雅/pipe逐连接EOF常驻/新方法edit·eval-async·result·cancel·close，5单测）＋`ace_client`（持久连接/重试10×200ms/30s收包/单调id）＋`mcp/ace_server`（initialize协商2024-11-05/旧7工具名保留+auto_edit·auto_vars，4单测）；真实roundtrip fixtures `t970_consumers.rs` 3/3（named pipe create/eval/edit 14→17、MCP工具面42/41→42/helper 7→9→10、T-08持久化）；旧autovm daemon/client/mcp不动（987退役）；commit bb8785b96] 

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：明确daemon/client/MCP版本、断连重连、请求幂等/取消及旧持久会话格式迁移/拒绝。源工件恢复与用户状态恢复分开，不用删用户状态完成升级。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。［✅ 2026-11-15：协议版本/幂等/取消/close语义在T-03面实现并测（V-08重复请求重放逐字/半条消息具名错误后继续/版本不匹配列支持版本/坏升级原文件不动+具名跳过/断连重连按id恢复/同PID更新后重连读新代17）；持久化＝`ac.session.state/1` op-log（eval/edit/reset），恢复走op重放重验（非反序列化Checked），legacy `.at` 显式源重放，Close/过期删record，shutdown flush原子替换；fixture t970_persistence_records 3阶段全绿；commit bb8785b96] 

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。［✅ 2026-11-16：V行映射（§5冻结+fixtures）——V-01=t970_session_isolation_variables；V-02=t970_async_edit_mixed_generations（混代2100<v<2700偶差+后2700）；V-03=t970_b01坏编辑revision不动仍17＋error_then_continue；V-04=t970_close_cancel（cancel停链/close拒后续）；V-05=probe P9（av级restore零冷回退+恢复后首改fast-path）＋t970_persistence_records（记录恢复/revision连续/变量重立）；V-08=daemon单测（幂等逐字重放/半条消息继续/版本不匹配）＋persistence fixture（坏升级原文件不动/close删record）＋pipe roundtrip（同PID更新后17）；V-DETAIL-01=t970_b01（14/17/20）；V-DETAIL-02=t970_edit_negative_controls（bounds/utf8具名拒+状态保持）；V-DETAIL-03=混代+P9+持久重放；V-DETAIL-04=§5冻结旧新映射表+实现按表；AUDIT-05=§5职责表落地（neutral合同=入口/host桥=状态/FIS=安装层/daemon·client·MCP=consumer）+各兄弟session域未删未并（F05裁定遵守）。完整职责组（worktree 29843b6bd）：ace-facade PASS 50.6s；ace-av 1222/1222过×3跑（组FAIL裁决＝MSVC vctip.exe工作负载后驻留，环境归因，本包未动ace crate）；ace-artifacts首跑r3_f01并发时序flake→隔离与整组重放均PASS；ace-semantics 282/283（1红＝ac-semantics语料executed状态缺失，主检出同红已核=预存非本包）；auto-lang-core PASS 100s；integration 3353/3354（1红＝plan702门扫../../../auto-ui兄弟仓，lang-970组内不存在该兄弟，主检出同测PASS已核=环境归因）。命令与时间全程log在plan-artifacts/plan-970/t04-*.log］ 

- [x] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。［✅ 2026-11-16：独立上下文final review @29843b6bd＝needs_fix（F-970-R-01..R-12，详§9）；修复@295e93f7c；targeted重审＝**pass**绑定295e93f7c：R-01..R-07全闭（R-06/R-12按处方文档化）、零新回归、全部计时≤10s（最慢单例~0.4s）、新码warning零、fmt clean、树clean；剩余N-1(fmt牵动sched_worker/sched_m2_968纯格式)/N-2(value字段多义须入SD-01)/N-3(merge冻结并集含ace-av)为合并期簿记］

- [x] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [x] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-970/auto-lang`，分支 `plan-970-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-970/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-16 stage: merge / PLAN-970 r6 / outcome: pass（delivered）：**prepared**＝reviewed pass绑定295e93f7c（r6，依赖969/968 archived）＋冻结SD（SD-01/PHASE/DETAIL→新`docs/specs/ace-migration/m2-sessions.md`；SD-02→replacement-scope-and-evidence.md追加"PLAN-970，2026-11-16"节）worktree内落盘commit 9fee3a268（docs-only后代）。**合并门（冻结并集）**＝final tree跑`cargo t ace-av ace-facade auto-lang-core integration`：ace-av PASS（本次vctip未复发）、ace-facade PASS、auto-lang-core PASS、integration 3353/3354（1红＝plan702门扫`../../../auto-ui`兄弟仓，lang-970组内无该兄弟，主检出同测PASS已核＝环境归因，非970）；ace-artifacts/ace-semantics未受修复触及→复用T-04证据（artifacts重放PASS/semantics预存红具名）。**landed**＝master推进（972已并）→rebase 5提交（range-diff全等：6e62352fc=735cd1888、bb8785b96=35947b44c、29843b6bd=8c006a034、295e93f7c=dc46f6629、9fee3a268=900ba5ba9）→main `git merge --ff-only`＝delivery commit `900ba5ba9`，无merge提交，线性历史；master无代码WIP（bookkeeping除外）。**canonical_verified**＝`docs/specs/ace-migration/m2-sessions.md`（sha256前16 `1e3b1376a56aa8d2`）＋replacement-scope-and-evidence.md（PLAN-970收据节）在landed commit内。**archived**＝git mv至docs/plans/archive/970-ace-m2-sessions.md，status: archived，8/8，completion_kind: delivered；N-1/N-2簿记（fmt牵动sched_worker/sched_m2_968纯格式；daemon value字段多义已入SD-01）随归档落账。**cleaned**＝见下一条收据（wt-guard→worktree/branch/组目录删除→plan-artifacts整scope删除→retention刷新）。

2026-11-16 stage: review / review_scope: final（同独立上下文targeted重审）/ PLAN-970 r6 / outcome: **pass**：reviewed `295e93f7cf5d5aaa40c87278bf2f66b04752e6d0`（基面95de79c11，树clean）。R-01闭（into_session additive＋Engine::Resident⟺async_id不变量成立，全unreachable守卫复核无可达项；wire侧cancel→第二async→跑完10000实测）；R-02闭（源文本入value，fixture纯wire取偏移）；R-03闭（pid+纳秒盐＋create碰撞跳过，restore后新建≠恢复id实测）；R-04/R-05闭；R-06/R-12按处方文档化闭；R-07闭（！rebuilt且revision恰+1真判别）。计时：av_resident 14/14 0.083s；core 14/14；t970_sessions 12/12 0.20s；t970_consumers 3/3 0.49s（最慢单例roundtrip~0.4s=300ms有意等待）；全部≤10s。健康：触及文件warning零、fmt clean、树clean。N-1（fmt牵动sched_worker.rs/sched_m2_968.rs纯格式，已逐hunk核）/N-2（daemon value字段多义：eval值/inspect·snapshot源/running标记——SD-01沉淀须写明）/N-3（修复功能触及experimental/ace，merge冻结并集须再含ace-av组）登记为合并期簿记。status: execution_done→reviewed，6/8。next: merge（T-06/T-07）。

2026-11-16 stage: work / PLAN-970 r6 / repair cycle 1：修复提交 `295e93f7c`（树29843b6bd→295e93f7c）。R-01＝ResidentRunner新增additive `into_session()`（av/resident.rs，866纪律不动；av_resident 14/14过）＋poll终态与cancel交接Engine::Direct（第二条async可再arm，fixture覆盖cancel→second-async→跑完10000）；R-02＝wire inspect/snapshot回源文本于value（F-970-R02），pipe fixture改为纯wire取偏移（不再借道进程内facade）；R-03＝会话id掺入进程身份+纳秒盐＋create碰撞跳过循环（fixture：restore后新建≠恢复id且恢复会话完好）；R-04＝删除未用import；R-05＝无id请求不入幂等缓存；R-06/R-12＝契约注释（Eval cancel入口快照语义；Edit偏移以Inspect原源为基）；R-07＝永久断言入t970_persistence_records（恢复后首改！rebuilt且revision恰+1，lane body prelude为靶避开anchor）；R-08＝fmt牵动7文件已在R-08记录（本修复轮fmt再触碰为格式收口）；R-09＝计数修正于本行：新测试函数共30（12+3+14+1结构）。全绿：ace-host全部套件（12+3+既有）、core 14、av_resident 14。next: 复审重验（targeted）。

2026-11-16 stage: review / review_scope: final（独立上下文general-purpose agent，同树独立会话；复审人非实施者）/ PLAN-970 r6 / outcome: **needs_fix**：worktree `D:/autostack/.wt/lang-970/auto-lang`，branch plan-970-dev，reviewed `29843b6bd`（基面95de79c11，树clean）。发现（F-970-R-*）：R-01 MAJOR＝首个async终态/取消后再次eval-async命中Engine::Taken守卫panic守主线程（session_bridge.rs:1538 unreachable）；R-02 MAJOR＝daemon wire inspect/snapshot仅回snapshot_len不带源文本（旧daemon回value源），wire侧edit偏移不可得、roundtrip fixture借道进程内facade；R-03 MAJOR＝会话id为进程内确定计数，restore后首个new-session可同号静默覆盖恢复会话；R-04 MINOR＝ace_server.rs未用import（ContentBlock/Id）；R-05 MINOR＝无id请求按0入幂等缓存跨内容重放；R-06 MINOR＝同步eval cancel仅入口快照（契约类型承诺高于实现，注释已声明）；R-07 MINOR＝V-05首改不重建/V-DETAIL-03 body改不重编未变宿主仅probe证据无永久断言；R-08 NONBLOCKING＝cargo fmt牵动7个声明面外文件（纯格式，已逐hunk核无语义变化）；R-09 NONBLOCKING＝簿记计数小偏（新测试函数实为30）；R-10/R-12 NONBLOCKING＝V-02/V-04字面→冻结语义细化与edit-base(Inspect原源)规则须入spec沉淀。AC图：AC-01/03/PHASE/AUDIT pass；AC-02/06 partial（R-01/02/03/07）；AC-04本审；AC-05未到。测试计时（复核）：core 14例0.00s/墙18.3s（编译17.96s）、t970_sessions 12例0.20s、t970_consumers 3例0.43s、最慢单例daemon roundtrip~0.4s（300ms为有意等待）——全部≤10s。三处非绿组归因全部复核诚实（ace-av 1222/1222×3仅vctip裁决红；sema_boundary主检出同红=预存；plan702主检出PASS=组内缺auto-ui兄弟）。健康：fmt clean；新码warning仅R-04一处。裁定：R-01..R-05+R-07为in-contract修复项回work（同树、影响面＝已触两crate）；R-06/R-12以契约注释+debt登记；R-08/R-09簿记修订；R-10入SD沉淀要求。next: work（repair cycle 1/3）。

2026-11-16 stage: work / PLAN-970 r6 / outcome: pass（全部work任务完成，execution_done）：work树 `D:/autostack/.wt/lang-970/auto-lang`（分支plan-970-dev，交付提交 `29843b6bd`；基面95de79c11）。交付面：neutral `ace_session`合同（5单测）＋参考host桥 `session_bridge`（冻结声明集+int/bool通道+变量携带/纯捕获+resident异步混代+Engine Taken交接）＋`ace_daemon`/`ace_client`（ac.session.daemon/1版本/幂等64/取消/EOF语义）＋`mcp/ace_server`（9工具）＋`ac.session.state/1`持久化（op-log重放重验/legacy .at/坏schema跳过/原子替换/Close删record）；旧autovm_persistent/daemon/client/mcp不动（987退役，conformance锚保持绿——ace-av/ace-facade组全过佐证）。Work-owned AC：AC-01（T-01冻结全表）、AC-02（V行+fixtures，见T-04行）、AC-03（真实consumer接线零legacy回退）、AC-06（V-08全绿）、AC-PHASE（A切片required闭）、AC-AUDIT（职责表落地+域保留）。新用例（25个测试函数，全部≤10s）：t970_sessions 12例0.21s、t970_consumers 3例0.42s（含真实named pipe往返）、auto-lang-core单测14例（p970_*，<0.01s）、T-01 probe P1..P10 38检查0.18s。完整职责组见T-04行（两处环境归因+一处预存红均有主检出对照证据）。无未闭in-contract发现；无阻塞。next_responsible_role: 独立review（T-05）。

2026-11-15 stage: work / PLAN-970 r6 / T-01：goal会话授权整链（work→review→bounded repair→merge）。建唯一树 `D:/autostack/.wt/lang-970/auto-lang`（分支`plan-970-dev`，基面master `95de79c11`）＋材料scope `D:/autostack/plan-artifacts/plan-970/`（retention已登记）。T-01前探38项全过（B01装载14/同PID17/坏编辑保旧/修复20/隔离/bool通道/变量策略/陈旧拒/restore零冷回退/resident混代）；策略裁定＝冻结声明集+通道模型；协议/持久迁移/职责表/命令组/资源合同全部冻结入§5"T-01冻结记录"。关键实测约束：函数增量域为封闭声明集（顶层增删具名拒）、最后区域编辑不收敛（锚不变量）、scalar profile仅int/bool返回。status: drafting→executing，1/8；outcome: in-progress；next: T-02（work）。

2026-11-12 stage:new / PLAN-970 r5：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原8任务/全部AC与V/依赖/阶段/预算和drafting 0/8；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。


2026-11-13 stage:new / r6 / outcome:pass（规划承接） / next:原前置及实施授权后work。用户授权将1024的F05纳入本包原任务，新增具名AUDIT oracle/AC-AUDIT；未增任务、未删原任务/AC/V/阶段/预算、0进度/drafting保持。旧r5规划记录保留；实际实现/执行未开始。1002依赖按任务分流，无整包循环。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
