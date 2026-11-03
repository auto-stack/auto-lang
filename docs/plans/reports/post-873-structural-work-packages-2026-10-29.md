# 873 之后的两个结构工作包：范围与顺序

> **2026-10-30当前立约更新（优先于下方全部旧快照）**：第二结构包已落为主责任UI仓的[auto-ui/014](../../../../auto-ui/docs/plans/014-ui-man-structure-closure.md)，drafting r1、0/11；本轮只规划，全部实施含建树/探针等U890及后续明确014实施授权。890收尾读回executing0/12，其合同记录用户对885历史证据缺口的处置，尚非完整交付。顺序U890→U014→按UI007原门推进；014只收CLI/Vue/Aura/renderer既有结构，不重搬Web算法，不迁整棵examples或实现新主题。UI007独占这些示例/主题目标，不是014的反向前置。详[UI结构设计](../../../../auto-ui/docs/design/autoui/ui-man-structure-closure.md)与[路线v19](v06-extraction-execution-roadmap.md)，语言不再取同包编号。

> **890规划更新（覆盖下方旧时点排程）**：884已归档10/10、887已归档r2/10/10并清理；887机制级性能负结果不等已加速。[890 Web完整搬迁](../890-auto-web-independent-repository-delivery.md)已立drafting r1、0/12，主线为已交付884/887→890→仍未编号UI/Man结构；888质量支线按实际X/M交接，不设U888全局门。Design50 r2和[路线v18](v06-extraction-execution-roadmap.md)为当前入口；旧“Web未编号/待887后取号”仅历史。本轮仅规划，不实施/建仓/建树/agent，不改他方合同或产品。

> 2026-10-29，用户授权：读取 873 最新结果，明确后续两个工作包，暂不实施。
> 本文是规划决策与后续立约输入，不是编号 Plan、实现验收或新的 canonical Spec。
> 本轮不取号、不建实施树、不改产品/默认特性/依赖/测试、不运行 Cargo 或 GUI，不创建 agent。
> 当前调度入口：[总路线](v06-extraction-execution-roadmap.md)；历史候选：[869 结构材料](../attachments/869/structural-packages.md)。后者保留原文，本文说明其当前取舍与修正。
> 同日最新排程更新（优先于初订时点）：884已有执行授权与实施提交，保持其原合同收口；[887测试治理](../887-test-governance-and-ace-first-corpus.md)已立约，实施等U884。用户确认后续依次为**884→887→auto-web有限迁移→UI/Man结构包**；[Design50](../../design/50-autoweb-service-framework-and-repo-boundaries.md)明确Web仓界和交接，887结束后再取迁移Plan号。Web/UI-Man均未编号，本轮仅设计，不建仓/树/agent、不实施或修改在途Plan。

## 1. 已交付依据与测量结论

[873 r3](../archive/873-build-cost-and-member-split-decision.md) 已归档 8/8：交付 `54f4ae6e9`，独立终审修复后通过，ledger v127，四树 guard clean 后清理，终收据 `643d58d5b`。U869/U872/U873 均成立。

测量权威输入为 [build-cost-baseline](../../specs/repository-boundaries/build-cost-baseline.md)；measurement pin 为 lang `a748017da`、UI `4adf2d03`、Man `e6cc06f`、down `c3464d0`。以下数值均绑定该组合、Windows 单机及记录的工具链/默认 feature，不是未来版本或 Linux 结果。

| 实测面 | 语言 | UI | Man | 含义与限制 |
|---|---|---|---|---|
| 真实函数体编辑后 check | 6.9s（4 站点） | 4.6s（compiler/runtime） | 1.2s（CLI 叶子） | 语言会重查 core、根包及 UI 两包，不能只用文件大小解释等待 |
| 温态编辑后测试二进制准备 | 27.4s | 22.9s | 4.9s | UI/Man 各 n=2 干净样本；不是全档执行时间，也未分解单二进制链接耗时 |
| 冷态 check | 根包 80.0s、core 55.5s | workspace 84.2s | workspace 105.9s | 包/依赖/features 不同，跨仓耗时不构成成员数量的因果对照 |

