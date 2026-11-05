---
plan_id: PLAN-950
status: reviewed
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
current_step: 6
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

- [x] **T-01｜owner_stage: work｜AC-01/02**：以§5两模块闭环核实际CST/AST/source-query→AV/A2X接口，冻结模块能力版本、最小模型/持久化布局与实例组合规则；补下表真实entrypoint/来源身份，不以纯schema成功交付。
- [x] **T-02｜owner_stage: work｜AC-01**：扩`auto-source.at`与core CST投影/AST，保旧门与诊断；补public/import/限定调用原span及V-02/03永久用例。
- [x] **T-03｜owner_stage: work｜AC-02**：有限module binder接source-query/唯一resolve，两个真实consumer实际执行V-01/02；拒绝未知/私有/重复/错签名，保遮蔽。
- [x] **T-04｜owner_stage: work｜AC-03/04**：沿source-query/artifacts恢复与原事务接模块身份、正负依赖与首编辑；V-04..07永久反例和合法对照，失败后真实读回。
- [x] **T-05｜owner_stage: work｜AC-01..04**：核实际生产计数/边界与所有新用例接固定组，完成受影响完整职责组和健康检查，提交冻结源码与拟议Spec delta。
- [x] **T-06｜owner_stage: review｜AC-05**：同一唯一树独立final review，逐AC与遗漏/债务候选核验，结论绑定revision/SHA；pass才reviewed。
- [ ] **T-07｜owner_stage: merge｜AC-06**：冻结职责组并集、ff来源核验、落Spec/归档、证据摘要折入Plan；逐命令所有输出根对账、刷新retention、guard clean后删除树/branch/scope。实际交付后回944。

T-01依赖已交付929/934/943，T-02..05依赖T-01冻结；T-06在work AC全闭后、T-07在final pass后。不得勾选未做任务，count含7项全生命周期任务。正文命名是拟议接线，开工核符号，不手写HIR顶替source验收。

## 9. 复审记录

2026-11-06 new r1：`stage: new | plan_id: PLAN-950 | plan_revision: 1 | outcome: pass | next: work | authorization: 用户选择先建并完成前置再继续944 | worktree: intended D:/autostack/.wt/lang-950/auto-lang | branch: plan-950-dev | evidence: 944 §9四例前探及现有源事实核验 | task_ids: T-01..07 all pending`

本轮仅建计划/修订944依赖，未实现/编译/执行本包产品用例，未独立review/merge，无D950声明。goal宿主仍显示blocked，执行连续目标需用户`/goal resume`；不新建第二目标或把blocked伪装active。

静态复核：950编号唯一、0–10章节及7未执行任务齐全、3个现有Spec目标存在，git diff --cached --check通过；文档提交`18ed4a9c6`（950新合同与944 r2）。首个预检使用.next-id严格等于951，因并发951合法取号/提交而失败；PowerShell未因该子进程失败中止后续文档提交，未把该次检查记PASS。已重核：951为独立a2r证书JSON修复计划，.next-id=952且已随其`1e85cb756`提交；950文件唯一，无编号冲突，不回退.next-id。调整为单调>=951及具体951归属检查通过，不影响语义合同。实施/产品验证仍未执行。

2026-11-06 work entry：goal恢复active，上一轮创建/提交950与修订944依赖属于progress。现已创建唯一`D:/autostack/.wt/lang-950/auto-lang`，`plan-950-dev`，base=27fb2cba5e28db66fd363efe176bb85f1287945e；只读auto-down=58ea33f699efa5cb1194473a2d45dcec51006842。主检出零代码WIP，其他docs WIP保留。scope已登记，无链接/第二输出根；T-01有界接线开始，未完成AC。

### 2026-11-06 work checkpoint：有限模块→事实→双consumer真实值腿

源码检查点 `dc9f63f4f6579596b4017f41e875fe351a57bb37`，唯一950树/branch不变、提交后clean，未落master。新增`core::source::modules::{parse_module_source,bind_modules,ModuleSyntax,BoundModules}`和`source_query::modules::compile_module_product`，显式能力`ac.source.modules.scalar/v1`，绑定版本`source-module-binding/1`。模块+实例组合具名拒绝，不隐式扩大模板域。共享grammar新增scalar-modules裁剪；pub wrapper、use-item、qualified-name为真实CST事实；ModuleSyntax保exports/imports，Fn AST投影复用原引擎。名称绑定在AST发生，私有函数按模块隔离，入口本地同名函数遮蔽裸导入而限定名仍取指定模块；未拼接源、未复制parser/类型/store。输入源独立保存，经不重叠虚拟span区间映射回原模块/字节坐标；发射器仅抽取既有边界检查以消费该map，原单源入口沿原hash和UTF-8边界行为。

