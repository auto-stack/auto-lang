---
status: accepted-direction
describes: target-state
created_at: 2026-10-20
updated_at: 2026-10-28
implemented_by_plans: []
planned_by: [PLAN-820, PLAN-821, PLAN-822, PLAN-823, PLAN-824, PLAN-825, PLAN-826]
---

> **PLAN-858 口径更新（2026-10-27）**：本文历史 ACE P95<1s 目标在开发初期由用户调整为 P90≤2000ms；原正式样本和正确性、结构、资源约束保持，架构基本成形后以实测另定长期目标。其他 P95 数值作为历史或诊断保留，不再作为 ACE 开发期交付硬门。当前规则见 [TestCase v2 周转口径](../specs/auto-test-case/profile-v2-native.md)。

# Design 44：AV 数据表示、类型布局、Place 与所有权

> **2026-10-28续接基线**：820–824具名typed数据/所有权/集合/闭包/Dyn合同及825/826有限执行/编码优化已有current Spec，不能把下文初稿计划修订号读成当前在途状态。864/865承担集合/闭包源码接线，864必须含List<Str>/List<拥有型type>；CTE/native扩面仍需自己的消费者证据。 统一进度快照与未完工作见[ACE续接基线](strategy/ace-implementation-handoff.md)，后续按current Spec/Plan及merge收据重核；本注不扩大原Plan验收。

## 1. 决策、范围与现状

