# PLAN-858 r1 — independent final review R1

2026-10-27，outcome **pass**；研究合同通过，无阻塞 finding。此结论不代表游戏、公共新接口、派生资源、离线包或 M2 已实现。T-05 完成；T-06/T-07 仍由 merge 执行。

## 受审身份与边界

- reviewer：Codex，本次独立检查上下文，未参与本次实现；未另开 agent。模型与会话 ID 未由运行环境暴露，不推定。
- review_scope=final，plan_revision=1；合同冻结为 [plan-reviewed-input](plan-reviewed-input.md)，SHA-256 `99fdb455a5b64b3b9403768dd782d12a649ca92970078a00223b1078976a9e1e`。
- 唯一研究树 `D:/autostack/.wt/lang-858/auto-lang`，`plan-858-dev`，受审 `a4812981e72df3d54ecb555dae5f77c3fabbd959`，base `7e498e4ccbce5f0a7b7b8008d8303b170d6f373e`；四文件全部是文档/索引，最终 status porcelain 为空。没有实施代码、SDK、锁或原始资产变更。
- 854 `523d27b863dbe306c89af0ea546334225a29f8e9`，已交付 855/autographics-dev `05fdc81c546aff735eefdb6c39fbc5fb35dc546a`，SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。854→855 API 目录 diff 为空。856 为 executing（当前 2/10），857 drafting，未计作公共运行相机交付。
- main 在复审期间由 `1a4ef0759fbedfbc8bc7a77ed5fcd84a3654be86` 前进到 `0543e431b646ca83c09d55560f96585adee73a6a`，为 823 簿记；AG Specs 与 goals 无差异。主检出的其他脏文件保持原状，复审只写 858 簿记、复审附件与风险记录。
- [baseline.json](baseline.json) 保存每个实际 Spec 输入（10 个 AG 文档及 goals）、四产物的 checkout/Git blob SHA、Plan/SD 身份和依赖。文本 checkout 为 CRLF、Git 为 LF，最初未经换行规范化的字节比较失败属于检查脚本误设；修正后逐文件内容与 clean Git 状态均一致，不是产品漂移。源素材逐字节比较不做换行放宽。

## AC → 任务 → 独立证据

| AC | 任务 | 判定 | 方法与覆盖 |
|---|---|---|---|
| AC-01 | T-01/T-04 | pass | worktree/branch/HEAD/status、base..HEAD 四文件写面及 854/855 身份；无引擎/855 在途代码写入 |
| AC-02 | T-02 | pass | G2-01..09、G3-01..08 逐项读；输入、60Hz、状态/数值/碰撞几何、敌人/物品/终点、失败/获胜新局及参考差异；固定参考对象与实际脚本相符 |
| AC-03 | T-03 | pass | 13/13 原件实际 SHA/长度与指定 Git blob 相同；4/4 GLB 的所有 URI/嵌入 buffer/动画与依赖列表相符；逐行用途/许可声明/通知及 2D/3D/共享分组复算；Godot 未明资产和制作源排除 |
| AC-04 | T-04 | pass | 17/17 需求行的基元、完整需求缺口与后继反例；对照固定 api/v1/v2/v5、Play 输入、编辑 camera、静态资产拒绝、特定样本 host；四类能力未混为实现支持 |
| AC-05 | T-03/T-04 | pass | 独立仓、同一公共引擎/SDK、资源身份、离线/搬迁/损坏恢复要求、预算建议和后继顺序可消费；CPU/128 节点/转换/实际包未伪报通过 |
| AC-06 | T-05 | pass | 本次独立核 AC/V/SD；全部选中资源行与 17 G 行语义审查；遗漏/延后/权宜扫描见下 |
| AC-07 | T-06/T-07 | pending merge | canonical/overview/plans/ledger/index、合入归档与 guard/清理尚未执行 |

