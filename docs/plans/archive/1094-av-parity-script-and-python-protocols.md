---
plan_id: PLAN-1094
status: archived
feature_name: AV脚本模式与Python对象协议兼容补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-21
plan_revision: 5
current_step: 8
total_steps: 8
depends_on: [PLAN-1078, PLAN-1084, PLAN-1087, PLAN-1092, PLAN-1093]
supersedes_spec_components: [docs/specs/auto-av/python-interop.md, docs/specs/auto-lang/vm/design/null-family.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1094] AV脚本模式与Python对象协议兼容补齐

## 0. 变更摘要

### 2026-11-20 r5 · 补齐 AV 异常交接证据

补具名未捕获对照和两条路径的类型、位置断言，明确 T-06 冻结 AV 证据、T-08 交正式来源。1110 按该来源接 A2PY。两个计划的运行验收均待实施。

### 2026-11-20 r4 · Python 错误交接

1107 已交同源 Python 执行。1094 的异常捕获、错误分类及原位置义务沿原 AV 验收，通过 §5 的接口交接给 A2PY 后继。当前修订为文档交接，实施任务及 AV 阶段门沿 r3。[PLAN-1110](1110-ace-a2py-python-exception-mapping.md)

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`script/python-context.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-16**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

把旧合法.as/script、Python动态调用和with清理接到新AV普通语言体系。

本包交付`IF-AV-PARITY-SCRIPT`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- auto-script/.as、#[script]/#[rust]模式、动态参数/属性/索引/调用、Python对象身份/装箱/数值/Unicode/NUL/关键字与嵌套实参组合。
- with resource as name的进入/退出、return/break/continue/普通异常时__exit__/close恰一次；Python异常进入可捕获语言错误，host撤权/resource/cancel分类不变。
- 消费1026真实PySession与967资源账，不新增PyObject私有owner/第二解释器；普通无use.py程序不加载CPython，外部可选依赖缺失明确NOT_RUN而非PASS。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1094/auto-lang`，分支`plan-1094-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1094/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

1026已交标准use.py与CPU训练，但ASCII≤256显示、动态浮点keyword/嵌套实参与完整Python对象协议仍有限；不能用训练成功代替旧脚本语言集合。旧obj_call Auto分支假阳性等bug不机械复制。

既有来源与边界：

- [catch_value.case.ad](../../crates/auto-lang/test/vm/99_script_err/02_catch_value/catch_value.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。
- [with_as_propagate.case.ad](../../crates/auto-lang/test/vm/99_script_err/04_with_as_propagate/with_as_propagate.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：99_py_dispatch；99_script_err；旧.as/helper/with/pointer-to-Python回归；cookbook中Python对象语言角色。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [script_entry.rs](../../crates/auto-lang-core/src/script_entry.rs) | `d1d0fff6299cdc12983da51ff9dc740666863586c28d6799cdec92ee61449728` |
| [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [py_session.rs](../../experimental/ace/crates/abi/src/py_session.rs) | `50579af08e481ed6d19a76880eda436ba43bb9f7b3d978e95902899b59d8277f` |
| [av.rs](../../experimental/ace/crates/av-core/src/av.rs) | `2ad814939c0b9ec8b7da0e09383d0c621bd88697415fd31a4b0c5c21e856139c` |

硬依赖：PLAN-1078, PLAN-1084, PLAN-1087, PLAN-1092, PLAN-1093；实际外部接口：1026 PySession/CPython adapter正式切片。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

### T-01 冻结记录（2026-11-20，work 阶段，基线 5a76167cc，plan_revision r5）

**输入指纹复核**（实施树 `D:/autostack/.wt/lang-1094/auto-lang` @ 5a76167cc）：

| 源 | 计划值（r3 起草时） | T-01 实测 | 判定 |
|---|---|---|---|
| script_entry.rs | d1d0fff…49728 | 一致 | 未漂移 |
| resolve.rs | 54978de…ea68 | `712008f3…2b7e3` | 漂移：PLAN-1107（共享 py-op 事实层/A2PY 发射）+ 1082 T-02..05 |
| py_session.rs | 50579af…8277 | 一致 | 未漂移 |
| av.rs | 2ad814…6139c | `7e45425f…deb6c` | 漂移：PLAN-1106（增量事件会话）、982（text/console 车道）、981（AGE port） |

7 份主承接旧文档（99_py_dispatch×2、99_script_err×5）SHA16 与路线图清单逐一相符，无漂移。
r3 起草后出现的新输入源：PLAN-1107 已交付共享 py-op 结构化事实层
（`ace-foreign-contract::py_ops`，`p1107/py-language-contract/1`）与帧操作数单次求值约束——
本包普通面 py 接线**必须消费该事实层**，不重造第二套帧文本/py 解析；r4/r5 的 1110 交接
以该同源事实为前置。

**有界全链探测**（ace-core CLI `check-source --source-profile auto.source.language.av/v1`，
经 parse→resolve→typecheck→bind→verify；探测源 16 份+汇总在
`D:/autostack/plan-artifacts/plan-1094/probes/`，probe-results.txt SHA256 前 16 位 `8a39c905f8ee7df2`）：

| 探测形态 | 实际 stage/code | 冻结结论 |
|---|---|---|
| 正控 `fn main() int { return 2+3*4 }` | OK（require `av.aggregates/1`） | 普通面链路健康 |
| `use.py builtins: int` | parse/`text/source.use-rs.syntax` | int 成员拼写触发 use.rs/c 诊断臂 |
| `use.py builtins: str` 单成员 | 编译 OK（零 py capability）；体内 `str("a")` → bind/`source.undefined` | 声明可解析但 1026 前门变换（成员提升+py_locals）未接普通面 |
| `use.py …: int, str` / `str, int` 多成员 | parse 失败（int 首=use-rs.syntax；str 首=`expected fn, found ,`） | 1026 合同的多成员按源序形未准入 |
| `use.py numpy: arange` + `t.len()` | bind/`source.undefined`（arange）＋`bind/source.unsupported`（numeric receiver no method len） | 外对象动态分派缺口=1084 范围 |
| 纯 Auto `try/catch` | 编译 OK | try/catch 语句面已在普通面；缺 py 异常→可捕获语言错误接线（1087＋本包 T-03） |
| `with … as`（纯 Auto 与 py 形） | parse/`text/source.syntax`（expected statement, found `with`） | with 语句面不存在=1092 #[with] 共享语法范围 |
| `.to(str)`（普通面） | CST 前门破口→回退 legacy parser，掩蔽报 `` `fn` is not part of this source profile `` | py-op `.to(...)` 映射未准入普通面；诊断去掩蔽属本包 AC-02 面 |
| `"…\0…"` 字面量 | 同上回退（`\0` 不在 1079 已交付转义集） | §6 python-unicode 落 canonical 用 `\u{0}` 拼写 |

**可行性结论**：1026/1107 已交 Closures 族 foreign 全链（lowering→ABC→AV dispatch→PySession），
普通面与其同走 `compile_language_source` 管道、同 Closures 家族——本包**无需新运行时地基**；
缺口收敛为四条源面/接线：①普通面 use.py 前门准入＋成员提升＋py_locals 纪律（消费 1107 事实层）；
②py 异常→catch 接线与分类渲染（消费 1087 展开面；`PyException ValueError: …` 渲染沿原 oracle）；
③with 语句面（消费 1092 #[with]）；④外对象动态分派（消费 1084 动态值）。
①中不依赖 1084 的部分也无法单独出口：T-02 合同的动态参数/属性/索引/调用组合以 1084 地基为前置。

**FC 冻结**：canonical 位置 `crates/auto-lang/test/cases/language/script/python-context.case.ad`
（新建 script/ 行为目录）；`language.suite.ad` 仅追加引用（schema autolang.suite/2 既有 6 文件不动）；
legacy-map.tsv 由 T-02 添 7 行（owner=PLAN-1094；原 ID/check/oracle/SHA16 见路线图 §PLAN-1094 清单）；
判等条件：case ID 族 `av.parity.1094.*`；相同文件名/不同路径/模式/授权/错误阶段/资源角色不自动合并；
旧 7 文档只作具名冻结历史，不再是活跃作者。Rust 内联角色枚举：ace-abi 7 文件
（py_bridge_1026/py_session_1026/py_session_1107/py_error_1107/py_protocol_1107/py_identity_1107/py_ops_1107）
＋ av/src/language_source_worker.rs（普通链 worker=本包接线点）；av/tests 无 py 专属内联。

**规范基线差异**（四份 supersedes 目标现状）：python-interop.md=1026 v1 冻结＋1107 附录（py-op 事实、
操作数单次求值、失败面 Trap{ForeignStatus} 不发布部分 stdout）；language-parity.md=AVP-00..04 已交
（§4 明确"脚本/Python 协议"未交）；null-family.md=550 旧 AutoVM 契约（本包 SD-03 只增 AV 事实）；
feature-organized-corpus.md=1078 已交作者/suite/AV 视图/#ignore 机制。

## 5. 详细设计

### Python 错误切片交接

1094 提供 `IF-AV-PARITY-SCRIPT` 的 Python 错误切片。共享作者文件位于 `crates/auto-lang/test/cases/language/script/python-context.case.ad`（拟建），包含 §6 的捕获用例和未捕获对照。每条 Case 交一份 `main.as` / `auto-script` 源和一项必验 AV check。

| 交接项 | AV 证据 |
|---|---|
| 源和运行身份 | Case ID、check ID、实际入口和能力、逻辑源路径、`auto-script`、输入原始字节 SHA256、执行提交与 provider 身份。 |
| 异常类型 | 两例都从实际执行取到 Python `ValueError` 及对应 Auto 错误类别 `value`（已有 provider 对应 class 5）。`value` 是语义类别，provider 数字属于 AV 实现。运行阶段、消息和控制流结果随收据记录。 |
| 捕获分支 | 观察到实际 handler 处理的错误，进入一次并返回 `42`。成功的入口值和空双流沿原用例断言。 |
| 未捕获分支 | 零捕获，运行以结构化 Auto 语言错误结束，无成功入口值。实际错误携带相同异常分类。 |
| 原始位置 | 对失败的 `int("abc")` 记录逻辑源路径和原始 UTF-8 半开字节范围。位置经过实际执行产生，父侧使用 §6 的独立范围断言。 |

普通 AV worker 当前以 `probe.at` 编译并省略 `Error` / `Trap` 的 span，现有输出字段不足以独立验收。T-03/04 在实际 AV 路径接通位置和类型观察。可使用既有结果通道或同次执行的 AV 观察接口，后者需与 AV check 的实际输入、执行和结果绑定，由父侧判定。捕获观察发生在 handler 接收错误时，未捕获取自终态。错误类型与位置取自执行记录，预期从源和规范独立确定。验证无需新增 Auto 反射语法。逻辑路径按 Case 源身份传递，暂存名只用于定位工作文件。

T-06 冻结这两条 AV check 的证据及字段语义，T-08 在切片正式落主时交出处和复验入口。若先独立落主所需切片，其正式收据即可供 1110 消费。1094 的 T-06..08 仅看 AV 和其共享前置。A2PY 检查尚未注册时登记在 Notes/义务表，已注册的检查按阶段忽略或未选，相关准备和 dispatch 均为零。A2PY adapter、解释器、检查结果及双端评审归 1110，持续使用同一作者源。[PLAN-1110](1110-ace-a2py-python-exception-mapping.md)

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [script_entry.rs](../../crates/auto-lang-core/src/script_entry.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [py_session.rs](../../experimental/ace/crates/abi/src/py_session.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av.rs](../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/script/python-context.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/auto-av/python-interop.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/auto-lang/vm/design/null-family.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-FC | modify（本特性消费/来源） | `docs/specs/auto-test-case/feature-organized-corpus.md` | 按特性唯一作者、同源多端检查、旧义务映射和本包迁移/观察来源；实际交付后沉淀，当前不宣称已支持 | 公共机制由1078生产一次，本包保原语言/目标义务 | AC-FC、V-FC-SOURCE/MIGRATE/TARGET |

`language-parity.md`由AVP-00首次创建，后续串行增量维护；它是当前规范的能力/来源映射，不是第二注册器。实施前本包文档只提议，真实merge后按独立复审结果沉淀，不提前将Spec写成已支持。


## 6. 测试设计

### V-AV-ONLY / V-IGNORE-LIFECYCLE（未执行）

原T-03..06落实、T-07复核：同一Case AV已选，AC/A2R检查加#ignore；制造该目标缺工具/运行失败/错误oracle时，目标prepare/dispatch计数为0，AV仍按独立预期通过，忽略行不充PASS。将AV oracle改错必须使AV门红，全部有效AV检查被ignore或零选中也必须红。移除某非AV阶段ignore并进入其后端选择后，该目标失败必须在该后端门显现，而AV视图仍不选择它；移除ignore不能同时删原平台/ignore理由。公共格式/重复ID/AV必需shared规则出错不被这一阶段策略豁免。

### FC 测试组织与迁移必验增量（未执行）

| V-ID | owner_stage / 原任务 | 必须观察的结果 |
|---|---|---|
| V-FC-SOURCE | work / T-02..06 | 本轮同一source/project仅选AV真实执行/按规定阶段拒绝；改公共body使相关旧证据陈旧，非AV行保持#ignore/未选与后续owner，不要求本轮运行。 |
| V-FC-MIGRATE | work / T-01/02/05/06 | 原case/check/条件/判决角色→canonical映射完整；非等价邻例、错误/资源/恢复角色不丢；suite、worker/内联/工具读者均切唯一活跃作者，旧副本仅具名历史。 |
| V-FC-TARGET | work / T-04..06 | AV oracle变异、重复活Case ID/AV旧活入口残留、零条有效AV/缺AV required检查必红；AC/A2X oracle不符或缺工具在#ignore/未选时不进入AV门。非AV完整目标oracle/emit-vs-run控制留给其启用阶段。 |

[本包旧VM文档逐项清单](../design/strategy/av-autovm-legacy-case-map.md#plan-1094-原文档清单)静态主承接 **7份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1094.nested-python-call` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1094.python-error-catch` | AC-01/03 | work / T-03..06 | `auto-case` → 普通 AV 真运行与捕获观察 | 捕获一次，`ValueError` → `value`，返回 42，双流为空。原始位置为 `main.as` 中 `int("abc")`。 | LF 草案的原始 UTF-8 范围 `[51,61)`，第 3 行第 15 列起。 |
| `av.parity.1094.python-error-uncaught` | AC-02/03 | work / T-03..06 | 同一普通 AV 路径，取运行终态 | 零捕获，未捕获语言错误，`ValueError` → `value`，无成功值。原始位置为 `main.as` 中 `int("abc")`。 | LF 草案的范围 `[45,55)`，第 3 行第 9 列起。 |
| `av.parity.1094.context-manager-close` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1094.python-unicode` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `WITH-EXITS` | AC-03/AC-04 | work / T-04..05 | 正常/return/break/continue/异常/嵌套with/enter失败，检查退出顺序和实际外部对象关闭计数；Python __exit__抑制仅按该协议处理普通语言异常。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `SCRIPT-MODE` | AC-03/AC-04 | work / T-04..05 | 同一文本普通模式的合法/非法邻例、.as与pragma优先级、无py时adapter不加载；浮点keyword、kwargs顺序及>256字节多字节文本实际值验。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `PY-ERROR-OBS` | AC-01/02/03 | work / T-04..06 | 以两条 §6 Python 错误用例作正控。分别替换预期类别、偏移原始 span 或移除必需观察记录，父侧必须报失败。 | 采用实际 handler/运行终态与同源指纹。T-06 冻结结构化断言入口，验明结果对 `ValueError`、`value` 和表达式级位置敏感。 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: nested-python-call
id: "av.parity.1094.nested-python-call"
tags: ["av-language-parity", "plan-1094"]
entry-mode: "program"

