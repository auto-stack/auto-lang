# PLAN-828 第一批 R3 窄修复复审（R4）

2026-10-22，Plan r3，review_scope=phase：**PASS**。B1-R3-01/P2 CLOSED；第一批 T-02/T-03/T-04 本轮受影响面通过，无新增阻断 finding。Plan 按用户约定保持 executing 0/11；本结论不是整份最终验收、后批 PASS 或合入许可。

## 冻结与独立性

- 同一实施树 `D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`。
- reviewed_commit=`a66d9d9e54e4db1de90edc078ed7d23d161e00d5`；base=`599cc15afee9fb964a8a88a243c2210ea0597aa2`；进入与结束 clean、HEAD/依赖/config 无漂移。
- 主检出交接簿记=`884aff1b52b9a7e527be0213007b1a4f8e64104f`；sibling auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，未消费/未改。
- Reviewer=本 Codex 独立审核上下文，未实施修复、未启代理，不编造不可见底层身份。技能/共享合同、Specs、[Plan（含SD01..06）](plan-input.md)、[接口决策](interface-input.md)及514源码配置输入SHA见 [baseline.json](baseline.json)，结束核对见 final-state.json。
- 实施差异仅 `cache.rs` 与 b1 永久测试，见 [repair.diff](repair.diff)。未修改实现/canonical Specs/ledger。

## B1-R3-01 完整关闭

can4 为每个 CallBinding 编码 param/arg，并用确定的 `!` 标记区分 moved=true；false 不加标记，两态不同。Binary 也显式编码 overflow，当前 Trap 单态有穷尽匹配。`can4` 格式头使当前生成的 body content version 与 can3 规则隔离；HIR/ABC/sidecar 现有生产/消费共享这一内容版本，未新增覆盖旧行或清库路径。正式 manifest/GC 深接仍按 T-07 验收，未借此声明已完成。

**上轮原件不改预期重放**：[canonical-controls.rs](canonical-controls.rs)，[原始结果](canonical-controls.rs.log)：

| 观察 | R3审查时 | 本冻结根 |
|---|---|---|
| 仅 moved=false→true，HIR PartialEq | 不等 | 不等 |
| canonical_same | true | **false** |
| 真实 verify_function 接受该变体 | true | true（没有削弱 verifier 或改探针前提） |
| 实际 restore 采纳/验证 | 3/3、3/3 | **2/3、2/3**，被改 main 不采纳 |
| resident main.moved | true | **false，与冷源一致** |

对应永久测试 `t03_r3_call_binding_moved_distinguished_at_restore` 还明确断言 `restore_hir_skip:canonical=1`。合法原始 call 工件仍能采纳；原 malformed owner/index、span、伪造 intrinsic、合法非零 owner、混合 intrinsic 正反控均保持。取消候选/迟到提交依赖正控按 R3 原输入通过，没有回退每产物依赖归属。

本缺陷未曾证明新的 i32 执行结果错误，本轮也不扩大为 owned Str/.move 语言支持；关闭的是当前工件内容身份及恢复准入边界。

## 定向验证

| 检查 | 实际结果 |
|---|---|
| R3 原 canonical 两控制 | 2/2 |
| R2 原 adjacent / semantic 控制 | 5/5、2/2；semantic沿用R3已确认的区分前提，输入/采纳oracle未变 |
| 第一轮原 artifact / plain+mixed roundtrip | 10/10、2/2 |
| b1 / session / contract / t01r / r1 | **11/11、18/18、22/22、4/4、86/86**；session明确跳过两个后批512负载 |
| 798共享调用方 cache_tier | **20/20** |
| rustfmt精确2改动文件 / diff-check | exit0 / exit0 |

构建命令：`cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_b1 --test function_incremental_828 --test function_incremental_828_contract --test function_incremental_828_r1 --test function_incremental_828_contract_t01r --no-run --message-format=json`；再运行 Cargo 明确返回的 executable，见 [target-results.json](target-results.json)、cargo-artifacts.jsonl。独立原控制使用 [run-controls.ps1](run-controls.ps1) 选择明确 Cargo rlib、记录哈希（linked-libraries.json），所有实际 exit=0。缓存兼容命令为同树 `cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test cache_tier -- --test-threads=1`，[raw](cache-tier.log)。

旧 close/pin/PausedRun 6+3 和 session组合6的 R3 证据仅限定复用：本轮 session/AV core/依赖/config 无改动，差异只有字段内容编码与新增测试；实际受影响内容版本/来源/历史另已由本轮 r1 86、b1 11、原19控制和cache20重放。没有把旧close证据冒称在本轮再次运行，也未重复跑512/整档。缓存依赖未全重编，不宣称全仓告警扫描通过。

## 当前范围与下一步

第一批已登记的 B1-C01..05、B1-R2-01/02 和 B1-R3-01 在已声明 scalar/source/HIR/query 接缝范围闭合。SD-01/02/03/05 的本批实现描述可保留为拟沉淀输入；最终SD全文、后继消费者与全计划规则仍由T-09/最终审查核对，canonical/ledger不在本轮发布。

**下一责任角色为 review：第二批 T-05+T-06 完整受影响面复审**，以当前冻结树而非旧168b6b565结果为基线；重点保持原 C02-01..05/V-04/05/06：实际单元派发payload身份和副本负控、三执行入口、异质callee绑定/局部表、真实链接拒绝及安装失败不改变root/source/revision/来源、旧在途链和关闭寿命。该包本轮没有追认PASS。

T-07恢复深接/manifest/GC/预算与512水位、T-08正式worker/IPC/P95/双线内存、T-09接口/SD/消费者、T-10整份验收仍按原任务；最终审查PASS后才执行merge-owned冻结裸cargo t+必要触面档。当前不得merge、归档或清树。未运行整档或修改验收语义/plan_revision。
