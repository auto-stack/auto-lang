# PLAN-785 final review R2 — needs_fix

2026-10-16；stage=review；review_scope=final；plan_revision=1。
reviewer_identity=本聊天 Codex，独立于实施上下文；沿用 R1 审查上下文，未委派子代理。
Plan 输入 SHA256=97A53BCDDC577972AB771A3CB6E28BFB0076579576A8846E019E52A9855378E9。
worktree=D:/autostack/.wt/lang-785/auto-lang；branch=plan-785-dev；
reviewed_commit=0ac6e2ff5ced743974ce274bb0fa8cf424c14453；
repair_commit=012c89b99d99fb2cc9c87f92d23de7275c97af88；
prior_review_base=64c72d20c2e081b563760c1c4fe38761ccc6d00b；
original_diff_base=340ab95fd6b7871f492b65f721f4cef56b9f4c11。

## Finding closure

| Stable ID | Old example | Full invariant / current verdict |
|---|---|---|
| F-785-R1-01 (P2) | 原 6 项 `clipboard_provider --test-threads=8` 30/30 轮绿 | **仍未闭合**。锁覆盖显式槽写测试和两个 Plan 的桥/高层测试，但未覆盖普通高层入口间接装配。加入既有 plan320_tests 七项普通 VM 测试后，第14轮同一 :55 缺席断言失败（19 pass/1 fail）。这是同一测试隔离要求的兄弟路径，非新增产品范围。 |
| F-785-R1-02 (P2) | unused shim_clipboard_text 已删除 | **closed**。no-ui consumer 10/10，UI consumer+独立控制13/13；两形态均无 fixture unused_imports。UI rustc 唯一诊断为 Windows linker 创建 lib/exp 的 linker_messages 提示，未称整个仓库零警告。 |

## Remaining counterexample and cause

实际执行命令（由当前 Cargo 定向构建确认的 binary）：

```text
cargo test -p auto-lang --lib --features test-cases clipboard_provider -- --test-threads=8
target/debug/deps/auto_lang-dc6ba392f7d81a4f.exe clipboard_provider frame_provider host_capabilities plan320_tests --test-threads=16
```

后者最多30轮、首红停止；第14轮：

```text
running 20 tests
tests::plan785_clipboard_provider::plan785_provider::shim_absent_error_and_provider_delegation FAILED
panicked at crates/auto-lang/src/tests/plan785_clipboard_provider.rs:55:9:
缺席=无 provider
test result: FAILED. 19 passed; 1 failed; 0 ignored; 6894 filtered out
```

完整14轮原始记录：[p785-r2-mixed-repeat.txt](p785-r2-mixed-repeat.txt)。
复用输入脚本：[replay-isolation.ps1](replay-isolation.ps1)。这是标准 libtest 过滤后的轻量定向面，未运行整档。

路径：`tests/plan320_tests.rs:11/28/...` 调用 `run_with_capture`，通过
`lib.rs::execute_autovm_with_deadline`（:1579）触发 `install_ui_host_capabilities`，再由
`ui::clipboard_provider::install_process_default` 调用进程 if-absent 注册。
这些正常程序测试没有取得新增 `host_capabilities::test_default_slot_lock`。
因此缺席测试虽持锁，clear→AutoVM::new 之间仍能被补槽。新增 helper 的文档声称所有高层入口测试须持锁，实际接线只有少数具名测试。

修复应隔离需要临时清空/替换全局默认的测试与所有隐式宿主安装路径，而不只继续添加具名 provider 测试。
可采用不与普通 libtest 共享进程的默认槽验收，或在测试装配机制上完整隔离；选择由 work 决定。
不能简单让 shim 缺席测试不再断言缺席，也不能把 nextest 隔离当作 libtest 全局状态安全证明。
生产 provider 接口与互斥实现未变；本轮未观察到生产优先关系缺陷。

## Acceptance mapping and checks

