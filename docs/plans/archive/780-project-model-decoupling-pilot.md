---
plan_id: PLAN-780
status: archived
feature_name: 轻项目模型与 pac.at 兼容性探针
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-15
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components: [docs/specs/auto-man/project.md]
new_spec_components: [docs/specs/project-model/project.md]
touched_goals: []
affects: [docs/specs/auto-man/project.md]
---

# [PLAN-780] 轻项目模型与 pac.at 兼容性探针

## 0. 变更摘要

验证轻project-model脱离compiler/VM/UI表达并读取静态工程声明，与当前执行型配置对拍后形成全面迁移决策。新增受控入口，不切换所有生产pac读取。

## 1. 目标

小模型crate独立构建；包/dep/member/render/展示等选定字段有DTO、错误/路径来源；静态旧新归一化结果相同，执行型/unknown明确分类，不截字段假成功。语言loader/UI注册表/man各字段owner与后续adapter明确。

非目标：马上摘整个man→lang、迁UI生成器/建仓、改dep声明门/锁格式、砍动态pac、重写Atom-v2、命令路由。

## 2. 架构方案

拟新 `crates/auto-project-model` 仅依赖auto-atom/auto-val与必要纯解析库（T01核定），不依赖lang/man/gen/UI。ProjectDescriptor/DependencyDecl/RenderSpec/source信息不含Builder/IndexStore/AutoVM。compiler adapter在上层，不把整个Pac直接搬成新包。

## 3. 技术栈

Rust、现有auto_atom::AtomParser；serde仅在实际协议需求下。对拍oracle：AutoConfig::from_code→Pac::new及真实pac输入。

## 4. 需求分析与背景调查

用户2026-10-13只授权本轮调查与报告/规划；实施需确认本Plan revision 1，不创建新仓、不发布/合入，不改其他在途Plan/WIP。

前置777。Specs auto-man/project的PLAN635声明门、四名称/opens规则、frontend/runtime、Design37；hash见reports/v06-modularization-baseline.jsonl。实锚man/pac.rs:12/196、automan/asset；lang/config.rs::AutoConfig::from_code、atom.rs::AtomReader::parse（执行VM）、auto-atom/parser.rs::AtomParser::parse（不支持变量/函数/循环/插值）；ui/app_registry.rs::parse_pac_fields/parse_pac_back_project；lib装载器dep/member文本读取。

## 5. 详细设计

T01盘所有受管pac.at，分类static/dynamic/unknown；涵盖comments、裸/引号dep、members、render single/split/multi、legacy backend、四名称/locale、back.project、未知fields与路径基点。新parser无法表达的项写unsupported-static，不catch失败后默默丢字段；需要执行的旧输入由显式adapter保持。

首轮冻结最小已证实static子集，所有其他形态仍有分类/限制记录；未支持的输入不得算对拍相等。新增 `test/fixtures/modularity/project-model/` 与man `src/project_model_adapter.rs`（拟新，显式probe不改默认）。新包表示source与未知字段策略，不含网络/物化/构建效果。

执行型pac去留、manifest严格化与static默认切换后续决定。当前所有生产入口保持。模板eval仍compiler adapter，不能因为model小包存在就声称auto-man不依赖语言。

### 规范增量

| delta_id | 操作 | docs/specs target | before/after | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/project-model/project.md（新） | 实际DTO/static支持面/错误/限制 | 独立消费 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-man/project.md | 旧读取不变；增加explicit pilot/分类/后续adapter归属 | 不假记已切换 | AC-02/03 |

## 6. 测试设计

实现者未指派；work自检后由独立上下文review在同一唯一worktree验收，不修代码；merge仅处理已验收沉淀/归档/清理。本草案没有派遣agent/chat。

T02后 `cargo check -p auto-project-model`+`cargo test -p auto-project-model`；T03拟建consumer gate `cargo test --manifest-path test/fixtures/modularity/project-model/consumer/Cargo.toml`、man新module filter `cargo test -p auto-man project_model`。man Rust变更需 `cargo check -p auto-lang -p auto-man`+裸`cargo t`；不碰VM/codegen则不tv/tu/tt/taa，不生成外仓book金样。

| V | AC/不变量 | 输入/入口（T03建） | oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/独立 | consumer只依赖model | parse/build通过，normal deps无lang/man/UI | auto-atom仍用val |
| V-02 | AC-02/static | dep裸/引号、comments、members、render多形态/backend/extras | 已支持子集旧新normalizedDTO相等；不支持明确分类 | 不是全pac语法 |
| V-03 | AC-02/dynamic | 真实或专门变量/调用/插值输入 | static拒/分类，explicit旧adapter保输出/错误，无silentfallback | 不砍旧能力 |
| V-04 | AC-03/策略 | 四名称/locale、back.project、declared/ghostdep | 当前规则/路径来源不变，effects不混模型 | 对照Spec |
| V-05 | AC-03/坏输入 | 缺括号、类型错误、重复冲突、源路径错误 | 明确stage/path诊断，无网络/写项目/build副作用 | extras≠坏类型 |

## 7. 验收标准

