# v0.3 → v0.5 滚动历史核查（2026-09-30）

## 范围与方法

仓库 auto-lang，基线 `v0.3=750cf1c09`（2026-04-12），终点 `14e444f02`（2026-09-30）。无 v0.5 标签。读取完整 `git log v0.3..14e444f02`，按月份和模块归类，结合标签分段、累计 diff、v0.3/v0.4 发布说明、归档 Plan、module Specs 及关键实现路径核查。提交标题和勾选项是线索；规范与实现不一致时优先采用具体实现、验证范围及后续勘误。此报告按重要能力聚合全部历史，非逐提交复述，也不是重新运行所有旧验收。

| 区间 | 提交数 | 标签/日期 |
|---|---:|---|
| v0.3 → v0.4 | 1,286 | da71b119f / 2026-06-24 |
| v0.4 → v0.4.1 | 183 | 8ea79cbb3 / 2026-07-02 |
| v0.4.1 → v0.4.2 | 4,247 | 278a17e3d / 2026-09-07 |
| v0.4.2 → 核查终点 | 2,317 | 14e444f02 / 2026-09-30 |
| 合计 | **8,033** | 包含功能、修复、文档及 merge |

按提交日期月份计：4 月 390、5 月 492、6 月 534、7 月 713、8 月 2,293、9 月 3,613。累计 diff 为 9,409 文件、+1,209,191/-118,224 行（默认 rename 检测因文件量跳过穷举）；包含文档、生成数据、语料、资产及移动，不可称为新增产品代码行数。旧 v0.4 文档的 1,285 是当时快照；按标签对象现算为 1,286，不改写旧版档案。

## 历史阶段与证据锚点

### 4–6 月：语言基础之上的真实后端

v0.3 已有泛型、所有权、闭包、AIE、Task/Msg、Vue/Jetpack/ArkTS 原型、基础 LSP 和结构化 Shell；它们不是 v0.5 首次出现。

- 4 月 UI 渲染模块化、headless 与多后端迁移（0284729ca、dcecd5bf6），r2a 逆转译从基础语法推进泛型/trait/异步（88f959a94、417359cf8、50c3e7256），VM 文件语料框架（48ce78a52）。
- 5 月 Playground 调试回放/源码关联（fb955d0d3、86243a9dc），在线画廊（ff1b57eea）；自举从 lexer（23da6c1d2）到 parser（83d39cac9、e80db2a08）。
- 6 月 Notes 从 UI 示例成为 CRUD 全栈应用：子组件、循环和状态同步、生命周期、API 导入、生成真实 HTTP 客户端（fac8bc064、76fcd3a69、68823ccb9、8c8efc565、d601ccb3a）。同期 MCP 截图与布局/VTree 接入（1cc079d5b、f57024ae6）。
- v0.4 的 Plans 310/312/317/321/328 收敛逃逸分析、HTTP/SSE、多模块链接、Actor/生成器调度、Axum/Tauri 输出；语言由简单语料推进真实应用后端。见 [v0.4 说明](../releases/v0.4.md)。

### 7–8 月：运行时、互操作、桌面与自举深化

- 7 月方言接口和统一 AST（f6a0ca06b、cacbc5380、685b963ba），store 状态抽取（93413e29e、2f1461673）；多模块全局初始化（a35327408）、真正非阻塞 SSE（36e78d565）、TLS/上传/下载/WebSocket（4cca6fbea、aa6c589d6、a2dd896ab、d326fae99、afc5f74f0）。
- 8 月 NanoValue 单槽化（d800c1f0c、ee4871ecb、3e4151e96），避免将运行时值表示变化误归为新语法。Plan 510 随后清偿池引用计数并建立长期 churn 防线。
- Rust 互操作 Plans 400/415/430、AURA 单一 schema Plan 435、虚拟桌面与进程协议 Plans 386/500/507/508 是该阶段主线。同期 AutoAI 相关 crate 迁回独立仓（91443c102）；产品和语言框架开始分仓。
- 自举 Plans 429–434 → 447/495/511/514/517：从简化实现推进方法、模块、容器、闭包、AA2R 和共享语料；不能只以最终 diff 误认为一次性完成。详见 [AAVM 规格](../specs/aavm/project.md)。

### 9 月：自举收官与桌面工程化

