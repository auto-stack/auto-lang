---
plan_id: PLAN-838
status: archived
completion_kind: delivered
feature_name: AutoGraphics静态GLB导入与可携带场景资源
plan_revision: 1
author: [Codex]
created_at: 2026-10-23
updated_at: 2026-10-24
current_step: 9
total_steps: 10
depends_on: [PLAN-837]
supersedes_spec_components:
  - docs/specs/auto-graphics/scene-document.md
  - docs/specs/auto-graphics/scene-hierarchy.md
  - docs/specs/auto-graphics/authoring-session.md
  - docs/specs/auto-graphics/viewport-interaction.md
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/scene-assets.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental, agp-resource-adapter]
---

# [PLAN-838] AutoGraphics 静态 GLB 导入与可携带场景资源

## 0. 变更摘要

837补场景组织之后，下一步让场景使用**用户自己的模型**：导入受限静态GLB → 实际Auto绑定资源并显示全部mesh → 放入组/编辑/撤销 → 保存场景及资源 → 将整个目录搬到别处 → 新进程重开一致。不再只使用内置circle/quad/cylinder/player/mob。

首版聚焦现有无贴图、固定颜色光照渲染能力，不扩材质/动画/纹理管线。新增内容寻址资产、有限资源注册及场景目录保存；不是通用资产管理平台或正式Forge产品。

837正式交付`cf2e99f50ee2a70b70ca2cfc23a5af56b405638b`为基面。final R1发现2P1+2P2；原树修复后冻结`eddf13ee6e5a9296bf11b0622561a669e7498b56`，2026-10-24独立final R2确认四项关闭、无新finding，当前reviewed9/10。T-10完整merge门/实际合入仍pending，用户“先不要合入”继续有效；[当前独立R2](../attachments/838/review-r2/review.md)、[修复交接](../attachments/838/work-repair-r1/readiness.md)，[原复审](../attachments/838/review-r1/review.md)保留历史。

当前合入状态：archived9/10；冻结完整门、autographics-dev快进与规范/台账已验证。T10仅归档提交及guard清理/到期收据待完成；下文pending是历史阶段记录。详[合入收据](../attachments/838/merge-r1/receipt.md)。

## 1. 目标

1. 通过真实Iced路径输入/按钮导入本地静态GLB，验证支持面后创建可选中、可挂组、可平移和撤销的3D叶子；支持将选中3D叶子换为已注册模型。
2. 资源身份绑定完整原始字节SHA256和格式/长度，实际Auto输出对象的资源引用，renderer/OBB/组边界统一消费同一受检资源，不能拿内置模型占位。
3. 场景包含自己的资源闭包，Save/Save As准备不可变资源后才提交入口文件；原路径消失、cwd改变、项目目录搬迁后仍可独立重开。
4. 错格式、未支持特性、超预算、缺失/被篡改资源、保存冲突或GPU准备失败明确拒绝且保last-good；后续合法导入/保存能恢复。
5. 原双游戏、837层级、836手势/实际合成内容门、835保存与history语义保持，形成可携带的有限创作雏形。

profile仍为Windows x64、stock Iced/wgpu单窗、固定2D/3D视口/相机、128组+叶子、64步/16MiB历史和1MiB场景入口。新增资源上限见§5.2，明确计量范围和边界行为。

非目标：贴图/图集/2D sprite导入、PBR材质、透明/双面材质、骨骼/变形/动画、Draco/meshopt、OBJ/FBX/网络资源、任意GLTF目录及外部引用、自动重载/原地覆盖资产、自动磁盘GC、ZIP/云端/协作、完整资产浏览器、生产AutoUI/RQHost/Forge仓迁入、ACE/AutoShader/CAP或新平台。原小游戏资产/协议不改。

## 2. 架构方案

```text
本地GLB路径 → 有界原始字节快照 → 格式/能力/预算预检
  → 原GLB decoder → 全mesh/材质/索引/变换复核 → CPU模型+SHA身份
  → 候选资产注册+文档编辑 → 显式Auto profile5资源引用/层级结果
  → 完整准入+同设备GPU资源准备 → 一次发布文档/资产owner/树/preview/history

Save snapshot → 目标root/assets/<sha>.glb不可变准备/核验
  → 原Storage单入口提交点 → 入口+完整闭包读回确认
  → 搬迁目录/新进程Open → 重验闭包 → Auto → 真实GPU/OBB
```

L1资源模块特性，写面限`experimental/autographics-auto-driver/`及AGP的**最小资源注册/容量公开接缝**（`experimental/autographics-prototype/src/render3d/pipeline.rs`与相关局部tests/docs）。沿用现有workspace/bin/渲染器，不改AGP Scene/CompositionGraph核心语义、shader、SDK、生产crates或根manifest/lock。driver可用现有锁中gltf版本做无IO预检；依赖闭包变化在T-02决策和源清单中明确。需要核心协议/引擎重构时回new，按L2先补正式架构设计。

| 门 | 成立条件 / 当前状态 | owner_stage |
|---|---|---|
| G837 | 837最终review、冻结完整门、实际合autographics-dev、Spec/归档/guard清理及到期检查完成；已核837 archived10/10，正式交付cf2e99f50ee2a70b70ca2cfc23a5af56b405638b，整门满足 | work进入T-01前核 |
| GSOURCE | 已在正式837交付冻结SDK/两锁/来源闭包；最终838来源264/SDK680/closure479/17local/29edge及独立consumer通过，身份见work-r1收据 | work T-01 |
| GSTART | 2026-10-23用户明确“计划838…实施”，G837满足后进入work；未授权自动review/merge或agent/新聊天 | work启动前 |

唯一实施树`D:/autostack/.wt/lang-838/auto-lang`、分支`plan-838-dev`，显式基于通过G837的autographics-dev完整SHA。work/internal/final review同树顺序执行，主检出维护Plan/附件。源码审毕仍合autographics-dev，master仅知识；不创建junction/symlink，移树前wt-guard。生产迁移门不因资源原型改变。

## 3. 技术栈