873 的决策为：**暂不细拆 crate，不启动全家族拆包或 trans 成员试点。** 后续两包保留当前成员粒度，以职责、装配和状态边界改善可维护性；不承诺编译提速比例，不以文件变短或成员变多计功。

仍须带着限制使用数据：部分样本 n=2，冷态/restore 有较大波动；隔离 warm-sccache 未测；测试二进制成本未逐个分解；完整档墙钟只引用 880 的近似收据。873 的拆分节省和默认特性调整收益均为估算，不是已测加速。无改动热 check <1s 也不能代替真实编辑成本。

原始证据已保全至 `D:/autostack/plan-artifacts/plan-873/measure-logs/` 与 `measure-pins/`；旧 `.wt/lang-873/.measure` 已随清理消失。Spec 文首仍有 proposal/旧 Plan 链接字样，是交付文档的元数据差额；本文按归档与实际合入收据确认 U873，不在规划轮改 canonical 或重新解释其测量限制。

## 2. 整体顺序与边界

**U873已成立 → 884完整交付 → 887完整交付 → auto-web有限迁移完整交付 → UI/Man结构包完整交付 → 核原三仓有限收口与新增Web交付。**

- 第一语言结构包为884，测试治理为887；新增Web与原第二UI/Man结构包仍未编号。后者不叫874/875，Web在887后冻结正式合同再取号；协议/探针/迁移/消费者在所属完整包内闭合。
- 语言包正式合同冻结前读回 869 清单、872 绑定指纹、873 pin 与届时源码差额；只有依赖图/feature/编译目标等实际影响测量结论时，才定向补测，不重跑整个 873。
- UI/Man包按新排程保守等U(Web)，准备实施盘点/探针/fixture/建树不提前；正式立约时接受Web公开服务出口和887清单，保留原有限结构目标，避免重复搬服务算法。本轮不放行实施。
- 每份 U 都包括完整 AC、独立终审、冻结门、实际修改仓合入、规范/账本、归档和 guard 收尾。通过内部检查或代码落地不能解锁后继。
- 既有结构债与新增语义分开：本轮不引入新状态体系、Feature/Presentation Host、设计包、保状态 HMR，不接管 ACE 或图形实施；行为缺陷仍由既有 owner 按明确授权修复。

## 3. 第一包：语言内部结构收口（必做范围）

owner 为 auto-lang；UI/Man 为固定版本真实消费者。保持现有 crate 和公开路径，不建立 auto-lang-trans，不复制 TypeStore、CompileSession、AutoVM、NativeInterface 或缓存。

| 内部顺序 | 范围 | 整理目标与边界 | 退出证据 |
|---|---|---|---|
| L0 | 冻结当前职责/状态/消费者及七热点差额 | 将 843 原 AC、869 候选、已解决部分、新目标逐项对账；先列顺序敏感面 | 新基线、owner/调用图、旧残项处置表；不能用旧行数冒称当前规模 |
| L1 | 同步/异步 run 入口与桥接 | 实际主要路径是 `crates/auto-lang-core/src/entry.rs` 及其现有 helper；根 `auto-lang/src/lib.rs` 多为再导出。先证明各入口的线程、超时、输入、panic/错误与退出差别，再提取同构桥接 | 各受影响公开入口行为对照与消费者；不强行把不同运行方式统一为一个语义 |
| L2 | `vm/ffi/stdlib.rs` 装配 | 按域组织已有注册与接线，保留 872 记录的名字→ID→覆盖顺序→生效实现。含 inventory/宏生成和源码指纹输入 | 绑定表及优先级保持、六族区分性与错误结果保持、manifest/include 输入归属正确；不删除被遮蔽实现 |
| L3 | `trans/rust/call_emit.rs::call_inner` | 先逐分支审计 early-return、首匹配序、写出时点与错误；证明边界后按调用形状提取同一 RustTrans 上的主题 helper | 形状覆盖、分派/return 对照、真实 a2r 生成及运行 oracle；不造第二套发射算法、不靠大 common 模块藏分支 |
| L4 | `trans/rust.rs` 主管线与 HTTP 旧残项对账 | 在 L3 稳定后整理生成装配/类型/模块输出职责。HTTP 入口在 869 样点已仅剩约 186 生产行，先核当前事实，若仍为装配壳则记录已解决而不再次拆 | 生成文本/导入/错误与返回保持；HTTP 正常/错误/取消/资源释放保持；不因历史超限记录重复计功 |

