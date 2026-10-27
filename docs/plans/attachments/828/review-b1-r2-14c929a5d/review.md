# PLAN-828 第一批修复受影响面复审 R2

2026-10-22，Plan r3；review_scope=phase（第一批 T-02/T-03/T-04 修复）；**needs_fix**。冻结实现 `14c929a5d9d54a90106b8e62b058e98e75a28b9f`，修复基面 `d93a9e37386fc6f8cb079b31c546df6ce37fdb5e`；唯一实施树 `D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`，进入与结束 clean。主检出交接簿记 `89d088472`。未撤销后批提交，后批及整份最终验收不由本次放行；Plan 保持 executing 0/11。

本轮由本 Codex 审核上下文独立取证，未实施该修复、未启代理；不编造不可见底层模型/会话身份。review skill 与共享合同、当前 Specs、Plan/SD/接口输入及 514 个源码配置文件哈希见 [baseline.json](baseline.json)，结束复核见 final-state.json。该文件继承的 base/first_batch_commit 是原 Plan/原批历史基面；**本次 repair diff base 是上段 d93a9e373**，见 [repair.diff](repair.diff)。sibling auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，未消费/未改动。

## 原五项关闭核验

| Finding | 已证修复/关闭范围 | 完整不变量结论 |
|---|---|---|
| B1-C01 P1 | 上轮越界 owner、范围内错误 owner、越界 param、伪造 intrinsic 清单全部拒于相应门；合法 owner=0/1 采纳。真实 verify_function 已接入 | **原语义无效工件反例闭合**；“同内容”仍有 R2-01 的语义身份缺口，不能凭这几个例子授整个恢复/复用面 PASS |
| B1-C02 P2 | 实际 decoded body span digest 双等式生效；原错 span 工件跳过，冷回退/后续 hit 正常 | CLOSED（本轮受影响来源控制+r1 矩阵保持） |
| B1-C03 P2 | 已有真实查询入口、版本发布与 staleness 门；基本已读失效/未读保持探针通过 | **PARTIAL/OPEN**：依赖仍挂 consumer 名，而非具体受检产物；重读和别的 remint 会抹掉旧产物前提，见下文 |
| B1-C04 P2 | 上轮 mixed mark_a/mark_b 工件正确持久化并恢复采用 3/3，plain 3/3，raw 原输入原断言通过；v2 版本键隔离与重绑接入 | 原跨表 callee 误解反例 CLOSED；后批完整身份/派发/性能不追认 |
| B1-C05 P2 | cold_bundle 在 close 显式释放；节点 oracle 前>0/后=0，源码拥有关系一致，旧 close/pin/PausedRun 保持 | 原新增深副本 owner 泄漏 CLOSED；新增 query 入口终态遗漏单列 R2-02，不重新宣布旧泄漏未修 |

上轮独立 12 控制 **12/12**：artifact 10/10（仅适配 v2 payload，原件与适配脚本保留）、roundtrip 2/2；新永久 b1 4/4。**原例转绿是真的，整批目前仍未通过。**

## B1-R2-01 / P1：内容身份遗漏写目标，合法编辑返回错误旧结果

**原合同**：AC-01/02/03/05、V-01/03/04、T-03/T-04，§5.1/5.3：正常 body 修改安装正确语义；历史 hit 的 body/code 必须是实际同内容，恢复内容必须等价当前源码。属于既有要求遗漏，**不是本修复新引入，也不是新语言支持项**。

**输入/实际入口**：[adjacent-controls.rs](adjacent-controls.rs) 的 `adjacent_assignment_target_changes_content_identity`：

```text
fn helper() int { var x = 1; var y = 2; x = 3; return x }
fn main() int { return helper() }
fn tail() int { return 9 }
```

只编辑 `x = 3` 的 `x` 为 `y`。真实 warmup/edit/run 同源冷编对照：

| 观察 | 期望 | 实测 |
|---|---|---|
| 编辑后结果 | 1（x 保持 1，赋值发生在 y） | **增量 3，冷编 1** |
| 是否复用旧 helper | 不可复用 | `reuse_hit:helper=1`，HIR/code 两个 Arc 均仍旧对象 |
| 源码/提交 | 已提交新源码并用新语义 | 源码已是 `y = 3`，却执行旧语义 |

同源合法正控把赋值值 3→4，增量/冷编均 4、无误 hit，排除输入未到达支持路径。独立期望来自标量赋值规则，不是测试自报计数。[最终 raw](adjacent-controls.log)保持失败出口 101。