沿用837固定SDK（观察`9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`）、Session/Building、stock Iced、wgpu、gltf 1.4、serde/原SHA工具和AGP ModelData/MeshStore。导入GLB只提供几何和受限颜色，层级/资源选择经真实Auto profile5，GLB二进制解码/GPU持有归host。

profile1/2游戏、profile3旧平面样本、profile4层级样本保留语义和原回归入口。资源创作显式使用新profile5，不能把新输出塞入旧协议或将外部模型伪装成Mob。新文档schema拟为`autographics.scene/3`，v1/v2严格映射导入；新增字段不塞进原schema以躲过严格codec。

## 4. 需求分析与背景调查

### 4.1 授权与依据

规划阶段用户要求“837之后的下一个计划是什么，请提前规划”。随后2026-10-23用户明确“计划838…实施”，授权本Plan work；未自动授权独立review/merge、创建新聊天/agent或指定模型/预算。

已读[全局overview](../../specs/overview.md)、[图形overview](../../specs/auto-graphics/overview.md)、[project](../../specs/auto-graphics/project.md)、[scene-document](../../specs/auto-graphics/scene-document.md)、[authoring-session](../../specs/auto-graphics/authoring-session.md)、[profile](../../specs/auto-graphics/prototype-profile.md)、[集成边界](../../specs/auto-graphics/integration-contract.md)、[创作策略](../../design/strategy/autographics-forge-v06.md)、[837归档Plan](837-autographics-scene-hierarchy-and-group-authoring.md)、[R1](../attachments/837/review-r1/review.md)、[修复readiness](../attachments/837/work-repair-r1/readiness.md)及[final R2](../attachments/837/review-r2/review.md)。上述为规划时观察；work进入前重新核验837正式交付、canonical/ledger及归档清理，未用候选充当冻结基面。

来源Git对象/主检出文档指纹见[观察清单](../attachments/838/planning/source-observation.json.txt)。只读观察837修复提交及其原有代码，不改当前实施树；他人WIP保留。

### 4.2 观察事实与具体约束

| 实际触点（837候选源码） | 已观察事实 | 对本Plan的约束 |
|---|---|---|
| driver document.rs::Kind/Object/Document | kind仅五种，resources固定清单；v2/parent/groups严格codec | 增独立Model kind及内容资源引用，新schema显式迁移 |
| driver host.rs::Host::new/prepare/pointer_down/组选框 | 内置GLB include_bytes，models按kind固定索引；CPU全mesh bounds与GPU资源一致 | 资源表统一替代新增模型的固定slot分支，不能只改一个渲染入口 |
| preview.rs::Building及samples/authoring_hierarchy/main.at | profile4真实Auto派生world/effective，无资源ordinal | profile5增加可受检资源引用，保留父子/gesture合同 |
| AGP assets/glb.rs::load_glb_bytes/collect_node | import_slice可能先处理image；只显式拒绝baseColor贴图，递归烘焙节点，未覆盖任意外部模型准入 | 必须在此decoder前做无外部IO、预算/能力/图预检；不能把“旧loader返回Ok”当完整支持 |
| AGP render3d/pipeline.rs::MeshStore | key=(source_sha,mesh.name)，GPU索引用`as u16`写出；model可能有重复mesh名字 | 显式u16准入和稳定primitive身份，禁止截断/同名碰撞；不照搬任意GLB |
| 同文件MAX_INSTANCES/render_with | 当前容量512，超过时break | 在提交前按真实mesh实例数拒绝超额，不让静默漏画成为成功 |
| MeshStore私有map+公开pin/get_or_upload | 未见候选事务合并/回滚的公开接口 | T-02验证最小AGP staged/fork接缝，复用已有GPU Arc，失败不污染resident owner |
| driver Storage::load/save_snapshot/verify | 已有单文件owned锁、同目录temp/rename/readback及unverified恢复 | 用资源先行、入口最后提交；不声称旧helper已提供跨目录原子事务 |
| 837 R1及修复 | state/命令/hash相等曾漏判真实透明/残影；修复补实际端点与原位置像素 | 新导入也必须验原模型内容及材质/全mesh，不能只看hash变化/两次错误画面相等 |

838优先资源是因为837后组织/编辑已具雏形，仍不能消费用户自己的内容。此切片同时验证引擎资源身份/生命周期和创作可携带性。831/832仍受整Plan启动锁、迁移尚未完整交付，生产I2/I3不提前；本轮不进入其准备/实施。

## 5. 详细设计

### 5.1 下一入口与有界可行性

入口按G837→T-01→T-02执行；现T-01..08完成，下一责任角色为T-09独立review。T-01核正式交付而不是仅work pass，冻结同一实施树/锁/SDK和输入。

T-02在树内建立新`research/asset-import-feasibility.md`。最多比较两种**资产暂存/同GPU资源共享**接法：一个自有多mesh静态GLB，经真实控件Import→Auto资源ordinal→全部GPU内容+OBB→挂组/移动→Save As到另一目录→原导入文件不可用后新进程Open；另做越界索引、512实例边界、GPU准备失败、资源完成但入口未提交故障及恢复。探针用最小scratch数据/临时目录，不要求正式v3模块已存在，不把scratch当正式任务完成。

同时冻结支持面预检方法、profile5/资源映射、GPU staged owner和计量、格式/路径、失败残留/恢复、同步parse/compile时延。旧索引/mesh名碰撞/容量问题只能在新资源准入/最小接缝内闭合，不扩shader/通用引擎。若需要大改SDK/生产UI/renderer或本合同支持面无法成立，给确切缺口回new，不降为内置资源换色、仅复制文件或只在原目录可运行。

### 5.2 GLB支持面、预算与准入顺序

支持GLB v2、恰好一个JSON和一个BIN chunk、单scene/内嵌buffer、静态三角形、POSITION+NORMAL、可选无材质用途的TEXCOORD_0、U8/U16/U32索引或合法无索引三角形；所有实际索引必须≤65535且小于对应顶点数。节点变换烘焙沿用原loader；所有变换/顶点/法线有限、法线有效，节点线性变换非奇异且det>0；不通过反转/丢弃几何来修镜像或坏法线。

