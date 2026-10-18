# PLAN-804 final review R2 — needs_fix

日期：2026-10-17，Asia/Shanghai。review_scope=final。当前Codex `/root` 独立于实施上下文；本轮沿用前次review历史，不是重新计数。模型/会话ID未暴露，未创建agent、新树，未改实施代码、canonical Specs或ledger。

## Baseline

- 工作树 `D:/autostack/.wt/lang-804/auto-lang`，branch `plan-804-dev`，受审提交 `12e8936281620f660883a30e90beaa846e61af0e`。
- diff base `b7ecea5e36ddf41d9effc49f0aba53cca47b99ad`；本轮修复diff从前次受审 `315ed5b5f325f8de12f7970981739cdb28ffdbd7` 起，7文件+492/-163。
- auto-down兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。锁/配置/Spec/技能/原Plan指纹见 [baseline](baseline.json.txt)。实际根锁SHA=`afcd298ef2f9c18b9970de66240d27de1e80e20930bd4cb6fad5b65fdee065fd`，与R1不同；未盲目复用旧依赖证据。
- 本轮主检出已含802规范/代码，804树未合流；所有实施测试都在804树，主检出仅写复审簿记。

## R1 closure

| finding | 旧例闭合 / 完整不变量 | 本轮证据 |
|---|---|---|
| F804-R1-01 | CLOSED | v04_3经ui::dialog门面，实际断言driver存活期live_owner==0；v04_1相反策略live_owner==共享窗；真save取消和默认名种子新增。dialog4/4、ui-dialog-only1/1（双腿）独立复跑 |
| F804-R1-02 | PARTIAL，仍OPEN | 真App handler内会话、真实效果队列、双App可区分状态有进展；测试仍手工镜像宿主分派/消费，见R2-01 |
| F804-R1-03 | CLOSED | 正式driver独立25/25；producer为12e893628、git_dirty=false，锁/source digest有原始清单绑定；fresh/重定位locked/负控/dev同源全过 |
| F804-R1-04 | 原格式/旧warning/线程源码说明已修；完整卫生仍PARTIAL | 新App测试引入3组warning，见R2-03；renderer原9处格式预存，不要求全库清理 |

## Open findings

### F804-R2-01 / P2 — R1-F02仍未经过真实宿主事件链

位置：`ui/iced/renderer.rs:36135–36176`，`v05_dnd_finished_reaches_initiating_app_over_real_chain`。

测试直接调用 `bridge_mut().call_handler("StartDrag",...)`，随后**自行**调用 `maybe_anchor_dnd_initiator`；又直接从DONE取效果，自行选 `dnd_finished_target` 并 `inject_native_event(...,"none")`。它没有让 `session::dnd_finished_subscription` 产出消息，也没有构造/投递 `DesktopEvent::DndFinished`，没有经过生产 `run_session` 的DM::App或DndFinished分支。状态断言虽是真App状态，但这些关键接线均由测试补上。移除生产订阅/发起锚定调用/完成分支中的接线，当前测试仍不受影响。

本轮独立复跑该测试exit0（0.79s），可证明“手工组合这些函数可交付”，不能证明R1要求的实际宿主链交付。work闭合行已承认update_inner/订阅泵不在单测面，这个限制不构成V05必验的批准缩减。

修复应走实际生产dispatch与订阅/完成事件消费入口。可用实机宿主验收，或让测试通过与生产共同的事件入口驱动真实消息；保留两个App、焦点他窗、真实会话与handler状态断言。禁止在测试中额外手工补锚/目标/注入来代替待验证的接线。与R1同一原合同，非新要求。

影响：AC03，V05，T05/T06；R1-F02延续。

### F804-R2-02 / P2 — 修复回归：App测试成功退出仍残留合成左键按下

位置：`ui/iced/renderer.rs:36074–36144`。驱动线程被detach。它发送ESC按下后DoDragDrop即返回，测试主线程标记session_done并完成断言；`LEFTUP`排在ESC后续两次sleep之后。单测进程可在驱动执行释放前退出，驱动被终止；session_done并不等待配对释放完成。

独立反控 [input-state-control.py](input-state-control.py) 用本提交已编译的同一测试二进制，证明运行前left/right/escape全部松开；测试exit0，退出后left=true，250ms后仍true。探针随后配对释放该测试注入的左键，最终三键均false。原始 [JSON](input-state-control.json.txt) 与 [子测试输出](input-state-test.log.txt) 保留。此前首次五会话复跑在首text会话“返回时Drop已完成”断言失败（10.24s）；恢复本轮测试输入后同代码五会话4.31s通过。这不能仅归为用户持续按键或安静桌面条件。

