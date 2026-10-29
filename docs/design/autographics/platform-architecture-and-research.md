# AutoGraphics桌面、浏览器WebGPU与移动端架构边界

更新：2026-10-28；设计基线v3，**878仍drafting、未交付跨端审计/适配**；874/877已有现场在途变化，按[交接快照](tracking-handoff-2026-10-28.md)与最新owner记录核实。归属[Design45](../45-autographics-wgpu-engine-and-game-prototypes.md)，与[M2设计](m2-editor-and-public-game-prototypes.md)、[运行设计](m2-runtime-and-character-control.md)、[Gallery](m2-gallery-and-capability-matrix.md)共同使用；实际交付以[Specs](../../specs/auto-graphics/overview.md)为准。

## 1. 授权、事实与当前范围

用户2026-10-28要求补跨端设计，随后要求将GY-A/XP-0正式规划，分别为[877](../../plans/877-autographics-gallery-catalog-and-basic-six.md)/[878](../../plans/878-autographics-platform-boundary-and-dependency-audit.md)。874/875保持原范围；当前仅规划，不实施审计/适配/探针，不改引擎，不建仓/站点/agent。其余XP及迁仓不预取号。

当前实验为Windows x64单窗、wgpu/Iced、固定旧AutoVM SDK，源码autographics-dev；[组合/输入](../../specs/auto-graphics/integration-contract.md)及[prototype profile](../../specs/auto-graphics/prototype-profile.md)只证明其绑定的原生实验。原生830同设备UI→3D→UI/3D→UI→2D证据不能抵扣DOM、浏览器或移动端验收。

[874](../../plans/874-autographics-public-character3d.md)→[875](../../plans/875-autographics-runtime-cameras.md)的任务、AC、输入键位、CPU/credits/源资源界及进入门保持；不追加WASM编译、触摸、DOM、后台恢复或device-loss任务，不把本设计当作它们的新复审门。后继能力Plan引用本设计，在自己的范围内冻结适配接缝；若发现旧实现需改变，另立Plan，不改写旧交付事实。

**M2交付仍是桌面两款游戏＋原生Gallery，M2-01..09不增跨端运行前置。** 下文XP调研是独立候选路线，不加入当前11–15份M2剩余估计。生产AutoUI/HIR/ACE统一与归仓仍等ACE完成后再讨论；跨端渲染/宿主机制可先独立调查，不借调研提前集成ACE。

## 2. 公共语义、执行环境与呈现分层

共同目标是同一声明语义/项目状态可选不同平台呈现。统一语义树不要求所有节点进入同一物理渲染树，也不要求所有UI持有GPU纹理。

| 逻辑层 | 共享合同/数据 | 平台差异与禁区 |
|---|---|---|
| 声明/场景 | 对象与资源身份、布局/变换、组件/图形语义、版本与能力要求 | 目标HIR方言与lowering另按ACE路线；不把Iced Element、DOM Node或winit事件变成公共场景类型 |
| 运行/游戏 | 动作、逻辑tick、角色/查询/事件、场景状态、候选/接受身份 | Auto执行adapter对接实际可用环境；Vue响应式状态是呈现映射，不成为第二份玩法真相/调度器 |
| 绘制/组合 | 有界绘制/资源描述、裁剪/颜色/alpha、挂载与命中关系、失败分类 | GPU对象在renderer侧；DOM、离屏纹理、原生绘制目标是不同呈现能力，不用一个texture接口强行覆盖 |
| UI呈现 | 组件语义、状态/事件、主题意图、逻辑尺寸/可访问性需求 | 原生Iced、浏览器Vue/DOM、移动端UI适配分别验证；字体/原生控件不默认逐像素相同 |
| 平台服务 | 资源取得、时钟/调度、输入/焦点、音频、存储、surface及生命周期合同 | 原生文件、浏览器fetch/cache、移动应用资源包分别实现；不能把本机绝对路径、线程/阻塞调用当公共前提 |