| ID | owner_stage | 可观察結果 |
|---|---|---|
| AC-01 | work | model normal deps无compiler/UI，external真通过；V01 |
| AC-02 | work | 全受管pac分类与冻结子集对拍，dynamic/unknown不假绿；V02/03 |
| AC-03 | work | path/展示/声明策略不变，读无effects；V04/05，后续owner明确 |
| AC-04 | review | 独立分类/兼容/consumer审计，bindcommit/revision |
| AC-05 | merge | SD01/02只沉淀实际pilot、归档/guard-clean清理 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 777+确认r1 | 受管pac/读点分类、static feasibility；冻结字段/normalizedoracle。**[✅ 2026-10-15]**：worktree lang-780@f3ca995e7；survey（149 文件：144 static / 5 dynamic 模板插值 / 0 unknown；dep 仅裸名形态；`${name}` 插值发生在 scaffold eval_template 拷贝层）+ 读点三类清单（VM 型 Pac::new/external_backend_dir、文本型 parse_pac_fields/parse_npm_deps/pac_declares_dep 等、dep_scanner 非 pac 面）；冻结子集与 oracle 定义见 `test/fixtures/modularity/project-model/survey/T01-REPORT.md`；T02 依赖核定=auto-atom+auto-val，AtomParser 复用于值层、文档层扫描器自持 | 02/03 |
| [x] | T-02 | work | T01 | 新model包/DTO/static入口，无compiler执行类型。**[✅ 2026-10-15]**：`crates/auto-project-model`（deps 仅 auto-atom/auto-val；文档级扫描器自持+值层对象/数组委托 AtomParser）；DTO 含 source/provenance/raw_props/raw_nodes/duplicates/unsupported；unsupported 分类不丢字段（插值/裸标识/赋值/调用/f-string/on-off 越界/单引号/dep 名不可表达/类型不匹配）；单测 21/21 | 01/02 |
| [x] | T-03 | work | T02 | 新fixtures/consumer/manadapter，V01–05，不切默认。**[✅ 2026-10-15]**：fixtures 15 例（static 10/dynamic 2/bad 3）+ consumer 独立 crate（manifest 即 V-01 证据，3/3）+ `auto-man/src/project_model_adapter.rs` 显式探针（5/5：corpus 149=**144 static 对拍相等**/4 dynamic 分类/1 clib template 双拒；V-03 旧臂保输出；V-04 路径基点与 external_backend_dir 对拍；V-05 stage/path 诊断+输入/目录零副作用）；生产读取零改动；语料实情补齐 `#` 注释/顶层对象文档/名字位插值三语法；发现 VM 对象键迭代序跨进程不稳定（对拍按结构比较） | 01/02/03 |
| [x] | T-04 | work | T03 | 明确man/loader/registry字段owner、后续消除文本副本接线决策。**[✅ 2026-10-15]**：`T04-OWNER-MAP.md`（26 字段行×当前读者/文本副本/目标 owner/接线注记；决策=auto-man 内部 vue.rs/rust_ui.rs 文本读先行替换（无新依赖边）、auto-lang registry/loader 声明门待依赖裁定立项、VM adapter 永久保留面（模板 eval_template/dynamic 分类文件）、npm_deps/targets 不入冻结 DTO；VM 对象键序不稳定注记） | 03 |
| [x] | T-05 | work | T04 | 包/consumer/man scoped+t门，证明旧production读取不变。**[✅ 2026-10-15]**：scoped 面=check -p auto-project-model ✓ / test -p auto-project-model **21/21** ✓ / consumer `--manifest-path` 门 **3/3** ✓（独立 manifest 即 AC-01 证据）/ `cargo test -p auto-man --lib project_model` **5/5** ✓（corpus 149=144 static 相等+4 dynamic+1 template 拒，P780 corpus 计数行）/ `cargo check -p auto-lang -p auto-man` exit 0 ✓；生产读取零改动=git diff master 对 pac.rs/vue.rs/rust_ui.rs/asset.rs/target.rs/auto-lang/src **空**，且 corpus 测试 149 次过旧臂 AutoConfig::from_code→Pac::new 生产路径；rustfmt 全量通过+复测三绿。**裸 `cargo t` 按 2026-10-14 时序裁定为 merge-owned 冻结提交门禁（pending）**；`cargo test -p auto-man`（非 --lib）单包构建因预存 plan734 tauri fixture 特性统一问题红（master 同红已验证，非本计划破坏，候选另立 L0） | 01/02/03 |
| [x] | T-06 | review | T05 | 独立review，不删动态输入假等价 | 04 |
| [x] | T-07 | merge | reviewpass | 真实Spec/归档/合流/guard-clean清理。[✅ 2026-10-15 merge r1]：SD-01/02 沉淀 ec934b000（ff-only 合入 master=tip）；裸 cargo t 三跑 5352/5353（旋转负载 flake 单跑全绿归因，证据见 merge 收据）；ledger v21→22 离线投影四条目；归档+guard-clean 清理完成；批回归 due 移交 regress | 05 |

拟tree `D:/autostack/.wt/lang-780/auto-lang`，branch `plan-780-dev`；构建/测试在该树，主检出只簿记。外仓代码不在本Plan改，cross-repo接线后续立项；禁止链接。

