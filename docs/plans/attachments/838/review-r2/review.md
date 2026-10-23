# PLAN-838 revision1 final review R2 — pass

2026-10-24。结论 **pass**：F-838-R1-01..04 均独立确认关闭，未发现新的合同内缺陷、必需验收缺口或未批准延期。Plan 进入 **reviewed 9/10**，T-09 完成，T-10 与全部 merge-owned 冻结完整门保持 pending。本结论不表示已合入或已交付，用户要求“先不要合入”继续有效。

本次执行者是独立子代理上下文 `/root/review838_r2`，没有参与实现；实施上下文的父代理不代替本结论。本上下文由父代理转交了本次用户再审要求和路径/基线，随后从当前主 Plan、代码、R1持久输入及原始日志重建验证。工具没有暴露精确模型或会话 ID，不虚构身份。没有创建其他代理、聊天或 worktree，没有修改实现或提交、合入、发布 canonical/ledger、归档或清树。

唯一实施树 `D:/autostack/.wt/lang-838/auto-lang`，分支 `plan-838-dev`，冻结 **eddf13ee6e5a9296bf11b0622561a669e7498b56**；diff base 是正式837交付 **cf2e99f50ee2a70b70ca2cfc23a5af56b405638b**。相对R1 `8494e06d4056e7f4e0f6dd1925e128bb93a83ea6` 仅7个driver文件500+/17-，符合原写面；未改SDK、AGP、两锁、原profile1..4或根生产源码。入口、运行后与结论前均复核同一HEAD和clean实施树。

SDK **9576b4dad5ceb8b201adedd46f9689f25c2dd5f9**，680文件；source264，closure479、17local、29edge。driver lock `2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db`；AGP lock `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`；source manifest `88c986e48458163517359b4003c809669854c8d3cf34a581ca1ddc2f57907b2a`。当前主检出9个相关canonical输入与R1指纹完全相同。SD提案逐字节冻结为 [frozen-spec-delta.md](frozen-spec-delta.md)，SHA256 **9f49bdb9699840af0add768a07508bc7a20904a19b69bb9f37d791730a630192**。完整代码/锁/SDK/规则/输入/工具链/复用边界在 [baseline](baseline.json.txt)，结论及输出hash在 [receipt](receipt.json.txt)。

## 四项 finding 的独立关闭

| Finding | 旧例关闭与实际实现 | 不变量/兄弟路径/永久回归判断 |
|---|---|---|
| F-838-R1-01 / P1 / CLOSED | `storage.rs::save_snapshot` 在任何收据验证或写入前，先 `regular_path`，再解析已存在目标完整canonical路径或新目标canonical parent，之后按Windows case equivalence分类blob/root-lock。原path4全部Err reserved，原blob全bytes保持且原场景重开成功。 | 永久 `reserved_save_targets_use_resolved_windows_identity_before_any_confirmation` 验已有blob7种别名、新保留名/锁3种、dirty/path/history/stamp及pending receipt、旧entry/world/GPU保持与普通Save正控。另独立save_snapshot组合4案，见下节，证明不是只黑名单扩充原字符串。 |
| F-838-R1-02 / P1 / CLOSED | `assets.rs::preflight` 第一遍遍历全mesh的所有primitive semantic和indices，核data_type/dimensions/normalized后才进入span/sparse/typed reader；POSITION/NORMAL为非normalized FLOAT VEC3，UV为FLOAT VEC2非normalized或normalized U8/U16 VEC2，indices为非normalized U8/U16/U32 SCALAR。原panic/误读各例现在均具名Err且decode=0；真实Host不发布。 | 永久拒绝测试实际执行50个非法声明的direct+Host双入口，保旧doc/history/stamp/Arc/upload与最终pixels，随后合法导入复用成功；9合法布局/变换controls覆盖interleaved及POSITION/NORMAL/indices sparse和三种UV。独立新增跨布局组合进一步覆盖新预检与原sparse读取的交互。 |
| F-838-R1-03 / P2 / CLOSED | raw JSON层检查mesh/node weights和primitive targets的“声明存在”，早于gltf convenience defaults；原mesh.weights反例零decoder拒绝。 | 永久覆盖有值/空/null与unused mesh；独立补未引用空weights、第二primitive null targets及node null weights，direct与Host都拒绝、状态保持。没有删除字段或静态化后接收。 |
| F-838-R1-04 / P2 / CLOSED | raw node遍历全部nodes，matrix与translation/rotation/scale任一同时存在即Err，早于transform选择。原matrix+translation反例零decoder拒绝。 | 永久6个used/unused互斥反例及matrix-only/TRS-only逐几何正控。独立补unused node matrix+null rotation，以及合法matrix-only与复杂accessor组合。原bytes/SHA不改，合法parts和几何精确。 |

