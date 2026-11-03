# ACE 927–929规划复核、近期状态与执行安排（2026-11-03）

状态：规划合同复核，非实施验收。用户授权直接修订927–929的遗漏并列近期计划，只做规划，不施工/建树/构建测试/派发。取材main `f2214a65f`（较初订`ec3e70414`只新增他方AG文档提交）；源码事实与M1范围仍以[机制审计](ace-m1-architecture-gap-audit-2026-11-03.md)、Spec及实际入口为准。不改923–926进度或合同，不重开归档计划，不取新号。

## 1. 复核结论与修订

三包职责与顺序保留，927/928/929均修订为**drafting r2**，进度分别0/12、0/12、0/14。r1已覆盖主体方向，但以下可被错误实现绕过的边界不够具体，本轮直接补入§5–8的现行合同及新增V例。这里的“修订完成”只表示文档已闭合，不表示实现通过。

| finding | 原合同不足/实码依据 | 修订与关闭oracle（全部待work执行） |
|---|---|---|
| PR-01 / P1 统一入口与片段授权 | 统一“函数/方法/lambda片段”未精确定义词法上下文。现手写fragment从range取子串；通用scanner checkpoint包含mode/mid/原始起始行，不能从错误上下文按默认mode开始 | 927新增入口请求/输出合同、孤立兼容解析与会话授权分离，方法不伪造free fn；V-08伪片段藏在字符串/注释、半token/CRLF/错owner/旧revision与合法近邻。929复核所有driver/profile升级尝试序及入口身份，不由backend重解语法。 |
| PR-02 / P1 资源态不能被兼容API抹成语法错 | 旧`parse_*`返回`Diagnostic`，通用`RtError::ResourceExhausted`必须穿过新入口。默认`Budget.work_bytes=256MiB`也不能当正式64MiB档或16MiB前端授权 | 927新typed outcome保source/resource/stale/infra分类，旧API只作兼容投影，正式consumer不得再由message反解；显式预算传递/原子预留，V-09缩预算与同源合法对照；原code/stage拒绝事实保持。 |
| PR-03 / P1 闭包身份与子体复用 | “同文本lambda复用”不足：同文异owner/不同capture站点、嵌套插入/重排可能错配。旧AST是Box结构，共用外包装不证明未深clone | 928新增逻辑单元/当前创建点与版本绑定、歧义重铸或具名非快路径、真实底层payload身份及生产计数；V-09同文双lambda/异owner/capture/layout变更；保持语法复用与当前语义复用裁决分离。 |
| PR-04 / P1 候选与最后owner | 候选共享旧节点时，失败/取消/close与外部pin的生命周期、临时峰值未落实到具体反例 | 928 V-10：current/history/候选/pin多持有，失败修复/旧handle仅在仍pin住的原snapshot中读/最后owner回收；复用允许Rc/Arc等价句柄，不强制跨线程。929同源安装/迟到/发布失败必须保唯一提交点，不以删除历史列表冒释放。 |
| PR-05 / P1 恢复后首编辑不能预热 | D921 `ensure_env`直接调用整源parse+resolve，虽记M也是真实前端工作；现有V-04未禁止测试在计量前先ensure_env/执行dummy edit | 929新增首操作序列：新进程restore→立刻真实编辑（含首编辑失败→修复），中间不允许预热/重建；全部实际producer attempt含早退计数，跨R/P/M搬家不豁免。V-12/13分清恢复deep-verify与源生产、暖未改artifact I/O，合法R读取不伪称零I/O。 |
| PR-06 / P1 预算适用性与统计 | 1MiB/99.99%/48KiB及A2X48MiB是各自原负载/职责门，不能变成任意profile/任意巨声明编辑统一比例；多个对象同时存活的计费也须防授权复用 | 三包增加适用预算表/观察窗口：原固定样本及其门一项不减；复杂新增代表例按工作集/影响集/真实读取另验，不以不适用减原样本。current+候选+历史/pin+恢复临时同时纳真实峰值；同一对象一次总账但所有适用域各核其占用；缩预算/漏计/采样缺失反控。 |
| PR-07 / P1 来源发布与恢复版本 | 新语法/CST/目录/context有多个产物，原文字没有显式封死“半新语法根+旧上下文”可恢复态；旧锚缺语法身份也不能当新格式P0 | 929 V-14版本隔离/required闭包/发布seam故障，恢复只安装同一次提交绑定的源/语法/声明/IR/目标集合；错版本具名拒绝或另选显式cold，禁止隐式repair重算冒restart。 |
| PR-08 / P2 写面交接不能只有一条S/H原则 | 923/925实际有WIP；924会接闭包registry，925/926会接method/type-enum/恢复，可能与语法计划撞同一文件 | 三包加具体交接矩阵与释放条件、私有准备/公共实现/最终验收三类前置，默认原owner当前符号窗先收口，再927→928→929；H全文件单写，接口变化按真实输入标STALE并核全部reader/construct/match。无923–926整波语义前置，也不准未交付适用腿冒绿。 |

