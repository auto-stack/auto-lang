# PLAN-806 最终复审 R3：needs_fix

2026-10-18（Asia/Shanghai） / stage: review / review_scope: final。
当前 Codex 独立检查会话，未参与实施，无额外 agent，不臆造模型/会话身份。

实施树 `D:/autostack/.wt/lang-806/auto-lang`，branch `codex/plan-806-dev`，
reviewed_commit=`e2fc6ebf38e576e1cfb9486fd72cf92f47c5738a`，
base_commit=`5d8834bcc9cb4bde66ca9b610259036b82642227`；
本轮修复差异对比前审 `a20fcf6fe9885102d0059c04c9ac1e0551a3e3dc`。
Plan revision 1，入口 execution_done 6/8；实施树 clean。
799/801 已合入；兄弟 auto-down 仍 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
[完整基线与配置/依赖/输入哈希](review-r3-baseline.json.txt)；
[五份Spec增量冻结](review-r3-spec-delta.patch)，SHA-256
`34d0b3c29970f4e3d256068b6e0552806d5336986c69f0a5d1f2a5cb37090975`。

## 旧 finding 闭合与本轮结论

| finding | 结论 | 独立证据 |
|---|---|---|
| F806-R2-01（原 msg 文法/注释/重复错误问题） | 原三例及声明文法层 CLOSED；新增端口的失败合同另见 R3-01 | 原12例消费者逐字重放，输出与R2历史旧臂12/12完全一致；原三例已入永久回归 |
| F806-R2-02（交付权威Spec漏包） | CLOSED | generated-ui-distribution改13成员，auto-ui-dialect依赖说明、Plan impact与SD-05同步 |
| F806-R2-03（三文件format） | CLOSED | 三文件及其余受影响轻源scoped rustfmt --check exit0 |

修复采用通用原始 token 探测，msg/model 算法仍在轻包，没有把语法搬回 Parser。
单源模型策略、默认/显式pilot区分、所有接线未改变。**不授最终pass**：新增通用
端口实际失败语义与其明确承诺不符，属于修复引入的接口合同问题；不是把已修旧例
重新判红，也不是增加语言文法范围。

## F806-R3-01（P2，T-01/T-02/T-05/T-06、AC-02/04）：新增端口失败时消费输入、丢失诊断

位置：`crates/auto-lang/src/parser.rs:730` 的 `port_peek_next_kind`；
`crates/auto-parser-api/src/context.rs:64–76` 的公开合同。

新合同明确承诺：词法失败返回None也零副作用；连续调用结果一致；随后的advance
正常执行错误记录。实际实现调用真实 `lexer.next()`，仅成功时回推 token，失败时
没有恢复 lexer 状态。`Lexer::char` 在非法转义/字符字面量失败前会推进输入。
测试只覆盖了Colon/CommentStart成功路径，没有测试这类失败。

对同一输入 `value '\q' : int`（字符字面量中的非法转义），通过公开ParserBridge：

| 操作 | 实际结果 |
|---|---|
| 不peek，直接advance | cur=EOF，errors=[UnknownEscapeSequence("\\q", offset6,len2)] |
| peek一次再advance | peek=None，cur=Ident("q")，**errors=[]** |
| 连续peek两次再advance | **None → Some(Ident)**，cur=Ident("q")，errors=[] |

`value 'ab' : int` 同样首peek=None、二次Some(Ident("b"))。未知反斜线
`value \ : int` 恰巧不推进失败字符，所以它的重复None对照正常，不能证明
所有词法失败均无状态变化。Colon、块注释、文档注释的正常探测对照通过。

复现源码 [review-r3-peek.rs](review-r3-peek.rs)，
[原始输出](review-r3-peek.log.txt)，包含零/一/二次探测对照及七组相邻输入。
直接编译到本实施树target，链接本轮定向测试的auto_lang库；无实现修改。

这是**公开端口/知识合同失配**。当前 msg 只探测一次，沿旧lexer失败消费规则运行，
其旧12例确已恢复；不能为使新“纯peek”承诺成立而不加区分地改变旧msg错误语义。
应统一端口的名称、精确定义、失败消费协议与调用实现，补会推进输入的词法失败
及重复调用/后续消费断言；既保留原msg文法/错误行为，又让通用接口如实可靠。
可在T-01授权的通用端口范围内修正，无需扩大语言语法或引入Parser/文法反调。

Spec复核：SD-01目前也称该端口“零副作用”，须随修复校准。
本次实际修改了auto-parser-api公开面，建议同步其项目规范中的端口语义、实现方
兼容性说明及Plan impact。当前该规范仍描述旧pilot位置/唯一依赖和12成员闭包，
与806最终事实有陈旧处；R2对此目标核查不足，本轮明确记录，不把旧文案伪装成
本次新代码回归，也不要求发布到main后再修。

