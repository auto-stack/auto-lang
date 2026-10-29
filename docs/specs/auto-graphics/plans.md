# auto-graphics 相关 Plan 索引

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 829 | AutoGraphics 独立调研与 2D/3D 游戏原型 | archived | [archive/829](../../plans/archive/829-autographics-independent-research-and-game-prototypes.md) | AGP 独立 wgpu workspace：Breakout/Squash 双游戏 bit-exact 原型 + Iced 同设备探针 + 十探针验证矩阵；代码落 `autographics-dev` 集成分支（不进 master），I1–I3 生产集成 pending |
| 830 | AutoGraphics 可组合 Surface/Portal 与嵌套输入原型 | archived | [archive/830](../../plans/archive/830-autographics-surface-portal-composition-and-input.md) | 在 829 单窗 probe 上验证可组合 Surface/View/Portal：UI→3D→UI / 3D→UI→2D 双向嵌套、离屏 iced 采样合成（预乘 alpha/clip/平面相机）、共享内容多挂载单 tick、捕获/唯一焦点/三态取消、真实 caret OS IME、全 adapter 同设备归属、真实上传/owned 内存计量与省略计数负控；代码落 `autographics-dev`，I1–I3/AutoShader 仍 pending |

| 833 | 真实 Auto 源码驱动 AutoGraphics 最小闭环 | archived | [archive/833](../../plans/archive/833-autographics-auto-source-minimal-loop.md) | 纯 SDK 公开 CPU slice 驱动真实 Auto 场景/Breakout，输出准入、共享单 tick、全资源归属与文字真实上传计量；源码在 autographics-dev，I1 首片，Auto Squash/I2/I3/Shader/CAP 仍 pending |

| 834 | 真实 Auto 驱动 Squash 与双游戏 SDK 闭环 | archived | [archive/834](../../plans/archive/834-autographics-auto-squash-and-dual-game-sdk.md) | 同一公开 SDK/Session 驱动真实 Auto Breakout+Squash，原轨迹/音画/输入/寿命/精确计量与正反控通过；I1实验双游戏分母交付，源码仅autographics-dev，I2/I3/Shader/CAP/生产迁移仍pending |

| 835 | 场景创作保存与撤销原型 | archived | [archive/835](../../plans/archive/835-autographics-scene-authoring-save-and-undo.md) | 独立stock Iced+真实Auto profile3的有限2D/3D文档、编辑事务、保存/撤销/重开与失败恢复；源码仅autographics-dev，I2/I3/生产Forge仍pending |

| 836 | 视口选择与平移交互原型 | archived | [archive/836](../../plans/archive/836-autographics-viewport-picking-and-translation.md) | 有限2D几何/3D OBB与世界轴平移，真实Auto临时预览、一次历史、失败恢复；源码autographics-dev，生产I2/I3 pending |

| 837 | 场景层级与组创作原型 | archived | [archive/837](../../plans/archive/837-autographics-scene-hierarchy-and-group-authoring.md) | 有限同维平移/显隐组、真实Auto profile4、v2保存与保持世界位置挂接；源码autographics-dev，生产I2/I3 pending |

| 838 | 静态GLB导入与可携带场景资源 | archived | [archive/838](../../plans/archive/838-autographics-glb-import-and-portable-scene-assets.md) | 严格静态GLB/真实Auto profile5、内容寻址资源/原字节身份、schema3闭包保存/搬迁重开；源码autographics-dev，生产I2/I3 pending |

| 839 | 场景运行模式与Auto最小行为回路 | archived | [archive/839](../../plans/archive/839-autographics-scene-play-mode-and-auto-runtime.md) | 当前accepted场景/资源进入独立真实Auto profile6，固定tick/实际焦点、暂停单步/故障/启动取消及Stop恢复编辑；schema3不变，源码autographics-dev，生产I2/I3 pending |

| 840 | 最小公共行为API与用户Auto脚本 | archived | [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md) | UserV1/profile7、真实Auto API/脚本、双便携示例及SDK扫描修复；旧支持面保留，源码autographics-dev，生产I2/I3 pending |

Provenance: [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md), [affected final review](../../plans/attachments/840/review-r2/review.md), [complete merge receipt](../../plans/attachments/840/merge-r1/receipt.md). Graphics source: autographics-dev. Production I2/I3 remain pending.

| 845 | 行为绑定与可搬迁项目闭环 | archived | [PLAN-845](../../plans/archive/845-autographics-bound-behavior-and-portable-projects.md) | scene4单源码/稳定角色、接受字节历史/闭包存储、真实UI输出搬迁及来源诊断；autographics-dev，生产I2/I3 pending |

