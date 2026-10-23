# PLAN-838 r1 final review R1 — needs_fix

日期：2026-10-24。本次为用户明确请求的独立最终验收复审；当前 Codex 复审上下文从主检出合同、源码和原始证据重建结论，没有参与本交接的实施，也没有创建 agent/新聊天。未暴露精确模型/会话身份，不虚构身份。本轮仅写主检出 Plan/复审附件/风险簿记，没有修实现、提交、合入、发布 canonical/ledger 或清理实施树。

受审唯一树 `D:/autostack/.wt/lang-838/auto-lang`，分支 `plan-838-dev`，冻结提交 `8494e06d4056e7f4e0f6dd1925e128bb93a83ea6`；diff base 为正式 837 交付 `cf2e99f50ee2a70b70ca2cfc23a5af56b405638b`。复审前后 HEAD 相同、tracked/untracked 交付面 clean；35 文件、4588+/88-，写面符合两个实验 workspace 的约束。主检出的其他文档 WIP 保留。

SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`；driver lock `2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db`，AGP lock `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`；source identity `15b2f909f56f407f8bd086ebc01eb75dc84998a8552135649ee3195ae0543354`。完整基线、受审 Spec 输入及 GLB 反例指纹见 [baseline](baseline.json.txt)。规范提案已逐字节冻结为 [frozen-spec-delta](frozen-spec-delta.md)，SHA256 `f029676ba7172a7eaa31f70557f4ade400609424b488405ec48f7452e9ca63b5`。

结论：**needs_fix，2×P1 + 2×P2，均为原合同范围内缺陷**。现有正式定向测试全部绿，但不能覆盖以下独立反例。状态回 `executing 4/10`，T-03/T-06/T-07/T-08 重开；T-01/T-02/T-04/T-05 的不受影响成果保留。T-09 不勾选，T-10 仍 pending。修复应回同树 work，一次补齐 findings 及兄弟路径反例，提交新基线后再复审；本次没有自动执行修复的授权。

## F-838-R1-01 / P1 / OPEN — Save As 的路径别名可覆盖不可变资产

- 对应 AC-06/08，T-06/T-07/T-08，V-07/08，SD-01/02；违反 §5.5 的 immutable blob、不覆盖 foreign/旧资源以及失败保护合同。Spec 提案还明确禁止把 canonical blob 用作 scene entry。
- 代码：`src/authoring/storage.rs:188..206` 在 `absolute(target)` 之前，以区分大小写的原始字符串识别 `assets/<sha>.glb`。大小写或含 `..` 的同一文件路径绕过该检查；后续 canonicalize parent 和入口 rename 实际落到原 blob。
- 输入：先保存带真实外部 GLB 声明的 `original.agscene`，再将另一个空场景 Save As 到原 blob 的别名。合法控制是精确小写路径，三个反例分别为扩展名 `.GLB`、父目录 `ASSETS`、`assets/nested/../<sha>.glb`。
- 实测：精确路径返回 reserved Err，原 blob 字节与原场景重开保持；三种别名全部 `Ok(Confirmed)`，`original_blob_preserved=false`，原场景随后 `conflict: asset length`、无法重开。不是并发写入或 malicious path replacement。
- 可重放输入/入口：[path-holdout.rs](path-holdout.rs)，原始四案输出：[path-holdout.log](path-holdout.log.txt)。实际调用生产 `Storage::save`；仅毁坏探针自己创建的 target 下临时资源。
- 关闭 oracle：路径规范化后的真实目标应在任何入口写入前被识别为 reserved；覆盖 Windows 大小写、合法 `..`/同一父目录别名及普通新 `.agscene` 正控。所有拒绝案必须保原 blob 全字节、原入口可重开、调用方 dirty/path 不被错误确认。不能仅扩大字符串黑名单后只重放原一个拼写。

## F-838-R1-02 / P1 / OPEN — 读取 accessor 前缺少语义类型检查，可 panic 或误读

- 对应 AC-02/05/08，T-03/T-07/T-08，V-02/06，SD-01。违反严格预检、非法输入明确拒绝、last-good 可继续使用的合同。
- 代码：`src/authoring/assets.rs:463..492` 核对 POSITION/NORMAL count 和 span 后直接调用 gltf typed reader，未核对每种 semantic 的 component type、dimensions 和 normalized 组合。`gltf::Gltf::from_slice` 只做部分验证，不能替代这里的语义准入。
- 输入：自有合法 two-parts GLB 只把 NORMAL accessor 的 componentType 从 FLOAT 改为 UNSIGNED_BYTE；其余 bytes/view 保持。默认 dev 构建在 gltf `accessor/util.rs:371` 的 `debug_assert_eq!(12,3)` panic，`decode_attempts` 增量 0。应用生产 Host import 路径同样 panic；当前 UI 路径没有 catch_unwind。探针为保留观测才额外捕获，产品并没有该保护。
- 兄弟反例：POSITION componentType 改为 UNSIGNED_INT，typed reader 仍把原始 bits 当 f32，decoder 调用 1 次，Host 最终 accepted 1 asset/1 object/1 history。FLOAT POSITION 却 normalized=true 也被接收。不能只修会 panic 的那个 NORMAL 类型；release 关闭 debug assertions 也不是合法准入。
- 证据：[glb-holdout.log](glb-holdout.log.txt)、[host-holdout.log](host-holdout.log.txt)。相同输入的合法 FLOAT 正控通过，现有 sparse/U8/U16/U32 **indices** 正控仍应保持；拒绝错误 POSITION/NORMAL 不等于拒绝合法整型索引。
- 关闭 oracle：在任何 typed reader/AGP decoder 前验证所有实际属性、索引以及 sparse base/value 的语义类型/形状/规范化组合；错误数据返回具名 Err、零 decode/上传、原 accepted/history/owner 不变，随后合法 import 成功。覆盖尺寸相同但类型不同的误读、尺寸不同的 panic，以及 interleaved/sparse 合法控制。

## F-838-R1-03 / P2 / OPEN — mesh 级 morph weights 被静默忽略

- 对应 AC-02/08，T-03/T-07/T-08，V-02，SD-01。§5.2 明确拒绝 morph、检查全文件声明；不是仅拒绝实际执行的 morph target。
- 代码：`src/authoring/assets.rs:445..453` 检查 primitive.morph_targets，另在节点循环检查 node.weights，却没有检查 mesh.weights。
- 输入：合法 GLB 增加 `meshes[0].weights=[1.0]`，没有 targets。预期 malformed/unsupported morph 声明在 decoder 前拒绝；实际 decoder 调用 1 次、两部件静态几何被返回，Host 接收并创建历史。没有声称完整带 targets 的 morph 模型能绕过现有 targets 检查。
- 证据同 GLB/Host holdout，合法无 weights 对照通过。
- 关闭 oracle：检查 mesh 级声明，包括未被 scene 引用的 mesh；mesh/node/primitive 三处 morph 声明都应具名拒绝，保留无 morph 的合法控制。不能通过删除字段后静态渲染来过门。

## F-838-R1-04 / P2 / OPEN — matrix 与 TRS 同时声明时悄悄丢弃 TRS

- 对应 AC-02/08，T-03/T-07/T-08，V-02，SD-01；违反严格 GLB/变换准入及不静默忽略数据的合同。固定依赖 gltf-json `scene.rs` 的 Node 文档也明确 matrix 与 TRS 二选一。
- 代码：`src/authoring/assets.rs:548` 只检查 `node.transform().matrix()`；该 accessor 选择 matrix 分支后已经丢失“原 JSON 还声明 TRS”的事实。
- 输入：同一个节点同时给 identity matrix 与 translation `[20,0,0]`。实际 import/Host accepted，first vertex x=-0.7，即使用 matrix 分支而忽略 translation。它既不是原 fixture 的 -2.2，也不是按 translation 得到的 19.3；正确行为是拒绝互斥声明。
- 证据同 GLB/Host holdout。关闭 oracle：在 node transform 归一化前按原始声明验证 matrix/TRS 互斥；覆盖 translation/rotation/scale、未引用节点及单独 matrix/合法 TRS 正控。不得默选一种解释或改写原资源 bytes。

## 独立核验与覆盖边界

全部 Rust 命令在唯一实施树的 driver 目录执行，`RUST_MIN_STACK=16777216`；额外独立探针通过 rustc 链接本轮已验证构建的 driver rlib，PE stack 32MiB。未修改受审实现；探针源码与日志保存在主检出附件。

| 验证 | 结果与可查证据 |
|---|---|
| `cargo test --locked --test authoring_assets -- --nocapture --test-threads=1` | 14/14，0 ignored，26.02s；[authoring-assets.log](authoring-assets.log.txt)。包含完整20轮、原字节 sole-scene、512真实 GPU 像素、预算和磁盘恢复 |
| 原相关 document/hierarchy/host/interaction/session/storage | 1+10+7+7+4+3=32/32；[legacy.log](legacy.log.txt) |
| `cargo test --locked --bin autographics-authoring -- --nocapture` | completion 3/3；[completion.log](completion.log.txt) |
| AGP `--test asset_resources` | 1/1，同设备/staged/u16 GPU/最后 pin；[agp-resource.log](agp-resource.log.txt) |
| `check_scene_assets.py` | 第二次完整7案聚合 pass；positive phase20、9种状态×4独立颜色/原位置检查，实际搬迁异cwd新进程、缺/坏 blob 拒绝及恢复；[native-rerun/results.json](native-rerun/results.json.txt) |
| native 初次运行 | transparent 案在 phase6 `OS group handle down` 提前失败，尚未达到透明像素 oracle；[native.log](native.log.txt) 和 native/ 原记录完整保留。不计有效透明负控，不据此宣称渲染缺陷；同代码串行完整重跑命中正确 oracle |
| `check_asset_owners.py` | retained-owner/fresh-upload 均101且分别命中 last-owner/warm-upload 断言，恢复20轮0；[owners/results.json](owners/results.json.txt) |
| `verify_source.py` / `verify_plan838.py --controls` | 680 SDK/264 source/479 package/17 local/29 path edges；LF正控、Auto-source/自有GLB/原GLB/lock四反控与恢复通过；[source.log](source.log.txt)、[source-controls.log](source-controls.log.txt) |
| all-targets check / driver fmt / AGP两文件 rustfmt / diffcheck | 全通过；仅固定 SDK 的146继承 warnings，driver/本改动新增 warning=0；[check.log](check.log.txt)、[fmt.log](fmt.log.txt)、[agp-fmt.log](agp-fmt.log.txt)、[diff-check.log](diff-check.log.txt)。没有新增 stray debug/TODO/FIXME；测试/probe诊断属于正常证据 |
| reviewer holdouts | 路径4案=1合法拒绝/3错误覆盖；GLB9案含合法控制、有效非法拒绝、1 panic和4错误接受；生产 Host5案=1合法接收/1 panic/3错误接收。输出为观测 harness，exit0不表示合同通过；判定依据见逐案 expected/actual |

本轮初版 Host 探针把 Windows verbatim canonical 路径错误转换成 `//?/D:/...`，导致所有输入都在 local-disk path 检查拒绝；[host-initial-invalid-probe.log](host-initial-invalid-probe.log.txt) 保留。该输入未到达声明的 GLB 预检，不算产品反例；修正为普通绝对路径后得到上述有效 Host 结果。初次裸 rustc 链接缺 windows native library 搜索路径，补 `-L native=.../windows_x86_64_msvc-0.52.6/lib` 后成功；不是产品构建失败。

