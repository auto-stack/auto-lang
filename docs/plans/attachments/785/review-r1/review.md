# PLAN-785 final review R1 — needs_fix

2026-10-16; stage=review; review_scope=final; reviewer=本聊天 Codex，独立于实施上下文，未委派子代理。
plan_revision=1；原 Plan SHA256=34644F3899310ACF43F633045B1486C521C22061246FCD5A43C623B83C7CC513。
worktree=D:/autostack/.wt/lang-785/auto-lang；branch=plan-785-dev；
reviewed_commit=64c72d20c2e081b563760c1c4fe38761ccc6d00b；
base_commit=340ab95fd6b7871f492b65f721f4cef56b9f4c11。

## Findings

| ID | Severity / category | Evidence and correction | AC / task |
|---|---|---|---|
| F-785-R1-01 | P2 / new test regression | 新 adapter 测试 clipboard_provider.rs:58 起 clear/set/restore 默认槽，未与 plan785 测试的 serialized_default_slot 共锁；plan785 的 vm_bridge_creation_installs_process_default 也未取得该锁。并行时 shim_absent_error_and_provider_delegation 在 clear 后、AutoVM 构造前被另一个测试安装 provider，:55 的“缺席=无 provider”失败。6 项定向 libtest 第 2 次重放即 5 pass/1 fail。须给所有相关写入及依赖默认状态的测试统一隔离（包括桥构造/高层入口），保留 save/restore；nextest 的每测独立进程不能替代同进程隔离。 | AC02/04；T-05/06 |
| F-785-R1-02 | P2 / new health gap | 新 fixture tests/clipboard_vm.rs:498 导入 shim_clipboard_text 未使用，no-ui cargo test 与 UI rustc 重放均产生 unused_imports。删除该未用导入并复跑两形态编译；无需处理本计划之外的数百条预存 warning。 | AC04；T-05/06 |

F-01 命令：`cargo test -p auto-lang --lib --features test-cases clipboard_provider -- --test-threads=8`。
首次绿，随后对同一 cargo 生成 binary 最多 30 次有界重放，第 2 次停止于失败：

```text
auto_lang-dc6ba392f7d81a4f.exe clipboard_provider --test-threads=8
tests::plan785_clipboard_provider::plan785_provider::shim_absent_error_and_provider_delegation FAILED
panicked at crates/auto-lang/src/tests/plan785_clipboard_provider.rs:55:9:
缺席=无 provider
test result: FAILED. 5 passed; 1 failed; 6908 filtered out
runs=2 failures=1
```

这是标准同进程测试模式下的隔离缺口，未发现生产 provider 锁本身违反优先关系。

## Acceptance and evidence

| AC | Verdict | Tasks → implementation → evidence |
|---|---|---|
| AC01 | pass | T02/03 → host_capabilities/native/engine + 独立 fixture → no-ui 10/10，UI 同源 10/10；native 两 shim 不再直引 UI，catalog 名/ID 未改 |
| AC02 | partial | T02/03/05 → String/Bool 原编码路径保留，UTF-8、None/false、快照、merge、实例覆盖通过；R1 新反控验证默认竞争/clear 后旧 VM/NI Clone 独立/含 NUL 文本跨 provider mutation；F-01 使测试可复现性未闭合 |
| AC03 | pass (declared supported forms) | T04 → adapter 一行委托、全部既有 frame 宿主位置镜像接线；真实 run/VmBridge 创建路径通过，兼容安装不覆盖显式；feature/独立新锁限制明示；不新增 OS 写入检查 |
| AC04 | fail | T07 → 独立正反控已补，779 历史控制通过，SD 冻结；F-01/02 阻止 final pass |
| AC05 | pending merge | T08 → 冻结代码最终复审后才跑裸 cargo t + tv；本轮未跑整档，无合流/规范发布/清理 |

- `cargo test --offline --manifest-path test/fixtures/modularity/clipboard-provider-consumer/Cargo.toml`：10 passed，无 skip，0.59s；新增 unused_imports 原始诊断如上。
- `cargo nextest run -p auto-lang --lib --features test-cases -E 'test(plan785) | test(plan779) | test(clipboard_provider::)' --no-fail-fast`：10/10（785 5、779 4、adapter 1），run ID 2f779434-9c8d-4e34-af4c-2d33082f9954。
- `python docs/plans/attachments/785/review-r1/replay-ui.py`：workspace --locked 默认 UI 库重建/直编；冻结 fixture 10 + 独立控制 3 = 13/13，exit0；32 轮真实并发 set/if-absent/UI installer，旧 VM 默认清除后保留、Clone 独立，owned String 含 NUL/中文/emoji 且 provider 修改后旧值不变。见 extra-controls.rs、ui-consumer-record.txt、ui-consumer.txt。新增控制不写真实剪贴板。
- `cargo check -p auto-lang -p auto --locked --offline`：exit0；大量基线 warning 未冒称清零，新增 fixture warning 单列 F-02。
- `cargo test --offline --manifest-path test/fixtures/modularity/native-provider-consumer/Cargo.toml`：779 consumer 6/6（779-consumer.txt）。
- `git diff --check 340ab95fd HEAD`：exit0。生产修改无新增 debug print/TODO 延后；files/images/editor 保持非目标。未运行整档 t/tv/tt/tb/tu/taa/tf/docs_gen。

## Baseline / knowledge / dependency audit

Spec delta 见 [spec-delta.md](spec-delta.md)，源指纹在其中冻结；未修改 canonical Specs 或 ledger。
本工作树 Cargo.lock SHA256=686D659A1322EEEF4A180F126DA03563059ED11C03D1E4BBA5398AA04DD5D719；
nextest.toml=0FBEA4B493E2437105D5CD68C616549B1A39AAAD6AE4B2F85FD83ADF4782D5A2；
.cargo/config.toml=A55E53FD140177F0B32CC7B24F577939E418DAC53973C79102B39370A2022445。
main@631a83afce79a8fa5e1309e90f2f1f06e532db1a 相对 base 仅 783 experimental/docs/ledger/Plan 簿记差异，未改变 785 crates/已跟踪测试配置。Cargo.lock 被本仓忽略；本树与主检出本地 lock 指纹不同，本轮 UI 证据只绑定本树锁，不外推主检出本地锁身份。
PLAN-781 仍未合入（其树 HEAD 26fcb89f4243ec9d966d63fb94fab8b8d6283310）；
实施者记录的“final 前同步 master”未执行（785 HEAD 仍单提交基于 340ab95fd），本轮不替实施者 merge。
返工时履行同步义务并重查 781 状态；不能据不相邻 hunk 声明原依赖已交付。
PLAN-784 已归档；auto-down 依赖复核见本目录 baseline.txt。
预存 F-785-P01/P02 和独立 UI 锁边界保留，未用债务豁免两项新缺口。

## Route

outcome=needs_fix；Plan 回 executing，重开 T-05/06；T-01..04 保留完成，current_step=4/8。
work 在原树修复并提交，重放并行隔离反例、两形态 warning 检查及受影响验收；之后独立复审。
merge-owned pending_closeout_gates=裸 cargo t + tv（新冻结提交）、SD/ledger 沉淀、归档合流、批量回归到期判定、guard-clean。
