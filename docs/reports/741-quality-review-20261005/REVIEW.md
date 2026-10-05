# PLAN-741 r2 合入后复审（2026-10-05）

**结论：needs_fix。** 本次确认上一轮四个P1缺陷已修复；44项原型测试、all-targets零warning、
fmt和README实例通过。新增边界检查仍有一个P1与两个P2缺口。
它们不能由旧复审pass或本轮44项绿覆盖。此次仅复审，不修改实现或Spec，不自动激活/执行计划。

## 基线、范围与证据

- reviewed_commit：965b368a20db7c97fab7d1b0d51863b3ccca0f11；plan_revision=2。
- Phase2合同基线3a7967262；修复主线提交c2dfe84ed、db7fccbd5、33e18aa61，
  Spec/ledger交付52c67d3df。52c67d3df为当前HEAD祖先，非仅采信合并收据。
- 来源：归档741的r2合同§5.5/AC-09..15、canonical auto-hir/auto-ac Specs、
  上轮报告/反例，以及当前源码。当前归档文件frontmatter实际status=reviewed，
  与归档位置/merge receipt的archived声明不符，另作低优先级簿记项。
- 未参与741修复实施；独立detached检出运行，实施源码无WIP。
- 依赖：Cranelift0.126.2、既有锁文件、Rust1.98.1、Windows x64 SDK/rust-lld。
- 新反例/helper只位于本报告资料或scratch，不改产品代码、原有测试或CI门禁。
- 本轮实际运行：cargo test --locked -- --test-threads=1（44/44）、
  cargo check --all-targets --locked（零warning）、cargo fmt -- --check、
  上轮reproduce.ps1、README的check/build/trace及额外反例。
- 相对c1ac219e7旧生产代码存在其他开发线vue.rs变更；本次不声称整仓零diff。
  Phase2相对其合同基线3a7967262的根生产面须单独区分；不重跑无关UI/全量tf。
  旧主仓check/t/tv引用归档历史收据，不算本轮新跑或全仓质量结论。
- 原始日志与输入相邻入库，冻结在本提交；不依赖已移除worktree才能查看。

### 输入指纹 / Spec增量快照

| 来源 | SHA-256 |
|---|---|
| docs/specs/auto-hir/project.md | ae1a898fe12d5cd9f1d883fb68564049355556b488bc995b894f0976babe9660 |
| docs/specs/auto-ac/project.md | 9679401e686e5de6ed11bc678ea0866d630f26fdfc7890249332a5a44e997f03 |
| experimental/ac-core/Cargo.lock | 2b67753a80d4dd3bd35d3fd417628acc3f1d81c0029739637d64d8d54af85462 |
| experimental/ac-core/src/verify.rs | 2091454643f54446ca04ff7f497a682a74cef27d98d519ea38470e7769ed5260 |
| experimental/ac-core/src/native.rs | 69ef4577b3d2089b73c2c4ee652d31111b8b4fb44a55cceec9ca2b24859eef88 |
| experimental/ac-core/src/link.rs | 6eb8c5ab0d3f3bfa41f9e252073db7fd7b9080ab3fae35c1633258ec848a7566 |
| experimental/ac-core/src/main.rs | 67e7667737d88c0c0458d5b4ae7ec8be76da713532b45bbc60aeceb1bbc1ceb0 |

SD-03的bindings映射/arg双射/函数-body双向owner已对应实现；
SD-04的命名与既有失败回滚路径已落实。但“全部子进程硬截止”受R2-QA-02反例挑战，
“失败清理暂存”受R2-QA-03挑战；SD-01的块结构/全部可达承诺受R2-QA-01挑战。
本轮不改canonical Spec以迁就失败。Spec影响metadata沿用r2合同，不新增目标/ledger组件。

## 旧发现的实际关闭情况

| 原发现 | 本轮观察 |
|---|---|
| P741-QA-01 | bool-local与bool-return check/build均0；原生bool-if=3，bool-abi=5；icmp_bool归一到i32 |
| P741-QA-02 | binding-valid check/build0，native swap=9；binding-invalid在verify拒绝，arg双射负例通过 |
| P741-QA-03 | shared-body check/build1且verify.owner-mismatch；同签名共享与冒领负例通过 |
| P741-QA-04 | 原收据最终路径目录占位：构建失败，exeChanged=false/objChanged=false，旧exe仍exit5 |
| P741-QA-05 | README三命令实跑：check0、add native5、trace native12/ba；链接和路径已修正 |
| P741-QA-06 | rustc/reg均接入执行器；1s超时与大输出测试通过。但硬截止整体承诺仍有R2-QA-02漏洞 |
| P741-QA-07 | all-targets零warning、测试构建零warning、fmt通过 |

## 新发现与修复要求

### P741-R2-QA-01 / P1：结构错误后仍递归走查；入边计数不能证明可达/无环

位置：experimental/ac-core/src/verify.rs:649–739，尤其:684及:726。
check_blocks只数入边，入口被引用虽记录error，随后仍调用walk_block；
没有先对全部块做环检测并在结构失败时阻止递归。
两份极小文本均经过真实binder：
- fixtures/block-self-cycle.atom：入口if的then引用入口本身，check栈溢出，
  exit=-1073741571（0xC00000FD），stderr“has overflowed its stack”，未正常返回verify诊断。
