# PLAN-839:r1 独立 R3 受影响复审 — PASS

**F-839-M1-01/P2 CLOSED。受审提交 `bccc77d10700a58e7f5aea8fcdf55ab3440c4b58` 的工具修复通过独立验收，无新增finding。** 可登记受影响T09通过、恢复reviewed9/10；T10剩余完整门/最终metadata/实际落点和收尾仍pending，不据此宣布已交付。

身份：用户已授权的Codex独立审查代理 `/root/review839`，继续同一独立上下文，使用auto-plan-review及仓库规则。未创建代理/树，未修改实现、共享Plan、canonical或ledger，未提交或合入。模型/会话ID未暴露，不编造。此次只核受影响工具与来源边界，未重跑整套driver/AGP/root测试。

唯一树 `D:/autostack/.wt/lang-839/auto-lang` / `plan-839-dev`；修复基面 `e44644e1fa03c126801caecfe84a8b91054ee09c`，整体基面 `37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c`。进入、测试后和落笔前均HEAD bccc77d10/clean，diff-check通过。当前主Plan r1为execution_done8/10，T07/08已由work关闭。§0旧“executing6/10”摘要已向父代理指出，由其在记录本复审时统一纠正；不将该历史句当当前work就绪状态。

[结构收据](receipt.json.txt)记录完整commit/base、SDK/锁/配置/9规范及SD、输入/原始证据hash、复用边界和pending门。原R2报告与receipt保持逐字节不变：SHA256分别为`4b8f4db1858d2742b86c249524c67813b7f977bea14ec26e52822cfc193afa06`、`aa547e0e8683b0d78aec04e81c1c595a1ede87bad7ac8d81ce71f7a384c7f8b4`。

## 闭合机制和独立验证

原失败为真实编译错误：旧工具只替换hierarchy/mod/assets三个固定文件名，新的`#[path = "authoring_probe/play.rs"]`未转绝对路径。生成入口在外部输出目录编译时找不到play.rs，实际GPU负控没有运行，不能将这个非零冒认像素敏感性通过。原始失败`merge-r1/pixel-faults.txt`保留。

当前diff仅两文件/8增4删：`tools/check_hierarchy_pixel_faults.py`动态遍历所有`authoring_probe/...`路径，解析源目录内实际文件并转绝对路径；零匹配/残留路径拒绝。`research/plan839-identity.json`仅刷新该工具一项SHA为`141a06fbc26c580ac290e2346e54df9279bf9be06d45c9434ea12178addb244a`，集合仍278，其他277项及所有Rust/测试/样本/原生入口/fixture/SDK/锁/配置/SD没有变化。

