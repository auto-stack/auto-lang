---
plan_id: PLAN-1043
status: archived
feature_name: ACE标准库context装配与最小#in平台选择
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-13
plan_revision: 1
phase: M2-A
current_step: 7
total_steps: 10
depends_on: [PLAN-936, PLAN-958, PLAN-944, PLAN-947, PLAN-960, PLAN-1042]
supersedes_spec_components: [docs/specs/stdlib/design/ace-binding-and-assembly.md, docs/specs/auto-lang/comptime/design/hash-syntax.md]
new_spec_components: [docs/specs/stdlib/design/context-source-assembly.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace, comptime]
---

# [PLAN-1043] ACE标准库context装配与最小#in平台选择

## 0. 变更摘要

2026-11-13 r1 work 代码面收尾中，executing 5/10：T-01 入口冻结（scope `plan-artifacts/plan-1043/t01-entry-freeze.md`：packages 面载体、packages-ct 新 profile+`ac.source.cte.in/v1`、旧 profile grammar_id 逐字节不变（RuleSpec/TokenSpec 不含 profiles 元数据，已核）、`pub spec fn` 需求拼写、OS 判别值 Win=1/Linux=2/Mac=3+current=target+host 另名、PackageStore /2、legacy `.at.at`/`.py.at`→Other、FRONT_CAPS_VERSION 不动（packages-ct 不经 FrontRequest 路由））；T-02 provider_sources 模型（后缀分类/三轴选择/相容表 At→全/C→Av+A2C/Rs→Av+A2R/Py→A2Py，Auto 默认=At 无则具名拒不回落）；T-03 packages-ct 语法+投影+绑定期静态事实求值（is/if/for 阶段继承、单行 use 符号表、等值重复臂拒、pratt primary 内 opt 尾巴会破——ns-ref 用规则级 alt；`0..3` 被 affix-num 吞成单词——融合 range 备选按 `..` 拆分）；T-04 合并/填充/签名核验/ctx 身份/恢复重放；T-10 legacy inventory 测试+旧配方回归。ASM-01..04/06 单测绿（core 26 + store 7 + loader 2）；ASM-05 av `stdlib_assembly_1043.rs` 四腿真跑绿（AV×4 配置/native mn/rustc/cl，A2PY 仅绑定证明）。**2026-11-13 1042交付解除依赖后：T-05 A2PY 执行腿真跑完成（`PythonTargetProvider`+pyrun，value:115/125），T-06 收口重跑冻结并集；executing 7/10，待 T-07 复审**。用户确定公共`.at`＋at/c/rs/py实现来源后缀，平台由`#in sys is current`继承编译期属性选择，不扩平台后缀。944/947已归档，因此另开本包实现新合同。设计：[覆盖/加载设计§8–10](../design/stdlib/backend-provider-coverage.md)。规划轮不建树/不编译；2026-11-13 `/goal 完成计划1043` 授权整链执行：worktree `D:/autostack/.wt/lang-1043/auto-lang`（分支`plan-1043-dev`，基 c2760324d），scope `D:/autostack/plan-artifacts/plan-1043/` 已登记。

## 1. 目标

普通Auto工程以唯一公共API/同一逻辑模块装配所选来源，目标平台静态选择与运行后端独立；AV/AC/A2R/A2PY及原A2C从同一统一source前门消费实际结果。实现最小#in namespace/阶段继承和条件导入，形成966完整CTE及967/标准库owner可消费的正式来源。

本包是加载/语言前置，不宣称新File/print、无libc OS服务、完整C/Rust importer或三provider标准库已经完成。非目标：裸in的任意运行时with、任意动态namespace、完整宏/反射/async、整库重写、绕过现授权或扩大macOS产品承诺。

## 2. 架构方案

公共/所选provider源及目标/namespace事实→既有Schema/CST/source-query→统一模块绑定与最小source-CT→已验HIR→AV/AC或共享A2X→真实执行。用原944/947装配及936/958 CT协议，不依legacy Parser/AutoVM执行或另建binder。可以在现owner下新增小类型/模块，禁止复制一套值/IR/store/loader。

完成依赖按frontmatter；T-01至既有后端腿只需已交936/958/944/947/960及其实际支持域，先核正式代码/收据而非只看archived。A2PY完整验证待1042，但不反等完整966/967/972/973；966随后消费本包最小能力，不能由本包再等966整包。遇未知语言/协议触面由T-01具名决定最小前置并复核DAG。