- fixtures/block-disconnected-cycle.atom：入口直接return；两个额外块互相loop引用，
  每块入边=1，check却exit0。入度不等于从入口可达，非法死循环块进入Checked。

不是“深递归资源政策未定义”的边界：输入仅2–3个块，
现有AC-03及Spec已明确禁止块环/不可达块。
此缺口此前已存在，未认定为本轮修复引入的回归。
影响T-03、AC-03/04/08及SD-01。
修复：数据流走查前做完整可达性/三色DFS或等价结构验证，
发现非法图后停止递归；后续走查亦防循环。新增自环、多块环、断开SCC反例，
要求check正常非零、定位到违例块，不panic/无限递归。

### P741-R2-QA-02 / P2：reader join不受硬截止约束

位置：experimental/ac-core/src/link.rs:109–113。
run_with_deadline在child.try_wait得到退出后直接join两个reader；
deadline仅在child尚未退出分支检查。如果子进程启动继承stdout/stderr的后代后退出，
reader等待后代关闭管道，join无限等，不再检查deadline。

使用本报告review-helper.rs实际调用库link::run_exe：
受测父进程快速spawn持有输出句柄的120s子进程后退出；
外层watchdog持续卡住。实测114.9633913秒时仍未返回，超过固定60s；
最后按精确路径/PID终止本次helper及其后代。watchdog-result.json记录UTC时间和结果。
无终止/修改系统工具。计时日志曾受JSON自动日期转换影响，已用原始ISO时间纠正，
不得将未纠正的8小时偏移当作实际耗时。

影响T-14，AC-14及SD-04。“用了deadline executor”不等于执行器全路径有deadline。
修复：进程等待+输出收集共同受同一单调deadline约束；限制reader等待，
合理处理继承句柄和进程树，保留诊断与回收策略。
补“父进程先退出、后代保持pipe”测试；1s测试也需覆盖退出后的收集阶段。

### P741-R2-QA-03 / P2：收据暂存失败遗留已链接exe

位置：experimental/ac-core/src/link.rs:309–311、main.rs:188–193。
publish_artifacts在写tmp_receipt失败时只清理收据，调用方仅删除tmp_obj，
已链接的staged.tmp_exe没有回收；备份阶段提前失败也没有统一回收该exe。

helper复制真实PE/COFF作为已有制品与合法暂存制品，
把进程唯一收据暂存路径预先设为目录，实际publish_artifacts返回link.receipt；
按main.rs原样执行tmp_obj清理后：
staged_exe_left=true；old_exe_unchanged=true；old_obj_unchanged=true。
证据receipt-prepare.txt。旧制品保护已修好，这项是更早失败路径的清理遗漏，
不是把原QA-04的“exe被覆盖”误报为仍未修复。

影响T-12、AC-12的无.tmp残留及§5.5失败清理承诺。
修复：显式stage所有权/RAII或统一失败清理，覆盖收据暂存、备份、发布所有失败出口；
保留旧制品及既有目录，不删除用户占位目录。
测试目前只覆盖收据最终路径目录占位，未覆盖准备阶段。

### P741-R2-QA-04 / P3：归档状态与收据不一致

归档文件docs/plans/archive/741-ac-hir-native-core.md:3仍为status: reviewed，
而r2 merge receipt声明已archived。会误导按frontmatter读取的工具。
本轮只记录，不修改状态或重开合同。修正簿记须保留既有历史收据与revision。

## 验收对账与下一步

| AC | 本轮结果 | 依据 |
|---|---|---|
| AC-01/02 | pass（原型边界内） | 原型独立依赖面/reader11绿；根其他开发线diff已明确区分 |
| AC-03/04 | fail/partial | owner与类型修复有效，但R2-QA-01打破结构门 |
| AC-05/06 | pass | native11、trace1、溢出和求值顺序均实跑 |
| AC-07/08 | partial | 常规CLI/44绿；新失败/截止缺口和非本轮重跑主仓门禁需明确 |
| AC-09/10/11 | pass | bool、映射、唯一body新增负例与原始反例翻转 |
| AC-12 | partial | 旧制品保护通过，R2-QA-03暂存清理失败 |
| AC-13 | pass | README check/build/trace复制运行+链接 |
| AC-14 | fail | 普通超时/大输出通过，R2-QA-02输出收集超时失败 |
| AC-15 | pass | all-targets与测试零warning、fmt通过 |

建议优先修R2-QA-01，再修R2-QA-02/03，并补对应反例；
修复后按实际变更范围跑门禁和独立复审。本轮不自动扩大/执行计划。
旧P1关闭证据保留，不回滚或覆盖旧报告；新增结论绑定上述当前HEAD。

复现入口：在专用修复/复审worktree构建原型后运行本目录reproduce.ps1；
输出到临时目录，子进程有外层截止。helper/fixtures是复审资料而非产品实现。
脚本快速部分以 -SkipWatchdog 独立重跑成功（reproduce-fast.json）；完整截止反例证据为 watchdog-result.json。