真实接线：`compile_module_product`→`bind_modules`→原`bind_with_facts`（唯一query/resolve事实）→`emit_resolved_module_parts`→`verify_scalar`；之后两条原消费者分别为`ace_lowering::lower_checked`→ABC encode/verify→`ace_av_core::execute`与`SemanticView::from_checked`→`ace_a2x_rust::lower_program/assemble_file_set`→真实rustc→exe。外部前探的同一两模块：std public answer调用其private helper返回42、app本地answer返回7；限定`std.answer()`与本地`answer()`分别AV=42/Rust=42、AV=7/Rust=7，独立预期逐端断言。每次solver_runs=4、body_misses=4、diagnostic_verdicts=0，未走legacy诊断/类型fallback。该值腿不是完整T-03/T-05交付，永久消费者回归接线及恢复仍待完成。

实际验证（均从950树；日志在统一scope）：

- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-core --lib source::modules:: -- --nocapture` exit0：5执行/5通过/0忽略、19过滤；运行0.06s（准备/编译16.95s）。可见性/导入/原span/旧scalar拒绝、重复/缺失/私有/循环及同名private隔离有永久断言。`binding-test.log` SHA256=fb17359978f55254c7aa1af684daa435ce321acbae33e846f0d94b27612acec1。
- `cargo test ... -p ace-source-query --lib modules::tests -- --nocapture` exit0：1执行/1通过/0忽略、37过滤；运行0.01s、准备1.50s。生成Checked与第二次query事实复用断言，非恢复证据。日志`query-compile-test-r2.log` SHA256=bd04647ea17f2a5abd3827b695058a4087d275cb6008d6310c4e195c0b756517。
- `cargo test ... -p ace-core --test conformance source_cst_927::` exit0：10执行/10通过/0忽略、337过滤；运行1.91s、准备10.60s，含旧profile corpus/edge诊断差分、片段/预算。不是完整ace-source组PASS。`legacy-cst-regress.log` SHA256=e45c6a7a75d13581f169f863b900e497cd5733210ee8c37f76cacd1c2ed48ad3。
- `cargo run --manifest-path D:/autostack/plan-artifacts/plan-950/probe/Cargo.toml --target-dir D:/autostack/plan-artifacts/plan-950/target` exit0，两输入×两个consumer=4个真实值观察；准备37.48s，内部rustc/exe都exit0，输出准确value:42/value:7。`t01-consumers.log` SHA256=e292b3bc368de5354dbd16169d632af83857b06edbff7a61fe4b092001c29a98；probe源在scope/probe，文件集/exe在scope/consumer-output/{qualified,local}，均可删除，不构成发行/安装证据。
- `cargo check ... -p ace-core -p ace-source-query` exit0，无产品库warning，准备/检查21.52s；全部9个修改Rust文件rustfmt --check通过，git diff --check/暂存检查通过。source-query lib-test有既有tests.rs:895 unused_mut，未改该文件，后续完整门需继续具名记账。Git自动维护提示unreachable loose objects为仓库维护信息，未运行prune或删其他工作。

开发红保留：首版public可选前缀使不带pub的fn未进入正确恢复路径，3语法case仅1通过；改为独立public-fn wrapper后3/3，再补namespace成为5/5。初次bind check有format/hash API编译错误，修复为既有runtime_contract::sha256_hex后通过；初次query测试误用了不存在的计数字段，改为真实session.counters().solver_runs，第二次通过。这些编译错误/首红不算语言拒绝PASS。一次.at换行处理使git diff --check红，已按HEAD精确LF恢复后通过，未引入整文件换行重写。

T-01仍有持久化接口/最小布局冻结未闭，T-02/T-03有产品化/永久consumer覆盖未闭，T-04..07未执行；保持0/7及executing，不以值腿替代AC-03/04或final review。下一步复用已核`SyntaxArtifactStore::{put_grammar,put_tokens,put_cst,open_cst}`及`CstView::decl_subtree`、既有ManagedStore required边/S934事实页：冻结并接模块图恢复/首编辑，不新造平行store；原UnitCatalog::from_restored只接旧profile，不能假称直接支持新模块profile。需将模块元数据、源/CST身份与query事实绑定，测试真实坏缓存、失败保旧和下一次读回。未建立任何950之外的材料输出根。

### 2026-11-06 work checkpoint：共享store源图恢复与候选发布

提交`7f0abd10d450275a69499f143268324150bf3c04`，950唯一树/branch不变、clean，未合master。上一goal回合交付dc9f63f4f双consumer值腿和真实测试，属于progress；本回合继续实际代码/反例进展，不是verified wait或状态重述。主检出crates/experimental/scripts/test零WIP，其他docs WIP保留。

新接口：`source_query::modules::{ModuleStore,ModuleReceipt,ModuleRecovery}`，模块根`source-modules-root/1`、S950 SnapshotRecord载体；`ModuleReceipt`是宿主持有的artifact+SHA256，不从待信任store自导expected。根绑定entry/package/module、canonical source capabilities、grammar、binding版本、全源图hash及每模块源hash/CST ID/每函数事实页ID。只存声明性能力，不存宿主地址/可执行指针或逐运行authority。`prepare`只写不可变候选，`publish`按原ManagedStore先注册新根→再撤旧根，base/cancel失败不替换旧根。`SyntaxArtifactStore::put_cst_unpublished`复用原put_cst实现，只禁候选抢占旧cst当前根；原put_cst发布语义未变。新增`CstView::artifact_id`与`ManagedStore::required_targets`只读接缝，比较父receipt与实际打开的A03身份、父required集合。

恢复真实链：重开同一store→核宿主receipt/codec/schema/binding/grammar/能力/声明唯一性/required集合→按原source+grammar key读A01/A02/A03源页及声明子树→原core严格投影（无scanner/parser）→模块AST绑定→S934事实页codec和body key/shape/coverage深验→seed原SourceQuerySession。`ModuleRecovery`此时只给bound graph+事实session，不假称已恢复目标可执行体。AST与原模块/字节坐标映射均来自既有CST；初始parse与恢复project分开。新module parse接入既有SyntaxWork/WholeParse真实观察，恢复负控不能靠遗漏计数自证0。

开发反例揭露原bare S934持久地址会把临时facts.revision写入不可变payload，同key第二次请求产生key conflict。模块页现用**独立版本化地址recipe**`sq-module-page/1`：仍用原S934事实codec与同一ManagedStore，storage key附读过的signature/negative版本摘要，生命周期revision规范为0；原bare S934键/codec读写不重解释。合法第二候选、原函数事实复用不再发生假冲突；同时能区别读取合同变化。原bare S934调用方的重复发布revision冲突是具名旧路径债务候选（owner=source-query旧ManagedStoreSink::put；本包模块producer不再使用该地址recipe，不能拿模块绿宣称旧路径已修）。其后若需要统一迁移应保持旧读取合同并给独立反例，不用改旧schema的hash混过去。

验证（所有日志scope内，temporary store通过AUTO_PLAN_ARTIFACTS显式放scope并在测试drop自动清理）：

- 冻结候选`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-core -p ace-source-query --lib modules::` exit0：core5/5、source-query5/5，共10执行/0忽略，分别19/37过滤；准备14.23s、运行0.01+0.15s。`module-recovery-union.log` SHA256=80c594458b9da28727027d48b530b1289a5d577bfd15852f56da6eb60fa1425c。
- 最后一处显式required token读取后重跑`-p ace-source-query --lib modules::persistence::tests` exit0：4/4执行、0忽略、38过滤，准备1.80s/运行0.14s。`module-store-final.log` SHA256=3e2b2cdbe943d8cdb0a08bce8a55aaa49450b890cfeae9acc98935a881189e30。真实覆盖：冷2模块/3函数计数非零；重开恢复tokenized_ranges/parsed_functions为空、solver_runs=0，恢复所有body查询hit；更改源/无能力/坏digest拒；schema/binding/grammar/能力/重复module/重复fact/未知字段7种实际新payload（重算receipt并真写CAS）在恢复和publish前均拒；删父required边证实mutation>0后恢复/publish拒。
- 旧42根→准备43候选→取消拒→在真实SQLite roots表安装trigger使新根注册报错→断言旧根集合不变→撤trigger并drop/reopen→旧42源图恢复成功；之后成功发布43、再drop/reopen→43源图恢复成功。这里只证明源图/事实持久状态保旧/读回，**尚未证明旧可执行结果或双consumer恢复首编辑**。
- `cargo nextest run --manifest-path experimental/ace/Cargo.toml -p ace-syntax-artifacts --test conformance -E 'test(root_) | test(revision_zero)'` exit0，3/3执行、28未选择（非条件排除）；准备9.03s、运行0.464s。原failed-root-preserves-current、fresh-session-revision-zero、root/lease/GC lifecycle均PASS。`syntax-root-replay.log` SHA256=921ab2b20874e1dffc4d723f64d25da10f39c960bc8453c334eaba8d277ae938。
- 本次9个Rust文件rustfmt --check、git diff --check/暂存检查通过；产品库check通过。lib-test唯一warning仍为既有source-query/tests.rs:895 unused_mut，未修改该行。新syntax-artifacts依赖改变实际触面，最终完整并集须增加**ace-syntax**，不是因为包标题而扩；ace-artifacts/ace-semantics/ace-source原已在并集。未改共享MIR/lowering，不因此自动添加native/comptime全套。

首红保留：初次store check两个API类型错误（StoreQueryKey::new需要LayerSpec、bounded读取参数u64）未当语义反例；修正后check绿。首次4方向中的3个persist case为2绿1红，原因S934 revision冲突；引入上述新地址recipe后3/3，再补7种坏root成为4/4。未把首次红抹成全绿。

当前仍executing/0-of-7，未final review/merge/落Spec。T-01/T-02/T-03前探主体已有可用接线，但持久化首编辑布局/永久双consumer集成尚未全闭，不能提前勾选。**下一工作关键**：现`prepare`仅接冷解析单CST，恢复给声明子树；恢复后修改一个函数并重新发布尚不可用。必须保留/复用认证的未变声明来源（可沿既有UnitCatalog的mint/current span与CST源receipt），只解析被编辑函数，重绑定可见性/正负依赖，接原保留CheckedEnv/单体candidate/consumer片段缓存；补旧结果与新43结果、无关函数parse/resolve/typecheck/lower/print零生产以及下一次真实恢复读回。不可把`compile_bound_modules`整图resolve/emit重跑伪装首编辑，亦不可冷重编源码换取恢复。现Root/codec仍是本包未发布候选，布局若变化需显式换版本，保留上述证据适用边界。

### 2026-11-06 work checkpoint：恢复首编辑与双目标闭环

提交 `dc0741bf7d689f725e030cab03de7df34f1707fc`，950唯一树 clean，实施仍未合master。此记录取代上一检查点的“尚无首编辑/目标恢复”现况判断，保留其历史证据边界。T-01..04已完成，4/7；T-05完整职责组正在运行，T-06/07未执行，不声明execution_done。

模型冻结：`ModuleSyntax::pieces` 保存认证CST origin及声明mint/current区间，`edit_module_function`仅解析实际pub声明片段、迁移未变区间；模块根明确升级 `source-modules-root/2`，不重解释旧候选/1。持久声明环境逐参数/mode/结果/owner/span验证，恢复组装stub-body环境，不整图resolve。`ModuleSession::prepare`经原query事实及`prepare_module_candidate`→原FnCandidate单体事务；恢复后的首编辑只有目标函数实际parse/namespace/resolve/typecheck/plan/bind，复用未变声明、事实和目标产物。

真实目标为`ModuleAvTarget`及`ModuleA2xTarget`：前者沿现有ABC envelope、FunctionCode admission/ProgramSnapshot link；后者沿现有A2xSession/provider/fragment缓存与文件装配。目标根包含source-set及现有producer/print/runtime身份，source父根保护目标receipt/digest与required闭包。所有候选先准备，核同一base后发布；旧source的目标receipt不可重绑新source。取消及真实SQLite roots INSERT trigger错误保持旧代码句柄、文件字节和持久源根，下一次重开真实执行仍为旧结果。

永久双consumer用例接入`a2x-worker/tests/cases/module_sources_950.rs`（Windows x64）及原conformance target：同一模块输入在AV与真实rustc/exe各断言42或7；drop/reopen恢复0 scanner/parser/solver/目标lower/print；首编辑42→43只有目标lower1/print1，AV未变Arc与Rust未变AXR ID保持；成功发布后再次drop/reopen，实际结果43或7。私有同类型替换、错误目标schema/print版本、取消、真实发布错误、旧目标重绑均为永久反例，不只检查wrapper计数。产品源码/测试/拟议3份Spec已提交；未修改旧9profile支持面或任何全局测试条件。

实证（统一scope日志，全部exit0）：

- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-source-query --lib modules:: -- --nocapture`：8/8执行、0忽略、37过滤；准备3.15s/运行0.90s。`module-source-closure.log` SHA256=747d7289c41f0c5fa6b7ec452bc312fc0188d113653f85950722cf54bd9118ac。含被读签名改变只使真实reader重算、缺失导出后来新增、坏root身份/required边、变长首编辑123及随后43读回。
- `cargo test ... -p ace-a2x-worker --test conformance module_sources_950:: -- --nocapture`：2/2执行、0忽略、169过滤；准备7.26s/运行1.93s。`dual-consumer-identities.log` SHA256=5607df4a39231731feaefaa01878509af34fa6ff92da4c2fc7ff11e454b6a283；后续加入真实发布失败及source绑定反例的`dual-consumer-closure.log`仍2/2，准备5.61s/运行1.68s。
- `restored-first-edit.log`：source-query变长首编辑1/1，准备11.17s/运行0.21s，SHA256=71af6827bee4aef38d42aaf4c8a01bf298bfbd5b950272a5a1ccc7f95f66a0b7。
- 新改Rust文件rustfmt及git diff/暂存--check通过；新AV unused import已移除。既有warning：source-query/tests.rs895 unused_mut；AV closures_fn_session.rs613 unused assignment、803 dead mix_u64、owned_session.rs111/112未读字段。非本包引入，不将其描述为零warning。

