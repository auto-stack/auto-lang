# PLAN-808 independent final review R5

2026-10-19；plan revision **1**；outcome **pass（final；merge-owned门禁pending）**。
受审提交 `b561796b5750bd2799f835c36235de7efd36d7de`。本会话没有实施修复，独立核读
主检出合同、原实施树代码、R4诊断表及本轮正负控制；未创建其他agent/工作树。

## 基线

- worktree `D:/autostack/.wt/lang-808/auto-lang`；branch `codex/plan-808-dev`；
  diff base `2046a025b9a445d35a02da2c72a1613259a0cb60`。
- 完整提交/依赖/配置/规范版本与hash见
  [baseline-current.json.txt](baseline-current.json.txt) 与
  [final-check-current.json.txt](final-check-current.json.txt)。起止实施树clean，HEAD/依赖/DLL/锁/配置/规范未漂移。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；auto-term
  `7a0d0e8b718fb260641f0d09ada95a7346771cd3`；两依赖树clean，只读消费。
- 实际DLL `D:/autostack/auto-term/target/debug/autoterm_core.dll`，SHA256
  `c17ecca15a84f86cb26ad1f391490afe2e8b7420a369047c4bac3f6b8f0e8f58`；
  root Cargo.lock SHA256 `9a431eb9e6163c154798db92543028db38a29af70ddac008014cc48ebd6a8fe1`。
- [冻结SD](spec-delta-frozen.md) SHA256
  `78957dc6cd2a1c0a21f54c757a525f687f200ff4bb75a26c4fbd06fa700afbb0`。
  canonical vm/ui来源均绑定PLAN-807沉淀 `6955e317859b34693ca4f941c07b4957c0459190`；
  mux/绘制规范、Cargo manifests/锁/配置与R4未变。new目标
  `docs/specs/auto-terminal-core/project.md` 当前不存在，add操作正确。
- Cargo仅在各命令进程清空继承的`RUSTC_WRAPPER`，未改配置。所有新输入/输出在R5，
  没有覆盖R1..R4工件，也没有改实施源码、canonical Specs或ledger。

## R4 findings闭合

| Finding | 本轮结论和独立证据 |
|---|---|
| F808-R4-01 P1 复活provider短暂丢路由/free | **CLOSED**。owners条目持有revival计数；重装和成功spawn发布在owners写锁内bump；剪枝先采样stamp再锁外调用count，只在同写锁内确认stamp未变才移除。复活者不再经过移除/事后重加。独立重放重装+真实spawn83×27：owner=true、两VM均83×27、真实free后A_live=0。额外隔离“仅spawn、无重装”路径同样通过，排除只靠重装bump掩盖会话发布缺口。见[r4-replay.txt](r4-replay.txt) |
| F808-R4-02 P2 4096 FIFO遗忘旧值 | **CLOSED**。retired Store改为仅HashMap，无容量淘汰。实际94×34会话、late消费者未预读，4096个其他条目后global仍Some、late/early/core均94×34；两个发布模式分别验证。core另有5000条全量值断言；新永久验收有4097条控制。旧free后值语义恢复，不依赖提前读取，不强持旧service。见[r4-replay.txt](r4-replay.txt)、[core-tests.txt](core-tests.txt) |
| F808-R4-03 P3 三条unused_mut | **CLOSED**。指定三处mut已删除，本轮完整定向test-profile编译没有plan808/core/adapter新增诊断；全仓既有库warning仍如实保留。见[plan808-tests.txt](plan808-tests.txt) |
| F808-R4-04 P2 与807规范冲突 | **CLOSED**。当前SD-01/02/03及handoff残边表将diff四入口标为807已迁；SD-03明确以当前canonical为基底，仅把term/media收窄为media，保留807条目与正文。vm侧同理追加terminal、不回写旧diff残边。已对照当前6955e3178版本审查 |