## 9. 复审记录

2026-10-13 stage=new，r1，outcome=pass（草案preflight）；next=确认后work。0/7任务；新包/fixture未创建运行，不是auto-man拆仓成功或独立实现复审pass。

2026-10-15：stage=work | PLAN-780 | r1 | outcome=pass（work-owned T-01..T-05 全部完成，交独立review） | worktree D:/autostack/.wt/lang-780/auto-lang | branch plan-780-dev | base 075a9b4ab（master tip at branch） | code 6aa07b630（f3ca995e7 T-01 survey → d7c7403e0 T-02 model crate → ac4e20b46 T-03 adapter/fixtures/consumer → b9738a272 T-04 owner map → 6aa07b630 rustfmt） | deps: auto-down 组内兄弟 detached@c39ac7f（只读，仅为 workspace path 依赖解析，未改） | tasks T-01..T-05=5/7（T-06 review、T-07 merge 待办） | evidence: ①model 单测 21/21；②consumer 独立门 3/3（manifest 仅 auto-project-model path 依赖=AC-01）；③adapter 探针 5/5——**corpus 对拍 149 受管 pac.at = 144 static 冻结字段逐项相等 / 4 dynamic 分类不假绿 / 1 clib 模板双臂拒（旧臂错误保留）**，fixture 边缘形态 10 例全等，V-03 旧臂仍解模板、V-04 路径基点与 external_backend_dir 对拍一致、V-05 stage/path 诊断+输入字节与目录零副作用；④生产读取零改动（六读点文件 diff 空；corpus 测试=旧臂生产路径 149 次绿）；⑤rustfmt 后三档复测全绿 | blockers: 无 | next=review（T-06：独立逐 AC 审计，重点=对拍 oracle 是否被弱化/unsupported 分类是否覆盖真实 dynamic 面/AC-01 依赖闭包证据/consumer+adapter 是否实际未接线任何生产读取）。**merge-owned pending 门禁**：裸 `cargo t`（2026-10-14 时序裁定，冻结提交上执行）+ SD-01/02 spec 沉淀 + tf 到期判定。实现要点偏离记录：①survey 后语料实情补齐三语法（`#` 行注释 19 文件、顶层 JSON 式对象文档 2 文件、节点名位插值 1 模板）——T-01 报告已勘误；②发现 **VM 对象字面量键迭代序跨进程不稳定**（三探针实测）——对拍对对象形态 repr 改结构化比较（排序键值对），语义字段（backend_shape/按键取值）不受影响，生产消费全部按键名读取无存量风险，已记 T-04 注记；③T-02 依赖核定落地=AtomParser 复用于对象/数组值层（`Atom::new` 拒顶层标量故标量本地解析），文档级扫描器自持；④adapter 测试跑 `--lib`（`cargo test -p auto-man` 全目标在单包构建下因**预存** plan734 tauri fixture 特性统一问题红，master 同红已验证，非本计划破坏）。

2026-10-15 R1：stage=review | review_scope=final | reviewer_identity=本评审会话（拆分规划/777/778审查参与，未实施780，从提交及独立组合重建） | plan_id=PLAN-780 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-780/auto-lang | branch=plan-780-dev | reviewed_commit=6aa07b630939966ac793c9f4cf7b07060330d736 | base_commit=075a9b4ab6cb4284f3c259b348f2571322107fec | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；root/consumer锁见指纹 | spec_inputs=auto-man/project SHA256 bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3；新project-model spec尚不存在，SD01/02 partial冻结 | acceptance_results=AC01 pass、AC02 fail、AC03 partial、AC04审计完成未pass、AC05 pending | findings=F780-R1-01(P1嵌套变量静态假绿)/02(P2 dep重复path取首)/03(P2未知子树与args丢失)/04(P1 I/O吞错且原corpus门假绿)/05(P2 root wrapper未unsupported)/06(P3 consumer格式收口) | evidence=[R1报告](attachments/780/review-r1/REVIEW.md)、[独立输入](attachments/780/review-r1/independent.rs)、[旧新结果](attachments/780/review-r1/independent.log)、[真实门禁反控](attachments/780/review-r1/gate-negative.json)、[指纹](attachments/780/review-r1/fingerprints.json)、[规范增量](attachments/780/review-r1/spec-delta-reviewed.md) | verification=model21/21/consumer3/3/adapter5/5/check通过、normal闭包无lang/man/UI；独立组合3pass/9fail，missing-required反控原门exit0(165/155假相等)；模型/adapter fmt通过、consumer fmt失败；受管输入/锁/HEAD/status/真实index不变，diff-check通过；没有整档回归 | pending_closeout_gates=最终review pass后merge-owned裸cargo t及T07 Spec/投影/合流/归档/批到期/guard-clean | task_state=保留T01，重开T02..05，executing 1/7，r1不变 | next=同树work修复全部finding并补永久容器分类/重复/信息保留/读取失败传播控制、同步当前164语料计数；提交后复审。本次不修实现/merge/发布canonical/清理树。