| 846 | 空间查询与用户触发交互 | delivered | [PLAN-846](../../plans/archive/846-autographics-spatial-queries-and-trigger-interactions.md) | 显式v2有限AABB/点/对查询、Auto触发双示例；SDK非有限编码纠正，生产I2/I3 pending |

| 848 | Finite segment queries and nearest-hit interaction | delivered | [PLAN-848](../../plans/archive/848-autographics-segment-casts-and-nearest-hit-interactions.md) | Explicit v3/profile9,128-node/32-credit finite Auto slab, bounded result token and OS/GPU/portable examples; I2/I3 pending |

| 849 | Finite swept AABB and script-controlled stop | delivered | [PLAN-849](../../plans/archive/849-autographics-swept-aabb-and-stop-before-impact.md) | Explicit v4/profile10, fixed-volume Minkowski query/typed result, Dash2D/Dash3D/source relocation/owner/content; I2/I3 pending |

| 851 | Bounded Auto move-and-slide | delivered | [PLAN-851](../../plans/archive/851-autographics-bounded-move-and-slide.md) | v5/profile11, two contacts, f32 final admission, Maze2D/Corridor3D, original limits/closure; I2/I3 pending |

| 853 | M2/S0 performance, resource and evidence baseline | delivered | [PLAN-853](../../plans/archive/853-autographics-m2-baselines-and-evidence.md) | Comparable five-workload measurements, current closure/consumer and immutable complete evidence; no new M2 capability or I2/I3 completion |

Provenance: [PLAN-853](../../plans/archive/853-autographics-m2-baselines-and-evidence.md), [independent final R3 review](../../plans/attachments/853/review-r3/review.md), [merge receipt](../../plans/attachments/853/merge-r1/receipt.md). Source: autographics-dev at c1961665d1a90a8705994085fd4344a1d6a9a564; SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. This delivers M2/S0 research baselines; production I2/I3 remain pending.

| 854 | Trusted behavior source assembly and compatibility | delivered | [PLAN-854](../../plans/archive/854-autographics-behavior-source-assembly.md) | Single checked five-version assembler, frozen original byte identities, independent consumer and same-condition resource evidence; no speedup or I2/I3 claim |

Provenance: [PLAN-854](../../plans/archive/854-autographics-behavior-source-assembly.md), [independent final R1 review](../../plans/attachments/854/review-r1/review.md), [merge receipt](../../plans/attachments/854/merge-r1/receipt.md). Source: autographics-dev at 523d27b863dbe306c89af0ea546334225a29f8e9; SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. M2/S0 compatibility/maintenance work; new M2 capabilities and production I2/I3 remain pending.

| [855](../../plans/archive/855-autographics-authoring-responsibility-split.md) | Host private responsibility modules and current CLI/probe/source entries; original ownership and semantics retained | autographics-dev 05fdc81c546aff735eefdb6c39fbc5fb35dc546a | M2/S0 complete; new M2/I2/I3 pending |

| 856 | Finite 2D editor viewport/grid/navigation | delivered | [PLAN-856](../../plans/archive/856-autographics-2d-viewport-grid-and-navigation.md) | Presented-view navigation, bounded grid and optional world snap; 3D/runtime camera and I2/I3 remain pending |

| 857 | AutoGraphics 3D编辑视口导航与深度网格 | delivered | [PLAN-857](../../plans/archive/857-autographics-3d-viewport-navigation-and-grid.md) | 有限View3d环绕/平移/缩放/聚焦/投影切换、真实相机拾取与透视轴拖动/吸附、世界对齐深度网格与计量、受控构建来源/精确锁证明；autographics-dev 81b0974ac，运行时相机与I2/I3仍pending |

| 858 | M2 dual game sample research and public API gaps | delivered | [PLAN-858](../../plans/archive/858-autographics-m2-game-samples-and-api-gaps.md) | Pinned source assets, 17 game requirements and capability/follow-on matrix; research only, no new game or runtime capability |
| 860 | AutoGraphics real-Auto game runtime feasibility | delivered | [PLAN-860](../../plans/archive/860-autographics-game-runtime-feasibility.md) | Frozen G2/G3 probes under unchanged hard limits: candidate A v5 composition and candidate B probe-only support unit both satisfy necessary semantics (go for 861); source autographics-dev 7144526c3, budgets/credits unchanged |

