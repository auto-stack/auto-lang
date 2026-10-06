# PLAN-741 r7 合入后独立复审（2026-10-06）

**outcome: needs_fix；两个P3验证问题。核心代码无新故障。**

## 基线与验证范围

- reviewed_commit=`18beff64b4ceefa1e7f4176296f5e37f77dd6da1`，diff_base=`8296ff305b8a15909903c61bb2ae15aeefd7373b`，plan_revision7。
- 主检出clean，原实施树已清理；验证dbec8dbe9修复及e797cf908归档位于当前祖先链，按实际提交审查，无新建/借用其它worktree。未参与r7实施；本会话负责r6复审，因此自行运行脚本/构造控制，不采信执行侧pass，没有委派代理。
- 原型src/tests、Cargo.toml/lock、verify-ac-741.ps1及auto-ac/auto-hir行为Spec相对前轮零diff；按指纹复用r5核心58+1ignored/一键13项/正式60秒/共享锁证据。Cranelift0.126.2、windows-sys0.59.0，历史auto-down=fba6563ed2148ce85e68208863159b4ccccac710，无新依赖。
- Category A：仅Python/docs检查，fixture全部写TEMP并回收；没有Cargo/native/docs_gen。root历史check382 warnings、daily37红/8待归因不被写成新通过或已解决。
- 原Plan压缩冻结、所有相关源/Spec/脚本/导航/ledger与空规范影响的SHA见source-manifest.json；本轮review不改产品、canonical行为Spec、live ledger或其它计划。

## 已修复与实际结果

1. 原r6归档找不到Plan、T35固定未勾选冲突、状态变换零操作已修。当前归档fixture直接运行exit0；动态冲突正确非零且匹配诊断；沙盒/tmp最终回收。
2. 独立用一致的repo骨架测试部分executing(40/41)、全完成execution_done与reviewed：三种源各10/10实际用例通过。完整archive源实际7/7通过。没有假定当前任务仍未完成。
3. 实际归档门正确验证41/41、14条Plan链接、prototype README、两模块真实delivered/r7+archive链接、7项P741-3..9 review指针，ALL-ASSERTIONS-PASS（actual-archive.txt）。当前真实导航全部正确，未将模拟当实际。
4. 独立遗忘auto-ac归档导航更新控制exit1且定位具体文件；正常对照exit0。新发现是下面控制执行/目标身份不足，不能把旧两模块实际断链重新当作未修。

## P741-R7-QA-01 / P3：导航控制未执行，却计入成功数字

`fixture_tests.py:289` 返回`len(cases)`；两分支创建`repo_ok,repo_stale`后未将导航控制加入cases，也未在循环后调用run_assert。`main:326`无条件`total_cases += n + 2`。

实测归档源输出只有7条PASS，却显示9 case(s)。交付fixture-run-final.txt只有active10+archive7=17条PASS，却称21，并在verification/Plan复审/merge收据写成“每矩阵两个导航控制全通过”。这是实际执行遗漏，不仅是文字舍入错误。两个repo骨架创建不等于控制运行。

修复：把全正确导航对照和遗漏导航负控制实际纳入统一case执行列表，真实检查exit/指定诊断/文件行，结果与总数都从执行记录派生；不得用固定+2或不运行就补PASS。旧17/21、7/9记录保留为历史并明确更正，新摘要/日志按实际交付HEAD输出。独立确认其中任一控制故意失败时整个fixture非零。T42，AC33，兑现AC31/32。

## P741-R7-QA-02 / P3：缺少必需引用和指针目标身份检查

`final_assertions.py:194..210`仅在README找到含741字样的链接时检查，没有找到则静默通过。导航和ledger只验证“存在且路径含archive”，没有验证解析到当前唯一PLAN-741生命周期文件。

隔离repo独立反例全部仍exit0/ALL-ASSERTIONS-PASS（independent-results.json）：
- 删除prototype README中的Plan链接，保留README文件。
- 保留741导航行label与delivered状态，将auto-ac的href改为已存在的archive/743计划。
- README保留741 label，将href改为archive/743计划。
- 将P741-8 review的file改成已存在的archive/743计划。

这些是原Phase7“正确唯一目标/引用覆盖”的受控反例；当前真实导航和ledger没有这样写错，不归因新native缺陷。仅file.exists()/archive路径无法证明导航身份正确。

修复：推导repo/docs/plans/(active或archive)/741-ac-hir-native-core.md的规范目标，必需README引用缺失即失败；README741引用、两个741导航行与所有P741 review指针解析后必须等于该目标。fixture传入外部副本时与repo内规范目标比较，不能错误要求等于TEMP副本；active ledger只允许规范archive/741目标尚不存在的已记录暂态。保留别的Plan路径作为假目标正可达、正确741目标正例、缺引用与错目标负例，失败定位具体文件/行/entry ID。T43，AC34，兑现AC32。

## 逐AC、规范与交接

| AC | 结果 | 证据与当前范围 |
|---|---|---|
| AC01..25 / 核心任务 | pass，按同源指纹复用 | src/tests/依赖/验收脚本与r5同源，复用原完整逐AC映射、58+1/13项/正式60秒，不扩大native profile |
| AC26..30 / 前轮生命周期/证据 | pass，限已承诺现有控制 | 实际41/41、真实链接/指针/数值/状态拒绝、当前库选择及计数旧证据；新增引用身份控制另见AC32失败，不声称最终门完备 |
| AC31 / T39,T41 | partial | 四种source状态当前真实控制通过、动态mutation/诊断正确；QA01漏导航控制且报告不实 |
| AC32 / T40,T41 | fail | 实际导航正确、旧遗漏更新能被拒绝；QA01没有真正运行该负控制，QA02缺引用/错误目标假通过 |

冻结规范增量为空（supersedes/new/touched=[]），无SD10或新的编译器/HIR/ABI行为。既有SD01..09与核心修复不重新打开；未发现新Rust警告/调试输出或未批准能力延后，r8仅验证脚本/证据/计划导航簿记。
继承用户“有问题则激活同计划追加phase”的明确授权，优先于skill默认Do not reopen an archived Plan：r8/executing；仅重开T39..41全部同ID行，新增T42..44，完成38/总44。旧r1..r7交付/pass/merge保留，P741-3..9归档指针激活期间暂缺，最终merge恢复；不占新ID、不碰742/743/ABI。

复现：在审查HEAD检出设PYTHONUTF8=1，运行`python -B docs/reports/741-phase7-lifecycle-fixture-nav/fixture_tests.py --source archive`及`python -B docs/reports/741-r7-quality-review-20261006/reproduce.py <reviewed-repo> <new-TEMP-output-dir>`。reproduce.py由本轮观察器固化，只读原脚本，构造隔离repo骨架并清理，不修实现。实际结果/旧日志均保留。
next:work修复T42/43 → 独立review → 真正archive44/44及双门重放、全部指针恢复、销账、guard收据。本review仅记录，不实施代码修复。