## 3. 技术栈

Auto标准库源、现Rust编译器/共享模块及CTE、既有AV/native/A2X worker/TestCase与固定Cargo组。唯一后续树`D:/autostack/.wt/lang-1043/auto-lang`，分支`plan-1043-dev`；材料`D:/autostack/plan-artifacts/plan-1043/`，均未创建；开工登记owner/用途，merge按AGENTS全scope删除及guard清树，不建链接。

## 4. 需求分析与背景调查

依据用户2026-11-13要求更新设计/计划与确认下一入口，不含本轮代码实施。已读Spec overview、stdlib project、ace-binding-and-assembly、hash-syntax、auto-cte scalar-query-profile，以及944/947/936/958/960和966/1042计划。现归档936/942存在旧任务计数与终态不齐的历史，不据archived推全部原任务运行完成；T-01按本包具体所需接口/原验收收据核可用性，不修改其历史。

真实owner：`crates/auto-lang-core/src/stdlib_assembly/{model,plan,provider_selection,manifest}.rs`保旧组织/兼容与947选择事实；新ACE主路径在`experimental/ace/crates/core/src/source/{cst/frontdoor,packages/stdlib,modules/binding,decl_env}.rs`、`experimental/ace/crates/source-query/src/assembly/host.rs`及现CT/query owner；后端消费用av/native/a2x-core，Python producer取1042正式来源，不复制旧`trans/python.rs`。legacy parser的`parse_hash_is_branch/parse_expr_or_body`只能作语义取材：Arrow为`->`，单行use尚未接入；不能把改旧parser冒新ACE前门支持。各路径T-01按当前真实结构再冻结。

## 5. 详细设计

1. **后缀与选择**：保留`io.at`公共源，at/c/rs/py四类候选为`io.at.at/io.c.at/io.rs.at/io.py.at`；保留后缀最长匹配，不把io.at.at列成公共模块io.at。后端与provider分轴，纯公共body可单源共用，不强制每库建四个空文件。显式来源缺失、歧义/重复实现、签名/资源合同不符均在调用前拒，native不静默回落外库。旧`.vm.at`与原Rust宿主producer路径仅按现旧配方兼容，不能混作新合同；退役归986/987。
2. **逻辑模块**：公共声明与选中实现由原binder按同一模块身份核名/签名/可见性/owner。每个需实现的公共操作恰好一个body；重复公共body不以文件顺序覆盖。保每文件CST origin/token/source map，合并是语义装配，不靠新文本宏或抹掉文件归因；临时namespace不会吞掉选中use的外层可见性。
3. **#in**：紧随结构`#in sys is current { Windows -> use ... }`或块式`#in sys { is current { ... } }`；内部is/if/for继承CT阶段，不重复#。namespace是编译期可解析的静态名字域，不接受运行时对象；sys与OS常量由统一声明/显式目标metadata事实供给，不按源码字面量硬编码一套专用查名；作用只在覆盖结构/块，禁止污染外层/无声覆盖同名绑定，歧义具名拒且可用限定名消歧。复用既有CTE/匹配/语句/绑定和phase传播，选中声明按Emit规则落原模块；普通函数定义不立即执行，函数自身阶段和CT权限/预算保持。最小OS判别值的真实标量/类型表示由T-01先证，不要求完整Dynamic。
4. **平台**：current来自target输入，Windows/Linux/Mac是同域常量，当前required目标仍Windows/Linux。未选分支只语法检查，不绑定/类型检查/链接；选中use/module需存在且可验证。匹配优先级保既有规则，平台重复等值臂诊断；无匹配留下必需操作时具名拒。不把普通#is无匹配空展开全局改写。
5. **增量/恢复**：在原query/store写真实公共/选中provider/namespace/target/ABI/配方版本和候选存在性读集。改变相关输入必须失效，未选body不误使本目标语义重编；失败/取消/晚到产物保旧CURRENT/在途generation。恢复不带授权、进程地址和外库handle；按原深验/重授信执行，新旧装配配方版本不能互冒。
6. **覆盖**：同源选择/绑定fixture实际执行AV/AC/A2R/A2PY及原A2C，各有手定独立expected/真实producer。AV三配置的纯选择部分可按相同身份/算法复用，但不能把标成c/rs的纯Auto返回值fixture冒实际C/Rust dispatch；967及各公共库能力另交真实调用/资源证明，未交仍pending。系统调用底座1039/1040与本包独立，两者均不代整库验收。