L1/L2 是先行的有限模块任务；L3→L4 共享转译接线顺序固定。机制可在正式 Plan 的有界决策任务中选择，不能未审计直接机械切开 call_inner。若原行为无法保持，先报告具体差额并修订合同，不悄悄减少验收或留下未授权延期。

**验收重点**：公开接口和唯一状态 owner 保持；真实调用和生成产物保持；867 测试 ID 的迁移有明确映射，零未解释丢失/新增红；新模块职责独立且实际消费者接线，原入口只留装配。源码迁移、净删及 helper 新增分别计量，不要求凭文件搬动达到新的 2,000 行净删目标。

**验证时序**：work/review 做定向入口/注册/分派/生成反例和必要 check；final review 后 merge 统一裸 t 及实际触面的 tv/tt。只有修改语法参考/生成器/Schema 才按规则触发 docs_gen；th 按 HTTP 实际触面判断；taa 只按 AGENTS 的明确路径触发，不因 stdlib 名字或下游消费而自动跑。完整三仓编排与各仓支持 profile 在正式合同中绑定其实际影响范围及可复用输入；tf 批量到期归主检出单实例，不在各阶段重复跑。

## 4. 第二包：UI/Man 内部结构收口（必做范围）

主改面为auto-ui，Man CLI在auto-man；唯一合同已为[auto-ui/014](../../../../auto-ui/docs/plans/014-ui-man-structure-closure.md)，接受884/887及U890后的正式SDK、测试和服务出口。保持下列原有限结构目标，实施时重核CLI服务分派/Vue客户端边界，不重新实现Web绑定/模板；语言只留调度/指针，不把独立仓产品实施归语言仓。007新主题/示例范围不并入014，结构出口U014后再按其原门推进。

默认内部顺序为 **M1 CLI → U1 Vue 生成 → U2 Aura 构建 → U3 renderer**。前两项用较小状态面建立迁移/输出验证方式，运行时逐步进入构建与渲染，GPU/消息循环放在最后；这是一份完整模块合同内的阶段，不为每阶段另取号。

| 阶段 | 现有路径 | 范围与必须保持的内容 |
|---|---|---|
| M1 | `auto-man/crates/auto/src/main.rs` 与既有 cmd_* | 按 CLI 解析、分派、诊断与工具编排整理；已有子命令模块优先复用。保持 flags、help、错误码、stdio、cwd/env、取消/watch 及 Core/full 能力缺失语义 |
| U1 | `auto-ui-compiler/src/ui_gen/vue.rs` | 按 template/script/style、导入策略与库模式分职责；复用已存在的 import_policy/shared 等模块。保生成文本、导入和资源输出、组件覆盖及跨包模式，不重写 Vue 后端或改变主题默认值 |
| U2 | `auto-ui-runtime/src/ui/aura_view_builder.rs` | 按节点族和构建阶段整理，保持借用、惰性子节点、引用解析、挂载身份及构建顺序；测试随 owner 迁移或外移，ID/覆盖完整 |
| U3 | `auto-ui-runtime/src/ui/iced/renderer.rs` | 先冻结状态机、消息顺序、active 判据和资源 owner，再整理消息处理、图标字形、部件宿主、纹理/快照与桌面组合边界。保持 GPU/纹理生命周期、窗口/焦点/输入和帧时序，不借结构整理改调度政策 |

M1 与 U1 没有语义依赖；默认仍按顺序做，未来正式合同如允许内部并行，也不得放宽机器/GPU独占和同写面单 owner 纪律。UI 侧临时重排不能改变会话/组件身份；renderer 的已知债、媒体/终端问题如未在合同内修复，具名保留为债而非当作新绿。

**验收重点**：CLI 正反例与工具真实进程；Vue 字节级 golden 与真实生成工程消费；Aura 构建与状态对照；renderer 必须有定向真实 VM/Iced 交互与 GPU/视觉证据，涉及 Vue 共同语义时做双端核对。不能只靠生成字符串、首帧或截图存在证明全部通过；范围不包括新平台移植或所有应用债清零。

