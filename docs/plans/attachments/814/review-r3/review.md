# PLAN-814 R3：M814-A 阶段复审 needs_fix

2026-10-20。stage: review；review_scope: **phase / PLAN-814:r2:M814-A**；reviewer_identity: 本聊天Codex，延续独立复审上下文，未参与实施，无新增agent/checkout。B/runtime尚未搬迁本身不作为本轮缺陷；A要求旧兼容和已公布支持面保持、迁出源码可独立消费。

## 基线与范围

- plan_id: PLAN-814；plan_revision: 2；worktree_path: `D:/autostack/.wt/lang-814/auto-lang`；branch: `codex/plan-814-dev`。
- reviewed_commit: `abe9c99002fe7bafb49a9f86c5867f68d796d65a`；base_commit: `203e2ce91302ff1ebd48cbd0a8365d76abb89009`。候选明确将T-14作为next，本轮按阶段范围进入，不把B未完判作最终失败。
- dependency_revisions: 组内auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；主检出初始HEAD `ea428af88b083bd1c3255f05aa80a69e4124e9b4`。初查实施树clean；末查HEAD/tree/依赖/洁净及源hash见 [baseline-evidence.txt](baseline-evidence.txt)。
- 主检出原计划 [plan-before-review.md](plan-before-review.md)、候选 [candidate-frozen.md](candidate-frozen.md)、合同与Spec输入 [input-hashes.txt](input-hashes.txt)。[sd-frozen.md](sd-frozen.md)包含计划SD表及本次候选事实底稿，SHA256 `57303faffb586ab0cce085bcce56987328143e5a8a120bf8d15f2f08cebdb8b2`。
- 前两轮就绪非pass后的根因/覆盖调整已核对：本次增加SDK显式ui-dialect组合、auto-vm直连、frame端口、新compiler真实实现和消费fixture。并非仅换测试总数；但下述实际调用/源码闭包组合未覆盖，不能据此授阶段pass。

## 上轮问题闭合范围

| 原项 | 本轮判定 |
|---|---|
| R1-01不可编译、auto-vm仍依赖旧包、compiler空实现 | 这些原例已关闭：两外部fixture独立复跑5/5与4/4，auto-vm manifest直连SDK、compiler真实算法已迁；不复活早期WIP问题。R1-01的完整A验收仍受本轮反例阻断，整体为partial，不能仅靠新候选改为全闭。 |
| R1-01 frame无效ui门/固定0 | 源码改为同一FrameProvider端口，原固定0实现已移除；本轮发现的是其他旧公开高层入口没有自动安装provider，见R3-01，不能混称原frame实现未修。 |
| R1-02无条件编译UI文法实现 | **原默认编译依赖反例关闭**：依赖optional、非默认ui-dialect；外部SDK五测包含拒绝负控，normal/build/dev实际cargo tree不含UI文法/compiler/runtime/旧包。完整“无需UI checkout”不变量未闭，见R3-03。显式组合是合同允许的选择，不因该feature存在另行判错。 |

## Findings（全部契约内，非新增支持要求）

### F814-R3-01 / P1 / OPEN：旧公开VM创建入口丢失兼容宿主装配

任务T-02/T-13；AC-02/04在A范围、AC-08；分类：迁移回归。

`crates/auto-lang/src/lib.rs:3`直接再导出SDK `create_vm_from_source`/`create_vm_from_abt`等入口；SDK lib.rs:4383/4325中的装配调用落到:1308的hook。全仓搜索仅有setter定义/再导出，没有上层注册调用。于是UI构建中这些旧高层入口不再补默认宿主能力；`run_with_capture`的上层包装仍补，测试只走该包装会漏掉差异。现行Spec `auto-lang/ui/overview.md`的PLAN-779装配节明确包含create_vm_from_source/abt、配置/REPL等入口。

独立新进程 [host-probe.log.txt](host-probe.log.txt)：

```text
legacy_create_vm_installed_frame=false
run_wrapper_installed_frame=true output=("", "42\n")
exit=101: old create_vm_from_source must preserve automatic UI host assembly
```

同进程后续包装入口是正控，证明UI能力可安装，不是系统无法提供宿主。修复须覆盖全部再导出/解释器/配置/REPL入口与显式默认优先关系，不能只补探针的单一入口；纯SDK仍须无UI默认实现。

### F814-R3-02 / P1 / OPEN：指纹在缓存加载之后注入，旧产物失去生成器升级失效保护

任务T-03/T-13；AC-04/05在A范围、AC-08；分类：迁移回归。