2026-10-26顺序更新：D828及全链路增量义务保持，正式数据集成为820→821→822→{823，824}，私有同源模块按§12可先并行；原823→824整计划前置取消。各Plan继续保留函数IR/已加载代码身份、真实依赖失效、snapshot/旧值-layout-drop保活及结构+P95双门，不因扩类型退回整模块重算/重载。见[Design43 §6.5](43-ac-av-typed-mir-and-abc.md#65-执行优先级与后继增量兼容2026-10-20用户裁定)、[实际审计](../plans/reports/ace-parallel-dependency-audit-2026-10-26.md)及[统一合同](../plans/reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)。本轮仅修订规划，不认领实施通过。

用户于2026-10-20认可“静态数据类型化、动态数据独立Variant、首版不使用NaN-boxing”的方向，并授权正式设计及拆分后续计划，明确先不写代码。本文衔接[Design43](43-ac-av-typed-mir-and-abc.md) §3–4，描述AC/AAC系列的新版AV；不是旧AutoVM/AAVM的改造或重命名。架构方向已确认，具体API/编码/布局通过各Plan的有界首例冻结，不凭本文宣称已实现。

初稿起草五个功能计划；用户随后认可ABC评审，前次将798修订为r3、820–824修订为r2，并新增825/826性能草案；本次文档复审进一步修订至798 r4、820–824 r3、825/826 r2。仍只改设计/计划，不改源码或canonical Spec，817/818合同不变；实际实施按各Plan已有授权和work→独立review→merge推进，本轮不启动实现。

| 事实源 | 当前事实与采用边界 |
|---|---|
| [798](../plans/archive/798-typed-mir-abc-av-probe.md)与[当前Spec](../specs/auto-av/project.md) | r7已交付有限i32/bool、typed mutable slots、MIR/ABC验证/执行与受管恢复；尚非复合数据引擎。D798收据齐全，完整函数增量归优先828 |
| [HIR阶段合同](../specs/auto-hir/stage-contract.md) | Checked凭证由对应域verify创建；HIR保留语义和来源，不泄漏native布局 |
| [774数据资源合同](../specs/auto-ac/data-resource-contract.md)、[782组合域](../specs/auto-ac/source-text-record.md) | AC已有借/拥有文本、char、嵌套record、根move/clone及有限所有权；不能从旧AVM另取冲突语义 |
| [796](../specs/auto-semantics/project.md) | 名义身份/签名/meta是语义事实；AV执行布局是下游投影，不另建推导权威 |
| [817](../plans/archive/817-av-cte-query-frozen-cache.md)、[818](../plans/archive/818-ac-shared-mir-native-parity.md) | 均已归档且仍限i32/bool；本设计不自动扩CTE/native面，不把新AV类型当其支持类型 |
| [旧VM安全边界](../specs/auto-lang/vm/design/memory-safety-boundary.md) | 旧闭包帧、裸ID与RC影子问题是迁移反例来源；不是新AV语义或可继承安全证明 |

逐文件SHA-256与调查提交见[planning-inputs.json.txt](../plans/attachments/av-data-20261020/planning-inputs.json.txt)。工作启动时重取实际合入的Spec/API，不照在途树复制实现。

## 2. 架构不变量

| ID | 必须维持的规则 |
|---|---|
| I-01 | 一个值的类型由语义类型及已验证执行上下文决定；不按整数范围、负数、堆中是否存在同号对象猜类型 |
| I-02 | 静态类型和显式Dyn不同；Unknown/Error/未完成推导不得降级成Dyn进入Checked |
| I-03 | 名义类型身份、执行布局、native ABI、wire格式分域；等宽不代表可互换，布局相同不代表同类型 |
| I-04 | Place始终指向原存储；求值顺序和求值次数固定，禁止克隆子对象后漏写回 |
| I-05 | 只有已初始化且有访问权限的位置可读；move转移责任，源变未初始化，不以Null/零模拟 |
| I-06 | 每个owned资源恰有一个清理责任；借用无释放权；句柄复制不隐含语言级所有权复制 |
| I-07 | replace先完成右值及准入，再提交；失败保留旧值，提交期间无挂起、回调或可观察半状态 |
| I-08 | 句柄只在正确store/执行域、槽位代际和类型上下文中有效；代际不代替借用/所有权 |
| I-09 | 持久输入无信任；恢复后重验，活对象/帧地址/内部索引/借用token不进入portable工件 |
| I-10 | 正常返回、break/continue、trap、取消、预算耗尽与构造失败都有明确清理责任和结果分类 |
| I-11 | 新profile只授予自身能力；已有core-i32在当前已交付版本路由下的字节/行为/拒绝面和817/818标量合同不被静默扩大；wire迁移仅按826显式版本政策 |
| I-12 | 分配、递归描述、路径/图遍历、验证、清理和冻结均受预算；不把超限、panic或timeout当语言负例 |

安全目标是让各类错误在类型检查、装载验证或受检访问处失败；不声称仅靠静态类型即可消除全部实现错误。

## 3. 三层类型与身份

```text
796语义事实 / Checked HIR
          ↓ 保留名义TypeRef、泛型实参、访问/资源语义
Typed MIR：类型化值、Place、Copy/Move/Borrow/Drop、CFG
          ↓ verify + profile准入
ABC：portable类型/常量/指令/来源；不保存宿主对象布局
          ↓ decode + verify + layout构建
Loaded AV：ExecTypeIdx、FrameLayout、TypeLayout、访问计划
          ↓ 受控访问与生命周期操作
帧 / owned聚合 / 类型化集合 / 独立Dyn / 受控资源表
```

1. **Semantic TypeRef**：带名义owner、声明/实例和语义revision；沿796。FieldRef带所属类型和字段身份，不能仅凭字段名字或位置混用。
2. **ExecTypeIdx / LayoutRef**：当前装载实例内紧凑索引，绑定profile、类型revision、布局版本和代码代际。不是业务ID或跨进程身份。
3. **Frame/Store/ObjectRef**：一份活数据的位置和有效期；与前两层分别验证。

公共MIR保持语义类型；AV布局服务和native target布局服务分别投影同一语义事实。797负责其native/foreign合同，本批只引用、不修改、不把AV布局包装为AutoABI。

TypeIdx/SlotId/FieldIdx即使底层都是u32，也应通过不同newtype与owner检查隔离。持久字段身份重定位为当前布局位置；同名不同owner、同宽不同类型均拒绝。缓存键记录实际读取的类型/字段集合及layout版本；修改字段布局使依赖的访问计划失效。

## 4. 静态数据表示与布局

### 4.1 表示规则

| 语义类别 | AV目标表示 | 生命周期 |
|---|---|---|
| bool、i32/i64/u64、f32/f64、char | 类型化数值存储；保留精确宽度和浮点位型 | 纯标量可Copy；bool/char校验合法域 |
| 纯值record/tuple | 已知字段布局的聚合；首版安全结构存储，之后可内联 | 按资源分类Copy或显式clone/move；非引用语义 |
| owned Str | 受控owned buffer引用 | move/clone/drop；UTF-8、Unicode标量索引沿774 |
| borrowed str / view | owner+范围+借用权限的有界视图 | 不持清理责任；不能保存到首批字段/容器 |
| List<T> | 确定元素布局、len/cap及连续或等价紧凑存储 | 按T的copy/move/drop操作 |
| Option/Result/有限enum | 判别值+当前分支有效载荷 | 仅清理有效且已初始化的分支 |
| 闭包 | 函数身份+typed owned环境 | 环境按字段清理；捕获规则见§8 |
| 显式Dyn/Any | 独立安全Variant；复杂值间接持有 | 保留被擦除T的所有权，见§9 |
| 外部资源 | 独立typed resource handle | 明确拥有者、权限、关闭状态和释放者 |

原始64位槽不等于NaN-box：已验证类型可以在槽描述中，payload不必携逐值标签。反过来，Rust安全enum作为实现载体也不表示Auto变为动态语言。首阶段允许参考实现保留运行时tag检查；不为追求8B槽提前引入unsafe union。

第一数据profile支持宽标量的常量/存取/调用往返，不自动承诺其算术、推导和源码语法；已有i32/bool运算保持798语义。f32/f64数据移动保存位型（含±0、±∞和NaN payload），未来运算产生NaN的规范另定，不做未验证的浮点运算承诺。

### 4.2 描述符与唯一操作源

执行描述符在逻辑上包含type/owner、size/align、字段/分支映射、元素stride、copyability、needs_drop、引用/资源位置及生命周期操作。字段位置、clone/drop/遍历所用布局来自同一已验证描述，不允许opcode/native/调试器各维护表。

AV安全参考存储可以用enum/字段数组执行逻辑布局；offset不授权对Rust内存直接reinterpret。切换紧凑表示必须有独立差分证据，保留相同访问合同。Rust enum、String、Vec、Arc或trait object均不作wire/native ABI。

record遵守声明字段及名义身份；大小、对齐、stride与offset用checked算术。按值递归record无法形成有限布局，验证拒绝；间接递归属于未来显式引用profile，不以无限展开或万能Dyn兜底。空record、零长集合、尾部对齐和嵌套布局有单独正反例。

默认参数view及774/782已定义的owned/借用语义不改变。首批含资源record只允许整根move/clone，字段replace可用；部分字段move明确拒绝，不能无证据宣称完整Rust借用检查。

## 5. Place：位置而非取出的值

逻辑模型（职责示意，不冻结Rust结构或wire编号）：

```text
Place = Base(local | global | environment | object)
        + Projection(field | index | dereference | guarded-variant)
        + FinalType + Access + Owner/Lifetime context
```

Base/projection按支持profile逐项准入，未交付global/deref/variant能力时明确unsupported。与已有HIR `PlaceKind::{Local,FieldPath}`映射，复用“沿根存储定位”语义；不另造与HIR相反的字段拷贝含义。

`users[i].address.city = rhs()`的规则：

1. 按源码/语义规定的从左到右顺序稳定根、下标与路径；每个有副作用的表达式只求值一次。
2. LHS计算后、RHS之前为目标建立写入reservation；RHS可读该根，不能move/drop、改写重叠区域或使目标容器扩容失效。首批按根保守冲突判定，允许的只读RHS以临时拥有值完整物化。
3. RHS完成后校验/激活写权限，检查边界、对象代际、容器结构epoch、类型与初始化状态，执行replace；失败释放准备的新值并保留旧值。
4. reservation不跨挂起、未知宿主回调或未知effect；首批静态拒绝。不得在恢复时重新执行下标表达式，也不得保留Vec内部指针跨扩容。

SSA/offset优化不改变以上求值、重叠和失败规则。初版不做通用两阶段借用优化；reservation是明确的受限规则，必须在MIR和ABC验证中可表示。调用中任何可能冲突的effect按保守侧拒绝。

静态字段访问在装载时形成已验证访问计划。动态下标仍做负数、上界和checked字节范围检查；型别验证不能代替运行期边界检查。

## 6. 统一读写与所有权状态机

| 操作 | 前置 | 后置与清理责任 |
|---|---|---|
| ReadCopy | 已初始化、Copy类型、可读 | 值复制，源保持初始化 |
| BorrowRead | 可读且无冲突mut | 受限只读token，无drop权 |
| BorrowWrite | 排他、可变、无冲突借用 | 受限mut token，无额外拥有份额 |
| MoveOut | owned且可移动、无活借用 | 目标接管责任，源变Uninit |
| Initialize | 目标Uninit、类型匹配 | 成功建立初始化/清理责任 |
| Replace | 目标Init、写权限、RHS已准备 | 提交新值，旧值按规定顺序清理 |
| Drop | Init且持有清理责任 | 清理一次，变Uninit；重复drop为内部错误 |

局部/参数/返回/字段/容器/环境统一经过这些机制。禁止公开未受检的`read_i32(addr)`、可任意铸造的拥有句柄，或让普通get隐式释放。内部快路径只能由私有已验证访问器进入，调试检查可补充但不能成为唯一安全来源。

初始化分析是控制流must分析；分支/循环合流不能把MayInit当Init，move后不能因物理比特仍在而可读。首批以根义务和当前分支状态为主；支持部分初始化的构造器用私有init bitmap/guard回滚，但不将半成品发布成普通值。借用返回追踪真正owner来源：合法参数来源可跨callee退帧，局部/move-in/活mut来源按774拒绝；不能简单以“创建token的callee已返回”误拒合法参数view。

**replace事务**：先准备且保有RHS→验证reservation→安装并提交move/init状态→释放旧值。旧值清理不是先释放再分配；同根重叠的copy/clone先物化，含资源的自move/重叠move前门拒绝，纯Copy自赋值合法。提交到旧值清理之间无用户代码/挂起。

## 7. 帧、对象与失败清理

owned token与可复制的定位handle分开，Rust `Copy`不能用于拥有资源的通用值。将798标量`Value: Copy`扩成复合值时必须重新审计所有slots读取、参数/返回、临时副本和trace，不直接延续derive。

对象表按执行域所有，外部只能经受控handle访问。槽位代际回绕前退役槽位，不静默复用；借用token另带frame/env/对象存活和结构epoch约束。真整数与任何活对象ID同值仍是整数，跨store拒绝。

正常出口以MIR显式清理边/义务执行逆建立序清理；return先将结果责任转出，再清理其他活值。AV trap、取消、fuel/frame/内存耗尽也在宿主进程存活时清理当前执行的owned对象；不能借旧native“退出进程由OS回收”的证据宣称AV已清理。

为保证失败清理可完成，首批析构仅是受控的非挂起、非用户回调、无guest分配的内建清理。正常执行fuel耗尽不取消其清理责任；清理遍历使用显式栈/预留记账，计入峰值，不能借Rust递归爆宿主栈。无界/可抛错用户析构不在首批；外部关闭失败单独报告cleanup/infra并保留原始失败，不重复free或返回成功。进程被外部强杀不承诺执行析构。

owned循环不能构造；首批拒显式共享可变图与shared/weak资源，不用裸RC模拟“全支持”。未来共享类型必须单独设计循环策略、原子性与跨任务边界；本批不引入tracing GC或全局Arc<RwLock<dyn Any>>堆。

## 8. 类型化集合、ADT与闭包

List<T>持有确定类型/布局，T必须可拥有；push/insert/replace/remove/clear/drop统一元素操作。get返回受限view或Copy值，remove返回owned值；不能把所有元素转为Dyn。失败扩容/插入不得丢失原列表或消耗调用方尚未提交的源值；必须证明转移发生的提交点。

Option/Result/有限enum验证分支存在、载荷类型/初始化和match guard；只能读取当前分支，切换分支先准备新载荷再提交、只清理旧活分支。普通Null、缺字段、Uninit、None、false/0不同态。

泛型在本批只做明确元素类型的内建实例化及描述符；完整用户泛型推导/trait/任意递归type函数另案，不把字符串mono_name当类型身份。

闭包环境是typed owned aggregate。逃逸闭包必须move捕获或显式clone进入环境，不保留创建者栈地址；闭包初版为可move/显式深clone的owned callable，按调用签名view/mut调用，不默认为共享引用。nested capture传递真实环境责任；捕获env释放时只清理仍归其所有的字段。

借用捕获首批仅限静态可证不逃逸的同步调用，不能返回/入容器/跨挂起；无法证明则前门拒绝。调度器、async park/resume、跨线程Send/Sync及共享可变捕获不在本批。其未来实现必须使用稳定frame/env身份并禁止临时Place/token跨挂起，不能将模型检查当真实调度器验收。

## 9. 独立Dyn与可移植冻结

有限Variant由ADT表示；开放Any用独立DynValue。支持`pack<T>`、`is<T>`、`borrow_as<T>`、`take_as<T>`，具体名字由实施冻结。类型不匹配为明确结果，失败take不消耗原值；不做bool/int、整数/句柄、缺字段/Null的隐式猜测。不可复制owned值装箱后依旧不可复制，借用不能借pack逃逸。

首版安全enum/结构实现，标量可内联、复杂值间接保存；名义具体类型绑定descriptor，不建立第二套推导器。首批Any只允许已交付的白名单类型；Fn签名、opaque host资源等不因可描述就可执行/冻结。

**NaN-boxing决策**：静态数据不用；Dyn首版也不用。将来单独Plan实测密度、分配、cache和总延迟后才可替换内部codec。若采用，必须定义NaN规范化/escape、全宽整数、句柄代际、平台限制与位型保真边界；不改变外部安全访问API，不能直接复用旧NanoValue。

freeze把活数据转成portable值图：稳定类型ref/声明revision、字段/分支身份、精确标量与bytes、图内编号；恢复重建当前执行域handle并重验。没有进程地址、当前ExecTypeIdx、frame/borrow token、宿主函数指针或trusted位。首批仅无环owned树，可保留显式允许的不可变共享引用身份；遇环明确拒绝并保留原值，不无限递归。若实现共享尚未交付，整个共享图类别拒绝，不伪造去重覆盖。

外部资源、闭包、借用和未知类型首批不可freeze；通用Batom引擎、JSON协议、817结果schema不被顺便替换。沿[Atom/Batom方向](strategy/atom-batom-v2-design.md)使用独立版本化portable映射；已有codec不能容纳的类别用具名新payload/producer，不假标旧scalar结果。图规模、字节、深度及校验预算在分配前检查。

## 10. 调用、宿主边界与代际

内部调用通过逻辑签名+已验证参数映射/FrameLayout，左到右求值后落参数位；返回位置由描述符决定，不以字符串尾Null或硬编码pop计数表示。已有774/782 source语义通过真实HIR adapter接入；新增宽标量/集合语法尚不存在时使用显式typed MIR输入，不能宣称完整source闭环。

新profile具有自己的typed入口/结果准入及宿主接收责任，可测试宽标量与owned返回；旧core-i32零参/i32 entry及817/818调用合同保持不变，不通过放松旧execute入口实现扩面。typed结果在宿主接收后仍有owner，drop/归还必须可观察。

宿主边界用typed invocation及受控resource表，统一验证store/type/权限/生命周期；首批只有具名测试资源shim，不能传任意函数地址。native adapter与797的真实AutoABI另立消费者接线；本批不承诺C ABI、零拷贝借用或不可信native隔离。

代码代际与布局lease绑定；旧帧/对象保留旧descriptor至结束，不能用同名新字段表解释旧对象。首批检测错代并拒绝，不实现在线热重载、活对象迁移或OSR。缓存数据命中仍经过当前profile/type/layout准入，不恢复trusted凭证。

## 11. 预算、诊断与可观测性

各Plan沿798已有输入/验证/fuel/frame/wall上界，首个新值同时纳入**合计活槽+聚合+buffer+环境+Dyn**预算；不能每增加一种对象再领取8MiB。提出的首批数据额为合计8MiB、请求级受管32MiB（hot8/staging16/execution8）；实际profile若无法满足，先明确revision，不减样本或漏计内存。原有更小cap取交集，未知cap先阻断执行准入。

用户没有指定新的性能/内存数字：上述是待实施首例验证的实验政策，不是永久语言上限。首例记录真实allocation attempts/success/free/live/peak、copy/move/borrow/drop事件与owner，观测无副作用；禁止强制回收后再冒称正常出口无泄漏。资源申请失败、invalid ABC、unsupported、语言trap、取消与infra分态。

每个新增profile首例都有真实MIR/ABC持久化→新worker恢复重验→执行、来源/布局版本失效和AutoDown实际父侧oracle；不缓存PASS。沿Design40/43的亚秒周转目标，只对沿798固定标量编辑负载维持P95<1s，不借本设计许诺任意大图/深clone在1s内完成；新增对象负载报告延迟/峰值及规模，不用无观测的性能宣称。

## 12. 实施拆分、依赖与单写面

2026-10-26实际依赖审计：优先功能/源码路线与D828保持，正式集成改为820→821→822→{823，824}；824白名单不消费闭包，不再以D823为前置。823/824的T-10可基于D821先开发私有同源模块，825/826可基于D798/D828并行开发派生表/codec；实际派发、准入、公共类型与注册仍单写，826正式集成仍等D825。具体输入/符号、阶段门与病例责任见[审计合同](../plans/reports/ace-parallel-dependency-audit-2026-10-26.md)，不改822在途合同和任何已交付验收。

| Plan | 独立交付与边界 | 依赖 |
|---|---|---|
| [820 类型化数据与布局](../plans/archive/820-av-typed-data-layout.md) | 新data profile、宽标量存取/纯值record、类型化字段Place与ABC布局准入；不做资源 | D-798 |
| [821 Place与拥有型访问](../plans/archive/821-av-place-owned-access.md) | 沿774/782的Str/char、owned record、reservation、move/borrow/replace/drop与全出口清理 | D-820 |
| [822 类型化集合与Variant](../plans/archive/822-av-typed-collections-variants.md) | List<T>、Option/Result/有限enum、Index/branch Place、失败提交与元素清理 | D-821 |
| [823 owned闭包捕获](../plans/archive/823-av-owned-closure-captures.md) | typed环境、逃逸move/clone、同步非逃逸借用、frame/env有效期；无async/shared | T-10模块D821/D828；正式集成D822/D828 |
| [824 Dyn与冻结边界](../plans/archive/824-av-dynamic-values-freeze.md) | 独立Dyn、受检类型擦除/恢复、portable树冻结及typed宿主准入；Fn/闭包拒绝，不消费823 | 正式集成D822/D828；T-10模块D821/D828 |
| [825 装载表示与执行效率](../plans/archive/825-av-loaded-program-runtime-efficiency.md) | 可选：标量LoadedProgram、O(1)PC、安全帧/参数策略、WASM/V8执行测量 | T-10模块D798/D828；正式集成另需核心窗口 |
| [826 紧凑ABC与体积](../plans/archive/826-av-compact-abc-format-and-size.md) | 可选：新wire、section/body边界、密度与大小/装载测量 | T-10模块D798/D828；正式集成D825/D828及核心窗口 |

D-N表示最终独立review、冻结门禁、合入、Spec/merge收据齐全，不是分支可编译或状态标签。823 T-02仍等D822，825 T-02仍取核心窗，826 T-02仍等D825；各自T-10按已交付基面只做无公共写面的同源模块，不等待其他模块全部review完。各一树，不消费在途实现；公共核心集成按窗口串行，不再把五个整Plan强制排成单链。

共享核心首选扩展已交付的`experimental/ac-av-probe/core`，不复制VM或建第二MIR。819已被其他归档计划占用，本批通过中央脚本领取820–824。817/818/827独立消费稳定API期间不独占整个核心；若公共定义仍有在途写者，则该面不能由数据线同时修改，先等owner交接/实际合入，而非要求817/818全部结束。各Plan T-01/T-02登记基面/Spec/API版本、具体写面、owner/消费者与窗口接受；交接记录含合入提交、兼容证据、限制、释放及下一owner。默认公共核心一个窗口，公共注册追加也串行；持窗不等上游，不要求消费者追逐新profile。已有scalar仅获得编译兼容与同面回归，不扩验收语义。

原生新数据lowering、CTE复合结果消费、生产SDK接线、完整数值运算、共享/weak图、async和实际热重载分别列为**后续未交付能力**，不作为本五计划的隐含交付。824portable新schema不等于817已能消费复合值。

## 13. 验证策略与交付纪律

按I-01..12建立反例，不以测试数或checked box代替证据。包括同宽异名类型、错owner FieldRef、精确位型、同值整数/handle、嵌套原位修改、重叠RHS、分支/零次循环初始化、失败replace、全出口drop、扩容epoch、错分支载荷、逃逸环境、跨store、失败take与freeze原值保留。

每个Plan分别列work-owned实现与定向反例、review-owned独立验收、merge-owned冻结全档。正常源入口支持面与typed MIR/ABC专用fixture分列；后者不能冒领源码语法。可对拍的record/text使用旧HIR-native及手审expected，其余用独立固定oracle/简单安全模型，禁止让同一被测布局算法生成expected。

实现只在对应组树，簿记在主检出；work/review不跑裸全档。独立review通过后merge按实际Rust触面执行裸cargo t及必要tv等；新独立workspace另跑其完整有限合同测试。tf只在主检出到期单实例，清树前wt-guard。当前只起草，所有实现/测试/复审/合入仍pending。

## 14. 一手参考与采用范围

检索日期2026-10-20（前序调研）。参考机制不构成AV已经实现的证据。

- [Rust MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html)：Place/Rvalue、copy/move与投影；不承诺照搬完整borrow checker。
- [Rust drop elaboration](https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html)：初始化状态与清理义务；首批部分move仍拒绝。
- [JVM verification](https://docs.oracle.com/javase/specs/jvms/se26/html/jvms-4.html#jvms-4.10.1.7)：装载时验证读写类型；AV不因此继承JVM GC。
- [Wasm struct验证](https://webassembly.github.io/spec/core/valid/instructions.html#valid-struct-set)：结构类型/字段/可变性/值类型共同检查。
- [Wasmtime Val/raw边界](https://docs.wasmtime.dev/api/wasmtime/enum.Val.html#method.to_raw)：安全值与绑定store/type上下文的裸表示分层。
- [QuickJS值表示](https://bellard.org/quickjs/quickjs.html#JSValue)：64位实现不要求NaN-boxing；AV的具体密度仍须自行测量。
- [Rust布局保证](https://doc.rust-lang.org/reference/type-layout.html)：不能以宿主默认enum布局替代wire/ABI合同。

### 初稿设计审阅（历史r1）

2026-10-20：设计/五Plan的静态交叉核查记录见[preflight.md](../plans/attachments/av-data-20261020/preflight.md)。仅为起草质量检查，不是独立实现review；实施状态未推进。


## 15. ABC补充合同与逐profile验收（r2）

全部A01..11决策及分工见[Design43 §12](43-ac-av-typed-mir-and-abc.md#12-abc评审采纳与实施责任)和[采纳记录](../plans/attachments/av-abc-adoption-20261020/decisions.md)。798 r4的类别往返及R10前门正确性先收口；下面的数据扩展和性能不搬入798。

### 15.1 表示与类型一致性

共享链为Typed MIR→portable ABC→私有LoadedProgram，语义类型、磁盘编码、解释器布局/帧分离。820固定安全访问接口及类型规则，不以825完成为前提；安全enum/CheckedMir参考路径仍经同一授信和生命周期API。825已交付时新profile明确适配或使用具名受检参考路径，禁止无验证fallback。

820建立统一类型等价规则与InstructionSpec表：primitive规范种类/索引归一化政策，Fn参数模式/返回/效果，名义record/enum的owner+声明revision。所有操作列read/def/kill/借用/trap/cleanup/逻辑fuel/来源/profile；新opcode、Place或类型连同各层覆盖及版本交付。

### 15.2 状态、清理、权限与预算

821引入资源时实现move后kill、借用权限、reservation/commit和正常/失败/取消全出口清理，不能沿用只增加初始化位的分析。只intern实际引用的根/投影MovePath及前缀，计路径数、状态字节和worklist步数，禁止按每槽展开所有类型字段组合；现有根move/部分move拒绝保持。

822将Index/branch状态、结构变更epoch和活分支清理接同一表；823将env字段/invoke的owner、模式接同一状态机；824将Dyn pack/take/borrow和portable恢复接同一类型规则。宿主边界区分artifact的requires与当前host的grants，字节码不能自授权限；命中缓存仍重验当前授权。

活分支、owner/borrow转换、资源清理、必要类型/签名不是debug，strip后仍完整。逻辑fuel按语义操作及数据量定义；大clone/drop/路径分析/freeze解码有有界计量与取消点，优化不隐藏工作或取消cleanup预留。cleanup失败须保留可诊断终态，不伪称释放成功。

### 15.3 性能边界与调度

820–824各增当前合同§5.4及V-11，负责自身新增profile正确性。825只优化已交付标量执行面，826只改其wire/装载；后续owned/collection/closure/Dyn不因此提前支持。两者不成为798或820–824的交付硬门。

两链共享核心，实现仍须串行交接；窗口以当时实际owner为准。体积/吞吐靠测量验证；NaN-boxing、unsafe存储、JIT及完整slot寄存器分配不随本次建议获得实施授权。

## 16. 所有权与验收责任复审补充

依据[本次复审](../plans/reports/av-architecture-contract-review-2026-10-20.md)，以下为当前合同；§15的r2为前次采纳批次标识，不改变本次实施边界。

**派生事实与合流。** Copy/needs_drop/size/align/stride从受检类型图推导，不能信任输入自称Copy而复制资源。可达CFG的确定初始化取交集；可能活owner/未结束loan取保守并集，cleanup依动态init/drop guard清理实际活值，不能把must-init交集直接当唯一Drop集合。借用返回的所有路径都须具有合法存活来源；终结/合流/循环back-edge在同一规则下求有界不动点。821负责，820建立Copy派生基础。

**guard与事务。** Variant guard证明在重赋值、分支切换、可能写入的调用后失效；动态Index依据owner/epoch重验。remove后旧Place失效，但重新计算合法索引仍可读移位后的新元素；普通MoveOut(Index/Field)不因remove合法而开放。822验证上述正反对照。

Replace/List提交/CaptureCreate分prepare与commit：所有可能失败的分配、clone、来源/借用校验和清理记账预留先完成，待move来源由reservation保护，最后在无失败/回调/取消观察的短提交区转移责任。提交前失败清理临时、不消耗尚未提交源；已发生的普通guest求值效果不回滚。提交后若trap/cancel，清理新owner而不“还原”已完成的move。823嵌套env字段转移只允许专用受检capture操作并跟踪活字段/调用读集，普通资源字段move仍拒绝；不能靠裸取出字段跳过部分初始化状态。

**动态与恢复。** Dyn白名单是递归闭包：List/Option/Result/record的内部类型也必须允许；包含closure/borrow/resource的外壳不能pack/freeze。首批owned树入边唯一，重复子节点引用即便无环也不是合法owned树；恢复先验证结构/预算，再分配重建，失败逐个回滚，不让同一资源有两份owner。

**计数、预算和后继。** 准入非法时dispatch=0；运行期受控失败按目标访问/提交/子调用计0，并核合法前缀和cleanup。预算前拒的申请attempt=0，allocator故障注入则该attempt存在但successful_alloc=0，先前合法事件不得抹掉。32MiB受管分区、128MiB进程增量及数据合计额度沿实际合同分别校验，不扣前端，不按对象类别重复领预算；798资源证据冲突仍归798收口，后继只消费真实D门。

各Plan在新增V-12（820–824）及V-11（825/826）落地这些责任；功能链/优化链与版本兼容、窗口无环启动以Design43 §12.5为准。源码空record仍沿既有前门拒绝，零长集合是822独立正例；typed空布局未另冻结时不能冒充已支持源码或绕过stride检查。

### 16.1 早期增量与热重载对数据合同的约束

AV是ACE的解释执行组件。后续函数内block/语义区域方向见[Design43 §8.1](43-ac-av-typed-mir-and-abc.md#81-函数级首版与block级扩展方向2026-10-20)：区域边界须描述入口/出口初始化、move/loan、活值、布局及清理责任；源码block不天然形成可独立替换的运行边界。首版按函数保持这些事实，未来区域化通过受检边界摘要与映射扩展，不允许复用旧块时遗漏跨块借用、Drop或别名影响。本节不新增现有Plan的block实现验收。

统一平台基础以[Design43 §6.1](43-ac-av-typed-mir-and-abc.md#61-早期架构约束与分期实现2026-10-20澄清)为准。数据布局/Place/所有权从首次设计就必须考虑代码多代共存：TypeRef与LayoutRef的持久身份不能只是当前进程索引；body、接口、layout及清理逻辑改变分别影响所需依赖。装载态短ID只能在其所属代际内使用，不能因逻辑类型同名直接复用旧布局。

活frame、closure环境、对象、借用及回调仍需要的类型/布局/代码/Drop实现按其真实依赖保活；旧对象不按新布局读取，旧闭包不因函数逻辑ID相同就改用新环境解释。新代安装、旧代释放和cache GC共用生命周期边界，资源压力时受控拒绝/背压，不能回收活借用或无限保留旧代。跨代传值适用哪些兼容检查、是否支持迁移，由实际重载profile明确冻结，未经证明的组合明确拒绝。

本节是后续接口与实施规划的约束，不声称820–824已承诺实际在线热重载；现有Plan不自动追加任务。每次扩source/集合/闭包/Dyn或外部资源，应检查新增存活路径及失效边是否兼容统一合同；实际多代行为由较早的专门验证计划承接，再随类型扩面补适用反例。

## 17. 从数据内核到真实Auto程序（2026-10-20）

[语言能力矩阵](strategy/av-language-capability-matrix.md)将本批数据profile与源码/库/产品能力分开；[G0–G8替换路线](strategy/av-source-programs-and-avm-replacement-roadmap.md)以真实程序和实际入口验收推进。List/Variant/closure/Dyn的typed MIR首例不等于源码语法/推导/方法/stdlib已经接通，821 strict Str/str规则也不能自动承接旧str字段程序。

后续R2-*..R6工作包按同一Type/Place/owner/预算合同扩source与consumer，已有生命周期正确性不能在源码adapter、host callback、async/共享扩面中绕过。新的挂起/共享/外对象必须先有明确设计和独立验收，不拿823同步环境或824portable树替代。完整应用另立版本化profile/预算，不反向豁免本批cap或冒称所有应用都受实验8MiB限制。

新增[827源程序验收基线](../plans/archive/827-av-source-program-baseline.md)在D798后首先验证现存scalar文件、实际AV管线/恢复/父侧oracle；不修改820–826当前revision/任务，也不要求798为其额外交付产品CLI。本文与Design43现有正确性/单写依赖继续有效，本轮未实施任何代码。