材质仅opaque/single-sided、无贴图的baseColorFactor（alpha=1），metallic=0、roughness=1、emissive=0；不宣称PBR。缺材质可映射成明确记载的固定默认颜色/上述值；显式不符字段则拒绝。任何images/textures、外部或data URI、动画/skin/morph、COLOR/JOINTS/WEIGHTS、压缩及未知extensionsUsed/Required、非三角形或透明/双面/纹理要求均在decoder前具名unsupported，不静默忽略。预检会检查整个文件的声明，不借未选scene或隐藏节点逃过范围/预算。

| 资源预算 | 单位/入口及上界 | 边界行为 |
|---|---|---|
| 原始文件 | 每GLB≤8MiB，JSON chunk≤1MiB；bounded read在parse前拒绝超额 | 不先read_to_end任意大文件 |
| GLB图/geometry | 文件nodes≤128，无环单父；展开后primitive≤16；每primitive顶点≤65536且所有索引可无损u16；每资产总顶点≤262144/索引≤786432 | count/buffer范围和展开预算先算，再收集/烘焙，不先分配后判断 |
| 资产集合 | 当前会话活跃并集（accepted/history/临时/待保存）外部SHA≤8，原始字节合计≤32MiB | 重复SHA不重复占额；超额不偷偷驱逐history或替换旧资源 |
| 解码/GPU | 外部模型最终CPU几何数组字节≤64MiB、GPU vertex/index预计字节≤64MiB，计所有在环candidate/resident唯一payload | 准备前预计并在实际调用处核算；临时parser开销另报告，不冒认进程硬内存上限 |
| 场景绘制 | 所有3D叶子潜在mesh实例总和≤512，含内置与隐藏对象、各叶子的全部parts；以公开实际renderer容量复核 | 全candidate拒绝，不能break漏后半场景或忽略隐藏后代 |
| 原文档/历史 | 128组+叶子、1MiB入口、64步/16MiB历史仍在 | 资源字节单独计，不靠缩短history假装资源有界 |

原始字节只读取一次有界快照，以其hash解析/复制，不按UI文件名作身份。先核GLB header/长度/chunks，再严格JSON/引用/accessor/buffer跨度、节点图、能力及预算；**在通过无IO预检之前不得调用import_slice/load_glb_file**。driver可用锁定gltf的无导入解析或局部严格GLB/JSON预检，T-02选定，不重新实现GLTF全规范。

decoder之后复核全部MeshData：非空geometry、positions/normals数量匹配、finite/范围、triangle计数/索引有效、实际展开数量及总字节。GPU key以原字节SHA+展开primitive稳定ordinal组成；重复mesh名称、同一mesh多节点烘焙的parts都保留独立身份，显示名不作key。不能通过改几何/丢mesh满足预算。非法/unsupported/资源超限/IO/内容hash冲突/GPU准备故障分别诊断；不是恶意原生代码安全沙箱或同步decoder抢占保证。

### 5.3 持久身份、schema3和真实Auto绑定

schema3保留既有固定resources清单和group/parent语义，新增有界assets条目（格式、lowercase SHA256、byte_length、可选显示名），以及Object的外部resource引用和独立Model kind。Builtin kind只能对应builtin定义；Model必须引用本Document声明的GLB SHA。唯一内容key拟为`sha256:<64hex>`，重复声明/格式不符/悬空引用拒绝。内存registry不包含用户路径，文件仅由entry所在root生成`assets/<sha>.glb`，不接受文档自带任意相对/绝对路径。

v1/v2合法输入严格映射为无外部assets的v3，原ID/组/顺序/local及画面不变，打开不自动写盘；源文件原始字节仍给冲突检查，语义saved_hash按当前canonical映射确认。显式Save才写v3；旧reader不能读v3须说明。未知版本、坏旧输入、重复/未知字段继续拒绝。具体字段标签/默认值由T-02固定，不能让缺失asset默认为内置模型。

profile5接完整层级和有界资源ordinal表，Auto保存并输出每叶子的资源ordinal、kind、world/effective和原叶子属性；ordinal只在本候选有效，Rust映射到builtin或受检SHA。新export/配置在新`samples/authoring_assets/main.at`定义，profile1..4原样本回归入口保留。Auto不解码二进制，Rust不能绕过其资源选择输出按旧slot直接渲染。

准入检查profile/ID/父/数量/资源ordinal/内容身份及world/effective；错SHA映射、外会话/旧候选、wrong-resource输出整包拒绝。actual Auto输出驱动GPU、全mesh OBB、组聚合轮廓和资源属性；三者必须指同一原字节模型，不以代理立方体或旧player代画。模型替换保留叶子ID/parent/local，新增引用与文档/history一次提交。

### 5.4 导入事务、history与GPU owner

沿用唯一Editor/Host和busy/epoch/revision/target token。Import读取路径→预检/解码→候选registry及Document→Auto→GPU准备→一次提交；完成前不修改accepted tree/selection/资源映射/history/dirty。导入并创建3D叶子是一条历史；资源面板可选择已注册模型再创建实例或替换选中3D叶子，命令绑定稳定ID而非显示名。新实例可放入同维group；预算在整candidate上校验。

同SHA同字节复用；同名不同字节得不同SHA，两份对象不互改。重复导入同资源的再次创建只增加新叶子，不重复解码/GPU上传；仅重复注册且无语义变化是noop。源路径文件后来变化不自动热载，重新导入按新SHA版本；不原地改旧资源。Undo/Redo回到准确引用/原像素，history所需原始字节/ModelData/GPU Arc仍有合法owner；64步/字节淘汰按原合同释放最后owner，不让redo读取原文件。

现MeshStore私有map不提供安全共享/分阶段提交。允许AGP增加最小公开fork/staged-adopt等价接缝：candidate复用未变GpuMesh Arc，新增mesh只由candidate owner持有，失败drop新资源而accepted store/Arc不变，commit一次交换完整资源视图。具体API由T-02证明；不得在共享resident store提前不可回滚插入，也不为每次导入重上传所有旧模型。actual write_buffer/创建计量覆盖尝试和成功，不能通过换store重置计数掩盖失败上传。