SDK `database/ui_cache.rs:16..22`改为运行时OnceLock注入；唯一生产注入在compiler `ui_gen/api.rs:639`的generate_component_from_file内。真实调用者 `auto-man/src/vue.rs:6759`先UICache::load，再按dirty判定是否生成，:7253保存最早创建的cache对象。Jet同样先load且不经该注入入口。load时拿到None，之后注入Some不会更新既有cache的字段；首次生成保存null，后续新进程仍按None加载，生成器源码变化也无法靠指纹发现。仅检测getter为16位hex的现有fixture不检验该顺序。

独立真实生成 [cache-probe.log.txt](cache-probe.log.txt)：

```text
actual_generator=42114e3b7e7cc6db persisted_before_generate=null
new_after_generate="42114e3b7e7cc6db"
exit=101: expected persisted fingerprint Some(...), got None
```

正控是同进程生成后新建cache，持有正确Some。违背现行 `docs/specs/auto-lang/runtime/overview.md` PLAN-698缓存键必含指纹与升级失效合同。应在所有cache load/dirty决策前建立真实指纹来源，覆盖Vue/Jet/CLI及直接生成出口；补缓存保存→新进程加载→生成器版本变化的正反控。不得以None被允许为foreign build为理由把正常producer当foreign；OnceLock首次None锁死等边界也需考虑。修复后SD还需纳入runtime缓存规范目标，当前frontmatter未包含该目标。

### F814-R3-03 / P1 / OPEN：默认编译闭包减少，但无需UI checkout的源码消费仍失败

任务T-02/T-13；AC-01/05在A范围、AC-08；分类：原纯SDK不变量未完整闭合。

完整检出内path fixture和cargo tree仅证明激活的编译依赖。真实声明的源码解析还要求：

1. SDK→auto-macros，其Cargo.toml:16的dev-dependency仍指向旧auto-lang；
2. SDK Cargo.toml:44的optional path auto-ui-dialect清单，即使feature off也要能读取。

以真实非UI依赖源码副本、相同workspace.dependencies、SDK唯一成员构造无UI源码快照；未改任何产品文件。[no-ui-metadata.py](no-ui-metadata.py)及[原始结果](no-ui-metadata.log.txt)：

```text
before UI manifest present=False
no_ui_checkout exit=101: auto-macros -> auto-lang/Cargo.toml missing
diagnostic control removes only copied macros dev edge
still_no_ui_checkout ... exit=101: auto-ui-dialect/Cargo.toml missing
restore actual dialect source in copied snapshot ... exit=0, 17 packages
original full-checkout consumer control ... exit=0
```

复制快照的两处诊断变异明确写在脚本中，恢复对照通过；不是通过修改真实manifest制造失败。正控仅证明清单解析，未宣称修补快照已完成fresh build。另SDK build.rs:41仍递归扫描整个根crates并把UI/Man源码纳入BUILD_INPUTS，源码/build输入闭包也未按SDK归属界定。

应交付真实的纯SDK source/manifest/build输入闭包，消除旧包dev清单链，处理显式UI组合的源码分发机制；给出真正缺UI/旧包目录的fresh和locked编译消费。当前792是含旧UI包的完整UI发行，不替代此负控。

### F814-R3-04 / P1 / OPEN：旧autodown支持配置退化为永久fallback

任务T-02/T-13；AC-02/04在A范围、AC-08；分类：迁移回归/未批准延期。

上层Cargo.toml:152仍声明 `autodown=[ui-iced,dep:autodown-core]`；native实现却已迁到SDK `vm/autodown_natives.rs:118/408..435`，SDK根本没有autodown feature和实现依赖，所有相关cfg恒假。开启旧包特性只启用上层依赖，native仍报未构建。候选§5把该项写为T-04限制，并未授权A阶段破坏旧支持面。

实际以 `cargo build ... --features auto-lang/autodown`构建成功后运行探针：[构建](autodown-build.log.txt)、[运行](autodown-probe.log.txt)：

```text
autodown_build_exit=0
autodown_parse_with_upper_feature=Err("built without the `autodown` feature: autodown_parse unavailable")
autodown_probe_exit=101
```

修复应将真实实现与装配接到有owner的上层/能力端口，使旧feature恢复原parse/editor族行为，保SDK纯净；不能仅删除告警或把兼容修复延期到B。样例`# Title`属于原支持面，不要求新能力。

### F814-R3-05 / P2 / OPEN：迁移后的原测试档选择面未完整映射

任务T-02/T-03/T-13；AC-05在A范围、AC-08；分类：测试迁移遗漏。

`.cargo/config.toml:91/92`的tt/tb仍只选auto-lang；已有inline测试迁入SDK（例如`src/trans/rust.rs:28701` test_transpile_fn_single_function、:28737 test_struct_pair_field_to_string），依赖crate的单元测试不会因上层`--lib`自动入选。因此tt/tb丢失原先本包内的这些测试；t扩包会覆盖部分测试，但不等于每个原档保持选择面。候选的池总数6851不提供逐族/逐档映射。

