# PLAN-817/818并行与共享合同

日期2026-10-19。两计划r1的共同附件，不修改797/798/814–816执行合同。仅授权本批起草，未创建树/代理或实施。

2026-10-20 r2排期修订（优先于下方r1图）：用户指定798→828优先，817/818保留T-01准备，实际共享核心实施/集成在D828后；原D798门不减。消费828只读函数查询/MIR/受检代码会话接缝，各增量兼容work项和正式反控须通过，不能另走整模块前端/重装载。D828须最终review/冻结门禁/合入/Spec与收据齐全；两计划互不反向依赖，798/828不等本批。见[统一合同](../../reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)及两Plan r2。

## 1. 依赖图（r1历史，现行在798与817/818之间增加828）

```text
796/788/783已交付 ──────────────┐
798实际review+门禁+合入+Spec ──┼→ 817 T-02..08（AV→CTE→冻结/consumer）
                              └→ 818 T-02..08（MIR→新native）
797公开原生ABI ──未来FFI接入，本批不调用
814–816拆分 ───公共测试注册窗口/路径owner交接，非全部纯实验代码的前置
```

817/818各自T-01可以先只读调查/固定Schema与独立golden。实际依赖798的编译/运行/测量必须等D-798；不从在途树copy/cherry-pick接口、不把mock执行器作为首例、不叠加送审分支开工。两者互不依赖，817的native观察用旧HIR-native，不等待818；818不消费817未交付的CT值生产者。

## 2. 唯一owner与禁止写面

| owner | 独占主写面 | 本批消费与禁止 |
|---|---|---|
| 817 | 新experimental/ac-cte-probe、test/cases/ac-cte、817脚本/报告/CTE recipe/adapter | 只读796身份/查询/模型、798执行服务、783；不改ModelCt/admit_as_model |
| 818 | 新experimental/ac-mir-native、test/cases/ac-mir-native、818脚本/报告/native recipe/adapter | 只读798 MIR/adapter、旧native仅参照；不改旧native.rs/link.rs或798定义 |
| 797 | AutoABI/target布局/C双向互调/正式ABI recipe与元数据 | 本批私有native ABI不冒称797兼容；797不作前置，不抢写其注册窗口 |
| 798 | 公共Typed MIR/ABC/AV、其测试/版本、sema绑定与AV recipe | 最终API/Spec按实际交付重取；本批不修改共享MIR/AV来适应某个后端 |
| 796/783 | 已交付语义事实与查询、受管store/ref/GC | 消费公开API与合法carrier，不造新Universe/TypeStore或第二数据库，不改旧producer Future声明 |
| 814 | SDK/UI编译与runtime包组、根manifest、旧VM/默认装配/消费者迁移 | 本批零旧产品默认接线、零根members/SDK/UI搬包，不将实验栈静态依赖带进产品 |
| 815/816及其前置拆分 | Man工具/资产及工具服务协议、编排/CLI等各自真实路径 | 本批只追加具名测试recipe/独立adapter，当前注册/CLI owner迁移时服从窗口，不重写其服务协议 |

公共写面包括当时实际位置的TestCase recipe支持表、模块声明/父侧dispatch、工具身份/stale与Spec导航。即使在不同worktree也必须单写：797/798尚在改时不触碰；817/818先合入者完成注册后，另一方同步新master再追加自己的条目并复验。若814–816正在迁移同文件，等待其公开边界交接；纯新workspace设计工作仍可继续。

不抽取新的通用executor、复制sema.rs整个生命周期、创建第二Markdown/parser/裁判。沿用已有probe/run的atom-data/query/edit-source/restart形态；recipe body由各自worker严格校验，case不能指定任意工具路径/命令。若公共输入形状不足，具体提出显式合同revision及owner，不让两个新计划各改一个私有协议版本。

## 3. 共同语义与消费信任

- 817首版值级CTE：i32/bool，纯callee执行；Meta只有受控FieldCount/HasField参数查询。consumer回灌用显式探针CtRef模型/HIR绑定，非新增Auto生产语法或完整泛型/类型生成。
- 796模型marking保持model_compute及拒伪vm规则；817真实执行结果有独立Schema/producer。宿主地址/Unknown不成为合法冻结值，模型结果不能因为改字符串就冒领执行。
- 818从CheckedMir直接生成新CLIF/COFF，不MIR→HIR绕旧native；私有内部ABI与公开AutoABI分域。三路径（AV、新native、旧native）都核独立expected，不只比较两者同错。
- 同一实际已合入MIR和语义版本，才能跨两后端比较。来源sidecar、阶段凭证和key都绑定当前输入；持久payload不携trusted。

## 4. cache、性能与内存

两个producer独立namespace/内层Schema/recipe，复用783 store/图引用/root/lease/GC，原生对象按小页落盘以遵守底层staging cap。817命中冻结结果跳AV（miss正控真执行），818命中对象跳codegen但真link/run，每次重新进行consumer/父侧比较。不同计数不混用，不缓存PASS。

首个可执行例T-02即正式AutoDown+真实持久化/新worker恢复+trace/峰值；没有registry窗口时T-02不可标完成。798合入前的T-01只是准备，不伪造其性能或fixture执行。

时间硬门：817局部helper/Meta编辑到当前checked consumer结果，818局部body/签名编辑到真实PE结果，均含请求准备、IPC/等待/准入/比较的P95<1000ms；cold/native参考另报，窗口与产品能力明确，不声称这两者单独就证明完整应用热重载亚秒。

拟议硬预算：817受管32MiB/worker384MiB/请求树512MiB；818受管48MiB/worker512MiB/请求树768MiB。真实前端/Cranelift与子孙归因单报但仍计入对应总峰值，瞬态spike反控必须红；超标回规划或修实现，不以减基线/删样本/上调阈值过关。各计划预算是本批有限实验政策，不是用户指定数值或语言永久上限。

性能/峰值/大规模GC测量在本机单实例串行，记录797/798及其他计划负载状态；代码和轻量定向验证可并行，不能用负载竞争下反复筛选绿样本。

规模/GC都需实际不同页/对象、512个不同输入修订、正确重算和输出断言；至少推动store超过高水位、证明真实物理删除和有条件回落，水位在承载冷集的同一个store测，lease保活/失败清理/用户哨兵一起验证。内存压力、全冷与发布/链接时延分别记录。

## 5. 顺序与review/merge

每计划一个组树，无junction/symlink。建议D-798后两者各自T-02先证明whole-path；公共注册短窗口依次接入，不让两个分支同步修改共享协议。实现和最终review在同树顺序、独立上下文完成；work只定向检查，merge在冻结提交按实际触面跑门禁，tf仅主检出到期单实例。

本合同不将797/798的历史blocked日志当成当前门；每次开工用实际合入/归档/Spec/提交祖先关系核验。新实验能力如何产品化/接SDK默认入口、公开ABI和完整AJ/发布器，后续单列，不混入本批交付。
