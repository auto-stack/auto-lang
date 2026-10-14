# PLAN-776 最终独立复审 R6（2026-10-14）

**outcome: needs_fix。** R5旧例已修，但完整身份/分类不变量未闭合，新增stream Future门有普通数字误驱动回归。AC08仍缺JSON/bus SSE生成wire oracle，不能给final pass。

## 基线与审查范围

- review_scope: final；reviewer_identity/limitations: 当前Codex会话独立于修复实施会话，延续R5审查上下文；无子agent/额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；reviewed_commit: `5bdca42e49483a79c8a0b69c4758b24986b8238b`。
- original base: `5e21561d2c303a02319ff7a29e4607933e09a38f`；repair diff base: `73d4f0ff3a89f507c4192e38304354688ea00b52`。
- worktree_path: `D:/autostack/.wt/lang-776/auto-lang`；branch: plan-776-dev；实施树clean。
- dependency_revisions: sibling auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- spec_inputs: [input-hashes.json](../attachments/776/review-r6/input-hashes.json)；reviewed_spec_delta: SHA256=`AF212A9F09D8FE8DCC54A02605DCF96F1FC49457B1CE9C52A0498559E0B01A94`，[冻结副本](../attachments/776/review-r6/reviewed-spec-delta.md)。
- 从主检出交接即识别出生成JSON/bus SSE必需oracle仍未完成。本轮按review技能的就绪审计规则进行针对性复审，重放原反例并检查新生产点/分类/Future门；不再运行另一整轮t/tv/th来重复确认已承认的缺口。实施交接的5306/162/127等总数仅为work声明，不能作本轮final pass证据。

## Findings

### F776-R6-01 / P1：生产点包装仍依赖所在函数声明，旧int/helper路径继续丢SSE来源

影响AC02/03/08，T01/02/03/07/08，SD01/03/04/07；位置`vm/codegen.rs:11525-11540`。

包装从return位移至CREATE_GENERATOR后，但仍有 `fn_ret_declares_stream(current_fn_ret_type)` 条件。相同generator在`~Iter`函数内才包装，在旧`int`函数内仍是裸iterator id。这不是任意新能力：Plan §0/§1/§5.1要求旧int源码兼容及helper/local传播；R5要求完整不变量而非仅原例。

实际真TCP、同一`events() ~Iter<int> { yield 42 }`：

| case | code | expected | actual |
|---|---|---|---|
| V6-S0..S3 | R5 direct/local/helper/Internal（流声明） | SSE data:42 | 全部SSE data:42，旧例关闭 |
| V6-S4 | 构造generator后返回普通0 | JSON0 | JSON0，负控制保持 |
| V6-S7 | `fn p() int { let stream=events(); return stream }` | 真实来源SSE data:42 | 200 application/json，body0 |
| V6-S8 | 流声明handler返回`legacy_helper()`；helper声明int，存generator再return | SSE data:42 | 200 application/json，body0 |

证据：[probe.rs](../attachments/776/review-r6/probe.rs)、[reproduce.ps1](../attachments/776/review-r6/reproduce.ps1)，同目录`legacy-int.log`/`helper-int.log`及旧控制日志。修producer身份授予与非HTTP兼容，使声明仅用于校验/能力选择；不能恢复表命中猜测，也不能在SD里默删旧int/helper路径。

### F776-R6-02 / P2：alias目标抹去Future实参，合法九层链被静默判Json

影响AC07/08，T01/06/07/08，SD02/06/07；位置`api/mod.rs:148/240-251`、`api/contract.rs:78-83`，back_proxy独立深度8展开也需同源核对。

实际parser+ApiExtractor正例重放：R5限定名/限定alias/两层alias现在均File，用户同名控制仍Json。新组合：

- V6-T4：`type Download = Future<FileResponse>; fn p() Download ...`，解析成功，但ApiType.alias_of成为`"Future"`，return classification=Json，应展开Future目标为File（与直接Future<FileResponse>规则一致）。信息在bare_type_name中已经丢失，单修contract无法恢复。
- V6-T5：A0=FileResponse、A1=A0…A8=A7（九个alias），解析成功，A8判Json；合法同源链应File。常量8是本轮实施新增限制，Plan revision1没有该限制的授权；用实际visited集合拒绝循环，不得把非循环深链降成JSON。

证据`identity.log`保存types/endpoints。边界是分类真实解析来源，不声称本轮执行了生成错误产物。修复须保留目标结构/包装信息、循环诊断与三消费者一致性，不只增加旧两层例。

### F776-R6-03 / P1：扩大stream Future门后，普通数字命中活跃Future被执行并改值

影响AC05（显式Future位模式普通数值守卫）、AC02/03/04/09，T03/04/07/08，SD02/03；位置`vm/ffi/http_server.rs:5719-5737`。

