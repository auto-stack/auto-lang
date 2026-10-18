# PLAN-808 independent final review R1

2026-10-18；plan revision **1**；outcome **needs_fix**。本会话未参与实现，直接从主检出合同、原实施树的提交、旧实现与独立反例重建结论；未创建其他 agent/工作树，未修改实施代码、合入或发布 canonical Specs。

## 冻结基线

- worktree：`D:/autostack/.wt/lang-808/auto-lang`；branch：`codex/plan-808-dev`。
- reviewed commit：`c346f107790f86b7d1d045331243b3ad7f6e2c7a`；base：`2046a025b9a445d35a02da2c72a1613259a0cb60`。开始与结束工作树均 clean。
- auto-down sibling：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；只读、clean。
- 实际 DLL：`D:/autostack/auto-term/target/debug/autoterm_core.dll`；auto-term `7a0d0e8b718fb260641f0d09ada95a7346771cd3`；SHA256 `c17ecca15a84f86cb26ad1f391490afe2e8b7420a369047c4bac3f6b8f0e8f58`。
- root Cargo.lock SHA256：`9a431eb9e6163c154798db92543028db38a29af70ddac008014cc48ebd6a8fe1`。
- 完整身份、Spec 各文件 hash/版本、配置 hash：[baseline.json.txt](baseline.json.txt)。SD 冻结副本：[spec-delta-frozen.md](spec-delta-frozen.md)，SHA256 `c8a68c441decba577c8b58f5e39df175bcf6a69d612ce7e9c16b6c1f26cdd948`。
- 出具结论前再次核对：[final-check.json.txt](final-check.json.txt)；提交、锁/配置、DLL、auto-down 未漂移，实施树 clean，最终反例两个子进程均已退出。
- 继承的 sccache 因端口排除报 10013，所有 Cargo 重放均仅在命令进程内清空 `RUSTC_WRAPPER`；未更改仓库配置。依赖守卫使用 Git Bash；System32/bash 首次 exit127 不作实现失败。

## Findings（均在原合同内，不批准延期）

### F808-R1-01 / P1：provider 替换会释放另一服务的同号句柄

AC-01/03；T-01/02/05/06/07。`crates/auto-terminal-core/src/engine.rs:128,175` 每个实例都从 1 分配；`vm/ffi/term_engine.rs:27,125` 对已有句柄始终取当前 provider；`vm/native.rs:420` 直接丢弃旧槽，没有 handle→owner 绑定。

**实际 DLL + 真实 VM shim 反例**：A/B 各成功 spawn 并读到各自 marker，均得到 handle=1。VM 从 A 切换到 B 后，经 `shim_term_free` 释放 A 的旧柄，结果 **A_live=1 / B_live=0**，所需结果为 A=0/B=1。既有 `provider_replacement_does_not_reroute_handles` 只给 B 注入 `without_dll()`，B 没有自己的活柄，掩盖碰撞，而且不验证旧柄继续路由到原 owner。

修复必须使已经暴露的句柄持续绑定原 owner、保留其寿命，并与新 provider 的句柄区分；仅把每 service ID 改成全局唯一后让旧句柄失效，仍不满足 §2/5.2 的旧句柄原 owner 要求。补两服务均有活会话、切换后读/写/free、两 VM、重复 free 和默认共享控制。

### F808-R1-02 / P1：VmBridge 独立组件入口没有装 terminal adapter

AC-01/02；T-05/06/07。`ui/vm_bridge.rs:634..662,780..805` 两构造安装已有各族 adapter 后直接构造 VM，均缺 terminal 安装；本轮仅在 `lib.rs` 两入口增加安装。SD-03 却写成两 VmBridge 构造已接线。

**独立进程实际组件反例**：清空 terminal 默认槽，调用真实 `VmBridge::new` 构造最小 AuraWidget，构造成功后槽仍 None。同样快照的 VM 默认泵返回 **0**，注册表仍有 **1** 条输入；显式安装 adapter 正控立即得到 Some。组件自己的 AutoVM 在同一构造路径快照相同的空槽，因此无 UI fallback 不会排空输入/投影/menu/geometry。既有 real-UI 测试手动 `vm.set_terminal_provider(provider)`，没有验证宿主入口。

在两处既有组件装配入口补 if-absent，并以独立进程验证两构造的真实生产消费和显式默认优先；不能只增加又一次手动注入的测试。

### F808-R1-03 / P1：安全 Send+Sync 服务允许同一裸指针被并发 free 两次