开发首红保留：首个真首编辑把source能力误传runtime MIR造成mir.capability-unknown，修为读取已验证bundle.requires；下一次恢复用ABC_SCHEMA误作MIR owner导致link拒foreign owner，改为与冷/热相同MIR_SCHEMA，不绕过owner guard。其后双consumer2/2。S934旧地址债务候选仍由独立review核验，不以新模块地址绿宣称旧路径修复。

冻结完整并集 `cargo t ace-source ace-syntax ace-semantics ace-artifacts ace-av ace-a2x --out D:/autostack/plan-artifacts/plan-950/work-gate` 正在同树单实例执行，源提交如上。新增ace-syntax因真实syntax-artifacts修改；未改共享MIR/lowering，未自动扩大native/comptime。工具准备冷编译未终态，不能计完整组PASS。scope内外置probe/target/consumer-output和work-gate全部登记为merge可删除材料，无新增散落输出根。下一步核实际完整组子终态、补齐任何work-owned覆盖缺口，才交独立final review。

### 2026-11-06 work完成与独立final review交接

源码冻结`d935caa85b39185d5f101dbe3433a7577d561a37`，branch/tree仍唯一且clean。独立上下文review950在work预检时只读审计全部风险面、未给final verdict或写代码；其发现已由work补永久反例/修复，属于work准备，不虚报final review修复轮次。

