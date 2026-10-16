# PLAN-798 r1 起草预检

2026-10-16；stage:new；outcome:pass（执行契约草案可审阅，不是实现/独立验收通过）；next:work，需用户实施指令。drafting 0/10；不运行cargo/docs_gen、不创建实施树/代理。

## 来源、取号与授权

- 读取auto-plan-new及verification-contract、仓库AGENTS、overview及相关HIR/产物/cache/TestCase Spec，然后核实ac-core/783当前接口和796/797/788在途合同。Design43及相关战略是已确认目标，尚无新MIR/AV实现，TestCase v2 Spec缺失如实记录。
- 起始调查HEAD为`3e5aa91f65eca89dab97d652f3782c72c5435526`；来源采集/预检时主检出已由并行合入推进到`f59de89d0d5480c52faaac0f74ba8a9e30b3c5cb`，未reset或覆盖他方改动，逐文件工作区SHA-256见planning-inputs.tsv。796的实施簿记可能继续变化；指纹记录快照而非锁住其进展。
- 用户只授权起草；明确不实现完整AJ/发布器。实施前五项可独立，正式D-796/D-788绑定才进入后段。797不是native-free执行子集前置。
- `.allocation.lock`排他文件句柄+命名mutex覆盖active/archive扫描、新脚本取号和唯一性检查；最大797、next798→PLAN-798，next799。受限环境Git Bash首次启动失败，提升后成功运行同一脚本；未手工覆盖编号。
- 主检出已有他方WIP；只提交798/.next-id/附件及本会话已获授权设计整合，不提交796或其他计划改动，不联系其他会话。

## 一致性与可执行性检查

1. 0–10编号章节、frontmatter r1、10个唯一任务及work/review/merge owner；work完成8/10才execution_done，review/merge保持未来项。AC-01–10、V-01–13、SD-01–07有任务和入口映射。
2. 选最小typed-slot/CFG非SSA模型，ABC独立wire/验证，AV显式帧栈真实执行。首例T-01有两函数vertical/真实cache/新进程恢复/AutoDown源和trace，不能只交格式或复用旧AVM。opcode等细节只在已约定范围内冻结，不伪造当前入口。
3. 首批仅core/scalar i32/bool及已批准trace测试intrinsic；资源、Meta/FFI/AJ/发布接缝标future/unsupported，不混入text/record执行。native参考从旧HIR直达Cranelift，独立expected能抓住两后端同错。
4. MIR/ABC私有信任门、恢复重验、全owner/type/初始化/控制流、好frame坏内层及实际消费反控有V。普通分支trap不提前报编译错误，未知能力在执行前拒绝。
5. 783 A08 Future不是实现；新producer用合法公共store carrier+内层envelope。T-01有可行性停止条件，不改底层、不拿HIR/Object tag冒充新数据、不造第二cache。命中禁调producer仍av_execute>0/重新比较，不与未来CTE结果缓存混淆。
6. source/HIR旧递归前门128预算、连续表达式/交错结构和4096深前拒；新MIR/ABC4096块CFG、初始化worklist、guest帧/fuel各自边界分别验。前门无法覆盖所有递归路径则T-01不通过，不把线程栈加大或进程崩溃当修复。资源限制是实验运行策略，不改旧独立CLI或永久语言语法。
7. 32MiB受管/OS增量观察、≥32MiB冷产物、512churn/lease/GC及坏块/冲突/取消退出有入口与预期；工具/native进程单报，不把整个RSS等同受管数据。
8. 256KiB/100有意义函数、六场景各30样本；body和签名错两路径含真实adapter/IPC/比较的P95<1000ms。冷构建/native对拍另报，不删失败/缩负载/提高阈值，不把阶段P95相加。
9. T-01的.ad只是待绑定源/oracle，D-796/D-788前不得声称正式runner通过。T-07真实probe/run recipe、坏actual/未知字段/旧协议兼容；接口不够需显式revision，由公共owner处理，不另造runner。
10. 796拥有语义ref/公共IPC，797拥有物理ABI，本计划不修改它们的执行合同。临时语义ref在T-06退出；跨层代际/原生可重定位是合同/准入，非热重载/JIT/发布能力宣称。
11. 唯一预定实施树/分支已列，同树独立复审/冻结证据；work只定向检查，merge-owned整档按实际diff，tf主检出到期单实例。Spec仅merge沉淀，新增覆盖不回溯改783旧producer状态。

## 本轮验证范围

文档章节、编号唯一性、任务/AC/V/SD引用、相对链接及来源文件存在、frontmatter计数、diff空白已检查。所有拟新代码/脚本/测试命令留未来实现；本轮没有执行AV/native对拍、资源或亚秒基准，没有实现或发布通过证据。工作前重新验证依赖实际合入revision/hash。