容量变异的旧收据绑定 HEAD3a9，但同时记录的完整 test SHA 实际精确对应后来提交 b0119d0d9；本轮已核该事实。当前完整 test 文件另有新增测试，所以不能宣称整文件相等。当前容量函数/辅助函数、显式scene输入、pipeline/锁/依赖保持，新增 sole-scene 分支不在该输入路径；仅据此复用 renderer511 sensitivity 的历史事实，当前512正控已重新运行。完整比对见 baseline.json.txt。本轮没有运行会改写 pipeline 的旧 mutation 脚本。

独立 source consumer 的原 work 结果、源码输入清单和部署栈前提已核阅；本轮未再冷建 consumer、未跑全 driver/AGP/两游戏/原835..837完整矩阵，不把它们记成本轮 pass。上述完整档属于最终 review pass 后的 merge-owned 门禁，当前 needs_fix 不可提前执行。

## AC → task → 实现与证据

| AC | 本轮结论 | 实现/证据与边界 |
|---|---|---|
| AC-01 | pass | T01/02/08；正式837基面、唯一树、来源闭包/写面和source反控；独立导出证据按上文范围核阅 |
| AC-02 | fail | T03/04；资产14、预算/u16/512绿，但 F02..04 使严格预检不成立 |
| AC-03 | pass | T03/05；document codec、v1/v2迁移/原字节、引用准入及assets14通过；F01为保存目标保护，归 AC06 |
| AC-04 | pass | T04/05；真实profile5、wrong-resource全拒绝、staged同设备及实际GPU定向通过 |
| AC-05 | partial | T05/06/07；正常路径、完整parts/OBB/gesture/undo/native绿；F02非法导入可能panic，失败恢复不完整 |
| AC-06 | fail | T06/07；正常资源先行/恢复/新进程搬迁绿，F01仍可破坏已有项目闭包 |
| AC-07 | pass（定向面） | T04/08；共享/计量/last-owner20轮及双负控、旧32通过；原游戏/完整合成兼容仍明确merge-owned pending |
| AC-08 | partial | T08；SD路径/目标ID和交接齐备，但“所有声明严格检查/保留blob路径保护”的当前实现与提案不符；随修复和新证据重核 |
| AC-09 | fail | T09最终needs_fix，没有final pass |
| AC-10 | pending | T10禁止开始合入/沉淀/归档清理 |

