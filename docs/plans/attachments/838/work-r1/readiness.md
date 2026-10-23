# PLAN-838 r1 work readiness — 2026-10-24

Work pass，T-01..08 / AC-01..08完成；execution_done 8/10。没有当前合同finding、必需覆盖gap或待决合同问题。此记录为实施交接，不是独立复审。T-09/T-10及merge完整门pending，未合入或宣称交付。

唯一实施树 `D:/autostack/.wt/lang-838/auto-lang`，`plan-838-dev`，base `cf2e99f50ee2a70b70ca2cfc23a5af56b405638b`，冻结code `8494e06d4056e7f4e0f6dd1925e128bb93a83ea6`，tracked/untracked交付文件均已提交且树clean。下一责任角色review；暂停实现写入，保留本树。源码最终只合autographics-dev，master仅Plan/知识。

SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`，680 files，metadata closure 479 packages / 17 local / 29 path edges。来源264 hashes，独立source consumer输入逐项相等、自己的target，不带producer编译结果。driver lock `2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db`；AGP lock `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`；source identity `15b2f909f56f407f8bd086ebc01eb75dc84998a8552135649ee3195ae0543354`；自有MIT GLB `42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671`（1528B）。见[source identity](source-identity.json.txt)、[机器收据](receipt.json.txt)。

## 工作验收与实际入口

全部命令从driver目录运行，除明确标注consumer。worker线程栈16MiB；Windows native程序PE栈32MiB。完整可执行命令和merge矩阵见[代码交接](scene-assets-handoff.md)。

| AC / V | 入口与实际结果 | 证据 |
|---|---|---|
| AC-01 / V-01 | 837正式交付及唯一树；verify_source和verify_plan838 --controls：264/680/479/17/29，LF正控及source/自有GLB/原GLB/lock漂移负控恢复通过；旧样本/SDK身份未变 | [source](logs/source-handoff.txt)、[controls](logs/source-controls-handoff.txt) |
| AC-02 / V-02/03 | authoring_assets 14/14（0 ignored）：无IO严格预检、全声明/展开预算、sparse/U8/U32、奇异/镜像/不支持能力拒绝、精确预算两侧；actual512th GPU红像素/hidden513拒绝；AGP真实65535索引红三角、65536拒绝、index8B/count3 | [assets](logs/assets-delivery.txt)、[AGP](logs/agp-resource-run.txt)、[容量负控](capacity-control/result.json.txt) |
| AC-03 / V-04 | 同14项的strict codec覆盖v1/v2→v3、旧raw冲突保护、资产声明/Model悬空/错格式长度/未知字段；缺/坏资源无占位fallback | [assets](logs/assets-delivery.txt) |
| AC-04 / V-05 | profile5真实Auto资源ordinal；wrong ordinal拒绝整包，兄弟/旧preview不变；重复mesh名稳定ordinal、同SHA跨路径Arc/解码/上传复用；staged同设备/失败新GPU owner丢弃而旧pin保持 | [assets](logs/assets-delivery.txt)、[AGP](logs/agp-resource-run.txt)、[原生wrong-asset](native/wrong-asset/report.json.txt) |
| AC-05 / V-06/10 | 真实OS→Iced→Host→Auto→GPU；Import/Create/挂组/移动/撤销redo/显隐/替换/保存，phase20、9状态×4独立颜色/原位置断言；OBB/pick全parts；失败/取消/token stale不发布 | [native](native/results.json.txt)、[positive](native/positive/report.json.txt)、[assets](logs/assets-delivery.txt) |
| AC-06 / V-07/08/10 | 实际旧external entry/blob/图像在precommit故障后完整；资源先行/entry最后、残留复用及磁盘预算、foreign不删、entry后unverified闭包复核；Save As新root、删原source/root、搬目录异cwd子进程重开和缺/坏blob失败/恢复 | [assets](logs/assets-delivery.txt)、[relocated child](native/positive/reopen.json.txt)、[missing](native/positive/missing.stderr.txt)、[tampered](native/positive/tampered.stderr.txt) |
| AC-07 / V-01/03/09 | 20真实VM/Asset/raw/CPU/GPU owner循环含import/reuse/replace/undo/redo/cancel/fail/磁盘Open/Close，last-owner弱引用及实际上传断言；未验证save独占第8预算位直到Verify；旧相关32测试、完成性3和层级原生/透明残影反控保持 | [assets](logs/assets-delivery.txt)、[owners](owners/results.json.txt)、[legacy](logs/legacy-closeout.txt)、[hierarchy](legacy-native/results.json.txt)、[pixel controls](legacy-pixel-controls/results.json.txt) |
| AC-08 / V-11 | README/支持边界/路径锁序/残留/计量/CLI/所有merge命令/拟议SD-01..04齐备；canonical/ledger未发布；生产I2/I3未冒认 | [feasibility](asset-import-feasibility.md)、[handoff](scene-assets-handoff.md)、[Spec delta](scene-assets-spec-delta.md) |

最新producer命令：`cargo test --locked --test authoring_assets -- --nocapture --test-threads=1`，14 passed / 0 failed / 0 ignored；`cargo check --locked --all-targets`和`cargo fmt -- --check`及AGP两改动文件rustfmt通过；GLB generator --check通过。[check](logs/check-handoff.txt)、[native build](logs/build-handoff.txt)。仅固定SDK继承146 warnings，作者新增/未处理warnings=0。

旧相关测试实际32（document1/hierarchy10/host7/interaction7/session4/storage3），native completion3。scratch有2个父测试名，但portable_child父进程分支是helper，不计成第二个可携带行为；实际子进程1执行并过。AGP资源定向1执行通过。该证据绑定3a9dc489e或更早记录的稳定路径；后续只增强容量/别名/失败像素测试及assets.rs中sole-scene decoder输入归一化，未改旧builtin行为/旧probe/owner路径；以当前14资产用例及最终native重新覆盖新增分支。完整旧双游戏/16 AGP/全部合成矩阵仍属merge，不从本记录外推已过。

独立consumer `D:/autostack/.wt/lang-838/consumer-work838-final/experimental/autographics-auto-driver`：最终代码264/SDK680逐输入相等；source/controls通过，all-targets通过，最终assets14/14。此前runtime assets14+storage3已过。独立导出不继承root .cargo/config.toml：初次默认worker栈失败，单设worker16MiB后child仍因PE默认栈溢出；明确同时设置 `RUST_MIN_STACK=16777216`、`RUSTFLAGS=-C link-arg=/STACK:33554432` 后全部通过。保留[原始默认失败](consumer/consumer-assets.txt)、[仅worker失败](consumer/consumer-assets-stack16.txt)以披露SDK运行前提，不把它们当故障反控。[最终source](consumer/consumer-source-delivery.txt)、[最终assets](consumer/consumer-assets-delivery.txt)、[最终check](consumer/consumer-check-handoff.txt)、[controls](consumer/consumer-controls-final.txt)。

## 不变量关闭与有效反控

| finding / invariant | 影响状态、修复/永久回归及结果 | 余gap |
|---|---|---|
| W-838-CROSSROOT：待验证receipt必须由其自身root锁保护 | e93bbc537前置反例：占用新root锁仍Save旧path得到Confirmed；3a9dc489修正先在pending root Verify再取目标root锁；同root/跨root/locked root/恢复及assets14+storage3通过 | 无；[红](logs/cross-root-prefix.txt) / [绿](logs/cross-root-fixed.txt) |
| decoder可接受单scene且省略可选scene，同时持久身份必须原字节 | 8494e06d4仅为旧decoder准备有界选择scene0的scratch，检查scratch hash，模型绑定原SHA；永久用例比较全部MeshData、原bytes/SHA/length及实际save/open；producer/consumer14均绿 | 无 |
| 512实例不能只在state计数而GPU少画末尾 | 前511 triangles屏外、唯一512th红像素必须出现；真实renderer break511变异101，原件恢复0；脚本记录原pipeline/test hash，可在本树重放 | 无；[replay](replay-capacity.py)、[result](capacity-control/result.json.txt) |
| 最后owner释放/暖资源必须真共享 | retained Asset强pin变异101命中last actual owner；强制物理重上传变异101命中warm upload；恢复20轮0，合法外部pin仍存活后释放 | 无；[owners](owners/results.json.txt) |
| 真最终内容/完成性而非state正确 | 最终native正门phase20；wrong-resource phase5、真实missing/transparent/wrongGPU phase7像素、drop phase8 timeout、frames1 incomplete phase0全部非零；原hierarchy transparent/ghost同样因终点/原位像素非零 | 无；[native](native/results.json.txt) |
| 换行传输不能伪报来源漂移 | 837历史CRLF hash先按原件复现；838把.agscene纳入文本规范化，LF consumer正控和四来源反控恢复通过；历史837清单不改 | 无；[controls](logs/source-controls-handoff.txt) |

## 实测与边界

最终native：cold 38706µs，warm P50/P95 16535/25265µs，compile 24417µs，working set 359342080B；targets4/7584000B，meshes9/137208B，mesh uploads18 writes/137208B。外部1资产：raw1528B、CPU168B、预计GPU256B、decode1次/431µs。不是进程硬内存上限；JSON/parser temporary、driver residency、Iced内部buffer/atlas及GPU timestamps排除。同步parse/compile不声称可抢占，device-loss重建设备不在本scope。

## 剩余责任

T-09独立复审AC-01..08/SD及健康、遗漏/延后/workaround；本轮没有独立review verdict。其后T-10在相同冻结提交统一执行完整locked driver/AGP测试和build、16 AGP/原游戏与835..837完整原生/完成性/合成counter+content、新asset/owner/source/fresh-consumer矩阵；完整矩阵见[handoff](scene-assets-handoff.md)及其引用的既有hierarchy-handoff。改代码需重核受影响证据。通过后才合autographics-dev并沉淀canonical/ledger/归档/guard清理和batch-due收据。根整档t/tv/tt/tb/tu/taa/docs_gen及tf均未运行；未创建agent/聊天。