### 规范增量（实际交付后落Spec）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/stdlib/design/context-source-assembly.md | 旧后端侧文件→已验provider后缀/选择/逻辑模块/拒绝/跨端来源 | 新装配合同 | AC-01/02/04 |
| SD-02 | modify | docs/specs/stdlib/design/ace-binding-and-assembly.md | 947有限选择→本包实际新语义及旧兼容/未交边界 | 不重开944/947 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-lang/comptime/design/hash-syntax.md | 原四#构造→已验#in名字域/阶段继承/条件导入与限制 | 用户确定语法 | AC-03/04 |

## 6. 测试设计

T-01冻结真实入口及修改对应完整职责组，候选`cargo t stdlib ace-syntax ace-source ace-semantics ace-comptime ace-artifacts ace-av ace-native ace-a2x ace-stdlib a2py auto-test-case`；按实际职责选并集，不把全部候选机械作为每次迭代门。共享AST/语义/CT改面须补真实下游，最终在冻结提交运行确定的完整组及必要消费者；无全仓/正式ace-perf或GUI门。

| V / AC | 普通源/负控 | 独立oracle与owner |
|---|---|---|
| ASM-01 / AC-01/02 | 公共模块+四类候选、同名用户模块/私有符号；切AV/AC/A2R/PY/原C及来源，移除/添加候选、重复body/错签名/把io.at.at误作公共模块 | 所选源/绑定符号及手定值、调用前拒绝阶段；T-02/T-04/T-05，纯fixture不代真实外库provider |
| ASM-02 / AC-03 | 用户#in单结构/块式、is/if/for无需重复#、单行use/块式use、外层函数引用；与完整限定#is对照 | 相同目标选择/值/模块可见和source origin；namespace局部、函数定义不立即执行；T-03/T-04/T-10 |
| ASM-03 / AC-03/04 | Windows宿主目标Linux及反向；未选SDK缺失、坏语法、选中缺库、重复平台臂、无匹配且缺必需实现；运行时对象冒namespace/名字歧义 | 正确target独立expected，未选库不绑定链接、负控在正确阶段拒绝；真OS操作的实跑另由1039/1040/库owner；T-03/T-06 |
| ASM-04 / AC-04 | 修改公共签名/选中body/target/namespace常量/ABI/候选存在性；仅改未选body；失败/取消后修复、重启首编辑/缺授权 | 原query计数及精确失效/无关复用、失败保旧、恢复重验/重授信，旧epoch拒绝且dispatch0；T-04/T-06 |
| ASM-05 / AC-04 | 从公开source/worker实际执行各后端；缺Python/helper、emit-only、旧AST/VM fallback、错来源/错误所有后端相等 | 独立值/副作用与原expected，不以target标签或emit冒run；A2PY需1042正式来源，缺环境pending；T-05/T-06 |
| ASM-06 / AC-02/04 | 旧738/944/947原例/manifest与新配方分开，错误定位公共/实现/#in臂；错误候选不覆盖已有运行 | 原行为/诊断位置及新旧版本边界保留，不偷偷切旧默认或删除旧producer；T-10/T-06 |

Windows/Linux按已有承诺平台执行格记录实际runner与工具；交叉选择证明不是目标OS已运行证明。Mac只覆盖输入/选择机制，不充产品平台PASS。每case留源/expected/target/provider/工具版本/子终态及适用性；未运行/条件排除不得算PASS。

## 7. 验收标准