资源解析/Auto/GPU失败及取消均释放候选，accepted可继续编辑/保存，下一合法导入成功。取消在阶段边界检查，沿用可信同步parse/compile限制并测耗时；不承诺中断任意同步原生调用。活动gesture/结构事务/导入/保存互斥，Open/New/Close按accepted dirty和835未确认结果决策，迟到结果不能写新epoch。

### 5.5 可携带目录和单入口提交

场景入口`.agscene`的parent目录为项目root，外部资源固定放root/assets/<sha>.glb。首次Save As可选择已存在的目标parent；asset目录可由本会话按所有权创建。普通Save和Save As都从**accepted保存snapshot**持有的资产闭包准备文件，不依赖原导入路径/开发仓/cwd；不保存UI焦点、history、VM/GPU或绝对源路径。

在已有owned单写锁下：核根/资产目录及文件为允许的本地regular路径、无reparse跳转；有界准备每个不可变blob到同root owned临时文件→flush/hash/readback→无覆盖安装到规范hash名。相同目标已有blob只有长度/hash/支持面一致才复用；内容不符明确冲突，不能覆盖或删foreign文件。资源全部就绪并复核后，才调用适配的Storage提交入口文件（原temp→rename单提交点），最后入口和完整资源闭包读回确认。

这不是多文件原子替换：**入口前失败保旧入口/旧资源；本次新装且未被入口引用的正确不可变blob可留下并记录为owned residual，重试复用**。不自动删除它或其他资源，不后台GC。相同root受一个明确项目写锁保护，和原entry锁的顺序在T-02固定；占用直接拒绝，无降级闭包，无删陈锁猜测。新增blob、资产临时文件和失败残留都计入root资产存储≤8文件/32MiB的磁盘预算，每次写前有界盘点/预留，不能靠改临时名绕过；完成rename不另占一份。超额明确拒绝，不能让重复失败无限积累；异物或无法确认所有权的部分写入不改不删，报告实际残留位置和恢复方法。场景入口/其owned锁及temp另按原1MiB保存合同计量，不混入GLB预算。

入口提交后若入口/闭包读回失败，结果unverified、路径/saved hash不确认且不关闭丢弃；verify/retry重验完整闭包，识别自己的已提交内容并恢复，不能永久误判为外部冲突。旧epoch结果不清新文档dirty，保存旧revision也不清更新内容。Save As只有完整确认才切换当前root/path；失败时旧项目仍可用。入口最后提交不能省略“新进程实际打开”验收。

Open分阶段有界读entry→严格图/资源表准入→按固定路径读/核所有声明blob→解码/Auto/GPU准备→一次替换；缺失、hash不符、预算或准备失败保旧会话。打开后使用已接受快照，不watch外部变动；下一Open/Save/Verify遇篡改必须拒绝，不能回落原源路径。跨目录搬迁复制entry+assets后在原root和原源不可用、不同cwd的新进程重建；只拷入口缺资源应明确失败。

路径保证限本地同卷单写、无并发恶意换路径，沿用835不保证掉电/网络文件系统的范围；不可将只检查file_type::is_symlink当作Windows所有reparse检查已完成。测试不在worktree内创建junction/symlink；必要路径拒绝验证用既有安全工具或仓外owned临时夹具，且不递归删除未核绝对路径。

### 5.6 验证画面与范围收口

至少一个自有多mesh/多节点、分离彩色parts的GLB作为合法fixture，附生成方法/来源/许可；不靠只改原player文件名。其实际GPU画面需验证每个part的预定颜色/位置与背景，选中OBB/移动后端点/原位置/隐藏/undo/redo/替换/搬迁重开均与实际Auto输出一致。透明、漏part或错资源的真实GPU故障必须使同门红，即使command/state/hash自洽。

20轮导入/重复复用/替换/undo/取消/失败/Open/Close记录实际VM弱owner、GPU Arc/pin、CPU/原字节预算、目标/pipeline/上传守恒。未变资源warm零模型重传，动态资源在最后合法owner释放后才释放；授权external pin可延迟但不能丢失归属。CPU parse/compile/帧时及工作集实测，GPU timestamp/Iced内部/驱动驻留排除如实披露。

838交付后可称“有限场景组织+自有静态资产+可搬迁保存的独立创作原型”；仍非生产AutoUI/RQHost/完整Forge或通用引擎。I1原双游戏分母保留，I2/I3、迁移/发行、Shader/CAP保持各自前置，源码不入master。

### 规范增量

全为拟议，当前不写canonical。837新增scene-hierarchy须先正式沉淀；review冻结本Plan实际增量，merge再写overview/plans/goals/ledger。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/scene-assets.md | 仅builtin→受限GLB支持/预算/内容身份/GPU事务/可携带目录及失败残留 | 资源闭包可消费 | AC-02, AC-03, AC-04, AC-06, AC-08 |
| SD-02 | modify | docs/specs/auto-graphics/scene-document.md | v1/v2固定资源→严格迁移/v3 assets与Model引用、根/保存/完整闭包确认 | 不破坏旧文件/dirty保护 | AC-03, AC-06, AC-08 |
| SD-03 | modify | docs/specs/auto-graphics/scene-hierarchy.md; docs/specs/auto-graphics/authoring-session.md; docs/specs/auto-graphics/viewport-interaction.md | 固定模型→导入/替换单事务、world/资源/OBB统一，512潜在实例及history资源owner | 结构/画面/拾取一致 | AC-04, AC-05, AC-07, AC-08 |
| SD-04 | modify | docs/specs/auto-graphics/auto-source-driver.md; docs/specs/auto-graphics/prototype-profile.md; docs/specs/auto-graphics/project.md; docs/specs/auto-graphics/integration-contract.md | profile1..4→显式5与原受限API共存，实测独立资产创作边界；I2/I3仍pending | 不冒认生产/完整GLTF支持 | AC-01, AC-04, AC-07, AC-08, AC-10 |

## 6. 测试设计

以下入口/fixtures由对应work任务建立，规划时的未来设计本身不算执行证据，实际结果见§8/9。规划/work阶段未指定reviewer模型/新聊天且未创建agent；2026-10-24本次用户明确要求再次独立复审，final R2由独立子代理上下文`/root/review838_r2`在同一树执行，未参与实现且未再创建其他代理/聊天/树。实施暂停写入；修复回同树。实现会话自查需披露局限，不冒充最终独立验收。