**同根恢复反例**：[semantic-artifact-controls.rs](semantic-artifact-controls.rs) 在同键正确 envelope 的合法 v2 工件中，仅把 `Place.local` 从 x 的 0 改成 y 的 1。该 HIR **通过真实 verify_function**，canonical 与冷源相同、span 不变；恢复 `adopted=3, verified=3`，resident helper 的写目标为 1，冷源为 0。现有代码结果仍由独立 ABC 返回 3，不能将两者不同的语义当成受检 pair。[raw](semantic-artifact-controls.log)。此例不夸称恢复工件已被 AV 重新 lower；真实错误执行由前一个公开 edit 例直接证明。

**已证根因**：`cache.rs:702–706` 的 place canonical 只混 type/access，省略 `p.local`。session 的 hit 使用这一 content version；真实 verifier 能证明“这是合法 HIR”，不能证明“这是当前源码的同一程序”。本轮新增 verifier 阻断原 owner/index 错误，却不能修复语义键碰撞。

**关闭 oracle/范围**：上述合法编辑与冷编一致、变更函数确实换为正确 body/code；同键不同写目标工件不能被作为当前内容采用，后续 hit/二次恢复一致。完整盘点参与受检 body 语义和消费者行为的字段，区分内容身份与单独准入规则，修机制而非仅加此次字符串/变量名特判。保留合法同内容复用、未改函数身份、owner=0/1、原 malformed 拒绝、来源与 mixed intrinsic。若内容版本/键语义改变，按共享 store 不可同键两容规则隔离相应产物语义版本，不能用 Conflict 容忍吞掉旧错行。

**审查覆盖说明**：上轮只选择了非法 owner/index 等代表，未用同类型的合法写目标变化检验 canonical 的区分能力；R2-01 是上轮遗漏的既有合同反例。本轮补上实际 mutation + 冷编 + 值 + 两层身份观测，不将其误记为 GLM 修复引入或新增验收范围。

## B1-C03 / P2 残项：依赖属于产物，不能由消费者全局表刷新或清空

**原合同**：§5.1/5.2/5.3、V-03、C01-03、AC-01/05；复用所依赖的实际读取事实仍成立，变化则失效，不能重新盖章旧凭证。不要求新增 CTE/inline，本轮继续沿实现者提供的真实 query core 探针。

**两个实际重放**（[adjacent-controls.rs](adjacent-controls.rs)）：

1. `adjacent_new_body_fact_read_does_not_refresh_old_dependency`：main 在 helper=1 时读取依赖，helper 成功改为 2；再次 query 读到新版本，但没有重建 main；同内容 edit main 后，**invalidated=0、hit=1、reverify=0**。再次读取新事实抹掉了原受检对的旧前提。
2. `adjacent_history_retains_its_own_body_dependencies`：A=main(+1) 记录 helper=1；main remint 到 B=(+2)，B 无 body 读取；helper 改为 2；回到仍在 history 的 A。**hit=1、invalidated=0，旧 A 的 HIR/code 均被复用**。B 无 body 读取不意味着 A 的旧依赖消失。

**已证根因**：`body_reads` 以消费者名字为 key，`query_body_fact:950–953` 将旧 read 原地替换；`commit:1998–1999` remint 时删除这个名字的全部 reads。`prepare_edit:1693...` 查同一个全局表，history 中的各个受检对并不拥有各自的依赖凭据。永久测试反而明确把“B remint 后回 A 可直接 hit”写成正控，未区分两产物，因此表中“remint 恢复”不能作为完整关闭证据。

**关闭 oracle**：旧 A 的依赖在 A 仍可命中期间随 A 保留；新候选 B 的重新读取/无读取只影响 B 的凭据，取消/丢弃不刷新 A；命中/消费检查拟消费对象自己的依赖。上述两个反例要重验或正确拒绝旧对，普通未读/接口-only 消费者仍可复用，合法重建后的新产物可以记录当前事实。把 query 的记录/冻结/当前/候选/history 归属先写清并以真实入口断言。不要以保留所有历史版本的全部 reads 给新产物误失效来替代精确依赖。

当前 scalar lower 仍无 inline/CTE，未伪造“body-fact 被嵌入 guest 代码后已返回错误”的运行证据；这里是实际查询核心的凭据/失效失败。R2-01 另有直接错误执行证据。

## B1-R2-02 / P2：新增 query_body_fact 在 close 后继续接收并保留新依赖工作

**原合同**：F-05b 终态、§5.7/C05、T-03 新查询 owner；Closed session 仅保留必要诊断/修订信息，不能重新启动可变的依赖查询工作。本例是新增公开入口漏终态守卫的修复回归，不是原 cold_bundle 释放反例复活。

**重放**：`adjacent_closed_query_refuses_new_work`，warmup→close（cold_nodes=0）→`query_body_fact("main","helper")`。