| 验证入口 | 实际检查与结果 |
|---|---|
| V-01 | Godot 固定提交的两个 README、`2d/platformer/player/player.gd`；Kenney README、`scripts/player.gd`。Godot 无敌/二段跳与 Kenney 二段跳/相机相关输入属参考事实；自有一触失败/单跳/集齐获胜单列，无完整复刻宣称 |
| V-02 | `python docs/plans/attachments/858/review-r1/check_sources.py`（main 执行，脚本只读研究树/仓外素材）；[source-replay.json](source-replay.json)：13 SHA/长度/blob、4 GLB 闭包通过；独立改变一字节和截短控制分别被 SHA/长度检查拒绝 |
| V-03 | 2D 9 文件 149188B；3D 12 文件 198763B；并集 13 文件 203825B，最大 character 87624B；音乐/制作源所选为空。1MiB/2MiB 是建议目标，ZIP/最小下载仍未知。原件不冒充未来派生运行包 |
| V-04 | 固定版本 `git show`/`git grep` 与 current Spec 对照；公开有限平移/查询、专用音画、缺失完整服务、未核转换/性能分开；具体逐 G 复核见下 |
| V-05 | CPU/managed/128 节点与角色可行性先行，身份/输入/角色→触发状态→表现/相机→音画与独立闭环；旧 API 和编辑视口非隐藏前置，未要求 ACE。每片有正/反和停止/释放要求，无固定新计划号 |

## 资源语义与许可复核

固定来源：Godot `b761b4cd5718a7118f2c95eb55f181d0a5f004f1`、Kenney `3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`；仓外 clone 的 origin、HEAD 与提交对象均核对。恢复方式为从索引中的仓库 URL 在新仓外目录取指定完整 commit，再按索引 path 读取原件；无需保留实现树或依赖缓存字节。