未来838树driver目录入口：已有`python tools/prepare_sdk.py --repository D:/autostack/auto-lang`、`python tools/verify_source.py`；新增`cargo test --locked --test authoring_assets -- --nocapture --test-threads=1`（T-03..07建立）。AGP新增资源接缝tests由T-04建立并运行定向过滤；开发仅`cargo check --locked --all-targets`、fmt和旧codec/storage/hierarchy/interaction/host/probe_exit受影响测试，不整档。

新原生模式拟为`autographics-authoring --asset-check --report <path>`及新`tools/check_scene_assets.py`（T-07建立）；保留window/interaction/hierarchy-check与inspect。exact CLI/测试名可等价调整，T-08必须给可执行命令/实际数量，不能只写“测试通过”。

| case_id | AC / work owner | valid/invalid及状态 | 实际入口/observable oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/07；T-01/08 | 正式837/SDK/两锁/源/资产，源码漂移/缺资源 | 原verify_source/独立consumer实消费闭包与反控；旧游戏/profile/数据身份保持 | 不以837候选作冻结 |
| V-02 | AC-02；T-03 | 自有多part合法GLB/重复名多节点；外URI/image/skin/morph/纹理/alpha/材质不符/奇异/镜像/环/坏buffer | Import正式有界预检→decoder→复核；合法全部parts/归一稳定key正确，非法在外IO/GPU前具名拒绝，no-fetch断言；不能仅检测后验loader Err | 只声明静态flat-color子集 |
| V-03 | AC-02/07；T-03/04 | index65535与65536、顶点/文件/JSON/primitive/活资源预算两侧，512/513潜在mesh实例（含hidden） | 新admission及实际upload/render；边界合法不截断，超额整包拒绝；actual u16写/实例数与预检对应，不能只少画一个part | 不改大索引/大场景renderer |
| V-04 | AC-03；T-03/05 | v1/v2合法/坏输入，v3重复SHA/错长度/缺引用/错误kind/未知schema，原entry不写 | codec/映射/Open；旧内容world/画面保持、导入clean、raw bytes冲突身份保留；坏输入拒绝且无占位fallback | 旧reader不承诺读v3 |
| V-05 | AC-04/05；T-04/05/06 | 同名不同字节、同SHA不同路径复用、Auto wrong-resource/foreign/旧epoch | 真实Session profile5→映射→GPU/OBB；实际SHA/parts/像素正确，同SHA模型Arc/解码复用，错引用整包拒绝、兄弟不变 | Auto绑定资源，host解码原始字节 |
| V-06 | AC-05；T-05/06 | Import&Create/选中换模型/父组移动、undo/redo、busy/draft/末端/取消/失败→合法重试 | Host真实命令/gesture/Auto；1历史、正确ID/local/资源/world、全部parts实际像素+pick；拒绝恢复旧树/preview/history和资源owner | 无新游戏/全编辑器支持 |
| V-07 | AC-06；T-06/07 | blob完成前/完成后但entry前故障，entry后读回故障，occupied/wrong-hash blob/外部entry变化 | 原Storage适配+实际文件；旧entry bytes可运行，owned残留有界/报告/下次复用，foreign不改；已提交unverified重试重验闭包后恢复，旧epoch不clean新文档 | 单入口提交，非多文件原子/掉电保证 |
| V-08 | AC-06；T-06/07 | Save As新root、搬目录、原源/root不可用、不同cwd、只拷entry、坏hash/路径/缺part、合法修复重试 | 新进程实际Open/inspect/render/pick；移动完整闭包成功且具体颜色/parts/world对应，缺/坏明确失败；不依赖原路径/SDK开发仓/target缓存 | 本地目录可携带，非ZIP/网络 |
| V-09 | AC-07；T-04/08 | 20轮重复/替换/history/失败/取消/Open/Close，合法external pin | 真VM弱owner/模型CPU字节/mesh Arc/实际writes；candidate失败新owner释放、未变Arc平坦，最后owner释放；保留candidate/漏上传计量负控同门红 | 披露parser/驱动/Iced内存排除 |
| V-10 | AC-05/06；T-07 | 原生路径输入Import/树挂接/移动/换资源/undo/save，透明/漏part/错资源GPU故障、wrong-asset/drop-release/frames1 | 新asset-check经OS→Iced/router→Auto→top capture；按独立颜色/位置验全部parts/原位置背景/重开，真实GPU错即native/report/aggregate非零；合法完整exit0/passed=true | 必须可交互桌面，不只state或两错误截图相等 |
| V-11 | AC-08；T-08 | SD-01..04/README/research/实际支持/残留/CLI | 拟议Spec逐项对应执行case/边界、版本/来源可追溯，生产I2/I3不冒认 | 静态检查不替代数据/GPU/文件门 |

每case保留command/退出码、真实断言、冻结code/SDK/锁/fixturehash和原始报告。必须有实际变异/故障区分错误实现，尤其大索引truncate、512静默break、同名mesh碰撞、对错误资源的正确state以及资源先完成/entry未提交的恢复；无法运行的必需oracle保留gap，不算pass。旧证据仅在实现/依赖/配置/输入不变时复用，新profile/asset/多文件闭包不能借旧835/837通过外推。

最终review后，merge在冻结提交统一执行完整locked driver/AGP test+build、原16 AGP cases、原双游戏/835/836/837原生正反与完成性门、新assets完整正反/重开/搬迁、counter/content双合成门、source/asset/独立source consumer。准确矩阵由T-08登记。开发/review不跑root整档t/tv/tt/tb/tu/taa/docs_gen或上述完整档；tf只由主检出批量到期规则触发。本Plan扩AGP局部资源接缝，需复审适用原合成/同设备/计量/寿命门，不自动豁免。

## 7. 验收标准

