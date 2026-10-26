# AG M2首批优化共同执行合同

日期：2026-10-26；适用于853/854/855 r1。它是三份Plan引用的共同保持面和门禁规则，不是额外任务/AC清单；各Plan第5–8节拥有唯一任务/验收表。本合同变更属于相关Plan语义修订。

## 基线、授权与保持面

- 用户授权本轮建立M2设计、战略及先期优化计划；没有授权本轮开始代码实施、建仓、推送或提前宣称M2。实施另由用户启动，已确认的同范围决策无需重复询问。
- M1=`ag-v0.6-m1^{commit}`=`11911a4a6d809bfd515f745b77d00bc8f04f6ba9`；固定SDK=`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。见[源码观察](source-observation.json.txt)、[历史门清单](inherited-gate-inventory.json.txt)、[阶段审计快照](m1-audit.snapshot.md.txt)。这些是new阶段输入，不是新测试结果。
- 保持所有旧游戏/作者profile、behavior/1..5、scene4及样本原始源码/资产、managed/API/bridge/combined字节身份和区域标签、接受/拒绝/诊断/数值/预算、32credits/128nodes、100000steps/120resumes/250ms callback/30s init/2s tick/8ms pump、所有权/输入/发布/取消/Stop规则。固定SDK、两Cargo锁、AGP、公共`.at`算法、资源/金样、根workspace不在优化写面。
- 有效输入、拒绝输入与回调故障均须保留；不能删除测试、扩大容差、加入忽略/跳过、调高预算、静默fallback，或把静态/预录结果替代真实Auto。单纯变少行不算性能证据。
- 旧manifest/receipt只描述旧提交，保持不改；当前源码清单必须包含新增/移动文件并检测文件集合，SDK身份单独核对。新清单明确当前版本，verifier/consumer都使用它；不能因新代码不符旧hash而更新历史证据或跳过校验。

## 唯一工作树和合入

每Plan仅一个实现树`D:/autostack/.wt/lang-<id>/auto-lang`、分支`plan-<id>-dev`。853从M1创建；854从853实际交付提交创建；855从854实际交付提交创建。实施前核HEAD/父链/允许diff并记录；前序代码并发变化不自动获准混入。

图形线源码只合回`autographics-dev`。主检出做Plan簿记、设计和合入后的canonical/ledger沉淀。new阶段不改canonical；各Plan的规范增量由最终review核实，merge后才写入。不使用默认master基面的一键建组来假装图形基线；无junction/symlink。review使用同一实现树但独立上下文，不另建同步代码树。退出/清理前执行`bash D:/autostack/wt-guard.sh <tree>`并确认clean；收据保存真正合入、归档和清理状态。

## 验证时序与原分母

work只做必要`cargo check`及命名test二进制/过滤case、工具正反例和定向性能/原生检查；修完全部finding、提交并最终独立review后，merge才跑冻结完整面。review通过须显式列出merge-owned pending，不得先声称交付。

实验是两个独立workspace，参考851的实证命令，不在根仓跑`t/tf/tv/tu/docs_gen`。若实际写面扩大到生产`crates/`或Schema/语法参考，须先修订Plan并按AGENTS触面规则处理，不能沿用实验豁免。当前设计/计划编写无Cargo门。

853 T-02把历史85组最终命令逐组映射为**当前必跑**或**输入完全不变且允许复用**，记录命令、实际工作目录、选中ID、oracle、资源/二进制身份和复用理由。85不是新测试数，原producer/consumer各280是基线分母，不是“只要≥280就绿”的数量门。不能合并丢弃同名例、把子进程portable_child重复计数，或用新fixture的绿替代旧case。

完整合入至少保留：

1. producer `cargo test --locked -- --test-threads=1`全部原二进制/Doc-test及新增测试；fresh own-SDK/空target消费者同样执行，并核选中ID、0failed/0ignored和实际结束。承诺含case的选择组0selected、意外skip或中断不能绿；合法空Doc-test按实际0单独登记，不当作完成某个必需case，也不误判为套件失败。
2. 当前source/SDK/目录集合、package consumer；全部原Auto数值/几何/预算/owner/状态/项目/诊断测试与坏输入。
3. 原AGP组合/嵌套输入、双游戏音画、authoring/gesture/hierarchy/assets/Play/用户行为/项目与查询/sweep/motion的OS/native及计数AND实际内容两道门。AGP23槽可按当前整个AGP、依赖/配置/输入完全不变的证明复用；driver改动影响的路径须新跑。复用决定由review核实，不能用“AGP未改”复用受driver改动影响的门。
4. motion两端14例＋4重开与6负owner＋恢复正例；所有原语义槽和负控仍具名可见。owner工具可能重建runner，内容门必须消费其记录的不可变runner，不能拿别的`target/debug`替换。
5. 新改动对应的定向验证和同条件性能对比。若门失败先归因，修复代码后复审受影响内容，并在新冻结提交上重跑受影响门；不循环跑整档掩盖定位。

原命令工作目录/输出路径是历史，不能原样重用已移除的851树；853应先显式重绑。继承原Windows配置`RUST_MIN_STACK=16777216`、`RUSTFLAGS=-C link-arg=/STACK:33554432`及对应实际编译配置，记录现有SDK146预存warning与driver新增0，不静默修改SDK压警告。性能的debug/release结果分开，不能跨配置对比。

## 证据与资源

在各Plan attachments记录metadata、选中ID、日志、SHA、状态与小型JSON；完整runner、target、SDK消费包、大型captures保留在`D:/autostack/plan-artifacts/plan-<id>/`并写恢复路径与SHA，不塞入Git，也不写入其他Plan目录。owner反控临时源码恢复后核hash；未恢复不能送review。

性能声明必须绑定机器/adapter/OS/工具链/锁/源码/样本/输入/构建配置/二进制，明确冷启动、预热、样本数、P50/P95算法与计时边界。逻辑tick不是实际帧时；yield不是精确指令；逻辑标量不是进程总内存。测量不可用写明原因与覆盖边界，必需的既有oracle不可用仍为gap，不冒充pass。
