# M2/S1视口计划共同合同（856/857 r1）

日期：2026-10-27。只适用于本批编辑视口；858是独立文档/来源调查，不继承此处运行门。各Plan第5–8节是唯一AC/V/T清单；本附件提供共同保持面，不增加另一套任务。

## 起点、写面和授权

用户本轮授权建立856/857/858草案，没有要求本轮实施、创建agent/工作树、建远端仓库或推送。853/854已交付；855实施交接`70f1addf0a12a6af7f3b9bf447428f558a779d26`只作[源码观察](source-observation.json.txt)，该观察不替代最终独立复审与实际合入收据，不能称后继基线已冻结。

856从855实际交付的autographics-dev提交创建唯一树`D:/autostack/.wt/lang-856/auto-lang`、分支`plan-856-dev`；857从856实际交付创建同布局857树/分支。不能用固定master基面的一键建组冒充图形基线。最终图形源码只落autographics-dev；主检出维护Plan/设计与merge后的Specs/ledger，不迁入auto-ui或生产主分支。一个Plan一棵实现树，work→独立review→merge顺序使用；禁止链接，清理前wt-guard必须clean。

冻结SDK仍为`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`，两锁、Auto算法/API/生成身份、scene4、旧profile/样本/资产/金样/CPU/128节点/32credits与资源准入保持。Nav/Grid只改编辑视图；不增加游戏相机、图形语言/HIR/ACE、角色/动态对象、精灵资源或新平台。

允许在driver内增加view state、renderer/input适配与验证。856优先不改AGP；若现有公开接点确实不足，只能采用Plan已列出的最小AGP适配并记录数据流/受影响门，不能改共享固定游戏语义。857允许有限编辑辅助深度绘制接点，不等于允许renderer重写或扩全套材质。新依赖、锁/SDK变化或生产仓写面必须先回new说明范围。

## 视图与文档/运行状态

每Surface的编辑ViewState有独立revision，包含导航/辅助设置；不进入Document/scene4/行为源码、dirty/history或游戏节点。文档stamp、实际呈现view revision/矩阵/viewport、选择/目标、指针捕获共同决定手势有效性。辅助网格不被拾取、不参与碰撞、不调用Auto，也不占128个场景节点。

渲染、拾取、轮廓/组边界、手柄、拖动与probe使用同一已呈现变换；不能只移动DrawCmd或只更改相机。旧帧/旧ViewState的结果不能发布；视图更新与屏幕呈现的先后关系必须真实可检。viewport尺寸以Surface child逻辑尺寸为准，窗口物理像素/DPI只在入口换算一次；保留现有2D800×380、3D800×400支持面，不额外承诺任意多窗口布局。

导航与对象手势互斥。对象候选在途时拒绝导航/设置/聚焦/重置，不排队迟到执行；导航捕获中不开始对象拖动/编辑事务。Esc取消连续导航并恢复起始view，失焦/离窗/resize/DPI/关闭同样结束捕获并丢弃迟到事件；原子wheel操作完成后不因后来失焦撤销。所有版本变化都使旧gesture候选无效。面板滚动/文本焦点/其他Surface不消费本视口快捷键。

Run前保存编辑ViewState；Starting/Running/Paused/Faulted禁用编辑导航、网格/吸附修改和聚焦，以原运行视图呈现，输入仍走原CompositionInput/Play。Stop恢复编辑内容和进入Run前的编辑view。此处没有新增游戏相机API或保状态热重载。New/Open成功重置编辑view，失败保旧；Save/SaveAs不改view；Undo/Redo只改内容，随后按当前view重画。

## 兼容与资源

默认reset视图、Grid off、Snap off复现原显示/交互；所有旧原生/数值/内容oracle仍有效。新模式变化使用明确UI/探针入口，不能重新生成旧golden来隐藏退化。几何全部保留，grid密度过高先整体变粗，不能耗尽4096 draw容量后静默截掉对象、选择轮廓或手柄。3D深度网格不能用置顶2D线冒充遮挡正确。

新增辅助buffer/几何/采样分别计量，导航不重编Auto、不复制场景/模型/VM、不重复上传不可变mesh、不在每帧重建target/pipeline。动态camera/grid合法上传单独报告，不能要求一切上传为零，也不能用总数掩盖原mesh重传。最后owner、关闭/resize/取消反复循环要验证真实释放，保留外部合法pin语义。

## 验证、证据和关闭

work只跑命名定向测试、必要check与新视口OS/GPU正反例；修完所有finding并提交/最终独立review后，merge才在冻结提交执行完整实验面。独立review不是另一棵同步代码树，不由实施自查替代；review pass显式列merge-owned pending。

复用853实际`m2_evidence.py`/当前source verifier/package consumer和855交付的分母表，逐组补新视口面。原producer/consumer原选中ID、双游戏、组合/嵌套输入、authoring/gesture/hierarchy/assets/Play/源码绑定/query/sweep/motion、owner/counter AND实际内容门保留；不能以较大通过总数代替漏项。需要测试的选择组0selected、意外skip、中断/partial不能绿；合法空Doc-test单列。

最终两端独立workspace `cargo test --locked -- --test-threads=1`及原/新native门由merge执行；受影响AGP若变化就重跑相关完整AGP/组合/原生面，AGP23槽只在各自所有输入/依赖/配置及consumer路径未变时逐项证明复用。driver变更影响的门不能仅以AGP未改为由复用。根`cargo t/tf/tv/tu/docs_gen`不适用于本独立实验；扩大到生产crate/Schema/语法参考前先修订范围。

当前代码变化必须有新的plan856/857来源清单，涵盖新增/移动文件及文件集合/构建入口；更新verify_source/package consumer当前选择，不改旧manifest/收据。独立consumer实际运行，不能依赖生产者target、绝对路径、漏导出模块或外部缓存。

保留Windows环境`RUST_MIN_STACK=16777216`、`RUSTFLAGS=-C link-arg=/STACK:33554432`及实际wrapper/工具链。以855最终基线报告同条件导航off/on、默认旧视图和辅助开启后的CPU/工作集/上传/owner；不同画面负载不伪称语义等价，无GPU timestamp不虚报GPU耗时，不把逻辑tick当60Hz保证。原SDK预存warnings与driver新增0分开记录。

JSON/日志/小型断言索引放各Plan attachments；runner/大captures/target/独立消费包放`D:/autostack/plan-artifacts/plan-<id>/`并记录不可变SHA与路径。新probe必须走实际OS→winit→CompositionInput/Host→Auto（对象编辑时）→最终合成readback；纯Host调用只能作单元证据。至少有“错用旧视图/变换”和“漏画辅助/错遮挡”真实变异使相同oracle红，恢复后正常；不能任意崩溃也算命中。
