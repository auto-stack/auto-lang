# ABC建议采纳与设计合同修订

日期：2026-10-20，Asia/Shanghai。授权：用户认可ABC评审，要求补入Design43/44及实施计划，区分798正确性与后续性能，先不写代码。

本附件记录**文档采纳**，不是代码修复或独立实现review。原[ABC评审](../../reports/798-abc-design-review-2026-10-20.md)、[八文件基线](../av-abc-review-20261020/baseline.json.txt)、798原R10报告和全部历史收据保留。规范现状不回写；未来merge按各Plan SD沉淀。

## 1. 当前合同与实施边界

| 文档 | 修订 | 本次结果 |
|---|---|---|
| Design43 | §12正式采纳 | MIR/portable ABC/私有LoadedProgram、类型/指令合同、调度及WASM/V8口径 |
| Design44 | §12/15 | 功能/性能链、语义元数据、路径/owner/预算逐profile责任 |
| 798 | r2→r3，executing | 新§5.7/V-14、AC/SD/任务重开；必须复现/收口kind往返，不宣称修复 |
| 820–824 | r1→r2，drafting 0/8 | 各新增§5.4、V-11，扩AC/T及SD-01/02/03；原功能链不变 |
| 825 | 新r1，drafting 0/8 | 装载表示、O(1)PC、安全调用/帧策略、执行成本对标 |
| 826 | 新r1，drafting 0/8 | 新版本紧凑wire、section/body边界、密度/体积与装载测量 |

798初次文档修订按受影响T-01/03/04从4/10降至1/10，旧证据保留。期间另一会话追加r2 work修复候选5def3af48，恢复不受V-14直接影响的T-02/05及原T-06，当前r3计3/10。该work记录声称关闭R10finding，但本轮未独立验证其实现；不能把r2“8/10/无work遗留”当r3验收通过，T-07/08还须覆盖V-14。进度为该协调时快照，后续以计划唯一任务表/新证据为准。

## 2. ABC-A01..11逐项责任

| 建议 | 采纳规则 | owner / 验收 |
|---|---|---|
| A01 | kind不是函数数；entry/imports/可执行条件贯穿encode/decode/公开手工输入/driver/cache/restart；版本/键/歧义旧产物同步 | 798 r3 §5.7、V-14，T-01/03/04/07/08，AC-01/03/04/05/07；SD-02/03 |
| A02 | primitive规范化政策唯一，签名含模式/返回/效果，record/enum保持owner/revision名义身份；同布局不等价 | 820 r2 §5.4/V-11；821–824继承 |
| A03 | 三层表示，私有执行层仅受检派生，禁止第二语义IR/portable宿主地址；允许安全参考适配 | Design43 §12；820接口合同；825实现及V-01/06/07 |
| A04 | 每步PC定位固定上界；装载O(B+I)、前置块扫描0，诊断不反复求前缀和 | 825 §5.2，V-02，T-03，AC-02 |
| A05 | 参数Vec/帧/typed_zero分开测；至少选一种安全策略并冻结可观测改善计数，不能让未初始化/复用破坏owner | 825 §5.2，V-03/05，T-01/04；821等保持生命周期兼容 |
| A06 | 比较ULEB与窄+Wide后选一个新wire，body密度与总长/loaded分列；不暗改旧版本 | 826 §5.1/3，V-02/03/08，AC-02/06 |
| A07 | section/body长度或目录、语义/debug明确分类和未知项政策；已有spanless code+sidecar保持；不强制懒授信 | 826 §5.2，V-04/05/06 |
| A08 | 非Copy值不能复制责任；move-kill/模式/借用/清理贯穿指令和所有出口，必要生命周期数据不能strip | 821 §5.4/V-11主责；820基础，822–824逐profile |
| A09 | 实际MovePath+前缀稀疏intern、验证工作量上限，数据量计费及cleanup预留；requires不等于host grants | 821–824 §5.4/V-11及既有资源/恢复矩阵；825/826继承 |
| A10 | 单一指令表列typing/operand/read/def/kill/借用/effect/trap/cleanup/fuel/profile/source，逐层覆盖检查 | 820建表，821–824扩展；SD-01/02；825/826消费 |
| A11 | 语义先对齐、按解释/编译tier分组，分阶段/计数/真实峰值/噪声；code/metadata/debug/封装/raw/压缩/loaded分列 | 825 §5.4/V-08..10，826 §5.4/V-08..10；数据扩面事件成本由各profile反例记录，完整owned性能专项另案 |

A02是扩面前统一合同，不是已证明798可混淆类型的漏洞；A04是确定复杂度开销，按性能后置。A01目前源于静态代码反例，work必须先用实际路径复现；若当前版本已修复须给当前提交与全路径反控证据，不能仅删除新增验收。R10前门合法输入误拒仍归798原正确性，其候选独立复审不被本轮替代。

## 3. 不追加的隐含实现

累加器ISA、superinstruction、完整slot寄存器分配、选择性常量池/短立即数、调用窗口若不在T-01选中则记录测量与未采纳原因，不冒称交付；不以其缺失阻塞798或820功能。NaN-boxing/unsafe union、shared/GC、JIT/OSR/deopt、稳定公开ABC、完整native新数据与CTE复合结果接线仍另案。Bool比较应在820指令表明确语义，是否装载专门化由测量选择，禁止让I32命名掩盖未来类型扩张。

## 4. 排期与协作

功能链D798→820→821→822→823→824保持。可选性能链D798→825→826，不成为功能硬依赖。D表示真实独立review、冻结门禁、合入/Spec/收据齐全。两个链修改同一共享核心，选择单写窗口串行交接；只读调研/输入准备可并行，不需要并发实施或新代理。817/818/814–816实际owner未交接时不抢写其消费接口/注册。

825/826由中央分配器在互斥与独占allocation.lock中取号；active/archive唯一，本批结束.next-id=827，保留原空锁文件，未建实施树。

## 5. 文档预检与证据

预检对象：Design43/44、索引、798 r3、820–824 r2、825/826 r1及本附件。核对编号节、status/revision/任务计数、AC/V/SD唯一性与责任、依赖无环、local links、Markdown fence、行尾空白及范围diff。结果见[document-checks](document-checks.json.txt)。

新增合同与链接检查通过；798旧审查历史有11处附件链接在当前主检出缺失，列入收据的legacy_missing_evidence_links并保留原文，不宣称历史附件全部可读或据此重授实现pass。

输入版本与SHA见[inputs](inputs.json.txt)。历史r1 planning-inputs保留内容相同的.json.txt副本并改引用，避免被仓库*.json忽略；r1 preflight标为历史，不覆盖新合同。

本轮未改Rust/测试/依赖/产品源码或canonical Spec，未运行cargo/docs_gen、VM反例或WASM/V8基准；未提交、合入、归档或清树。其他会话并发变更及其测试成果不属于本轮实现证据。