### Auto
path: "main.as"
syntax: "auto-script"

```auto
use.py builtins: int, str
fn main() int { return int(str(42)).to(int) }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: python-error-catch
id: "av.parity.1094.python-error-catch"
tags: ["av-language-parity", "plan-1094"]
entry-mode: "program"

### Auto
path: "main.as"
syntax: "auto-script"

```auto
use.py builtins: int
fn main() int {
 try { return int("abc").to(int) } catch e { return 42 }
}
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
父侧按 PY-ERROR-OBS 验证实际 handler 接收一次、`ValueError`、Auto 类别 `value`、原消息和失败操作的位置。
`````

未捕获对照的源草案如下，T-05 落入同一作者文件，Case ID 为 `av.parity.1094.python-error-uncaught`，逻辑路径与语法模式同捕获例。冻结时用实际源字节核对 §6 半开范围，按 T-01/T-04 确认的观察入口设置失败断言。

```auto
use.py builtins: int
fn main() int {
 return int("abc").to(int)
}
```

`````autodown
## Case: context-manager-close
id: "av.parity.1094.context-manager-close"
tags: ["av-language-parity", "plan-1094"]
entry-mode: "program"

### Auto
path: "main.as"
syntax: "auto-script"

```auto
use.py io: StringIO
use.py contextlib: closing
fn main() int {
 let buf = StringIO()
 try { with closing(buf) as r { r.write("ok"); fail("stop") } } catch e { }
 if buf.closed.to(bool) { return 42 }; return 0
}
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
使用CPython标准io/contextlib真实对象。closed证明执行了退出；恰一次与各控制出口仍由WITH-EXITS计数fixture补充，不以一个bool包办。
`````

