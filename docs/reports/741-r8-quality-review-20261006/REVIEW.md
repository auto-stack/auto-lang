# PLAN-741 r8 合入后独立复审（2026-10-06）

**outcome: pass。上轮两个P3问题已修复，未发现新的阻塞项。**

## 基线与独立性

- reviewed_commit: `646ab16f5b76f9f32e55da491b33e76885a78d68`；diff_base: `d59298e95d4f3a8d88dab4d4b2c167139fae7e85`；plan_revision8，实际archived/44-of-44。
- 不参与r8实施；本会话负责r7复审，独立读取提交和运行控制，不以执行侧pass为证据，没有委派代理。
- 主检出clean，实施worktree已经清理。11993d94c实现、970eb09ba卫生修正、0af78400c复审、cd0d3db5b投影、af02c7285归档及47edf5e21真实导航更新均在当前祖先链。没有新建/移除worktree或分支，742/ABI检出不动。
- Category A：只运行Python/docs门，全部控制写隔离TEMP并回收；没有Cargo/native/docs_gen。核心src/tests、原型Cargo.toml/lock、verify-ac-741.ps1与auto-ac/auto-hir行为Spec零diff；原r5关键原始SHA逐项一致（manifest列明），所以复用已识别的58通过+1ignored、一键13项、正式60秒/共享锁核心证据。旧root382 warnings、daily37红/8待归因不称已解决。
- 原Plan压缩冻结、当前源/Spec/导航/ledger SHA与空规范增量冻结见source-manifest.json；仅记录review，canonical行为Spec/live ledger不修改。

## 本轮实测

1. **真实archive fixture 13/13通过，实际执行数与13 reported完全一致**；日志中包含真正运行的两个导航控制与四个身份负控制。删除了固定+2计数（fixture-real-archive.txt）。
2. **实际archive最终门通过**：44/44任务与metadata一致、17条真实父目录Plan链接、prototype README和两模块delivered+archive引用、8项P741-3..10 review指针均指向当前唯一741规范路径（actual-archive.txt）。
3. **三种active源各12/12通过**：准确43/44的部分executing、完整execution_done和完整reviewed；均生成一致的隔离repo骨架，两个导航控制真实运行（independent-results.json）。
4. **独立active/archive两种模式的12个引用控制通过**：各含正常对照、遗漏模块导航更新、缺必需README链接、模块/README/ledger错指向另一个已经存在的Plan。正常对照exit0，其余exit1；缺引用/目标不符诊断符合，ledger失败包含P741-8。错误文件确实存在，排除了只是FileNotFound导致拒绝的假验证。
5. **fixture自身失败传播通过**：仅在审查Python内存中让nav-stale控制错误返回0，不修改任何仓库脚本；实际main检测该case FAIL，整体exit1。恢复原函数/argv后清理TEMP。证明控制不是只创建而未运行，且不能失败后仍FIXTURES-ALL-PASS。
6. 原r7的缺文件、固定T35冲突、状态零变换均没有复发；旧计数记录的更正在Phase8当前证据中为13/12或交付双源25（archive13+active12），不将历史17/21混用到本轮。

## 全部AC与任务映射

| AC | 结果 | 任务 / 证据 |
|---|---|---|
| AC01..25 | pass，按同源指纹复用 | 原核心任务；r5完整逐AC映射/58+1/13项/正式60秒；当前核心、测试、依赖、验收脚本及行为Spec零diff；限定原profile，未声称完整Auto源码/AAC支持 |
| AC26..30 | pass | 历史生命周期/当前库版本/证据任务；本轮44/44/真实链接/指针与active/archive控制重新检查，库选择及native证据仅同源复用，不扩大既有保证 |
| AC31..32 | pass | T39..41：partial/full/archive正负源可重放、冲突/状态/诊断/实际导航/引用身份；之前的漏控制与假计数由本轮真实执行关闭 |
| AC33 | pass | T42/T44：active12和archive13包括实际导航控制；记录数=实际用例数，故意失效时整体非零；当前日志与正确总数一致 |
| AC34 | pass | T43/T44：README必需引用、模块与全部P741 review规范目标相等；active/archive存在但错误目标全部拒绝、正确对照通过，外部TEMP副本按repo规范目标校验；active ledger仅精确archive/741暂态容忍 |

## 规范、遗漏与状态

规范影响为空（supersedes/new/touched=[]），冻结原Phase8提案与hash在本报告目录；没有SD10、语言/HIR/native/ABI或依赖变化。既有SD01..09保持，不新增修复phase。
没有发现未执行的必做项、未批准延后或以CLI非零替代内部控制的绕行；r8首次审查的DBG输出已删。既有P741P8-R3输出观感/只读首个导航行的非阻塞硬化建议仍作为历史观察，不升级成额外必做范围。
当前真实源/导航/全部P741指针均正确，旧P741-R7-QA01/02清偿有本轮独立证据，不重新打开已通过核心任务。

next：保持PLAN-741 archived、revision8、44/44；只追加本次pass证据和复审记录，不重新合入实现、不沉淀Spec、不改live ledger或其它计划。原实施tree已清理，本轮没有新增tree需清理。

复现（设PYTHONUTF8=1）：`python -B docs/reports/741-phase7-lifecycle-fixture-nav/fixture_tests.py --source archive`；`python -B docs/reports/741-phase7-lifecycle-fixture-nav/final_assertions.py --plan docs/plans/archive/741-ac-hir-native-core.md --location archive`；`python -B docs/reports/741-r8-quality-review-20261006/reproduce.py <reviewed-repo> <TEMP-output-dir>`。复现器只写TEMP，固定审查HEAD运行以避免后续工具变更。