work/review 仅定向反例/模块/必要 check；final review 后各实际修改仓统一其适用冻结门（UI t、生成器触面 tu、接口/工具触面 ti；Man t/tcore/tc/ti 依实际影响），测试 ID/金样输入不能缩减。画廊全量/其他批量档仍按现行里程碑、到期和单实例规则；定向视觉证据是本包验收项，不延后用一次总测替代。AutoGraphics 无改动不追加全量 AG suite。

## 5. 两包之外的候选与错误前置纠正

### A1 默认 UI 特性：单独决策，不自动加为第三个必做包

873 唯一建议跟进的是评估从 `auto-lang` 默认 feature 移出 `ui-iced`。当前根包确实启用图形闭包，但这涉及默认支持合同；约 25s 冷 check 差额及减少重查单元只是收益假设。UI compiler/runtime 当前还是根包的非 optional 依赖，因此移出一个 feature 不自动意味着整个 UI 依赖边消失。

正式候选须先核 CLI/SDK/OS启动脚本/CI/发行和默认消费承诺，给出显式 feature/profile 的替代入口与恢复配方，取得具体签核后才实施；之后定向复测依赖图和 873 的受影响站点。**本轮两结构包默认不改 feature/default/profile**，A1 不作为它们的启动前置，也不以未做 A1 判里程碑未完成。

### 869 历史材料的修正

1. S1 把 U871 写成 call_inner 的全局前置，需要按实际写面纠正：871 退役的是 `experimental/ac-core/src/native.rs` 的 HIR→CLIF 管线，S1 是 `crates/auto-lang-core/src/trans/rust/**` 的 Auto→Rust 生成；两条路径职责不同。未来只对真实共享接口/符号留交接门，不要求无关 U871，也不因 871 退役 native 而取消 A2R 能力。
2. S3 的“根 lib.rs 九处 glue”按当前路径重核，主体在 core/entry.rs，不为凑旧清单改已为再导出的根。
3. S2 的 HTTP 入口已壳化部分、S4–S7 历史行数/测试数在正式冻结时重核；不把历史样点当新测量。
4. stdlib 触面不自动触发 taa；依 AGENTS 明确 AAVM 路径映射，原候选中的泛化测试要求不升级为当前门禁。

### 细拆 crate 的重启条件

沿 873 保留 T-A～T-D：代表温态编辑→定向测试可运行 P50 >60s 持续两周；文件级整理无法容纳的 feature/依赖冲突；补测证实单二进制重链 >40s；或同依赖闭包试点证实细拆有收益。触发表示重新评估/试点立约，不是直接授权拆包。若未来需试点，trans 仅为候选，命名、依赖边界与收益仍须实证。

## 6. 本轮核查与后续动作

本轮读回 873 全量结论及终审/merge/cleanup收据、869 七热点材料、现行 language-sdk 结构规约、872 绑定要求和源码职责；另核 871 实际退役路径及三仓入口现状。未重跑测量，不改归档合同、canonical Spec、ledger 或默认分支代码。

规划输入 SHA-256（本轮读取时）：

| 输入 | SHA-256 |
|---|---|
| `docs/specs/repository-boundaries/build-cost-baseline.md` | `061464e7ee8a4f223fe680dd5f6a1c9c8e20e7045135e2646e63c33a37867dd6` |
| `docs/plans/archive/873-build-cost-and-member-split-decision.md` | `213834a84b58551c54b50ca0830262755f44e5baaf55983e451dada4a0dd98c9` |
| `docs/plans/attachments/869/structural-packages.md` | `fd6dd4dada390a8eae163c78f0e6aa4fdc270a19a986a192610a5feff74f33a4` |

本轮限定规划文档的 `git diff --check`、本地 Markdown 目标存在性及新增/未跟踪文档空白与围栏检查通过；仅文档校验，不是两个未来包的实现验收。

884按原合同继续收口；887优先在U884后执行；Web现阶段只完成Design50，887结束后正式立项，再按相关885修复/接口释放推进。UI/Man包保留本范围，排在U(Web)后接受新基线。原三仓有限收口与新增Web交付分列，当前选定顺序可能后移总体到达时间；到点再重审Design38/47，不追加未来Web扩展、数据/runtime新仓、全部测试改写、无限减法、ACE全线或80万达标前置。
