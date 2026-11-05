---
plan_id: PLAN-950
status: drafting
feature_name: ACE 有限公共函数模块与受检绑定前置
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-syntax/source-frontdoor.md, docs/specs/auto-semantics/source-query-session.md]
new_spec_components: [docs/specs/auto-syntax/finite-source-modules.md]
touched_goals: [GOAL-002]
affects: [auto-syntax, auto-semantics, auto-a2x, auto-av]
depends_on: [PLAN-929, PLAN-934, PLAN-943]
current_step: 0
total_steps: 7
---

# [PLAN-950] ACE 有限公共函数模块与受检绑定前置

## 0. 变更摘要

为944补齐有限公共函数模块前置：统一CST识别公共函数与显式导入，按真实模块/可见性绑定，经现有source-query、resolve、Checked HIR进入AV与HIR→Rust；在现有store恢复后首编辑仍能正确绑定/失效。正式交付D950后944重新执行T-01，不以本包替代944的标准操作proof、装配身份或逐运行授权验收。

## 1. 目标

支持显式宿主供给的有限命名模块图、顶层i32/bool自由函数、`pub fn`、显式导入和限定调用，保留用户同名函数遮蔽。两个真实consumer分别对独立42/7 oracle执行。模块/导出/依赖事实可版本化持久化，首编辑正确，失败候选不替换旧可用程序。

非目标：任意包/目录发现、包管理器、全语言模块系统、重导出/通配导入/循环模块、泛型模块、公共type/方法/资源/文本stdlib实现、完整CAP、默认切换、外仓写入。既有九profile与不含新能力的旧输入保持原支持/拒绝语义。943的i64数学合同不窄化；944及945..948标准库交付仍由原包承担。

## 2. 架构方案

沿Design54 §5/7/11及Design51的既有分层，不创建新编译器、全局registry、平行store或类型分析：显式模块输入→统一`auto-source.at`/CST→携带可见性/导入事实的AST→模块绑定→既有source-query事实及唯一resolve/typecheck→既有HIR验证→AV/shared MIR与A2X HIR→Rust。绑定改变AST符号身份而非字符串替换源码；输入与诊断保留模块源身份和原字节span。生产禁走legacy parser。模块绑定层只能给事实/引用，不铸Checked凭证。

## 3. 技术栈

现有ACE独立Rust workspace、共享语法规则、ace-core、ace-source-query、ace-artifacts、ace-av、ace-a2x-core/rust；原Rust执行设施与固定`cargo t`职责组。实现位置由T-01核定最小无环依赖，拟新增`core/src/source/modules.rs`与`source-query/src/modules.rs`为候选，未声称已有。

## 4. 需求分析与背景调查

授权：用户在944问题中明确选择“先建并完成前置计划，再继续944（推荐）”，授权本有限语言前置的新建、实施、独立复审、合并/清理，随后继续944；无需重复请求同范围施工确认。目标仍为完成944。未指定模型/预算/agent，不虚构限制；独立复审按仓库AGENTS在同一树的独立上下文执行。

来源：`docs/specs/overview.md`、`docs/specs/auto-syntax/source-frontdoor.md`、`docs/specs/auto-semantics/source-query-session.md`、`docs/specs/auto-a2x/incremental-and-cache.md`、Design54及944 §9四例前探。初查master=`6e2abe9fe2faf14e83ed2707e26a9e183d9dfa2c`；943 delivered=`72ac9e7b`。944原实施树仍在`3b09e32b0`且clean；不能拿它作950实施树。

已核真实文件：`experimental/ace/grammars/auto-source.at`、`crates/core/src/source/{ast.rs,resolve.rs,cst/frontdoor.rs,cst/project.rs,decl_env.rs}`、`crates/source-query/src/{consumer.rs,session.rs,host.rs,restore.rs,store.rs}`（后两组路径均相对`experimental/ace/`）。当前fn-item只接fn，Program/FnItem没有模块/可见性事实；944前探普通函数Checked成功，pub/use/限定调用各为source.unsupported。这是缺能力证据，不是公共模块或消费者执行PASS。

编号949已被`949-vm-handler-async-parking-and-store-api-await.md`及其实施树占用；本次核实后将滞后的.next-id从949推进950，独占分配标记内执行`scripts/new-plan.sh ace-finite-source-modules`得到950/.next-id=951，不覆盖949。主检出其他文档WIP不属于本包，不混入提交。

## 5. 详细设计

首期语义合同：