| 发现 | 不变量/实际修复与反例 | 当前结果/边界 |
|---|---|---|
| F950-PRE-01 | 冷类型拒绝投影已有当前key/deps事实页PortableDiag结构节点，暖事实/candidate经bound.locate；片段parse加当前声明偏移；不重solve/legacy fallback | 永久bool/int错调用以accept_bool(true)合法对照，冷/暖均std且同原字节位置，parse偏移；关闭 |
| F950-PRE-02 | 新r-pub词法全局作用令旧scalar的fn pub/let pub拒绝；compiler token复用现有profile gate，r-pub只scalar-modules | 真实首红3/4→core5/5；原9profile corpus/edge10/10，grammar profiles5/5；关闭 |
| F950-PRE-03 | register新root后独立DELETE旧root失败会半安装；同ManagedStore新增事务内compare/liveness/insert/delete/commit | 真实INSERT及DELETE trigger各Err、严格仅old root；独立重开两个target真实执行42/7；关闭 |
| F950-PRE-04 | source候选必须源于目标旧source-set；PreparedModuleEdit私有base_source，两target生产前核对 | app.answer=8另一合法source会话的std43候选被旧7 target拒；lower/print0；关闭 |
| E950-PRE-01 | 每次失败之后须新会话实际执行旧目标，不只旧句柄/源根is_ok | cancellation/INSERT/DELETE各重开store+AV+Rust，真执行42或7，恢复lower/print0；关闭 |