原输入脚本/源码直接来自 review-r1，本次重新编译链接当前已验证rlib；9个生成GLB与R1 baseline中的每个SHA逐字节匹配。日志为 [path4](path-replay.log.txt)、[GLB9](glb-replay.log.txt)、[Host5](host-replay.log.txt)。GLB9含合法2parts正控与8个非法；Host5含合法1asset/1object/1history正控及4个非法0asset/0object/0history。harness退出0本身不是合同证明，以上逐案字段与预期实际核对。

## 独立新增反例和合法组合

没有引用新增永久测试的构造函数。独立 [combination-inputs.py](combination-inputs.py) 生成 [13个持久GLB及期望/hash](combination-inputs/manifest.json.txt)，[combination-probe.rs](combination-probe.rs) 通过正式 `assets::decode` 和生产 `Host::dispatch → settle` 消费它们；[原始输出](combination.log.txt)。

- 4个合法控制共同组合interleaved POSITION/NORMAL base、POSITION sparse覆盖、U32 indices、非零normalized U16 UV；再分别变为省略default-scene、第二mesh无索引、matrix-only节点。全部精确保原positions/normals/indices、完整2parts和原raw bytes；实际Host接受组合基线并形成真实GPU内容。
- 9个非法组合分别使interleaved NORMAL normalized、sparse POSITION U32、U32 indices normalized、UV shape错误、unused TEXCOORD_1、unused matrix+null rotation、unused empty mesh weights、第二primitive null targets、node null weights。direct与Host必须返回同一个预检错误，排除stale/busy/无效路径提前拒绝。全部decode/upload增量0，旧document/stamp/history/Arc不变，最终GPU图像与此前相同；最后合法重试增加一个实例，asset数仍1、无GPU重传。
- 独立 [path-combination.rs](path-combination.rs) 直接走 `Storage::save_snapshot`，在另一个有效pending收据和真实旧项目存在时，分别使用相对blob、`ordinary/../ASSETS/./大写SHA.GLB`、verbatim canonical blob、带尾空格的已存在root-lock别名。全部在收据确认前返回reserved，原blob/entry/lock bytes及重开document、pending/dirty/path/stamp/history保持；解除探针锁后原pending可Verify，普通dotdot Save As可确认并重开。[原始输出](path-combination.log.txt)。这覆盖Windows本地路径及原单写者范围，不外推恶意并发路径替换或新网络/设备根支持。

初版组合探针使用固定asset_generation=0，在首个合法import后会从stale guard拒绝，不能证明Host预检；[无效初版记录](combination-stale-token-probe.log.txt)保留。改为当前UI token/selection，并要求Host错误精确等于direct预检错误后重跑，才计本轮覆盖。最初path replay复用了已有target目录，第四案在建立原项目时遇foreign nested目录；[setup失败](path-stale-setup.log.txt)保留，换全新GUID目录重跑后四案完整有效。这两项是探针setup错误，不是产品finding，也没有弱化断言。

## 本轮实际运行

所有cargo和探针构建在唯一实施树的driver目录执行；`RUST_MIN_STACK=16777216`，独立探针/原生exe链接 `/STACK:33554432`。未跑完整driver/AGP或任何root整档。

