# AV语言能力矩阵与源码替换路线：起草预检

日期：2026-10-20，Asia/Shanghai。范围：Design43/44衔接、两份strategy专题、23个程序目录、827草案及导航。用户明确先不写代码，当前仅规划文档。

## 1. 已交付的文档

- [43项能力矩阵](../../../design/strategy/av-language-capability-matrix.md)：源码、MIR/ABC/AV、stdlib/产品状态分列；“限/T/无”是当前计划完成后的条件预期，不是本轮运行PASS。
- [G0–G8路线](../../../design/strategy/av-source-programs-and-avm-replacement-roadmap.md)：优先级、R2-*..R8工作包、10类入口、每阶段Q门、显式回滚/会话边界与最终退役。
- [23程序种子](../../reports/av-source-program-catalog-2026-10-20.md)：真实现存文件/相邻expected、依赖和需补oracle；AP-18等旧bug见证不当新规范。
- [827 r1](../../827-av-source-program-baseline.md)：drafting0/8，G0证据与G1现有scalar源文件实际执行/恢复/父侧judge；8任务/8 AC/12 V/5 SD，全部未执行。
- Design43新增§13、44新增§17、索引和版本roadmap只做衔接，不改798/817/818/820–826的合同或当前进度。

## 2. 已消歧的风险

| 风险 | 本轮处理 | 后续责任 |
|---|---|---|
| fixture当全语言、已声明stdlib当真实执行 | source/内核/库/产品六层证据分开，43类别与23种子不是最终替换分母 | 827库存；各R包真实source/actual manifest/入口 |
| 新旧语义不同却要求输出相等 | SEM-01..08列wrapping/trap、负索引、str字段、捕获/alias、入口/错误、旧误报 | 先材料后具名profile/确认决定，不改原798/821合同 |
| G1等待全部824/826，持续没有实际源码 | 827构建只等D798，不反向阻798；数据source随相关D821/822/823推进 | 最早拓扑798→827→数据链可行 |
| HOF要模块、模块又等待全部HOF | R3-M纯函数模块窄切片从G1可先交付；G2b消费切片，完整G3后汇总 | 取号时按功能切片展开依赖，不按阶段号全序 |
| 为source入口再造一套worker/VM | 827 T-01先核D798现有API/driver/recipe；足够就只复用/登记，确有缺口才薄适配 | 不复制解释器、checker、codec或协议decoder |
| 缺expected、缺Python/端口占用被跳过 | 明记gap/infra；缺工具不能成必需case PASS；无oracle先补强，不复制actual作expected | 按保留SKU各自收口，不能删未来分母 |
| AP-15无限循环/8080和AP-10文件副作用 | 专属scratch/端口环境及有界owned进程；改源另记变体，强杀不证明Drop | G4/G5及typed host owner |
| 改默认等于退役或隐式fallback | G7分SKU，G8核全保留入口/依赖/旧dispatch/冷安装与旧工件策略 | 回滚显式；旧会话drain/重启或独立状态迁移 |
| 实验8MiB/128MiB机械套所有应用或随意抬cap | 原profile预算不变；新product profile在其Plan中冻结合理独立预算与版本 | 未达不得记债后PASS |
| 自举或JIT成为全VM替换的无限前置 | compiler-source demand独立RB；已有保留消费者必须交付，新JIT/OSR无自动硬门 | 不用旧AAVM展示冒领新AV/AAC实证 |

## 3. 静态校验

校验对象包含本轮正文、827 frontmatter/编号节/owner_stage/任务AC/V/SD、C/SEM/AP/EP唯一编号、Markdown本地链接/fence/table、取号active/archive唯一性与.next-id、实际源码/旧oracle SHA和依赖图。

结果见[document-checks.json.txt](document-checks.json.txt)：当前新增合同检查通过。依赖图验证了最早source排期与性能先行排期；注入798反向等827或G2b等完整G3的循环均能被检测。文档检查不是运行时资源/性能/安全验收。

相关导航修复：version roadmap的770/771/774链接指向实际archive位置；索引表内pipe转义/旧两列表行修正，战略文档数量不再伪装精确人工计数。没有改动这些归档计划或其历史证据。

## 4. 证据及交接

[输入收据](inputs.json.txt)含主检出/Spec/相关Plan/旧engine与发现式VM test框架SHA，[程序收据](program-inputs.json.txt)含23源文件SHA与原oracle内容，[输出身份](output-identities.json.txt)冻结本轮正文。

取号：中央scripts/new-plan.sh在互斥和独占allocation.lock下分配827；active/archive无同号覆盖，.next-id从827到828，空锁文件保留。本轮不创建实施worktree/junction，不提交/合入/归档、不发消息或创建agent。

stage:new / outcome:pass只表示起草质量通过，不表示work授权或依赖交付；实施handoff等待明确执行授权与D798真交付，T-01/02仅可在已授权的静态准备范围内进行。用户当前只要求文档，无本轮必须回答的审批问题。canonical Spec/ledger、Rust/测试/依赖和产品默认均未改；cargo/docs_gen/VM/AutoDown/native/WASM/V8运行均未执行。本轮不宣称任何新程序已通过或完整语言已覆盖。