这里是职责边界，不要求现在创建一组新crate/trait或搬动现有代码。公共数据以明确字段/单位/版本与所有权表达，不穿透Rust引用、DOM对象、原生窗口句柄或GPU资源。接口是否序列化/进程内借用由实际adapter决定，不预建跨进程大协议。

JS/WASM桥接须显式规定数值与身份编码：现有持久u64 ID不能无检查转成JS Number；使用精确可往返的编码并保留Run/tick/代次，f32/非有限拒绝、颜色空间、预乘alpha、坐标轴与屏幕方向等语义也要对齐。支持Plan先核原数据约束，不以平台转换静默截断身份或改变碰撞/相机算法。

### 目标平台路线

| 目标 | 图形路线 | UI/宿主路线 | 尚未证实的部分 |
|---|---|---|---|
| 桌面原生 | 现有wgpu，复用已交付资源/合成基础 | 当前winit/Iced；其他桌面系统另验 | Windows事实不自动推广Linux/macOS |
| 浏览器WebGPU | 优先调查Rust图形核心→WASM/wgpu→WebGPU Canvas | Vue/DOM负责普通UI；浏览器宿主连接布局/事件/生命周期 | 实际依赖可否编WASM、包体/启动、JS桥接、执行环境与浏览器能力 |
| 移动原生 | 调查wgpu对应原生后端及surface生命周期 | 触摸/软键盘/应用挂起；UI后端待定 | 不能直接承诺Iced或完整Vue适配、所有GPU/设备支持 |
| 移动浏览器/WebView | 按WebGPU实际能力单独判定 | 移动DOM/Canvas/软键盘/页面生命周期 | 不以桌面浏览器、系统版本或WebView名称推断可用性 |