2026-10-15 R1 修复：stage=work | PLAN-780 | r1 合同不变 | outcome=pass（R1 needs_fix 五项+格式项全部修复，重新交独立review） | worktree D:/autostack/.wt/lang-780/auto-lang | branch plan-780-dev | base 075a9b4ab 不变 | code 修复提交（6aa07b630 之上，含 R1 修复+fixture+复测） | 逐 finding 闭合表（finding → 不变量 → 修复与永久用例 → 现结果）：
- **F780-R1-01/P1 结构化值位裸标识符被静态化** → 不变量=冻结子集"值位裸标识符须 unsupported"必须覆盖**全部值入口**（标量/数组元素/对象值/顶层对象文档），不得让 AtomParser 丢词法身份后猜 → 修复=新增结构感知 `scan_value_positions` 值位分类器（键位/值位区分；值位白名单=字符串(插值检查)/数字/true-false-null-nil；on/off 无 toggle 上下文亦分类；容器内调用/节点形态分类），数组/对象/顶层对象文档三入口全接 → **根因**=初版沿用了 `expr?;` 传播——Option 的 `?` 在 Some 时解包继续、None 时返回，方向与意图相反：Some 被丢弃遇 `]` 返回 None（假静态），未终止输入则死循环（复审 hang 同根）；改 if-let 显式传播 → 永久用例=model `r1_structured_value_bare_idents_are_dynamic`（5 反例+顶层对象+2 正控）+adapter `project_model_r1_dynamic_structured_and_wrapper`（含旧臂 `members:[port]→["win32"]` 求值证明）+fixture `dynamic-structured-values/-top-object-values` → 现结果全绿。
- **F780-R1-02/P2 dep 体重复 path 取首值** → 不变量=节点体与顶层同守"后声明覆盖+重复记录" → 修复=DependencyDecl.path 取末次声明；`NodeProp{key,value,line}` 带行号保留全部声明；`prop_duplicates` 记录 → 永久用例=`r1_dep_duplicate_path_last_wins` + fixture `static-dep-dup-path`（对拍 old=second ✓）。
- **F780-R1-03/P2 未知节点子树/参数丢失** → 不变量=未建模内容必须可在 DTO 恢复（不能靠重读源文件） → 修复=DependencyDecl.children 保留 dep 体子树；RawNodeDecl.raw_args 保留原始参数文本 → 永久用例=`r1_unknown_node_children_and_args_preserved`（`__CHILD_SENTINEL__`/`__ARG_ONE__`/`__ARG_TWO__`）+ fixture `static-unknown-node-content`。
- **F780-R1-04/P1 读取失败吞为空配置相等** → 不变量=required 输入读取/编码失败必须使验收门禁非零，不得退化为空配置对拍 → 修复=ComparisonReport.read_error 面（缺失/坏 UTF-8 双臂 Err 带 path）；corpus 门禁遇 read_error 直接入 failures 断言失败；`(Err,Err)` 桶收紧为独立 both_rejected（预期坏 fixture），不与 template 混同；空文件保留为合法区别控制 → 永久用例=adapter `project_model_r1_read_failures_propagate`（missing/bad-utf8/empty 三控）+ 复审 gate-negative 复跑 **contract_pass=true（exit 101，真实 index 前后 hash 不变）**。
- **F780-R1-05/P2 root wrapper 假静态** → 不变量=T-01 冻结合同显式排除 root wrapper，未实施 hoist 兼容前不得给静态等价标志 → 修复=UnsupportedKind::RootWrapper 分类+节点原样保留（不 hoist、不伪造值） → 永久用例=model `r1_root_wrapper_is_not_static` + fixture `dynamic-root-wrapper`（旧臂仍 hoist 求值，V-03 不损失）。
- **F780-R1-06/P3 consumer 未格式化** → 独立 rustfmt 收口（workspace exclude 不入根 fmt 面）。
| 复测：model **27/27**、adapter **7/7**、consumer **3/3**、`cargo check -p auto-lang -p auto-man` exit0、复审 `replay.py` 独立 12 组合 **12/12**、`gate-negative.py` contract_pass=true；corpus 门禁最终计数（含 21 个受管 fixture）=**170 = 157 static 相等 + 9 dynamic + 1 template_rejected + 3 both_rejected**（修正初版收据的 149=144+4+1 采样面口径——现按 `git ls-files` 全受管面计数并在测试内打印）。生产读取仍零改动（六读点文件 diff 空）。无剩余 gap；merge-owned pending 不变（裸 cargo t 于冻结提交 + SD-01/02 沉淀 + tf 到期判定）。next=review（T-06 r2：绑定修复提交，重点=R1 五项闭合的独立反例重放与计数口径）。