- 新能力显式选择且纳入语法/producer版本，不把scalar旧profile全局静默扩展。以`ac.source.modules.scalar/v1`为拟议能力字面量，T-01核与实例能力的共存/拒绝规则；无模块能力时旧拒绝不变。
- 每个输入为稳定逻辑module ID与独立源文本；宿主供有限完整集合，拒重复ID、缺失依赖、循环，不能搜索磁盘兜底。ID与绝对路径分离；同内容异根不改逻辑身份。
- 顶层`pub fn answer() int { return 42 }`导出，未pub函数仅本模块可用；`use std.answer`导入模块std的公共answer；限定`std.answer()`绑定同一公共函数。首期不支持星号、别名、重导出或嵌套模块声明；这些形态具名拒绝。导入在函数声明之前，重复导入拒绝。
- 用户本地同名自由函数优先于裸导入；显式限定调用仍取指定模块。局部值遮蔽沿现有resolve规则，不绕过非可调用错误。私有函数同模块可调、跨模块不可调；未知模块、未知导出、私有访问、重复函数/导入、错签名调用在后端执行前诊断，未准入dispatch=0。两个不同模块的同名private helper必须保持不同真实符号身份。
- 模块绑定只遍历受支持AST并形成稳定符号/引用，内部可采用唯一限定owner；不拼接源码，不删除pub token，不复制旧parser，不把私有函数人为导出。绑定策略参与版本；原span、源身份、签名/mode保持，跨模块错误不借用入口模块的错误坐标。
- query/store沿原owner、签名/body事实与正/负依赖。模块导出集/可见性/所读签名进入依赖；body改动按真实读取规则失效，不能把所有module内容塞每个fn键导致无关重产。provider/host/ABI装配轴归944；本包仅模块绑定与源事实。
- 恢复使用现有ManagedStore的required边与版本化无地址数据；核module/source/grammar/capability/binding/schema及源事实身份后消费。首编辑沿恢复绑定继续，不全模块冷编译冒恢复；合法恢复后单体编辑不得重产无关函数。错误/取消/发布失败保持旧CURRENT及可执行目标，下一次真实恢复读回验证持久状态。授权不写入序列化数据。

T-01 bounded闭环：仅两模块、三个函数（std public answer=42、private helper、用户answer=7）在合法和私有访问负控下核真实API、绑定与两个consumer。冻结缺失接口/持久化布局/能力组合后才T-02..05。若设计不可行，走有界修订，不能删除任一AC；无需用户重新确认普通实现选择。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-syntax/source-frontdoor.md | 旧能力不变；新增显式有限模块能力与统一CST入口/诊断 | public/import为真实语法事实 | AC-01/02 |
| SD-02 | add | docs/specs/auto-syntax/finite-source-modules.md | 新增有限模块输入、可见性、绑定、版本与边界 | 944前置合同 | AC-01..04 |
| SD-03 | modify | docs/specs/auto-semantics/source-query-session.md | 模块绑定/导出依赖与恢复首编辑、失败保旧 | 沿原store，不造第二体系 | AC-03/04 |

## 6. 测试设计

以下为待实现/待执行用例，owner=work/T-02..05，非已通过证据。真实调用入口由T-01定位并在下表补符号，不添加专用“自报通过”runner。旧profile差分保原样本/条件，资源预算沿原入口；未执行/条件排除/环境红分别记账。

| Case | AC/invariant | 输入/状态 | 入口/观察oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/02 | std.pub answer=42，entry本地answer=7，显式/裸调用；含同名private helper | 新模块入口→既有AV与A2X Rust真编译运行；分别独立断言42/7，不只端间相等 | i32/bool显式模块 |
| V-02 | AC-01/02 | 私有跨模块与同模块合法对照、未知模块/导出、重复ID/导入/函数、循环、错bool/int参数 | 同一前门/绑定入口具名诊断，dispatch=0；同类型私有负控避免由类型错代证 | 声明与绑定门 |
| V-03 | AC-01 | 新语法不开能力；现有九profile语料；能力与实例组合 | 原拒绝/AST/诊断保持；新模块能力正例实际到Checked，无legacy fallback | T-01冻结能力组合 |
| V-04 | AC-03 | 重开store后首编辑std.answer body 42→43；entry本地answer不变 | 两consumer新结果43与7；真实parse/resolve/typecheck/lower/print计数按原依赖核，无关fn生产0；下次重开读回新状态 | 同签名单体body |
| V-05 | AC-03 | 改pub→private、被读签名或缺失导出→后来新增；未引用模块变化对照 | 正负依赖重新准入；不可见/类型错调用前拒，无关函数不重产；移除/新增恢复后仍正确 | 绑定/导出依赖 |
| V-06 | AC-03/04 | 合法恢复控制，篡改schema/module/source/binding/capability身份、重复声明或required边 | 使用前具名拒，未安装；没有冷fallback充恢复；旧目标字节/结果保持，下一次有效恢复成功 | 缓存不可信输入 |
| V-07 | AC-04 | 旧合法42后注入绑定/发布失败及取消候选 | 复用现有事务入口；CURRENT与旧bytes/executable仍42；新会话读取旧持久根仍42 | 发布前失败/原取消合同 |