- Plans 532/572：GOAL-017 双目标、塔顶静态语义差分、AA2R 自译及两代原生二进制对拍闭环。是借道 Rust 编译器的曲线自举，宿主 Rust 仍为参考；不代表所有语法已被自举实现覆盖。
- Plans 550/555/560/567/569/598/602：null 门控、`.as` lowering、Python 语法糖/异常/上下文管理与 PyTorch 子类回调（06553c1d6）。Specs 中部分早期“未实现”标题被后文勘误，发布内容以落地细节为准。
- Plans 591/594/596：真实 crates.io 库及非白名单 pack 面对拍、字段布局探针（5c5789458）、泛型具体实例（27cf4da20）；SQLite/Redis 和真实 memmap 语料补齐（85d8949f0、b7ef75066、0326241b5）。这些证据支持具体能力，无法支持“任意 Rust 库/90% 全生态兼容”。
- 桌面应用资产迁至 auto-os（Plan 590），Blueprint 更名和 stylekit 配方（f961386de、415484a65）；Select Anything（Plan 646）提供结构化界面采集。语言示例轨与 OS 应用轨后来分开演化（Plan 666），不能把网页画廊自动等同系统应用当前成品。
- 真媒体服务及视频整合（2763657fb、3f8dbd325），API proxy 的 stream/native-ns/binary 消费（b55e9cba4、c733bb8c7）；图片服务与导航（231cca2fd、5778b1b55）。
- Plan 673 rope/COW/delta 编辑器；Plan 703 文本/目录 diff（6d2c9fd98、4adc67514、9e2faeb00）。rope 并不消除现存全文物化及每键 O(n) 路径，大文件性能仍留债。
- RqProjector 统一与旧 AppProjector 退役（98b778f44、0c60a9396）；headless remote/display-list（194064a8c、78977ca24、79f69f952）与输入法回传（1d3437293、3d1b54f0a）。9 月 24 日 desktop endpoint 接入（d7ef23b2d），这是跨进程渲染，互联网远程桌面仍属路线图。
- 持久化键级合并及写入安全（bc6fece01、c67468d9d）；UI 生成器指纹缓存（63a89d8dd）；HTTP Axum/Hyper 传输（e48d4e366）；UI handler park/resume（b1a440420）。GPUI 死后端移除（3ac279750），不能继续作为当前可用渲染器宣传。
- 9 月末仍在修桌面验收问题（16387c84a、14e444f02）及 a2r 语料（33a5d56c3），因此不给“全部应用完备/全部双端全等”的发行保证。

## 三类结论

| 类别 | 实质变化 | 发布表达 |
|---|---|---|
| 功能 | 真实 HTTP/流式后端、Rust/Python 互操作、跨端 UI、桌面及媒体/应用集 | 用具体能力与示例替代笼统兼容率 |
| 架构 | NanoValue/池、AURA schema、方言/AST、桌面协议、rope/delta、自举闭环 | 解释单源如何走不同路径及其边界 |
| 开发者体验 | DevTools/MCP/框选、Playground、缓存、Parity、Plan/Spec 及测试分层 | 提供可检查事实，不宣称每次历史提交都经同一门禁 |

## release notes 与宣传页处理

1. 重写 v0.5 说明，补足历史范围、功能/架构/开发者体验及已知边界；纠正自举与 v0.6 路线的过时描述。
2. 中英 v05 页同步：新增 Rust 真实库桥、Python 脚本/PyTorch、MCP 结构化采集、编辑器内核与后端/缓存进展；生态百分比改为具体验证能力。
3. 修正“旗舰全部 Auto 写成”：AutoShell 核心与 AutoMusk 后端为 Rust；AutoEdit 尚无独立落地页，去掉“每个都有”。媒体、桌面与自举保留展示，减少全面可用/全面一致承诺。
4. 旧 5,711 次提交及行数是 2026-09-07 宣传快照，明确标注日期和统计来源；不把全仓行数说成五个月新增量。1000 亿 Token 为用户提供数据，不作为 git 可验证统计。

## 事实源及限制

- [全局模块地图](../specs/overview.md)、[UI](../specs/auto-lang/ui/overview.md)、[VM](../specs/auto-lang/vm/overview.md)、[转译](../specs/auto-lang/trans/overview.md)、[runtime](../specs/auto-lang/runtime/overview.md)、[Parity](../specs/parity/project.md)。
- [Rust 互操作设计](../design/28-rust-interop-architecture.md)、[宣传设计与口径](../design/documents/v05-release-promo.md)、[已知债](../plans/KNOWN-DEBT-AND-RISKS.md)。
- 没有重跑全部历史验收，也没有审计全部兄弟仓历史。宣传中的旧样本数、应用数、截图及代码量属于已有素材快照；本轮不增加未经实测的最新数量或性能承诺。

## 本轮验证与素材修复

- 中英文新增 5 类 FeatureCard 同构，生态百分比、全 Auto 实现及历史门禁过度承诺已检查移除；保留应用成熟度与脚本/桥接边界。
- 宣传页已有的两个桌面 JPG 仅在主工作区未跟踪：检查原图后原样纳入版本管理。八张 app-* 截图引用不存在，改为已入库 gallery-home.png 概览，避免空图与不可复现构建。
- 文档相对链接和 v05 静态资产检查通过；`git diff --check` 通过。
- 在独立 worktree 安装网站及 auto-playground-vue 的声明依赖后，`website/npm run build` 通过（59.22s）。存在已有 Auto 高亮降级及 chunk 体积警告。未执行 Rust 测试（纯文档/素材变更，遵守 Category A 门禁）。
- 合入前按四项用户目标另行逐项复核：历史分段和证据、v0.5 覆盖、双语宣传修正、文件/验证清单齐备；不将文档构建通过误报为产品全部功能验收通过。