2026-10-15 R2：stage=review | review_scope=final | reviewer_identity=本规划/R1评审会话，未实施780 | plan_revision=1 | outcome=needs_fix | worktree=D:/autostack/.wt/lang-780/auto-lang | branch=plan-780-dev | reviewed_commit=ee67ea5df5b7cf47a6eaea30bc798489e1ef6618 | base=075a9b4ab6cb4284f3c259b348f2571322107fec | deps=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；锁/受管输入最终复核见指纹 | spec_sha256=bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3 | AC01 pass、AC02/03 partial、AC04审计完成未pass、AC05 pending | findings=F780-R2-01(P1值位扫描器无进度导致循环)/F780-R2-02(P2 both_rejected未限定预期坏输入) | evidence=[R2报告](attachments/780/review-r2/REVIEW.md)、[终止性控制](attachments/780/review-r2/bounds.json)、[原门坏输入反控](attachments/780/review-r2/gate-malformed.json)、[规范增量冻结](attachments/780/review-r2/spec-delta-reviewed.md) | verification=原12控制12/12、model27/27、consumer3/3、adapter7/7、check/fmt/diff-check通过；正常corpus170=157+9+1+3；missing-required原门exit101控制通过；新增边界2pass/4timeout；未批准损坏required原门exit0假绿(171=157+9+1+4)；生产读取未切换 | task_state=保留T01/T04，重开T02/T03/T05，T06未pass/T07待办，executing 2/7，r1不变 | next=同树work先补连续两轮final失败的根因/覆盖矩阵及设计调整，再闭合全部finding，提交后复审 | merge-owned裸cargo t及SD/ledger/合流/归档/批回归到期判定/guard-clean pending；本轮不修实现、不跑整档、不合入或清理。

2026-10-15 R2 修复：stage=work | PLAN-780 | r1 合同不变 | outcome=pass（R2 needs_fix 两项修复+附带分层 bug，重新交独立review） | worktree D:/autostack/.wt/lang-780/auto-lang | branch plan-780-dev | base 075a9b4ab 不变 | code R2 修复提交（ee67ea5df 之上）。
**两连败诊断（技能检查点，R1+R2 同源）**：值位扫描器把"非本语法"当"跳过/延后"处理而无前进保证——R1=Option `?` 方向反（Some 被丢弃）、R2=三处零消费分支（未知字符→None 不消费、非标识键→消费 0 字符、缺冒号→continue 无约束）。前两轮检查为何未发现：R1 回归只断言分类**结果**，从未对对抗输入测**终止性**，且无边界矩阵（未知 token/错配闭符/缺冒号/表达式/同邻元素）。本轮设计调整（非旧反例追加）：①前进保证为代码级不变量（容器循环每轮消费≥1 字符或返回）；②分隔语义按 **VM oracle 实测**钉死（同行相邻=Err、换行/逗号/尾逗号=Ok——探针转正为 `p780_separator_semantics` 永久测试）；③终止性测试=线程+recv_timeout(2s) 外部截止（回归时 fail 而非挂死套件）；④门禁豁免桶=身份 allowlist+结构化 stage 诊断。
- **F780-R2-01/P1 扫描器无限循环** → 修复=未知字符/非标识键分支消费 1 字符并分类 `ValueSyntax`；元素/键值对间加 `check_separation`（同行相邻分类，换行/`,` 放行）；`cfg:{a 1}` 类缺冒号路径受同约束 → 永久用例=`r2_value_scanner_terminates_and_classifies`（复审 4 超时例+8 边界扩展，Ok→非静态有理由 / Err→stage∈{Value,Document}；3 正控；全部 2s 截止保护）→ 复审 `bounds.py` **5/5 contract_pass**。
- **F780-R2-02/P2 双拒豁免未绑定身份** → 修复=corpus 门禁 `EXPECTED_BOTH_REJECTED`（bad-missing-brace/bad-missing-value，各绑预期 stage=value）与 `EXPECTED_TEMPLATE_REJECTED`（clib 模板+dynamic-expressions fixture）身份 allowlist；`ComparisonReport.new_error_stage` 结构化诊断字段；非清单双拒/模板输入直接红门禁 → 复审 `gate-malformed.py`（未批准损坏 required `name: [`）**contract_pass=true（exit 101）**、`gate-negative.py` 保持 contract_pass=true。
- **附带修复**：clib `lib ${name} {}` 的 '$' 名位插值在 parse_entry 层未路由到 parse_node（R1 修错了层）→ 误报语法错误落入双拒桶；现正确分类 Interpolation → template 桶。桶分布校正：**170 = 157 static + 9 dynamic + 2 template（clib+dynamic-expressions）+ 2 both（bad-missing-brace/bad-missing-value）**，逐桶身份与 allowlist 精确一致（测试内 eprintln 证据面）。
| 复测：model **28/28**（+r2 终止性套件）、adapter **8/8**+`p780_separator_semantics` 1/1、consumer 3/3、check -p auto-lang -p auto-man exit0、rustfmt 三面 --check 通过、R1 replay（R2 版）**12/12**。无剩余 gap；merge-owned pending 不变（裸 cargo t 冻结提交 + SD-01/02 + tf 到期判定）。next=review（T-06 r3：绑定 R2 修复提交，重点=终止性矩阵外部截止真实性/allowlist 身份穷尽性/分隔语义 oracle 钉面）。

