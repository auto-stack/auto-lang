# PLAN-796 / PLAN-797 并行、依赖与共同接口

2026-10-16。两份draft r1的共同附件；仅规划，不创建代理/执行树，不修787/788。主检出调查9c773601abfdb25b80c08d06922d0ad50edc653a。当前793–795为他路线已领取编号，本轮排他领取796/797，不改前述合同。

## 写入所有权

| owner | 独占主写面 | 消费/禁止写面 |
|---|---|---|
| [796](../../796-ac-semantic-metadata-inference-probe.md) | 新experimental/ac-semantics、sema fixtures/报告；语义ref/schema、有限推导/查询；D-788后受限probe adapter公共接缝 | 只读旧AST/类型实现与783 API；不改ac-core语言/783底层/787/793–795代码 |
| [797](../../797-autoabi-v01-bidirectional-probe.md) | 新experimental/auto-abi-probe、独立C/reference、ABI fixtures/报告与target layout/calling/资源合同 | 796合入前只读ref快照，合入后消费其库；不改语义Schema/公共adapter、旧774/770/771/782布局 |
| 787 | ac-syntax与783有限schema/codec/store接入/修复 | 新两计划不改这些文件，不复制或cherry-pick在途实现 |
| 788 | auto-test-case v2、auto-lang test_cases/auto-case、ac-case-worker与ac-core观察/执行器、783 driver窄接缝 | 新两计划等其合入版；不修改worker在途IPC或与其同时写decoder/CLI |
| 792/793/794/795 | UI独立构建、diagnostics、scroll host、package-lock等各自生产路径 | 新两计划不写旧error/lexer/VM/TypeStore/native宿主/lock/Target，不迁仓或添加根members |

不同worktree不是无冲突证明。实现开始核实际diff、manifest/路径与工具版本。796完成公共TestCase扩展后，797才追加其recipe，不能两树并行修改contract.rs/mod.rs/CLI。根manifest/锁文件/公共注册与Spec导航若确需改，短窗口单写者，后合入者同步新master并复验。新语义库放独立workspace，不因方便把实验编译器接进auto-lang/UI/man产品构建图。

## 依赖图和非循环时序

1. 796 T-01–05、797 T-01–05可从已合入master分别实施；不依赖787/788全部完成，cache首例就用783已交付受管store。
2. D-788=788最终review、冻结提交门禁、实际合入、Spec沉淀/merge收据齐全。只读在途原型不解锁依赖。
3. 796 T-06等待D-788，真正注册/执行AutoDown；完成全部验收并review/merge后产生D-796。
4. 797 T-06等待D-796和D-788，接真实语义contract/公共probe adapter，再全重验互调/AutoDown/周转；之前不能标完成或把模型/fence存在当runner已执行。
5. 新两计划都不直接依赖787 API；788本身需要787，故最终共享测试集成间接等待787。这个等待只约束末段，不把全条路线冻结。797不成为796前置，依赖图无环。

## 共同语义与测试接缝

796拥有DeclRef/TypeRef/版本内容指纹与规范描述的逻辑合同，797拥有target layout、public symbol/physical signature/ownership服务合同。逻辑bool与ABI公开u8不混同，转换显式；宿主InternId/地址永不持久化。按值递归布局与间接递归语义分开处理。

正式字段名/编码在796 T-01冻结；797记录只读schema快照hash并使用独立物理fixture，不永久复制语义库。D-796后须删除临时重复校验/改用真实codec并复验，不允许为赶冻结保留两套权威模型。共同接口变更在计划收据记录新版本/受影响case，不能无声覆盖。

796负责受限probe/run注册合同（复用788执行/报告/预算机制）。只接受语义/ABI的已登记recipe和可信工具身份，AutoDown不能携shell或任意外部命令。797按该合同追加自己的worker/recipe文件，不另写.ad解析器或第二runner。若788最终支持表/协议不够用，在796公共接缝单一所有者处修订并测试，797不打补丁绕过admission。

AutoDown文件/oracle从T-01即进入版本控制，早期Rust定向探针执行的状态和待绑定runner状态分列；依赖未满足只记pending，不声称AutoDown通过，也不能因此跳过最终AC。

## cache、资源与周转共识

两producer独立namespace，复用同一783 storage/ref/root/lease/GC约束，不改底层；字段/候选集合/ABI/target/body/工具链/算法等真实输入决定key。compile-time结果接缝是模型，不伪造AVM运行；原生对象命中仍真link/run，每次重比较oracle。

每阶段从首例有trace/计数/资源，保留实际等待/I/O/发布；原始测量由独立执行工具与报告双向绑定。阶段P95不可相加成总P95；最终硬门以各计划规定的用户路径P95<1s，不能通过减检查、丢失败样本、提高预算或换成空查询满足。完整cold构建/广泛失效另报，支持面如实。

使用每计划唯一组树，禁止junction/symlink；组内依赖按env→兄弟→主检出解析。最终review各自同树独立上下文顺序进行，merge-owned整档和主检出单实例批量回归按现行纪律；并行探针不能扩大为多树tf/heavy并发。
