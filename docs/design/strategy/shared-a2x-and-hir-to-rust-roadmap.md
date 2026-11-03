# 共享A2X与HIR→Rust实施战略

> **2026-11-02最新M2目标**：[M2合同r2](../../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)要求补全ACE并接管旧AutoVM/A2X所有现有应用、目标和入口，M2结束完成旧实现disable。C为M1第二验证目标；旧Python/JS/TS/Godot等实际目标仍须在M2由新provider接管，不能调用旧AST emitter兜底或只Rust/C绿就收口。旧M4产品迁移/退役义务并入M2，M3/M4未来新增扩展暂不规划；既有M1及在途合同保持。

> **2026-11-02当前r5：C第二目标纳入M1。** 用户已明确批准，见[M1增补决策](../../plans/reports/ace-m1-a2c-scope-addendum-2026-11-02.md)、[Design51 §13](../51-shared-a2x-and-hir-to-rust.md#13-a2c第二目标与m1范围增补2026-11-02target-state)、[C实施路线](shared-a2x-and-hir-to-c-roadmap.md)。892/902/904的Rust首期均已交付；新增C预计CA-A共享接缝+scalar、CA-B record/text资源、CA-C集合/变体、CA-D有限ABI桥、CA-E新增登记/Q4总验收五包（工作标签，尚未取号）。C替Python成为M1第二目标，不能只交标量演示；Q4=AC/AV/新A2R-HIR/新A2C-HIR，各对独立oracle，旧A2R身份/313门不被替代。新M1为旧门AND新增C/ABI/Q4门，未接新总门不得凭旧acceptance/2签章。913/914/916已实际在途，915/917仍drafting，shared store/worker/H按具体SHA/全consumer单写交接，不新增整波前置。下方r4与893/903/904在途描述保为历史；本轮只有设计/规划，不改current Spec/source/expected/binding或913–917合同/进度，不取新Plan号、不建树材料或施工。

> **2026-11-01当前治理交接**：[908](../../plans/908-ace-m1-rules-registry-and-acceptance.md)drafting r2、0/12，G-DEC-1..5已全部按推荐批准，已有批准直接继承，详见[决策件§13](../../plans/reports/ace-m1-scope-decision-2026-10-28.md#13-补充裁定g-dec-15plan-9082026-11-01)。AX-C904已archived r2、14/14，delivery12f9adf18在main，含type/enum片段与collections资源/增量/认证正式接口；AX-A892/AX-B902也已交付，旧“等待S修复/904占窗”不再适用。893/903均已归档；新组case-a2r-hir-collections-904及显式a2x-coll-formal只影响对应成本/审计输入，不重做整包。908新旧a2r身份严格分列，collections cert必须绑定execution_config、旧schema不得夹带、互不授信；不以新producer替旧M1能力/证据。908私有规则/来源可先实施，公共固定组/宿主注册与906当前WIP逐符号交接；顺序见908 §5.8，批准适用不冒执行PASS。本轮只修文档，binding/expected/规则实现未迁移；下方r4等保为历史规划，当前状态以本条/交付Spec为准。

> 2026-11-01，规划r4；[Design51 §5.4](../51-shared-a2x-and-hir-to-rust.md)，[ACE总路线](auto-native-backend-evolution.md)。AX-A892、AX-B902已交付；AX-C正式立[904](../../plans/904-hir-to-rust-collections-variants.md) drafting r2、0/14，type/enum片段缺口同包有界S前置。893/903由用户确认实施中，按实际符号/机器窗交接，不设整包前置。本轮只规划，无实施/测试/当前Spec修改；旧立项状态由此和实际收据取代。

## 1. 定位与本轮结论

v0.6 Rust版ACE包含A2X；AACE的Auto主体仍延后v0.7。新增ACE-A2R先消费共享Checked HIR，不依赖native的COFF/私有ABI，也不把所有转译后端强制改为MIR输入。v0.6全现有AutoVM/A2X对应示例仍是版本成熟度方向，首期下面三包只交付有限source profile，不自动完成M2/M4或旧转译器替换。

**Rust首期三个Plan已交付：AX-A892、AX-B902、AX-C904。** C作为M1第二目标新增五包CA-A..E，详见当前r5与C路线；Rust原语义/资源/增量/认证合同保持，不重开归档，也不沿历史“号源905”预占C计划号。

不另立无真实消费者的通用A2X基础平台包；CA-A以C真实消费者带动a2x-store/Rust TargetProfile接缝去耦，每包含自己的文件化、独立oracle、增量/cache、资源/周转、消费者与交付门。C验证静态表示/显式清理与有限ABI，Python后继再验证动态整数/GC差异，不为接口共用率重写所有目标。

## 2. 共享层与目标后端合同

| 共用部分 | 权威来源/实现方向 | 目标私有部分 |
|---|---|---|
| 语义上下文 | ac-core受检HIR/函数、源类型/参数模式、绑定、CTE/实例及依赖查询；查询缺口由producer补正式出口 | Rust/Python/C等的类型与库映射、生态适配和能力准入 |
| 结构化准备 | 源码序/单求值/短路、临时ID、共同walk/rewrite及必要规范化；保持结构和来源 | 每目标需要的转换pass、Rust borrow/生命周期安排、C清理/控制流、Python语句化 |
| 操作合同 | 溢出/索引/字符单位/move/clone/drop/失败原子性及helper需求 | 目标原语/typed helper具体实现、Target AST、字符串转义和打印 |
| 编译服务 | 函数/声明依赖、source map、分层cache与resident、完整文件集装配、原子发布/GC、独立认证框架 | 目标edition/flags/runtime身份、imports/模块组织、真实编译器调用 |

目标后端应按能力组合公共服务，不扩展为每节点几十个必实现hook。共享core不能重新维护推导/ownership权威，也不能依赖旧AST/Database、AV执行器、Cranelift或UI/Man。当前实现统一位于experimental/ace/crates/a2x-{core,rust,store,worker}，904扩既有四crate，不新增workspace/member或根产品实验依赖。

## 3. 首期三个交付包的规模与验收

| 工作包 | 目标与实际闭环 | 非目标 | 主要写面/前置 | 必须关闭的设计case |
|---|---|---|---|---|
| AX-A [892](../../plans/archive/892-shared-a2x-hir-to-rust-scalar.md) 已交付 | core/scalar HIR→共享prepare→Rust、真认证及增量cache/资源基础；current实现见auto-a2x/auto-a2r Spec | 非record/text、第二目标或旧产品切换 | 当前experimental/ace四A2X crate；最终11/11收据，不按header旧6/11误判 | 已交付有限合同，后继域各自独立认证 |
| AX-B [902](../../plans/archive/902-hir-to-rust-record-text-resources.md) 已交付14/14 | record/text/text-record、参数模式/资源、真Rust认证与缓存基础；实际接口见current Spec | 非List/Variant，含type暖片段仍留前端项，方法编辑仍冷路径 | delivery4303c4457050在main，已清树/材料；不重开历史Plan | 904补type自由函数片段，不能沿902 type-free证据冒含声明增量 |
| AX-C [904](../../plans/archive/904-hir-to-rust-collections-variants.md) 已交付14/14 | 原14 L/V+6补源、拥有List/Variant、is/for-in、资源/失败/真函数增量/认证 | 保原源码边界，typed不扩任意参数/字段域 | D12f9adf18在main；type/enum片段与资源事实已交付 | 20原源+独立reference/正式增量与认证收据，见current Spec；不是C目标CA-C |

每包先最小全链feasibility，再扩实现、完成work-owned验收、独立final review、冻结门禁与真实交付/Spec沉淀/归档guard。需要新语义/预算/支持集时修正式Plan，不靠“后续再做”关闭既定AC。AX-B/C正式立项必须逐条列源/类型/参数模式/操作闭包，当前Spec支持与实际source缺口不一致者明确S owner及前置；必需输入未闭合不能仅登记gap后把包判交付。

902已交付R07/R08=42、T03=23/T05=42及受检text资源出口，但记录含type暖片段和方法编辑冷路径的明确限制；source Collections已由864/889交付，不等Rust新域已过。904不重开902/881/894，补共享source声明/fragment来源和Collections帧拥有事实，用新实际证据更新限制。

整包能力默认AX-A→AX-B→AX-C；前两包已经交付。当前type片段缺口与AX-C source候选/名义图共用同一S缝，直接纳904 T-02最小真链/T-03修复，不再取一个无新消费者的前置包号。若有界机制不成立则正式replan，不以whole重编或全span0放行。独立reference和P准备可与893/903并行，不消费他方WIP。

904 r2规划复核补齐V-13..17：同context异body凭证拒、完整source snapshot/range与连续两函数编辑/恢复、操作计费/取消和无余量清理、证书config/mode/每修订两腿证据、893903精确文件符号与受影响来源回交。20源/14任务及原数值门不减。type片段仍同包有界S前置，当前未施工，next仍T-01/T-02。

## 4. AX-C立项时的执行入口（历史）

以下为904立项时的执行顺序历史；904已归档交付，不再按此启动实施。当前C工作入口见C路线§7/§10：

1. T-01核902 delivery/Spec限制、864889原20源hash/独立值、唯一树、893/903实际符号/预算/组。
2. T-02含type/enum短fragment+L04/L05/L06/V03/拥有Variant最小source→Rust→认证和正式首例，冻结资源事实/typed表示/深链清理/新身份/负载预算。
3. T-03关闭共享type片段S修复，含902自由函数兼容见证；T-04再扩Collections受检事实/view/依赖，T-05目标runtime。
4. T-06真函数候选/source/env/居民/坐标/fileset同代发布、失败保旧/历史/restart/只读/lease/close；T-07新recipe/cert/20源TestCase与H接线。
5. T-08正式AC/V/四状态30样本及物理/受管预算与反控；其后独立review、冻结职责门、真实沉淀/归档/guard清理，不把计划表当通过证据。

## 5. AX-C与893、903的写面（历史）

本节保留904实施时与893/903的交接理由，两者现已归档。新C实际交接913–917见C路线§9，仍按实际合同/diff与稳定SHA，不延用旧owner/工作树/等待状态。

| 面 | 904范围 | 接受/回交 |
|---|---|---|
| P-A2X/Rust | 四crate的Collections语义扩展、typed runtime/私有测试/独立reference | 可并行，不新建workspace/worker体系，不碰native/旧Trans |
| S | core source声明/fragment来源、Collections verified facts与A2X中立view/依赖 | 893若写同测试/观测符号则单写；903只读审计不占写权；必要AV/native/CTE消费者验证 |
| H | 新recipe/contract/dispatcher/ac_a2r_hir/cert/current身份、suite/固定组/tools | 893接入/工具注册按具体符号交接，append-only不等无冲突；903不被动承担新producer迁移或新M1范围 |
| 正式运行 | 904资源/认证/性能 | 与893慢采样、903必要窄重验错峰，不停止他方进程、不持H等机器窗 |

无D893/D903整包前置；新能力只消费review/gate合入的正式版本，既有不变主线符号明确停止并写可释放。收据含owner/revision、文件符号、delivery、schema/profile/pass/helper/tool/flags身份、已跑/pending及回交。不从902继承缺失的集合资源事实或type-free性能结论。

## 6. 首期之后的扩面与里程碑

- 第二目标C现为M1必需，按CA-A..E及Q4/有限ABI门执行；Python改后继，验证其整数/失败/GC差异。各目标独立oracle，公共bug不能以目标间相等掩盖，C未交付前不宣传多目标复用已实证。
- 闭包/环境/高阶调用、host-import/console输出：消费865/863已交付source事实，各自适配Rust表示、授权和精确输出/失败，分包是否可合并在真实模式/资源矩阵后定，不顺带加入AX-C。
- 多模块/stdlib/泛型/meta/反射/复合CTE/FFI、wide源码、多平台：按真实producer和对应目标需求推进。实例/类型/CT结果必须追全实际依赖，source相同不能作为命中依据；缺前端支持不由生成器猜类型兜底。
- 旧产品迁移/退役已纳M2：新增ACE各目标覆盖先并列记录，按原示例/后端/平台/依赖矩阵真实编译运行，不要求原不支持跨积、不减分母。M2需全目标/全consumer接管、实际部署切换及旧生成器退出构建/发行/调用，不能只Rust三包或C目标通过就签M2，不再等未来M4退役。

现有M1 a2r缺口（名称解析/未定型发射/缺金样等）仍以原source和oracle对账；新producer成功可以提供新证据，但是否替换原路径及认证profile须正式迁移/审计决定，不把旧accepted-diff或旧认证缺口抹掉。完整AJ/发布器/block级增量及AACE不作本线首期统一前置。

## 7. 本轮交付与下一步

2026-11-01正式立904（drafting r2、0/14），同步Design51 §5.4/本战略r3及ACE入口/交接；type声明片段修复同包有界前置，原20源/资源/函数增量/认证和893903边界按904。所有实施/测试/性能门待执行，本轮不建树/agent、不改current Spec/原源/expected/binding或他方计划。下一号905，其他后继不预占号。
