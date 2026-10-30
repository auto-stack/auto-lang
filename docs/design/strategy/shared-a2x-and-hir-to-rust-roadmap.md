# 共享A2X与HIR→Rust实施战略

> 2026-10-30，规划r1；技术合同：[Design51](../51-shared-a2x-and-hir-to-rust.md)，统一入口：[ACE总路线](auto-native-backend-evolution.md)。初订轮只授权设计/规划。后续用户已授权AX-A正式立项：[PLAN-892](../../plans/892-shared-a2x-hir-to-rust-scalar.md)，drafting r1/0-of-11；仍不建树、不施工、不改889/891/888合同或current Spec/ledger。

## 1. 定位与本轮结论

v0.6 Rust版ACE包含A2X；AACE的Auto主体仍延后v0.7。新增ACE-A2R先消费共享Checked HIR，不依赖native的COFF/私有ABI，也不把所有转译后端强制改为MIR输入。v0.6全现有AutoVM/A2X对应示例仍是版本成熟度方向，首期下面三包只交付有限source profile，不自动完成M2/M4或旧转译器替换。

**首期拟拆三个Plan：AX-A共享骨架＋Rust标量闭环，AX-B Rust record/text与资源，AX-C Rust集合/变体。** 标签是设计工作包标识；AX-A现对应[PLAN-892](../../plans/892-shared-a2x-hir-to-rust-scalar.md)，AX-B/C尚未取号，仍不一次预占后两号。892合同冻结本包scope/AC/V/T/SD和S/P/H交接，取号892→893只属计划簿记，未授权施工。

不另立无真实消费者的通用A2X基础平台包；每包含自己的正式文件化、独立oracle、增量/cache、资源/周转、消费者与交付门。第二目标Python scalar用后继实切片验证共享抽象，当前不为追求接口共用率而重写所有目标。

## 2. 共享层与目标后端合同

| 共用部分 | 权威来源/实现方向 | 目标私有部分 |
|---|---|---|
| 语义上下文 | ac-core受检HIR/函数、源类型/参数模式、绑定、CTE/实例及依赖查询；查询缺口由producer补正式出口 | Rust/Python/C等的类型与库映射、生态适配和能力准入 |
| 结构化准备 | 源码序/单求值/短路、临时ID、共同walk/rewrite及必要规范化；保持结构和来源 | 每目标需要的转换pass、Rust borrow/生命周期安排、C清理/控制流、Python语句化 |
| 操作合同 | 溢出/索引/字符单位/move/clone/drop/失败原子性及helper需求 | 目标原语/typed helper具体实现、Target AST、字符串转义和打印 |
| 编译服务 | 函数/声明依赖、source map、分层cache与resident、完整文件集装配、原子发布/GC、独立认证框架 | 目标edition/flags/runtime身份、imports/模块组织、真实编译器调用 |

目标后端应按能力组合公共服务，不扩展为每节点几十个必实现hook。共享core不能重新维护推导/ownership权威，也不能依赖旧AST/Database、AV执行器、Cranelift或UI/Man。拟新实验workspace为experimental/auto-a2x，core/rust/probe角色及crate划分由AX-A最小可行性任务冻结，根产品workspace零新增实验依赖。

## 3. 首期三个交付包的规模与验收

| 工作包 | 目标与实际闭环 | 非目标 | 主要写面/前置 | 必须关闭的设计case |
|---|---|---|---|---|
| AX-A [PLAN-892](../../plans/892-shared-a2x-hir-to-rust-scalar.md) 共享骨架＋Rust标量 | .at→已交付core/scalar前端→Checked HIR→共享最小prepare→Rust→真实rustc认证；i32/bool、函数、递归/控制流/短路/已支持调用、精确算术trap与诊断；新显式入口/金样/recipe、单体编辑/恢复/预算首例齐全 | record/text/集合/闭包/console/wide源码、多模块/FFI、旧a2r默认切换；不要求先完整meta/query平台 | 新P-A2X私有实现；必要S前端/受检函数出口与H SDK/certify窗口；不依赖D889/D891语义 | AXV-01..05/08/09；第一条source真实生成+编译+运行，不仅文本快照 |
| AX-B Rust record/text与资源 | 名义record、str/Str/char、字段/构造、静态/view方法及已支持view/mut/move/显式clone、owned返回；source顺序/borrow/资源事实→Rust表示/typed helper→独立认证，真实source增量 | List/Variant、容器clone新语法、闭包/HOF、console、全泛型/stdlib、全既有a2r生态 | 正式D(AX-A)共享接口；新增P-Rust资源实现、受检resource事实缺口归S、公共H交接；语义依据770/771/774/881等已交付合同 | AXV-02..06/08/09；Unicode/NUL、参数模式、重赋/多move、clone独立、失败清理/一次释放与借用生存 |
| AX-C Rust集合/变体 | 864原14 L/V源及Str/拥有record/Variant List，有限for-in/is受检结构；索引trap、活动臂、转移/view/失败原子性与清理→真实Rust认证；复合source精确增量 | 嵌List/List参数/返回/字段、源码List/Variant.clone、wide集合、闭包/host输出、多平台 | 正式D(AX-B)资源与共享接口；864 source合同，命名enum等具体S修复需owner正式交付；不依赖889的native runtime/ABI或891 wide entry | AXV-02..05/07..09；原L04/L05=42、L06具名trap、V03=3与独立复合补例；source/typed辅助资源分账 |