以上均为原M1目标内合同补实，未新增语法/捕获/ABI语义，未放宽预算或门限。r1记录保留§9作为历史；只保一份现行§5–8与任务行，不另起竞争清单。新V例的使能任务/AC绑定在各计划中。

## 2. 当前未交付计划：完整清单与安排

此处“近期ACE”指908–929主线及其真实支撑计划；活跃区核对后，尚未交付为以下七包。没有把旧891/893等历史状态复活为待办。

| Plan | 职责 | 文档状态/进度 | 实际证据/下一步 |
|---|---|---|---|
| [923](../923-native-allocation-fault-and-place-conformance.md) | native fault/Place，独占R03，交R02非闭包基础 | executing r1，3/12 | lang-923 @ `ca9bcafb9`，collections-runtime/native worker/layout/lower/observe/tests存在WIP；继续原范围，不消费WIP当D923 |
| [924](../924-native-closures-source-and-resource-closure.md) | 有限native闭包，R05与完整R02 | drafting r1，0/14 | 未见专属树；消费923稳定fault/runtime接缝后执行公共N，私有准备可先；与928/929的closure S符号取窗 |
| [925](../925-a2c-record-text-resource-closure.md) | C record/text/方法/mode/资源 | 头drafting r1、2/14；**实际开发中** | lang-925 @ `ec3e70414`，私有C `lower_rt.rs`/`rt_runtime.rs`/`seed.rs`未跟踪WIP；状态头滞后，不代owner改状态/勾选 |
| [926](../926-a2c-collections-variants-resource-closure.md) | C集合/变体/owned资源/认证 | drafting r1，0/14 | 未见专属树；消费925稳定资源/会话接缝，P研究可提前，公共A2X串行释放 |
| [927](../927-unified-source-syntax-foundation.md) | 七profile统一语法基础、typed结果/诊断/预算、冷前门 | drafting **r2**，0/12 | 未开工；下一步T-01/T-02私有盘点/可行性，公共source/AST/manifest取窗后实施 |
| [928](../928-unified-source-collections-closures-syntax.md) | collections/closures、嵌套单元身份/子体复用 | drafting **r2**，0/12 | 未开工；私有case/身份研究可准备，公共grammar/core投影须927稳定交接 |
| [929](../929-unified-source-entry-incremental-retirement.md) | 全入口冷暖恢复、首编辑、重复parser退役与SYN准入 | drafting **r2**，0/14 | 未开工；调用清单/父侧schema可准备，公共迁移消费927/928及真实native/C稳定接口；最终适用认证不能省 |

925的实际WIP和923的头进度均为读取时快照，不是本会话启动施工或认证；后续开工/接窗必须重读当前HEAD/WIP。927–929本轮仍无实施树、无work完成项。

## 3. 已交付的近期基线

下面交付提交均由归档计划收据定位，并在本轮只读核对为当前main祖先；不是重跑旧测试。阶段/跨仓支撑与ACE能力分开列明。

| Plan | 内容 | 状态 | 语言仓交付基线 |
|---|---|---|---|
| 908 | M1规则/适用性/严格登记 | archived / delivered | `dc8eca3ae` |
| 909 | 共享可变捕获C-19/K06 | archived / delivered | `59b8107eb`，增量残余后由912/913接续 |
| 910 | 生成Cargo构建治理（跨路线支撑） | archived / delivered | `f665ce113`，跨仓及具名残余仍按原owner |
| 911 | native诊断事实/B17–19准入 | archived / delivered | `ca4848f5d` |
| 912 | 闭包全链函数级增量 | archived / delivered | `13a465336` |
| 913 | 闭包源码绑定恢复与首编辑 | archived / delivered | `04f487b4b` |
| 914 | 求值序/layout场景观察 | archived / delivered | `6ffbaf49e` |
| 915 | AV资源fault/陈旧Place | archived / delivered | `5ba5d4cd2` |
| 916 | 旧A2R语义准入 | archived / delivered | `edfdabc2e` |
| 917 | 旧A2R mut record/text发射证据 | archived / delivered | `9ae8df983` |
| 918 | 跨仓交付/边界准入（支撑） | archived / delivered | `ca9bcafb9` |
| 919 | AC/AV资源事件六格 | archived / delivered | `4ea3daddf` |
| 920 | 旧AVM B18一格 | archived / delivered | `06a6f8c72` |
| 921 | C scalar第二目标/来源/认证 | archived / delivered | `3c760865c` |
| 922 | M2替换场景与关键路径盘点 | archived / delivered | `d66fdef04`，归档文档链至`593c33480` |