- 四模型、共用色图和 coin 精灵：全部实际 bytes/blob/hash 一致；GLB asset 元数据无另列权利人声明，所有外部 URI 都是已纳入的色图。角色四轨 static/idle/walk/jump、无 skin；每模型 KHR_texture_transform、无 extensionsRequired，与报告相符。
- jump/land/coin 三个 OGG：完整原始字节而非 LFS 指针，无外部文件依赖；未来播放解码支持没有被本次 hash 检查冒领。
- [固定 Kenney README](https://github.com/kenneyNL/Starter-Kit-3D-Platformer/blob/3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0/README.md) 对包内 sprites/models/SFX 作 CC0 声明；附带 LICENSE 为代码 MIT。逐文件确认对应资产类别，字体使用单独许可，未以顶层 MIT 吞并第三方字体。纹理是这四模型的实际共同依赖，与模型一起追溯。README、LICENSE、字体许可原文均已进入哈希索引，当前不声称未来新派生文件已许可/发行就绪。
- 字体实际 name 表的作者/OFL 与附带 license 一致。上游许可文件保留名写 Lilita，TTF copyright 记录写 Lilita One；这是原始通知措辞差异，后继如修改字体须保留/核对两份原始通知，不能只凭文件名判断。当前只是选择完整未修改原件，没有发生重命名或许可转换。
- 索引 LICENSE.md 的混合声明标签应按其 role 和原文理解：MIT 在 LICENSE，资产 CC0 在 README；此处不把 LICENSE 本身当第二份 CC0 授权。后继 manifest 应按上述原文拆清，原始素材许可事实不受该描述精度影响。
- Godot 2D 原素材不再作为发行输入；50762B FLA 也明确未选/许可未核。没有用提交作者推断未明素材许可。

联网复核取得 [Godot 2D 固定 README](https://github.com/godotengine/godot-demo-projects/blob/b761b4cd5718a7118f2c95eb55f181d0a5f004f1/2d/platformer/README.md) 与 [3D 固定 README](https://raw.githubusercontent.com/godotengine/godot-demo-projects/b761b4cd5718a7118f2c95eb55f181d0a5f004f1/3d/platformer/README.md)。Kenney 网页返回 cache miss；未把失败当核验成功，其内容以 origin/固定本地 Git 对象及实际原件独立复核。

## 17 项能力分类的语义核对

代码路径均相对于 855 固定对象的 `experimental/autographics-auto-driver/`；表中 v1/v2/v5 为已交付 API 文件，不是未来服务签名。

| G | 本次独立核对结果/边界 |
|---|---|
| G2-01 | v1 ag_input_x/ag_move_local 存在；host/viewport::key 当前映射箭头和 V，不提供本稿 A/D/通用动作合同；完整目标仍有缺口 |
| G2-02/03 | v5 ag_move_and_slide 的有限位移/两接触/返回快照存在；地面/速度/重力/跳边沿/关卡失败不因此成为完整已验角色。高速度/角落必须留给有界可行性探针 |
| G2-04 | v2 ag_aabb_overlaps 和 v1 显隐可用，Auto 能维护一次收集状态；此处 missing 指完整服务/目标未交付，不得被后继误解为完全没有触发基元，亦不自动要求把业务移入 Rust |
| G2-05 | v1 已有节点平移且早有巡逻示例；本关敌人/双运动参与者/失败和事件顺序尚待公共游戏合同及真实 Auto 验收 |
| G2-06 | Play Stop/Run 恢复编辑态和新 VM 存在；不等于结束态 Enter 的新局、资源和输入代次要求 |
| G2-07 | 编辑 viewport::camera 不是脚本可控运行跟随/边界相机 |
| G2-08 | assets.rs 实际拒绝 images/textures/animations/skins；角色原始模型不能充作已交付 2D 精灵，派生仍未制作 |
| G2-09 | host.rs::enable_audio/score 绘制确为固定样本服务；公共资源绑定、事件音效/HUD 未因此获得支持 |
| G3-01 | v1 只有离散轴/平移，无公共相机朝向或节点运行朝向；相机相对移动/角色朝向目标有缺口 |
| G3-02/03 | 同 v5 有界运动事实，非三维角色/危险事件闭环；Auto 业务、碰撞顺序和预算失败需要后继验证 |
| G3-04 | 查询/显隐存在但隐藏非销毁；完整身份/重开代次/一次收集服务未由原件证明 |
| G3-05 | 真实 Play 新 VM 能清状态，但不提供所要求的公共游戏新局/异步结果合同 |
| G3-06 | 编辑相机和 Squash 固定相机不提供脚本跟随/环绕/缩放运行服务 |
| G3-07 | 四选中模型实际含外部纹理、角色动画；现行 static GLB 明确不收，保留未来适配或派生分支，没有丢表现凑绿 |
| G3-08 | squash_host::enable_audio/Score 是专用，未证明任意游戏可绑字体/事件音效或正确释放 |

分类可用于后继立约；missing 并不否认已列出的公共基元，亦不证明必须新增全部引擎机制。后继应优先探针验证 Auto 组合能力，再决定最小新增公共合同。

## 规范增量、风险与收尾

SD-01 **pass**。新 target `docs/specs/auto-graphics/m2-game-sample-research.md`，before=无固定研究结果；after=冻结研究事实/未来要求，不修改已交付运行语义。GOAL-008 存在；supersedes=[] 的原因是无旧规则退休或替换；new_spec_components 与 target 一致。原文冻结 [spec-delta-frozen.md](spec-delta-frozen.md)，SHA-256 `9b2ca30398ff0533bde7df4c0f633314e107c4d561cfee0c21edd54a827e8a9a`。当前不写 canonical/ledger。

遗漏/延期/权宜扫描：没有发现删减两款必需玩法、隐含第四来源、引擎私有宿主实现或把未测结果写成通过。尚待制作的 2D 可辨识帧、3D 表现适配、真实 Auto CPU/节点预算和正式包属于合同明确的后继工作，不是本研究完成的游戏功能；登记为后继风险候选。许可未核的 Godot/FLA 仍排除，未作为获准技术债交付。

Category A：`git diff --check base HEAD` 通过；所有内容、来源和索引检查如上。没有修改 Rust，未运行 cargo t/tv/tt/tb/tu/taa/tf 或 docs_gen；这些不是本 Plan 待补门。无编译警告检查需求，无调试输出或实现补丁。

pending_closeout_gates：merge 对冻结文档/Spec 变化做适用链接/内容检查；合入 main、canonical/overview/plans/ledger/index 沉淀、归档；移除树前实际 wt-guard clean，随后分支/组清理和 7/7 核实。不得把本次 5/7 研究复审记为 M2 游戏或发行交付。

next=auto-plan-merge（用户当前只请求检查，本次不合入/归档/清树）。