每包先最小全链feasibility，再扩实现、完成work-owned验收、独立final review、冻结门禁与真实交付/Spec沉淀/归档guard。需要新语义/预算/支持集时修正式Plan，不靠“后续再做”关闭既定AC。AX-B/C正式立项必须逐条列源/类型/参数模式/操作闭包，当前Spec支持与实际source缺口不一致者明确S owner及前置；必需输入未闭合不能仅登记gap后把包判交付。

AX-B明确把原R07/R08=42、T03=23/T05=42列required关闭目标；其中T03/T05当前不是可直接消费的全绿source基线。先核源门/Checked HIR/MIR消费者各自拒绝阶段，已有合法语义的源门缺口由AX-B接受S-text有界修复或消费其正式前置；若仅MIR lowering不支持，A2R直接消费合法HIR，无需修整个AV/native消费者。缺口过大时在正式立项前进一步拆共享前置，不能用“新后端只做typed”削掉原源目标。此范围判断不自动授权重开/修改他方Plan。

默认串行交付依赖为AX-A→AX-B→AX-C，原因是同一个共享core及Rust语义/资源基面。没有共享写入的oracle、runtime参考材料及后端私有可行性调查可并行；不消费前包WIP、不复制暂定core接口。首期不把增量/认证/资源独立排为末尾“收口包”。

## 4. 首个Plan的可执行准备入口

AX-A已由[892合同](../../plans/892-shared-a2x-hir-to-rust-scalar.md)冻结scope/revision、基面、拟实施树和SD；本节保留设计入场要求，执行任务/验收以正式Plan为准。

1. 核ac-core::verify::{CheckedModule,CheckedFunction}、source::{CheckedEnv,prepare_candidate_document/FnCandidate}以及body_work（在lib.rs内联模块）实际API。读入成功不等后端已经有所有facts；source候选受检证据、decl/signature/param-mode/context与span投影若不足先做最小S出口，不从AV执行结果推导Rust代码。
2. 冻结core/scalar最小支持矩阵、TargetProfile/语义和pass版本、新Rust入口/结果/trap wrapper、输入/target身份、typed helper及受控rustc复用边界；用源码→HIR→Rust→真实程序先证短路/溢出/参数顺序，再扩普通发射。原旧int=i64发射经验不覆盖新i32 profile。
3. 冻结函数/准备/目标片段/装配/认证五层key、实际依赖清单、fresh/hit/read-only/restart/body-edit和候选发布/close规则。必须有真实生产点计数与resident identity反控，不能整module编译后筛hash。
4. 第一例接文件化AutoDown/独立oracle和新金样/证书轴。日常emit+snapshot spawn0，独立认证消费现场actual经真实rustc和独立reference各自对oracle。新recipe/profile/wrapper注册需H接受，不复用旧a2r/emit身份冒认。
5. 核当时有效测试门/模块与源锁，冻结样本源/规模/数量、P90≤2000ms日常面及单列认证预算；按Design51先核48MiB拟议管理工作集、4MiB源/IPC与产物cap及更严上游约束，给项目cache/磁盘配额与测量方法。缺预算/窗口/调用事实不进入广铺实现。

AX-A输出拟增加current Spec：auto-a2x架构/准备/能力与cache合同、auto-a2r受检scalar后端、TestCase新profile/认证面；实际docs/specs路径在正式Plan核目录后确定。本轮不创建Spec来声称实现。AX-B/C沿同一Spec扩支持矩阵与资源/集合规则，不另建平行后端语义体系。

## 5. 与889、891、888的真实交接

读取状态：master=4ea7780aa；889 r2执行5/10（T-06仍有fresh/restart性能修复及S复验，未交付），888 r2执行5/10（T-06/07/08成本/入口/验收未关闭），891 r2草案0/10。此为读取快照，不据该数字占有或释放任何窗口。