- [x] AC-01 / work：ASM-01选中唯一来源且公共API/模块身份正确，最长后缀、缺候选/歧义/错签名负控关闭，纯公共body不重复生成。
- [x] AC-02 / work：ASM-01/06验证可见性/单实现/来源映射/旧兼容及多文件诊断，现947/944原合同不退步。
- [x] AC-03 / work：ASM-02/03真实新ACE source前门闭合#in两种形态、阶段继承、局部namespace与单行use；无旧Parser/VM替执行，无运行时OS误用。
- [x] AC-04 / work：ASM-03..06与实际完整职责/必要消费者、AV/AC/A2R/A2PY/旧A2C真实运行及缓存/恢复/资源正反例通过，缺环境诚实pending；不冒公共库三provider完成。
- [x] AC-05 / review：停写后同树独立逐AC/来源/diff/遗漏/延后/workaround/健康及Spec-impact核验通过，in-scope问题回同树修复。
- [x] AC-06 / merge：冻结身份和有效证据核对，Spec及代码正式落主/归档；摘要折入Plan、全输出路径清点、scope删除/具名持有和wt-guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 前置 | 操作与结果 | AC |
|---|---|---|---|
| T-01 / work | 936/958/944/947/960正式所需来源、实施指令；1042为最终Python前置 | 树/scope登记；核namespace/阶段传播/target标量表示/来源后缀/真实命令/旧兼容的全链可行性与Spec差额。改变已定语义须先修约，不把细节反抛用户 | AC-01..04 |
| T-02 / work | T-01 | 扩既有选择模型与后缀/候选发现，分backend/provider/target并冻结逻辑source集合；建ASM-01 | AC-01/02 |
| T-03 / work | T-01 | 统一新ACE前门接#in的namespace/stage上下文、单结构/块式、is/if/for阶段继承及条件use，建ASM-02/03 | AC-03 |
| T-04 / work | T-02/T-03 | 现binder/decl_env/source-query/assembly host接公共+所选实现与选中平台/namespace依赖，保原子发布/恢复/权限；建ASM-04 | AC-01..04 |
| T-10 / work | T-04 | 多文件CST来源/诊断及旧配方兼容，保原738/944/947完整相关例与source map；建ASM-06 | AC-02/03/04 |
| T-05 / work | T-04、既有各后端及1042正式Python接口 | [✅ 已完成 r1] 接现AV/AC/A2X/TestCase真实生成运行及独立expected；ASM-01/02/05注册于 av `stdlib_assembly_1043.rs`（AV×4配置/native mn/rustc/cl + 1042交付后真 Python `value:115/125`，提交 05ed31a2c/a55773462）| AC-01..04 |
| T-06 / work | T-05/T-10 | [✅ 已完成 r1] ASM全正反例（core 26/store 7/loader 2/av 3全绿）；冻结并集两轮：基线6组2184 executed全PASS（run-4rtqswu5+run-akwglhgm）+ 合并1042后重跑（见§9收口记录）；fmt/零新警告 | AC-04 |
| T-07 / review | T-06 | 停写，同树独立逐AC/Case/差额与健康核验，失败修复后定向复核 | AC-05 |
| T-08 / merge | T-07 pass | 核冻结并集/正式来源/Spec delta，合主归档，给966/967/库owner真实接口与限制 | AC-06 |
| T-09 / merge | T-08 | 全命令实际输出路径对账、摘要折Plan、scope全删或具名持有/登记，guard clean后清树/分支 | AC-06 |

## 9. 复审记录

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / r1：仅设计与计划草案，0/10，无产品运行或独立实现review。标准取号脚本在master实际调用失败：Git Bash子退出3221225794（0xC0000142），未产骨架/改号；随后在中央.next-id文件用msvcrt单字节锁、查active/archive/旧archived冲突、exclusive create并flush/fsync更新到1044。未创建旁路ID计数器、未提交/建树。


2026-11-13 stage:work / r1 handoff | plan_id: PLAN-1043 | plan_revision: 1 | outcome: blocked（唯一缺口=PLAN-1042 A2PY 执行腿）| worktree: D:/autostack/.wt/lang-1043/auto-lang | branch: plan-1043-dev | base: c2760424d（master）| code_commit: 05ed31a2c | 依赖修订：936/958/944/947/960 按 T-01 核实已交付；1042 仍 drafting 0/8。任务：T-01..T-04、T-10 完成；T-05（AV×4配置/native mn/rustc/cl 真跑+expected 112/122/113/114/value:113/value:123；A2PY 仅绑定证明）、T-06（冻结并集+fmt+零新警告）代码与证据齐但按合同 A2PY 行 pending 故不勾。证据：core packages 26绿（plan1043_tests 9：ASM-01..03/06 正反例）、source-query store 7绿（ASM-04 身份轴/未选body复用/恢复重验/失败保旧/根内容边界）、legacy loader 2绿（io.at.at→Other 不冒公共模块）；`cargo t ace-source ace-syntax ace-semantics ace-stdlib stdlib`（run-4rtqswu5，514/132/275/2/170 executed 全 PASS 0 ignore）+ `cargo t ace-av`（run-akwglhgm，1091 executed 全 PASS，含 stdlib_assembly_1043 三用例）；冻结提交=05ed31a2c。blockers: T-05 前置"1042正式Python接口"未交付——A2PY 执行腿（ASM-05 py 行）在 1042 交付前诚实 pending；unblock=交付 PLAN-1042（最小 Checked HIR/A2X→Python producer+真实 Python 运行）。next_responsible_role: user 决策（先做1042 / 等待 / 修约descoping A2PY）。