| ID | owner_stage | 可观察标准/验证 |
|---|---|---|
| AC-01 | work | G837/GSOURCE及唯一树/固定SDK/写面、来源闭包成立；V-01 |
| AC-02 | work | 支持子集严格预检，无外部IO/静默忽略；预算/索引/512完整拒绝正确；V-02/03 |
| AC-03 | work | v3内容身份/引用和v1/v2映射/原字节保护成立，无路径/占位fallback；V-04 |
| AC-04 | work | 真实Auto profile5资源绑定及完整准入，同设备/owner staging、错误引用不发布；V-05 |
| AC-05 | work | Import/Create/替换/挂组/拾取/手势/撤销端到端正确，全部parts真实画面；V-05/06/10 |
| AC-06 | work | 资源先行/单入口最后提交、残留/冲突/unverified恢复、Save As/搬迁新进程完整闭包成立；V-07/08/10 |
| AC-07 | work | 旧游戏/层级/手势不漂移，预算/共享/最后owner/计量在20轮成立；V-01/03/09 |
| AC-08 | work | SD-01..04及准确支持/路径/残留/入口/资源/计量/交接完整；V-11 |
| AC-09 | review | 独立核AC-01..08、Spec delta、遗漏/延后/workaround/fmt/warnings/debug，冻结revision/commit pass且merge完整门pending |
| AC-10 | merge | 冻结完整门通过才合autographics-dev；canonical/ledger/归档/guard清理和批量到期收据实际完成，源码不入master |

## 8. 执行步骤

当前reviewed9/10：final R2独立确认F-838-R1-01..04关闭，AC01..09通过，T-01..09完成。T-10及merge完整门pending，未合入/交付；[独立报告/AC映射/反例与复用边界](../attachments/838/review-r2/review.md)、[收据](../attachments/838/review-r2/receipt.json.txt)。T-10实际完毕才归档交付10/10。

| task_id | owner_stage | 依赖 | 实际触点／新增路径与动作 | 关闭条件；AC |
|---|---|---|---|---|
| T-01 | [✅] work | GSTART/G837 | 主合同/837正式收据、838唯一树、prepare_sdk/verify_source/两锁/资产 | [x] 正式cf2e99f50ee2a70b70ca2cfc23a5af56b405638b，SDK680/来源247/closure479/17local/29edge及三来源反控通过；锁/换行复现差异见§9与树内research/asset-import-feasibility.md；AC-01 |
| T-02 | [✅] work | T-01 | driver Host/Storage/Building；AGP glb/MeshStore；新research/asset-import-feasibility.md | [x] asset_feasibility定向2 parent+实际child1通过（父进程child helper不另算可携带验证）；真实Iced/Auto资源ordinal+parent/local、双部件独立GPU颜色/完整OBB、staged Arc/失败丢弃计量、65536拒绝、512/513、blob先于入口故障保旧与复用、搬目录/删源/异cwd新进程通过；接缝/预算/严格预检/schema/profile/锁序决策见树内research/asset-import-feasibility.md，正式验收仍归T-03..08；AC-02, AC-04, AC-06 |
| T-03 | [✅] work | T-02 | 新src/authoring/assets.rs及tests/authoring_assets.rs；document.rs codec；必要锁内预检依赖 | [x] F02..04修复：全属性/indices类型形状normalized在typed reader前检查，raw morph/矩阵TRS声明拒绝；50种direct+Host反例、9种合法布局/变换控制，assets17/17通过；AC-02/03。 见[work-repair-r1](../attachments/838/work-repair-r1/readiness.md)。 |
| T-04 | [✅] work | T-03 | AGP render3d/pipeline.rs最小staged资源/容量接缝及定向tests；driver资产registry | [x] 实际u16 GPU/同设备staging 1通过、512末实例GPU正反控及20轮owner通过，见work-r1 AGP/capacity/owners证据；V-03上传/实例部分、V-05共享/同设备和V-09 owner部分成立，失败drop/旧Arc不变、计量不重置；AC-02, AC-04, AC-07 |
| T-05 | [✅] work | T-04 | 新samples/authoring_assets/main.at；preview/session/Host资源binding/Edit命令 | [x] profile5实际ordinal/whole-candidate拒绝、单历史/换模型/取消及跨路径共享通过，见14资产测试与最终native；V-05及V-06事务部分，profile5真绑定/全结果准入、资源替换/ImportCreate单历史/失败恢复；AC-04, AC-05 |
| T-06 | [✅] work | T-05 | assets/Storage闭包准备与提交/Verify，Host/Panel路径/资源控件、OBB/组边界/gesture | [x] F01修复：真实保存目标canonical身份后保留路径分类，覆盖Windows大小写/..及普通新scene正控；旧blob/entry/world/pixels/dirty/path/receipt保护通过；Host7+Storage3及assets17通过；AC-05/06。 见[work-repair-r1](../attachments/838/work-repair-r1/readiness.md)。 |
| T-07 | [✅] work | T-06 | 新asset-check/aggregate/自有fixture及原inspect；原probe完成性保留 | [x] review持久path4/glb9/host5前红后绿；新增3个永久测试覆盖四finding及兄弟路径/合法控制；当前native7案聚合pass，保存/搬迁异cwd子进程通过；AC-05/06。 见[work-repair-r1](../attachments/838/work-repair-r1/readiness.md)。 |
| T-08 | [✅] work | T-07 | README/new research/scene-assets-handoff.md、身份/consumer/资源计量、树内Spec proposal | [x] 新冻结source264/SDK680/closure479/17local/29edge，四来源反控及独立consumer当前assets17+storage3/check通过；README/SD-01..04/当前closure与merge矩阵完整；AC-01/07/08。 见[work-repair-r1](../attachments/838/work-repair-r1/readiness.md)。 |
| T-09 | [✅] review | T-08 | 同838树冻结code/Plan r1/Spec，主检出独立review记录 | [x] final R2独立pass，四finding关闭/无新finding；assets17+Host7+Storage3、R1 path4/glb9/Host5、新13GLB及4路径组合、native7、source正反/check/fmt通过，规范与AC01..08核验；完整merge门pending；AC-09。见[review-r2](../attachments/838/review-r2/review.md)。 |
| T-10 | merge | T-09 pass | 同树冻结完整门→autographics-dev；主检出知识/ledger/archive→guard | AC-10实际门/落点/沉淀/清理/到期收据；改代码先受影响复审再重冻，不提前交付 |

