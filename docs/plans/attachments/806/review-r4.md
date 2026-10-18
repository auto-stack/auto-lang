# PLAN-806 最终复审 R4

2026-10-18（Asia/Shanghai） / stage: review / review_scope: final。
outcome: **pass**（代码、验收及Spec审查通过；merge-owned冻结门禁仍pending，尚未合入）。

## 冻结基线

当前独立 Codex 检查会话，未参与实施，无额外 agent，不臆造模型/会话身份。
实施树 `D:/autostack/.wt/lang-806/auto-lang`，branch `codex/plan-806-dev`，
reviewed_commit=`50644ad43c89f8ada86214a5f4ac2a7998b8b494`；
base_commit=`5d8834bcc9cb4bde66ca9b610259036b82642227`，
前审提交=`e2fc6ebf38e576e1cfb9486fd72cf92f47c5738a`，Plan revision **1**。
799/801已合入；兄弟 auto-down仍为`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
实施树入口clean；[基线与输入哈希](review-r4-baseline.json.txt)。
[六份实际Spec增量冻结](review-r4-spec-delta.patch)，SHA-256：
`d04e9f8b1457f1a1f0c94fa6492755c3418a7001b892630bede35091de45e2f1`。

## Finding闭合与诊断调整

| Finding | 本轮结论 | 证据 |
|---|---|---|
| F806-R1-01 | CLOSED（历史就绪缺口） | 实施树/已提交代码/consumer/收据齐备 |
| F806-R2-01 | CLOSED（原msg文法/重复副作用） | 原12案例重放保持，下面额外6组合也与旧实现一致 |
| F806-R2-02 | CLOSED | 权威交付闭包13成员、SD-05和impact同步；实际包数14=13受管+应用 |
| F806-R2-03 | CLOSED | 原三文件及其余受影响轻源scoped format通过 |
| F806-R3-01 | CLOSED | 原语/调用名为probe_next_kind，成功/失败语义分别定义，明确失败可消费前缀/丢弃错误/非幂等；与实际旧msg协议一致 |
| F806-R3-02（P3） | CLOSED（本计划新增诊断） | 真实import测试加ui-iced条件，无UI测试编译不再出现plan806文件诊断 |

已审阅t05-receipt的R3诊断简表：以前只用成功token和恰巧不推进的UnknownCharacter
证明“纯peek”，漏掉字符字面量推进型失败；本轮按成功/非推进失败/推进失败×单次/
重复调用×后续消费扩展oracle。该调整没有改变用户要求的旧msg语法或错误行为，
也没有以扩大文法/补记旧错误来迎合新接口承诺。

独立消费者[review-r4-probe.rs](review-r4-probe.rs)仅把上一轮探针的调用名换成
本轮端口名，重放非法转义、坏字符字面量、UnknownCharacter、Colon、CommentStart、
DocComment、Ident及零/一/二次探测对照。结果见[原始输出](review-r4-probe.log.txt)，
全部符合已明确的双路径合同。

[review-r4-grammar.rs](review-r4-grammar.rs)包含18案例：原12、3类推进失败与
named/positional组合、泛型/嵌套type/expr三个正控。当前、历史库、无UI库三方结果
18/18逐字相同，含AST、错误及span、docs、cur。
[机器比对](review-r4-probe-comparison.json.txt)、[当前输出](review-r4-grammar.log.txt)、
[旧臂输出](review-r4-historical.log.txt)、[无UI输出](review-r4-no-ui-probes.log.txt)。
历史库为R2已冻结的主检出预构建，不声称本轮重编了base SHA；旧规则同时按精确
base的原算法源码核对。当前与无UI库均在本实施树本轮构建/定向验证，身份有迹可循。

## 验证矩阵

| 检查 | 实际结果 | evidence |
|---|---|---|
| light consumer manifest test | 10/10 | review-r4-light.log.txt |
| ui-dialect-consumer manifest test | 48/48（6+12+8+12+4+3+3） | review-r4-driver-resumed.log.txt |
| cargo t plan806 | 18/18，含真实use文件/VM handler状态变化和被导入文件坏注解负控 | review-r4-plan806.log.txt |
| cargo t msg model dialect | 45/45 | review-r4-scoped-resumed.log.txt |
| quoted emit/multiarg/Rust msg/Ark注解四项生成消费 | 4/4，无金样变更 | review-r4-codegen.log.txt |
| cargo check -p auto-lang | pass，411既有warnings，无本计划新增警告 | review-r4-check-resumed.log.txt |
| 无UI库features=[]构建+18例实际声明解析 | pass，三方输出相同 | review-r4-no-ui-build.jsonl.txt / review-r4-no-ui-probes.log.txt |
| 无UI libtest定向编译 | 仍exit101（62处其他模块基线红），**plan806诊断0处**；不将整档失败归因本计划或改记全绿 | review-r4-no-ui-tests-resumed.log.txt |
| scoped rustfmt七文件 | exit0 | review-r4-fmt.log.txt |
| 轻闭包normal/build/dev | pass，无auto-lang/Man/VM/renderer逆向依赖 | review-r4-deps.log.txt |
| 777 strict | pass，0违规/2既有grandfathered | review-r4-boundaries.log.txt |
| 正式792交付 | **25/25，exit0**；fresh解析8s/A构建127s/B重定位locked构建116s，source身份/资源/缺输入/坏hash/失败保旧/恢复/sidecar/开发路由控制全过 | review-r4-delivery-repeat.log.txt / review-r4-delivery-receipt.json.txt |

本轮未运行裸t/tv/tt/tb/tu/taa/tf，也没有docs_gen。定向cargo t均有明确filter；
ui_gen消费仅选4个必需断言，不用历史越时序tu整档代替冻结合入门。

## 单源/知识与净删校正

完整msg/model/注解实现仍在auto-ui-dialect，normal依赖只有auto-parser-api轻层；
expr/type经端口委托，AST类型不复制。Parser两个inner只建桥委托，原注解方法已删；
顶层UiDialect、widget/store/component、默认import装配均通到同一算法。
pilot调用同一model字段循环的ExternalSubset，默认无裸字段、显式pilot保留裸字段。
Override/builtin/Append、fn/on护栏、None/Err恢复与旧公开入口保持。

新通用端口只试读原始token，不是parse_msg/parse_model反调；未暴露Parser/lexer/
TypeStore引用。成功时回推原token；失败保留旧词法器各失败模式的消费结果，
不能用它做“所有失败都无副作用”的纯peek。实现ParseContext的消费方需实现此
新增方法；仅实现ExternalDialect的现有调用方不需要实现宿主端口。

未迁UI族与组合宿主auto-lang→轻UI文法边如实保留，无遗漏延期冒充完成。
按最终`git diff --numstat base..HEAD`校正旧收据的行数口径：
Parser **205删/25增，净删180行**；pilot **65删/8增，净删57行**；新轻crate三文件
共420行。旧收据“parser −213/pilot −65”混用了diff总变化/删除数与净删，
历史保留，最终以本节区分迁位与净删的实测为准，不据行数夸大模块独立程度。

SD-01～06逐项核对：新包范围/两种model策略、生产接线/权限与恢复、剩余UI面、
交付13成员及新通用端口双路径定义均与源码一致。R4补齐本计划实际已修改的
`docs/specs/auto-parser-api/project.md`到supersedes_spec_components和SD-06；
这是review-owned影响导航补正，不改实现/AC语义，不增加plan_revision。
保留GOAL-003/007，未发布main canonical Specs或live ledger。

## 交付早退及有界复验记录

恢复“继续”时，两个旧工具会话已失效，未完成日志不计成功；保留原文件并将
续跑写入*-resumed日志。完整交付首个可观察轮次：CLI39s、fresh解析7s、A构建145s
成功，V02在2s早退、本地target未创建，driver exit1，原工具未保存该次stderr。
[首轮日志](review-r4-delivery-resumed.log.txt)、[退出码](review-r4-delivery-exit.json.txt)。

保持原B包现场，A/B锁文件hash完全相同，直接复放`cargo build --locked`已exit0、
136s完成，B内target/debug/uiminimal.exe在位。
[锁身份](review-r4-first-delivery-locks.json.txt)、[B完整诊断](review-r4-delivery-v02-diagnostic.log.txt)、
[B产物检查](review-r4-v02-replay.json.txt)。未给首次早退臆造确定性环境根因。

再作**一次**完整官方driver复验，独立scratch=review-r4-delivery-repeat。
[观察包装器](review-r4-delivery-trace.py)仅存每条原始命令的stdout/stderr/返回码，
不改命令、环境、timeout、返回值或oracle；driver本体不变。
[逐步原始诊断](review-r4-delivery-trace/)、[完整复验日志](review-r4-delivery-repeat.log.txt)。
首次红轮保留，不挑删/改记；最后完整复验**25/25、exit0**，每条命令有原始
stdout/stderr/退出码，V02真正在B内重编出本地可执行文件，所有余下控制完成。

两轮fresh解析的锁并非完全相同：唯一版本差异为redox_syscall 0.9.5→0.9.4；
不能宣称首轮早退在逐字相同的全部外部依赖上得到确定性归因。各轮内部A/B锁
相同，首轮原B也已在自己的原锁上复放成功；最终完整复验在第二轮同一锁下
fresh/重定位及所有控制通过。[首轮完整锁](review-r4-first-Cargo.lock.txt)、
[复验完整锁](review-r4-repeat-Cargo.lock.txt)、[精确差异](review-r4-fresh-lock-delta.patch)、
[锁身份](review-r4-repeat-locks.json.txt)。生产root Cargo.lock及本树源/配置未变，
不以这次外部fresh解析差异掩盖首轮缺少诊断，也不做未经验证的失败原因断言。

## 最终AC/状态/后置门

| AC | result | task / evidence |
|---|---|---|
| AC-01 | pass | T01/02/05，真轻算法、单源AST、无反调/逆向依赖，轻consumer10与闭包守卫 |
| AC-02 | pass | T01/02/04/05，18组独立新旧/无UI对照、18永久回归、48注册/恢复/pilot验收；失败协议已如实定义 |
| AC-03 | pass | T01/03/04/05，顶层/全部结构委托、真实import与VM状态变更、Core护栏、quoted/named等定向生成消费 |
| AC-04 | pass | T04/05/06，单源/净删及未迁清单、777、实际新SHA正式交付25控制、SD01～06与impact |
| AC-05 | pass | T07，同树独立最终验收及知识增量审查；原finding全部关闭，无新增阻断项 |
| AC-06 | pending（merge-owned） | T08，未合入/沉淀/归档/清实施树 |

Plan置**reviewed 7/8**，T-07勾选；next=merge阶段。裸cargo t+tv与实际触面档、
规范沉淀/ledger/归档/guard清理、批量回归到期判断均未执行，必须在冻结提交上
完成后才能合入/宣称交付。ui_gen/trans源未改，不因消费验证追加整档tu/tt；
后续若同步master/修复门禁改变代码、依赖、配置或输入，应先核对受影响复审和
证据有效性，不能将本pass自动覆盖新代码。

落盘终核：[final-check](review-r4-final-check.json.txt)确认HEAD未漂移、实施树clean，
实现/依赖/配置/规范输入/重放源码哈希均一致，正式收据25/25、报告链接无缺失。
主检出本轮Plan/风险记录的git diff --check通过。两轮大型交付scratch保存收据与
完整锁后，分别核定绝对路径、wt-guard clean再用原生操作移除；
[首轮清理](review-r4-scratch-first-cleanup.log.txt)、
[完整复验清理](review-r4-scratch-repeat-cleanup.log.txt)。806实施树和分支保持在位。
