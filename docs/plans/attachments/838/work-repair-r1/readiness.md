# PLAN-838 r1 — final R1修复交接

2026-10-24，work pass；T-03/06/07/08重新关闭，execution_done **8/10**。F-838-R1-01..04均已修复并完成实施侧关闭oracle；没有当前合同finding、必需覆盖gap或待决合同问题。最终独立复审仍pending，本记录不替代复审结论。

唯一实施树 `D:/autostack/.wt/lang-838/auto-lang`，`plan-838-dev`；受审/修复起点 `8494e06d4056e7f4e0f6dd1925e128bb93a83ea6`，新冻结提交 **`eddf13ee6e5a9296bf11b0622561a669e7498b56`**，树clean。仅7个driver文件500+/17-，SDK/AGP/两锁/原四profile未改。按原合同收紧准入和目标身份，不升语义revision、不删验收、不增加产品范围。实现写权交回review，保留同树；没有新agent/聊天、合入、canonical/ledger发布或清树。

SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`，680文件；SDK metadata closure479/17 local/29 path edges；来源264，清单SHA `88c986e48458163517359b4003c809669854c8d3cf34a581ca1ddc2f57907b2a`，Spec提案SHA `9f49bdb9699840af0add768a07508bc7a20904a19b69bb9f37d791730a630192`。两锁完整SHA/fixture/重放rlib及原始反例源码指纹见[收据](receipt.json.txt)。[新冻结SD-01..04](frozen-spec-delta.md)明确类型组合、声明级morph/互斥变换和规范化保存目标，canonical尚不发布。

## Finding → 不变量 → 兄弟状态 → 修复和永久证据

| finding | 不变量与适用路径/状态 | 修复、永久回归、当前结果 | remaining gap |
|---|---|---|---|
| F01 / P1 | Save/Save As/save_snapshot的目标必须按Windows实际文件身份分类；不得写blob/lock，不得错误确认dirty/path/receipt | `absolute`对已存在目标canonicalize完整路径、新目标canonicalize父目录；之后大小写等价识别保留blob/lock，先于pending receipt验证/任何写入。永久 `reserved_save_targets_use_resolved_windows_identity_before_any_confirmation`：exact/.GLB/ASSETS/..两种父别名/大写SHA/trailing-dot共7种已有blob别名；3种新保留名/lock别名；旧entry/blob全bytes和重开world/GPU、dirty/history/stamp/path不变；pending receipt不误确认；普通case/..保存正控及同文件大小写冲突检查通过。review原4案新code均Err+preserved+reopens | 无；[原红](path-before.txt)、[当前绿](path-after.txt)、[永久回归](logs/scoped.txt) |
| F02 / P1 | 任何typed reader/decoder之前验证所有primitive属性/indices语义组合；sparse values继承同一类型，不能panic/按错误bits读取 | 全mesh第一遍准入：POSITION/NORMAL FLOAT VEC3非normalized；UV FLOAT VEC2非normalized或U8/U16 VEC2 normalized；indices U8/U16/U32 SCALAR非normalized；之后才验证base/sparse spans和读值。永久 `typed_accessors_and_declared_morph_transforms_reject_before_readers_and_host_publication`含错误component/shape/normalized、sparse及未引用mesh、UV/indices反例；直接decode与生产Host均Err，decode/upload零增量，旧doc/history/stamp/Arc和最终pixels保持，合法import重试成功。另一永久control测试验证interleaved、POSITION/NORMAL/indices sparse、3种UV合法组合和原有U8/U16/U32索引正控 | 无；[原GLB红](glb-before.txt)、[原Host红](host-before.txt)、[GLB绿](glb-after.txt)、[Host绿](host-after.txt)、[永久测试](logs/scoped.txt) |
| F03 / P2 | 全文件声明级拒绝morph，不能因未引用或空值静默忽略 | 原JSON层拒绝mesh/node weights和primitive targets的声明存在，包含空/空数组/null及unused mesh；永久case覆盖上述分支，原mesh.weights=[1]反例通过Host也在decode前拒绝；合法无morph控制及暖同SHA重试保持 | 无；同上GLB/Host原始重放与50非法声明永久回归 |
| F04 / P2 | gltf convenience accessor选择matrix之前，原node声明必须matrix/TRS互斥 | 原JSON遍历全部node检查matrix与translation/rotation/scale任一同时存在；永久6个互斥声明反例（含unused node）；matrix-only和TRS-only正控比较全部baked positions/normals/indices与原fixture，并保持原bytes/SHA；review原matrix+translation明确拒绝而不默选分支 | 无；[GLB绿](glb-after.txt)、[Host绿](host-after.txt)、[永久回归](logs/scoped.txt) |

最初[修复入口](entry.md)列出实际producer/consumer/owner/状态和关闭oracle。仅1次final needs_fix，本轮为第一次work repair，没有以内部检查重置final轮次。

## 本轮执行与结果

全部producer命令位于driver目录，`RUST_MIN_STACK=16777216`；consumer独立目录额外 `RUSTFLAGS=-C link-arg=/STACK:33554432`。保存原始日志，不把holdout harness的exit0当合同pass。

| 执行入口 | 观察结果/证据 |
|---|---|
| reviewer原始path/glb/host源码 + glb-holdout.py | 编辑前针对8494原样重放，3种覆盖/1panic/错误接收复现；编辑后新rlib重编译相同源码和9个逐字节相等GLB输入，path4/glb9/host5全部符合预期。合法控制仍2parts；非法GLB8个均零decoder；Host4非法均0asset/0object/0history。原始源码/输入在[inputs](inputs)，前后日志在本目录 |
| `cargo test --locked --test authoring_assets --test authoring_storage --test authoring_host -- --nocapture --test-threads=1` | assets17、Host7、Storage3，全部0 failed/0 ignored；新增一个case执行50种非法声明的direct和Host双入口；另有9种合法布局/变换控制；原14中512GPU、完整20轮、存储恢复/搬迁子进程均保持。[scoped](logs/scoped.txt) |
| `cargo check --locked --all-targets` / `cargo fmt -- --check` / `git diff --check` | 通过；仅固定SDK146继承warnings，新增/未处理作者warnings=0。[check](logs/check.txt) |
| `cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432` + `python tools/check_scene_assets.py research/asset-native-repair-r1` | 7案聚合pass；positive phase20，9种状态实际parts/原位像素与保存/搬迁异cwd子进程；wrong-asset phase5、missing/transparent/wrong-GPU phase7、drop phase8、partial phase0均非零命中各自oracle。本轮无OS早期误触发。[build](logs/native-build.txt)、[results](native/results.json.txt)、[positive](native/positive/report.json.txt)、[relocated child](native/positive/reopen.json.txt) |
| `verify_source.py` / `verify_plan838.py --controls` | SDK680/source264/closure479/17local/29edge；LF正控、Auto-source/自有GLB/原GLB/lock四负控和恢复通过。[source](logs/source.txt)、[controls](logs/source-controls.txt) |
| 原独立source consumer增量输入更新 | 先验证原264输入等于其旧receipt，再只复制6个改变的receipt输入及新manifest；原consumer自己的target构建，无producer编译产物。全264输入和SDK验证相等、check过、assets17+storage3过（0ignored）。这是原独立consumer的当前源码重建，不冒认fresh冷导出/完整merge面。[inputs](consumer/inputs.txt)、[source](consumer/source.txt)、[controls](consumer/controls.txt)、[check](consumer/check.txt)、[tests](consumer/tests.txt) |

复用边界：review-r1已核的其余旧25（32减本次Host7）、completion3、AGP资源1、原层级/游戏source、GPU同设备/上传敏感性等原证据保留；本次storage3虽与旧32重合，已重跑，不重复加总。新code仅改外部GLB预检和路径规范化，未改旧builtin样本、renderer、VM/owner和native完成性；受影响的新资产、实际Host/Storage、完整20轮/512GPU/可携带子进程及新native均在本轮重跑。旧独立复审needs_fix不可被历史work pass覆盖；四项须由后续独立复审确认关闭。

## Work-owned AC readiness 与后续责任

AC01：相同正式837基面/SDK/两锁、当前source/consumer身份通过。AC02：F02..04及所有原预算/512控制闭合。AC03：原schema/迁移/原bytes/引用在17资产测试保留。AC04：profile5及wrong ordinal、共享/staged资源路径保留并重跑。AC05：非法Host零发布/旧pixels与合法重试、新native通过。AC06：F01闭合，原资源先行/entry后确认、pending/残留/冲突/搬迁和异cwd子进程通过。AC07：17资产含20轮owner及旧Host7保持；完整游戏/合成兼容仍merge-owned。AC08：README/17case入口/SD/源码清单及此closure已冻结。所有工作AC pass，open findings/gaps/decisions均空。

T09下一责任角色独立review，在同树新冻结提交核四finding/AC01..08/Spec与遗漏、延后、workaround/健康。T10及最终pass后的完整locked driver/AGP test+build、16 AGP、原双游戏/835..837原生正反/完成性、counter+content合成、新asset/owner/source/fresh-consumer门，实际autographics-dev合入、canonical/ledger/归档guard清理和batch-due收据均pending。准确命令矩阵见[代码交接](scene-assets-handoff.md)。根整档t/tv/tt/tb/tu/taa/docs_gen/tf未运行，不提前宣称交付。