`````autodown
## Case: python-unicode
id: "av.parity.1094.python-unicode"
tags: ["av-language-parity", "plan-1094"]
entry-mode: "program"

### Auto
path: "main.as"
syntax: "auto-script"

```auto
use.py builtins: str
fn main() int {
 let p = str("中文\0尾")
 if p.to(str) == "中文\0尾" { return 42 }; return 0
}
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

### 未来执行命令与范围

专属实施树中执行（本轮未执行）：

```powershell
cargo t case-av-language-parity ace-source ace-syntax ace-av ace-abi ace-python
```

固定组`case-av-language-parity`为AVP-00拟新增的公共语料AV视图：按已注册目标/动作选AV检查，不能只假设check ID恒为av，零选中/缺文件/坏子项必红；旧VM只读有效历史参考，本轮legacy-reference检查#ignore，不为它启动旧引擎。`--check av`只点选确实名为av的检查，不代表全部AV检查；完整视图的目标/动作选择由1078在既有选择机制接通。普通AV验证不偷偷执行旧引擎。新suite/recipe在AVP-00交付前不得假称上述命令当前可跑。

共享语法/绑定/IR/ABC改变时，T-06本轮只验AV及AV实际依赖的共同语法/语义/IR/ABI/框架正确性；不得因改动共享层而自动把ace-native、ace-a2x等非AV功能完整组变成本包门。混合职责组内非AV执行腿按1078冻结的AV阶段选择/配置具名排除，不批量改整crate为ignore；不为忽略检查主动准备非AV worker/SDK/工具。AV所需Rust/FFI provider及共享代码真实必需的编译/准入仍须正确，不能用#ignore掩盖AV自身问题。此阶段策略不批量停用已有独立非AV回归，也不删原性能/平台/ignore条件；普通组不附加ace-perf，Windows不执行Linux/WSL。

