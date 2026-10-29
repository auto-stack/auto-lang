# AG M2 Demo Gallery与能力矩阵

> **计划管理归属更正（2026-10-29）**：AG引擎计划在auto-ui/docs/plans，Gallery独立内容计划在ag-examples/docs/plans；跨仓计划由主责任仓持一份合同。882/883已迁至auto-ui（drafting r2），880最新收据已交付归档并迁至ag-examples。auto-lang原位置只留定位指引，历史Specs/设计本次未批量迁移；下文早期“中央Plan仍在语言仓”表述已由本条取代。

> **879交付后续接（2026-10-29）**：引擎唯一实现仓已为auto-ui；880在途仍固定消费旧875。G11两维产品正式由[ag-examples/001](../../../../ag-examples/docs/plans/001-autographics-gallery-runtime-objects.md)承接，硬等[auto-ui/002](../../../../auto-ui/docs/plans/002-autographics-trigger-events-and-round-restart.md)实际引擎交付；[auto-ui/001](../../../../auto-ui/docs/plans/001-autographics-runtime-object-lifecycle.md)先交付对象生命周期，旧880已交付现有内容工具。G11目前仅规划，不改变在途880、未交付项不标Ready。

> **2026-10-29滚动更新**：877基础六项已交付（ag-examples6a223d9）；874/875公共角色与相机已交付（AG1cc68aba0），C07/C08现为有限公共P能力，G07/G08仓外内容尚待[880 GY-B](../../../../ag-examples/docs/plans/legacy/auto-lang/archive/880-autographics-gallery-character-and-cameras.md)。880固定875、按项锁保留旧六项861版本；两逻辑项四入口，实施后总8项14入口。本轮仅规划，可与[879纯迁仓](../../plans/archive/879-autographics-source-move-to-auto-ui.md)并行。下文的“待874/875/877”均为早期依赖背景，当前以前述实际收据和880合同为准；G09..12/最终统一发行仍待后继。OS自动化暂停，按最新裁定做变更相关抽样与未变证据复用，新内容必需AC不可省略。

更新：2026-10-28；设计基线v4，**877已executing 0/10、本地仓/树建立，尚无内容交付；878仍drafting**。先核[交接快照](tracking-handoff-2026-10-28.md)/最新Plan。归属[Design45](../45-autographics-wgpu-engine-and-game-prototypes.md)与[M2设计](m2-editor-and-public-game-prototypes.md)，路线见[M2战略](../strategy/autographics-m2-v06.md)，跨端见[专题](platform-architecture-and-research.md)。设计目标不等于实现。

## 1. 决策与保持面

用户要求补充three.js参考、Demo Gallery、首批示例难度/顺序、仓库/资源和并行路径。采用“能力矩阵→最小可运行示例→两款完整游戏”的组织方式：小示例解释单项能力，游戏验证多个服务协作，二者分别验收。

首批固定为下文12个**逻辑示例ID**，分四波交付；一个ID有两维或多个子入口时，全部列明子入口都通过才算该ID完成，不能用页面数量充数。仅复用M2已有或原定服务，不因参考three.js额外加入全套PBR、骨骼、GPU计算等能力。M2新增Gallery目标在后继实施Plan细化，不能把本次规划说成已完成展示。

[874](../../plans/archive/874-autographics-public-character3d.md)与[875](../../plans/archive/875-autographics-runtime-cameras.md)范围/AC/预算保持，不承担Gallery。GY-A=877、XP-0=878；877已进入执行准备、仓/树实际存在，不能继续按“未建仓”启动；878尚未交付。当前跟踪会话仅核对文档，不实施或接管，后继消费任务不塞回874/875。

平台仍为Windows x64原生研究线，源码仍autographics-dev；网页初版只做说明、截图/录屏与下载索引，运行使用本地公共入口。浏览器在线运行需后续Web目标合同；完整AutoUI、任意UI/2D/3D统一声明、ACE集成仍按原路线等待再议。

## 2. three.js样例的取材方法与难度分析