控制验证：暂时去诊断节点映射→type-location-control exit101；去两source-base guard→两consumer case均FAIL/exit101；替换原子发布为仅注册新根→retirement-atomicity-control exit101。均try/finally逐字恢复，再跑最终合法闭环；控制未入提交。原诊断探针首版用保留字take作函数名，被语法拒绝，明确无效输入；改accept_bool后才作bool/int类型反例，未把该首红当产品类型证据。grammar declared_set首跑4/5，新增合法scalar-modules未同步名单导致红；补第11个精确期望（未删旧项）后5/5。

最终定向实证（exit0；scope内日志）：core module5/5、source-query module9/9，0 ignored，分别19/37过滤；准备4.10s前后，最终运行0.01+0.22s，`preflight-final-source.log` SHA256=a2f24a2e7657d7a8e722eb2dd0c3b59f10c40f5ffa8a6d3fbff0247668f05108。双consumer2/2、0 ignored169过滤，准备14.14s/运行2.26s，`preflight-consumer-repair.log` SHA256=1ee653463637e290524de7020b12c1301e8c936bc6368a700ae0e8e24d033f14。grammar profile5/5（26过滤）0.05s，`token-profile-conformance-final.log` SHA256=c398a6f57c6ff1583a3c0a454fc8619e1c58fb6f0f4a03a97d4bf676de9bd987。原CST conformance10/10（337过滤）1.93s，`legacy-profile-final.log` SHA256=1f8b2f9f58ffe040b8cf5a129ad07ce499df3c679b7f54f15d31bb6999e475c1。另grammar --lib实际0用例，仅编译成功，不充测试PASS。rustfmt/git diff/暂存check通过；既有具名warning如上，无新warning/debug。