R4输入原件hash未变。由于新协议已删除“移除后再次count并重加”，旧探针等待第二次count
会等待一个不存在的调用；本轮只移除该同步点，改在陈旧零判定完成后断言原owner/读/free，
保留第一次零值采样暂停、同VM真实spawn/切换以及所有原期望，再加spawn-only控制。
输入为[repair-probe.rs](repair-probe.rs)，与原输入的差异保存为
[r4-probe-adaptation.patch](r4-probe-adaptation.patch)，命令为
[run-repair-probe.ps1](run-repair-probe.ps1)。新探针对结果有实际assert，任一不符退出非零。
连续可路由还由源码的同锁stamp条件移除及原剪枝窗口重放共同验证，不凭最终列表猜测。

## 本轮实际验证

| 入口 / 范围 | 结果与证据 |
|---|---|
| DLL env + `cargo test -p auto-lang --lib plan808 -- --test-threads=1 --nocapture` | **24/24**，无PENDING；34codec/装配/缺席/真实DLL/两桥接构造/路由/缓存/清理/并发，[plan808-tests.txt](plan808-tests.txt) |
| `cargo test -p auto-terminal-core` | **7/7**，含lease/free隔离与5000条不淘汰，[core-tests.txt](core-tests.txt) |
| 当前test binary `ui::terminal --test-threads=1` | **41/41**，定向、once、几何、颜色、window、Mux回声，[ui-tests.txt](ui-tests.txt) |
| 当前test binary `ui_gen::rust::tests::terminal_ --test-threads=1` | **3/3**，既有Rust组件事件/scroll/scheme，[codegen-tests.txt](codegen-tests.txt) |
| 当前生产库build、双包check、no-default-features check | exit0，[lib-build.txt](lib-build.txt)、[check.txt](check.txt)、[no-ui-check.txt](no-ui-check.txt) |
| 只复制当前core+consumer到新目录，fresh offline lock→offline locked test，真实DLL env | **4/4**；仅调整消费方path为`../core`，无语言/UI源码，[standalone.txt](standalone.txt) |
| 独立normal/build/dev闭包 | 仅core→libloading0.8.9→windows-link0.2.1，[deps.txt](deps.txt) |
| 全部改动Rust文件rustfmt check、diff空白 | exit0，无输出，[format.txt](format.txt) |
| 777 strict/no-cargo-check | pass，0violations/2grandfathered，[audit.txt](audit.txt) |
| 当前提交792独立完整driver | **25/25**；fresh locked build182s、重定位locked build170s；资源/patch/pin/发布/用户数据负控及开发路由均过，[delivery.txt](delivery.txt)、[delivery-receipt.json.txt](delivery-receipt.json.txt) |
| R4陈旧零判定+4096边界、spawn-only补控 | 两模式均通过，真实free后0、late保持94×34，[r4-replay.txt](r4-replay.txt) |
| R3常活剪枝窗口、跨VM/直接free | 全期望值：true/80×24/free后0、共享91×31/直接92×32保持，[r3-replay.txt](r3-replay.txt) |
| R2四种重入×direct正控、同VM两种free/replace顺序 | 8/8完成，缓存四行全期望，[r2-callback-replay.txt](r2-callback-replay.txt)、[r2-cache-replay.txt](r2-cache-replay.txt) |
| R1双活路由/并发free/在途写/Drop/组件入口/实际child | 全期望值：不误free、并发FFI free=1、在途free=0、Drop free=1、桥接泵1/0；OS child PID23432/29496均退出，[r1-replay.txt](r1-replay.txt) |

缺必须符号的panic属于catch_unwind负控，不是未处理失败。新包/新增测试零未处理编译
warning；默认库408条/no-ui库194条为已有warning，不全局静音、不冒称全仓零warning。
既有ui-only组合F-785-P01问题不记为本轮成功面；本次验证默认UI及关闭默认feature的
明确保持要求。没有运行应用窗口或宣称Vue/VM全应用视觉parity；renderer未改，本轮
覆盖真实VM shim/UI注册表/DLL/生产组件构造与组件生成。没有运行t/tv/tt/tb/tu/taa/tf整档。