AC-03；T-01/02/03/06/07。`engine.rs:166..173,360..377` 在句柄表锁内复制 raw pointer，释放锁后调用 DLL，调用结束才移除表项。两个 `Arc<EngineService>` 线程可同时获取同一 pointer；后到的 free 即使被 DLL 内锁串行，仍会对已释放的 Box 再次 `from_raw`。

**受控 DLL 反例**：首个 free 在 DLL 内等待，第二线程也 free，同一 handle 观察到 **FFI free calls=2**，期望 1；正在 write 时也能让 free 进入 DLL。mock 永不解引用/释放 sentinel，避免为诊断制造真实内存破坏。对真实 auto-term 源码的只读核对：`autoterm-core/src/ffi.rs:490` 在每柄锁内无条件 `drop(Box::from_raw(h))`，该锁并不验证句柄还活着。

这是新公开安全多线程服务的既定生命周期要求；原 glue 的同型窗口不能作为本计划豁免。应增加每会话的存活状态/操作租约，free 原子退休并等在途操作结束，保护完整 FFI 操作寿命；保持表锁不跨 FFI/UI。补同时 free、free 抢先于待执行读写、在途操作、不同会话不互堵及失败出口控制。

### F808-R1-04 / P1：独立服务 drop 丢失会话 owner，子进程仍活着

AC-03；T-01/02/03/06/07。`engine.rs:78..89` 直接拥有 Library 与 `HashMap<i64,usize>`，没有 Drop/会话 RAII。最后一个 service Arc 被释放时只丢整数表，不调用 DLL free；Library 可以先于活会话结束。

**真实 DLL/OS 对照**：探针启动自己创建的 PowerShell child，让其写 PID 后 sleep。显式 free→service drop 的控制中，OS 等待确认 child 已退出；直接 service drop 后 **child 仍存活**。最终重放 PID/结果见 [probes.txt](probes.txt)。另受控 DLL 观察到 service drop 的 free 调用数=0（期望1）。为了使诊断安全，额外 observer Library 只在探针内钉住 DLL，防止未释放 reader 执行到已卸载代码；探针仅终止自己记录 PID 的 child，wait 确认回收。

为独立 service 的会话提供 owner/RAII 与明确释放顺序，provider 切换不得丢失旧 owner。**不要求每个 VM drop 杀死进程共享会话**；默认 shared 服务的旧共享语义继续保留。永久回归须覆盖活会话 service drop、provider 替换末引用、部分 spawn 失败和异常出口，并观测 OS 子进程/线程，而非仅断言私有表归零。

### F808-R1-05 / P3：新增测试仍有两条未处理编译警告

AC-05；T-06/07。`tests/plan808_terminal_domain.rs:646` 未使用 `TerminalHostProvider`；`:1006` 不必要 `mut reaper`。当前定向编译输出直接定位到本计划新增文件：[plan808-tests.txt](plan808-tests.txt)。移除未用项并定向复验。新核心/迁移文件 rustfmt 与 diff 空白门已通过；全仓既有警告未冒充本计划新增，也未全局静音。

## 独立验证与覆盖

| 验证 | 本轮结果 | 证据/边界 |
|---|---|---|
| `cargo test -p auto-terminal-core` | 4/4 | [core-tests.txt](core-tests.txt) |
| DLL env + `cargo test -p auto-lang --lib plan808 -- --test-threads=1 --nocapture` | 10/10 | [plan808-tests.txt](plan808-tests.txt)；这些现有断言不足以关闭 F01..04 |
| 已构建当前 test binary，`ui::terminal --test-threads=1` | 41/41 | [ui-tests.txt](ui-tests.txt)；含 Mux 三段回声、定向/once、geometry/window/palette helper |
| 同 binary，`ui_gen::rust::tests::terminal_ --test-threads=1` | 3/3 | [codegen-tests.txt](codegen-tests.txt)；onmenu/onkeydown/scheme/scroll 发射 |
| 双包 `cargo check` | exit0 | [check.txt](check.txt) |
| `cargo check -p auto-lang --no-default-features` | exit0 | [no-ui-check.txt](no-ui-check.txt) |
| 独立 consumer `cargo test --manifest-path … --locked` + 真实 DLL | 4/4 | [consumer.txt](consumer.txt) |
| consumer Git Bash `verify-deps.sh` | PASS | [deps.txt](deps.txt)，仅 core/libloading/windows-link |
| 仅复制 core+consumer 到 `target/review808-r1/standalone`；fresh `generate-lockfile --offline`→`test --offline --locked` | 4/4 | [standalone.txt](standalone.txt)；Cargo path 仅改为复制包 `../core`，无语言/UI 源码 |
| `python scripts/modularity-audit.py --mode strict --no-cargo-check` | pass，0 violations / 2 grandfathered | [audit.txt](audit.txt) |
| 新核心、UI adapter、shim、plan808 测试与 consumer `rustfmt --check --config skip_children=true`；`git diff --check base..HEAD` | pass | [format.txt](format.txt)，无输出为成功；未改源文件 |
| 真实 DLL owner 碰撞、受控并发生命期、真实 service drop、组件入口 | 四项缺陷复现 | [probes.txt](probes.txt)、[probes.rs](probes.rs)、[mock.rs](mock.rs)、[replay.ps1](replay.ps1) |
| 背压单位独立控制 | PASS，1,500,000 bytes→1 MiB 与旧臂相同 | [legacy-backlog.rs.txt](legacy-backlog.rs.txt)。调查初期曾误读相邻 pending-bytes helper，核对完整旧函数后撤回；不是 finding |

