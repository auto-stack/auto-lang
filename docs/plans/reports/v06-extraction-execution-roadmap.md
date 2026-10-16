# v0.6 正式拆分与搬迁执行顺序

2026-10-15；基于 Design 37、已交付777–780、当前781/783合同及代码盘点。此文是目标路线，不代表后续改动已实施；只为近期三项分配正式计划号，远期细节随前置成果展开。

## 当前切换点

**2026-10-16 更新（789–791合流后）**：781、784、785、786、789、790、791均已合并归档；792前置已满足，可独立开工。新增793/794/795为可与792并行的三份drafting合同。下方早期计数与“准备状态”为历史规划快照，不作为今日放行依据；783/787/788按实际共享写面协调，不设为全局等待条件。

## 与792并行的三个实施计划

| Plan | 独立交付与主写面 | 与792及彼此的边界 |
|---|---|---|
| [793 诊断基础抽取](../793-diagnostics-foundation-extraction.md) | 新auto-diagnostics、error.rs薄兼容出口；切断error→AST依赖 | 不改rust_ui/ui_gen、native/scroll或Man锁。旧错误类型身份和路径保持，不要求792改调用协议 |
| [794 scroll宿主迁移](../794-scroll-host-provider.md) | native八函数9900–9905/9922–9923、owned provider及UI adapter | 不改诊断/包锁/生成工程发行；完整继承781锚点状态与真实可见性oracle |
| [795 包锁核心](../795-package-lock-core-extraction.md) | 新auto-package-lock、lock.rs轻门面与Target/Git adapter | pac.lock与792的Cargo.lock不同；不接管rust_ui或依赖版本/patch决策 |

四条线均不等待其余新计划整体完成。各自专用组树可并行实现、定向测试；新增crate/public facade只按兼容方式接入，不借并行重写公共合同。共享写面单独排短窗口：

| 共用区域 | 所有权/合流规则 |
|---|---|
| 根Cargo.toml | 792拥有依赖版本/source/patch决策；793/795各自仅加入新member/精确本地依赖。修改及合流短窗口串行，不批量重排或升级其他依赖 |
| auto-lang Cargo/lib/统一宿主安装、auto-man Cargo/lib | 分别限定诊断re-export、scroll安装、新包锁依赖片段；扩大区域先协调，不整文件覆盖 |
| 777配置/基线及Specs索引 | 各自加精确domain/edge，合流后取并集并重新核对；不能拣一份基线覆盖另一线 |
| 792 source/resource清单与锁 | 792冻结当前受管闭包；若793/794/795先合流，792必须同步并重验fresh/locked及重定位。若792先合流，后续改变闭包者按792真实接口更新交付清单并定向重验；不能留硬编码allowlist漏掉新crate |
| 重型门禁 | work只做规定的定向验收；最终review后merge门串行安排，tf仍主检出单实例 |

规划与实施范围见[本批指纹](../attachments/parallel-extraction-20261016/inputs.json)。本轮仅建立合同，不创建agents/工作树或执行Rust。下一批完整AST/parser和UI文法外置在793等切口上继续拆解；Man核心/tool协议、其余宿主族及G/H搬仓仍分别验收。

## 下一批实施计划（2026-10-16）

| Plan | 实际交付 | 依赖与编排 |
|---|---|---|
| [789 词法基础抽取](../archive/789-lexical-foundation-extraction.md) | 新 auto-lex 独立 crate；生产 Parser 消费单一 Token/Lexer 实现，兼容错误/位置/桥面；完整前端剩余环登记 | 已合并归档8/8 |
| [790 文件/图像剪贴板](../archive/790-clipboard-files-images-provider.md) | 2934..2937 宿主 provider、真实 no-ui VM 消费及 Windows OS 接线；引用与临时图像路径寿命验收 | 已合并归档8/8 |
| [791 生成核心抽取](../archive/791-generation-core-extraction.md) | 新 auto-gen-core；guard/输出编排单一实现，Auto解释执行留 adapter，原CLI/legacy保持 | 已合并归档8/8 |
| [792 生成UI独立构建](../792-generated-ui-independent-build.md) | 修复 fresh-lock UI消费；显式可重定位交付根、根patch/应用锁/source资源身份与真正外部构建 | 789/790/791 final review+merge后冻结闭包并实施；此前仅可只读准备 |

本批是实际仓内解耦与发行地基，不再只做探针；每项都有物理删边/单一实现或独立构建验收。四份合同不代表本轮已获实施、发布或旧仓主分支切换授权。取材与验证见 [本批输入指纹](../attachments/extraction-batch-20261016/inputs.json)。

本批结束后仍需落实 A 的 AST/type-store/inference/完整parser DAG切口、B完整UI文法、C其余scroll/editor/term/media宿主族、D UI分层、E剩余Man核心/tool协议与薄CLI；792只证明选定Iced生成应用发行面，不覆盖所有后端/资源。满足D/F等真实退出条件后才进入G/H整仓搬迁；I/J兼容退出和净瘦身继续保留。后续按已交付代码再建中等计划，不预占空壳编号。

777基线、778外部Dialect、779 frame provider、780 project-model均已合并归档。779落地并不证明字符串/媒体寿命、全部文法、完整独立仓发行已解决。

781当前合同仍executing（18/26），涉及UI/VM/UI生成及原生验收；它的早期合并历史不能替代当前修复收口。783仍executing（7/9），范围限定experimental/ac-artifact-probe及资料，不是UI/Man搬迁硬前置。实施前重核状态和真实diff，不凭这些计数永久放行。

## 本轮正式建立的三个计划