修复应持有驱动join handle，在成功、失败和提前返回路径中保证其按键配对释放、光标恢复及资源收尾完成，再允许测试/进程退出。正常返回的断言必须在收尾后；仅增加sleep或设置停止旗标不足。新增同一“测试进程结束后无按键残留”的反控，并验证App测试→五会话顺序可重复运行。

影响：AC03，V05资源/运行证据，T05/T06；本轮修复引入。

### F804-R2-03 / P3 — R1卫生旧例已修，新App测试仍有新增warning

`ui/iced/renderer.rs:35940` 未用VIRTUAL_KEY；`:35944–35945` 未用PeekMessageW/PM_REMOVE；`:36057` 忽略必须使用的ShowWindow BOOL。三组均来自本轮新增273行，不是预存警告。

证据：[dnd-app-chain.log.txt](dnd-app-chain.log.txt)，可定位上述源码行及rustc诊断。work只检查plan804文件或全库warning总数会漏掉renderer新增段；“default+native-dnd双档新增警告零”不成立。修复新增项并按实际changed-line范围清点，不清理其他预存warning。

影响：AC04/05健康门，T06；R1-F04完整不变量延续。

## 本轮独立验证

| 检查 | 结果 / 原始证据 |
|---|---|
| 真dialog（默认形态） | 4/4，8.80s；[dialog-real](dialog-real.log.txt)，含存活owner及open/save取消 |
| 真ui-dialog-only高层入口 | 1/1双腿，2.26s；[dialog-only-real](dialog-only-real.log.txt) |
| 外部no-ui consumer | 7/7；[no-wrapper](consumer-no-ui-no-wrapper.log.txt)。首跑仅sccache端口10013失败，去wrapper后成功，环境故障不算实现回归 |
| 新App链测试 | 1/1，0.79s；[log](dnd-app-chain.log.txt)。覆盖边界见R2-01，退出输入状态见R2-02 |
| 原五会话 | 首跑fail/10.24s，[原始](dnd-real.log.txt)；释放测试残留后1/1、4.31s，[clean-input](dnd-real-clean-input.log.txt)。两次结果均保留 |
| 当前锁上provider/兄弟族/COM/parser/adapter/目标helper | 56/56、3.672s；[定向回归](scoped-regression.log.txt)，不含env-off OS测试伪计数 |
| 默认与ui-dialog-only check | 均exit0；[default](check-default.log.txt)、[dialog-only](check-dialog-only.log.txt)，不等于lib-test新增warning已清 |
| 修复文件rustfmt | 6文件exit0；renderer只有原9处预存格式差异，新测试区域无差异；[fmt](fmt.json.txt) |
| git diff --check | exit0；未将它代替rustfmt检查 |
| 792正式driver | 25/25；fresh构建200s，重定位locked耗时见[原始收据](792-driver-receipt.json.txt)。[完整调用日志](792-driver.log.txt)、[producer/源码清单](792-delivery-manifest.json.txt)；未使用work的未绑定摘要冒充独立结果 |

## 验收 / 知识 / next

AC01 pass；AC02 pass；AC03 partial（R2-01/02）；AC04 partial（R2-03；792已25/25闭合）；AC05 fail；AC06 merge pending。双接口/三shim/装配生产逻辑未被本轮改写，单一OS实现及旧门面保持。SD01..04冻结于 [spec-delta](spec-delta.md)，其SHA单列；规范元数据路径与GOAL003/007维持。主检出802已合入的知识须保留，不得用804树旧清单回滚。未发布规范/ledger。

792绑定补充：producer git_head=12e893628且git_dirty=false；auto-lang目录digest=`02edc4e05879e18b40d1bcc949d10cd19d9b2c2a7e9a3f0339ada4bf5e0a4d33`（4453文件、33034112字节），root_lock_sha256与本轮baseline相等。报告中的11个crates条目包含10个源码crate+1个patch，不误读为804已消费主检出的auto-ast新闭包。与802/799最新master的合流后再核仍属merge责任。

此为第二次连续final non-pass。下轮work交接前须先给出诊断/覆盖调整表：为什么把真实生产消息入口排除在验收之外、为什么新测试清理未被子进程退出反控捕获、为什么renderer新增行未进warning审计。先核对改进后的oracle再提交“全部关闭”，不以总通过数替代。

Plan回executing，保留T01..T04完成，T05/T06重开，current_step=4/8，T07/T08未完成。next=原804树work关闭R2-01..03（同时闭合R1-F02/F04剩余不变量），提交后受影响复审。

merge-owned门仍pending：最终pass之后冻结裸cargo t+tv，实际master/802合流与777重绑、受影响consumer、规范/ledger/索引/归档及wt-guard清理。未改ui_gen实现不触发tu，未改aavm不触发taa；tf仅主检出到期单实例批量档。审查通过前不合入，本轮没有运行这些整档。