| 写面 | 当前归属/交集 | A2X接入原则 |
|---|---|---|
| P-A2X / P-Rust拟新私有目录 | 新实验共享core、Rust provider、私有测试/新helper；当前无实现 | 用户未来授权实施后，确认新路径独占，可与889 N及891 N并行准备；只读复用主线已交付HIR/源接口，不编译链接他方候选来证明前置 |
| S：ac-core/source/adapter/resolve、HIR/verify/schema/资源facts、单体候选 | 889 T-05/06正在使用/修复；包括冷绑定批处理和ownership相关优化，不因功能已提交就默认释放 | AX-A所需受检函数出口、AX-B resource投影、AX-C命名enum或单体修复逐文件/符号取窗。默认889该S稳定交付后接，或正式阶段收据交接；不复制shared lower/前端，也不把修复所有source gap塞进AX-A |
| N：ac-mir-native emit/lower/layout/fingerprint/config/cache/runtime | 889先行；891后续按其r2接N | A2X不写这些文件，不依赖MN_ABI/COFF/private wrapper；不要求等D891。即使A2R由Rust输出机器码，也仍是独立目标工具链合同 |
| H：test_cases dispatcher/ac_mn/ac_av、contract、report/stale、certify、worker/helper执行器 | 888的模块/批量工具，889的collections正式SDK和891拟typed入口均可能相交 | 新A2R adapter/recipe注册、共用certificate/exec helper按真实diff交接，跨P/S/H同时取需要的子窗；独立目录或append-only不等无冲突。旧ac_av/av_probe_worker、native worker没有具体必要则只读 |
| H治理：测试发现/模块、消费者边、源锁、Cargo/nextest、正式采样 | 888仍在冻结和成本收口；其获批规则非所有工具代码均已交付 | 新义务进当时有效治理清单；888已交付则接其叶模块/affected并集，否则沿现行入口并登记待映射，不读写888 WIP配置/冻结报告，不强迫采样中rebase，不默认把新recipe加入B/C |
| R旧A2X/SDK：trans.rs/Sink、旧Rust/Python发射、a2r-std、legacy golden/cert入口 | 已交付884/885等为基面，仍可能被Web/UI等真实owner占用 | 仅审计复用中立工具/adapter，涉及修改拿实际R/H窗口；不顺手重构旧后端/改变金样或产品默认，不改外仓Web/UI/Man |
| 机器计量/重型门 | 889及888均有正式性能/成本样本 | 设计/只读与不争载材料可并行；未来构建与认证会占CPU/磁盘，正式采样明确错峰，不能凭空闲或静默当交接授权，不主动跨聊天发消息 |

D889不是A2R全部工作的语义前置；**具体未交付S事实才是对应任务的前置**。D891也不是A2R源码后端前置；wide typed MIR与宽源码HIR分开。U888不是整包前置，但公共H写权、模块/源锁和计量窗口不能越过。

正式交接收据至少含：前owner/Plan revision、文件/符号、真实主线/阶段delivery SHA、schema/profile/语义/pass/target/helper/ABI与缓存版本、依赖/限制、已跑与pending门、后owner接受及回交。稳定阶段若作为新输入，必须经过适用review/gate并合入主线；仅commit/scoped绿不消费。若只是释放主线既有未变符号，记录正式基面/停止并写即可，不新增整项依赖。不可持有一窗等待另一窗；窗口不可用先推进已释放独立工作。

## 6. 首期之后的扩面与里程碑

- 后继第二目标Python scalar：真实复用core语义view/pass/依赖与artifact服务，Python自己的整数/失败/引用行为由目标适配保证；各目标独立oracle，公共bug不能以Rust=Python相等掩盖。范围验证后再决定更多接口提取，未验证前不宣传通用多目标平台已成熟。
- 闭包/环境/高阶调用、host-import/console输出：消费865/863已交付source事实，各自适配Rust表示、授权和精确输出/失败，分包是否可合并在真实模式/资源矩阵后定，不顺带加入AX-C。
- 多模块/stdlib/泛型/meta/反射/复合CTE/FFI、wide源码、多平台：按真实producer和对应目标需求推进。实例/类型/CT结果必须追全实际依赖，source相同不能作为命中依据；缺前端支持不由生成器猜类型兜底。
- 旧产品迁移/退役：新增ACE-A2R覆盖先并列记录，按原示例/后端/平台/依赖矩阵真实编译运行；不要求全部交叉积，也不减原分母。切默认与删除旧生成器另需覆盖/消费者/恢复门，不凭前三包结束宣布M1/M2/M4完成。

现有M1 a2r缺口（名称解析/未定型发射/缺金样等）仍以原source和oracle对账；新producer成功可以提供新证据，但是否替换原路径及认证profile须正式迁移/审计决定，不把旧accepted-diff或旧认证缺口抹掉。完整AJ/发布器/block级增量及AACE不作本线首期统一前置。

## 7. 本轮交付与下一步

初订交付Design51及本战略、设计索引/Design43与旧生成设计的导航、ACE路线/交接更新；随后按用户授权正式立892，仅规划。所有未来probe/测试/编译运行/资源和性能门未执行；未建树/agent，未改implementation/canonical Spec/ledger，未联系或接管其他会话。

初订历史静态收据（892立项前）：Design51编号唯一、设计索引/路线回链、本地文件引用及已读source hash一致；Design51的13个编号章、战略7章、三个当时未编号包与AXV-01..10映射齐全，变更范围diff-check通过。Plan号源当时892；这些检查只证明文档结构/引用一致，不证明目标架构或任何实现已验收。

下一步用户指定执行892时从其T-01核真实D门/树和写面，T-02首例需P/H最小窗及必要S。AX-B/C保持未编号建议，待前包正式接口/资源证据与实际工作量再立项。旧“未取号/号源892”静态收据是初订历史，正式取号后为893；本文不自动授权施工。