| 顺序 | Plan | 交付 | 入场条件 |
|---|---|---|---|
| 1 | [784 旧仓保全](../archive/784-repo-extraction-legacy-preservation.md) | legacy恢复锚、未提交载荷保全、隔离准备树及刷新迁出owner清单 | 已合并归档；原入场条件保留在归档合同 |
| 2 | [785 文本剪贴板能力](../archive/785-clipboard-text-host-provider.md) | 按779模式真迁2925/2926实现依赖，覆盖字符串ownership与feature错误 | 已合并归档 |
| 3 | [786 展示字段读取](../archive/786-project-display-reader-consolidation.md) | 780模型用于Man首批生产读取，重复扫描收口、明确兼容策略 | 已合并归档 |

计划号是推荐编排顺序，不把无依赖的工作强制串行；并行不意味着整档测试并发。只建drafting合同，不在本次创建树、改代码、移动旧仓refs或改主分支。

## 其余工作包及依赖顺序（尚未分配Plan号）

| 波次 | 工作包 | 必须证明的退出条件 |
|---|---|---|
| A | 前端/来源/兼容AST依赖切口；UI parser注册与宿主factory方案 | 真实crate DAG无环，parser所需type store/inference有归属；不靠公开全部Parser或复制grammar；解决core与外部UI parser互依赖后才外置实现 |
| B | UI文法按族迁移（先有限声明族，再widget/view/on等） | 完整现有语法、import/递归/恢复/诊断可用，Core view参数/task on等对照不变；778 model pilot不是完整替代品 |
| C | 其余宿主能力按族迁移 | clipboard文件/图像、scroll/editor、term/media分别验ID/encoding/ownership/线程/退出；必须包含781新增scroll族，不因早期清单漏掉它们 |
| D | UI内部crate分层与SDK门面切换 | 模型/分析生成/运行桥/renderer职责清晰，core选定生产闭包无反向实现依赖；全链消费者使用显式宿主装配 |
| E | Man管理核心、构建adapter、UI tool协议和薄CLI | 包/锁核心不拉完整VM/renderer，动态配置和可执行模板由显式上层adapter承担；命令路由/取消/错误保持 |
| F | 生成runtime、资源bundle与发行依赖身份 | 生成工程不绑临时tree；schema/模板/stdlib版本与producer/cache身份完整；独立manifest/lock/根patch真实可建 |
| G | auto-ui正式搬迁并交付 | 781当前合同已落地；D/F完成；旧仓迁移分支承接来源代码/资源/历史映射，外部无源checkout消费通过后才切主分支 |
| H | auto-man正式搬迁并三仓集成 | E/F/G完成，包管理/CLI/语言与UI工具组合验收；依赖来源无重复类型，旧仓主分支按已验收新提交推进 |
| I | UI专属AST与compatibility退出 | 独立UiAst/lowering完整覆盖已有UI/Core合同；core基础AST不再拥有UI专属声明；旧facade有可验退出条件 |
| J | 语言内部model/VM/codegen/trans及数据/runtime域深化、去重退役 | 不把实验AC/HIR当全语言IR；按consumer合同拆模块，净删除/剩余兼容依赖有实际清单，数据/runtime是否独立仓另以可消费边界验收 |

A/B可与不重叠的C或Man E准备交错；D要求相关grammar/native切口成立。F须早期设计，不能到G/H搬完目录后才补锁/资源。I/J属于干净终态，不能在G/H首次独立交付后默默取消。

此前“12～16个实施计划”仍是预算估算，不是必须压缩到该数量的验收上限。工作包C、E、I/J如涉及不同ownership/发布合同，应拆成独立中等Plan；只在前置路径、公共合同、真实oracle明确后取号。远期不创建空壳Plan或预占编号。

## 两个明确里程碑

1. **首次三仓独立交付**：仓界/真实构建/外部消费/资源与命令保持；可以仍有登记的兼容UI AST。需明确“实现可交付”与“纯core依赖闭包”分别达到了什么。
2. **领域完全解耦与瘦身**：UI专属AST退出core、残留宿主反向边/兼容入口按合同退役，重复实现净减少；不是简单把Rust行数挪到旁边。

两者分别有计划与验收；中途每批可独立合入，不要求全工程停工。

## 搬仓前的共同硬门

- 相关实现已review并合入；781当前修复全收口，其他在途计划只按实际重叠要求合流/保全。
- 源码、生成物、模板/schema/stdlib、测试、示例、脚本、文档、CI及构建脚本都有归属与来源映射；不能只复制src。
- 独立消费不依赖主检出/临时worktree；完整依赖source identity、应用/工具锁及顶层patch通过实际构建。779遗留的独立UI锁解析问题在此必须实证闭合或明确支持合同，不能沿用rlib链接证据宣称发行成功。
- legacy refs与未提交载荷已保全。主分支在验收前维持旧版本，不reset/orphan/force-push；新提交更新代码树，保留旧tag和来源历史。默认分支更名/远端push单独明确授权。
- 结构搬运与语义重构分Plan；禁止借搬仓去更新金样掩盖漂移。失败时恢复旧入口与旧产物的方法可执行。
- 每Plan同组唯一worktree，无链接；最终review后才跑冻结提交merge-owned门禁，批量tf仍主检出单实例；不与783的探针误混成生产替换。

## 准备状态

取材见[输入指纹](../attachments/modularization-planning-20261015/inputs.md)。旧auto-ui/auto-man本地tip和WIP已只读确认，未fetch、未改变refs。三份新计划0步完成，待各合同确认后work；本轮不实施、不发布canonical Spec/ledger。
