# PLAN-808 independent final review R3

2026-10-19；plan revision **1**；outcome **needs_fix**。受审代码
`69337f5416e5c54fa12850274cab572b81e2b7ea`。本会话仅复审，未实施修复、未创建其他
agent/工作树。已核读 work 的第二次失败诊断和设计/oracle 调整表（handoff §4.3）；
该表满足再次进入审查的就绪条件，但其并发和共享状态断言仍有下述缺口。

## 冻结基线

- 原树 `D:/autostack/.wt/lang-808/auto-lang`，branch `codex/plan-808-dev`；
  base `2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 起止实施树 clean，HEAD、依赖、DLL、锁/配置及规范输入无漂移：
  [baseline-current.json.txt](baseline-current.json.txt)、
  [final-check-current.json.txt](final-check-current.json.txt)。主检出其他工作的改动保留。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；auto-term
  `7a0d0e8b718fb260641f0d09ada95a7346771cd3`；均 clean、只读消费。
- 实际 DLL `D:/autostack/auto-term/target/debug/autoterm_core.dll` SHA256
  `c17ecca15a84f86cb26ad1f391490afe2e8b7420a369047c4bac3f6b8f0e8f58`；
  root Cargo.lock SHA256 `9a431eb9e6163c154798db92543028db38a29af70ddac008014cc48ebd6a8fe1`。
- [冻结 SD](spec-delta-frozen.md) SHA256
  `4653c651c40ccfc98dc260369051e63fbb9f895ab3993ffc63450aaf1447d7bc`。
  当前 canonical vm/ui/mux/绘制规范、README、批次合同 hash 与 R2 相同；
  当前 Plan、contract-baseline、SD 已更新并按新版本审查。完整来源版本见 baseline。
- 本轮 Cargo 命令在进程内清空继承的 `RUSTC_WRAPPER`；未改仓库配置。
  原工作树写入/构建使用授权的隔离树访问，未迁移工作目录或私建 checkout。

## Findings

### F808-R3-01 / P1：剪枝期间清空 owner 列表，活会话瞬间不可路由，真实 free 被丢弃

AC-01/03；T-01/05/06/07。`crates/auto-lang/src/vm/native.rs:576..582`
在锁内 `mem::take` 整个 owner 列表，再锁外逐项调用 `live_handle_count`，直到
`:610` 合并回去才更新 generation。该窗口中，`terminal_owner_for` 的快照为空，
前后 generation 相同，`:532` 直接返回 None。即使提早递增 generation，也不能把
仍为空且不再变化的快照变成正确 owner；增加重试数或 sleep 不是这个不变量的修复。

**确定性实际 DLL + 真实 VM shim 反例**：A 的 80×24 会话活着，VM 从 A 切到 B。
另一个线程只对共享 NativeInterface 执行剪枝，受控 wrapper 在 A 的
`live_handle_count` 暂停以暴露该窗口。VM 保持在原主线程，并未迁移 VM 跨线程。
暂停期内读取 viewport、调用 `shim_term_free(A_handle)`：

| 观测 | 实际 | 期望 |
|---|---|---|
| 剪枝前 owner / viewport | true / 80×24 | true / 80×24，正控 |
| 剪枝暂停期 owner / viewport | **false / 0×0** | true / 80×24 |
| VM free 返回后 A 的 live_handle_count | **1** | 0 |
| 恢复剪枝后 owner | true，A 仍 live=1 | 恢复列表不能补做已丢弃的 free |

暂停只发生一次，协调通道有 3s/5s 截止；当前失败调用耗时 0ms，不是压力规模或
调度概率问题。探针释放暂停、join worker 后，显式释放自己创建的 A 会话。

证据：[combinations.txt](combinations.txt)，输入
[combination-probe.rs](combination-probe.rs)，入口
[run-combinations.ps1](run-combinations.ps1)。诊断程序 exit0 表示完成观测与清理，
不表示上述期望通过。

这是 R2 剪枝修复引入的回归，违反原 Plan §2/5.2、R2 修复承诺的“并发
set/query/prune 不丢活 owner”。新的永久测试只要求 500 次查询中至少成功一次，
并在所有 pruner 结束后再查一次；它允许中间真实操作全部误路由。

修复须让锁外检查/析构期间仍可解析活 owner，并正确协调并发新增/移除；保留锁外回调
要求。补受控并发窗口中实际读/写/free 的断言，不能以最终列表恢复代替调用成功。

### F808-R3-02 / P2：retired 缓存只在释放者 VM 或提前读取者中保存，其他共享消费者仍丢旧值

AC-01/03；T-01/05/06/07。`vm/ffi/term_engine.rs:57..64` 只把缓存放进执行 free 的
本 VM；`:73..91` 在首次查询时才能从旧 provider 懒复制。其他 VM 或直接 core free
没有预填该 VM 的 retired 表。随后普通 free shim 触发
`native.rs:576..614` 的零会话剪枝，就把唯一可查询旧缓存的 provider 历史删掉了。

**实际 DLL + 真实 VM shim 的两种顺序**，所有消费者曾安装同一 A provider，并已切到 B：

| 场景 | 实际 | 期望/正控 |
|---|---|---|
| VM1 free 共享 91×31 会话；VM2 尚未读缓存；VM2 对未知柄 free 触发剪枝后读旧柄 | VM1=91×31，**VM2=0×0**，A 仍缓存91×31 | 所有消费者仍为91×31 |
| 同上，但独立 control VM 在剪枝前先读缓存 | 剪枝前后均91×31 | 证明成功依赖查询顺序 |
| 直接 `EngineService::free` 释放92×32会话；该 VM 对未知柄 free 触发剪枝后读旧柄 | **VM=0×0**，A 仍缓存92×32 | 均92×32 |
| 同上，独立 control VM 提前读一次 | 剪枝前后均92×32 | 懒复制正控 |

这里用未知柄 free 触发的是生产 shim 原有清理路径，该 free 自身应为 no-op，不能改变
另一旧柄的冻结返回值；没有只调用测试私有入口。A 的外部 Arc 始终保留，排除了
“服务已销毁所以缓存本来不存在”的解释。所有终端会话由探针显式 free。

同份 [combinations.txt](combinations.txt) 与可复用输入保存完整顺序。此为
R2-02 的**完整不变量仍未关闭**，不是新增支持面：原 Plan 要求保留进程共享服务、
跨调用/两 VM 语义及 free 后 CURSORS/VIEWPORTS；新增代码自己也声明支持 direct free
后的 retired owner 查询。现有永久回归只覆盖在同一 VM 里执行 shim free 再切换。

修复须使已绑定消费者的旧值不依赖“在剪枝前碰巧读过一次”，同时保留末引用资源释放，
不得用永远强持整个旧 service 或重触已释放 DLL 指针解决。补跨 VM free/后续清理、
直接 free/后续清理、先读正控、重复/未知 free，以及 cursor/viewport 两类值。

## 旧例闭合与实际验证

| 验证 | 本轮结果 / 原始证据 |
|---|---|
| R2 四种回调/析构重入 × direct/control | **8/8** 均 3s 内完成、exit0，[callback-probe-current.txt](callback-probe-current.txt) |
| R2 同 VM free→replace / replace→free 原缓存反例 | 四行期望值全中，[r2-cache-replay.txt](r2-cache-replay.txt) |
| R2 两条 unused_mut | 新位置已清除；本轮编译无本计划新增文件警告，R2-03 CLOSED |
| R1 原反例 | 双活句柄不误 free、并发 DLL free=1、在途 write 时 free=0、service Drop free=1、VmBridge 泵1/0；实际 child PID30392/27948 均退出，[r1-replay.txt](r1-replay.txt) |
| `cargo test -p auto-terminal-core` | 6/6，[core-tests.txt](core-tests.txt) |
| DLL env + `cargo test -p auto-lang --lib plan808 -- --test-threads=1 --nocapture` | **20/20**，无 PENDING，[plan808-tests.txt](plan808-tests.txt)；缺必须符号 panic 是 catch_unwind 负控 |
| 同轮当前 test binary `ui::terminal --test-threads=1` | 41/41，[ui-tests.txt](ui-tests.txt)，定向/once/Mux/几何/颜色/窗口 |
| 同 binary `ui_gen::rust::tests::terminal_ --test-threads=1` | 3/3，[codegen-tests.txt](codegen-tests.txt)，既有 Rust 组件消费 |
| 当前生产库 build / 双包 check / no-default-features check | exit0，[lib-build.txt](lib-build.txt)、[check.txt](check.txt)、[no-ui-check.txt](no-ui-check.txt)；全仓既有库警告未计作新缺陷 |
| 仅复制当前 core+consumer，fresh offline lock→offline locked test | 4/4，真实 DLL env，[standalone.txt](standalone.txt)；路径仅改为 `../core`，没有语言/UI 源码 |
| 独立 normal/build/dev 依赖闭包 | 仅 core→libloading0.8.9→windows-link0.2.1，[deps.txt](deps.txt) |
| 全部改动 Rust 文件 rustfmt check + diff whitespace | exit0，无输出，[format.txt](format.txt) |
| 777 strict/no-cargo-check | pass，0 violations/2 grandfathered，[audit.txt](audit.txt) |
| 新组合反例 | 剪枝窗口活 owner/free fail；两种共享/直接释放后缓存 fail；原/提前读取正控 pass，[combinations.txt](combinations.txt) |

本轮据 work 的新诊断检查了相应路径，认可锁外回调和纯值 retired 表的修复方向，
但不认可“最终恢复列表”及“释放者本 VM 的缓存”足以覆盖完整合同。
R1-01..04 原例继续 CLOSED；R2-01 旧重入死锁例 CLOSED，但其修复并发回归由 R3-01
具名；R2-02 原同 VM 顺序例 CLOSED，完整规则受 R3-02 阻断；R2-03 CLOSED。

work 的新 `r2-792-driver-receipt.json.txt` 已核读25/25 true，日志含当前原树生成/fresh
build/relocated locked/资源与发布负控。其完整 driver 本轮未独立重跑，不把 work 总数
冒称独立证据。配置、manifest、交付生成器与 R2 未变，解析规则证据可复用；改变的 core
由本轮新复制/新锁/真实 DLL 消费验证。修复后 T-07 仍须重绑最终交付输入和受影响检查。

所有本轮输入输出另存 R3。入场时 R2 的固定 `*-current` 日志已经是 work 重放的成功
输出，不能再把它当 R2 失败现场；R2 失败事实保留在其报告/本会话原始输出，本轮不覆盖
R1/R2 文件。没有运行应用窗口或宣称 Vue/VM 全应用视觉 parity；renderer 未改，实际
覆盖为 VM shim/UI 注册表/DLL/生产组件构造及组件生成回归。未运行任何整档 cargo t/tv/
tt/tb/tu/taa/tf。

## AC、规范增量与路由

| AC | 关联任务 | 结果 |
|---|---|---|
| AC-01 | T01/02/03/05/06 | partial：34形状、默认/无UI/独立消费过；并发路由与共享旧值未过 |
| AC-02 | T04/05/06 | 当前默认UI路径 pass：真实定向投影、两桥接构造、Mux、Rust发射回归过；不替代并发owner验收 |
| AC-03 | T01/02/03/05/06 | fail：FFI lease/free/Drop及OS退出原例过；新剪枝窗口使真实free失效，共享旧缓存仍丢失 |
| AC-04 | T07 | partial：源切割/单向闭包/strict/fresh-locked过；owner/SD/最终交付证据随修复重冻 |
| AC-05 | T08 | fail：两项合同内 findings，不批准延期 |
| AC-06 | T09/10 | pending，merge-owned |

SD-01/02 中“generation 覆盖并发 set/prune”“缓存在清理/释放后保持”的当前描述不满足
上述反例，修复后按实际不变量校准，不能据现实现状降低旧值/共享要求。SD-03 的生产
VmBridge装配继续与代码相符，SD-04保持display_offset/Mux算法及owner登记不变。
impact metadata 保持正确：new `docs/specs/auto-terminal-core/project.md`；modify
`docs/specs/auto-lang/vm/overview.md`、`docs/specs/auto-lang/ui/overview.md`、
`docs/specs/terminal-mux-model.md`；goals GOAL-003/007。media/i18n仍有后续owner，
不宣称R03整包完成。未修改canonical Specs或live ledger。

Plan → **executing，3/10**：保留T-02/03/04；重开T-01/05/06/07；T-08/09/10 pending。
下一轮原树work一次修复两项、提交，并更新不变量诊断表：解释为何“至少查询成功一次”
漏掉调用窗口，以及为何只复制释放者/提前读取者的缓存漏掉其他已绑定消费者。
永久验收须观察每个受控窗口中的真实操作结果，不能靠次数、sleep或最终静态状态闭合。

最终验收通过后，merge在新冻结提交统一执行裸 `cargo t`+`cargo tv`、最终组合777及
受影响792；再Spec/ledger/index/归档/资源交接、wt-guard clean后清理。未改ui_gen
发射算法，不机械触发整档tu；tf仍是主检出单实例批量门。本轮未修实施、未合入/归档/清树。
