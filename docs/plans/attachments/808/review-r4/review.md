# PLAN-808 independent final review R4

2026-10-19；revision **1**；outcome **needs_fix**。受审代码
`9f02a18dc8364cce12e1b62c84a670eb7d679bea`。本会话未实施修复，按主检出合同、原树
差异和独立正负控制重建结论；未创建其他 agent/工作树。已核读 handoff §4.4 的 R3
诊断/设计调整后进入本轮；认可原反例闭合，尚不能授予 reviewed。

## 冻结基线

- worktree `D:/autostack/.wt/lang-808/auto-lang`，branch `codex/plan-808-dev`；
  base `2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 起止 HEAD/依赖/配置/规范输入稳定，实施树 clean：
  [baseline-current.json.txt](baseline-current.json.txt)、
  [final-check-current.json.txt](final-check-current.json.txt)。主检出其他计划改动保留。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；auto-term
  `7a0d0e8b718fb260641f0d09ada95a7346771cd3`；两依赖树 clean、只读消费。
- 实际 DLL `D:/autostack/auto-term/target/debug/autoterm_core.dll` SHA256
  `c17ecca15a84f86cb26ad1f391490afe2e8b7420a369047c4bac3f6b8f0e8f58`；
  root Cargo.lock SHA256 `9a431eb9e6163c154798db92543028db38a29af70ddac008014cc48ebd6a8fe1`。
- [冻结 SD](spec-delta-frozen.md) SHA256
  `a6e18913c17259d8d347ca7aa0ac2f41a8c5e3999a69faf2807fb8b802ae0eed`。
  Cargo manifests/配置/锁较 R3 未变；当前 canonical vm/ui Specs 已因 PLAN-807 合入更新，
  按本轮新来源版本审查，不能复用旧版本的“剩余边”结论（见 F04）。
- 本轮 Cargo 命令只在进程内清空继承的 `RUSTC_WRAPPER`，未改配置。探针均链接本轮
  当前生产库，输入和输出另存 R4，不覆盖 R1/R2/R3 工件。探针初稿的导入/类型拼写
  在复审附件内修正后重编；失败编译不算实现失败或行为证据。

## Findings

### F808-R4-01 / P1：零会话 provider 在剪枝中复活，已暴露句柄仍会被短暂摘掉

AC-01/03；T-01/05/06/07。`crates/auto-lang/src/vm/native.rs:589..592`
锁外采样死项后，`:600..619` 仅按 Arc 身份将它们移出列表；它不协调采样后发生的
重新安装/新会话发布。`:625..628` 第二次调用 `live_handle_count` 后才重新加入复活项。
因此原来“全列表摘空”变成了“摘掉已复活的项”，仍有真实调用不可路由的窗口。

**实际 DLL、同一 VM 的生产 spawn→切换→free 反例**：

1. A 最初确实为空；剪枝的第一次 count 在此时采样 0，受控 provider 暂停返回。
2. 同一个 VM1 重新装 A，经真实 `shim_term_spawn_ex` 创建83×27会话，切回B。
   此时 VM1 读旧柄确实返回83×27，证明是已经暴露且已正常绑定的句柄。
3. count 返回其合法的先前采样0，剪枝据此移除A。在“移除后、复活重加前”的第二次
   count 暂停，VM1 查owner/读viewport/执行真实 `shim_term_free`。

| 观测 | 实际 | 期望 |
|---|---|---|
| 移除前 VM1 已创建柄的 viewport | 83×27 | 83×27，正控 |
| 复活重加前 owner / VM1 viewport | **false / 0×0** | true / 83×27 |
| 同服务另一个VM的 viewport | 83×27 | 83×27，服务仍正常 |
| VM1 free 返回后 A_live | **1** | 0 |
| 允许重新加入后 | owner恢复true，但A_live仍1 | 恢复列表不能补做已丢失的free |

VM1 始终在原主线程；工作线程只执行共享 NativeInterface 的剪枝，不假设 VM 可跨线程
迁移。两次暂停均有3s/5s截止。第一次计数在A为空时采样，和普通计数返回后线程被抢占
等价，并未让provider对同一时刻的活状态说假话。当前失败调用0ms返回；探针释放暂停、
join worker并显式free自建会话。不是压力/规模测试。

证据：[repair-probe.txt](repair-probe.txt)，输入
[repair-probe.rs](repair-probe.rs)，命令
[run-repair-probe.ps1](run-repair-probe.ps1)。exit0表示诊断和清理完成，不表示期望通过。

这是 R3-01 完整不变量仍未闭合，违反原 Plan §2/5.2 与最新 T-01 的“新增/复活 owner
保留、每次真实读/free有效”。修复须协调死项采样后的重新安装和会话发布，保证复活者在
整个窗口可路由；仍须锁外调用provider/析构。不能仅在最后重加、增加重试次数或sleep。
永久oracle补零→活状态、重新安装/真实spawn与剪枝交错、窗口内读/write/free的结果。

### F808-R4-02 / P2：新增4096条FIFO会遗忘已冻结旧值，属于未批准的语义缩减

AC-01/03；T-01/02/03/06/07。新 `crates/auto-terminal-core/src/retired.rs:20,35..43`
将进程级缓存限为4096条，满后移除最老条目。对未预读、owner已经剪枝的消费者，
`terminal_cache` 只能依赖此表；被淘汰后又回落到当前provider的0×0。

**实际真实会话 + 受控条目边界**：A的94×34会话经实际DLL创建/释放。两个已绑定VM
切到B、经未知柄free触发正常清理；early控制VM提前读取一次，late VM尚未读取。

- 发布时 global=Some(94×34)，early=94×34。
- 4095个其他条目后，目标global仍存在。
- 第4096个其他条目后，**global=None，late=0×0**；early及原A缓存仍为94×34。

同份 [repair-probe.txt](repair-probe.txt) 保存结果。边界条目通过公开
`EngineService::without_dll().set_viewport(...); free(...)` 产生，不访问私有Store，
不创建数千个PTY。这是新硬编码常量的语义边界反例，不新增Plan的性能/资源规模承诺。
仅需一条真实被观察会话，所有真实会话均已释放。

原 Plan §1/5.2、此前冻结矩阵以及 canonical旧行为均要求保留free后的cursor/viewport，
未批准“其他4096次条目插入后可以归零”。修复轮把“有界FIFO”写入草案不构成对原合同
的授权变更；plan_revision仍为1，草案还同时声称所有消费者在清理后始终可读旧值。

须保留所承诺消费者的冻结可观察值，不能静默淘汰仍可被句柄查询的数据。若拟改变支持
合同，需具体说明语义差异并走有授权的合同修订，不能靠自加容量上限关闭旧finding。
`retired_store_keeps_values_and_evicts_fifo` 当前只插入一个条目，未断言容量/淘汰边界，
其通过不支持这个策略。需补边界、未预读消费者、owner销毁及重复/未知free正控。

### F808-R4-03 / P3：本轮新增测试又引入三条unused_mut

AC-05；T-06/07。`tests/plan808_terminal_domain.rs:1702,1754,1798` 的
`mut reaper`/`mut reaper`/`mut reaper2` 均未发生可变使用。当前定向编译逐条报warning，
见 [plan808-tests.txt](plan808-tests.txt)。这是修复轮新增位置，不能归入全仓既有408条
库warning。清除后定向编译确认；不全局静音。R1/R2原位置警告的闭合记录继续保留。

### F808-R4-04 / P2：SD-03仍把已落地807的diff族列为未迁，须与当前规范调和

AC-04/05；T-07。当前主线 `6955e317859b34693ca4f941c07b4957c0459190` 已沉淀
PLAN-807：vm/ui overview 明确diff四入口9915/9916/9917/9921已经迁出，且从当前
“未迁边界”删除。808的SD-03却仍拟写入“vm/native.rs口径剩余diff族9915..9917
（PLAN-703/807）”，handoff残边表亦未更新已完成状态。

这是并发 canonical 输入更新带来的**规范增量冲突**，不是9f02a18dc的终端运行回归。
新来源hash/version已绑定baseline；对照保存为
[canonical-input-diff.patch](canonical-input-diff.patch)。修复T-07的拟沉淀文本和剩余
owner清单，说明807已迁、只需合流；不得在merge盲写旧残边覆盖当前知识。不要求在本轮
私自修改canonical Specs/ledger，也不扩张808实施范围去重做807。

## 原例闭合与本轮验证

| 验证 | 结果 / 原始证据 |
|---|---|
| R3原剪枝常活窗口、跨VM/直接free原缓存例 | 全期望值命中：暂停期true/80×24/free后0；共享91×31、直接92×32均保持，[r3-replay.txt](r3-replay.txt) |
| R2四路重入 × direct控制 | 8/8完成、exit0，[r2-callback-replay.txt](r2-callback-replay.txt) |
| R2同VM两种缓存顺序 | 四行期望值全中，[r2-cache-replay.txt](r2-cache-replay.txt) |
| R1安全/装配/释放原例 | 均保持：双活不误free、并发free=1/在途free=0、Drop=1、桥接泵1/0；实际child PID28332/23268均退出，[r1-replay.txt](r1-replay.txt) |
| DLL env + `cargo test -p auto-lang --lib plan808 -- --test-threads=1 --nocapture` | **22/22**，无PENDING，[plan808-tests.txt](plan808-tests.txt)；缺必须符号panic为catch_unwind负控 |
| `cargo test -p auto-terminal-core` | 7/7，[core-tests.txt](core-tests.txt) |
| 当前test binary `ui::terminal --test-threads=1` | 41/41，[ui-tests.txt](ui-tests.txt)，定向/once/几何/Mux/颜色/窗口 |
| 当前test binary `ui_gen::rust::tests::terminal_ --test-threads=1` | 3/3，[codegen-tests.txt](codegen-tests.txt) |
| 当前生产库build、双包check、no-default-features check | exit0，[lib-build.txt](lib-build.txt)、[check.txt](check.txt)、[no-ui-check.txt](no-ui-check.txt) |
| 仅复制当前core+consumer，fresh offline lock→offline locked test | 4/4，真实DLL env，无语言/UI源码，[standalone.txt](standalone.txt) |
| 独立normal/build/dev依赖树 | 仅core→libloading0.8.9→windows-link0.2.1，[deps.txt](deps.txt) |
| 全部改动Rust文件rustfmt、diff空白 | exit0，[format.txt](format.txt) |
| 777 strict/no-cargo-check | pass，0violations/2grandfathered，[audit.txt](audit.txt) |
| 新修复路径/边界反例 | 复活窗口free失效、FIFO导致late旧值归零；两个原值/early/另一VM正控过，[repair-probe.txt](repair-probe.txt) |

R1旧安全/生命周期原例继续闭合；R2重入原例继续闭合；R3原常活窗口和少量共享旧值例
闭合，但完整的复活路由及旧值保持仍受F01/F02阻断。work的新第三次失败诊断已确实调整
oracle为逐次查询和受控暂停，方向认可；缺少零→活状态与新FIFO边界。

work新 `r3-792-driver-receipt.json.txt` 已逐例核读25/25true，含fresh/relocated locked/
资源与发布负控；本轮未独立重跑完整792，不冒称其总数为独立结果。manifest/配置/
交付生成器未变，解析规则证据可复用，改变的核心由本轮新复制/新锁/实际DLL消费验证。
修复后T-07重绑最终输入，按实际触面重放受影响交付检查。

没有运行应用窗口或宣称Vue/VM全应用视觉parity；renderer未改，实际覆盖为真实
VM shim/UI注册表/DLL/桥接构造和组件生成回归。未运行t/tv/tt/tb/tu/taa/tf整档，
未修改canonical Specs/live ledger，未合入/归档/清树。

## AC、规范增量与下一步

| AC | tasks | 结论 |
|---|---|---|
| AC-01 | T01/02/03/05/06 | partial：34形状/独立消费/无UI过；复活路由与被淘汰旧值失败 |
| AC-02 | T04/05/06 | 当前默认UI路径pass：真实投影、Mux、两桥接构造与Rust生成回归过，不替代并发owner验收 |
| AC-03 | T01/02/03/05/06 | fail：原FFI安全/Drop/退出过；复活窗口真实free丢失，冻结旧值仍可丢 |
| AC-04 | T07 | partial：strict/源切割/fresh-locked过；当前SD并发输入冲突及最终证据待修 |
| AC-05 | T08 | fail：四项findings开放 |
| AC-06 | T09/10 | pending，merge-owned |

SD-01/02需按F01/F02恢复完整路由/旧值语义；有界FIFO与原承诺的矛盾不得直接沉淀。
SD-03需按F04与最新807知识调和，终端生产装配本身仍与代码一致。SD-04只登记owner路径、
display_offset/Mux规则不变，与定向回归相容。impact metadata路径/类型仍正确：new
`docs/specs/auto-terminal-core/project.md`；modify vm/overview、ui/overview、
terminal-mux-model；真实goals GOAL-003/007。media/i18n后续owner保留，不虚报R03整包完成。

Plan→**executing，1/10**：保留T-04；重开T-01/02/03/05/06/07，T-08/09/10 pending。
T02/03重开是因为这轮核心新增了改变可观察语义的retired服务，需修复其契约与边界；
既有会话lease/FFI释放的正控仍保留，并不撤销其已验证事实。

原树work一次修完四项、提交，继续更新诊断表：旧零值采样如何与重新安装/新spawn协调，
为何“事后重加”仍不保护窗口；新增容量策略为何未经语义对照即入实现，实际边界应如何
验证；新增警告与并发Spec输入也需收口。不以sleep/最终恢复/提高总数替代结果断言。

最终验收通过后，merge在新冻结提交统一裸`cargo t`+`cargo tv`、最终组合777和受影响792，
再Spec/ledger/index/归档/资源交接与wt-guard clean后清理。未改ui_gen发射算法，不机械
触发整档tu；tf仍属主检出单实例批量门。本轮仅复审和主检出计划簿记。
