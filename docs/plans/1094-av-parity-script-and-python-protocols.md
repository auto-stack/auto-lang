---
plan_id: PLAN-1094
status: executing
feature_name: AV脚本模式与Python对象协议兼容补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-20
plan_revision: 5
current_step: 1
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
- [ ] **T-02** work：见上表唯一任务合同。阻塞：硬依赖 1084（动态值）/1093 未交付（均 drafting 0/N）。
- [ ] **T-03** work：见上表唯一任务合同。阻塞：硬依赖 1087（错误/展开）、1092（#[with] 语句面）未交付。
- [ ] **T-04** work：见上表唯一任务合同。阻塞：随 T-02/T-03 前置（1084/1087/1092/1093）。
- [ ] **T-05** work：见上表唯一任务合同。
- [ ] **T-06** work：见上表唯一任务合同。
- [ ] **T-07** review：见上表唯一任务合同。
- [ ] **T-08** merge：见上表唯一任务合同。

下一执行入口：核主检出、本包 r5 和依赖正式来源，创建唯一专属树后从 T-01 开始。依赖缺失先做可独立推进的输入审计。新发现的高风险未定语义先有界探测并作决定，超本包目标时显式修订或拆 Plan，保留全部旧义务。

## 9. 复审记录

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

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
