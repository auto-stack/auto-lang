# PLAN-820 r6 R3 — needs_fix

2026-10-24；final readiness / affected repair + targeted resource review；本聊天 Codex，独立于实施上下文，未启动代理。已读 R2 具名根因/关闭表，逐项对照代码。受审树 D:/autostack/.wt/lang-820/auto-lang，plan-820-dev，HEAD e8497f2f8d275873f55b4d7c0e9dd6ac1b75e3bf，repair base baf4d7d49492444cfd8063495a0bbd26af1b1d86，原 base 37e51253689631ffa61a9af58041c5f6646a7569。前后 HEAD/clean一致；diff-check exit0。

入审 master 83d3333ec7a65e63a26e77d02d7accfb915f3721；Plan SHA256 6E2A8A105A3E3FB4F59F87A5DE21AC72AC0A7A7D89F7C419DF4A3312BF8C19F7；SD SHA256 50B2176B9E9AB83017A26235C0A680B25B771D315C2DC06056FC1B910DF727D8。实际 Plan 为 executing7/9，按完成声明进入再审；不改语义 revision。

## 既有 findings

### F820-R2-01 CLOSED

独立 incremental 7/7、0 ignored，0.02s；另用 R2 原始失败输入重新编译重放：[源码](failed-edit-replay.rs)、[原始输出](failed-edit-result.txt)、[运行器](run-replays.ps1)。结果：before42；edit返回DivByZero；after42且 identities_preserved=true；旧snapshot42；retry43，代际2（上轮错误安装后为3）。fast/apply_source已改为候选先run/门后publish，apply_caps显式失败恢复旧caps/code；源码检查各Result错误出口未见原先发布顺序。范围不冒称所有宿主panic/未来取消接口已实测。

### R2 health CLOSED（受影响 probe 测试）

Local0及相关未使用项已修；本轮 incremental/admission编译没有原新增warning。根仓定向4测仍显示大量基面warning，不混作本修复新告警。

### F820-R1-01 PARTIAL（原验收反控仍未实现真实操作）

已改进：parse计数来自实际解析；lower/admit调用点记录各def事件；保留生产bundle Arc身份；原 whole frontend路径已移除。尚未闭合的具体原因：data_session.rs:435..439 的两个注入分别仅 `ev.record_disk_read(bytes)` 和 `ProductionEvents::bump(&mut ev.verify_mir, &def)`。没有实际读取任何文件、也没有实际对未改def调用verifier；set_control_*的文档自己标为 simulated。ProductionEvents磁盘字段没有真实IO调用者。该控制只证明门会拒绝人为改大的数字，不证明实际额外工作必然被捕获。incremental 7/7不能据此宣称原R2要求的“实际IO/重验事件”关闭。

关闭范围：保持本次真实生产计量/身份改进；通过实际读盘及未改def的实际verify/decode/load操作注入，经真正统一计量入口/owner观测后使同一门判红，并保持零安装。可以源码确认正常路径无IO作为辅助，不能继续用模拟事件等价替代该 oracle。未要求扩展生产支持面或全套性能重跑。

### F820-R2-02 PARTIAL（skip关闭，漏必需族仍可绿，真实root ID还会误红）

GuardSelfTest 2正+5负通过；独立 admission Scope PASS，ignored>0原反例现已能拒绝。剩余：

1. acceptance 中 cache-compat/core/incremental-v13 仍传 `@()` required。实际运行[控制脚本](missing-id-control.ps1)，只给增量与缓存族各一个 `unrelated_control ... ok`，没有任何必需增量/缓存用例，却得到 exit0/acceptance PASS，见[输出](missing-id-result.txt)。这是验收器failure-propagation模拟，不冒称产品运行结果；控制未调用真实cargo/check或整档。
2. `$REQ_CORPUS/$REQ_V10`用无模块前缀名字，但真实root libtest ID为 `tests::ac_av_probe_tests::<name>`。独立实际 `cargo test -p auto-lang --features test-cases --lib ac_av_data_probe` 4 passed/0 ignored（11.18s），四行均带该前缀。现有测试二进制 `--list ac_av_probe_corpus_executes_green` 也确认完整ID。把该真实形状的成功输出送给原脚本 autodown（仅本次shell cargo输出控制），exit1报 missing required test。合法正控会被正式聚合误红；本轮没有为此重跑真实完整corpus。