## F806-R3-02（P3，新增测试health，建议同轮修正）

`r2_real_use_import_chain_uses_default_declarations`（plan806测试文件289起）
直接调用 `crate::build_dynamic_component`，却没有与该API相同的
`#[cfg(feature = "ui-iced")]` 条件。

`cargo test -p auto-lang --lib --no-default-features plan806 --no-run` exit101；
该新增测试在319/338两处产生E0425。全库这档还存在大量其他模块的既有无UI测试
编译问题，**不能把整个失败归因本计划**。本轮无UI库本身构建和12例解析均通过，
因此本项不单独决定needs_fix。局部给新增UI装配测试加对应feature门控即可；保留
不依赖UI的基础文法测试，不要求本计划清理其他模块的基线红。

[完整编译输出](review-r3-no-ui-test-build.log.txt)。

## 本轮验证与复用边界

| 检查 | 结果 | artifact |
|---|---|---|
| light consumer manifest test | 10/10 | review-r3-light.log.txt |
| ui-dialect-consumer manifest test | 48/48 | review-r3-driver.log.txt |
| cargo t plan806 | 14/14（含真实use/VM状态变更、被导入文件坏注解控制） | review-r3-plan806.log.txt |
| cargo t msg model dialect | 45/45 | review-r3-scoped.log.txt |
| R2原始12例消费者 | 当前输出与历史旧臂12/12逐字相同 | review-r3-r2-replay.log.txt；输入仍review-r2-probes.rs |
| 新Core端口失败/重复调用对照 | F806-R3-01，不满足新接口合同 | review-r3-peek.rs / review-r3-peek.log.txt |
| cargo check -p auto-lang | pass，411既有warnings | review-r3-check.log.txt |
| cargo build -p auto-lang --lib --no-default-features +12例消费者 | pass，features=[]，解析行为保持 | review-r3-no-ui-build.jsonl.txt / review-r3-no-ui-probes.log.txt |
| 七个受影响轻源/test文件rustfmt --check | pass | review-r3-fmt.log.txt |
| Git Bash ui-declarations-consumer/verify-deps.sh | pass，轻闭包无逆向依赖 | review-r3-deps.log.txt |
| 777 strict | pass，0违规/2既有grandfathered | review-r3-boundaries.log.txt |
| 无UI libtest定向编译 | exit101，区分既有红与新增2处 | review-r3-no-ui-test-build.log.txt |

测试没有整体t/tv/tt/tb/tu/taa/tf；没有docs_gen/merge/规范发布或实施修改。
R2真实792交付25/25仍是历史证据：交付driver、包/资源清单、所有manifest、锁、
777配置和测试档配置未变，可据其确认本轮未增加闭包或资源规则；**API与Parser源码
已变，不把R2的fresh/locked成功冒称新SHA的重编成功**。本轮发现原语开放项后先
退回work，不再重跑大型交付scratch；新冻结面最终验收需确认其影响后补/复用相应
验证。R2的四个生成消费测试未重跑：ui_gen/trans/消费断言均无改动，原R2证据仍可
定位，后续若修复触及生成消费需复验。

## AC与路由

| AC | 结果 | 理由 |
|---|---|---|
| AC-01 | pass（实现/闭包） | 完整算法轻包、单源AST、Core expr/type仍委托、闭包守卫通过 |
| AC-02 | partial | 旧语法/原反例/默认pilot已通过；新增通用端口合同与失败路径未闭合 |
| AC-03 | pass（现有验收面） | 默认/结构/import/注册/递归/VM定向回归保持，未宣称无限输入全覆盖 |
| AC-04 | partial | SD-05及format已修；SD-01的端口承诺须按R3-01校准，当前完整交付重编未授pass |
| AC-05 | fail | final needs_fix |
| AC-06 | pending（merge-owned） | 未合入/归档/清实施树 |

保留T-03/T-04，重开T-01/T-02/T-05/T-06，Plan `executing 2/8`。
下一步在原树统一修复端口失败合同与相应断言，建议一并加新增UI测试门控，提交后
受影响复审。R1只是就绪检查；R2与本R3是两次实际最终审查未通过，下一次送审前
应补“剩余不变量/此前漏检原因/适用状态路径/设计或oracle调整/当前结果”的简表，
尤其说明为何只测成功Colon与不推进的UnknownCharacter漏掉了会推进的字符错误。
不要用14/14或原三例绿替代该接口整体合同验证。

最终pass后冻结裸cargo t+tv及实际触面档、规范/ledger/归档/guard与批量到期判定
仍为merge-owned pending；reviewed不等于交付完成，本轮未授合入权限。

终核：[review-r3-final-check.json.txt](review-r3-final-check.json.txt)确认HEAD未漂移、
实施树clean、全部基线输入哈希一致；本轮Plan/风险记录差异git diff --check通过。