调查基线为官方`r186` tag下的[examples/files.json](https://github.com/mrdoob/three.js/blob/r186/examples/files.json)及下列源码，查阅日2026-10-28；不是浏览器实时运行或帧率测试。实际移植/引用代码前由实施Plan再钉commit、文件SHA与许可；参考效果不等于复制源码/素材或承诺API一一对应。

| 官方例子/目录 | 实际包含的依赖 | 对AG的拆分与先后 | AG实施难点判断 |
|---|---|---|---|
| [geometries](https://github.com/mrdoob/three.js/blob/r186/examples/webgl_geometries.html) | 多种Geometry、纹理采样、Phong/灯光、相机及对象旋转；还包含参数曲面 | 先G01/G02已有形状/静态GLB；纹理另等表现能力；参数曲面和运行旋转不混入基础例子 | 现有形状陈列低；完整原例需新几何/材质/变换接口，明显更高 |
| [interactive cubes](https://github.com/mrdoob/three.js/blob/r186/examples/webgl_interactive_cubes.html) | 屏幕坐标转ray、最近mesh相交、材质高亮、大量随机对象 | G04先演示已有编辑拾取；G05独立演示世界空间有限segment/sweep | 小对象编辑拾取低中；通用屏幕ray/精确mesh交点和大规模对象另立能力，不能把AABB当等价 |
| [GLTF loader](https://github.com/mrdoob/three.js/blob/r186/examples/webgl_loader_gltf.html) | loader、OrbitControls、HDR环境、模型列表网络读取、动画、异步编译与相机fit | G02先单个本地静态GLB，G08后接公开运行相机，G10再接纹理/节点动画 | 展示壳简单，完整资源图/解码/准入/寿命复杂；不直接照搬联网加载流程 |
| [sprites](https://github.com/mrdoob/three.js/blob/r186/examples/webgl_sprites.html) | 纹理、透明sprite、3D sprite组与正交HUD叠加 | G09只选M2所需2D精灵/帧序；G12再展示公共HUD | 显示单张图片相对低，帧身份/翻向/层序/暂停/释放中高；不额外承诺3D billboard |
| [animation keyframes](https://github.com/mrdoob/three.js/blob/r186/examples/webgl_animation_keyframes.html) | GLTF、Draco、mixer、环境/天空及orbit，模型还有独立署名许可 | G10先本地未压缩小模型及已选无skin节点动画；压缩/天空/混合树后置 | 播放单clip看似简单，但静态协议转动画输出、clip切换与资源保持复杂 |
| [shadowmap](https://github.com/mrdoob/three.js/blob/r186/examples/webgl_shadowmap.html) | 光源/投影/偏移、shadow pass、受光物体、模型与辅助显示 | 先记录AGP已有定向阴影为底层能力；通用可配置灯光/阴影不列首12项 | 展示已有固定阴影低；公共灯光配置/预算/跨pass一致性高，不能由渲染器内部实现推断已公开 |
| [compute particles](https://github.com/mrdoob/three.js/blob/r186/examples/webgpu_compute_particles.html) | GPU存储数组、compute初始化/更新、实例绘制、输入命中与计算/渲染同步 | 列为后M2能力候选，不纳入首批 | 高；需计算资源/调度/生命周期/着色接口，不能用Auto逐粒子循环替代并声称同级能力 |

以上分析解释了“画面简单”和“引擎支持简单”的区别。排序以现有公开入口、资源复杂度和状态/跨消费者合同为准，不按原HTML行数或截图效果排序。three.js用于图形覆盖面参考，Godot/Bevy继续用于角色/游戏/工具协作参考；AG额外保留2D与未来UI混合能力维度。

## 3. 能力矩阵

状态含义：**P**=已交付有限公共入口；**E**=编辑工具能力；**I**=底层/专用实验已有，不代表公共脚本服务；**N**=待支持Plan交付。它们均不表示Gallery示例已经完成。当前源码观察锚点autographics-dev `8b12632d7b5b1b0df72fbb4eef84297a054de31d`，权威事实以链接Specs为准。

| 能力ID/维度 | 当前支持与边界 | 首批示例/目标 | 支持路线与优先级 |
|---|---|---|---|
| C01 2D基本绘制/层序 | P：有限quad/circle、颜色/alpha；[场景](../../specs/auto-graphics/scene-document.md) | G01，先现有形状；不新增路径/曲线/任意矢量库 | 现有能力，首波 |
| C02 层级/显隐/平移 | P：同维树、平移祖先与显隐继承；[层级](../../specs/auto-graphics/scene-hierarchy.md) | G03；不许诺通用运行TRS | 现有能力，首波 |
| C03 3D静态几何/资源 | P：有限静态平色GLB及内建cylinder/player/mob；[资产](../../specs/auto-graphics/scene-assets.md) | G02，当前无纹理/动画/外部URI | 现有能力，首波 |
| C04 编辑导航/拾取 | E：856/857网格、导航、有限拾取/平移；[视口](../../specs/auto-graphics/editor-viewports.md) | G04明确为编辑操作教程，不称用户Auto pointer API | 现有能力，首波 |
| C05 查询/有限运动 | P：世界AABB/有限segment/sweep/move；[segment](../../specs/auto-graphics/segment-query-profile.md)与[motion](../../specs/auto-graphics/bounded-motion-profile.md) | G05，非无限ray/triangle精确拾取/完整物理 | 现有能力，首波 |
| C06 输入/2D角色 | P：character/1、单2D角色/静态solid；[2D角色](../../specs/auto-graphics/game-actions-and-character2d.md) | G06、原hard limits | 861已交付，首波 |
| C07 3D角色 | P：874公共character3d/1已交付，有限包络 | G07，六动作/归一/最终支撑；视觉朝向不提前认领 | 874后消费，第二波 |
| C08 运行相机 | P：875公共两维运行camera已交付，有限单角色服务 | G08两维子入口；含相机相对运动/同tick发布 | 875后消费，第二波 |
| C09 2D精灵/帧动画 | I/N：旧游戏专用资源不能直接推广为公共sprite | G09，静态纹理→帧序/翻向/状态 | 原M2精灵能力Plan，第三波 |
| C10 3D纹理/节点动画 | N：当前静态loader显式拒绝；选中模型无skin | G10a本地色图/材质→G10b idle/walk/jump/视觉朝向 | 原M2纹理/动画1–2个Plan，第三波分步 |
| C11 对象身份/事件 | P/N：瞬时查询有，动态实例/代次/事件服务无 | G11，真实创建/销毁、首次触发/新局清理 | 原M2对象/触发1–2个Plan，第三波可先于美术 |
| C12 音效/HUD | I/N：Squash等专用面已有，公共服务待建 | G12a一次音效→G12b与Auto事件一致HUD | 原M2声音/HUD1–2个Plan，第四波 |
| C13 UI/2D/3D嵌套 | I：830组合/输入已实测；非完整AutoUI声明与统一HIR | 独立研究展区R01，可链接原演示但不计首12个公共示例 | 保留研究事实；完整统一等ACE后议 |
| C14 灯光/阴影 | I：现有固定定向光/阴影；无完整公共灯光编辑合同 | 记录支持限制；公开light/shadow对照例后置 | 后M2候选，未加新必需服务 |
| C15 完整PBR/环境/后处理 | I/N：当前固定不透明平色材质不等于通用PBR；多pass效果待建 | HDR/IBL、bloom/AA等逐片规划 | 后M2候选 |
| C16 大规模实例/LOD/粒子/compute | N：小场景与已有渲染实例缓冲不等于公共实例化/计算接口 | 明确负载和GPU能力后建立对应样例 | 后M2候选，不抬现有128节点界 |
| C17 骨骼/morph/通用导入 | N：当前不支持，M2选中节点动画不依赖skin | 骨骼混合/IK/更多格式各自调查 | 后M2候选 |

矩阵后续每项记录公共profile、支持/拒绝集、实际引擎版本、示例ID、证据及缺口；只允许在支持Plan实际交付后从N转P。Gallery能展示I/E能力，但须另标“研究演示”/“编辑工具”，不得把其标为公共Auto接口。

### 平台与呈现方式是独立维度

C01..17能力行保持，另按Windows原生、浏览器WebGPU、移动原生、移动Web/WebView记录实际profile。Windows的P/E/I不能自动复制到其他平台；示例项目仍待Gallery实施，Web/移动列只能从未测开始。每项记录宿主/执行环境、GPU能力要求、输入方式、资源provider、DOM/Canvas/可采样UI适用范围、已验证/受限/不支持与证据。

网页catalog/视频不是WebGPU在线运行；DOM overlay不等于3D深度正确UI纹理，浏览器内G04编辑教程也需独立工具/输入验收。跨端研究先消费G01/G02小场景，再测试Vue/Canvas和3D挂UI；探针只列研究区，不充当12项公共示例已跨端交付。平台限制与XP顺序见[专题§3/6](platform-architecture-and-research.md)。

## 4. 首批12项、顺序与复杂度

难度拆成**示例内容 / 引擎补齐**：低=重组小型现有工程；中=多状态、输入/资源/跨维验证；高=需要新资源或运行协议与多消费者一致性。“已有”仅指支持能力，所有Gallery项目本轮都未制作。

| ID/名称 | 最小内容与通过结果 | 内容/补齐难度 | 前置与顺序 | 可并行工作 |
|---|---|---|---|---|
| G01 2D形状与层序 | 少量quad/circle、alpha重叠、显隐；改颜色/位置后真实变化；不加纹理或任意绘图语法 | 低 / 已有 | W0第1；现有scene入口 | 可与874/875同时做仓外内容 |
| G02 本地静态GLB | 一个已许可平色小模型、静态取景、搬迁重开、坏hash拒绝；不默认orbit/纹理 | 低 / 已有 | W0第2，与G01无硬依赖 | G01/G02可分别制作；GPU验收串行 |
| G03 层级与继承 | 两维分别建同维小树；移动父组/显隐使后代同步、Stop恢复；无运行旋转/scale writer | 低中 / 已有 | W0第3，复用G01/G02素材 | 自有Auto源码可并行，不改引擎树 |
| G04 编辑网格与拾取 | 两维编辑教程：导航→选中→平移/吸附→保存重开；不同视图拾取仍对应对象 | 中 / 已有E | W0第4；消费856/857 | 场景/说明独立；明确非游戏pointer脚本 |
| G05 查询与接触实验台 | G05a二维segment/sweep、G05b三维segment/sweep；最近阻挡/无命中/合法初始接触对照，有限几何显示 | 中 / 已有P | W0第5；复用Beam/Sight/Dash/Maze，至少两维子入口 | 不依赖874/875；不做新屏幕反投影/三角ray API |
| G06 2D角色跑跳 | 两布局、松手/跳边沿/空中重跳拒绝、地墙顶/Stop；公开源码参数4→5改变实际运动 | 中 / 已有P | W0第6；消费861，不附加完整玩法 | 内容可与874/875并行，固定已交付引擎pin |
| G07 3D角色跑跳 | 两布局、四向/对角归一、z墙与最终支撑；固定方向基，明确尚无交互相机/动画 | 中 / 高（874既定范围） | W1a；874实际交付后；不等875 | 可与875实施并行，独立内容树消费874产物 |
| G08 两维运行相机 | G08a 2D跟随夹界/等比内容区；G08b 3D orbit/zoom与相对移动；两维全部通过才完成G08 | 中 / 高（875既定范围） | W1b；875实际交付后 | 说明/场景草稿可早准备；实际接口与运行证据后补 |
| G09 2D精灵与帧动画 | 先一张自制测试图验证alpha/采样/翻向，再idle/run/jump/fall；Pause/Stop不残留旧帧 | 中 / 中高（原M2能力） | W2；公共sprite交付后；角色联动消费G06 | 自制低分辨率素材/许可清单可早做；不依赖3D模型支持 |
| G10 纹理模型与节点动画 | G10a色图/UV变换/离线资源闭包；G10b同一无skin模型idle/walk/jump与视觉朝向；坏资源明确拒绝 | 中高 / 高（原M2能力） | W2a纹理先于W2b动画，后者依赖G07及动画/朝向公共服务 | 模型/纹理盘点可与874/875并行；纹理先用自制棋盘，再选中资产 |
| G11 生成、触发与重置 | 小场景创建/销毁对象，触发一次计数、旧代次拒绝、新局恢复；先用现有平色资源 | 中 / 高（原M2能力） | W2；对象/触发交付后；不依赖G09/G10美术 | 可与表现路线并行；协议接线须避开Play/assembly共同写面 |
| G12 音效与HUD | G12a公开动作触发一次音效并验证Pause/Stop；G12b消费G11事件显示0..3/重置，声音与文本不重复/错局 | 中 / 中高（原M2能力） | W3；声音/HUD交付；b再依赖G11事件 | 音频许可/裁剪可早做；a不等触发系统，b必须等事件合同 |

W0可形成第一批6项可运行内容，W1扩成8项，W2扩成11项（G09/G10/G11不要求互相全串行），W3补齐12项。优先级是依赖优先级，非固定日期或要求美术等前一个页面制作完。G10/G12各有分步完成状态，半项完成不能算整项完成。

### 每项应有的独立判别

- G01/G02/G03：实际图像＋可变源码/场景；图像全黑/错误层序/漏子节点不能只靠加载成功过门。
- G04：真实OS编辑和保存重开；录屏不能代替可执行教程工程。G05：独立最近交点/接触几何断言，明确有限AABB而非mesh精确拾取。
- G06/G07/G08：数值＋实际OS＋合成内容；冻结输入、旧heading/相机不动等代表性负控能被门检出。
- G09/G10：正确纹理/帧/clip/朝向被实际渲染，错帧/漏图不绿；模型动画不以骨骼全功能包装。已有静态资源保持及失败last-good规则由对应新能力合同承接。
- G11/G12：对象ID/代次、一次事件与真实显示/声音消费者一致；销毁/Stop/重开后无旧事件/残声，不能只数发送命令。

每个后继Plan固定适用平台/profile/上限、正负例、真实入口与证据来源；预算不足/能力缺失必须列为blocked，不用专用Rust业务或假的截图填补。

## 5. 仓库、目录和资源组织

### 5.1 仓库职责

- **引擎仓**：现阶段autographics-dev内AG源码；保留最小回归夹具和原研究资产，提供公开运行/编辑入口、版本与下载索引。Gallery不触发旧资产搬迁/删除。
- **独立`ag-examples`仓（建议名）**：12个小型示例、说明/目录元数据、许可、锁和小必需资源。仓外运行验证是主要交付入口，不能只是链接引擎tests。
- **独立`ag-platformer-2d`、`ag-platformer-3d`仓（建议名）**：两款完整游戏，Gallery只索引其截图、简介、固定版本与下载链接，不复制游戏代码/大资产。

远端所有者、确切名称与创建动作在未来实施Plan确认；本轮不创建。示例内容仓与引擎实现树分开，未来每个Plan每仓一棵独立worktree，review仍顺序用同一树。

```text
ag-examples/
  catalog/                     # 分类/状态/预览索引
  examples/<Gxx-slug>/
    README.md                  # 目的、操作、限制、预期画面
    example.json               # 元数据（拟定，不是已发布schema）
    project/                   # 独立源码/scene/配置/本地资源闭包
    checks/                    # 小型可复现断言/入口，不嵌引擎实现
  assets/manifest/              # 仓级来源/许可/内容身份索引
  tools/                       # 通用列举/校验/启动包装，不含游戏逻辑
  licenses/
  previews/                    # 小预览；大录屏外置
```

`example.json`未来至少描述ID/子入口、类别（公开Auto/编辑教程/研究）、能力ID、engine/SDK/profile/平台、源码/资源manifest、操作与已验证限制、Ready/Blocked/Research状态和支持Plan来源。锁与清单机器可读，但本轮不定义另一个引擎场景/资源格式。包装工具只把固定元数据传给现有公共入口，禁止调用私有Host/MCP测试指令驱动正常示例玩法。

### 5.2 资源和下载

小源码/scene/必要小图和模型随Git；优先自制简单几何/棋盘和已有明确许可的低多边形资产。参考three.js示例不授权使用其所有模型/照片/字体/声音，逐文件核许可、署名、源版本、内容SHA和是否允许派生。

大纹理/音乐/制作源/长录屏作为可选固定版本包；沿用858“实测后选择大包”的思路，单个>1MiB为外置候选而非强制新运行限额，不把仓库5MB入库钩子当下载预算。基础启动不下载所有示例/游戏/高清源，不承诺未经测量的总大小。每个example、共享包、预览和完整ZIP分别实测字节。

共享源资源用内容hash去重与许可索引，**每项可下载包仍须自包含**：公共资源重用在打包时按manifest实体化到该项目支持的本地资源布局；原始制作源、转换产物分别记录hash/转换版本/参数。不得依赖父仓路径、symlink/junction、生产者绝对路径或仅有的全局cache。缺失/篡改先拒绝，修复后实际再运行；大小/路径/解包边界在分发Plan冻结。

提供三种入口：轻量catalog索引；单项源码＋必需资源ZIP；可选全套内容包（引擎按固定版本单独取得）。单项ZIP收到后能离线运行、编辑、保存、搬迁重开；有需要时完整发行组合包含固定引擎。网页预览与原生可运行包分开标识，不能把视频播放当浏览器引擎已支持。

## 6. 并行路线与后继工作包

本节为分工/依赖，**不是立即实施授权**。877/878已正式起草，其余不预占编号，不替874/875追加任务。

| 工作包 | 工作/写面 | 能与谁并行 | 必须等待/避免 |
|---|---|---|---|
| [GY-A/877](../../plans/archive/877-autographics-gallery-catalog-and-basic-six.md) | 目录/六项十子入口/单项包，固定消费861公开入口 | 可与874/875/878并行，不等其结束 | 唯一ag-examples树已建、executing 0/10；不重复建仓/改引擎或锁，按owner最新记录接续 |
| GY-R 资源准备 | 自制小测试图、许可/来源/纹理/clip盘点、派生脚本方案、大小清单 | 可与874/875及GY-A并行 | 只证明准备结果；不能把静态loader拒绝的模型写成已可运行；正式转换和导入依支持Plan |
| GY-B 角色/相机增量 | ag-examples的G07/G08，消费公共接口 | G07在874交付后可与875实施并行；G08在875交付后做 | 不调用未来接口；本包不修改874/875任务或占其实现树 |
| CAP-P 表现支持 | 原定2D精灵、3D纹理/节点动画/朝向及相应G09/G10 | 资源研究可立即单独规划；2D/3D实现可在共同资源身份/输出合同冻结且写面互斥后并行 | 新能力Plan先冻结资源/发布规则；公共Registry/输出协议/assembler变更串行落地 |
| CAP-E 对象/触发支持 | auto-ui/001→002交付引擎，ag-examples/001独立交付G11 | G11等002实际交付；各自独立review/merge | Play、callback、句柄/帧协议与874/875重叠时须排共享接线顺序，不靠分支名假定隔离 |
| CAP-A 声音/HUD | 原定服务及G12a/b | 音频资源准备早做，a独立于触发，b等事件合同 | 不把专用Squash宿主当公开音频/HUD |
| GY-C 发布与整体验证 | final版本矩阵、12项所有子入口、索引/单包/离线迁移 | 内容就绪后整理可与两游戏开发并行 | 最终验收统一引擎/SDK基线；共享资源/重型/原生GPU测试单实例串行 |
| [XP-0/878](../../plans/archive/878-autographics-platform-boundary-and-dependency-audit.md)及后继XP | 878只读静态审计；其他双端绘制/Vue/移动仍为后继候选 | 878可与874/875/877并行，不依赖877内容交付 | 878不编译/运行/迁仓，不增M2前置；XP-1可先用现有scene，不被新Gallery阻塞 |

原引擎874→875序列保持；旁路使用已交付固定产物，不能追`latest`。候选分支变动不是可消费交付。Gallery合入内容不改变引擎默认路径；共同CI/构建入口和资源协议修改由明确单一owner负责，冻结后消费者才接线。每个写面独立Plan/树，禁止多人共改同一实现树；本文件不创建或消息调度agent。

GY-A预计独立1份Plan，GY-B消费G07/G08再独立1份；G09..12在尚未冻结的对应能力Plan中明确认领仓外内容任务，不能倒塞进874/875。GY-C优先并入原M2分发/验收Plan，规模需要时再独立1份。**Gallery净增估计2–3份Plan**，不按12个示例各立一份、不重复建设引擎能力；若内容/旁路适配超过该规模则显式拆分重估，不能按零成本排期。

该估计仅含当前桌面Gallery。XP与浏览器/移动产品化另按研究结果估算，不为让数字不变而把新平台适配隐入内容任务；本轮不实施或取号。

## 7. M2收敛与后M2边界

M2 Gallery增补目标：轻量分类目录＋首12项全部必需子入口，在同一最终公共引擎/SDK基线上验证；两款游戏作为独立工程被索引。示例先用各自已交付pin开发，M2收敛时统一版本并重跑受影响项，不能拼接不同版本的绿灯冒充同一发行。

后继Plan需冻结：GY-AC-01目录可按能力/维度/类别查找且Blocked不会标Ready；GY-AC-02十二项及子入口真实内容/输入/Auto/编辑语义成立；GY-AC-03单项下载/完整资源闭包、坏包拒绝/修复、离线搬迁、固定来源与许可；GY-AC-04原引擎/两游戏兼容和最后owner/资源证据。原M2-01..08不因Gallery被删除或降级；新增[M2-09](m2-editor-and-public-game-prototypes.md#7-m2验收矩阵)只承担本节范围，不加入后M2图形效果。

完整PBR/IBL、可配置灯光阴影、后处理、GPU粒子/compute、规模化实例/LOD、骨骼/IK、曲面UI及浏览器在线运行属于矩阵后续候选；根据用途/实际支持/资源预算逐个立Plan。M2首12项全部完成也不能宣称达到three.js全功能覆盖。

## 8. 首版设计核对记录（历史；正式规划见877/878）

只读输入：canonical AG overview、scene-document/scene-hierarchy/scene-assets、behavior-api、segment-query、bounded-motion、editor-viewports、character2d、prototype-profile、surface-composition及M2设计/策略；另核autographics-dev samples中现有behavior/segment/sweep/motion/character2d工程路径。外部参考为§2官方r186源码，只做静态分析，无运行原three.js示例、无帧率/渲染效果实测。

受保护Plan的规划前SHA-256：874=`8e8d2b6093837e368a2cbfbc0e1e40be75e218b2bdb63367f19055d2f4a0dfad`；875=`4b88df0d35b6c5c7e9838a33e84ba9c08c5e7fb964ed70afffed36030d50534c`。本轮文档预检应复核二者不变、.next-id未取号；不执行Cargo/native/GPU/素材制作或Gallery实现。

本轮实际文档预检exit 0：17个唯一能力行、12个有序示例行、M2-01..09共9项；6份相关AG设计/策略的110个本地文件链接均可解析，无冲突标记/行尾空白。874/875 SHA-256与上述值逐字一致，.next-id仍876；`git diff --check`目标范围通过（00-intro仅提示既存CRLF规范化）。未运行Cargo/原生窗口/GPU/素材管线，未制作示例/建仓/取号；结构预检不代表任何Gallery或新引擎能力已通过运行验收。

## 7. 2026-10-29 表现阶段接棒

本节更新前文历史候选状态：引擎auto-ui/001、002已实际交付；ag-examples/001 G11正在实施（主Plan状态可能滞后，不从勾选单独推断）。新引擎合同为[004 PNG/2D帧](../../../../auto-ui/docs/plans/004-autographics-public-images-and-sprite-frames.md)→[005 3D纹理/UV](../../../../auto-ui/docs/plans/005-autographics-textured-models-and-uv.md)→[006 节点clip/视觉朝向](../../../../auto-ui/docs/plans/006-autographics-node-animation-and-visual-facing.md)，均drafting、未实施，按实际delivery顺序推进。

G11继续固定002，能与该引擎阶段并行。G09需004交付，G10a需005，G10b需006；它们的内容计划各自在ag-examples后续取本地号，不混入引擎合同，本轮未占ag-examples/002。共同catalog/gallery.py/verify_packages改动须在ag-001实际交付后冻结基线，不消费在途工具tip。各自独立review/merge/离线包，GPU单实例；G09/G10/G12未交付，首12项未齐。

新增资源/预算/准入与共享发布设计见[auto-ui专题](../../../../auto-ui/docs/design/autographics/m2-presentation-and-resources.md)，来源缺口见[基线](../../../../auto-ui/docs/plans/attachments/004/planning/baseline.md)。素材准备不等于运行支持，旧858缺失清单不由同号ACE材料替代；固定原件身份恢复由005 T-01认领。