关闭：每个必需族均绑定实际完整ID集合；用真实命令输出作正控，漏单项/整族/ignored/重复冒计为负控。当前ok行数==passed无法禁止重复同名的冒计，也应按集合/重复约束处理。此项是同一R2漏检机制的未闭余量，不是新产品范围。

## F820-R3-01 P1 — 布局推导独享预算导致整体验证超步仍接受（AC-04/05, T-04/06, V-07/12）

既有 §5.2/§5.4、contract-decision §8：验证≤2M工作步，类型图/路径/布局推导全部计步，共享上限不能每类各领一份。

实测输入：[layout-budget-replay.rs](layout-budget-replay.rs)，2100个各含一个前驱字段的非递归record，I32叶、合法返回42的完整函数；总类型/字段/指令均远低于族cap。命令 `pwsh -File docs/plans/attachments/820/review-r3/run-budget.ps1`（运行器链接受审core rlib；没有改实现）。[原始输出](layout-budget-result.txt)：`result=Ok("accepted"), elapsed=667.0439ms, minimum_layout_field_visits=2206050`。

原因已建立：mir_verify.rs:373..379对每个record只在共享Verifier上tick(1)，再调用compute_layout；layout.rs每次调用重置steps=0/全新memo，重新遍历其所有祖先。2100层的字段放置访问之和=2100×2101/2=2,206,050，仅该子项已超过2M（还未加DFS/其他验证），但各次局部计数均小于上限，最终返回CheckedMir。667ms低于wall不抵销独立work-step门失效；未声称当前小例造成超时或OOM。

关闭oracle：布局工作真正扣同一验证预算/截止时间，或共享memo消除重复访问且实际总计步在预算内；保留合法深图正控，超总预算反控必须以资源分态拒绝，不能靠另加语法深度限制或改invalid掩盖。复核重启/ABC同一验证路径，开发仅定向验证。

### 同一资源矩阵的既有覆盖缺口：V-04 overflow 并非不可达

av_data_820_resources.rs::av_data_820_layout_overflow_arm_is_defense_only只验证1024个I32字段宽度4096，注释用“每record≤8KiB”错误推断嵌套布局永不溢出。实际支持嵌套并允许多个字段重复前驱类型：62层二叉展开、63个类型（每类型2字段）即使u64逻辑布局溢出，远低于输入/表族预算。

[独立输入](layout-overflow-replay.rs)、[输出](layout-overflow-result.txt)：60层size4611686018427387904；61层9223372036854775808；62层compute_layout和verify均受控 `mir.layout-overflow`，无panic。**实现此例拒绝正确**；缺陷是必需V-04反例被错误标成不可达而未测试。关闭：将该实际溢出与前一层合法对照加入真实MIR/ABC准入回归，修正测试/SD证据，不需要放宽任何合同。

## 覆盖、SD与下一步

AC01/05 R1已关闭native/采样证据因相关路径未改保持有效；AC09本轮事务反例关闭，但F-R1-01计量反控仍partial；AC06聚合fail；AC04/05新增共享验证预算fail；AC02/03及其他V独立全核pending，不用已有执行方总数补成pass。SD01/02资源承诺、SD05验证闭包、SD07事件/失败描述须待修复与实际Spec对账；SD03/04/06未完成最终内容/hash审查。依赖祖先/全部Spec输入final核验pending，尚未发放T07 pass。

回executing4/9：保留T01/02/03/05，重开T04/06/09。work先更新具名根因/调整表，对上面两个未闭原finding作真实关闭，再修共享布局预算及其漏掉的overflow对照；提交后再审受影响项和未审面。未改实现/canonical/ledger，未合入、未整档/acceptance真实聚合；merge-owned裸cargo t+tv+独立workspace完整有限合同档仍pending。