stage: goal | plan_id: PLAN-1043 | plan_revision: r1 | round: 1 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1043/auto-lang | commit: 05ed31a2c | next: user（1042 依赖三选一决策）

stage: goal | plan_id: PLAN-1043 | plan_revision: r1 | round: 2 | from: user | to: waiting | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1043/auto-lang | commit: 05ed31a2c | next: user（1042 交付后恢复 T-05 A2PY 腿→T-06 收口→T-07 review→T-08/09 merge）

2026-11-13 stage:work / r1 恢复+T-05/T-06 收口 | 1042 依赖解除（用户指示）：merge master（eb745b1bd，带入 1017/1027/1042）；A2PY 腿改真执行（`PythonTargetProvider`+`pyrun::resolve_python`+`run_py_fileset`，Windows/Linux 目标 `value:115`/`value:125`，断言新 producer 输出形并拒绝旧 `PythonTrans` emitter），av stdlib_assembly_1043 3/3 绿；提交 a55773462。冻结并集重跑（含 ace-a2x）见下一条记录。next_responsible_role: review（T-07）。

2026-11-13 stage:work / r1 完成 | 冻结提交 7ef7f9919（=05ed31a2c 实装 + eb745b1bd merge master(1042) + a55773462 A2PY腿 + 8ad1a30f8 1042调用点收编 + 7ef7f9919 Spec delta与ASM-06补测）。全部 work 任务（T-01..T-06、T-10）完成：core packages 27绿（+impl诊断归位）、store 7绿、loader 2绿、av 3绿（真Python腿）；冻结并集 7组 2421 executed 全 PASS 0 ignore（合并树）。Spec delta SD-01/02/03 已在树内备好。next_responsible_role: review。