早期完整并集门禁：dc0741基线ace-source完整370/370、准备17.356s/运行3.372s通过；ace-syntax已发现132项，在64MiB原压力运行中主动停止，因为work审计出现待修缺口（本机936另一个相同压力并发观察也保留），非timeout/不是PASS；ace-semantics/artifacts/av/a2x未执行。外层exit1及原results/日志保留。该旧source PASS不自动覆盖本次grammar/query/root修复；最终六组并集明确pending，由merge在final code/AC/Spec pass后核冻结基线，不减原样本/预算/条件。

Readiness：AC01..04均由永久合法/非法对照、真实双执行值及恢复/首编辑/身份/发布失败覆盖；work-owned开放缺陷、必需覆盖缺口、合同决定均为空。T01..05完成5/7；T06独立final review、T07合并门禁/沉淀/归档/清理未做。旧bare S934 revision重复发布地址冲突为本包之外旧recipe债务，在KNOWN-DEBT具名登记，不宣称已修。拟议Spec三路径已冻结提交；现模块能力仅i32/bool，不代替944标准操作/装配授权验收。主检出代码无WIP，其他会话docs保留。

`stage: work | plan_id: PLAN-950 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-950/auto-lang | branch: plan-950-dev | base_commit: 27fb2cba5e28db66fd363efe176bb85f1287945e | code_commit: d935caa85b39185d5f101dbe3433a7577d561a37 | dependency_revisions: auto-down=58ea33f699efa5cb1194473a2d45dcec51006842 | task_ids: T-01..05 complete; T-06/07 pending | evidence: scoped source/dual-consumer/profile logs above | blockers: none | next_responsible_role: independent final review950; frozen six-group gate merge-owned`

`stage: goal | plan_id: PLAN-950 | plan_revision: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-950/auto-lang | commit: d935caa85b39185d5f101dbe3433a7577d561a37 | next: review`

2026-11-06 review baseline同步：worktree已无冲突rebase至master `3eaaff58a`（包括已交付951证书JSON修复及Plan簿记）；4个950提交range-diff全部`=`：dc9f63f4f→d2e2aaace、7f0abd10d→2cfe77207、dc0741bf7→297edb844、d935caa85→8f595a9a35c839928c080c18e4ebb17cdc4f82a9。实际final review以该新HEAD、base=3eaaff58a为准，不用旧SHA覆盖新上下文。唯一树clean，implementation暂停；受影响新基线目标结果与最终固定组由review/merge核，纯rebase不伪称重新执行。

### 2026-11-06 independent final review r1

