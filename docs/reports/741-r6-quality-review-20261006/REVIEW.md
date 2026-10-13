# PLAN-741 r6 合入后独立复审（2026-10-06）

**outcome: needs_fix；仅两个 P3 验证/归档问题。核心 AC 没有新增故障。**

## 基线与范围

- plan_revision: 6；reviewed_commit: `d1adc03be39d2897f5dfd7090e0909095cf24465`。
- base_commit: `ad1fe269c21bc9021220645b52a32301e680f321`；交付实现 `91a76351e26e23aec5f57a4fff43d4b379493086` 是 reviewed_commit 的祖先。
- 已无 741 worktree，依据真实主线提交和收据审查。实际归档 Plan 是 archived、38/38；原文件压缩冻结在 reviewed-plan.md.gz，精确 SHA 见 source-manifest.json。
- 本会话负责上一轮 r5 复审、未参与 r6 实施；重建实际行为，不采信执行侧 pass 摘要。没有新建任务/子代理。
- 原型 src/tests、Cargo.toml/lock、verify-ac-741.ps1 从 r5 审查基线到本轮完全不变；auto-ac/auto-hir 行为 Spec 同源。Cranelift 0.126.2、windows-sys 0.59.0、历史只读 auto-down=fba6563ed2148ce85e68208863159b4ccccac710；未引入依赖。
- Category A：只运行 Python 生命周期检查和 PowerShell 库选择控制，未运行 Cargo/docs_gen。复用明确源指纹匹配的 r5 原型58通过+1ignored、一键13/13、正式60秒/共享锁及 root check/t/tv 原始证据。root daily exit100/37红（8项超出历史并集待归因）、check既有382 warnings不冒充绿/解决。
- 记录期间主线并行推进 743/742，记录HEAD见 manifest；上述741文件与审查HEAD零差异。本轮不改其它计划、Rust、canonical行为Spec或live ledger。

## 独立结果与 AC 对账

| AC / task | 结果 | 方法与证据 |
|---|---|---|
| AC01..25 / 既有核心任务 | pass，按指纹复用 | r5报告的逐AC映射、58+1、一键13、当前库nonzero/timeout/public60s；完整原型源码/测试/依赖/验收脚本差异为空；不扩大已确认profile |
| AC26..27 / 历史生命周期与证据 | partial | 当前归档计数、10条Plan链接、6项P741 review ledger真实通过；AC29导航缺口、AC28 fixture缺口见下 |
| AC28 / T36 | partial | final_assertions实际archive PASS；独立17/17正负控制PASS（数值错、缺字段、非数字、同ID冲突、准确部分executing、完整execution_done/reviewed/archived、状态错、active/archive断链）；交付fixture入口不能归档后重放 |
| AC29 / T36,T38 | fail | Plan自身10链接与P741-3..8全可解析，但两模块741导航仍executing且指向缺失active文件；当前ALL-ASSERTIONS-PASS未覆盖它们 |
| AC30 / T37 | pass | 脚本直接消费Cargo JSON唯一lib/bin、拒绝歧义、不枚举rlib；独立旧/新物理库混存控制确选Cargo当前库；现有60s重放日志与源hash一致；58=15+43+1ignored计数及root红保留，空Spec影响无新语义 |

受控输入/输出：controlled-inputs.json.gz、controlled-outputs.json.gz；矩阵见matrix.json；入口失败、仅规范化输入的7/8、真实归档PASS分别见delivered-fixtures.txt、normalized-fixtures.txt、real-archive.txt。
原入口命令：`PYTHONUTF8=1 python -B docs/reports/741-phase6-lifecycle-gate/fixture_tests.py`，在审查提交检出运行。
实际archive命令：`python -B docs/reports/741-phase6-lifecycle-gate/final_assertions.py --plan docs/plans/archive/741-ac-hir-native-core.md --location archive`。
Windows PowerShell设 `$env:PYTHONUTF8='1'`；库选择控制只执行已提交脚本的artifact选择段，模拟Cargo JSON并同时提供旧/当前rlib，未执行cargo/rustc/native。

## Findings

### P741-R6-QA-01 / P3：fixture与合入后的真实生命周期脱节

`fixture_tests.py:18,34` 固定读取 `docs/plans/741-ac-hir-native-core.md`；归档之后文件不存在，入口真实exit1/FileNotFoundError，尚未执行任何控制。
仅将输入改为由真实归档文本生成的active控制（状态executing、报告链接回active形，其余38个已完成任务保留），复用交付fixture函数，得到7/8；`negative-conflicting-duplicate-checkbox` 实际exit0、expected1。
原因是:70..74假定T35未勾选，固定把`[ ]`换成`[x]`，对完整Plan变成零次替换，反例不再制造冲突。这是fixture问题；独立实际翻转一行的冲突控制已证明validator正确exit1。

修复：fixture入口支持真实active/archive来源，生成隔离的显式active/archive控制；按真实同ID行状态翻转恰好一行并断言发生修改，不依赖T35未完成。针对部分executing与全部完成/已归档两种输入都运行受控矩阵，输出每例原因并按预期诊断检查，不能仅用无关非零掩盖错误。用TemporaryDirectory回收自有临时数据。T39，AC31；重开T36当前验收，保留历史证据。

### P741-R6-QA-02 / P3：两模块归档导航漏更新，最终门未检查

审查HEAD中 `docs/specs/auto-ac/plans.md:7` 与 `docs/specs/auto-hir/plans.md:7` 都还是
`executing（r6 Phase6；核心r5通过，验收/证据待修）`，链接 `../../plans/741-ac-hir-native-core.md` 实际不存在。
主Plan与prototype README已归档，6项ledger指针有效。r6 merge收据声称两模块已更新delivered+archive，真实提交与该声明不符（navigation.json）。
final_assertions仅检查Plan本体链接/ledger，仍输出ALL-ASSERTIONS-PASS，不能代替AC29明确列出的README和两个模块导航验收。

修复：把现有最终门扩到这两条741导航及实际含741链接的prototype README（根README目前没有741项），验证状态、指向当前唯一Plan及链接存在。active阶段正确指向active/真实状态；最终实际archive必须delivered/archive且指向真实archive。给遗漏更新导航的归档负控制，非零且定位具体文件/行；检查门不可只跑Plan/ledger。merge完成实际更新后再记录收据，不用本次再激活造成active路径暂时恢复来销账。T40，AC32；重开T38当前有效性。

## 规范、遗漏及下一步

- 空Spec增量已冻结，SHA见manifest；不产生SD10，不改变已有SD01..09。
- 无新增平台/能力/后端需求，也不重新打开已过native任务或强跑Cargo。未发现代码层新缺陷、debug输出/新增编译警告；现有生产零diff说明仅限本阶段复审结论。
- 用户此前明确要求失败则同741激活并追加Phase，此授权优先于skill的“Do not reopen an archived Plan”。本轮新增r7 Phase7：T39..41，AC31..32；只重开T36/T38所有同ID行，其余36个唯一任务保留完成，current_step=36,total_steps=41。
- 按顺序work修复fixture/最终导航门 → 独立review → merge真正归档/指针收口。旧r6交付与pass保留为历史，不覆盖当前两项失败；不修改live ledger，P741-3..8归档指针激活期间暂缺，最终merge负责恢复。