2026-11-13 stage:work / T-06 收口 | 合并1042后冻结并集（run-t9u5j7p_ + ace-av 补跑）：`cargo t ace-source ace-syntax ace-semantics ace-stdlib stdlib ace-a2x` = PASS 516/132/275/2/170/235 executed，0 ignored；`cargo t ace-av`（合并树，run-qoiqet7w）= PASS 1091 executed 0 ignored（含 stdlib_assembly_1043 三用例的真 Python 腿）。合并跟随修复：1042 侧 `pyready_1042.rs` 两处 `PackageModuleInput` 补 `provider: None`（提交 33d..见git log，本包API扩展的调用点收编，非1042回退）。fmt（本包文件）+ 零新增警告；av-core mir_verify 2 警告为 master 基线。当前冻结提交见 §9 末行。
2026-11-13 stage:review / final r1 | reviewer: 实现会话内独立上下文复审（限制：非独立会话；从已提交工件重建结论）| reviewed_commit: 7ef7f9919 | base: c2760424d+master09119bd3c(merge eb745b1bd) | deps: 936/944/947/958/960 archived；1042 archived ab1199386 | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1043/auto-lang | branch: plan-1043-dev | findings: R1（AC-03/§5.2"禁止无声覆盖已有绑定"）：`#in` 选中 use 落位时未做重名检查——与公共/其他落位 use 同绑定名时 binds 后到无声覆盖（反例：`use winio.plat` + `#in...Windows -> use lnxio.plat` 绑定成功，imports 含两条 plat，无诊断；顺带公共 use 位于 #in 之后的对撞同样漏检）。影响面：assemble_unit 两处 splice（公共/实现）+ 投影期 plain-only 检查盲区。修复范围：合并末对最终 imports（公共/实现各自）做全局绑定名去重诊断（source.module-duplicate-import）+ 反例测试（含 plain-after-#in 变体）。其余核验 pass：AC-01..04 证据映射、树干净、零调试打印、fmt、旧面 grammar-id 不变（七组并集复跑）、Spec delta SD-01..03 备好且与实现一致、双评 ct_ins 为确定性重复（无行为差，备注）。next: work 修复 R1 → 定向复核 → 复审。
2026-11-13 stage:review / final r2 (R1 复核+终判) | reviewed_commit: d1b38449c | outcome: pass | R1 修复核验：公共/实现两处落位 imports 全局绑定名去重（`duplicate_import_diag`），反例测试两变体（landed-vs-plain、plain-after-#in）均断言 `source.module-duplicate-import`；定向复核 core packages 28绿 / store 7绿 / av stdlib_assembly_1043 3绿（真Python腿）+ 受影响组 ace-source 重跑 PASS（run-rubytk23）。其余 AC 证据沿 r1 记录（输入未变复用）：AC-01 后缀/三轴/负控（provider_sources 5 + binding asm01×3 + loader 2）；AC-02 可见性/单实现/来源映射/旧兼容（asm06×3 含 impl 段归位 + stdlib 170 + ace-stdlib 2）；AC-03 两形态/阶段继承/局部namespace/单行use/能力门（asm02×2+asm03×3；"sys成员不入模块域"由代码路径核验——落位仅 imports/decls/requires，无 sys 名进入 units）；AC-04 五后端真跑+缓存/恢复（av 3 + store 7）+ 七组并集（run-t9u5j7p_+run-qoiqet7w，2421 executed 0 ignore）。健康：树干净、零调试打印、fmt、旧面 grammar-id 不变。Spec delta SD-01(context-source-assembly.md 新增)/SD-02(ace-binding-and-assembly §PLAN-1043)/SD-03(hash-syntax §#in) 已审与实现一致，frontmatter supersedes/new/touched_goals 核对无误。pending_closeout_gates（merge owner）：冻结提交 d1b38449c 上其余六组按未变输入复用或按需重跑。next: merge（T-08/09）。
2026-11-14 stage:merge / r1 consolidated | plan_id: PLAN-1043 | plan_revision: 1 | outcome: pass | checkpoints: prepared(reviewed d1b38449c→rebase 全等) → landed(master ff-only = c6af4845b；rebase 旧→新映射 05ed31a2c→a9a3e8781/a55773462→2839b2af1/8ad1a30f8→1db781f39/7ef7f9919→83a170a84/d1b38449c→c6af4845b，`git range-diff` 五提交全 `=`) → canonical_verified(SD-01 docs/specs/stdlib/design/context-source-assembly.md sha256 2d8b6358…；SD-02 ace-binding-and-assembly.md 7c245a45…含 PLAN-1043 节；SD-03 auto-lang/comptime/design/hash-syntax.md 5e98a192…含 #in 节) → gate(冻结并集于交付提交：run-5w75ws9n，ace-source 518/ace-syntax 132/ace-semantics 275/ace-stdlib 2/stdlib 170/ace-a2x 235/ace-av 1091 = 2423 executed 全 PASS 0 ignore) → archived(本文件 docs/plans/archive/)。落地检查观察项：auto-lang 无部署产物消费本变更（编译器仓，release 二进制/web bundle 不随本包重建——如需发布另起任务）。批回归到期检查：last_covered=1015@2026-11-12T13:22，其后落地的 1017/1027/1042/1043 均非 5 倍数且收据 <48h → 不到期。consumers: 966（完整CTE消费本包最小#in）、967/946/972-975/1041（按覆盖合同消费装配/端口）、948（分母追加）。
## 10. 待澄清事项

**[2026-11-13 work blocker]** T-05 的 A2PY 执行腿与 T-06 对应 ASM-05 py 行在 PLAN-1042（drafting 0/8，无 ACE Python producer；旧 `trans/python.rs` 是 ASM-05 负控不可替代）交付前无法完成。用户三选一：(a) 授权下一目标交付 1042 后回来关闭 1043；(b) 等其他会话交付 1042；(c) 修约把 A2PY 腿移出本包。**2026-11-13 用户裁定：选 (b)——暂停等待 1042 交付**；**同日 1042 已交付归档（master ab1199386，`a2x-python` crate/`PythonTargetProvider`/pyrun 落地），用户指示恢复**：worktree 同步 master（eb745b1bd），T-05 A2PY 腿改为真执行并提交（a55773462），阻塞解除。

T-01在现owner核定namespace lookup/歧义诊断、#in AST阶段载体、OS判别值标量编码/target映射及manifest版本，保用户选定语义，不另造语法或runtime。完整C/Rust外库功能归967，OS原语归1039/1040，Python目标归1042；若现基础能力不够，先具名修订前置/分工，不能临时手填HIR或调用旧VM过门。