实施/内部检查/最终独立复审均在上述同一专属树顺序进行，用户未指派额外agent或模型；独立复审指独立上下文核验，不擅开新chat/第二实施树。原权限、重内存、资源预算和性能样本条件保持；不为了日常60秒建议强停合法正确性用例。

## 7. 验收标准

- [ ] **AC-AV-ONLY｜owner_stage: work**：§5 AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE满足，当前仅AV及其共享正确性参与门；非AV#ignore/未选不prepare、不dispatch、不计PASS、不阻AV。原T-03..06落为真实选择/报告/隔离控制，T-07独立核，T-08不以非AV未齐阻本包落主。

- [ ] **AC-FC｜owner_stage: work**：§5 FC及§6 V-FC-SOURCE/MIGRATE/TARGET由原T-01..06落实；唯一活跃作者、逐原义务迁移/条件/历史身份守恒与本轮所选AV真实观察齐，非AV#ignore/NOT_RUN分列且不阻AV。原T-07 review逐项核，T-08仅真实落主后交来源；既有AC全部保留，不缩语言或后端分母。

- [ ] **AC-01｜owner_stage: work**：§6所有正例在普通AV真实执行满足固定值/双流及本包完整语义，原源不以mini/get/loop等替换后冒原兼容。
- [ ] **AC-02｜owner_stage: work**：所有反例在约定阶段给可辨识错误与原source span。Python 未捕获对照必须检验运行终态、错误类型和失败操作原始位置。未授权/预算/取消/host失败分类保留；不静默默认值/空成功。
- [ ] **AC-03｜owner_stage: work**：§6补充ID逐个有真实断言与结果；适用的调用次数/轨迹/owned账/关闭与恢复不是从主exit0推断。
- [ ] **AC-04｜owner_stage: work**：本包原case/check与L行映射有唯一owner/预期/状态；AV required行必须有actual，AV未知/未跑阻AV出口。非AV保持#ignore/NOT_RUN及后续owner，迁移数据不得丢，但不要求本轮actual、不阻AV出口。
- [ ] **AC-05｜owner_stage: work**：走共享受检前门与AV执行，legacy dispatch=0；无case名特判/第二parser/临时registry；跨仓依赖与其它路线未重复实现。
- [ ] **AC-06｜owner_stage: work**：受影响完整固定组及必要下游在冻结提交有有效证据；实际case/check数、排除/未跑/原红、准备/运行/清理时间分别记录，无新增未处理告警/调试输出。
- [ ] **AC-07｜owner_stage: review**：T-07独立逐AC核验源码/diff/实际消费者与规范增量，遗漏/延后/workaround具名处理，通过后status reviewed；不得仅复述自检收据。
- [ ] **AC-08｜owner_stage: merge**：真实落主后沉淀已审Spec/接口、归档并核来源身份；证据折Plan后材料/树guard与路径对账完成，未关闭named hold保owner/关闭条件。

## 8. 执行步骤

| task | owner_stage | depends | 实际操作/预期产物 | acceptance |
|---|---|---|---|---|
| T-01 | work | 无；本包所有已交依赖按身份复核 | 冻结旧/新实际输入、规范差异、精确诊断/资源边界；对新增表示/借用/调度/CT等未定机制先用现有入口做有界全链可行性探测，产出本Plan内决策与下一步，不预先删required。 FC：冻结本特性的canonical位置、判等条件和所有旧case/check/角色/消费者。 | AC-04、AC-05、AC-FC |
| T-02 | work | T-01 | auto-script/.as、#[script]/#[rust]模式、动态参数/属性/索引/调用、Python对象身份/装箱/数值/Unicode/NUL/关键字与嵌套实参组合。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | with resource as name的进入/退出、return/break/continue/普通异常时__exit__/close恰一次；Python异常进入可捕获语言错误，host撤权/resource/cancel分类不变。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 消费1026真实PySession与967资源账，不新增PyObject私有owner/第二解释器；普通无use.py程序不加载CPython，外部可选依赖缺失明确NOT_RUN而非PASS。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-SCRIPT。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态（T-01 已完成；T-02..06 阻塞于硬依赖未交付，见 §10）：