2026-10-15 R3：stage=review | review_scope=final | reviewer_identity=本规划/R1/R2评审会话，未实施780 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-780/auto-lang | branch=plan-780-dev | reviewed_commit=61eea96971c1290c745da4ff74e80795834aa9cb | base_commit=075a9b4ab6cb4284f3c259b348f2571322107fec | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；锁/受管输入见指纹最终复核 | spec_inputs=auto-man/project SHA256 bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3；SD01新目标尚不存在，SD01/02 partial冻结 | readiness=两轮失败根因分析、前进保证/截止与分隔oracle/异常清单设计调整已核验，允许final复审 | acceptance_results=AC01 pass、AC02/03 partial、AC04审计完成未pass、AC05 pending | findings=F780-R3-01(P2新分隔检查误拒合法注释及builtin换行)/F780-R3-02(P2异常桶路径后缀匹配可被同名/同后缀影子绕过) | evidence=[R3报告](attachments/780/review-r3/REVIEW.md)、[组合旧新结果](attachments/780/review-r3/combinations.json)、[bad影子原门反控](attachments/780/review-r3/gate-shadow-bad.json)、[template影子原门反控](attachments/780/review-r3/gate-shadow-template.json)、[规范增量冻结](attachments/780/review-r3/spec-delta-reviewed.md) | verification=model28/28、consumer3/3、adapter8/8、separator1/1、check/fmt/diff-check通过，正常corpus170=157+9+2+2；R1独立12/12、R2 bounds6/6、missing/原unexpected-required反控exit101均通过；新增组合4pass/5fail无超时，两个身份影子原门均exit0假绿；生产读取未切换 | closure=R2-01终止性关闭，新兼容回归单列R3-01；R2-02旧例关闭、完整身份不变量partial延续R3-02 | task_state=保留T01/T04、重开T02/T03/T05、T06未pass/T07待办，executing 2/7，r1不变 | pending_closeout_gates=最终review通过后merge-owned冻结提交裸cargo t及T07 Spec/ledger/合流/归档/批回归到期判定/guard-clean | next=同树work修复全部finding并补trivia/分隔组合与精确身份矩阵，提交后复审。本轮不修实现、不跑整档、不发布canonical、不合入或清理。

2026-10-15 R3 修复：stage=work | PLAN-780 | r1 合同不变 | outcome=pass（R3 needs_fix 两项修复，重新交独立review） | worktree D:/autostack/.wt/lang-780/auto-lang | branch plan-780-dev | base 075a9b4ab 不变 | code R3 修复提交（61eea9697 之上）。
- **F780-R3-01/P2 合法语法误拒** → 根因=R2 的 check_separation 把 trivia 与分隔割裂：只跳空格不识别值后注释（`/` 直接分类），且 ident 臂 skip_ws_and_comments 吞换行使下一行元素被误判同行相邻 → 修复=**per-container oracle 语义**（R3 期间对拍探针实证不对称语法并转正 `oracle_asymmetric_separator_semantics` 永久断言：数组=值后逗号/换行/注释任意混排 Ok、元素位逗号/先行逗号 Err、双逗号 Ok；对象=逗号仅同行紧随值 Ok（含尾逗号/逗号+换行）、passed-break 后逗号 Err（"Expected key, got Comma"）、双逗号 Err）；check_separation 以 passed_break 状态位统一 trivia 循环（行注释→换行承载、块注释=trivia 且内嵌换行计 break），ident 臂改仅内联空白保留合法换行信息；值类型（字符串/数字/builtin）不再走不同分隔语义 → 永久用例=model `r3_trivia_separation_matrix_stays_static`（R3 复审 5 失败例+4 通过例钉面+11 扩展=注释位置×值类型×容器+7 负控含不对称形态）+adapter `project_model_r3_trivia_parity_matrix` 双臂对拍 8 例 → **复审 combinations.py 9/9**。
- **F780-R3-02/P2 豁免后缀匹配** → 修复=豁免身份改**精确仓根相对路径**匹配（`rel_identity` 反斜杠归一 + 全路径清单条目；corpus 内直接等值 find/any；`project_model_r3_exemption_identity_is_exact` 影子负控单测断言同名/同后缀影子不获豁免、真实清单项匹配）→ 复审 **gate-shadow-bad/gate-shadow-template 双控 contract_pass=true**（影子文件现在使门禁 exit 101）。
| 复测：model **29/29**（干净重建）、adapter **11/11**（干净重建，corpus 170=157+9+2+2 稳定）、consumer 3/3、check -p auto-lang -p auto-man exit0、rustfmt 三面 --check 通过、R3 全部脚本= replay 12/12 + bounds 全过 + combinations 9/9 + gate-negative/gate-malformed/gate-shadow-bad/gate-shadow-template contract_pass 全 true。无剩余 gap；merge-owned pending 不变。next=review（T-06 r4：绑定 R3 修复提交，重点=不对称分隔语义钉面与影子负控的独立性重放）。

