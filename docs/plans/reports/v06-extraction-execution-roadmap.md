# v0.6 正式拆分与搬迁执行顺序

2026-10-15；基于 Design 37、已交付777–780、当前781/783合同及代码盘点。此文是目标路线，不代表后续改动已实施；只为近期三项分配正式计划号，远期细节随前置成果展开。

## 当前切换点

777基线、778外部Dialect、779 frame provider、780 project-model均已合并归档。779落地并不证明字符串/媒体寿命、全部文法、完整独立仓发行已解决。

781当前合同仍executing（18/26），涉及UI/VM/UI生成及原生验收；它的早期合并历史不能替代当前修复收口。783仍executing（7/9），范围限定experimental/ac-artifact-probe及资料，不是UI/Man搬迁硬前置。实施前重核状态和真实diff，不凭这些计数永久放行。

## 本轮正式建立的三个计划

| 顺序 | Plan | 交付 | 入场条件 |
|---|---|---|---|
| 1 | [784 旧仓保全](../784-repo-extraction-legacy-preservation.md) | legacy恢复锚、未提交载荷保全、隔离准备树及刷新迁出owner清单 | 可先确认执行；不需等781/783结束 |
| 2 | [785 文本剪贴板能力](../785-clipboard-text-host-provider.md) | 按779模式真迁2925/2926实现依赖，覆盖字符串ownership与feature错误 | 784完成；781当前修复review+merge且宿主热点释放后接生产代码 |
| 3 | [786 展示字段读取](../786-project-display-reader-consolidation.md) | 780模型用于Man首批生产读取，重复扫描收口、明确兼容策略 | 784完成；不依赖785或781，可在无同文件写入时独立推进 |

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