## 规范与遗漏扫描

已核 Plan 的 SD-01..04 与当前8个 modify目标、新 `scene-assets.md` 目标及 GOAL-007/008（均真实存在）；无 retire 操作。schema3/旧迁移、实验profile5、有限GLB、保存闭包及 I2/I3 pending 方向与当前 canonical 合同兼容；本轮不改 semantic contract、不升 plan_revision、不发布规范。SD-01/02 的声明尚不能作为“当前已验证事实”沉淀，必须修复 F01..04 后重新冻结。metadata路径维持准确，不以删掉严格准入/资产保护规则解决 mismatch。

遗漏/延后/workaround扫描已覆盖声明/实际展开预算、Auto资源映射、全parts渲染/pick/组bounds、history/取消/stale token、owner union/unverified receipt、blob→entry→完整读回与跨root锁、来源与stack前提、native完成性和内容负控。发现上述四项合同缺口；未发现其他已确证的未批准延期或新增产品范围。四项登记 KNOWN-DEBT-AND-RISKS 是追踪，不是准许豁免。

merge-owned pending：新冻结提交上的完整 locked driver/AGP test+build、原16 AGP cases、双游戏/835/836/837原生正反/完成性、counter+content双合成门、新assets/owner/source/fresh-consumer矩阵，随后才可 autographics-dev 合入、canonical/ledger/归档/guard清理及批量到期判定。root t/tv/tt/tb/tu/taa/docs_gen 不属本改动触面；tf只由主检出批量到期规则决定。本轮均未运行，也未宣称已交付。