[wgpu官方说明](https://wgpu.rs/)列出原生后端和通过WASM进入WebGPU的路径，这证明路线值得调查，不能证明AG当前整个依赖闭包或旧SDK已可在浏览器运行。直接JS调用WebGPU最多作为有限对照候选，不先复制第二套场景/角色/资源/游戏实现。WebGL降级不默认承诺；如要支持需明确能力与验收。

渲染器可用、Auto程序可执行、完整UI/2D/3D可交互是三个独立结论。Auto浏览器执行候选（现有环境移植、未来ACE/WASM或经批准的其他入口）在对应路线冻结前保持未决，不依赖无限线程、同步文件访问或未证实的执行预算。

## 3. DOM与Canvas混合嵌套的支持层次

普通Vue界面主要是DOM布局/输入；WebGPU绘制面是Canvas或离屏目标。原生“同一Device/Queue”是GPU纹理合成域内的归属要求，不能要求DOM控件提供GPUDevice，也不能把多个域的资源句柄直接互传。

常规[copyExternalImageToTexture](https://developer.mozilla.org/en-US/docs/Web/API/GPUQueue/copyExternalImageToTexture)来源包括图片、视频和Canvas等，不提供通用任意DOM子树来源。不得假设把Vue组件挂到div上，就能直接作为可采样纹理进入3D。此处限制针对通用可移植路径，不提前依赖特定浏览器实验API。

| 组合形式 | 推荐调查方式 | 能表达什么 | 必须明确的限制/成本 |
|---|---|---|---|
| Vue页面→Canvas 2D/3D | DOM组件布局内挂Canvas，共享运行状态，通过明确bridge传数据 | 普通UI容器包含图形区域 | DOM布局/CSS尺寸与像素分辨率分别处理，resize/scroll/DPI/clip影响输入映射 |
| Canvas上方DOM HUD | DOM overlay、明确z-order/命中区域、单一焦点仲裁 | 普通覆盖面板/表单/文字 | DOM不进入场景depth buffer；不能默认被3D物体遮挡或与半透明物体正确交错 |
| 3D平面上的DOM外观 | 有限CSS投影覆盖作为候选对照 | 某些投影随动UI，保留真实DOM输入的可能性 | 外观随动不等于GPU纹理合成；遮挡/裁剪/透视点击/scroll/IME/可访问性须逐项测，未支持情况需标识/拒绝 |
| 3D→可采样UI→2D/3D | 调查可离屏绘制的UI adapter，在共享语义/布局下生成Canvas/GPU纹理 | 真正纹理采样、深度/嵌套组合的候选 | DOM排版/CSS/控件不能自动复用；须定义支持组件子集、字体/主题、文本编辑与IME桥接 |
| DOM截图或快照 | 只作为有限静态内容/实验候选 | 非实时、可声明限制的预览纹理 | 动态控件、跨源资源、失效频率和交互语义不能假装完整；不作为默认实时UI后端 |
| 多层DOM/Canvas混合 | 语义树lower到各呈现域，挂载/焦点/输入路径桥接 | 在受支持子集内做嵌套 | 能力协商、无环/预算、身份/迟到事件、命中/遮挡/可访问性共同成立才认领；不默认任意无限嵌套 |

因此浏览器端至少需区分DOM承载能力、可采样内容能力、3D深度参与能力、文本编辑/IME能力。未来挂载声明按所需能力选择adapter；不支持时明确诊断，不静默换成截图、禁用交互或把overlay称为深度正确合成。

共享语义/主题意图不等于各端文本排版逐像素一致。尺寸/布局/焦点/状态/可访问性目标和视觉容差须按支持组件冻结；字体fallback、抗锯齿、CSS与原生控件差异记录在平台profile，不据此更改当前Design47/AutoUI合同。

DOM更新和GPU提交属于不同呈现域，不能照搬原生同设备“整帧一次发布”并承诺二者物理原子呈现。未来bridge需定义共同逻辑commit/layout revision、提交顺序、暂未呈现状态与输入所用的已呈现布局；整帧准入保证在各域分别落实，跨域可观察一致性由XP-2/3验证。未证明同帧呈现时，不以日志中的同tick冒称没有UI/画面错位。

## 4. 平台服务与资源生命周期

以下是未来adapter的设计要求；实现/验证由后继Plan认领，不追加入874/875。

| 边界 | 未来合同要点 | 需要区分的错误实现 |
|---|---|---|
| 输入 | 语义动作/指针意图与键鼠、touch、pen、虚拟摇杆映射分开；坐标明确CSS/logical/physical/world，转换一次；pointer/挂载身份与capture/cancel | 一个事件同时触发DOM和游戏；DPI重复换算；迟到touch修改新场景 |
| 调度 | 逻辑dt与原生event loop/浏览器动画帧分离，固定步可测试；后台暂停/恢复丢弃或按明确规则处理墙钟债务 | DOM组件与引擎各推进一次tick；恢复后追赶巨大积压 |
| 资源取得 | 资源ID＋hash/长度/版本；异步取得/解码/准入/发布分开；跨源/访问限制明确分类；旧Run/请求代次响应不进入新状态 | 晚到HTTP响应写新项目；缺资源默用占位图认领成功 |
| 存储/下载 | 平台provider实现本地项目、浏览器持久化/导入导出、移动应用存储；访问/配额/取消显式分类 | 浏览器依赖本地路径/同步文件；关闭对话框仍保存半份工程 |
| GPU/surface | CPU资源描述与当前device对象分离；device generation、surface重建、取消与最后owner有定义 | 新device复用旧纹理句柄；过期异步prepare仍发布；无效surface继续绘制 |
| 音频 | 资源/一次事件与具体音频设备分离；允许平台出现需用户激活/挂起等状态，明确pending/失败与重开清理 | 重试导致重复播放；恢复后旧局声音继续；无声音仍宣称通过 |
| 移动布局 | touch target、safe area、方向/尺寸变化、软键盘/IME占位及焦点；移动原生与移动Web分别profile | 只缩小桌面截图；软键盘遮住输入框；旋屏后点击位置错误 |

浏览器[device lost](https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/lost)可能使当前device及其资源不可再用；未来恢复策略必须用可重建描述与新代次重新准备，不能继续绑定旧GPU对象。若GPU/surface已不可用，所谓last-good是保留最后有效**逻辑状态/资源身份**，不承诺损失的帧缓冲继续可见；UI可以报告暂停/恢复/失败，新的完整帧准备成功后才重新呈现。此说明不追改当前原生故障保帧合同。

平台功能是required还是optional、缺失时拒绝还是受检降级，须由样例/能力profile定义。跨端不等于所有端相同性能/纹理尺寸/内存；不得把桌面原型的128节点/32credits/250ms预算重命名为移动实时性保证，也不为移植静默提高旧预算。

## 5. Gallery增加平台维度

保留C01..17能力轴与首G01..G12，增加独立平台/profile记录，而非把一项P状态扩散到所有端。每个示例子入口记录：平台/宿主/执行环境/renderer版本、设备能力要求、输入映射、资源来源、UI呈现方式与已测限制；状态至少区分未测、已验证、受限和不支持。

- 当前只将已交付Windows事实用于能力准备；Gallery示例本身仍待其实施Plan交付，浏览器/移动列均不能预填Ready。
- 网页catalog/截图/视频与WebGPU真实运行分开标识；研究探针只进研究展区，不计入12个公共示例的跨端完成数。
- 先复用G01/G02几何/模型研究图形可移植性，再研究Vue桥接/嵌套；角色/相机跨端示例等对应公共入口与执行adapter都具备后再接。
- 跨driver像素验证使用相同场景与独立几何/颜色区域/深度/命中oracle，明确文字/抗锯齿容差；不要求全图hash一致，也不能以任意“看起来相近”免除内容检查。

## 6. 可并行调研步骤与退出条件

XP是研究工作包名，**XP-0已落实正式878，其余仍无编号，均非立即实施授权**。各Plan先固定源/依赖/环境/支持集/资源规模/命令，再建唯一树。go/limited/requires_replan均须按各合同解释；878输出audit_complete与next_probe_readiness，不把负面/受限审计冒称跨端通过。

| 步骤/先后 | 工作与限定入口 | 产出/可观察判据 | 并行与阻塞 |
|---|---|---|---|
| XP-0 先核边界/闭包 | 只读已交付AG/SDK/依赖、native专属调用、着色器/格式/线程/文件/同步等待；列平台服务和执行候选 | 文件/依赖分类、需要抽离的触面、wasm/宿主支持假设清单；每项注明事实或未测 | 可与874/875、GY-A及资源盘点并行，只读固定pin，不改共享代码 |
| XP-1 最小图形双端 | XP-0后，用同一份小型G01/G02场景输入/资产，比较原生与浏览器WebGPU；优先WASM/wgpu，最多加一个直接JS对照候选 | 两端真实最终内容/几何/资源身份、resize/DPI/坏资源；依赖闭包/启动包体/CPU/上传/内存可测范围；输出可复用部分与阻塞 | 独立研究宿主，不引入游戏Rust分支；可与引擎/图库内容线并行。只有原始渲染成功时仍标render-only，不认领Auto执行 |
| XP-2 Vue/Canvas宿主 | 有可用Canvas后，普通Vue输入框/按钮＋一个图形面；明确单一逻辑状态、布局/焦点和输入bridge | 真实键鼠/IME、滚动/resize/DPI、控件与图形互斥输入、暂停/关闭/迟到消息；冻结重复事件/漏DPI等负控 | 可与XP-1后段/资源准备并行，先用已明确的最小bridge；不接生产AutoUI或完整组件迁移 |
| XP-3 3D挂UI关键决策 | XP-1/2后，比较有限DOM投影overlay与可采样UI adapter，两候选最多；先文本/按钮/输入小子集 | 明确深度遮挡/透明/clip、3D命中/焦点/IME/辅助访问、更新成本/释放；overlay不会因外观通过而认领纹理深度合成；每项结论/限制可复查 | DOM限制/布局方案可先分析；完整运行probe等所需adapter，不扩874/875，不把研究小组件当完整AutoUI |
| XP-4 移动两路线 | XP-0后可盘点原生和移动Web；实际probe在XP-1成熟后复用输入/资源，分别选定设备/系统/宿主 | touch/多指cancel、软键盘/旋转/safe area、前后台/内存压力/surface或device重建、低能力拒绝/降级、资源/能耗可测范围 | 设备/依赖调查可并行；原生与Web分别记录，不能由桌面或一个WebView推断另一端 |
| XP-5 决策收敛 | 汇总已有XP结论与实际Auto执行环境可用性 | 支持矩阵、宿主/资源/执行接缝建议、需改代码清单、成本与后继实施Plan建议；未测项保留未测 | 正式适配/生产融合另立Plan；ACE/HIR依赖仍按原门，不因研究pass自动启动 |

代表性判别：XP-1少画一个mesh/不更新viewport应被内容门发现；XP-2 DOM文字焦点时游戏不得响应同一键；XP-3前景几何遮挡是纹理合成与DOM覆盖不同结果的必要观察；XP-4恢复后旧Run事件/旧device资源不被新局消费。真实OS/浏览器交互、资源owner、生命周期和内容结果分开记录；平台不具备所需设备/权限时记blocked，不用截图或模拟事件替代必需真机/真实消费者。

研究规模与资源上限在对应Plan预先冻结，沿用现有小场景优先，不以全Gallery/完整游戏移植开局。原始截图/日志/探针/报告本地保留，入库仅Markdown摘要/契约（命令、退出码、关键数字、本地路径/hash），遵守attachments政策。研究源码若需保存是对应研究Plan的明确代码交付路径，不把review探针塞入附件跟踪。

## 7. 规划、所有权与集成门

近期引擎仍874→875，Gallery首批正式877；XP-0正式878，可独立并行审计固定源/依赖，不依赖三份计划完成。XP-1..5据其结果再细化，无新编号或完整适配数量/月份承诺。M2剩余11–15份仅计桌面/Gallery（已含877），878及各端产品化另计，不阻挡原M2标签。auto-ui已拆分后的纯归仓触面由878只读评估，时机/保持门见[归仓建议](repository-move-and-integration-gates.md)，不执行迁移。

并行条件是写面/产物独立：研究使用冻结副本/明确公共输出，自己的Plan/工作树和构建目录，不动在途874/875或共享锁/assembler/Registry/Play；遇到抽象提取/公共格式变更先单列并串行合入，消费者再同步。原生GPU/重型门按机器单实例安排，不为并行让多探针争抢设备/焦点。review独立上下文顺序用同一权威树，清理先guard，无链接。

未来适配Plan必须分别证明图形、执行、UI/输入、资源/存储、生命周期/能力和实际示例；不得用一个renderer截图替代全栈移植。没有冻结执行环境时可以交付render-only调研；没有可采样UI后端时DOM宿主只认领明确层次的嵌套。选择生产Vue/AutoUI/ACE接线、切默认或旧路径退役需要消费相应成熟路线和独立合同。

## 8. 首版调查依据与状态（历史；后续XP-0正式计划为878）

已读取：Design45、AG integration-contract/project/prototype-profile及M2/Gallery/运行设计。源码锚点autographics-dev `8b12632d7b5b1b0df72fbb4eef84297a054de31d`；本轮不宣称完成依赖闭包或WASM可编译性审计，XP-0承担它。外部资料为wgpu官网、WebGPU API/外部图像复制与device-lost文档，2026-10-28核对的能力边界；具体浏览器/设备支持表由未来Plan实测冻结，不假定全浏览器覆盖。

受保护874 SHA-256=`8e8d2b6093837e368a2cbfbc0e1e40be75e218b2bdb63367f19055d2f4a0dfad`；875=`4b88df0d35b6c5c7e9838a33e84ba9c08c5e7fb964ed70afffed36030d50534c`。取号器入场为876。本轮仅设计/战略/导航，未运行构建/浏览器探针/真机、未实现跨端适配、未创建新工作树或远端仓。

实际文档预检exit 0：XP-0..5共6个有序工作包，Gallery仍17项能力/12项示例，M2仍01..09；6份相关AG设计/策略的111个本地文件链接可解析，Markdown围栏/空白/冲突检查通过。874/875 SHA与入场逐字一致，.next-id仍876，新增专题在00-intro唯一登记；`git diff --check`通过，00-intro仅有既存CRLF规范化提示。上述是文档检查，不是WASM/DOM/移动/GPU功能验证；无新增适配或运行证据。