- [x] **T-01** work：已完成（2026-11-20）。输入指纹复核＋漂移来源、7 份旧文档哈希核验、
  16 份有界探测（正控绿；use.py/with/动态分派/`.to(str)`/`\0` 精确 stage·code 冻结）、
  可行性结论（无新运行时地基，缺口=四条源面/接线）、FC canonical 位置与判等条件、
  Rust 内联角色枚举、四份规范基线差异——全部见 §4 T-01 冻结记录。
  探测原始件：`D:/autostack/plan-artifacts/plan-1094/probes/`（probe-results.txt `8a39c905f8ee7df2`）。
  AC-04/AC-05/AC-FC 的 T-01 份额就绪；AC 的最终判定仍待 T-02..06 实施后真实运行。
- [x] **T-02** work：已完成（2026-11-21）。普通 AV py lane 实施——
  `use.py` 前门普通面接入（多成员按源序形）、动态参数/属性/索引/调用、
  py 对象 `.len()`→`__len__` 协议分派（纯 Auto str/List 原生语义不变）、
  Unicode/NUL 文本通道（UTF-32-LE codepoint-wise 升级）。FC：canonical
  `language/script/python-context.case.ad` 唯一作者建立、legacy-map.tsv 添 7 行
  （owner=PLAN-1094，原 ID/check/oracle/SHA16 守恒）。证据见 §9 T-06 收据与 §4 修订。
- [x] **T-03** work：已完成（2026-11-21）。`with expr as name { }` 语句面
  （grammar `with-stmt`/`with-resource`/`with-tail`＋CST＋resolve＋lowering＋AV runtime：
  进入/退出、`__exit__`/close 恰一次、return/break/continue/普通异常传播、嵌套、
  enter 失败不调 `__exit__`）；Python 异常→可捕获语言错误（`python.{class}` 分类、
  `PyException {msg}` 渲染、原 span），host 撤权/resource/cancel 分类不变。FC/AV-ONLY 份额完成。
- [x] **T-04** work：已完成（2026-11-21）。消费 1026 真实 py-bridge provider（AV 侧
  foreign 闭包，非 abi lane；无 use.py 不加载 CPython，纯 Auto 源零 py 依赖）；
  缺解释器 NOT_RUN 而非 PASS；WITH-EXITS（break/嵌套/异常/enter 失败关闭计数）、
  SCRIPT-MODE（.as/auto-script 与 pragma 优先级）、PY-ERROR-OBS（error-kind/
  error-span 结构化断言）oracle 落地。
- [x] **T-05** work：已完成（2026-11-21）。§6 AutoDown 草案落实为唯一活跃文档作者源
  `crates/auto-lang/test/cases/language/script/python-context.case.ad`（11 Case：§6 五个
  ＋未捕获对照＋旧 7 例迁移＋WITH-EXITS break/嵌套），`language.suite.ad` 追加引用；
  旧 99_py_dispatch×2 / 99_script_err×5 按 FC 迁移表映射（原义务/预期/ID 守恒），
  无后端源码副本；正/负/故意红元控制逐一运行（AV 视图 522 例 0 FAIL）。
- [x] **T-06** work：已完成（2026-11-21）。冻结候选提交后执行 §6 固定组：
  `cargo t case-av-language-parity ace-source ace-syntax ace-av ace-abi ace-python`
  ——case-av-language-parity 522/522 PASS（12 条 `av.parity.1094.*` 全部绿，0.86–1.06s/条）、
  ace-syntax 159 PASS、ace-python 10 PASS（p1026 py-bridge 断言随 UTF-32-LE 通道升级同步后绿）。
  剩余红全部命中 1092/1093 继承红清单（预存基面红）：ace-source text_record_source×5
  （D-1055/D-1088）、ace-av×13、ace-abi source_foreign×28（1093 收据主检出基面实测同红）。
  本包引入回归 3 处已修复提交 44878805f（p981 Aggregates 重冻结＋py_bridge_1026 两断言同步）。
  IF-AV-PARITY-SCRIPT 接口与可复现摘要见 §5 修订。AC-01..06/AC-FC/AC-AV-ONLY 待 T-07 核。
- [x] **T-07** review：已完成（2026-11-21）。独立上下文逐 AC 核验源码→消费者→实际 oracle，
  详见 §9 final 复审。通过（status → reviewed）。
- [ ] **T-08** merge：见上表唯一任务合同。

下一执行入口：核主检出、本包 r5 和依赖正式来源，创建唯一专属树后从 T-01 开始。依赖缺失先做可独立推进的输入审计。新发现的高风险未定语义先有界探测并作决定，超本包目标时显式修订或拆 Plan，保留全部旧义务。

## 9. 复审记录

- 2026-11-21 **merge 收据（T-08）**：
  `stage: merge | plan_id: PLAN-1094 | plan_revision: r5 | outcome: pass
  | delivery_commit: ce8ba4fbc | archive_commit: 待本簿记提交
  | worktree: D:/autostack/.wt/lang-1094/auto-lang | branch: plan-1094-dev（已并入，将删除）
  | next: done`。

  | Checkpoint | 证据 |
  |---|---|
  | prepared | reviewed baseline c8ec8e3bc（T-07 复审后 master 顶）＋canonical Spec 增量（SD-01..03、SD-FC，worktree 提交 12a3154a9）＋delivery commit ce8ba4fbc；T-06 门禁复用（自 run-_ixne_gm 后仅 docs 变动，tested 代码不变，按技能不变输入复用规则）。 |
  | landed | master tip = ce8ba4fbc（`git merge --ff-only plan-1094-dev`，无 merge commit）；11 个提交链 c921199f08..ce8ba4fbc 全部落位；rebase 重放安全证明：master 侧新增 PLAN-1048 系列后 rebase，逐提交 +/- 行内容等价（仅 hunk 行号漂移），44878805f→6b222d3e2 等 patch body 逐字节相等。 |
  | canonical_verified | master 上 `docs/specs/auto-av/language-parity.md`（§PLAN-1094 增量 379 行起）、`python-interop.md`（§PLAN-1094 追加）、`null-family.md`（§普通 AV 脚本/Python）、`feature-organized-corpus.md`（§PLAN-1094 script/Python 家族）四份增量落位；canonical case `language/script/python-context.case.ad` 与 suite 单引用均在 master。 |
  | archived | 本文件 git mv 至 `docs/plans/archive/1094-av-parity-script-and-python-protocols.md`，frontmatter `status: archived`、`current_step: 8`；簿记随本 commit 落 master（untracked docs/ = 未归档，已避免）。 |
  | cleaned | guard clean 后移除 worktree/分支/组目录；plan-artifacts/plan-1094 折证据后整目录删除；registry 刷新（见 §9 末尾）。 |

  **Merge 后门禁**：delivery commit ce8ba4fbc 与 T-06 冻结基线（44878805f＋b3e64105c）
  的 tested 代码逐字节一致（spec 提交只动 docs），case-av-language-parity 522/522、
  ace-syntax 159、ace-python 10、ace-lowering 9 证据有效；剩余红（ace-source×5、
  ace-av×13、ace-abi×28）为 1092/1093 继承红清单预存基面红（D-1055/D-1088、
  1093 收据同清单），非本包引入。
  批回归到期检查：`docs/plans/.last-batch-regression.json` 于归档后按 PLAN-898
  规程评估（1094 %5=4 非到期；移交 regress 技能按需）。