V6-F1真实handler：`fn p() ~Iter<int> { let active=~{42}; return 496 }`。496=0x1F0，是普通整数字面量，与该独立进程活跃fid1编码相同。期望200 JSON496；实际200 JSON42——新`handler_async || handler_declares_stream_return`门把数字作为Future，执行了未await的body。V6-F0对照：相同body使用`fn p() int`，实际正确JSON496。

证据`future-number.log`、`future-number-int.log`；输入和运行入口在probe。修复diff明确新增stream分支，原stream旧门没有该通道，这是修复引入的新回归。声明/低字节/注册表三者仍不是逐值Future来源；必须保留真实Future返回正例与活跃同值普通数字负例，不能只测不存在fid的任意数字。

### F776-R6-04 / P2：生成File/Upload证据补齐，但JSON/bus SSE必需wire仍未闭合

影响AC08/09，T07/08，SD07；继承R5-F03剩余缺口。交接§9明确“JSON-record与bus SSE仍生成级验证”，verification AC08也保留边界。生成级签名/字符串断言不能替代实编/真TCP；Plan revision仍1，未见用户批准删除该oracle。

本轮实际重跑plan729 File和plan730 Upload两项生成wire矩阵 **2/2通过**（43.194s+16.139s），确认hyper-util harness修复有效，撤销旧“生成编译漂移归742”的笼统归因。但这两项通过不证明JSON/bus SSE。补现行支持面可运行fixture/oracle；确受742前置影响则明确未满足或经用户批准修改合同，勿把“776不接管742”解释为删除776验收。

## R5闭合对账与AC结果

| R5 finding | R6 closure |
|---|---|
| R5-01 | 旧流声明S0/S1/S2/S3/S5/S6及for-in54族绿；完整不变量partial（R6-01），新增Future门回归R6-03 |
| R5-02 | 旧T1/T2/T3关闭；完整来源partial（R6-02） |
| R5-03 | File/Upload生成wire关闭；JSON/bus SSE仍partial（R6-04） |
| R5-04 | 关闭：原dead_code侧道移除、unused_doc_comments消失；stream声明门恢复有实际调用，触面格式干净 |

| AC | verdict | mapping/evidence |
|---|---|---|
| AC01 | pass（现有碰撞范围） | T02/03/07，54族普通资源同值数字/负控制绿；Future专门要求见AC05 |
| AC02 | fail | T01/02/07，V6-S7/S8与Future数字改值 |
| AC03 | fail | T03/07，合法传播流失真；旧Internal正例已闭合 |
| AC04 | partial | T04，原失败/Nil/取消控制54族保持；新错误Future驱动通道待修，不能泛化所有路径正确 |
| AC05 | fail | T02/04/07，owner/id边界原例保持，V6-F1违反明确数字守卫 |
| AC06 | pass（既有覆盖） | T05，54族存活/常量/1000闭环保持；不推导尚未闭合传播面 |
| AC07 | fail | T06，V6-T4/T5 |
| AC08 | partial | T06/07，File/Upload实编wire通过，其余生成wire欠缺且来源/传播有实际缺陷 |
| AC09 | fail | T08，SD和收据未覆盖完整不变量/新增限制/普通数字误驱动 |

reopened_tasks=[T01,T02,T03,T04,T06,T07,T08]；T05保留，current_step=1/8。历史通过子项不抹去；不能以重置轮次/增加同类旧例/降低期望实现闭合。

## 本轮actual evidence与next

- `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 -j1`：54/54，28.686s。
- `cargo nextest run -p auto-man --lib --features test-http-e2e http_e2e_plan7 -j1`：生成File/Upload两例2/2，59.339s；无timeout。
- `cargo build -p auto-lang --lib`：成功，406总warnings；R5新增警告已消失。库hash/HEAD见`baseline.txt`；全部probe成功执行不等于期望通过（日志表中明确失败）。
- 修复触面9个Rust文件`rustfmt --check --edition2021 --config skip_children=true`，`git diff --check 73d4f0ff3..HEAD`：通过。
- t/tv/th全档本轮未运行：交接承认必需oracle未完成，已按技能就绪审计返回work；不声称复用work总数作final验收。tf/merge-owned与无触面tt/tu/taa也未跑。
- 结论前HEAD、实施树clean、dependency clean复核；本轮只写主检出docs/plans/**。

next: work，同一lang-776树修复上述四项、补必需oracle并提交；重新交接完整不变量→漏测原因→设计/路径/状态调整→实际结果闭合表，然后独立复审。Spec/ledger发布、归档、wt-guard、到期批量回归仍为merge-owned，待最终pass。本轮未修实现/合入/归档。
