# PLAN-891规划复核与r2修订（2026-10-30）

用户授权：复核ABI、精确位型结果通道、函数级增量及889/888写面交接，有遗漏直接修订；只做规划、不施工。适用技能auto-plan-new及verification-contract；本文是计划合同静态复核，不是实施完成后的独立auto-plan-review，也不授予N/S/H窗口。

结论：[891](../891-native-wide-scalar-typed-entry.md)由drafting r1修订为**drafting r2、0/10**，保持单独一个计划。宽算术/源码、public AutoABI/FFI、wide拥有集合组合仍排除；原W01–05/独立oracle、P90≤2000ms及10个生命周期任务保留。正文§§5–8是唯一当前执行合同，r1规划预检仅作历史。

## 1. 静态输入与当前相邻计划

- master读取基面b3f43a7c4。889=executing r2/1-of-10，权威树提交69d7b6b26；r2按真实源域前置把T-02改为S命名enum支持修复→N资源全链probe，N ABI/runtime尚未稳定释放。不能沿891 r1记录的889 r1/0-of-10推断可开N。
- 888=executing r2/2-of-10；共享父侧ac_av、worker/helper与测试发现/模块选择/源锁正在实施，产品native语义在其排除面内。是否冲突按真实文件/符号及冻结输入判断，不由目录或append-only判断。
- 只读输入：AGENTS、Spec overview，auto-mir/data-place-profile/native-data-owned-lowering、auto-av/data-layout/resident-function-install相关规则及两计划当前交接合同；native emit/lower/layout/config/cache/fingerprint/symbols/incremental/worker/link_driver；共享probe cache::split_units/typed_fixture、ac-core link、父侧ac_mn与mod dispatcher。未进入他方实施树写入/构建/执行。

## 2. 遗漏与修订绑定

| ID | r1空白或矛盾、静态依据 | r2关闭合同 | 所有者与验收绑定 |
|---|---|---|---|
| R891-01 | r1只说“身份绑定逻辑类型/位宽”，没列fingerprint.rs。实际shape把所有wide归unsupported，const_repr只记discriminant，准入放开后不同值/类型可碰撞；逻辑符号实际只含owner+unit | typed渲染完整类型、原始bits与import签名，等价重映射不变，旧占位键版本隔离；按请求传ABI、新符号域及闭包import/callee ABI核验 | T-03/04/05；AC-01/04；V-11及V-06/07 |
| R891-02 | r1“F32/F64以I32/I64载体”未冻结物理调用、pure record副本和隐藏结果别名责任 | Win64 WindowsFastcall统一生成，前四位置/栈参及helper声明一致；明确低32位规则；pure record入口复制、尾随out指针及结果暂存、RHS快照和padding非语义 | T-02/03/05；AC-01/02；V-02/04 |
| R891-03 | r1 nonce/request-hash描述不够：未明确实际PE/wrapper/helper与请求关联，可能只信worker自洽回显或固定fixture表 | 完整帧schema/长度前置检查，输入/ABI/entry签名/实际产物内容绑定；从原生寄存器/结果槽序列化，constant-return和identity/改body实跑控制；不把hash当任意worker可信凭证 | T-02/04/05；AC-02/03；V-05/12 |
| R891-04 | r1称trap有具名原因，但原native exit70无法区分具体原因；收到成功帧后的异常退出/挂起未有区分性验收 | typed value/trap/resource/error互斥，真实trap分支发有界原因；typed value70=exit0/raw46；无合法失败证据的异常归infra；EOF/进程结束/清理前不提交PASS | T-04/05；AC-02/03/05；V-05/08/12 |
| R891-05 | r1说未改verify/split/emit/persist=0，但现有MnIncrementalSession仍scalar；暖更新和冷恢复重验容易混淆。现有worker/增量每次emit_wrapper，r1“换参wrapper生产0”缺实际任务 | typed居民单体patch、真实producer观测、payload token+generation+COFF身份三方核；暖态未改单元prepare/verify/fp/admit/emit/persist为0，cold准入单列；wrapper有界居民/持久身份；A→B→A历史复用与失败原子保旧 | T-04/05；AC-04/05；V-07/10/14及V-06 |
| R891-06 | r1泛称N/H分窗，没覆盖必要共享split/I/O helper的S子窗、mn-worker的N/H交界及模块清单/源锁变化；888不是整项硬前置，但其WIP也未释放 | §5.1逐符号交接表，889稳定阶段需真实提交/验证/停止并写及接受/回交；必要S/H与N同时取窗；不抢888采样、不默认纳入B/C，新增义务进当时有效模块/消费者并集与源锁 | T-01/02/04/05；AC-06；V-09/13，SD-06 |
| R891-07 | r1预算只说帧/arg记账，record暂存/栈帧/深类型渲染与观测硬门边界不清；“20样本四状态”未指明每状态样本及结束点 | 分配/执行前checked限额，受预算闭包遍历，资源/非法输入/宿主失败分列；不编造物理上限；各状态20样本、nearest-rank、正式入口→PE结果/父判/清理，warmup/工具构建单列 | T-02/04/05；AC-05；V-08/10/14 |

这些是原已承诺精确ABI/结果/增量/资源/交接的补全，不增加宽运算或源码目标。新增稳定补例V-11..14及SD-06（TestCase架构增量），同步原V-02/04/05/07/10、AC-01..06/09和T-01..05，保留原ID与未勾选状态。具体tag/帧编码、helper位置和可行性仍由T-02的有界真链probe关闭；当前未假定probe已通过。

## 3. r1历史要点与当前接手入口

r1概述曾写“独立版本的mn-private typed-data ABI”“未改verify/split/emit/persist=0”“COFF与wrapper生产为0”，并把T-03列为emit/lower/layout/ABI工作。这些目标保留，但r2显式补fingerprint/symbols、typed居民更新与wrapper缓存任务。r1相邻状态是e0b29ff7a、889 r1/0-of-10；当前相邻基面以上表及891 §4更新为准，不能把历史快照当已释放收据。

下一步不是直接写codegen：用户后续指定执行时先T-01核当前889/888已提交版本及窗口；T-02须拿到N与必要共享helper子窗后才跑真PE最小链。T-03/04实现闭包，T-05拿H及采样窗做正式AutoDown/模块发现和全部mandatory补例，随后独立final review与merge-owned门。未知窗口不阻止只读核查，也不授予相邻Plan写权。

## 4. 规划验证与限制

- 规划静态检查覆盖11个编号章节、AC-01..09、T-01..10、V-01..14、SD-01..06、10个未勾选任务、规范目标/本地链接、已读代码hash及W原oracle。
- 未运行cargo、native程序、性能采样、故障注入或实现验证；上述V条目均为待执行合同。独立实施review和merge-owned门全部pending。
- 未改889/888计划、实施文件、canonical Spec、ledger或号源，未建树/agent、未联系其他聊天；他方WIP保留。路线/交接仅同步891 r2与已读889 r2快照，不作为窗口接受证明。