## 反例重放

在 driver 目录先准备当前冻结的定向构建；本轮 rlib 为 `target/debug/deps/libautographics_auto_driver-2fb5bb86cd093c10.rlib`。三个 Rust 文件只有观测代码，没有产品替代实现。每次使用新的 target 子目录，避免复用已经被反例改坏的临时 blob。

```powershell
$evidence = 'D:/autostack/auto-lang/docs/plans/attachments/838/review-r1'
$nativeLib = 'C:/Users/zhaop/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/windows_x86_64_msvc-0.52.6/lib'
$runRoot = 'target/review838-replay-' + [guid]::NewGuid().ToString('N')
python "$evidence/glb-holdout.py" research/asset-probe/two-parts.glb "$runRoot/glb"
foreach ($probe in @('path','glb','host')) {
  rustc --edition 2021 "$evidence/$probe-holdout.rs" --extern autographics_auto_driver=target/debug/deps/libautographics_auto_driver-2fb5bb86cd093c10.rlib -L dependency=target/debug/deps -L "native=$nativeLib" -C link-arg=/STACK:33554432 -o "target/review838-$probe.exe"
  if ($LASTEXITCODE -ne 0) { throw 'probe build failed' }
}
./target/review838-path.exe "$runRoot/path" research/asset-probe/two-parts.glb
./target/review838-glb.exe "$runRoot/glb"
./target/review838-host.exe "$runRoot/glb"
```

修复时应把这些输入及兄弟路径转成正式永久回归：expected/actual 为每项 finding 的关闭 oracle。新代码需先构建对应新 rlib、记录新SHA；旧二进制和本次 pass 不覆盖未来编辑。