914/917/921归档头计数分别6/10、6/10、13/14与终态不完全一致，但真实merge/交付/清理收据存在；作为簿记差异保留，不据此重开或误报未交付。897等非本轮ACE包或其他图形线活跃文件不混入本表。

## 4. 执行安排与并行边界

```text
native N：   923 ──稳定fault/Place接口──→ 924
A2X P：      925 ──稳定资源/会话接口────→ 926
语法 S：     927 ──基础语法/片段/预算───→ 928 ──单元/复用接口──→ 929
最终SYN：    929适用native/C认证 ← 实际924/925/926能力与稳定接口
最终M1：     原门 + C门 + SYN门 + ARCH各真实机制/组合门
```

三条线主体可分窗并行，不是七包按编号串行；也不是三个团队可以同时写公共core/AST/session/manifest。下一阶段可准备927私有语法/反例与928/929私有调查；当前923/925继续各自已授权工作，924/926在其稳定接口就绪后接续。

| 共享位置 | 原owner与新包职责 | 默认交接/释放条件 |
|---|---|---|
| core source AST/Profile/parser/cst/diagnostic，syntax-core重导出 | 927先交基础API；925/926仅其原必要消费/事实出口，924仅其闭包消费 | 已登记符号若有原owner WIP，先交可验证稳定SHA再转927；未占用符号可先领。927→928→929逐符号释放，不把整文件永久预占；任何类型变化核全部construct/match/reader |
| core closures_fn/context、registry/verify/共享lowering接缝 | 924资源/原闭包语义，928语法单元/保留子体，929会话/恢复 | 924与928同符号串行，语法事实/受检语义分层；稳定接口可提前交，不要求D924全包后才能研究；最终closure运行腿待其真实能力 |
| A2xSession/type-enum/method片段/source-origin/manifest | 925→926先交各自目标/资源原合同，929统一语法与恢复首编辑 | 若C包已修相同ensure_env接线，929复用；未修则929负责通用迁移，不重复实现。consumer和来源版本联合交接；未稳定时仅私有准备 |
| native collections/closure driver及runtime N | 923→924保原执行/故障/清理语义，929只迁源码入口 | 不由语法包重写allocator/Place/env；接线后原fault/typed/cleanup反例与新源码身份重验 |
| 公共H：driver/SDK/父decoder/注册/固定组/manifest-lock/规则来源 | 原owner释放后各包按真实必要改动接续 | 全文件单写，记录owner/revision/base+handoff SHA/schema/完整producer-consumer/有效-STALE-pending/验证与释放。正式采样错峰，不持窗等慢认证 |

这是规划调度约定，不是本轮替其他会话授予实际锁或发送交接消息。各次开工在计划§5表中填实际owner与SHA；未落实的冲突写面不能开写。

## 5. 尚未立项的后继职责

| 职责 | 准备/实施安排 | M1关系 |
|---|---|---|
| M1-SQ/INST/CT：普通源码语义查询、有限真实实例、CT冻结事实接线 | 可先只读盘点/设计，与语法私有研究并行；先核796/817实际消费者与有限代表例，再决定合包/拆包，不预占号 | 必需机制缺口，不能改称M2语法覆盖而延期；不要求一次完整泛型/meta语法 |
| CA-D有限ABI桥 | A/B稳定接口就绪即可规划并与926分窗；复用797，区分内部/目标/公共ABI，不等待未来全生态 | M1 C新增必需门，尚无正式Plan |
| CA-E/M1架构组合验收 | 规则/schema/证据owner准备可早做，最终等待原门+C+SYN+ARCH全部所需机制真实交付 | 尚无正式Plan；不能由929只签SYN就宣布整个M1通过 |
| M2-EXEC/HOST/SESSION/TEST、其余在用目标与应用/安装发行/disable | 按922盘点另定有限包，与M1独立写面可规划，不在本轮自动编号 | v0.6 M2全替换仍硬门，M1完成不自动满足产品替换 |

## 6. 本轮核验记录

只做计划frontmatter/任务-AC-V映射/本地链接/固定组名/ID与进度保留/交付祖先/文档空白检查；新增V例尚未执行。全部语义/性能/资源断言仍待work及独立review取证。未创建worktree/材料scope，`.next-id`保持930，原923–926文件不改。