| 861 | AutoGraphics公共动作输入与2D角色控制 | delivered | [PLAN-861](../../plans/archive/861-autographics-public-actions-and-character-control.md) | 公开`autographics.character/1`：显式OS动作输入（单router、边沿语义）、单2D角色冻结静态solid与几何最终支撑（char_move 25cr池32不变）、只读强制与fault 20..26；autographics-dev 0f53f05da，3D角色/运行时相机/I2/I3仍pending |
| 877 | AutoGraphics Gallery目录与基础六项（GY-A） | delivered | [PLAN-877](../../plans/archive/877-autographics-gallery-catalog-and-basic-six.md) | 本地ag-examples仓：六逻辑ID/十子入口全ready、固定引擎锁（8b12632d7/SDK 7b987ca8/runner 498ce814）、公开CLI-only启动、单项包离线A→B+篡改拒绝；66断言验收+28负控；2026-10-28裁定：探针验收暂停、门禁抽样 |
| 878 | AutoGraphics跨端边界与依赖审计（XP-0） | delivered | [PLAN-878](../../plans/archive/878-autographics-platform-boundary-and-dependency-audit.md) | 固定版本只读静态审计：222第一方文件全分类（22共享候选/16需adapter/73宿主/106工具）、10/10离线metadata解析、SDK宿主栈（axum/tokio/reqwest/notify/libloading）全target无门控为最大未测前置、readiness=limited（R-01..04）、纯迁仓静态可行且四门分离；报告docs/reports/autographics-platform-audit-878/，无平台能力宣称 |
| 879 | AutoGraphics 纯源码迁入 auto-ui 与单一权威切换 | archived | [archive/879](../../plans/archive/879-autographics-source-move-to-auto-ui.md) | AG 两 workspace（702 文件）逐字节迁 auto-ui `experimental/`（695 blob 恒等+7声明修改）；旧 SDK（pin 7b987ca8 不可达）版本保全为 `sdk/frozen/` 681 文件快照+`--from-snapshot` 冷准备；新身份 `plan879-identity.json` 含旧 blob→新路径映射；单一权威切 auto-ui，autographics-dev 保历史+指引；默认 UI 五包零改动 | 
| 880 | AutoGraphics Gallery公共3D角色与两维运行相机（GY-B） | delivered | [PLAN-880](../../plans/archive/880-autographics-gallery-character-and-cameras.md) | ag-examples 8逻辑ID/14子入口：G07两布局3D角色（speed4→5对照/归一/碰撞/支撑）+G08两维运行相机（夹界/letterbox/orbit/zoom/共发布/Stop复原）；按项锁plan875.json（AG 1cc68aba0/runner 6c461c35/manifest 6f100c88）、861旧pin字节不动；checks/gyb无窗口适配32/32 oracle、8/8包离线A→B+源漂移负控；OS自动化零调用 |
| 002 (ag-examples) | AutoGraphics Gallery G09精灵与帧动画 | delivered | [Gallery G09 Spec](gallery-sprites-and-frame-animation.md) | 10逻辑项/18入口；atlas透明/帧/翻向 + 真实Auto角色五姿态，scene/5/PNG/CC0；固定PLAN-004 consumer身份，18 oracle与双入口离线A→B通过；全Gallery/AG仍pending |
| 005 (auto-ui) | Finite textured models and UV closures | delivered | [auto-ui PLAN-005](../../../../auto-ui/docs/plans/archived/005-autographics-textured-models-and-uv.md) | Original static Kenney GLBs, complete PNG closure/scene6, real UV/sampling/depth/double-sided shadows, transactional owners, offline SDK consumer and retained006 association; full games/animation/AG milestone remain pending |

| 003 (ag-examples) | Gallery G10a textured models and UV | delivered | [G10a Spec](gallery-textured-models.md) | Fixed005 consumer; original GLB/PNG closures, public v1/actual overlap/hidden states, nine realGPU oracles and ZIP A→B; inventory11/19, G10b/G12/full AG pending |

| 006 (auto-ui) | Finite node animation and visual facing | delivered | [auto-ui PLAN-006](../../../../auto-ui/docs/plans/archived/006-autographics-node-animation-and-visual-facing.md) | Original four Kenney clips, LINEAR/STEP/slerp, Auto input/move/event/visual32-credit composition, independent color/shadow/facing/physics and owner oracles; independent R2 pass a85980c, equivalent code delivery c681663, SDK7b987ca8/source e2994efa; full games/G10b/full AG pending |

| 011 (auto-ui) | G2 full-loop budget feasibility | delivered | [auto-ui PLAN-011](../../../../auto-ui/docs/plans/archived/011-autographics-g2-full-loop-budget-feasibility.md) | Two fixed004+008 candidates fail original callback1 at33/32; source-derived phase-aware accounts and isolated GPU/intent/lifecycle controls; exact steps/resumes approved unmeasured; full M2 remains unproved |