- 2026-11-21 **final 独立复审（T-07）**：
  `stage: review | review_scope: final | plan_id: PLAN-1094 | plan_revision: r5
  | reviewed_commit: b3e64105c（分支顶；44878805f 修复＋b3e64105c 换行） | base: master b9351b37c
  | worktree: D:/autostack/.wt/lang-1094/auto-lang | branch: plan-1094-dev | outcome: pass | next: merge`。

  | AC | 结果 | 核验 |
  |---|---|---|
  | AC-AV-ONLY | pass | 公共 suite 522 例只选 av 检查（case-av-language-parity 全 PASS）；非 AV 检查（ac-native/legacy-reference）经 `#ignore` 或未选呈 SKIP，不 prepare、不 dispatch、不计 PASS、不阻 AV；本包对非 AV 零准备/零结果依赖。 |
  | AC-FC | pass | 唯一活跃作者 `language/script/python-context.case.ad`（12 Case ID 全唯一，suite 单引用）；legacy-map 7 行 12 列格式全对、owner=PLAN-1094、status=mapped、原 ID/check/oracle/SHA16 守恒（与路线图 §PLAN-1094 清单逐一相符）；旧 7 文档保具名冻结历史；V-FC 元控制：错 oracle 必红、零选中必红的既有机制沿用（T-05 验证）。 |
  | AC-01 | pass | 12 条 canonical case 普通 AV 真实执行全 PASS（0.86–1.06s/条）：nested-python-call 42、python-error-catch 42（handled_language_errors=1 证明 handler 恰一次）、context-manager-close 42（buf.closed）、python-unicode 42（中文+NUL 往返）、py-dispatch-dual-len（stdout 6\n3）、native-len-unchanged 42（纯 Auto .len 原生）、error-message-oracle（PyException ValueError 原文）、with-error-message、py-object-not-callable（TypeError 可捕获）、with-break-exits-once 42、with-nested-exits 42；双流按 oracle 精确断言。 |
  | AC-02 | pass | python-error-uncaught 为运行终态语言错误：diag run/lang.error、`python.5` 分类、`error-span "45..55"` 与源字节位置实测一致（[45,55)，第 3 行）；parent 侧 ac_av.rs:2659-2680 对 error-kind/error-span 做精确匹配判断；NOT_RUN：`py-unavailable` 标记＋缺 descriptor → Status::NotRun（"never PASS"），ac_av.rs:2027-2047。 |
  | AC-03 | pass | WITH-EXITS：with-break-exits-once（break 内 `__exit__` 恰一次，buf.closed=42 出口）、with-nested-exits（LIFO 双关闭）、with-error-message（异常传播仍退出）、probe `plan1094_py_with_enter_failure_no_exit`（enter 失败不调 `__exit__`）；PY-ERROR-OBS：两条 §6 错误用例作正控，父侧断言对 `python.5`/span 敏感（错类别/偏移 span 必 FAIL——ac_av.rs 判断逻辑直接读 oracle）；SCRIPT-MODE：.as/auto-script 模式、#[script]/#[rust] 优先级、无 use.py 纯 Auto 零 py 加载（native-len-unchanged 无 foreign 闭包）。 |
  | AC-04 | pass | 7 行 L 行映射（99_py_dispatch×2、99_script_err×5）唯一 owner=PLAN-1094、原 SHA16 六位全对、status=mapped；AV required 行均有 actual（PASS）；非 AV 保持 #ignore/未选与后续 owner（不丢义务、不阻出口）。 |
  | AC-05 | pass | 走共享 `compile_language_source_with_foreign`（front_door 注入 helper/hoisted 成员 import）→ verify_foreign_closures → ABC → `execute_console_foreign`；`with` 走 resolve_with_stmt 全链（manager 单求值、`__enter__` 绑定、保护体 loop、`__exit__` 恰一次含控制流保存/恢复、truthy 抑制）；无 case 名特判（grep 无命中）、无第二 parser、无临时 registry；`.len()` py 对象经 py_ops 事实层分派，纯 Auto 值原生不变。 |
  | AC-06 | pass | 冻结提交后固定组：case-av-language-parity 522 PASS、ace-syntax 159 PASS、ace-python 10 PASS、ace-lowering 9 PASS；剩余红（ace-source×5 text_record_source、ace-av×13、ace-abi×28 source_foreign）与 1092 收据继承红清单（D-1055/D-1088）及 1093 收据"主检出基面实测同红"逐字一致，预存基面红非本包引入；本包 3 处回归修复（44878805f）＋legacy-map 换行修正（b3e64105c）后 `--check` 干净；worker/av-core/foreign_source 零 println（仅 prepare-av-providers bin 的过程日志）；无新增未处理警告。 |
  | AC-07 | pass | 本复审独立逐 AC 追源码→消费者→oracle（上表证据均为本次核验所得，非复述自检）；遗漏/延后/workaround 扫描：无未批准缩减；具名说明两项卫生注记（见下）。 |
  | AC-08 | merge | 未执行（T-08）。 |

  卫生注记（具名，非债务）：① `.c-build-parity1093_c/parity1093_c.obj`（696B）为 1093 C provider canonize 镜像中的真实编译产物，随 23a95edd3 入库（<5MB 钩子不触发、gitignore 无 `*.obj`）——保留为 canonize 完整性，不另开清理任务；② toolchains.json 整文件重排（新增 av-providers 工具，其余语义等价已程序核验）。规范增量（SD-01..03/SD-FC）按 §5 约定在 merge（T-08）时沉淀，本次 review 未提前写入 specs。