## AC、代码与知识增量

| AC | task→代码/工件 | 结论 |
|---|---|---|
| AC-01 | T01/02/03/05/06→core/provider、VM34codec、默认/实例、独立consumer、feature | pass：34ID/shape/pop/缺席值/配置stub保持，绑定revision不变，非UI单向闭包；回归及原反例过 |
| AC-02 | T04/05/06→terminal_provider、UI注册表、VmBridge、原Rust生成器 | pass：定向/一次性/几何/颜色/滚动真源及生产装配过，无新增a2r引擎支持面 |
| AC-03 | T01/02/03/05/06→SessionTable、Library owner、provider条目、retired值 | pass：真实DLL/受控DLL、OS退出、两VM/替换/重复free、并发/复活和缓存边界过；默认共享service不承诺每VM drop杀会话，保持原合同 |
| AC-04 | T07→源切割、独立fixture、777/792、当前SD/残边表 | pass：源切割/闭包/strict/core fresh、当前完整792的fresh/relocated locked/负控均过，剩余owner/807调和明确 |
| AC-05 | T08→独立AC/SD/health审查 | pass：旧findings闭合，无新增阻断项，最终输入稳定性核对通过；明确merge-owned门禁pending |
| AC-06 | T09/10→冻结合入门/知识沉淀/归档/guard | pending，merge-owned |

SD-01新增终端核心项目规范，owned值/依赖边界/真实DLL缺件与释放/无淘汰兼容性与源码
和控制一致；SD-02 VM34接口、共享/实例/owner/stamp及retired规则一致；SD-03装配位置
及最新807残边调和正确；SD-04仅登记实现owner，display_offset/Mux规则不变。
四项不是削弱原合同的延期或workaround。没有发布canonical Specs/live ledger。

impact metadata保持：new `docs/specs/auto-terminal-core/project.md`；modify
`docs/specs/auto-lang/vm/overview.md`、`docs/specs/auto-lang/ui/overview.md`、
`docs/specs/terminal-mux-model.md`；goals GOAL-003/007。media/i18n的后续owner仍明确，
不把终端34项完成当成R03整包完成。

## final结论与路由

**pass**，findings=[]；R4-01..04及所有此前合同内阻断项本轮独立闭合，未以延期/削减支持
换取通过。当前792收据25个唯一case均ok=true；[交付manifest](delivery-manifest.json.txt)
和[源码绑定](delivery-source-binding.json.txt)持久保存，交付包native.rs/retired.rs与冻结
实施源码逐字节一致。本轮没有仅借用work绿色总数签发结论。

Plan→**reviewed，8/10**：T-01..T-08完成；T-09/T-10保留pending，交由merge。
本pass仅绑定本提交及本轮依赖/配置/规范输入，任何后续代码或语义变更需复审受影响内容。

**merge-owned pending，未通过前不能合入/宣称交付完成**：

- 在最终冻结代码、最终合流依赖上统一执行裸`cargo t`+`cargo tv`；若门禁要求代码修复，
  回work提交并复审受影响内容，再在新冻结提交重跑受影响门禁。
- 与已落地807/并行809等合流时核对manifest、宿主装配和delivery crates并集；最终组合
  777重绑/strict、受影响792 fresh/locked需按合流输入执行，不能用本树结果代替组合证据。
- 按已审SD沉淀Specs/ledger/index，保留当前807知识，归档及资源owner交接；最后
  wt-guard clean后再清工作树/分支。到期tf归主检出单实例批量门。

本计划未改ui_gen发射算法，不机械触发整档tu。此轮没有执行根整档、合入、归档或清树，
也没有修改canonical Specs/live ledger。保留默认共享service及媒体/i18n等后续边界，
不宣称R03整包完成。