2026-10-15 R4：stage=review | review_scope=final | reviewer_identity=本规划/R1–R3评审会话，未实施780 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-780/auto-lang | branch=plan-780-dev | reviewed_commit=3ab22eeb7b50dcbdc012cc2ce0a7aede9cc5a4dc | base_commit=075a9b4ab6cb4284f3c259b348f2571322107fec | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；锁/源码最终复核见指纹 | spec_inputs=auto-man/project SHA256 bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3；SD01新目标尚不存在，SD01/02 partial冻结 | readiness=R3根因/统一trivia及容器分隔设计调整/精确身份清单与永久控制已核验 | acceptance_results=AC01 pass、AC02/03 partial、AC04审计完成未pass、AC05 pending | findings=F780-R4-01(P2块注释内物理换行被当语法分隔：合法带逗号对象误拒，两个旧VM拒绝输入却static=true) | evidence=[R4报告](attachments/780/review-r4/REVIEW.md)、[新增分隔矩阵](attachments/780/review-r4/separator-matrix.json)、[源输入](attachments/780/review-r4/separator-matrix.rs)、[规范增量冻结](attachments/780/review-r4/spec-delta-reviewed.md) | verification=model29/29、consumer3/3、adapter11/11、separator2/2、check/fmt/diff-check通过，正常corpus170=157+9+2+2；R1重放12/12、bounds6/6、R3组合9/9、missing/原损坏required/两影子原门exit101控制通过；新增16例矩阵12一致、3阻断差异、1安全拒绝边界差异(双逗号夹注释，非冻结必需语法，不新增阻断)，无超时；生产读取未切换 | closure=R3-02完整身份closed；R3-01原例closed、完整trivia不变量partial延续R4-01；R1/R2已闭合具体控制保持 | task_state=保留T01/T04，重开T02/T03/T05，T06未pass/T07待办，executing 2/7，r1不变 | pending_closeout_gates=最终pass后merge-owned冻结提交裸cargo t及Spec/ledger/合流/归档/批到期/guard-clean | next=同树work纠正物理换行与语法分隔混同的根因，补块注释/真实换行成对正反矩阵，提交后复审；本轮不修实现、不跑整档、不发布canonical、不合入或清理。

2026-10-15 R4 修复：stage=work | PLAN-780 | r1 合同不变 | outcome=pass（R4 needs_fix 单项修复+两项实证校正，重新交独立review） | worktree D:/autostack/.wt/lang-780/auto-lang | branch plan-780-dev | base 075a9b4ab 不变 | code R4 修复提交（3ab22eeb7 之上）。
- **F780-R4-01/P2 块注释内换行被当语法分隔** → 根因=R3 的 check_separation 块注释臂把注释内 `
` 同时计了行号与 passed_break——混淆"源码行号变化"与"语法换行事件" → 修复=块注释臂仅 `self.line += 1`，分隔状态不变（VM 视整块注释为 trivia）→ 阻断例 0 `{ project: \"b\" /*multi
line*/, extra: 1 }` 恢复 static 且 external_backend 保留；例 1/2（注释换行后无逗号相邻）正确分类非静态（VM "Expected infix operator"）→ 永久用例=model `r4_comment_newline_is_not_a_separator`（R4 三阻断例+成对真实换行/注释换行矩阵 17 例+CRLF 真分隔+嵌套 builtin 换行+top-object 对照）。
- **实证校正①**：`#` 注释在 VM **容器值位本就非法**（R4 matrix id3/4/11 旧臂全 Err "Expected infix operator, got Hash"）——此前我误当合法例；现钉面为双臂一致拒绝（模型 Err stage∈{Value,Document} 或非静态，永不假绿）。文档级 `#` 行注释（pac.at 顶层）不受影响。
- **实证校正②/id6 非阻断限制**：`[\"a\", /*c*/, \"b\"]` 逗号与注释混排 VM-Ok 但值层 AtomParser 拒绝——模型诚实 Err 而非假 static；按复审裁定不入冻结子集、不扩支持面，已在 scanner.rs 文档化并记入 **SD-01 限制清单素材**（Spec 不得宣称支持所有 VM 逗号混排）。
| 复测：model **30/30**、adapter **11/11**（corpus 170=157+9+2+2 稳定）+separator oracle 2/2、consumer 3/3、check exit0、rustfmt 三面过；R4 全套脚本= replay 12/12 + bounds 全过 + **combinations 9/9** + gate-negative/gate-malformed/gate-shadow-bad/gate-shadow-template contract_pass 全 true + separator-matrix **仅 id6 失败（已文档化非阻断限制）**。无剩余阻断 gap；merge-owned pending 不变（裸 cargo t 冻结提交 + SD-01/02 沉淀〔含 id6 限制〕+ tf 到期判定）。next=review（T-06 r5：绑定 R4 修复提交，重点=注释内换行语义与成对矩阵的独立重放、id6 限制如实入 Spec）。