- 2026-11-21 **T-02..T-06 work 完成收据**（goal「完成计划1094」恢复后实施，计划 r5）：
  `stage: work | plan_id: PLAN-1094 | plan_revision: r5 | outcome: pass（T-02..T-06
  完成；T-07 review 待核） | worktree: D:/autostack/.wt/lang-1094/auto-lang
  | branch: plan-1094-dev | base: a2249c685（fast-forward 后 merge master 52ffc3c23）
  | code_commits: c5d9f3527（py lane）→ d964d07be（with 语句）→ d27684d69（py probe
  串行锁）→ a57f06c18（`.len()`→`__len__`、error-kind/error-span oracle、NOT_RUN）
  → fd557273c（canonical 语料＋legacy-map 7 行）→ 2a25936a3（WITH-EXITS break/嵌套）
  → 23a95edd3（av-providers dir_env 修复＋parity1093_c canonize）→ ee7b2a0d2（trim
  debug probe）→ 44878805f（p981 Aggregates 重冻结＋py_bridge_1026 断言同步）
  | task_ids: T-02..T-06 | evidence: §4 T-01 冻结记录＋§8 任务状态＋本节 T-06 门禁
  | next: review`。
  实施要点：普通 AV py lane 走 1093 prepared-provider foreign 闭包（kind rust/provider py，
  非 abi lane，无 use.py 不加载 CPython）；with 语句面全链（grammar→CST→resolve→lowering→
  AV runtime）；py 文本通道升级为 UTF-32-LE codepoint-wise（§6 python-unicode 需要 Unicode/
  NUL 往返，1026 冻结测试 word oracle 同步更新——与 1093 重冻结 p981 grammar id 同属
  "通道/语法扩后同步冻结契约测试"惯例，非改语义收据）。FC：唯一活跃作者
  `language/script/python-context.case.ad`（11 Case），legacy-map.tsv 7 行 owner=PLAN-1094，
  原 SHA16/ID/check/oracle 守恒；旧 7 文档保具名冻结历史。
  **T-06 门禁**（`--prepare-only`＋`cargo t`，prepared source_state
  `13573e1c39a173014c74b78867c67b18cfd8a0598bb1d67442b8275bcc1d0e42`，结果
  `target/test-groups/run-_ixne_gm`，wall 213.7s）：
  - case-av-language-parity：**522/522 PASS**（12 条 `av.parity.1094.*` 全绿：
    nested-python-call 42、python-error-catch 42（handled_language_errors=1）、
    python-error-uncaught（diag run/lang.error、python.5、span 45..55）、
    context-manager-close 42、python-unicode 42、py-dispatch-dual-len（6/3 双流）、
    native-len-unchanged 42、error-message-oracle（PyException ValueError 原文）、
    with-error-message、py-object-not-callable（TypeError 可捕获）、with-break-exits-once 42、
    with-nested-exits 42；0.86–1.06s/条；av-worker sha256
    `49e6d0c55aa7a16f6b…34557ab`、avworker/1.0、av.closures/1、mir-lowering 9）。
  - ace-syntax 159 PASS；ace-python 10 PASS（p1026 py-bridge math/torch 断言已同步新通道）。
  - 剩余红全部预存基面红（1092/1093 收据同清单）：ace-source text_record_source×5
    （1092 §9 清单 452-456 行，D-1055/D-1088）、ace-av×13（1092 清单 458-470 行）、
    ace-abi source_foreign×28（1093 收据"主检出基面实测同红"）——非本包引入，保留原
    owner 与失败计数，不阻 AV 出口。
  - 本包引入回归修复：p981 Aggregates 面重冻结（前 ea0f50bc…→96383c1a…，13 邻居
    逐字节不变）、py_bridge_1026 两条 word oracle 更新为 UTF-32-LE codepoint 语义——
    提交 44878805f，单测与全组复验绿。
  - 健康：无新增未处理警告（ace-core 3 条 dead_code 为既有）；无调试输出（trim 完成）；
    工作树 clean @ 44878805f。
  - 待交接口 `IF-AV-PARITY-SCRIPT`：普通源 auto-script/.as＋py-bridge provider 闭包
    （kind rust/provider py）→ 同一普通 AV 链（`ac-av/language-source-run-v1`，
    avworker/1.0，av.closures/1，mir-lowering 9）→ 真实值/双流/`language_error`
    （`python.{class}`、`PyException {msg}`）/`source_span`。绑定：12 条 canonical
    case（ID 族 `av.parity.1094.*`）＋legacy-map 7 行（owner=PLAN-1094）＋工具/AV-worker
    身份（见上）＋真实逐 case 结果。1110 按该来源接 A2PY（§5 交接表）。

- 2026-11-20 goal round 50（**用户裁定①，goal 暂停收尾**）：`stage: goal | plan_id: PLAN-1094
  | plan_revision: r5 | round: 50 | from: user | to: paused | status: executing | outcome: waiting
  | worktree: D:/autostack/.wt/lang-1094/auto-lang（保留，clean @ 5a76167cc）| branch: plan-1094-dev
  | commit: c6544ee5e | next: user`。用户裁定 §10 解锁路径 **①**：按路线图顺序以独立 goal 完成
  依赖链（1083 并行会话推进中 → 1084→…→1093 各自独立 goal），链后以新 goal「完成计划1094」
  从 T-02 无损恢复。裁定即时生效并记录于 §10；本会话 goal 随裁定暂停，不做后续轮询/写入；
  恢复点不变（§4 T-01 冻结记录＋§9 收据链＋worktree/分支/plan-artifacts/登记）。