## 9. 复审记录

2026-10-24：stage: merge | PLAN-838:r1 | checkpoint: landed/ledger_refreshed/archived | reviewed_commit: eddf13ee6e5a9296bf11b0622561a669e7498b56 | delivery_commit: 37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c（仅规范/投影后继，实施/依赖/配置未改）| 完整冻结门55个入口/官方16case/原双游戏及创作/层级/新资产/owner正反/全新consumer通过，最终delivery独立AGP消费重建亦通过；四次OS输入早期失败保留并在同冻结代码重放命中真实oracle；规范/台账已校验落地。归档提交及guard清理仍pending，尚不声明最终merge pass。详[receipt](../attachments/838/merge-r1/receipt.md)和[gate audit](../attachments/838/merge-r1/validated.json.txt)。

2026-10-24：stage: merge | PLAN-838:r1 | checkpoint: gate_running | reviewed_commit: eddf13ee6e5a9296bf11b0622561a669e7498b56 | final_review: [R2 pass](../attachments/838/review-r2/review.md) | user_authorization: 本轮明确执行完整门、通过后合入autographics-dev/规范沉淀/归档/清树 | 同一唯一树plan-838-dev保持冻结，主检出代码面clean，他人文档WIP保留；完整driver/AGP/16case/原生与新资源/独立fresh consumer门串行执行；root生产整档无触面，不运行；T10未提前勾选，规范与台账尚未发布。详[门禁记录](../attachments/838/merge-r1/gates.json.txt)。

2026-10-24：stage: review | review_scope: final | final_review_round: 2（第一次repair后） | reviewer_identity: 独立子代理上下文`/root/review838_r2`，未参与实现；父实施上下文不代替本结论；精确模型/会话身份未暴露，不虚构 | plan_id: PLAN-838 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-838/auto-lang | branch: plan-838-dev | reviewed_commit: eddf13ee6e5a9296bf11b0622561a669e7498b56 | base_commit: cf2e99f50ee2a70b70ca2cfc23a5af56b405638b | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9、driver lock2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db、AGP lockc3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf、source264清单88c986e48458163517359b4003c809669854c8d3cf34a581ca1ddc2f57907b2a | spec_inputs: 9个canonical输入与R1相同，SD-01..04冻结SHA9f49bdb9699840af0add768a07508bc7a20904a19b69bb9f37d791730a630192；完整指纹见[baseline](../attachments/838/review-r2/baseline.json.txt) | acceptance_results: AC01..09 pass、AC10 pending | findings: F-838-R1-01..04 CLOSED；无新合同finding/必需gap/未决合同问题 | evidence: [独立完整报告](../attachments/838/review-r2/review.md)、[结构收据](../attachments/838/review-r2/receipt.json.txt)；assets17+Host7+Storage3全部0failed/0ignored，原path4/GLB9/Host5字节相同重放；独立4合法+9非法组合经direct及Host同预检错误/零decode-upload/旧pixels-history-owner保持和重试，save_snapshot+pending receipt的4路径组合保持；本轮重构建native7聚合pass（phase20/9×4 pixels/搬迁异cwd新进程/全部具体负控），source正反/check/fmt/diff通过。未重跑旧22/completion3/AGP1/owner双负控敏感性按不变输入/代码/依赖/配置限定复用；独立consumer265来源输入+SDK680逐字节审计，当前work consumer原构建证据限定复用，fresh完整门不冒认。HEAD与实施树clean复核不变，主检出其他WIP保留 | pending_closeout_gates: 完整locked driver/AGP test+build、16AGP/原双游戏/835..837原生正反/完成性、counter+content双合成、新assets-owner-source-fresh-consumer冻结门；其后实际autographics-dev/知识ledger/归档guard/到期收据 | next: reviewed9/10，用户“先不要合入”下保留树；T10待后续merge授权且完整门通过。未改实现/提交/整档/合入/canonical-ledger/归档清理或新建其他代理/聊天。以下work/R1为历史。

2026-10-24：stage: work | plan_id: PLAN-838 | plan_revision: 1 | repair_from: final R1 needs_fix F01..04 | outcome: pass | worktree_path: D:/autostack/.wt/lang-838/auto-lang | branch: plan-838-dev | base_commit: cf2e99f50ee2a70b70ca2cfc23a5af56b405638b | repair_baseline: 8494e06d4056e7f4e0f6dd1925e128bb93a83ea6 | code_commit: eddf13ee6e5a9296bf11b0622561a669e7498b56 | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9、driver lock2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db、AGP lockc3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf、source264清单88c986e48458163517359b4003c809669854c8d3cf34a581ca1ddc2f57907b2a | task_ids: T03/06/07/08重新关闭；T01..08 complete | evidence: [修复readiness/四项不变量闭合](../attachments/838/work-repair-r1/readiness.md)、[结构收据](../attachments/838/work-repair-r1/receipt.json.txt) | blockers: none；工作AC01..08 pass，open findings/required gaps/decisions均空 | next_responsible_role: review。原review持久反例path4/glb9/Host5已前红后绿，新增永久3case覆盖50非法声明双入口、9合法布局/变换及Windows路径别名/dirty/path/receipt不变。producer资产17+Host7+Storage3/独立consumer资产17+Storage3/check及native7均通过；源码正反/新SD冻结。第一次repair，不重置final轮次。源码7文件500+/17-已提交、树clean；同树保留并暂停实现写入。T09独立再审/T10冻结完整门pending，未整档/合入/发布canonical-ledger/归档清理或创建agent/聊天。以下R1 needs_fix和原work为历史记录。