| 命令/入口 | 实测结果 |
|---|---|
| `cargo test --locked --test authoring_assets --test authoring_storage --test authoring_host -- --nocapture --test-threads=1` | assets17、Host7、Storage3，合计27 tests，0failed/0ignored；含50非法双入口、9合法control、512实际GPU与完整20轮owner及搬迁子进程。[scoped.log](scoped.log.txt) |
| R1原始path/glb/host探针重编译并重放 | path4 / GLB9 / Host5逐案符合关闭oracle，所有原GLB hash匹配；非把harness exit当pass |
| 新独立组合与save_snapshot探针 | 13GLB（4合法+9非法双入口）及4路径/收据组合全部符合预期，最终合法恢复；见上节 |
| `cargo check --locked --all-targets` / `cargo fmt -- --check` / `git diff --check` | 通过。固定SDK146继承warnings，driver/此次改动新增warning=0；无stray debug/TODO/FIXME。[check](check.log.txt)、[fmt](fmt.log.txt)、[diffcheck](diff-check.log.txt) |
| `python tools/verify_source.py` / `python tools/verify_plan838.py --controls` | SDK680/source264/closure479/17local/29edge；LF文本正控、Auto-source/自有GLB/原GLB/lock四实际负控与恢复全过。[source](source.log.txt)、[controls](source-controls.log.txt) |
| `cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432` + `python tools/check_scene_assets.py <本附件/native>` | 本轮重新构建并实际跑7案，聚合pass；positive phase20、9状态×4parts/origin独立像素，保存后删除原source/搬原root到新目录，异cwd新进程inspect比document/world/hash/2D/3D，缺blob/篡改拒绝及恢复。wrong-resource phase5、missing/transparent/wrong-GPU phase7、drop timeout phase8、frame1 incomplete phase0均exit1且到各自oracle。[native/results.json](native/results.json.txt)、[positive原始report](native/positive/report.json.txt)、[reopen](native/positive/reopen.json.txt)、[build](native-build.log.txt) |
| 现有独立source consumer输入审计 | 全264 receipt输入+manifest1与producer逐字节相同，SDK680输入逐字节相同；有独立target、没有继承root config。未把此审计冒认为本轮fresh冷导出/运行，后者仍merge-owned。见baseline复用边界。 |

已查看本轮native正控截图，两部分红/绿三角形与Iced资源/保存状态呈现一致；图片人工查看仅辅助，结论依赖上述实际像素与状态断言。

## AC / task / 当前证据

| AC | 结论 | 证据及边界 |
|---|---|---|
| AC-01 | pass | T01/02/08；正式837base/同树/写面、SDK/两锁/source实际正反控，265导出输入+680 SDK逐字节审计；原独立consumer当前构建结果按下节限定复用。 |
| AC-02 | pass | T03/04；F02..04关闭；assets17中的strict/no-fetch、sparse/primitive/u16/65536/512/513/hidden与raw/active/CPU/GPU预算当前重跑；独立组合direct+Host进一步检查读取前边界。 |
| AC-03 | pass | T03/05；schema3引用/身份/严格旧迁移/原bytes无重写与真实Auto仍在assets17通过；codec/document代码与R1不变，旧document/hierarchy范围证据有效。 |
| AC-04 | pass | T04/05；profile5/whole-candidate wrong-resource、共用staged owner/同SHA复用在assets17及当前native通过；AGP同设备/上传/最后pin的未变接缝证据限定复用。 |
| AC-05 | pass | T05/06/07；F02零解码/上传拒绝+已有Host状态/pixels+合法重试，Host7与assets17及native9状态真实全部parts/OBB/pick/gesture/undo/replacement通过。 |
| AC-06 | pass | T06/07；F01原例/永久/独立save_snapshot组合关闭；资源先于entry失败/residual/unverified/占用冲突与合法恢复、SaveAs/搬迁异cwd新进程当前实际通过。 |
| AC-07 | pass（work-owned） | T04/08；当前assets完整20轮/Host20轮实际owners、CPU/raw/mesh/pin和warm uploads；原双游戏/profile未变。完整兼容/双合成/原游戏和835..837原生矩阵仍属于最终pass后的merge门。 |
| AC-08 | pass | T08；SD01..04冻结且逐条对实现/证据，README/路径/残留/预算/stack/独立导出/生产边界准确。 |
| AC-09 | pass | T09；本独立R2及baseline/receipt/反例、遗漏/延期/workaround/健康扫描；四旧finding CLOSED，无新finding。 |
| AC-10 | pending | T10；完整冻结门、实际autographics-dev交付与知识/归档/guard/批量判定尚未做。 |