需给迁前/迁后实际测试名→owner→别名选择表，对迁移的各档作必要接线与负控。tv等使用自定义filter的档应按实际族位置判断，不要求无依据地给所有别名机械扩包。review可用list与定向测试核对，裸整档仍归后置merge。

### F814-R3-06 / P3 / OPEN：新增门面未格式化，迁移后的无效feature告警未清偿

任务T-02/T-03/T-13；AC-06健康要求的A对应面、AC-09；分类：交接健康缺口。

`rustfmt --edition 2021 --check --config skip_children=true crates/auto-lang/src/lib.rs crates/auto-lang-core/src/lib.rs`退出1，首处就是新增长行再导出（[fmt.log.txt](fmt.log.txt)）。定向构建还在SDK autodown_natives报告unexpected_cfgs；该类告警与R3-04同根，需修语义后清理。未把本轮全部173/223个告警都判为新增或要求清除所有历史告警；必须对实际新增/迁移告警归因，不能只以zero error代替zero unhandled warning。

## 验证证据、覆盖及规范

| 入口/验收 | 本轮结果与边界 |
|---|---|
| V-01纯SDK parse/compile/VM/trans、默认UI拒绝 | [SDK外部fixture](sdk-consumer.log.txt) **5/5**；[默认依赖树](sdk-dependency-tree.log.txt)不含禁止激活边。无UI源码消费另有R3-03失败，不能泛化为AC-01全部pass。 |
| V-02旧默认与显式组合/API/单状态 | compiler fixture真实UI解析生成正控；旧高层装配R3-01失败，autodown配置R3-04失败。旧字段/trait全矩阵尚非pass。 |
| V-03生成出口/当期runtime | [compiler外部fixture](compiler-consumer.log.txt) **4/4**，含真实SFC、style、lucide、指纹getter。源码四生成器真实迁移已核实；四端生成工程/运行完整矩阵仍未在本轮重放。 |
| V-04状态/寿命与剩余边 | STORE_EXTRA_FILES经同一SDK再导出，无新副本；frame原固定零已去除。但宿主漏装配/指纹顺序反例失败；未授整个provider/TLS/退出矩阵pass。 |
| V-05测试/资源/发行 | git diff --check(base..HEAD) **0**；格式检查失败。792的25/25当前只有work摘要，本轮未将摘要当独立绑定raw证据，未重跑其冷构建/重定位全集；受当前反例影响的源码闭包与实际门选择仍失败。777整项本轮未复验。 |
| V-06 / AC-08阶段完整性 | **fail**：R3-01..05。AC-09/T-14不通过；AC-10/T-15 merge-owned pending。AC-01..05全Plan最终验收仍pending，不对B未完成另发缺陷。 |

探针源码 [probes/src/main.rs](probes/src/main.rs) 与manifest、[冻结依赖锁](probe-Cargo.lock.txt)保留；初始构建 [probe-build.log.txt](probe-build.log.txt)和autodown追加构建均只构建外部验收探针。host/cache各运行于独立新进程，panic=预期反例断言；两个fixture正常退出作正控。未运行裸t/tv/tt/tb/tu/taa、tf、docs_gen或GUI全局测试；构建/探针都在原树工作目录，产品源码零修改。

Spec review: 候选§2可作为事实底稿，但“已发布事实”尚未真实合入；本轮冻结它作为拟议SD，未批准为现状。SD-01的独立源码闭包、SD-05/06/07的旧装配行为、SD-08选择/发行证据仍受上述findings阻断；runtime缓存规范目标须同步补入适用delta。需要准备精确的A子集before/after正文和当前输入版本，不能将记录“正文随T-14/T-15沉淀”视为已审Spec patch。canonical Specs/live ledger未改。

## 路由与后置门

outcome: **needs_fix**；Plan保持executing。重开T-02/T-03/T-13，保留T-01完成，current_step **1/15**；T-14未勾、T-15及B任务继续pending。原工作历史/已完成修复证据保留，不回滚实现。

next: 同一原树work一次修齐R3-01..06，回放本轮共享反例和正控，并补对应入口/配置/跨进程/独立源码消费组合；提交后再申请M814-A独立阶段审查。此轮没有批准API/支持面退役或缩减；若必须改变合同，按needs_replan路径提交具体变更。

pending_closeout_gates: 阶段独立pass后，merge owner才在最终冻结源码/依赖/配置/输入上统一裸cargo t与实际tv/tu/tt等触面档，并完成A实际合入/Spec子集/受管ledger/版本来源收据；A不归档不清树。B最终收口另走原T-04..12。tf仅主检出到期单实例。本轮未合入、发布、归档或清理。