`stage: review | review_scope: final | reviewer_identity: independent review950 context (implementation未参与；未声称其他model身份) | plan_id: PLAN-950 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-950/auto-lang | branch: plan-950-dev | reviewed_commit: 8f595a9a35c839928c080c18e4ebb17cdc4f82a9 | base_commit: 3eaaff58a | dependency_revisions: auto-down=58ea33f699efa5cb1194473a2d45dcec51006842 | findings: no open in-contract findings | next: merge-owned frozen affected-group gate, then deposit/archive/cleanup`

已独立读取本技能/AGENTS、主Plan r1、完整新增模块绑定/编辑/存储/两目标实现、共享改动与三份Spec delta。初次准备审计不是final review round；本次为正式final review第1轮。冻结进入与结束HEAD相同、实施clean，同一唯一树无review checkout。四个原提交rebase映射与真实依赖核对；951新增case使worker当前未选171（旧日志169不混算）。

| AC → task | verdict | 实际路径/反例与边界 |
|---|---|---|
| AC-01 → T01/02/05 | pass | 统一auto-source.at scalar-modules CST及严格原AST投影；模块能力显式且instances组合拒绝；pub词法profile gate实核；旧fn pub/let pub合法对照、新语法无能力拒绝、原九profile corpus/edge重放通过；无legacy生产fallback |
| AC-02 → T01/03/05 | pass | AST限定身份、可见性/本地遮蔽/不同private helper独立符号；真实AV与HIR→Rust生成文件经rustc/exe各自42/7；bool/int错调用冷暖原std/span和合法bool对照；独立return/arity/非callable sibling反例保持std及非零源span，diagnostic_verdicts=0 |
| AC-03 → T04/05 | pass | 原A01/A02/A03、S934 codec及required闭包/外部SHA receipt；恢复CST重投影+声明env准入+facts seed，真实scanner/parser/solver零；变长首编辑只目标实际生产、未变AV Arc/AXR artifact复用，成功下次恢复43/7；错误schema/grammar/binding/source/caps/重复/required边真变异拒绝；source-base跨合法会话反控在lower/print前拒 |
| AC-04 → T04/05 | pass | 单写candidate先准备，原FnCandidate owner/base检查；ManagedStore replace_current_root同liveness锁及SQLite事务compare/insert/delete/commit，DELETE错误回滚；cancel/INSERT/DELETE三方向各独立新store+AV+Rust重开真实旧42/7且旧文件相同；未序列化执行地址/授权，未改原预算或测试条件 |
| AC-05 → T06 | pass | 全AC/Spec/负控及PRE共同机制闭合审查完成，无遗漏/未批准延后/冷编译冒恢复；旧bare S934冲突已具名D950-1，不将其冒充本包已修 |
| AC-06 → T07 | pending merge | 六固定组完整并集、canonical来源落地、归档、实际guard/path/retention清理仍待merge，不声称已交付D950 |

独立当前冻结重放（工作目录均唯一950树，临时材料显式scope，测试AUTO_PLAN_ARTIFACTS=scope；常规target在树）：

- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-core -p ace-source-query --lib modules:: -- --nocapture` exit0，core5/5、query9/9，ignored0，过滤19/37；准备4.57s，运行0.01+0.29s，整命令5.52s。`review-final-modules.log` SHA256=6fb185f66cf9bd10e0f62735a021599a6e24d8fb755a75cf05935fc39ec313e4。
- `cargo test ... -p ace-a2x-worker --test conformance module_sources_950:: -- --nocapture` exit0，2/2、ignored0、过滤171；准备9.03s/运行8.31s（含真实rustc/exe及各失败后新会话），`review-final-consumers.log` SHA256=53a6807d58da30b3b3d5ac74a5cb968326a0eccdb2b6c67a6d539afd722dfa6b。
- 独立`cargo run --manifest-path D:/autostack/plan-artifacts/plan-950/review-probe/Cargo.toml --target-dir D:/autostack/plan-artifacts/plan-950/target` exit0，1合法bool控制+3不同拒绝（return bool/int、缺参数、局部非callable），全部std/有效原span且legacy诊断计数0；准备34.60s，执行<0.1s。`review-independent-probe.log` SHA256=a3a7e4a5fd5afeb0f09341b6feabfde85d7c9a21e9936471371c9755ecd18d30。probe源码/lock/产物仅scope，merge全删。
- `cargo test ... -p ace-syntax-grammar --test conformance profiles_927:: -- --nocapture` exit0，5/5、ignored0、过滤26，准备0.83s/运行0.07s/整1.14s；`review-final-grammar.log` SHA256=7a7cdd5abefcbd6aeebf90fee6ac2f3337510a5a0f30e4ba0fe144a29257e064。
- `cargo test ... -p ace-core --test conformance source_cst_927:: -- --nocapture` exit0，10/10、ignored0、过滤337，准备2.60s/运行1.81s/整4.73s；`review-final-legacy.log` SHA256=af8baaa0c11e0963b38d8d683d60303688f7763e41a3ea93ba66c49e88cd1e00。

PRE控制原日志的实际断言已读：移除诊断投影使std断言失败；移除两base guard使预期Err收到Ok而两consumer失败；破坏原子replace使失败预期收到Ok。正式源码相应实现存在且当前上述定向均绿；未依赖自报banner。所有计数为实际执行，过滤不是条件SKIP；本次定向无条件排除，清理由fixture drop完成（未单独计时）。

健康：本包所有改动Rust文件的`rustfmt --check --config skip_children=true`通过，唯一例外为a2x-worker/tests/conformance.rs中951继承的cert_json_951模块排序；独立将base同文件保存scope后同样fmt红，950新增行没有新格式问题。首个递归fmt还命中未改旧子模块，保留review-final-fmt.log/scoped.log，不假称全仓零红；排除该继承文件的本包检查review-final-fmt-owned.log exit0，git diff --check exit0。编译只见已具名旧source-query/tests.rs895 unused_mut及AV closures_fn_session613/803、owned_session111/112 warning；未新增warning/debug输出。

Spec delta SD01..03通过，路径/当前规则/AC绑定准确；不扩大944标准操作/装配授权，不写legacy ledger或手工索引。冻结文本SHA256：source-frontdoor=578e98e68e2d88691310d80b199546c0d6b39c564191106a54cbaa758370861e，source-query-session=7d8239fdecc3fc82ebd8f01f785ca76aa869db9c7e4cad1782d2a1aaa96020f9，finite-source-modules=a5bcc32fa15162bfd77d9769fe8156e5157f3cb090f01966cb14ec9dee2b1806。三文件精确diff冻结scope/review-frozen-spec-delta.diff SHA256=0ac21c2a91dbacd1f623fae19340226f52782ac8308d1700e2726c375810db93；impact元数据保持两个modify、一个add、GOAL-002。

pending_closeout_gates：冻结`cargo t ace-source ace-syntax ace-semantics ace-artifacts ace-av ace-a2x`由merge单实例运行；旧syntax132发现后64MiB压力主动中止不是PASS，旧source370不自动覆盖新修复。review pass不允许未满足门禁时合入。清理还须精确核查旧C:/Users/zhaop/AppData/Local/Temp/ac787-budget-64m-45528、scope内失败fixture及本次review-probe，不碰936异pid目录；后续TEMP/TMP显式scope/runtime-temp。T06闭合6/7，T07待实际交付。

2026-11-06 merge prepared：已消费独立final pass，冻结review/source=`8f595a9a35c839928c080c18e4ebb17cdc4f82a9`。当前六组完整门禁在同树执行，scope/merge-gate，TEMP/TMP显式scope/runtime-temp；保持原样本/预算/平台/ignored条件。第一次启动exit2（旧中止work-gate留下active lease owner pid22124），核owner根确为950、PID已退出、目录无reparse后仅删该owner文件/空lease并重启；未删活动其他会话租约。原admission错误保留merge-gate-admission.log，不算产品红。早期测试默认Temp外散只确认为ac787-budget-64m-45528，merge最终逐路径核对/清除，尚未删除；runtime-temp及review材料仍在scope保护。修正旧work goal收据被簿记替换为reviewed的笔误，恢复该历史时点execution_done，本次reviewed仅由final receipt授予。

`stage: goal | plan_id: PLAN-950 | plan_revision: 1 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-950/auto-lang | commit: 8f595a9a35c839928c080c18e4ebb17cdc4f82a9 | next: merge`

## 10. 待澄清事项

无新增用户范围决定；用户已选独立前置路线。T-01负责普通接口/模型落点和能力组合冻结，若真实证据要求改变本合同公开支持/阈值/跨仓范围才另提决定。2026-11-06宿主已由用户恢复active；原状态阻塞解除。