| 独立验证 | 实际结果 |
|---|---|
| 当前入口四模块映射 | assets/hierarchy/mod/**play**全部指向有效源文件，生成入口无残余相对模块引用。 |
| 新增/兄弟路径正控 | 从受审工具AST提取并实际执行同一resolver/rewrite/guard；新增future、nested/extra、含空格文件均接受，不再依赖手工名字清单。 |
| 路径与失败反控 | 缺文件、目录代文件、已存在但越出probe目录的`../outside.rs`、零模块、空未解析路径共5项全部拒绝。见[可重放输入](path_holdouts.py)与[8项结果](path-holdouts.json.txt)。未在实施树创建fixture。 |
| 来源与副本 | producer和`consumer-merge839-final`独立运行verify_source，均SDK680/source278/metadata479/local17/path29通过；逐278输入和680 SDK核实际指纹相同，两锁及配置和R2相同。consumer用自己的target，未拷producer库/二进制。 |
| **consumer实际透明GPU故障** | 在父代理明确分配的独占GUI窗口运行当前工具。复制入口真实rustc成功，stderr有actual GPU fault witness；native正门与aggregate均非零，report passed=false、hierarchy phase9、`move: composed endpoint/origin pixels mismatch`。endpoint=background=`[39,48,56,255]`，预期leaf color=`[243,149,124,255]`，实际透明被原像素oracle识别。 |
| **consumer实际残影GPU故障** | 同样真实编译/运行成功到phase9；endpoint=color、origin也=color≠background，原位置残影被同一像素oracle拒绝。不是只查host状态/命令/hash。 |

两GPU case outer工具退出0，代表按预期验证两种实际故障；内部native/aggregate失败才是命名oracle结果。命令：在`D:/autostack/.wt/lang-839/consumer-merge839-final/experimental/autographics-auto-driver`运行`python tools/check_hierarchy_pixel_faults.py D:/autostack/auto-lang/docs/plans/attachments/839/review-r3/independent/consumer-pixel`。完整[日志](consumer-pixel.log.txt)、[两case结果](consumer-pixel/results.json.txt)、各case的build/provenance/生成入口/native report/stderr持久保存，关键JSON另存`.json.txt`。所用六个rlib实际SHA、源副本SHA与编译命令已复核；build日志为空且exit0，无新增warning。工具在任何build failure先assert退出，随后还要求具体phase/error/GPU witness/像素关系，编译红无法冒充受控红。

GUI结束已及时通知父代理释放窗口；不再扩测。源文件前后hash与clean状态复核无漂移。

## AC、SD与精确复用

| 范围 | 判定与边界 |
|---|---|
| AC01 / T08来源闭包 | pass，当前278 identity及producer/consumer实际闭包通过；工作四来源反控的实际修改/拒绝/恢复输出已核。 |
| AC05/06/08 / T07旧层级实际像素负控及失败传播 | pass，本轮consumer两case实际重跑，闭合F-839-M1-01；不改变运行/输入/恢复产品语义。 |
| AC02/03/04/07及未变AC05/06内容 | R2 pass按精确输入复用。Rust实现/永久测试/Auto样本/入口/fixture/资源/SDK/锁/配置逐身份未变；R2的23独立case、work/consumer/native/owner证据保持原SHA和原执行提交，不声称它们重新在bccc运行。F-839-R1-01继续CLOSED。 |
| AC09 / T09 | 本受影响独立复审pass，无新finding/遗漏/未授权延期/workaround。 |
| SD01..04 | 全部与R2冻结提案及9canonical规范输入hash相同；原schema3、profile6/30s/2s/取消/owner及生产I2/I3边界未改。R2知识delta结论精确复用，不改revision/canonical。 |
| AC10 / T10 | pending，受影响pass后继续剩余完整矩阵和实际收尾。 |

旧完整门以**未变输入的行为证据**复用：已核gates55条原始调用身份/退出及完整driver第二次119项、AGP163项均0failed/ignored；16个official AGP正门报告均pass，另外blank_blit/transparent_blit按原命名反控fail；原双游戏/property/interaction/hierarchy门保留原e446命令与结果。此工具未被这些未变测试和其他原生门读取，Rust/资产/依赖/配置/输入均等价，不为工具修复重复整套测试。原driver首次callback budget exhausted与层级首次非预期像素失败保留为原失败，随后相同源/预算的成功重跑另列，未确定的环境成因不作断言。

**Git metadata不适用上述等价外推。** 旧AGP报告实际嵌入`agp_commit=e44644e1fa03`，其中independent-consumer的collateral.base_commit仍为完整e446提交。这些报告不能证明新bccc或将来delivery SHA的fingerprint正确。merge必须在实际最终SHA重新构建/执行最终Git-sensitive independent-consumer/metadata门，原记录不改签为bccc。其他未完成资产/play/fresh-consumer窗口矩阵仍由merge逐项完成；本报告没有将55条历史调用凑成全门通过。

后续由父代理把主Plan受影响T09登记pass、恢复reviewed9/10并更新摘要/债务，继续merge-owned剩余门、最终metadata、真实autographics-dev落点（源码不入master）、规范/ledger/index沉淀、归档、fresh guard/清理与batch到期收据。实施树和consumer保留；任何后续实现/依赖/配置/输入漂移需受影响复审及新冻结门。