2026-10-24：stage: review | review_scope: final | reviewer_identity: Codex当前独立复审上下文，未参与本交接实施，精确模型/会话身份未暴露 | plan_id: PLAN-838 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-838/auto-lang | branch: plan-838-dev | reviewed_commit: 8494e06d4056e7f4e0f6dd1925e128bb93a83ea6 | base_commit: cf2e99f50ee2a70b70ca2cfc23a5af56b405638b | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9、driver lock2b131d3d…、AGP lockc3df168e…、source identity15b2f909…，完整SHA/Spec输入见[baseline](../attachments/838/review-r1/baseline.json.txt) | spec_inputs: 当前canonical8目标/overview及冻结SD-01..04 SHA f029676ba7172a7eaa31f70557f4ade400609424b488405ec48f7452e9ca63b5 | acceptance_results: AC01/03/04 pass、AC02/06 fail、AC05/08 partial、AC07定向pass且完整兼容merge-owned pending、AC09 fail、AC10 pending | findings: F-838-R1-01(P1)路径别名Save As覆盖不可变blob并错误Confirmed，F-838-R1-02(P1)accessor语义类型漏检造成默认dev panic/错误float解释，F-838-R1-03(P2)mesh morph weights漏拒绝，F-838-R1-04(P2)matrix/TRS互斥声明漏检而静默忽略TRS；均原合同缺口，不记债豁免 | evidence: [完整报告/关闭oracle](../attachments/838/review-r1/review.md)、路径4案/GLB9案/真实Host5案持久反例、资产14/旧32/completion3/AGP1及source四反控均过、owner20轮双负控101/101恢复0，native首轮一次OS点击提前失败保留、第二轮完整7案命中实际oracle通过，check/fmt/diff过、固定SDK继承146warnings/新增0 | pending_closeout_gates: 最终pass后统一locked driver/AGP完整test+build、16AGP/双游戏/835..837原生与完成性、counter+content双合成、新assets/owner/source/fresh-consumer；随后实际autographics-dev交付/知识ledger/归档guard/批量到期判定 | next: work在唯一原树一次修齐F01..04、覆盖兄弟路径/合法控制及依赖证据后新提交再复审。当前executing4/10，T03/06/07/08重开；未修实现/整档/提交/合入/发布canonical-ledger/清理，HEAD与树clean复核不变。以下work pass为历史交接，不能覆盖本次final needs_fix。

2026-10-24：stage: work | plan_id: PLAN-838 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-838/auto-lang | branch: plan-838-dev | base_commit: cf2e99f50ee2a70b70ca2cfc23a5af56b405638b | code_commit: 8494e06d4056e7f4e0f6dd1925e128bb93a83ea6 | dependency_revisions: SDK 9576b4dad5ceb8b201adedd46f9689f25c2dd5f9，driver lock 2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db，AGP lock c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf，source264清单 15b2f909f56f407f8bd086ebc01eb75dc84998a8552135649ee3195ae0543354 | task_ids: T-01..08 | evidence: [work-r1 readiness](../attachments/838/work-r1/readiness.md)、[结构收据](../attachments/838/work-r1/receipt.json.txt) | blockers: none；work-owned AC-01..08 pass，open findings / required gaps / decisions均空 | next_responsible_role: review。冻结35文件4588+/88-，源提交已完成、树clean；保留同树，停止实现写入。原生7case聚合通过、producer/独立consumer资产14/14、受影响旧32+completion3、AGP资源1、真实owner/512GPU变异反控及source反控均有证据；复用界限和默认consumer栈失败的环境修正已披露。T-09/T-10/完整merge门pending，未review/merge/归档/清理/发布canonical/ledger，未跑根整档或创建agent/聊天。

2026-10-23：stage: work；PLAN-838:r1；用户明确授权实施，GSTART满足。G837已复核archived10/10、完整门、正式autographics-dev交付cf2e99f50ee2a70b70ca2cfc23a5af56b405638b、canonical/ledger/归档/guard清理及batch not-due。主检出无代码WIP，其余文档WIP保留。唯一树D:/autostack/.wt/lang-838/auto-lang、plan-838-dev、base同正式交付，root/branch已断言。固定SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9准备680文件，metadata closure479/17local/29edge；driver锁84182b4c30ad44ac4d6c43f0c68d49dbfbbbe6d390f41f2d7433a26182a88e7d，AGP锁c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf。继承来源核验首跑发现hierarchy example.agscene检出LF但登记CRLF；逐字节只转换换行即命中原登记1a6d26cc…，不改场景语义，恢复该字节后verify_source247及source-drift/missing-fixture/missing-GLB负控和恢复正控全过；T-08须使新的source consumer可复现该文本传输差异，不能把新检出失败隐藏。T-01关闭，T-02最小staged/GPU/真实Iced及可携带闭包scratch验证进行中，后续任务不提前勾选。技能来源D:/autostack/auto-musk/.agents/skills/auto-plan-work/SKILL.md；未创建agent/聊天，未跑整档或review/merge。

2026-10-23：stage:new；PLAN-838 revision1；outcome:blocked（G837尚未完成，草案内容可交接）；next:837在final R2通过后完成完整合入交付，838 work T-01冻结正式基面，再T-02可行性。用户仅请求提前规划，本轮未实施/建树/测试或创建agent。

规划阶段历史快照（非当前状态）：drafting0/10；T-01..10、AC-01..10、V-01..11、SD-01..04为新合同，不是执行pass。静态preflight见[检查记录](../attachments/838/planning/preflight.md)。最终review未开始，完整merge门pending；连续两轮同因不收敛，保留轮次并先诊断漏测/兄弟路径再修，不能改fixture/范围偷偷过门。

## 10. 待澄清事项

当前无work/review阻塞。F-838-R1-01..04经独立final R2确认CLOSED，没有新增finding、必需gap或待决合同问题；未降约或扩产品范围。T01..09完成，reviewed9/10；T10完整冻结门/实际autographics-dev交付/知识与清理保持pending。用户“先不要合入”继续有效，同一实施树保留且clean；本轮独立pass不等于已交付。[当前R2报告](../attachments/838/review-r2/review.md)；修复readiness、原R1及work-r1保留历史。第二次final为pass，没有触发两次final失败后的work诊断义务。

## spec-sync 回写记录

PLAN-838:r1：新增auto-graphics/scene-assets.md；按已审SD-02..04更新8个canonical合同，回写模块overview/plans、全局overview/GOAL-007/008并生成INDEX。台账按canonical稳定ID复用及最少design/review/report/test投影，原件备份/六段schema/无关raw条目及顶层字段保护、version+1、读回通过；见merge-r1/ledger-projection.json.txt。源码只合autographics-dev，master只接本知识/归档与证据。