- 2026-11-20 goal round 4（waiting×3 → **goal blocked**）：`stage: goal | plan_id: PLAN-1094
  | plan_revision: r5 | round: 4 | from: user | to: blocked | status: executing | outcome: blocked
  | worktree: D:/autostack/.wt/lang-1094/auto-lang（保留，clean @ 5a76167cc）| branch: plan-1094-dev
  | commit: 7ce32d13e | next: user`。三连等待后按协议终止轮询：两次 AskUserQuestion＋两轮运行时转达
  均未获裁定；依赖链侧无变化（1084/1087/1092/1093 仍 drafting 0/N；1083 仍 executing step 1）。
  **阻塞问题**（=§10 条目）：T-02..T-06 消费 1084/1087/1092/1093 已交付能力，未施工，并入本包属
  范围变更需用户批准。**解锁**：①按路线图顺序以独立 goal 完成依赖链（1083 进行中）后，以新 goal
  恢复本包 T-02（§4 T-01 冻结记录＋§9 收据链＋worktree/分支/plan-artifacts 全部保留可无损恢复）；
  或②用户显式修订本包解除硬依赖（同步路线图与四个活跃作者合同）。计划保持 executing/1/8，
  本会话不再产生新写入。

- 2026-11-20 goal round 3（waiting×2）：`stage: goal | plan_id: PLAN-1094 | plan_revision: r5 | round: 3
  | from: user | to: user | status: executing | outcome: waiting | worktree: 保留（clean @ 5a76167cc）
  | commit: 4438d4933 | next: user`。重读 §10/依赖链无变化（1083 仍 executing current_step 1，
  无新 master 交付）；按运行时指示再次 AskUserQuestion 请求裁定（①按序完成依赖链/②修订解除硬依赖/
  ③标记 blocked）——仍未获回答。连续等待计数 2/3；下一轮若仍无裁定/交付变化，按协议以 §10 阻塞问题
  标记 goal blocked 并保留全部状态。

- 2026-11-20 goal round 2（waiting）：`stage: goal | plan_id: PLAN-1094 | plan_revision: r5 | round: 2
  | from: work(blocked) | to: user | status: executing | outcome: waiting | worktree: 保留（clean @ 5a76167cc）
  | commit: 9938058d0 | next: user`。重读 §10 无新裁定；依赖链侧观测：PLAN-1083 已 executing
  （其 T-01 提交 6e0337167，T-02 施工中：ast.rs/cst/project.rs/auto-source.at 已改）——用户以并行
  goal 会话按路线图顺序推进依赖链（路径①的行为信号），但 1084/1087/1092/1093 仍未开工，
  本包 T-02..T-06 无可解锁工作。按 waiting 协议本轮不执行阶段工作、不跑检查；裁定请求仍开于 §10。

- 2026-11-20 T-01 work（goal 完成计划1094 round 1，按 r5 执行；同日 r4/r5 修订单独落）：
  `stage: work | plan_id: PLAN-1094 | plan_revision: r5 | outcome: blocked（T-01 完成=1/8；
  T-02..T-06 阻塞硬依赖） | worktree: D:/autostack/.wt/lang-1094/auto-lang | branch: plan-1094-dev
  | base_commit: 5a76167cc | code_commit: 无代码改动（T-01 为审计/探测，产物在 plan-artifacts）
  | dependency_revisions: 1078=archived；1084/1087/1092/1093=drafting 0/N；1107=archived（新输入源）
  | task_ids: T-01 | evidence: §4 T-01 冻结记录＋probes/probe-results.txt（8a39c905f8ee7df2，
  16 探测源；正控绿，其余按形态冻结 stage/code） | blockers: 见 §10 依赖阻塞条目
  | next_responsible_role: 用户裁定推进路径（按序完成依赖链或修订解除硬依赖）`。
  并发说明：本日另一会话对本计划做 r4/r5 修订单纯文档交接（1110 Python 错误切片消费），
  与本 T-01 记录不同记录类型、无行冲突；实施合同沿 r3/r5 未变。

- 2026-11-20 r5：静态复查补齐 AV 的捕获/未捕获类型和位置证据，具名 `PY-ERROR-OBS`、AC-01..03 的观察路径及 T-06/T-08 交接。依赖方向、AV-only 阶段门与 0/8 进度沿原计划。

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

## 10. 待澄清事项

- **[2026-11-20·阻塞] 硬依赖未交付，T-02..T-06 无法开工**（goal「完成计划1094」round 1）：
  T-01（不依赖硬依赖的输入审计）已完成；T-02..T-06 按合同消费 1084（动态值/对象键）、
  1087（跨调用错误展开）、1092（#[with] 共享语句面）、1093（受检 FFI 接面）的已交付能力，
  四计划均 drafting 0/N，传导依赖 1083/1085/1086/1088–1091 亦未施工
  （路线图顺序下一批为 1083/AVP-05；观察到 `D:/autostack/.wt/lang-1083/auto-lang` 工作树已由另一会话建立）。
  已问询用户未获回答，按 §8 默认路径（"依赖缺失先做不依赖它的输入审计"）执行 T-01 后停。
  解锁动作二选一：①按路线图顺序完成 1083→…→1093 后恢复本包 T-02（多 goal 会话推进）；
  ②用户显式修订本包解除/缩减硬依赖（改变范围与验收阈值，需同步路线图与 1084/1087/1092/1093 活跃作者合同）。
  本包保持 executing；worktree/分支/plan-artifacts scope 保留待恢复。
  **[2026-11-20 用户裁定①，本条已决]**：选择路径①——按路线图顺序以独立 goal 完成依赖链
  （1083→1084→1085→1086→1087→1088→1089→1090→1091→1092→1093），链后以新 goal「完成计划1094」
  从 T-02 恢复；本包合同/验收/硬依赖不变，路径②作废。

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
