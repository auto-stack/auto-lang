# native第二阶段：拆包判断与PLAN-889边界（2026-10-29）

本轮用户授权：判断native集合/变体与带位宽typed entry应合包还是拆包，并规划下一项；只规划、不施工。读取master=5af8e0cbe3f5a5a4971d5142abb4fb53b0f983b4；D871=87dd2b6ad，归档/清理已完成。无匹配既有native第二阶段计划。中央取号建立[PLAN-889](../889-native-collections-variants-source-closure.md)；typed entry保留未编号后继包，不预占编号。

## 1. 决定：两个独立计划，先集合/变体

| 轴 | 集合/变体源码闭环（889） | typed entry（后继、未立项） |
|---|---|---|
| 核心缺口 | mn拒profile4、List/Variant/Place槽及集合指令；缺native容器布局/内存/释放和消费者接线 | 宽标量槽/常量/签名明确拒绝；wrapper仅零参i32-family结果 |
| 原始义务 | L01–L07、V01–V07，共14个native源码格；864已有AV原源与oracle | W01–W05，位型精确，不是源码算术/数值完整 |
| 主要数据流 | source collections→共享MIR→单位COFF→PE，维持既有i32入口/退出结果 | typed输入→宽位型传递/调用/返回→独立可校验结果通道；不能塞进32位进程退出码 |
| 正确性/风险 | 分配失败原子性、拥有元素/活动载荷、借用/释放、容器与文本总预算、类型布局/恢复 | I64/U64/F32/F64位型、目标调用约定、参数宽度/结果编码、NaN payload/负零及坏通道反控 |
| 为什么可分 | 容器源码只需既有i32/bool/Str/record/variant；不依赖native宽数值 | W测试不需要List/Variant运行时；在D871稳定标量/data链即可定界 |
| 为什么List和Variant合包 | 864合法List元素包含Variant，Variant载荷可含record/Str/无环Variant；布局/转移/清理闭包共享，拆开会暂时拒绝已承诺合法组合 | 不适用 |

两个包都占experimental/ac-mir-native的N，emit/lower/config/cache/worker/SDK H还会重叠，**不能因为拆包就并写N**。先889、再typed entry是当前优先顺序而非D889语义依赖；后继取窗前按实际交付/阶段释放刷新ABI/runtime/cache基面。889不要求先补宽类型，typed包也不得偷偷把宽数值算术/源码泛型加进来。

## 2. 静态证据与新增调查点

- emit.rs::admit_unit只放行av.core-i32/1、av.data/1、av.owned/1，宽类型/常量触达拒绝；cranelift_sig_of仅i32-family、Str pair、Record/View地址及聚合出参。emit_wrapper_coff固定零参i32入口→ExitProcess；driver.rs重复检查入口。
- lower.rs的slot表示直接拒List/Variant/Place；18个集合/变体/Place指令在同一受控拒绝臂。driver.rs编译升级和HIR arm也未选择Collections。单改profile准入不构成支持。
- 现有text runtime只证明文本buffer合同；它不是已实现的List/Variant运行时。native新容器helper/布局属于889显式任务，不允许用调用AV解释器/执行MIR/复制VM的方式冒充native。
- LinkManifest已有text_lib_sha，COFF键有profile/abi/runtime轴；新增容器runtime必须有独立合同/内容身份、库依赖与失效，不能复用旧身份。旧profile产物不无故失效。
- **全链增量缺口**：CollectionsSourceSession::edit实际调用compile_units；后者整源compile_collections_source→lower_checked_collections→split_units，再按fp提交改变单元。当前carry/verified/encoded/admitted计数只证明后段复用，不能认领前端/共享lowering零工作。889必须扩真实单体前端/保留环境/单元消费并在实际生产点计数；这会触及S，与865/863必须取窗。
- collections typed MIR支持更广的Place/List参数接口不等864源码已支持；889承诺864完整合法源码域和它产生的MIR闭包，**不宣称整个822 typed profile机器面已全覆盖**。不需要的Place指令/List参数返回/宽数值等触达受控拒绝，新增完整typed Place能力另立合同。
- 822 Spec的Variant payload概述偏重拥有型，864原V01/V02已含Some/Ok(i32)；889照原源/实际verifier合法集合保留copyable载荷，不能误用概述缩掉它。T-01核差额，merge在本包支持矩阵明确。

## 3. 本包/后继包验收分界

889：原L/V逐字源；L04/L05=42，L06=真实list-index trap（native exit70+可区分原因），V03=3，其余正例按原oracle；List<Str>变体51、for-in原变体、List<Variant>与深无环载荷、拥有clone/失败/清理、fresh/hit/restart/body/interface编辑及P90≤2000ms。W01–05继续gap，不更改NA/expected/旧分母。

typed entry后继应另冻结：输入类型/位宽、有限Const/Copy/Call/Return机器面、内部ABI、独立结果帧及解析边界、W01–05原位型、f32不拓宽、坏/少/多结果帧与截断/调包负控、restart/cache身份。当前不指定未验证的编码方案或C ABI；不要求宽数值算术、native闭包/Dyn/loader或公开AutoABI。没有合适结果通道就先做该包自己的有界探针。

## 4. 与865/863/887/888交接

- 889 T-01只读、T-02 N内私有全链probe与T-03/04 N容器helper/CLIF可准备并行；只读调用已交付S的compile/lower，不改共享定义/注册。若probe必须先改S，停在边界登记，不复制前端绕过。
- 889 T-05真实单函数源码增量属S，默认865→889的S短窗→863；这保持865→863相对顺序，但只是规划，未获释放。不就绪就让863先走，随后释放点再接，不持N等S。
- T-06 SDK/recipe/公共测试注册/binding属H，观察887 reviewed但U887未证终态、888 drafting/等U887；必须核真实owner和释放，不能把“尚未启动”或“append-only”当空窗。889不改887/888合同、runner配置/选择守卫，不借测试批处理改变父侧oracle。
- 现有MN增量会话绑定scalar FunctionIncrementalSession；集合单体接线须复用828保留环境与受检单元发布模型，扩已有会话而非复制语义引擎。没有S窗口就只能交付N阶段证据，不能宣称整Plan可交付。

## 5. 来源指纹

| 文件 | SHA-256 |
|---|---|
| experimental/ac-mir-native/src/emit.rs | 4b2252b06f1ea0c6961b09ada01537525a7f78bc06cbb591db16c07207fcd58b |
| experimental/ac-mir-native/src/lower.rs | 9096045beb9533563730db46d8d316feee50cb85d0bf4638059574efe76d284e |
| experimental/ac-mir-native/src/driver.rs | 3727031c5f4de3d395b32fb2904b0dd23e4c98cee4e6894b31221aa662aa5b4d |
| experimental/ac-mir-native/src/config.rs | 59411657583ed120e3591e05d2dd93a9eaec2edd44d2ff49f2b4b1a85c444459 |
| docs/specs/auto-ac/source-collections-variants.md | a82dafa63e2a2426c4d4c2716a1bca64c4adee9385207b7fc9c6fdeb080cfdcd |
| docs/specs/auto-av/collections-variants.md | ba6261bf69f6e82145e5a0daf599370fe1cf31140b6399ecbf0ed4c53b445126 |

本轮未编译、运行源码、测试或native探针；所有未来验收均未执行。.next-id主检出已有其他计划的取号WIP，本轮仅累进889→890，不夹带提交未提交的888计划/其他簿记。未来开worktree前按仓规提交自己的合同及协调后的号源。
