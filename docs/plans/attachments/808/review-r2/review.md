# PLAN-808 independent final review R2

2026-10-18；plan revision **1**；outcome **needs_fix**。受审提交
`69961e0465a65e1e3fec8607194fede9db739f28`。本会话没有实施修改，按主检出当前
Plan、实际差异、当前源码及独立正负控制重建结论；没有创建其他 agent/工作树。
本轮为第二次完整 final review，不能以现有绿色总数代替剩余不变量。

## 冻结基线与边界

- 实施树 `D:/autostack/.wt/lang-808/auto-lang`，branch `codex/plan-808-dev`；
  base `2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 起止实施树 clean，HEAD、依赖、DLL、锁/配置及规范输入无漂移，见
  [baseline-current.json.txt](baseline-current.json.txt) 与
  [final-check-current.json.txt](final-check-current.json.txt)。主检出其他计划的在途改动保留。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；auto-term
  `7a0d0e8b718fb260641f0d09ada95a7346771cd3`；两依赖树 clean，只读消费。
- 实际 DLL `D:/autostack/auto-term/target/debug/autoterm_core.dll`，SHA256
  `c17ecca15a84f86cb26ad1f391490afe2e8b7420a369047c4bac3f6b8f0e8f58`；
  root Cargo.lock SHA256 `9a431eb9e6163c154798db92543028db38a29af70ddac008014cc48ebd6a8fe1`。
- 当前 SD 冻结副本 [spec-delta-frozen.md](spec-delta-frozen.md)，SHA256
  `483d81a1efcc59864c59d5527c9e6e6cac5a8423059bca969970cb1980b08982`。
  当前 vm/ui/mux/绘制规范及批次合同版本和 hash 均记录在 baseline。
- Cargo 命令仅在各自进程清空继承的 `RUSTC_WRAPPER`，未改配置。
  原沙箱不能完整访问工作树状态，使用授权的原隔离树完成定向测试；没有另建 checkout。
- 初次新增探针误用了 test-only 构造方法，编译失败后改用生产 `AutoVM::new`；
  缺少探针可执行文件的初次运行不计任何行为证据。下列 `*-current.txt` 为修正后实际重放。
  没有改实施源码来适配反例。

## Findings（合同内，不批准延期）

### F808-R2-01 / P1：owner 查询、剪枝和槽位替换持锁执行 provider 代码，重入死锁

AC-01/03/05；T-01/05/06/07。`crates/auto-lang/src/vm/native.rs:475..481`
在 `terminal_owners` 读锁内执行外部 `provider.owns_handle`；`:489..493`
在其写锁内执行 `live_handle_count`，并在 `retain` 内析构被移除的 provider。
`:442..446` 的 provider 槽位赋值也会在槽写锁内析构旧值。

独立 provider 使用 `Weak<NativeInterface>`，回调自身不持任何实现锁，满足轻端口
的 Send+Sync/无持锁回调要求。通过真实 `AutoVM` 的 `shim_term_cursor_row` 或
`shim_term_free` 进入查询/剪枝，让 `owns_handle`、`live_handle_count` 或
旧 owner 的 Drop 重入 `remember_terminal_owner`，三种情况均卡在 `callback_enter`，
**3 秒内不完成**。槽位替换的第四种反例先正常剪掉零会话 owner，然后替换仅由当前槽
保活的 provider，其 Drop 重入 `terminal_provider()`，同样卡住。
四个不经持锁调用点的直接正控均完整退出且 exit0。

证据：[callback-probe-current.txt](callback-probe-current.txt)；可复用输入
[callback-probe.rs](callback-probe.rs)、[build-probes.ps1](build-probes.ps1)、
[run-callback-probe.ps1](run-callback-probe.ps1)。driver 每例截止 3 秒，超时仅终止自己
创建的探针并 wait 确认退出；不启动终端子进程，不遗留阻塞进程。

这违反 Plan §5.2 / T-01 §3.2 的“不持 registry 锁回调导致重入锁死”原要求，属于
R1 owner 历史/剪枝修复引入的调用路径问题。修复应在锁外执行 provider 方法和析构；
快照、并发修改的再协调及移除值释放均需覆盖，不能只把 `owns_handle` 单处移到锁外。
永久控制应包含四种重入、普通非重入正控及 concurrent set/query/prune 的有效所有者不丢失。

### F808-R2-02 / P2：释放后的旧视口缓存经 provider 切换变成零值

AC-01/03；T-01/05/06/07。`terminal_provider_for`（`vm/ffi/term_engine.rs:37..49`）
只认 `owns_handle` 的**活会话**；free 后 owner 查询不能找到旧 service，
`prune_terminal_owners` 又删掉其历史。因此 core 虽保留旧 CURSORS/VIEWPORTS，VM
带旧句柄的缓存查询却回落到当前 provider，无法读到它们。

实际 DLL + 生产 AutoVM + 真实 viewport/free shim 的对照：

| 路径 | VM 返回 | 原 service 缓存 / 冻结期望 |
|---|---|---|
| A 的 80×24 会话 free 后，仍使用 A | 80×24 | 80×24，正控 |
| free 后把 A 替换成 B，再查询旧柄 | **0×0** | **80×24** |
| A 的 90×30 活会话先切到 B，再 free 旧柄并查询 | **0×0** | **90×30** |

证据：[cache-probe-current.txt](cache-probe-current.txt)，输入
[cache-probe.rs](cache-probe.rs)。探针均 spawn `cmd /C exit 0`，显式 free；
外部 Arc 故意保留 A，使差异不能归咎于 service 被销毁。

违反 Plan §1/5.2 的旧返回值保持，以及 T-01 §1/3.1/8 和 SD-01/02 明确冻结的
“free 后光标/视口保留旧缓存”。R1 双活服务误 free 的旧例已闭合，但仅验证活柄 owner
不能证明这条完整不变量。修复应保留 retired handle 可观察值的归属或等价快照，且不重新
使用已释放 DLL 指针，也不靠永远强持旧 service 阻止所需资源释放。永久回归补上述两个
顺序和 cursor/viewport，保留重复 free 与未知柄缺席正控；不批准把旧缓存语义改为空值。

### F808-R2-03 / P3：修复测试新引入两条 unused_mut，健康门仍未通过

AC-05；T-06/07。`tests/plan808_terminal_domain.rs:1005..1006` 的
`mut reaper_a`、`mut reaper_b` 都不需要 mut，当前定向编译仍明确报告两条警告。
见 [plan808-tests-current.txt](plan808-tests-current.txt) 对应源码诊断。

R1 指定的未用导入和原单个 `mut reaper` 已移除，但“本计划新增文件零未处理 warning”
尚未闭合；这是修复轮新增的两个位置，不是全仓既有 408 条库警告。清除后重新定向编译，
不得全局静音。

## R1 旧例闭合与覆盖边界

| 原 finding | 本轮独立结论 |
|---|---|
| F808-R1-01 双活服务同号误 free | 旧例 CLOSED：ha=1/hb=2，旧柄 free 后 A_live=0/B_live=1；实际 plan808 永久控制含读/写/free、两 VM、默认共享。完整 owner/旧值不变量仍受 R2-01/02 阻断 |
| F808-R1-02 两 VmBridge 漏装 | CLOSED：原独立组件反例装配 Some，泵=1/剩队列=0；13 条中的新增测试实际覆盖两个生产构造与显式默认优先 |
| F808-R1-03 并发重复 free | CLOSED：独立受控 DLL 重放并发 free=1，在途 write 时 free=0；core 每会话 lease、退役检查和跨会话控制过 |
| F808-R1-04 service Drop 泄漏 | CLOSED：受控 Drop free=1；真实 DLL 显式 free/drop 的 OS child 均退出；永久实测另含 provider 末引用释放 |
| F808-R1-05 新增文件警告 | 原两个位置 CLOSED；整体 health 仍 OPEN，见新位置 R2-03 |

原反例当前提交重放：[r1-replay-current.txt](r1-replay-current.txt)。其中真实 child
PID 15052/9200 均由 probe 观测退出。不是只复用 work 的同名日志或 R2 目录中的早期输出。

## 实际验证

| 检查 / 实际入口 | 本轮结果与证据 |
|---|---|
| `cargo test -p auto-terminal-core` | 6/6，[core-tests-current.txt](core-tests-current.txt) |
| DLL env + `cargo test -p auto-lang --lib plan808 -- --test-threads=1 --nocapture` | 13/13，无 PENDING，[plan808-tests-current.txt](plan808-tests-current.txt)；缺必须符号的 panic 为 catch_unwind 预期负控 |
| 同轮构建的 test binary `ui::terminal --test-threads=1` | 41/41，[ui-current.txt](ui-current.txt)，定向/once/geometry/窗口/颜色/Mux 回声 |
| 同 binary `ui_gen::rust::tests::terminal_ --test-threads=1` | 3/3，[codegen-current.txt](codegen-current.txt)，既有 Rust 组件事件/scroll/scheme |
| 双包 `cargo check` / 当前 `cargo build -p auto-lang --lib` | exit0，[check-current.txt](check-current.txt)、[lib-build-current.txt](lib-build-current.txt)；后者生成外部生产探针实际链接产物 |
| `cargo check -p auto-lang --no-default-features` | exit0，[no-ui-current.txt](no-ui-current.txt) |
| 仅复制当前 core+consumer 到全新 `target/review808-r2/standalone-69961e0-current` | fresh `generate-lockfile --offline` → `test --offline --locked` **4/4**，实际 DLL env；[standalone-current.txt](standalone-current.txt)，仅调整消费方 path 为 `../core`，没有语言/UI 源码 |
| 独立 consumer `cargo tree --offline --locked --edges normal,build,dev` | 仅 core → libloading 0.8.9 → windows-link 0.2.1，[deps-current.txt](deps-current.txt) |
| `python scripts/modularity-audit.py --mode strict --no-cargo-check` | pass，0 violations/2 grandfathered，[audit-current.txt](audit-current.txt) |
| 差异全部 Rust 文件 `rustfmt --check --config skip_children=true`、`git diff --check base..HEAD` | exit0，无输出，[format-all-current.txt](format-all-current.txt)、[format-current.txt](format-current.txt) |
| 新增独立组合反例 | 缓存两种顺序 fail；重入四种路径 fail，四个直接正控 pass；见 findings |

792 的 work R1 receipt 已逐例核对 25/25 true，包括 fresh/relocated locked/资源篡改负控，
本轮没有把其总数冒称独立重放。Cargo manifests、交付生成器和锁/配置较 R1 审查未改，
相关依赖解析规则证据可复用；已改变的核心行为由本轮新复制/新锁及真实 DLL 回放验证。
当前未再次执行完整 792 driver；修复后 T-07 必须重新绑定最终输入，并按实际触面判断其受影响重放。
没有运行应用窗口或宣称 Vue/VM 全应用视觉 parity；本轮真实 UI 注册表/DLL/shim/生产组件构造
及 renderer 未改的覆盖边界与 R1 一致。未跑 `t/tv/tt/tb/tu/taa/tf` 整档。

## AC、任务与 Spec delta

| AC | tasks / 实现 | 结果 |
|---|---|---|
| AC-01 | T01/02/03/05/06，34 codec、core、默认/实例/feature | partial：34 形状、独立消费及无 UI 通过；旧缓存及重入失败 |
| AC-02 | T04/05/06，真实 UI adapter/组件入口/Mux | 当前默认 UI 路径 pass：两个构造已安装，41+3 与真实 DLL 投影过；不覆盖任意 provider 的重入 |
| AC-03 | T01/02/03/05/06，service/session/library/owner | partial：原并发 FFI、Drop、OS reap 与双活路由过；VM owner 重入和释放后旧值未通过 |
| AC-04 | T07、独立 fixture、777/792、残边清单 | partial：core 单向闭包、strict、fresh/locked 过；最终 owner/SD/交付证据需随修复再冻结 |
| AC-05 | T08、独立审查/health | fail：三项 findings 开放 |
| AC-06 | T09/10 | pending，merge-owned |

SD-01/02 的 live-only `owns_handle` + 零 live owner 剪枝描述不能同时支持已冻结的
free 后缓存保持，按 R2-02 修复实际行为后更新。SD 中锁/资源规则须与 R2-01 完整失败出口
相容；SD-03 两 VmBridge 接线现已与代码一致，沉淀时移除“R1 修复”执行日记表述，保留
当前持久规则。SD-04 对 display_offset/Mux 只登记 owner 路径，未改变算法，与现有回归相容。
未发布 canonical Specs 或 live ledger。

impact metadata 路径保持正确：new `docs/specs/auto-terminal-core/project.md`；modify
`docs/specs/auto-lang/vm/overview.md`、`docs/specs/auto-lang/ui/overview.md`、
`docs/specs/terminal-mux-model.md`；真实 goals 为 GOAL-003/007。media/i18n/其他残边
仍具名归属后续 R03，未当作本计划缺项或冒称 R03 整包完成。

## 路由及下次就绪条件

Plan 回 **executing，3/10**：保留 T-02/03/04；重开 T-01/05/06/07；T-08/09/10 pending。
原树 work 一次修完全部三项并提交，不在 review 私修实现。

这是第二次完整 final review 未通过。下次 handoff 除 finding→修复→正负控制外，须给出
“剩余不变量 / 之前为何漏检 / 受影响路径和状态 / 设计或 oracle 调整 / 当前结果”诊断表：
特别覆盖 live/retired/unknown handle、两种 replace/free 顺序、强 owner 与资源释放的区别、
查询/剪枝/槽替换中的 provider 方法和析构。原 13 条没有 retired 缓存切换断言，也没有
可重入 provider；绿色总数不能证明这两类规则。不得用 live-only 验收再次替代完整合同。

修复及最终验收通过后，merge 再在新冻结提交执行裸 `cargo t` + `cargo tv`、最终组合 777
及受影响 792；其后 Spec/ledger/index/归档/资源交接、wt-guard clean 后清理。
本计划没有改 `ui_gen` 发射算法，不机械触发整档 `cargo tu`；`tf` 仍归主检出批量门。
本轮没有合入、归档、清树或改动其他计划实施文件。