**实测**：`closed=true` 仍返回 `Some(("d_helper", version))`，`body_fact_query:helper` 从 0→1；源码随后将新 read 插回 close 已清空的 body_reads。这不是只读诊断 getter，方法新建/更新记录和工作计数。

**已证根因**：`session.rs:941` 查询入口没有 closed 检查，FnEnv 的版本元数据仍存在，故成功走完整写入分支。

**关闭 oracle**：终态查询受控拒绝（Option API 可返回 None，或合同一致的受控错误），不新建依赖、不推进工作计数；幂等 close 与其他关闭入口保持，open 查询/合法读取保持。不要用丢弃函数返回值或追加 close 调用掩盖其已接受工作。

## 覆盖、原始结果与归因

| 路径 | 命令/产物 | 实际结果 |
|---|---|---|
| 原 artifact 10 控制 | artifact-controls-v2.rs；原件 artifact-controls-original.rs；适配只增 normalize+v2 引用表字段，预期不变 | 10/10 通过 |
| 原 plain/mixed roundtrip | 原文件原输入与断言，含直读诊断 | 2/2 通过 |
| 新公开编辑/依赖/关闭相邻控制 | adjacent-controls.rs | 5 项：1 正控过、4 失败（归并 R2-01/C03/R2-02 三 finding） |
| 恢复语义等价及 verifier 反控 | semantic-artifact-controls.rs | 2 项：1 过、1 败（R2-01 同根）；初始 entry 前提不成立的 raw 另存 |
| 新永久 b1 | function_incremental_828_b1 | 4/4 |
| 生产 session | function_incremental_828 | 18/18；显式跳过两个后批 512 负载，不冒称执行者的 20/20 |
| contract/t01r/r1 | 明确目标二进制 | 22/22、4/4、86/86 |
| 旧 close/pin/PausedRun | close-replay `close_review_`、`close_adjacent_` | 6/6、3/3；精确 filter 不把 include 的 96 项全报为新覆盖 |
| 独立 session 组合 | session-controls.rs 原件 | 6/6 |
| 格式/差异/冻结 | 精确 5 改动 Rust 文件 rustfmt --check；git diff --check d93..HEAD | exit 0/0；结束源码/依赖/config 与入口指纹相等 |

构建使用 `cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_b1 --test function_incremental_828 --test function_incremental_828_contract --test function_incremental_828_r1 --test function_incremental_828_contract_t01r --no-run --message-format=json`。Cargo 明确返回的 executable/rlib 与哈希见 [target-results.json](target-results.json)、cargo-artifacts.jsonl、linked-libraries.json；独立 runner=[run-controls.ps1](run-controls.ps1)，新失败控制实际 exit 101，外层输出日志命令 exit 0 不覆盖该失败。

本次 Cargo stderr 没有 warning；依赖缓存复用，不据此宣称全仓或 all-targets 无警告。ac-core/AV core/解析及测试配置相对 d93 未改，旧未变边界以已记录版本限定复用，不伪称本轮又执行全部 ac-core/source 目标。

**探针校正**：entry 从根改为现有 child 的初次探针假设会通过 verifier；实际 `verify.block-structure` 明确拒绝（entry 角色重复、原根/另一分支不可达），所以该初始失败归为审核前提错误，raw 保留 semantic-artifact-premise.log。已改为“必须由真实 verifier 拒绝”的正确控制，现通过；不作为产品 finding，不弱化 Place.local 的有效失败。原控制日志中的 `corrupted_pair_reused_after_edit=true` 是旧打印标签；本轮指向的是干净 fallback 对象，不能引用该标签声称原坏对象仍复活。

## 收敛与路由

这是第一批 phase 的第二次非通过，不能以“原五例全绿”递归进入只补个案的第三轮。下一轮先在同一现行合同/接口决策中核准两个机制：**完整语义内容身份**、**每个受检产物自己的依赖快照与候选发布归属**；再一次修齐本轮三项，补永久正反控、完整关闭表并冻结 SHA。原已修 owner/index/span/intrinsic/close 深副本机制保留，不回滚后批提交。[GLM 任务单](glm-next-round.md)。

SD-01/02/03/05 当前完成声明受 R2-01/C03 阻断，SD-04 的新增查询终态须对齐；冻结 [Plan（含 SD-01..06）](plan-input.md) 与 [当前接口合同](interface-input.md)已保存。canonical Specs/ledger 未写入，后批 T-05..T-09 完整验收、T-10 最终验收、最终 PASS 后 merge-owned 冻结裸 cargo t + 必要触面档仍 pending。未执行整档、实施修复、合入、归档或清树。