未运行应用窗口或宣称 Vue/VM 全应用视觉 parity；本轮复现为真实 VM shim/UI 注册表/实际 DLL 链与既有组件生成验收。renderer 绘制算法未改，view/input 服务装配的实际缺口已具名，后续修复需补真实生产入口控制。

792 work 日志中的 25/25 fresh/relocated locked/负控已逐项核读，但本轮没有把该历史总数记作独立重放 pass；当前所有代码必须修复后再冻结 T-07 的交付输入。`t07-792-driver-receipt.json`、`t07-777-observe.json` 被全局忽略，尚非 tracked；后续收据必须以 `.json.txt` 等持久路径保存或显式入库，不能仅留下 worktree 临时路径。

## AC → tasks → implementation → verdict

| AC | 关联任务/实现 | 结论 |
|---|---|---|
| AC-01 | T01/02/03/05/06；core/provider、VM34 codec/default/feature | partial：34 形状/无 UI/独立消费过；owner 路由和组件装配 F01/02 fail |
| AC-02 | T04/05/06；terminal_provider、原注册表、生成 Rust 消费 | partial：定向/once/几何/颜色投影/Mux/发射现有回归过；默认组件入口 F02 fail |
| AC-03 | T01/02/03/05/06；service/library/raw table、provider slot | fail：F01/03/04；成功/无效/缺符号、顺序 free 的现有正控保留 |
| AC-04 | T07、独立 fixture、777/792/残边表 | partial：Cargo 单向闭包/源切割/strict/fresh core 已复验；修复后的交付输入和 SD 需重新冻结，R03 media/i18n 残边仍有明确 owner |
| AC-05 | T08、独立 AC/SD/health | fail：五项开放；不授 reviewed |
| AC-06 | T09/10 | pending，merge-owned |

## Spec delta 审核

SD-01/02 中“每实例句柄跨 service 返回缺席且不会误释放”与实际碰撞矛盾；“DLL 内锁保证生命周期、不需额外机制”不足以支持 free 安全；独立 service drop/Library 清理规则缺失。SD-03 声称两 VmBridge 构造已安装，与实际源码不符。SD-04 owner 路径与现有 display_offset/Mux 规则相容，既有回归通过，但不能代替默认生产装配验收。

元数据路径/目标类型保持：new `docs/specs/auto-terminal-core/project.md`；modify `docs/specs/auto-lang/vm/overview.md`、`docs/specs/auto-lang/ui/overview.md`、`docs/specs/terminal-mux-model.md`；`touched_goals` 为 GOAL-003/007。不在 review 改 canonical Specs/ledger。按原合同修复并据最终行为修订草案；若改变共享句柄/ABI/支持面，先明确修订合同，不用弱化 SD 关闭 finding。

## 路由与 pending 门

Plan → **executing，1/10**；重开 T-01/02/03/05/06/07，T-08 保持未完成；T-04 原 adapter 实现及定向正控保留，服务/装配改变后仍需重放受影响证据。旧 work 完成叙述作为历史保留，以本轮任务标记和 findings 为准。

原树 work 一次修完所有 findings、提交、给出 finding→修复→正负控制表后再终审。不得在单项修复/再送审运行整档。`cargo t`+`cargo tv`、最终组合 777 重绑/受影响792、Spec/ledger/index/归档/guard 清理仍是 merge-owned pending；本轮未执行这些整档或合入动作。UI 生成算法未改，不机械触发 `cargo tu`；如修复触面改变则重新判定。