## 复用范围与规范核验

R1未受修复影响的document1/hierarchy10/interaction7/session4共 **22** 个测试结果、completion3和AGP资源1保留复用。旧32中Host7及Storage3已经本轮重跑，不能重复加总；work readiness“其余旧25”若以只减Host计算包含Storage3，本报告统一按未重跑22计算。旧测试文件/Host/Session/Preview/interaction/bin、原profile/fixture、AGP pipeline/test、manifest/两锁/root config与R1保持相同；`freeze-review.py`已自动核对，文本仅LF/CRLF传输规范化、二进制逐字节。原14个资产测试body也保持相同，本轮全部重跑。受影响的preflight和Storage绝不靠旧证据外推。

R1实际owner双负控101/101/恢复0只复用其oracle敏感性：正式20轮函数/故障开关脚本、VM/Host/GPU owner实现/依赖/配置/合法fixture均相同，当前20轮正控已经重跑。renderer511负控只复用R1 baseline明确的旧敏感性：其旧receipt HEAD与实际test快照不一致已经溯源到b0119d0d985a7a72472545be0ba8339172a30251；不把旧receipt算当前完整运行，当前512正控与513拒绝已重新执行。本轮没有改写pipeline或开启任何旧源码mutation脚本。

work-repair-r1 consumer的source/check/assets17+storage3原始结果只在当前export265输入/SDK680/配置逐字节复核相同的范围复用；编译产物属于该consumer自己的target，原增量复制清单仅6个receipt源码输入+新manifest，不把producer binaries作为证据。本轮没有再次冷建完整consumer；最终fresh导出/独立构建/native仍明确pending。consumer需独立设置 `RUST_MIN_STACK=16777216` 和 `RUSTFLAGS=-C link-arg=/STACK:33554432`，README/handoff已明确。

SD-01 add `docs/specs/auto-graphics/scene-assets.md`；SD-02 modify scene-document；SD-03 modify scene-hierarchy/authoring-session/viewport-interaction；SD-04 modify auto-source-driver/prototype-profile/project/integration-contract。8个modify目标存在、版本与R1未变，新target尚未发布；GOAL-007/008实际存在。新增typed组合、声明级morph/互斥变换、resolved保存目标规则是原strict/immutable合同修正，并非降约或未批准扩面，plan_revision仍1。schema3严格迁移、原字节SHA身份/profile5/owner事务/有限预算/目录单入口和残留、真实像素/native完成性、stack前提与代码相符。生产I2/I3及完整GLTF/PBR/网络/任意平台仍pending，无冒认生产能力或新scope。

遗漏/延期/workaround扫描核了全文件声明与unused路径、sparse/span/typed reader顺序、candidate发布与失败恢复、raw/CPU/GPU/实例预算、含hidden及history/保存snapshot owners、root→entry锁/foreign/residual/verify、SaveAs身份与receipt、原始字节/来源闭包和必需原生oracle。没有新确证缺陷或未批准延期；四finding未通过记债或删承诺豁免。R2是第二次final review、第一次repair后的再次验收；本次pass，未触发“两次final失败后work诊断”义务。

## 仍须完成的合入门

T-10在用户后续允许进入merge时，于本冻结提交统一执行完整locked driver/AGP test+build、原16 AGP cases、原双游戏/835/836/837原生正反及完成性、counter+content双合成门、新assets/owner/source/独立fresh-consumer矩阵。具体命令和应失败的负控oracle在当前 `research/scene-assets-handoff.md` 及其继承hierarchy-handoff矩阵，不能以本轮定向绿代替。门禁代码修复会改变冻结基线，须先受影响复审再新基线门禁。

门禁全过后才可实际合autographics-dev，master只接知识；随后canonical/ledger/upsert/索引、归档、guard后安全移树与批量到期收据才能关闭AC10。用户“先不要合入”下本轮止于reviewed9/10，树保留。root t/tv/tt/tb/tu/taa/docs_gen和完整driver/AGP/游戏merge档均未运行；tf仅按主检出批量到期规则。最终HEAD/clean复核见receipt。