| AC | Result | Tasks / artifacts / evidence |
|---|---|---|
| AC01 | pass | T02/03，core 与外部 consumer 供值链未变；重放无UI10/10、UI10/10 |
| AC02 | partial | T02/03/05，编码、UTF-8、None/false、快照/merge/实例覆盖、owned 生命周期均通过；默认槽测试隔离 F-R1-01 未闭 |
| AC03 | pass within declared feature boundary | T04，所有宿主接线与 adapter 生产体未变；UI13/13涵盖真实高层入口默认补缺及显式优先；新独立UI锁发行边界保持 |
| AC04 | fail | T07，独立反控与 SD 冻结完成，但完整隔离不变式仍失败 |
| AC05 | pending merge | T08，最终代码验收通过后裸 cargo t+tv，以及规范沉淀/合流/归档/批到期判定/guard-clean |

- 原反例定向 Cargo 6/6；同 binary 原过滤30/30轮通过（p785-r2-original.txt / p785-r2-original-repeat.txt）。
- 混合20项定向 libtest 第14轮19/20，exit101；没有忽略测试或改断言。
- `cargo test --offline --manifest-path test/fixtures/modularity/clipboard-provider-consumer/Cargo.toml`：10/10，exit0，无本 fixture 源码警告（p785-r2-no-ui.txt）。
- `python docs/plans/attachments/785/review-r2/replay-ui.py`：当前 workspace --locked 库重建，未改的 fixture 冻结副本+R1三个独立控制，13/13 exit0；fixture SHA256=1ab9ee3ea9c8b7ecf8cc239c42756684ed7046a72591f892f50300e0adcfc9d0。记录见 ui-consumer-record.txt / ui-consumer.txt / ui-consumer-build.txt。
- 当前修复触面的6个 Rust 文件 `rustfmt --check --edition 2021 --config skip_children=true`：exit0；`git diff --check 64c72d20c HEAD`：exit0。
- 未再跑 cargo check/779外部fixture：相对R1仅 cfg(test) 共享锁与 fixture 导入/格式变化；所有生产实现/配置/依赖指纹未变，复用R1两包check和779外部6/6证据。当前 libtest已实际编译修复后的测试路径；UI库重建及consumer两形态已重跑。不复用有失败风险的测试隔离证据。
- 未运行整档 t/tv/tt/tb/tu/taa/tf/docs_gen；未改实现、未合并。

## Baseline and Spec delta

实施已将 master@7c1eb2753a298fa280e6930c35a3a352dbf835f4 合入（受审提交第二父），R1同步义务此次确认履行。
当前 master@1cb4009eb1a2195ae39c69ace6faaa0123128ac5 相对该同步点仅 Plan/复审附件变化，无本计划生产或已跟踪配置差异。
PLAN-781 仍未合入，当前树 HEAD=5bc7e58ad2b3d19782e9f0b98532b45aeaab0624；未把其依赖标成已交付。
auto-down 组内依赖=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，未变。

Spec inputs 仍为 vm overview AB378E0B0567A96DF5302CBC96776402BB7481BE7CEF1EFB2CDCCA9FE2C25471、
ui overview BB13D7246FE96852C1E3ECF21D6778FDFD2E7A48FE7D650212EB86C837827A27。
冻结 [spec-delta.md](spec-delta.md) 仅更新绑定提交，语义仍与生产体一致；保留原两个 target 与 GOAL-003/GOAL-007 元数据，不发布 canonical Specs/ledger。
本树Cargo.lock=686D659A1322EEEF4A180F126DA03563059ED11C03D1E4BBA5398AA04DD5D719；
nextest.toml=0FBEA4B493E2437105D5CD68C616549B1A39AAAD6AE4B2F85FD83ADF4782D5A2；
.cargo/config.toml=A55E53FD140177F0B32CC7B24F577939E418DAC53973C79102B39370A2022445（与R1相同）。
结束时 HEAD 未漂移，git status --short 为空。

## Route

outcome=needs_fix；Plan 回 executing，T-05/06重开，4/8。
F-785-R1-02关闭，F-785-R1-01原例关闭但完整不变式未闭；不新增编号隐藏历史。
已连续两轮 final 非通过，下一次 work 交接须附诊断/覆盖调整表：明确为什么九个具名测试共锁仍漏普通高层入口，以及采用的完整隔离与永久组合反例；完成该调整再送审。