职责组拟定完整并集：`cargo t ace-source ace-semantics ace-artifacts ace-av ace-a2x`，新永久用例进入这些组的现有包/target；若实际修改grammar-runtime才加`ace-syntax`，若共享MIR/lowering则加`ace-lowering ace-native`。T-01/T-05按实际diff确认，不能因依赖包构建自动扩大全实验套件；不得缩减必须consumer值腿。周期cargo t all归主检出due门。最终冻结并集由merge在独立review pass后核验，身份未变的实证可复用；发现/执行/条件排除数与准备/运行/清理/总时分列。独立review按仓库强制要求在同一唯一树、实施暂停后执行，不以自查替代。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 证据 |
|---|---|---|---|
| AC-01 | work | 显式模块能力经统一CST承载public/import/源身份；原profile不变，零legacy fallback | V-02/03及源码路径核验 |
| AC-02 | work | 真实namespace/可见性/遮蔽与签名绑定，AV和HIR→Rust分别执行42/7 | V-01/02，真实可执行值与拒绝前dispatch |
| AC-03 | work | 模块源事实/正负依赖入原query/store；恢复首编辑实际正确，无关fn零生产，坏身份使用前拒 | V-04..06，生产计数与下一次恢复 |
| AC-04 | work | 失败候选/发布保旧可执行与持久恢复；原预算/授权序列化边界保持 | V-06/07，旧字节/结果和原入口合同 |
| AC-05 | review | 独立逐AC/合法负控/遗漏延期/workaround、warning/fmt/debug及Spec delta审查，绑定revision/SHA | final independent pass |
| AC-06 | merge | 来源一致、canonical Specs落地、归档及树/branch/scope实际清理；D950可供944消费 | merge收据与guard/path核验 |

## 8. 执行步骤

唯一实施树拟`D:/autostack/.wt/lang-950/auto-lang`，branch=`plan-950-dev`；主检出保留Plan簿记，代码/测试/Spec编辑在该树。建组前核所有权，使用`scripts/new-wt-group.sh lang-950 --branch plan-950-dev`。scope=`D:/autostack/plan-artifacts/plan-950/`开工登记owner/用途/状态；常规target在树，外置仅scope；不建链接。950不得借用944树。D950正式落地后将master同步至944原唯一树，再重做944 T-01冻结。

- [ ] **T-01｜owner_stage: work｜AC-01/02**：以§5两模块闭环核实际CST/AST/source-query→AV/A2X接口，冻结模块能力版本、最小模型/持久化布局与实例组合规则；补下表真实entrypoint/来源身份，不以纯schema成功交付。
- [ ] **T-02｜owner_stage: work｜AC-01**：扩`auto-source.at`与core CST投影/AST，保旧门与诊断；补public/import/限定调用原span及V-02/03永久用例。
- [ ] **T-03｜owner_stage: work｜AC-02**：有限module binder接source-query/唯一resolve，两个真实consumer实际执行V-01/02；拒绝未知/私有/重复/错签名，保遮蔽。
- [ ] **T-04｜owner_stage: work｜AC-03/04**：沿source-query/artifacts恢复与原事务接模块身份、正负依赖与首编辑；V-04..07永久反例和合法对照，失败后真实读回。
- [ ] **T-05｜owner_stage: work｜AC-01..04**：核实际生产计数/边界与所有新用例接固定组，完成受影响完整职责组和健康检查，提交冻结源码与拟议Spec delta。
- [ ] **T-06｜owner_stage: review｜AC-05**：同一唯一树独立final review，逐AC与遗漏/债务候选核验，结论绑定revision/SHA；pass才reviewed。
- [ ] **T-07｜owner_stage: merge｜AC-06**：冻结职责组并集、ff来源核验、落Spec/归档、证据摘要折入Plan；逐命令所有输出根对账、刷新retention、guard clean后删除树/branch/scope。实际交付后回944。

T-01依赖已交付929/934/943，T-02..05依赖T-01冻结；T-06在work AC全闭后、T-07在final pass后。不得勾选未做任务，count含7项全生命周期任务。正文命名是拟议接线，开工核符号，不手写HIR顶替source验收。

## 9. 复审记录

2026-11-06 new r1：`stage: new | plan_id: PLAN-950 | plan_revision: 1 | outcome: pass | next: work | authorization: 用户选择先建并完成前置再继续944 | worktree: intended D:/autostack/.wt/lang-950/auto-lang | branch: plan-950-dev | evidence: 944 §9四例前探及现有源事实核验 | task_ids: T-01..07 all pending`

本轮仅建计划/修订944依赖，未实现/编译/执行本包产品用例，未独立review/merge，无D950声明。goal宿主仍显示blocked，执行连续目标需用户`/goal resume`；不新建第二目标或把blocked伪装active。

## 10. 待澄清事项

无新增用户范围决定；用户已选独立前置路线。T-01负责普通接口/模型落点和能力组合冻结，若真实证据要求改变本合同公开支持/阈值/跨仓范围才另提决定。当前宿主blocked状态仅可由用户resume恢复；该状态不代表本包已交付或944已解完所有工程缺口。