2026-10-15 R5：stage=review | review_scope=final | reviewer_identity=本规划/R1–R4评审会话，未实施780，依据diff及独立旧新oracle重建 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-780/auto-lang | branch=plan-780-dev | reviewed_commit=68fcc3bf768a53f5c0b1c9088c02f2165224f7b0 | base_commit=075a9b4ab6cb4284f3c259b348f2571322107fec | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；锁/受管源码最终复核见指纹 | spec_inputs=auto-man/project SHA256 bb857003628365be87cec91912ab36c672388a943163d5fc3584fc44600e80c3；SD01新目标尚不存在，SD01/02已reviewed冻结 | acceptance_results=AC01/02/03/04 pass（冻结pilot范围）、AC05 merge-owned pending | findings=无新增阻断；R1六项/R2两项/R3两项/R4一项全部closed | evidence=[R5报告及完整闭合表](attachments/780/review-r5/REVIEW.md)、[16例矩阵](attachments/780/review-r5/separator-matrix.json)、[规范增量冻结](attachments/780/review-r5/spec-delta-reviewed.md)、[指纹](attachments/780/review-r5/fingerprints.json) | verification=model30/30、consumer3/3、adapter11/11、separator2/2、check/fmt/diff-check通过；正常corpus170=157+9+2+2；独立R1重放12/12、bounds6/6、R3组合9/9、四原门禁反控cargo exit101均通过；R4矩阵15一致+1已知安全拒绝限制（id6双逗号夹注释，探索脚本仍exit1，非冻结必需语法，已如实入SD01）；原三阻断例闭合，无超时；生产读取未切换 | task_state=T01..T06完成，reviewed 6/7，r1不变 | pending_closeout_gates=merge在冻结提交裸cargo t通过后，SD01/02 canonical/ledger沉淀、合流/归档、批回归到期判定、guard-clean清理；未触VM/codegen/trans/aavm，无tv/tu/tt/taa触面档 | next=merge；review pass不等于交付或可跳合入门禁。本轮未跑整档、未修实现、未发布canonical/ledger、未合入/归档/清理。

2026-10-15 merge 收据：stage=merge | PLAN-780:r1 | outcome=pass（delivered） | 收据 checkpoints：
- **prepared**：reviewed_commit=68fcc3bf7（R5 pass，spec 源 SHA256 bb857003… 应用前实测一致）；SD-01 落 docs/specs/project-model/project.md（新）、SD-02 落 auto-man/project.md 增"静态项目模型 pilot"节+spec-index GROUPS 登记；delivery_commit=ec934b000（SD 沉淀为 reviewed 代码纯文档后代，实现/依赖零变更）。
- **门禁（merge-owned 裸 cargo t，冻结提交上）**：三跑均 5352/5353（5353 全选择面），每次单个不同失败：plan484_024_charts_streaming_recompute（run1，fail）→ plan707_wait_generator_park_drive_count_static（run2，TIMEOUT 120s）→ plan776_r7_iterator_credential_native_family（run3，TIMEOUT 120s）。归因：三者单跑全绿（plan484 双树、plan707/776 worktree），逐次轮换身份+全部位于本计划零改动 crates/auto-lang（diff 为空）+机器整日处于他 session 并发 cargo 负载（nextest slow-timeout 被竞争拖爆不同重测试）=负载 flake 而非代码缺陷；plan707 为 AGENTS 在案 flake 族。不触 VM/codegen/trans/book/ui_gen/aavm → 无 tv/tt/tb/tu/taa 触面档。
- **landed**：rebase 10 提交到 master（range-diff 9/9 全 `=`+1 纯文档新增=安全重写证明；旧→新映射 f3ca995e7→97fde72d1/d7c7403e0→a6f508655/ac4e20b46→5fe68e5d0/b9738a272→d23bc73de/6aa07b630→328ef4a0c/ee67ea5df→6ae677314/61eea9697→dc1ead6ad/3ab22eeb7→51ddbb6c8/68fcc3bf7→12e6e77db）；`git merge --ff-only plan-780-dev`，master tip=ec934b000=delivery（无合并提交）；主检出冒烟 model 30/30+adapter 11/11 绿。
- **ledger_refreshed**：`.autoos/specs.json` 受管离线投影（Regime B，无 store writer 可达；字节备份 /tmp/specs-backup-780.json），version 21→22，新增 P780-D1（designs，file=docs/specs/project-model/project.md）/P780-T（tests，file=crates/auto-project-model/src/tests.rs）/P780-R5（reviews，file=review-r5/REVIEW.md）/P780-M（reports，file=归档计划路径），全部带 file+tags；git diff 44+/1− 外科范围证明+读回 ID/字段校验通过、六 section 恒等、旧 items 全保留。
- **archived**：git mv → docs/plans/archive/780-project-model-decoupling-pilot.md，status: archived，T-01..T-07=7/7；review-r1..r5 复审附件随本收据入库（K13）。
- **cleaned**：wt-guard 双树 clean（reparse point 扫描均空）→ lang-780/auto-lang 与组内 auto-down 兄弟 worktree 移除（后者从属仓 auto-down 注销）、plan-780-dev 分支删除（was ec934b000）、组目录 D:/autostack/.wt/lang-780 已删；git worktree list 仅剩主检出与 779/781/782/783。
- **部署产物核对**：auto-lang 无发布二进制/web bundle 生产面，N/A。
- **批回归到期判定（fix-test-tiering）**：**due**——780%5=0 落地；移交 /auto-plan:regress 主检出单实例执行。

## 10. 待澄清事项

T01必须明确字段/未知输入支持oracle；长期动态manifest去留另议。纯parser不足则显式保adapter+最小扩展提案，不另造完整compiler。
